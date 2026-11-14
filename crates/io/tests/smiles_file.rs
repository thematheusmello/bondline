use bondline_io::smiles::{
    SmilesDocument, SmilesError, SmilesFileDocument, SmilesFileError,
    file::{self, RecordId, RecordInfo},
};
use bondline_render::Cancellation;
use serde_json::{Value, json};
use std::{
    fs,
    io::Write as _,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
#[allow(dead_code)]
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;
fn id(value: u32) -> Result<RecordId, Box<dyn std::error::Error>> {
    RecordId::new(value).ok_or_else(|| "record ID".into())
}

#[test]
fn original_raw_spans_retain_ignored_bytes_opaque_data_and_every_record() -> TestResult {
    let raw: Arc<[u8]> =
        Arc::from(b"\n \tignored \xff\nCC\t \tname \0\xff\r\nC=C \r\nN".as_slice());
    let input = Arc::clone(&raw);
    let file = SmilesFileDocument::parse(input)?;
    assert_eq!(file.as_bytes(), raw.as_ref());
    assert_eq!(file.as_bytes().as_ptr(), raw.as_ptr());
    assert_eq!(file.records().len(), 3);
    for (ordinal, line, full, notation, separator, data, ending, atoms, bonds) in [
        (
            1,
            3,
            13..27,
            13..15,
            Some(15..18),
            Some(18..25),
            25..27,
            2,
            1,
        ),
        (
            2,
            4,
            27..33,
            27..30,
            Some(30..31),
            Some(31..31),
            31..33,
            2,
            1,
        ),
        (3, 5, 33..34, 33..34, None, None, 34..34, 1, 0),
    ] {
        let record = file.record(id(ordinal)?).ok_or("missing record")?;
        assert_eq!(record.id().get(), ordinal);
        assert_eq!(record.line_number(), line);
        assert_eq!(record.full_range(), full);
        assert_eq!(record.notation_range(), notation.clone());
        assert_eq!(record.separator_range(), separator);
        assert_eq!(record.application_data_range(), data);
        assert_eq!(record.line_ending_range(), ending);
        assert_eq!(record.atom_count(), atoms);
        assert_eq!(record.bond_count(), bonds);
        let graph = file.read_record(id(ordinal)?)?;
        assert_eq!(graph.as_bytes(), raw.get(notation).ok_or("source span")?);
        assert_eq!(graph.atoms().len(), atoms);
        assert_eq!(graph.bonds().len(), bonds);
        assert_eq!(graph.atoms()[0].range.start, 0);
    }
    assert!(RecordId::new(0).is_none());
    assert!(file.record(id(4)?).is_none());
    Ok(())
}

#[test]
fn empty_ignored_prefixes_eof_and_cr_boundaries_have_no_synthetic_graph() -> TestResult {
    for raw in [
        b"".as_slice(),
        b"\n\r\n \xff\0 ignored\n\tC(\n\rCC\rmetadata",
    ] {
        let file = SmilesFileDocument::parse(raw.into())?;
        assert_eq!(file.as_bytes(), raw);
        assert!(file.records().is_empty());
        assert!(file.record(id(1)?).is_none());
        let error = file
            .read_record(id(1)?)
            .err()
            .ok_or("fabricated empty graph")?;
        assert_eq!(error.code(), "not_found");
        assert_eq!(error.record_id(), Some(id(1)?));
        assert_eq!(error.line_number(), None);
        assert_eq!(error.offset(), None);
    }
    for (raw, data, ending) in [
        (b"CC\r\n".as_slice(), None, 2..4),
        (b"CC\n".as_slice(), None, 2..3),
        (b"CC".as_slice(), None, 2..2),
        (b"CC name\r".as_slice(), Some(3..8), 8..8),
        (b"CC name\r\r\n".as_slice(), Some(3..8), 8..10),
    ] {
        let file = SmilesFileDocument::parse(raw.into())?;
        assert_eq!(file.records().len(), 1);
        let first = file.record(id(1)?).ok_or("first")?;
        assert_eq!(first.notation_range(), 0..2);
        assert_eq!(first.application_data_range(), data);
        assert_eq!(first.line_ending_range(), ending);
        assert_eq!(file.as_bytes(), raw);
    }
    Ok(())
}

#[test]
fn any_invalid_or_unsupported_later_notation_rejects_the_whole_file_with_absolute_context()
-> TestResult {
    for (raw, line, record, offset, code) in [
        (b"CC\nC(\nCO\n".as_slice(), 2, 2, 5, "invalid_smiles"),
        (b"CC\nC(C\nCO\n".as_slice(), 2, 2, 4, "invalid_smiles"),
        (
            b"CC\nC1 SECRET_ERROR_DATA_4a\nCO\n".as_slice(),
            2,
            2,
            4,
            "invalid_smiles",
        ),
        (
            b"CC\n* query\nN\n".as_slice(),
            2,
            2,
            3,
            "unsupported_content",
        ),
        (
            b"CC\n[C@H](F)(Cl)Br stereo\nN\n".as_slice(),
            2,
            2,
            5,
            "unsupported_content",
        ),
        (b"CC\r".as_slice(), 1, 1, 2, "invalid_smiles"),
        (b"C\rC\n".as_slice(), 1, 1, 1, "invalid_smiles"),
        (b"C\0C\n".as_slice(), 1, 1, 1, "invalid_smiles"),
        (b"\x0bCC\n".as_slice(), 1, 1, 0, "invalid_smiles"),
        (b"\x0cCC\n".as_slice(), 1, 1, 0, "invalid_smiles"),
        (
            b"\xef\xbb\xbfCC\n".as_slice(),
            1,
            1,
            0,
            "unsupported_content",
        ),
        (b"\xc2\xa0CC\n".as_slice(), 1, 1, 0, "unsupported_content"),
        (b"#comment\nCC\n".as_slice(), 1, 1, 0, "invalid_smiles"),
    ] {
        let original: Arc<[u8]> = raw.into();
        let error = SmilesFileDocument::parse(Arc::clone(&original))
            .err()
            .ok_or("partial file accepted")?;
        assert_eq!(error.code(), code, "{error}");
        assert_eq!(error.line_number(), Some(line));
        assert_eq!(error.record_id(), Some(id(record)?));
        assert_eq!(error.offset(), Some(offset));
        assert_eq!(original.as_ref(), raw);
        assert!(!format!("{error:?} {error}").contains("SECRET_ERROR_DATA_4a"));
        assert!(matches!(error, SmilesFileError::Notation { .. }));
    }
    let aromatic =
        SmilesFileDocument::parse(b"CC first\nc1ccccc1 second\nC:C third\n".as_slice().into())?;
    assert_eq!(aromatic.records().len(), 3);
    assert_eq!(aromatic.records()[1].atom_count(), 6);
    assert_eq!(aromatic.records()[1].bond_count(), 6);
    Ok(())
}

#[test]
fn notation_limit_is_independent_of_file_framing_and_opaque_metadata_length() -> TestResult {
    let notation = format!("[{}C]", "0".repeat(65_533));
    assert_eq!(notation.len(), 65_536);
    for ending in ["", "\n", "\r\n"] {
        let raw = format!("{notation}{ending}");
        let file = SmilesFileDocument::parse(Arc::from(raw.as_bytes()))?;
        assert_eq!(file.as_bytes(), raw.as_bytes());
        assert_eq!(file.records()[0].notation_range(), 0..65_536);
        assert_eq!(file.records()[0].line_ending_range(), 65_536..raw.len());
        assert_eq!(file.read_record(id(1)?)?.as_bytes(), notation.as_bytes());
        if !ending.is_empty() {
            assert!(matches!(
                SmilesDocument::parse(Arc::from(raw.as_bytes())),
                Err(SmilesError::Limit { .. })
            ));
        }
    }
    let over = format!("CC\n[{}C]\n", "0".repeat(65_534));
    let error = SmilesFileDocument::parse(Arc::from(over.as_bytes()))
        .err()
        .ok_or("oversized notation")?;
    assert_eq!(error.code(), "resource_limit");
    assert_eq!(error.line_number(), Some(2));
    assert_eq!(error.record_id(), Some(id(2)?));
    assert_eq!(error.offset(), Some(3 + 65_536));
    assert!(matches!(
        error,
        SmilesFileError::Notation {
            error: SmilesError::Limit { offset: 65_536, .. },
            ..
        }
    ));
    let mut raw = b"CC \t".to_vec();
    raw.extend(vec![0xff; 70_000]);
    raw.extend(b"\0\r\n");
    let file = SmilesFileDocument::parse(Arc::from(raw.as_slice()))?;
    assert_eq!(file.as_bytes(), raw);
    assert_eq!(file.records()[0].application_data_range(), Some(4..70_005));
    Ok(())
}

#[test]
fn original_file_line_and_record_quotas_count_ignored_lines_and_final_eof() -> TestResult {
    assert_eq!(file::MAX_FILE_BYTES, 64 * 1024 * 1024);
    assert_eq!(file::MAX_PHYSICAL_LINES, 1_000_000);
    assert_eq!(file::MAX_RECORDS, 65_536);
    let mut maximum = vec![0xff; 64 * 1024 * 1024];
    maximum[0] = b' ';
    let source: Arc<[u8]> = maximum.into();
    let doc = SmilesFileDocument::parse(Arc::clone(&source))?;
    assert!(doc.records().is_empty());
    assert_eq!(doc.as_bytes().as_ptr(), source.as_ptr());
    drop(doc);
    drop(source);
    let error = SmilesFileDocument::parse(vec![b' '; 64 * 1024 * 1024 + 1].into())
        .err()
        .ok_or("file cap")?;
    assert_eq!(error.code(), "resource_limit");
    assert_eq!(error.offset(), Some(64 * 1024 * 1024));
    let lines = vec![b'\n'; 1_000_000];
    assert!(
        SmilesFileDocument::parse(Arc::from(lines.as_slice()))?
            .records()
            .is_empty()
    );
    let mut too_many = lines;
    too_many.push(b'C');
    let error = SmilesFileDocument::parse(too_many.into())
        .err()
        .ok_or("line cap")?;
    assert_eq!(error.code(), "resource_limit");
    assert_eq!(error.line_number(), Some(1_000_001));
    assert_eq!(error.offset(), Some(1_000_000));
    let records = b"C\n".repeat(65_536);
    let doc = SmilesFileDocument::parse(Arc::from(records.as_slice()))?;
    assert_eq!(doc.records().len(), 65_536);
    assert_eq!(doc.records()[65_535].id().get(), 65_536);
    drop(doc);
    let mut too_many = records;
    too_many.push(b'C');
    let error = SmilesFileDocument::parse(too_many.into())
        .err()
        .ok_or("record cap")?;
    assert_eq!(error.code(), "resource_limit");
    assert_eq!(error.line_number(), Some(65_537));
    assert_eq!(error.record_id(), Some(id(65_537)?));
    assert_eq!(error.offset(), Some(131_072));
    Ok(())
}

#[test]
fn clones_share_source_and_index_equality_debug_and_retained_accounting_stay_complete() -> TestResult
{
    let source: Arc<[u8]> = b"CC SECRET_METADATA_7e49\nN other\r\n".as_slice().into();
    let first = SmilesFileDocument::parse(Arc::clone(&source))?;
    let copy = first.clone();
    assert_eq!(first, copy);
    assert_eq!(first.as_bytes().as_ptr(), copy.as_bytes().as_ptr());
    assert_eq!(first.records().as_ptr(), copy.records().as_ptr());
    let inline = size_of::<SmilesFileDocument>();
    let index = 2 * size_of::<RecordInfo>();
    assert_eq!(
        SmilesFileDocument::retained_bytes([&first, &first]),
        inline + source.len() + index
    );
    assert_eq!(
        SmilesFileDocument::retained_bytes([&first, &copy]),
        2 * inline + source.len() + index
    );
    let reparsed = SmilesFileDocument::parse(Arc::clone(&source))?;
    assert_eq!(
        SmilesFileDocument::retained_bytes([&first, &reparsed]),
        2 * inline + source.len() + 2 * index
    );
    let independent = SmilesFileDocument::parse(source.to_vec().into())?;
    assert_eq!(
        SmilesFileDocument::retained_bytes([&first, &independent]),
        2 * (inline + source.len() + index)
    );
    let changed =
        SmilesFileDocument::parse(b"CC SECRET_METADATA_7e48\nN other\r\n".as_slice().into())?;
    assert_ne!(first, changed);
    assert!(!format!("{first:?}").contains("SECRET_METADATA_7e49"));
    assert!(!format!("{:?}", first.records()).contains("SECRET_METADATA_7e49"));
    let error = first.read_record(id(u32::MAX)?).err().ok_or("missing ID")?;
    assert_eq!(error.code(), "not_found");
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        SmilesFileDocument::parse_with_cancel(Arc::clone(&source), &cancel)
            .err()
            .ok_or("ignored cancellation")?
            .code(),
        "cancelled"
    );
    assert_eq!(source.as_ref(), first.as_bytes());
    Ok(())
}

fn external(script: &str, raw: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
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
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn independent_binary_scanner_and_unchanged_per_record_consumers_read_actual_source() -> TestResult
{
    let raw = b"\n \xffignored\r\nCC\t \xff\0name\t42\r\n\tCO ignored\nC#N id\nO last";
    assert_eq!(raw.len(), 52);
    let expected = external("smiles_file_raw_oracle.py", raw)?;
    assert_eq!(
        expected["sha256"],
        "df5888fbca54c70581fa9942319996ea3e098137da111201eab3404d58ba2a08"
    );
    assert_eq!(expected["physical_lines"], 6);
    let doc = SmilesFileDocument::parse(raw.as_slice().into())?;
    for (index, (line, full, notation, separator, data, ending, element, bonds)) in [
        (
            3,
            12..27,
            12..14,
            14..16,
            16..25,
            25..27,
            vec![6, 6],
            json!([[0, 1, "SINGLE"]]),
        ),
        (
            5,
            39..46,
            39..42,
            42..43,
            43..45,
            45..46,
            vec![6, 7],
            json!([[0, 1, "TRIPLE"]]),
        ),
        (
            6,
            46..52,
            46..47,
            47..48,
            48..52,
            52..52,
            vec![8],
            json!([]),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let record = doc.records().get(index).ok_or("record")?;
        assert_eq!(record.line_number(), line);
        assert_eq!(record.full_range(), full.clone());
        assert_eq!(record.notation_range(), notation.clone());
        assert_eq!(record.separator_range(), Some(separator.clone()));
        assert_eq!(record.application_data_range(), Some(data.clone()));
        assert_eq!(record.line_ending_range(), ending.clone());
        let scanner = &expected["records"][index];
        for (field, range) in [
            ("full_range", full),
            ("notation_range", notation.clone()),
            ("separator_range", separator),
            ("application_data_range", data),
            ("line_ending_range", ending),
        ] {
            assert_eq!(scanner[field], json!([range.start, range.end]));
        }
        let actual = external("smiles_oracle.py", raw.get(notation).ok_or("notation")?)?;
        assert_eq!(actual["accepted"], true);
        let atoms = actual["atoms"].as_array().ok_or("atoms")?;
        assert_eq!(
            atoms
                .iter()
                .map(|a| a["element"].as_u64().ok_or("element"))
                .collect::<Result<Vec<_>, _>>()?,
            element
        );
        assert_eq!(actual["bonds"], bonds);
    }
    for changed in [
        raw.iter()
            .copied()
            .filter(|b| *b != b'\r')
            .collect::<Vec<_>>(),
        raw.as_slice()
            .strip_suffix(b"O last")
            .ok_or("last record")?
            .to_vec(),
        [
            raw.get(..14).ok_or("prefix")?,
            b" ",
            raw.get(16..).ok_or("suffix")?,
        ]
        .concat(),
        raw[12..].to_vec(),
        raw.iter()
            .copied()
            .map(|b| if b == 0xff { 0xfe } else { b })
            .collect::<Vec<_>>(),
    ] {
        assert_ne!(
            external("smiles_file_raw_oracle.py", &changed)?["records"],
            expected["records"]
        );
    }
    Ok(())
}

#[test]
fn existing_real_export_reopens_as_one_exact_record_without_gaining_drawing_authority() -> TestResult
{
    let model = SmilesDocument::parse(b"CC".as_slice().into())?.normalize_explicit_h_fragment()?;
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("smiles-file-export-λ.smi");
    let receipt =
        bondline_io::files::export_smiles_fragment_file(&path, &model, &Cancellation::default())?;
    let bytes = fs::read(&path)?;
    let file = SmilesFileDocument::parse(bytes.clone().into())?;
    assert_eq!(file.as_bytes(), b"[CH3][CH3]\n");
    assert_eq!(receipt.byte_length, 11);
    assert_eq!(file.records().len(), 1);
    assert_eq!(file.records()[0].notation_range(), 0..10);
    assert_eq!(file.records()[0].line_ending_range(), 10..11);
    let graph = file.read_record(id(1)?)?;
    let actual = external("smiles_write_oracle.py", graph.as_bytes())?;
    assert_eq!(actual["meaning"]["smiles"], "CC");
    assert_eq!(actual["meaning"]["formula"], "C2H6");
    assert_eq!(fs::read(&path)?, bytes);
    Ok(())
}
