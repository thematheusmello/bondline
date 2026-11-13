use bondline_io::mol::{
    MolTransform,
    v3000::{MolFile, Value, write_v3000, write_v3000_with_header},
};
use bondline_model::{Atom, AtomId, BondOrder, Element, Fragment, FragmentId, Point2, Radical};
use serde_json::Value as Json;
use std::{
    io::Write,
    num::NonZeroU16,
    process::{Command, Stdio},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");

fn frame() -> Result<MolTransform, Box<dyn std::error::Error>> {
    Ok(MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?)
}
fn single(line: &str) -> Vec<u8> {
    format!("Original V3000 vector\n  Bondline          2D\nOriginal MIT OR Apache-2.0\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS 1 0 0 0 0\nM  V30 BEGIN ATOM\nM  V30 {line}\nM  V30 END ATOM\nM  V30 END CTAB\nM  END\n").into_bytes()
}
fn external_raw(mode: &str, input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/v3000_oracle.py"
        ))
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle input")?.write_all(input)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "external RDKit: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(result.stdout)
}
fn external(input: &[u8]) -> Result<Json, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&external_raw("describe", input)?)?)
}
fn identity() -> Result<MolTransform, Box<dyn std::error::Error>> {
    Ok(MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?)
}

#[test]
fn published_source_ids_and_valence_project_without_dense_index_assumptions() -> TestResult {
    let raw = MolFile::parse(SOURCE)?;
    assert_eq!(raw.as_bytes(), SOURCE);
    assert_eq!(raw.as_bytes().as_ptr(), SOURCE.as_ptr());
    assert_eq!(
        raw.header(),
        [
            "Bondline original V3000 probe",
            "  Bondline          2D",
            "Original specification-derived vector"
        ]
    );
    assert!(raw.unsupported().is_empty());
    assert_eq!(
        raw.atoms()
            .iter()
            .map(|atom| atom.source_index)
            .collect::<Vec<_>>(),
        [10, 40]
    );
    let first_source = raw.atoms().first().ok_or("first source atom")?;
    assert_eq!(first_source.line, 8);
    assert_eq!(
        SOURCE.get(first_source.range.clone()),
        Some(b"M  V30 10 C -1.25 0.5 0 0 VAL=4\n".as_slice())
    );
    let bond = raw.bonds().first().ok_or("source bond")?;
    assert_eq!(
        (bond.source_index, bond.begin, bond.end, bond.order),
        (7, 10, 40, 1)
    );
    let graph = raw.to_fragment(frame()?)?;
    assert_eq!(
        graph
            .atoms()
            .iter()
            .map(|atom| atom.id.get())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        graph
            .atoms()
            .iter()
            .map(|atom| atom.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(3), Some(3)]
    );
    let positions = graph
        .atoms()
        .iter()
        .map(|atom| atom.position.ok_or("position"))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        positions,
        [Point2::new(42.0, 60.0)?, Point2::new(78.0, 60.0)?]
    );
    assert_eq!(graph.bonds().first().ok_or("bond")?.begin.get(), 1);
    assert_eq!(graph.bonds().first().ok_or("bond")?.end.get(), 2);
    let rewritten = write_v3000_with_header(&graph, raw.header(), frame()?)?;
    assert_eq!(MolFile::parse(&rewritten)?.to_fragment(frame()?)?, graph);
    Ok(())
}

#[test]
fn continuation_quoted_strings_and_counted_lists_retain_original_source_and_refuse_lossy_projection()
-> TestResult {
    let bytes = single("10 C 0 0 0 0 MA-\nM  V30 SS=13 RAD=2 VAL=3");
    let raw = MolFile::parse(&bytes)?;
    let atom = raw
        .to_fragment(frame()?)?
        .atoms()
        .first()
        .cloned()
        .ok_or("atom")?;
    assert_eq!(atom.isotope.ok_or("isotope")?.get(), 13);
    assert_eq!(atom.radical, Radical::Doublet);
    assert_eq!(atom.explicit_hydrogens, Some(3));
    assert_eq!(raw.as_bytes(), bytes);
    let unknown = single("10 C 0 0 0 0 VAL=-1 XNOTE=\"ab-\nM  V30 c\"\"d\"");
    let raw = MolFile::parse(&unknown)?;
    assert_eq!(raw.as_bytes(), unknown);
    assert!(raw.to_fragment(frame()?).is_err());
    assert_eq!(
        raw.atoms()
            .first()
            .ok_or("atom")?
            .attributes
            .last()
            .ok_or("attribute")?
            .value,
        Value::Text("abc\"d".to_owned())
    );
    let collection=std::str::from_utf8(SOURCE)?.replace("M  V30 END CTAB","M  V30 BEGIN COLLECTION\nM  V30 MDLV30/STEABS ATOMS=(2 10 40)\nM  V30 END COLLECTION\nM  V30 END CTAB");
    let raw = MolFile::parse(collection.as_bytes())?;
    assert_eq!(raw.as_bytes(), collection.as_bytes());
    assert!(raw.to_fragment(frame()?).is_err());
    assert!(raw.blocks().iter().any(|block| block.name == "COLLECTION"));
    assert!(
        MolFile::parse(
            collection
                .replace("ATOMS=(2 10 40)", "ATOMS=(3 10 40)")
                .as_bytes()
        )
        .is_err()
    );
    let extensions = include_bytes!("fixtures/v3000/continuations-and-extensions.mol");
    let extended = MolFile::parse(extensions)?;
    assert_eq!(extended.as_bytes(), extensions);
    assert!(extended.to_fragment(frame()?).is_err());
    let fields = &extended.atoms().first().ok_or("extended atom")?.attributes;
    assert_eq!(
        fields.get(1).ok_or("quoted field")?.value,
        Value::Text("abc\"d".to_owned())
    );
    assert_eq!(
        fields.get(2).ok_or("counted field")?.value,
        Value::List(vec!["one two".to_owned(), "three\"four".to_owned()])
    );
    let encoded = write_v3000(
        &MolFile::parse(&bytes)?.to_fragment(frame()?)?,
        "Original methyl",
        frame()?,
    )?;
    assert!(
        encoded
            .split(|byte| *byte == b'\n')
            .all(|line| line.len() <= 80)
    );
    Ok(())
}

#[test]
fn independent_rdkit_verifies_original_graph_and_externally_generated_eight_decimal_input()
-> TestResult {
    let self_test: Json = serde_json::from_slice(&external_raw("self-test", b"")?)?;
    assert_eq!(self_test["self_test"], "passed");
    let reference = external(SOURCE)?;
    assert_eq!(
        reference["source_sha256"],
        "1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380"
    );
    assert_eq!(reference["atoms"].as_array().ok_or("atoms")?.len(), 2);
    assert_eq!(reference["atoms"][0]["explicit_h"], 3);
    assert_eq!(reference["atoms"][1]["explicit_h"], 3);
    assert_eq!(reference["bonds"][0]["begin"], 0);
    assert_eq!(reference["bonds"][0]["end"], 1);
    assert_eq!(reference["bonds"][0]["order"], 1.0);
    let emitted = external_raw("emit", b"")?;
    assert_eq!(emitted.len(), 314);
    assert_eq!(
        external(&emitted)?["source_sha256"],
        "c5260d67d8bf6e416c6d3be94e2dd4019516300a4fb4e36ba6c3f01193c50e38"
    );
    let raw = MolFile::parse(&emitted)?;
    assert_eq!(raw.as_bytes(), emitted);
    assert!(raw.unsupported().is_empty());
    let first = raw.atoms().first().ok_or("external atom")?;
    assert_eq!((first.x, first.y, first.z), (-1.23456789, 2.34567891, 0.0));
    assert_eq!(first.isotope.ok_or("isotope")?.get(), 13);
    let graph = raw.to_fragment(identity()?)?;
    assert!(
        graph
            .atoms()
            .iter()
            .all(|atom| atom.explicit_hydrogens.is_none()),
        "external writer omitted VAL; never invent authored H"
    );
    let encoded = write_v3000_with_header(&graph, raw.header(), identity()?)?;
    let reread = external(&encoded)?;
    assert_eq!(reread["atoms"][0]["isotope"], 13);
    assert_eq!(
        reread["atoms"][0]["position"],
        reference_position(-1.23456789, 2.34567891)
    );
    assert_eq!(reread["atoms"][0]["explicit_h"], 0);
    Ok(())
}
fn reference_position(x: f64, y: f64) -> Json {
    serde_json::json!([x, y, 0.0])
}

#[test]
fn thousand_atom_external_file_exceeds_the_v2000_limit_and_roundtrips_through_external_consumer()
-> TestResult {
    let bytes = external_raw("emit1000", b"")?;
    assert_eq!(bytes.len(), 49058);
    let raw = MolFile::parse(&bytes)?;
    assert_eq!(raw.atoms().len(), 1000);
    assert!(raw.bonds().is_empty());
    assert_eq!(raw.atoms().last().ok_or("last atom")?.source_index, 1000);
    let graph = raw.to_fragment(identity()?)?;
    assert_eq!(graph.atoms().len(), 1000);
    let encoded = write_v3000(&graph, "Original 1000 atom export", identity()?)?;
    let independent = external(&encoded)?;
    assert_eq!(
        independent["atoms"]
            .as_array()
            .ok_or("external atoms")?
            .len(),
        1000
    );
    assert!(
        independent["bonds"]
            .as_array()
            .ok_or("external bonds")?
            .is_empty()
    );
    assert_eq!(
        external(&bytes)?["source_sha256"],
        "3e7d06cca8a916618536ee915ac7b52dac10ba7f657398496ccc76578a64f27c"
    );
    assert!(
        encoded
            .split(|byte| *byte == b'\n')
            .all(|line| line.len() <= 80)
    );
    Ok(())
}

#[test]
fn primary_charge_radical_mass_and_explicit_valence_keep_spin_and_unspecified_h_distinct()
-> TestResult {
    for (attributes, charge, radical, isotope, hydrogens) in [
        (
            "MASS=13 RAD=2 VAL=3",
            0,
            Radical::Doublet,
            Some(13),
            Some(3),
        ),
        ("CHG=1 VAL=4", 1, Radical::None, None, Some(4)),
        ("CHG=-15 RAD=1 VAL=-1", -15, Radical::Singlet, None, Some(0)),
        ("CHG=15 RAD=3 VAL=-1", 15, Radical::Triplet, None, Some(0)),
        ("VAL=0", 0, Radical::None, None, None),
        ("", 0, Radical::None, None, None),
    ] {
        let bytes = single(&format!("4294967295 C 0 0 0 0 {attributes}"));
        let raw = MolFile::parse(&bytes)?;
        assert_eq!(raw.atoms().first().ok_or("raw")?.source_index, u32::MAX);
        let graph = raw.to_fragment(identity()?)?;
        let atom = graph.atoms().first().ok_or("atom")?;
        assert_eq!(
            (
                atom.charge,
                atom.radical,
                atom.isotope.map(NonZeroU16::get),
                atom.explicit_hydrogens
            ),
            (charge, radical, isotope, hydrogens)
        );
        let encoded = write_v3000(&graph, "Original authored fields", identity()?)?;
        let projected = MolFile::parse(&encoded)?.to_fragment(identity()?)?;
        assert_eq!(projected, graph);
        if matches!(radical, Radical::Singlet | Radical::Triplet) {
            let number = if radical == Radical::Singlet { 1 } else { 3 };
            assert!(std::str::from_utf8(&encoded)?.contains(&format!("RAD={number}")));
            assert_eq!(
                external(&bytes)?["atoms"][0]["radical_electrons"],
                2,
                "consumer collapses spin; published raw RAD and model enum are the fidelity oracle"
            );
        }
    }
    for (order, hydrogens) in [(1, 3), (2, 2), (3, 1)] {
        let bytes = std::str::from_utf8(SOURCE)?
            .replace("M  V30 7 1 10 40", &format!("M  V30 7 {order} 10 40"));
        let graph = MolFile::parse(bytes.as_bytes())?.to_fragment(identity()?)?;
        assert!(
            graph
                .atoms()
                .iter()
                .all(|atom| atom.explicit_hydrogens == Some(hydrogens))
        );
        assert_eq!(
            external(bytes.as_bytes())?["atoms"][0]["explicit_h"],
            hydrogens
        );
    }
    Ok(())
}

#[test]
fn full_precision_writer_is_independently_read_and_transformed_with_declared_error_bound()
-> TestResult {
    let mut atoms = Vec::new();
    for (index, point) in [
        Point2::new(100.0, 99.0)?,
        Point2::new(-0.12345678912345, 0.98765432198765)?,
    ]
    .into_iter()
    .enumerate()
    {
        let mut atom = Atom::new(
            AtomId::new(u32::try_from(index + 1)?).ok_or("id")?,
            Element::new(6).ok_or("element")?,
            Some(point),
        );
        atom.explicit_hydrogens = Some(0);
        atoms.push(atom);
    }
    let graph = Fragment::new(FragmentId::new(1).ok_or("fragment")?, atoms, vec![])?;
    let frame = frame()?;
    let encoded = write_v3000(&graph, "Original precision", frame)?;
    let independent = external(&encoded)?;
    assert_eq!(
        independent["atoms"][0]["position"],
        reference_position(1.1666666666666667, -1.125)
    );
    assert!(
        std::str::from_utf8(&encoded)?.contains("1.1666666666666667"),
        "V3000 must not apply V2000's f10.4 quantization"
    );
    let raw = MolFile::parse(&encoded)?;
    let recovered = raw.to_fragment(frame)?;
    for ((before, after), raw) in graph.atoms().iter().zip(recovered.atoms()).zip(raw.atoms()) {
        let before = before.position.ok_or("position")?;
        let after = after.position.ok_or("position")?;
        for (point, actual, origin, coordinate) in [
            (before.x_pt(), after.x_pt(), 72.0_f64, raw.x),
            (before.y_pt(), after.y_pt(), 72.0_f64, raw.y),
        ] {
            let tolerance = 16.0
                * f64::EPSILON
                * point
                    .abs()
                    .max(origin.abs())
                    .max((coordinate * 24.0).abs())
                    .max(1.0);
            assert!((point - actual).abs() <= tolerance);
        }
    }
    let tiny = MolTransform::new(1e-280, Point2::new(0.0, 0.0)?)?;
    let continued = write_v3000(&graph, "Original long finite coordinates", tiny)?;
    assert!(std::str::from_utf8(&continued)?.contains("-\nM  V30 "));
    assert!(
        continued
            .split(|byte| *byte == b'\n')
            .all(|line| line.len() <= 80)
    );
    let reparsed = MolFile::parse(&continued)?;
    assert_eq!(
        reparsed.atoms().first().ok_or("first")?.x.to_bits(),
        (100.0_f64 / 1e-280).to_bits()
    );
    assert_eq!(
        external(&continued)?["atoms"][0]["position"][0]
            .as_f64()
            .ok_or("external x")?
            .to_bits(),
        (100.0_f64 / 1e-280).to_bits()
    );
    let smallest = MolTransform::new(f64::from_bits(1), Point2::new(0.0, 0.0)?)?;
    assert_eq!(
        write_v3000(&graph, "Overflow must refuse", smallest)
            .err()
            .ok_or("overflow")?
            .code(),
        "unsupported_content"
    );
    Ok(())
}

#[test]
fn all_118_element_symbols_are_verified_by_an_external_consumer() -> TestResult {
    let mut atoms = Vec::new();
    for number in 1..=118_u8 {
        let mut atom = Atom::new(
            AtomId::new(u32::from(number)).ok_or("id")?,
            Element::new(number).ok_or("element")?,
            Some(Point2::new(f64::from(number), 0.0)?),
        );
        atom.explicit_hydrogens = Some(0);
        atoms.push(atom);
    }
    let graph = Fragment::new(FragmentId::new(1).ok_or("fragment")?, atoms, vec![])?;
    let bytes = write_v3000(&graph, "Original elemental table vector", identity()?)?;
    let external = external(&bytes)?;
    let numbers = external["atoms"]
        .as_array()
        .ok_or("atoms")?
        .iter()
        .map(|atom| atom["element"].as_u64().ok_or("element number"))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(numbers, (1..=118_u64).collect::<Vec<_>>());
    assert_eq!(MolFile::parse(&bytes)?.to_fragment(identity()?)?, graph);
    Ok(())
}

#[test]
fn query_stereo_mapping_3d_unknown_attributes_and_blocks_are_preserved_but_never_projected()
-> TestResult {
    for record in [
        "10 C 0 0 0 0 HCOUNT=3",
        "10 C 0 0 0 0 CFG=0",
        "10 C 0 0 0 5 VAL=-1",
        "10 C 0 0 1 0",
        "10 R# 0 0 0 0",
        "10 C 0 0 0 0 MASS=13.5",
        "10 C 0 0 0 0 MASS=65536",
        "10 C 0 0 0 0 XDATA=(2 \"a b\" \"c\"\"d\")",
    ] {
        let bytes = single(record);
        let raw = MolFile::parse(&bytes)?;
        assert_eq!(raw.as_bytes(), bytes);
        assert!(!raw.unsupported().is_empty());
        assert_eq!(
            raw.to_fragment(identity()?)
                .err()
                .ok_or("unsupported projection")?
                .code(),
            "unsupported_content"
        );
    }
    let bytes = single("10 C 0 0 0 0 HCOUNT=3");
    let query = external(&bytes)?;
    assert!(
        query["atoms"][0]["query"]
            .as_str()
            .ok_or("query")?
            .contains("AtomImplicitHCount 3")
    );
    assert_eq!(query["atoms"][0]["explicit_h"], 0);
    assert_eq!(
        external(&single("10 C 0 0 0 0 MASS=13.5"))?["atoms"][0]["isotope"],
        13,
        "pinned consumer truncates; the strict projection must not"
    );
    for replacement in ["M  V30 7 4 10 40", "M  V30 7 1 10 40 CFG=3"] {
        let bytes = std::str::from_utf8(SOURCE)?.replace("M  V30 7 1 10 40", replacement);
        let raw = MolFile::parse(bytes.as_bytes())?;
        assert_eq!(raw.as_bytes(), bytes.as_bytes());
        assert_eq!(
            raw.to_fragment(frame()?)
                .err()
                .ok_or("bond feature")?
                .code(),
            "unsupported_content"
        );
    }
    let source =
        std::str::from_utf8(SOURCE)?.replace("  Bondline          2D", "  Bondline          3D");
    assert!(
        MolFile::parse(source.as_bytes())?
            .to_fragment(frame()?)
            .is_err()
    );
    Ok(())
}

#[test]
fn normative_ctab_counts_ignore_legacy_counts_and_unknown_blocks_keep_flat_source_spans()
-> TestResult {
    let legacy = std::str::from_utf8(SOURCE)?.replace(
        "  0  0  0  0  0  0            999 V3000",
        "  9  8  0  0  1  0            999 V3000",
    );
    let raw = MolFile::parse(legacy.as_bytes())?;
    assert_eq!(
        raw.to_fragment(frame()?)?.atoms().len(),
        2,
        "published CTAB counts take precedence; strict RDKit disagrees on this legacy header"
    );
    let unknown=std::str::from_utf8(SOURCE)?.replace("M  V30 END CTAB","M  V30 BEGIN EXTENSION\nM  V30 UNKNOWN X=(2 \"one two\" \"three\"\"four\")\nM  V30 BEGIN CHILD\nM  V30 INFO=\"original\"\nM  V30 END CHILD\nM  V30 END EXTENSION\nM  V30 END CTAB");
    let raw = MolFile::parse(unknown.as_bytes())?;
    assert_eq!(raw.as_bytes(), unknown.as_bytes());
    assert!(raw.to_fragment(frame()?).is_err());
    for (index, block) in raw.blocks().iter().enumerate() {
        let bytes = raw
            .as_bytes()
            .get(block.range.clone())
            .ok_or("block span")?;
        assert!(bytes.starts_with(format!("M  V30 BEGIN {}", block.name).as_bytes()));
        if let Some(parent) = block.parent {
            assert!(parent < index);
            let outer = raw.blocks().get(parent).ok_or("parent")?;
            assert!(outer.range.start <= block.range.start && outer.range.end >= block.range.end);
        }
    }
    Ok(())
}

#[test]
fn nonzero_coordinate_underflow_cannot_be_flattened_to_an_apparently_2d_zero() -> TestResult {
    for coordinate in ["1e-9999", "-1e-9999"] {
        let bytes = single(&format!("10 C 0 0 {coordinate} 0"));
        assert!(
            MolFile::parse(&bytes).is_err(),
            "a nonzero source z cannot become supported 2D by f64 underflow"
        );
    }
    let bytes = single("10 C -0.0 0e100 -0 0 VAL=-1");
    assert!(MolFile::parse(&bytes)?.to_fragment(identity()?).is_ok());
    Ok(())
}

#[test]
fn malformed_counts_references_order_quotes_lists_and_nonfinite_numbers_are_refused() -> TestResult
{
    let source = std::str::from_utf8(SOURCE)?;
    for bytes in [
        source.replace("COUNTS 2 1", "COUNTS 3 1"),
        source.replace(
            "M  V30 END CTAB",
            "M  V30 COUNTS 2 1 0 0 0\nM  V30 END CTAB",
        ),
        source.replace("40 C", "10 C"),
        source.replace("10 C", "0 C"),
        source.replace("7 1 10 40", "7 1 10 99"),
        source.replace("7 1 10 40", "7 1 10 10"),
        source
            .replace("M  V30 7 1 10 40", "M  V30 7 1 10 40\nM  V30 7 2 10 40")
            .replace("COUNTS 2 1", "COUNTS 2 2"),
        source.replace("END ATOM", "END OTHER"),
        source.replace("BEGIN ATOM", "BEGIN BOND"),
        source.replace("M  V30 END CTAB\n", ""),
        source.replace("M  END\n", ""),
        source.replace("M  V30 COUNTS", "M V30 COUNTS"),
        source.replace("999 V3000", "998 V3000"),
        format!("{source}\n"),
        format!("{source}$$$$\n"),
    ] {
        assert!(
            MolFile::parse(bytes.as_bytes()).is_err(),
            "accepted malformed V3000"
        );
    }
    for line in [
        "10 C NaN 0 0 0",
        "10 C inf 0 0 0",
        "10 C 1e9999 0 0 0",
        "10 C 1e 0 0 0",
        "10 C 0x1 0 0 0",
        "10 C 0 0 0 0 CHG=16",
        "10 C 0 0 0 0 RAD=4",
        "10 C 0 0 0 0 VAL=15",
        "10 C 0 0 0 0 HCOUNT=-2",
        "10 C 0 0 0 0 X=\"unterminated",
        "10 C 0 0 0 0 X=\"closed\"extra",
        "10 C 0 0 0 0 X=(3 a b)",
        "10 C 0 0 0 0 X=(1 a b)",
        "10 C 0 0 0 0 X=(1 \"bad)",
        "10 C 0 0 0 0 X=a'b",
        "10 C 0 0 0 0 CHG=1 CHG=2",
        "10 C 0 0 0 0 VAL=(1 4)",
    ] {
        assert!(
            MolFile::parse(&single(line)).is_err(),
            "accepted malformed entry {line:?}"
        );
    }
    for end in 0..SOURCE.len().saturating_sub(2) {
        assert!(
            MolFile::parse(SOURCE.get(..end).ok_or("source prefix")?).is_err(),
            "accepted truncated source at {end}"
        );
    }
    let invalid_h = source.replace("VAL=4", "VAL=-1");
    let raw = MolFile::parse(invalid_h.as_bytes())?;
    assert!(
        raw.to_fragment(frame()?).is_err(),
        "declared zero valence cannot imply a negative H count"
    );
    let crlf = source.replace('\n', "\r\n");
    assert_eq!(MolFile::parse(crlf.as_bytes())?.as_bytes(), crlf.as_bytes());
    for bad in [
        source.replace('\n', "\r\r\n").into_bytes(),
        [SOURCE, b"\0"].concat(),
        [SOURCE, b"\xff"].concat(),
    ] {
        assert!(MolFile::parse(&bad).is_err());
    }
    Ok(())
}

fn continued(entry: &str) -> String {
    let chunks = entry.as_bytes().chunks(72).collect::<Vec<_>>();
    let mut output = String::new();
    for (index, chunk) in chunks.iter().enumerate() {
        output.push_str("M  V30 ");
        for byte in *chunk {
            output.push(char::from(*byte));
        }
        if index + 1 < chunks.len() {
            output.push('-');
        }
        output.push('\n');
    }
    output
}

#[test]
fn byte_physical_logical_segment_token_depth_and_count_budgets_are_enforced() -> TestResult {
    let source = std::str::from_utf8(SOURCE)?;
    let over_count = source.replace("COUNTS 2 1", "COUNTS 100001 1");
    assert_eq!(
        MolFile::parse(over_count.as_bytes())
            .err()
            .ok_or("count bound")?
            .code(),
        "resource_limit"
    );
    assert_eq!(
        MolFile::parse(&vec![b' '; 64 * 1024 * 1024 + 1])
            .err()
            .ok_or("byte bound")?
            .code(),
        "resource_limit"
    );
    let long_line = format!(
        "{}\n{}",
        "x".repeat(81),
        source.split_once('\n').ok_or("first line")?.1
    );
    assert_eq!(
        MolFile::parse(long_line.as_bytes())
            .err()
            .ok_or("physical bound")?
            .code(),
        "resource_limit"
    );
    let max_header = source.replacen(source.lines().next().ok_or("title")?, &"x".repeat(80), 1);
    assert!(MolFile::parse(max_header.as_bytes()).is_ok());
    let dangling = source.replace(
        "M  V30 10 C -1.25 0.5 0 0 VAL=4",
        "M  V30 10 C -1.25 0.5 0 0 VAL=4-",
    );
    assert!(MolFile::parse(dangling.as_bytes()).is_err());
    let too_many_segments = source.replace(
        "M  V30 BEGIN CTAB",
        &format!("{}M  V30 BEGIN CTAB", "M  V30 -\n".repeat(1024)),
    );
    assert_eq!(
        MolFile::parse(too_many_segments.as_bytes())
            .err()
            .ok_or("segment bound")?
            .code(),
        "resource_limit"
    );
    let entry = format!("10 C 0 0 0 0 X=\"{}\"", "a".repeat(65536));
    let logical = single("10 C 0 0 0 0 X=\"placeholder\"");
    let logical = std::str::from_utf8(&logical)?.replace(
        "M  V30 10 C 0 0 0 0 X=\"placeholder\"\n",
        &continued(&entry),
    );
    assert_eq!(
        MolFile::parse(logical.as_bytes())
            .err()
            .ok_or("logical bound")?
            .code(),
        "resource_limit"
    );
    let many_tokens = continued(&format!("UNKNOWN {}", "a ".repeat(8192)));
    let many_tokens = source.replace("M  V30 END CTAB", &format!("{many_tokens}M  V30 END CTAB"));
    assert_eq!(
        MolFile::parse(many_tokens.as_bytes())
            .err()
            .ok_or("token bound")?
            .code(),
        "resource_limit"
    );
    let too_many_list = single("10 C 0 0 0 0 X=(8193)");
    assert_eq!(
        MolFile::parse(&too_many_list)
            .err()
            .ok_or("list bound")?
            .code(),
        "resource_limit"
    );
    let depth = source.replace(
        "M  V30 END CTAB",
        &format!(
            "{}{}M  V30 END CTAB",
            "M  V30 BEGIN EXTRA\n".repeat(256),
            "M  V30 END EXTRA\n".repeat(256)
        ),
    );
    assert_eq!(
        MolFile::parse(depth.as_bytes())
            .err()
            .ok_or("depth bound")?
            .code(),
        "resource_limit"
    );
    Ok(())
}

#[test]
fn writer_refuses_unmodeled_graph_data_bad_headers_and_coordinate_overflow_without_touching_inputs()
-> TestResult {
    let base = MolFile::parse(SOURCE)?.to_fragment(frame()?)?;
    let original = base.clone();
    for header in [
        ["bad\nheader", "", ""],
        ["$$$$reserved", "", ""],
        ["title", "  Bondline          3D", ""],
        ["title", "non-ASCII é", ""],
    ] {
        assert!(write_v3000_with_header(&base, header, frame()?).is_err());
    }
    for variant in 0..4 {
        let mut atoms = base.atoms().to_vec();
        let mut bonds = base.bonds().to_vec();
        match variant {
            0 => atoms.first_mut().ok_or("atom")?.position = None,
            1 => atoms.first_mut().ok_or("atom")?.charge = 16,
            2 => atoms.first_mut().ok_or("atom")?.explicit_hydrogens = Some(u16::MAX),
            _ => bonds.first_mut().ok_or("bond")?.order = BondOrder::Aromatic,
        }
        let graph = Fragment::new(base.id(), atoms, bonds)?;
        assert_eq!(
            write_v3000(&graph, "Original refusal", frame()?)
                .err()
                .ok_or("writer refusal")?
                .code(),
            "unsupported_content"
        );
    }
    assert_eq!(base, original);
    assert_eq!(MolFile::parse(SOURCE)?.as_bytes(), SOURCE);
    let empty = Fragment::new(FragmentId::new(1).ok_or("id")?, vec![], vec![])?;
    let encoded = write_v3000(&empty, "", identity()?)?;
    assert!(MolFile::parse(&encoded)?.atoms().is_empty());
    assert!(
        external(&encoded)?["atoms"]
            .as_array()
            .ok_or("external empty")?
            .is_empty()
    );
    Ok(())
}

#[test]
fn complete_file_line_and_decoded_token_budgets_are_checked_across_entries() -> TestResult {
    let original = std::str::from_utf8(SOURCE)?;
    let extra = "M  V30 X\n".repeat(1_000_000 - original.lines().count());
    let maximum = original.replace("M  V30 END CTAB", &format!("{extra}M  V30 END CTAB"));
    assert_eq!(maximum.lines().count(), 1_000_000);
    assert_eq!(
        MolFile::parse(maximum.as_bytes())?.as_bytes(),
        maximum.as_bytes()
    );
    let excess = maximum.replace("M  V30 END CTAB", "M  V30 X\nM  V30 END CTAB");
    assert_eq!(
        MolFile::parse(excess.as_bytes())
            .err()
            .ok_or("file line budget")?
            .code(),
        "resource_limit"
    );
    // Each entry is within the 8192-token/64KiB bounds; the separate file budget
    // limits amplification from retained decoded strings across many entries.
    let row = continued(&format!("X {}", "a ".repeat(8191)));
    let many = original.replace(
        "M  V30 END CTAB",
        &format!("{}M  V30 END CTAB", row.repeat(489)),
    );
    let error = MolFile::parse(many.as_bytes())
        .err()
        .ok_or("whole-file token budget")?;
    assert_eq!(error.code(), "resource_limit");
    assert!(error.to_string().contains("4000000"));
    Ok(())
}
