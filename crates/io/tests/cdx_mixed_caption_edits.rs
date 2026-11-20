//! Source-identity, paint-order and retained-byte witnesses for mixed Page Text edits.
use std::sync::Arc;

use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{AtomId, PageId, PageSize, TextId};
use serde_json::{Value, json};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const BEFORE: &[u8] = include_bytes!("fixtures/cdx-mixed-caption-edit/before.cdx");
const EXTENDED: &[u8] = include_bytes!("fixtures/cdx-mixed-caption-edit/extended_before.cdx");
const PREFIX: &[u8] = include_bytes!("fixtures/cdx-mixed-caption-edit/after_prefix.cdx");
const BOTH: &[u8] = include_bytes!("fixtures/cdx-mixed-caption-edit/after_both.cdx");
const TAIL: &[u8] = include_bytes!("fixtures/cdx-mixed-caption-edit/after_tail.cdx");

fn model(prefix: &str, tail: &str) -> Result<Value> {
    let cases: Value = serde_json::from_str(include_str!("fixtures/cdx-label-preview/cases.json"))?;
    let mut expected = cases["renderable"]["boxed"]["model"].clone();
    let run = |text: &str, size: f64, rgb: [f64; 3]| json!({"text":text,"style":{"font_family":"Roboto","size_pt":size,"face":0,"rgb":rgb}});
    expected["pages"][0]["texts"] = json!([
        {"id":1,"position":{"x_pt":80.25,"y_pt":100.75},"runs":[run("H",12.0,[0.0;3])]},
        {"id":19,"position":{"x_pt":158.947265625,"y_pt":131.65625},
         "runs":[run(prefix,24.0,[0.0,0.0,1.0]),run(tail,12.0,[0.0;3])]}
    ]);
    Ok(expected)
}

#[test]
fn mixed_run_edit_selects_the_page_owner_and_preserves_both_aliased_node_labels() -> Result {
    let page = PageId::new(1).ok_or("page ID")?;
    let text = TextId::new(19).ok_or("live caption ID")?;
    let atom = AtomId::new(1).ok_or("first atom ID")?;
    let original = StoredDocument::from_cdx(Arc::from(BEFORE), PageSize::A4)?;
    let mut current = original.clone();
    assert_eq!(serde_json::to_value(current.model())?, model("H ", "λΩé")?);
    for (index, prefix, tail, bytes) in [
        (0, "H H ", "λΩé", PREFIX),
        (1, "H H ", "λéπΩ", BOTH),
        (0, "H ", "λéπΩ", TAIL),
        (1, "H ", "λΩé", BEFORE),
    ] {
        let body = if index == 0 { prefix } else { tail };
        current = current.with_cdx_caption_run(page, text, index, body.to_owned())?;
        assert!(current.can_edit_cdx_caption_run());
        assert!(current.has_cdx_mixed_preview() && current.has_display_projection());
        assert!(!current.has_cdx_caption_preview() && !current.can_create_caption());
        assert!(!current.can_edit_cdx_atom_label(atom) && !current.is_fully_mapped());
        assert!(current.to_cdxml().is_err());
        assert_eq!(current.export_cdx()?, bytes);
        assert_eq!(serde_json::to_value(current.model())?, model(prefix, tail)?);
        let reopened = StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)?;
        assert_eq!(reopened.model(), current.model());
        assert_eq!(original.export_cdx()?, BEFORE);
        assert_eq!(
            current.source_info().ok_or("original CDX")?.modified,
            bytes != BEFORE
        );
    }
    assert!(Arc::ptr_eq(
        &original.model_snapshot(),
        &current.model_snapshot()
    ));
    assert_eq!(
        current.export_cdx()?.as_ptr(),
        original.export_cdx()?.as_ptr()
    );
    Ok(())
}

#[test]
fn mixed_noops_failures_and_real_inverse_preserve_exact_extended_source_provenance() -> Result {
    let page = PageId::new(1).ok_or("page ID")?;
    let text = TextId::new(19).ok_or("live caption ID")?;
    let original = StoredDocument::from_cdx(Arc::from(EXTENDED), PageSize::A4)?;
    let unchanged = original.with_cdx_caption_run(page, text, 0, "H ".into())?;
    assert!(Arc::ptr_eq(
        &original.model_snapshot(),
        &unchanged.model_snapshot()
    ));
    assert_eq!(
        unchanged.export_cdx()?.as_ptr(),
        original.export_cdx()?.as_ptr()
    );

    let raw_id = TextId::new(73).ok_or("raw ID")?;
    for (id, index, body) in [
        (raw_id, 0, "NH2".to_owned()),
        (text, 2, "bad run".into()),
        (text, 0, "λ ".into()),
        (text, 1, String::new()),
        (text, 1, "W".repeat(200)),
    ] {
        assert!(
            original
                .with_cdx_caption_run(page, id, index, body)
                .is_err()
        );
        assert_eq!(original.export_cdx()?, EXTENDED);
        assert!(!original.source_info().ok_or("original CDX")?.modified);
    }
    let changed = original.with_cdx_caption_run(page, text, 0, "H H ".into())?;
    assert_eq!(changed.export_cdx()?, PREFIX);
    let inverse = changed.with_cdx_caption_run(page, text, 0, "H ".into())?;
    assert_eq!(inverse.model(), original.model());
    assert_eq!(inverse.export_cdx()?, BEFORE);
    let source = inverse.source_info().ok_or("original extended CDX")?;
    assert_eq!(
        (source.byte_length, source.modified),
        (EXTENDED.len(), true)
    );

    let mut detached = changed.clone();
    *detached.model_mut() = original.model().clone();
    assert!(!detached.can_edit_cdx_caption_run());
    assert!(matches!(
        detached.with_cdx_caption_run(page, text, 0, "H ".into()),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(detached.export_cdx().is_err());
    assert_eq!(changed.export_cdx()?, PREFIX);
    Ok(())
}
