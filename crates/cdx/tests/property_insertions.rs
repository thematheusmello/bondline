//! Independent byte constructions for combined CDX insertions and replacements.
use bondline_cdx::{
    Document, Error, Framing, MAX_INPUT_BYTES, MAX_RECORDS, PropertyInsertion, Record, RecordKind,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const UNKNOWN: &[u8] = include_bytes!("fixtures/unknown-implicit.cdx");
const ELEMENT: u16 = 0x0402;
const HYDROGENS: u16 = 0x042b;
const FLAG: u16 = 0x4066;
const OPAQUE: u16 = 0x4123;

fn property(tag: u16, value: &[u8], extended: bool) -> Vec<u8> {
    let mut raw = Vec::from(tag.to_le_bytes());
    if extended {
        raw.extend_from_slice(&[0xff, 0xff]);
        raw.extend_from_slice(&(value.len() as u32).to_le_bytes());
    } else {
        raw.extend_from_slice(&(value.len() as u16).to_le_bytes());
    }
    raw.extend_from_slice(value);
    raw
}

fn object(tag: u16, id: u32, body: &[Vec<u8>]) -> Vec<u8> {
    let mut raw = Vec::from(tag.to_le_bytes());
    raw.extend_from_slice(&id.to_le_bytes());
    raw.extend(body.iter().flatten());
    raw.extend_from_slice(&[0, 0]);
    raw
}

fn scene(framing: Framing, edited: bool) -> Vec<u8> {
    let mut target = vec![property(OPAQUE, b"old bytes\0\xfe", true)];
    if edited {
        target.push(property(ELEMENT, &[14, 0], false));
        target.push(property(FLAG, &[], false));
    }
    target.push(property(HYDROGENS, &[u8::from(edited), 0], !edited));
    target.push(object(0x8006, 12, &[property(0x0700, b"owned", true)]));
    let peer = [
        property(ELEMENT, &[8, 0], true),
        property(FLAG, b"same tag in a different object", true),
        property(HYDROGENS, &[0, 0], false),
    ];
    let page = object(
        0x8001,
        4,
        &[
            object(0x8003, 7, &[object(0x8004, 91, &target)]),
            object(0x8003, 8, &[object(0x8004, 91, &peer)]),
        ],
    );
    let fields = [property(0x4012, b"root", true), page];
    let mut raw = b"VjCD0100\x04\x03\x02\x01".to_vec();
    raw.resize(28, 0);
    match framing {
        Framing::ImplicitDocument | Framing::ExplicitDocument | Framing::HeaderDocument => {
            raw[12] = 0x73;
        }
        _ => {}
    }
    match framing {
        Framing::ImplicitDocument | Framing::ImplicitDocumentFileEnd => {
            raw.extend(fields.into_iter().flatten());
            raw.extend_from_slice(&[0, 0]);
        }
        Framing::ExplicitDocument | Framing::ExplicitDocumentFileEnd => {
            raw.extend(object(0x8000, 1, &fields));
        }
        Framing::HeaderDocument => {
            raw.truncate(22);
            raw.extend(object(0x8000, 1, &fields));
        }
    }
    if matches!(
        framing,
        Framing::HeaderDocument
            | Framing::ImplicitDocumentFileEnd
            | Framing::ExplicitDocumentFileEnd
    ) {
        raw.extend_from_slice(&[0, 0]);
    }
    raw
}

fn node(document: &Document<'_>, fragment: u32) -> Result<usize> {
    let records = document.records();
    let fragment = records
        .iter()
        .position(|record| {
            matches!(record.kind(), RecordKind::ObjectStart { tag: 0x8003, id } if *id == fragment)
        })
        .ok_or("missing scoped fragment")?;
    records
        .iter()
        .position(|record| {
            record.parent() == Some(fragment)
                && matches!(
                    record.kind(),
                    RecordKind::ObjectStart {
                        tag: 0x8004,
                        id: 91
                    }
                )
        })
        .ok_or_else(|| "missing scoped Node".into())
}

fn field<'a>(document: &'a Document<'_>, node: usize, tag: u16) -> Result<&'a Record> {
    document
        .records()
        .iter()
        .find(|record| {
            record.parent() == Some(node)
                && matches!(record.kind(), RecordKind::Property { tag: found, .. } if *found == tag)
        })
        .ok_or_else(|| format!("missing {tag:#06x} on record {node}").into())
}

fn new_property<'a>(before: &'a Record, tag: u16, value: &'a [u8]) -> PropertyInsertion<'a, 'a> {
    PropertyInsertion { before, tag, value }
}

#[test]
fn scoped_insertions_compose_with_replacing_their_anchor_in_every_framing() -> Result {
    for framing in [
        Framing::ImplicitDocument,
        Framing::ExplicitDocument,
        Framing::HeaderDocument,
        Framing::ImplicitDocumentFileEnd,
        Framing::ExplicitDocumentFileEnd,
    ] {
        let original = scene(framing, false);
        let expected = scene(framing, true);
        let document = Document::parse(&original)?;
        assert_eq!(document.framing(), framing);
        let target = node(&document, 7)?;
        let anchor = field(&document, target, HYDROGENS)?;
        let untouched = field(&document, target, OPAQUE)?;
        let changes = [
            (anchor, &[1, 0][..]),
            (untouched, document.payload(untouched)?),
        ];
        let insertions = [
            new_property(anchor, FLAG, &[]),
            new_property(anchor, ELEMENT, &[14, 0]),
        ];
        let output = document.edit_properties(&changes, &insertions)?;
        assert_eq!(output, expected, "{framing:?}");
        assert_eq!(
            document.edit_properties(&[changes[1], changes[0]], &[insertions[1], insertions[0]])?,
            expected
        );
        let parsed = Document::parse(&output)?;
        assert_eq!(parsed.framing(), framing);
        assert_eq!(document.header(), parsed.header());
        assert_eq!(parsed.records().len(), document.records().len() + 2);
        assert_eq!(
            parsed.payload(field(&parsed, node(&parsed, 7)?, ELEMENT)?)?,
            &[14, 0]
        );
        assert_eq!(
            parsed.payload(field(&parsed, node(&parsed, 8)?, ELEMENT)?)?,
            &[8, 0]
        );
        assert_eq!(document.as_bytes(), original);
    }
    Ok(())
}

#[test]
fn insertions_refuse_conflicting_scopes_invalid_tags_and_unowned_anchors() -> Result {
    let original = scene(Framing::ExplicitDocumentFileEnd, false);
    let document = Document::parse(&original)?;
    let alternative = Document::parse(&original)?;
    let target = node(&document, 7)?;
    let hydrogens = field(&document, target, HYDROGENS)?;
    let opaque = field(&document, target, OPAQUE)?;
    let existing = new_property(opaque, HYDROGENS, &[0, 0]);
    let first = new_property(hydrogens, ELEMENT, &[14, 0]);
    for change in [
        [first, first],
        [first, new_property(opaque, ELEMENT, &[14, 0])],
        [first, existing],
    ] {
        assert_eq!(
            document.edit_properties(&[(hydrogens, &[1, 0])], &change),
            Err(Error::ConflictingEdits)
        );
    }
    for tag in [0, 0x8000, 0xffff] {
        assert_eq!(
            document.edit_properties(&[], &[new_property(hydrogens, tag, &[])]),
            Err(Error::InvalidTag)
        );
    }
    let foreign = field(&alternative, node(&alternative, 7)?, HYDROGENS)?;
    for bad_anchor in [
        foreign,
        document.records().get(target).ok_or("Node opening")?,
        document.records().last().ok_or("File end")?,
    ] {
        assert_eq!(
            document.edit_properties(&[], &[first, new_property(bad_anchor, FLAG, &[])]),
            Err(Error::InvalidProperty)
        );
    }
    assert_eq!(document.as_bytes(), original);
    Ok(())
}

#[test]
fn one_byte_budget_covers_extended_insertion_and_a_balancing_replacement() -> Result {
    let document = Document::parse(UNKNOWN)?;
    let first = document.records().first().ok_or("first property")?;
    let extended = document.records().get(2).ok_or("extended property")?;
    // The new property costs eight bytes plus payload. The old record shrinks 11 → 4.
    let at_limit = MAX_INPUT_BYTES - UNKNOWN.len() - 8 + 7;
    let too_long = vec![0x6e; at_limit + 1];
    let value = too_long.get(..at_limit).ok_or("inserted payload")?;
    let insertion = new_property(first, FLAG, value);
    assert_eq!(
        document.edit_properties(&[], &[insertion]),
        Err(Error::InputTooLarge)
    );
    assert_eq!(
        document.edit_properties(&[(extended, &[])], &[new_property(first, FLAG, &too_long)]),
        Err(Error::InputTooLarge)
    );
    let output = document.edit_properties(&[(extended, &[])], &[insertion])?;
    assert_eq!(output.len(), MAX_INPUT_BYTES);
    let parsed = Document::parse(&output)?;
    let inserted = parsed.records().first().ok_or("inserted property")?;
    assert_eq!(inserted.parent(), None);
    assert_eq!(parsed.payload(inserted)?, value);
    assert_eq!(
        output.get(28..36),
        Some(
            [
                FLAG.to_le_bytes().as_slice(),
                &[0xff, 0xff],
                &(at_limit as u32).to_le_bytes()
            ]
            .concat()
            .as_slice()
        )
    );
    assert_eq!(
        parsed.payload(parsed.records().get(3).ok_or("shortened record")?)?,
        &[]
    );
    Ok(())
}

#[test]
fn inserting_records_checks_final_index_budget_before_building_output() -> Result {
    let mut raw = b"VjCD0100\x04\x03\x02\x01".to_vec();
    raw.resize(28, 0);
    for _ in 0..MAX_RECORDS - 2 {
        raw.extend_from_slice(&[0x42, 0x44, 0, 0]);
    }
    raw.extend_from_slice(&[0, 0]);
    let document = Document::parse(&raw)?;
    let first = document.records().first().ok_or("first property")?;
    let first_insertion = new_property(first, FLAG, &[]);
    let output = document.edit_properties(&[], &[first_insertion])?;
    assert_eq!(Document::parse(&output)?.records().len(), MAX_RECORDS);
    assert_eq!(
        document.edit_properties(
            &[],
            &[first_insertion, new_property(first, ELEMENT, &[6, 0])]
        ),
        Err(Error::RecordLimit)
    );
    assert_eq!(document.as_bytes(), raw);
    Ok(())
}
