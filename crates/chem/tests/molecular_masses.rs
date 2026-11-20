//! Published atomic data plus an independent, external whole-molecule consumer.
use bondline_chem::{Composition, MassUnavailable, MolecularMasses};
use bondline_model::{Atom, AtomId, Element};
use std::{num::NonZeroU16, process::Command};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Spec = (u8, u16, u16, i8);

fn masses(specs: &[Spec]) -> TestResult<MolecularMasses> {
    let mut atoms = Vec::new();
    for (i, &(z, isotope, h, charge)) in specs.iter().enumerate() {
        let mut atom = Atom::new(
            AtomId::new(u32::try_from(i + 1)?).ok_or("id")?,
            Element::try_from(z)?,
            None,
        );
        atom.isotope = NonZeroU16::new(isotope);
        atom.explicit_hydrogens = Some(h);
        atom.charge = charge;
        atoms.push(atom);
    }
    Ok(Composition::from_atoms(&atoms, || false)?.molecular_masses())
}

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected} ± {tolerance}"
    );
}

#[test]
fn neutral_ion_isotope_and_inline_h_agree_with_nist_and_codata() -> TestResult {
    const ELECTRON: f64 = 0.0005485799090441;
    for (spec, average, exact) in [
        (
            vec![(8, 0, 2, 0)],
            18.015,
            15.99491461957 + 2.0 * 1.00782503223,
        ),
        (
            vec![(8, 0, 0, 0), (1, 2, 0, 0), (1, 2, 0, 0)],
            15.999 + 2.0 * 2.01410177812,
            15.99491461957 + 2.0 * 2.01410177812,
        ),
        (
            vec![(6, 13, 3, -1)],
            13.00335483507 + 3.0 * 1.008,
            13.00335483507 + 3.0 * 1.00782503223 + ELECTRON,
        ),
        (vec![(26, 0, 0, 2)], 55.845, 55.93493633 - 2.0 * ELECTRON),
        (vec![(17, 0, 0, -1)], 35.45, 34.968852682 + ELECTRON),
        (vec![(43, 99, 0, 0)], 98.9062508, 98.9062508),
        (vec![], 0.0, 0.0),
    ] {
        let m = masses(&spec)?;
        close(m.molecular_weight?, average, 1e-9);
        close(m.exact_mass_da?, exact, 1e-9);
    }
    Ok(())
}

#[test]
fn all_elements_distinguish_natural_data_from_representative_mass_numbers() -> TestResult {
    for z in 1..=118 {
        let m = masses(&[(z, 0, 0, 0)])?;
        if matches!(z,43|61|84..=89|93..=118) {
            assert_eq!(
                m.molecular_weight,
                Err(MassUnavailable::NoStandardAtomicWeight { atomic_number: z })
            );
            assert_eq!(
                m.exact_mass_da,
                Err(MassUnavailable::NoNaturalIsotope { atomic_number: z })
            );
        } else {
            assert!(m.molecular_weight? > 1.0);
            assert!(m.exact_mass_da? > 1.0);
        }
    }
    let unknown = masses(&[(6, u16::MAX, 2, 0)])?;
    let reason = Err(MassUnavailable::UnknownIsotope {
        atomic_number: 6,
        mass_number: u16::MAX,
    });
    assert_eq!(unknown.molecular_weight, reason);
    assert_eq!(unknown.exact_mass_da, reason);
    Ok(())
}

#[test]
fn external_rdkit_calculates_whole_molecules_and_both_ion_signs() -> TestResult {
    let cases: &[(&str, &[Spec], f64, f64)] = &[
        (
            "CCO",
            &[(6, 0, 3, 0), (6, 0, 2, 0), (8, 0, 1, 0)],
            1e-6,
            1e-7,
        ),
        (
            "[2H]O[2H]",
            &[(1, 2, 0, 0), (8, 0, 0, 0), (1, 2, 0, 0)],
            1e-6,
            1e-7,
        ),
        ("[13CH3-]", &[(6, 13, 3, -1)], 1e-7, 1e-7),
        ("[Fe+2]", &[(26, 0, 0, 2)], 1e-6, 1e-7),
        ("[Cl-]", &[(17, 0, 0, -1)], 0.0031, 1e-7),
        ("[99Tc]", &[(43, 99, 0, 0)], 5e-6, 5e-6),
        ("[238U]", &[(92, 238, 0, 0)], 3e-7, 3e-7),
    ];
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Command::new(python).args(["-B", "-c", "import sys\nfrom rdkit import Chem\nfrom rdkit.Chem import Descriptors as D\nfor s in sys.argv[1:]:\n m=Chem.MolFromSmiles(s); print(D.MolWt(m), D.ExactMolWt(m))"]).args(cases.iter().map(|case|case.0)).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(stdout.lines().count(), cases.len());
    for ((notation, spec, avg_eps, exact_eps), line) in cases.iter().zip(stdout.lines()) {
        let (avg, exact) = line.split_once(' ').ok_or("RDKit output")?;
        let m = masses(spec)?;
        close(m.molecular_weight?, avg.parse()?, *avg_eps);
        // RDKit uses 55.9349375 for neutral Fe-56. Compare ionization
        // on the same atomic-data basis and pin the full NIST ion above.
        let rebased = if *notation == "[Fe+2]" {
            55.93493633 - 55.9349375
        } else {
            0.0
        };
        close(
            m.exact_mass_da?,
            exact.parse::<f64>()? + rebased,
            *exact_eps,
        );
    }
    Ok(())
}

#[test]
fn predominant_nuclides_agree_with_external_periodic_table_for_all_84_natural_elements()
-> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Command::new(python).args(["-B", "-c", "from rdkit import Chem\np=Chem.GetPeriodicTable()\nfor z in range(1,119): print(z, p.GetMostCommonIsotope(z), p.GetMostCommonIsotopeMass(z))"]).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut compared = 0;
    for line in std::str::from_utf8(&output.stdout)?.lines() {
        let fields = line
            .split_whitespace()
            .map(str::parse::<f64>)
            .collect::<Result<Vec<_>, _>>()?;
        let [z, isotope, mass] = fields.as_slice() else {
            return Err("RDKit row".into());
        };
        let z = *z as u8;
        if matches!(z,43|61|84..=89|93..=118) {
            continue;
        }
        let natural = masses(&[(z, 0, 0, 0)])?.exact_mass_da?;
        assert_eq!(
            natural,
            masses(&[(z, *isotope as u16, 0, 0)])?.exact_mass_da?,
            "predominant isotope of {z}"
        );
        // Published editions of atomic data differ slightly. This tolerance
        // distinguishes neighbouring nuclides by over four orders of magnitude.
        close(natural, *mass, 0.000_05);
        compared += 1;
    }
    assert_eq!(compared, 84);
    Ok(())
}
