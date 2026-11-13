#![no_main]

use bondline_cdx::{Document, RecordKind};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(document) = Document::parse(data) else {
        return;
    };
    assert_eq!(document.as_bytes(), data);
    assert_eq!(document.to_bytes(), data);
    let mut offset = document.header().len();
    for record in document.records() {
        let range = record.range();
        assert_eq!(range.start, offset);
        assert!(range.end <= data.len());
        offset = range.end;
    }
    assert_eq!(offset, data.len());

    if let Some((index, record)) = document
        .records()
        .iter()
        .enumerate()
        .find(|(_, record)| matches!(record.kind(), RecordKind::Property { .. }))
    {
        let payload = document.payload(record).unwrap();
        assert_eq!(document.replace_property(record, payload).unwrap(), data);
        let replacement = [0, 0xff, 0x80, 0, 1, 2, 3, 4];
        let output = document.replace_property(record, &replacement).unwrap();
        let edited = Document::parse(&output).unwrap();
        assert_eq!(edited.records().len(), document.records().len());
        assert_eq!(edited.framing(), document.framing());
        let changed = edited.records().get(index).unwrap();
        assert_eq!(edited.payload(changed).unwrap(), replacement);
        let original_range = record.range();
        let changed_range = changed.range();
        assert_eq!(
            data.get(..original_range.start),
            output.get(..changed_range.start)
        );
        assert_eq!(
            data.get(original_range.end..),
            output.get(changed_range.end..)
        );
        assert_eq!(document.as_bytes(), data);
    }
});
