use bondline_io::mol::{MolFile, MolTransform, write_v2000, write_v2000_with_header};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Point2, Radical,
};
use std::{
    io::Write,
    process::{Command, Stdio},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const CH3: &[u8] = include_bytes!("fixtures/mol/rdkit-CH3.mol");
const TWO_ATOMS: &str = include_str!("fixtures/mol/ethane-v2000.mol");

fn transform() -> Result<MolTransform, bondline_io::mol::MolError> {
    MolTransform::new(
        20.0,
        Point2::new(100.0, 200.0).map_err(bondline_io::mol::MolError::Model)?,
    )
}

fn external(bytes: &[u8]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    external_mode(bytes, "describe")
}

fn external_mode(
    bytes: &[u8],
    mode: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mol_oracle.py"))
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "external RDKit: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

#[test]
fn pinned_methyl_radical_retains_bytes_and_decodes_declared_hydrogens_without_valence_inference()
-> TestResult {
    assert_eq!(CH3.len(), 170);
    assert_eq!(
        external(CH3)?["source_sha256"],
        "afd2352697ca20c0d580c4981fe8bd24da36c4e1661e9d2aee501a40364fd42d"
    );
    let parsed = MolFile::parse(CH3)?;
    assert_eq!(parsed.as_bytes(), CH3);
    assert_eq!(parsed.atoms().len(), 1);
    assert!(parsed.bonds().is_empty());
    let raw = parsed.atoms().first().ok_or("atom")?;
    assert_eq!(raw.element.ok_or("element")?.get(), 6);
    assert_eq!(raw.declared_valence, Some(3));
    assert_eq!(raw.radical, Radical::Doublet);
    let fragment = parsed.to_fragment(transform()?)?;
    let atom = fragment.atoms().first().ok_or("model atom")?;
    assert_eq!(atom.explicit_hydrogens, Some(3));
    assert_eq!(atom.radical, Radical::Doublet);
    let point = atom.position.ok_or("position")?;
    assert!((point.x_pt() - 75.25).abs() < 1e-10);
    assert!((point.y_pt() - 156.982).abs() < 1e-10);
    let encoded = write_v2000(&fragment, "methyl radical", transform()?)?;
    let independent = external(&encoded)?;
    assert_eq!(independent["smiles"], "[CH3]");
    assert_eq!(independent["formula"], "CH3");
    assert_eq!(independent["atoms"][0]["hydrogens"], 3);
    assert_eq!(independent["atoms"][0]["radical_electrons"], 1);
    Ok(())
}

#[test]
fn literal_fixed_columns_and_native_cc_export_agree_with_external_rdkit() -> TestResult {
    let parsed = MolFile::parse(TWO_ATOMS.as_bytes())?;
    let fragment = parsed.to_fragment(transform()?)?;
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        vec![Some(3), Some(3)]
    );
    let encoded = write_v2000(&fragment, "ethane", transform()?)?;
    let text = std::str::from_utf8(&encoded)?;
    let counts = text.lines().nth(3).ok_or("counts")?;
    assert_eq!(counts.get(..6), Some("  2  1"));
    assert_eq!(counts.get(33..39), Some(" V2000"));
    for line in text.lines().skip(4).take(2) {
        assert_eq!(line.get(48..51), Some("  4"));
    }
    let independent = external(&encoded)?;
    assert_eq!(independent["smiles"], "CC");
    assert_eq!(independent["formula"], "C2H6");
    assert_eq!(
        independent["atoms"][0]["position"],
        serde_json::json!([0.0, 0.0, 0.0])
    );
    assert_eq!(
        independent["atoms"][1]["position"],
        serde_json::json!([1.5, 0.0, 0.0])
    );
    Ok(())
}

#[test]
fn zero_valence_encodes_explicit_zero_hydrogens_and_differs_from_unspecified() -> TestResult {
    let id = AtomId::new(9).ok_or("id")?;
    let mut atom = Atom::new(
        id,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(100.0, 200.0)?),
    );
    atom.explicit_hydrogens = Some(0);
    let fragment = Fragment::new(FragmentId::new(5).ok_or("fragment")?, vec![atom], vec![])?;
    let encoded = write_v2000(&fragment, "zero authored H", transform()?)?;
    assert_eq!(
        std::str::from_utf8(&encoded)?
            .lines()
            .nth(4)
            .and_then(|line| line.get(48..51)),
        Some(" 15")
    );
    let recovered = MolFile::parse(&encoded)?.to_fragment(transform()?)?;
    assert_eq!(
        recovered.atoms().first().ok_or("atom")?.explicit_hydrogens,
        Some(0)
    );
    assert_eq!(
        external_mode(&encoded, "raw")?["atoms"][0]["hydrogens"],
        0,
        "external reader must retain an explicitly zero H count"
    );
    let unmarked = TWO_ATOMS.replacen("  4  0  0  0  0  0  0", "  0  0  0  0  0  0  0", 2);
    let unmarked = MolFile::parse(unmarked.as_bytes())?.to_fragment(transform()?)?;
    assert!(
        unmarked
            .atoms()
            .iter()
            .all(|a| a.explicit_hydrogens.is_none())
    );
    Ok(())
}

#[test]
fn hostile_or_unmodeled_content_is_not_silently_projected_or_written() -> TestResult {
    for (source, expected) in [
        (TWO_ATOMS.replace("V2000", "V3000"), "unsupported_content"),
        (
            TWO_ATOMS.replace("  1  2  1  0  0  0  0", "  1  9  1  0  0  0  0"),
            "invalid_mol",
        ),
        (TWO_ATOMS.replace("M  END\n", ""), "invalid_mol"),
        (TWO_ATOMS.replace("    1.5000", "       NaN"), "invalid_mol"),
    ] {
        assert_eq!(
            MolFile::parse(source.as_bytes())
                .err()
                .ok_or("invalid input accepted")?
                .code(),
            expected
        );
    }
    for source in [
        TWO_ATOMS.replace("M  END", "M  XYZ  1   1   7\nM  END"),
        TWO_ATOMS.replace("  1  2  1  0  0  0  0", "  1  2  1  1  0  0  0"),
        TWO_ATOMS.replacen("    0.0000 C", "    1.0000 C", 1),
        TWO_ATOMS.replacen(" C   0  0  0  0", " C   0  0  0  2", 1),
    ] {
        let parsed = MolFile::parse(source.as_bytes())?;
        assert_eq!(parsed.as_bytes(), source.as_bytes());
        assert_eq!(
            parsed
                .to_fragment(transform()?)
                .err()
                .ok_or("unsupported feature dropped")?
                .code(),
            "unsupported_content"
        );
    }
    assert!(MolTransform::new(0.0, Point2::new(0.0, 0.0)?).is_err());
    Ok(())
}

#[test]
fn edited_graph_export_preserves_all_three_retained_header_lines() -> TestResult {
    let parsed = MolFile::parse(TWO_ATOMS.as_bytes())?;
    let fragment = parsed.to_fragment(transform()?)?;
    let output = write_v2000_with_header(&fragment, parsed.header(), transform()?)?;
    let output = std::str::from_utf8(&output)?;
    assert_eq!(
        output.lines().take(3).collect::<Vec<_>>(),
        vec![
            "Original V2000 vector",
            "  Bondline test",
            "Original independent fixed columns"
        ]
    );
    Ok(())
}

#[test]
fn property_records_override_legacy_fields_globally_and_keep_charge_radical_isotope_separate()
-> TestResult {
    let source = mixed_property_source();
    let parsed = MolFile::parse(source.as_bytes())?;
    let fragment = parsed.to_fragment(transform()?)?;
    let first = fragment.atoms().first().ok_or("first")?;
    let second = fragment.atoms().get(1).ok_or("second")?;
    assert_eq!(
        (first.charge, first.radical, first.isotope),
        (0, Radical::Triplet, None)
    );
    assert_eq!(
        (
            second.charge,
            second.radical,
            second.isotope.map(|i| i.get())
        ),
        (-1, Radical::None, Some(13))
    );
    assert_eq!(
        parsed.atoms().first().ok_or("raw")?.mass_difference,
        1,
        "superseded syntax still retained"
    );
    assert_eq!(parsed.as_bytes(), source.as_bytes());
    let only_rad = TWO_ATOMS
        .replacen(" C   0  0", " C   0  3", 1)
        .replace("M  END", "M  RAD  1   2   2\nM  END");
    let mapped = MolFile::parse(only_rad.as_bytes())?.to_fragment(transform()?)?;
    assert_eq!(mapped.atoms().first().ok_or("first")?.charge, 0);
    assert_eq!(
        mapped.atoms().get(1).ok_or("second")?.radical,
        Radical::Doublet
    );
    let independent = external_mode(only_rad.as_bytes(), "raw")?;
    assert_eq!(
        independent["atoms"][0]["charge"], 0,
        "an M RAD line globally supersedes an unlisted atom's legacy charge"
    );
    let unverified_mass = TWO_ATOMS.replacen(" C   0  0", " C   1  0", 1);
    assert_eq!(
        MolFile::parse(unverified_mass.as_bytes())?
            .to_fragment(transform()?)
            .err()
            .ok_or("guessed isotope mass")?
            .code(),
        "unsupported_content"
    );
    Ok(())
}

fn mixed_property_source() -> String {
    TWO_ATOMS
        .replacen(" C   0  0", " C   1  3", 1)
        .replacen(" C   0  0", " C   0  4", 1)
        .replace(
            "M  END",
            "M  CHG  1   2  -1\nM  RAD  1   1   3\nM  ISO  1   2  13\nM  END",
        )
}

#[test]
fn external_reader_isotope_quirk_is_explicit_and_canonical_output_has_normative_isotopes()
-> TestResult {
    let source = mixed_property_source();
    let raw = MolFile::parse(source.as_bytes())?;
    let model = raw.to_fragment(transform()?)?;
    assert!(
        model.atoms().first().ok_or("first")?.isotope.is_none(),
        "published M ISO globally supersedes atom-block isotope fields"
    );
    assert_eq!(
        raw.as_bytes(),
        source.as_bytes(),
        "source passthrough must not normalize away the consumer discrepancy"
    );
    let observed = external_mode(source.as_bytes(), "raw")?;
    assert_eq!(
        (
            observed["atoms"][0]["charge"].as_i64(),
            observed["atoms"][0]["radical_electrons"].as_u64(),
            observed["atoms"][0]["isotope"].as_u64()
        ),
        (Some(0), Some(2), Some(13)),
        "pinned RDKit retains the unlisted legacy mass difference: this input is not a claimed isotope agreement"
    );
    assert_eq!(
        (
            observed["atoms"][1]["charge"].as_i64(),
            observed["atoms"][1]["radical_electrons"].as_u64(),
            observed["atoms"][1]["isotope"].as_u64()
        ),
        (Some(-1), Some(0), Some(13))
    );
    let encoded = write_v2000_with_header(&model, raw.header(), transform()?)?;
    let reread = external_mode(&encoded, "raw")?;
    assert_eq!(
        reread["atoms"][0]["isotope"], 0,
        "canonical writer must express the normative natural-abundance atom unambiguously"
    );
    assert_eq!(reread["atoms"][1]["isotope"], 13);
    assert_eq!(reread["atoms"][0]["charge"], 0);
    assert_eq!(reread["atoms"][0]["radical_electrons"], 2);
    Ok(())
}

#[test]
fn fixed_width_properties_bounds_and_duplicate_entries_have_decisive_failures() -> TestResult {
    for property in [
        "M  CHG  1   3   1",
        "M  CHG  1   1  16",
        "M  RAD  1   1   4",
        "M  ISO  1   1   0",
        "M  CHG  2   1   1",
        "M  CHG  0",
        "M  CHG  2   1   1   1   2",
    ] {
        let source = TWO_ATOMS.replace("M  END", &format!("{property}\nM  END"));
        assert_eq!(
            MolFile::parse(source.as_bytes())
                .err()
                .ok_or("invalid property accepted")?
                .code(),
            "invalid_mol",
            "{property}"
        );
    }
    let too_long = format!("{}\n", "x".repeat(4097));
    assert_eq!(
        MolFile::parse(too_long.as_bytes())
            .err()
            .ok_or("unbounded line")?
            .code(),
        "resource_limit"
    );
    let too_many_lines = format!("{TWO_ATOMS}{}", "\n".repeat(100_001));
    assert_eq!(
        MolFile::parse(too_many_lines.as_bytes())
            .err()
            .ok_or("unbounded lines")?
            .code(),
        "resource_limit"
    );
    let too_large = vec![b' '; 64 * 1024 * 1024 + 1];
    assert_eq!(
        MolFile::parse(&too_large)
            .err()
            .ok_or("unbounded input")?
            .code(),
        "resource_limit"
    );
    assert_eq!(
        MolFile::parse(format!("{TWO_ATOMS}unexpected\n").as_bytes())
            .err()
            .ok_or("ignored trailing data")?
            .code(),
        "invalid_mol"
    );
    let crlf = TWO_ATOMS.replace('\n', "\r\n");
    assert_eq!(MolFile::parse(crlf.as_bytes())?.as_bytes(), crlf.as_bytes());
    assert!(
        MolFile::parse(TWO_ATOMS.replace("    1.5000", "          ").as_bytes()).is_ok(),
        "blank numerical fields mean zero"
    );
    Ok(())
}

#[test]
fn declared_hydrogens_use_integer_bond_order_sum_and_refuse_ambiguous_aromatic_valence()
-> TestResult {
    for (order, hydrogens) in [(1, 3), (2, 2), (3, 1)] {
        let source = TWO_ATOMS.replace(
            "  1  2  1  0  0  0  0",
            &format!("  1  2{order:>3}  0  0  0  0"),
        );
        let fragment = MolFile::parse(source.as_bytes())?.to_fragment(transform()?)?;
        assert!(
            fragment
                .atoms()
                .iter()
                .all(|atom| atom.explicit_hydrogens == Some(hydrogens))
        );
    }
    let aromatic = TWO_ATOMS.replace("  1  2  1  0  0  0  0", "  1  2  4  0  0  0  0");
    assert_eq!(
        MolFile::parse(aromatic.as_bytes())?
            .to_fragment(transform()?)
            .err()
            .ok_or("aromatic H guessed")?
            .code(),
        "unsupported_content"
    );
    let mut atoms = Vec::new();
    for id in [1, 2] {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("id")?,
            Element::new(6).ok_or("C")?,
            Some(Point2::new(100.0 + 30.0 * f64::from(id - 1), 200.0)?),
        );
        atom.explicit_hydrogens = Some(3);
        atoms.push(atom);
    }
    let bond = Bond {
        id: BondId::new(1).ok_or("bond")?,
        begin: AtomId::new(1).ok_or("begin")?,
        end: AtomId::new(2).ok_or("end")?,
        order: BondOrder::Aromatic,
    };
    let fragment = Fragment::new(FragmentId::new(1).ok_or("fragment")?, atoms, vec![bond])?;
    assert_eq!(
        write_v2000(&fragment, "must refuse", transform()?)
            .err()
            .ok_or("aromatic authored H lost")?
            .code(),
        "unsupported_content"
    );
    Ok(())
}

#[test]
fn native_non_decimal_coordinates_use_independently_bounded_v2000_quantization() -> TestResult {
    let frame = MolTransform::new(24.0, Point2::new(0.0, 0.0)?)?;
    let atom = Atom::new(
        AtomId::new(1).ok_or("id")?,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(100.0, -25.0)?),
    );
    let fragment = Fragment::new(FragmentId::new(1).ok_or("fragment")?, vec![atom], vec![])?;
    let encoded = write_v2000(&fragment, "quantization", frame)?;
    let record = std::str::from_utf8(&encoded)?
        .lines()
        .nth(4)
        .ok_or("atom row")?;
    assert_eq!(record.get(..20), Some("    4.1667    1.0417"));
    let independent = external(&encoded)?;
    assert_eq!(
        independent["atoms"][0]["position"],
        serde_json::json!([4.1667, 1.0417, 0.0])
    );
    let decimal = external_mode(&encoded, "quantization")?;
    assert_eq!(decimal["expected"], serde_json::json!(["4.1667", "1.0417"]));
    for (actual, original) in [(4.1667_f64, 100.0 / 24.0), (1.0417, 25.0 / 24.0)] {
        assert!((actual - original).abs() <= 0.00005 + 8.0 * f64::EPSILON * original.abs());
    }
    Ok(())
}

#[test]
fn external_rdkit_atom_symbols_and_authored_metadata_agree_with_the_writer() -> TestResult {
    let mut atoms = Vec::new();
    for number in 1_u8..=118 {
        let mut atom = Atom::new(
            AtomId::new(u32::from(number)).ok_or("id")?,
            Element::new(number).ok_or("element")?,
            Some(Point2::new(100.0 + 20.0 * f64::from(number), 200.0)?),
        );
        if number == 6 {
            atom.isotope = std::num::NonZeroU16::new(13);
            atom.charge = -1;
            atom.radical = Radical::Doublet;
            atom.explicit_hydrogens = Some(2);
        }
        atoms.push(atom);
    }
    let fragment = Fragment::new(FragmentId::new(1).ok_or("fragment")?, atoms, vec![])?;
    let encoded = write_v2000(&fragment, "element facts", transform()?)?;
    let independent = external_mode(&encoded, "raw")?;
    for (index, atom) in independent["atoms"]
        .as_array()
        .ok_or("external atoms")?
        .iter()
        .enumerate()
    {
        assert_eq!(atom["element"].as_u64(), Some(u64::try_from(index + 1)?));
    }
    let carbon = independent["atoms"].get(5).ok_or("carbon")?;
    assert_eq!(
        (
            carbon["charge"].as_i64(),
            carbon["isotope"].as_u64(),
            carbon["radical_electrons"].as_u64()
        ),
        (Some(-1), Some(13), Some(1))
    );
    let parsed = MolFile::parse(&encoded)?.to_fragment(transform()?)?;
    let carbon = parsed.atoms().get(5).ok_or("mapped carbon")?;
    assert_eq!(
        (
            carbon.charge,
            carbon.isotope.map(|i| i.get()),
            carbon.radical,
            carbon.explicit_hydrogens
        ),
        (-1, Some(13), Radical::Doublet, Some(2))
    );
    Ok(())
}

#[test]
fn unsupported_raw_features_are_addressable_before_model_conversion() -> TestResult {
    for source in [
        TWO_ATOMS.replacen(" C   0  0", " *   0  0", 1),
        TWO_ATOMS.replacen("    0.0000 C", "    1.0000 C", 1),
        TWO_ATOMS.replacen(" C   0  0", " C   1  0", 1),
    ] {
        let raw = MolFile::parse(source.as_bytes())?;
        assert!(
            !raw.unsupported().is_empty(),
            "unmodeled source facts need a diagnostic, not a misleading complete mapping"
        );
        assert_eq!(raw.as_bytes(), source.as_bytes());
    }
    Ok(())
}

#[test]
fn a_separately_run_rdkit_writer_supplies_an_independent_import_vector() -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let source = Command::new(python)
        .arg("-B")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mol_oracle.py"))
        .arg("emit")
        .output()?;
    assert!(
        source.status.success(),
        "{}",
        String::from_utf8_lossy(&source.stderr)
    );
    let raw = MolFile::parse(&source.stdout)?;
    assert!(raw.unsupported().is_empty(), "{:?}", raw.unsupported());
    let fragment = raw.to_fragment(transform()?)?;
    assert_eq!(
        fragment
            .atoms()
            .first()
            .ok_or("C")?
            .isotope
            .map(|i| i.get()),
        Some(13)
    );
    assert_eq!(fragment.atoms().get(1).ok_or("O")?.charge, -1);
    let model_c = fragment
        .atoms()
        .first()
        .ok_or("C")?
        .position
        .ok_or("position")?;
    assert_eq!((model_c.x_pt(), model_c.y_pt()), (75.0, 150.0));
    let reexported = write_v2000_with_header(&fragment, raw.header(), transform()?)?;
    let read = external(&reexported)?;
    assert_eq!(read["smiles"], "[13CH3][O-]");
    assert_eq!(
        read["atoms"][0]["position"],
        serde_json::json!([-1.25, 2.5, 0.0])
    );
    Ok(())
}

#[test]
fn writer_refuses_unrepresentable_authored_fields_without_mutating_the_graph() -> TestResult {
    let element = Element::new(6).ok_or("C")?;
    let id = AtomId::new(1).ok_or("id")?;
    let basic = Atom::new(id, element, Some(Point2::new(100.0, 200.0)?));
    for case in ["no position", "charge", "isotope", "valence", "coordinate"] {
        let mut atom = basic.clone();
        match case {
            "no position" => atom.position = None,
            "charge" => atom.charge = 16,
            "isotope" => atom.isotope = std::num::NonZeroU16::new(1000),
            "valence" => atom.explicit_hydrogens = Some(15),
            "coordinate" => atom.position = Some(Point2::new(-100_000.0, 200.0)?),
            _ => {}
        }
        let fragment = Fragment::new(FragmentId::new(1).ok_or("fragment")?, vec![atom], vec![])?;
        let before = fragment.clone();
        let frame = if case == "coordinate" {
            MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?
        } else {
            transform()?
        };
        assert_eq!(
            write_v2000(&fragment, "refuse", frame)
                .err()
                .ok_or("unsupported field silently dropped")?
                .code(),
            "unsupported_content",
            "{case}"
        );
        assert_eq!(fragment, before);
    }
    let mut atoms = Vec::new();
    for index in 1..=1000 {
        atoms.push(Atom::new(
            AtomId::new(index).ok_or("id")?,
            element,
            Some(Point2::new(0.0, 0.0)?),
        ));
    }
    let fragment = Fragment::new(FragmentId::new(1).ok_or("fragment")?, atoms, vec![])?;
    assert_eq!(
        write_v2000(&fragment, "too many", transform()?)
            .err()
            .ok_or("unbounded V2000 count")?
            .code(),
        "resource_limit"
    );
    Ok(())
}
