//! One owned chemical label edit beside unchanged, independently encoded Page captions.
use std::sync::Arc;

use bondline_io::StoredDocument;
use bondline_model::{AtomId, Element, FragmentId, PageId, PageSize, TextId};
use serde_json::Value;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const MIXED: &[u8] = include_bytes!("fixtures/cdx-chemical-caption-create/mixed_first.cdx");
const OH: &[u8] = include_bytes!("fixtures/cdx-mixed-label-edit/mixed_first_oh.cdx");
const BOTH: &[u8] = include_bytes!("fixtures/cdx-mixed-label-edit/mixed_first_both.cdx");

fn open(raw: &[u8]) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdx(Arc::from(raw), PageSize::A4)?)
}

fn label(
    source: &StoredDocument,
    fragment: u32,
    atom: u32,
    element: u8,
    h: u16,
) -> Result<StoredDocument> {
    Ok(source.with_cdx_atom_label(
        PageId::new(1).ok_or("page")?,
        FragmentId::new(fragment).ok_or("fragment")?,
        AtomId::new(atom).ok_or("atom")?,
        Element::new(element).ok_or("element")?,
        h,
    )?)
}

#[test]
fn both_aliased_node_labels_can_change_without_touching_the_aliased_page_text() -> Result {
    let original = open(MIXED)?;
    let first = label(&original, 1, 1, 8, 1)?;
    let both = label(&first, 2, 2, 35, 0)?;
    assert_eq!(original.export_cdx()?, MIXED);
    assert_eq!(first.export_cdx()?, OH);
    assert_eq!(both.export_cdx()?, BOTH);
    let oracle: Value = serde_json::from_str(include_str!("fixtures/cdx-label-edit/cases.json"))?;
    let mut expected = serde_json::to_value(original.model())?;
    for (name, index) in [("boxed_nh2_to_oh", 0), ("boxed_cl_to_br", 1)] {
        expected["pages"][0]["fragments"][index] =
            oracle["cases"][name]["after_model"]["pages"][0]["fragments"][index].clone();
    }
    assert_eq!(serde_json::to_value(both.model())?, expected);
    assert_eq!(open(BOTH)?.model(), both.model());
    assert!(both.has_cdx_mixed_preview() && both.can_edit_cdx_caption_run());
    assert!(!both.is_fully_mapped() && !both.has_cdx_atom_label_preview());
    assert!(both.to_cdxml().is_err());
    assert!(label(&both, 2, 1, 8, 1).is_err());
    let unchanged = label(&both, 2, 2, 35, 0)?;
    assert!(Arc::ptr_eq(
        &both.model_snapshot(),
        &unchanged.model_snapshot()
    ));
    assert_eq!(
        both.export_cdx()?.as_ptr(),
        unchanged.export_cdx()?.as_ptr()
    );
    let mut detached = both.clone();
    *detached.model_mut() = original.model().clone();
    assert!(!detached.can_edit_cdx_atom_label(AtomId::new(1).ok_or("atom")?));
    assert!(label(&detached, 1, 1, 8, 1).is_err() && detached.export_cdx().is_err());
    Ok(())
}

#[test]
fn paper_bounds_are_checked_again_after_the_literal_label_changes() -> Result {
    let raw = include_bytes!("fixtures/cdx-mixed-label-edit/edge_cl_before.cdx");
    let bad = include_bytes!("fixtures/cdx-mixed-label-edit/edge_mg_off_paper.cdx");
    let input = open(raw)?;
    assert!(input.can_edit_cdx_atom_label(AtomId::new(2).ok_or("atom")?));
    assert!(open(bad).is_err());
    assert!(label(&input, 2, 2, 12, 0).is_err());
    assert_eq!(input.export_cdx()?, raw);
    Ok(())
}

#[test]
fn mixed_label_edits_keep_source_order_connected_bond_clearance_and_maximum_caption_depth() -> Result
{
    let page_first = include_bytes!("fixtures/cdx-mixed-caption/overlap_before.cdx");
    let connected = include_bytes!("fixtures/cdx-mixed-caption/water_after.cdx");
    let maximum = include_bytes!("fixtures/cdx-mixed-label-edit/depth_max_before.cdx");
    let oracle: Value = serde_json::from_str(include_str!("fixtures/cdx-label-edit/cases.json"))?;
    for (before, after, witness, fragment, atom, element, h) in [
        (
            page_first.as_slice(),
            include_bytes!("fixtures/cdx-mixed-label-edit/overlap_before_oh.cdx").as_slice(),
            "boxed_nh2_to_oh",
            1,
            1,
            8,
            1,
        ),
        (
            connected.as_slice(),
            include_bytes!("fixtures/cdx-mixed-label-edit/water_after_nh.cdx").as_slice(),
            "water_to_nh",
            1,
            2,
            7,
            1,
        ),
        (
            maximum.as_slice(),
            include_bytes!("fixtures/cdx-mixed-label-edit/depth_max_oh.cdx").as_slice(),
            "boxed_nh2_to_oh",
            1,
            1,
            8,
            1,
        ),
    ] {
        let source = open(before)?;
        let changed = label(&source, fragment, atom, element, h)?;
        assert_eq!(changed.export_cdx()?, after);
        assert!(changed.has_cdx_mixed_preview());
        assert_eq!(open(after)?.model(), changed.model());
        let mut expected = serde_json::to_value(source.model())?;
        expected["pages"][0]["fragments"] =
            oracle["cases"][witness]["after_model"]["pages"][0]["fragments"].clone();
        assert_eq!(serde_json::to_value(changed.model())?, expected);
        assert_eq!(source.export_cdx()?, before);
    }
    Ok(())
}

#[test]
fn changing_an_aliased_node_preserves_the_extended_page_text_until_its_own_edit() -> Result {
    let extended = include_bytes!("fixtures/cdx-mixed-caption-edit/extended_before.cdx");
    let saved = include_bytes!("fixtures/cdx-mixed-label-edit/extended_br.cdx");
    let prefix = include_bytes!("fixtures/cdx-mixed-caption-edit/after_prefix.cdx");
    let both = include_bytes!("fixtures/cdx-mixed-label-edit/prefix_br.cdx");
    let original = open(extended)?;
    let label_first = label(&original, 2, 2, 35, 0)?;
    assert_eq!(label_first.export_cdx()?, saved);
    let page = PageId::new(1).ok_or("page")?;
    let id = TextId::new(19).ok_or("caption")?;
    let result = label_first.with_cdx_caption_run(page, id, 0, "H H ".into())?;
    assert_eq!(result.export_cdx()?, both);
    assert_eq!(result.model(), open(both)?.model());
    let caption_first = original.with_cdx_caption_run(page, id, 0, "H H ".into())?;
    assert_eq!(caption_first.export_cdx()?, prefix);
    let reversed = label(&caption_first, 2, 2, 35, 0)?;
    assert_eq!(reversed.export_cdx()?, both);
    assert_eq!(reversed.model(), result.model());
    assert_eq!(original.export_cdx()?, extended);
    Ok(())
}
