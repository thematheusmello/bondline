//! Independent original binary, style, position and Save witnesses.
use bondline_io::{
    DocumentError, StoredDocument,
    display::{DisplayError, drawing_page_scene},
    files::{read_cdx_file, save_cdx_file},
};
use bondline_model::{PageId, PageSize};
use bondline_render::{Cancellation, cpu_scene};
use serde_json::Value;
use std::{fs, path::Path, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const WITNESSES: &str = include_str!("fixtures/cdx-caption/runs/cases.json");
const SOURCE: &[u8] = include_bytes!("fixtures/cdx-caption/runs/mixed_runs.cdx");
const LEGACY_SPLIT: &[u8] = include_bytes!("fixtures/cdx-caption/two_runs.cdx");
const LEGACY_JOINED: &[u8] = include_bytes!("fixtures/cdx-caption/caption_ascii.cdx");

fn bytes(case: &Value) -> Result<Arc<[u8]>> {
    Ok(Arc::from(fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cdx-caption/runs")
            .join(case["file"].as_str().ok_or("missing binary witness")?),
    )?))
}

fn page() -> Result<PageId> {
    PageId::new(1).ok_or_else(|| "missing positive page ID".into())
}

#[test]
fn explicit_run_boundaries_and_all_literal_styles_reach_the_model() -> Result {
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let mut failures = Vec::new();
    let mut admitted = 0;
    for (name, case) in witnesses["cases"].as_object().ok_or("missing run cases")? {
        if case["model"].is_null() {
            continue;
        }
        let source = bytes(case)?;
        match StoredDocument::from_cdx(Arc::clone(&source), PageSize::A4) {
            Ok(document) => {
                assert_eq!(
                    serde_json::to_value(document.model())?,
                    case["model"],
                    "{name}"
                );
                assert!(document.has_cdx_caption_preview(), "{name}");
                assert!(!document.has_caption_projection(), "{name}");
                assert!(!document.is_fully_mapped(), "{name}");
                assert_eq!(document.export_cdx()?.as_ptr(), source.as_ptr(), "{name}");
                admitted += 1;
            }
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    assert!(failures.is_empty(), "explicit run rejections: {failures:?}");
    assert_eq!(admitted, 5);
    Ok(())
}

#[test]
fn ambiguous_or_invalid_runs_refuse_the_whole_source_with_legacy_errors() -> Result {
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let mut refused = 0;
    for (name, case) in witnesses["cases"].as_object().ok_or("missing run cases")? {
        if !case["model"].is_null() {
            continue;
        }
        let source = bytes(case)?;
        assert_eq!(
            bondline_cdx::Document::parse(&source)?.to_bytes(),
            source.as_ref()
        );
        let error = StoredDocument::from_cdx(source, PageSize::A4)
            .err()
            .ok_or_else(|| format!("unproved run sequence opened: {name}"))?;
        assert_eq!(error.code(), case["legacy"]["code"], "{name}");
        assert_eq!(error.to_string(), case["legacy"]["detail"], "{name}");
        refused += 1;
    }
    assert_eq!(refused, 20);
    Ok(())
}

#[test]
fn legacy_two_runs_preserve_the_joined_geometry_and_their_separate_identities() -> Result {
    let split = StoredDocument::from_cdx(Arc::from(LEGACY_SPLIT), PageSize::A4)?;
    let joined = StoredDocument::from_cdx(Arc::from(LEGACY_JOINED), PageSize::A4)?;
    let cancel = Cancellation::default();
    let a = drawing_page_scene(1024, 1024, &split, page()?, &cancel)?;
    let b = drawing_page_scene(1024, 1024, &joined, page()?, &cancel)?;
    assert_eq!(cpu_scene(a.scene())?, cpu_scene(b.scene())?);
    assert_ne!(split.model(), joined.model());
    assert_eq!(split.export_cdx()?, LEGACY_SPLIT);
    assert_eq!(joined.export_cdx()?, LEGACY_JOINED);
    Ok(())
}

#[test]
fn native_file_path_saves_all_explicit_runs_and_refuses_detached_text_changes() -> Result {
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("runs-原-λ.cdx");
    let output = directory.path("runs-再開-β.cdx");
    fs::write(&input, SOURCE)?;
    let cancel = Cancellation::default();
    let mut loaded = read_cdx_file(&input, &cancel)?.document;
    let before = loaded.clone();
    let frame = drawing_page_scene(1024, 1024, &loaded, page()?, &cancel)?;
    let pixels = cpu_scene(frame.scene())?;
    for ink in [[255, 0, 0, 255], [64, 128, 191, 255]] {
        assert!(pixels.rgba.chunks_exact(4).any(|pixel| pixel == ink));
    }
    assert!(
        !pixels
            .rgba
            .chunks_exact(4)
            .any(|pixel| pixel == [0, 0, 0, 255])
    );
    assert_eq!(frame.hit_test([192.0, 210.0], 100_000.0)?, None);
    assert!(matches!(
        loaded.to_cdxml(),
        Err(DocumentError::CrossFormatSource {
            from: "cdx",
            to: "cdxml"
        })
    ));
    let original = loaded.model().pages()[0]
        .texts()
        .first()
        .ok_or("text")?
        .as_ref()
        .clone();
    loaded
        .model_mut()
        .replace_text(page()?, original.with_run_text(2, "He".into())?)?;
    assert!(!loaded.has_cdx_caption_preview());
    assert!(matches!(
        loaded.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(matches!(
        drawing_page_scene(1024, 1024, &loaded, page()?, &cancel),
        Err(DisplayError::IncompleteSource)
    ));

    loaded.model_mut().replace_text(page()?, original)?;
    assert_eq!(loaded, before);
    save_cdx_file(&output, &loaded, false, &cancel)?;
    assert_eq!(fs::read(&output)?, SOURCE);
    let reopened = read_cdx_file(&output, &cancel)?.document;
    assert_eq!(reopened, before);
    assert!(reopened.has_cdx_caption_preview());
    Ok(())
}
