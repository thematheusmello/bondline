//! Frozen FontTools probes and analytic straight-outline cells, never renderer-picked points.
use bondline_model::{Document, Page, PageSize, Point2, StyledText, TextId, TextRun, TextStyle};
use bondline_render::{
    Cancellation, CaptionRunTarget, Error, GraphScene, PageView, cpu_scene, gpu_scene,
};
use serde_json::Value;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const WITNESSES: &str = include_str!("../../io/tests/fixtures/cdx-caption-edit/cases.json");

fn target(text: u32, run_index: usize) -> TestResult<Option<CaptionRunTarget>> {
    Ok(Some(CaptionRunTarget {
        text_id: TextId::new(text).ok_or("positive Text ID")?,
        run_index,
    }))
}

fn page(texts: Vec<StyledText>, size: PageSize) -> TestResult<Page> {
    let mut document = Document::default();
    let id = document.pages().first().ok_or("Page")?.id();
    document.set_page_size(id, size)?;
    document.append_texts(id, texts)?;
    Ok(document.pages().first().ok_or("Page")?.clone())
}

fn witness(after: bool) -> TestResult<Page> {
    let data: Value = serde_json::from_str(WITNESSES)?;
    let model = &data["cases"]["multi_shift"][if after { "after_model" } else { "before_model" }];
    let texts = serde_json::from_value(model["pages"][0]["texts"].clone())?;
    let result = page(texts, PageSize::new(512.0, 512.0)?)?;
    assert_eq!(serde_json::to_value(&result)?, model["pages"][0]);
    Ok(result)
}

fn literal(id: u32, position: [f64; 2], runs: &[(&str, f64, [f64; 3])]) -> TestResult<StyledText> {
    Ok(StyledText::new(
        TextId::new(id).ok_or("positive Text ID")?,
        Point2::new(position[0], position[1])?,
        runs.iter()
            .map(|(text, em, rgb)| {
                Ok(TextRun::new(
                    (*text).into(),
                    TextStyle::new("Roboto".into(), *em, 0, *rgb)?,
                )?)
            })
            .collect::<TestResult<Vec<_>>>()?,
    )?)
}

fn fit(page: &Page) -> TestResult<GraphScene> {
    Ok(PageView::fit(576, 576, page.size())?.graph_scene(page, &Cancellation::default())?)
}

#[test]
fn independent_fonttools_interiors_keep_stable_text_and_run_ids_after_growth() -> TestResult {
    // Independent original source/FontTools witnesses: cdx-caption-edit/SOURCES.md.
    // At Fit 576, the 512pt paper has exact unit scale and origin (32,32).
    for (after, omega_x) in [(false, 188.0_f32), (true, 217.0)] {
        let graph = fit(&witness(after)?)?;
        for (point, wanted) in [
            ([132.5, 128.5], target(4, 0)?),
            ([157.75, 250.5], target(2, 0)?),
            ([omega_x + 32.0, 248.5], target(2, 2)?),
        ] {
            assert_eq!(graph.caption_hit_test(point)?, wanted, "{after}: {point:?}");
            assert_eq!(graph.hit_test(point, 100_000.0)?, None);
        }
        if !after {
            assert_eq!(graph.caption_hit_test([183.875, 248.5])?, None);
        }
    }
    Ok(())
}

#[test]
fn actual_paint_order_including_white_ink_wins_over_numeric_text_identity() -> TestResult {
    let original = witness(false)?;
    let mut texts: Vec<_> = original
        .texts()
        .iter()
        .map(|text| text.as_ref().clone())
        .collect();
    texts.swap(1, 2);
    let reversed = fit(&page(texts, original.size())?)?;
    assert_eq!(reversed.caption_hit_test([132.5, 128.5])?, target(3, 0)?);

    let paper = page(
        vec![
            literal(9, [80.0, 128.0], &[("H", 48.0, [1.0, 0.0, 0.0])])?,
            literal(3, [80.0, 128.0], &[("H", 48.0, [1.0; 3])])?,
        ],
        PageSize::new(512.0, 512.0)?,
    )?;
    let graph = fit(&paper)?;
    assert_eq!(graph.caption_hit_test([116.5, 130.5])?, target(3, 0)?);
    let frame = cpu_scene(graph.scene())?;
    let offset = ((130 * frame.width + 116) * 4) as usize;
    assert_eq!(frame.rgba.get(offset..offset + 4), Some(&[255; 4][..]));
    Ok(())
}

#[test]
fn analytic_h_ink_cells_are_half_open_and_do_not_fill_the_glyph_hole() -> TestResult {
    let paper = page(
        vec![literal(19, [80.0, 128.0], &[("H", 48.0, [0.0; 3])])?],
        PageSize::new(512.0, 512.0)?,
    )?;
    let graph = fit(&paper)?;
    // FontTools' H contour has x=169..361 for the left stem, y=0..1456,
    // and crossbar y=673..830, in 2048 units/em. With baseline (112,160),
    // em48 and coverage>=1/2, stem cells are x[116,120), y[126,160).
    for point in [
        [116.0, 130.5],
        [119.999, 130.5],
        [116.5, 126.0],
        [128.0, 142.0],
    ] {
        assert_eq!(graph.caption_hit_test(point)?, target(19, 0)?, "{point:?}");
    }
    for point in [
        [115.999, 130.5],
        [120.0, 130.5],
        [116.5, 125.999],
        [116.5, 160.0],
        [128.0, 130.5],
    ] {
        assert_eq!(graph.caption_hit_test(point)?, None, "{point:?}");
    }
    Ok(())
}

#[test]
fn camera_clipping_aspect_and_fractional_density_keep_the_same_ink_owners() -> TestResult {
    let original = witness(false)?;
    let cancel = Cancellation::default();
    let cropped = PageView::zoom(256, 256, original.size(), 100.0, Point2::new(128.0, 128.0)?)?;
    let graph = cropped.graph_scene(&original, &cancel)?;
    assert_eq!(graph.caption_hit_test([100.5, 96.5])?, target(4, 0)?);
    assert_eq!(graph.caption_hit_test([188.0, 216.5])?, target(2, 2)?);

    let physical = PageView::fit(576, 576, original.size())?.at_display_scale(1.5, [0.25, 0.75])?;
    let graph = physical.graph_scene(&original, &cancel)?;
    assert_eq!([graph.scene().width, graph.scene().height], [865, 865]);
    for (point, wanted) in [
        ([199.0, 193.5], target(4, 0)?),
        ([236.875, 376.5], target(2, 0)?),
        ([330.25, 373.5], target(2, 2)?),
        ([276.0625, 373.5], None),
    ] {
        assert_eq!(graph.caption_hit_test(point)?, wanted, "{point:?}");
    }
    let cpu = cpu_scene(graph.scene())?;
    let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert!(!adapter.name.trim().is_empty());
    assert_eq!(cpu, gpu);

    let wide = page(
        original
            .texts()
            .iter()
            .map(|text| text.as_ref().clone())
            .collect(),
        PageSize::new(512.0, 256.0)?,
    )?;
    let graph = PageView::fit(640, 480, wide.size())?.graph_scene(&wide, &cancel)?;
    // Exact 576x288 paper at (32,96): scale 1.125.
    assert_eq!(graph.caption_hit_test([145.0625, 204.5625])?, target(4, 0)?);

    let lone = page(
        vec![literal(7, [80.0, 128.0], &[("H", 48.0, [0.0; 3])])?],
        original.size(),
    )?;
    let invisible = PageView::zoom(96, 96, lone.size(), 100.0, Point2::new(400.0, 400.0)?)?
        .graph_scene(&lone, &cancel)?;
    for point in [[0.0, 0.0], [48.0, 48.0], [95.999, 95.999]] {
        assert_eq!(invisible.caption_hit_test(point)?, None);
    }
    Ok(())
}

#[test]
fn blank_runs_invalid_points_and_existing_work_and_cancel_bounds_stay_closed() -> TestResult {
    let paper = page(
        vec![literal(
            31,
            [80.0, 128.0],
            &[
                ("", 12.0, [0.0; 3]),
                (" ", 12.0, [0.0; 3]),
                ("H", 48.0, [0.0; 3]),
            ],
        )?],
        PageSize::new(512.0, 512.0)?,
    )?;
    let graph = fit(&paper)?;
    // Space advance is 507/2048 * 12; the point is well inside H's left stem.
    assert_eq!(graph.caption_hit_test([119.5, 130.5])?, target(31, 2)?);
    assert_eq!(graph.caption_hit_test([113.0, 130.5])?, None);
    for point in [
        [f32::NAN, 1.0],
        [1.0, f32::INFINITY],
        [f32::NEG_INFINITY, 1.0],
    ] {
        assert!(matches!(
            graph.caption_hit_test(point),
            Err(Error::InvalidHitTest)
        ));
    }
    for point in [[-0.01, 130.5], [576.0, 130.5], [119.5, 576.0]] {
        assert_eq!(graph.caption_hit_test(point)?, None);
    }
    let spaces = " ".repeat(4096);
    let blank = page(
        vec![literal(1, [80.0, 128.0], &[(&spaces, 12.0, [0.0; 3])])?],
        paper.size(),
    )?;
    assert_eq!(fit(&blank)?.caption_hit_test([112.0, 160.0])?, None);
    let too_many = page(
        vec![literal(
            1,
            [80.0, 128.0],
            &[(&(spaces + " "), 12.0, [0.0; 3])],
        )?],
        paper.size(),
    )?;
    assert!(matches!(
        PageView::fit(576, 576, too_many.size())?.graph_scene(&too_many, &Cancellation::default()),
        Err(Error::SceneTooComplex)
    ));
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        PageView::fit(576, 576, paper.size())?.graph_scene(&paper, &cancel),
        Err(Error::Cancelled)
    ));
    Ok(())
}
