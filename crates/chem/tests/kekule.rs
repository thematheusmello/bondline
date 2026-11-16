//! Direct public-core expectations; no SMILES parser, writer or external runtime.

use bondline_chem::{AtomSemantics, Hydrogens, normalize_neutral_kekule};
use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, Point2};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn sidecar_implicit_cannot_silently_replace_an_authored_h_count() -> TestResult {
    let mut carbon = Atom::new(
        AtomId::new(701).ok_or("atom identity")?,
        Element::try_from(6)?,
        Some(Point2::new(12.0, 34.0)?),
    );
    carbon.explicit_hydrogens = Some(4);
    let fragment = Fragment::new(
        FragmentId::new(19).ok_or("fragment identity")?,
        vec![carbon],
        vec![],
    )?;
    let before = fragment.clone();
    let error = normalize_neutral_kekule(
        &fragment,
        &[AtomSemantics {
            aromatic: false,
            hydrogens: Hydrogens::Implicit,
        }],
        || false,
    )
    .err()
    .ok_or("mismatched H sidecar must refuse")?;
    assert_eq!(error.code(), "invalid_document");
    assert_eq!(fragment, before);
    Ok(())
}

use bondline_model::{Bond, BondId, BondOrder, Radical};
use std::{cell::Cell, num::NonZeroU16};

const S: BondOrder = BondOrder::Single;
const D: BondOrder = BondOrder::Double;
const A: BondOrder = BondOrder::Aromatic;

fn graph(
    atoms: &[(u32, u8, Option<u16>)],
    bonds: &[(u32, u32, u32, BondOrder)],
) -> TestResult<Fragment> {
    let atoms = atoms
        .iter()
        .map(|&(id, z, h)| {
            let mut atom = Atom::new(
                AtomId::new(id).ok_or("atom")?,
                Element::try_from(z)?,
                Some(Point2::new(f64::from(id) / 4.0, -f64::from(id) / 8.0)?),
            );
            atom.explicit_hydrogens = h;
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = bonds
        .iter()
        .map(|&(id, begin, end, order)| {
            Ok(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(id).ok_or("bond")?,
                begin: AtomId::new(begin).ok_or("begin")?,
                end: AtomId::new(end).ok_or("end")?,
                order,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    Ok(Fragment::new(
        FragmentId::new(19).ok_or("fragment")?,
        atoms,
        bonds,
    )?)
}

fn modes(fragment: &Fragment, marked: &[u32]) -> Vec<AtomSemantics> {
    fragment
        .atoms()
        .iter()
        .map(|atom| AtomSemantics {
            aromatic: marked.contains(&atom.id.get()),
            hydrogens: atom
                .explicit_hydrogens
                .map_or(Hydrogens::Implicit, Hydrogens::Count),
        })
        .collect()
}

fn plain(hydrogens: Hydrogens) -> AtomSemantics {
    AtomSemantics {
        aromatic: false,
        hydrogens,
    }
}

fn ring(elements: &[u8], h: &[Option<u16>], orders: &[BondOrder]) -> TestResult<Fragment> {
    assert_eq!((elements.len(), h.len()), (orders.len(), orders.len()));
    let count = u32::try_from(elements.len())?;
    let atoms = elements
        .iter()
        .zip(h)
        .enumerate()
        .map(|(i, (&z, &h))| Ok((u32::try_from(i + 1)?, z, h)))
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = orders
        .iter()
        .enumerate()
        .map(|(i, &order)| {
            let id = u32::try_from(i + 1)?;
            Ok((id, id, id % count + 1, order))
        })
        .collect::<TestResult<Vec<_>>>()?;
    graph(&atoms, &bonds)
}

fn refuses(fragment: &Fragment, semantics: &[AtomSemantics], code: &str) -> TestResult {
    let before = fragment.clone();
    let error = normalize_neutral_kekule(fragment, semantics, || false)
        .err()
        .ok_or("normalization unexpectedly succeeded")?;
    assert_eq!(error.code(), code);
    assert_eq!(fragment, &before);
    Ok(())
}

#[test]
fn sidecar_length_and_every_literal_h_mode_must_match_exactly() -> TestResult {
    let literal = graph(&[(9, 6, Some(4))], &[])?;
    refuses(&literal, &[], "invalid_document")?;
    refuses(
        &literal,
        &[plain(Hydrogens::Count(4)), plain(Hydrogens::Count(4))],
        "invalid_document",
    )?;
    refuses(&literal, &[plain(Hydrogens::Count(3))], "invalid_document")?;
    let implicit = graph(&[(9, 6, None)], &[])?;
    refuses(&implicit, &[plain(Hydrogens::Count(4))], "invalid_document")
}

#[test]
fn matching_backtracks_past_the_first_pyrrole_edge_without_touching_donor_n() -> TestResult {
    let input = ring(
        &[6, 6, 6, 7, 6],
        &[None, None, None, Some(1), None],
        &[A; 5],
    )?;
    let before = input.clone();
    let output = normalize_neutral_kekule(&input, &modes(&input, &[1, 2, 3, 4, 5]), || false)?;
    // 1--2 would strand C3 beside donor N4. The first complete matching is 1==5,2==3.
    assert_eq!(
        output,
        ring(&[6, 6, 6, 7, 6], &[Some(1); 5], &[S, D, S, S, D])?
    );
    assert_eq!(input, before);
    Ok(())
}

#[test]
fn nondense_out_of_order_c4_uses_id_matching_and_preserves_positive_isotope_metadata() -> TestResult
{
    // This C4 integer projection is not a claim of perceived aromaticity or a 4n+2 rule.
    let source = graph(
        &[(90, 6, None), (10, 6, None), (70, 6, None), (30, 6, None)],
        &[
            (19, 30, 90, A),
            (71, 90, 10, A),
            (60, 70, 30, A),
            (5, 10, 70, A),
        ],
    )?;
    let mut atoms = source.atoms().to_vec();
    atoms.get_mut(2).ok_or("C70")?.isotope = Some(NonZeroU16::new(13).ok_or("13C")?);
    let input = Fragment::new(source.id(), atoms, source.bonds().to_vec())?;
    let before = input.clone();
    let output = normalize_neutral_kekule(&input, &modes(&input, &[10, 30, 70, 90]), || false)?;
    let mut expected_atoms = input.atoms().to_vec();
    for atom in &mut expected_atoms {
        atom.explicit_hydrogens = Some(1);
    }
    let mut expected_bonds = input.bonds().to_vec();
    for bond in &mut expected_bonds {
        bond.order = if [5, 19].contains(&bond.id.get()) {
            D
        } else {
            S
        };
    }
    assert_eq!(
        output,
        Fragment::new(input.id(), expected_atoms, expected_bonds)?
    );
    assert_eq!(input, before);
    Ok(())
}

#[test]
fn physical_h_and_fixed_triple_preserve_direction_array_order_and_positions() -> TestResult {
    let input = graph(
        &[(700, 7, Some(0)), (91, 1, Some(0)), (13, 6, None)],
        &[(99, 91, 13, S), (10, 13, 700, BondOrder::Triple)],
    )?;
    let before = input.clone();
    let output = normalize_neutral_kekule(&input, &modes(&input, &[]), || false)?;
    let mut atoms = input.atoms().to_vec();
    atoms.get_mut(2).ok_or("C13")?.explicit_hydrogens = Some(0);
    assert_eq!(
        output,
        Fragment::new(input.id(), atoms, input.bonds().to_vec())?
    );
    assert_eq!(input, before);
    Ok(())
}

#[test]
fn donor_must_have_an_actual_sp2_cycle_and_mixed_kekule_cycle_is_allowed() -> TestResult {
    for (z, h) in [(7, Some(1)), (8, Some(0))] {
        let input = ring(&[z, 6, 6, 6, 6], &[h, None, None, None, None], &[S; 5])?;
        refuses(&input, &modes(&input, &[1]), "unsupported_content")?;
    }
    let exocyclic = graph(
        &(1..=7).map(|id| (id, 6, None)).collect::<Vec<_>>(),
        &[
            (1, 1, 2, D),
            (2, 2, 3, S),
            (3, 3, 4, S),
            (4, 4, 5, S),
            (5, 5, 6, S),
            (6, 6, 7, S),
            (7, 7, 2, S),
        ],
    )?;
    refuses(&exocyclic, &modes(&exocyclic, &[2]), "unsupported_content")?;
    let mixed = ring(
        &[7, 6, 6, 6, 6],
        &[Some(1), None, None, None, None],
        &[S, D, S, D, S],
    )?;
    let output = normalize_neutral_kekule(&mixed, &modes(&mixed, &[1]), || false)?;
    assert_eq!(output.bonds(), mixed.bonds());
    assert!(
        output
            .atoms()
            .iter()
            .all(|a| a.explicit_hydrogens == Some(1))
    );
    Ok(())
}

#[test]
fn unsupported_valence_matching_markers_charge_and_radical_are_never_cleared() -> TestResult {
    for (input, marked) in [
        (graph(&[(1, 6, Some(5))], &[])?, vec![]),
        (
            graph(&[(1, 6, None), (2, 6, None)], &[(1, 1, 2, A)])?,
            vec![],
        ),
        (ring(&[6; 3], &[None; 3], &[A; 3])?, vec![1, 2, 3]),
        (ring(&[5, 6, 6], &[None; 3], &[A; 3])?, vec![1, 2, 3]),
        (
            ring(&[6; 4], &[Some(0), None, None, None], &[A; 4])?,
            vec![1, 2, 3, 4],
        ),
    ] {
        refuses(&input, &modes(&input, &marked), "unsupported_content")?;
    }
    let base = graph(&[(1, 6, Some(4))], &[])?;
    for charged in [true, false] {
        let mut atoms = base.atoms().to_vec();
        let carbon = atoms.first_mut().ok_or("C")?;
        if charged {
            carbon.charge = 1;
        } else {
            carbon.radical = Radical::Doublet;
        }
        let input = Fragment::new(base.id(), atoms, vec![])?;
        refuses(&input, &modes(&input, &[]), "unsupported_content")?;
    }
    Ok(())
}

#[test]
fn whole_input_caps_are_64_atoms_128_bonds_not_per_component_and_not_valence_errors() -> TestResult
{
    let atoms = (1..=64).map(|id| (id, 6, Some(0))).collect::<Vec<_>>();
    let mut bonds = Vec::new();
    for id in 1..=64 {
        for step in [1, 2] {
            bonds.push((
                u32::try_from(bonds.len() + 1)?,
                id,
                (id + step - 1) % 64 + 1,
                S,
            ));
        }
    }
    let at_limit = graph(&atoms, &bonds)?;
    assert_eq!(
        normalize_neutral_kekule(&at_limit, &modes(&at_limit, &[]), || false)?,
        at_limit
    );
    bonds.push((129, 1, 4, S));
    let too_many_bonds = graph(&atoms, &bonds)?;
    refuses(
        &too_many_bonds,
        &modes(&too_many_bonds, &[]),
        "resource_limit",
    )?;
    let too_many_atoms = graph(
        &(1..=65).map(|id| (id, 6, Some(4))).collect::<Vec<_>>(),
        &[],
    )?;
    refuses(
        &too_many_atoms,
        &modes(&too_many_atoms, &[]),
        "resource_limit",
    )
}

#[test]
fn cancellation_is_distinct_at_entry_and_during_bounded_work() -> TestResult {
    let input = graph(
        &(1..=64).map(|id| (id, 6, Some(4))).collect::<Vec<_>>(),
        &[],
    )?;
    let semantics = modes(&input, &[]);
    let before = input.clone();
    let early = normalize_neutral_kekule(&input, &semantics, || true)
        .err()
        .ok_or("early cancellation")?;
    assert_eq!(early.code(), "cancelled");
    let calls = Cell::new(0);
    let later = normalize_neutral_kekule(&input, &semantics, || {
        calls.set(calls.get() + 1);
        calls.get() >= 8
    })
    .err()
    .ok_or("cancellation during validation")?;
    assert_eq!(later.code(), "cancelled");
    assert_eq!(input, before);
    Ok(())
}
