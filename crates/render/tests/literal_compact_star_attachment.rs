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
fn compact_methine_star_keeps_literal_h_and_all_three_incident_bonds() -> TestResult {
    // Fixed original C4H10; the center/raster phase reproduces the12C diagnostic
    // without its ring/search API or any unrelated atom/label geometry.
    let center = [226.27681403987702, 300.14032773029817];
    let angles = [50.0_f64, 170.0, 290.0].map(f64::to_radians);
    let mut atoms = Vec::new();
    for (index, (position, hydrogens)) in std::iter::once((center, 1))
        .chain(angles.map(|a| ([center[0] + 64.0 * a.cos(), center[1] + 64.0 * a.sin()], 3)))
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
    document.set_page_size(page_id, PageSize::new(1000.0, 1000.0)?)?;
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
        let point = fragment.atoms()[index + 1].position.ok_or("position")?;
        let dx = point.x_pt() - center[0];
        let dy = point.y_pt() - center[1];
        assert!((dx.hypot(dy) - 64.0).abs() < 1e-12);
        assert!((dx / 64.0 - angle.cos()).abs() < 1e-14);
        assert!((dy / 64.0 - angle.sin()).abs() < 1e-14);
    }
    let counts = fragment
        .atoms()
        .iter()
        .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("literal H")?)))
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    let literal = graph_page_scene(1024, 1024, page)?;
    let safe = graph_page_scene_with_display_hydrogens(
        1024,
        1024,
        page,
        &counts,
        &Cancellation::default(),
    )?;
    assert!(literal.scene() == safe.scene());
    assert!((literal.label_em_px().ok_or("em")? - 11.52).abs() < 1e-5);
    let cpu = cpu_scene(literal.scene())?;
    assert_eq!(cpu, cpu_scene(safe.scene())?);
    // The actual1000pt paper maps to32+0.96*coordinate. These fixed pixel
    // centers lie inside the width0.96 bonds and beyond all central C ink.
    let canvas = center.map(|value| 32.0 + 0.96 * value);
    for (index, (x, y)) in [(262_usize, 335_usize), (229, 323), (256, 301)]
        .into_iter()
        .enumerate()
    {
        let point = [x as f64 + 0.5, y as f64 + 0.5];
        let dx = point[0] - canvas[0];
        let dy = point[1] - canvas[1];
        let angle = angles[index];
        assert!((dx * angle.sin() - dy * angle.cos()).abs() < 0.48);
        let target = Some(HitTarget::Bond {
            fragment_id,
            bond_id: BondId::new(u32::try_from(index + 1)?).ok_or("bond")?,
        });
        assert_eq!(literal.hit_test(point.map(|v| v as f32), 0.0)?, target);
        assert_eq!(safe.hit_test(point.map(|v| v as f32), 0.0)?, target);
        let at = (y * 1024 + x) * 4;
        assert_eq!(
            cpu.rgba.get(at..at + 4),
            Some([0, 0, 0, 255].as_slice()),
            "incident-bond pixel at{x},{y}"
        );
    }
    // Other atoms are>61px away. This neighborhood excludes C's conservative
    // box and every bond corridor, so its remaining black pixels are literal H.
    let mut hydrogen_pixels = 0;
    for y in 296..344 {
        for x in 230..268 {
            let point = [x as f64 + 0.5, y as f64 + 0.5];
            let offset = [point[0] - canvas[0], point[1] - canvas[1]];
            let in_carbon =
                (244.0..=255.0).contains(&point[0]) && (313.0..=327.0).contains(&point[1]);
            let in_bond = angles.into_iter().any(|angle| {
                offset[0] * angle.cos() + offset[1] * angle.sin() >= 0.0
                    && (offset[0] * angle.sin() - offset[1] * angle.cos()).abs() <= 2.0
            });
            let at = (y * 1024 + x) * 4;
            if !in_carbon && !in_bond && cpu.rgba.get(at..at + 4) == Some([0, 0, 0, 255].as_slice())
            {
                hydrogen_pixels += 1;
            }
        }
    }
    assert!(
        hydrogen_pixels >= 8,
        "literal central H1 must remain visible"
    );
    assert_eq!(pollster::block_on(gpu_scene(literal.scene()))?.0, cpu);
    assert_eq!(pollster::block_on(gpu_scene(safe.scene()))?.0, cpu);
    assert_eq!(document, before);
    Ok(())
}
