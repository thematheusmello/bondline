use bondline_cdx::{Document, Error, LengthEncoding, MAX_INPUT_BYTES, RecordKind};

const UNKNOWN: &[u8] = include_bytes!("fixtures/unknown-implicit.cdx");

#[test]
fn replacing_one_payload_preserves_every_untouched_byte_and_no_op_encoding()
-> Result<(), Box<dyn std::error::Error>> {
    let document = Document::parse(UNKNOWN)?;
    let first = document.records().first().ok_or("first")?;
    let changed = document.replace_property(first, &[0x12, 0x34])?;
    let mut expected = UNKNOWN.get(..30).ok_or("prefix")?.to_vec();
    expected.extend_from_slice(&[2, 0, 0x12, 0x34]);
    expected.extend_from_slice(UNKNOWN.get(35..).ok_or("suffix")?);
    assert_eq!(changed, expected);
    assert_eq!(document.as_bytes(), UNKNOWN);
    let extended = document.records().get(2).ok_or("extended")?;
    assert_eq!(
        document.replace_property(extended, &[0xff, 0x80, 0])?,
        UNKNOWN
    );
    let shorter = document.replace_property(extended, &[])?;
    let mut expected = UNKNOWN.get(..43).ok_or("extended prefix")?.to_vec();
    expected.extend_from_slice(&[0, 0]);
    expected.extend_from_slice(UNKNOWN.get(52..).ok_or("extended suffix")?);
    assert_eq!(
        shorter, expected,
        "a changed small payload uses its documented short length"
    );
    Ok(())
}

#[test]
fn edited_length_changes_at_the_documented_65535_sentinel() -> Result<(), Box<dyn std::error::Error>>
{
    let document = Document::parse(UNKNOWN)?;
    let first = document.records().first().ok_or("first")?;
    for (length, prefix, encoding) in [
        (65_534, &[0xfe, 0xff][..], LengthEncoding::Short),
        (
            65_535,
            &[0xff, 0xff, 0xff, 0xff, 0, 0][..],
            LengthEncoding::Extended,
        ),
        (
            65_536,
            &[0xff, 0xff, 0, 0, 1, 0][..],
            LengthEncoding::Extended,
        ),
    ] {
        let payload = vec![0x8f; length];
        let changed = document.replace_property(first, &payload)?;
        assert_eq!(changed.get(..30), UNKNOWN.get(..30));
        assert_eq!(changed.get(30..30 + prefix.len()), Some(prefix));
        assert_eq!(changed.get(changed.len() - 35..), UNKNOWN.get(35..));
        let parsed = Document::parse(&changed)?;
        let record = parsed.records().first().ok_or("changed property")?;
        assert!(
            matches!(record.kind(), RecordKind::Property { length, .. } if *length == encoding)
        );
        assert_eq!(parsed.payload(record)?, payload);
    }
    Ok(())
}

#[test]
fn property_handles_must_belong_to_the_exact_document_and_select_a_property()
-> Result<(), Box<dyn std::error::Error>> {
    let first = Document::parse(UNKNOWN)?;
    let second = Document::parse(UNKNOWN)?;
    let foreign = first.records().first().ok_or("foreign")?;
    assert_eq!(
        second.replace_property(foreign, &[1]),
        Err(Error::InvalidProperty)
    );
    assert_eq!(second.payload(foreign), Err(Error::InvalidProperty));
    let object = second.records().get(1).ok_or("object")?;
    assert_eq!(
        second.replace_property(object, &[1]),
        Err(Error::InvalidProperty)
    );
    assert_eq!(second.payload(object), Err(Error::InvalidProperty));
    let changed = first.replace_property(foreign, &[4, 5, 6])?;
    let reparsed = Document::parse(&changed)?;
    assert_eq!(
        reparsed.replace_property(foreign, &[1]),
        Err(Error::InvalidProperty)
    );
    assert_eq!(first.as_bytes(), UNKNOWN);
    assert_eq!(second.as_bytes(), UNKNOWN);
    Ok(())
}

#[test]
fn output_budget_includes_framing_and_rejects_growth_before_building_an_output()
-> Result<(), Box<dyn std::error::Error>> {
    let document = Document::parse(UNKNOWN)?;
    let first = document.records().first().ok_or("first")?;
    // Fixture70 bytes, replaced record7 bytes, new extended property prefix8 bytes.
    let payload = vec![0x80; MAX_INPUT_BYTES - 71];
    let boundary = document.replace_property(first, &payload)?;
    assert_eq!(boundary.len(), MAX_INPUT_BYTES);
    drop(boundary);
    let oversized = vec![0x80; MAX_INPUT_BYTES - 70];
    assert_eq!(
        document.replace_property(first, &oversized),
        Err(Error::InputTooLarge)
    );
    assert_eq!(document.as_bytes(), UNKNOWN);
    Ok(())
}
