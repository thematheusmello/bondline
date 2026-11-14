use bondline_io::{
    mol::{MolFile, MolTransform, Version},
    sdf::{RecordId, SdfDocument, SdfError},
};
use bondline_model::Point2;
use serde_json::{Value, json};
use std::{
    io::Write as _,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORDERED: &[u8] = include_bytes!("fixtures/sdf/ordered-fields-v2000.sdf");
const MIXED: &[u8] = include_bytes!("fixtures/sdf/mixed-v2000-v3000.sdf");
const REPLACEMENT: &[u8] = b"revised\nmultiline\n";

#[test]
fn only_the_second_duplicate_occurrence_changes_by_the_predeclared_literal_splice() -> TestResult {
    for (source, field_index, value_range, record_end, delta, first_version) in [
        (ORDERED, 5, 383..387, 424, 14, Version::V2000),
        (MIXED, 2, 378..385, 462, 11, Version::V3000),
    ] {
        let original = SdfDocument::parse(Arc::from(source))?;
        let before = original.clone();
        let expected = [
            source.get(..value_range.start).ok_or("literal prefix")?,
            REPLACEMENT,
            source.get(value_range.end..).ok_or("literal suffix")?,
        ]
        .concat();
        let edited = original.with_field_value(
            RecordId::new(1).ok_or("record")?,
            field_index,
            &["revised", "multiline"],
        )?;
        assert_eq!(edited.as_bytes(), expected);
        assert_ne!(edited, original);
        assert_eq!(original, before);
        assert_eq!(original.as_bytes(), source);
        assert_eq!(edited.as_bytes().len(), source.len() + delta);
        assert_eq!(edited.records().len(), 2);
        assert_eq!(edited.records()[0].id().get(), 1);
        assert_eq!(edited.records()[1].id().get(), 2);
        assert_eq!(edited.records()[0].mol_version(), first_version);
        assert_eq!(edited.records()[1].mol_version(), Version::V2000);
        assert_eq!(edited.records()[0].range(), 0..record_end + delta);
        assert_eq!(
            edited.records()[1].range(),
            record_end + delta..source.len() + delta
        );
        assert_eq!(
            edited.as_bytes().get(record_end + delta..),
            source.get(record_end..)
        );
        let old = original.read()?;
        let changed = edited.read()?;
        for (old, changed) in old.records().iter().zip(changed.records()) {
            assert_eq!(old.mol_bytes(), changed.mol_bytes());
            assert_eq!(old.mol_version(), changed.mol_version());
            assert_eq!(old.fields().len(), changed.fields().len());
            for (index, (old, changed)) in old.fields().iter().zip(changed.fields()).enumerate() {
                assert_eq!(old.header(), changed.header());
                assert_eq!(old.name(), changed.name());
                assert_eq!(old.dt_number(), changed.dt_number());
                if old.name() == Some("DUP") && index == field_index {
                    assert_eq!(changed.value_lines(), ["revised", "multiline"]);
                    assert_eq!(changed.raw_value(), REPLACEMENT);
                } else {
                    assert_eq!(old.raw_value(), changed.raw_value());
                    assert_eq!(old.value_lines(), changed.value_lines());
                }
            }
        }
    }
    Ok(())
}

#[test]
fn other_records_and_dt_only_fields_use_the_same_explicit_identity_and_occurrence_rules()
-> TestResult {
    let original = SdfDocument::parse(MIXED.into())?;
    let record_two = original.with_field_value(
        RecordId::new(2).ok_or("second record")?,
        0,
        &["second edited"],
    )?;
    let expected = [&MIXED[..757], b"second edited\n", &MIXED[771..]].concat();
    assert_eq!(record_two.as_bytes(), expected);
    assert_eq!(record_two.as_bytes().len(), 777);
    assert_eq!(record_two.records(), original.records());
    assert_ne!(
        record_two, original,
        "same-size data-only changes must affect equality"
    );
    assert_eq!(record_two.as_bytes().get(..757), MIXED.get(..757));
    assert_eq!(
        record_two.read()?.records()[1].fields()[0].value_lines(),
        ["second edited"]
    );
    let noop = original.with_field_value(
        RecordId::new(2).ok_or("second record")?,
        0,
        &["second record"],
    )?;
    assert_eq!(noop.as_bytes().as_ptr(), original.as_bytes().as_ptr());
    assert_eq!(noop.records().as_ptr(), original.records().as_ptr());
    assert!(matches!(
        original.with_field_value(RecordId::new(2).ok_or("second record")?, 1, &["x"]),
        Err(SdfError::MissingField {
            record_id: 2,
            field_index: 1
        })
    ));
    let numbered =
        original.with_field_value(RecordId::new(1).ok_or("first record")?, 4, &["modified"])?;
    assert_eq!(
        numbered.as_bytes(),
        [&MIXED[..436], b"modified\n", &MIXED[440..]].concat()
    );
    let numbered_view = numbered.read()?;
    let field = &numbered_view.records()[0].fields()[4];
    assert_eq!(field.name(), None);
    assert_eq!(field.dt_number(), Some("0012"));
    assert_eq!(field.header(), "> DT0012");
    assert_eq!(field.value_lines(), ["modified"]);
    assert_eq!(original.as_bytes(), MIXED);
    Ok(())
}

#[test]
fn identical_value_lines_share_the_source_and_index_without_normalizing_crlf() -> TestResult {
    let crlf = std::str::from_utf8(ORDERED)?.replace('\n', "\r\n");
    for (source, index, values) in [
        (ORDERED.to_vec(), 5, vec!["two"]),
        (crlf.into_bytes(), 5, vec!["two"]),
        (MIXED.to_vec(), 2, vec!["second"]),
        (ORDERED.to_vec(), 1, vec![]),
    ] {
        let original = SdfDocument::parse(source.clone().into())?;
        let same = original.with_field_value(RecordId::new(1).ok_or("record")?, index, &values)?;
        assert_eq!(same, original);
        assert_eq!(same.as_bytes(), source);
        assert_eq!(same.as_bytes().as_ptr(), original.as_bytes().as_ptr());
        assert_eq!(same.records().as_ptr(), original.records().as_ptr());
        assert_eq!(
            SdfDocument::retained_bytes([&original, &same]),
            SdfDocument::retained_bytes([&original]) + size_of::<SdfDocument>()
        );
    }
    Ok(())
}

#[test]
fn missing_record_and_field_are_typed_not_found_failures_with_the_original_unchanged() -> TestResult
{
    let original = SdfDocument::parse(ORDERED.into())?;
    let before = original.clone();
    for value in [3, u32::MAX] {
        let error = original
            .with_field_value(RecordId::new(value).ok_or("record")?, 0, &[""])
            .err()
            .ok_or("missing record")?;
        assert!(matches!(error, SdfError::MissingRecord { record_id } if record_id == value));
        assert_eq!(error.code(), "not_found");
    }
    for (record_id, field_index) in [(1, 7), (1, usize::MAX), (2, 0)] {
        let error = original
            .with_field_value(
                RecordId::new(record_id).ok_or("record")?,
                field_index,
                &[""],
            )
            .err()
            .ok_or("missing field")?;
        assert!(
            matches!(error, SdfError::MissingField { record_id: actual_record, field_index: actual_field } if actual_record == record_id && actual_field == field_index)
        );
        assert_eq!(error.code(), "not_found");
    }
    assert_eq!(original, before);
    assert_eq!(original.as_bytes(), ORDERED);
    assert_eq!(original.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(original.records().as_ptr(), before.records().as_ptr());
    Ok(())
}

#[test]
fn malformed_value_lines_and_existing_profile_errors_do_not_produce_a_candidate() -> TestResult {
    let original = SdfDocument::parse(ORDERED.into())?;
    let before = original.clone();
    for line in ["", "line\nbreak", "line\rbreak", "$$$$", "$$$$suffix"] {
        assert!(matches!(
            original.with_field_value(RecordId::new(1).ok_or("record")?, 5, &[line]),
            Err(SdfError::InvalidValue)
        ));
    }
    for line in ["\0", "\u{000b}", "\u{007f}"] {
        assert!(matches!(
            original.with_field_value(RecordId::new(1).ok_or("record")?, 5, &[line]),
            Err(SdfError::InvalidControl { offset: 0 })
        ));
    }
    assert!(matches!(
        original.with_field_value(RecordId::new(1).ok_or("record")?, 5, &["é"]),
        Err(SdfError::UnsupportedEncoding { offset: 0 })
    ));
    let overlong = "x".repeat(201);
    assert!(matches!(
        original.with_field_value(RecordId::new(1).ok_or("record")?, 5, &[&overlong]),
        Err(SdfError::LineTooLong {
            line: 0,
            limit: 200
        })
    ));
    for length in [199, 200] {
        let value = "x".repeat(length);
        let result = original.with_field_value(RecordId::new(1).ok_or("record")?, 5, &[&value])?;
        assert_eq!(
            result.read()?.records()[0].fields()[5].value_lines(),
            [value.as_str()]
        );
    }
    assert_eq!(original, before);
    assert_eq!(original.as_bytes(), ORDERED);
    Ok(())
}

#[test]
fn empty_space_tab_and_header_looking_data_splice_only_the_value_and_keep_ids() -> TestResult {
    let original = SdfDocument::parse(ORDERED.into())?;
    let cases: &[(&[&str], &[u8])] = &[
        (&[], b""),
        (&[" "], b" \n"),
        (&["\t", "   "], b"\t\n   \n"),
        (
            &["M  END", "> <OTHER>", " $$$$"],
            b"M  END\n> <OTHER>\n $$$$\n",
        ),
    ];
    for (values, literal) in cases {
        let edited = original.with_field_value(RecordId::new(1).ok_or("record")?, 5, values)?;
        let expected = [
            ORDERED.get(..383).ok_or("prefix")?,
            *literal,
            ORDERED.get(387..).ok_or("suffix")?,
        ]
        .concat();
        assert_eq!(edited.as_bytes(), expected);
        let view = edited.read()?;
        assert_eq!(view.records()[0].fields()[5].value_lines(), *values);
        assert_eq!(view.records()[0].fields()[5].raw_value(), *literal);
        assert_eq!(view.records()[0].fields().len(), 7);
        assert_eq!(view.records().len(), 2);
        assert_eq!(
            edited
                .records()
                .iter()
                .map(|r| r.id().get())
                .collect::<Vec<_>>(),
            [1, 2]
        );
        let restored = edited.with_field_value(RecordId::new(1).ok_or("record")?, 5, &["two"])?;
        assert_eq!(restored, original);
        assert_eq!(restored.as_bytes(), ORDERED);
    }
    Ok(())
}

#[test]
fn changed_values_use_lf_while_crlf_terminator_padding_and_unterminated_eof_stay_exact()
-> TestResult {
    let crlf = std::str::from_utf8(ORDERED)?.replace('\n', "\r\n");
    assert_eq!(crlf.len(), 730);
    assert_eq!(crlf.as_bytes().get(408..413), Some(b"two\r\n".as_slice()));
    let original = SdfDocument::parse(crlf.as_bytes().into())?;
    let edited = original.with_field_value(
        RecordId::new(1).ok_or("record")?,
        5,
        &["revised", "multiline"],
    )?;
    let expected = [
        &crlf.as_bytes()[..408],
        REPLACEMENT,
        &crlf.as_bytes()[413..],
    ]
    .concat();
    assert_eq!(edited.as_bytes(), expected);
    assert_eq!(edited.as_bytes().len(), 743);
    assert_eq!(edited.as_bytes().get(426..428), Some(b"\r\n".as_slice()));
    assert_eq!(edited.as_bytes().get(468..), crlf.as_bytes().get(455..));
    let changed = edited.read()?;
    assert_eq!(changed.records()[0].fields()[5].raw_value(), REPLACEMENT);
    assert_eq!(
        changed.records()[0].fields()[2].raw_value(),
        b"alpha\r\nbeta\r\n"
    );
    assert_eq!(original.as_bytes(), crlf.as_bytes());
    let no_final_lf = ORDERED.strip_suffix(b"\n").ok_or("last LF")?;
    let original = SdfDocument::parse(no_final_lf.into())?;
    let edited = original.with_field_value(
        RecordId::new(1).ok_or("record")?,
        5,
        &["revised", "multiline"],
    )?;
    assert_eq!(
        edited.as_bytes(),
        [&no_final_lf[..383], REPLACEMENT, &no_final_lf[387..]].concat()
    );
    assert!(edited.as_bytes().ends_with(b"$$$$"));
    assert!(!edited.as_bytes().ends_with(b"\n"));
    Ok(())
}

#[test]
fn existing_global_line_and_byte_quotas_apply_to_the_complete_changed_source() -> TestResult {
    let original = SdfDocument::parse(ORDERED.into())?;
    let before = original.clone();
    assert_eq!(ORDERED.iter().filter(|byte| **byte == b'\n').count(), 40);
    let maximum_lines = vec!["x"; 999_961];
    let overflow_lines = vec!["x"; 999_962];
    for source in [ORDERED, ORDERED.strip_suffix(b"\n").ok_or("final LF")?] {
        let document = SdfDocument::parse(source.into())?;
        let boundary =
            document.with_field_value(RecordId::new(1).ok_or("record")?, 5, &maximum_lines)?;
        assert_eq!(
            boundary
                .as_bytes()
                .iter()
                .filter(|byte| **byte == b'\n')
                .count(),
            if source.ends_with(b"\n") {
                1_000_000
            } else {
                999_999
            }
        );
        assert!(matches!(
            document.with_field_value(RecordId::new(1).ok_or("record")?, 5, &overflow_lines),
            Err(SdfError::LineLimit)
        ));
        assert_eq!(document.as_bytes(), source);
    }
    let long_line = "x".repeat(200);
    let byte_overflow = vec![long_line.as_str(); 64 * 1024 * 1024 / 201];
    assert!(byte_overflow.len() * 201 < 64 * 1024 * 1024);
    assert!(byte_overflow.len() * 201 + 686 > 64 * 1024 * 1024);
    assert!(matches!(
        original.with_field_value(RecordId::new(1).ok_or("record")?, 5, &byte_overflow),
        Err(SdfError::ByteLimit)
    ));
    assert_eq!(original, before);
    assert_eq!(original.as_bytes(), ORDERED);
    Ok(())
}

#[test]
fn changed_snapshot_accounting_and_opaque_mol_authority_remain_truthful() -> TestResult {
    let opaque = std::str::from_utf8(ORDERED)?.replacen("M  END", "M  XYZ  1   1   7\nM  END", 1);
    let original = SdfDocument::parse(opaque.as_bytes().into())?;
    let edited = original.with_field_value(RecordId::new(1).ok_or("record")?, 5, &["edited"])?;
    let expected = opaque.replacen("two\n", "edited\n", 1);
    assert_eq!(edited.as_bytes(), expected.as_bytes());
    assert_ne!(edited, original);
    assert_eq!(original.as_bytes(), opaque.as_bytes());
    assert_ne!(original.as_bytes().as_ptr(), edited.as_bytes().as_ptr());
    assert_ne!(original.records().as_ptr(), edited.records().as_ptr());
    assert_eq!(
        SdfDocument::retained_bytes([&original, &edited]),
        SdfDocument::retained_bytes([&original]) + SdfDocument::retained_bytes([&edited])
    );
    let old = original.read()?;
    let new = edited.read()?;
    let transform = MolTransform::new(24.0, Point2::new(0.0, 0.0)?)?;
    assert!(
        MolFile::parse(new.records()[0].mol_bytes())?
            .to_fragment(transform)
            .is_err()
    );
    for (old, new) in old.records().iter().zip(new.records()) {
        assert_eq!(old.mol_bytes(), new.mol_bytes());
        assert_eq!(old.mol_version(), new.mol_version());
    }
    Ok(())
}

fn oracle(script: &str, source: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
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
        .arg("describe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(source)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "external SDF edit consumer: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

#[test]
fn actual_edited_owned_bytes_pass_independent_raw_and_per_molecule_consumers() -> TestResult {
    let ordered = SdfDocument::parse(ORDERED.into())?.with_field_value(
        RecordId::new(1).ok_or("record")?,
        5,
        &["revised", "multiline"],
    )?;
    let raw = oracle("sdf_oracle.py", ordered.as_bytes())?;
    assert_eq!(
        raw["source_sha256"],
        "a38749176c76066c2326853f55974f4d450c34bb03fa4f25c9adbf1b2477358b"
    );
    assert_eq!(raw["records"][0]["range"], json!([0, 438]));
    assert_eq!(raw["records"][1]["range"], json!([438, 704]));
    assert_eq!(
        raw["records"][0]["fields"][4]["value_lines"],
        json!(["one"])
    );
    assert_eq!(
        raw["records"][0]["fields"][5]["value_range"],
        json!([383, 401])
    );
    assert_eq!(
        raw["records"][0]["fields"][5]["value_lines"],
        json!(["revised", "multiline"])
    );
    assert_eq!(raw["records"][0]["fields"][3]["dt_number"], "0009");
    assert_eq!(raw["records"][0]["rdkit"]["smiles"], "CC");
    assert_eq!(raw["records"][1]["rdkit"]["smiles"], "CC");
    assert_eq!(
        raw["rdkit_records"]
            .as_array()
            .ok_or("supplier diagnostics")?
            .len(),
        1
    );
    let crlf = std::str::from_utf8(ORDERED)?.replace('\n', "\r\n");
    let crlf = SdfDocument::parse(crlf.as_bytes().into())?.with_field_value(
        RecordId::new(1).ok_or("record")?,
        5,
        &["revised", "multiline"],
    )?;
    let raw = oracle("sdf_oracle.py", crlf.as_bytes())?;
    assert_eq!(
        raw["records"][0]["fields"][5]["raw_value"],
        "revised\nmultiline\n"
    );
    assert_eq!(
        raw["records"][0]["fields"][2]["raw_value"],
        "alpha\r\nbeta\r\n"
    );
    assert_eq!(raw["records"][0]["rdkit"]["smiles"], "CC");
    assert_eq!(raw["records"][1]["rdkit"]["smiles"], "CC");
    let mixed = SdfDocument::parse(MIXED.into())?.with_field_value(
        RecordId::new(1).ok_or("record")?,
        2,
        &["revised", "multiline"],
    )?;
    let view = mixed.read()?;
    let v3 = oracle("v3000_oracle.py", view.records()[0].mol_bytes())?;
    let v2 = oracle("mol_oracle.py", view.records()[1].mol_bytes())?;
    assert_eq!(v3["source_bytes"], 344);
    assert_eq!(v3["atoms"].as_array().ok_or("V3000 atoms")?.len(), 2);
    assert_eq!(v3["bonds"].as_array().ok_or("V3000 bonds")?.len(), 1);
    assert_eq!(v2["smiles"], "CC");
    assert_eq!(v2["formula"], "C2H6");
    Ok(())
}
