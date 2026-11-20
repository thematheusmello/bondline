//! Independent values use CIAAW conventional weights, NIST labelled masses and decimal division.
use bondline_chem::{Composition, MassUnavailable};
use bondline_model::{Atom, AtomId, Element};
use std::num::NonZeroU16;

type Test<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Spec = (u8, u16, u16, i8);

fn composition(specs: &[Spec]) -> Test<Composition> {
    let mut atoms = Vec::new();
    for (i, &(z, a, h, charge)) in specs.iter().enumerate() {
        let mut atom = Atom::new(
            AtomId::new(u32::try_from(i + 1)?).ok_or("id")?,
            Element::try_from(z)?,
            None,
        );
        atom.isotope = NonZeroU16::new(a);
        atom.explicit_hydrogens = Some(h);
        atom.charge = charge;
        atoms.push(atom);
    }
    Ok(Composition::from_atoms(&atoms, || false)?)
}

fn check(specs: &[Spec], reference: &[(u8, &str, f64, f64)]) -> Test {
    let c = composition(specs)?;
    let (masses, rows) = c.masses_with_elemental_analysis();
    assert_eq!(masses, c.molecular_masses());
    let rows = rows?;
    assert_eq!(rows.len(), reference.len());
    for (row, &(z, symbol, contribution, percent)) in rows.iter().zip(reference) {
        assert_eq!((row.atomic_number, row.symbol), (z, symbol));
        assert!((row.weight_contribution - contribution).abs() < 1e-10);
        assert!(
            (row.percent_by_weight - percent).abs() < 1e-9,
            "{row:?} expected {percent}"
        );
    }
    if !rows.is_empty() {
        assert!((rows.iter().map(|r| r.percent_by_weight).sum::<f64>() - 100.0).abs() < 1e-10);
    }
    Ok(())
}

#[test]
fn grouped_isotopes_ordinary_carbon_free_ions_and_empty_composition() -> Test {
    check(
        &[(6, 0, 3, 0), (6, 0, 2, 0), (8, 0, 1, 0)],
        &[
            (1, "H", 6.048, 13.128133886127),
            (6, "C", 24.022, 52.143523844668),
            (8, "O", 15.999, 34.728342269205),
        ],
    )?;
    check(
        &[(1, 2, 0, 0), (8, 0, 1, 0)],
        &[
            (1, "H", 3.02210177812, 15.888153133150),
            (8, "O", 15.999, 84.111846866850),
        ],
    )?;
    check(
        &[(7, 0, 4, 1)],
        &[
            (1, "H", 4.032, 22.351571594878),
            (7, "N", 14.007, 77.648428405122),
        ],
    )?;
    check(&[(17, 37, 0, -1)], &[(17, "Cl", 36.965902602, 100.)])?;
    check(&[], &[])?;
    Ok(())
}

#[test]
fn incomplete_denominator_produces_no_partial_table() -> Test {
    for (last, expected) in [
        (
            (43, 0, 0, 0),
            MassUnavailable::NoStandardAtomicWeight { atomic_number: 43 },
        ),
        (
            (118, 293, 0, 0),
            MassUnavailable::UnknownIsotope {
                atomic_number: 118,
                mass_number: 293,
            },
        ),
    ] {
        let c = composition(&[(6, 0, 3, 0), last])?;
        let (masses, rows) = c.masses_with_elemental_analysis();
        assert_eq!(masses.molecular_weight, Err(expected));
        assert_eq!(rows, Err(expected));
    }
    Ok(())
}

#[test]
fn all_118_elements_fit_exactly_once_in_atomic_number_order() -> Test {
    // Original synthetic composition. These explicit nuclides are present
    // in the NIST all-isotope table, including for elements without a standard weight.
    let numbered = [
        (43, 97),
        (61, 145),
        (84, 209),
        (85, 210),
        (86, 222),
        (87, 223),
        (88, 226),
        (89, 227),
        (93, 236),
        (94, 238),
        (95, 241),
        (96, 243),
        (97, 247),
        (98, 249),
        (99, 252),
        (100, 257),
        (101, 258),
        (102, 259),
        (103, 262),
        (104, 267),
        (105, 268),
        (106, 271),
        (107, 270),
        (108, 269),
        (109, 278),
        (110, 281),
        (111, 281),
        (112, 285),
        (113, 284),
        (114, 289),
        (115, 288),
        (116, 293),
        (117, 294),
        (118, 294),
    ];
    let mut atoms: Vec<Spec> = (1..=118)
        .rev()
        .map(|z| {
            let mass = numbered
                .iter()
                .find(|(number, _)| *number == z)
                .map_or(0, |(_, a)| *a);
            (z, mass, 0, 0)
        })
        .collect();
    atoms.push((6, 13, 0, 0));
    let (masses, rows) = composition(&atoms)?.masses_with_elemental_analysis();
    let rows = rows?;
    assert_eq!(
        rows.iter().map(|r| r.atomic_number).collect::<Vec<_>>(),
        (1..=118).collect::<Vec<_>>()
    );
    assert!(
        rows.iter()
            .all(|row| row.percent_by_weight > 0.0 && row.percent_by_weight <= 100.0)
    );
    assert!(
        (rows.iter().map(|r| r.weight_contribution).sum::<f64>() - masses.molecular_weight?).abs()
            < 1e-9
    );
    assert!((rows.iter().map(|r| r.percent_by_weight).sum::<f64>() - 100.0).abs() < 1e-9);
    Ok(())
}
