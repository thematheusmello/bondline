//! Existing-API Right Double acceptance; the old renderer refuses at runtime.
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, Element, Fragment, FragmentId,
    PageId, PageSize, Point2,
};
use bondline_render::{Stroke, graph_page_scene};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn directed_right_double_ends_on_both_ninety_degree_bisectors() -> TestResult {
    let atoms = [
        (29, 32.0, 64.0),
        (11, 96.0, 64.0),
        (7, 32.0, 112.0),
        (53, 96.0, 112.0),
    ]
    .into_iter()
    .map(|(id, x, y)| {
        Ok(Atom::new(
            AtomId::new(id).ok_or("atom")?,
            Element::try_from(6)?,
            Some(Point2::new(x, y)?),
        ))
    })
    .collect::<TestResult<Vec<_>>>()?;
    let bonds = [
        (41, 29, 11, BondOrder::Double, DoublePosition::Right),
        (42, 29, 7, BondOrder::Single, DoublePosition::Unspecified),
        (43, 11, 53, BondOrder::Single, DoublePosition::Unspecified),
    ]
    .into_iter()
    .map(|(id, begin, end, order, double_position)| {
        Ok(Bond {
            id: BondId::new(id).ok_or("bond")?,
            begin: AtomId::new(begin).ok_or("begin")?,
            end: AtomId::new(end).ok_or("end")?,
            order,
            double_position,
        })
    })
    .collect::<TestResult<Vec<_>>>()?;
    let mut document = Document::default();
    let pid = PageId::new(1).ok_or("page")?;
    document.set_page_size(pid, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(
        pid,
        vec![Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    let before = document.clone();
    let page = document.pages().first().ok_or("page")?;
    // Independent map: 576-square -> origin32, paper512, scale2, w2 and s6.
    // Both same-side 90-degree rays require6 units of trim at their own ends.
    let graph = graph_page_scene(576, 576, page)?;
    assert_eq!(
        graph
            .bond_strokes(
                FragmentId::new(17).ok_or("fragment")?,
                BondId::new(41).ok_or("bond")?
            )
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            Stroke {
                begin: [96.0, 160.0],
                end: [224.0, 160.0],
                width: 2.0,
                color: [0, 0, 0, 255]
            },
            Stroke {
                begin: [102.0, 166.0],
                end: [218.0, 166.0],
                width: 2.0,
                color: [0, 0, 0, 255]
            },
        ]
    );
    assert_eq!(document, before);
    assert!(
        page.fragments()[0]
            .atoms()
            .iter()
            .all(|atom| atom.explicit_hydrogens.is_none())
    );
    Ok(())
}

// Additive declarations retain the exact 2,208-byte initial existing-API case.
use bondline_model::Page;
use bondline_render::{
    Cancellation, Error, Frame, GraphScene, HitTarget, PageView, cpu_scene, gpu_scene,
    graph_page_scene_with_cancel,
};

// Coordinates here stay below2048. Four half-ULPs at that range cover f64->f32
// construction plus one f32 camera projection. This is not a pixel tolerance.
const COORD_EPS: f64 = 1.0 / 4096.0;
const SINGLE: BondOrder = BondOrder::Single;
const DOUBLE: BondOrder = BondOrder::Double;
const UNSPECIFIED: DoublePosition = DoublePosition::Unspecified;
const RIGHT: DoublePosition = DoublePosition::Right;
const LEFT: DoublePosition = DoublePosition::Left;
type Edge = (u32, u32, u32, BondOrder, DoublePosition);

fn drawing(points: &[(u32, [f64; 2])], edges: &[Edge], extent: f64) -> TestResult<Page> {
    let atoms = points
        .iter()
        .map(|(id, p)| {
            Ok(Atom::new(
                AtomId::new(*id).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(p[0], p[1])?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .iter()
        .map(|(id, begin, end, order, position)| {
            Ok(Bond {
                id: BondId::new(*id).ok_or("bond")?,
                begin: AtomId::new(*begin).ok_or("begin")?,
                end: AtomId::new(*end).ok_or("end")?,
                order: *order,
                double_position: *position,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut doc = Document::default();
    let pid = PageId::new(1).ok_or("page")?;
    doc.set_page_size(pid, PageSize::new(extent, extent)?)?;
    doc.append_fragments(
        pid,
        vec![Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(doc.pages().first().ok_or("page")?.clone())
}

fn rays(
    a: &[[f64; 2]],
    b: &[[f64; 2]],
    position: DoublePosition,
    reversed: bool,
) -> TestResult<Page> {
    let mut points = vec![(29, [64.0, 96.0]), (11, [192.0, 96.0])];
    let (begin, end) = if reversed { (11, 29) } else { (29, 11) };
    let mut edges = vec![(41, begin, end, DOUBLE, position)];
    for (i, (anchor, p)) in a
        .iter()
        .map(|p| (29, p))
        .chain(b.iter().map(|p| (11, p)))
        .enumerate()
    {
        let i = u32::try_from(i)?;
        let id = 100 + i * 7;
        points.push((id, *p));
        edges.push((50 + i * 11, anchor, id, SINGLE, UNSPECIFIED));
    }
    drawing(&points, &edges, 256.0)
}

fn bundle(graph: &GraphScene, expected: [([f64; 2], [f64; 2]); 2], width: f32) -> TestResult {
    let strokes = graph
        .bond_strokes(
            FragmentId::new(17).ok_or("fragment")?,
            BondId::new(41).ok_or("bond")?,
        )
        .collect::<Vec<_>>();
    assert_eq!(strokes.len(), 2);
    for (stroke, (begin, end)) in strokes.into_iter().zip(expected) {
        for (actual, expected) in stroke
            .begin
            .into_iter()
            .chain(stroke.end)
            .zip(begin.into_iter().chain(end))
        {
            assert!(
                (f64::from(actual) - expected).abs() <= COORD_EPS,
                "coordinate {actual} vs {expected}"
            );
        }
        assert_eq!(stroke.width, width);
        assert_eq!(stroke.color, [0, 0, 0, 255]);
    }
    Ok(())
}

fn hit(graph: &GraphScene, point: [f32; 2], bond: Option<u32>) -> TestResult {
    let expected = match bond {
        Some(id) => Some(HitTarget::Bond {
            fragment_id: FragmentId::new(17).ok_or("fragment")?,
            bond_id: BondId::new(id).ok_or("bond")?,
        }),
        None => None,
    };
    assert_eq!(graph.hit_test(point, 0.0)?, expected);
    Ok(())
}

fn pixel(frame: &Frame, x: u32, y: u32) -> TestResult<&[u8]> {
    let offset = usize::try_from((u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4)?;
    Ok(frame.rgba.get(offset..offset + 4).ok_or("RGBA pixel")?)
}

fn horizontal_pixels(
    frame: &Frame,
    xs: std::ops::Range<u32>,
    ys: std::ops::Range<u32>,
    strips: &[(f64, f64, f64, f64)],
) -> TestResult {
    for y in ys {
        for x in xs.clone() {
            let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let ink = strips.iter().any(|(a, b, row, width)| {
                px >= a.min(*b) && px <= a.max(*b) && (py - row).abs() <= width / 2.0
            });
            assert_eq!(
                pixel(frame, x, y)?,
                if ink {
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

fn gpu_matches(graph: &GraphScene) -> TestResult<Frame> {
    let cpu = cpu_scene(graph.scene())?;
    let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert_eq!(gpu, cpu);
    eprintln!(
        "inset actual GPU backend={:?}, adapter={}",
        adapter.backend, adapter.name
    );
    Ok(cpu)
}

fn collision(result: Result<GraphScene, Error>) -> TestResult {
    let error = result.err().ok_or("colliding inset scene was accepted")?;
    assert!(matches!(error, Error::LabelOverlap), "{error}");
    Ok(())
}

fn scale_refusal(result: Result<GraphScene, Error>) -> TestResult {
    let error = result
        .err()
        .ok_or("missing component/common-span gap was accepted")?;
    assert!(matches!(error, Error::BondScale), "{error}");
    Ok(())
}

#[test]
fn three_four_five_and_one_twenty_rays_have_independent_bisector_trims() -> TestResult {
    let root = 3.0_f64.sqrt();
    for (a, b, trim) in [
        ([112.0, 160.0], [144.0, 160.0], 6.0), // v=(3/5,4/5): cot(theta/2)=2, s=3.
        (
            [32.0, 96.0 + 32.0 * root],
            [224.0, 96.0 + 32.0 * root],
            root,
        ), // theta120: s/sqrt3.
    ] {
        let page = rays(&[a], &[b], RIGHT, false)?;
        let before = page.clone();
        let graph = graph_page_scene(320, 320, &page)?;
        bundle(
            &graph,
            [
                ([96.0, 128.0], [224.0, 128.0]),
                ([96.0 + trim, 131.0], [224.0 - trim, 131.0]),
            ],
            1.0,
        )?;
        horizontal_pixels(
            &cpu_scene(graph.scene())?,
            128..192,
            124..136,
            &[
                (96.0, 224.0, 128.0, 1.0),
                (96.0 + trim, 224.0 - trim, 131.0, 1.0),
            ],
        )?;
        hit(&graph, [160.0, 131.0], Some(41))?;
        hit(&graph, [160.0, 129.5], None)?;
        assert_eq!(page, before);
    }
    Ok(())
}

#[test]
fn terminal_across_axis_and_trans_only_ends_have_zero_trim() -> TestResult {
    let terminal = rays(&[], &[], RIGHT, false)?;
    let opposite = rays(&[[64.0, 32.0]], &[[192.0, 32.0]], RIGHT, false)?;
    let trans = rays(&[[16.0, 96.0]], &[[240.0, 96.0]], RIGHT, false)?;
    for page in [&terminal, &opposite, &trans] {
        let before = (*page).clone();
        let graph = graph_page_scene(320, 320, page)?;
        bundle(
            &graph,
            [
                ([96.0, 128.0], [224.0, 128.0]),
                ([96.0, 131.0], [224.0, 131.0]),
            ],
            1.0,
        )?;
        horizontal_pixels(
            &cpu_scene(graph.scene())?,
            112..208,
            124..136,
            &[(96.0, 224.0, 128.0, 1.0), (96.0, 224.0, 131.0, 1.0)],
        )?;
        assert_eq!(page, &before);
    }
    gpu_matches(&graph_page_scene(320, 320, &terminal)?)?;
    Ok(())
}

#[test]
fn left_mirror_reversal_and_quarter_turn_keep_the_authored_side_and_shortened_hits() -> TestResult {
    let right = rays(&[[112.0, 160.0]], &[[144.0, 160.0]], RIGHT, false)?;
    let reversed = rays(&[[112.0, 160.0]], &[[144.0, 160.0]], LEFT, true)?;
    let a = graph_page_scene(320, 320, &right)?;
    let b = graph_page_scene(320, 320, &reversed)?;
    bundle(
        &b,
        [
            ([224.0, 128.0], [96.0, 128.0]),
            ([218.0, 131.0], [102.0, 131.0]),
        ],
        1.0,
    )?;
    assert_eq!(cpu_scene(a.scene())?, cpu_scene(b.scene())?);
    let mirrored = rays(&[[112.0, 32.0]], &[[144.0, 32.0]], LEFT, false)?;
    bundle(
        &graph_page_scene(320, 320, &mirrored)?,
        [
            ([96.0, 128.0], [224.0, 128.0]),
            ([102.0, 125.0], [218.0, 125.0]),
        ],
        1.0,
    )?;
    let rotated = drawing(
        &[
            (29, [160.0, 64.0]),
            (11, [160.0, 192.0]),
            (7, [96.0, 64.0]),
            (53, [96.0, 192.0]),
        ],
        &[
            (41, 29, 11, DOUBLE, RIGHT),
            (42, 29, 7, SINGLE, UNSPECIFIED),
            (43, 11, 53, SINGLE, UNSPECIFIED),
        ],
        256.0,
    )?;
    let before = rotated.clone();
    let graph = graph_page_scene(320, 320, &rotated)?;
    bundle(
        &graph,
        [
            ([192.0, 96.0], [192.0, 224.0]),
            ([189.0, 99.0], [189.0, 221.0]),
        ],
        1.0,
    )?;
    hit(&graph, [189.0, 160.0], Some(41))?;
    hit(&graph, [189.0, 97.5], None)?;
    hit(&graph, [192.0, 97.5], Some(41))?;
    assert_eq!(rotated, before);
    Ok(())
}

#[test]
fn nearest_same_side_ray_wins_independently_of_atom_and_bond_array_order() -> TestResult {
    let root = 3.0_f64.sqrt();
    let points = vec![
        (29, [64.0, 96.0]),
        (11, [192.0, 96.0]),
        (7, [64.0, 160.0]),
        (53, [112.0, 160.0]),
        (71, [224.0, 96.0 + 32.0 * root]),
    ];
    let edges = vec![
        (41, 29, 11, DOUBLE, RIGHT),
        (42, 29, 7, SINGLE, UNSPECIFIED),
        (43, 29, 53, SINGLE, UNSPECIFIED),
        (44, 11, 71, SINGLE, UNSPECIFIED),
    ];
    let page = drawing(&points, &edges, 256.0)?;
    let before = page.clone();
    let mut reverse_points = points;
    reverse_points.reverse();
    let mut reverse_edges = edges;
    reverse_edges.reverse();
    let reversed = drawing(&reverse_points, &reverse_edges, 256.0)?;
    let reverse_before = reversed.clone();
    let a = graph_page_scene(320, 320, &page)?;
    let b = graph_page_scene(320, 320, &reversed)?;
    for graph in [&a, &b] {
        bundle(
            graph,
            [
                ([96.0, 128.0], [224.0, 128.0]),
                ([102.0, 131.0], [224.0 - root, 131.0]),
            ],
            1.0,
        )?;
    }
    assert_eq!(cpu_scene(a.scene())?, cpu_scene(b.scene())?);
    assert_eq!(page, before);
    assert_eq!(reversed, reverse_before);
    Ok(())
}

#[test]
fn width_three_natural_axes_join_but_offset_offset_overlap_still_refuses() -> TestResult {
    let root = 3.0_f64.sqrt();
    let points = [
        (29, [64.0, 96.0]),
        (11, [192.0, 96.0]),
        (7, [32.0, 96.0 + 32.0 * root]),
    ];
    let page = drawing(
        &points,
        &[
            (41, 29, 11, DOUBLE, RIGHT),
            (42, 29, 7, SINGLE, UNSPECIFIED),
        ],
        256.0,
    )?;
    let before = page.clone();
    let graph = graph_page_scene(832, 832, &page)?;
    bundle(
        &graph,
        [
            ([224.0, 320.0], [608.0, 320.0]),
            ([224.0 + 3.0 * root, 329.0], [608.0, 329.0]),
        ],
        3.0,
    )?;
    let frame = gpu_matches(&graph)?;
    // Pixel224,321 is inside both authored axes, outside the old anchor's rows319/320.
    assert_eq!(pixel(&frame, 224, 321)?, &[0, 0, 0, 255]);
    assert_eq!(page, before);
    let colliding = drawing(
        &points,
        &[(41, 29, 11, DOUBLE, RIGHT), (42, 29, 7, DOUBLE, LEFT)],
        256.0,
    )?;
    let before = colliding.clone();
    // Both offset caps meet at(224+3sqrt3,329); center(229.5,329.5) lies inside both
    // width3 rectangles, away from the shared authored atom. Neither is an axis role.
    collision(graph_page_scene(832, 832, &colliding))?;
    assert_eq!(colliding, before);
    // An upper centered component is not an authored axis merely because it is index0.
    // The acute Single crosses its real ink away from the authored atom's anchor mask.
    for (order, position) in [
        (DOUBLE, DoublePosition::Center),
        (BondOrder::Triple, UNSPECIFIED),
    ] {
        let legacy = drawing(
            &[(29, [64.0, 96.0]), (11, [192.0, 96.0]), (7, [112.0, 32.0])],
            &[
                (41, 29, 11, order, position),
                (42, 29, 7, SINGLE, UNSPECIFIED),
            ],
            256.0,
        )?;
        let before = legacy.clone();
        collision(graph_page_scene(832, 832, &legacy))?;
        assert_eq!(legacy, before);
    }
    Ok(())
}

#[test]
fn actual_offset_ink_and_common_corridor_are_protected_without_reserving_the_empty_side()
-> TestResult {
    let base = [(29, [64.0, 96.0]), (11, [192.0, 96.0])];
    let mut points = base.to_vec();
    points.extend([(7, [128.0, 98.0]), (53, [128.0, 104.0])]);
    let edges = [
        (41, 29, 11, DOUBLE, RIGHT),
        (99, 7, 53, SINGLE, UNSPECIFIED),
    ];
    let offset = drawing(&points, &edges, 256.0)?;
    let before = offset.clone();
    // Actual secondary covers y130.5; old centered width4 envelope stops at y130.
    collision(graph_page_scene(320, 320, &offset))?;
    assert_eq!(offset, before);
    let mut points = base.to_vec();
    points.extend([(7, [118.0, 97.5]), (53, [138.0, 97.5])]);
    let corridor = drawing(&points, &edges, 256.0)?;
    let before = corridor.clone();
    // This Single lies only in the real gap at y129.5, between y128 and y131 axes.
    collision(graph_page_scene(320, 320, &corridor))?;
    assert_eq!(corridor, before);
    let mut points = base.to_vec();
    points.extend([(7, [118.0, 94.0]), (53, [138.0, 94.0])]);
    let empty_side = drawing(&points, &edges, 256.0)?;
    let before = empty_side.clone();
    let graph = graph_page_scene(320, 320, &empty_side)?;
    bundle(
        &graph,
        [
            ([96.0, 128.0], [224.0, 128.0]),
            ([96.0, 131.0], [224.0, 131.0]),
        ],
        1.0,
    )?;
    horizontal_pixels(
        &cpu_scene(graph.scene())?,
        144..176,
        123..135,
        &[
            (96.0, 224.0, 128.0, 1.0),
            (96.0, 224.0, 131.0, 1.0),
            (150.0, 170.0, 126.0, 1.0),
        ],
    )?;
    assert_eq!(empty_side, before);
    Ok(())
}

#[test]
fn shortened_secondary_tail_is_empty_but_the_primary_tail_remains_real_ink() -> TestResult {
    let fixed = [
        (29, [32.0, 64.0]),
        (11, [96.0, 64.0]),
        (7, [32.0, 112.0]),
        (53, [96.0, 112.0]),
    ];
    let edges = [
        (41, 29, 11, DOUBLE, RIGHT),
        (42, 29, 7, SINGLE, UNSPECIFIED),
        (43, 11, 53, SINGLE, UNSPECIFIED),
        (99, 71, 83, SINGLE, UNSPECIFIED),
    ];
    let mut points = fixed.to_vec();
    points.extend([(71, [33.5, 66.0]), (83, [33.5, 69.0])]);
    let empty = drawing(&points, &edges, 256.0)?;
    let before = empty.clone();
    let graph = graph_page_scene(576, 576, &empty)?;
    bundle(
        &graph,
        [
            ([96.0, 160.0], [224.0, 160.0]),
            ([102.0, 166.0], [218.0, 166.0]),
        ],
        2.0,
    )?;
    hit(&graph, [99.0, 166.0], Some(99))?;
    hit(&graph, [104.0, 166.0], Some(41))?;
    assert_eq!(pixel(&cpu_scene(graph.scene())?, 99, 166)?, &[0, 0, 0, 255]);
    assert_eq!(empty, before);
    let mut points = fixed.to_vec();
    points.extend([(71, [33.5, 63.0]), (83, [33.5, 65.0])]);
    let blocked = drawing(&points, &edges, 256.0)?;
    let before = blocked.clone();
    collision(graph_page_scene(576, 576, &blocked))?;
    assert_eq!(blocked, before);
    Ok(())
}

#[test]
fn common_span_gap_and_component_pixels_are_required_without_a_blanket_width_rule() -> TestResult {
    let edges = [
        (41, 29, 11, DOUBLE, RIGHT),
        (42, 29, 7, SINGLE, UNSPECIFIED),
        (43, 11, 53, SINGLE, UNSPECIFIED),
    ];
    // At320px/page512, w=.5,s=1.5; outgoing ray(27.2,1.65) has norm27.25.
    // Its exact unit(544/545,33/545) yields trim49.5 at both ends: common x129.5..130.5.
    let missing_gap = drawing(
        &[
            (29, [96.0, 136.5]),
            (11, [296.0, 136.5]),
            (7, [150.4, 139.8]),
            (53, [241.6, 139.8]),
        ],
        &edges,
        512.0,
    )?;
    let before = missing_gap.clone();
    // Axis y100.25 and inset y101.75 each have ink, but only adjacent rows100/101.
    // No common-span white gap exists. Averaged x104.75..155.25 would falsely find one.
    scale_refusal(graph_page_scene(320, 320, &missing_gap))?;
    assert_eq!(missing_gap, before);
    let visible = drawing(
        &[
            (29, [96.0, 137.5]),
            (11, [296.0, 137.5]),
            (7, [150.4, 140.8]),
            (53, [241.6, 140.8]),
        ],
        &edges,
        512.0,
    )?;
    let before = visible.clone();
    let graph = graph_page_scene(320, 320, &visible)?;
    bundle(
        &graph,
        [
            ([80.0, 100.75], [180.0, 100.75]),
            ([129.5, 102.25], [130.5, 102.25]),
        ],
        0.5,
    )?;
    let cpu = cpu_scene(graph.scene())?;
    assert_eq!(pixel(&cpu, 129, 100)?, &[0, 0, 0, 255]);
    assert_eq!(pixel(&cpu, 129, 101)?, &[255, 255, 255, 255]);
    assert_eq!(pixel(&cpu, 129, 102)?, &[0, 0, 0, 255]);
    hit(&graph, [130.0, 102.25], Some(41))?;
    assert_eq!(visible, before);
    let no_component = drawing(
        &[(29, [96.0, 137.0]), (11, [296.0, 137.0])],
        &[(41, 29, 11, DOUBLE, RIGHT)],
        512.0,
    )?;
    // Axis y100.5 has a row, but inset y102.0 width.5 has none.
    scale_refusal(graph_page_scene(320, 320, &no_component))?;
    Ok(())
}

#[test]
fn physical_fractional_origin_and_signed_camera_project_the_actual_trimmed_components() -> TestResult
{
    let page = rays(&[[64.0, 160.0]], &[[192.0, 160.0]], RIGHT, false)?;
    let before = page.clone();
    let logical = PageView::fit(320, 320, page.size())?;
    let physical = logical.at_display_scale(2.0, [0.25, 0.75])?;
    let graph = physical.graph_scene(&page, &Cancellation::default())?;
    assert_eq!([graph.scene().width, graph.scene().height], [641, 641]);
    bundle(
        &graph,
        [
            ([192.25, 256.75], [448.25, 256.75]),
            ([198.25, 262.75], [442.25, 262.75]),
        ],
        2.0,
    )?;
    let frame = gpu_matches(&graph)?;
    horizontal_pixels(
        &frame,
        230..400,
        252..268,
        &[(192.25, 448.25, 256.75, 2.0), (198.25, 442.25, 262.75, 2.0)],
    )?;
    hit(&graph, [320.25, 262.75], Some(41))?;
    hit(&graph, [195.0, 262.75], None)?;
    let view = PageView::zoom(320, 320, page.size(), 100.0, Point2::new(160.0, 128.0)?)?;
    let graph = view.graph_scene(&page, &Cancellation::default())?;
    bundle(
        &graph,
        [
            ([64.0, 128.0], [192.0, 128.0]),
            ([67.0, 131.0], [189.0, 131.0]),
        ],
        1.0,
    )?;
    hit(&graph, [128.0, 131.0], Some(41))?;
    hit(&graph, [65.5, 131.0], None)?;
    assert_eq!(page, before);
    Ok(())
}

#[test]
fn zero_codirected_and_exhausted_adjoining_geometry_refuses_atomically() -> TestResult {
    for page in [
        rays(&[[64.0, 96.0]], &[], RIGHT, false)?, // Distinct source atom, zero outgoing vector.
        rays(&[[112.0, 96.0]], &[], RIGHT, false)?, // Exact co-directed overlapping axis.
        drawing(
            &[
                (29, [64.0, 96.0]),
                (11, [100.0, 96.0]),
                (7, [88.0, 103.0]),
                (53, [76.0, 103.0]),
            ],
            &[
                (41, 29, 11, DOUBLE, RIGHT),
                (42, 29, 7, SINGLE, UNSPECIFIED),
                (43, 11, 53, SINGLE, UNSPECIFIED),
            ],
            256.0,
        )?,
    ] {
        // Last rays have unit(24/25,7/25): a=b=7s=21, exceeding length36.
        let before = page.clone();
        assert!(graph_page_scene(320, 320, &page).is_err());
        assert_eq!(page, before);
    }
    Ok(())
}

#[test]
fn cancellation_and_existing_hit_budget_precede_inset_topology_work() -> TestResult {
    let page = rays(&[], &[], RIGHT, false)?;
    let before = page.clone();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        graph_page_scene_with_cancel(320, 320, &page, &cancel),
        Err(Error::Cancelled)
    ));
    assert_eq!(page, before);
    let points = (1_u32..=4096)
        .map(|id| (id, [20.0 + f64::from(id % 200), 20.0 + f64::from(id / 200)]))
        .collect::<Vec<_>>();
    let large = drawing(&points, &[(41, 1, 2, DOUBLE, RIGHT)], 256.0)?;
    let before = large.clone();
    //4096 atoms +1 bond exceeds the unchanged4096 hit-region cap before isolated-H refusal.
    assert!(matches!(
        graph_page_scene(320, 320, &large),
        Err(Error::SceneTooComplex)
    ));
    assert!(matches!(
        graph_page_scene_with_cancel(320, 320, &large, &cancel),
        Err(Error::Cancelled)
    ));
    assert_eq!(large, before);
    Ok(())
}
