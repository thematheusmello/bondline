//! Independent complete output bytes for atomic CDX property replacement.
use bondline_cdx::{Document, Error, Framing, MAX_INPUT_BYTES, MAX_RECORDS, Record, RecordKind};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const UNKNOWN: &[u8] = include_bytes!("fixtures/unknown-implicit.cdx");
const ELEMENT: u16 = 0x0402;
const HYDROGENS: u16 = 0x042b;
const TEXT: u16 = 0x0700;
const BOX: u16 = 0x0204;

fn property(tag: u16, bytes: &[u8], extended: bool) -> Vec<u8> {
    let mut result = Vec::from(tag.to_le_bytes());
    if extended {
        result.extend_from_slice(&[255, 255]);
        result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    } else {
        result.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
    }
    result.extend_from_slice(bytes);
    result
}

fn object(tag: u16, id: u32, fields: &[Vec<u8>]) -> Vec<u8> {
    let mut result = Vec::from(tag.to_le_bytes());
    result.extend_from_slice(&id.to_le_bytes());
    result.extend(fields.iter().flatten());
    result.extend_from_slice(&[0, 0]);
    result
}

fn text_payload(body: &str) -> Vec<u8> {
    let mut result = vec![1, 0, 0, 0, 0, 0, 0, 0, 240, 0, 4, 0];
    result.extend_from_slice(body.as_bytes());
    result
}

fn bounds(right: i32) -> Vec<u8> {
    [6 << 16, -2 << 16, 15 << 16, right << 16]
        .into_iter()
        .flat_map(i32::to_le_bytes)
        .collect()
}

fn scene(framing: Framing, edited: bool) -> Vec<u8> {
    let label = object(
        0x8006,
        91,
        &[
            property(BOX, &bounds(if edited { 19 } else { 7 }), false),
            property(0x4011, b"embedded metadata\0", true),
            property(
                TEXT,
                &text_payload(if edited { "NH" } else { "O" }),
                !edited,
            ),
        ],
    );
    let atom = object(
        0x8004,
        83,
        &[
            property(ELEMENT, &[if edited { 7 } else { 8 }, 0], !edited),
            property(0x4001, &[0xff, 0, 0x80], true),
            label,
            property(HYDROGENS, &[u8::from(edited), 0], false),
        ],
    );
    let peer = object(
        0x8004,
        17,
        &[
            property(ELEMENT, &[6, 0], true),
            property(HYDROGENS, &[0, 0], false),
            object(
                0x8006,
                91,
                &[
                    property(TEXT, &text_payload("C"), true),
                    property(BOX, &bounds(9), true),
                ],
            ),
        ],
    );
    let sheet = object(0x8001, 2, &[object(0x8003, 3, &[atom, peer])]);
    let fields = vec![property(0x4031, b"opaque\x00\x81", true), sheet];
    let mut header = b"VjCD0100\x04\x03\x02\x01".to_vec();
    header.resize(28, 0);
    if !matches!(
        framing,
        Framing::ImplicitDocumentFileEnd | Framing::ExplicitDocumentFileEnd
    ) {
        header[12] = 0x19;
    }
    match framing {
        Framing::ImplicitDocument | Framing::ImplicitDocumentFileEnd => {
            header.extend(fields.into_iter().flatten());
            header.extend_from_slice(&[0, 0]);
        }
        Framing::ExplicitDocument | Framing::ExplicitDocumentFileEnd => {
            header.extend(object(0x8000, 12, &fields));
        }
        Framing::HeaderDocument => {
            header.truncate(22);
            header.extend(object(0x8000, 12, &fields));
        }
    }
    if matches!(
        framing,
        Framing::ImplicitDocumentFileEnd
            | Framing::ExplicitDocumentFileEnd
            | Framing::HeaderDocument
    ) {
        header.extend_from_slice(&[0, 0]);
    }
    header
}

fn owned_property<'a>(document: &'a Document<'_>, owner: usize, tag: u16) -> Result<&'a Record> {
    document
        .records()
        .iter()
        .find(|record| {
            record.parent() == Some(owner)
                && matches!(record.kind(), RecordKind::Property { tag: actual, .. } if *actual == tag)
        })
        .ok_or_else(|| format!("owner {owner} missing property {tag:#06x}").into())
}

#[test]
fn four_scoped_fields_splice_in_source_order_across_all_framings() -> Result {
    for framing in [
        Framing::ImplicitDocument,
        Framing::ExplicitDocument,
        Framing::HeaderDocument,
        Framing::ImplicitDocumentFileEnd,
        Framing::ExplicitDocumentFileEnd,
    ] {
        let source = scene(framing, false);
        let expected = scene(framing, true);
        let document = Document::parse(&source)?;
        assert_eq!(document.framing(), framing);
        let node = document
            .records()
            .iter()
            .position(|record| {
                matches!(
                    record.kind(),
                    RecordKind::ObjectStart {
                        tag: 0x8004,
                        id: 83
                    }
                )
            })
            .ok_or("missing original node")?;
        let label = document
            .records()
            .iter()
            .position(|record| {
                record.parent() == Some(node)
                    && matches!(
                        record.kind(),
                        RecordKind::ObjectStart {
                            tag: 0x8006,
                            id: 91
                        }
                    )
            })
            .ok_or("missing originally scoped Text")?;
        let text = text_payload("NH");
        let box_bytes = bounds(19);
        let changes = [
            (owned_property(&document, node, HYDROGENS)?, &[1, 0][..]),
            (owned_property(&document, label, BOX)?, box_bytes.as_slice()),
            (owned_property(&document, node, ELEMENT)?, &[7, 0][..]),
            (owned_property(&document, label, TEXT)?, text.as_slice()),
        ];
        let output = document.replace_properties(&changes)?;
        assert_eq!(output, expected, "{framing:?}");
        let mut reordered = changes;
        reordered.reverse();
        assert_eq!(document.replace_properties(&reordered)?, expected);
        let readback = Document::parse(&output)?;
        assert_eq!(readback.framing(), framing);
        assert_eq!(readback.records().len(), document.records().len());
        assert_eq!(readback.header(), document.header());
        for (before, after) in document.records().iter().zip(readback.records()) {
            assert_eq!(before.parent(), after.parent());
        }
        assert_eq!(document.as_bytes(), source);
    }
    Ok(())
}

#[test]
fn empty_and_equal_payloads_keep_original_bytes_while_changes_use_canonical_lengths() -> Result {
    let document = Document::parse(UNKNOWN)?;
    let first = document.records().first().ok_or("first property")?;
    let extended = document.records().get(2).ok_or("extended property")?;
    let value = document.payload(extended)?;
    assert_eq!(document.replace_properties(&[])?, UNKNOWN);
    assert_eq!(
        document.replace_properties(&[(extended, value), (first, document.payload(first)?)])?,
        UNKNOWN
    );
    let threshold_value = vec![0xa7; 65_535];
    let output = document.replace_properties(&[(extended, value), (first, &threshold_value)])?;
    let mut expected = UNKNOWN.get(..30).ok_or("header and tag")?.to_vec();
    expected.extend_from_slice(&[255, 255, 255, 255, 0, 0]);
    expected.extend_from_slice(&threshold_value);
    expected.extend_from_slice(UNKNOWN.get(35..).ok_or("unchanged suffix")?);
    assert_eq!(output, expected);
    let output = document.replace_properties(&[(first, &[]), (extended, &[1, 2, 3])])?;
    let mut expected = UNKNOWN.get(..30).ok_or("header and tag")?.to_vec();
    expected.extend_from_slice(&[0, 0]);
    expected.extend_from_slice(UNKNOWN.get(35..43).ok_or("untouched in-between bytes")?);
    expected.extend_from_slice(&[3, 0, 1, 2, 3]);
    expected.extend_from_slice(UNKNOWN.get(52..).ok_or("untouched suffix")?);
    assert_eq!(output, expected);
    assert_eq!(
        document.replace_properties(&[(extended, &[1, 2, 3])])?,
        document.replace_property(extended, &[1, 2, 3])?
    );
    Ok(())
}

#[test]
fn duplicate_noops_and_foreign_or_non_property_handles_conflict_atomically() -> Result {
    let document = Document::parse(UNKNOWN)?;
    let other = Document::parse(UNKNOWN)?;
    let first = document.records().first().ok_or("first property")?;
    let value = document.payload(first)?;
    let foreign = other.records().first().ok_or("foreign property")?;
    let object = document.records().get(1).ok_or("object opening")?;
    assert_eq!(
        document.replace_properties(&[(first, value), (first, value)]),
        Err(Error::ConflictingEdits)
    );
    assert_eq!(
        document.replace_properties(&[(first, &[3]), (first, value)]),
        Err(Error::ConflictingEdits)
    );
    for invalid in [foreign, object] {
        assert_eq!(
            document.replace_properties(&[(first, &[3]), (invalid, value)]),
            Err(Error::InvalidProperty)
        );
    }
    let too_many = vec![(first, value); MAX_RECORDS + 1];
    assert_eq!(
        document.replace_properties(&too_many),
        Err(Error::RecordLimit)
    );
    let output = document.replace_properties(&[(first, &[0])])?;
    let edited = Document::parse(&output)?;
    assert_eq!(
        edited.replace_properties(&[(first, value)]),
        Err(Error::InvalidProperty)
    );
    assert_eq!(document.as_bytes(), UNKNOWN);
    assert_eq!(other.as_bytes(), UNKNOWN);
    Ok(())
}

#[test]
fn final_byte_budget_allows_growth_balanced_by_a_later_shrink() -> Result {
    let document = Document::parse(UNKNOWN)?;
    let first = document.records().first().ok_or("first property")?;
    let last = document.records().get(2).ok_or("extended property")?;
    // 70 original bytes minus records of 7 and 11 bytes, plus new prefixes of 8 and 4.
    let too_large = vec![0x8f; MAX_INPUT_BYTES - 63];
    let length_at_limit = MAX_INPUT_BYTES - 64;
    let at_limit = too_large.get(..length_at_limit).ok_or("test payload")?;
    assert_eq!(
        document.replace_property(first, at_limit),
        Err(Error::InputTooLarge)
    );
    let output = document.replace_properties(&[(first, at_limit), (last, &[])])?;
    assert_eq!(output.len(), MAX_INPUT_BYTES);
    let parsed = Document::parse(&output)?;
    assert_eq!(
        parsed.payload(parsed.records().first().ok_or("larger property")?)?,
        at_limit
    );
    assert_eq!(
        parsed.payload(parsed.records().get(2).ok_or("smaller property")?)?,
        &[]
    );
    assert_eq!(parsed.framing(), document.framing());
    drop(output);
    assert_eq!(
        document.replace_properties(&[(last, &[]), (first, &too_large)]),
        Err(Error::InputTooLarge)
    );
    assert_eq!(document.as_bytes(), UNKNOWN);
    Ok(())
}
