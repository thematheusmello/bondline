use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    PageSize, Point2,
};
use bondline_render::{
    Cancellation, Error, HitTarget, cpu_scene, gpu_scene, graph_page_scene,
    graph_page_scene_with_display_hydrogens,
};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn drawing(specs: &[(u32, u8, [f64; 2], Option<u16>)], edges: &[[u32; 2]]) -> TestResult<Page> {
    let atoms = specs
        .iter()
        .map(|&(id, z, p, h)| {
            let mut atom = Atom::new(
                AtomId::new(id).ok_or("atom")?,
                Element::try_from(z)?,
                Some(Point2::new(p[0], p[1])?),
            );
            atom.explicit_hydrogens = h;
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = edges
        .iter()
        .enumerate()
        .map(|(index, &[a, b])| {
            Ok(Bond {
                id: BondId::new(u32::try_from(index + 1)?).ok_or("bond")?,
                begin: AtomId::new(a).ok_or("begin")?,
                end: AtomId::new(b).ok_or("end")?,
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

fn target_atom(id: u32) -> TestResult<HitTarget> {
    Ok(HitTarget::Atom {
        fragment_id: FragmentId::new(1).ok_or("fragment")?,
        atom_id: AtomId::new(id).ok_or("atom")?,
    })
}

#[test]
fn literal_methine_branch_attaches_right_bond_to_c_and_keeps_h_elsewhere() -> TestResult {
    // Three exact88pt rays at0/120/240degrees. The paper maps x/y ->32+2*x/y.
    let rise = 44.0 * 3.0_f64.sqrt();
    let page = drawing(
        &[
            (1, 6, [128.0, 128.0], Some(1)),
            (2, 6, [216.0, 128.0], Some(3)),
            (3, 6, [84.0, 128.0 + rise], Some(3)),
            (4, 6, [84.0, 128.0 - rise], Some(3)),
        ],
        &[[1, 2], [1, 3], [1, 4]],
    )?;
    let before = page.clone();
    let graph = graph_page_scene(576, 576, &page)?;
    let frame = cpu_scene(graph.scene())?;
    assert_eq!(page, before);
    assert_eq!(graph.hit_test([288.0, 288.0], 0.0)?, Some(target_atom(1)?));
    let bond = HitTarget::Bond {
        fragment_id: FragmentId::new(1).ok_or("fragment")?,
        bond_id: BondId::new(1).ok_or("bond")?,
    };
    // Pinned C hmtx/glyf bounds give <14px ink at24px em. This point20px
    // right is on the actual bond, outside C, and inside the old horizontal H.
    assert_eq!(
        graph.hit_test([308.0, 288.0], 0.0)?,
        Some(bond),
        "literal H must not replace the incident C-bond attachment corridor"
    );
    for x in 304..=320 {
        for y in 287..=288 {
            let at = (y * 576 + x) * 4;
            assert_eq!(
                frame.rgba.get(at..at + 4),
                Some([0, 0, 0, 255].as_slice()),
                "analytic width2 horizontal bond corridor at{x},{y}"
            );
        }
    }
    for x in 300..318 {
        for y in 278..299 {
            if y == 287 || y == 288 {
                continue;
            }
            let at = (y * 576 + x) * 4;
            assert_eq!(
                frame.rgba.get(at..at + 4),
                Some([255; 4].as_slice()),
                "no H stem may occupy the off-axis part of the bond corridor"
            );
        }
    }
    // The other two rays miss all three candidate H regions, and C occupies none.
    // H may mirror or stack; it may not disappear to make the corridor clear.
    let h_ink: usize = [
        (264, 278, 280, 299),
        (280, 254, 296, 275),
        (280, 302, 296, 323),
    ]
    .into_iter()
    .map(|(left, top, right, bottom)| {
        (top..bottom)
            .flat_map(|y| (left..right).map(move |x| (y * 576 + x) * 4))
            .filter(|&at| frame.rgba.get(at..at + 4) == Some([0, 0, 0, 255].as_slice()))
            .count()
    })
    .sum();
    assert!(
        h_ink >= 20,
        "the literal H1 glyph must remain visible away from the incident bond"
    );
    assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, frame);
    assert_eq!(page, before);
    Ok(())
}

#[test]
fn unrelated_oxygen_on_stacked_h_requires_whole_scene_overlap_refusal() -> TestResult {
    let specs = [
        (1, 6, [40.0, 128.0], None),
        (2, 7, [128.0, 128.0], Some(1)),
        (3, 6, [216.0, 128.0], None),
    ];
    let base = drawing(&specs, &[[1, 2], [2, 3]])?;
    let counts = BTreeMap::from([
        (AtomId::new(1).ok_or("atom")?, 3),
        (AtomId::new(2).ok_or("atom")?, 1),
        (AtomId::new(3).ok_or("atom")?, 3),
    ]);
    let graph = graph_page_scene_with_display_hydrogens(
        576,
        576,
        &base,
        &counts,
        &Cancellation::default(),
    )?;
    // Opposed arms require stacked H. One24px em above N is(288,264).
    assert_eq!(graph.hit_test([288.0, 264.0], 0.0)?, Some(target_atom(2)?));
    let frame = cpu_scene(graph.scene())?;
    assert_eq!(pollster::block_on(gpu_scene(graph.scene()))?.0, frame);
    let mut specs = specs.to_vec();
    specs.push((4, 8, [128.0, 116.0], Some(0)));
    let overlap = drawing(&specs, &[[1, 2], [2, 3]])?;
    let before = overlap.clone();
    let mut counts = counts;
    counts.insert(AtomId::new(4).ok_or("atom")?, 0);
    assert!(matches!(
        graph_page_scene_with_display_hydrogens(
            576,
            576,
            &overlap,
            &counts,
            &Cancellation::default()
        ),
        Err(Error::LabelOverlap)
    ));
    assert_eq!(overlap, before);
    assert_eq!(base.fragments()[0].atoms()[0].explicit_hydrogens, None);
    assert_eq!(base.fragments()[0].atoms()[2].explicit_hydrogens, None);
    Ok(())
}
