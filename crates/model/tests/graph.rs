use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, GraphError,
    MAX_GRAPH_ITEMS, PageId, PageSize, Point2, Radical,
};
use std::{num::NonZeroU16, sync::Arc};

fn atom(id: u32, element: u8) -> Result<Atom, Box<dyn std::error::Error>> {
    Ok(Atom::new(
        AtomId::new(id).ok_or("invalid atom id")?,
        Element::new(element).ok_or("invalid element")?,
        None,
    ))
}
fn bond(
    id: u32,
    begin: u32,
    end: u32,
    order: BondOrder,
) -> Result<Bond, Box<dyn std::error::Error>> {
    Ok(Bond {
        double_position: bondline_model::DoublePosition::Unspecified,
        id: BondId::new(id).ok_or("invalid bond id")?,
        begin: AtomId::new(begin).ok_or("invalid begin")?,
        end: AtomId::new(end).ok_or("invalid end")?,
        order,
    })
}
fn fragment_id(id: u32) -> Result<FragmentId, Box<dyn std::error::Error>> {
    FragmentId::new(id).ok_or_else(|| "invalid test fragment id".into())
}

#[test]
fn published_ethanol_connectivity_and_explicit_hydrogens_survive_document_storage()
-> Result<(), Box<dyn std::error::Error>> {
    // DAYLIGHT-SMILES: [CH3][CH2][OH] and C-C-O describe the same ethanol connectivity.
    let mut atoms = vec![atom(1, 6)?, atom(2, 6)?, atom(3, 8)?];
    for (atom, count) in atoms.iter_mut().zip([3, 2, 1]) {
        atom.explicit_hydrogens = Some(count);
    }
    let ethanol = Fragment::new(
        fragment_id(1)?,
        atoms,
        vec![
            bond(1, 1, 2, BondOrder::Single)?,
            bond(2, 2, 3, BondOrder::Single)?,
        ],
    )?;
    assert_eq!(
        ethanol
            .atoms()
            .iter()
            .map(|atom| atom.element.get())
            .collect::<Vec<_>>(),
        [6, 6, 8]
    );
    assert_eq!(
        ethanol
            .bonds()
            .iter()
            .map(|bond| (bond.begin.get(), bond.end.get()))
            .collect::<Vec<_>>(),
        [(1, 2), (2, 3)]
    );
    let mut document = Document::default();
    let page = PageId::new(1).ok_or("invalid page")?;
    document.append_fragments(page, vec![ethanol])?;
    let old = document.clone();
    document.set_page_size(page, PageSize::new(612.0, 792.0)?)?;
    let first = old
        .pages()
        .first()
        .and_then(|page| page.fragments().first())
        .ok_or("missing fragment")?;
    let second = document
        .pages()
        .first()
        .and_then(|page| page.fragments().first())
        .ok_or("missing fragment")?;
    assert!(Arc::ptr_eq(first, second));
    assert_eq!(
        first
            .atoms()
            .iter()
            .map(|atom| atom.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(3), Some(2), Some(1)]
    );
    assert_eq!(
        serde_json::to_value(&document)?["pages"][0]["fragments"][0]["bonds"][1],
        serde_json::json!({"id":2,"begin":2,"end":3,"order":"single"})
    );
    Ok(())
}

#[test]
fn cycles_and_disconnected_fragments_are_preserved_without_chemical_inference()
-> Result<(), Box<dyn std::error::Error>> {
    let ring = Fragment::new(
        fragment_id(1)?,
        (1..=6).map(|id| atom(id, 6)).collect::<Result<_, _>>()?,
        (1..=6)
            .map(|id| {
                bond(
                    id,
                    id,
                    if id == 6 { 1 } else { id + 1 },
                    BondOrder::Aromatic,
                )
            })
            .collect::<Result<_, _>>()?,
    )?;
    assert_eq!(ring.atoms().len(), 6);
    assert_eq!(ring.bonds().len(), 6);
    let mut sodium = atom(7, 11)?;
    sodium.charge = 1;
    let mut chloride = atom(8, 17)?;
    chloride.charge = -1;
    let salt = Fragment::new(fragment_id(2)?, vec![sodium, chloride], vec![])?;
    assert!(salt.bonds().is_empty());
    let mut document = Document::default();
    document.append_fragments(PageId::new(1).ok_or("invalid page")?, vec![ring, salt])?;
    assert_eq!(
        document
            .pages()
            .first()
            .ok_or("missing page")?
            .fragments()
            .len(),
        2
    );
    Ok(())
}

#[test]
fn isotopes_radicals_charge_and_missing_hydrogen_counts_remain_distinct()
-> Result<(), Box<dyn std::error::Error>> {
    let mut carbon = atom(1, 6)?;
    carbon.position = Some(Point2::new(72.0, 144.0)?);
    carbon.isotope = NonZeroU16::new(13);
    carbon.charge = -1;
    carbon.radical = Radical::Doublet;
    carbon.explicit_hydrogens = Some(0);
    let value = serde_json::to_value(&carbon)?;
    assert_eq!(
        value,
        serde_json::json!({"id":1,"element":6,"position":{"x_pt":72.0,"y_pt":144.0},"isotope":13,"charge":-1,"radical":"doublet","explicit_hydrogens":0})
    );
    assert_eq!(serde_json::from_value::<Atom>(value)?, carbon);
    let unspecified = atom(2, 6)?;
    assert_eq!(unspecified.position, None);
    assert_eq!(unspecified.isotope, None);
    assert_eq!(unspecified.explicit_hydrogens, None);
    assert_eq!(unspecified.radical, Radical::None);
    for (key, value) in [
        ("id", 0),
        ("element", 0),
        ("element", 119),
        ("isotope", 0),
        ("charge", 128),
    ] {
        let mut invalid = serde_json::to_value(&carbon)?;
        invalid[key] = serde_json::json!(value);
        assert!(serde_json::from_value::<Atom>(invalid).is_err());
    }
    assert!(Element::new(118).is_some());
    assert!(AtomId::new(0).is_none());
    assert!(BondId::new(0).is_none());
    assert!(FragmentId::new(0).is_none());
    Ok(())
}

#[test]
fn directional_bonds_retain_the_authored_begin_and_end_order()
-> Result<(), Box<dyn std::error::Error>> {
    // CDXML-BOND-ORDER identifies the positive endpoint as the dative begin atom.
    let mut oxygen = atom(1, 8)?;
    oxygen.charge = -1;
    let mut nitrogen = atom(2, 7)?;
    nitrogen.charge = 1;
    let molecule = Fragment::new(
        fragment_id(1)?,
        vec![oxygen, nitrogen],
        vec![bond(1, 2, 1, BondOrder::Dative)?],
    )?;
    let dative = molecule.bonds().first().ok_or("missing bond")?;
    assert_eq!((dative.begin.get(), dative.end.get()), (2, 1));
    assert_eq!(
        serde_json::to_value(dative)?,
        serde_json::json!({"id":1,"begin":2,"end":1,"order":"dative"})
    );
    Ok(())
}

#[test]
fn invalid_connections_and_duplicate_ids_are_rejected_before_storage()
-> Result<(), Box<dyn std::error::Error>> {
    assert!(matches!(
        Fragment::new(fragment_id(1)?, vec![atom(1, 6)?, atom(1, 8)?], vec![]),
        Err(GraphError::DuplicateAtom(1))
    ));
    assert!(matches!(
        Fragment::new(
            fragment_id(1)?,
            vec![atom(1, 6)?],
            vec![bond(1, 1, 2, BondOrder::Single)?]
        ),
        Err(GraphError::MissingAtom(2))
    ));
    assert!(matches!(
        Fragment::new(
            fragment_id(1)?,
            vec![atom(1, 6)?],
            vec![bond(1, 1, 1, BondOrder::Single)?]
        ),
        Err(GraphError::SelfBond(1))
    ));
    assert!(matches!(
        Fragment::new(
            fragment_id(1)?,
            vec![atom(1, 6)?, atom(2, 6)?, atom(3, 6)?],
            vec![
                bond(1, 1, 2, BondOrder::Single)?,
                bond(1, 2, 3, BondOrder::Single)?
            ]
        ),
        Err(GraphError::DuplicateBond(1))
    ));
    assert!(matches!(
        Fragment::new(
            fragment_id(1)?,
            vec![atom(1, 6)?, atom(2, 6)?],
            vec![
                bond(1, 1, 2, BondOrder::Single)?,
                bond(2, 2, 1, BondOrder::Double)?
            ]
        ),
        Err(GraphError::DuplicateConnection)
    ));
    Ok(())
}

#[test]
fn document_identity_checks_are_global_and_batch_failures_are_atomic()
-> Result<(), Box<dyn std::error::Error>> {
    let mut document = Document::default();
    let page = PageId::new(1).ok_or("invalid page")?;
    document.append_fragments(
        page,
        vec![Fragment::new(
            fragment_id(1)?,
            vec![atom(1, 6)?, atom(2, 6)?],
            vec![bond(1, 1, 2, BondOrder::Single)?],
        )?],
    )?;
    let before = serde_json::to_vec(&document)?;
    for invalid in [
        Fragment::new(fragment_id(1)?, vec![], vec![])?,
        Fragment::new(fragment_id(2)?, vec![atom(1, 8)?], vec![])?,
        Fragment::new(
            fragment_id(2)?,
            vec![atom(3, 8)?, atom(4, 6)?],
            vec![bond(1, 3, 4, BondOrder::Double)?],
        )?,
    ] {
        assert!(
            document
                .append_fragments(
                    page,
                    vec![
                        Fragment::new(fragment_id(3)?, vec![atom(9, 6)?], vec![])?,
                        invalid
                    ]
                )
                .is_err()
        );
        assert_eq!(serde_json::to_vec(&document)?, before);
    }
    assert!(
        document
            .append_fragments(PageId::new(2).ok_or("invalid page")?, vec![])
            .is_err()
    );
    assert_eq!(serde_json::to_vec(&document)?, before);
    Ok(())
}

#[test]
fn graph_resource_limits_apply_before_duplicates_or_document_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    assert!(matches!(
        Fragment::new(
            fragment_id(1)?,
            vec![atom(1, 6)?; MAX_GRAPH_ITEMS + 1],
            vec![]
        ),
        Err(GraphError::Limit("atoms"))
    ));
    assert!(matches!(
        Fragment::new(
            fragment_id(1)?,
            vec![atom(1, 6)?],
            vec![bond(1, 1, 1, BondOrder::Single)?; MAX_GRAPH_ITEMS + 1]
        ),
        Err(GraphError::Limit("bonds"))
    ));
    let mut document = Document::default();
    let before = serde_json::to_vec(&document)?;
    let page = PageId::new(1).ok_or("invalid page")?;
    assert!(
        document
            .append_fragments(
                page,
                vec![Fragment::new(fragment_id(1)?, vec![], vec![])?; MAX_GRAPH_ITEMS + 1]
            )
            .is_err()
    );
    assert_eq!(serde_json::to_vec(&document)?, before);
    let atoms = (1..=MAX_GRAPH_ITEMS)
        .map(|id| atom(u32::try_from(id)?, 6))
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    document.append_fragments(page, vec![Fragment::new(fragment_id(1)?, atoms, vec![])?])?;
    assert!(
        document
            .append_fragments(
                page,
                vec![Fragment::new(
                    fragment_id(2)?,
                    vec![atom(100001, 6)?],
                    vec![]
                )?]
            )
            .is_err()
    );
    assert_eq!(
        document
            .pages()
            .first()
            .ok_or("missing page")?
            .fragments()
            .len(),
        1
    );
    Ok(())
}
