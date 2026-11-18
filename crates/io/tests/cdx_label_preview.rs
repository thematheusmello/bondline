//! Independent CDX placement, full-source refusal and preservation witnesses.
use bondline_io::{
    DocumentError, StoredDocument,
    display::{DisplayError, drawing_page_scene_with_view},
    files::{read_cdx_file, save_cdx_file},
};
use bondline_model::{AtomId, BondId, FragmentId, PageSize, Point2};
use bondline_render::{Cancellation, Frame, HitTarget, PageView, cpu_scene};
use serde_json::Value;
use std::{fs, path::Path, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CASES: &str = include_str!("fixtures/cdx-label-preview/cases.json");
const WATER: &[u8] = include_bytes!("fixtures/cdx-label-preview/water.cdx");
const FRAME: u32 = 1024;
const SCALE: f64 = 2.0;

fn source(name: &str) -> Result<Arc<[u8]>> {
    Ok(Arc::from(fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cdx-label-preview")
            .join(format!("{name}.cdx")),
    )?))
}

fn actual_view(size: PageSize) -> Result<PageView> {
    Ok(PageView::zoom(
        FRAME,
        FRAME,
        size,
        200.0,
        Point2::new(size.width_pt() / 2.0, size.height_pt() / 2.0)?,
    )?)
}

fn opaque_ink(frame: &Frame, color: [u8; 4]) -> Option<[u32; 4]> {
    let mut bounds = [frame.width, frame.height, 0, 0];
    for (index, pixel) in frame.rgba.chunks_exact(4).enumerate() {
        if pixel == color {
            let index = u32::try_from(index).ok()?;
            let (x, y) = (index % frame.width, index / frame.width);
            bounds[0] = bounds[0].min(x);
            bounds[1] = bounds[1].min(y);
            bounds[2] = bounds[2].max(x + 1);
            bounds[3] = bounds[3].max(y + 1);
        }
    }
    (bounds[0] < bounds[2] && bounds[1] < bounds[3]).then_some(bounds)
}

#[test]
fn complete_atom_label_pages_have_independent_models_ink_and_picking() -> Result {
    let vectors: Value = serde_json::from_str(CASES)?;
    let mut unavailable = Vec::new();
    let cancel = Cancellation::default();
    let expected = vectors["renderable"]
        .as_object()
        .ok_or("missing renderable cases")?;
    assert_eq!(expected.len(), 6);
    for (name, case) in expected {
        let raw = source(name)?;
        let stored = StoredDocument::from_cdx(Arc::clone(&raw), PageSize::A4)?;
        if !stored.has_display_projection() {
            unavailable.push(name);
            continue;
        }
        assert!(!stored.is_fully_mapped(), "{name}");
        assert!(!stored.has_cdx_caption_preview(), "{name}");
        assert!(!stored.has_boxed_atom_labels(), "{name}");
        assert!(!stored.has_connected_atom_labels(), "{name}");
        assert_eq!(
            serde_json::to_value(stored.model())?,
            case["model"],
            "{name}"
        );
        assert_eq!(stored.export_cdx()?.as_ptr(), raw.as_ptr(), "{name}");
        let page = stored.model().pages().first().ok_or("missing Page")?;
        let view = actual_view(page.size())?;
        let graph = drawing_page_scene_with_view(&view, &stored, page.id(), &cancel)?;
        let pixels = cpu_scene(graph.scene())?;
        for ink in case["ink"]
            .as_array()
            .ok_or("missing independent ink cases")?
        {
            let [r, g, b]: [u8; 3] = serde_json::from_value(ink["rgb8"].clone())?;
            let expected: [f64; 4] = serde_json::from_value(ink["bounds_pt"].clone())?;
            let actual = opaque_ink(&pixels, [r, g, b, 255])
                .ok_or_else(|| format!("{name} is missing opaque label ink for {ink}"))?;
            for (observed, wanted) in actual.into_iter().zip(expected) {
                assert!(
                    (f64::from(observed) - wanted * SCALE).abs() <= 2.5,
                    "{name}: {ink}"
                );
            }
            let fragment_id = FragmentId::new(serde_json::from_value(ink["fragment_id"].clone())?)
                .ok_or("invalid fragment ID")?;
            let atom_id = AtomId::new(serde_json::from_value(ink["atom_id"].clone())?)
                .ok_or("invalid atom ID")?;
            let anchor = page
                .fragments()
                .iter()
                .find(|fragment| fragment.id() == fragment_id)
                .and_then(|fragment| fragment.atoms().iter().find(|atom| atom.id == atom_id))
                .and_then(|atom| atom.position)
                .ok_or("atom lacks a position")?;
            assert_eq!(
                graph.hit_test(view.canvas_position(anchor)?, 0.0)?,
                Some(HitTarget::Atom {
                    fragment_id,
                    atom_id
                }),
                "{name}"
            );
        }
    }
    assert!(
        unavailable.is_empty(),
        "qualified model(s) still cannot preview: {unavailable:?}"
    );
    Ok(())
}

#[test]
fn explicit_multiple_bond_spacing_controls_actual_rail_pixels_and_hits() -> Result {
    let cancel = Cancellation::default();
    let mut frames = Vec::new();
    for (name, x, axis_y, centers, id) in [
        ("dioxide_manual", 401, 496, &[490.5, 502.5][..], 1),
        ("dioxide_automatic", 401, 496, &[490.5, 502.5][..], 1),
        ("cyanide", 622, 619, &[607.0, 619.0, 631.0][..], 2),
    ] {
        let raw = source(name)?;
        let stored = StoredDocument::from_cdx(raw, PageSize::A4)?;
        let page = stored.model().pages().first().ok_or("missing Page")?;
        let view = actual_view(page.size())?;
        let graph = drawing_page_scene_with_view(&view, &stored, page.id(), &cancel)?;
        let pixels = cpu_scene(graph.scene())?;
        let rows: Vec<u32> = ((axis_y - 20)..=(axis_y + 20))
            .filter(|&y| {
                let index = ((y * FRAME + x) * 4) as usize;
                pixels.rgba.get(index..index + 4) == Some(&[0, 0, 0, 255])
            })
            .collect();
        let rails: Vec<_> = rows.chunk_by(|a, b| a + 1 == *b).collect();
        assert_eq!(rails.len(), centers.len(), "{name}: {rows:?}");
        for (rail, expected_y) in rails.into_iter().zip(centers) {
            let middle = (f64::from(*rail.first().ok_or("missing rail")?)
                + f64::from(*rail.last().ok_or("missing rail")?)
                + 1.0)
                / 2.0;
            assert!((middle - expected_y).abs() <= 0.51, "{name}");
            assert!((2..=4).contains(&rail.len()), "{name}");
            assert_eq!(
                graph.hit_test([x as f32 + 0.5, middle as f32], 0.0)?,
                Some(HitTarget::Bond {
                    fragment_id: FragmentId::new(1).ok_or("invalid fragment ID")?,
                    bond_id: BondId::new(id).ok_or("invalid bond ID")?,
                })
            );
        }
        frames.push((stored, pixels));
    }
    let (manual, manual_pixels) = frames.first().ok_or("missing manual case")?;
    let (automatic, automatic_pixels) = frames.get(1).ok_or("missing automatic case")?;
    assert_eq!(manual.model(), automatic.model());
    assert_eq!(manual_pixels, automatic_pixels);
    assert_ne!(manual.export_cdx()?, automatic.export_cdx()?);
    Ok(())
}

#[test]
fn unproved_layout_keeps_old_projection_or_exact_decode_error_even_when_cropped() -> Result {
    let vectors: Value = serde_json::from_str(CASES)?;
    let cancel = Cancellation::default();
    let mut counts = [0, 0];
    for (name, case) in vectors["retained"]
        .as_object()
        .ok_or("missing control cases")?
    {
        let raw = source(name)?;
        let document = bondline_cdx::Document::parse(&raw)?;
        assert_eq!(document.to_bytes(), raw.as_ref(), "{name}");
        match StoredDocument::from_cdx(Arc::clone(&raw), PageSize::A4) {
            Ok(stored) => {
                assert_eq!(
                    serde_json::to_value(stored.model())?,
                    case["legacy"]["model"],
                    "{name}"
                );
                assert!(!stored.has_display_projection(), "{name}");
                assert_eq!(stored.export_cdx()?.as_ptr(), raw.as_ptr(), "{name}");
                let page = stored.model().pages().first().ok_or("missing Page")?;
                let cropped =
                    PageView::zoom(200, 100, page.size(), 100.0, Point2::new(147.0, 123.0)?)?;
                assert!(
                    matches!(
                        drawing_page_scene_with_view(&cropped, &stored, page.id(), &cancel),
                        Err(DisplayError::IncompleteSource)
                    ),
                    "{name}"
                );
                counts[0] += 1;
            }
            Err(error) => {
                assert_eq!(error.code(), case["legacy"]["code"], "{name}");
                assert_eq!(error.to_string(), case["legacy"]["detail"], "{name}");
                counts[1] += 1;
            }
        }
    }
    assert_eq!(counts, [36, 3]);
    Ok(())
}

#[test]
fn detached_style_drift_retires_display_and_exact_save_until_the_source_model_is_restored() -> Result
{
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("proof-原-λ.cdx");
    let output = directory.path("proof-再開-β.cdx");
    fs::write(&input, WATER)?;
    let cancel = Cancellation::default();
    let mut stored = read_cdx_file(&input, &cancel)?.document;
    assert!(stored.has_display_projection());
    let baseline = stored.clone();
    let style = stored.model().drawing_style();
    stored
        .model_mut()
        .set_drawing_style(style.with_margin_width(Some(11.0))?);
    assert!(!stored.has_display_projection());
    assert!(matches!(
        stored.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(save_cdx_file(&output, &stored, false, &cancel).is_err());
    assert!(!output.exists());
    stored.model_mut().set_drawing_style(style);
    assert_eq!(stored, baseline);
    assert!(stored.has_display_projection());
    save_cdx_file(&output, &stored, false, &cancel)?;
    assert_eq!(fs::read(&output)?, WATER);
    assert_eq!(read_cdx_file(&output, &cancel)?.document, baseline);
    assert!(matches!(
        stored.to_cdxml(),
        Err(DocumentError::CrossFormatSource {
            from: "cdx",
            to: "cdxml"
        })
    ));
    Ok(())
}
