use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, PageId,
    PageSize, Point2,
};
use bondline_render::{
    Cancellation, HitTarget, cpu_scene, gpu_scene, graph_page_scene,
    graph_page_scene_with_display_hydrogens,
};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn rotated_methine_star_keeps_all_incident_bonds_and_literal_hydrogen_ink() -> TestResult {
    // An original C4H10 star, independent of every SMILES layout/search API.
    // The256pt paper maps to32+2*coordinate in the576px frame.
    let angles = [10.0_f64, 130.0, 250.0].map(f64::to_radians);
    let mut atoms = Vec::new();
    for (index, (position, hydrogens)) in std::iter::once(([128.0, 128.0], 1))
        .chain(angles.map(|a| ([128.0 + 88.0 * a.cos(), 128.0 + 88.0 * a.sin()], 3)))
        .enumerate()
    {
        let mut atom = Atom::new(
            AtomId::new(u32::try_from(index + 1)?).ok_or("atom")?,
            Element::try_from(6)?,
            Some(Point2::new(position[0], position[1])?),
        );
        atom.explicit_hydrogens = Some(hydrogens);
        atoms.push(atom);
    }
    let bonds = (1..=3)
        .map(|id| {
            Ok(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(id).ok_or("bond")?,
                begin: AtomId::new(1).ok_or("begin")?,
                end: AtomId::new(id + 1).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let fragment_id = FragmentId::new(1).ok_or("fragment")?;
    let page_id = PageId::new(1).ok_or("page")?;
    let mut document = Document::default();
    document.set_page_size(page_id, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(page_id, vec![Fragment::new(fragment_id, atoms, bonds)?])?;
    let before = document.clone();
    let page = document.pages().first().ok_or("page")?;
    let fragment = page.fragments().first().ok_or("fragment")?;
    assert_eq!(fragment.atoms().len(), 4);
    assert_eq!(fragment.bonds().len(), 3);
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(1), Some(3), Some(3), Some(3)]
    );
    for (index, angle) in angles.into_iter().enumerate() {
        let p = fragment.atoms()[index + 1].position.ok_or("position")?;
        let dx = p.x_pt() - 128.0;
        let dy = p.y_pt() - 128.0;
        assert!((dx.hypot(dy) - 88.0).abs() < 1e-12);
        assert!((dx / 88.0 - angle.cos()).abs() < 1e-14);
        assert!((dy / 88.0 - angle.sin()).abs() < 1e-14);
    }
    let counts = fragment
        .atoms()
        .iter()
        .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("literal H")?)))
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    let literal = graph_page_scene(576, 576, page)?;
    let safe =
        graph_page_scene_with_display_hydrogens(576, 576, page, &counts, &Cancellation::default())?;
    let cpu = cpu_scene(literal.scene())?;
    assert_eq!(cpu, cpu_scene(safe.scene())?);
    assert!(literal.scene() == safe.scene());
    assert_eq!(literal.label_em_px(), Some(24.0));
    // These20px-from-C points are outside the pinned C ink and still on each
    // width2 incident bond. No H placement may turn them into Atom hits/white.
    for (index, angle) in angles.into_iter().enumerate() {
        let point = [288.0 + 20.0 * angle.cos(), 288.0 + 20.0 * angle.sin()];
        let target = Some(HitTarget::Bond {
            fragment_id,
            bond_id: BondId::new(u32::try_from(index + 1)?).ok_or("bond")?,
        });
        assert_eq!(literal.hit_test(point.map(|v| v as f32), 0.0)?, target);
        assert_eq!(safe.hit_test(point.map(|v| v as f32), 0.0)?, target);
        let x = point[0].floor() as usize;
        let y = point[1].floor() as usize;
        let at = (y * 576 + x) * 4;
        assert_eq!(
            cpu.rgba.get(at..at + 4),
            Some([0, 0, 0, 255].as_slice()),
            "incident-bond pixel at{x},{y}"
        );
    }
    // Count actual H ink outside C's conservative16x24 box and outside the
    // three bond corridors; preserving source H alone cannot satisfy this.
    let mut hydrogen_pixels = 0;
    for y in 252..324 {
        for x in 252..324 {
            let point = [x as f64 + 0.5 - 288.0, y as f64 + 0.5 - 288.0];
            let in_carbon = point[0].abs() <= 8.0 && point[1].abs() <= 12.0;
            let in_bond = angles.into_iter().any(|a| {
                point[0] * a.cos() + point[1] * a.sin() >= 0.0
                    && (point[0] * a.sin() - point[1] * a.cos()).abs() <= 2.0
            });
            let at = (y * 576 + x) * 4;
            if !in_carbon && !in_bond && cpu.rgba.get(at..at + 4) == Some([0, 0, 0, 255].as_slice())
            {
                hydrogen_pixels += 1;
            }
        }
    }
    assert!(
        hydrogen_pixels >= 20,
        "literal central H1 must stay visible"
    );
    assert_eq!(pollster::block_on(gpu_scene(literal.scene()))?.0, cpu);
    assert_eq!(pollster::block_on(gpu_scene(safe.scene()))?.0, cpu);
    assert_eq!(document, before);
    Ok(())
}
