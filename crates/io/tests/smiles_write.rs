use bondline_io::smiles::{Limit, SmilesDocument, WriteError, write_fragment};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Point2, Radical,
};
use serde_json::{Value, json};
use std::{
    io::Write as _,
    num::NonZeroU16,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn atom(id: u32, element: u8, hydrogens: u16) -> Result<Atom, Box<dyn std::error::Error>> {
    let mut atom = Atom::new(
        AtomId::new(id).ok_or("atom ID")?,
        Element::new(element).ok_or("element")?,
        None,
    );
    atom.explicit_hydrogens = Some(hydrogens);
    Ok(atom)
}

#[test]
fn literal_branches_cycles_orders_isotopes_and_physical_h_preserve_authored_counts() -> TestResult {
    use BondOrder::{Double as D, Single as S, Triple as T};
    for (atoms, bonds, expected) in [
        (
            vec![
                atom(1, 6, 1)?,
                atom(2, 6, 3)?,
                atom(3, 6, 3)?,
                atom(4, 6, 3)?,
            ],
            vec![(1, 2, S), (1, 3, S), (1, 4, S)],
            "[CH1]([CH3])([CH3])[CH3]",
        ),
        (
            vec![atom(1, 6, 2)?, atom(2, 6, 2)?],
            vec![(1, 2, D)],
            "[CH2]=[CH2]",
        ),
        (
            vec![atom(1, 6, 1)?, atom(2, 6, 1)?],
            vec![(1, 2, T)],
            "[CH1]#[CH1]",
        ),
        (vec![atom(1, 6, 4)?, atom(2, 8, 2)?], vec![], "[CH4].[OH2]"),
        (
            vec![atom(1, 1, 0)?, atom(2, 1, 0)?],
            vec![(1, 2, S)],
            "[H][H]",
        ),
        (
            (1..=6).map(|id| atom(id, 6, 2)).collect::<Result<_, _>>()?,
            vec![
                (1, 2, S),
                (2, 3, S),
                (3, 4, S),
                (4, 5, S),
                (5, 6, S),
                (6, 1, S),
            ],
            "[CH2]0[CH2][CH2][CH2][CH2][CH2]0",
        ),
    ] {
        let input = fragment(atoms, &bonds)?;
        let before = input.clone();
        assert_eq!(write_fragment(&input)?, expected.as_bytes());
        assert_eq!(input, before);
    }
    let mut carbon = atom(1, 6, 2)?;
    carbon.isotope = NonZeroU16::new(13);
    let mut deuterium = atom(3, 1, 0)?;
    deuterium.isotope = NonZeroU16::new(2);
    let physical = fragment(
        vec![carbon, atom(2, 1, 0)?, deuterium],
        &[(1, 2, S), (1, 3, S)],
    )?;
    assert_eq!(write_fragment(&physical)?, b"[13CH2]([H])[2H]");
    for (z, h, expected) in [
        (7, 3, "[NH3]"),
        (8, 2, "[OH2]"),
        (9, 1, "[FH1]"),
        (17, 1, "[ClH1]"),
        (35, 1, "[BrH1]"),
        (53, 1, "[IH1]"),
    ] {
        assert_eq!(
            write_fragment(&fragment(vec![atom(1, z, h)?], &[])?)?,
            expected.as_bytes()
        );
    }
    let mut maximum_mass = atom(1, 6, 4)?;
    maximum_mass.isotope = NonZeroU16::new(u16::MAX);
    assert_eq!(
        write_fragment(&fragment(vec![maximum_mass], &[])?)?,
        b"[65535CH4]"
    );
    for (order, weight, token) in [(D, 2_u16, "="), (T, 3_u16, "#")] {
        let mut atoms = (1..=8)
            .map(|id| atom(id, 6, 2))
            .collect::<Result<Vec<_>, _>>()?;
        atoms[0].explicit_hydrogens = Some(3 - weight);
        atoms[7].explicit_hydrogens = Some(3 - weight);
        let mut bonds = (1..8).map(|id| (id, id + 1, S)).collect::<Vec<_>>();
        bonds.push((8, 1, order));
        let input = fragment(atoms, &bonds)?;
        let expected = format!(
            "[CH{}]{token}0{}[CH{}]0",
            3 - weight,
            "[CH2]".repeat(6),
            3 - weight
        );
        assert_eq!(write_fragment(&input)?, expected.as_bytes());
    }
    Ok(())
}

#[test]
fn ids_choose_deterministic_traversal_independent_of_storage_and_bond_direction() -> TestResult {
    use BondOrder::Single as S;
    let input = fragment(
        vec![
            atom(10, 6, 1)?,
            atom(20, 6, 3)?,
            atom(30, 6, 3)?,
            atom(40, 6, 3)?,
        ],
        &[(10, 20, S), (10, 30, S), (10, 40, S)],
    )?;
    let shuffled = fragment(
        vec![
            input.atoms()[3].clone(),
            input.atoms()[1].clone(),
            input.atoms()[0].clone(),
            input.atoms()[2].clone(),
        ],
        &[(40, 10, S), (30, 10, S), (20, 10, S)],
    )?;
    assert_eq!(write_fragment(&input)?, b"[CH1]([CH3])([CH3])[CH3]");
    assert_eq!(write_fragment(&shuffled)?, write_fragment(&input)?);
    Ok(())
}

#[test]
fn unsupported_metadata_unspecified_h_and_finite_valence_errors_leave_input_exact() -> TestResult {
    for kind in [
        "missing_h",
        "charge",
        "radical",
        "element",
        "wrong_h",
        "huge_h",
        "isolated_physical_h",
        "physical_attached_h",
    ] {
        let mut first = atom(1, 6, 4)?;
        match kind {
            "missing_h" => first.explicit_hydrogens = None,
            "charge" => first.charge = 1,
            "radical" => first.radical = Radical::Doublet,
            "element" => first.element = Element::new(15).ok_or("P")?,
            "wrong_h" => first.explicit_hydrogens = Some(0),
            "huge_h" => first.explicit_hydrogens = Some(u16::MAX),
            "isolated_physical_h" => {
                first.element = Element::new(1).ok_or("H")?;
                first.explicit_hydrogens = Some(0);
            }
            "physical_attached_h" => {
                first.element = Element::new(1).ok_or("H")?;
                first.explicit_hydrogens = Some(1);
            }
            _ => return Err("test case".into()),
        }
        let input = fragment(vec![first], &[])?;
        let before = input.clone();
        let error = write_fragment(&input)
            .err()
            .ok_or_else(|| format!("accepted {kind}"))?;
        assert_eq!(error.code(), "unsupported_content", "{kind}: {error}");
        if kind == "wrong_h" || kind == "huge_h" {
            assert!(matches!(
                error,
                WriteError::Valence {
                    atom_id: 1,
                    expected: 4,
                    bond_order_sum: 0,
                    ..
                }
            ));
        } else {
            assert!(matches!(
                error,
                WriteError::UnsupportedAtom { atom_id: 1, .. }
            ));
        }
        assert_eq!(input, before);
    }
    for order in [
        BondOrder::Aromatic,
        BondOrder::Dative,
        BondOrder::Quadruple,
        BondOrder::Hydrogen,
    ] {
        let input = fragment(vec![atom(1, 6, 3)?, atom(2, 6, 3)?], &[(1, 2, order)])?;
        let before = input.clone();
        assert!(matches!(
            write_fragment(&input),
            Err(WriteError::UnsupportedBond { bond_id: 1, .. })
        ));
        assert_eq!(input, before);
    }
    let wrong = fragment(
        vec![atom(1, 6, 4)?, atom(2, 6, 3)?],
        &[(1, 2, BondOrder::Single)],
    )?;
    assert!(matches!(
        write_fragment(&wrong),
        Err(WriteError::Valence {
            atom_id: 1,
            expected: 4,
            bond_order_sum: 1,
            hydrogens: 4
        })
    ));
    let empty = fragment(vec![], &[])?;
    assert!(matches!(write_fragment(&empty), Err(WriteError::Empty)));
    Ok(())
}

fn carbon_graph(
    count: usize,
    edges: &[(usize, usize)],
) -> Result<Fragment, Box<dyn std::error::Error>> {
    let mut degree = vec![0_u16; count];
    for &(left, right) in edges {
        degree[left] += 1;
        degree[right] += 1;
    }
    let atoms = degree
        .into_iter()
        .enumerate()
        .map(|(index, degree)| {
            atom(
                u32::try_from(index + 1)?,
                6,
                4_u16.checked_sub(degree).ok_or("degree above4")?,
            )
        })
        .collect::<Result<_, _>>()?;
    let bonds = edges
        .iter()
        .map(|&(left, right)| {
            Ok((
                u32::try_from(left + 1)?,
                u32::try_from(right + 1)?,
                BondOrder::Single,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    fragment(atoms, &bonds)
}

fn comb(depth: usize) -> Result<Fragment, Box<dyn std::error::Error>> {
    let mut edges = Vec::new();
    for index in 0..depth {
        edges.push((index, index + 1));
        edges.push((index, depth + 1 + index));
    }
    carbon_graph(2 * depth + 1, &edges)
}

fn open_rings(count: usize) -> Result<Fragment, Box<dyn std::error::Error>> {
    let mut edges = (0..2 * count)
        .map(|index| (index, index + 1))
        .collect::<Vec<_>>();
    edges.extend((0..count).map(|index| (index, count + 1 + index)));
    carbon_graph(2 * count + 1, &edges)
}

#[test]
fn closed_ring_labels_reuse_and_simultaneous_label_budget_are_independent() -> TestResult {
    let edges = (0..1001)
        .flat_map(|i| {
            [
                (3 * i, 3 * i + 1),
                (3 * i + 1, 3 * i + 2),
                (3 * i + 2, 3 * i),
            ]
        })
        .collect::<Vec<_>>();
    let many = carbon_graph(3003, &edges)?;
    let output = write_fragment(&many)?;
    assert_eq!(output, vec!["[CH2]0[CH2][CH2]0"; 1001].join(".").as_bytes());
    let hundred = open_rings(100)?;
    let output = write_fragment(&hundred)?;
    assert_eq!(std::str::from_utf8(&output)?.matches("%73").count(), 2);
    assert_eq!(std::str::from_utf8(&output)?.matches("%74").count(), 2);
    assert_eq!(SmilesDocument::parse(Arc::from(output))?.bonds().len(), 300);
    let excessive = open_rings(101)?;
    let before = excessive.clone();
    assert!(matches!(
        write_fragment(&excessive),
        Err(WriteError::Limit { kind: Limit::Rings })
    ));
    assert_eq!(excessive, before);
    Ok(())
}

#[test]
fn atom_bond_component_and_actual_parenthesis_depth_boundaries_are_checked() -> TestResult {
    let path =
        |count: usize| carbon_graph(count, &(1..count).map(|i| (i - 1, i)).collect::<Vec<_>>());
    let maximum = path(4096)?;
    let output = write_fragment(&maximum)?;
    assert_eq!(output.len(), 4096 * 5);
    assert!(!output.contains(&b'('));
    assert!(matches!(
        write_fragment(&path(4097)?),
        Err(WriteError::Limit { kind: Limit::Atoms })
    ));
    assert!(write_fragment(&comb(128)?).is_ok());
    assert!(matches!(
        write_fragment(&comb(129)?),
        Err(WriteError::Limit {
            kind: Limit::Branches
        })
    ));
    assert!(write_fragment(&carbon_graph(1024, &[])?).is_ok());
    assert!(matches!(
        write_fragment(&carbon_graph(1025, &[])?),
        Err(WriteError::Limit {
            kind: Limit::Chains
        })
    ));
    let mut edges = Vec::new();
    for group in 0..818 {
        for a in 0..5 {
            for b in a + 1..5 {
                edges.push((5 * group + a, 5 * group + b));
            }
        }
    }
    for a in 0..6 {
        for b in a + 1..6 {
            if a / 2 != b / 2 {
                edges.push((4090 + a, 4090 + b));
            }
        }
    }
    let dense = carbon_graph(4096, &edges)?;
    assert_eq!(dense.bonds().len(), 8192);
    assert!(write_fragment(&dense).is_ok());
    let mut bonds = dense
        .bonds()
        .iter()
        .map(|b| (b.begin.get(), b.end.get(), b.order))
        .collect::<Vec<_>>();
    bonds.push((1, 6, BondOrder::Single));
    let over = fragment(dense.atoms().to_vec(), &bonds)?;
    let before = over.clone();
    assert!(matches!(
        write_fragment(&over),
        Err(WriteError::Limit { kind: Limit::Bonds })
    ));
    assert_eq!(over, before);
    Ok(())
}

fn external(source: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(format!(
            "{}/tests/smiles_write_oracle.py",
            env!("CARGO_MANIFEST_DIR")
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(source)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "writer external consumer: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn actual_generated_bytes_have_independent_unsanitized_graph_h_and_sanitized_meaning() -> TestResult
{
    for (atoms, bonds, smiles, formula) in [
        (
            vec![atom(1, 6, 3)?, atom(2, 6, 3)?],
            vec![(1, 2, BondOrder::Single)],
            "CC",
            "C2H6",
        ),
        (
            vec![atom(1, 6, 3)?, atom(2, 6, 2)?, atom(3, 6, 3)?],
            vec![(1, 2, BondOrder::Single), (2, 3, BondOrder::Single)],
            "CCC",
            "C3H8",
        ),
        (
            vec![atom(1, 6, 2)?, atom(2, 6, 2)?],
            vec![(1, 2, BondOrder::Double)],
            "C=C",
            "C2H4",
        ),
        (
            vec![atom(1, 6, 1)?, atom(2, 6, 1)?],
            vec![(1, 2, BondOrder::Triple)],
            "C#C",
            "C2H2",
        ),
    ] {
        let model = fragment(atoms, &bonds)?;
        let result = external(&write_fragment(&model)?)?;
        assert_eq!(result["notation"]["accepted"], true);
        assert_eq!(result["meaning"]["smiles"], smiles);
        assert_eq!(result["meaning"]["formula"], formula);
        assert_eq!(
            result["meaning"]["radicals"],
            json!(vec![0; model.atoms().len()])
        );
        assert_eq!(result["meaning"]["conformers"], 0);
        for (actual, expected) in result["notation"]["atoms"]
            .as_array()
            .ok_or("atoms")?
            .iter()
            .zip(model.atoms())
        {
            assert_eq!(actual["element"], expected.element.get());
            assert_eq!(
                actual["explicit_h"],
                expected.explicit_hydrogens.ok_or("H")?
            );
            assert_eq!(actual["isotope"], 0);
            assert_eq!(actual["no_implicit"], true);
        }
    }
    // Unique authored isotopes make the external atom-to-model mapping independent of traversal.
    let mut model = open_rings(100)?;
    let mut atoms = model.atoms().to_vec();
    for atom in &mut atoms {
        atom.isotope = NonZeroU16::new(u16::try_from(atom.id.get())?);
    }
    model = Fragment::new(model.id(), atoms, model.bonds().to_vec())?;
    let output = write_fragment(&model)?;
    let result = external(&output)?;
    let actual_atoms = result["notation"]["atoms"]
        .as_array()
        .ok_or("external atoms")?;
    let masses = actual_atoms
        .iter()
        .map(|a| a["isotope"].as_u64().ok_or("isotope"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut actual = result["notation"]["bonds"]
        .as_array()
        .ok_or("external bonds")?
        .iter()
        .map(|b| {
            let left = masses[usize::try_from(b[0].as_u64().ok_or("left")?)?];
            let right = masses[usize::try_from(b[1].as_u64().ok_or("right")?)?];
            Ok((
                left.min(right),
                left.max(right),
                b[2].as_str().ok_or("order")?.to_owned(),
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    actual.sort();
    let mut expected = model
        .bonds()
        .iter()
        .map(|b| {
            (
                u64::from(b.begin.get().min(b.end.get())),
                u64::from(b.begin.get().max(b.end.get())),
                "SINGLE".to_owned(),
            )
        })
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(actual, expected);
    for actual in actual_atoms {
        let original = model
            .atoms()
            .iter()
            .find(|a| u64::from(a.id.get()) == actual["isotope"].as_u64().unwrap_or(0))
            .ok_or("model atom")?;
        assert_eq!(
            actual["explicit_h"],
            original.explicit_hydrogens.ok_or("model H")?
        );
    }
    let mut carbon = atom(1, 6, 2)?;
    carbon.isotope = NonZeroU16::new(13);
    let mut hydrogen = atom(3, 1, 0)?;
    hydrogen.isotope = NonZeroU16::new(2);
    let model = fragment(
        vec![carbon, atom(2, 1, 0)?, hydrogen],
        &[(1, 2, BondOrder::Single), (1, 3, BondOrder::Single)],
    )?;
    let result = external(&write_fragment(&model)?)?;
    assert_eq!(
        result["notation"]["atoms"]
            .as_array()
            .ok_or("physical atoms")?
            .len(),
        3
    );
    assert_eq!(result["notation"]["atoms"][0]["explicit_h"], 2);
    assert_eq!(result["notation"]["atoms"][0]["isotope"], 13);
    assert_eq!(result["notation"]["atoms"][2]["isotope"], 2);
    assert_eq!(
        result["notation"]["bonds"],
        json!([[0, 1, "SINGLE"], [0, 2, "SINGLE"]])
    );
    Ok(())
}

fn fragment(
    atoms: Vec<Atom>,
    bonds: &[(u32, u32, BondOrder)],
) -> Result<Fragment, Box<dyn std::error::Error>> {
    Ok(Fragment::new(
        FragmentId::new(1).ok_or("fragment ID")?,
        atoms,
        bonds
            .iter()
            .enumerate()
            .map(|(index, (begin, end, order))| {
                Ok(Bond {
                    double_position: bondline_model::DoublePosition::Unspecified,
                    id: BondId::new(u32::try_from(index + 1)?).ok_or("bond ID")?,
                    begin: AtomId::new(*begin).ok_or("begin")?,
                    end: AtomId::new(*end).ok_or("end")?,
                    order: *order,
                })
            })
            .collect::<Result<_, Box<dyn std::error::Error>>>()?,
    )?)
}

#[test]
fn explicit_h_ethane_and_propane_export_literal_brackets_without_changing_model() -> TestResult {
    let mut first = atom(10, 6, 3)?;
    first.position = Some(Point2::new(72.0, 144.0)?);
    let ethane = fragment(vec![first, atom(20, 6, 3)?], &[(10, 20, BondOrder::Single)])?;
    let before = ethane.clone();
    assert_eq!(write_fragment(&ethane)?, b"[CH3][CH3]");
    assert_eq!(ethane, before);
    let propane = fragment(
        vec![atom(1, 6, 3)?, atom(2, 6, 2)?, atom(3, 6, 3)?],
        &[(1, 2, BondOrder::Single), (2, 3, BondOrder::Single)],
    )?;
    assert_eq!(write_fragment(&propane)?, b"[CH3][CH2][CH3]");
    Ok(())
}
