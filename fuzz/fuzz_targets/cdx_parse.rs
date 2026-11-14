#![no_main]

use bondline_cdx::{Document, Error, Framing, Record, RecordKind, Writer};
use libfuzzer_sys::fuzz_target;

fn body<'a>(document: &'a Document<'_>) -> Vec<&'a Record> {
    let explicit = document.framing() == Framing::ExplicitDocument;
    document.records().iter().enumerate().filter_map(|(index, record)| {
        let root = explicit && index == 0
            || matches!(record.kind(), RecordKind::ObjectEnd { start } if start.is_none() || explicit && *start == Some(0));
        (!root).then_some(record)
    }).collect()
}

fn construct(document: &Document<'_>) -> Result<Vec<u8>, Error> {
    let mut writer = Writer::new()?;
    for record in body(document) {
        match record.kind() {
            RecordKind::ObjectStart { tag, id } => writer.start_object(*tag, *id)?,
            RecordKind::Property { tag, .. } => writer.property(*tag, document.payload(record)?)?,
            RecordKind::ObjectEnd { .. } => writer.end_object()?,
        }
    }
    writer.finish()
}

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

    // New syntax uses an explicit ID0 root and canonical lengths, not source passthrough.
    match construct(&document) {
        Ok(bytes) => {
            let written = Document::parse(&bytes).unwrap();
            assert_eq!(written.framing(), Framing::ExplicitDocument);
            let before = body(&document);
            let after = body(&written);
            assert_eq!(before.len(), after.len());
            for (before, after) in before.into_iter().zip(after) {
                match (before.kind(), after.kind()) {
                    (
                        RecordKind::ObjectStart { tag: a, id: b },
                        RecordKind::ObjectStart { tag: c, id: d },
                    ) => assert_eq!((a, b), (c, d)),
                    (RecordKind::Property { tag: a, .. }, RecordKind::Property { tag: b, .. }) => {
                        assert_eq!(a, b);
                        assert_eq!(
                            document.payload(before).unwrap(),
                            written.payload(after).unwrap()
                        );
                    }
                    (RecordKind::ObjectEnd { .. }, RecordKind::ObjectEnd { .. }) => {}
                    _ => panic!("new syntax changed object/property order"),
                }
            }
            assert_eq!(document.as_bytes(), data);
        }
        Err(error) => assert!(matches!(
            error,
            Error::InputTooLarge | Error::DepthLimit | Error::RecordLimit | Error::Allocation
        )),
    }
});
