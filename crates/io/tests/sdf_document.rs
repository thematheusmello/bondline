use bondline_io::{
    mol::{MolFile, MolTransform, Version},
    sdf::{RecordId, RecordInfo, SdfDocument, SdfError},
};
use bondline_model::Point2;
use serde_json::{Value, json};
use std::{
    io::Write as _,
    mem::size_of,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORDERED: &[u8] = include_bytes!("fixtures/sdf/ordered-fields-v2000.sdf");
const MIXED: &[u8] = include_bytes!("fixtures/sdf/mixed-v2000-v3000.sdf");
const V2: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");
const V3: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");

#[test]
fn owned_sources_keep_both_original_records_versions_fields_and_literal_spans() -> TestResult {
    for (bytes, expected) in [
        (
            ORDERED,
            [
                (0..424, 0..282, 404..424, Version::V2000, 7),
                (424..690, 424..685, 685..690, Version::V2000, 0),
            ],
        ),
        (
            MIXED,
            [
                (0..462, 0..344, 457..462, Version::V3000, 6),
                (462..777, 462..744, 772..777, Version::V2000, 1),
            ],
        ),
    ] {
        let (document, source_pointer) = {
            let caller_buffer = bytes.to_vec();
            let source: Arc<[u8]> = caller_buffer.into();
            let source_pointer = source.as_ptr();
            (SdfDocument::parse(source)?, source_pointer)
        };
        assert_eq!(document.as_bytes(), bytes);
        assert_eq!(document.as_bytes().as_ptr(), source_pointer);
        assert_eq!(document.records().len(), 2);
        let view = document.read()?;
        assert_eq!(view.as_bytes().as_ptr(), document.as_bytes().as_ptr());
        for (index, ((range, mol_range, delimiter_range, version, fields), metadata)) in
            expected.into_iter().zip(document.records()).enumerate()
        {
            let id = RecordId::new(u32::try_from(index + 1)?).ok_or("record ID")?;
            assert_eq!(metadata.id(), id);
            assert_eq!(metadata.id().get(), u32::try_from(index + 1)?);
            assert_eq!(document.record(id), Some(metadata));
            assert_eq!(metadata.range(), range.clone());
            assert_eq!(metadata.mol_range(), mol_range.clone());
            assert_eq!(metadata.delimiter_range(), delimiter_range.clone());
            assert_eq!(metadata.mol_version(), version);
            assert_eq!(metadata.field_count(), fields);
            let record = view.records().get(index).ok_or("borrowed record")?;
            assert_eq!(record.range(), range);
            assert_eq!(record.mol_range(), mol_range);
            assert_eq!(record.delimiter_range(), delimiter_range);
            assert_eq!(record.mol_version(), version);
            assert_eq!(record.fields().len(), fields);
            assert_eq!(
                bytes.get(record.mol_range()).ok_or("literal MOL span")?,
                record.mol_bytes()
            );
            assert_eq!(
                record.mol_bytes(),
                if version == Version::V3000 {
                    V3
                } else if index == 0 || bytes == MIXED {
                    V2
                } else {
                    V2.get(21..).ok_or("blank-title MOL")?
                }
            );
        }
        let first = view.records().first().ok_or("first record")?;
        let field_spans = if bytes == ORDERED {
            vec![
                (282..296, 282..289, 289..295),
                (296..307, 296..306, 306..306),
                (308..343, 308..331, 331..342),
                (343..362, 343..352, 352..361),
                (362..375, 362..370, 370..374),
                (375..388, 375..383, 383..387),
                (388..404, 388..399, 399..403),
            ]
        } else {
            vec![
                (344..355, 344..354, 354..354),
                (355..370, 355..363, 363..369),
                (370..386, 370..378, 378..385),
                (386..427, 386..411, 411..426),
                (427..441, 427..436, 436..440),
                (441..457, 441..452, 452..456),
            ]
        };
        for (field, (whole, header, value)) in first.fields().iter().zip(field_spans) {
            assert_eq!(field.range(), whole);
            assert_eq!(field.header_range(), header);
            assert_eq!(field.value_range(), value.clone());
            assert_eq!(
                field.raw_value(),
                bytes.get(value).ok_or("literal value span")?
            );
        }
        if bytes == ORDERED {
            assert_eq!(first.fields()[0].value_lines(), ["first"]);
            assert_eq!(first.fields()[1].value_lines(), [] as [&str; 0]);
            assert_eq!(first.fields()[2].value_lines(), ["alpha", "beta"]);
            assert_eq!(first.fields()[2].dt_number(), Some("12"));
            assert_eq!(first.fields()[3].name(), None);
            assert_eq!(first.fields()[3].dt_number(), Some("0009"));
            assert_eq!(first.fields()[4].name(), Some("DUP"));
            assert_eq!(first.fields()[4].value_lines(), ["one"]);
            assert_eq!(first.fields()[5].name(), Some("DUP"));
            assert_eq!(first.fields()[5].value_lines(), ["two"]);
            assert_eq!(first.fields()[6].value_lines(), ["   "]);
            assert_eq!(bytes.get(306..308), Some(b"\n\n".as_slice()));
        } else {
            assert_eq!(first.fields()[0].value_lines(), [] as [&str; 0]);
            assert_eq!(first.fields()[1].name(), Some("DUP"));
            assert_eq!(first.fields()[1].value_lines(), ["first"]);
            assert_eq!(first.fields()[2].name(), Some("DUP"));
            assert_eq!(first.fields()[2].value_lines(), ["second"]);
            assert_eq!(first.fields()[3].value_lines(), ["carbon", "isotope"]);
            assert_eq!(first.fields()[3].dt_number(), Some("0009"));
            assert_eq!(first.fields()[4].name(), None);
            assert_eq!(first.fields()[4].dt_number(), Some("0012"));
            assert_eq!(first.fields()[5].value_lines(), ["   "]);
            assert_eq!(
                view.records()[1].fields()[0].value_lines(),
                ["second record"]
            );
            assert_eq!(view.records()[1].fields()[0].range(), 744..772);
            assert_eq!(view.records()[1].fields()[0].header_range(), 744..757);
            assert_eq!(view.records()[1].fields()[0].value_range(), 757..771);
        }
    }
    Ok(())
}

#[test]
fn clones_share_source_and_index_while_accounting_counts_distinct_equal_allocations() -> TestResult
{
    let source: Arc<[u8]> = ORDERED.into();
    let first = SdfDocument::parse(Arc::clone(&source))?;
    let clone = first.clone();
    let shared_source = SdfDocument::parse(source)?;
    let separate = SdfDocument::parse(ORDERED.into())?;
    assert_eq!(first, clone);
    assert_eq!(first, shared_source);
    assert_eq!(first, separate);
    assert_eq!(first.as_bytes().as_ptr(), clone.as_bytes().as_ptr());
    assert_eq!(first.records().as_ptr(), clone.records().as_ptr());
    assert_ne!(first.records().as_ptr(), shared_source.records().as_ptr());
    assert_ne!(first.as_bytes().as_ptr(), separate.as_bytes().as_ptr());
    let index_bytes = 2 * size_of::<RecordInfo>();
    let once = size_of::<SdfDocument>() + ORDERED.len() + index_bytes;
    assert_eq!(SdfDocument::retained_bytes(std::iter::empty()), 0);
    assert_eq!(SdfDocument::retained_bytes([&first]), once);
    assert_eq!(SdfDocument::retained_bytes([&first, &first]), once);
    assert_eq!(
        SdfDocument::retained_bytes([&first, &clone]),
        once + size_of::<SdfDocument>()
    );
    assert_eq!(
        SdfDocument::retained_bytes([&first, &shared_source]),
        once + size_of::<SdfDocument>() + index_bytes
    );
    assert_eq!(SdfDocument::retained_bytes([&first, &separate]), 2 * once);
    drop(first);
    assert_eq!(clone.as_bytes(), ORDERED);
    assert_eq!(clone.read()?.records().len(), 2);
    Ok(())
}

#[test]
fn complete_source_affects_equality_and_debug_exposes_only_counts() -> TestResult {
    let first = SdfDocument::parse(ORDERED.into())?;
    let changed = std::str::from_utf8(ORDERED)?.replacen("one\n", "ONE\n", 1);
    assert_eq!(changed.len(), ORDERED.len());
    let changed = SdfDocument::parse(changed.into_bytes().into())?;
    assert_eq!(first.records(), changed.records());
    assert_ne!(
        first, changed,
        "an unchanged molecular/index view cannot erase data-only differences"
    );
    assert_eq!(
        format!("{first:?}"),
        "SdfDocument { byte_length: 690, record_count: 2 }"
    );
    assert_eq!(
        format!("{changed:?}"),
        "SdfDocument { byte_length: 690, record_count: 2 }"
    );
    Ok(())
}

#[test]
fn record_identity_is_positive_bounded_and_stable_without_using_titles_or_fields() -> TestResult {
    assert!(RecordId::new(0).is_none());
    let record_bytes = [V2, b"$$$$\n"].concat();
    let maximum = SdfDocument::parse(record_bytes.repeat(1024).into())?;
    assert_eq!(maximum.records().len(), 1024);
    for (index, record) in maximum.records().iter().enumerate() {
        assert_eq!(record.id().get(), u32::try_from(index + 1)?);
        assert_eq!(maximum.record(record.id()), Some(record));
    }
    let last = RecordId::new(1024).ok_or("last ID")?;
    assert_eq!(maximum.record(last), maximum.records().last());
    assert!(
        maximum
            .record(RecordId::new(1025).ok_or("missing ID")?)
            .is_none()
    );
    assert!(
        maximum
            .record(RecordId::new(u32::MAX).ok_or("max ID")?)
            .is_none()
    );
    let clone = maximum.clone();
    assert_eq!(clone.record(last), maximum.record(last));
    assert!(matches!(
        SdfDocument::parse(record_bytes.repeat(1025).into()),
        Err(SdfError::RecordLimit)
    ));
    Ok(())
}

#[test]
fn malformed_later_records_and_unsupported_encoding_refuse_the_whole_constructor() -> TestResult {
    let first = [V2, b"$$$$\n"].concat();
    assert!(matches!(
        SdfDocument::parse(Arc::from([])),
        Err(SdfError::Empty)
    ));
    assert!(matches!(
        SdfDocument::parse(
            [first.as_slice(), b"unfinished next record\n"]
                .concat()
                .into()
        ),
        Err(SdfError::MissingMolEnd { record: 2 })
    ));
    let bad_v2 = std::str::from_utf8(V2)?.replacen("  2  1", "  3  1", 1);
    assert!(matches!(
        SdfDocument::parse(
            [first.as_slice(), bad_v2.as_bytes(), b"$$$$\n"]
                .concat()
                .into()
        ),
        Err(SdfError::Mol { record: 2, .. })
    ));
    let bad_v3 = std::str::from_utf8(V3)?.replacen("COUNTS 2 1", "COUNTS 3 1", 1);
    assert!(matches!(
        SdfDocument::parse(
            [first.as_slice(), bad_v3.as_bytes(), b"$$$$\n"]
                .concat()
                .into()
        ),
        Err(SdfError::MolV3000 { record: 2, .. })
    ));
    assert!(matches!(
        SdfDocument::parse(
            [first.as_slice(), V2, b"> <VALUE>\nunfinished\n"]
                .concat()
                .into()
        ),
        Err(SdfError::MissingFieldTerminator { .. })
    ));
    let encoding_offset = first.len() + V2.len() + b"> <VALUE>\n".len();
    for (byte, unsupported) in [(0xff, true), (0, false)] {
        let source = [first.as_slice(), V2, b"> <VALUE>\n", &[byte], b"\n\n$$$$\n"].concat();
        let error = SdfDocument::parse(source.into())
            .err()
            .ok_or("invalid encoding")?;
        if unsupported {
            assert!(
                matches!(error, SdfError::UnsupportedEncoding { offset } if offset == encoding_offset)
            );
        } else {
            assert!(
                matches!(error, SdfError::InvalidControl { offset } if offset == encoding_offset)
            );
        }
    }
    assert_eq!(first, [V2, b"$$$$\n"].concat());
    Ok(())
}

#[test]
fn whole_source_byte_field_line_and_value_limits_are_enforced_before_success() -> TestResult {
    assert!(matches!(
        SdfDocument::parse(vec![b' '; 64 * 1024 * 1024 + 1].into()),
        Err(SdfError::ByteLimit)
    ));
    let fields = b"> <A>\nx\n\n".repeat(65_537);
    assert!(matches!(
        SdfDocument::parse([V2, fields.as_slice(), b"$$$$\n"].concat().into()),
        Err(SdfError::FieldLimit)
    ));
    let lines = b"x\n".repeat(1_000_001);
    assert!(matches!(
        SdfDocument::parse(
            [V2, b"> <A>\n", lines.as_slice(), b"\n$$$$\n"]
                .concat()
                .into()
        ),
        Err(SdfError::LineLimit)
    ));
    for (line, limit) in [
        (format!("> <A>{}\n", "x".repeat(4092)), 4096),
        (format!("> <A>\n{}\n", "x".repeat(201)), 200),
    ] {
        let error = SdfDocument::parse([V2, line.as_bytes(), b"\n$$$$\n"].concat().into())
            .err()
            .ok_or("overlong line")?;
        assert!(matches!(error, SdfError::LineTooLong { limit: actual, .. } if actual == limit));
    }
    Ok(())
}

#[test]
fn crlf_delimiter_tails_padding_tabs_and_empty_titles_are_never_reframed() -> TestResult {
    let source = std::str::from_utf8(ORDERED)?.replace('\n', "\r\n");
    let original = source.as_bytes().to_vec();
    let document = SdfDocument::parse(source.into_bytes().into())?;
    assert_eq!(document.as_bytes(), original);
    let view = document.read()?;
    assert_eq!(view.records().len(), 2);
    assert_eq!(
        view.records()[0].fields()[2].raw_value(),
        b"alpha\r\nbeta\r\n"
    );
    assert_eq!(
        document.as_bytes().get(view.records()[0].delimiter_range()),
        Some(b"$$$$BONDLINE_RECORD\r\n".as_slice())
    );
    assert!(view.records()[1].mol_bytes().starts_with(b"\r\n"));
    let tabbed = [V2, b"> <VALUE>\n\t\n   \n\n\n$$$$original_tail"].concat();
    let document = SdfDocument::parse(tabbed.clone().into())?;
    assert_eq!(document.as_bytes(), tabbed);
    let view = document.read()?;
    assert_eq!(view.records()[0].fields()[0].value_lines(), ["\t", "   "]);
    assert_eq!(view.records()[0].fields()[0].raw_value(), b"\t\n   \n");
    Ok(())
}

#[test]
fn valid_opaque_mol_content_survives_even_when_elemental_projection_refuses() -> TestResult {
    let opaque = std::str::from_utf8(V2)?.replace("M  END", "M  XYZ  1   1   7\nM  END");
    let source = [opaque.as_bytes(), b"$$$$\n", V2, b"$$$$\n"].concat();
    let document = SdfDocument::parse(source.clone().into())?;
    assert_eq!(document.as_bytes(), source);
    let view = document.read()?;
    assert_eq!(view.records().len(), 2);
    assert_eq!(view.records()[0].mol_bytes(), opaque.as_bytes());
    let transform = MolTransform::new(24.0, Point2::new(0.0, 0.0)?)?;
    assert!(
        MolFile::parse(view.records()[0].mol_bytes())?
            .to_fragment(transform)
            .is_err()
    );
    assert_eq!(
        MolFile::parse(view.records()[1].mol_bytes())?
            .to_fragment(transform)?
            .atoms()
            .len(),
        2
    );
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
        "external owned-SDF oracle: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

fn assert_independent_raw_spans(document: &SdfDocument, independent: &Value) -> TestResult {
    let view = document.read()?;
    let records = independent["records"]
        .as_array()
        .ok_or("independent records")?;
    assert_eq!(view.records().len(), records.len());
    for (record, external) in view.records().iter().zip(records) {
        for (key, range) in [
            ("range", record.range()),
            ("mol_range", record.mol_range()),
            ("delimiter_range", record.delimiter_range()),
        ] {
            assert_eq!(external[key], json!([range.start, range.end]));
        }
        let fields = external["fields"].as_array().ok_or("independent fields")?;
        assert_eq!(record.fields().len(), fields.len());
        for (field, external) in record.fields().iter().zip(fields) {
            for (key, range) in [
                ("range", field.range()),
                ("header_range", field.header_range()),
                ("value_range", field.value_range()),
            ] {
                assert_eq!(external[key], json!([range.start, range.end]));
            }
            assert_eq!(external["header"], json!(field.header()));
            assert_eq!(external["name"], json!(field.name()));
            assert_eq!(external["dt_number"], json!(field.dt_number()));
            assert_eq!(
                external["raw_value"],
                std::str::from_utf8(field.raw_value())?
            );
            assert_eq!(external["value_lines"], json!(field.value_lines()));
        }
    }
    Ok(())
}

#[test]
fn independent_unchanged_raw_and_molecular_consumers_read_the_actual_owned_source() -> TestResult {
    let ordered = SdfDocument::parse(ORDERED.into())?;
    let raw = oracle("sdf_oracle.py", ordered.as_bytes())?;
    assert_independent_raw_spans(&ordered, &raw)?;
    assert_eq!(
        raw["source_sha256"],
        "f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a"
    );
    assert_eq!(raw["records"][0]["range"], json!([0, 424]));
    assert_eq!(raw["records"][1]["range"], json!([424, 690]));
    assert_eq!(raw["records"][0]["fields"][3]["dt_number"], "0009");
    assert_eq!(
        raw["records"][0]["fields"][4]["value_lines"],
        json!(["one"])
    );
    assert_eq!(
        raw["records"][0]["fields"][5]["value_lines"],
        json!(["two"])
    );
    assert_eq!(raw["records"][0]["rdkit"]["smiles"], "CC");
    assert_eq!(raw["records"][1]["rdkit"]["smiles"], "CC");
    assert_eq!(
        raw["rdkit_records"]
            .as_array()
            .ok_or("supplier diagnostics")?
            .len(),
        1,
        "the original DT-only full-supplier loss must stay visible, never define source fidelity"
    );
    let crlf = std::str::from_utf8(ORDERED)?.replace('\n', "\r\n");
    let owned_crlf = SdfDocument::parse(crlf.as_bytes().into())?;
    let raw_crlf = oracle("sdf_oracle.py", owned_crlf.as_bytes())?;
    assert_independent_raw_spans(&owned_crlf, &raw_crlf)?;
    assert_eq!(
        raw_crlf["records"].as_array().ok_or("CRLF records")?.len(),
        2
    );
    assert_eq!(
        raw_crlf["records"][0]["fields"][2]["raw_value"],
        "alpha\r\nbeta\r\n"
    );
    let mixed = SdfDocument::parse(MIXED.into())?;
    let view = mixed.read()?;
    let v3 = oracle("v3000_oracle.py", view.records()[0].mol_bytes())?;
    let v2 = oracle("mol_oracle.py", view.records()[1].mol_bytes())?;
    assert_eq!(v3["source_bytes"], 344);
    assert_eq!(v3["atoms"].as_array().ok_or("V3000 atoms")?.len(), 2);
    assert_eq!(v3["bonds"].as_array().ok_or("V3000 bonds")?.len(), 1);
    assert_eq!(v2["smiles"], "CC");
    assert_eq!(v2["formula"], "C2H6");
    assert_eq!(v2["bonds"], json!([{"begin":0,"end":1,"order":1.0}]));
    Ok(())
}
