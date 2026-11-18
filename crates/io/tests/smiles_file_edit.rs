//! Independent notation-span editing: complete source bytes stay authoritative.

use bondline_io::smiles::{SmilesFileDocument, file::RecordId};
use bondline_render::Cancellation;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn existing_record_notation_changes_without_rebuilding_duplicates_or_opaque_data() -> TestResult {
    let source = b"CC first\r\nC1.N.C1\tPRIVATE\0\xff\nCC";
    let owner = SmilesFileDocument::parse(Arc::from(source.as_slice()))?;
    let before = owner.clone();
    let changed = owner.with_record_notation(
        RecordId::new(2).ok_or("record 2")?,
        b"CO",
        &Cancellation::default(),
    )?;
    assert_eq!(changed.as_bytes(), b"CC first\r\nCO\tPRIVATE\0\xff\nCC");
    assert_eq!(changed.records().len(), 3);
    let second = changed
        .record(RecordId::new(2).ok_or("record 2")?)
        .ok_or("second")?;
    assert_eq!((second.atom_count(), second.bond_count()), (2, 1));
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes(), source);
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(owner.records().as_ptr(), before.records().as_ptr());
    Ok(())
}

use bondline_io::smiles::{Limit, SmilesError, SmilesFileError};
use serde_json::{Value, json};
use std::{
    io::Write as _,
    ops::Range,
    process::{Command, Stdio},
};

const SOURCE: &[u8] = b"\r\n \xff\0ignored\nCC\t \tDUP\0\xff\r\nC1.N.C1 \tMETA\xff\0\nCC\t\r\nN";
const GROWN: &[u8] = b"\r\n \xff\0ignored\nCC\t \tDUP\0\xff\r\n[13CH3][OH] \tMETA\xff\0\nCC\t\r\nN";
const SHRUNK: &[u8] = b"\r\n \xff\0ignored\nCC\t \tDUP\0\xff\r\nO \tMETA\xff\0\nCC\t\r\nN";

fn id(value: u32) -> TestResult<RecordId> {
    Ok(RecordId::new(value).ok_or("positive record ID")?)
}

fn original() -> TestResult<SmilesFileDocument> {
    Ok(SmilesFileDocument::parse(Arc::from(SOURCE))?)
}

fn retained(file: &SmilesFileDocument, before: &SmilesFileDocument) {
    assert_eq!(file, before);
    assert_eq!(file.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(file.records().as_ptr(), before.records().as_ptr());
}

fn record(
    file: &SmilesFileDocument,
    ordinal: u32,
    line: usize,
    ranges: [Range<usize>; 3],
    separator: Option<Range<usize>>,
    data: Option<Range<usize>>,
    counts: [usize; 2],
) -> TestResult {
    let record = file.record(id(ordinal)?).ok_or("record")?;
    let [full, notation, ending] = ranges;
    assert_eq!((record.id().get(), record.line_number()), (ordinal, line));
    assert_eq!(record.full_range(), full);
    assert_eq!(record.notation_range(), notation.clone());
    assert_eq!(record.separator_range(), separator);
    assert_eq!(record.application_data_range(), data);
    assert_eq!(record.line_ending_range(), ending);
    assert_eq!([record.atom_count(), record.bond_count()], counts);
    assert_eq!(
        file.read_record(id(ordinal)?)?.as_bytes(),
        file.as_bytes().get(notation).ok_or("notation span")?
    );
    Ok(())
}

fn describe(script: &str, bytes: &[u8]) -> TestResult<Value> {
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
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

#[test]
fn length_changes_reindex_absolute_spans_without_touching_opaque_or_ignored_bytes() -> TestResult {
    assert_eq!((SOURCE.len(), GROWN.len(), SHRUNK.len()), (47, 51, 41));
    let file = original()?;
    let before = file.clone();
    record(
        &file,
        2,
        4,
        [25..41, 25..32, 40..41],
        Some(32..34),
        Some(34..40),
        [3, 1],
    )?;
    let grown = file.with_record_notation(id(2)?, b"[13CH3][OH]", &Cancellation::default())?;
    assert_eq!(grown.as_bytes(), GROWN);
    assert_eq!(grown.records().len(), 4);
    record(
        &grown,
        1,
        3,
        [13..25, 13..15, 23..25],
        Some(15..18),
        Some(18..23),
        [2, 1],
    )?;
    record(
        &grown,
        2,
        4,
        [25..45, 25..36, 44..45],
        Some(36..38),
        Some(38..44),
        [2, 1],
    )?;
    record(
        &grown,
        3,
        5,
        [45..50, 45..47, 48..50],
        Some(47..48),
        Some(48..48),
        [2, 1],
    )?;
    record(&grown, 4, 6, [50..51, 50..51, 51..51], None, None, [1, 0])?;
    let raw = describe("smiles_file_raw_oracle.py", grown.as_bytes())?;
    assert_eq!(raw["byte_length"], 51);
    assert_eq!(raw["physical_lines"], 6);
    assert_eq!(
        raw["records"],
        json!([
            {"line_number":3,"full_range":[13,25],"notation_range":[13,15],"separator_range":[15,18],
             "application_data_range":[18,23],"line_ending_range":[23,25],"notation_hex":"4343","application_data_hex":"44555000ff"},
            {"line_number":4,"full_range":[25,45],"notation_range":[25,36],"separator_range":[36,38],
             "application_data_range":[38,44],"line_ending_range":[44,45],"notation_hex":"5b31334348335d5b4f485d","application_data_hex":"4d455441ff00"},
            {"line_number":5,"full_range":[45,50],"notation_range":[45,47],"separator_range":[47,48],
             "application_data_range":[48,48],"line_ending_range":[48,50],"notation_hex":"4343","application_data_hex":""},
            {"line_number":6,"full_range":[50,51],"notation_range":[50,51],"separator_range":null,
             "application_data_range":null,"line_ending_range":[51,51],"notation_hex":"4e","application_data_hex":null}
        ])
    );
    // Independent raw ranges, rather than our own RecordInfo, choose each consumer input.
    let records = raw["records"].as_array().ok_or("external records")?;
    for (index, (raw_record, notation)) in records
        .iter()
        .zip(["CC", "[13CH3][OH]", "CC", "N"])
        .enumerate()
    {
        let start = usize::try_from(raw_record["notation_range"][0].as_u64().ok_or("start")?)?;
        let end = usize::try_from(raw_record["notation_range"][1].as_u64().ok_or("end")?)?;
        let molecular = describe(
            "smiles_oracle.py",
            grown.as_bytes().get(start..end).ok_or("external slice")?,
        )?;
        assert_eq!(molecular["accepted"], true);
        assert_eq!(molecular["source_ascii"], notation);
        let atoms = molecular["atoms"].as_array().ok_or("external atoms")?;
        assert_eq!(atoms.len(), if index == 3 { 1 } else { 2 });
        assert_eq!(
            molecular["bonds"],
            if index == 3 {
                json!([])
            } else {
                json!([[0, 1, "SINGLE"]])
            }
        );
        if index == 1 {
            assert_eq!(
                (
                    atoms[0]["element"].clone(),
                    atoms[0]["isotope"].clone(),
                    atoms[0]["explicit_h"].clone()
                ),
                (json!(6), json!(13), json!(3))
            );
            assert_eq!(
                (atoms[1]["element"].clone(), atoms[1]["explicit_h"].clone()),
                (json!(8), json!(1))
            );
        }
    }
    let original_interleaved = describe("smiles_oracle.py", b"C1.N.C1")?;
    assert_eq!(original_interleaved["bonds"], json!([[0, 2, "SINGLE"]]));
    assert_eq!(
        original_interleaved["atoms"]
            .as_array()
            .ok_or("interleaved atoms")?
            .iter()
            .map(|a| a["element"].clone())
            .collect::<Vec<_>>(),
        vec![json!(6), json!(7), json!(6)]
    );
    let grown_before = grown.clone();
    let shrunk = grown.with_record_notation(id(2)?, b"O", &Cancellation::default())?;
    assert_eq!(shrunk.as_bytes(), SHRUNK);
    record(
        &shrunk,
        2,
        4,
        [25..35, 25..26, 34..35],
        Some(26..28),
        Some(28..34),
        [1, 0],
    )?;
    record(
        &shrunk,
        3,
        5,
        [35..40, 35..37, 38..40],
        Some(37..38),
        Some(38..38),
        [2, 1],
    )?;
    record(&shrunk, 4, 6, [40..41, 40..41, 41..41], None, None, [1, 0])?;
    assert_eq!(
        shrunk,
        file.with_record_notation(id(2)?, b"O", &Cancellation::default())?
    );
    retained(&file, &before);
    retained(&grown, &grown_before);
    Ok(())
}

#[test]
fn exact_noop_shares_source_and_index_but_equivalent_spelling_and_duplicate_targets_do_not()
-> TestResult {
    let file = original()?;
    let before = file.clone();
    let separate_input = b"C1.N.C1".to_vec();
    let unchanged = file.with_record_notation(id(2)?, &separate_input, &Cancellation::default())?;
    retained(&unchanged, &file);
    assert_eq!(
        SmilesFileDocument::retained_bytes([&file, &unchanged])
            - SmilesFileDocument::retained_bytes([&file]),
        std::mem::size_of::<SmilesFileDocument>()
    );
    let equivalent = file.with_record_notation(id(2)?, b"CC.N", &Cancellation::default())?;
    assert_eq!(
        equivalent.as_bytes(),
        b"\r\n \xff\0ignored\nCC\t \tDUP\0\xff\r\nCC.N \tMETA\xff\0\nCC\t\r\nN"
    );
    assert_ne!(equivalent.as_bytes().as_ptr(), file.as_bytes().as_ptr());
    let duplicate = file.with_record_notation(id(3)?, b"C=C", &Cancellation::default())?;
    assert_eq!(
        duplicate.as_bytes(),
        b"\r\n \xff\0ignored\nCC\t \tDUP\0\xff\r\nC1.N.C1 \tMETA\xff\0\nC=C\t\r\nN"
    );
    assert_eq!(duplicate.read_record(id(1)?)?.as_bytes(), b"CC");
    assert_eq!(
        duplicate
            .record(id(3)?)
            .ok_or("duplicate target")?
            .application_data_range(),
        Some(45..45)
    );
    let final_eof = file.with_record_notation(id(4)?, b"[H]", &Cancellation::default())?;
    assert_eq!(
        final_eof.as_bytes(),
        b"\r\n \xff\0ignored\nCC\t \tDUP\0\xff\r\nC1.N.C1 \tMETA\xff\0\nCC\t\r\n[H]"
    );
    record(
        &final_eof,
        4,
        6,
        [46..49, 46..49, 49..49],
        None,
        None,
        [1, 0],
    )?;
    retained(&file, &before);
    Ok(())
}

#[test]
fn missing_targets_and_invalid_replacements_have_absolute_record_context_without_partial_edits()
-> TestResult {
    let file = original()?;
    let before = file.clone();
    assert!(RecordId::new(0).is_none());
    for missing in [5, u32::MAX] {
        let error = file
            .with_record_notation(id(missing)?, b"CO", &Cancellation::default())
            .err()
            .ok_or("missing record accepted")?;
        assert_eq!(error.code(), "not_found");
        assert_eq!(error.record_id(), Some(id(missing)?));
        assert_eq!((error.line_number(), error.offset()), (None, None));
    }
    for (notation, local, code) in [
        (b"".as_slice(), 0, "invalid_smiles"),
        (b" CC", 0, "invalid_smiles"),
        (b"CC ", 2, "invalid_smiles"),
        (b"CC\tname", 2, "invalid_smiles"),
        (b"CC\n", 2, "invalid_smiles"),
        (b"CC\r\n", 2, "invalid_smiles"),
        (b"CC\r", 2, "invalid_smiles"),
        (b"CC\nO", 2, "invalid_smiles"),
        (b"C\0C", 1, "invalid_smiles"),
        (b"C\x0bC", 1, "invalid_smiles"),
        (b"C\x0cC", 1, "invalid_smiles"),
        (b"C(", 2, "invalid_smiles"),
        (b"\xff", 0, "unsupported_content"),
        (b"*", 0, "unsupported_content"),
        (b"[C@H](F)(Cl)Br", 2, "unsupported_content"),
    ] {
        let error = file
            .with_record_notation(id(2)?, notation, &Cancellation::default())
            .err()
            .ok_or("invalid replacement accepted")?;
        assert_eq!(error.code(), code, "{error}");
        assert_eq!(
            (error.line_number(), error.record_id(), error.offset()),
            (Some(4), Some(id(2)?), Some(25 + local))
        );
        assert!(matches!(error, SmilesFileError::Notation { .. }));
        retained(&file, &before);
    }
    let empty = SmilesFileDocument::parse(Arc::from(b"\n \xff\0ignored\r\n".as_slice()))?;
    assert_eq!(
        empty
            .with_record_notation(id(1)?, b"C", &Cancellation::default())
            .err()
            .ok_or("empty file fabricated a record")?
            .code(),
        "not_found"
    );
    retained(&file, &before);
    Ok(())
}

#[test]
fn cancellation_precedes_noop_missing_record_and_replacement_validation() -> TestResult {
    let file = original()?;
    let before = file.clone();
    let cancel = Cancellation::default();
    cancel.cancel();
    for (target, notation) in [(2, b"C1.N.C1".as_slice()), (99, b"CO"), (2, b"")] {
        let error = file
            .with_record_notation(id(target)?, notation, &cancel)
            .err()
            .ok_or("pre-cancelled edit accepted")?;
        assert_eq!(error.code(), "cancelled");
        retained(&file, &before);
    }
    Ok(())
}

#[test]
fn notation_file_and_record_boundaries_keep_the_existing_codec_limits() -> TestResult {
    let file = original()?;
    let before = file.clone();
    // Existing syntax-limit vector, not a valid-isotope/chemistry or writer maximum claim.
    let maximum = format!("[{}C]", "0".repeat(65_533));
    assert_eq!(maximum.len(), 65_536);
    let changed =
        file.with_record_notation(id(2)?, maximum.as_bytes(), &Cancellation::default())?;
    let second = changed.record(id(2)?).ok_or("maximum notation")?;
    assert_eq!(second.notation_range(), 25..65_561);
    assert_eq!(changed.read_record(id(2)?)?.as_bytes(), maximum.as_bytes());
    assert_eq!(
        changed.as_bytes().get(..25).ok_or("prefix")?,
        SOURCE.get(..25).ok_or("old prefix")?
    );
    assert_eq!(
        changed.as_bytes().get(65_561..).ok_or("suffix")?,
        SOURCE.get(32..).ok_or("old suffix")?
    );
    let over = format!("[{}C]", "0".repeat(65_534));
    let error = file
        .with_record_notation(id(2)?, over.as_bytes(), &Cancellation::default())
        .err()
        .ok_or("oversized notation")?;
    assert_eq!(error.code(), "resource_limit");
    assert_eq!(
        (error.record_id(), error.line_number(), error.offset()),
        (Some(id(2)?), Some(4), Some(65_561))
    );
    assert!(matches!(
        error,
        SmilesFileError::Notation {
            error: SmilesError::Limit {
                offset: 65_536,
                kind: Limit::Bytes
            },
            ..
        }
    ));
    retained(&file, &before);

    let mut maximum_file = vec![0xff; 64 * 1024 * 1024];
    let prefix = maximum_file.get_mut(..2).ok_or("file prefix")?;
    prefix.copy_from_slice(b"C ");
    let maximum_file = SmilesFileDocument::parse(Arc::from(maximum_file))?;
    let before = maximum_file.clone();
    let error = maximum_file
        .with_record_notation(id(1)?, b"CC", &Cancellation::default())
        .err()
        .ok_or("oversized complete candidate")?;
    assert_eq!(error.code(), "resource_limit");
    assert!(matches!(error, SmilesFileError::InputLimit { offset } if offset==64*1024*1024));
    retained(&maximum_file, &before);
    drop(before);
    drop(maximum_file);

    let mut records = b"C\n".repeat(65_535);
    records.push(b'C');
    let maximum_records = SmilesFileDocument::parse(Arc::from(records))?;
    let before = maximum_records.clone();
    let changed =
        maximum_records.with_record_notation(id(65_536)?, b"CO", &Cancellation::default())?;
    assert_eq!(changed.records().len(), 65_536);
    record(
        &changed,
        65_536,
        65_536,
        [131_070..131_072, 131_070..131_072, 131_072..131_072],
        None,
        None,
        [2, 1],
    )?;
    assert_eq!(
        changed
            .as_bytes()
            .get(..131_070)
            .ok_or("all earlier records")?,
        maximum_records
            .as_bytes()
            .get(..131_070)
            .ok_or("old records")?
    );
    retained(&maximum_records, &before);
    Ok(())
}
