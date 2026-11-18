use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    PageSize, Point2,
};
use bondline_render::{
    Cancellation, Frame, HitTarget, cpu_scene, gpu_scene, graph_page_scene,
    graph_page_scene_with_display_hydrogens,
};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn hexagon(hydrogens: Option<u16>) -> TestResult<Page> {
    let side = 32.0 * 3.0_f64.sqrt();
    let points = [
        [128.0 + side, 160.0],
        [128.0, 192.0],
        [128.0 - side, 160.0],
        [128.0 - side, 96.0],
        [128.0, 64.0],
        [128.0 + side, 96.0],
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
            atom.explicit_hydrogens = hydrogens;
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = [(1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (1, 6)]
        .into_iter()
        .enumerate()
        .map(|(i, (a, b))| {
            Ok(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(u32::try_from(i + 1)?).ok_or("bond")?,
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

fn incident_bond() -> TestResult<HitTarget> {
    Ok(HitTarget::Bond {
        fragment_id: FragmentId::new(1).ok_or("fragment")?,
        bond_id: BondId::new(4).ok_or("bond")?,
    })
}

fn pixel(frame: &Frame, x: usize, y: usize) -> Option<&[u8]> {
    let at = (y * 576 + x) * 4;
    frame.rgba.get(at..at + 4)
}

#[test]
fn literal_ring_ch2_keeps_diagonal_bond_ink_and_same_count_attachment_scene() -> TestResult {
    // Radius64 regular hexagon,64pt sides, preferred vertical-left edge. At2x,
    // Atom5(top) is(288,160), Atom4 is(288-64sqrt3,224). The pixel-center
    // (270.5,170.5) is0.344px from their width2 bond axis and outside C ink.
    let page = hexagon(Some(2))?;
    let before = page.clone();
    let counts = page.fragments()[0]
        .atoms()
        .iter()
        .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("H2")?)))
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    let literal = graph_page_scene(576, 576, &page)?;
    let safe = graph_page_scene_with_display_hydrogens(
        576,
        576,
        &page,
        &counts,
        &Cancellation::default(),
    )?;
    let actual = cpu_scene(literal.scene())?;
    let expected = cpu_scene(safe.scene())?;
    let probe = [270.5, 170.5];
    let bond = Some(incident_bond()?);
    assert_eq!(safe.hit_test(probe, 0.0)?, bond);
    assert_eq!(pixel(&expected, 270, 170), Some([0, 0, 0, 255].as_slice()));
    assert_eq!(pollster::block_on(gpu_scene(safe.scene()))?.0, expected);
    assert_eq!(page, before);
    assert!(
        page.fragments()[0]
            .atoms()
            .iter()
            .all(|a| a.explicit_hydrogens == Some(2))
    );
    assert_eq!(
        pixel(&actual, 270, 170),
        Some([0, 0, 0, 255].as_slice()),
        "ordinary literal drawing erased incident-bond ink; actual hit={:?}",
        literal.hit_test(probe, 0.0)?
    );
    assert_eq!(literal.hit_test(probe, 0.0)?, bond);
    assert!(
        literal.scene() == safe.scene(),
        "actual ordinary drawing must equal the same-count attachment-safe scene"
    );
    assert_eq!(actual, expected);
    assert_eq!(pollster::block_on(gpu_scene(literal.scene()))?.0, actual);
    assert_eq!(page, before);
    Ok(())
}

#[test]
fn unknown_h_ring_is_a_separate_skeletal_control_and_never_gains_literal_counts() -> TestResult {
    // This control is not a replacement for the Some(H2) authored source above.
    let page = hexagon(None)?;
    let before = page.clone();
    let graph = graph_page_scene(576, 576, &page)?;
    let frame = cpu_scene(graph.scene())?;
    assert_eq!(pixel(&frame, 270, 170), Some([0, 0, 0, 255].as_slice()));
    assert_eq!(graph.hit_test([270.5, 170.5], 0.0)?, Some(incident_bond()?));
    assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, frame);
    assert_eq!(page, before);
    assert!(
        page.fragments()[0]
            .atoms()
            .iter()
            .all(|a| a.explicit_hydrogens.is_none())
    );
    Ok(())
}
