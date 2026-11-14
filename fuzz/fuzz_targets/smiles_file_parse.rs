#![no_main]

use bondline_io::smiles::{SmilesFileDocument, file::RecordId};
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

fuzz_target!(|data: &[u8]| {
    let document = match SmilesFileDocument::parse(Arc::from(data)) {
        Ok(document) => document,
        Err(error) => {
            if let Some(offset) = error.offset() {
                assert!(offset <= data.len());
            }
            if let Some(id) = error.record_id() {
                assert!(id.get() > 0);
            }
            return;
        }
    };
    assert_eq!(document.as_bytes(), data);
    assert!(RecordId::new(0).is_none());
    let shared = document.clone();
    assert_eq!(document, shared);
    assert!(std::ptr::eq(document.as_bytes(), shared.as_bytes()));
    assert!(std::ptr::eq(document.records(), shared.records()));
    let retained = SmilesFileDocument::retained_bytes([&document]);
    assert_eq!(
        SmilesFileDocument::retained_bytes([&document, &document]),
        retained
    );
    assert_eq!(
        SmilesFileDocument::retained_bytes([&document, &shared]),
        retained + size_of::<SmilesFileDocument>()
    );

    let mut physical_line = 1_usize;
    let mut start = 0_usize;
    let mut ordinal = 0_usize;
    // Walk byte boundaries independently of the parser's split_inclusive iterator.
    for end in 1..=data.len() {
        if data.get(end - 1) != Some(&b'\n') && end != data.len() {
            continue;
        }
        let line = data.get(start..end).expect("physical line");
        let mut body_end = end;
        if line.last() == Some(&b'\n') {
            body_end -= 1;
            if body_end > start && data.get(body_end - 1) == Some(&b'\r') {
                body_end -= 1;
            }
        }
        let body = data.get(start..body_end).expect("line body");
        if !body.is_empty() && !matches!(body.first(), Some(b' ' | b'\t' | b'\r')) {
            let record = document
                .records()
                .get(ordinal)
                .expect("every meaningful line is indexed");
            ordinal += 1;
            assert_eq!(record.id().get() as usize, ordinal);
            assert_eq!(record.line_number(), physical_line);
            assert_eq!(record.full_range(), start..end);
            assert_eq!(record.line_ending_range(), body_end..end);
            let notation_end = record.notation_range().end;
            assert_eq!(record.notation_range().start, start);
            assert!(start < notation_end && notation_end <= body_end);
            let notation = data.get(start..notation_end).expect("notation span");
            assert!(
                !notation
                    .iter()
                    .any(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
            );
            match (record.separator_range(), record.application_data_range()) {
                (Some(separator), Some(application)) => {
                    assert_eq!(separator.start, notation_end);
                    assert!(separator.start < separator.end && separator.end <= body_end);
                    assert!(
                        data.get(separator.clone())
                            .expect("separator")
                            .iter()
                            .all(|byte| matches!(byte, b' ' | b'\t'))
                    );
                    assert_eq!(application, separator.end..body_end);
                    assert!(!matches!(data.get(separator.end), Some(b' ' | b'\t')));
                }
                (None, None) => assert_eq!(notation_end, body_end),
                _ => panic!("separator and opaque data spans must agree"),
            }
            let read = document
                .read_record(record.id())
                .expect("validated notation reparses");
            assert_eq!(read.as_bytes(), notation);
            assert_eq!(read.atoms().len(), record.atom_count());
            assert_eq!(read.bonds().len(), record.bond_count());
            assert_eq!(document.record(record.id()), Some(record));
        }
        physical_line += 1;
        start = end;
    }
    assert_eq!(ordinal, document.records().len());
    let missing = RecordId::new(u32::try_from(ordinal + 1).expect("bounded record count"))
        .expect("positive ordinal");
    assert!(document.record(missing).is_none());
    assert_eq!(
        document
            .read_record(missing)
            .expect_err("no fallback record")
            .code(),
        "not_found"
    );
});
