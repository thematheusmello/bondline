//! Independent API/renderer declaration; missing DrawingStyle/view methods are compile qualification.
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, DrawingStyle, Element,
    Fragment, FragmentId, PageId, PageSize, Point2,
};
use bondline_render::{Cancellation, PageView, Stroke};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn explicit_linewidth_changes_ink_only_and_keeps_the_legacy_geometric_unit() -> TestResult {
    let mut document = Document::default();
    let page_id = PageId::new(1).ok_or("page")?;
    document.set_page_size(page_id, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(
        page_id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![
                Atom::new(
                    AtomId::new(1).ok_or("atom")?,
                    Element::try_from(6)?,
                    Some(Point2::new(32.0, 64.0)?),
                ),
                Atom::new(
                    AtomId::new(2).ok_or("atom")?,
                    Element::try_from(6)?,
                    Some(Point2::new(96.0, 64.0)?),
                ),
            ],
            vec![Bond {
                id: BondId::new(1).ok_or("bond")?,
                begin: AtomId::new(1).ok_or("atom")?,
                end: AtomId::new(2).ok_or("atom")?,
                order: BondOrder::Single,
                double_position: DoublePosition::Unspecified,
            }],
        )?],
    )?;
    let before = document.clone();
    let page = &document.pages()[0];
    let view =
        PageView::fit(576, 576, page.size())?.with_drawing_style(DrawingStyle::new(Some(2.0))?);
    assert_eq!(
        view.stroke_width(),
        2.0,
        "old 1pt geometric unit remains independent"
    );
    assert_eq!(view.ink_width(), 4.0, "2pt ink at exact2x geometric scale");
    let graph = view.graph_scene(page, &Cancellation::default())?;
    assert_eq!(
        graph
            .bond_strokes(
                FragmentId::new(1).ok_or("fragment")?,
                BondId::new(1).ok_or("bond")?
            )
            .cloned()
            .collect::<Vec<_>>(),
        vec![Stroke {
            begin: [96.0, 160.0],
            end: [224.0, 160.0],
            width: 4.0,
            color: [0, 0, 0, 255]
        }]
    );
    assert_eq!(document, before);
    Ok(())
}

#[test]
fn data_valid_but_unrepresentable_physical_width_is_a_scale_error() -> TestResult {
    let page = horizontal(BondOrder::Single)?;
    for width in [f64::from_bits(1), 100000.0] {
        let view = view(&page, Some(width))?;
        assert!(matches!(
            view.graph_scene(&page, &Cancellation::default()),
            Err(Error::BondScale)
        ));
    }
    Ok(())
}

#[test]
fn owned_margin_shortens_the_actual_native_label_components_instead_of_being_ignored() -> TestResult
{
    let page = drawing(
        &[
            (1, [32.0, 64.125], 6, Some(3)),
            (2, [96.0, 64.125], 6, Some(3)),
        ],
        &[(1, 1, 2, BondOrder::Single, DoublePosition::Unspecified)],
    )?;
    let original = page.clone();
    for margin in [0.0, 2.0] {
        let view = PageView::fit(576, 576, page.size())?
            .with_drawing_style(DrawingStyle::default().with_margin_width(Some(margin))?);
        let graph = view.graph_scene(&page, &Cancellation::default())?;
        let component = graph
            .bond_strokes(
                FragmentId::new(1).ok_or("fragment")?,
                BondId::new(1).ok_or("bond")?,
            )
            .next()
            .ok_or("component")?;
        assert!(component.begin[0] > 96.0 + (margin * 2.0) as f32);
        assert!(component.end[0] < 224.0 - (margin * 2.0) as f32);
        assert_eq!(component.width, 2.0);
    }
    assert_eq!(page, original);
    Ok(())
}

// Expanded independent declarations; the first1762-byte source remains exact.
use bondline_model::Page;
use bondline_render::{
    Error, Frame, GraphScene, HitTarget, Primitive, cpu_scene, gpu_scene, graph_page_scene,
};
use std::collections::BTreeMap;
type Edge = (u32, u32, u32, BondOrder, DoublePosition);

fn drawing(points: &[(u32, [f64; 2], u8, Option<u16>)], edges: &[Edge]) -> TestResult<Page> {
    let atoms = points
        .iter()
        .map(|(id, p, z, h)| {
            let mut a = Atom::new(
                AtomId::new(*id).ok_or("atom")?,
                Element::try_from(*z)?,
                Some(Point2::new(p[0], p[1])?),
            );
            a.explicit_hydrogens = *h;
            Ok(a)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .iter()
        .map(|(id, a, b, order, side)| {
            Ok(Bond {
                id: BondId::new(*id).ok_or("bond")?,
                begin: AtomId::new(*a).ok_or("begin")?,
                end: AtomId::new(*b).ok_or("end")?,
                order: *order,
                double_position: *side,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut d = Document::default();
    let pid = PageId::new(1).ok_or("page")?;
    d.set_page_size(pid, PageSize::new(256.0, 256.0)?)?;
    d.append_fragments(
        pid,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(d.pages()[0].clone())
}
fn horizontal(order: BondOrder) -> TestResult<Page> {
    drawing(
        &[(1, [32.0, 64.125], 6, None), (2, [96.0, 64.125], 6, None)],
        &[(1, 1, 2, order, DoublePosition::Unspecified)],
    )
}
fn view(page: &Page, width: Option<f64>) -> TestResult<PageView> {
    Ok(PageView::fit(576, 576, page.size())?.with_drawing_style(DrawingStyle::new(width)?))
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
fn pixel(frame: &Frame, x: u32, y: u32) -> TestResult<&[u8]> {
    let i = usize::try_from((u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4)?;
    Ok(frame.rgba.get(i..i + 4).ok_or("pixel")?)
}
fn gpu_equal(graph: &GraphScene) -> TestResult<Frame> {
    let cpu = cpu_scene(graph.scene())?;
    let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert!(!adapter.name.trim().is_empty());
    println!("linewidth GPU {:?} {}", adapter.backend, adapter.name);
    assert_eq!(gpu, cpu);
    Ok(cpu)
}
// Independent horizontal butt-strip oracle, using only declared endpoint/center/width numbers.
fn strips(
    frame: &Frame,
    xs: [f64; 2],
    ys: &[f64],
    width: f64,
    xrange: std::ops::Range<u32>,
    yrange: std::ops::Range<u32>,
) -> TestResult {
    for y in yrange {
        for x in xrange.clone() {
            let px = f64::from(x) + 0.5;
            let py = f64::from(y) + 0.5;
            let black =
                px >= xs[0] && px <= xs[1] && ys.iter().any(|cy| (py - cy).abs() <= width / 2.0);
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

#[test]
fn half_and_two_point_centered_ink_keeps_literal_six_pixel_separation_and_real_white_gaps()
-> TestResult {
    for (order, rows) in [
        (BondOrder::Single, vec![160.25]),
        (BondOrder::Double, vec![157.25, 163.25]),
        (BondOrder::Triple, vec![154.25, 160.25, 166.25]),
    ] {
        let page = horizontal(order)?;
        let before = page.clone();
        for width_pt in [0.5, 2.0] {
            let v = view(&page, Some(width_pt))?;
            assert_eq!(v.stroke_width(), 2.0);
            assert_eq!(v.ink_width(), (width_pt * 2.0) as f32);
            let graph = v.graph_scene(&page, &Cancellation::default())?;
            let expected = rows
                .iter()
                .map(|y| Stroke {
                    begin: [96.0, *y as f32],
                    end: [224.0, *y as f32],
                    width: (width_pt * 2.0) as f32,
                    color: [0, 0, 0, 255],
                })
                .collect::<Vec<_>>();
            assert_eq!(strokes(&graph, 1)?, expected);
            let cpu = gpu_equal(&graph)?;
            strips(
                &cpu,
                [96.0, 224.0],
                &rows,
                width_pt * 2.0,
                90..230,
                148..173,
            )?;
            assert_eq!(
                graph.hit_test([128.5, rows[0] as f32 + 0.25], 0.0)?,
                Some(HitTarget::Bond {
                    fragment_id: FragmentId::new(1).ok_or("fragment")?,
                    bond_id: BondId::new(1).ok_or("bond")?
                })
            );
            if rows.len() > 1 {
                assert_eq!(
                    graph.hit_test([128.5, ((rows[0] + rows[1]) / 2.0) as f32], 0.0)?,
                    None
                );
            }
            assert_eq!(page, before);
        }
    }
    Ok(())
}

#[test]
fn sided_width_changes_neither_six_pixel_offset_nor_ninety_degree_trim_and_reversal_keeps_pixels()
-> TestResult {
    let mut previous: Option<Frame> = None;
    for width_pt in [0.5, 2.0] {
        for reversed in [false, true] {
            let (a, b, side) = if reversed {
                (2, 1, DoublePosition::Left)
            } else {
                (1, 2, DoublePosition::Right)
            };
            let page = drawing(
                &[
                    (1, [32.0, 64.0], 6, None),
                    (2, [96.0, 64.0], 6, None),
                    (3, [32.0, 112.0], 6, None),
                    (4, [96.0, 112.0], 6, None),
                ],
                &[
                    (1, a, b, BondOrder::Double, side),
                    (2, 1, 3, BondOrder::Single, DoublePosition::Unspecified),
                    (3, 2, 4, BondOrder::Single, DoublePosition::Unspecified),
                ],
            )?;
            let before = page.clone();
            let graph =
                view(&page, Some(width_pt))?.graph_scene(&page, &Cancellation::default())?;
            let endpoints = if reversed {
                [
                    ([224.0, 160.0], [96.0, 160.0]),
                    ([218.0, 166.0], [102.0, 166.0]),
                ]
            } else {
                [
                    ([96.0, 160.0], [224.0, 160.0]),
                    ([102.0, 166.0], [218.0, 166.0]),
                ]
            };
            let expected = endpoints
                .into_iter()
                .map(|(begin, end)| Stroke {
                    begin,
                    end,
                    width: (width_pt * 2.0) as f32,
                    color: [0, 0, 0, 255],
                })
                .collect::<Vec<_>>();
            assert_eq!(strokes(&graph, 1)?, expected);
            let cpu = gpu_equal(&graph)?;
            strips(
                &cpu,
                [102.0, 218.0],
                &[160.0, 166.0],
                width_pt * 2.0,
                110..210,
                153..173,
            )?;
            assert_eq!(graph.hit_test([160.5, 163.5], 0.0)?, None);
            if reversed {
                assert_eq!(Some(&cpu), previous.as_ref());
            } else {
                previous = Some(cpu);
            }
            assert_eq!(page, before);
        }
    }
    Ok(())
}

#[test]
fn linewidth_never_resizes_glyphs_masks_or_literal_h_and_default_wrappers_remain_exact()
-> TestResult {
    let page = drawing(
        &[
            (1, [32.0, 64.125], 6, None),
            (2, [96.0, 64.125], 6, None),
            (3, [128.0, 128.0], 8, Some(2)),
        ],
        &[(1, 1, 2, BondOrder::Single, DoublePosition::Unspecified)],
    )?;
    let before = page.clone();
    let old = graph_page_scene(576, 576, &page)?;
    let cancel = Cancellation::default();
    let baseline = PageView::fit(576, 576, page.size())?.graph_scene(&page, &cancel)?;
    assert_eq!(old.scene(), baseline.scene());
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
    assert_eq!(baseline.label_em_px(), Some(24.0));
    assert!(rectangles(&baseline).len() > 3);
    for width in [None, Some(1.0), Some(0.5), Some(2.0)] {
        let graph = view(&page, width)?.graph_scene(&page, &cancel)?;
        assert_eq!(graph.label_em_px(), Some(24.0));
        assert_eq!(rectangles(&graph), rectangles(&baseline));
        assert_eq!(
            graph.hit_test([288.0, 288.0], 0.0)?,
            Some(HitTarget::Atom {
                fragment_id: FragmentId::new(1).ok_or("fragment")?,
                atom_id: AtomId::new(3).ok_or("atom")?
            })
        );
        if width.is_none() || width == Some(1.0) {
            assert_eq!(graph.scene(), baseline.scene());
        } else {
            let _ = gpu_equal(&graph)?;
        }
    }
    assert_eq!(page, before);
    assert_eq!(page.fragments()[0].atoms()[2].explicit_hydrogens, Some(2));
    Ok(())
}

#[test]
fn styled_ink_survives_physical_cameras_and_fractional_origin_without_changing_geometric_spacing()
-> TestResult {
    let page = horizontal(BondOrder::Double)?;
    let before = page.clone();
    let origin = [0.25, 0.75];
    for density in [1.5, 2.0] {
        for width_pt in [0.5, 2.0] {
            let style = DrawingStyle::new(Some(width_pt))?;
            let logical = PageView::fit(576, 576, page.size())?;
            let original_position = logical.canvas_position(Point2::new(32.0, 64.125)?)?;
            let physical = logical
                .clone()
                .with_drawing_style(style)
                .at_display_scale(density, origin)?;
            let after = logical
                .clone()
                .at_display_scale(density, origin)?
                .with_drawing_style(style);
            assert_eq!(physical.stroke_width(), (2.0 * density) as f32);
            assert_eq!(physical.ink_width(), (width_pt * 2.0 * density) as f32);
            let graph = physical.graph_scene(&page, &Cancellation::default())?;
            assert_eq!(
                graph.scene(),
                after.graph_scene(&page, &Cancellation::default())?.scene()
            );
            let xs = [96.0 * density + origin[0], 224.0 * density + origin[0]];
            let ys = [157.25 * density + origin[1], 163.25 * density + origin[1]];
            let expected = ys
                .iter()
                .map(|y| Stroke {
                    begin: [xs[0] as f32, *y as f32],
                    end: [xs[1] as f32, *y as f32],
                    width: (width_pt * 2.0 * density) as f32,
                    color: [0, 0, 0, 255],
                })
                .collect::<Vec<_>>();
            assert_eq!(strokes(&graph, 1)?, expected);
            let frame = gpu_equal(&graph)?;
            let expected_dimension = (576.0 * density + 0.75).ceil() as u32;
            assert_eq!(
                [frame.width, frame.height],
                [expected_dimension, expected_dimension]
            );
            strips(
                &frame,
                xs,
                &ys,
                width_pt * 2.0 * density,
                (xs[0] as u32 - 3)..(xs[1] as u32 + 4),
                (ys[0] as u32 - 6)..(ys[1] as u32 + 7),
            )?;
            let midx = ((xs[0] + xs[1]) / 2.0) as f32;
            assert!(matches!(
                graph.hit_test([midx, ys[0] as f32], 0.0)?,
                Some(HitTarget::Bond { .. })
            ));
            assert_eq!(
                graph.hit_test([midx, ((ys[0] + ys[1]) / 2.0) as f32], 0.0)?,
                None
            );
            let point = physical.canvas_position(Point2::new(32.0, 64.125)?)?;
            assert_eq!(
                point,
                [
                    (96.0 * density + origin[0]) as f32,
                    (160.25 * density + origin[1]) as f32
                ]
            );
            assert_eq!(
                physical.document_position(point)?,
                Point2::new(32.0, 64.125)?
            );
            assert_eq!(
                logical.canvas_position(Point2::new(32.0, 64.125)?)?,
                original_position
            );
            assert_eq!(page, before);
        }
    }
    Ok(())
}

#[test]
fn actual_width_envelopes_and_collapsed_gaps_keep_pixel_and_nonincident_guards() -> TestResult {
    let make = |y: f64| {
        drawing(
            &[
                (1, [32.0, 64.125], 6, None),
                (2, [96.0, 64.125], 6, None),
                (3, [48.0, y], 6, None),
                (4, [80.0, y], 6, None),
            ],
            &[
                (1, 1, 2, BondOrder::Double, DoublePosition::Unspecified),
                (2, 3, 4, BondOrder::Single, DoublePosition::Unspecified),
            ],
        )
    };
    // At2x: s6/w4 => real outer Double envelope10, not16 and not fixed8.
    let clear = make(59.375)?;
    let before = clear.clone();
    assert!(
        view(&clear, None)?
            .graph_scene(&clear, &Cancellation::default())
            .is_ok()
    );
    let graph = view(&clear, Some(2.0))?.graph_scene(&clear, &Cancellation::default())?;
    let _ = gpu_equal(&graph)?;
    assert_eq!(clear, before);
    // Other Single center153.75, width4 intersects upper Double at pixel155.5.
    let collision = make(60.875)?;
    let before = collision.clone();
    assert!(
        view(&collision, None)?
            .graph_scene(&collision, &Cancellation::default())
            .is_ok()
    );
    assert!(matches!(
        view(&collision, Some(2.0))?.graph_scene(&collision, &Cancellation::default()),
        Err(Error::LabelOverlap)
    ));
    assert_eq!(collision, before);
    for order in [BondOrder::Double, BondOrder::Triple] {
        let page = horizontal(order)?;
        let before = page.clone();
        assert!(matches!(
            view(&page, Some(3.0))?.graph_scene(&page, &Cancellation::default()),
            Err(Error::BondScale)
        ));
        assert_eq!(page, before);
    }
    Ok(())
}

#[test]
fn styled_paper_ink_complexity_and_precancellation_remain_truthful() -> TestResult {
    let mut page = drawing(
        &[
            (1, [1.25, 32.0], 6, Some(3)),
            (2, [1.25, 80.0], 6, Some(2)),
            (3, [49.25, 104.0], 6, Some(3)),
        ],
        &[
            (1, 1, 2, BondOrder::Single, DoublePosition::Unspecified),
            (2, 2, 3, BondOrder::Single, DoublePosition::Unspecified),
        ],
    )?;
    // Glyphless valid C3 keeps the paper counterexample about ink, not label clipping.
    let f = page.fragments()[0].as_ref();
    let mut atoms = f.atoms().to_vec();
    for a in &mut atoms {
        a.show_terminal_carbon_labels = Some(false);
        a.show_non_terminal_carbon_labels = Some(false);
    }
    let mut d = Document::default();
    let pid = d.pages()[0].id();
    d.set_page_size(pid, page.size())?;
    d.append_fragments(pid, vec![Fragment::new(f.id(), atoms, f.bonds().to_vec())?])?;
    page = d.pages()[0].clone();
    let before = page.clone();
    let counts = page.fragments()[0]
        .atoms()
        .iter()
        .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("H")?)))
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    let base = view(&page, None)?.graph_scene_with_display_hydrogens(
        &page,
        &counts,
        &Cancellation::default(),
    )?;
    assert_eq!(base.label_em_px(), None);
    // Physical vertical axis x34.5: width2 stays inside paper inner left33,
    // width4 reaches32.5 and must be refused without changing its authored centers.
    assert!(matches!(
        view(&page, Some(2.0))?.graph_scene_with_display_hydrogens(
            &page,
            &counts,
            &Cancellation::default()
        ),
        Err(Error::LabelPlacement)
    ));
    assert_eq!(page, before);
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        view(&page, Some(2.0))?.graph_scene(&page, &cancel),
        Err(Error::Cancelled)
    ));
    let many = (1..=4097)
        .map(|id| {
            Ok((
                id,
                [32.0 + f64::from(id % 128), 32.0 + f64::from(id / 128)],
                6,
                None,
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let crowded = drawing(&many, &[])?;
    let snapshot = crowded.clone();
    assert!(matches!(
        view(&crowded, Some(0.5))?.graph_scene(&crowded, &Cancellation::default()),
        Err(Error::SceneTooComplex)
    ));
    assert_eq!(crowded, snapshot);
    Ok(())
}
