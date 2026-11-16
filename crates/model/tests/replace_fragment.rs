use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, GraphError,
    MAX_GRAPH_ITEMS, PageId, Point2,
};
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn atom(id: u32, x: f64, y: f64, hydrogens: u16) -> TestResult<Atom> {
    let mut atom = Atom::new(
        AtomId::new(id).ok_or("atom")?,
        Element::try_from(6)?,
        Some(Point2::new(x, y)?),
    );
    atom.explicit_hydrogens = Some(hydrogens);
    Ok(atom)
}

fn bond(id: u32, begin: u32, end: u32) -> TestResult<Bond> {
    Ok(Bond {
        double_position: bondline_model::DoublePosition::Unspecified,
        id: BondId::new(id).ok_or("bond")?,
        begin: AtomId::new(begin).ok_or("begin")?,
        end: AtomId::new(end).ok_or("end")?,
        order: BondOrder::Single,
    })
}

fn ethane(id: u32, first: u32, bond_id: u32) -> TestResult<Fragment> {
    Ok(Fragment::new(
        FragmentId::new(id).ok_or("fragment")?,
        vec![
            atom(first, 100.0, 200.0, 3)?,
            atom(first + 1, 140.0, 200.0, 3)?,
        ],
        vec![bond(bond_id, first, first + 1)?],
    )?)
}

fn propane() -> TestResult<Fragment> {
    Ok(Fragment::new(
        FragmentId::new(10).ok_or("fragment")?,
        vec![
            atom(100, 100.0, 200.0, 3)?,
            atom(101, 140.0, 200.0, 2)?,
            atom(102, 180.0, 240.0, 3)?,
        ],
        vec![bond(1000, 100, 101)?, bond(1001, 101, 102)?],
    )?)
}

#[test]
fn replacement_is_same_id_same_position_and_preserves_unrelated_shared_payloads() -> TestResult {
    let page = PageId::new(1).ok_or("page")?;
    let mut document = Document::default();
    document.append_fragments(page, vec![ethane(10, 100, 1000)?, ethane(20, 200, 2000)?])?;
    let original = document.clone();
    document.replace_fragment(page, propane()?)?;
    let fragments = document.pages().first().ok_or("page")?.fragments();
    let before = original.pages().first().ok_or("page")?.fragments();
    assert_eq!(
        fragments.iter().map(|f| f.id().get()).collect::<Vec<_>>(),
        [10, 20]
    );
    assert_eq!(
        fragments.first().ok_or("replacement")?.as_ref(),
        &propane()?
    );
    assert!(!Arc::ptr_eq(
        fragments.first().ok_or("replacement")?,
        before.first().ok_or("before")?
    ));
    assert!(Arc::ptr_eq(
        fragments.get(1).ok_or("unrelated")?,
        before.get(1).ok_or("unrelated before")?
    ));
    assert_eq!(
        before.first().ok_or("before")?.as_ref(),
        &ethane(10, 100, 1000)?
    );
    assert_eq!(
        Document::retained_bytes([&document, &document]),
        Document::retained_bytes([&document])
    );
    assert!(
        Document::retained_bytes([&document, &original])
            < Document::retained_bytes([&document]) + Document::retained_bytes([&original])
    );
    Ok(())
}

#[test]
fn failed_replacement_never_appends_or_removes_existing_data() -> TestResult {
    let page = PageId::new(1).ok_or("page")?;
    let mut document = Document::default();
    document.append_fragments(page, vec![ethane(10, 100, 1000)?, ethane(20, 200, 2000)?])?;
    let original = document.clone();
    let absent = document.replace_fragment(PageId::new(2).ok_or("page")?, propane()?);
    assert!(matches!(absent, Err(bondline_model::Error::MissingPage(2))));
    assert_eq!(document, original);
    assert!(matches!(
        document.replace_fragment(page, ethane(99, 300, 3000)?),
        Err(bondline_model::Error::MissingFragment(99))
    ));
    assert_eq!(document, original);
    let collision = Fragment::new(
        FragmentId::new(10).ok_or("fragment")?,
        vec![atom(100, 1.0, 1.0, 3)?, atom(200, 2.0, 2.0, 3)?],
        vec![bond(1000, 100, 200)?],
    )?;
    assert!(matches!(
        document.replace_fragment(page, collision),
        Err(bondline_model::Error::Graph(GraphError::DuplicateAtom(200)))
    ));
    assert_eq!(document, original);
    let collision = Fragment::new(
        FragmentId::new(10).ok_or("fragment")?,
        vec![atom(100, 1.0, 1.0, 3)?, atom(101, 2.0, 2.0, 3)?],
        vec![bond(2000, 100, 101)?],
    )?;
    assert!(matches!(
        document.replace_fragment(page, collision),
        Err(bondline_model::Error::Graph(GraphError::DuplicateBond(
            2000
        )))
    ));
    assert_eq!(document, original);
    Ok(())
}

#[test]
fn replacement_counts_the_new_fragment_once_and_enforces_total_atom_budget() -> TestResult {
    let page = PageId::new(1).ok_or("page")?;
    let mut atoms = (1..MAX_GRAPH_ITEMS)
        .map(|id| atom(id as u32, 0.0, 0.0, 0))
        .collect::<TestResult<Vec<_>>>()?;
    let id = FragmentId::new(10).ok_or("fragment")?;
    let fragment = Fragment::new(id, atoms.clone(), vec![])?;
    let mut document = Document::default();
    document.append_fragments(
        page,
        vec![
            fragment.clone(),
            Fragment::new(
                FragmentId::new(20).ok_or("fragment")?,
                vec![atom(u32::MAX, 0.0, 0.0, 0)?],
                vec![],
            )?,
        ],
    )?;
    document.replace_fragment(page, fragment)?;
    let original = document.clone();
    atoms.push(atom(MAX_GRAPH_ITEMS as u32, 0.0, 0.0, 0)?);
    let larger = Fragment::new(id, atoms, vec![])?;
    assert!(matches!(
        document.replace_fragment(page, larger),
        Err(bondline_model::Error::Graph(GraphError::Limit("atoms")))
    ));
    assert_eq!(document, original);
    Ok(())
}
