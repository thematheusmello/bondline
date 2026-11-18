//! Independent source, offset and model acceptance for editing existing CDX captions.
use bondline_io::{
    DocumentError, SourceInfo, StoredDocument,
    display::{DisplayError, drawing_page_scene},
    files::{read_cdx_file, save_cdx_file},
};
use bondline_model::{PageId, PageSize, TextId};
use bondline_render::{Cancellation, cpu_scene};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CASES: &str = include_str!("fixtures/cdx-caption-edit/cases.json");
const RUNS: &str = include_str!("fixtures/cdx-caption/runs/cases.json");

#[derive(Clone, Deserialize)]
struct Params {
    page_id: PageId,
    text_id: TextId,
    run_index: usize,
    text: String,
}

impl Params {
    fn apply(
        &self,
        document: &StoredDocument,
    ) -> std::result::Result<StoredDocument, DocumentError> {
        document.with_cdx_caption_run(
            self.page_id,
            self.text_id,
            self.run_index,
            self.text.clone(),
        )
    }
}

#[derive(Deserialize)]
struct Case {
    before: String,
    after: String,
    params: Params,
    before_model: Value,
    after_model: Value,
    source: Value,
}

#[derive(Deserialize)]
struct Manifest {
    cases: BTreeMap<String, Case>,
}

fn cases() -> Result<BTreeMap<String, Case>> {
    Ok(serde_json::from_str::<Manifest>(CASES)?.cases)
}

fn bytes(name: &str) -> Result<Arc<[u8]>> {
    Ok(Arc::from(fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cdx-caption-edit")
            .join(name),
    )?))
}

fn read(raw: Arc<[u8]>) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdx(raw, PageSize::A4)?)
}

fn source(raw: &[u8], modified: bool) -> Option<SourceInfo> {
    Some(SourceInfo {
        format: "cdx",
        byte_length: raw.len(),
        modified,
    })
}

fn starts(raw: &[u8], property: usize) -> Result<Vec<u16>> {
    let document = bondline_cdx::Document::parse(raw)?;
    let record = document
        .records()
        .get(property)
        .ok_or("missing original Text property")?;
    Ok(document
        .string(record)?
        .runs()
        .map(|run| run.start())
        .collect())
}

fn original_text(case: &Case) -> Result<String> {
    Ok(case.before_model["pages"][0]["texts"]
        .as_array()
        .ok_or("missing independent captions")?
        .iter()
        .find(|text| text["id"] == case.params.text_id.get())
        .ok_or("missing independent target caption")?["runs"]
        .get(case.params.run_index)
        .and_then(|run| run["text"].as_str())
        .ok_or("missing independent target run")?
        .to_owned())
}

#[test]
fn caption_edits_match_independent_bytes_models_offsets_and_unicode_reopen() -> Result {
    let cases = cases()?;
    let directory = files::FileTestDirectory::new()?;
    let cancel = Cancellation::default();
    assert_eq!(cases.len(), 4);
    for name in ["single_grow", "multi_shift", "tail_unicode", "multi_shrink"] {
        let case = cases.get(name).ok_or("missing caption edit witness")?;
        let before = bytes(&case.before)?;
        let after = bytes(&case.after)?;
        let original = read(Arc::clone(&before))?;
        let property = serde_json::from_value(case.source["text_property_record"].clone())?;
        assert_eq!(
            starts(&before, property)?,
            serde_json::from_value::<Vec<u16>>(case.source["before_starts"].clone())?,
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(original.model())?,
            case.before_model,
            "{name}"
        );

        let edited = case.params.apply(&original)?;
        assert_eq!(
            serde_json::to_value(edited.model())?,
            case.after_model,
            "{name}"
        );
        assert_eq!(edited.export_cdx()?, after.as_ref(), "{name}");
        assert_eq!(
            starts(edited.export_cdx()?, property)?,
            serde_json::from_value::<Vec<u16>>(case.source["after_starts"].clone())?,
            "{name}"
        );
        assert_eq!(edited.source_info(), source(&before, true), "{name}");
        assert!(edited.has_cdx_caption_preview(), "{name}");
        assert!(!edited.has_caption_projection(), "{name}");
        assert!(!edited.is_fully_mapped(), "{name}");
        assert!(
            matches!(
                edited.to_cdxml(),
                Err(DocumentError::CrossFormatSource {
                    from: "cdx",
                    to: "cdxml"
                })
            ),
            "{name}"
        );
        let graph = drawing_page_scene(1024, 1024, &edited, case.params.page_id, &cancel)?;
        assert_eq!(graph.hit_test([512.0, 512.0], 100_000.0)?, None, "{name}");
        assert!(
            cpu_scene(graph.scene())?
                .rgba
                .chunks_exact(4)
                .any(|pixel| pixel != [255; 4]),
            "{name}"
        );

        let path = directory.path(&format!("{name}-保存-λ.cdx"));
        let receipt = save_cdx_file(&path, &edited, false, &cancel)?;
        assert_eq!(receipt.byte_length, after.len(), "{name}");
        assert_eq!(fs::read(&path)?, after.as_ref(), "{name}");
        let reopened = read_cdx_file(&path, &cancel)?.document;
        assert_eq!(reopened.model(), edited.model(), "{name}");
        assert_eq!(reopened.export_cdx()?, after.as_ref(), "{name}");
        assert_eq!(reopened.source_info(), source(&after, false), "{name}");
        assert!(reopened.has_cdx_caption_preview(), "{name}");
        assert_eq!(original.export_cdx()?.as_ptr(), before.as_ptr(), "{name}");
        assert_eq!(original.export_cdx()?, before.as_ref(), "{name}");
        assert_eq!(
            serde_json::to_value(original.model())?,
            case.before_model,
            "{name}"
        );
    }
    Ok(())
}

#[test]
fn exact_caption_noops_preserve_original_and_edited_source_and_model_arcs() -> Result {
    for (name, case) in cases()? {
        let before = bytes(&case.before)?;
        let original = read(Arc::clone(&before))?;
        let same_params = Params {
            text: original_text(&case)?,
            ..case.params.clone()
        };
        let same = same_params.apply(&original)?;
        assert_eq!(same, original, "{name}");
        assert!(
            Arc::ptr_eq(&same.model_snapshot(), &original.model_snapshot()),
            "{name}"
        );
        assert_eq!(same.export_cdx()?.as_ptr(), before.as_ptr(), "{name}");
        assert_eq!(same.source_info(), source(&before, false), "{name}");

        let edited = case.params.apply(&original)?;
        let same = case.params.apply(&edited)?;
        assert_eq!(same, edited, "{name}");
        assert!(
            Arc::ptr_eq(&same.model_snapshot(), &edited.model_snapshot()),
            "{name}"
        );
        assert_eq!(
            same.export_cdx()?.as_ptr(),
            edited.export_cdx()?.as_ptr(),
            "{name}"
        );
        assert_eq!(same.source_info(), source(&before, true), "{name}");
    }
    Ok(())
}

#[test]
fn detached_geometry_and_old_models_cannot_be_recertified_through_caption_edits() -> Result {
    let cases = cases()?;
    let case = cases
        .get("multi_shift")
        .ok_or("missing ordered caption witness")?;
    let original = read(bytes(&case.before)?)?;
    let edited = case.params.apply(&original)?;
    let mut geometry = edited.clone();
    let page = geometry.model().pages().first().ok_or("missing Page")?;
    let size = page.size();
    geometry.model_mut().set_page_size(
        case.params.page_id,
        PageSize::new(size.width_pt() + 1.0, size.height_pt())?,
    )?;
    let mut old_model = edited.clone();
    *old_model.model_mut() = original.model().clone();
    let old_params = Params {
        text: original_text(case)?,
        ..case.params.clone()
    };
    for (name, detached, params) in [
        ("geometry", geometry, &case.params),
        ("old model beside edited bytes", old_model, &old_params),
    ] {
        let before = detached.clone();
        let snapshot = detached.model_snapshot();
        assert!(!detached.has_cdx_caption_preview(), "{name}");
        let error = params
            .apply(&detached)
            .err()
            .ok_or("detached caption edited")?;
        assert_eq!(error.code(), "unsupported_content", "{name}");
        assert_eq!(detached, before, "{name}");
        assert!(Arc::ptr_eq(&snapshot, &detached.model_snapshot()), "{name}");
        assert!(
            matches!(
                detached.export_cdx(),
                Err(DocumentError::UnsupportedCdxEdit)
            ),
            "{name}"
        );
        assert!(
            matches!(
                drawing_page_scene(
                    1024,
                    1024,
                    &detached,
                    case.params.page_id,
                    &Cancellation::default()
                ),
                Err(DisplayError::IncompleteSource)
            ),
            "{name}"
        );
    }
    assert!(edited.has_cdx_caption_preview());
    assert_eq!(edited.export_cdx()?, bytes(&case.after)?.as_ref());
    Ok(())
}

#[test]
fn unsupported_caption_text_keeps_the_complete_preview_and_save_source() -> Result {
    let cases = cases()?;
    let case = cases
        .get("multi_shift")
        .ok_or("missing run shift witness")?;
    let original = read(bytes(&case.before)?)?;
    for (name, run_index, text) in [
        ("empty", 0, String::new()),
        ("non-ASCII before another run", 0, "λ".into()),
        ("nonzero pair across the boundary", 2, "T".into()),
        ("unknown glyph", 2, "\u{1fae0}".into()),
        (
            "paper extent below the glyph count limit",
            0,
            "W".repeat(80),
        ),
        // This exceeds both the u16 start domain and the existing logical work bound.
        ("offset and logical work limits", 0, " ".repeat(65_536)),
    ] {
        let params = Params {
            run_index,
            text,
            ..case.params.clone()
        };
        assert_refused(name, &original, &params)?;
    }
    let ligature = read(Arc::from(
        include_bytes!("fixtures/cdx-caption/two_runs.cdx").as_slice(),
    ))?;
    assert_refused(
        "cross-run fl ligature",
        &ligature,
        &Params {
            page_id: PageId::new(1).ok_or("invalid Page")?,
            text_id: TextId::new(1).ok_or("invalid Text")?,
            run_index: 0,
            text: "f".into(),
        },
    )?;
    Ok(())
}

fn assert_refused(name: &str, document: &StoredDocument, params: &Params) -> Result {
    let before = document.clone();
    let snapshot = document.model_snapshot();
    let raw = document.export_cdx()?;
    let error = params
        .apply(document)
        .err()
        .ok_or_else(|| format!("{name} was accepted"))?;
    assert_eq!(error.code(), "unsupported_content", "{name}: {error}");
    assert_eq!(document, &before, "{name}");
    assert!(Arc::ptr_eq(&snapshot, &document.model_snapshot()), "{name}");
    assert_eq!(document.export_cdx()?.as_ptr(), raw.as_ptr(), "{name}");
    assert!(document.has_cdx_caption_preview(), "{name}");
    let graph = drawing_page_scene(
        1024,
        1024,
        document,
        params.page_id,
        &Cancellation::default(),
    )?;
    cpu_scene(graph.scene())?;
    Ok(())
}

#[test]
fn ambiguous_imported_encoding_and_style_keep_the_exact_legacy_failures() -> Result {
    let cases: Value = serde_json::from_str(RUNS)?;
    for name in ["ambiguous_utf8", "late_face", "ligature_fi"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cdx-caption/runs")
            .join(format!("{name}.cdx"));
        let bytes = fs::read(path)?;
        assert_eq!(
            bondline_cdx::Document::parse(&bytes)?.to_bytes(),
            bytes,
            "{name}"
        );
        let error = StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)
            .err()
            .ok_or("unsupported caption source opened")?;
        assert_eq!(
            error.code(),
            cases["cases"][name]["legacy"]["code"],
            "{name}"
        );
        assert_eq!(
            error.to_string(),
            cases["cases"][name]["legacy"]["detail"],
            "{name}"
        );
    }
    Ok(())
}
