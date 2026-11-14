use bondline_io::{
    mol::Version,
    sdf::{DataItemInput, SdfFile, SdfRecordInput, write, write_v2000},
};
use serde_json::{Value, json};
use std::{
    io::Write as _,
    process::{Command, Stdio},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const MIXED: &[u8] = include_bytes!("fixtures/sdf/mixed-v2000-v3000.sdf");
const V2: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");
const V3: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");

fn oracle(script: &str, mode: &str, input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(format!("{}/tests/{script}", env!("CARGO_MANIFEST_DIR")))
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(input)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "external consumer: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(result.stdout)
}
fn describe(source: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&oracle(
        "sdf_mixed_oracle.py",
        "describe",
        source,
    )?)?)
}
fn rewrite(source: &SdfFile<'_>, legacy: bool) -> Result<Vec<u8>, bondline_io::sdf::SdfError> {
    let items = source
        .records()
        .iter()
        .map(|record| {
            record
                .fields()
                .iter()
                .map(|field| DataItemInput {
                    header: field.header(),
                    value_lines: field.value_lines(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let inputs = source
        .records()
        .iter()
        .zip(&items)
        .map(|(record, items)| SdfRecordInput {
            mol_bytes: record.mol_bytes(),
            items,
        })
        .collect::<Vec<_>>();
    if legacy {
        write_v2000(&inputs)
    } else {
        write(&inputs)
    }
}

#[test]
fn original_mixed_records_expose_the_actual_mol_version_and_keep_every_source_byte() -> TestResult {
    let source = SdfFile::parse(MIXED)?;
    assert_eq!(source.as_bytes(), MIXED);
    assert_eq!(source.as_bytes().as_ptr(), MIXED.as_ptr());
    assert_eq!(source.records().len(), 2);
    let first = source.records().first().ok_or("first record")?;
    let second = source.records().get(1).ok_or("second record")?;
    assert_eq!(first.mol_version(), Version::V3000);
    assert_eq!(second.mol_version(), Version::V2000);
    assert_eq!(first.mol_bytes(), V3);
    assert_eq!(second.mol_bytes(), V2);
    assert_eq!(MIXED.len(), 777);
    assert_eq!(first.range(), 0..462);
    assert_eq!(first.mol_range(), 0..344);
    assert_eq!(first.delimiter_range(), 457..462);
    assert_eq!(second.range(), 462..777);
    assert_eq!(second.mol_range(), 462..744);
    assert_eq!(second.delimiter_range(), 772..777);
    assert_eq!(first.fields().len(), 6);
    let expected = [
        (344..355, 344..354, 354..354),
        (355..370, 355..363, 363..369),
        (370..386, 370..378, 378..385),
        (386..427, 386..411, 411..426),
        (427..441, 427..436, 436..440),
        (441..457, 441..452, 452..456),
    ];
    for (field, (whole, header, value)) in first.fields().iter().zip(expected) {
        assert_eq!(field.range(), whole);
        assert_eq!(field.header_range(), header);
        assert_eq!(field.value_range(), value.clone());
        assert_eq!(MIXED.get(value).ok_or("raw value span")?, field.raw_value());
    }
    assert_eq!(
        first.fields().get(1).ok_or("duplicate one")?.value_lines(),
        ["first"]
    );
    assert_eq!(
        first.fields().get(2).ok_or("duplicate two")?.value_lines(),
        ["second"]
    );
    assert_eq!(
        first.fields().get(3).ok_or("combined")?.dt_number(),
        Some("0009")
    );
    assert_eq!(first.fields().get(4).ok_or("DT")?.name(), None);
    assert_eq!(first.fields().get(4).ok_or("DT")?.dt_number(), Some("0012"));
    assert_eq!(
        first.fields().get(5).ok_or("spaces")?.value_lines(),
        ["   "]
    );
    let items = source
        .records()
        .iter()
        .map(|record| {
            record
                .fields()
                .iter()
                .map(|field| DataItemInput {
                    header: field.header(),
                    value_lines: field.value_lines(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let inputs = source
        .records()
        .iter()
        .zip(&items)
        .map(|(record, items)| SdfRecordInput {
            mol_bytes: record.mol_bytes(),
            items,
        })
        .collect::<Vec<_>>();
    assert_eq!(write(&inputs)?, MIXED);
    assert_eq!(
        write_v2000(&inputs)
            .err()
            .ok_or("V2000 writer refusal")?
            .code(),
        "unsupported_content"
    );
    assert_eq!(source.as_bytes(), MIXED);
    Ok(())
}

#[test]
fn external_sdwriter_switches_versions_and_full_supplier_reads_ordinary_output() -> TestResult {
    let result: Value = serde_json::from_slice(&oracle("sdf_mixed_oracle.py", "self-test", b"")?)?;
    assert_eq!(result["self_test"], "passed");
    assert_eq!(result["source_bytes"], 667);
    assert_eq!(
        result["source_sha256"],
        "88516961b6a197950ac83e09c441387bd2e9ed1c2da2d94accdffd1b9c359ca6"
    );
    let external = oracle("sdf_mixed_oracle.py", "emit", b"")?;
    let source = SdfFile::parse(&external)?;
    assert_eq!(source.as_bytes(), external);
    assert_eq!(
        source
            .records()
            .iter()
            .map(|record| record.mol_version())
            .collect::<Vec<_>>(),
        [Version::V3000, Version::V2000]
    );
    assert!(
        source
            .records()
            .iter()
            .all(|record| record.fields().len() == 3)
    );
    let output = rewrite(&source, false)?;
    let reopened = SdfFile::parse(&output)?;
    for (before, after) in source.records().iter().zip(reopened.records()) {
        assert_eq!(before.mol_bytes(), after.mol_bytes());
        assert_eq!(before.mol_version(), after.mol_version());
        for (old, new) in before.fields().iter().zip(after.fields()) {
            assert_eq!(old.header(), new.header());
            assert_eq!(old.value_lines(), new.value_lines());
        }
    }
    let consumer = describe(&output)?;
    assert_eq!(
        consumer["records"]
            .as_array()
            .ok_or("consumer records")?
            .len(),
        2
    );
    assert_eq!(consumer["records"][0]["atoms"][1]["element"], 8);
    assert_eq!(consumer["records"][1]["atoms"][1]["element"], 7);
    for record in consumer["records"].as_array().ok_or("records")? {
        assert_eq!(record["properties"]["EMPTY"], "");
        assert_eq!(record["properties"]["MULTI"], "one\ntwo");
        assert_eq!(record["bonds"], json!([{"begin":0,"end":1,"order":1.0}]));
    }
    Ok(())
}

#[test]
fn existing_molecular_oracle_checks_eight_decimal_and_thousand_atom_payloads_without_conversion()
-> TestResult {
    for mode in ["emit", "emit1000"] {
        let mol = oracle("v3000_oracle.py", mode, b"")?;
        let molecular: Value =
            serde_json::from_slice(&oracle("v3000_oracle.py", "describe", &mol)?)?;
        let output = write(&[SdfRecordInput {
            mol_bytes: &mol,
            items: &[],
        }])?;
        let source = SdfFile::parse(&output)?;
        let record = source.records().first().ok_or("record")?;
        assert_eq!(record.mol_version(), Version::V3000);
        assert_eq!(record.mol_bytes(), mol);
        let consumer = describe(&output)?;
        let read = consumer["records"]
            .as_array()
            .ok_or("records")?
            .first()
            .ok_or("record")?;
        let atoms = read["atoms"].as_array().ok_or("atoms")?;
        assert_eq!(atoms.len(), if mode == "emit" { 2 } else { 1000 });
        assert_eq!(
            atoms.len(),
            molecular["atoms"]
                .as_array()
                .ok_or("molecular atoms")?
                .len()
        );
        if mode == "emit" {
            assert_eq!(read["atoms"][0]["isotope"], 13);
            assert_eq!(
                read["atoms"][0]["position"],
                json!([-1.23456789, 2.34567891, 0.0])
            );
        }
        assert_eq!(rewrite(&source, false)?, output);
        assert_eq!(
            rewrite(&source, true).err().ok_or("V2 refusal")?.code(),
            "unsupported_content"
        );
    }
    Ok(())
}

#[test]
fn opaque_v3000_queries_extensions_crlf_and_missing_final_lf_are_never_projected() -> TestResult {
    let extended = include_bytes!("fixtures/v3000/continuations-and-extensions.mol");
    let extended = std::str::from_utf8(extended)?.replace('\n', "\r\n");
    let frame = bondline_io::mol::MolTransform::new(1.0, bondline_model::Point2::new(0.0, 0.0)?)?;
    assert!(
        bondline_io::mol::v3000::MolFile::parse(extended.as_bytes())?
            .to_fragment(frame)
            .is_err()
    );
    let query = std::str::from_utf8(V3)?
        .replace("VAL=4", "HCOUNT=1 CFG=1")
        .replace("7 1 10 40", "7 4 10 40");
    let opaque_v2 = std::str::from_utf8(V2)?.replace("M  END", "M  XYZ  1   1   7\nM  END");
    let empty_title = std::str::from_utf8(V3)?.replacen("Bondline original V3000 probe", "", 1);
    for mol in [
        extended.as_bytes(),
        query.as_bytes(),
        opaque_v2.as_bytes(),
        empty_title.as_bytes(),
        V3.strip_suffix(b"\n").ok_or("last LF")?,
    ] {
        let output = write(&[SdfRecordInput {
            mol_bytes: mol,
            items: &[],
        }])?;
        let expected = [
            mol,
            if mol.ends_with(b"\n") { b"" } else { b"\n" },
            b"$$$$\n",
        ]
        .concat();
        assert_eq!(output, expected);
        let source = SdfFile::parse(&output)?;
        assert_eq!(source.as_bytes(), output);
        assert_eq!(rewrite(&source, false)?, output);
    }
    Ok(())
}

#[test]
fn generic_writer_keeps_the_existing_v2000_profile_and_all_ordered_data_unchanged() -> TestResult {
    let original = include_bytes!("fixtures/sdf/ordered-fields-v2000.sdf");
    let source = SdfFile::parse(original)?;
    assert!(
        source
            .records()
            .iter()
            .all(|record| record.mol_version() == Version::V2000)
    );
    let old = rewrite(&source, true)?;
    assert_eq!(rewrite(&source, false)?, old);
    let legacy: Value = serde_json::from_slice(&oracle("sdf_oracle.py", "describe", original)?)?;
    assert_eq!(legacy["records"].as_array().ok_or("raw records")?.len(), 2);
    assert_eq!(
        legacy["rdkit_records"]
            .as_array()
            .ok_or("consumer diagnostic")?
            .len(),
        1
    );
    assert_eq!(source.as_bytes(), original);
    Ok(())
}

#[test]
fn malformed_later_molecules_and_resource_failures_never_return_partial_sdf() -> TestResult {
    let valid = std::str::from_utf8(V3)?;
    let cases = [
        valid.replace("COUNTS 2 1", "COUNTS 3 1"),
        valid.replace("7 1 10 40", "7 1 10 999"),
        valid.replace("-1.25", "nan"),
        valid.replace("M  V30 END CTAB\n", ""),
        std::str::from_utf8(V2)?.replace(" V2000", " V3000"),
        valid.replace(" V3000", " V4000"),
    ];
    for (case_index, bad) in cases.into_iter().enumerate() {
        let input = [V2, b"$$$$\n", bad.as_bytes(), b"$$$$\n"].concat();
        let before = input.clone();
        assert_eq!(
            SdfFile::parse(&input)
                .err()
                .ok_or_else(|| format!("invalid later record case {case_index} was accepted"))?
                .code(),
            "invalid_sdf"
        );
        assert_eq!(
            write(&[
                SdfRecordInput {
                    mol_bytes: V2,
                    items: &[]
                },
                SdfRecordInput {
                    mol_bytes: bad.as_bytes(),
                    items: &[]
                }
            ])
            .err()
            .ok_or("writer refusal")?
            .code(),
            "invalid_sdf"
        );
        assert_eq!(input, before);
    }
    let large_counts = valid.replace("COUNTS 2 1", "COUNTS 100001 1");
    assert_eq!(
        write(&[SdfRecordInput {
            mol_bytes: large_counts.as_bytes(),
            items: &[]
        }])
        .err()
        .ok_or("count limit")?
        .code(),
        "resource_limit"
    );
    let long_line = valid.replace("M  V30 10 C", &format!("M  V30 {} C", "1".repeat(70)));
    assert_eq!(
        write(&[SdfRecordInput {
            mol_bytes: long_line.as_bytes(),
            items: &[]
        }])
        .err()
        .ok_or("physical line limit")?
        .code(),
        "resource_limit"
    );
    for bad in [
        format!("{valid}\n"),
        valid.replacen("Bondline original V3000 probe", "$$$$metadata", 1),
    ] {
        assert!(
            write(&[SdfRecordInput {
                mol_bytes: bad.as_bytes(),
                items: &[]
            }])
            .is_err()
        );
    }
    for tail in [
        b"".as_slice(),
        b"\n",
        b"> <DATA>\nunfinished",
        b"> <DATA>\nvalue\n$$$$\n",
    ] {
        let input = [V3, tail].concat();
        assert!(SdfFile::parse(&input).is_err());
    }
    assert_eq!(V3, include_bytes!("fixtures/v3000/noncontiguous-cc.mol"));
    Ok(())
}

#[test]
fn actual_mol_stamp_selects_validation_while_unknown_property_names_do_not_guess_a_version()
-> TestResult {
    // Its V2000 counts are zero. Opaque M V30 property lines are not a license to
    // override the declared, actually validated encoding or erase their bytes.
    let source = std::str::from_utf8(V3)?.replace(" V3000", " V2000");
    let original = bondline_io::mol::MolFile::parse(source.as_bytes())?;
    assert!(original.atoms().is_empty());
    assert!(!original.unsupported().is_empty());
    let output = write_v2000(&[SdfRecordInput {
        mol_bytes: source.as_bytes(),
        items: &[],
    }])?;
    assert_eq!(
        write(&[SdfRecordInput {
            mol_bytes: source.as_bytes(),
            items: &[]
        }])?,
        output
    );
    let parsed = SdfFile::parse(&output)?;
    let record = parsed.records().first().ok_or("record")?;
    assert_eq!(record.mol_version(), Version::V2000);
    assert_eq!(record.mol_bytes(), source.as_bytes());
    Ok(())
}
