//! Independent first numeric-composition declaration; no notation reader/writer.
use bondline_chem::{ElementCount, FormulaIsotopes, IsotopeCount, analyze_fragment};
use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId};
use std::num::NonZeroU16;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn carbon13_methane_counts_inline_h_without_inheriting_carbon_isotope() -> TestResult {
    let carbon = Element::try_from(6)?;
    let hydrogen = Element::try_from(1)?;
    let isotope = NonZeroU16::new(13).ok_or("isotope")?;
    let mut atom = Atom::new(AtomId::new(71).ok_or("atom")?, carbon, None);
    atom.isotope = Some(isotope);
    atom.explicit_hydrogens = Some(4);
    let fragment = Fragment::new(FragmentId::new(8).ok_or("fragment")?, vec![atom], vec![])?;
    let original = fragment.clone();
    let result = analyze_fragment(&fragment, || false)?;
    assert_eq!(
        result.element_counts(),
        &[
            ElementCount {
                element: hydrogen,
                count: 4
            },
            ElementCount {
                element: carbon,
                count: 1
            },
        ]
    );
    assert_eq!(
        result.isotope_counts(),
        &[
            IsotopeCount {
                element: hydrogen,
                isotope: None,
                count: 4
            },
            IsotopeCount {
                element: carbon,
                isotope: Some(isotope),
                count: 1
            },
        ]
    );
    assert_eq!(result.total_charge(), 0);
    assert_eq!(fragment, original);
    Ok(())
}

use bondline_chem::CompositionError;
use bondline_model::{Bond, BondId, BondOrder, DoublePosition, MAX_GRAPH_ITEMS, Point2, Radical};
use std::cell::Cell;

fn atom(id: u32, z: u8, isotope: Option<u16>, h: Option<u16>, charge: i8) -> TestResult<Atom> {
    let mut atom = Atom::new(AtomId::new(id).ok_or("atom")?, Element::try_from(z)?, None);
    atom.isotope = isotope
        .map(|n| NonZeroU16::new(n).ok_or("isotope"))
        .transpose()?;
    atom.explicit_hydrogens = h;
    atom.charge = charge;
    Ok(atom)
}

fn fragment(atoms: Vec<Atom>, bonds: &[(u32, u32, u32, BondOrder)]) -> TestResult<Fragment> {
    let bonds = bonds
        .iter()
        .map(|&(id, begin, end, order)| {
            Ok(Bond {
                id: BondId::new(id).ok_or("bond")?,
                begin: AtomId::new(begin).ok_or("begin")?,
                end: AtomId::new(end).ok_or("end")?,
                order,
                double_position: DoublePosition::Right,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    Ok(Fragment::new(
        FragmentId::new(53).ok_or("fragment")?,
        atoms,
        bonds,
    )?)
}

#[test]
fn natural_and_numbered_h_nodes_are_not_double_counted_or_collapsed() -> TestResult {
    // H, 1H, 2H and 3H are nodes, whereas C has two additional inline H.
    let graph = fragment(
        vec![
            atom(90, 6, Some(13), Some(2), 0)?,
            atom(8, 1, None, Some(0), 0)?,
            atom(7, 1, Some(1), Some(0), 0)?,
            atom(6, 1, Some(2), Some(0), 0)?,
            atom(5, 1, Some(3), Some(0), 0)?,
            atom(4, 6, None, Some(0), 0)?,
            atom(3, 6, Some(12), Some(0), 0)?,
        ],
        &[(900, 90, 8, BondOrder::Single)],
    )?;
    let original = graph.clone();
    let composition = analyze_fragment(&graph, || false)?;
    let h = Element::try_from(1)?;
    let c = Element::try_from(6)?;
    assert_eq!(
        composition.element_counts(),
        &[
            ElementCount {
                element: h,
                count: 6
            },
            ElementCount {
                element: c,
                count: 3
            },
        ]
    );
    assert_eq!(
        composition.isotope_counts(),
        &[
            IsotopeCount {
                element: h,
                isotope: None,
                count: 3
            },
            IsotopeCount {
                element: h,
                isotope: NonZeroU16::new(1),
                count: 1
            },
            IsotopeCount {
                element: h,
                isotope: NonZeroU16::new(2),
                count: 1
            },
            IsotopeCount {
                element: h,
                isotope: NonZeroU16::new(3),
                count: 1
            },
            IsotopeCount {
                element: c,
                isotope: None,
                count: 1
            },
            IsotopeCount {
                element: c,
                isotope: NonZeroU16::new(12),
                count: 1
            },
            IsotopeCount {
                element: c,
                isotope: NonZeroU16::new(13),
                count: 1
            },
        ]
    );
    assert_eq!(composition.total_charge(), 0);
    assert_eq!(composition.hill_formula(FormulaIsotopes::Combined)?, "C3H6");
    assert_eq!(
        composition.hill_formula(FormulaIsotopes::Separate)?,
        "C[12C][13C]H3[1H][2H][3H]"
    );
    assert_eq!(graph, original);
    Ok(())
}

#[test]
fn unknown_h_refuses_even_on_physical_h_and_reports_authored_array_order() -> TestResult {
    for z in [1, 6, 8, 118] {
        let graph = fragment(
            vec![
                atom(1, 8, None, Some(0), 0)?,
                atom(700, z, None, None, 0)?,
                atom(2, 6, None, None, 0)?,
            ],
            &[],
        )?;
        let before = graph.clone();
        assert_eq!(
            analyze_fragment(&graph, || false),
            Err(CompositionError::UnresolvedHydrogens(
                AtomId::new(700).ok_or("atom")?
            ))
        );
        assert_eq!(graph, before);
    }
    // Explicit zero is fully specified; no carbon valence is inferred.
    let zero = fragment(vec![atom(6, 6, None, Some(0), 0)?], &[])?;
    let composition = analyze_fragment(&zero, || false)?;
    assert_eq!(
        composition.element_counts(),
        &[ElementCount {
            element: Element::try_from(6)?,
            count: 1
        }]
    );
    assert_eq!(composition.isotope_counts().len(), 1);
    Ok(())
}

#[test]
fn charge_metadata_and_all_bond_kinds_do_not_infer_or_change_composition() -> TestResult {
    let orders = [
        BondOrder::Single,
        BondOrder::Double,
        BondOrder::Triple,
        BondOrder::Quadruple,
        BondOrder::Quintuple,
        BondOrder::Sextuple,
        BondOrder::Half,
        BondOrder::Aromatic,
        BondOrder::TwoAndHalf,
        BondOrder::ThreeAndHalf,
        BondOrder::FourAndHalf,
        BondOrder::FiveAndHalf,
        BondOrder::Dative,
        BondOrder::Ionic,
        BondOrder::Hydrogen,
        BondOrder::ThreeCenter,
    ];
    for (index, order) in orders.into_iter().enumerate() {
        let mut oxygen = atom(u32::MAX, 8, Some(18), Some(1), i8::MIN)?;
        oxygen.position = Some(Point2::new(12.25, -17.0)?);
        oxygen.radical = match index % 4 {
            0 => Radical::None,
            1 => Radical::Singlet,
            2 => Radical::Doublet,
            _ => Radical::Triplet,
        };
        oxygen.show_terminal_carbon_labels = Some(false);
        oxygen.show_non_terminal_carbon_labels = Some(true);
        let graph = fragment(
            vec![
                oxygen,
                atom(10, 7, None, Some(2), i8::MIN)?,
                atom(2, 6, None, Some(3), i8::MAX)?,
            ],
            &[(123, u32::MAX, 10, order)],
        )?;
        let before = graph.clone();
        let result = analyze_fragment(&graph, || false)?;
        assert_eq!(result.total_charge(), -129);
        assert_eq!(
            result.element_counts(),
            &[
                ElementCount {
                    element: Element::try_from(1)?,
                    count: 6
                },
                ElementCount {
                    element: Element::try_from(6)?,
                    count: 1
                },
                ElementCount {
                    element: Element::try_from(7)?,
                    count: 1
                },
                ElementCount {
                    element: Element::try_from(8)?,
                    count: 1
                },
            ]
        );
        assert_eq!(graph, before);
    }
    Ok(())
}

#[test]
fn empty_isomeric_and_permuted_graphs_have_literal_not_topological_totals() -> TestResult {
    let empty = fragment(vec![], &[])?;
    let zero = analyze_fragment(&empty, || false)?;
    assert!(zero.element_counts().is_empty());
    assert!(zero.isotope_counts().is_empty());
    assert_eq!(zero.total_charge(), 0);
    assert_eq!(zero.hill_formula(FormulaIsotopes::Separate)?, "");
    let ethanol = fragment(
        vec![
            atom(9, 6, None, Some(3), 0)?,
            atom(4, 6, None, Some(2), 0)?,
            atom(1, 8, None, Some(1), 0)?,
        ],
        &[(50, 9, 4, BondOrder::Single), (51, 4, 1, BondOrder::Single)],
    )?;
    let ether = fragment(
        vec![
            atom(1, 6, None, Some(3), 0)?,
            atom(9, 8, None, Some(0), 0)?,
            atom(4, 6, None, Some(3), 0)?,
        ],
        &[(8, 1, 9, BondOrder::Single), (2, 9, 4, BondOrder::Single)],
    )?;
    let a = analyze_fragment(&ethanol, || false)?;
    let b = analyze_fragment(&ether, || false)?;
    assert_ne!(ethanol, ether);
    assert_eq!(a, b);
    assert_eq!(
        a.element_counts(),
        &[
            ElementCount {
                element: Element::try_from(1)?,
                count: 6
            },
            ElementCount {
                element: Element::try_from(6)?,
                count: 2
            },
            ElementCount {
                element: Element::try_from(8)?,
                count: 1
            },
        ]
    );
    let mut atoms = ethanol.atoms().to_vec();
    atoms.reverse();
    let mut bonds = ethanol.bonds().to_vec();
    bonds.reverse();
    for bond in &mut bonds {
        std::mem::swap(&mut bond.begin, &mut bond.end);
    }
    let permuted = Fragment::new(ethanol.id(), atoms, bonds)?;
    assert_eq!(analyze_fragment(&permuted, || false)?, a);
    Ok(())
}

#[test]
fn all_model_elements_and_isotope_numbers_are_literal_without_existence_claims() -> TestResult {
    let atoms = (1..=118)
        .map(|z| atom(u32::from(z), z, Some(u16::MAX), Some(0), 0))
        .collect::<TestResult<Vec<_>>>()?;
    let graph = fragment(atoms, &[])?;
    let result = analyze_fragment(&graph, || false)?;
    assert_eq!(result.element_counts().len(), 118);
    assert_eq!(result.isotope_counts().len(), 118);
    for (index, (element, isotope)) in result
        .element_counts()
        .iter()
        .zip(result.isotope_counts())
        .enumerate()
    {
        assert_eq!(usize::from(element.element.get()), index + 1);
        assert_eq!(element.count, 1);
        assert_eq!(isotope.element, element.element);
        assert_eq!(isotope.isotope, NonZeroU16::new(u16::MAX));
        assert_eq!(isotope.count, 1);
    }
    assert_eq!(result.total_charge(), 0);
    Ok(())
}

#[test]
fn full_model_atom_and_bond_bounds_do_not_wrap_u32_counts_or_i8_charges() -> TestResult {
    assert_eq!(MAX_GRAPH_ITEMS, 100_000);
    // This is an arithmetic control on literal model fields, not a valid molecule.
    let atoms = (1..=MAX_GRAPH_ITEMS)
        .map(|n| atom(u32::try_from(n)?, 1, None, Some(u16::MAX), i8::MAX))
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = (1..=MAX_GRAPH_ITEMS)
        .map(|n| {
            Ok(Bond {
                id: BondId::new(u32::try_from(n)?).ok_or("bond")?,
                begin: AtomId::new(u32::try_from(n)?).ok_or("begin")?,
                end: AtomId::new(u32::try_from(n % MAX_GRAPH_ITEMS + 1)?).ok_or("end")?,
                order: BondOrder::Single,
                double_position: DoublePosition::Unspecified,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let graph = Fragment::new(FragmentId::new(7).ok_or("fragment")?, atoms, bonds)?;
    let result = analyze_fragment(&graph, || false)?;
    let hydrogen = Element::try_from(1)?;
    assert_eq!(
        result.element_counts(),
        &[ElementCount {
            element: hydrogen,
            count: 6_553_600_000
        }]
    );
    assert_eq!(
        result.isotope_counts(),
        &[IsotopeCount {
            element: hydrogen,
            isotope: None,
            count: 6_553_600_000
        }]
    );
    assert_eq!(result.total_charge(), 12_700_000);
    assert_eq!(
        result.hill_formula(FormulaIsotopes::Combined)?,
        "H6553600000+12700000"
    );
    let atoms = graph
        .atoms()
        .iter()
        .cloned()
        .map(|mut atom| {
            atom.charge = i8::MIN;
            atom
        })
        .collect();
    let negative = Fragment::new(graph.id(), atoms, graph.bonds().to_vec())?;
    let negative = analyze_fragment(&negative, || false)?;
    assert_eq!(negative.total_charge(), -12_800_000);
    assert_eq!(
        negative.hill_formula(FormulaIsotopes::Separate)?,
        "H6553600000-12800000"
    );
    Ok(())
}

#[test]
fn cancellation_precedes_unknown_h_and_interrupts_a_nonempty_scan_without_mutation() -> TestResult {
    let unresolved = fragment(vec![atom(8, 6, None, None, 0)?], &[])?;
    assert_eq!(
        analyze_fragment(&unresolved, || true),
        Err(CompositionError::Cancelled)
    );
    let empty = fragment(vec![], &[])?;
    assert_eq!(
        analyze_fragment(&empty, || true),
        Err(CompositionError::Cancelled)
    );
    let graph = fragment(
        (1..=100)
            .map(|n| atom(n, 6, None, Some(4), 0))
            .collect::<TestResult<Vec<_>>>()?,
        &[],
    )?;
    let original = graph.clone();
    let checks = Cell::new(0);
    assert_eq!(
        analyze_fragment(&graph, || {
            checks.set(checks.get() + 1);
            checks.get() >= 5
        }),
        Err(CompositionError::Cancelled)
    );
    assert!(checks.get() >= 5);
    assert_eq!(graph, original);
    assert_eq!(
        analyze_fragment(&graph, || false)?.element_counts()[0].count,
        400
    );
    Ok(())
}

#[test]
fn strict_hill_order_uses_symbols_and_plural_isotope_bins() -> TestResult {
    let mut atoms = vec![
        atom(8, 17, None, Some(0), -1)?,
        atom(2, 7, None, Some(4), 1)?,
        atom(4, 1, Some(2), Some(0), 0)?,
        atom(9, 1, Some(2), Some(0), 0)?,
    ];
    let no_carbon = analyze_fragment(&fragment(atoms.clone(), &[])?, || false)?;
    assert_eq!(no_carbon.hill_formula(FormulaIsotopes::Combined)?, "ClH6N");
    assert_eq!(
        no_carbon.hill_formula(FormulaIsotopes::Separate)?,
        "ClH4[2H]2N"
    );
    atoms.push(atom(5, 6, Some(u16::MAX), Some(0), 0)?);
    atoms.reverse();
    let with_carbon = analyze_fragment(&fragment(atoms, &[])?, || false)?;
    assert_eq!(
        with_carbon.hill_formula(FormulaIsotopes::Combined)?,
        "CH6ClN"
    );
    assert_eq!(
        with_carbon.hill_formula(FormulaIsotopes::Separate)?,
        "[65535C]H4[2H]2ClN"
    );
    Ok(())
}

#[test]
fn a_source_default_supplies_only_an_omitted_atom_count() -> TestResult {
    let mut carbon = Atom::new(AtomId::new(1).ok_or("carbon")?, Element::try_from(6)?, None);
    carbon.explicit_hydrogens = Some(0);
    let nitrogen = Atom::new(
        AtomId::new(2).ok_or("nitrogen")?,
        Element::try_from(7)?,
        None,
    );
    let atoms = [carbon, nitrogen];
    let composition = bondline_chem::Composition::from_atoms_with_hydrogen_defaults(
        &atoms,
        |id| {
            assert_eq!(id, atoms[1].id);
            Some(3)
        },
        || false,
    )?;
    assert_eq!(composition.hill_formula(FormulaIsotopes::Separate)?, "CH3N");
    assert!(
        matches!(bondline_chem::Composition::from_atoms(&atoms, || false),
        Err(bondline_chem::CompositionError::UnresolvedHydrogens(id)) if id == atoms[1].id)
    );
    Ok(())
}
