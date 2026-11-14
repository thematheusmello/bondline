use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    PageSize, Point2,
};
use bondline_render::{
    Cancellation, Error, HitTarget, carbon_page_scene, cpu_scene, gpu_scene, graph_page_scene,
    graph_page_scene_with_cancel,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn page(points: &[[f64; 2]], edges: &[[u32; 2]]) -> TestResult<Page> {
    let atoms = points
        .iter()
        .enumerate()
        .map(|(index, [x, y])| {
            Ok(Atom::new(
                AtomId::new(u32::try_from(index + 1)?).ok_or("id")?,
                Element::try_from(6)?,
                Some(Point2::new(*x, *y)?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .iter()
        .enumerate()
        .map(|(index, [begin, end])| {
            Ok(Bond {
                id: BondId::new(u32::try_from(index + 1)?).ok_or("id")?,
                begin: AtomId::new(*begin).ok_or("begin")?,
                end: AtomId::new(*end).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    document.set_page_size(id, PageSize::new(60.0, 60.0)?)?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(document.pages().first().ok_or("page")?.clone())
}

fn changed(page: &Page, mutator: impl FnOnce(&mut Vec<Atom>, &mut Vec<Bond>)) -> TestResult<Page> {
    let fragment = page.fragments().first().ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    let mut bonds = fragment.bonds().to_vec();
    mutator(&mut atoms, &mut bonds);
    let mut doc = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    doc.set_page_size(id, page.size())?;
    doc.append_fragments(id, vec![Fragment::new(fragment.id(), atoms, bonds)?])?;
    Ok(doc.pages().first().ok_or("page")?.clone())
}

fn atom(id: u32) -> TestResult<HitTarget> {
    Ok(HitTarget::Atom {
        fragment_id: FragmentId::new(1).ok_or("fragment")?,
        atom_id: AtomId::new(id).ok_or("atom")?,
    })
}
fn bond(id: u32) -> TestResult<HitTarget> {
    Ok(HitTarget::Bond {
        fragment_id: FragmentId::new(1).ok_or("fragment")?,
        bond_id: BondId::new(id).ok_or("bond")?,
    })
}

#[test]
fn typed_hits_follow_independent_finite_stroke_distances_and_atom_priority() -> TestResult {
    let page = page(
        &[[10.5, 20.5], [30.5, 20.5], [30.5, 40.5]],
        &[[1, 2], [2, 3]],
    )?;
    let graph = graph_page_scene(80, 80, &page)?;
    for (position, target) in [
        ([20.5, 30.5], atom(1)?),
        ([40.5, 30.5], atom(2)?),
        ([40.5, 50.5], atom(3)?),
        ([30.5, 30.5], bond(1)?),
        ([40.5, 40.5], bond(2)?),
        ([30.5, 36.0], bond(1)?),
        ([46.0, 40.5], bond(2)?),
    ] {
        assert_eq!(graph.hit_test(position, 5.0)?, Some(target), "{position:?}");
    }
    for position in [
        [30.5, 36.0001],
        [46.0001, 40.5],
        [30.5, 60.0],
        [30.5, 10.0],
        [-0.001, 30.5],
        [80.0, 30.5],
    ] {
        assert_eq!(graph.hit_test(position, 5.0)?, None, "{position:?}");
    }
    for (position, tolerance) in [
        ([f32::NAN, 30.0], 5.0),
        ([0.0, f32::INFINITY], 5.0),
        ([10.0, 10.0], -1.0),
        ([10.0, 10.0], f32::NAN),
        ([10.0, 10.0], 100001.0),
    ] {
        assert!(matches!(
            graph.hit_test(position, tolerance),
            Err(Error::InvalidHitTest)
        ));
    }
    // Twice the canvas scale keeps a five-canvas-unit tolerance; pointer DPI is not doubled separately.
    let larger = graph_page_scene(160, 160, &page)?;
    assert_eq!(larger.hit_test([41.0, 61.0], 0.0)?, Some(atom(1)?));
    assert_eq!(larger.hit_test([61.0, 67.0], 5.0)?, Some(bond(1)?));
    assert_eq!(larger.hit_test([61.0, 67.0001], 5.0)?, None);
    Ok(())
}

#[test]
fn equal_distance_hits_are_stable_under_storage_and_endpoint_reversal() -> TestResult {
    let page = page(
        &[[10.5, 30.5], [50.5, 30.5], [30.5, 10.5], [30.5, 50.5]],
        &[[1, 2], [3, 4]],
    )?;
    let reversed = changed(&page, |atoms, bonds| {
        atoms.reverse();
        bonds.reverse();
        for bond in bonds {
            std::mem::swap(&mut bond.begin, &mut bond.end);
        }
    })?;
    for page in [&page, &reversed] {
        assert_eq!(
            graph_page_scene(80, 80, page)?.hit_test([40.5, 40.5], 0.0)?,
            Some(bond(1)?)
        );
    }
    let close = self::page(&[[10.5, 20.5], [20.5, 20.5]], &[[1, 2]])?;
    assert_eq!(
        graph_page_scene(80, 80, &close)?.hit_test([25.5, 30.5], 5.0)?,
        Some(atom(1)?)
    );
    Ok(())
}

#[test]
fn clipped_visible_ink_is_pickable_without_invisible_extension_hits() -> TestResult {
    let clipped = page(&[[-20.0, 20.5], [30.5, 20.5]], &[[1, 2]])?;
    let graph = graph_page_scene(80, 80, &clipped)?;
    assert_eq!(graph.hit_test([1.0, 30.5], 0.0)?, Some(bond(1)?));
    assert_eq!(graph.hit_test([-0.01, 30.5], 5.0)?, None);
    let corner = page(&[[-20.0, 1.0], [1.0, -20.0]], &[[1, 2]])?;
    assert_eq!(
        graph_page_scene(80, 80, &corner)?.hit_test([8.0, 0.0], 5.0)?,
        None,
        "outside ink must not be selected through the clipped edge"
    );
    Ok(())
}

#[test]
fn a_wholly_clipped_tangent_is_not_a_visible_bond_target() -> TestResult {
    let page = page(&[[-20.0, -10.5], [100.0, -10.5]], &[[1, 2]])?;
    let graph = graph_page_scene(80, 80, &page)?;
    assert_eq!(graph.hit_test([40.0, 0.0], 5.0)?, None);
    Ok(())
}

#[test]
fn supported_cycles_branches_and_disconnected_graphs_have_one_target_per_atom() -> TestResult {
    for (points, edges) in [
        (
            vec![[10.5, 10.5], [40.5, 10.5], [40.5, 40.5], [10.5, 40.5]],
            vec![[1, 2], [2, 3], [3, 4], [4, 1]],
        ),
        (
            vec![[30.5, 30.5], [10.5, 30.5], [50.5, 30.5], [30.5, 10.5]],
            vec![[1, 2], [1, 3], [1, 4]],
        ),
        (
            vec![[10.5, 10.5], [40.5, 10.5], [40.5, 40.5], [10.5, 40.5]],
            vec![[1, 2], [3, 4]],
        ),
    ] {
        let page = page(&points, &edges)?;
        let graph = graph_page_scene(80, 80, &page)?;
        assert_eq!(graph.scene().primitives.len(), 3 + edges.len());
        for (index, [x, y]) in points.iter().enumerate() {
            assert_eq!(
                graph.hit_test([*x as f32 + 10.0, *y as f32 + 10.0], 0.0)?,
                Some(atom(u32::try_from(index + 1)?)?)
            );
        }
    }
    Ok(())
}

#[test]
fn no_supported_scene_silently_hides_unsupported_atom_metadata_or_bond_orders() -> TestResult {
    let base = page(&[[10.5, 20.5], [40.5, 20.5]], &[[1, 2]])?;
    for kind in 0..5 {
        let page = changed(&base, |atoms, _| {
            if let Some(atom) = atoms.first_mut() {
                match kind {
                    0 => {
                        atom.element = Element::try_from(1).unwrap_or(atom.element);
                        atom.explicit_hydrogens = Some(1);
                    }
                    1 => atom.charge = 1,
                    2 => atom.isotope = std::num::NonZeroU16::new(13),
                    3 => atom.radical = bondline_model::Radical::Doublet,
                    _ => atom.position = None,
                }
            }
        })?;
        assert!(matches!(
            graph_page_scene(80, 80, &page),
            Err(Error::UnsupportedChemicalContent)
        ));
    }
    for order in [
        BondOrder::Quadruple,
        BondOrder::Quintuple,
        BondOrder::Sextuple,
        BondOrder::Half,
        BondOrder::Aromatic,
        BondOrder::TwoAndHalf,
        BondOrder::ThreeAndHalf,
        BondOrder::FourAndHalf,
        BondOrder::FiveAndHalf,
        BondOrder::Dative,
        BondOrder::Ionic,
        BondOrder::Hydrogen,
        BondOrder::ThreeCenter,
    ] {
        let page = changed(&base, |_, bonds| {
            if let Some(bond) = bonds.first_mut() {
                bond.order = order;
            }
        })?;
        assert!(matches!(
            graph_page_scene(80, 80, &page),
            Err(Error::UnsupportedChemicalContent)
        ));
    }
    let coincident = changed(&base, |atoms, _| {
        let position = atoms.first().and_then(|a| a.position);
        if let Some(atom) = atoms.get_mut(1) {
            atom.position = position;
        }
    })?;
    assert!(matches!(
        graph_page_scene(80, 80, &coincident),
        Err(Error::InvalidStroke)
    ));
    let isolated = page(&[[30.0, 30.0]], &[])?;
    assert!(matches!(
        graph_page_scene(80, 80, &isolated),
        Err(Error::UnsupportedChemicalContent)
    ));
    let explicit_zero = changed(&isolated, |atoms, _| {
        if let Some(atom) = atoms.first_mut() {
            atom.explicit_hydrogens = Some(0);
        }
    })?;
    assert_eq!(
        graph_page_scene(80, 80, &explicit_zero)?.hit_test([40.0, 40.0], 0.0)?,
        Some(atom(1)?)
    );
    Ok(())
}

#[test]
fn original_unknown_h_double_and_triple_cases_keep_none_and_have_exact_bundle_hits() -> TestResult {
    let original = page(&[[10.5, 20.5], [40.5, 20.5]], &[[1, 2]])?;
    for (order, ys, ink_rows) in [
        (BondOrder::Double, vec![29.0, 32.0], vec![28, 29, 31, 32]),
        (BondOrder::Triple, vec![27.5, 30.5, 33.5], vec![27, 30, 33]),
    ] {
        let page = changed(&original, |_, bonds| bonds[0].order = order)?;
        let before = page.clone();
        let graph = graph_page_scene(80, 80, &page)?;
        let actual = graph
            .bond_strokes(
                FragmentId::new(1).ok_or("fragment")?,
                BondId::new(1).ok_or("bond")?,
            )
            .cloned()
            .collect::<Vec<_>>();
        let expected = ys
            .iter()
            .map(|y| bondline_render::Stroke {
                begin: [20.5, *y],
                end: [50.5, *y],
                width: 1.0,
                color: [0, 0, 0, 255],
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        let cpu = cpu_scene(graph.scene())?;
        for y in 25..36 {
            let at = ((y * 80 + 35) * 4) as usize;
            assert_eq!(
                &cpu.rgba[at..at + 4],
                if ink_rows.contains(&y) {
                    &[0, 0, 0, 255]
                } else {
                    &[255; 4]
                }
            );
        }
        for y in ys {
            assert_eq!(graph.hit_test([35.0, y], 0.0)?, Some(bond(1)?));
        }
        assert_eq!(
            graph.hit_test(
                [
                    35.0,
                    29.0 + if order == BondOrder::Double { 1.5 } else { 0.0 }
                ],
                0.0
            )?,
            None
        );
        assert!(
            page.fragments()[0]
                .atoms()
                .iter()
                .all(|a| a.explicit_hydrogens.is_none())
        );
        assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, cpu);
        assert_eq!(page, before);
    }
    Ok(())
}

#[test]
fn labels_are_once_per_atom_and_cannot_erase_an_unrelated_bond() -> TestResult {
    let base = page(
        &[[10.0, 30.0], [50.0, 30.0], [30.0, 30.0], [30.0, 50.0]],
        &[[1, 2], [3, 4]],
    )?;
    let masked = changed(&base, |atoms, _| {
        if let Some(atom) = atoms.get_mut(2) {
            atom.explicit_hydrogens = Some(0);
        }
    })?;
    assert!(matches!(
        graph_page_scene(80, 80, &masked),
        Err(Error::LabelOverlap)
    ));
    let base = page(
        &[[10.0, 30.0], [30.0, 30.0], [50.0, 30.0]],
        &[[1, 2], [2, 3]],
    )?;
    let labelled = changed(&base, |atoms, _| {
        if let Some(atom) = atoms.get_mut(1) {
            atom.explicit_hydrogens = Some(0);
        }
    })?;
    let graph = graph_page_scene(80, 80, &labelled)?;
    let masks=graph.scene().primitives.iter().filter(|primitive|matches!(primitive,bondline_render::Primitive::Rectangle(r) if r.color==[255;4] && r.height<20)).count();
    assert_eq!(masks, 1, "one carbon label despite two incident bonds");
    assert_eq!(graph.hit_test([40.0, 40.0], 0.0)?, Some(atom(2)?));
    Ok(())
}

#[test]
fn bounded_hit_lists_cancel_before_raster_or_gpu_work() -> TestResult {
    let base = page(
        &[[10.5, 20.5], [30.5, 20.5], [30.5, 40.5]],
        &[[1, 2], [2, 3]],
    )?;
    let before = base.clone();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        graph_page_scene_with_cancel(80, 80, &base, &cancel),
        Err(Error::Cancelled)
    ));
    assert_eq!(base, before);
    let points = (0..2048)
        .map(|i| [10.0 + f64::from(i) * 0.001, 20.5])
        .collect::<Vec<_>>();
    let edges = (1..2048).map(|i| [i, i + 1]).collect::<Vec<_>>();
    let chain = page(&points, &edges)?;
    let mut a = Atom::new(
        AtomId::new(2049).ok_or("atom")?,
        Element::try_from(6)?,
        Some(Point2::new(50.0, 50.0)?),
    );
    a.explicit_hydrogens = Some(0);
    let exact = changed(&chain, |atoms, _| atoms.push(a))?;
    assert!(graph_page_scene(80, 80, &exact).is_ok());
    let mut a = Atom::new(
        AtomId::new(2050).ok_or("atom")?,
        Element::try_from(6)?,
        Some(Point2::new(30.0, 50.0)?),
    );
    a.explicit_hydrogens = Some(0);
    let excess = changed(&exact, |atoms, _| atoms.push(a))?;
    assert!(matches!(
        graph_page_scene(80, 80, &excess),
        Err(Error::SceneTooComplex)
    ));
    Ok(())
}

#[test]
fn glyph_spans_count_toward_the_scene_budget_without_a_large_hit_list() -> TestResult {
    let mut atoms = Vec::new();
    for index in 0..129_u32 {
        let mut atom = Atom::new(
            AtomId::new(index + 1).ok_or("atom")?,
            Element::try_from(6)?,
            Some(Point2::new(
                100.0 + 60.0 * f64::from(index % 16),
                100.0 + 40.0 * f64::from(index / 16),
            )?),
        );
        atom.explicit_hydrogens = Some(u16::MAX);
        atoms.push(atom);
    }
    let mut doc = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    doc.set_page_size(id, PageSize::new(2048.0, 2048.0)?)?;
    doc.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![],
        )?],
    )?;
    assert!(matches!(
        graph_page_scene(2048, 2048, doc.pages().first().ok_or("page")?),
        Err(Error::SceneTooComplex)
    ));
    Ok(())
}

#[test]
fn three_carbon_l_has_the_independent_41_pixel_chemical_mask() -> TestResult {
    let page = page(
        &[[10.5, 20.5], [30.5, 20.5], [30.5, 40.5]],
        &[[1, 2], [2, 3]],
    )?;
    let scene = carbon_page_scene(80, 80, &page)?;
    let mut expected = Vec::new();
    for y in 0..80 {
        for x in 0..80 {
            let mut shade = 232;
            if (13..73).contains(&x) && (14..74).contains(&y) {
                shade = 210;
            }
            if (10..70).contains(&x) && (10..70).contains(&y) {
                shade = 196;
            }
            if (11..69).contains(&x) && (11..69).contains(&y) {
                shade = 255;
            }
            if y == 30 && (20..=40).contains(&x) || x == 40 && (30..=50).contains(&y) {
                shade = 0;
            }
            expected.extend_from_slice(&[shade, shade, shade, 255]);
        }
    }
    assert_eq!(cpu_scene(&scene)?.rgba, expected);
    assert_eq!(pollster::block_on(gpu_scene(&scene))?.0.rgba, expected);
    Ok(())
}
