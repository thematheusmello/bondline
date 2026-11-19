//! Independent wire and live-identity acceptance for creating CDX Page captions.
use bondline_io::{
    DocumentError, SourceInfo, StoredDocument,
    files::{read_cdx_file, save_cdx_file},
};
use bondline_model::{PageId, PageSize, TextId};
use bondline_render::Cancellation;
use serde_json::{Value, json};
use std::{fs, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const BASELINE: &[u8] = include_bytes!("fixtures/cdx-caption-edit/single_before.cdx");
const LEGACY_EDIT: &[u8] = include_bytes!("fixtures/cdx-caption-edit/single_after.cdx");
const FIRST: &[u8] = include_bytes!("fixtures/cdx-caption-create/first.cdx");
const SPARSE: &[u8] = include_bytes!("fixtures/cdx-caption-create/after_undo.cdx");
const IMPLICIT: &[u8] = include_bytes!("fixtures/cdx-caption-create/implicit_persistence.cdx");
const NEW_TEXT: &str = "Phase β & q";
const LEGACY_EDIT_TEXT: &str = "Bondline λ notes";
const SPARSE_EDIT_TEXT: &str = "Phase β and q";

fn read(raw: &[u8]) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdx(Arc::from(raw), PageSize::A4)?)
}

fn page_id() -> Result<PageId> {
    PageId::new(1).ok_or_else(|| "positive Page ID".into())
}

fn text_id(id: u32) -> Result<TextId> {
    TextId::new(id).ok_or_else(|| "positive Text ID".into())
}

fn provenance(raw: &[u8], modified: bool) -> Option<SourceInfo> {
    Some(SourceInfo {
        format: "cdx",
        byte_length: raw.len(),
        modified,
    })
}

fn expected_model(legacy_edited: bool, added_id: Option<u32>, body: &str) -> Result<Value> {
    let legacy: Value = serde_json::from_str(include_str!("fixtures/cdx-caption-edit/cases.json"))?;
    let model = if legacy_edited {
        "after_model"
    } else {
        "before_model"
    };
    let mut model = legacy["cases"]["single_grow"][model].clone();
    if let Some(id) = added_id {
        model["pages"][0]["texts"]
            .as_array_mut()
            .ok_or("independent caption collection")?
            .push(json!({
                "id": id,
                "position": {"x_pt": 144.25, "y_pt": 216.75},
                "runs": [{"text": body, "style": {
                    "font_family": "Roboto", "size_pt": 12.0, "face": 0, "rgb": [0.0, 0.0, 0.0]
                }}]
            }));
    }
    Ok(model)
}

fn check_marked_read(raw: &[u8], id: u32) -> Result {
    let bytes: Arc<[u8]> = Arc::from(raw);
    let document = StoredDocument::from_cdx(Arc::clone(&bytes), PageSize::A4)?;
    assert!(document.has_cdx_caption_preview());
    assert!(!document.is_fully_mapped());
    assert_eq!(
        serde_json::to_value(document.model())?,
        expected_model(false, Some(id), NEW_TEXT)?
    );
    assert_eq!(document.source_info(), provenance(raw, false));
    assert_eq!(document.export_cdx()?.as_ptr(), bytes.as_ptr());
    assert_eq!(document.export_cdx()?, raw);
    Ok(())
}

#[test]
fn legacy_unmarked_captions_keep_dense_ids_in_their_independent_paint_order() -> Result {
    assert_eq!(
        serde_json::to_value(read(BASELINE)?.model())?,
        expected_model(false, None, "")?
    );
    let ordered: Value =
        serde_json::from_str(include_str!("fixtures/cdx-caption/ordered/cases.json"))?;
    for (case, raw) in [
        (
            "back_first",
            include_bytes!("fixtures/cdx-caption/ordered/back_first.cdx").as_slice(),
        ),
        (
            "separate_anonymous",
            include_bytes!("fixtures/cdx-caption/ordered/separate_anonymous.cdx").as_slice(),
        ),
    ] {
        let document = read(raw)?;
        assert_eq!(
            serde_json::to_value(document.model())?,
            ordered[case]["model"],
            "{case}"
        );
        assert_eq!(document.export_cdx()?, raw, "{case}");
    }
    Ok(())
}

#[test]
fn first_marked_caption_reopens_with_dense_live_ids() -> Result {
    check_marked_read(FIRST, 2)
}

#[test]
fn sparse_marker_preserves_existing_ids_even_ahead_of_them_in_raw_id_order() -> Result {
    check_marked_read(SPARSE, 3)?;
    // Source record 10 is the inserted caption. Set its raw ID below old ID 73.
    assert_eq!(&SPARSE[152..158], b"\x06\x80\x5b\0\0\0");
    let mut lower_raw_id = SPARSE.to_vec();
    lower_raw_id[154..158].copy_from_slice(&72_u32.to_le_bytes());
    check_marked_read(&lower_raw_id, 3)
}

#[test]
fn documented_implicit_tag_persistence_has_the_same_sparse_identity() -> Result {
    check_marked_read(IMPLICIT, 3)
}

#[test]
fn invalid_markers_preserve_standalone_bytes_and_retain_open_refusal() -> Result {
    let directory = files::FileTestDirectory::new()?;
    let cancel = Cancellation::default();
    for (name, raw) in [
        (
            "duplicate",
            include_bytes!("fixtures/cdx-caption-create/duplicate_live_id.cdx").as_slice(),
        ),
        (
            "unknown-version",
            include_bytes!("fixtures/cdx-caption-create/unknown_version.cdx").as_slice(),
        ),
        (
            "nonpersistent",
            include_bytes!("fixtures/cdx-caption-create/nonpersistent.cdx").as_slice(),
        ),
    ] {
        let syntax = bondline_cdx::Document::parse(raw)?;
        assert_eq!(syntax.to_bytes(), raw, "{name}");
        assert_eq!(syntax.as_bytes().as_ptr(), raw.as_ptr(), "{name}");
        let error = StoredDocument::from_cdx(Arc::from(raw), PageSize::A4)
            .err()
            .ok_or("invalid identity marker admitted by the caption profile")?;
        assert_eq!(error.code(), "unsupported_content", "{name}: {error}");

        let path = directory.path(&format!("{name}-原-β.cdx"));
        fs::write(&path, raw)?;
        let error = read_cdx_file(&path, &cancel)
            .err()
            .ok_or("invalid file opened")?;
        assert_eq!(error.code(), "unsupported_content", "{name}: {error}");
        assert_eq!(fs::read(path)?, raw, "{name}");
    }
    Ok(())
}

#[test]
fn legacy_and_sparse_edits_keep_both_physical_owners_and_all_other_bytes() -> Result {
    let original = read(SPARSE)?;
    let page = page_id()?;
    assert!(
        original
            .with_cdx_caption_run(page, text_id(2)?, 0, "vacant".into())
            .is_err()
    );

    // The independently frozen old edit grows its Text record by 5 bytes. Keep
    // the complete appended object (including its ID marker) before the same Page end.
    let expected_legacy = [&LEGACY_EDIT[..157], &SPARSE[152..264], &LEGACY_EDIT[157..]].concat();
    let first = original.with_cdx_caption_run(page, text_id(1)?, 0, LEGACY_EDIT_TEXT.into())?;
    assert_eq!(first.export_cdx()?, expected_legacy);
    assert_eq!(
        serde_json::to_value(first.model())?,
        expected_model(true, Some(3), NEW_TEXT)?
    );

    // Only the newly appended caption's short styled Text property changes next.
    let text_header = b"\x00\x07\x1a\0\x01\0\0\0\0\0\0\0\xf0\0\0\0";
    assert_eq!(&expected_legacy[202..214], NEW_TEXT.as_bytes());
    let expected_sparse = [
        &expected_legacy[..186],
        text_header,
        SPARSE_EDIT_TEXT.as_bytes(),
        &expected_legacy[214..],
    ]
    .concat();
    let edited = first.with_cdx_caption_run(page, text_id(3)?, 0, SPARSE_EDIT_TEXT.into())?;
    assert_eq!(edited.export_cdx()?, expected_sparse);
    assert_eq!(
        serde_json::to_value(edited.model())?,
        expected_model(true, Some(3), SPARSE_EDIT_TEXT)?
    );
    assert!(edited.has_cdx_caption_preview());
    assert!(!edited.is_fully_mapped());
    assert!(matches!(
        edited.to_cdxml(),
        Err(DocumentError::CrossFormatSource {
            from: "cdx",
            to: "cdxml"
        })
    ));
    assert_eq!(edited.source_info(), provenance(SPARSE, true));
    assert_eq!(original.export_cdx()?, SPARSE);

    let directory = files::FileTestDirectory::new()?;
    let path = directory.path("captions-保存-β.cdx");
    let cancel = Cancellation::default();
    let receipt = save_cdx_file(&path, &edited, false, &cancel)?;
    assert_eq!(receipt.byte_length, expected_sparse.len());
    assert_eq!(fs::read(&path)?, expected_sparse);
    let reopened = read_cdx_file(&path, &cancel)?.document;
    assert_eq!(reopened.model(), edited.model());
    assert_eq!(reopened.source_info(), provenance(&expected_sparse, false));
    assert_eq!(reopened.export_cdx()?, expected_sparse);
    let same = reopened.with_cdx_caption_run(page, text_id(3)?, 0, SPARSE_EDIT_TEXT.into())?;
    assert!(Arc::ptr_eq(
        &reopened.model_snapshot(),
        &same.model_snapshot()
    ));
    assert_eq!(reopened.export_cdx()?.as_ptr(), same.export_cdx()?.as_ptr());
    Ok(())
}
