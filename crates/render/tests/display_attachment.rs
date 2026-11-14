use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    PageSize, Point2,
};
use bondline_render::{
    Cancellation, Error, HitTarget, Primitive, cpu_scene, gpu_scene, graph_page_scene,
    graph_page_scene_with_display_hydrogens,
};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn opposed(element: u8, hydrogens: u16) -> TestResult<Page> {
    let mut atoms = Vec::new();
    for (id, number, x, h) in [
        (1, 6, 40.0, None),
        (2, element, 128.0, Some(hydrogens)),
        (3, 6, 216.0, None),
    ] {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("id")?,
            Element::try_from(number)?,
            Some(Point2::new(x, 128.0)?),
        );
        atom.explicit_hydrogens = h;
        atoms.push(atom);
    }
    let bonds = [(1, 1, 2), (2, 2, 3)]
        .into_iter()
        .map(|(id, a, b)| {
            Ok(Bond {
                id: BondId::new(id).ok_or("bond")?,
                begin: AtomId::new(a).ok_or("begin")?,
                end: AtomId::new(b).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    document.set_page_size(id, PageSize::new(256.0, 256.0)?)?;
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

fn drawing(specs: &[(u32, u8, [f64; 2], Option<u16>)], edges: &[[u32; 2]]) -> TestResult<Page> {
    let atoms = specs
        .iter()
        .map(|(id, z, p, h)| {
            let mut atom = Atom::new(
                AtomId::new(*id).ok_or("id")?,
                Element::try_from(*z)?,
                Some(Point2::new(p[0], p[1])?),
            );
            atom.explicit_hydrogens = *h;
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .iter()
        .enumerate()
        .map(|(i, [a, b])| {
            Ok(Bond {
                id: BondId::new(u32::try_from(i + 1)?).ok_or("bond")?,
                begin: AtomId::new(*a).ok_or("begin")?,
                end: AtomId::new(*b).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut document = Document::default();
    let page = PageId::new(1).ok_or("page")?;
    document.set_page_size(page, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(document.pages().first().ok_or("page")?.clone())
}

#[test]
fn opposed_bonds_approach_the_element_without_passing_through_its_hydrogens() -> TestResult {
    for (element, hydrogens) in [(7, 1), (6, 2)] {
        let page = opposed(element, hydrogens)?;
        let before = page.clone();
        let counts = [(1, 3), (2, hydrogens), (3, 3)]
            .into_iter()
            .map(|(id, h)| Ok((AtomId::new(id).ok_or("atom")?, h)))
            .collect::<TestResult<BTreeMap<_, _>>>()?;
        let graph = graph_page_scene_with_display_hydrogens(
            576,
            576,
            &page,
            &counts,
            &Cancellation::default(),
        )?;
        let frame = cpu_scene(graph.scene())?;
        assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, frame);
        // Page scale is exactly2: the unchanged chemical anchor is(288,288).
        // A bond20canvas units from either side is beyond the N/C glyph's
        // published <=14-unit ink width but within a misplaced horizontal H.
        for x in [268, 308] {
            assert_eq!(
                graph.hit_test([x as f32, 288.0], 0.0)?,
                Some(HitTarget::Bond {
                    fragment_id: FragmentId::new(1).ok_or("fragment")?,
                    bond_id: BondId::new(if x < 288 { 1 } else { 2 }).ok_or("bond")?
                }),
                "hydrogen label covers the incident attachment corridor"
            );
            let at = ((288 * 576 + x) * 4) as usize;
            assert_eq!(
                frame.rgba.get(at..at + 4),
                Some([0, 0, 0, 255].as_slice()),
                "Z{element}/H{hydrogens}: bond at{x} was obscured by H instead of meeting the element"
            );
        }
        assert!(graph.scene().primitives.iter().any(|p| matches!(p, Primitive::Rectangle(r) if r.color == [0,0,0,255] && (r.y < 274 || r.y > 306))), "the H must remain visible on a separate line");
        assert_eq!(page, before);
    }
    Ok(())
}

#[test]
fn terminal_oh_nh2_keep_existing_mirrored_vertical_pixels_and_source_unknowns() -> TestResult {
    for (z, h) in [(8, 1), (7, 2)] {
        for neighbor in [[40.0, 128.0], [216.0, 128.0], [128.0, 40.0], [128.0, 216.0]] {
            let literal = drawing(
                &[(1, z, [128.0, 128.0], Some(h)), (2, 6, neighbor, Some(3))],
                &[[1, 2]],
            )?;
            let unknown = drawing(
                &[(1, z, [128.0, 128.0], None), (2, 6, neighbor, None)],
                &[[2, 1]],
            )?;
            let before = unknown.clone();
            let counts = BTreeMap::from([
                (AtomId::new(1).ok_or("id")?, h),
                (AtomId::new(2).ok_or("id")?, 3),
            ]);
            let graph = graph_page_scene_with_display_hydrogens(
                576,
                576,
                &unknown,
                &counts,
                &Cancellation::default(),
            )?;
            let cpu = cpu_scene(graph.scene())?;
            let expected = cpu_scene(graph_page_scene(576, 576, &literal)?.scene())?;
            assert!(
                expected == cpu,
                "Z{z}/neighbor{neighbor:?}: first differing RGBA byte={:?}",
                expected
                    .rgba
                    .iter()
                    .zip(&cpu.rgba)
                    .position(|(a, b)| a != b)
            );
            assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, cpu);
            assert_eq!(
                graph.hit_test([288.0, 288.0], 0.0)?,
                Some(HitTarget::Atom {
                    fragment_id: FragmentId::new(1).ok_or("fragment")?,
                    atom_id: AtomId::new(1).ok_or("atom")?
                })
            );
            assert_eq!(unknown, before);
            assert!(
                unknown.fragments()[0]
                    .atoms()
                    .iter()
                    .all(|a| a.explicit_hydrogens.is_none())
            );
        }
    }
    Ok(())
}

#[test]
fn source_bond_directions_choose_the_free_second_line_and_never_erase_a_short_bond() -> TestResult {
    for (third_y, above) in [(40.0, false), (216.0, true)] {
        // Literal methine geometry: three C bonds and one supplied H. The third
        // source bond selects below/above; no coordinates are rotated or repaired.
        let page = drawing(
            &[
                (1, 6, [128.0, 128.0], Some(1)),
                (2, 6, [40.0, 128.0], Some(3)),
                (3, 6, [216.0, 128.0], Some(3)),
                (4, 6, [128.0, third_y], Some(3)),
            ],
            &[[1, 2], [1, 3], [1, 4]],
        )?;
        let before = page.clone();
        let counts = page.fragments()[0]
            .atoms()
            .iter()
            .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("H")?)))
            .collect::<TestResult<BTreeMap<_, _>>>()?;
        let graph = graph_page_scene_with_display_hydrogens(
            576,
            576,
            &page,
            &counts,
            &Cancellation::default(),
        )?;
        let hpoint = [288.0, if above { 264.0 } else { 312.0 }];
        assert_eq!(
            graph.hit_test(hpoint, 0.0)?,
            Some(HitTarget::Atom {
                fragment_id: FragmentId::new(1).ok_or("fragment")?,
                atom_id: AtomId::new(1).ok_or("atom")?
            })
        );
        assert_eq!(page, before);
    }
    let short = drawing(
        &[
            (1, 6, [128.0, 128.0], Some(0)),
            (2, 7, [129.0, 128.0], Some(0)),
        ],
        &[[1, 2]],
    )?;
    let counts = BTreeMap::from([
        (AtomId::new(1).ok_or("id")?, 0),
        (AtomId::new(2).ok_or("id")?, 0),
    ]);
    assert!(matches!(
        graph_page_scene_with_display_hydrogens(
            576,
            576,
            &short,
            &counts,
            &Cancellation::default()
        ),
        Err(Error::LabelOverlap)
    ));
    Ok(())
}
