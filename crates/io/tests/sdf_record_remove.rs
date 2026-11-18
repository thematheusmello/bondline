use bondline_io::{
    mol::Version,
    sdf::{MAX_SDF_LINE_BYTES, MAX_SDF_RECORDS, RecordId, RecordInfo, SdfDocument, SdfError},
};
use serde_json::{Value, json};
use std::{
    io::Write as _,
    mem::size_of,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const MIXED: &[u8] = include_bytes!("fixtures/sdf/mixed-v2000-v3000.sdf");
const ORDERED: &[u8] = include_bytes!("fixtures/sdf/ordered-fields-v2000.sdf");
const V2: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");
const V3: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");

#[test]
fn remove_first_is_exact_literal_splice_read_by_independent_raw_and_mol_consumers() -> TestResult {
    let original = SdfDocument::parse(MIXED.into())?;
    let original_pointer = original.as_bytes().as_ptr();
    let removed = original.without_record(RecordId::new(1).ok_or("record ID")?)?;

    // Literal spans of the unchanged original fixture, not spans obtained from our reader.
    assert_eq!(MIXED.len(), 777);
    assert_eq!(
        removed.as_bytes(),
        MIXED.get(462..).ok_or("literal suffix")?
    );
    assert_eq!(removed.as_bytes().len(), 315);
    assert_eq!(original.as_bytes(), MIXED);
    assert_eq!(original.as_bytes().as_ptr(), original_pointer);
    assert_eq!(original.records().len(), 2);
    assert_eq!(removed.records().len(), 1);
    let retained = removed.records().first().ok_or("retained record")?;
    assert_eq!(retained.id().get(), 1);
    assert_eq!(retained.range(), 0..315);
    assert_eq!(retained.mol_range(), 0..282);
    assert_eq!(retained.delimiter_range(), 310..315);

    let raw = oracle("sdf_oracle.py", removed.as_bytes())?;
    assert_eq!(raw["records"].as_array().ok_or("raw records")?.len(), 1);
    assert_eq!(raw["records"][0]["range"], json!([0, 315]));
    assert_eq!(raw["records"][0]["mol_range"], json!([0, 282]));
    assert_eq!(raw["records"][0]["delimiter_range"], json!([310, 315]));
    assert_eq!(raw["records"][0]["fields"][0]["name"], "ORDINARY");
    assert_eq!(
        raw["records"][0]["fields"][0]["value_lines"],
        json!(["second record"])
    );

    let view = removed.read()?;
    let mol_bytes = view.records().first().ok_or("retained MOL")?.mol_bytes();
    assert_eq!(mol_bytes, V2);
    let molecule = oracle("mol_oracle.py", mol_bytes)?;
    assert_eq!(molecule["smiles"], "CC");
    assert_eq!(molecule["formula"], "C2H6");
    assert_eq!(molecule["bonds"], json!([{"begin":0,"end":1,"order":1.0}]));
    Ok(())
}

#[test]
fn first_middle_and_final_of_three_preserve_ordered_fields_and_reindex_every_span() -> TestResult {
    let first = ORDERED.get(..424).ok_or("first literal record")?;
    let second = ORDERED.get(424..).ok_or("second literal record")?;
    let third = MIXED.get(462..).ok_or("third literal record")?;
    let source = [first, second, third].concat();
    assert_eq!(source.len(), 1005);
    let original = SdfDocument::parse(source.clone().into())?;
    let pieces = [
        (first, 282, 404, 7),
        (second, 261, 261, 0),
        (third, 282, 310, 1),
    ];

    for deleted in 0..3 {
        let id = RecordId::new(u32::try_from(deleted + 1)?).ok_or("record ID")?;
        let remaining: Vec<_> = pieces
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != deleted)
            .map(|(_, piece)| *piece)
            .collect();
        let expected = remaining
            .iter()
            .map(|piece| piece.0)
            .collect::<Vec<_>>()
            .concat();
        let candidate = original.without_record(id)?;
        assert_eq!(candidate.as_bytes(), expected);
        assert_eq!(original.as_bytes(), source);
        assert_eq!(original.records().len(), 3);
        assert_eq!(candidate.records().len(), 2);
        let independent = oracle("sdf_oracle.py", candidate.as_bytes())?;
        let records = independent["records"].as_array().ok_or("raw records")?;
        assert_eq!(records.len(), 2);
        let view = candidate.read()?;
        let mut offset = 0;
        for (index, ((bytes, mol_end, delimiter_start, fields), metadata)) in
            remaining.into_iter().zip(candidate.records()).enumerate()
        {
            let end = offset + bytes.len();
            assert_eq!(metadata.id().get(), u32::try_from(index + 1)?);
            assert_eq!(metadata.range(), offset..end);
            assert_eq!(metadata.mol_range(), offset..offset + mol_end);
            assert_eq!(metadata.delimiter_range(), offset + delimiter_start..end);
            assert_eq!(metadata.mol_version(), Version::V2000);
            assert_eq!(metadata.field_count(), fields);
            assert_eq!(candidate.record(metadata.id()), Some(metadata));
            assert_eq!(records[index]["range"], json!([offset, end]));
            assert_eq!(
                records[index]["mol_range"],
                json!([offset, offset + mol_end])
            );
            assert_eq!(
                records[index]["delimiter_range"],
                json!([offset + delimiter_start, end])
            );
            assert_eq!(records[index]["rdkit"]["smiles"], "CC");
            assert_eq!(
                records[index]["rdkit"]["atoms"]
                    .as_array()
                    .ok_or("atoms")?
                    .len(),
                2
            );
            assert_eq!(
                records[index]["rdkit"]["bonds"],
                json!([{"begin":0,"end":1,"order":1.0}])
            );
            let record = view.records().get(index).ok_or("retained record")?;
            assert_eq!(
                record.mol_bytes(),
                bytes.get(..mol_end).ok_or("literal MOL")?
            );
            if fields == 7 {
                // Duplicate occurrences, raw DT digits, empty/multiline/spaces and
                // the prefix delimiter all survive, without a property dictionary.
                let fields = &records[index]["fields"];
                assert_eq!(fields[1]["value_lines"], json!([]));
                assert_eq!(fields[2]["dt_number"], "12");
                assert_eq!(fields[2]["value_lines"], json!(["alpha", "beta"]));
                assert_eq!(fields[3]["dt_number"], "0009");
                assert_eq!(fields[4]["name"], "DUP");
                assert_eq!(fields[4]["value_lines"], json!(["one"]));
                assert_eq!(fields[5]["name"], "DUP");
                assert_eq!(fields[5]["value_lines"], json!(["two"]));
                assert_eq!(fields[6]["value_lines"], json!(["   "]));
                assert_eq!(
                    candidate.as_bytes().get(metadata.delimiter_range()),
                    Some(b"$$$$BONDLINE_RECORD\n".as_slice())
                );
            }
            offset = end;
        }
        assert_eq!(offset, candidate.as_bytes().len());
        assert!(
            candidate
                .record(RecordId::new(3).ok_or("missing ID")?)
                .is_none()
        );
    }
    Ok(())
}

#[test]
fn removing_final_mixed_record_preserves_the_actual_v3000_molecule_and_data() -> TestResult {
    let original = SdfDocument::parse(MIXED.into())?;
    let candidate = original.without_record(RecordId::new(2).ok_or("record ID")?)?;
    assert_eq!(
        candidate.as_bytes(),
        MIXED.get(..462).ok_or("literal prefix")?
    );
    assert_eq!(original.as_bytes(), MIXED);
    let metadata = candidate.records().first().ok_or("V3000 record")?;
    assert_eq!(metadata.id().get(), 1);
    assert_eq!(metadata.range(), 0..462);
    assert_eq!(metadata.mol_range(), 0..344);
    assert_eq!(metadata.delimiter_range(), 457..462);
    assert_eq!(metadata.mol_version(), Version::V3000);
    assert_eq!(metadata.field_count(), 6);
    let view = candidate.read()?;
    let record = view.records().first().ok_or("retained record")?;
    assert_eq!(record.mol_bytes(), V3);
    let independent = oracle("v3000_oracle.py", record.mol_bytes())?;
    assert_eq!(independent["source_bytes"], 344);
    assert_eq!(
        independent["atoms"].as_array().ok_or("V3000 atoms")?.len(),
        2
    );
    assert_eq!(independent["atoms"][0]["element"], 6);
    assert_eq!(
        independent["atoms"][0]["position"],
        json!([-1.25, 0.5, 0.0])
    );
    assert_eq!(independent["atoms"][1]["position"], json!([0.25, 0.5, 0.0]));
    assert_eq!(independent["bonds"][0]["begin"], 0);
    assert_eq!(independent["bonds"][0]["end"], 1);
    assert_eq!(independent["bonds"][0]["order"], 1.0);
    assert_eq!(record.fields()[1].value_lines(), ["first"]);
    assert_eq!(record.fields()[2].value_lines(), ["second"]);
    assert_eq!(record.fields()[3].dt_number(), Some("0009"));
    assert_eq!(record.fields()[4].dt_number(), Some("0012"));
    assert_eq!(record.fields()[5].raw_value(), b"   \n");
    Ok(())
}

#[test]
fn crlf_prefix_delimiter_blank_title_and_final_eof_are_retained_exactly() -> TestResult {
    let crlf = std::str::from_utf8(ORDERED)?.replace('\n', "\r\n");
    let source = crlf.as_bytes().get(..728).ok_or("final EOF vector")?;
    assert_eq!(crlf.len(), 730);
    assert!(source.ends_with(b"$$$$"));
    let original = SdfDocument::parse(source.into())?;
    for (id, expected, mol_end, delimiter_start) in [
        (1, source.get(455..).ok_or("CRLF suffix")?, 269, 269),
        (2, source.get(..455).ok_or("CRLF prefix")?, 290, 434),
    ] {
        let candidate = original.without_record(RecordId::new(id).ok_or("record ID")?)?;
        assert_eq!(candidate.as_bytes(), expected);
        assert_eq!(original.as_bytes(), source);
        let metadata = candidate.records().first().ok_or("retained record")?;
        assert_eq!(metadata.id().get(), 1);
        assert_eq!(metadata.range(), 0..expected.len());
        assert_eq!(metadata.mol_range(), 0..mol_end);
        assert_eq!(metadata.delimiter_range(), delimiter_start..expected.len());
        let independent = oracle("sdf_oracle.py", candidate.as_bytes())?;
        assert_eq!(
            independent["records"][0]["range"],
            json!([0, expected.len()])
        );
        assert_eq!(independent["records"][0]["mol_range"], json!([0, mol_end]));
        assert_eq!(independent["records"][0]["rdkit"]["smiles"], "CC");
        if id == 1 {
            assert!(candidate.as_bytes().starts_with(b"\r\n"));
            assert!(!candidate.as_bytes().ends_with(b"\n"));
        } else {
            assert!(candidate.as_bytes().ends_with(b"$$$$BONDLINE_RECORD\r\n"));
            assert_eq!(
                independent["records"][0]["fields"][2]["raw_value"],
                "alpha\r\nbeta\r\n"
            );
        }
    }
    Ok(())
}

#[test]
fn missing_target_precedes_last_record_refusal_and_never_mutates_the_owner() -> TestResult {
    assert!(RecordId::new(0).is_none());
    let multiple = SdfDocument::parse(MIXED.into())?;
    let sole = multiple.without_record(RecordId::new(1).ok_or("record ID")?)?;
    for document in [&multiple, &sole] {
        let snapshot = document.clone();
        let source_pointer = document.as_bytes().as_ptr();
        let index_pointer = document.records().as_ptr();
        for id in [3, u32::MAX] {
            let error = document
                .without_record(RecordId::new(id).ok_or("record ID")?)
                .err()
                .ok_or("missing record accepted")?;
            assert!(matches!(error, SdfError::MissingRecord { record_id } if record_id == id));
            assert_eq!(error.code(), "not_found");
        }
        assert_eq!(document, &snapshot);
        assert_eq!(document.as_bytes().as_ptr(), source_pointer);
        assert_eq!(document.records().as_ptr(), index_pointer);
    }
    let error = sole
        .without_record(RecordId::new(1).ok_or("record ID")?)
        .err()
        .ok_or("last record accepted")?;
    assert!(matches!(error, SdfError::Empty));
    assert_eq!(error.code(), "invalid_sdf");
    assert_eq!(sole.as_bytes(), MIXED.get(462..).ok_or("retained source")?);
    assert_eq!(multiple.as_bytes(), MIXED);
    Ok(())
}

#[test]
fn candidate_owns_only_its_new_source_and_index_and_shares_them_with_its_clones() -> TestResult {
    let source: Arc<[u8]> = ORDERED.into();
    let weak = Arc::downgrade(&source);
    let original = SdfDocument::parse(source)?;
    let candidate = original.without_record(RecordId::new(2).ok_or("record ID")?)?;
    let clone = candidate.clone();
    assert_ne!(original.as_bytes().as_ptr(), candidate.as_bytes().as_ptr());
    assert_ne!(original.records().as_ptr(), candidate.records().as_ptr());
    assert_eq!(candidate.as_bytes().as_ptr(), clone.as_bytes().as_ptr());
    assert_eq!(candidate.records().as_ptr(), clone.records().as_ptr());
    assert_eq!(candidate, clone);
    let original_bytes = size_of::<SdfDocument>() + 690 + 2 * size_of::<RecordInfo>();
    let candidate_bytes = size_of::<SdfDocument>() + 424 + size_of::<RecordInfo>();
    assert_eq!(SdfDocument::retained_bytes([&original]), original_bytes);
    assert_eq!(SdfDocument::retained_bytes([&candidate]), candidate_bytes);
    assert_eq!(
        SdfDocument::retained_bytes([&original, &candidate, &clone, &clone]),
        original_bytes + candidate_bytes + size_of::<SdfDocument>()
    );
    drop(original);
    assert!(
        weak.upgrade().is_none(),
        "candidate must not retain deleted source bytes"
    );
    drop(candidate);
    assert_eq!(
        clone.as_bytes(),
        ORDERED.get(..424).ok_or("literal prefix")?
    );
    assert_eq!(clone.read()?.records().len(), 1);
    Ok(())
}

#[test]
fn removal_at_existing_record_and_delimiter_line_limits_does_not_expand_the_source() -> TestResult {
    let plain = [V2, b"$$$$\n"].concat();
    let source = plain.repeat(MAX_SDF_RECORDS);
    let original = SdfDocument::parse(source.clone().into())?;
    assert_eq!(original.records().len(), MAX_SDF_RECORDS);
    for id in [1, MAX_SDF_RECORDS / 2, MAX_SDF_RECORDS] {
        let candidate =
            original.without_record(RecordId::new(u32::try_from(id)?).ok_or("record ID")?)?;
        assert_eq!(candidate.as_bytes(), plain.repeat(MAX_SDF_RECORDS - 1));
        assert_eq!(candidate.records().len(), MAX_SDF_RECORDS - 1);
        assert_eq!(
            candidate.records().last().ok_or("last record")?.id().get(),
            1023
        );
        assert_eq!(original.as_bytes(), source);
    }
    let mut maximal_delimiter = b"$$$$".to_vec();
    maximal_delimiter.resize(MAX_SDF_LINE_BYTES, b'x');
    let last = [V2, maximal_delimiter.as_slice()].concat();
    let source = [plain.as_slice(), last.as_slice()].concat();
    let document = SdfDocument::parse(source.clone().into())?;
    let candidate = document.without_record(RecordId::new(1).ok_or("record ID")?)?;
    assert_eq!(candidate.as_bytes(), last);
    assert_eq!(
        candidate.records()[0].delimiter_range(),
        282..282 + MAX_SDF_LINE_BYTES
    );
    assert_eq!(document.as_bytes(), source);
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
        "independent SDF-removal consumer: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}
