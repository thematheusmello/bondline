//! Original fixed pixel-phase control; no native-window or general DPI claim.
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    Point2,
};
use bondline_render::{Cancellation, Error, HitTarget, PageView, cpu_scene, gpu_scene};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn drawing() -> TestResult<Page> {
    let a = AtomId::new(1).ok_or("atom")?;
    let b = AtomId::new(2).ok_or("atom")?;
    let mut document = Document::default();
    document.append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![
                Atom::new(a, Element::try_from(6)?, Some(Point2::new(72.0, 72.0)?)),
                Atom::new(b, Element::try_from(6)?, Some(Point2::new(120.0, 72.0)?)),
            ],
            vec![Bond {
                id: BondId::new(1).ok_or("bond")?,
                begin: a,
                end: b,
                order: BondOrder::Double,
            }],
        )?],
    )?;
    Ok(document.pages().first().ok_or("page")?.clone())
}

fn near(actual: f32, expected: f64) {
    assert!(
        (f64::from(actual) - expected).abs() <= 1e-4,
        "{actual} != {expected}"
    );
}

#[test]
fn fixed_fit_double_refuses_one_x_but_has_two_independent_physical_pixel_strips_at_two_x()
-> TestResult {
    let page = drawing()?;
    let before = page.clone();
    let logical = PageView::fit(848, 728, page.size())?;
    // Independent A4 fit arithmetic: 32-unit margins and rounded469x664 paper.
    let sx = 469.0 / (210.0 * 72.0 / 25.4);
    let sy = 664.0 / (297.0 * 72.0 / 25.4);
    assert!((logical.zoom_percent() - sx * 100.0).abs() < 1e-10);
    assert!(matches!(
        logical.graph_scene(&page, &Cancellation::default()),
        Err(Error::BondScale)
    ));
    let p = Point2::new(96.0, 72.0)?;
    let original_position = logical.canvas_position(p)?;
    near(original_position[0], 189.0 + 96.0 * sx);
    near(original_position[1], 32.0 + 72.0 * sy);
    let physical = logical.at_pixel_size(1696, 1456)?;
    assert_eq!(physical.pixel_density(), 2.0);
    assert_eq!(physical.zoom_percent(), logical.zoom_percent());
    let graph = physical.graph_scene(&page, &Cancellation::default())?;
    let strokes = graph
        .bond_strokes(
            FragmentId::new(1).ok_or("fragment")?,
            BondId::new(1).ok_or("bond")?,
        )
        .collect::<Vec<_>>();
    assert_eq!(strokes.len(), 2);
    let center = 2.0 * (32.0 + 72.0 * sy);
    for (stroke, sign) in strokes.iter().zip([-1.0, 1.0]) {
        near(stroke.begin[0], 2.0 * (189.0 + 72.0 * sx));
        near(stroke.end[0], 2.0 * (189.0 + 120.0 * sx));
        near(stroke.begin[1], center + sign * 3.0 * sx);
        near(stroke.end[1], center + sign * 3.0 * sx);
        near(stroke.width, 2.0 * sx);
    }
    let cpu = cpu_scene(graph.scene())?;
    assert_eq!([cpu.width, cpu.height], [1696, 1456]);
    // Literal independent rows: the two2px strips retain three complete white rows.
    for y in 172..184 {
        for x in 500..560 {
            let offset = usize::try_from((y * 1696 + x) * 4)?;
            assert_eq!(
                cpu.rgba.get(offset..offset + 4).ok_or("pixel")?,
                if [174, 175, 179, 180].contains(&y) {
                    &[0, 0, 0, 255]
                } else {
                    &[255, 255, 255, 255]
                },
                "physical pixel{x},{y}"
            );
        }
    }
    let target = HitTarget::Bond {
        fragment_id: FragmentId::new(1).ok_or("fragment")?,
        bond_id: BondId::new(1).ok_or("bond")?,
    };
    assert_eq!(graph.hit_test([530.5, 174.5], 0.0)?, Some(target));
    assert_eq!(graph.hit_test([530.5, 177.5], 0.0)?, None);
    let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
    println!("fixed physical control GPU backend={:?}", adapter.backend);
    assert_eq!(gpu, cpu);
    assert_eq!(logical.canvas_position(p)?, original_position);
    assert_eq!(page, before);
    assert!(
        page.fragments()
            .first()
            .ok_or("fragment")?
            .atoms()
            .iter()
            .all(|a| a.explicit_hydrogens.is_none())
    );
    Ok(())
}

#[test]
fn display_scale_builds_large_vector_geometry_without_relaxing_frame_readback_limits() -> TestResult
{
    let page = drawing()?;
    let before = page.clone();
    let logical = PageView::fit(2048, 1536, page.size())?;
    assert!(matches!(
        logical.at_pixel_size(4096, 3072),
        Err(Error::InvalidDimensions)
    ));
    let display = logical.at_display_scale(2.0, [0.0, 0.0])?;
    assert_eq!(display.pixel_density(), 2.0);
    assert_eq!(display.zoom_percent(), logical.zoom_percent());
    let graph = display.graph_scene(&page, &Cancellation::default())?;
    assert_eq!([graph.scene().width, graph.scene().height], [4096, 3072]);
    assert_eq!(
        graph
            .bond_strokes(
                FragmentId::new(1).ok_or("fragment")?,
                BondId::new(1).ok_or("bond")?
            )
            .count(),
        2
    );
    assert!(matches!(
        cpu_scene(graph.scene()),
        Err(Error::InvalidDimensions)
    ));
    assert!(matches!(
        pollster::block_on(gpu_scene(graph.scene())),
        Err(Error::InvalidDimensions)
    ));
    assert_eq!(page, before);
    Ok(())
}

#[test]
fn display_and_capture_share_small_geometry_and_one_x_keeps_large_legacy_vectors_exact()
-> TestResult {
    let page = drawing()?;
    let cancel = Cancellation::default();
    let logical = PageView::fit(848, 728, page.size())?;
    assert_eq!(
        logical
            .at_display_scale(2.0, [0.0, 0.0])?
            .graph_scene(&page, &cancel)?
            .scene(),
        logical
            .at_pixel_size(1696, 1456)?
            .graph_scene(&page, &cancel)?
            .scene()
    );
    let large = PageView::fit(4096, 3072, page.size())?;
    let unchanged = large.graph_scene(&page, &cancel)?;
    let display = large.at_display_scale(1.0, [0.0, 0.0])?;
    assert_eq!(display.pixel_density(), 1.0);
    assert_eq!(display.zoom_percent(), large.zoom_percent());
    assert_eq!(
        display.graph_scene(&page, &cancel)?.scene(),
        unchanged.scene()
    );
    assert_eq!(
        display.canvas_position(Point2::new(96.0, 72.0)?)?,
        large.canvas_position(Point2::new(96.0, 72.0)?)?
    );
    assert!(matches!(
        display.at_display_scale(1.0, [0.0, 0.0]),
        Err(Error::InvalidView)
    ));
    assert!(matches!(
        display.at_pixel_size(100, 100),
        Err(Error::InvalidView)
    ));
    Ok(())
}

#[test]
fn fractional_display_density_keeps_exact_camera_phase_and_inverse_hits() -> TestResult {
    let page = drawing()?;
    let before = page.clone();
    let logical = PageView::zoom(320, 240, page.size(), 200.0, Point2::new(96.0, 72.0)?)?;
    assert_eq!(
        logical.canvas_position(Point2::new(96.0, 72.0)?)?,
        [160.0, 120.0]
    );
    let display = logical.at_display_scale(1.5, [0.25, 0.75])?;
    assert_eq!(display.pixel_density(), 1.5);
    assert_eq!(display.zoom_percent(), 200.0);
    assert_eq!(
        display.canvas_position(Point2::new(96.0, 72.0)?)?,
        [240.25, 180.75]
    );
    assert_eq!(
        display.document_position([240.25, 180.75])?,
        Point2::new(96.0, 72.0)?
    );
    let graph = display.graph_scene(&page, &Cancellation::default())?;
    assert_eq!([graph.scene().width, graph.scene().height], [481, 361]);
    let strokes = graph
        .bond_strokes(
            FragmentId::new(1).ok_or("fragment")?,
            BondId::new(1).ok_or("bond")?,
        )
        .collect::<Vec<_>>();
    assert_eq!(strokes.len(), 2);
    for (stroke, y) in strokes.iter().zip([176.25, 185.25]) {
        assert_eq!(stroke.begin, [168.25, y]);
        assert_eq!(stroke.end, [312.25, y]);
        assert_eq!(stroke.width, 3.0);
    }
    assert_eq!(
        graph.hit_test([240.25, 176.25], 0.0)?,
        Some(HitTarget::Bond {
            fragment_id: FragmentId::new(1).ok_or("fragment")?,
            bond_id: BondId::new(1).ok_or("bond")?
        })
    );
    assert_eq!(graph.hit_test([240.25, 180.75], 0.0)?, None);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        display.graph_scene(&page, &cancelled),
        Err(Error::Cancelled)
    ));
    assert_eq!(page, before);
    Ok(())
}

#[test]
fn display_scale_refuses_bad_phase_scale_composition_and_vector_bounds_without_changing_capture_caps()
-> TestResult {
    let page = drawing()?;
    let logical = PageView::fit(320, 240, page.size())?;
    for density in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            logical.at_display_scale(density, [0.0, 0.0]),
            Err(Error::InvalidView)
        ));
    }
    for phase in [
        [-0.01, 0.0],
        [1.0, 0.0],
        [0.0, -0.01],
        [0.0, 1.0],
        [f64::NAN, 0.0],
        [0.0, f64::INFINITY],
    ] {
        assert!(matches!(
            logical.at_display_scale(2.0, phase),
            Err(Error::InvalidView)
        ));
    }
    assert!(matches!(
        logical.at_display_scale(1_000_000.0, [0.0, 0.0]),
        Err(Error::InvalidViewport)
    ));
    let extent = PageView::fit(50_000, 50_000, page.size())?;
    assert_eq!(
        extent.at_display_scale(2.0, [0.0, 0.0])?.source_size(),
        [100_000, 100_000]
    );
    assert!(matches!(
        extent.at_display_scale(2.0, [0.25, 0.0]),
        Err(Error::InvalidViewport)
    ));
    let virtual_page = bondline_model::PageSize::new(40_000.0, 40_000.0)?;
    let source = PageView::zoom(100, 100, virtual_page, 100.0, Point2::new(64.0, 64.0)?)?;
    assert!(matches!(
        source.at_display_scale(3.0, [0.0, 0.0]),
        Err(Error::InvalidViewport)
    ));
    for dimensions in [[0, 1], [1, 0], [2049, 1], [1, 2049]] {
        assert!(matches!(
            logical.at_pixel_size(dimensions[0], dimensions[1]),
            Err(Error::InvalidDimensions)
        ));
    }
    Ok(())
}
