use bondline_io::smiles::SmilesDocument;
use bondline_model::{BondOrder, PageSize, Point2, Radical};
use bondline_render::Cancellation;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected} within {tolerance}"
    );
}

#[test]
fn literal_cyclohexane_is_a_regular_closed_hexagon_without_source_changes() -> TestResult {
    let source = b"C1CCCCC1\r\n";
    let owner = SmilesDocument::parse(Arc::from(source.as_slice()))?;
    let before = owner.clone();
    let length = 64.0;
    let document = owner.layout_neutral_simple_ring(
        length,
        Point2::new(144.0, 144.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    assert_eq!(document.pages().len(), 1);
    let page = document.pages().first().ok_or("page")?;
    assert_eq!(page.id().get(), 1);
    assert_eq!(page.size(), PageSize::A4);
    assert_eq!(page.fragments().len(), 1);
    let fragment = page.fragments().first().ok_or("fragment")?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), 6);
    let mut points = Vec::new();
    for (index, atom) in fragment.atoms().iter().enumerate() {
        assert_eq!(atom.id.get(), u32::try_from(index + 1)?);
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.explicit_hydrogens, Some(2));
        assert_eq!(atom.isotope, None);
        assert_eq!(atom.charge, 0);
        assert_eq!(atom.radical, Radical::None);
        let p = atom.position.ok_or("position")?;
        points.push([p.x_pt(), p.y_pt()]);
    }
    assert_eq!(
        fragment
            .bonds()
            .iter()
            .map(|b| (b.id.get(), b.begin.get(), b.end.get(), b.order))
            .collect::<Vec<_>>(),
        [
            (1, 1, 2, BondOrder::Single),
            (2, 2, 3, BondOrder::Single),
            (3, 3, 4, BondOrder::Single),
            (4, 4, 5, BondOrder::Single),
            (5, 5, 6, BondOrder::Single),
            (6, 1, 6, BondOrder::Single),
        ]
    );
    for index in 0..6 {
        let a = points[index];
        let b = points[(index + 1) % 6];
        near((a[0] - b[0]).hypot(a[1] - b[1]), length, length * 1e-9);
        let previous = points[(index + 5) % 6];
        let u = [previous[0] - a[0], previous[1] - a[1]];
        let v = [b[0] - a[0], b[1] - a[1]];
        near((u[0] * v[0] + u[1] * v[1]) / (length * length), -0.5, 1e-9);
        let opposite = points[(index + 3) % 6];
        near(
            (opposite[0] - a[0]).hypot(opposite[1] - a[1]),
            2.0 * length,
            length * 1e-9,
        );
    }
    near(
        points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min),
        144.0,
        1e-9,
    );
    near(
        points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min),
        144.0,
        1e-9,
    );
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes(), source);
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
    assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
    Ok(())
}

use bondline_io::{StoredDocument, cdxml::write_document, smiles::LayoutError};
use bondline_model::BondOrder::{Double as D, Single as S, Triple as T};
use bondline_model::{Document, Fragment};
use bondline_render::{cpu_scene, graph_page_scene, graph_page_scene_with_display_hydrogens};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Write as _,
    process::{Command, Output, Stdio},
};

struct Example {
    source: &'static str,
    atoms: Vec<(u8, u16)>,
    bonds: Vec<(u32, u32, BondOrder)>,
    cycle: Vec<u32>,
    formula: &'static str,
}

fn examples() -> Vec<Example> {
    vec![
        Example {
            source: "CC1CCCCC1",
            atoms: vec![(6, 3), (6, 1), (6, 2), (6, 2), (6, 2), (6, 2), (6, 2)],
            bonds: vec![
                (1, 2, S),
                (2, 3, S),
                (3, 4, S),
                (4, 5, S),
                (5, 6, S),
                (6, 7, S),
                (2, 7, S),
            ],
            cycle: vec![2, 3, 4, 5, 6, 7],
            formula: "C7H14",
        },
        Example {
            source: "C=C1CCCCC1",
            atoms: vec![(6, 2), (6, 0), (6, 2), (6, 2), (6, 2), (6, 2), (6, 2)],
            bonds: vec![
                (1, 2, D),
                (2, 3, S),
                (3, 4, S),
                (4, 5, S),
                (5, 6, S),
                (6, 7, S),
                (2, 7, S),
            ],
            cycle: vec![2, 3, 4, 5, 6, 7],
            formula: "C7H12",
        },
        Example {
            source: "O=C1CCCCC1",
            atoms: vec![(8, 0), (6, 0), (6, 2), (6, 2), (6, 2), (6, 2), (6, 2)],
            bonds: vec![
                (1, 2, D),
                (2, 3, S),
                (3, 4, S),
                (4, 5, S),
                (5, 6, S),
                (6, 7, S),
                (2, 7, S),
            ],
            cycle: vec![2, 3, 4, 5, 6, 7],
            formula: "C6H10O",
        },
        Example {
            source: "CC(=O)C1CCCCC1",
            atoms: vec![
                (6, 3),
                (6, 0),
                (8, 0),
                (6, 1),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
            ],
            bonds: vec![
                (1, 2, S),
                (2, 3, D),
                (2, 4, S),
                (4, 5, S),
                (5, 6, S),
                (6, 7, S),
                (7, 8, S),
                (8, 9, S),
                (4, 9, S),
            ],
            cycle: vec![4, 5, 6, 7, 8, 9],
            formula: "C8H14O",
        },
        Example {
            source: "N#CC1CCCCC1",
            atoms: vec![
                (7, 0),
                (6, 0),
                (6, 1),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
            ],
            bonds: vec![
                (1, 2, T),
                (2, 3, S),
                (3, 4, S),
                (4, 5, S),
                (5, 6, S),
                (6, 7, S),
                (7, 8, S),
                (3, 8, S),
            ],
            cycle: vec![3, 4, 5, 6, 7, 8],
            formula: "C7H11N",
        },
        Example {
            source: "N1CCCCC1",
            atoms: vec![(7, 1), (6, 2), (6, 2), (6, 2), (6, 2), (6, 2)],
            bonds: vec![
                (1, 2, S),
                (2, 3, S),
                (3, 4, S),
                (4, 5, S),
                (5, 6, S),
                (1, 6, S),
            ],
            cycle: vec![1, 2, 3, 4, 5, 6],
            formula: "C5H11N",
        },
        Example {
            source: "C1COCCN1",
            atoms: vec![(6, 2), (6, 2), (8, 0), (6, 2), (6, 2), (7, 1)],
            bonds: vec![
                (1, 2, S),
                (2, 3, S),
                (3, 4, S),
                (4, 5, S),
                (5, 6, S),
                (1, 6, S),
            ],
            cycle: vec![1, 2, 3, 4, 5, 6],
            formula: "C4H9NO",
        },
        Example {
            source: "[H]C1CCCCC1",
            atoms: vec![(1, 0), (6, 1), (6, 2), (6, 2), (6, 2), (6, 2), (6, 2)],
            bonds: vec![
                (1, 2, S),
                (2, 3, S),
                (3, 4, S),
                (4, 5, S),
                (5, 6, S),
                (6, 7, S),
                (2, 7, S),
            ],
            cycle: vec![2, 3, 4, 5, 6, 7],
            formula: "C6H12",
        },
        Example {
            source: "CCC1(C)CCCCC1",
            atoms: vec![
                (6, 3),
                (6, 2),
                (6, 0),
                (6, 3),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
            ],
            bonds: vec![
                (1, 2, S),
                (2, 3, S),
                (3, 4, S),
                (3, 5, S),
                (5, 6, S),
                (6, 7, S),
                (7, 8, S),
                (8, 9, S),
                (3, 9, S),
            ],
            cycle: vec![3, 5, 6, 7, 8, 9],
            formula: "C9H18",
        },
        Example {
            source: "CC(C)C1(C(C)C)CCCCC1",
            atoms: vec![
                (6, 3),
                (6, 1),
                (6, 3),
                (6, 0),
                (6, 1),
                (6, 3),
                (6, 3),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
            ],
            bonds: vec![
                (1, 2, S),
                (2, 3, S),
                (2, 4, S),
                (4, 5, S),
                (5, 6, S),
                (5, 7, S),
                (4, 8, S),
                (8, 9, S),
                (9, 10, S),
                (10, 11, S),
                (11, 12, S),
                (4, 12, S),
            ],
            cycle: vec![4, 8, 9, 10, 11, 12],
            formula: "C12H24",
        },
        Example {
            source: "C12.CC1C2",
            atoms: vec![(6, 2), (6, 3), (6, 1), (6, 2)],
            bonds: vec![(2, 3, S), (1, 3, S), (3, 4, S), (1, 4, S)],
            cycle: vec![1, 3, 4],
            formula: "C4H8",
        },
        Example {
            source: "CCC=CCC1CCCCC1",
            atoms: vec![
                (6, 3),
                (6, 2),
                (6, 1),
                (6, 1),
                (6, 2),
                (6, 1),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
            ],
            bonds: vec![
                (1, 2, S),
                (2, 3, S),
                (3, 4, D),
                (4, 5, S),
                (5, 6, S),
                (6, 7, S),
                (7, 8, S),
                (8, 9, S),
                (9, 10, S),
                (10, 11, S),
                (6, 11, S),
            ],
            cycle: vec![6, 7, 8, 9, 10, 11],
            formula: "C11H20",
        },
    ]
}

fn parse(source: &str) -> TestResult<SmilesDocument> {
    Ok(SmilesDocument::parse(Arc::from(source.as_bytes()))?)
}
fn fragment(d: &Document) -> TestResult<&Fragment> {
    assert_eq!(d.pages().len(), 1);
    let p = d.pages().first().ok_or("page")?;
    assert_eq!(p.fragments().len(), 1);
    Ok(p.fragments().first().ok_or("fragment")?)
}
fn xy(d: &Document, id: u32) -> TestResult<[f64; 2]> {
    let p = fragment(d)?
        .atoms()
        .iter()
        .find(|a| a.id.get() == id)
        .ok_or("atom")?
        .position
        .ok_or("position")?;
    Ok([p.x_pt(), p.y_pt()])
}
fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn unit(a: [f64; 2]) -> [f64; 2] {
    let r = a[0].hypot(a[1]);
    [a[0] / r, a[1] / r]
}
fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn same_owner(a: &SmilesDocument, b: &SmilesDocument) {
    assert_eq!(a, b);
    assert_eq!(a.as_bytes(), b.as_bytes());
    assert_eq!(a.atoms(), b.atoms());
    assert_eq!(a.bonds(), b.bonds());
    assert_eq!(a.notation_range(), b.notation_range());
    assert_eq!(a.chain_starts(), b.chain_starts());
    assert_eq!(a.as_bytes().as_ptr(), b.as_bytes().as_ptr());
    assert_eq!(a.atoms().as_ptr(), b.atoms().as_ptr());
    assert_eq!(a.bonds().as_ptr(), b.bonds().as_ptr());
}
fn layout(owner: &SmilesDocument) -> TestResult<Document> {
    Ok(owner.layout_neutral_simple_ring(
        64.0,
        Point2::new(144.0, 144.0)?,
        PageSize::new(1000.0, 1000.0)?,
        &Cancellation::default(),
    )?)
}
fn center(d: &Document, cycle: &[u32]) -> TestResult<[f64; 2]> {
    let mut c = [0.0, 0.0];
    for &id in cycle {
        let p = xy(d, id)?;
        c[0] += p[0] / cycle.len() as f64;
        c[1] += p[1] / cycle.len() as f64;
    }
    Ok(c)
}
fn literal_graph(d: &Document, e: &Example) -> TestResult {
    let f = fragment(d)?;
    assert_eq!(f.id().get(), 1);
    assert_eq!(f.atoms().len(), e.atoms.len());
    for (i, (a, &(z, h))) in f.atoms().iter().zip(&e.atoms).enumerate() {
        assert_eq!(a.id.get(), u32::try_from(i + 1)?);
        assert_eq!(a.element.get(), z);
        assert_eq!(a.explicit_hydrogens, Some(h));
        assert_eq!(a.isotope, None);
        assert_eq!(a.charge, 0);
        assert_eq!(a.radical, Radical::None);
        assert!(a.position.is_some());
    }
    assert_eq!(
        f.bonds()
            .iter()
            .map(|b| (b.begin.get(), b.end.get(), b.order))
            .collect::<Vec<_>>(),
        e.bonds
    );
    assert_eq!(
        f.bonds().iter().map(|b| b.id.get()).collect::<Vec<_>>(),
        (1..=u32::try_from(e.bonds.len())?).collect::<Vec<_>>()
    );
    Ok(())
}
fn regular(d: &Document, cycle: &[u32], length: f64) -> TestResult {
    let c = center(d, cycle)?;
    let n = cycle.len();
    let radius = length / (2.0 * (std::f64::consts::PI / n as f64).sin());
    let interior = -(2.0 * std::f64::consts::PI / n as f64).cos();
    let mut turn = None;
    for (i, &id) in cycle.iter().enumerate() {
        let p = xy(d, id)?;
        let a = xy(d, cycle[(i + n - 1) % n])?;
        let b = xy(d, cycle[(i + 1) % n])?;
        near(sub(p, b)[0].hypot(sub(p, b)[1]), length, length * 1e-9);
        near(sub(p, c)[0].hypot(sub(p, c)[1]), radius, length * 1e-9);
        let u = sub(a, p);
        let v = sub(b, p);
        near(dot(u, v) / (length * length), interior, 1e-9);
        let cross = u[0] * v[1] - u[1] * v[0];
        assert!(cross.abs() > 1e-8);
        if let Some(t) = turn {
            assert_eq!(cross > 0.0, t);
        }
        turn = Some(cross > 0.0);
    }
    let points = cycle
        .iter()
        .map(|&id| xy(d, id))
        .collect::<TestResult<Vec<_>>>()?;
    let axis = if n == 6 { 0 } else { 1 };
    let extreme = if n == 6 {
        points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min)
    } else {
        points
            .iter()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max)
    };
    assert!(
        (0..n).any(|i| (points[i][axis] - extreme).abs() < 1e-8
            && (points[(i + 1) % n][axis] - extreme).abs() < 1e-8),
        "preferred ring edge must be vertical-left(6) or horizontal-bottom(other)"
    );
    Ok(())
}
fn scene_certificate(d: &Document) -> TestResult {
    let p = d.pages().first().ok_or("page")?;
    let before = p.clone();
    let literal = graph_page_scene(1024, 1024, p)?;
    let counts = fragment(d)?
        .atoms()
        .iter()
        .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("fixed H")?)))
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    let resolved =
        graph_page_scene_with_display_hydrogens(1024, 1024, p, &counts, &Cancellation::default())?;
    assert_eq!(
        literal.scene(),
        resolved.scene(),
        "ordinary native drawing must match the attachment-safe/paper-checked scene"
    );
    assert!(
        cpu_scene(literal.scene())?
            .rgba
            .chunks_exact(4)
            .any(|p| p == [0, 0, 0, 255])
    );
    assert_eq!(*p, before);
    Ok(())
}

#[test]
fn all_supported_simple_ring_sizes_have_literal_h_regular_geometry_and_preferred_edges()
-> TestResult {
    for n in 3..=8 {
        let source = format!("C1{}1", "C".repeat(n - 1));
        let owner = parse(&source)?;
        let before = owner.clone();
        let d = layout(&owner)?;
        let cycle = (1..=u32::try_from(n)?).collect::<Vec<_>>();
        let mut bonds = (1..u32::try_from(n)?)
            .map(|i| (i, i + 1, S))
            .collect::<Vec<_>>();
        bonds.push((1, u32::try_from(n)?, S));
        literal_graph(
            &d,
            &Example {
                source: "dynamic original saturated ring",
                atoms: vec![(6, 2); n],
                bonds,
                cycle: cycle.clone(),
                formula: "",
            },
        )?;
        regular(&d, &cycle, 64.0)?;
        scene_certificate(&d)?;
        same_owner(&owner, &before);
    }
    Ok(())
}

#[test]
fn single_substituent_follows_exterior_bisector_and_source_qualified_five_degree_snap() -> TestResult
{
    for n in 3..=8 {
        let source = format!("CC1{}1", "C".repeat(n - 1));
        let owner = parse(&source)?;
        let before = owner.clone();
        let d = layout(&owner)?;
        let cycle = (2..=u32::try_from(n + 1)?).collect::<Vec<_>>();
        regular(&d, &cycle, 64.0)?;
        let p = xy(&d, 2)?;
        let radial = unit(sub(p, center(&d, &cycle)?));
        let angle = radial[1].atan2(radial[0]);
        let snap = (angle.to_degrees() / 30.0).round() * 30.0;
        let target = if (snap - angle.to_degrees()).abs() <= 5.0 + 1e-9 {
            snap.to_radians()
        } else {
            angle
        };
        let branch = unit(sub(xy(&d, 1)?, p));
        near(dot(branch, [target.cos(), target.sin()]), 1.0, 1e-9);
        scene_certificate(&d)?;
        same_owner(&owner, &before);
    }
    Ok(())
}

#[test]
fn geminal_equal_hex_unequal_and_two_bulky_roots_use_their_sourced_contexts() -> TestResult {
    for n in 3..=8 {
        let source = format!("CC1(C){}1", "C".repeat(n - 1));
        let d = layout(&parse(&source)?)?;
        let cycle = std::iter::once(2)
            .chain(4..=u32::try_from(n + 2)?)
            .collect::<Vec<_>>();
        regular(&d, &cycle, 64.0)?;
        let a = xy(&d, 2)?;
        let u = unit(sub(xy(&d, 1)?, a));
        let v = unit(sub(xy(&d, 3)?, a));
        let cosine = dot(u, v);
        assert!(
            (cosine - 0.5).abs() < 1e-9 || n == 4 && cosine.abs() < 1e-9,
            "equal geminal60 degrees; square90 alternative"
        );
        let radial = unit(sub(a, center(&d, &cycle)?));
        assert!(dot(u, radial) > 0.0 && dot(v, radial) > 0.0);
        scene_certificate(&d)?;
    }
    let e = examples()
        .into_iter()
        .find(|e| e.source == "CCC1(C)CCCCC1")
        .ok_or("unequal")?;
    let d = layout(&parse(e.source)?)?;
    literal_graph(&d, &e)?;
    let p = xy(&d, 3)?;
    let r = unit(sub(p, center(&d, &e.cycle)?));
    let larger = unit(sub(xy(&d, 2)?, p));
    let small = unit(sub(xy(&d, 4)?, p));
    near(dot(larger, r), 1.0, 1e-9);
    near(dot(larger, small), 0.5, 1e-9);
    scene_certificate(&d)?;
    let e = examples()
        .into_iter()
        .find(|e| e.source == "CC(C)C1(C(C)C)CCCCC1")
        .ok_or("bulky")?;
    let d = layout(&parse(e.source)?)?;
    literal_graph(&d, &e)?;
    let p = xy(&d, 4)?;
    let u = unit(sub(xy(&d, 2)?, p));
    let v = unit(sub(xy(&d, 5)?, p));
    near(dot(u, v), 80.0_f64.to_radians().cos(), 1e-9);
    let r = unit(sub(p, center(&d, &e.cycle)?));
    near(dot(unit([u[0] + v[0], u[1] + v[1]]), r), 1.0, 1e-9);
    scene_certificate(&d)?;
    Ok(())
}

#[test]
fn heterocycles_physical_h_terminal_exocyclic_and_extended_multiple_trees_preserve_literal_graphs()
-> TestResult {
    for e in examples() {
        let owner = parse(e.source)?;
        let before = owner.clone();
        let d = layout(&owner)?;
        literal_graph(&d, &e)?;
        regular(&d, &e.cycle, 64.0)?;
        scene_certificate(&d)?;
        same_owner(&owner, &before);
        if e.source == "N#CC1CCCCC1" {
            near(
                dot(
                    unit(sub(xy(&d, 1)?, xy(&d, 2)?)),
                    unit(sub(xy(&d, 3)?, xy(&d, 2)?)),
                ),
                -1.0,
                1e-9,
            );
        }
        if e.source == "CCC=CCC1CCCCC1" {
            for (a, b, c) in [(2, 3, 4), (3, 4, 5)] {
                near(
                    dot(
                        unit(sub(xy(&d, a)?, xy(&d, b)?)),
                        unit(sub(xy(&d, c)?, xy(&d, b)?)),
                    ),
                    -1.0,
                    1e-9,
                );
            }
        }
    }
    Ok(())
}

#[test]
fn source_spelling_origin_translation_and_repeated_layout_do_not_change_identity_or_ir()
-> TestResult {
    let a = parse("C1CCCCC1\r\n")?;
    let before = a.clone();
    let d = layout(&a)?;
    assert_eq!(d, layout(&a)?);
    assert_eq!(d, layout(&parse("C2CCCCC2")?)?);
    assert_eq!(d, layout(&parse("[CH2]1[CH2][CH2][CH2][CH2][CH2]1")?)?);
    let moved = a.layout_neutral_simple_ring(
        64.0,
        Point2::new(194.0, 244.0)?,
        PageSize::new(1000.0, 1000.0)?,
        &Cancellation::default(),
    )?;
    for atom in fragment(&d)?.atoms() {
        let p = xy(&d, atom.id.get())?;
        let q = xy(&moved, atom.id.get())?;
        near(q[0] - p[0], 50.0, 1e-9);
        near(q[1] - p[1], 100.0, 1e-9);
    }
    same_owner(&a, &before);
    let dot = parse("C12.CC1C2")?;
    assert_eq!(dot.chain_starts(), 2);
    let d = layout(&dot)?;
    let e = examples()
        .into_iter()
        .find(|e| e.source == "C12.CC1C2")
        .ok_or("cross dot")?;
    literal_graph(&d, &e)?;
    regular(&d, &e.cycle, 64.0)?;
    Ok(())
}

#[test]
fn unsupported_topology_cycle_orders_metadata_bounds_cancellation_and_page_ink_refuse_atomically()
-> TestResult {
    for source in [
        "CC",
        "C1.CCCCC1",
        "C1CCCCCCCC1",
        "C1CC2CCC1C2",
        "C1CCC2(CC1)CCCC2",
        "C1CC1.C1CCC1",
        "C1CCCCC1.C",
        "C1=CCCCC1",
        "C1#CCCCC1",
        "C1=C=CCCC1",
        "c1ccccc1",
        "[13CH2]1CCCCC1",
        "[NH2+]1CCCCC1",
        "[CH]1CCCCC1",
        "P1CCCCC1",
        "CCC(C)=CCC1CCCCC1",
    ] {
        let owner = parse(source)?;
        let before = owner.clone();
        assert_eq!(
            owner
                .layout_neutral_simple_ring(
                    64.0,
                    Point2::new(144.0, 144.0)?,
                    PageSize::new(1000.0, 1000.0)?,
                    &Cancellation::default()
                )
                .err()
                .ok_or("refusal")?
                .code(),
            "unsupported_content",
            "{source}"
        );
        same_owner(&owner, &before);
    }
    let owner = parse("C1CCCCC1")?;
    let before = owner.clone();
    for length in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = owner
            .layout_neutral_simple_ring(
                length,
                Point2::new(144.0, 144.0)?,
                PageSize::A4,
                &Cancellation::default(),
            )
            .err()
            .ok_or("invalid length")?;
        assert!(matches!(error, LayoutError::InvalidBondLength));
        assert_eq!(error.code(), "invalid_params");
    }
    for source in [
        format!("{}C1CCCCC1", "C".repeat(59)),
        format!("P{}C1CCCCC1", "C".repeat(58)),
    ] {
        let big = parse(&source)?;
        assert_eq!(big.atoms().len(), 65);
        let old = big.clone();
        let error = big
            .layout_neutral_simple_ring(
                64.0,
                Point2::new(144.0, 144.0)?,
                PageSize::A4,
                &Cancellation::default(),
            )
            .err()
            .ok_or("cap")?;
        assert!(matches!(error, LayoutError::SimpleRingAtomLimit));
        assert_eq!(error.code(), "resource_limit");
        same_owner(&big, &old);
    }
    let atcap = parse(&format!("{}C1CCCCC1", "C".repeat(58)))?;
    assert_eq!(atcap.atoms().len(), 64);
    let old = atcap.clone();
    match atcap.layout_neutral_simple_ring(64.0,Point2::new(-1.0,144.0)?,PageSize::A4,&Cancellation::default()) {Err(LayoutError::Geometry(_) | LayoutError::Render(_))=>{},other=>return Err(format!("64 valid atoms must reach geometry validation, with impossible negative min-origin: {other:?}").into())};
    same_owner(&atcap, &old);
    let cancel = Cancellation::default();
    cancel.cancel();
    let error = owner
        .layout_neutral_simple_ring(64.0, Point2::new(144.0, 144.0)?, PageSize::A4, &cancel)
        .err()
        .ok_or("cancel")?;
    assert!(matches!(error, LayoutError::Cancelled));
    assert_eq!(error.code(), "cancelled");
    for (length, origin, page) in [
        (1e-300, Point2::new(144.0, 144.0)?, PageSize::A4),
        (f64::MAX, Point2::new(144.0, 144.0)?, PageSize::A4),
        (64.0, Point2::new(-1.0, 144.0)?, PageSize::A4),
        (64.0, Point2::new(1.0, 144.0)?, PageSize::A4),
        (64.0, Point2::new(480.0, 144.0)?, PageSize::A4),
        (64.0, Point2::new(144.0, 1.0)?, PageSize::A4),
        (64.0, Point2::new(144.0, 710.0)?, PageSize::A4),
        (
            64.0,
            Point2::new(144.0, 144.0)?,
            PageSize::new(1.0, 100_000.0)?,
        ),
    ] {
        assert!(
            owner
                .layout_neutral_simple_ring(length, origin, page, &Cancellation::default())
                .is_err(),
            "unsupported geometry/ink must not shrink or clip"
        );
    }
    same_owner(&owner, &before);
    Ok(())
}

fn external(script: &str, raw: &[u8], args: &[String]) -> TestResult<Output> {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let mut child = Command::new(python)
        .arg("-B")
        .arg(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), script))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(raw)?;
    Ok(child.wait_with_output()?)
}

#[test]
fn actual_cdxml_faces_new_ring_and_unchanged_general_consumers_then_reopens_editably() -> TestResult
{
    for e in examples() {
        let owner = parse(e.source)?;
        let before = owner.clone();
        let d = layout(&owner)?;
        literal_graph(&d, &e)?;
        let raw = write_document(&d)?;
        let options = json!({"atoms":e.atoms.iter().map(|(z,h)|json!([z,h])).collect::<Vec<_>>(),"bonds":e.bonds.iter().map(|(a,b,o)|json!([a,b,match o {S=>1,D=>2,T=>3,_=>0}])).collect::<Vec<_>>(),"formula":e.formula,"length":"64","origin":["144","144"],"page":["1000","1000"]});
        let output = external(
            "tests/smiles_ring_oracle.py",
            &raw,
            &[e.source.into(), options.to_string()],
        )?;
        assert!(
            output.status.success(),
            "{}: {}",
            e.source,
            String::from_utf8_lossy(&output.stderr)
        );
        let info: Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(info["ring_size"], e.cycle.len());
        assert_eq!(info["formula"], e.formula);
        assert_eq!(info["no_assigned_stereo"], true);
        let mut stereo = vec!["STEREONONE"; e.bonds.len()];
        if e.source == "CCC=CCC1CCCCC1" {
            stereo[2] = "STEREOANY";
        }
        assert_eq!(info["bond_stereo"], json!(stereo));
        let general = external("../../xtask/oracles/cdxml_graph.py", &raw, &[])?;
        assert!(general.status.success());
        let projected: Value = serde_json::from_slice(&general.stdout)?;
        assert_eq!(
            projected[0]["atoms"].as_array().ok_or("atoms")?.len(),
            e.atoms.len()
        );
        assert_eq!(
            projected[0]["bonds"].as_array().ok_or("bonds")?.len(),
            e.bonds.len()
        );
        let reopened = StoredDocument::from_cdxml(Arc::from(raw.as_slice()), PageSize::A4)?;
        assert!(reopened.is_fully_mapped());
        assert_eq!(reopened.model(), &d);
        assert_eq!(reopened.to_cdxml()?.as_ref(), raw.as_slice());
        same_owner(&owner, &before);
        if e.source == "CC1CCCCC1" {
            let mut wrong = options.clone();
            wrong["atoms"][1][1] = json!(2);
            assert!(
                !external(
                    "tests/smiles_ring_oracle.py",
                    &raw,
                    &[e.source.into(), wrong.to_string()]
                )?
                .status
                .success()
            );
            let mut wrong = options.clone();
            wrong["length"] = json!("63");
            assert!(
                !external(
                    "tests/smiles_ring_oracle.py",
                    &raw,
                    &[e.source.into(), wrong.to_string()]
                )?
                .status
                .success()
            );
        }
    }
    Ok(())
}

#[test]
fn off_ring_degree_four_tert_butyl_keeps_cycle_boundary_and_native_output() -> TestResult {
    // Added coverage after the first implementation, not a retroactive RED claim.
    // Off-ring Atom2 has three terminal carbons and the cyclic neighbor5. Computing
    // its neighbor signatures must stop at the ring boundary, not recurse around it.
    let expected = Example {
        source: "CC(C)(C)C1CCCCC1",
        atoms: vec![
            (6, 3),
            (6, 0),
            (6, 3),
            (6, 3),
            (6, 1),
            (6, 2),
            (6, 2),
            (6, 2),
            (6, 2),
            (6, 2),
        ],
        bonds: vec![
            (1, 2, S),
            (2, 3, S),
            (2, 4, S),
            (2, 5, S),
            (5, 6, S),
            (6, 7, S),
            (7, 8, S),
            (8, 9, S),
            (9, 10, S),
            (5, 10, S),
        ],
        cycle: vec![5, 6, 7, 8, 9, 10],
        formula: "C10H20",
    };
    let owner = parse(expected.source)?;
    let before = owner.clone();
    let document = layout(&owner)?;
    literal_graph(&document, &expected)?;
    regular(&document, &expected.cycle, 64.0)?;
    scene_certificate(&document)?;
    let at = xy(&document, 2)?;
    let mut angles = [1, 3, 4, 5]
        .into_iter()
        .map(|id| {
            let p = sub(xy(&document, id)?, at);
            Ok(p[1].atan2(p[0]).to_degrees().rem_euclid(360.0))
        })
        .collect::<TestResult<Vec<_>>>()?;
    angles.sort_by(f64::total_cmp);
    let mut gaps = angles.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
    gaps.push(360.0 + angles[0] - angles[3]);
    gaps.sort_by(f64::total_cmp);
    assert!(
        [
            [90.0, 90.0, 90.0, 90.0],
            [60.0, 90.0, 90.0, 120.0],
            [60.0, 60.0, 120.0, 120.0]
        ]
        .iter()
        .any(|family| gaps.iter().zip(family).all(|(a, b)| (a - b).abs() <= 1e-8)),
        "closed sourced degree-four angle families: {gaps:?}"
    );
    let raw = write_document(&document)?;
    let options = json!({
        "atoms":expected.atoms.iter().map(|(z,h)|json!([z,h])).collect::<Vec<_>>(),
        "bonds":expected.bonds.iter().map(|(a,b,_)|json!([a,b,1])).collect::<Vec<_>>(),
        "formula":"C10H20","length":"64","origin":["144","144"],"page":["1000","1000"]
    });
    let consumed = external(
        "tests/smiles_ring_oracle.py",
        &raw,
        &[expected.source.into(), options.to_string()],
    )?;
    assert!(
        consumed.status.success(),
        "{}",
        String::from_utf8_lossy(&consumed.stderr)
    );
    let metadata: Value = serde_json::from_slice(&consumed.stdout)?;
    assert_eq!(metadata["atoms"], 10);
    assert_eq!(metadata["bonds"], 10);
    assert_eq!(metadata["ring_size"], 6);
    assert_eq!(metadata["formula"], "C10H20");
    assert_eq!(metadata["stereo_none"], true);
    assert_eq!(metadata["no_assigned_stereo"], true);
    assert_eq!(metadata["bond_stereo"], json!(vec!["STEREONONE"; 10]));
    let reopened = StoredDocument::from_cdxml(Arc::from(raw.as_slice()), PageSize::A4)?;
    assert!(reopened.is_fully_mapped());
    assert_eq!(reopened.model(), &document);
    assert_eq!(reopened.to_cdxml()?.as_ref(), raw.as_slice());
    same_owner(&owner, &before);
    Ok(())
}
