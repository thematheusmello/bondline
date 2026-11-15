use bondline_io::smiles::SmilesDocument;
use bondline_model::{BondOrder, Radical};
use bondline_render::Cancellation;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn benzene_authoring_materializes_literal_h_and_a_complete_deterministic_kekule_graph() -> TestResult
{
    let source = b"c1ccccc1\r\n";
    let owner = SmilesDocument::parse(Arc::from(source.as_slice()))?;
    let before = owner.clone();
    let fragment = owner.normalize_authoring_fragment(&Cancellation::default())?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), 6);
    for (index, atom) in fragment.atoms().iter().enumerate() {
        assert_eq!(atom.id.get(), u32::try_from(index + 1)?);
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.explicit_hydrogens, Some(1));
        assert_eq!(atom.position, None);
        assert_eq!(atom.isotope, None);
        assert_eq!(atom.charge, 0);
        assert_eq!(atom.radical, Radical::None);
    }
    assert_eq!(
        fragment
            .bonds()
            .iter()
            .map(|bond| (bond.id.get(), bond.begin.get(), bond.end.get(), bond.order))
            .collect::<Vec<_>>(),
        [
            (1, 1, 2, BondOrder::Double),
            (2, 2, 3, BondOrder::Single),
            (3, 3, 4, BondOrder::Double),
            (4, 4, 5, BondOrder::Single),
            (5, 5, 6, BondOrder::Double),
            (6, 1, 6, BondOrder::Single),
        ],
        "lowest-demand atom and sorted neighbor IDs select the first complete matching"
    );
    assert_eq!(
        fragment
            .bonds()
            .iter()
            .filter(|bond| bond.order == BondOrder::Double)
            .count(),
        3
    );
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes(), source);
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
    assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
    assert!(owner.atoms().iter().all(|atom| atom.aromatic));
    assert!(
        owner
            .bonds()
            .iter()
            .all(|bond| bond.order == bondline_io::smiles::BondOrder::Aromatic)
    );
    Ok(())
}

use bondline_model::Fragment;
use serde_json::{Value, json};
use std::{
    io::Write as _,
    process::{Command, Output, Stdio},
};

fn literal_cases() -> TestResult<Vec<Value>> {
    Ok(serde_json::from_str(LITERAL_CASES)?)
}

fn parse(source: &str) -> TestResult<SmilesDocument> {
    Ok(SmilesDocument::parse(Arc::from(source.as_bytes()))?)
}

fn retained(owner: &SmilesDocument, before: &SmilesDocument) {
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
    assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
}

fn normalize(owner: &SmilesDocument) -> TestResult<Fragment> {
    Ok(owner.normalize_authoring_fragment(&Cancellation::default())?)
}

fn literal_graph(fragment: &Fragment, expected: &Value) -> TestResult {
    assert_eq!(fragment.id().get(), 1);
    let atoms = expected["atoms"].as_array().ok_or("literal atoms")?;
    assert_eq!(fragment.atoms().len(), atoms.len());
    for (index, (atom, literal)) in fragment.atoms().iter().zip(atoms).enumerate() {
        assert_eq!(atom.id.get(), u32::try_from(index + 1)?);
        assert_eq!(
            json!([
                atom.element.get(),
                atom.isotope.map(|value| value.get()),
                atom.explicit_hydrogens.ok_or("fixed H")?
            ]),
            *literal
        );
        assert_eq!(atom.position, None);
        assert_eq!(atom.charge, 0);
        assert_eq!(atom.radical, Radical::None);
    }
    let edges = fragment
        .bonds()
        .iter()
        .enumerate()
        .map(|(index, bond)| {
            assert_eq!(bond.id.get(), u32::try_from(index + 1)?);
            let order = match bond.order {
                BondOrder::Single => 1,
                BondOrder::Double => 2,
                BondOrder::Triple => 3,
                _ => return Err("authoring output must use integer bonds".into()),
            };
            Ok(json!([bond.begin.get(), bond.end.get(), order]))
        })
        .collect::<TestResult<Vec<Value>>>()?;
    assert_eq!(json!(edges), expected["bonds"]);
    Ok(())
}

fn external(fragment: &Fragment, source: &str, expected: &Value) -> TestResult<Output> {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let mut options = expected.clone();
    options.as_object_mut().ok_or("options")?.remove("source");
    let mut child = Command::new(python)
        .arg("-B")
        .arg(format!(
            "{}/tests/smiles_authoring_oracle.py",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg(source)
        .arg(options.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("stdin")?
        .write_all(&serde_json::to_vec(fragment)?)?;
    Ok(child.wait_with_output()?)
}

#[test]
fn source_ordered_hetero_mixed_antiaromatic_fused_isotope_and_component_vectors_are_literal()
-> TestResult {
    for expected in literal_cases()? {
        let source = expected["source"].as_str().ok_or("source")?;
        let owner = parse(source)?;
        let before = owner.clone();
        let old = owner.normalize_explicit_h_fragment();
        let fragment = normalize(&owner)?;
        literal_graph(&fragment, &expected)?;
        assert_eq!(
            normalize(&owner)?,
            fragment,
            "deterministic complete assignment for {source}"
        );
        if owner.atoms().iter().any(|atom| atom.aromatic)
            || owner
                .bonds()
                .iter()
                .any(|bond| bond.order == bondline_io::smiles::BondOrder::Aromatic)
        {
            let original_error = old.err().ok_or("old aromatic normalizer was widened")?;
            assert_eq!(original_error.code(), "unsupported_content");
            assert_eq!(
                owner.normalize_explicit_h_fragment().err(),
                Some(original_error)
            );
        } else {
            assert_eq!(
                old?, fragment,
                "nonaromatic output remains exactly the original normalizer"
            );
        }
        retained(&owner, &before);
    }
    Ok(())
}

#[test]
fn actual_authoring_fragments_face_the_new_qualified_external_chemistry_consumer() -> TestResult {
    for expected in literal_cases()? {
        let source = expected["source"].as_str().ok_or("source")?;
        let owner = parse(source)?;
        let before = owner.clone();
        let fragment = normalize(&owner)?;
        let output = external(&fragment, source, &expected)?;
        assert!(
            output.status.success(),
            "{source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let actual: Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(actual["atoms"], fragment.atoms().len());
        assert_eq!(actual["bonds"], fragment.bonds().len());
        assert_eq!(
            actual["components"],
            expected["components"].as_array().ok_or("components")?.len()
        );
        assert_eq!(actual["formula"], expected["formula"]);
        assert_eq!(
            actual["perceived_aromatic_atoms"],
            expected["perceived_aromatic_atoms"]
        );
        assert_eq!(
            actual["perceived_aromatic_bonds"],
            expected["perceived_aromatic_bonds"]
        );
        assert_eq!(actual["no_assigned_stereo"], true);
        assert_eq!(actual["no_radicals"], true);
        assert_eq!(
            actual["raw_graph"],
            json!({"atoms":expected["atoms"],"bonds":expected["bonds"],"components":expected["components"]})
        );
        retained(&owner, &before);
    }
    Ok(())
}

#[test]
fn source_spelling_and_old_nonaliphatic_refusals_stay_separate_from_new_projection() -> TestResult {
    for source in [
        "c2ccccc2",
        "c1ccccc1\n",
        "c1ccccc1\r\n",
        "[cH]1[cH][cH][cH][cH][cH]1",
        "c1c=cc=cc=1",
    ] {
        let owner = parse(source)?;
        let before = owner.clone();
        let actual = normalize(&owner)?;
        assert_eq!(actual.atoms().len(), 6);
        assert!(
            actual
                .atoms()
                .iter()
                .all(|atom| atom.explicit_hydrogens == Some(1) && atom.position.is_none())
        );
        assert_eq!(
            actual
                .bonds()
                .iter()
                .filter(|b| b.order == BondOrder::Double)
                .count(),
            3
        );
        assert_eq!(
            owner
                .normalize_explicit_h_fragment()
                .err()
                .ok_or("old API accepted aromatic source")?
                .code(),
            "unsupported_content"
        );
        retained(&owner, &before);
    }
    for source in [
        "CC",
        "CC(=O)O",
        "CC#N",
        "C[H]",
        "[2H]C",
        "[13CH4]",
        "C1CCCCC1.CC.O",
    ] {
        let owner = parse(source)?;
        let before = owner.clone();
        assert_eq!(
            normalize(&owner)?,
            owner.normalize_explicit_h_fragment()?,
            "exact old aliphatic data for {source}"
        );
        retained(&owner, &before);
    }
    Ok(())
}

#[test]
fn unsupported_whole_inputs_never_repair_h_fixed_bonds_stereo_charge_or_sp2_context() -> TestResult
{
    for source in [
        "c1cccc1",
        "n1cccc1",
        "cc",
        "C:C",
        "C:1CCCCC1",
        "[nH]1CCCC1",
        "o1CCCC1",
        "C=c1CCCCC1",
        "[nH+]1ccccc1",
        "[c]1ccccc1",
        "[oH]1cccc1",
        "[0cH]1ccccc1",
        "[0CH4]",
        "c1#ccccc1",
        "c1=cc=ccc=1",
        "s1cccc1",
        "b1ccccc1",
        "p1ccccc1",
        "[as]1ccccc1",
        "[se]1cccc1",
        "c1ccccc1.[NH4+]",
    ] {
        let owner = parse(source)?;
        let before = owner.clone();
        let error = owner
            .normalize_authoring_fragment(&Cancellation::default())
            .err()
            .ok_or("unsupported source returned a partial projection")?;
        assert_eq!(error.code(), "unsupported_content", "{source}: {error}");
        retained(&owner, &before);
    }
    for (source, code) in [
        ("c1ccccc", "invalid_smiles"),
        ("c1ccccc1)", "invalid_smiles"),
        ("c1ccccc1 PRIVATE", "invalid_smiles"),
        ("c1ccccc1\nCC", "invalid_smiles"),
        ("c1ccccc1[C@H](F)Cl", "unsupported_content"),
        ("F/C=C/c1ccccc1", "unsupported_content"),
    ] {
        let error = SmilesDocument::parse(Arc::from(source.as_bytes()))
            .err()
            .ok_or("existing parser silently accepted unsupported input")?;
        assert_eq!(error.code(), code);
    }
    Ok(())
}

fn dense_source(extra_bond: bool) -> TestResult<String> {
    // Original64-vertex circulant: cycle edges plus distance-two chords =>4-regular/E128.
    // The optional1-4 chord creates E129 before chemical validation, testing the input cap.
    let mut closures = vec![(0_usize, 63_usize)];
    for at in 0_usize..64 {
        let other = (at + 2) % 64;
        closures.push((at.min(other), at.max(other)));
    }
    if extra_bond {
        closures.push((0, 3));
    }
    let mut markers = vec![String::new(); 64];
    for (index, (a, b)) in closures.into_iter().enumerate() {
        let label = index + 1;
        let token = if label < 10 {
            label.to_string()
        } else {
            format!("%{label}")
        };
        markers.get_mut(a).ok_or("endpoint")?.push_str(&token);
        markers.get_mut(b).ok_or("endpoint")?.push_str(&token);
    }
    Ok(markers
        .into_iter()
        .map(|suffix| format!("C{suffix}"))
        .collect())
}

#[test]
fn total_atom_bond_caps_and_cancellation_precede_partial_or_unsupported_work() -> TestResult {
    let at_limit = format!("{}CCCC", "c1ccccc1.".repeat(10));
    let owner = parse(&at_limit)?;
    let before = owner.clone();
    let fragment = normalize(&owner)?;
    assert_eq!(fragment.atoms().len(), 64);
    assert_eq!(fragment.bonds().len(), 63);
    assert!(
        fragment
            .atoms()
            .iter()
            .take(60)
            .all(|a| a.explicit_hydrogens == Some(1))
    );
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .skip(60)
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(3), Some(2), Some(2), Some(3)]
    );
    assert_eq!(
        fragment
            .bonds()
            .iter()
            .filter(|b| b.order == BondOrder::Double)
            .count(),
        30
    );
    retained(&owner, &before);
    let dense = dense_source(false)?;
    let owner = parse(&dense)?;
    let before = owner.clone();
    assert_eq!(owner.atoms().len(), 64);
    assert_eq!(owner.bonds().len(), 128);
    let fragment = normalize(&owner)?;
    assert_eq!(fragment, owner.normalize_explicit_h_fragment()?);
    assert!(
        fragment.atoms().iter().all(|a| a.element.get() == 6
            && a.explicit_hydrogens == Some(0)
            && a.position.is_none())
    );
    for atom in fragment.atoms() {
        assert_eq!(
            fragment
                .bonds()
                .iter()
                .filter(|b| b.begin == atom.id || b.end == atom.id)
                .count(),
            4
        );
    }
    retained(&owner, &before);
    for source in [
        vec!["C"; 65].join("."),
        format!("[0cH]1ccccc1.{}", vec!["C"; 59].join(".")),
        dense_source(true)?,
    ] {
        let owner = parse(&source)?;
        let before = owner.clone();
        let error = owner
            .normalize_authoring_fragment(&Cancellation::default())
            .err()
            .ok_or("over-budget projection succeeded")?;
        assert_eq!(
            error.code(),
            "resource_limit",
            "whole graph cap must precede invalid-isotope or valence work"
        );
        let cancel = Cancellation::default();
        cancel.cancel();
        let error = owner
            .normalize_authoring_fragment(&cancel)
            .err()
            .ok_or("pre-cancelled projection succeeded")?;
        assert_eq!(
            error.code(),
            "cancelled",
            "cancellation checked before every other operation"
        );
        retained(&owner, &before);
    }
    let owner = parse("c1cc[nH]c1")?;
    let before = owner.clone();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        owner
            .normalize_authoring_fragment(&cancel)
            .err()
            .ok_or("cancelled matching succeeded")?
            .code(),
        "cancelled"
    );
    retained(&owner, &before);
    Ok(())
}

const LITERAL_CASES: &str = r#"[
  {
    "source": "c1ccccc1",
    "atoms": [
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        1,
        6,
        1
      ]
    ],
    "formula": "C6H6",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6
      ]
    ]
  },
  {
    "source": "C1=CC=CC=C1",
    "atoms": [
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        1,
        6,
        1
      ]
    ],
    "formula": "C6H6",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6
      ]
    ]
  },
  {
    "source": "n1ccccc1",
    "atoms": [
      [
        7,
        null,
        0
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        1,
        6,
        1
      ]
    ],
    "formula": "C5H5N",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6
      ]
    ]
  },
  {
    "source": "[nH]1cccc1",
    "atoms": [
      [
        7,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        1
      ],
      [
        2,
        3,
        2
      ],
      [
        3,
        4,
        1
      ],
      [
        4,
        5,
        2
      ],
      [
        1,
        5,
        1
      ]
    ],
    "formula": "C4H5N",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5
      ]
    ]
  },
  {
    "source": "c1cc[nH]c1",
    "atoms": [
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        7,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        1
      ],
      [
        2,
        3,
        2
      ],
      [
        3,
        4,
        1
      ],
      [
        4,
        5,
        1
      ],
      [
        1,
        5,
        2
      ]
    ],
    "formula": "C4H5N",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5
      ]
    ]
  },
  {
    "source": "o1cccc1",
    "atoms": [
      [
        8,
        null,
        0
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        1
      ],
      [
        2,
        3,
        2
      ],
      [
        3,
        4,
        1
      ],
      [
        4,
        5,
        2
      ],
      [
        1,
        5,
        1
      ]
    ],
    "formula": "C4H4O",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5
      ]
    ]
  },
  {
    "source": "[nH]1C=CC=C1",
    "atoms": [
      [
        7,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        1
      ],
      [
        2,
        3,
        2
      ],
      [
        3,
        4,
        1
      ],
      [
        4,
        5,
        2
      ],
      [
        1,
        5,
        1
      ]
    ],
    "formula": "C4H5N",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5
      ]
    ]
  },
  {
    "source": "o1C=CC=C1",
    "atoms": [
      [
        8,
        null,
        0
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        1
      ],
      [
        2,
        3,
        2
      ],
      [
        3,
        4,
        1
      ],
      [
        4,
        5,
        2
      ],
      [
        1,
        5,
        1
      ]
    ],
    "formula": "C4H4O",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5
      ]
    ]
  },
  {
    "source": "c1C=CC=CC=1",
    "atoms": [
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        1
      ],
      [
        2,
        3,
        2
      ],
      [
        3,
        4,
        1
      ],
      [
        4,
        5,
        2
      ],
      [
        5,
        6,
        1
      ],
      [
        1,
        6,
        2
      ]
    ],
    "formula": "C6H6",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6
      ]
    ]
  },
  {
    "source": "c1ccc1",
    "atoms": [
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        1,
        4,
        1
      ]
    ],
    "formula": "C4H4",
    "perceived_aromatic_atoms": [
      false,
      false,
      false,
      false
    ],
    "perceived_aromatic_bonds": [
      false,
      false,
      false,
      false
    ],
    "components": [
      [
        1,
        2,
        3,
        4
      ]
    ]
  },
  {
    "source": "c1ccccccc1",
    "atoms": [
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        6,
        7,
        1
      ],
      [
        7,
        8,
        2
      ],
      [
        1,
        8,
        1
      ]
    ],
    "formula": "C8H8",
    "perceived_aromatic_atoms": [
      false,
      false,
      false,
      false,
      false,
      false,
      false,
      false
    ],
    "perceived_aromatic_bonds": [
      false,
      false,
      false,
      false,
      false,
      false,
      false,
      false
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8
      ]
    ]
  },
  {
    "source": "[13cH]1ccccc1",
    "atoms": [
      [
        6,
        13,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        1,
        6,
        1
      ]
    ],
    "formula": "C6H6",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6
      ]
    ]
  },
  {
    "source": "[2H]n1cccc1",
    "atoms": [
      [
        1,
        2,
        0
      ],
      [
        7,
        null,
        0
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        1
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        2,
        6,
        1
      ]
    ],
    "formula": "C4H5N",
    "perceived_aromatic_atoms": [
      false,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      false,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6
      ]
    ]
  },
  {
    "source": "Cn1cccc1",
    "atoms": [
      [
        6,
        null,
        3
      ],
      [
        7,
        null,
        0
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        1
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        2,
        6,
        1
      ]
    ],
    "formula": "C5H7N",
    "perceived_aromatic_atoms": [
      false,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      false,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6
      ]
    ]
  },
  {
    "source": "c1ccccc1.CC",
    "atoms": [
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        3
      ],
      [
        6,
        null,
        3
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        1,
        6,
        1
      ],
      [
        7,
        8,
        1
      ]
    ],
    "formula": "C8H12",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true,
      true,
      false,
      false
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true,
      true,
      false
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6
      ],
      [
        7,
        8
      ]
    ]
  },
  {
    "source": "c1ccc2ccccc2c1",
    "atoms": [
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        0
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        0
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        6,
        7,
        1
      ],
      [
        7,
        8,
        2
      ],
      [
        8,
        9,
        1
      ],
      [
        4,
        9,
        1
      ],
      [
        9,
        10,
        2
      ],
      [
        1,
        10,
        1
      ]
    ],
    "formula": "C10H8",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9,
        10
      ]
    ]
  },
  {
    "source": "O=c1[nH]cccc1",
    "atoms": [
      [
        8,
        null,
        0
      ],
      [
        6,
        null,
        0
      ],
      [
        7,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        1
      ],
      [
        4,
        5,
        2
      ],
      [
        5,
        6,
        1
      ],
      [
        6,
        7,
        2
      ],
      [
        2,
        7,
        1
      ]
    ],
    "formula": "C5H5NO",
    "perceived_aromatic_atoms": [
      false,
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      false,
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6,
        7
      ]
    ]
  },
  {
    "source": "c1ccccc1-c2ccccc2",
    "atoms": [
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        0
      ],
      [
        6,
        null,
        0
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ],
      [
        6,
        null,
        1
      ]
    ],
    "bonds": [
      [
        1,
        2,
        2
      ],
      [
        2,
        3,
        1
      ],
      [
        3,
        4,
        2
      ],
      [
        4,
        5,
        1
      ],
      [
        5,
        6,
        2
      ],
      [
        1,
        6,
        1
      ],
      [
        6,
        7,
        1
      ],
      [
        7,
        8,
        2
      ],
      [
        8,
        9,
        1
      ],
      [
        9,
        10,
        2
      ],
      [
        10,
        11,
        1
      ],
      [
        11,
        12,
        2
      ],
      [
        7,
        12,
        1
      ]
    ],
    "formula": "C12H10",
    "perceived_aromatic_atoms": [
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "perceived_aromatic_bonds": [
      true,
      true,
      true,
      true,
      true,
      true,
      false,
      true,
      true,
      true,
      true,
      true,
      true
    ],
    "components": [
      [
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9,
        10,
        11,
        12
      ]
    ]
  }
]"#;
