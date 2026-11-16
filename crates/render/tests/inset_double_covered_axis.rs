//! Distinct authored-axis ink, independent of the acute angle itself.
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, Element, Fragment, FragmentId,
    Page, PageId, PageSize, Point2,
};
use bondline_render::{HitTarget, cpu_scene, graph_page_scene};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn page(end: [f64; 2]) -> TestResult<Page> {
    let atoms = [(29, [64.0, 96.0]), (11, [192.0, 96.0]), (7, end)]
        .into_iter()
        .map(|(id, p)| {
            Ok(Atom::new(
                AtomId::new(id).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(p[0], p[1])?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = [
        (41, 29, 11, BondOrder::Double, DoublePosition::Right),
        (42, 29, 7, BondOrder::Single, DoublePosition::Unspecified),
    ]
    .into_iter()
    .map(|(id, begin, end, order, double_position)| {
        Ok(Bond {
            id: BondId::new(id).ok_or("bond")?,
            begin: AtomId::new(begin).ok_or("begin")?,
            end: AtomId::new(end).ok_or("end")?,
            order,
            double_position,
        })
    })
    .collect::<TestResult<Vec<_>>>()?;
    let mut doc = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    doc.set_page_size(id, PageSize::new(256.0, 256.0)?)?;
    doc.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(doc.pages().first().ok_or("page")?.clone())
}

#[test]
fn a_joined_axis_cannot_disappear_but_the_same_acute_angle_with_distinct_ink_is_valid() -> TestResult
{
    let hidden = page([112.0, 95.9])?;
    let before = hidden.clone();
    // Scale3: Double axis(224,320)->(608,320), w3. The Single ends(368,319.7).
    // Its x caps contain no centers before224. Its entire y range is above318.2
    // and below321.5: every represented Single pixel lies in Double axis rows318..321.
    // The ray is neither zero nor exactly co-directed; Right has zero trim here.
    assert!(
        graph_page_scene(832, 832, &hidden).is_err(),
        "joined Single had no distinct unmasked pixel"
    );
    assert_eq!(hidden, before);

    // Same authored slope -1/480, longer arm: it ends(704,319), beyond Double's x608.
    // A blanket minimum angle must not reject this independently visible control.
    let visible = page([224.0, 96.0 - 1.0 / 3.0])?;
    let before = visible.clone();
    let graph = graph_page_scene(832, 832, &visible)?;
    let frame = cpu_scene(graph.scene())?;
    for (x, y, bond) in [(680_u32, 319_u32, 42_u32), (500, 321, 41), (500, 329, 41)] {
        let offset = usize::try_from((u64::from(y) * 832 + u64::from(x)) * 4)?;
        assert_eq!(
            frame.rgba.get(offset..offset + 4).ok_or("RGBA")?,
            &[0, 0, 0, 255]
        );
        assert_eq!(
            graph.hit_test([x as f32 + 0.5, y as f32 + 0.5], 0.0)?,
            Some(HitTarget::Bond {
                fragment_id: FragmentId::new(17).ok_or("fragment")?,
                bond_id: BondId::new(bond).ok_or("bond")?
            })
        );
    }
    assert_eq!(visible, before);
    assert!(
        visible.fragments()[0]
            .atoms()
            .iter()
            .all(|atom| atom.explicit_hydrogens.is_none())
    );
    Ok(())
}
