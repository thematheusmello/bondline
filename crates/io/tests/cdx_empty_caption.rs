//! Exact binary witnesses for creating the first caption on a retained CDX page.
use bondline_io::StoredDocument;
use bondline_model::{PageSize, Point2, StyledText, TextId, TextRun, TextStyle};
use serde_json::Value;
use std::{error::Error, fs, path::Path, sync::Arc};

const CASES: &str = include_str!("fixtures/cdx-caption-empty/cases.json");

fn bytes(case: &Value) -> Result<Arc<[u8]>, Box<dyn Error>> {
    let name = case["file"].as_str().ok_or("fixture filename")?;
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cdx-caption-empty")
        .join(name);
    Ok(fs::read(path)?.into())
}

#[test]
fn empty_poster_maps_physical_size_and_writes_the_first_caption_in_each_framing()
-> Result<(), Box<dyn Error>> {
    let manifest: Value = serde_json::from_str(CASES)?;
    let cases = manifest["cases"].as_object().ok_or("fixture cases")?;
    assert_eq!(cases.len(), 6);
    for (name, case) in cases {
        let source = bytes(&case["before"])?;
        let original = StoredDocument::from_cdx(Arc::clone(&source), PageSize::A4)?;
        assert!(original.has_cdx_caption_preview(), "{name}");
        assert!(original.has_display_projection(), "{name}");
        assert!(original.can_create_caption(), "{name}");
        assert!(!original.is_fully_mapped(), "{name}");
        assert!(original.to_cdxml().is_err(), "{name}");
        assert_eq!(original.export_cdx()?, source.as_ref(), "{name}");
        assert_eq!(
            serde_json::to_value(original.model())?,
            case["before"]["model"],
            "{name}"
        );
        let page = original.model().pages().first().ok_or("Poster page")?;
        for (live_id, outcome) in [(1, "first"), (2, "after_undo")] {
            let id = TextId::new(live_id).ok_or("live ID")?;
            let run = TextRun::new(
                "Phase β & q".into(),
                TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
            )?;
            let caption = StyledText::new(id, Point2::new(144.25, 216.75)?, vec![run])?;
            let change = original.with_cdx_caption(page.id(), caption)?;
            let expected = bytes(&case[outcome])?;
            assert_eq!(change.export_cdx()?, expected.as_ref(), "{name}/{outcome}");
            assert_eq!(original.export_cdx()?, source.as_ref(), "{name}/{outcome}");
            assert_eq!(
                serde_json::to_value(change.model())?,
                case[outcome]["model"],
                "{name}/{outcome}"
            );
            let reopened = StoredDocument::from_cdx(expected, PageSize::A4)?;
            assert_eq!(reopened.model(), change.model(), "{name}/{outcome}");
            assert!(reopened.can_create_caption(), "{name}/{outcome}");
        }
    }
    Ok(())
}

#[test]
fn unsupported_empty_sources_keep_their_previous_app_and_unchanged_source_behavior()
-> Result<(), Box<dyn Error>> {
    let manifest: Value = serde_json::from_str(CASES)?;
    let controls = manifest["controls"].as_object().ok_or("fixture controls")?;
    assert_eq!(controls.len(), 19);
    for (name, case) in controls {
        let source = bytes(case)?;
        assert_eq!(
            bondline_cdx::Document::parse(&source)?.to_bytes(),
            source.as_ref(),
            "{name}"
        );
        let projection = StoredDocument::from_cdx(Arc::clone(&source), PageSize::A4);
        if case["expected_open"] == "legacy_elemental" {
            let document = projection?;
            assert_eq!(document.model().pages().len(), 1, "{name}");
            let page = document.model().pages().first().ok_or("fallback page")?;
            assert_eq!(page.size(), PageSize::A4, "{name}");
            assert!(
                page.fragments().is_empty() && page.texts().is_empty(),
                "{name}"
            );
            assert!(!document.has_cdx_caption_preview(), "{name}");
            assert!(!document.can_create_caption(), "{name}");
            assert!(!document.has_display_projection(), "{name}");
            assert!(!document.is_fully_mapped(), "{name}");
            assert!(document.to_cdxml().is_err(), "{name}");
            assert_eq!(document.export_cdx()?, source.as_ref(), "{name}");
        } else {
            assert!(projection.is_err(), "{name}");
        }
    }
    Ok(())
}
