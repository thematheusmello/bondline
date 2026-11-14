use bondline_io::smiles::{
    BondOrder, Hydrogens, Limit, NormalizeError, SmilesDocument, SmilesError,
};
use serde_json::{Value, json};
use std::{
    io::Write as _,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

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
fn aromatic_cycle_has_literal_adjacency_without_perceiving_chemistry() -> TestResult {
    let document = SmilesDocument::parse(Arc::from(b"c1ccccc1".as_slice()))?;
    assert_eq!(document.as_bytes(), b"c1ccccc1");
    assert_eq!(document.atoms().len(), 6);
    assert!(document.atoms().iter().all(|atom| atom.aromatic));
    assert_eq!(document.bonds()[0].order, BondOrder::Aromatic);
    assert_eq!(
        document
            .bonds()
            .iter()
            .map(|bond| (bond.atoms, bond.order))
            .collect::<Vec<_>>(),
        [
            ([0, 1], BondOrder::Aromatic),
            ([1, 2], BondOrder::Aromatic),
            ([2, 3], BondOrder::Aromatic),
            ([3, 4], BondOrder::Aromatic),
            ([4, 5], BondOrder::Aromatic),
            ([0, 5], BondOrder::Aromatic),
        ]
    );
    Ok(())
}

#[test]
fn lowercase_symbols_and_bracket_metadata_remain_literal_and_source_owned() -> TestResult {
    for (text, element, aromatic, symbol, hydrogens) in [
        ("b", 5, true, 0..1, Hydrogens::Implicit),
        ("c", 6, true, 0..1, Hydrogens::Implicit),
        ("n", 7, true, 0..1, Hydrogens::Implicit),
        ("o", 8, true, 0..1, Hydrogens::Implicit),
        ("p", 15, true, 0..1, Hydrogens::Implicit),
        ("s", 16, true, 0..1, Hydrogens::Implicit),
        ("[b]", 5, true, 1..2, Hydrogens::Bracket(0)),
        ("[c]", 6, true, 1..2, Hydrogens::Bracket(0)),
        ("[n]", 7, true, 1..2, Hydrogens::Bracket(0)),
        ("[o]", 8, true, 1..2, Hydrogens::Bracket(0)),
        ("[p]", 15, true, 1..2, Hydrogens::Bracket(0)),
        ("[s]", 16, true, 1..2, Hydrogens::Bracket(0)),
        ("[as]", 33, true, 1..3, Hydrogens::Bracket(0)),
        ("[se]", 34, true, 1..3, Hydrogens::Bracket(0)),
        ("[As]", 33, false, 1..3, Hydrogens::Bracket(0)),
        ("[Se]", 34, false, 1..3, Hydrogens::Bracket(0)),
        ("[nH]", 7, true, 1..2, Hydrogens::Bracket(1)),
        ("[nH0]", 7, true, 1..2, Hydrogens::Bracket(0)),
    ] {
        let document = parse(text)?;
        let atom = &document.atoms()[0];
        assert_eq!(atom.element.get(), element, "{text}");
        assert_eq!(atom.aromatic, aromatic, "{text}");
        assert_eq!(atom.symbol_range, symbol, "{text}");
        assert_eq!(atom.range, 0..text.len(), "{text}");
        assert_eq!(atom.hydrogens, hydrogens, "{text}");
        assert_eq!(atom.isotope, None, "{text}");
        assert_eq!(atom.charge, 0, "{text}");
    }
    assert_eq!(parse("[n]")?.atoms()[0].hydrogen_range, None);
    assert_eq!(parse("[nH]")?.atoms()[0].hydrogen_range, Some(2..3));
    assert_eq!(parse("[nH0]")?.atoms()[0].hydrogen_range, Some(2..4));

    let source: Arc<[u8]> = Arc::from(b"[013nH1+1]([2H])[0cH0-]\r\n".as_slice());
    let pointer = source.as_ptr();
    let document = SmilesDocument::parse(source)?;
    let atoms = document.atoms();
    assert_eq!(document.notation_range(), 0..23);
    assert_eq!(atoms[0].range, 0..10);
    assert_eq!(atoms[0].symbol_range, 4..5);
    assert_eq!(atoms[0].isotope, Some(13));
    assert_eq!(atoms[0].isotope_range, Some(1..4));
    assert_eq!(atoms[0].hydrogens, Hydrogens::Bracket(1));
    assert_eq!(atoms[0].hydrogen_range, Some(5..7));
    assert_eq!(atoms[0].charge, 1);
    assert_eq!(atoms[0].charge_range, Some(7..9));
    assert!(atoms[0].aromatic);
    assert_eq!(atoms[1].range, 11..15);
    assert_eq!(atoms[1].symbol_range, 13..14);
    assert_eq!(atoms[1].element.get(), 1);
    assert_eq!(atoms[1].isotope, Some(2));
    assert_eq!(atoms[1].isotope_range, Some(12..13));
    assert_eq!(atoms[1].hydrogens, Hydrogens::Bracket(0));
    assert_eq!(atoms[1].hydrogen_range, None);
    assert!(!atoms[1].aromatic);
    assert_eq!(atoms[2].range, 16..23);
    assert_eq!(atoms[2].symbol_range, 18..19);
    assert_eq!(atoms[2].isotope, Some(0));
    assert_eq!(atoms[2].isotope_range, Some(17..18));
    assert_eq!(atoms[2].hydrogens, Hydrogens::Bracket(0));
    assert_eq!(atoms[2].hydrogen_range, Some(19..21));
    assert_eq!(atoms[2].charge, -1);
    assert_eq!(atoms[2].charge_range, Some(21..22));
    assert!(atoms[2].aromatic);
    assert_eq!(
        edges(&document),
        [([0, 1], BondOrder::Single), ([0, 2], BondOrder::Aromatic)]
    );
    let clone = document.clone();
    assert_eq!(document, clone);
    assert_eq!(document.as_bytes(), b"[013nH1+1]([2H])[0cH0-]\r\n");
    assert_eq!(document.as_bytes().as_ptr(), pointer);
    assert_eq!(clone.as_bytes().as_ptr(), pointer);
    assert_eq!(clone.atoms().as_ptr(), atoms.as_ptr());
    assert_eq!(clone.bonds().as_ptr(), document.bonds().as_ptr());
    assert_eq!(
        format!("{document:?}"),
        "SmilesDocument { byte_length: 25, atom_count: 3, bond_count: 2 }"
    );
    assert_ne!(parse("[0c]")?, parse("[c]")?);
    Ok(())
}

#[test]
fn implicit_and_explicit_bond_orders_do_not_promote_uppercase_atom_markers() -> TestResult {
    use BondOrder::{Aromatic as A, Double as D, Single as S, Triple as T};
    for (source, flags, expected, ranges) in [
        ("cc", vec![true, true], vec![([0, 1], A)], vec![None]),
        ("c-c", vec![true, true], vec![([0, 1], S)], vec![Some(1..2)]),
        ("c:c", vec![true, true], vec![([0, 1], A)], vec![Some(1..2)]),
        ("cC", vec![true, false], vec![([0, 1], S)], vec![None]),
        ("Cc", vec![false, true], vec![([0, 1], S)], vec![None]),
        (
            "C:C",
            vec![false, false],
            vec![([0, 1], A)],
            vec![Some(1..2)],
        ),
        (
            "c:C",
            vec![true, false],
            vec![([0, 1], A)],
            vec![Some(1..2)],
        ),
        ("c=c", vec![true, true], vec![([0, 1], D)], vec![Some(1..2)]),
        ("c#c", vec![true, true], vec![([0, 1], T)], vec![Some(1..2)]),
        (
            "c(C)c",
            vec![true, false, true],
            vec![([0, 1], S), ([0, 2], A)],
            vec![None, None],
        ),
        (
            "C(:c)C",
            vec![false, true, false],
            vec![([0, 1], A), ([0, 2], S)],
            vec![Some(2..3), None],
        ),
        ("[se][As]", vec![true, false], vec![([0, 1], S)], vec![None]),
    ] {
        let document = parse(source)?;
        assert_eq!(
            document
                .atoms()
                .iter()
                .map(|atom| atom.aromatic)
                .collect::<Vec<_>>(),
            flags,
            "{source}"
        );
        assert_eq!(edges(&document), expected, "{source}");
        assert_eq!(
            document
                .bonds()
                .iter()
                .map(|bond| bond.bond_range.clone())
                .collect::<Vec<_>>(),
            ranges,
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn ring_defaults_use_both_endpoints_and_explicit_declarations_keep_precedence() -> TestResult {
    use BondOrder::{Aromatic as A, Double as D, Single as S, Triple as T};
    for (source, order) in [
        ("c1cccc1", A),
        ("c-1cccc1", S),
        ("c1cccc-1", S),
        ("c-1cccc-1", S),
        ("c:1cccc1", A),
        ("c1cccc:1", A),
        ("c:1cccc:1", A),
        ("c1cccC1", S),
        ("c:1cccC1", A),
        ("c1cccC:1", A),
        ("C:1CCC1", A),
        ("C1CCC:1", A),
        ("C:1CCC:1", A),
        ("c=1cccc1", D),
        ("c1cccc#1", T),
    ] {
        let document = parse(source)?;
        let closure = document.bonds().last().ok_or("closure")?;
        assert_eq!(closure.order, order, "{source}");
        assert_eq!(closure.bond_range, None, "{source}");
    }
    let document = parse("c:1cccc:1")?;
    let closure = document.bonds().last().ok_or("closure")?;
    assert_eq!(closure.atoms, [0, 4]);
    let spans = closure.ring.as_ref().ok_or("ring spans")?;
    assert_eq!(spans.open_range, 2..3);
    assert_eq!(spans.close_range, 8..9);
    assert_eq!(spans.open_bond_range, Some(1..2));
    assert_eq!(spans.close_bond_range, Some(7..8));
    for source in ["c1ccc%01", "c0ccc%00"] {
        assert_eq!(parse(source)?.bonds()[3].order, A);
    }
    let distinct = parse("c%73cc%74ccc%73cc%74")?;
    assert_eq!(distinct.atoms().len(), 8);
    assert_eq!(distinct.bonds().len(), 9);
    assert_eq!(
        distinct
            .bonds()
            .iter()
            .filter_map(|bond| bond
                .ring
                .as_ref()
                .map(|ring| (ring.label, bond.atoms, bond.order)))
            .collect::<Vec<_>>(),
        [(73, [0, 5], A), (74, [2, 7], A)]
    );
    assert_eq!(
        edges(&parse("c0cc0c0cc0")?),
        [
            ([0, 1], A),
            ([1, 2], A),
            ([0, 2], A),
            ([2, 3], A),
            ([3, 4], A),
            ([4, 5], A),
            ([3, 5], A)
        ]
    );
    for (source, order) in [("c1.c1", A), ("c1.C1", S), ("c:1.C1", A)] {
        let cross = parse(source)?;
        assert_eq!(cross.chain_starts(), 2);
        assert_eq!(edges(&cross), [([0, 1], order)]);
    }
    Ok(())
}

#[test]
fn unsupported_aromatic_spellings_and_invalid_topology_remain_typed_and_bounded() -> TestResult {
    for (source, offset) in [
        ("as", 0),
        ("se", 1),
        ("[te]1cccc1", 1),
        ("[si]1ccccc1", 1),
        ("[sb]1ccccc1", 1),
        ("[n@H]1cccc1", 2),
        ("[n:7]", 2),
        ("c/c=c\\c", 1),
        ("c$c", 1),
        ("c~c", 1),
        ("[65536c]", 5),
    ] {
        let error = parse(source).err().ok_or("accepted unsupported notation")?;
        assert_eq!(error.code(), "unsupported_content", "{source}: {error}");
        assert_eq!(error.offset(), offset, "{source}");
    }
    for (source, offset) in [
        ("c:1cccc-1", 8),
        ("c-1cccc:1", 8),
        ("c::c", 2),
        (":cc", 0),
        ("cc:", 3),
        ("c:11", 3),
        ("c1c:1", 4),
        ("c1cc", 1),
        ("c1ccccc1 name", 8),
        ("c\nc", 1),
        ("c\r", 1),
        ("c\n\n", 1),
    ] {
        let error = parse(source).err().ok_or("accepted invalid notation")?;
        assert_eq!(error.code(), "invalid_smiles", "{source}: {error}");
        assert_eq!(error.offset(), offset, "{source}");
    }
    Ok(())
}

#[test]
fn normalization_refuses_any_aromatic_marker_before_fixed_hydrogen_mapping() -> TestResult {
    for (source, offset, feature) in [
        ("c", 0, "atom"),
        ("[nH]", 1, "atom"),
        ("[0c]", 2, "atom"),
        ("[CH0].c", 6, "atom"),
        ("[NH4+].c", 7, "atom"),
        ("C:C", 1, "bond"),
        ("C:1CCC1", 1, "bond"),
        ("C1CCC:1", 5, "bond"),
        ("C:1CCC:1", 1, "bond"),
        ("[0CH4]:[CH4]", 6, "bond"),
    ] {
        let document = parse(source)?;
        let before = document.clone();
        let error = document
            .normalize_explicit_h_fragment()
            .err()
            .ok_or("normalized aromatic notation")?;
        assert_eq!(error.code(), "unsupported_content");
        match error {
            NormalizeError::Unsupported {
                offset: actual,
                feature: actual_feature,
            } => {
                assert_eq!(actual, offset, "{source}");
                assert!(
                    actual_feature.contains(feature),
                    "{source}: {actual_feature}"
                );
            }
            other => return Err(format!("wrong aromatic refusal: {other}").into()),
        }
        assert_eq!(document, before);
        assert_eq!(document.as_bytes(), source.as_bytes());
        assert_eq!(document.as_bytes().as_ptr(), before.as_bytes().as_ptr());
        assert_eq!(document.atoms().as_ptr(), before.atoms().as_ptr());
        assert_eq!(document.bonds().as_ptr(), before.bonds().as_ptr());
    }
    Ok(())
}

#[test]
fn aromatic_notation_uses_the_existing_source_atom_branch_and_chain_budgets() -> TestResult {
    let maximum = format!("[{}c]", "0".repeat(65_533));
    assert_eq!(maximum.len(), 65_536);
    assert_eq!(parse(&maximum)?.atoms()[0].isotope, Some(0));
    assert!(parse(&maximum)?.atoms()[0].aromatic);
    for (source, kind) in [
        (maximum + "\n", Limit::Bytes),
        ("c".repeat(4097), Limit::Atoms),
        (
            format!("{}c{}", "c(".repeat(129), ")".repeat(129)),
            Limit::Branches,
        ),
        (vec!["c"; 1025].join("."), Limit::Chains),
    ] {
        let error = parse(&source)
            .err()
            .ok_or("accepted over-budget notation")?;
        assert!(matches!(error, SmilesError::Limit { kind: actual, .. } if actual == kind));
        assert!(error.offset() <= source.len());
    }
    let chain = parse(&"c".repeat(4096))?;
    assert_eq!(chain.atoms().len(), 4096);
    assert_eq!(chain.bonds().len(), 4095);
    assert!(
        chain
            .bonds()
            .iter()
            .all(|bond| bond.order == BondOrder::Aromatic)
    );
    assert_eq!(
        parse(&format!("{}c{}", "c(".repeat(128), ")".repeat(128)))?
            .atoms()
            .len(),
        129
    );
    assert_eq!(parse(&vec!["c"; 1024].join("."))?.chain_starts(), 1024);
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
            "{}/tests/smiles_oracle.py",
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
        "aromatic notation consumer: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn whole_aromatic_sources_face_the_unchanged_pinned_unsanitized_consumer() -> TestResult {
    let six_cycle = json!([
        [0, 1, "AROMATIC"],
        [0, 5, "AROMATIC"],
        [1, 2, "AROMATIC"],
        [2, 3, "AROMATIC"],
        [3, 4, "AROMATIC"],
        [4, 5, "AROMATIC"]
    ]);
    let five_cycle = json!([
        [0, 1, "AROMATIC"],
        [0, 4, "AROMATIC"],
        [1, 2, "AROMATIC"],
        [2, 3, "AROMATIC"],
        [3, 4, "AROMATIC"]
    ]);
    for (source, atoms, bonds) in [
        (
            "c1ccccc1\r\n",
            vec![(6, 0, 0, 0, false); 6],
            six_cycle.clone(),
        ),
        (
            "c:1ccccc:1",
            vec![(6, 0, 0, 0, false); 6],
            six_cycle.clone(),
        ),
        (
            "[nH]1cccc1",
            vec![
                (7, 0, 0, 1, true),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
            ],
            five_cycle.clone(),
        ),
        (
            "[13cH-]1cc[nH+]cc1",
            vec![
                (6, 13, -1, 1, true),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (7, 0, 1, 1, true),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
            ],
            six_cycle.clone(),
        ),
        (
            "[0c]1cccc1",
            vec![
                (6, 0, 0, 0, true),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
            ],
            five_cycle.clone(),
        ),
        (
            "[se]1cccc1",
            vec![
                (34, 0, 0, 0, true),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
            ],
            five_cycle.clone(),
        ),
        (
            "[as]1ccccc1",
            vec![
                (33, 0, 0, 0, true),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
                (6, 0, 0, 0, false),
            ],
            six_cycle,
        ),
        (
            "c1cccC1",
            vec![(6, 0, 0, 0, false); 5],
            json!([
                [0, 1, "AROMATIC"],
                [0, 4, "SINGLE"],
                [1, 2, "AROMATIC"],
                [2, 3, "AROMATIC"],
                [3, 4, "SINGLE"]
            ]),
        ),
        (
            "C:C",
            vec![(6, 0, 0, 0, false); 2],
            json!([[0, 1, "AROMATIC"]]),
        ),
        (
            "C:1CCC1",
            vec![(6, 0, 0, 0, false); 4],
            json!([
                [0, 1, "SINGLE"],
                [0, 3, "AROMATIC"],
                [1, 2, "SINGLE"],
                [2, 3, "SINGLE"]
            ]),
        ),
        (
            "c1.C1",
            vec![(6, 0, 0, 0, false); 2],
            json!([[0, 1, "SINGLE"]]),
        ),
        (
            "c1.c1",
            vec![(6, 0, 0, 0, false); 2],
            json!([[0, 1, "AROMATIC"]]),
        ),
        (
            "b.n.o.p.s",
            vec![
                (5, 0, 0, 0, false),
                (7, 0, 0, 0, false),
                (8, 0, 0, 0, false),
                (15, 0, 0, 0, false),
                (16, 0, 0, 0, false),
            ],
            json!([]),
        ),
        (
            "[se][Se].[as][As]",
            vec![
                (34, 0, 0, 0, true),
                (34, 0, 0, 0, true),
                (33, 0, 0, 0, true),
                (33, 0, 0, 0, true),
            ],
            json!([[0, 1, "SINGLE"], [2, 3, "SINGLE"]]),
        ),
        (
            "c-c:c=c#c",
            vec![(6, 0, 0, 0, false); 5],
            json!([
                [0, 1, "SINGLE"],
                [1, 2, "AROMATIC"],
                [2, 3, "DOUBLE"],
                [3, 4, "TRIPLE"]
            ]),
        ),
    ] {
        let document = parse(source)?;
        let result = external(document.as_bytes())?;
        assert_eq!(result["rdkit"], "2026.09.2");
        assert_eq!(result["source_ascii"], source);
        assert_eq!(result["source_bytes"], source.len());
        assert_eq!(result["accepted"], true, "{source}: {result}");
        assert_eq!(
            result["parameters"],
            json!({"sanitize":false,"removeHs":false,"parseName":false,"allowCXSMILES":false})
        );
        assert_eq!(result["skip_cleanup_exposed"], false);
        let consumed = result["atoms"].as_array().ok_or("consumer atoms")?;
        assert_eq!(consumed.len(), atoms.len(), "{source}");
        assert_eq!(document.atoms().len(), atoms.len(), "{source}");
        for (actual, (element, isotope, charge, h, no_implicit)) in consumed.iter().zip(atoms) {
            assert_eq!(actual["element"], element, "{source}");
            assert_eq!(actual["isotope"], isotope, "{source}");
            assert_eq!(actual["charge"], charge, "{source}");
            assert_eq!(actual["explicit_h"], h, "{source}");
            assert_eq!(actual["no_implicit"], no_implicit, "{source}");
            assert_eq!(actual["needs_update_cache"], true, "{source}");
        }
        assert_eq!(result["bonds"], bonds, "{source}");
    }
    // These consumer extensions do not broaden the pinned grammar/profile.
    for source in ["[te]1cccc1", "[si]1ccccc1"] {
        assert_eq!(external(source.as_bytes())?["accepted"], true);
        assert!(matches!(
            parse(source),
            Err(SmilesError::Unsupported { .. })
        ));
    }
    assert_eq!(external(b"c:1ccccc-1")?["accepted"], true);
    assert!(matches!(
        parse("c:1ccccc-1"),
        Err(SmilesError::Syntax { .. })
    ));
    assert_eq!(external(b"c1ccc%01")?["accepted"], false);
    assert_eq!(parse("c1ccc%01")?.bonds()[3].order, BondOrder::Aromatic);
    Ok(())
}

#[test]
fn committed_aromatic_fuzz_seeds_preserve_literal_markers_and_external_bonds() -> TestResult {
    use BondOrder::{Aromatic as A, Single as S};
    for (name, expected, flags, expected_edges) in [
        (
            "aromatic-cycle.smi",
            b"c1ccccc1\n".as_slice(),
            vec![true; 6],
            vec![
                ([0, 1], A),
                ([0, 5], A),
                ([1, 2], A),
                ([2, 3], A),
                ([3, 4], A),
                ([4, 5], A),
            ],
        ),
        (
            "explicit-aromatic-bond.smi",
            b"C:C\n".as_slice(),
            vec![false; 2],
            vec![([0, 1], A)],
        ),
        (
            "explicit-ring-orders.smi",
            b"C:1CCC1.c-2ccccc2\n".as_slice(),
            [vec![false; 4], vec![true; 6]].concat(),
            vec![
                ([0, 1], S),
                ([0, 3], A),
                ([1, 2], S),
                ([2, 3], S),
                ([4, 5], A),
                ([4, 9], S),
                ([5, 6], A),
                ([6, 7], A),
                ([7, 8], A),
                ([8, 9], A),
            ],
        ),
    ] {
        let source = std::fs::read(format!(
            "{}/tests/fixtures/smiles/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        assert_eq!(source, expected);
        let document = SmilesDocument::parse(Arc::from(source.as_slice()))?;
        assert_eq!(document.as_bytes(), expected);
        assert_eq!(document.notation_range(), 0..source.len() - 1);
        assert_eq!(document.atoms().len(), flags.len());
        for (atom, flag) in document.atoms().iter().zip(flags) {
            assert_eq!(atom.element.get(), 6);
            assert_eq!(atom.aromatic, flag);
            assert_eq!(atom.hydrogens, Hydrogens::Implicit);
        }
        let mut actual_edges = document
            .bonds()
            .iter()
            .map(|bond| {
                let [left, right] = bond.atoms;
                ([left.min(right), left.max(right)], bond.order)
            })
            .collect::<Vec<_>>();
        actual_edges.sort_unstable_by_key(|(edge, _)| *edge);
        assert_eq!(actual_edges, expected_edges);
        let external = external(&source)?;
        assert_eq!(external["accepted"], true);
        assert_eq!(external["source_ascii"], std::str::from_utf8(expected)?);
        assert_eq!(external["source_bytes"], expected.len());
        let expected_edges = expected_edges
            .into_iter()
            .map(|([left, right], order)| {
                json!([left, right, if order == A { "AROMATIC" } else { "SINGLE" }])
            })
            .collect::<Vec<_>>();
        assert_eq!(external["bonds"], json!(expected_edges));
        assert_eq!(
            document
                .normalize_explicit_h_fragment()
                .err()
                .ok_or("aromatic mapping accepted")?
                .code(),
            "unsupported_content"
        );
    }
    Ok(())
}
