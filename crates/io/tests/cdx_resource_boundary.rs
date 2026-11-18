//! Resource-query diagnostics must never become automatic raw CDX Open failures.
use bondline_io::{
    DocumentError, StoredDocument,
    display::{DisplayError, drawing_page_scene},
    files::{read_cdx_file, save_cdx_file},
    mol::{MolTransform, Version},
};
use bondline_model::{PageId, PageSize, Point2};
use bondline_render::Cancellation;
use serde_json::json;
use std::{borrow::Cow, fs, sync::Arc};
#[path = "../../../xtask/oracles/cdx_resource_support.rs"]
#[allow(
    dead_code,
    reason = "shared independent resource source and raw-query facts"
)]
mod f;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated file directory")]
mod files;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn root_resource_query_failures_keep_old_guarded_io_and_exact_unicode_files() -> Result {
    let control = StoredDocument::from_cdx(Arc::from(f::source("absent")), PageSize::A4)?;
    let page = PageId::new(1).ok_or("page id")?;
    let frame = MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
    for (name, query_ok) in [
        ("tables_explicit", true),
        ("invalid_font_header", false),
        ("unsupported_platform_2", false),
        ("duplicate_font_table", false),
        ("duplicate_color_table", false),
    ] {
        let raw = f::source(name);
        let facts = f::facts(name);
        assert_eq!(facts["query"]["ok"], query_ok);
        assert_eq!(f::read(raw)?, facts);
        // These are syntax/IO tests using existing APIs, never Document::resources().
        let syntax = bondline_cdx::Document::parse(raw)?;
        assert_eq!(syntax.as_bytes(), raw);
        assert_eq!(syntax.to_bytes(), raw);
        let bytes: Arc<[u8]> = Arc::from(raw);
        let pointer = bytes.as_ptr();
        let stored = StoredDocument::from_cdx(bytes, PageSize::A4)?;
        assert_eq!(
            stored.model(),
            control.model(),
            "resources stay outside the model"
        );
        assert!(!stored.is_fully_mapped());
        assert!(!stored.has_display_projection());
        assert!(matches!(stored.to_cdx()?, Cow::Borrowed(_)));
        assert_eq!(stored.export_cdx()?.as_ptr(), pointer);
        assert_eq!(stored.to_cdx()?.as_ref(), raw);
        assert_eq!(
            serde_json::to_value(stored.source_info())?,
            json!({"format":"cdx","byte_length":raw.len(),"modified":false})
        );
        assert!(matches!(
            drawing_page_scene(1024, 1024, &stored, page, &Cancellation::default()),
            Err(DisplayError::IncompleteSource)
        ));
        assert!(matches!(
            stored.to_cdxml(),
            Err(DocumentError::CrossFormatSource {
                from: "cdx",
                to: "cdxml"
            })
        ));
        for version in [Version::V2000, Version::V3000] {
            assert!(matches!(
                stored.to_mol_as(version, frame),
                Err(DocumentError::CrossFormatSource {
                    from: "cdx",
                    to: "mol"
                })
            ));
        }
        let directory = files::FileTestDirectory::new()?;
        let input = directory.path("resources-原-λ.cdx");
        let output = directory.path("resources-保存-λ.cdx");
        fs::write(&input, raw)?;
        let loaded = read_cdx_file(&input, &Cancellation::default())?;
        assert_eq!(loaded.document, stored);
        let saved = save_cdx_file(&output, &loaded.document, false, &Cancellation::default())?;
        assert_eq!(saved.byte_length, raw.len());
        let actual = fs::read(&output)?;
        assert_eq!(actual, raw);
        assert_eq!(f::read(&actual)?, facts);
        assert_eq!(
            read_cdx_file(&output, &Cancellation::default())?.document,
            stored
        );
        assert_eq!(fs::read(&input)?, raw);
    }
    Ok(())
}

#[test]
fn malformed_resource_source_survives_equal_detach_and_page_drift_restoration() -> Result {
    let raw = f::source("invalid_font_header");
    let mut stored = StoredDocument::from_cdx(Arc::from(raw), PageSize::A4)?;
    let original = stored.model().clone();
    *stored.model_mut() = original.clone();
    assert!(!stored.source_info().ok_or("source")?.modified);
    assert_eq!(stored.to_cdx()?.as_ref(), raw);
    stored.model_mut().set_page_size(
        PageId::new(1).ok_or("page id")?,
        PageSize::new(612.0, 792.0)?,
    )?;
    assert!(stored.source_info().ok_or("source")?.modified);
    assert!(matches!(
        stored.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    *stored.model_mut() = original;
    assert!(!stored.source_info().ok_or("source")?.modified);
    assert_eq!(stored.to_cdx()?.as_ref(), raw);
    assert_eq!(
        f::read(stored.to_cdx()?.as_ref())?,
        f::facts("invalid_font_header")
    );
    Ok(())
}
