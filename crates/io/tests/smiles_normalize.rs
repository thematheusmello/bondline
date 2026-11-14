use bondline_io::smiles::{Hydrogens, NormalizeError, SmilesDocument, SmilesError, write_fragment};
use bondline_model::{BondOrder, Radical};
use serde_json::{Value, json};
use std::{
    io::Write as _,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn parse(source: &str) -> Result<SmilesDocument, Box<dyn std::error::Error>> {
    Ok(SmilesDocument::parse(Arc::from(source.as_bytes()))?)
}

#[test]
fn normalization_is_opt_in_while_bare_bracket_and_crlf_source_distinctions_stay_exact() -> TestResult
{
    let bare = parse("CC\r\n")?;
    let bracket = parse("[CH3][CH3]")?;
    let before_bare = bare.clone();
    let before_bracket = bracket.clone();
    assert_ne!(bare, bracket);
    assert_eq!(bare.atoms()[0].hydrogens, Hydrogens::Implicit);
    assert_eq!(bracket.atoms()[0].hydrogens, Hydrogens::Bracket(3));
    let normalized = bare.normalize_explicit_h_fragment()?;
    assert_eq!(normalized, bracket.normalize_explicit_h_fragment()?);
    assert_eq!(write_fragment(&normalized)?, b"[CH3][CH3]");
    for (current, before) in [(&bare, &before_bare), (&bracket, &before_bracket)] {
        assert_eq!(current, before);
        assert_eq!(current.as_bytes().as_ptr(), before.as_bytes().as_ptr());
        assert_eq!(current.atoms().as_ptr(), before.atoms().as_ptr());
        assert_eq!(current.bonds().as_ptr(), before.bonds().as_ptr());
    }
    assert_eq!(bare.as_bytes(), b"CC\r\n");
    assert_eq!(bracket.as_bytes(), b"[CH3][CH3]");
    // The existing authored model contract stays unchanged outside this opt-in conversion.
    let unspecified = bondline_model::Atom::new(
        bondline_model::AtomId::new(99).ok_or("ID")?,
        bondline_model::Element::new(6).ok_or("C")?,
        None,
    );
    assert_eq!(unspecified.explicit_hydrogens, None);
    assert_eq!(
        serde_json::to_value(&unspecified)?["explicit_hydrogens"],
        Value::Null
    );
    Ok(())
}

#[test]
fn brackets_isotopes_physical_h_and_cross_dot_graphs_keep_literal_meaning() -> TestResult {
    for (source, counts, elements, output) in [
        ("[CH4]", vec![4], vec![6], "[CH4]"),
        ("[OH2]", vec![2], vec![8], "[OH2]"),
        ("[H][H]", vec![0, 0], vec![1, 1], "[H][H]"),
        ("C[H]", vec![3, 0], vec![6, 1], "[CH3][H]"),
        (
            "[CH2]([H])[2H]",
            vec![2, 0, 0],
            vec![6, 1, 1],
            "[CH2]([H])[2H]",
        ),
        ("[13CH3]C", vec![3, 3], vec![6, 6], "[13CH3][CH3]"),
        ("C1.C1", vec![3, 3], vec![6, 6], "[CH3][CH3]"),
        ("C.O", vec![4, 2], vec![6, 8], "[CH4].[OH2]"),
        (
            "C(F)(Cl)(Br)I",
            vec![0, 0, 0, 0, 0],
            vec![6, 9, 17, 35, 53],
            "[CH0]([FH0])([ClH0])([BrH0])[IH0]",
        ),
    ] {
        let document = parse(source)?;
        let before = document.clone();
        let model = document.normalize_explicit_h_fragment()?;
        assert_eq!(
            model
                .atoms()
                .iter()
                .map(|a| a.element.get())
                .collect::<Vec<_>>(),
            elements,
            "{source}"
        );
        assert_eq!(
            model
                .atoms()
                .iter()
                .map(|a| a.explicit_hydrogens)
                .collect::<Vec<_>>(),
            counts.into_iter().map(Some).collect::<Vec<_>>(),
            "{source}"
        );
        assert_eq!(write_fragment(&model)?, output.as_bytes(), "{source}");
        assert_eq!(document, before);
    }
    let isotope = parse("[13CH3]C")?.normalize_explicit_h_fragment()?;
    assert_eq!(isotope.atoms()[0].isotope.map(|m| m.get()), Some(13));
    let physical = parse("[CH2]([H])[2H]")?.normalize_explicit_h_fragment()?;
    assert_eq!(physical.atoms()[2].isotope.map(|m| m.get()), Some(2));
    assert_eq!(
        physical
            .bonds()
            .iter()
            .map(|b| [b.begin.get(), b.end.get()])
            .collect::<Vec<_>>(),
        [[1, 2], [1, 3]]
    );
    let cross = parse("C1.C1")?;
    assert_eq!(cross.chain_starts(), 2);
    assert_eq!(cross.normalize_explicit_h_fragment()?.bonds().len(), 1);
    Ok(())
}

#[test]
fn unsupported_and_radical_ambiguous_contexts_refuse_without_changing_the_owner() -> TestResult {
    for source in [
        "[C]",
        "[CH0]",
        "[CH3]",
        "[CH2]",
        "[O]",
        "[OH]",
        "[H]",
        "CC[O]",
        "[CH4]C",
        "[CH9]",
        "C(C)(C)(C)(C)C",
        "C#C#C",
        "N(C)(C)(C)C",
        "N(C)(C)(C)(C)C",
        "[NH4+]",
        "[Cl-]",
        "[0CH4]",
        "B",
        "P",
        "S",
        "[Fe]",
        "[65535U]",
    ] {
        let document = parse(source)?;
        let before = document.clone();
        let error = document
            .normalize_explicit_h_fragment()
            .err()
            .ok_or_else(|| format!("accepted {source}"))?;
        assert_eq!(error.code(), "unsupported_content", "{source}: {error}");
        assert_eq!(document, before);
        assert_eq!(document.as_bytes(), source.as_bytes());
        assert_eq!(document.as_bytes().as_ptr(), before.as_bytes().as_ptr());
        assert_eq!(document.atoms().as_ptr(), before.atoms().as_ptr());
        assert_eq!(document.bonds().as_ptr(), before.bonds().as_ptr());
    }
    assert!(matches!(
        parse("[C]")?.normalize_explicit_h_fragment(),
        Err(NormalizeError::Valence {
            offset: 0,
            expected: 4,
            bond_order_sum: 0,
            hydrogens: Hydrogens::Bracket(0)
        })
    ));
    assert!(matches!(
        parse("C#C#C")?.normalize_explicit_h_fragment(),
        Err(NormalizeError::Valence {
            offset: 2,
            expected: 4,
            bond_order_sum: 6,
            hydrogens: Hydrogens::Implicit
        })
    ));
    assert!(matches!(
        parse("[0CH4]")?.normalize_explicit_h_fragment(),
        Err(NormalizeError::Unsupported { offset: 1, .. })
    ));
    assert!(matches!(
        parse("[NH4+]")?.normalize_explicit_h_fragment(),
        Err(NormalizeError::Unsupported { offset: 4, .. })
    ));
    // Aromatic syntax is retained, while its model normalization still refuses.
    assert!(matches!(
        parse("c1ccccc1")?.normalize_explicit_h_fragment(),
        Err(NormalizeError::Unsupported { offset: 0, .. })
    ));
    for source in ["[C:4]", "[C@H](F)(Cl)Br", "*", "C~C"] {
        assert!(
            matches!(
                SmilesDocument::parse(Arc::from(source.as_bytes())),
                Err(SmilesError::Unsupported { .. })
            ),
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn parser_bounds_remain_authoritative_and_normalization_uses_no_graph_recursion() -> TestResult {
    let source = "C".repeat(4096);
    let document = parse(&source)?;
    let model = document.normalize_explicit_h_fragment()?;
    assert_eq!(model.atoms().len(), 4096);
    assert_eq!(model.bonds().len(), 4095);
    assert_eq!(model.atoms()[0].explicit_hydrogens, Some(3));
    assert_eq!(model.atoms()[2048].explicit_hydrogens, Some(2));
    assert_eq!(model.atoms()[4095].explicit_hydrogens, Some(3));
    assert_eq!(model.atoms()[4095].id.get(), 4096);
    assert_eq!(model.bonds()[4094].id.get(), 4095);
    assert_eq!(document.as_bytes(), source.as_bytes());
    assert!(matches!(
        SmilesDocument::parse(Arc::from("C".repeat(4097).into_bytes())),
        Err(SmilesError::Limit {
            kind: bondline_io::smiles::Limit::Atoms,
            ..
        })
    ));
    let nested = format!("{}C{}", "C(".repeat(128), ")".repeat(128));
    assert_eq!(
        parse(&nested)?
            .normalize_explicit_h_fragment()?
            .atoms()
            .len(),
        129
    );
    let bytes = format!("[{}1CH4]", "0".repeat(65_530));
    assert_eq!(bytes.len(), 65_536);
    let maximum = parse(&bytes)?;
    let normalized = maximum.normalize_explicit_h_fragment()?;
    assert_eq!(normalized.atoms()[0].isotope.map(|m| m.get()), Some(1));
    assert_eq!(normalized.atoms()[0].explicit_hydrogens, Some(4));
    assert_eq!(maximum.as_bytes(), bytes.as_bytes());
    assert_eq!(write_fragment(&normalized)?, b"[1CH4]");
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
        "normalized-output consumer: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn actual_normalized_generated_output_faces_literal_and_separate_chemical_consumers() -> TestResult
{
    for (source, counts, generated, smiles, formula) in [
        ("C", vec![4], "[CH4]", "C", "CH4"),
        ("CC", vec![3, 3], "[CH3][CH3]", "CC", "C2H6"),
        ("CCC", vec![3, 2, 3], "[CH3][CH2][CH3]", "CCC", "C3H8"),
        ("C=C", vec![2, 2], "[CH2]=[CH2]", "C=C", "C2H4"),
        ("C#N", vec![1, 0], "[CH1]#[NH0]", "C#N", "CHN"),
        ("O", vec![2], "[OH2]", "O", "H2O"),
        ("N", vec![3], "[NH3]", "N", "H3N"),
    ] {
        let document = parse(source)?;
        let model = document.normalize_explicit_h_fragment()?;
        let output = write_fragment(&model)?;
        assert_eq!(output, generated.as_bytes());
        let result = external(&output)?;
        assert_eq!(result["notation"]["source_ascii"], generated);
        assert_eq!(
            result["notation"]["atoms"].as_array().ok_or("atoms")?.len(),
            counts.len()
        );
        for (atom, count) in result["notation"]["atoms"]
            .as_array()
            .ok_or("atoms")?
            .iter()
            .zip(counts)
        {
            assert_eq!(atom["explicit_h"], count);
            assert_eq!(atom["no_implicit"], true);
        }
        assert_eq!(result["meaning"]["smiles"], smiles);
        assert_eq!(result["meaning"]["formula"], formula);
        assert_eq!(
            result["meaning"]["radicals"],
            json!(vec![0; model.atoms().len()])
        );
        assert_eq!(result["meaning"]["conformers"], 0);
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    let model = parse("[13CH2]([H])[2H]")?.normalize_explicit_h_fragment()?;
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

#[test]
fn ordinary_bare_atoms_normalize_to_literal_counts_dense_ids_and_no_coordinates() -> TestResult {
    use BondOrder::{Double as D, Single as S, Triple as T};
    for (source, elements, counts, edges) in [
        ("C", vec![6], vec![4], vec![]),
        ("CC", vec![6, 6], vec![3, 3], vec![([1, 2], S)]),
        (
            "CCC",
            vec![6, 6, 6],
            vec![3, 2, 3],
            vec![([1, 2], S), ([2, 3], S)],
        ),
        ("C=C", vec![6, 6], vec![2, 2], vec![([1, 2], D)]),
        ("C#N", vec![6, 7], vec![1, 0], vec![([1, 2], T)]),
        ("O", vec![8], vec![2], vec![]),
        ("N", vec![7], vec![3], vec![]),
    ] {
        let document = parse(source)?;
        let before = document.clone();
        let normalized = document.normalize_explicit_h_fragment()?;
        assert_eq!(normalized.id().get(), 1);
        assert_eq!(
            normalized
                .atoms()
                .iter()
                .map(|a| a.element.get())
                .collect::<Vec<_>>(),
            elements,
            "{source}"
        );
        assert_eq!(
            normalized
                .atoms()
                .iter()
                .map(|a| a.explicit_hydrogens)
                .collect::<Vec<_>>(),
            counts.into_iter().map(Some).collect::<Vec<_>>(),
            "{source}"
        );
        for (index, atom) in normalized.atoms().iter().enumerate() {
            assert_eq!(atom.id.get(), u32::try_from(index + 1)?);
            assert_eq!(atom.position, None);
            assert_eq!(atom.isotope, None);
            assert_eq!(atom.charge, 0);
            assert_eq!(atom.radical, Radical::None);
        }
        assert_eq!(
            normalized
                .bonds()
                .iter()
                .map(|b| ([b.begin.get(), b.end.get()], b.order))
                .collect::<Vec<_>>(),
            edges,
            "{source}"
        );
        assert_eq!(document, before);
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    Ok(())
}
