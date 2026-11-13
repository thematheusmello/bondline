use bondline_cdx::{Document, Error, Framing, LengthEncoding, RecordKind};

const UNKNOWN: &[u8] = include_bytes!("fixtures/unknown-implicit.cdx");

#[test]
fn original_unknown_records_have_exact_published_byte_spans_and_parent_ownership()
-> Result<(), Box<dyn std::error::Error>> {
    let document = Document::parse(UNKNOWN)?;
    assert_eq!(document.framing(), Framing::ImplicitDocument);
    assert_eq!(document.header(), UNKNOWN.get(..28).ok_or("header")?);
    assert_eq!(document.as_bytes(), UNKNOWN);
    assert_eq!(document.to_bytes(), UNKNOWN);
    let records = document.records();
    assert_eq!(records.len(), 8);
    let first = records.first().ok_or("first property")?;
    assert_eq!(first.range(), 28..35);
    assert_eq!(first.parent(), None);
    assert_eq!(
        first.kind(),
        &RecordKind::Property {
            tag: 0x4002,
            length: LengthEncoding::Short,
            payload: 32..35
        }
    );
    assert_eq!(document.payload(first)?, [0xa5, 0, 0x7f]);
    let page = records.get(1).ok_or("page")?;
    assert_eq!(page.range(), 35..41);
    assert_eq!(
        page.kind(),
        &RecordKind::ObjectStart {
            tag: 0x8001,
            id: 42
        }
    );
    assert_eq!(page.parent(), None);
    let extended = records.get(2).ok_or("extended property")?;
    assert_eq!(extended.range(), 41..52);
    assert_eq!(extended.parent(), Some(1));
    assert_eq!(
        extended.kind(),
        &RecordKind::Property {
            tag: 0x7ffe,
            length: LengthEncoding::Extended,
            payload: 49..52
        }
    );
    assert_eq!(document.payload(extended)?, [0xff, 0x80, 0]);
    let custom = records.get(3).ok_or("custom object")?;
    assert_eq!(
        custom.kind(),
        &RecordKind::ObjectStart {
            tag: 0xc123,
            id: 77
        }
    );
    assert_eq!(custom.range(), 52..58);
    assert_eq!(custom.parent(), Some(1));
    assert_eq!(records.get(4).ok_or("custom property")?.parent(), Some(3));
    let close = records.get(5).ok_or("custom end")?;
    assert_eq!(close.kind(), &RecordKind::ObjectEnd { start: Some(3) });
    assert_eq!(close.parent(), Some(1));
    assert_eq!(
        records.get(6).ok_or("page end")?.kind(),
        &RecordKind::ObjectEnd { start: Some(1) }
    );
    assert_eq!(
        records.last().ok_or("document end")?.kind(),
        &RecordKind::ObjectEnd { start: None }
    );
    Ok(())
}

#[test]
fn truncation_and_unknown_header_versions_never_return_partial_documents() {
    for length in 0..UNKNOWN.len() {
        let prefix = UNKNOWN.get(..length).unwrap_or_default();
        assert!(
            Document::parse(prefix).is_err(),
            "accepted truncation at{length}"
        );
    }
    let mut invalid = UNKNOWN.to_vec();
    if let Some(byte) = invalid.get_mut(0) {
        *byte = b'X';
    }
    assert!(matches!(
        Document::parse(&invalid),
        Err(Error::InvalidHeader)
    ));
    let mut marker = UNKNOWN.to_vec();
    if let Some(byte) = marker.get_mut(8) {
        *byte = 1;
    }
    assert!(matches!(
        Document::parse(&marker),
        Err(Error::InvalidHeader)
    ));
    let mut trailing = UNKNOWN.to_vec();
    trailing.extend_from_slice(&[0, 0]);
    assert!(matches!(
        Document::parse(&trailing),
        Err(Error::TrailingData)
    ));
}

#[test]
fn explicit_document_framing_keeps_the_encoded_root_and_requires_exact_eof()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = UNKNOWN.get(..28).ok_or("header")?.to_vec();
    source.extend_from_slice(&[0, 0x80, 0, 0, 0, 0]);
    source.extend_from_slice(UNKNOWN.get(28..).ok_or("body")?);
    let document = Document::parse(&source)?;
    assert_eq!(document.framing(), Framing::ExplicitDocument);
    assert_eq!(
        document.records().first().ok_or("document")?.kind(),
        &RecordKind::ObjectStart { tag: 0x8000, id: 0 }
    );
    assert_eq!(
        document.records().get(1).ok_or("root property")?.parent(),
        Some(0)
    );
    assert_eq!(
        document.records().last().ok_or("root end")?.kind(),
        &RecordKind::ObjectEnd { start: Some(0) }
    );
    assert_eq!(document.as_bytes(), source);
    assert_eq!(document.to_bytes(), source);
    for length in 0..source.len() {
        assert!(Document::parse(source.get(..length).ok_or("prefix")?).is_err());
    }
    let mut trailing = source;
    trailing.extend_from_slice(&[0, 0]);
    assert!(
        matches!(Document::parse(&trailing), Err(Error::TrailingData)),
        "Bondline requires exact EOF even though some external readers tolerate extra terminators"
    );
    Ok(())
}
