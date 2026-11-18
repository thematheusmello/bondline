use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    PageSize, Point2,
};
use bondline_render::{
    Cancellation, HitTarget, PageView, Primitive, Rectangle, Scene, Stroke, graph_page_scene,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn pair(order: BondOrder) -> TestResult<Page> {
    let a = AtomId::new(11).ok_or("atom")?;
    let b = AtomId::new(19).ok_or("atom")?;
    let page = PageId::new(1).ok_or("page")?;
    let mut document = Document::default();
    document.set_page_size(page, PageSize::new(128.0, 128.0)?)?;
    document.append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(7).ok_or("fragment")?,
            vec![
                Atom::new(a, Element::try_from(6)?, Some(Point2::new(32.0, 64.0)?)),
                Atom::new(b, Element::try_from(6)?, Some(Point2::new(96.0, 64.0)?)),
            ],
            vec![Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(23).ok_or("bond")?,
                begin: a,
                end: b,
                order,
            }],
        )?],
    )?;
    Ok(document.pages().first().ok_or("page")?.clone())
}

#[test]
fn fit_keeps_the_literal_legacy_paper_geometry_and_application_hits() -> TestResult {
    let page = pair(BondOrder::Single)?;
    let view = PageView::fit(320, 240, page.size())?;
    assert_eq!(view.source_size(), [320, 240]);
    assert_eq!(
        view.canvas_position(Point2::new(32.0, 64.0)?)?,
        [115.0, 120.0]
    );
    assert_eq!(
        view.canvas_position(Point2::new(96.0, 64.0)?)?,
        [205.0, 120.0]
    );
    assert_eq!(
        view.document_position([160.0, 120.0])?,
        Point2::new(64.0, 64.0)?
    );
    assert_eq!(view.stroke_width(), 1.40625);
    assert!(!view.contains([70.0, 30.0]));
    assert!(view.contains([70.5, 30.5]));
    let graph = view.project(graph_page_scene(320, 240, &page)?, &Cancellation::default())?;
    let rect = |x, y, width, height, color| {
        Primitive::Rectangle(Rectangle {
            x,
            y,
            width,
            height,
            color,
        })
    };
    assert_eq!(
        graph.scene(),
        &Scene {
            width: 320,
            height: 240,
            background: [232, 232, 232, 255],
            primitives: vec![
                rect(73, 34, 180, 180, [210, 210, 210, 255]),
                rect(70, 30, 180, 180, [196, 196, 196, 255]),
                rect(71, 31, 178, 178, [255; 4]),
                Primitive::Stroke(Stroke {
                    begin: [115.0, 120.0],
                    end: [205.0, 120.0],
                    width: 1.40625,
                    color: [0, 0, 0, 255],
                }),
            ],
        }
    );
    assert_eq!(
        graph.hit_test([115.0, 120.0], 0.0)?,
        Some(HitTarget::Atom {
            fragment_id: FragmentId::new(7).ok_or("fragment")?,
            atom_id: AtomId::new(11).ok_or("atom")?,
        })
    );
    assert_eq!(
        graph.hit_test([160.0, 120.0], 0.0)?,
        Some(HitTarget::Bond {
            fragment_id: FragmentId::new(7).ok_or("fragment")?,
            bond_id: BondId::new(23).ok_or("bond")?,
        })
    );
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

use bondline_render::{Error, cpu_scene, gpu_scene};
use std::collections::BTreeMap;

fn changed(
    page: &Page,
    size: PageSize,
    change: impl FnOnce(&mut Vec<Atom>, &mut Vec<Bond>),
) -> TestResult<Page> {
    let fragment = page.fragments().first().ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    let mut bonds = fragment.bonds().to_vec();
    change(&mut atoms, &mut bonds);
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    document.set_page_size(id, size)?;
    document.append_fragments(id, vec![Fragment::new(fragment.id(), atoms, bonds)?])?;
    Ok(document.pages().first().ok_or("page")?.clone())
}

fn image(width: u32, height: u32, black: impl Fn(u32, u32) -> bool) -> Vec<u8> {
    (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .flat_map(|(x, y)| {
            if black(x, y) {
                [0, 0, 0, 255]
            } else {
                [255; 4]
            }
        })
        .collect()
}

fn bond_target() -> TestResult<HitTarget> {
    Ok(HitTarget::Bond {
        fragment_id: FragmentId::new(7).ok_or("fragment")?,
        bond_id: BondId::new(23).ok_or("bond")?,
    })
}

#[test]
fn uniform_two_hundred_percent_has_literal_component_pixels_and_stable_logical_hits() -> TestResult
{
    for (order, rows) in [
        (BondOrder::Single, &[59, 60][..]),
        (BondOrder::Double, &[56, 57, 62, 63][..]),
        (BondOrder::Triple, &[53, 54, 59, 60, 65, 66][..]),
    ] {
        let page = pair(order)?;
        let view = PageView::zoom(160, 120, page.size(), 200.0, Point2::new(64.0, 64.0)?)?;
        assert_eq!(view.source_size(), [320, 320]);
        assert_eq!(view.zoom_percent(), 200.0);
        assert_eq!(view.stroke_width(), 2.0);
        assert_eq!(
            view.canvas_position(Point2::new(32.0, 64.0)?)?,
            [16.0, 60.0]
        );
        assert_eq!(
            view.canvas_position(Point2::new(96.0, 64.0)?)?,
            [144.0, 60.0]
        );
        assert_eq!(
            view.document_position([80.0, 60.0])?,
            Point2::new(64.0, 64.0)?
        );
        let graph = view.graph_scene(&page, &Cancellation::default())?;
        let expected = image(160, 120, |x, y| (16..144).contains(&x) && rows.contains(&y));
        let cpu = cpu_scene(graph.scene())?;
        assert_eq!(cpu.rgba, expected, "literal {order:?} strips");
        let (gpu, _) = pollster::block_on(gpu_scene(graph.scene()))?;
        assert_eq!(gpu.rgba, expected, "GPU {order:?} strips");
        assert_ne!(
            cpu.rgba,
            image(160, 120, |x, y| (17..145).contains(&x) && rows.contains(&y)),
            "one-pixel translated geometry is a rejected control"
        );
        assert_eq!(graph.label_em_px(), None);
        assert_eq!(
            graph.hit_test(
                [80.0, rows.first().copied().ok_or("row")? as f32 + 0.5],
                0.0
            )?,
            Some(bond_target()?)
        );
        if order == BondOrder::Double {
            assert_eq!(graph.hit_test([80.0, 60.0], 0.0)?, None);
        }
        assert_eq!(
            graph.hit_test([16.0, 60.0], 0.0)?,
            Some(HitTarget::Atom {
                fragment_id: FragmentId::new(7).ok_or("fragment")?,
                atom_id: AtomId::new(11).ok_or("atom")?,
            })
        );
        assert_eq!(
            graph
                .bond_strokes(
                    FragmentId::new(7).ok_or("fragment")?,
                    BondId::new(23).ok_or("bond")?
                )
                .count(),
            rows.len() / 2
        );
    }
    Ok(())
}

#[test]
fn pan_clips_visible_ink_and_hits_without_shortening_a_diagonal_butt_axis() -> TestResult {
    let base = pair(BondOrder::Single)?;
    for (center_x, x_range) in [(80.0, 0..112), (48.0, 48..160)] {
        let view = PageView::zoom(160, 120, base.size(), 200.0, Point2::new(center_x, 64.0)?)?;
        let graph = view.graph_scene(&base, &Cancellation::default())?;
        assert_eq!(
            cpu_scene(graph.scene())?.rgba,
            image(160, 120, |x, y| x_range.contains(&x)
                && [59, 60].contains(&y))
        );
        assert!(!view.contains([-0.01, 60.0]));
        assert!(!view.contains([160.0, 60.0]));
        assert_eq!(graph.hit_test([-0.01, 60.0], 5.0)?, None);
        assert_eq!(graph.hit_test([160.0, 60.0], 5.0)?, None);
        assert_eq!(graph.hit_test([80.0, 60.0], 0.0)?, Some(bond_target()?));
    }
    let diagonal = changed(&base, base.size(), |atoms, _| {
        for (atom, point) in atoms.iter_mut().zip([[32.0, 32.0], [96.0, 96.0]]) {
            atom.position = Point2::new(point[0], point[1]).ok();
        }
    })?;
    let view = PageView::zoom(160, 120, diagonal.size(), 400.0, Point2::new(80.0, 64.0)?)?;
    let graph = view.graph_scene(&diagonal, &Cancellation::default())?;
    let components = graph
        .bond_strokes(
            FragmentId::new(7).ok_or("fragment")?,
            BondId::new(23).ok_or("bond")?,
        )
        .collect::<Vec<_>>();
    let stroke = components.first().ok_or("stroke")?;
    assert_eq!(stroke.begin, [-112.0, -68.0]);
    assert_eq!(stroke.end, [144.0, 188.0]);
    assert_eq!(stroke.width, 4.0);
    // The 45-degree line is y=x+44; perpendicular half-width2 covers integer differences ±2.
    let expected = image(160, 120, |x, y| {
        (i64::from(y) - i64::from(x) - 44).abs() <= 2
    });
    assert_eq!(cpu_scene(graph.scene())?.rgba, expected);
    let (gpu, _) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert_eq!(gpu.rgba, expected);
    assert_eq!(graph.hit_test([0.5, 42.5], 0.0)?, Some(bond_target()?));
    assert_eq!(graph.hit_test([159.0, 119.0], 0.0)?, None);
    Ok(())
}

#[test]
fn fractional_zoom_uses_one_uniform_scale_not_the_ceil_paper_axis_ratios() -> TestResult {
    let page = changed(
        &pair(BondOrder::Single)?,
        PageSize::new(123.5, 81.25)?,
        |_, _| {},
    )?;
    let view = PageView::zoom(320, 240, page.size(), 87.5, Point2::new(61.75, 40.625)?)?;
    assert_eq!(view.source_size(), [133, 96]);
    assert_eq!(view.zoom_percent(), 87.5);
    assert_eq!(view.stroke_width(), 0.875);
    let point = Point2::new(10.5, 20.25)?;
    assert_eq!(view.canvas_position(point)?, [115.1875, 101.71875]);
    assert_eq!(view.document_position([115.1875, 101.71875])?, point);
    assert!(view.contains([214.0, 100.0]));
    assert!(!view.contains([214.125, 100.0]));
    let graph = view.graph_scene(&page, &Cancellation::default())?;
    let stroke = graph
        .bond_strokes(
            FragmentId::new(7).ok_or("fragment")?,
            BondId::new(23).ok_or("bond")?,
        )
        .next()
        .ok_or("stroke")?;
    assert_eq!(stroke.begin, [134.0, 140.0]);
    assert_eq!(stroke.end, [190.0, 140.0]);
    assert_eq!(stroke.width, 0.875);
    assert_eq!((stroke.end[0] - stroke.begin[0]) / 64.0, 0.875);
    Ok(())
}

#[test]
fn clipped_literal_labels_are_new_geometry_not_resampled_fit_pixels() -> TestResult {
    let oxygen = Element::try_from(8)?;
    let page = changed(
        &pair(BondOrder::Single)?,
        PageSize::new(128.0, 128.0)?,
        |atoms, bonds| {
            atoms.truncate(1);
            if let Some(atom) = atoms.first_mut() {
                atom.element = oxygen;
                atom.position = Point2::new(64.0, 64.0).ok();
                atom.explicit_hydrogens = Some(1);
            }
            bonds.clear();
        },
    )?;
    let fit = PageView::fit(320, 240, page.size())?;
    assert_eq!(
        fit.graph_scene(&page, &Cancellation::default())?.scene(),
        graph_page_scene(320, 240, &page)?.scene()
    );
    let source = graph_page_scene(320, 320, &page)?;
    assert_eq!(source.label_em_px(), Some(24.0));
    let full = cpu_scene(source.scene())?;
    let view = PageView::zoom(40, 48, page.size(), 200.0, Point2::new(64.0, 64.0)?)?;
    let graph = view.graph_scene(&page, &Cancellation::default())?;
    assert_eq!(graph.label_em_px(), Some(24.0));
    let mut expected = Vec::new();
    for y in 136..184 {
        let start = ((y * 320 + 140) * 4) as usize;
        let end = ((y * 320 + 180) * 4) as usize;
        expected.extend_from_slice(full.rgba.get(start..end).ok_or("source crop")?);
    }
    let actual = cpu_scene(graph.scene())?;
    assert_eq!(actual.rgba, expected, "literal integer camera crop");
    let black = |rgba: &[u8]| {
        rgba.chunks_exact(4)
            .filter(|pixel| *pixel == [0, 0, 0, 255])
            .count()
    };
    assert!(black(&actual.rgba) > 0);
    assert!(
        black(&actual.rgba) < black(&full.rgba),
        "the destination really clips glyph ink"
    );
    let (gpu, _) = pollster::block_on(gpu_scene(graph.scene()))?;
    assert_eq!(gpu.rgba, expected);
    assert_eq!(
        graph.hit_test([20.0, 24.0], 0.0)?,
        Some(HitTarget::Atom {
            fragment_id: FragmentId::new(7).ok_or("fragment")?,
            atom_id: AtomId::new(11).ok_or("atom")?,
        })
    );
    assert_eq!(
        page.fragments()
            .first()
            .ok_or("fragment")?
            .atoms()
            .first()
            .ok_or("atom")?
            .explicit_hydrogens,
        Some(1)
    );
    Ok(())
}

#[test]
fn complete_display_counts_and_full_source_guards_remain_authoritative_before_camera_clip()
-> TestResult {
    let page = pair(BondOrder::Single)?;
    let counts = BTreeMap::from([
        (AtomId::new(11).ok_or("atom")?, 3),
        (AtomId::new(19).ok_or("atom")?, 3),
    ]);
    let view = PageView::zoom(160, 120, page.size(), 200.0, Point2::new(64.0, 64.0)?)?;
    assert_eq!(
        view.graph_scene_with_display_hydrogens(&page, &counts, &Cancellation::default())?
            .label_em_px(),
        Some(24.0)
    );
    assert!(
        page.fragments()
            .first()
            .ok_or("fragment")?
            .atoms()
            .iter()
            .all(|atom| atom.explicit_hydrogens.is_none())
    );
    assert!(matches!(
        view.graph_scene_with_display_hydrogens(&page, &BTreeMap::new(), &Cancellation::default()),
        Err(Error::UnsupportedChemicalContent)
    ));
    let tiny = PageView::zoom(160, 120, page.size(), 5.0, Point2::new(0.0, 0.0)?)?;
    let labeled = changed(&page, page.size(), |atoms, _| {
        for atom in atoms {
            atom.explicit_hydrogens = Some(3);
        }
    })?;
    assert!(matches!(
        tiny.graph_scene(&labeled, &Cancellation::default()),
        Err(Error::LabelScale)
    ));
    assert!(matches!(
        tiny.graph_scene(&pair(BondOrder::Triple)?, &Cancellation::default()),
        Err(Error::BondScale)
    ));
    let overlap = changed(&labeled, page.size(), |atoms, _| {
        for (atom, x) in atoms.iter_mut().zip([64.0, 64.25]) {
            atom.position = Point2::new(x, 64.0).ok();
        }
    })?;
    let offscreen = PageView::zoom(40, 48, page.size(), 200.0, Point2::new(0.0, 0.0)?)?;
    assert!(matches!(
        offscreen.graph_scene(&overlap, &Cancellation::default()),
        Err(Error::LabelOverlap)
    ));
    Ok(())
}

#[test]
fn virtual_geometry_can_exceed_readback_size_without_allocating_a_virtual_frame() -> TestResult {
    let page = changed(
        &pair(BondOrder::Single)?,
        PageSize::new(1000.0, 1000.0)?,
        |_, _| {},
    )?;
    let view = PageView::zoom(100, 100, page.size(), 400.0, Point2::new(64.0, 64.0)?)?;
    assert_eq!(view.source_size(), [4064, 4064]);
    let graph = view.graph_scene(&page, &Cancellation::default())?;
    assert_eq!(
        cpu_scene(graph.scene())?.rgba,
        image(100, 100, |_, y| (48..52).contains(&y))
    );
    let huge = PageView::zoom(
        100_000,
        100_000,
        PageSize::new(128.0, 128.0)?,
        1.0,
        Point2::new(64.0, 64.0)?,
    )?;
    assert_eq!(huge.source_size(), [2, 2]);
    let graph = huge.graph_scene(&pair(BondOrder::Single)?, &Cancellation::default())?;
    assert_eq!(
        [graph.scene().width, graph.scene().height],
        [100_000, 100_000]
    );
    assert!(matches!(
        cpu_scene(graph.scene()),
        Err(Error::InvalidDimensions)
    ));
    Ok(())
}

#[test]
fn bounded_view_rejects_invalid_parameters_mismatched_sources_and_unsafe_source_strokes()
-> TestResult {
    let page = pair(BondOrder::Single)?;
    for percent in [f64::NAN, f64::INFINITY, 0.999, 999.000_001] {
        assert!(matches!(
            PageView::zoom(160, 120, page.size(), percent, Point2::new(64.0, 64.0)?),
            Err(Error::InvalidView)
        ));
    }
    for point in [[-0.01, 64.0], [128.01, 64.0], [64.0, -0.01], [64.0, 128.01]] {
        assert!(matches!(
            PageView::zoom(
                160,
                120,
                page.size(),
                100.0,
                Point2::new(point[0], point[1])?
            ),
            Err(Error::InvalidView)
        ));
    }
    assert!(matches!(
        PageView::fit(0, 120, page.size()),
        Err(Error::InvalidViewport)
    ));
    assert!(matches!(
        PageView::zoom(100_001, 120, page.size(), 100.0, Point2::new(64.0, 64.0)?),
        Err(Error::InvalidViewport)
    ));
    assert!(matches!(
        PageView::zoom(
            160,
            120,
            PageSize::new(100_000.0, 100_000.0)?,
            999.0,
            Point2::new(64.0, 64.0)?
        ),
        Err(Error::InvalidViewport)
    ));
    let view = PageView::zoom(160, 120, page.size(), 200.0, Point2::new(64.0, 64.0)?)?;
    assert!(matches!(
        view.project(graph_page_scene(80, 80, &page)?, &Cancellation::default()),
        Err(Error::InvalidView)
    ));
    let other = changed(&page, PageSize::new(127.0, 128.0)?, |_, _| {})?;
    assert!(matches!(
        view.graph_scene(&other, &Cancellation::default()),
        Err(Error::InvalidView)
    ));
    let outside = changed(&page, page.size(), |atoms, _| {
        if let Some(atom) = atoms.first_mut() {
            atom.position = Point2::new(-64.0, 64.0).ok();
        }
    })?;
    assert!(
        PageView::fit(320, 240, outside.size())?
            .graph_scene(&outside, &Cancellation::default())
            .is_ok(),
        "legacy Fit does not acquire the new safe-source restriction"
    );
    assert!(matches!(
        view.graph_scene(&outside, &Cancellation::default()),
        Err(Error::UnsupportedChemicalContent)
    ));
    let long = changed(&page, PageSize::new(99_936.0, 256.0)?, |atoms, _| {
        for (atom, x) in atoms.iter_mut().zip([0.0, 99_936.0]) {
            atom.position = Point2::new(x, 128.0).ok();
        }
    })?;
    assert!(graph_page_scene(100_000, 320, &long).is_ok());
    let long_view = PageView::zoom(100_000, 1000, long.size(), 100.0, Point2::new(0.0, 128.0)?)?;
    assert!(matches!(
        long_view.graph_scene(&long, &Cancellation::default()),
        Err(Error::InvalidStroke)
    ));
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        view.graph_scene(&page, &cancel),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        view.project(graph_page_scene(320, 320, &page)?, &cancel),
        Err(Error::Cancelled)
    ));
    Ok(())
}

#[test]
fn view_entry_keeps_the_existing_whole_source_hit_budget_preflight() -> TestResult {
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    document.set_page_size(id, PageSize::new(128.0, 128.0)?)?;
    let atoms = (1..=4097)
        .map(|id| {
            Ok(Atom::new(
                AtomId::new(id).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(64.0, 64.0)?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(7).ok_or("fragment")?,
            atoms,
            vec![],
        )?],
    )?;
    let page = document.pages().first().ok_or("page")?;
    let view = PageView::zoom(40, 48, page.size(), 200.0, Point2::new(0.0, 0.0)?)?;
    assert!(matches!(
        view.graph_scene(page, &Cancellation::default()),
        Err(Error::SceneTooComplex)
    ));
    Ok(())
}

#[test]
fn a_camera_in_a_valid_double_gap_does_not_reclassify_clipping_as_hidden_chemistry() -> TestResult {
    let page = pair(BondOrder::Double)?;
    let view = PageView::zoom(160, 2, page.size(), 200.0, Point2::new(64.0, 64.0)?)?;
    let graph = view.graph_scene(&page, &Cancellation::default())?;
    let components = graph
        .bond_strokes(
            FragmentId::new(7).ok_or("fragment")?,
            BondId::new(23).ok_or("bond")?,
        )
        .collect::<Vec<_>>();
    assert_eq!(components.len(), 2);
    assert_eq!(components.first().ok_or("first")?.begin, [16.0, -2.0]);
    assert_eq!(components.last().ok_or("last")?.begin, [16.0, 4.0]);
    assert_eq!(cpu_scene(graph.scene())?.rgba, image(160, 2, |_, _| false));
    assert_eq!(graph.hit_test([80.0, 1.0], 5.0)?, None);
    assert_eq!(
        page.fragments()
            .first()
            .ok_or("fragment")?
            .bonds()
            .first()
            .ok_or("bond")?
            .order,
        BondOrder::Double
    );
    Ok(())
}

#[test]
fn large_authored_positions_map_to_the_exact_projected_atom_anchor_before_bond_picking()
-> TestResult {
    let center = Point2::new(10_000.000_1, 10_000.000_1)?;
    let first = Point2::new(9_990.000_1, 10_000.000_1)?;
    let last = Point2::new(10_010.000_1, 10_010.000_1)?;
    let carbon = Element::try_from(6)?;
    let third = AtomId::new(31).ok_or("third")?;
    let middle_id = AtomId::new(19).ok_or("middle")?;
    let third_bond = BondId::new(37).ok_or("bond")?;
    let page = changed(
        &pair(BondOrder::Single)?,
        PageSize::new(20_000.0, 20_000.0)?,
        |atoms, bonds| {
            for (atom, point) in atoms.iter_mut().zip([first, center]) {
                atom.position = Some(point);
            }
            atoms.push(Atom::new(third, carbon, Some(last)));
            bonds.push(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: third_bond,
                begin: middle_id,
                end: third,
                order: BondOrder::Single,
            });
        },
    )?;
    let view = PageView::zoom(100, 100, page.size(), 100.0, center)?;
    assert_eq!(view.source_size(), [20_064, 20_064]);
    let graph = view.graph_scene(&page, &Cancellation::default())?;
    let middle = HitTarget::Atom {
        fragment_id: FragmentId::new(7).ok_or("fragment")?,
        atom_id: middle_id,
    };
    assert_eq!(graph.hit_test([50.0, 50.0], 0.0)?, Some(middle));
    let mapped = view.canvas_position(center)?;
    assert_eq!(
        graph.hit_test(mapped, 0.0)?,
        Some(middle),
        "forward conversion must select the actual zero-tolerance atom, not the adjoining bond"
    );
    assert_eq!(mapped, [50.0, 50.0]);
    assert_eq!(
        view.document_position(mapped)?,
        Point2::new(10_000.0, 10_000.0)?
    );
    assert_eq!(
        page.fragments()
            .first()
            .ok_or("fragment")?
            .atoms()
            .get(1)
            .ok_or("middle")?
            .position,
        Some(center)
    );
    Ok(())
}
