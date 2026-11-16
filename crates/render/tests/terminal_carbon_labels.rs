//! Original terminal-carbon display declarations; Root runs only after DATA adoption.
use bondline_model::{
    Atom, AtomId, BondId, Document, Fragment, FragmentId, Page, PageId, PageSize,
};
use bondline_render::{
    Frame, HitTarget, Primitive, Rectangle, Stroke, cpu_scene, gpu_scene, graph_page_scene,
};
use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn terminal_page(atoms: Vec<Value>, bonds: Vec<Value>) -> TestResult<Page> {
    let atoms = atoms
        .into_iter()
        .map(serde_json::from_value::<Atom>)
        .collect::<Result<Vec<_>, _>>()?;
    let bonds = bonds
        .into_iter()
        .map(serde_json::from_value)
        .collect::<Result<Vec<_>, _>>()?;
    let pid = PageId::new(1).ok_or("page")?;
    let mut document = Document::default();
    document.set_page_size(pid, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(
        pid,
        vec![Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(document.pages().first().ok_or("page")?.clone())
}
fn terminal_atom(
    id: u32,
    z: u8,
    xy: [f64; 2],
    h: Option<u16>,
    terminal: Value,
    internal: Value,
) -> Value {
    json!({"id":id,"element":z,"position":{"x_pt":xy[0],"y_pt":xy[1]},
        "isotope":null,"explicit_hydrogens":h,
        "show_terminal_carbon_labels":terminal,"show_non_terminal_carbon_labels":internal})
}
fn terminal_edge(id: u32, a: u32, b: u32, order: &str) -> Value {
    json!({"id":id,"begin":a,"end":b,"order":order})
}
fn terminal_pixel(frame: &Frame, x: u32, y: u32) -> TestResult<&[u8]> {
    let at = usize::try_from((u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4)?;
    Ok(frame.rgba.get(at..at + 4).ok_or("pixel")?)
}
fn terminal_atom_hit(id: u32) -> TestResult<HitTarget> {
    Ok(HitTarget::Atom {
        fragment_id: FragmentId::new(17).ok_or("fragment")?,
        atom_id: AtomId::new(id).ok_or("atom")?,
    })
}
fn terminal_bond_hit(id: u32) -> TestResult<HitTarget> {
    Ok(HitTarget::Bond {
        fragment_id: FragmentId::new(17).ok_or("fragment")?,
        bond_id: BondId::new(id).ok_or("bond")?,
    })
}

#[test]
fn terminal_three_carbon_zigzag_is_only_two_full_axes_without_glyphs_or_masks() -> TestResult {
    // Original C3H8, L48, interior120deg. 256pt paper in576px gives exact2x+32.
    let points = [
        [64.0, 112.0],
        [112.0, 112.0],
        [136.0, 112.0 + 24.0 * 3.0_f64.sqrt()],
    ];
    let page = terminal_page(
        points
            .into_iter()
            .zip([3, 2, 3])
            .enumerate()
            .map(|(i, (p, h))| {
                terminal_atom(
                    u32::try_from(i + 1).unwrap_or(0),
                    6,
                    p,
                    Some(h),
                    json!(false),
                    json!(false),
                )
            })
            .collect(),
        vec![
            terminal_edge(41, 1, 2, "single"),
            terminal_edge(42, 2, 3, "single"),
        ],
    )?;
    let before = page.clone();
    let graph = graph_page_scene(576, 576, &page)?;
    assert_eq!(
        graph.label_em_px(),
        None,
        "all three authored carbons are skeletal, including both termini"
    );
    let expected_rectangles = vec![
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
            color: [255; 4],
        },
    ];
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
        expected_rectangles,
        "no chemical glyph rectangles or white label masks may remain"
    );
    assert_eq!(graph.scene().primitives.len(), 5);
    let canvas = points.map(|p| [(32.0 + 2.0 * p[0]) as f32, (32.0 + 2.0 * p[1]) as f32]);
    for (i, id) in [41, 42].into_iter().enumerate() {
        let expected = Stroke {
            begin: canvas[i],
            end: canvas[i + 1],
            width: 2.0,
            color: [0, 0, 0, 255],
        };
        assert_eq!(
            graph
                .bond_strokes(
                    FragmentId::new(17).ok_or("fragment")?,
                    BondId::new(id).ok_or("bond")?
                )
                .collect::<Vec<_>>(),
            vec![&expected]
        );
    }
    for (i, p) in canvas.into_iter().enumerate() {
        assert_eq!(
            graph.hit_test(p, 0.0)?,
            Some(terminal_atom_hit(u32::try_from(i + 1)?)?)
        );
    }
    assert_eq!(
        graph.hit_test([200.5, 256.5], 0.0)?,
        Some(terminal_bond_hit(41)?)
    );
    let cpu = cpu_scene(graph.scene())?;
    // Independent horizontal2px butt strip. Glyphless endpoints must not truncate it.
    for x in 160..256 {
        for y in 253..259 {
            assert_eq!(
                terminal_pixel(&cpu, x, y)?,
                if [255, 256].contains(&y) {
                    &[0, 0, 0, 255]
                } else {
                    &[255; 4]
                }
            );
        }
    }
    assert_eq!(terminal_pixel(&cpu, 176, 247)?, &[255; 4]);
    let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert!(
        !adapter.name.is_empty(),
        "an actual GPU adapter is required"
    );
    println!("terminal C3 GPU backend={:?}", adapter.backend);
    assert_eq!(gpu, cpu);
    assert_eq!(page, before);
    assert_eq!(
        page.fragments()[0]
            .atoms()
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        vec![Some(3), Some(2), Some(3)]
    );
    Ok(())
}

// Additive declarations after the immutable first4,734-byte analytic case.
use bondline_render::{
    Cancellation, Error, GraphScene, graph_page_scene_with_cancel,
    graph_page_scene_with_display_hydrogens,
};
use std::collections::BTreeMap;

fn terminal_patch(page: &Page, id: u32, fields: &[(&str, Value)]) -> TestResult<Page> {
    let fragment = page.fragments().first().ok_or("fragment")?;
    let atoms = fragment
        .atoms()
        .iter()
        .map(|atom| {
            let mut value = serde_json::to_value(atom)?;
            if atom.id.get() == id {
                let object = value.as_object_mut().ok_or("atom object")?;
                for (name, field) in fields {
                    object.insert((*name).to_owned(), field.clone());
                }
            }
            Ok(value)
        })
        .collect::<TestResult<Vec<_>>>()?;
    terminal_page(
        atoms,
        fragment
            .bonds()
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?,
    )
}
fn terminal_zigzag(n: usize) -> TestResult<Page> {
    let points = [
        [64.0, 112.0],
        [112.0, 112.0],
        [136.0, 112.0 + 24.0 * 3.0_f64.sqrt()],
        [184.0, 112.0 + 24.0 * 3.0_f64.sqrt()],
    ];
    let atoms = points[..n]
        .iter()
        .enumerate()
        .map(|(i, p)| {
            Ok(terminal_atom(
                u32::try_from(i + 1)?,
                6,
                *p,
                Some(if i == 0 || i + 1 == n { 3 } else { 2 }),
                json!(false),
                json!(false),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = (1..n)
        .map(|i| {
            Ok(terminal_edge(
                40 + u32::try_from(i)?,
                u32::try_from(i)?,
                u32::try_from(i + 1)?,
                "single",
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    terminal_page(atoms, bonds)
}
fn terminal_black_label_near(graph: &GraphScene, point: [f64; 2]) -> bool {
    // Only glyphs are black rectangles; actual bonds are Stroke primitives.
    graph.scene().primitives.iter().any(|p| {
        matches!(p,Primitive::Rectangle(r)
        if r.color==[0,0,0,255] && f64::from(r.x)<point[0]+16.0
        && f64::from(r.x+r.width)>point[0]-16.0 && f64::from(r.y)<point[1]+16.0
        && f64::from(r.y+r.height)>point[1]-16.0)
    })
}
fn terminal_counts(page: &Page) -> TestResult<BTreeMap<AtomId, u16>> {
    page.fragments()
        .iter()
        .flat_map(|f| f.atoms())
        .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("literal H")?)))
        .collect()
}
fn terminal_unsupported(result: Result<GraphScene, Error>) -> TestResult {
    let error = result.err().ok_or("invalid terminal-H proof accepted")?;
    assert!(
        matches!(error, Error::UnsupportedChemicalContent),
        "{error}"
    );
    Ok(())
}

#[test]
fn terminal_four_carbon_zigzag_and_independent_active_dormant_tristates() -> TestResult {
    let page = terminal_zigzag(4)?;
    let before = page.clone();
    let graph = graph_page_scene(576, 576, &page)?;
    assert_eq!(graph.label_em_px(), None);
    assert_eq!(
        graph.scene().primitives.len(),
        6,
        "three paper rectangles and three complete axes"
    );
    assert_eq!(
        graph
            .scene()
            .primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Rectangle(_)))
            .count(),
        3
    );
    let expected = [
        ([160.0, 256.0], [256.0, 256.0]),
        (
            [256.0, 256.0],
            [304.0, (256.0 + 48.0 * 3.0_f64.sqrt()) as f32],
        ),
        (
            [304.0, (256.0 + 48.0 * 3.0_f64.sqrt()) as f32],
            [400.0, (256.0 + 48.0 * 3.0_f64.sqrt()) as f32],
        ),
    ];
    for (i, (begin, end)) in expected.into_iter().enumerate() {
        assert_eq!(
            graph
                .bond_strokes(
                    FragmentId::new(17).ok_or("fragment")?,
                    BondId::new(41 + u32::try_from(i)?).ok_or("bond")?
                )
                .collect::<Vec<_>>(),
            vec![&Stroke {
                begin,
                end,
                width: 2.0,
                color: [0, 0, 0, 255]
            }]
        );
    }
    let frame = cpu_scene(graph.scene())?;
    for x in 330..396 {
        for y in 336..343 {
            assert_eq!(
                terminal_pixel(&frame, x, y)?,
                if [338, 339].contains(&y) {
                    &[0, 0, 0, 255]
                } else {
                    &[255; 4]
                }
            );
        }
    }
    assert_eq!(
        graph.hit_test([400.0, (256.0 + 48.0 * 3.0_f64.sqrt()) as f32], 0.0)?,
        Some(terminal_atom_hit(4)?)
    );
    let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert!(!adapter.name.is_empty());
    assert_eq!(gpu, frame);
    // The inactive flag never controls this atom, including explicit true/false.
    for value in [Value::Null, json!(false), json!(true)] {
        for (id, field) in [
            (2, "show_terminal_carbon_labels"),
            (1, "show_non_terminal_carbon_labels"),
        ] {
            let altered = terminal_patch(&page, id, &[(field, value.clone())])?;
            let snapshot = altered.clone();
            assert_eq!(graph_page_scene(576, 576, &altered)?.scene(), graph.scene());
            assert_eq!(altered, snapshot);
        }
        let terminal = terminal_patch(&page, 1, &[("show_terminal_carbon_labels", value.clone())])?;
        let terminal_graph = graph_page_scene(576, 576, &terminal)?;
        assert_eq!(
            terminal_black_label_near(&terminal_graph, [160.0, 256.0]),
            value != json!(false)
        );
        let internal = terminal_patch(
            &page,
            2,
            &[("show_non_terminal_carbon_labels", value.clone())],
        )?;
        let internal_graph = graph_page_scene(576, 576, &internal)?;
        assert_eq!(
            terminal_black_label_near(&internal_graph, [256.0, 256.0]),
            value != json!(false)
        );
    }
    assert_eq!(page, before);
    assert_eq!(
        page.fragments()[0]
            .atoms()
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        vec![Some(3), Some(2), Some(2), Some(3)]
    );
    Ok(())
}

#[test]
fn terminal_methane_and_isolated_c2_are_labeled_from_actual_incidence_not_fragment_size()
-> TestResult {
    for (order, h) in [("single", 3), ("double", 2), ("triple", 1)] {
        // Sparse/reordered IDs, reversed directed edge, unrelated O and isolated methane
        // in the SAME Fragment. All four have degree0/1; only actual C2 is special.
        let page = terminal_page(
            vec![
                terminal_atom(71, 8, [192.0, 192.0], Some(2), json!(false), json!(false)),
                terminal_atom(19, 6, [64.0, 72.0], Some(h), json!(false), json!(true)),
                terminal_atom(37, 6, [64.0, 192.0], Some(4), json!(false), json!(false)),
                terminal_atom(3, 6, [128.0, 72.0], Some(h), json!(false), json!(true)),
            ],
            vec![terminal_edge(41, 3, 19, order)],
        )?;
        let before = page.clone();
        let graph = graph_page_scene(576, 576, &page)?;
        for point in [
            [160.0, 176.0],
            [288.0, 176.0],
            [160.0, 416.0],
            [416.0, 416.0],
        ] {
            assert!(
                terminal_black_label_near(&graph, point),
                "{order} mandatory label at{point:?}"
            );
        }
        for (id, point) in [
            (19, [160.0, 176.0]),
            (3, [288.0, 176.0]),
            (37, [160.0, 416.0]),
        ] {
            assert_eq!(graph.hit_test(point, 0.0)?, Some(terminal_atom_hit(id)?));
        }
        let shown = terminal_patch(
            &terminal_patch(
                &terminal_patch(&page, 19, &[("show_terminal_carbon_labels", json!(true))])?,
                3,
                &[("show_terminal_carbon_labels", json!(true))],
            )?,
            37,
            &[("show_terminal_carbon_labels", json!(true))],
        )?;
        assert_eq!(
            graph_page_scene(576, 576, &shown)?.scene(),
            graph.scene(),
            "mandatory full labels equal explicitly shown labels; no C2 blank line"
        );
        let map = terminal_counts(&page)?;
        assert_eq!(
            graph_page_scene_with_display_hydrogens(
                576,
                576,
                &page,
                &map,
                &Cancellation::default()
            )?
            .scene(),
            graph.scene()
        );
        assert_eq!(page, before);
        assert_eq!(
            page.fragments()[0]
                .atoms()
                .iter()
                .map(|a| a.explicit_hydrogens)
                .collect::<Vec<_>>(),
            vec![Some(2), Some(h), Some(4), Some(h)]
        );
    }
    Ok(())
}

#[test]
fn terminal_requires_known_or_complete_resolved_c4_but_true_remains_literal() -> TestResult {
    let page = terminal_zigzag(3)?;
    let known = graph_page_scene(576, 576, &page)?;
    let unknown = terminal_patch(&page, 1, &[("explicit_hydrogens", Value::Null)])?;
    let snapshot = unknown.clone();
    terminal_unsupported(graph_page_scene(576, 576, &unknown))?;
    let valid = terminal_counts(&page)?;
    assert_eq!(
        graph_page_scene_with_display_hydrogens(
            576,
            576,
            &unknown,
            &valid,
            &Cancellation::default()
        )?
        .scene(),
        known.scene()
    );
    for h in [0, 1, 2, 4, u16::MAX] {
        let mut invalid = valid.clone();
        invalid.insert(AtomId::new(1).ok_or("atom")?, h);
        terminal_unsupported(graph_page_scene_with_display_hydrogens(
            576,
            576,
            &unknown,
            &invalid,
            &Cancellation::default(),
        ))?;
        terminal_unsupported(graph_page_scene(
            576,
            576,
            &terminal_patch(&page, 1, &[("explicit_hydrogens", json!(h))])?,
        ))?;
    }
    let mut missing = valid.clone();
    missing.remove(&AtomId::new(3).ok_or("atom")?);
    let mut extra = valid.clone();
    extra.insert(AtomId::new(999).ok_or("atom")?, 0);
    for map in [missing, extra] {
        terminal_unsupported(graph_page_scene_with_display_hydrogens(
            576,
            576,
            &unknown,
            &map,
            &Cancellation::default(),
        ))?;
    }
    // Literal display of H9 is intentionally not a chemistry-validity claim.
    let literal = terminal_patch(
        &page,
        1,
        &[
            ("explicit_hydrogens", json!(9)),
            ("show_terminal_carbon_labels", json!(true)),
        ],
    )?;
    assert!(terminal_black_label_near(
        &graph_page_scene(576, 576, &literal)?,
        [160.0, 256.0]
    ));
    let true_unknown =
        terminal_patch(&unknown, 1, &[("show_terminal_carbon_labels", json!(true))])?;
    terminal_unsupported(graph_page_scene(576, 576, &true_unknown))?;
    assert!(terminal_black_label_near(
        &graph_page_scene_with_display_hydrogens(
            576,
            576,
            &true_unknown,
            &valid,
            &Cancellation::default()
        )?,
        [160.0, 256.0]
    ));
    assert_eq!(unknown, snapshot);
    assert_eq!(unknown.fragments()[0].atoms()[0].explicit_hydrogens, None);
    Ok(())
}

#[test]
fn terminal_degree_counts_physical_bonds_not_weight_and_other_label_rules_stay_exact() -> TestResult
{
    // C=O is degree1, not degree2; the nonterminal true is dormant. Full H2 stays.
    let carbonyl = terminal_page(
        vec![
            terminal_atom(1, 6, [80.0, 128.0], Some(2), json!(false), json!(true)),
            terminal_atom(2, 8, [160.0, 128.0], Some(0), json!(false), json!(false)),
        ],
        vec![terminal_edge(41, 1, 2, "double")],
    )?;
    let graph = graph_page_scene(576, 576, &carbonyl)?;
    assert!(!terminal_black_label_near(&graph, [192.0, 288.0]));
    assert!(terminal_black_label_near(&graph, [352.0, 288.0]));
    assert_eq!(
        carbonyl.fragments()[0].atoms()[0].explicit_hydrogens,
        Some(2)
    );
    // A physical H contributes an actual second bond. C2 is not an isolated C2 component.
    let physical = terminal_page(
        vec![
            terminal_atom(1, 6, [128.0, 128.0], Some(2), json!(true), json!(false)),
            terminal_atom(2, 1, [64.0, 96.0], Some(0), json!(false), json!(false)),
            terminal_atom(3, 6, [192.0, 96.0], Some(3), json!(false), json!(true)),
        ],
        vec![
            terminal_edge(41, 1, 2, "single"),
            terminal_edge(42, 1, 3, "single"),
        ],
    )?;
    let before = physical.clone();
    let graph = graph_page_scene(576, 576, &physical)?;
    assert!(!terminal_black_label_near(&graph, [288.0, 288.0]));
    assert!(!terminal_black_label_near(&graph, [416.0, 224.0]));
    assert!(terminal_black_label_near(&graph, [160.0, 224.0]));
    assert_eq!(
        graph.hit_test([288.0, 288.0], 0.0)?,
        Some(terminal_atom_hit(1)?)
    );
    assert_eq!(
        graph.hit_test([416.0, 224.0], 0.0)?,
        Some(terminal_atom_hit(3)?)
    );
    assert_eq!(physical, before);
    assert_eq!(
        physical.fragments()[0]
            .atoms()
            .iter()
            .map(|a| (a.element.get(), a.explicit_hydrogens))
            .collect::<Vec<_>>(),
        vec![(6, Some(2)), (1, Some(0)), (6, Some(3))]
    );
    // Noncarbon flags are dormant, including the terminal/nonterminal pair independently.
    let oxygen = terminal_page(
        vec![terminal_atom(
            1,
            8,
            [128.0, 128.0],
            Some(2),
            Value::Null,
            Value::Null,
        )],
        vec![],
    )?;
    let old = graph_page_scene(576, 576, &oxygen)?;
    for t in [Value::Null, json!(false), json!(true)] {
        for n in [Value::Null, json!(false), json!(true)] {
            assert_eq!(
                graph_page_scene(
                    576,
                    576,
                    &terminal_patch(
                        &oxygen,
                        1,
                        &[
                            ("show_terminal_carbon_labels", t.clone()),
                            ("show_non_terminal_carbon_labels", n)
                        ]
                    )?
                )?
                .scene(),
                old.scene()
            );
        }
    }
    // Exact legacy mandatory center rules: straight Single/Single and Double/Double.
    for (order, end_h, center_h) in [("single", 3, 2), ("double", 2, 0)] {
        let page = terminal_page(
            vec![
                terminal_atom(1, 6, [64.0, 128.0], Some(end_h), Value::Null, Value::Null),
                terminal_atom(
                    2,
                    6,
                    [128.0, 128.0],
                    Some(center_h),
                    Value::Null,
                    json!(false),
                ),
                terminal_atom(3, 6, [192.0, 128.0], Some(end_h), Value::Null, Value::Null),
            ],
            vec![
                terminal_edge(41, 1, 2, order),
                terminal_edge(42, 2, 3, order),
            ],
        )?;
        let old = graph_page_scene(576, 576, &page)?;
        assert!(terminal_black_label_near(&old, [288.0, 288.0]));
        for flag in [Value::Null, json!(false), json!(true)] {
            assert_eq!(
                graph_page_scene(
                    576,
                    576,
                    &terminal_patch(&page, 2, &[("show_terminal_carbon_labels", flag)])?
                )?
                .scene(),
                old.scene()
            );
        }
    }
    Ok(())
}

#[test]
fn terminal_hidden_glyphs_never_relax_actual_ink_paper_resource_or_cancel_guards() -> TestResult {
    let page = terminal_zigzag(3)?;
    let before = page.clone();
    let mut atoms = page.fragments()[0]
        .atoms()
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    atoms.push(terminal_atom(
        90,
        8,
        [88.0, 112.0],
        Some(0),
        Value::Null,
        Value::Null,
    ));
    let crossed = terminal_page(
        atoms,
        vec![
            terminal_edge(41, 1, 2, "single"),
            terminal_edge(42, 2, 3, "single"),
        ],
    )?;
    assert!(
        matches!(
            graph_page_scene(576, 576, &crossed),
            Err(Error::LabelOverlap)
        ),
        "an unrelated O label cannot hide the now fully exposed C-C ink"
    );
    let tiny_double = terminal_page(
        vec![
            terminal_atom(1, 6, [64.0, 112.0], Some(2), json!(false), json!(false)),
            terminal_atom(2, 6, [64.5, 112.0], Some(1), json!(false), json!(false)),
            terminal_atom(
                3,
                6,
                [88.5, 112.0 + 24.0 * 3.0_f64.sqrt()],
                Some(3),
                json!(false),
                json!(false),
            ),
        ],
        vec![
            terminal_edge(41, 1, 2, "double"),
            terminal_edge(42, 2, 3, "single"),
        ],
    )?;
    // The existing 9w short-axis preflight is distinct from the visible-pixel guard.
    assert!(matches!(
        graph_page_scene(576, 576, &tiny_double),
        Err(Error::UnsupportedChemicalContent)
    ));
    let subpixel_double = terminal_page(
        vec![
            terminal_atom(1, 6, [64.0, 112.0], Some(2), json!(false), json!(false)),
            terminal_atom(2, 6, [112.0, 112.0], Some(1), json!(false), json!(false)),
            terminal_atom(
                3,
                6,
                [136.0, 112.0 + 24.0 * 3.0_f64.sqrt()],
                Some(3),
                json!(false),
                json!(false),
            ),
        ],
        vec![
            terminal_edge(41, 1, 2, "double"),
            terminal_edge(42, 2, 3, "single"),
        ],
    )?;
    // At80px, scale15/64: the lower Double arm has no pixel-center sample despite L48>=9w.
    assert!(matches!(
        graph_page_scene(80, 80, &subpixel_double),
        Err(Error::BondScale)
    ));
    let outside = terminal_page(
        vec![
            terminal_atom(1, 6, [-4.0, 96.0], Some(3), json!(false), json!(false)),
            terminal_atom(2, 6, [44.0, 96.0], Some(2), json!(false), json!(false)),
            terminal_atom(
                3,
                6,
                [68.0, 96.0 + 24.0 * 3.0_f64.sqrt()],
                Some(3),
                json!(false),
                json!(false),
            ),
        ],
        vec![
            terminal_edge(41, 1, 2, "single"),
            terminal_edge(42, 2, 3, "single"),
        ],
    )?;
    assert!(
        matches!(
            graph_page_scene_with_display_hydrogens(
                576,
                576,
                &outside,
                &terminal_counts(&outside)?,
                &Cancellation::default()
            ),
            Err(Error::LabelPlacement)
        ),
        "resolved whole-ink paper bound also applies when no glyph remains"
    );
    for fields in [
        vec![("charge", json!(1))],
        vec![("isotope", json!(13))],
        vec![("radical", json!("doublet"))],
        vec![("position", Value::Null)],
    ] {
        terminal_unsupported(graph_page_scene(
            576,
            576,
            &terminal_patch(&page, 1, &fields)?,
        ))?;
    }
    let crowded = terminal_page(
        (1..=4097_u32)
            .map(|id| terminal_atom(id, 6, [128.0, 128.0], Some(4), json!(false), json!(false)))
            .collect(),
        vec![],
    )?;
    assert!(matches!(
        graph_page_scene(576, 576, &crowded),
        Err(Error::SceneTooComplex)
    ));
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        graph_page_scene_with_cancel(576, 576, &page, &cancel),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        graph_page_scene_with_display_hydrogens(576, 576, &page, &terminal_counts(&page)?, &cancel),
        Err(Error::Cancelled)
    ));
    assert_eq!(page, before);
    Ok(())
}
