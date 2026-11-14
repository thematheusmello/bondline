use bondline_io::smiles::{BondOrder, Hydrogens, Limit, SmilesDocument, SmilesError};
use serde_json::{Value, json};
use std::{
    io::Write as _,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn source_ownership_and_literal_atoms_distinguish_implicit_and_bracket_hydrogen() -> TestResult {
    let source: Arc<[u8]> = Arc::from(b"CC\r\n".as_slice());
    let pointer = source.as_ptr();
    let document = SmilesDocument::parse(source)?;
    assert_eq!(document.as_bytes(), b"CC\r\n");
    assert_eq!(document.as_bytes().as_ptr(), pointer);
    assert_eq!(document.notation_range(), 0..2);
    assert_eq!(document.atoms().len(), 2);
    assert_eq!(document.atoms()[0].range, 0..1);
    assert_eq!(document.atoms()[1].range, 1..2);
    for atom in document.atoms() {
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.isotope, None);
        assert_eq!(atom.hydrogens, Hydrogens::Implicit);
        assert_eq!(atom.charge, 0);
    }
    assert_eq!(document.bonds().len(), 1);
    assert_eq!(document.bonds()[0].atoms, [0, 1]);
    assert_eq!(document.bonds()[0].order, BondOrder::Single);
    assert_eq!(document.bonds()[0].bond_range, None);
    assert_eq!(document.bonds()[0].ring, None);
    let clone = document.clone();
    assert_eq!(clone, document);
    assert_eq!(clone.as_bytes().as_ptr(), pointer);
    assert_eq!(clone.atoms().as_ptr(), document.atoms().as_ptr());
    assert_eq!(clone.bonds().as_ptr(), document.bonds().as_ptr());
    assert_eq!(
        format!("{document:?}"),
        "SmilesDocument { byte_length: 4, atom_count: 2, bond_count: 1 }"
    );

    let explicit = SmilesDocument::parse(Arc::from(b"[CH3][CH3]".as_slice()))?;
    assert_eq!(explicit.atoms()[0].range, 0..5);
    assert_eq!(explicit.atoms()[1].range, 5..10);
    assert_eq!(explicit.atoms()[0].hydrogens, Hydrogens::Bracket(3));
    assert_eq!(explicit.atoms()[0].hydrogen_range, Some(2..4));
    assert_eq!(explicit.atoms()[1].hydrogen_range, Some(7..9));
    assert_eq!(explicit.bonds()[0].atoms, [0, 1]);
    assert_ne!(explicit, document);
    Ok(())
}

fn oracle(source: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
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
            "{}/tests/smiles_oracle.py",
            env!("CARGO_MANIFEST_DIR")
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(source)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "external SMILES consumer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn actual_whole_source_and_notation_graph_face_the_pinned_unsanitized_external_parser() -> TestResult
{
    for text in [
        "CC",
        "CC\n",
        "CC\r\n",
        "[CH3][CH3]",
        "[C]",
        "[CH0]",
        "[CH3]",
        "[CH4]",
        "O",
        "[O]",
        "[OH2]",
        "[H][H]",
        "[2H][CH3]",
        "[CH2]([H])[2H]",
        "[13CH4]",
        "[0S]",
        "[NH4+].[Cl-]",
        "[Cu++]",
        "[I-15]",
        "CC(=O)O",
        "C#N",
        "C1CCCCC1",
        "C1.C1",
        "C0CC0C0CC0",
        "C=1CCCCC=1",
        "C%73C%74CC%73C%74",
        "C%123CCC%12CC3",
        "[Nh].[Mc].[Ts].[Og]",
        "[13CH3][CH2][OH]\n",
        "C%73CC%73.C%74CCC%74\n",
        "[Na+].[O-][CH2]C(C)C\n",
    ] {
        let document = parse(text)?;
        let external = oracle(document.as_bytes())?;
        assert_eq!(external["accepted"], true, "{text}: {external}");
        assert_eq!(external["source_ascii"], text);
        assert_eq!(external["source_bytes"], text.len());
        assert_eq!(external["rdkit"], "2026.09.2");
        assert_eq!(
            external["parameters"],
            json!({"sanitize":false, "removeHs":false, "parseName":false, "allowCXSMILES":false})
        );
        assert_eq!(external["skip_cleanup_exposed"], false);
        let atoms = external["atoms"].as_array().ok_or("external atoms")?;
        assert_eq!(atoms.len(), document.atoms().len(), "{text}");
        for (actual, atom) in atoms.iter().zip(document.atoms()) {
            assert_eq!(actual["element"], atom.element.get(), "{text}");
            assert_eq!(actual["isotope"], atom.isotope.unwrap_or(0), "{text}");
            assert_eq!(actual["charge"], atom.charge, "{text}");
            assert_eq!(actual["needs_update_cache"], true, "{text}");
            match atom.hydrogens {
                Hydrogens::Implicit => {
                    assert_eq!(actual["explicit_h"], 0, "{text}");
                    assert_eq!(actual["no_implicit"], false, "{text}");
                }
                Hydrogens::Bracket(count) => {
                    assert_eq!(actual["explicit_h"], count, "{text}");
                    assert_eq!(actual["no_implicit"], true, "{text}");
                }
            }
        }
        let mut expected = document
            .bonds()
            .iter()
            .map(|bond| {
                let [left, right] = bond.atoms;
                (
                    left.min(right),
                    left.max(right),
                    match bond.order {
                        BondOrder::Single => "SINGLE",
                        BondOrder::Double => "DOUBLE",
                        BondOrder::Triple => "TRIPLE",
                    }
                    .to_owned(),
                )
            })
            .collect::<Vec<_>>();
        expected.sort();
        let actual: Vec<(usize, usize, String)> =
            serde_json::from_value(external["bonds"].clone())?;
        assert_eq!(actual, expected, "{text}");
    }
    for text in ["c1ccccc1", "[C:17]", "*", "[C@H](F)(Cl)Br"] {
        assert_eq!(oracle(text.as_bytes())?["accepted"], true, "{text}");
        assert!(matches!(parse(text), Err(SmilesError::Unsupported { .. })));
    }
    assert_eq!(oracle(b"C%73CC%74")?["accepted"], false);
    assert!(parse("C%73CC%74").is_err());
    // Consumer acceptance differs from the declared H/closure rules and nonempty policy.
    for text in ["[HH]", "[HH0]", "C=1CC-1", ""] {
        assert_eq!(oracle(text.as_bytes())?["accepted"], true, "{text}");
        assert!(matches!(parse(text), Err(SmilesError::Syntax { .. })));
    }
    // These OpenSMILES grammar/label cases are outside the consumer's accepted dialect.
    for text in ["C1CCC%01", "C0CCC%00", "C(.N)O"] {
        assert!(parse(text).is_ok(), "{text}");
        assert_eq!(oracle(text.as_bytes())?["accepted"], false, "{text}");
    }
    Ok(())
}

#[test]
fn original_fuzz_seeds_keep_exact_checkout_bytes_and_independent_literal_graphs() -> TestResult {
    use Hydrogens::{Bracket, Implicit};
    for (source, expected_bytes, expected_atoms, expected_edges, chains) in [
        (
            include_bytes!("fixtures/smiles/isotopic-chain.smi").as_slice(),
            b"[13CH3][CH2][OH]\n".as_slice(),
            vec![
                (6, Some(13), 0, Bracket(3)),
                (6, None, 0, Bracket(2)),
                (8, None, 0, Bracket(1)),
            ],
            vec![[0, 1], [1, 2]],
            1,
        ),
        (
            include_bytes!("fixtures/smiles/two-ring-labels.smi").as_slice(),
            b"C%73CC%73.C%74CCC%74\n".as_slice(),
            vec![(6, None, 0, Implicit); 7],
            vec![[0, 1], [0, 2], [1, 2], [3, 4], [3, 6], [4, 5], [5, 6]],
            2,
        ),
        (
            include_bytes!("fixtures/smiles/salt-branch.smi").as_slice(),
            b"[Na+].[O-][CH2]C(C)C\n".as_slice(),
            vec![
                (11, None, 1, Bracket(0)),
                (8, None, -1, Bracket(0)),
                (6, None, 0, Bracket(2)),
                (6, None, 0, Implicit),
                (6, None, 0, Implicit),
                (6, None, 0, Implicit),
            ],
            vec![[1, 2], [2, 3], [3, 4], [3, 5]],
            2,
        ),
    ] {
        assert_eq!(source, expected_bytes);
        let document = SmilesDocument::parse(Arc::from(source))?;
        assert_eq!(document.as_bytes(), expected_bytes);
        assert_eq!(document.notation_range(), 0..source.len() - 1);
        assert_eq!(document.chain_starts(), chains);
        assert_eq!(
            document
                .atoms()
                .iter()
                .map(|a| (a.element.get(), a.isotope, a.charge, a.hydrogens))
                .collect::<Vec<_>>(),
            expected_atoms
        );
        let mut actual_edges = document
            .bonds()
            .iter()
            .map(|bond| {
                assert_eq!(bond.order, BondOrder::Single);
                [
                    bond.atoms[0].min(bond.atoms[1]),
                    bond.atoms[0].max(bond.atoms[1]),
                ]
            })
            .collect::<Vec<_>>();
        actual_edges.sort_unstable();
        assert_eq!(actual_edges, expected_edges);
    }
    Ok(())
}

fn parse(source: &str) -> Result<SmilesDocument, SmilesError> {
    SmilesDocument::parse(Arc::from(source.as_bytes()))
}

fn edges(document: &SmilesDocument) -> Vec<([usize; 2], BondOrder)> {
    document
        .bonds()
        .iter()
        .map(|bond| (bond.atoms, bond.order))
        .collect()
}

#[test]
fn brackets_preserve_zero_isotopes_hydrogen_spellings_elements_and_charges() -> TestResult {
    for (text, element, isotope, h, h_span, charge) in [
        ("[C]", 6, None, 0, None, 0),
        ("[CH0]", 6, None, 0, Some(2..4), 0),
        ("[CH]", 6, None, 1, Some(2..3), 0),
        ("[CH3]", 6, None, 3, Some(2..4), 0),
        ("[CH4]", 6, None, 4, Some(2..4), 0),
        ("[O]", 8, None, 0, None, 0),
        ("[OH2]", 8, None, 2, Some(2..4), 0),
        ("[H]", 1, None, 0, None, 0),
        ("[002H]", 1, Some(2), 0, None, 0),
        ("[13CH4]", 6, Some(13), 4, Some(4..6), 0),
        ("[0S]", 16, Some(0), 0, None, 0),
        ("[999U]", 92, Some(999), 0, None, 0),
        ("[65535C]", 6, Some(65535), 0, None, 0),
        ("[CH9]", 6, None, 9, Some(2..4), 0),
        ("[NH4+]", 7, None, 4, Some(2..4), 1),
        ("[Fe+15]", 26, None, 0, None, 15),
        ("[I-15]", 53, None, 0, None, -15),
        ("[Cu++]", 29, None, 0, None, 2),
        ("[O--]", 8, None, 0, None, -2),
        ("[C+00]", 6, None, 0, None, 0),
    ] {
        let document = parse(text)?;
        let atom = &document.atoms()[0];
        assert_eq!(atom.range, 0..text.len(), "{text}");
        assert_eq!(atom.element.get(), element, "{text}");
        assert_eq!(atom.isotope, isotope, "{text}");
        assert_eq!(atom.hydrogens, Hydrogens::Bracket(h), "{text}");
        assert_eq!(atom.hydrogen_range, h_span, "{text}");
        assert_eq!(atom.charge, charge, "{text}");
        assert_eq!(document.as_bytes(), text.as_bytes());
    }
    let zero = parse("[0S]")?;
    assert_eq!(zero.atoms()[0].isotope_range, Some(1..2));
    assert_ne!(zero, parse("[S]")?);
    let physical = parse("[H][H].[2H][CH3]")?;
    assert_eq!(
        physical
            .atoms()
            .iter()
            .map(|a| a.element.get())
            .collect::<Vec<_>>(),
        [1, 1, 1, 6]
    );
    assert_eq!(physical.atoms()[2].isotope, Some(2));
    assert_eq!(physical.atoms()[3].hydrogens, Hydrogens::Bracket(3));
    let organic = parse("BCNOFPSClBrI")?;
    assert_eq!(
        organic
            .atoms()
            .iter()
            .map(|a| a.element.get())
            .collect::<Vec<_>>(),
        [5, 6, 7, 8, 9, 15, 16, 17, 35, 53]
    );
    Ok(())
}

#[test]
fn literal_adjacency_covers_branches_orders_dots_and_cross_dot_ring_connections() -> TestResult {
    use BondOrder::{Double as D, Single as S, Triple as T};
    for (text, expected, chains) in [
        ("CC(=O)O", vec![([0, 1], S), ([1, 2], D), ([1, 3], S)], 1),
        ("C#N", vec![([0, 1], T)], 1),
        (
            "C0CCCCC0",
            vec![
                ([0, 1], S),
                ([1, 2], S),
                ([2, 3], S),
                ([3, 4], S),
                ([4, 5], S),
                ([0, 5], S),
            ],
            1,
        ),
        ("C1.C1", vec![([0, 1], S)], 2),
        ("C(.N)O", vec![([0, 2], S)], 2),
        ("C(C.C)N", vec![([0, 1], S), ([0, 3], S)], 2),
        (
            "C0CC0C0CC0",
            vec![
                ([0, 1], S),
                ([1, 2], S),
                ([0, 2], S),
                ([2, 3], S),
                ([3, 4], S),
                ([4, 5], S),
                ([3, 5], S),
            ],
            1,
        ),
        (
            "C%73C%74CC%73C%74",
            vec![
                ([0, 1], S),
                ([1, 2], S),
                ([2, 3], S),
                ([0, 3], S),
                ([3, 4], S),
                ([1, 4], S),
            ],
            1,
        ),
        (
            "C%123CCC%12CC3",
            vec![
                ([0, 1], S),
                ([1, 2], S),
                ([2, 3], S),
                ([0, 3], S),
                ([3, 4], S),
                ([4, 5], S),
                ([0, 5], S),
            ],
            1,
        ),
    ] {
        let document = parse(text)?;
        assert_eq!(edges(&document), expected, "{text}");
        assert_eq!(document.chain_starts(), chains, "{text}");
    }
    let branch = parse("CC(=O)O")?;
    assert_eq!(
        branch
            .atoms()
            .iter()
            .map(|a| a.range.clone())
            .collect::<Vec<_>>(),
        [0..1, 1..2, 4..5, 6..7]
    );
    assert_eq!(branch.bonds()[1].bond_range, Some(3..4));
    Ok(())
}

#[test]
fn ring_token_spans_numeric_aliases_and_both_bond_declarations_are_preserved() -> TestResult {
    for text in ["C=1CCCCC1", "C1CCCCC=1", "C=1CCCCC=1"] {
        let document = parse(text)?;
        let ring = document.bonds().last().ok_or("ring bond")?;
        assert_eq!(ring.atoms, [0, 5]);
        assert_eq!(ring.order, BondOrder::Double);
        assert_eq!(ring.bond_range, None);
    }
    let document = parse("C=1CCCCC=1")?;
    let ring = document.bonds()[5].ring.as_ref().ok_or("ring data")?;
    assert_eq!(ring.label, 1);
    assert_eq!(ring.open_range, 2..3);
    assert_eq!(ring.close_range, 9..10);
    assert_eq!(ring.open_bond_range, Some(1..2));
    assert_eq!(ring.close_bond_range, Some(8..9));
    for text in ["C1CCC%01", "C0CCC%00", "C%00CCC0"] {
        assert_eq!(parse(text)?.bonds()[3].atoms, [0, 3]);
    }
    Ok(())
}

#[test]
fn malformed_topology_properties_and_complete_eof_refuse_with_offsets() -> TestResult {
    for text in [
        "",
        "\n",
        "C(",
        "C()",
        "C((C))",
        "C(C))",
        "(C)",
        "C-",
        "-C",
        ".C",
        "C.",
        "C..C",
        "C(-.C)",
        "C(1C)",
        "C1",
        "C11",
        "C1C1",
        "C12CCC12",
        "C-1CCC=1",
        "C%73CC%74",
        "C%",
        "C%1",
        "C%(100)CC",
        "C(C)1CC1",
        "[",
        "[]",
        "[CH10]",
        "[HH]",
        "[HH0]",
        "[C+001]",
        "[C+++]",
        "[C-H3]",
        "C\nC",
        "C\n\n",
        "C\r",
        "C\t",
        "C name",
        "C\0",
    ] {
        let error = parse(text)
            .err()
            .ok_or_else(|| format!("accepted {text:?}"))?;
        assert_eq!(error.code(), "invalid_smiles", "{text:?}: {error}");
        assert!(error.offset() <= text.len(), "{text:?}: {error}");
    }
    for (text, offset) in [
        ("C)", 1),
        ("C\nC", 1),
        ("C name", 1),
        ("[HH]", 2),
        ("C11", 2),
        ("C%73CC%74", 1),
    ] {
        assert_eq!(
            parse(text).err().ok_or("expected error")?.offset(),
            offset,
            "{text}"
        );
    }
    for text in ["C", "C\n", "C\r\n"] {
        assert_eq!(parse(text)?.as_bytes(), text.as_bytes());
    }
    Ok(())
}

#[test]
fn unsupported_features_are_typed_and_never_stripped() -> TestResult {
    for text in [
        "c1ccccc1",
        "[nH]",
        "[C@H](F)(Cl)Br",
        "[C@AL1](F)Cl",
        "C/C=C\\C",
        "C:C",
        "[C:7]",
        "*",
        "[*]",
        "C$C",
        "C~C",
        "[C;H3]",
        "[#6]",
        "C|x|",
        "C>>O",
        "H",
        "[65536C]",
        "[Fe+16]",
        "é",
    ] {
        let error = parse(text)
            .err()
            .ok_or_else(|| format!("accepted {text:?}"))?;
        assert_eq!(error.code(), "unsupported_content", "{text:?}: {error}");
        assert!(matches!(error, SmilesError::Unsupported { .. }));
        assert!(error.offset() <= text.len());
    }
    Ok(())
}

fn ring_label(label: usize) -> String {
    if label < 10 {
        label.to_string()
    } else {
        format!("%{label:02}")
    }
}

fn dense_notation(extra: usize) -> String {
    let mut text = String::new();
    for index in 0..4096 {
        text.push('C');
        if index >= 2 {
            text.push_str(&ring_label((index - 2) % 2));
        }
        if (3..3 + extra).contains(&index) {
            text.push_str(&ring_label(index - 1));
        }
        if index < 4094 {
            text.push_str(&ring_label(index % 2));
        }
        if index < extra {
            text.push_str(&ring_label(index + 2));
        }
    }
    text
}

#[test]
fn all_resource_caps_are_enforced_before_graph_insertion_without_recursion() -> TestResult {
    let byte_boundary = format!("[{}C]", "0".repeat(65_533));
    assert_eq!(byte_boundary.len(), 65_536);
    assert_eq!(parse(&byte_boundary)?.atoms()[0].isotope, Some(0));
    assert!(matches!(
        parse(&(byte_boundary + "\n")),
        Err(SmilesError::Limit {
            kind: Limit::Bytes,
            offset: 65_536
        })
    ));
    assert_eq!(parse(&"C".repeat(4096))?.atoms().len(), 4096);
    assert!(matches!(
        parse(&"C".repeat(4097)),
        Err(SmilesError::Limit {
            kind: Limit::Atoms,
            ..
        })
    ));
    let branch = |count: usize| format!("{}C{}", "C(".repeat(count), ")".repeat(count));
    assert_eq!(parse(&branch(128))?.atoms().len(), 129);
    assert!(matches!(
        parse(&branch(129)),
        Err(SmilesError::Limit {
            kind: Limit::Branches,
            ..
        })
    ));
    assert_eq!(parse(&vec!["C"; 1024].join("."))?.chain_starts(), 1024);
    assert!(matches!(
        parse(&vec!["C"; 1025].join(".")),
        Err(SmilesError::Limit {
            kind: Limit::Chains,
            ..
        })
    ));
    let dense = parse(&dense_notation(3))?;
    assert_eq!(dense.atoms().len(), 4096);
    assert_eq!(dense.bonds().len(), 8192);
    assert!(matches!(
        parse(&dense_notation(4)),
        Err(SmilesError::Limit {
            kind: Limit::Bonds,
            ..
        })
    ));
    let labels = (0..100).map(ring_label).collect::<String>();
    let mut hundred = format!("C{labels}C");
    for label in 0..100 {
        hundred.push_str(&format!("C{}", ring_label(label)));
    }
    let document = parse(&hundred)?;
    assert_eq!(
        document
            .bonds()
            .iter()
            .filter(|bond| bond.ring.is_some())
            .count(),
        100
    );
    // The closed label domain has only 100 values; a 101st distinct open label has no spelling.
    assert!(parse("C%(100)CC").is_err());
    Ok(())
}
