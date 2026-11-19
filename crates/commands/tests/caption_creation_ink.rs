#![cfg(not(target_arch = "wasm32"))]

//! Render the real creation command against the independently measured caption outline.
use bondline_commands::{Cancellation, Engine};
use bondline_io::encode_png;
use bondline_model::PageId;
use bondline_render::{Frame, cpu_scene};
use serde_json::json;
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the isolated actual-file directory")]
mod files;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const FRAME_PX: u32 = 1088;
const WHITE: [u8; 4] = [255; 4];
// FontTools 4.61.1, caption-create/cases.json, first case. Coordinates are page points.
const INK_PT: [f64; 4] = [145.240234375, 207.75, 206.880859375, 219.1875];
// The fixed 32px margin leaves 1024px for this 512pt paper.
const FIT_MARGIN_PX: f64 = 32.0;
const PIXELS_PER_POINT: f64 = 2.0;
// Pixel edges and partial curve coverage can differ from mathematical outline bounds.
const RASTER_ALLOWANCE_PX: f64 = 2.0;
const SAVED: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/first.cdxml");

fn cpu_frame(engine: &Engine) -> TestResult<Frame> {
    let graph = engine.drawing_page_scene(
        FRAME_PX,
        FRAME_PX,
        PageId::new(1).ok_or("invalid fixture page")?,
        &Cancellation::default(),
    )?;
    Ok(cpu_scene(graph.scene())?)
}

fn measured_ink(blank: &Frame, painted: &Frame) -> TestResult<[u32; 4]> {
    blank.validate()?;
    painted.validate()?;
    assert_eq!((blank.width, blank.height), (FRAME_PX, FRAME_PX));
    assert_eq!((painted.width, painted.height), (FRAME_PX, FRAME_PX));
    let mut bounds = [FRAME_PX, FRAME_PX, 0, 0];
    let mut has_black = false;
    for (index, (before, after)) in blank
        .rgba
        .chunks_exact(4)
        .zip(painted.rgba.chunks_exact(4))
        .enumerate()
    {
        let index = u32::try_from(index)?;
        let (x, y) = (index % FRAME_PX, index / FRAME_PX);
        if (33..1055).contains(&x) && (33..1055).contains(&y) {
            assert_eq!(before, WHITE, "empty paper pixel ({x}, {y})");
        }
        if before == after {
            continue;
        }
        assert_eq!(before, WHITE, "backdrop changed at ({x}, {y})");
        let gray = after[0];
        assert_eq!(after, [gray, gray, gray, 255]);
        has_black |= gray == 0;
        bounds[0] = bounds[0].min(x);
        bounds[1] = bounds[1].min(y);
        bounds[2] = bounds[2].max(x + 1);
        bounds[3] = bounds[3].max(y + 1);
    }
    assert!(
        has_black,
        "expected a painted caption with opaque black ink"
    );
    Ok(bounds)
}

#[test]
fn literal_caption_creation_paints_the_independent_outline_and_survives_reopen() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":512,"height_pt":512}),
    )?;
    assert!(engine.stored_document()?.source_info().is_none());
    let blank = cpu_frame(&engine)?;
    let created = engine.dispatch(
        "text.create",
        json!({"expected_revision":engine.state().revision,"page_id":1,
               "x_pt":144.25,"y_pt":216.75,"text":"Phase β & q"}),
    )?;
    assert!(created.changed);
    let painted = cpu_frame(&engine)?;
    let expected = INK_PT.map(|pt| FIT_MARGIN_PX + pt * PIXELS_PER_POINT);
    let observed = measured_ink(&blank, &painted)?;
    for (actual, wanted) in observed.into_iter().zip(expected) {
        assert!(
            (f64::from(actual) - wanted).abs() <= RASTER_ALLOWANCE_PX,
            "ink pixels {observed:?} differ from FontTools bounds {expected:?}"
        );
    }

    let gpu = engine
        .dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":FRAME_PX,"height_px":FRAME_PX,"backend":"gpu"}),
        )?
        .capture
        .ok_or("missing GPU PNG")?;
    assert!(
        gpu.png_bytes()? == encode_png(&painted)?,
        "GPU capture differs from the CPU frame encoded at the same fit"
    );
    let adapter = gpu.adapter.ok_or("missing GPU adapter")?;
    assert!(!adapter.name.is_empty() && !adapter.backend.is_empty());

    let directory = files::FileTestDirectory::new()?;
    let path = directory.path("caption-新規-β.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    assert_eq!(fs::read(&path)?, SAVED);
    engine.dispatch("document.open_cdxml", json!({"path":path}))?;
    assert!(
        cpu_frame(&engine)? == painted,
        "reopened drawing changes the actual painted frame"
    );
    Ok(())
}
