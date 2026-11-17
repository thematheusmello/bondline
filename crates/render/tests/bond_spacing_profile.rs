//! Independent source-point/canvas arithmetic and original external numeric vectors.
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, DrawingSettings, DrawingStyle,
    Element, Fragment, FragmentId, Page, PageSize, Point2,
};
use bondline_render::{
    Cancellation, Error, Frame, GraphScene, HitTarget, PageView, Primitive, Stroke,
    centered_bond_strokes_with_unit, cpu_scene, gpu_scene, graph_page_scene,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Edge = (u32, u32, BondOrder, DoublePosition);

fn drawing(points: &[(f64, f64, u8, Option<u16>)], edges: &[Edge]) -> TestResult<Page> {
    let atoms = points
        .iter()
        .enumerate()
        .map(|(i, (x, y, z, h))| {
            let mut atom = Atom::new(
                AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?,
                Element::try_from(*z)?,
                Some(Point2::new(*x, *y)?),
            );
            atom.explicit_hydrogens = *h;
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .iter()
        .enumerate()
        .map(|(i, (a, b, order, side))| {
            Ok(Bond {
                id: BondId::new(u32::try_from(i + 1)?).ok_or("bond")?,
                begin: AtomId::new(*a).ok_or("begin")?,
                end: AtomId::new(*b).ok_or("end")?,
                order: *order,
                double_position: *side,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut document = Document::default();
    let id = document.pages()[0].id();
    document.set_page_size(id, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(document.pages()[0].clone())
}

fn horizontal(length: f64, order: BondOrder) -> TestResult<Page> {
    drawing(
        &[(32.0, 64.125, 6, None), (32.0 + length, 64.125, 6, None)],
        &[(1, 2, order, DoublePosition::Unspecified)],
    )
}

fn view(
    page: &Page,
    preferred: Option<f64>,
    percent: Option<f64>,
    width: f64,
) -> TestResult<PageView> {
    Ok(PageView::fit(576, 576, page.size())?
        .with_drawing_settings(DrawingSettings::new(preferred, Some(120.0))?)
        .with_drawing_style(DrawingStyle::new(Some(width))?.with_bond_spacing_percent(percent)?))
}

fn strokes(graph: &GraphScene, id: u32) -> TestResult<Vec<Stroke>> {
    Ok(graph
        .bond_strokes(
            FragmentId::new(1).ok_or("fragment")?,
            BondId::new(id).ok_or("bond")?,
        )
        .cloned()
        .collect())
}

fn near(actual: f32, expected: f64) {
    assert!(
        (f64::from(actual) - expected).abs() < 1.0 / 4096.0,
        "{actual} != {expected}"
    );
}

fn pixel(frame: &Frame, x: u32, y: u32) -> TestResult<&[u8]> {
    let i = usize::try_from((u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4)?;
    Ok(frame.rgba.get(i..i + 4).ok_or("pixel")?)
}

fn strips(
    frame: &Frame,
    xs: [f64; 2],
    rows: &[f64],
    width: f64,
    xr: std::ops::Range<u32>,
    yr: std::ops::Range<u32>,
) -> TestResult {
    for y in yr {
        for x in xr.clone() {
            let [px, py] = [f64::from(x) + 0.5, f64::from(y) + 0.5];
            let black =
                px >= xs[0] && px <= xs[1] && rows.iter().any(|r| (py - r).abs() <= width / 2.0);
            assert_eq!(
                pixel(frame, x, y)?,
                if black {
                    &[0, 0, 0, 255]
                } else {
                    &[255, 255, 255, 255]
                },
                "pixel{x},{y}"
            );
        }
    }
    Ok(())
}

fn gpu_equal(graph: &GraphScene) -> TestResult<Frame> {
    let cpu = cpu_scene(graph.scene())?;
    let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
    println!("spacing GPU {:?} {}", adapter.backend, adapter.name);
    assert!(!adapter.name.trim().is_empty());
    assert_eq!(cpu, gpu);
    Ok(cpu)
}

#[test]
fn external_numeric_vectors_use_preferred_length_and_centerline_not_ink_gap() -> TestResult {
    // Numeric output only, from our original inputs executed against Apache-2.0
    // Octet-Draw6395bd582d2c5ba9404e9ddaf175e52e2a9f387c. No upstream code/assets.
    // Source-coordinate render scale was explicit; the external median autoscale
    // is deliberately outside this source-preserving renderer contract.
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/bond-spacing-oracle.json"))?;
    for vector in vectors["vectors"].as_array().ok_or("vectors")? {
        let number = |key: &str| vector[key].as_f64().ok_or("number");
        let order = if vector["order"] == 3 {
            BondOrder::Triple
        } else {
            BondOrder::Double
        };
        let page = horizontal(number("actual")?, order)?;
        let before = page.clone();
        let graph = view(
            &page,
            Some(number("preferred")?),
            Some(number("percent")?),
            number("width")?,
        )?
        .graph_scene(&page, &Cancellation::default())?;
        let rows = if order == BondOrder::Double {
            vec![
                160.25 - number("separation")?,
                160.25 + number("separation")?,
            ]
        } else {
            vec![
                160.25 - 2.0 * number("separation")?,
                160.25,
                160.25 + 2.0 * number("separation")?,
            ]
        };
        let actual = strokes(&graph, 1)?;
        assert_eq!(actual.len(), rows.len());
        for (stroke, y) in actual.iter().zip(&rows) {
            near(stroke.begin[0], 96.0);
            near(stroke.end[0], 96.0 + 2.0 * number("actual")?);
            near(stroke.begin[1], *y);
            near(stroke.end[1], *y);
            near(stroke.width, 2.0 * number("width")?);
        }
        for pair in actual.windows(2) {
            near(
                pair[1].begin[1] - pair[0].begin[1],
                2.0 * number("separation")?,
            );
            near(
                pair[1].begin[1] - pair[0].begin[1] - pair[0].width,
                2.0 * number("gap")?,
            );
        }
        let frame = if vector["name"] == "center-actual20" || vector["name"] == "triple-base" {
            gpu_equal(&graph)?
        } else {
            cpu_scene(graph.scene())?
        };
        strips(
            &frame,
            [96.0, 96.0 + 2.0 * number("actual")?],
            &rows,
            2.0 * number("width")?,
            100..130,
            120..200,
        )?;
        let midx = (96.0 + number("actual")?) as f32;
        assert!(matches!(
            graph.hit_test([midx, actual[0].begin[1]], 0.0)?,
            Some(HitTarget::Bond { .. })
        ));
        assert_eq!(
            graph.hit_test([midx, (rows[0] + rows[1]) as f32 / 2.0], 0.0)?,
            None
        );
        assert_eq!(page, before);
    }
    Ok(())
}

#[test]
fn explicit_sided_spacing_uses_existing_bisectors_and_direction_at_large_ratio() -> TestResult {
    for reversed in [false, true] {
        let (a, b, side) = if reversed {
            (2, 1, DoublePosition::Left)
        } else {
            (1, 2, DoublePosition::Right)
        };
        let page = drawing(
            &[
                (32.0, 64.125, 6, None),
                (52.0, 64.125, 6, None),
                (32.0, 112.125, 6, None),
                (52.0, 112.125, 6, None),
            ],
            &[
                (a, b, BondOrder::Double, side),
                (1, 3, BondOrder::Single, DoublePosition::Unspecified),
                (2, 4, BondOrder::Single, DoublePosition::Unspecified),
            ],
        )?;
        let before = page.clone();
        let graph = view(&page, Some(40.0), Some(18.0), 0.6)?
            .graph_scene(&page, &Cancellation::default())?;
        let expected = if reversed {
            vec![
                ([136.0, 160.25], [96.0, 160.25]),
                ([121.6, 174.65], [110.4, 174.65]),
            ]
        } else {
            vec![
                ([96.0, 160.25], [136.0, 160.25]),
                ([110.4, 174.65], [121.6, 174.65]),
            ]
        };
        let actual = strokes(&graph, 1)?;
        assert_eq!(actual.len(), 2);
        for (stroke, (begin, end)) in actual.iter().zip(expected) {
            for i in 0..2 {
                near(stroke.begin[i], begin[i]);
                near(stroke.end[i], end[i]);
            }
            near(stroke.width, 1.2);
        }
        let cpu = gpu_equal(&graph)?;
        strips(
            &cpu,
            [110.4, 121.6],
            &[160.25, 174.65],
            1.2,
            112..120,
            154..181,
        )?;
        assert_eq!(graph.hit_test([116.5, 167.5], 0.0)?, None);
        assert_eq!(page, before);
    }
    Ok(())
}

#[test]
fn spacing_none_retains_legacy_helpers_frames_guards_and_glyph_metrics() -> TestResult {
    let page = horizontal(64.0, BondOrder::Double)?;
    let before = page.clone();
    let baseline = graph_page_scene(576, 576, &page)?;
    for preferred in [None, Some(20.0), Some(80.0)] {
        let graph =
            view(&page, preferred, None, 1.0)?.graph_scene(&page, &Cancellation::default())?;
        assert_eq!(graph.scene(), baseline.scene());
    }
    let axis = Stroke {
        begin: [96.0, 160.25],
        end: [224.0, 160.25],
        width: 2.0,
        color: [0, 0, 0, 255],
    };
    assert_eq!(
        centered_bond_strokes_with_unit(&axis, BondOrder::Double, 2.0)?,
        strokes(&baseline, 1)?
    );
    let short = horizontal(8.0, BondOrder::Double)?;
    assert!(matches!(
        view(&short, Some(40.0), None, 1.0)?.graph_scene(&short, &Cancellation::default()),
        Err(Error::UnsupportedChemicalContent)
    ));
    let labels = drawing(
        &[
            (32.0, 64.125, 6, None),
            (96.0, 64.125, 6, None),
            (160.0, 160.0, 8, Some(2)),
        ],
        &[(1, 2, BondOrder::Double, DoublePosition::Center)],
    )?;
    let legacy = view(&labels, None, None, 1.0)?.graph_scene(&labels, &Cancellation::default())?;
    let spaced = view(&labels, Some(40.0), Some(18.0), 1.0)?
        .graph_scene(&labels, &Cancellation::default())?;
    let rectangles = |g: &GraphScene| {
        g.scene()
            .primitives
            .iter()
            .filter_map(|p| match p {
                Primitive::Rectangle(r) => Some(r.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(legacy.label_em_px(), Some(24.0));
    assert_eq!(spaced.label_em_px(), Some(24.0));
    assert_eq!(rectangles(&legacy), rectangles(&spaced));
    assert_eq!(page, before);
    Ok(())
}

#[test]
fn physical_density_phase_camera_and_later_settings_share_exact_spacing() -> TestResult {
    let page = horizontal(40.0, BondOrder::Double)?;
    let before = page.clone();
    let settings = DrawingSettings::new(Some(40.0), Some(120.0))?;
    let style = DrawingStyle::new(Some(0.6))?.with_bond_spacing_percent(Some(18.0))?;
    for density in [1.5, 2.0] {
        let logical = PageView::fit(576, 576, page.size())?;
        let physical = logical
            .clone()
            .with_drawing_settings(settings)
            .with_drawing_style(style)
            .at_display_scale(density, [0.25, 0.75])?;
        let later = logical
            .clone()
            .at_display_scale(density, [0.25, 0.75])?
            .with_drawing_settings(settings)
            .with_drawing_style(style);
        let graph = physical.graph_scene(&page, &Cancellation::default())?;
        assert_eq!(
            graph.scene(),
            later.graph_scene(&page, &Cancellation::default())?.scene()
        );
        let expected_rows = [153.05 * density + 0.75, 167.45 * density + 0.75];
        let actual = strokes(&graph, 1)?;
        for (stroke, y) in actual.iter().zip(expected_rows) {
            near(stroke.begin[0], 96.0 * density + 0.25);
            near(stroke.end[0], 176.0 * density + 0.25);
            near(stroke.begin[1], y);
            near(stroke.end[1], y);
            near(stroke.width, 1.2 * density);
        }
        assert_eq!(physical.stroke_width(), (2.0 * density) as f32);
        near(physical.ink_width(), 1.2 * density);
        let point = Point2::new(52.0, 64.125)?;
        assert_eq!(
            physical.document_position(physical.canvas_position(point)?)?,
            point
        );
        let frame = gpu_equal(&graph)?;
        strips(
            &frame,
            [96.0 * density + 0.25, 176.0 * density + 0.25],
            &expected_rows,
            1.2 * density,
            (110.0 * density) as u32..(150.0 * density) as u32,
            (148.0 * density) as u32..(172.0 * density) as u32,
        )?;
    }
    assert_eq!(page, before);
    Ok(())
}

#[test]
fn actual_clear_gap_corridor_and_paper_remain_guards_for_explicit_spacing() -> TestResult {
    let page = horizontal(40.0, BondOrder::Double)?;
    let before = page.clone();
    for (percent, width) in [(0.0, 0.6), (1.0, 0.6), (18.0, 7.2)] {
        assert!(
            matches!(
                view(&page, Some(40.0), Some(percent), width)?
                    .graph_scene(&page, &Cancellation::default()),
                Err(Error::BondScale)
            ),
            "p{percent},w{width}"
        );
    }
    // Same physical components can have no ink or no complete gap pixel, despite s>w.
    assert!(matches!(
        view(&page, Some(40.0), Some(2.0), 0.6)?.graph_scene(&page, &Cancellation::default()),
        Err(Error::BondScale)
    ));
    let single = horizontal(40.0, BondOrder::Single)?;
    assert!(
        view(&single, Some(40.0), Some(0.0), 0.6)?
            .graph_scene(&single, &Cancellation::default())
            .is_ok()
    );
    let crowded = drawing(
        &[
            (32.0, 64.125, 6, None),
            (96.0, 64.125, 6, None),
            (48.0, 60.125, 6, None),
            (80.0, 60.125, 6, None),
        ],
        &[
            (1, 2, BondOrder::Double, DoublePosition::Center),
            (3, 4, BondOrder::Single, DoublePosition::Unspecified),
        ],
    )?;
    assert!(
        view(&crowded, Some(40.0), None, 1.0)?
            .graph_scene(&crowded, &Cancellation::default())
            .is_ok()
    );
    assert!(matches!(
        view(&crowded, Some(40.0), Some(18.0), 1.0)?
            .graph_scene(&crowded, &Cancellation::default()),
        Err(Error::LabelOverlap)
    ));
    let edge = drawing(
        &[(32.0, 2.0, 6, None), (96.0, 2.0, 6, None)],
        &[(1, 2, BondOrder::Double, DoublePosition::Center)],
    )?;
    assert!(matches!(
        view(&edge, Some(40.0), Some(18.0), 0.6)?.graph_scene(&edge, &Cancellation::default()),
        Err(Error::LabelPlacement)
    ));
    let max = view(&page, Some(100000.0), Some(3276.7), 0.6)?;
    assert!(max.graph_scene(&page, &Cancellation::default()).is_err());
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        max.graph_scene(&page, &cancel),
        Err(Error::Cancelled)
    ));
    assert_eq!(page, before);
    Ok(())
}

#[test]
fn explicit_one_hundred_percent_can_render_but_impossible_inset_is_not_clamped() -> TestResult {
    let page = horizontal(20.0, BondOrder::Triple)?;
    let before = page.clone();
    let graph =
        view(&page, Some(40.0), Some(100.0), 1.0)?.graph_scene(&page, &Cancellation::default())?;
    let actual = strokes(&graph, 1)?;
    assert_eq!(actual.len(), 3);
    for (s, y) in actual.iter().zip([80.25, 160.25, 240.25]) {
        near(s.begin[1], y);
        near(s.end[1], y);
    }
    let impossible = drawing(
        &[
            (32.0, 64.125, 6, None),
            (44.0, 64.125, 6, None),
            (32.0, 112.125, 6, None),
            (44.0, 112.125, 6, None),
        ],
        &[
            (1, 2, BondOrder::Double, DoublePosition::Right),
            (1, 3, BondOrder::Single, DoublePosition::Unspecified),
            (2, 4, BondOrder::Single, DoublePosition::Unspecified),
        ],
    )?;
    let unchanged = impossible.clone();
    assert!(matches!(
        view(&impossible, Some(40.0), Some(18.0), 0.6)?
            .graph_scene(&impossible, &Cancellation::default()),
        Err(Error::UnsupportedChemicalContent)
    ));
    assert_eq!(impossible, unchanged);
    assert_eq!(page, before);
    Ok(())
}
