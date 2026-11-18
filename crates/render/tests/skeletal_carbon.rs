//! Existing public API: the DATA baseline refuses every specified flag at runtime.
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, Element, Fragment, FragmentId,
    PageId, PageSize, Point2,
};
use bondline_render::{HitTarget, Primitive, Rectangle, graph_page_scene};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn regular_single_hexagon_keeps_h2_and_atom_hits_without_any_carbon_glyph_or_mask() -> TestResult {
    let half = 32.0 * 3.0_f64.sqrt();
    let points = [
        [96.0, 128.0],
        [96.0, 192.0],
        [96.0 + half, 224.0],
        [96.0 + 2.0 * half, 192.0],
        [96.0 + 2.0 * half, 128.0],
        [96.0 + half, 96.0],
    ];
    let atoms = points
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            let mut atom = Atom::new(
                AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(p[0], p[1])?),
            );
            atom.explicit_hydrogens = Some(2);
            atom.show_non_terminal_carbon_labels = Some(false);
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = (1_u32..=6)
        .map(|id| {
            Ok(Bond {
                id: BondId::new(id).ok_or("bond")?,
                begin: AtomId::new(id).ok_or("begin")?,
                end: AtomId::new(if id == 6 { 1 } else { id + 1 }).ok_or("end")?,
                order: BondOrder::Single,
                double_position: DoublePosition::Unspecified,
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
    let graph = graph_page_scene(576, 576, page)?;
    let rectangles = graph
        .scene()
        .primitives
        .iter()
        .filter_map(|p| match p {
            Primitive::Rectangle(r) => Some(r.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rectangles,
        vec![
            Rectangle {
                x: 35,
                y: 36,
                width: 512,
                height: 512,
                color: [210, 210, 210, 255]
            },
            Rectangle {
                x: 32,
                y: 32,
                width: 512,
                height: 512,
                color: [196, 196, 196, 255]
            },
            Rectangle {
                x: 33,
                y: 33,
                width: 510,
                height: 510,
                color: [255, 255, 255, 255]
            },
        ]
    );
    assert_eq!(graph.scene().primitives.len(), 9);
    assert_eq!(graph.label_em_px(), None);
    for (i, p) in points.into_iter().enumerate() {
        assert_eq!(
            graph.hit_test(
                [(32.0 + 2.0 * p[0]) as f32, (32.0 + 2.0 * p[1]) as f32],
                0.0
            )?,
            Some(HitTarget::Atom {
                fragment_id: FragmentId::new(17).ok_or("fragment")?,
                atom_id: AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?
            })
        );
    }
    assert_eq!(document, before);
    Ok(())
}

// Exact first2,529-byte public runtime declaration is retained above.
use bondline_model::Page;
use bondline_render::{
    Cancellation, Error, Frame, GraphScene, PageView, Stroke, cpu_scene, gpu_scene,
    graph_page_scene_with_display_hydrogens,
};
use std::collections::BTreeMap;

const SINGLE: BondOrder = BondOrder::Single;
const DOUBLE: BondOrder = BondOrder::Double;
const TRIPLE: BondOrder = BondOrder::Triple;
const U: DoublePosition = DoublePosition::Unspecified;
const L: DoublePosition = DoublePosition::Left;
const R: DoublePosition = DoublePosition::Right;
const COORD_EPS: f64 = 1.0 / 4096.0; // bounded f64->f32 geometry; never a pixel/error-policy tolerance.
type AtomSpec = (u32, u8, [f64; 2], Option<u16>, Option<bool>);
type Edge = (u32, u32, u32, BondOrder, DoublePosition);

fn drawing(atoms: &[AtomSpec], edges: &[Edge], extent: [f64; 2]) -> TestResult<Page> {
    let atoms = atoms
        .iter()
        .map(|(id, z, p, h, flag)| {
            let mut atom = Atom::new(
                AtomId::new(*id).ok_or("atom")?,
                Element::try_from(*z)?,
                Some(Point2::new(p[0], p[1])?),
            );
            atom.explicit_hydrogens = *h;
            atom.show_non_terminal_carbon_labels = *flag;
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .iter()
        .map(|(id, a, b, order, position)| {
            Ok(Bond {
                id: BondId::new(*id).ok_or("bond")?,
                begin: AtomId::new(*a).ok_or("begin")?,
                end: AtomId::new(*b).ok_or("end")?,
                order: *order,
                double_position: *position,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut doc = Document::default();
    let pid = PageId::new(1).ok_or("page")?;
    doc.set_page_size(pid, PageSize::new(extent[0], extent[1])?)?;
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
fn change(page: &Page, edit: impl FnOnce(&mut Vec<Atom>, &mut Vec<Bond>)) -> TestResult<Page> {
    let fragment = page.fragments().first().ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    let mut bonds = fragment.bonds().to_vec();
    edit(&mut atoms, &mut bonds);
    let mut doc = Document::default();
    let pid = PageId::new(1).ok_or("page")?;
    doc.set_page_size(pid, page.size())?;
    doc.append_fragments(pid, vec![Fragment::new(fragment.id(), atoms, bonds)?])?;
    Ok(doc.pages().first().ok_or("page")?.clone())
}
fn junction(
    orders: &[(BondOrder, DoublePosition)],
    h: Option<u16>,
    flag: Option<bool>,
) -> TestResult<Page> {
    let root = 3.0_f64.sqrt();
    let points = match orders.len() {
        2 => vec![[64.0, 96.0], [192.0, 96.0]],
        3 => vec![
            [64.0, 128.0],
            [160.0, 128.0 - 32.0 * root],
            [160.0, 128.0 + 32.0 * root],
        ],
        4 => vec![[64.0, 128.0], [192.0, 128.0], [128.0, 64.0], [128.0, 192.0]],
        _ => return Err("test junction degree".into()),
    };
    let mut atoms = vec![(29, 6, [128.0, 128.0], h, flag)];
    let mut edges = Vec::new();
    for (i, (point, (order, position))) in points.into_iter().zip(orders).enumerate() {
        let id = 100 + u32::try_from(i)? * 7;
        atoms.push((id, 6, point, None, None));
        edges.push((41 + u32::try_from(i)?, 29, id, *order, *position));
    }
    drawing(&atoms, &edges, [256.0, 256.0])
}
fn paper() -> Vec<Rectangle> {
    vec![
        Rectangle {
            x: 35,
            y: 36,
            width: 512,
            height: 512,
            color: [210, 210, 210, 255],
        },
        Rectangle {
            x: 32,
            y: 32,
            width: 512,
            height: 512,
            color: [196, 196, 196, 255],
        },
        Rectangle {
            x: 33,
            y: 33,
            width: 510,
            height: 510,
            color: [255, 255, 255, 255],
        },
    ]
}
fn hidden(graph: &GraphScene, strokes: usize) {
    assert_eq!(
        graph
            .scene()
            .primitives
            .iter()
            .filter_map(|p| match p {
                Primitive::Rectangle(r) => Some(r.clone()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        paper()
    );
    assert_eq!(graph.scene().primitives.len(), 3 + strokes);
    assert_eq!(graph.label_em_px(), None);
}
fn shown(graph: &GraphScene) {
    assert!(graph.label_em_px().is_some());
    assert!(
        graph
            .scene()
            .primitives
            .iter()
            .any(|p| matches!(p,Primitive::Rectangle(r) if r.color==[0,0,0,255]))
    );
}
fn center_hit(graph: &GraphScene, point: [f32; 2]) -> TestResult {
    assert_eq!(
        graph.hit_test(point, 0.0)?,
        Some(HitTarget::Atom {
            fragment_id: FragmentId::new(17).ok_or("fragment")?,
            atom_id: AtomId::new(29).ok_or("atom")?,
        })
    );
    Ok(())
}
fn unsupported(result: Result<GraphScene, Error>) -> TestResult {
    let error = result
        .err()
        .ok_or("unknown/invalid skeletal proof accepted")?;
    assert!(
        matches!(error, Error::UnsupportedChemicalContent),
        "{error}"
    );
    Ok(())
}
fn counts(values: &[(u32, u16)]) -> TestResult<BTreeMap<AtomId, u16>> {
    values
        .iter()
        .map(|(id, h)| Ok((AtomId::new(*id).ok_or("atom")?, *h)))
        .collect()
}
fn rgba(frame: &Frame, x: u32, y: u32) -> TestResult<&[u8]> {
    let at = usize::try_from((u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4)?;
    Ok(frame.rgba.get(at..at + 4).ok_or("RGBA")?)
}

#[test]
fn bent_and_weighted_c4_junctions_hide_glyphs_and_masks_but_keep_atom_hits_and_literal_data()
-> TestResult {
    for (orders, h, strokes) in [
        (vec![(SINGLE, U), (SINGLE, U)], 2, 2),
        (vec![(SINGLE, U), (DOUBLE, R)], 1, 3),
        (vec![(SINGLE, U), (TRIPLE, U)], 0, 4),
        (vec![(SINGLE, U); 3], 1, 3),
        (vec![(SINGLE, U); 4], 0, 4),
    ] {
        let page = junction(&orders, Some(h), Some(false))?;
        let before = page.clone();
        let graph = graph_page_scene(576, 576, &page)?;
        hidden(&graph, strokes);
        center_hit(&graph, [288.0, 288.0])?;
        let true_page = junction(&orders, Some(h), Some(true))?;
        let legacy = junction(&orders, Some(h), None)?;
        let printed = graph_page_scene(576, 576, &true_page)?;
        shown(&printed);
        assert_eq!(
            printed.scene(),
            graph_page_scene(576, 576, &legacy)?.scene()
        );
        assert_eq!(page, before);
        assert_eq!(page.fragments()[0].atoms()[0].explicit_hydrogens, Some(h));
    }
    Ok(())
}

#[test]
fn literal_or_complete_resolved_h_is_required_and_coverage_alone_does_not_prove_c4() -> TestResult {
    let orders = [(SINGLE, U), (SINGLE, U)];
    let page = junction(&orders, None, Some(false))?;
    let before = page.clone();
    unsupported(graph_page_scene(576, 576, &page))?;
    let valid = counts(&[(29, 2), (100, 3), (107, 3)])?;
    let graph =
        graph_page_scene_with_display_hydrogens(576, 576, &page, &valid, &Cancellation::default())?;
    // Only terminal labels remain: the resolved central H2 was not written into the model.
    center_hit(&graph, [288.0, 288.0])?;
    assert!(!graph.scene().primitives.iter().any(|p|matches!(p,Primitive::Rectangle(r)
        if r.x<306 && r.x+r.width>270 && r.y<306 && r.y+r.height>270 && r.width<256 && r.height<256)));
    let view = PageView::fit(576, 576, page.size())?;
    assert_eq!(
        view.graph_scene_with_display_hydrogens(&page, &valid, &Cancellation::default())?
            .scene(),
        graph.scene()
    );
    for invalid in [
        counts(&[(29, 3), (100, 3), (107, 3)])?,
        counts(&[(100, 3), (107, 3)])?,
        counts(&[(29, 2), (100, 3), (107, 3), (900, 0)])?,
    ] {
        unsupported(graph_page_scene_with_display_hydrogens(
            576,
            576,
            &page,
            &invalid,
            &Cancellation::default(),
        ))?;
    }
    for h in [0, 1, 3, u16::MAX] {
        unsupported(graph_page_scene(
            576,
            576,
            &junction(&orders, Some(h), Some(false))?,
        ))?;
    }
    // Printed literals intentionally have no new C4 restriction; false alone needs W+h=4.
    shown(&graph_page_scene(
        576,
        576,
        &junction(&orders, Some(3), Some(true))?,
    )?);
    unsupported(graph_page_scene(
        576,
        576,
        &junction(&orders, None, Some(true))?,
    ))?;
    assert_eq!(page, before);
    assert_eq!(page.fragments()[0].atoms()[0].explicit_hydrogens, None);
    Ok(())
}

#[test]
fn terminal_noncarbon_and_isolated_flags_are_dormant_and_degree_is_not_bond_weight() -> TestResult {
    for (atoms, edges) in [
        (
            vec![
                (29, 6, [64.0, 128.0], Some(1), None),
                (11, 6, [192.0, 128.0], Some(1), None),
            ],
            vec![(41, 29, 11, TRIPLE, U)],
        ),
        (
            vec![
                (29, 8, [128.0, 128.0], Some(0), None),
                (11, 6, [64.0, 96.0], None, None),
                (7, 6, [192.0, 96.0], None, None),
            ],
            vec![(41, 29, 11, SINGLE, U), (42, 29, 7, SINGLE, U)],
        ),
        (vec![(29, 6, [128.0, 128.0], Some(4), None)], vec![]),
    ] {
        let legacy = drawing(&atoms, &edges, [256.0, 256.0])?;
        let expected = graph_page_scene(576, 576, &legacy)?;
        shown(&expected);
        for flag in [false, true] {
            let flagged = change(&legacy, |atoms, _| {
                for atom in atoms {
                    atom.show_non_terminal_carbon_labels = Some(flag);
                }
            })?;
            let before = flagged.clone();
            assert_eq!(
                graph_page_scene(576, 576, &flagged)?.scene(),
                expected.scene()
            );
            assert_eq!(flagged, before);
        }
    }
    Ok(())
}

fn hexagon(kekule: bool) -> TestResult<Page> {
    let half = 32.0 * 3.0_f64.sqrt();
    let points = [
        [96.0, 128.0],
        [96.0, 192.0],
        [96.0 + half, 224.0],
        [96.0 + 2.0 * half, 192.0],
        [96.0 + 2.0 * half, 128.0],
        [96.0 + half, 96.0],
    ];
    let atoms = points
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            Ok((
                u32::try_from(i + 1)?,
                6,
                p,
                Some(if kekule { 1 } else { 2 }),
                Some(false),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let edges = (1_u32..=6)
        .map(|id| {
            (
                id,
                id,
                if id == 6 { 1 } else { id + 1 },
                if kekule && id % 2 == 0 {
                    DOUBLE
                } else {
                    SINGLE
                },
                if kekule && id % 2 == 0 { L } else { U },
            )
        })
        .collect::<Vec<_>>();
    drawing(&atoms, &edges, [256.0, 256.0])
}
fn geometry(actual: &Stroke, begin: [f64; 2], end: [f64; 2], width: f32) {
    for (a, b) in actual
        .begin
        .into_iter()
        .chain(actual.end)
        .zip(begin.into_iter().chain(end))
    {
        assert!((f64::from(a) - b).abs() <= COORD_EPS, "coordinate{a} vs{b}");
    }
    assert_eq!(actual.width, width);
    assert_eq!(actual.color, [0, 0, 0, 255]);
}
fn strip(frame: &Frame, a: [f64; 2], b: [f64; 2], width: f64, color: &[u8]) -> TestResult {
    let v = [b[0] - a[0], b[1] - a[1]];
    let length = v[0].hypot(v[1]);
    let mut checked = 0;
    for y in 0..frame.height {
        for x in 0..frame.width {
            let d = [f64::from(x) + 0.5 - a[0], f64::from(y) + 0.5 - a[1]];
            let along = (d[0] * v[0] + d[1] * v[1]) / (length * length);
            let across = (d[0] * v[1] - d[1] * v[0]).abs() / length;
            if (0.3..=0.7).contains(&along) && across <= width / 4.0 {
                assert_eq!(rgba(frame, x, y)?, color, "analytic strip{x},{y}");
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "analytic strip has no sampled centers");
    Ok(())
}

#[test]
fn an_all_hidden_kekule_hexagon_keeps_nine_exact_inset_components_gaps_hits_and_cpu_gpu()
-> TestResult {
    let page = hexagon(true)?;
    let before = page.clone();
    let graph = graph_page_scene(576, 576, &page)?;
    hidden(&graph, 9);
    let root = 3.0_f64.sqrt();
    let x = 224.0 + 64.0 * root;
    let right = 224.0 + 128.0 * root;
    let points = [
        [224.0, 288.0],
        [224.0, 416.0],
        [x, 480.0],
        [right, 416.0],
        [right, 288.0],
        [x, 224.0],
    ];
    let secondary = [
        ([230.0, 416.0 - 2.0 * root], [x, 480.0 - 4.0 * root]),
        (
            [right - 6.0, 416.0 - 2.0 * root],
            [right - 6.0, 288.0 + 2.0 * root],
        ),
        ([x, 224.0 + 4.0 * root], [230.0, 288.0 + 2.0 * root]),
    ];
    let frame = cpu_scene(graph.scene())?;
    for i in 0..6 {
        let id = BondId::new(u32::try_from(i + 1)?).ok_or("bond")?;
        let actual = graph
            .bond_strokes(FragmentId::new(17).ok_or("fragment")?, id)
            .collect::<Vec<_>>();
        let (a, b) = (points[i], points[(i + 1) % 6]);
        assert_eq!(actual.len(), if i % 2 == 1 { 2 } else { 1 });
        geometry(actual[0], a, b, 2.0);
        strip(&frame, a, b, 2.0, &[0, 0, 0, 255])?;
        if i % 2 == 1 {
            let (c, d) = secondary[i / 2];
            geometry(actual[1], c, d, 2.0);
            strip(&frame, c, d, 2.0, &[0, 0, 0, 255])?;
            // Mid-half lies within the true common span; no tail can qualify a white gap.
            strip(
                &frame,
                [(a[0] + c[0]) / 2.0, (a[1] + c[1]) / 2.0],
                [(b[0] + d[0]) / 2.0, (b[1] + d[1]) / 2.0],
                2.0,
                &[255, 255, 255, 255],
            )?;
            assert_eq!(
                graph.hit_test(
                    [((c[0] + d[0]) / 2.0) as f32, ((c[1] + d[1]) / 2.0) as f32],
                    0.0
                )?,
                Some(HitTarget::Bond {
                    fragment_id: FragmentId::new(17).ok_or("fragment")?,
                    bond_id: id
                })
            );
        }
    }
    assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, frame);
    assert_eq!(page, before);
    assert!(
        page.fragments()[0]
            .atoms()
            .iter()
            .all(|atom| atom.explicit_hydrogens == Some(1))
    );
    Ok(())
}

// Supplement declared after checkpoint-v1 was sealed, using only public APIs.
fn degree_two(
    point: [f64; 2],
    other: [f64; 2],
    orders: [(BondOrder, DoublePosition); 2],
    h: u16,
) -> TestResult<Page> {
    drawing(
        &[
            (29, 6, point, Some(h), Some(false)),
            (100, 6, [point[0] + 64.0, point[1]], None, None),
            (107, 6, other, None, None),
        ],
        &[
            (41, 29, 100, orders[0].0, orders[0].1),
            (42, 29, 107, orders[1].0, orders[1].1),
        ],
        [256.0, 256.0],
    )
}
fn has_no_glyphs_or_masks(graph: &GraphScene, components: usize) {
    assert_eq!(graph.label_em_px(), None);
    assert_eq!(
        graph
            .scene()
            .primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Rectangle(_)))
            .count(),
        3
    );
    assert_eq!(
        graph
            .scene()
            .primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Stroke(_)))
            .count(),
        components
    );
}
fn same_label_decision(page: &Page, must_show: bool) -> TestResult {
    let before = page.clone();
    let printed = change(page, |atoms, _| {
        atoms[0].show_non_terminal_carbon_labels = Some(true);
    })?;
    let graph = graph_page_scene(576, 576, page)?;
    if must_show {
        shown(&graph);
        assert_eq!(graph.scene(), graph_page_scene(576, 576, &printed)?.scene());
    } else {
        has_no_glyphs_or_masks(&graph, 2);
    }
    for density in [1.0, 1.5, 2.0] {
        let logical = PageView::zoom(576, 576, page.size(), 150.0, Point2::new(128.0, 128.0)?)?;
        let view = logical.at_display_scale(density, [0.25, 0.75])?;
        let actual = view.graph_scene(page, &Cancellation::default())?;
        if must_show {
            shown(&actual);
            assert_eq!(
                actual.scene(),
                view.graph_scene(&printed, &Cancellation::default())?
                    .scene()
            );
        } else {
            has_no_glyphs_or_masks(&actual, 2);
        }
        let source = page.fragments()[0].atoms()[0].position.ok_or("position")?;
        center_hit(&actual, view.canvas_position(source)?)?;
    }
    assert_eq!(page, &before);
    Ok(())
}

#[test]
fn identical_ray_169_171_and_straight_decisions_survive_direction_array_camera_and_density_changes()
-> TestResult {
    for (degrees, must_show) in [
        (169.0_f64, false),
        (171.0, true),
        (180.0, true),
        (-171.0, true),
    ] {
        let angle = degrees.to_radians();
        let point = [128.0, 128.0];
        let page = degree_two(
            point,
            [128.0 + 64.0 * angle.cos(), 128.0 + 64.0 * angle.sin()],
            [(SINGLE, U); 2],
            2,
        )?;
        same_label_decision(&page, must_show)?;
        let reversed = change(&page, |atoms, bonds| {
            atoms.reverse();
            bonds.reverse();
            for bond in bonds {
                std::mem::swap(&mut bond.begin, &mut bond.end);
            }
        })?;
        // Reverse storage independently of the target's array position.
        let graph = graph_page_scene(576, 576, &reversed)?;
        if must_show {
            shown(&graph);
        } else {
            has_no_glyphs_or_masks(&graph, 2);
        }
        center_hit(&graph, [288.0, 288.0])?;
        assert_eq!(
            reversed.fragments()[0]
                .atoms()
                .iter()
                .find(|a| a.id.get() == 29)
                .ok_or("center")?
                .explicit_hydrogens,
            Some(2)
        );
    }
    Ok(())
}

#[test]
fn the_exact_authored_f64_threshold_is_inclusive_before_f32_projection() -> TestResult {
    const BOUNDARY: f64 = -0.984807753012208;
    const Y: f64 = 0.17364817766693058;
    // Exact source subtraction reconstructs these rays. The test explicitly checks
    // its floating-point boundary setup instead of assuming sin/cos after translation.
    let page = degree_two(
        [64.0, 0.0],
        [64.0 + 64.0 * BOUNDARY, 64.0 * Y],
        [(SINGLE, U); 2],
        2,
    )?;
    let atoms = page.fragments()[0].atoms();
    let p = atoms[0].position.ok_or("center")?;
    let a = atoms[1].position.ok_or("ray")?;
    let b = atoms[2].position.ok_or("ray")?;
    let u = [a.x_pt() - p.x_pt(), a.y_pt() - p.y_pt()];
    let v = [b.x_pt() - p.x_pt(), b.y_pt() - p.y_pt()];
    let dot = (u[0] / u[0].hypot(u[1])) * (v[0] / v[0].hypot(v[1]))
        + (u[1] / u[0].hypot(u[1])) * (v[1] / v[0].hypot(v[1]));
    assert_eq!(
        dot, BOUNDARY,
        "literal source fixture is not the locked exact boundary"
    );
    let graph = graph_page_scene(576, 576, &page)?;
    shown(&graph);
    let printed = change(&page, |atoms, _| {
        atoms[0].show_non_terminal_carbon_labels = Some(true);
    })?;
    assert_eq!(graph.scene(), graph_page_scene(576, 576, &printed)?.scene());
    center_hit(&graph, [160.0, 32.0])?;
    assert_eq!(page.fragments()[0].atoms()[0].explicit_hydrogens, Some(2));
    Ok(())
}

#[test]
fn every_two_double_center_keeps_its_literal_c_label_even_when_bent() -> TestResult {
    for (points, positions) in [
        ([[64.0, 128.0], [192.0, 128.0]], [U, U]),
        ([[64.0, 128.0], [128.0, 64.0]], [L, R]),
    ] {
        let page = drawing(
            &[
                (29, 6, [128.0, 128.0], Some(0), Some(false)),
                (100, 6, points[0], None, None),
                (107, 6, points[1], None, None),
            ],
            &[
                (41, 29, 100, DOUBLE, positions[0]),
                (42, 29, 107, DOUBLE, positions[1]),
            ],
            [256.0, 256.0],
        )?;
        let before = page.clone();
        let graph = graph_page_scene(576, 576, &page)?;
        shown(&graph);
        assert_eq!(
            graph
                .scene()
                .primitives
                .iter()
                .filter(|p| matches!(p, Primitive::Stroke(_)))
                .count(),
            4
        );
        for flag in [None, Some(true)] {
            let shown_page = change(&page, |atoms, _| {
                atoms[0].show_non_terminal_carbon_labels = flag;
            })?;
            assert_eq!(
                graph.scene(),
                graph_page_scene(576, 576, &shown_page)?.scene()
            );
        }
        center_hit(&graph, [288.0, 288.0])?;
        assert_eq!(page, before);
        assert_eq!(page.fragments()[0].atoms()[0].explicit_hydrogens, Some(0));
    }
    Ok(())
}

#[test]
fn physical_hydrogen_is_a_real_incident_bond_and_never_merged_with_inline_h2() -> TestResult {
    let page = drawing(
        &[
            (29, 6, [128.0, 128.0], Some(2), Some(false)),
            (100, 1, [64.0, 96.0], Some(0), None),
            (107, 6, [192.0, 96.0], None, None),
        ],
        &[(41, 29, 100, SINGLE, U), (42, 29, 107, SINGLE, U)],
        [256.0, 256.0],
    )?;
    let before = page.clone();
    let graph = graph_page_scene(576, 576, &page)?;
    shown(&graph);
    // The separate H glyph remains, while central carbon has no glyph or white mask.
    assert!(!graph.scene().primitives.iter().any(|p|matches!(p,Primitive::Rectangle(r)
        if r.x<306 && r.x+r.width>270 && r.y<306 && r.y+r.height>270 && r.width<256 && r.height<256)));
    center_hit(&graph, [288.0, 288.0])?;
    assert_eq!(
        graph.hit_test([160.0, 224.0], 0.0)?,
        Some(HitTarget::Atom {
            fragment_id: FragmentId::new(17).ok_or("fragment")?,
            atom_id: AtomId::new(100).ok_or("physical H")?,
        })
    );
    assert_eq!(page, before);
    assert_eq!(page.fragments()[0].atoms().len(), 3);
    assert_eq!(
        page.fragments()[0]
            .atoms()
            .iter()
            .map(|a| (a.element.get(), a.explicit_hydrogens))
            .collect::<Vec<_>>(),
        vec![(6, Some(2)), (1, Some(0)), (6, None)]
    );
    Ok(())
}

#[test]
fn suppression_retains_metadata_geometry_and_entry_cancellation_guards() -> TestResult {
    let page = junction(&[(SINGLE, U); 2], Some(2), Some(false))?;
    for altered in [
        change(&page, |atoms, _| {
            atoms[0].charge = 1;
        })?,
        change(&page, |atoms, _| {
            atoms[0].isotope = std::num::NonZeroU16::new(13);
        })?,
        change(&page, |atoms, _| {
            atoms[0].radical = bondline_model::Radical::Doublet;
        })?,
        change(&page, |atoms, _| {
            atoms[0].position = None;
        })?,
        change(&page, |_, bonds| {
            bonds[0].order = BondOrder::Aromatic;
        })?,
    ] {
        unsupported(graph_page_scene(576, 576, &altered))?;
    }
    let cancel = Cancellation::default();
    cancel.cancel();
    let map = counts(&[(29, 2), (100, 3), (107, 3)])?;
    assert!(matches!(
        bondline_render::graph_page_scene_with_cancel(576, 576, &page, &cancel),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        graph_page_scene_with_display_hydrogens(576, 576, &page, &map, &cancel),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        PageView::fit(576, 576, page.size())?.graph_scene(&page, &cancel),
        Err(Error::Cancelled)
    ));
    Ok(())
}
