use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    PageSize, Point2,
};
use bondline_render::{Primitive, Stroke, cpu_scene, gpu_scene, graph_page_scene, page_scene};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn page(order: BondOrder) -> TestResult<Page> {
    let first = AtomId::new(1).ok_or("atom")?;
    let second = AtomId::new(2).ok_or("atom")?;
    let atoms = vec![
        Atom::new(first, Element::try_from(6)?, Some(Point2::new(32.0, 64.0)?)),
        Atom::new(
            second,
            Element::try_from(6)?,
            Some(Point2::new(96.0, 64.0)?),
        ),
    ];
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    document.set_page_size(id, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(1).ok_or("bond")?,
                begin: first,
                end: second,
                order,
            }],
        )?],
    )?;
    Ok(document.pages()[0].clone())
}

#[test]
fn centered_double_and_triple_have_independent_strips_and_white_gaps() -> TestResult {
    for (order, rows) in [
        (BondOrder::Double, vec![157.0, 163.0]),
        (BondOrder::Triple, vec![154.0, 160.0, 166.0]),
    ] {
        let page = page(order)?;
        let before = page.clone();
        let graph = graph_page_scene(576, 576, &page)?;
        let actual = graph
            .scene()
            .primitives
            .iter()
            .filter_map(|p| match p {
                Primitive::Stroke(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let expected = rows
            .iter()
            .map(|y| Stroke {
                begin: [96.0, *y],
                end: [224.0, *y],
                width: 2.0,
                color: [0, 0, 0, 255],
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        let cpu = cpu_scene(graph.scene())?;
        let base = cpu_scene(&page_scene(576, 576, page.size())?)?;
        for y in 148..172 {
            for x in 90..230 {
                let index = ((y * 576 + x) * 4) as usize;
                let ink = (96..224).contains(&x)
                    && rows
                        .iter()
                        .any(|row| (f64::from(y) + 0.5 - f64::from(*row)).abs() <= 1.0);
                assert_eq!(
                    &cpu.rgba[index..index + 4],
                    if ink {
                        &[0, 0, 0, 255]
                    } else {
                        &base.rgba[index..index + 4]
                    },
                    "{order:?} pixel{x},{y}"
                );
            }
        }
        assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, cpu);
        assert_eq!(page, before);
    }
    Ok(())
}

fn phase_page(order: BondOrder, width: f64, y: f64) -> TestResult<Page> {
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    document.set_page_size(id, PageSize::new(192.0 / width, 192.0 / width)?)?;
    let atoms = [80.0, 180.0]
        .into_iter()
        .enumerate()
        .map(|(i, x)| {
            Ok(Atom::new(
                AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new((x - 32.0) / width, (y - 32.0) / width)?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(1).ok_or("bond")?,
                begin: AtomId::new(1).ok_or("a")?,
                end: AtomId::new(2).ok_or("b")?,
                order,
            }],
        )?],
    )?;
    Ok(document.pages()[0].clone())
}

#[test]
fn invisible_components_and_merged_pixel_rows_must_refuse() -> TestResult {
    for (order, width, y) in [
        (BondOrder::Double, 0.95, 100.425),
        (BondOrder::Triple, 0.95, 100.0),
        (BondOrder::Double, 0.5, 100.0),
    ] {
        let page = phase_page(order, width, y)?;
        let result = graph_page_scene(256, 256, &page);
        assert!(
            matches!(result, Err(bondline_render::Error::BondScale)),
            "{order:?} width{width} phase{y} cannot retain all strips and a white gap"
        );
    }
    Ok(())
}

#[test]
fn represented_subpixel_components_are_not_rejected_by_a_blanket_width_rule() -> TestResult {
    for (order, width, y) in [
        (BondOrder::Double, 0.95, 100.0),
        (BondOrder::Triple, 0.95, 100.5),
        (BondOrder::Double, 0.5, 100.5),
    ] {
        let page = phase_page(order, width, y)?;
        let graph = graph_page_scene(256, 256, &page)?;
        let strokes = graph
            .bond_strokes(
                FragmentId::new(1).ok_or("fragment")?,
                BondId::new(1).ok_or("bond")?,
            )
            .collect::<Vec<_>>();
        assert_eq!(
            strokes.len(),
            if order == BondOrder::Double { 2 } else { 3 }
        );
        let cpu = cpu_scene(graph.scene())?;
        for stroke in strokes {
            let row = (0..256).any(|row| {
                let y = f64::from(row) + 0.5;
                let expected =
                    (y - f64::from(stroke.begin[1])).abs() <= f64::from(stroke.width) / 2.0;
                expected
                    && cpu
                        .rgba
                        .get(((row * 256 + 128) * 4) as usize..((row * 256 + 128) * 4 + 4) as usize)
                        == Some(&[0, 0, 0, 255])
            });
            assert!(
                row,
                "actual subpixel component lost its independently predicted ink"
            );
        }
        assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, cpu);
    }
    Ok(())
}

#[test]
fn components_share_one_logical_hit_and_white_gaps_are_not_axis_ink() -> TestResult {
    let fid = FragmentId::new(1).ok_or("fragment")?;
    let bid = BondId::new(1).ok_or("bond")?;
    for order in [BondOrder::Single, BondOrder::Double, BondOrder::Triple] {
        let page = page(order)?;
        let graph = graph_page_scene(576, 576, &page)?;
        let strokes = graph.bond_strokes(fid, bid).collect::<Vec<_>>();
        assert_eq!(
            strokes.len(),
            match order {
                BondOrder::Double => 2,
                BondOrder::Triple => 3,
                _ => 1,
            }
        );
        for stroke in &strokes {
            assert_eq!(
                graph.hit_test([160.0, stroke.begin[1]], 0.0)?,
                Some(bondline_render::HitTarget::Bond {
                    fragment_id: fid,
                    bond_id: bid
                })
            );
        }
        if order != BondOrder::Single {
            let gap = (strokes[0].begin[1] + strokes[1].begin[1]) / 2.0;
            assert_eq!(graph.hit_test([160.0, gap], 0.0)?, None);
            assert_eq!(
                graph.hit_test([160.0, gap], 3.0)?,
                Some(bondline_render::HitTarget::Bond {
                    fragment_id: fid,
                    bond_id: bid
                })
            );
        }
        assert_eq!(
            graph
                .bond_strokes(fid, BondId::new(99).ok_or("missing")?)
                .count(),
            0
        );
        assert_eq!(
            graph
                .bond_strokes(FragmentId::new(99).ok_or("missing")?, bid)
                .count(),
            0
        );
    }
    Ok(())
}

fn crossing_page(order: BondOrder, corridor: bool) -> TestResult<Page> {
    let original = page(order)?;
    let mut document = Document::default();
    let id = original.id();
    document.set_page_size(id, original.size())?;
    document.append_fragments(id, vec![original.fragments()[0].as_ref().clone()])?;
    let coordinates = if corridor {
        [[44.0, 64.0], [84.0, 64.0]]
    } else {
        [[64.0, 32.0], [64.0, 96.0]]
    };
    let atoms = coordinates
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            Ok(Atom::new(
                AtomId::new(u32::try_from(i + 3)?).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(p[0], p[1])?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(2).ok_or("fragment")?,
            atoms,
            vec![Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(2).ok_or("bond")?,
                begin: AtomId::new(3).ok_or("a")?,
                end: AtomId::new(4).ok_or("b")?,
                order: BondOrder::Single,
            }],
        )?],
    )?;
    Ok(document.pages()[0].clone())
}

#[test]
fn unrelated_ink_cannot_cross_or_occupy_a_multiple_bundle_corridor() -> TestResult {
    for corridor in [false, true] {
        let page = crossing_page(BondOrder::Double, corridor)?;
        let before = page.clone();
        assert!(matches!(
            graph_page_scene(576, 576, &page),
            Err(bondline_render::Error::LabelOverlap)
        ));
        assert_eq!(page, before);
        let old_single = crossing_page(BondOrder::Single, corridor)?;
        assert!(
            graph_page_scene(576, 576, &old_single).is_ok(),
            "new congestion policy is scoped to multiple bundles"
        );
    }
    Ok(())
}

#[test]
fn original_spacing_rule_rejects_short_axes_and_accepts_its_exact_boundary() -> TestResult {
    for order in [BondOrder::Double, BondOrder::Triple] {
        for (length, supported) in [(8.0, false), (8.999, false), (9.0, true), (9.001, true)] {
            let base = page(order)?;
            let f = &base.fragments()[0];
            let mut atoms = f.atoms().to_vec();
            atoms[1].position = Some(Point2::new(32.0 + length, 64.0)?);
            let mut doc = Document::default();
            doc.set_page_size(base.id(), base.size())?;
            doc.append_fragments(
                base.id(),
                vec![Fragment::new(f.id(), atoms, f.bonds().to_vec())?],
            )?;
            let result = graph_page_scene(576, 576, &doc.pages()[0]);
            if supported {
                assert!(result.is_ok(), "{order:?} length{length}");
            } else {
                assert!(
                    matches!(
                        result,
                        Err(bondline_render::Error::UnsupportedChemicalContent)
                    ),
                    "{order:?} length{length} violates original s<=L/3"
                );
            }
        }
    }
    Ok(())
}

fn chain(order: BondOrder, literal: bool) -> TestResult<Page> {
    let mut doc = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    doc.set_page_size(id, PageSize::new(256.0, 256.0)?)?;
    let h = if order == BondOrder::Double {
        [2, 1, 3]
    } else {
        [1, 0, 3]
    };
    let atoms = [32.0, 96.0, 160.0]
        .into_iter()
        .enumerate()
        .map(|(i, x)| {
            let mut atom = Atom::new(
                AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(x, 128.0)?),
            );
            if literal {
                atom.explicit_hydrogens = Some(h[i]);
            }
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = vec![
        Bond {
            double_position: bondline_model::DoublePosition::Unspecified,
            id: BondId::new(1).ok_or("bond")?,
            begin: atoms[0].id,
            end: atoms[1].id,
            order,
        },
        Bond {
            double_position: bondline_model::DoublePosition::Unspecified,
            id: BondId::new(2).ok_or("bond")?,
            begin: atoms[1].id,
            end: atoms[2].id,
            order: BondOrder::Single,
        },
    ];
    doc.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(doc.pages()[0].clone())
}

#[test]
fn literal_multiple_junctions_use_component_aware_h_clearance_without_recounting() -> TestResult {
    for order in [BondOrder::Double, BondOrder::Triple] {
        let page = chain(order, true)?;
        let before = page.clone();
        let counts = page.fragments()[0]
            .atoms()
            .iter()
            .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("literal H")?)))
            .collect::<TestResult<std::collections::BTreeMap<_, _>>>()?;
        let literal = graph_page_scene(576, 576, &page)?;
        let resolved = bondline_render::graph_page_scene_with_display_hydrogens(
            576,
            576,
            &page,
            &counts,
            &bondline_render::Cancellation::default(),
        )?;
        assert_eq!(
            literal.scene(),
            resolved.scene(),
            "same literal counts require the same clear element attachment"
        );
        assert_eq!(
            pollster::block_on(gpu_scene(literal.scene()))?.0,
            cpu_scene(resolved.scene())?
        );
        assert_eq!(page, before);
        assert_eq!(
            literal
                .bond_strokes(
                    FragmentId::new(1).ok_or("fragment")?,
                    BondId::new(2).ok_or("bond")?
                )
                .count(),
            1
        );
        let unknown = chain(order, false)?;
        assert!(
            graph_page_scene(576, 576, &unknown).is_ok(),
            "safe nonterminal skeletal bundles must remain supported"
        );
        assert!(
            unknown.fragments()[0]
                .atoms()
                .iter()
                .all(|a| a.explicit_hydrogens.is_none())
        );
    }
    Ok(())
}

#[test]
fn public_drag_constructor_is_the_same_closed_geometry_used_by_graphs() -> TestResult {
    let axis = Stroke {
        begin: [96.0, 160.0],
        end: [224.0, 160.0],
        width: 2.0,
        color: [0, 0, 0, 255],
    };
    for order in [BondOrder::Single, BondOrder::Double, BondOrder::Triple] {
        let components = bondline_render::centered_bond_strokes(&axis, order)?;
        let page = page(order)?;
        let graph = graph_page_scene(576, 576, &page)?;
        assert_eq!(
            graph
                .bond_strokes(
                    FragmentId::new(1).ok_or("fragment")?,
                    BondId::new(1).ok_or("bond")?
                )
                .cloned()
                .collect::<Vec<_>>(),
            components
        );
        if order == BondOrder::Single {
            assert_eq!(components, vec![axis.clone()]);
        }
    }
    for order in [
        BondOrder::Aromatic,
        BondOrder::Quadruple,
        BondOrder::Half,
        BondOrder::Dative,
    ] {
        assert!(matches!(
            bondline_render::centered_bond_strokes(&axis, order),
            Err(bondline_render::Error::UnsupportedChemicalContent)
        ));
    }
    assert!(matches!(
        bondline_render::centered_bond_strokes(
            &Stroke {
                end: axis.begin,
                ..axis
            },
            BondOrder::Double
        ),
        Err(bondline_render::Error::InvalidStroke)
    ));
    Ok(())
}

#[test]
fn vertical_mirrored_and_reversed_bundles_retain_exact_components_and_pixels() -> TestResult {
    for order in [BondOrder::Double, BondOrder::Triple] {
        let base = page(order)?;
        let original = graph_page_scene(576, 576, &base)?;
        let expected = original
            .bond_strokes(
                FragmentId::new(1).ok_or("fragment")?,
                BondId::new(1).ok_or("bond")?,
            )
            .cloned()
            .collect::<Vec<_>>();
        for transform in 0..3 {
            let f = &base.fragments()[0];
            let mut atoms = f.atoms().to_vec();
            let mut bonds = f.bonds().to_vec();
            for atom in &mut atoms {
                let p = atom.position.ok_or("point")?;
                atom.position = Some(if transform == 0 {
                    Point2::new(256.0 - p.y_pt(), p.x_pt())?
                } else if transform == 1 {
                    Point2::new(256.0 - p.x_pt(), p.y_pt())?
                } else {
                    p
                });
            }
            if transform == 2 {
                let b = &mut bonds[0];
                std::mem::swap(&mut b.begin, &mut b.end);
            }
            let mut doc = Document::default();
            doc.set_page_size(base.id(), base.size())?;
            doc.append_fragments(base.id(), vec![Fragment::new(f.id(), atoms, bonds)?])?;
            let graph = graph_page_scene(576, 576, &doc.pages()[0])?;
            let components = graph
                .bond_strokes(f.id(), BondId::new(1).ok_or("bond")?)
                .collect::<Vec<_>>();
            let map = |p: [f32; 2]| {
                if transform == 0 {
                    [576.0 - p[1], p[0]]
                } else if transform == 1 {
                    [576.0 - p[0], p[1]]
                } else {
                    p
                }
            };
            for (i, actual) in components.iter().enumerate() {
                let source = if transform == 0 {
                    &expected[i]
                } else {
                    &expected[expected.len() - 1 - i]
                };
                assert_eq!(
                    actual.begin,
                    map(if transform == 2 {
                        source.end
                    } else {
                        source.begin
                    })
                );
                assert_eq!(
                    actual.end,
                    map(if transform == 2 {
                        source.begin
                    } else {
                        source.end
                    })
                );
                assert_eq!(actual.width, 2.0);
            }
            assert_eq!(
                pollster::block_on(gpu_scene(graph.scene()))?.0,
                cpu_scene(graph.scene())?
            );
        }
    }
    Ok(())
}

#[test]
fn literal_components_clip_to_the_viewport_but_all_required_ink_must_survive() -> TestResult {
    let base = page(BondOrder::Double)?;
    let f = &base.fragments()[0];
    let mut atoms = f.atoms().to_vec();
    atoms[0].position = Some(Point2::new(-26.0, 64.0)?);
    atoms[1].position = Some(Point2::new(34.0, 64.0)?);
    let mut doc = Document::default();
    doc.set_page_size(base.id(), base.size())?;
    doc.append_fragments(
        base.id(),
        vec![Fragment::new(f.id(), atoms, f.bonds().to_vec())?],
    )?;
    let before = doc.clone();
    let page = &doc.pages()[0];
    let graph = graph_page_scene(576, 576, page)?;
    assert_eq!(
        graph.hit_test([0.5, 157.0], 0.0)?,
        Some(bondline_render::HitTarget::Bond {
            fragment_id: f.id(),
            bond_id: BondId::new(1).ok_or("bond")?
        })
    );
    assert_eq!(graph.hit_test([50.0, 160.0], 0.0)?, None);
    assert_eq!(
        pollster::block_on(gpu_scene(graph.scene()))?.0,
        cpu_scene(graph.scene())?
    );
    let counts = page.fragments()[0]
        .atoms()
        .iter()
        .map(|a| (a.id, 0))
        .collect();
    assert!(matches!(
        bondline_render::graph_page_scene_with_display_hydrogens(
            576,
            576,
            page,
            &counts,
            &bondline_render::Cancellation::default()
        ),
        Err(bondline_render::Error::LabelPlacement)
    ));
    let hidden = phase_page(BondOrder::Triple, 1.0, 1.5)?;
    assert!(matches!(
        graph_page_scene(256, 256, &hidden),
        Err(bondline_render::Error::BondScale)
    ));
    assert_eq!(doc, before);
    Ok(())
}

#[test]
fn expanded_budget_is_checked_before_geometry_and_cancellation_precedes_it() -> TestResult {
    for (triples, singles, excess) in [(1365, 0, true), (1364, 1, false)] {
        let total = triples + singles;
        let mut doc = Document::default();
        let id = PageId::new(1).ok_or("page")?;
        let atoms = (1..=total + 1)
            .map(|n| {
                Ok(Atom::new(
                    AtomId::new(n).ok_or("atom")?,
                    Element::try_from(6)?,
                    Some(Point2::new(100.0, 100.0)?),
                ))
            })
            .collect::<TestResult<Vec<_>>>()?;
        let bonds = (1..=total)
            .map(|n| {
                Ok(Bond {
                    double_position: bondline_model::DoublePosition::Unspecified,
                    id: BondId::new(n).ok_or("bond")?,
                    begin: AtomId::new(n).ok_or("a")?,
                    end: AtomId::new(n + 1).ok_or("b")?,
                    order: if n <= triples {
                        BondOrder::Triple
                    } else {
                        BondOrder::Single
                    },
                })
            })
            .collect::<TestResult<Vec<_>>>()?;
        doc.append_fragments(
            id,
            vec![Fragment::new(
                FragmentId::new(1).ok_or("fragment")?,
                atoms,
                bonds,
            )?],
        )?;
        let page = &doc.pages()[0];
        let result = graph_page_scene(576, 576, page);
        if excess {
            assert!(matches!(
                result,
                Err(bondline_render::Error::SceneTooComplex)
            ));
        } else {
            assert!(
                matches!(result, Err(bondline_render::Error::InvalidStroke)),
                "4096 actual primitives pass preflight before deliberately degenerate geometry"
            );
        }
        let cancel = bondline_render::Cancellation::default();
        cancel.cancel();
        assert!(matches!(
            bondline_render::graph_page_scene_with_cancel(576, 576, page, &cancel),
            Err(bondline_render::Error::Cancelled)
        ));
    }
    Ok(())
}

#[test]
fn diagonal_3_4_geometry_has_independent_pixel_coverage_and_full_gpu_agreement() -> TestResult {
    for (order, lines) in [
        (
            BondOrder::Double,
            vec![
                ([102.4, 98.2], [150.4, 162.2]),
                ([97.6, 101.8], [145.6, 165.8]),
            ],
        ),
        (
            BondOrder::Triple,
            vec![
                ([104.8, 96.4], [152.8, 160.4]),
                ([100.0, 100.0], [148.0, 164.0]),
                ([95.2, 103.6], [143.2, 167.6]),
            ],
        ),
    ] {
        let base = page(order)?;
        let f = &base.fragments()[0];
        let mut atoms = f.atoms().to_vec();
        atoms[0].position = Some(Point2::new(34.0, 34.0)?);
        atoms[1].position = Some(Point2::new(58.0, 66.0)?);
        let mut doc = Document::default();
        doc.set_page_size(base.id(), base.size())?;
        doc.append_fragments(
            base.id(),
            vec![Fragment::new(f.id(), atoms, f.bonds().to_vec())?],
        )?;
        let graph = graph_page_scene(576, 576, &doc.pages()[0])?;
        for (stroke, (a, b)) in graph
            .bond_strokes(f.id(), BondId::new(1).ok_or("bond")?)
            .zip(&lines)
        {
            for (actual, expected) in stroke
                .begin
                .iter()
                .chain(&stroke.end)
                .zip(a.iter().chain(b))
            {
                assert!((f64::from(*actual) - *expected).abs() <= 1e-5);
            }
        }
        let cpu = cpu_scene(graph.scene())?;
        for y in 90..174 {
            for x in 90..158 {
                let ink = lines.iter().any(|(a, b)| {
                    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                    let (px, py) = (f64::from(x) + 0.5 - a[0], f64::from(y) + 0.5 - a[1]);
                    let along = px * dx + py * dy;
                    let cross = px * dy - py * dx;
                    let length2 = dx * dx + dy * dy;
                    along >= 0.0 && along <= length2 && cross * cross <= length2
                });
                let at = ((y * 576 + x) * 4) as usize;
                assert_eq!(
                    &cpu.rgba[at..at + 4],
                    if ink { &[0, 0, 0, 255] } else { &[255; 4] },
                    "{order:?} diagonal pixel{x},{y}"
                );
            }
        }
        assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, cpu);
    }
    Ok(())
}
