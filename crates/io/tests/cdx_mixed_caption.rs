//! Complete imported drawings and depth samples from the independent binary/font oracle.
use std::{fs, path::Path, sync::Arc};

use bondline_io::{StoredDocument, display::drawing_page_scene_with_view};
use bondline_model::{PageSize, Point2};
use bondline_render::{Cancellation, PageView, cpu_scene};
use serde_json::Value;

const CASES: &str = include_str!("fixtures/cdx-mixed-caption/cases.json");
const PRIOR_OPEN: &str = include_str!("fixtures/cdx-mixed-caption/legacy-open.json");
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn bytes(name: &str) -> Result<Arc<[u8]>> {
    Ok(fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cdx-mixed-caption")
            .join(format!("{name}.cdx")),
    )?
    .into())
}

#[test]
fn mixed_pages_retain_every_fragment_and_caption_and_the_exact_source() -> Result {
    let cases: Value = serde_json::from_str(CASES)?;
    let mut accepted = 0;
    for (name, case) in cases.as_object().ok_or("mixed cases")? {
        if case["model"].is_null() {
            continue;
        }
        let source = bytes(name)?;
        let mut stored = StoredDocument::from_cdx(Arc::clone(&source), PageSize::A4)
            .map_err(|e| format!("{name}: {e}"))?;
        assert_eq!(
            serde_json::to_value(stored.model())?,
            case["model"],
            "{name}"
        );
        assert!(
            stored.has_cdx_mixed_preview() && stored.has_display_projection(),
            "{name}"
        );
        assert!(
            !stored.has_cdx_atom_label_preview() && !stored.has_cdx_caption_preview(),
            "{name}"
        );
        assert!(
            !stored.is_fully_mapped() && stored.can_create_caption(),
            "{name}"
        );
        assert!(stored.to_cdxml().is_err(), "{name}");
        assert_eq!(stored.export_cdx()?.as_ptr(), source.as_ptr(), "{name}");
        let page = stored.model().pages().first().ok_or("page")?.id();
        stored.model_mut().set_page_size(page, PageSize::A4)?;
        assert!(
            !stored.has_cdx_mixed_preview() && !stored.has_display_projection(),
            "{name}"
        );
        assert!(!stored.can_create_caption(), "{name}");
        assert!(stored.export_cdx().is_err(), "{name}");
        accepted += 1;
    }
    assert_eq!(accepted, 12);
    Ok(())
}

#[test]
fn unsupported_mixed_pages_keep_the_prior_error_and_raw_binary() -> Result {
    let cases: Value = serde_json::from_str(CASES)?;
    let prior: Value = serde_json::from_str(PRIOR_OPEN)?;
    let mut count = 0;
    for (name, case) in cases.as_object().ok_or("mixed cases")? {
        if !case["model"].is_null() {
            continue;
        }
        let source = bytes(name)?;
        assert_eq!(bondline_cdx::Document::parse(&source)?.to_bytes(), &*source);
        let error = StoredDocument::from_cdx(source, PageSize::A4)
            .err()
            .ok_or_else(|| format!("unexpected mixed authority for {name}"))?;
        assert_eq!(error.code(), prior[name]["code"], "{name}");
        assert_eq!(error.to_string(), prior[name]["detail"], "{name}");
        count += 1;
    }
    assert_eq!(count, 17);
    Ok(())
}

#[test]
fn higher_captions_cover_lower_label_ink_in_both_source_orders_and_zooms() -> Result {
    let cases: Value = serde_json::from_str(CASES)?;
    for name in ["overlap_before", "overlap_after", "two_captions"] {
        let source = StoredDocument::from_cdx(bytes(name)?, PageSize::A4)?;
        let page = source.model().pages().first().ok_or("page")?;
        let probes = cases[name]["optical_probes"]
            .as_array()
            .ok_or("optical cases")?;
        assert_eq!(probes.len(), 2);
        for probe in probes {
            let size: u32 = serde_json::from_value(probe["size_px"].clone())?;
            let [x, y]: [f64; 2] = serde_json::from_value(probe["center_pt"].clone())?;
            let view = PageView::zoom(
                size,
                size,
                page.size(),
                probe["zoom_percent"].as_f64().ok_or("zoom")?,
                Point2::new(x, y)?,
            )?;
            let graph =
                drawing_page_scene_with_view(&view, &source, page.id(), &Cancellation::default())?;
            let frame = cpu_scene(graph.scene())?;
            let samples = probe["samples"].as_array().ok_or("font outline samples")?;
            assert_eq!(samples.len(), 3);
            for sample in samples {
                let [x, y]: [u32; 2] = serde_json::from_value(sample["xy"].clone())?;
                let [r, g, b]: [u8; 3] = serde_json::from_value(sample["rgb"].clone())?;
                let offset = ((y * size + x) * 4) as usize;
                assert_eq!(
                    frame.rgba.get(offset..offset + 4),
                    Some([r, g, b, 255].as_slice()),
                    "{name}: {sample}"
                );
            }
        }
    }
    Ok(())
}
