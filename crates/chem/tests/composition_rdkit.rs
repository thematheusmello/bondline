//! Independent explicit-H references, parsed by an external RDKit process.
use bondline_chem::{Composition, analyze_fragment};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, DoublePosition, Element, Fragment, FragmentId,
};
use std::{
    io::Write,
    num::NonZeroU16,
    process::{Command, Output, Stdio},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type AtomSpec = (u8, Option<u16>, u16, i8);
const S: BondOrder = BondOrder::Single;
const D: BondOrder = BondOrder::Double;
const T: BondOrder = BondOrder::Triple;
const A: BondOrder = BondOrder::Aromatic;

struct Case {
    notation: &'static str,
    formula: &'static str,
    isotope_formula: &'static str,
    atoms: &'static [AtomSpec],
    bonds: &'static [(u32, u32, BondOrder)],
}

fn graph(case: &Case) -> TestResult<Fragment> {
    let atoms = case
        .atoms
        .iter()
        .enumerate()
        .map(|(index, &(z, isotope, h, charge))| {
            let id = AtomId::new(u32::try_from(index + 1)?).ok_or("atom")?;
            let mut atom = Atom::new(id, Element::try_from(z)?, None);
            atom.isotope = isotope
                .map(|n| NonZeroU16::new(n).ok_or("isotope"))
                .transpose()?;
            atom.explicit_hydrogens = Some(h);
            atom.charge = charge;
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = case
        .bonds
        .iter()
        .enumerate()
        .map(|(index, &(begin, end, order))| {
            Ok(Bond {
                id: BondId::new(u32::try_from(index + 1)?).ok_or("bond")?,
                begin: AtomId::new(begin).ok_or("begin")?,
                end: AtomId::new(end).ok_or("end")?,
                order,
                double_position: DoublePosition::Unspecified,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    Ok(Fragment::new(
        FragmentId::new(31).ok_or("fragment")?,
        atoms,
        bonds,
    )?)
}

// Only numeric validated values enter this tiny test transport; no serde dependency.
fn payload(composition: &Composition) -> String {
    let elements = composition
        .element_counts()
        .iter()
        .map(|n| format!("[{},{}]", n.element.get(), n.count))
        .collect::<Vec<_>>()
        .join(",");
    let isotopes = composition
        .isotope_counts()
        .iter()
        .map(|n| {
            let isotope = n
                .isotope
                .map_or_else(|| "null".to_owned(), |n| n.get().to_string());
            format!("[{},{},{}]", n.element.get(), isotope, n.count)
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"element_counts\":[{elements}],\"isotope_counts\":[{isotopes}],\"total_charge\":{}}}",
        composition.total_charge()
    )
}

fn oracle(
    notation: &str,
    formula: &str,
    isotope_formula: &str,
    payload: &str,
) -> TestResult<Output> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-X",
            "utf8",
            "-B",
            "-c",
            include_str!("composition_oracle.py"),
            notation,
            formula,
            isotope_formula,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(payload.as_bytes())?;
    Ok(child.wait_with_output()?)
}

#[test]
fn external_checker_accepts_literal_controls_and_rejects_semantic_or_schema_changes() -> TestResult
{
    // Durable subset of the manually authored, pre-production qualification controls.
    let methane = r#"{"element_counts":[[1,4],[6,1]],"isotope_counts":[[1,null,4],[6,null,1]],"total_charge":0}"#;
    let heavy = r#"{"element_counts":[[1,6],[6,2],[8,1]],"isotope_counts":[[1,null,5],[1,2,1],[6,null,1],[6,13,1],[8,null,1]],"total_charge":0}"#;
    assert!(oracle("[CH4]", "CH4", "CH4", methane)?.status.success());
    assert!(
        oracle("[2H][13CH2][CH2][OH]", "C2H6O", "C[13C]H5[2H]O", heavy)?
            .status
            .success()
    );
    for wrong in [
        r#"{"element_counts":[[1,3],[6,1]],"isotope_counts":[[1,null,4],[6,null,1]],"total_charge":0}"#,
        r#"{"element_counts":[[1,4],[6,1]],"isotope_counts":[[1,null,4],[6,null,1]],"total_charge":1}"#,
        r#"{"element_counts":[[1,4],[6,1]],"isotope_counts":[[1,1,4],[6,null,1]],"total_charge":0}"#,
        r#"{"element_counts":[[6,1],[1,4]],"isotope_counts":[[1,null,4],[6,null,1]],"total_charge":0}"#,
        r#"{"element_counts":[[1,4],[6,1]],"isotope_counts":[[1,null,4],[6,null,1]],"total_charge":false}"#,
        r#"{"element_counts":[[1,4],[6,1]],"isotope_counts":[[1,null,4],[6,null,1]],"total_charge":0,"extra":0}"#,
    ] {
        assert_eq!(oracle("[CH4]", "CH4", "CH4", wrong)?.status.code(), Some(1));
    }
    let collapsed = r#"{"element_counts":[[1,6],[6,2],[8,1]],"isotope_counts":[[1,null,6],[6,null,2],[8,null,1]],"total_charge":0}"#;
    assert_eq!(
        oracle("[2H][13CH2][CH2][OH]", "C2H6O", "C[13C]H5[2H]O", collapsed)?
            .status
            .code(),
        Some(1)
    );
    Ok(())
}

#[test]
fn explicitly_resolved_graphs_match_literal_formula_and_external_isotope_counters() -> TestResult {
    let cases = [
        Case {
            notation: "[CH4]",
            formula: "CH4",
            isotope_formula: "CH4",
            atoms: &[(6, None, 4, 0)],
            bonds: &[],
        },
        Case {
            notation: "[C]([H])([H])([H])[H]",
            formula: "CH4",
            isotope_formula: "CH4",
            atoms: &[
                (6, None, 0, 0),
                (1, None, 0, 0),
                (1, None, 0, 0),
                (1, None, 0, 0),
                (1, None, 0, 0),
            ],
            bonds: &[(1, 2, S), (1, 3, S), (1, 4, S), (1, 5, S)],
        },
        Case {
            notation: "[CH3][CH2][OH]",
            formula: "C2H6O",
            isotope_formula: "C2H6O",
            atoms: &[(6, None, 3, 0), (6, None, 2, 0), (8, None, 1, 0)],
            bonds: &[(1, 2, S), (2, 3, S)],
        },
        Case {
            notation: "[CH3][O][CH3]",
            formula: "C2H6O",
            isotope_formula: "C2H6O",
            atoms: &[(6, None, 3, 0), (8, None, 0, 0), (6, None, 3, 0)],
            bonds: &[(1, 2, S), (2, 3, S)],
        },
        Case {
            notation: "[CH2]=[O]",
            formula: "CH2O",
            isotope_formula: "CH2O",
            atoms: &[(6, None, 2, 0), (8, None, 0, 0)],
            bonds: &[(1, 2, D)],
        },
        Case {
            notation: "[CH]#[N]",
            formula: "CHN",
            isotope_formula: "CHN",
            atoms: &[(6, None, 1, 0), (7, None, 0, 0)],
            bonds: &[(1, 2, T)],
        },
        Case {
            notation: "[NH4+].[Cl-]",
            formula: "H4ClN",
            isotope_formula: "H4ClN",
            atoms: &[(7, None, 4, 1), (17, None, 0, -1)],
            bonds: &[],
        },
        Case {
            notation: "[H+]",
            formula: "H+",
            isotope_formula: "H+",
            atoms: &[(1, None, 0, 1)],
            bonds: &[],
        },
        Case {
            notation: "[OH-]",
            formula: "HO-",
            isotope_formula: "HO-",
            atoms: &[(8, None, 1, -1)],
            bonds: &[],
        },
        Case {
            notation: "[2H][13CH2][CH2][OH]",
            formula: "C2H6O",
            isotope_formula: "C[13C]H5[2H]O",
            atoms: &[
                (1, Some(2), 0, 0),
                (6, Some(13), 2, 0),
                (6, None, 2, 0),
                (8, None, 1, 0),
            ],
            bonds: &[(1, 2, S), (2, 3, S), (3, 4, S)],
        },
        Case {
            notation: "[2H][CH]([3H])[OH]",
            formula: "CH4O",
            isotope_formula: "CH2[2H][3H]O",
            atoms: &[
                (1, Some(2), 0, 0),
                (6, None, 1, 0),
                (1, Some(3), 0, 0),
                (8, None, 1, 0),
            ],
            bonds: &[(1, 2, S), (2, 3, S), (2, 4, S)],
        },
        Case {
            notation: "[cH]1[cH][cH][cH][cH][cH]1",
            formula: "C6H6",
            isotope_formula: "C6H6",
            atoms: &[(6, None, 1, 0); 6],
            bonds: &[
                (1, 2, A),
                (2, 3, A),
                (3, 4, A),
                (4, 5, A),
                (5, 6, A),
                (1, 6, A),
            ],
        },
        Case {
            notation: "[NH3+][CH2][C](=[O])[O-]",
            formula: "C2H5NO2",
            isotope_formula: "C2H5NO2",
            atoms: &[
                (7, None, 3, 1),
                (6, None, 2, 0),
                (6, None, 0, 0),
                (8, None, 0, 0),
                (8, None, 0, -1),
            ],
            bonds: &[(1, 2, S), (2, 3, S), (3, 4, D), (3, 5, S)],
        },
    ];
    for case in cases {
        let fragment = graph(&case)?;
        let original = fragment.clone();
        let result = analyze_fragment(&fragment, || false)?;
        let output = oracle(
            case.notation,
            case.formula,
            case.isotope_formula,
            &payload(&result),
        )?;
        assert!(
            output.status.success(),
            "{}: {}",
            case.notation,
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fragment, original);
    }
    Ok(())
}
