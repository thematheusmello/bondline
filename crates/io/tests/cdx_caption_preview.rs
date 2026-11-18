//! Existing Open and display APIs, with independent source and model witnesses.
use bondline_io::{
    DocumentError, StoredDocument,
    display::{DisplayError, drawing_page_scene},
    files::{read_cdx_file, save_cdx_file},
};
use bondline_model::{PageId, PageSize};
use bondline_render::{Cancellation, cpu_scene};
use serde_json::Value;
use std::{borrow::Cow, fs, path::PathBuf, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const MANIFEST: &str = include_str!("fixtures/cdx-caption/caption-vectors.json");
const LEGACY_OPEN: &str = include_str!("fixtures/cdx-caption/legacy-open.json");

fn raw(name: &str) -> Result<Arc<[u8]>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cdx-caption")
        .join(format!("{name}.cdx"));
    Ok(Arc::from(fs::read(path)?))
}

fn page() -> Result<PageId> {
    PageId::new(1).ok_or_else(|| "missing positive page ID".into())
}

#[test]
fn each_positive_cdx_caption_opens_with_its_complete_independent_model() -> Result {
    let manifest: Value = serde_json::from_str(MANIFEST)?;
    let mut opened = 0;
    let mut failures = Vec::new();
    for case in manifest["cases"]
        .as_array()
        .ok_or("missing original cases")?
    {
        if case["preview_candidate"] != true {
            continue;
        }
        let name = case["name"].as_str().ok_or("missing source name")?;
        let bytes = raw(name)?;
        match StoredDocument::from_cdx(Arc::clone(&bytes), PageSize::A4) {
            Ok(document) => {
                assert_eq!(
                    serde_json::to_value(document.model())?,
                    case["model"],
                    "{name}"
                );
                assert!(document.has_display_projection(), "{name}");
                assert!(!document.has_caption_projection(), "{name}");
                assert!(!document.is_fully_mapped(), "{name}");
                assert!(matches!(document.to_cdx()?, Cow::Borrowed(_)));
                assert_eq!(document.export_cdx()?.as_ptr(), bytes.as_ptr());
                assert_eq!(document.export_cdx()?, bytes.as_ref());
                let image =
                    drawing_page_scene(1024, 1024, &document, page()?, &Cancellation::default())?;
                assert_eq!(image.hit_test([512.0, 512.0], 100_000.0)?, None, "{name}");
                assert!(image.text_em_range_px().is_some(), "{name}");
                opened += 1;
            }
            Err(error) => failures.push(format!("{name}: {} {error}", error.code())),
        }
    }
    assert!(failures.is_empty(), "caption Open rejections: {failures:?}");
    assert_eq!(opened, 12);
    Ok(())
}

#[test]
fn unsupported_caption_profiles_reproduce_every_legacy_refusal() -> Result {
    let baseline: Value = serde_json::from_str(LEGACY_OPEN)?;
    let mut refused = 0;
    for case in baseline["cases"].as_array().ok_or("missing old cases")? {
        if case["preview_candidate"] != false {
            continue;
        }
        let name = case["name"].as_str().ok_or("missing source name")?;
        let bytes = raw(name)?;
        let syntax = bondline_cdx::Document::parse(&bytes)?;
        assert_eq!(syntax.to_bytes(), bytes.as_ref(), "{name}");
        let error = StoredDocument::from_cdx(bytes, PageSize::A4)
            .err()
            .ok_or("old unsupported source opened")?;
        assert_eq!(Value::from(error.code()), case["outcome"]["code"], "{name}");
        assert_eq!(
            Value::from(error.to_string()),
            case["outcome"]["detail"],
            "{name}"
        );
        refused += 1;
    }
    assert_eq!(refused, 63);
    Ok(())
}

#[test]
fn caption_file_capture_and_unicode_save_reopen_preserve_actual_bytes() -> Result {
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("caption-原-λ.cdx");
    let output = directory.path("caption-保存-λ.cdx");
    let source = raw("caption_rgb16")?;
    fs::write(&input, &source)?;
    let cancel = Cancellation::default();
    let loaded = read_cdx_file(&input, &cancel)?;
    let graph = drawing_page_scene(1024, 1024, &loaded.document, page()?, &cancel)?;
    let cpu = cpu_scene(graph.scene())?;
    assert!(
        cpu.rgba
            .chunks_exact(4)
            .any(|pixel| pixel == [64, 128, 191, 255])
    );
    save_cdx_file(&output, &loaded.document, false, &cancel)?;
    assert_eq!(fs::read(&input)?, source.as_ref());
    assert_eq!(fs::read(&output)?, source.as_ref());
    let reopened = read_cdx_file(&output, &cancel)?;
    assert_eq!(reopened.document, loaded.document);
    Ok(())
}

#[test]
fn source_equal_model_allows_preview_and_model_drift_retires_it() -> Result {
    let bytes = raw("caption_ascii")?;
    let mut document = StoredDocument::from_cdx(Arc::clone(&bytes), PageSize::A4)?;
    let original = document.clone();
    let snapshot = document.model_snapshot();
    let baseline_size = document.model().pages()[0].size();
    let cancel = Cancellation::default();
    let before = drawing_page_scene(1024, 1024, &document, page()?, &cancel)?;
    let _ = document.model_mut();
    assert!(!Arc::ptr_eq(&snapshot, &document.model_snapshot()));
    assert!(document.has_display_projection());
    assert_eq!(document.export_cdx()?.as_ptr(), bytes.as_ptr());

    document
        .model_mut()
        .set_page_size(page()?, PageSize::new(612.0, 792.0)?)?;
    assert!(!document.has_display_projection());
    assert!(matches!(
        document.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(matches!(
        drawing_page_scene(1024, 1024, &document, page()?, &cancel),
        Err(DisplayError::IncompleteSource)
    ));

    document.model_mut().set_page_size(page()?, baseline_size)?;
    assert_eq!(document, original);
    let restored = drawing_page_scene(1024, 1024, &document, page()?, &cancel)?;
    assert_eq!(cpu_scene(restored.scene())?, cpu_scene(before.scene())?);
    assert_eq!(document.export_cdx()?.as_ptr(), bytes.as_ptr());
    Ok(())
}
