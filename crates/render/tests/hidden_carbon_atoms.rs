//! Direct model declarations; no IO/source or physical-render authority is implied.
use bondline_model::{
    Atom, AtomId, AtomLabel, Bond, BondId, BondOrder, DoublePosition, Element, Fragment,
    FragmentId, Point2, TextRun, TextStyle,
};
use bondline_render::{Cancellation, Error, hidden_carbon_atoms};
use std::{collections::BTreeSet, sync::Arc};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn atom(id: u32, z: u8, point: [f64; 2], h: Option<u16>) -> TestResult<Atom> {
    let mut atom = Atom::new(
        AtomId::new(id).ok_or("atom")?,
        Element::try_from(z)?,
        Some(Point2::new(point[0], point[1])?),
    );
    atom.explicit_hydrogens = h;
    Ok(atom)
}

fn fragment(atoms: Vec<Atom>, edges: &[(u32, u32, u32, BondOrder)]) -> TestResult<Fragment> {
    let bonds = edges
        .iter()
        .map(|&(id, begin, end, order)| {
            Ok(Bond {
                id: BondId::new(id).ok_or("bond")?,
                begin: AtomId::new(begin).ok_or("begin")?,
                end: AtomId::new(end).ok_or("end")?,
                order,
                double_position: DoublePosition::Unspecified,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    Ok(Fragment::new(
        FragmentId::new(97).ok_or("fragment")?,
        atoms,
        bonds,
    )?)
}

fn changed(
    f: &Fragment,
    edit: impl FnOnce(&mut Vec<Atom>, &mut Vec<Bond>),
) -> TestResult<Fragment> {
    let mut atoms = f.atoms().to_vec();
    let mut bonds = f.bonds().to_vec();
    edit(&mut atoms, &mut bonds);
    Ok(Fragment::new(f.id(), atoms, bonds)?)
}

fn literal_ids(ids: BTreeSet<AtomId>) -> Vec<u32> {
    ids.into_iter().map(AtomId::get).collect()
}

fn zigzag() -> TestResult<Fragment> {
    let mut atoms = vec![
        atom(73, 8, [96.0, 160.0], Some(1))?,
        atom(12, 6, [144.0, 128.0], Some(2))?,
        atom(601, 6, [192.0, 160.0], Some(2))?,
        atom(41, 6, [240.0, 128.0], Some(2))?,
        atom(7, 7, [288.0, 160.0], Some(2))?,
    ];
    for a in &mut atoms {
        if a.element.get() == 6 {
            a.show_non_terminal_carbon_labels = Some(false);
        }
    }
    fragment(
        atoms,
        &[
            (83, 73, 12, BondOrder::Single),
            (17, 12, 601, BondOrder::Single),
            (211, 601, 41, BondOrder::Single),
            (9, 41, 7, BondOrder::Single),
        ],
    )
}

#[test]
fn explicit_hidden_ids_are_stable_literal_ids_and_never_legacy_inference() -> TestResult {
    let f = zigzag()?;
    let before = f.clone();
    let cancel = Cancellation::default();
    assert_eq!(
        literal_ids(hidden_carbon_atoms(&f, &cancel)?),
        [12, 41, 601]
    );
    let reordered = changed(&f, |atoms, bonds| {
        atoms.reverse();
        bonds.reverse();
        for bond in bonds {
            std::mem::swap(&mut bond.begin, &mut bond.end);
        }
    })?;
    assert_eq!(
        literal_ids(hidden_carbon_atoms(&reordered, &cancel)?),
        [12, 41, 601]
    );
    let legacy = changed(&f, |atoms, _| {
        atoms[2].show_non_terminal_carbon_labels = None;
    })?;
    assert_eq!(
        literal_ids(hidden_carbon_atoms(&legacy, &cancel)?),
        [12, 41]
    );
    assert_eq!(f, before);
    Ok(())
}

#[test]
fn current_positions_and_mandatory_two_double_labels_are_reproved() -> TestResult {
    let f = zigzag()?;
    let moved_point = Point2::new(192.0, 128.0)?;
    let moved = changed(&f, |atoms, _| {
        atoms[2].position = Some(moved_point);
    })?;
    assert_eq!(
        literal_ids(hidden_carbon_atoms(&moved, &Cancellation::default())?),
        [12, 41],
        "the moved middle carbon has opposite rays and must print its label"
    );
    for (angle, expected) in [(169.0_f64, vec![29]), (171.0, vec![])] {
        let radians = angle.to_radians();
        let mut middle = atom(29, 6, [128.0, 128.0], Some(2))?;
        middle.show_non_terminal_carbon_labels = Some(false);
        let junction = fragment(
            vec![
                atom(11, 8, [64.0, 128.0], Some(1))?,
                middle,
                atom(
                    7,
                    7,
                    [128.0 - 64.0 * radians.cos(), 128.0 + 64.0 * radians.sin()],
                    Some(2),
                )?,
            ],
            &[
                (41, 11, 29, BondOrder::Single),
                (42, 29, 7, BondOrder::Single),
            ],
        )?;
        assert_eq!(
            literal_ids(hidden_carbon_atoms(&junction, &Cancellation::default())?),
            expected
        );
    }
    let two_double = changed(&f, |atoms, bonds| {
        atoms[2].explicit_hydrogens = Some(0);
        atoms[1].show_non_terminal_carbon_labels = None;
        atoms[3].show_non_terminal_carbon_labels = None;
        bonds[1].order = BondOrder::Double;
        bonds[2].order = BondOrder::Double;
    })?;
    assert!(hidden_carbon_atoms(&two_double, &Cancellation::default())?.is_empty());
    Ok(())
}

#[test]
fn isolated_and_c2_rules_follow_current_neighbour_elements_not_fragment_size() -> TestResult {
    for (order, h) in [
        (BondOrder::Single, 3),
        (BondOrder::Double, 2),
        (BondOrder::Triple, 1),
    ] {
        let mut first = atom(31, 6, [64.0, 64.0], Some(h))?;
        let mut second = atom(89, 6, [112.0, 64.0], Some(h))?;
        first.show_terminal_carbon_labels = Some(false);
        second.show_terminal_carbon_labels = Some(false);
        let isolated = atom(17, 8, [200.0, 200.0], Some(2))?;
        let c2 = fragment(vec![first, second, isolated], &[(43, 31, 89, order)])?;
        assert!(hidden_carbon_atoms(&c2, &Cancellation::default())?.is_empty());
        let heteroatom = Element::try_from(if order == BondOrder::Triple { 7 } else { 8 })?;
        let neighbour_changed = changed(&c2, |atoms, _| {
            atoms[1].element = heteroatom;
            atoms[1].explicit_hydrogens = Some(if order == BondOrder::Single { 1 } else { 0 });
        })?;
        assert_eq!(
            literal_ids(hidden_carbon_atoms(
                &neighbour_changed,
                &Cancellation::default()
            )?),
            [31]
        );
    }
    let mut isolated = atom(211, 6, [128.0, 128.0], Some(4))?;
    isolated.show_terminal_carbon_labels = Some(false);
    let isolated = fragment(vec![isolated], &[])?;
    assert!(hidden_carbon_atoms(&isolated, &Cancellation::default())?.is_empty());
    Ok(())
}

#[test]
fn unknown_h_invalid_c4_and_unsupported_orders_refuse_without_inference() -> TestResult {
    let f = zigzag()?;
    for changed_h in [None, Some(1)] {
        let invalid = changed(&f, |atoms, _| atoms[2].explicit_hydrogens = changed_h)?;
        assert!(matches!(
            hidden_carbon_atoms(&invalid, &Cancellation::default()),
            Err(Error::UnsupportedChemicalContent)
        ));
    }
    let invalid = changed(&f, |_, bonds| bonds[1].order = BondOrder::Aromatic)?;
    assert!(matches!(
        hidden_carbon_atoms(&invalid, &Cancellation::default()),
        Err(Error::UnsupportedChemicalContent)
    ));
    let printed = changed(&f, |atoms, _| {
        atoms[2].explicit_hydrogens = Some(1);
        atoms[2].show_non_terminal_carbon_labels = Some(true);
    })?;
    assert_eq!(
        literal_ids(hidden_carbon_atoms(&printed, &Cancellation::default())?),
        [12, 41]
    );
    Ok(())
}

#[test]
fn authored_labels_take_priority_and_cancellation_covers_empty_and_legacy_fast_paths() -> TestResult
{
    let f = zigzag()?;
    let label = Arc::new(AtomLabel::new(
        None,
        None,
        None,
        None,
        vec![TextRun::new(
            "C".into(),
            TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
        )?],
    )?);
    let authored = changed(&f, |atoms, _| atoms[2].authored_label = Some(label))?;
    assert_eq!(
        literal_ids(hidden_carbon_atoms(&authored, &Cancellation::default())?),
        [12, 41]
    );
    let legacy = changed(&f, |atoms, _| {
        for atom in atoms {
            atom.show_terminal_carbon_labels = None;
            atom.show_non_terminal_carbon_labels = None;
        }
    })?;
    let empty = fragment(vec![], &[])?;
    let cancel = Cancellation::default();
    cancel.cancel();
    for fragment in [&f, &authored, &legacy, &empty] {
        assert!(matches!(
            hidden_carbon_atoms(fragment, &cancel),
            Err(Error::Cancelled)
        ));
    }
    Ok(())
}

#[test]
fn implied_carbon_flags_require_hidden_geometry_and_complete_c4_counts() -> TestResult {
    use bondline_model::{Document, PageId, PageSize};
    use bondline_render::PageView;
    use std::collections::BTreeMap;
    let cancel = Cancellation::default();
    let size = PageSize::new(512.0, 512.0)?;
    let page_id = PageId::new(1).ok_or("page")?;
    let view = PageView::zoom(512, 512, size, 100.0, Point2::new(256.0, 256.0)?)?;
    for (angle, active, inconsistent, qualifies) in [
        (169.0_f64, false, false, true),
        (170.0, false, false, false),
        (171.0, false, false, false),
        (180.0, false, false, false),
        (106.0, true, false, false),
        (106.0, false, true, false),
    ] {
        let theta = angle.to_radians();
        let mut terminal = atom(1, 6, [96.0, 256.0], None)?;
        terminal.show_non_terminal_carbon_labels = Some(true);
        let mut center = atom(2, 6, [160.0, 256.0], None)?;
        center.show_terminal_carbon_labels = Some(true);
        if active {
            center.show_non_terminal_carbon_labels = Some(true);
        }
        let end = atom(
            3,
            6,
            [160.0 - 64.0 * theta.cos(), 256.0 + 64.0 * theta.sin()],
            Some(3),
        )?;
        let graph = fragment(
            vec![terminal, center, end],
            &[(1, 1, 2, BondOrder::Single), (2, 2, 3, BondOrder::Single)],
        )?;
        let mut doc = Document::default();
        doc.set_page_size(page_id, size)?;
        doc.append_fragments(page_id, vec![graph])?;
        let page = &doc.pages()[0];
        let counts = [(1, 3), (2, if inconsistent { 1 } else { 2 }), (3, 3)]
            .into_iter()
            .map(|(i, h)| Ok((AtomId::new(i).ok_or("id")?, h)))
            .collect::<TestResult<BTreeMap<_, _>>>()?;
        let result = view.graph_scene_with_hidden_carbons(page, &counts, &cancel);
        assert_eq!(
            result.is_ok(),
            qualifies,
            "angle {angle} active {active} inconsistent {inconsistent}"
        );
        if let Ok(scene) = result {
            assert!(scene.label_em_px().is_none());
            // Generic resolved-H rendering still treats an absent active flag as unspecified.
            assert!(
                view.graph_scene_with_display_hydrogens(page, &counts, &cancel)?
                    .label_em_px()
                    .is_some()
            );
            let mut missing = counts.clone();
            missing.remove(&AtomId::new(2).ok_or("middle")?);
            assert!(
                view.graph_scene_with_hidden_carbons(page, &missing, &cancel)
                    .is_err()
            );
        }
        assert_eq!(page.fragments()[0].atoms()[1].explicit_hydrogens, None);
    }
    Ok(())
}
