use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    PageSize, Point2,
};
use bondline_render::{Error, HitTarget, Primitive, cpu_scene, gpu_scene, graph_page_scene};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn atom(id: u32, element: u8, position: [f64; 2], hydrogens: Option<u16>) -> TestResult<Atom> {
    let mut atom = Atom::new(
        AtomId::new(id).ok_or("atom id")?,
        Element::try_from(element)?,
        Some(Point2::new(position[0], position[1])?),
    );
    atom.explicit_hydrogens = hydrogens;
    Ok(atom)
}

fn page(atoms: Vec<Atom>, edges: &[[u32; 2]]) -> TestResult<Page> {
    let bonds = edges
        .iter()
        .enumerate()
        .map(|(index, [begin, end])| {
            Ok(Bond {
                id: BondId::new(u32::try_from(index + 1)?).ok_or("bond id")?,
                begin: AtomId::new(*begin).ok_or("begin")?,
                end: AtomId::new(*end).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut document = Document::default();
    let page_id = PageId::new(1).ok_or("page id")?;
    document.set_page_size(page_id, PageSize::new(256.0, 128.0)?)?;
    document.append_fragments(
        page_id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment id")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(document.pages().first().ok_or("page")?.clone())
}

#[test]
fn neutral_element_labels_keep_authored_positions_hydrogens_and_shared_pixels() -> TestResult {
    let page = page(
        vec![
            atom(1, 7, [64.0, 64.0], Some(2))?,
            atom(2, 8, [160.0, 64.0], None)?,
        ],
        &[[1, 2]],
    )?;
    let before = page.clone();
    let graph = graph_page_scene(576, 320, &page)?;
    for (id, position) in [(1, [160.0, 160.0]), (2, [352.0, 160.0])] {
        assert_eq!(
            graph.hit_test(position, 0.0)?,
            Some(HitTarget::Atom {
                fragment_id: FragmentId::new(1).ok_or("fragment")?,
                atom_id: AtomId::new(id).ok_or("atom")?,
            })
        );
    }
    assert_eq!(
        graph
            .scene()
            .primitives
            .iter()
            .filter(|p| matches!(p, Primitive::Stroke(_)))
            .count(),
        1
    );
    let cpu = cpu_scene(graph.scene())?;
    assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, cpu);
    // The geometric middle is ink; the labels clip the incident stroke at both ends.
    let middle = ((160 * 576 + 256) * 4) as usize;
    assert_eq!(
        cpu.rgba.get(middle..middle + 4),
        Some([0, 0, 0, 255].as_slice())
    );
    let masks = graph
        .scene()
        .primitives
        .iter()
        .filter_map(|p| match p {
            Primitive::Rectangle(r) if r.color == [255; 4] && r.height < 128 => Some(r),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(masks.len(), 2);
    for x in [masks[0].x + masks[0].width - 1, masks[1].x] {
        let offset = ((160 * 576 + x) * 4) as usize;
        assert_eq!(
            cpu.rgba.get(offset..offset + 4),
            Some([255; 4].as_slice()),
            "incident bond crosses label padding at{x}"
        );
    }
    assert_eq!(page, before);
    assert_eq!(page.fragments()[0].atoms()[1].explicit_hydrogens, None);
    Ok(())
}

#[test]
fn every_catalog_element_has_literal_label_ink_and_unknown_h_stays_unknown() -> TestResult {
    for number in 1..=118 {
        let authored = atom(1, number, [96.0, 64.0], Some(0))?;
        let explicit = page(vec![authored.clone()], &[])?;
        let graph = graph_page_scene(576, 320, &explicit)?;
        assert!(
            graph
                .scene()
                .primitives
                .iter()
                .any(|p| matches!(p,Primitive::Rectangle(r) if r.color==[0,0,0,255]))
        );
        assert!(graph.hit_test([224.0, 160.0], 0.0)?.is_some());
        if number != 6 {
            let mut unknown = authored;
            unknown.explicit_hydrogens = None;
            let unknown = page(vec![unknown], &[])?;
            let before = unknown.clone();
            assert_eq!(graph_page_scene(576, 320, &unknown)?.scene(), graph.scene());
            assert_eq!(unknown, before);
            assert_eq!(unknown.fragments()[0].atoms()[0].explicit_hydrogens, None);
        }
    }
    Ok(())
}

#[test]
fn literal_hydrogen_and_horizontal_orientation_do_not_repair_counts() -> TestResult {
    for (number, hydrogens) in [
        (7, 3),
        (8, 1),
        (9, 0),
        (17, 0),
        (35, 0),
        (14, 12),
        (118, u16::MAX),
    ] {
        let left = page(
            vec![
                atom(1, number, [64.0, 64.0], Some(hydrogens))?,
                atom(2, 6, [160.0, 64.0], None)?,
            ],
            &[[1, 2]],
        )?;
        let reversed = page(
            vec![
                atom(1, number, [64.0, 64.0], Some(hydrogens))?,
                atom(2, 6, [160.0, 64.0], None)?,
            ],
            &[[2, 1]],
        )?;
        let left_graph = graph_page_scene(576, 320, &left)?;
        assert_eq!(
            cpu_scene(left_graph.scene())?,
            cpu_scene(graph_page_scene(576, 320, &reversed)?.scene())?
        );
        let right = page(
            vec![
                atom(1, number, [160.0, 64.0], Some(hydrogens))?,
                atom(2, 6, [64.0, 64.0], None)?,
            ],
            &[[1, 2]],
        )?;
        let right_graph = graph_page_scene(576, 320, &right)?;
        assert_ne!(
            cpu_scene(left_graph.scene())?,
            cpu_scene(right_graph.scene())?
        );
        assert_eq!(
            left.fragments()[0].atoms()[0].explicit_hydrogens,
            Some(hydrogens)
        );
        assert_eq!(
            right.fragments()[0].atoms()[0].explicit_hydrogens,
            Some(hydrogens)
        );
    }
    let physical = page(
        vec![
            atom(1, 1, [64.0, 64.0], Some(0))?,
            atom(2, 8, [160.0, 64.0], Some(1))?,
        ],
        &[[1, 2]],
    )?;
    assert!(graph_page_scene(576, 320, &physical).is_ok());
    for hydrogens in [1, 2, u16::MAX] {
        let invalid = page(vec![atom(1, 1, [96.0, 64.0], Some(hydrogens))?], &[])?;
        assert!(matches!(
            graph_page_scene(576, 320, &invalid),
            Err(Error::UnsupportedChemicalContent)
        ));
    }
    Ok(())
}

#[test]
fn neutral_labels_keep_fragment_ids_and_refuse_occlusion_or_unreadable_geometry() -> TestResult {
    let mut document = Document::default();
    let page_id = PageId::new(1).ok_or("page")?;
    document.set_page_size(page_id, PageSize::new(256.0, 128.0)?)?;
    for (fragment, id, number, x) in [(7, 11, 7, 64.0), (9, 12, 17, 160.0)] {
        document.append_fragments(
            page_id,
            vec![Fragment::new(
                FragmentId::new(fragment).ok_or("fragment")?,
                vec![atom(id, number, [x, 64.0], None)?],
                vec![],
            )?],
        )?;
    }
    let graph = graph_page_scene(576, 320, document.pages().first().ok_or("page")?)?;
    for (fragment, id, x) in [(7, 11, 160.0), (9, 12, 352.0)] {
        assert_eq!(
            graph.hit_test([x, 160.0], 0.0)?,
            Some(HitTarget::Atom {
                fragment_id: FragmentId::new(fragment).ok_or("fragment")?,
                atom_id: AtomId::new(id).ok_or("atom")?
            })
        );
    }
    let overlapping = page(
        vec![
            atom(1, 7, [96.0, 64.0], Some(12))?,
            atom(2, 8, [97.0, 64.0], None)?,
        ],
        &[],
    )?;
    assert!(matches!(
        graph_page_scene(576, 320, &overlapping),
        Err(Error::LabelOverlap)
    ));
    let crossed = page(
        vec![
            atom(1, 6, [32.0, 64.0], None)?,
            atom(2, 6, [160.0, 64.0], None)?,
            atom(3, 7, [96.0, 64.0], None)?,
        ],
        &[[1, 2]],
    )?;
    assert!(matches!(
        graph_page_scene(576, 320, &crossed),
        Err(Error::LabelOverlap)
    ));
    let ordinary = page(vec![atom(1, 8, [96.0, 64.0], None)?], &[])?;
    assert!(matches!(
        graph_page_scene(80, 80, &ordinary),
        Err(Error::LabelScale)
    ));
    let outside = page(vec![atom(1, 17, [-20.0, 64.0], None)?], &[])?;
    assert!(matches!(
        graph_page_scene(576, 320, &outside),
        Err(Error::LabelPlacement)
    ));
    Ok(())
}
