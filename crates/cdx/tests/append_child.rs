//! Independently authored CDX byte and ownership witnesses for appending a leaf.
use bondline_cdx::{Document, Error, Framing, Record, RecordKind};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const IMPLICIT: &[u8] = include_bytes!("fixtures/child-insertion/implicit_before.cdx");
const HEADER_DOCUMENT: &[u8] = include_bytes!("fixtures/child-insertion/header_before.cdx");
const HEADER: &[u8] = b"VjCD0100\x04\x03\x02\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const OWNER: &[u8] = b"\x01\x80\x37\0\0\0";
const EMPTY_LEAF: &[u8] = b"\x32\xc1\x4a\0\0\0\0\0";
const LEAF_TAG: u16 = 0xc132;
const LEAF_PROPERTIES: &[(u16, &[u8])] = &[(0x4132, b"\0\x12"), (0x4133, b"")];

fn end_of<'doc>(document: &'doc Document<'_>, opening: usize) -> Result<&'doc Record> {
    document
        .records()
        .iter()
        .find(|record| {
            record.kind()
                == &RecordKind::ObjectEnd {
                    start: Some(opening),
                }
        })
        .ok_or_else(|| "matched closing record".into())
}

fn page_55(document: &Document<'_>) -> Result<usize> {
    document
        .records()
        .iter()
        .position(|record| {
            record.kind()
                == &RecordKind::ObjectStart {
                    tag: 0x8001,
                    id: 55,
                }
        })
        .ok_or_else(|| "opening of Page 55".into())
}

#[test]
fn append_matches_complete_independent_bytes_for_every_framing() -> Result {
    let cases: &[(Framing, usize, &[u8], &[u8])] = &[
        (
            Framing::ImplicitDocument,
            87,
            IMPLICIT,
            include_bytes!("fixtures/child-insertion/implicit_after.cdx"),
        ),
        (
            Framing::ExplicitDocument,
            93,
            include_bytes!("fixtures/child-insertion/explicit_before.cdx"),
            include_bytes!("fixtures/child-insertion/explicit_after.cdx"),
        ),
        (
            Framing::HeaderDocument,
            87,
            HEADER_DOCUMENT,
            include_bytes!("fixtures/child-insertion/header_after.cdx"),
        ),
        (
            Framing::ImplicitDocumentFileEnd,
            87,
            include_bytes!("fixtures/child-insertion/implicit_file_end_before.cdx"),
            include_bytes!("fixtures/child-insertion/implicit_file_end_after.cdx"),
        ),
        (
            Framing::ExplicitDocumentFileEnd,
            93,
            include_bytes!("fixtures/child-insertion/explicit_file_end_before.cdx"),
            include_bytes!("fixtures/child-insertion/explicit_file_end_after.cdx"),
        ),
    ];
    for &(framing, offset, before, after) in cases {
        let source = Document::parse(before)?;
        assert_eq!(source.framing(), framing);
        let page = page_55(&source)?;
        let close = end_of(&source, page)?;
        assert_eq!(close.range(), offset..offset + 2);
        let output = source.append_child(close, LEAF_TAG, 74, LEAF_PROPERTIES)?;
        assert_eq!(output, after, "{framing:?}");
        assert_eq!(source.as_bytes(), before);

        let reopened = Document::parse(&output)?;
        assert_eq!(reopened.framing(), framing);
        assert_eq!(reopened.header(), source.header());
        assert_eq!(reopened.records().len(), source.records().len() + 4);
        let inserted = reopened
            .records()
            .iter()
            .find(|record| record.range().start == offset)
            .ok_or("inserted child")?;
        assert_eq!(
            inserted.kind(),
            &RecordKind::ObjectStart {
                tag: LEAF_TAG,
                id: 74
            }
        );
        assert_eq!(inserted.parent(), Some(page));
        assert_eq!(end_of(&reopened, page)?.range(), offset + 18..offset + 20);
    }
    Ok(())
}

#[test]
fn only_the_documents_matched_encoded_owner_end_is_eligible() -> Result {
    let source = Document::parse(HEADER_DOCUMENT)?;
    let duplicate_parse = Document::parse(HEADER_DOCUMENT)?;
    let owner = page_55(&source)?;
    let close = end_of(&source, owner)?;
    let foreign = end_of(&duplicate_parse, owner)?;
    assert_eq!(foreign.range(), close.range());
    assert_eq!(foreign.kind(), close.kind());
    let opening = source.records().get(owner).ok_or("page opening")?;
    let property = source.records().get(owner + 1).ok_or("page property")?;
    let file_end = source.records().last().ok_or("file end")?;
    assert_eq!(file_end.kind(), &RecordKind::FileEnd);
    let implicit = Document::parse(IMPLICIT)?;
    let implicit_end = implicit.records().last().ok_or("implicit root end")?;
    assert_eq!(implicit_end.kind(), &RecordKind::ObjectEnd { start: None });
    for (document, wrong_end) in [
        (&source, foreign),
        (&source, opening),
        (&source, property),
        (&source, file_end),
        (&implicit, implicit_end),
    ] {
        assert_eq!(
            document.append_child(wrong_end, LEAF_TAG, 74, &[]),
            Err(Error::InvalidObjectEnd)
        );
    }

    let root_close = end_of(&source, 0)?;
    let offset = root_close.range().start;
    let output = source.append_child(root_close, LEAF_TAG, 74, &[])?;
    assert_eq!(
        output,
        [
            &HEADER_DOCUMENT[..offset],
            EMPTY_LEAF,
            &HEADER_DOCUMENT[offset..]
        ]
        .concat()
    );
    let reopened = Document::parse(&output)?;
    assert_eq!(reopened.framing(), Framing::HeaderDocument);
    let new_root_child = reopened
        .records()
        .iter()
        .find(|record| record.range().start == offset)
        .ok_or("new root child")?;
    assert_eq!(new_root_child.parent(), Some(0));
    assert_eq!(
        reopened
            .records()
            .last()
            .ok_or("unchanged file end")?
            .kind(),
        &RecordKind::FileEnd
    );
    Ok(())
}

#[test]
fn nonzero_identity_collisions_are_local_and_existing_opaque_data_survives() -> Result {
    let source = Document::parse(IMPLICIT)?;
    let page = page_55(&source)?;
    let close = end_of(&source, page)?;
    assert_eq!(
        source.append_child(close, LEAF_TAG, 73, LEAF_PROPERTIES),
        Err(Error::DuplicateObjectId(73))
    );

    let collision = include_bytes!("fixtures/child-insertion/duplicate_direct_id.cdx");
    let existing = Document::parse(collision)?;
    let close = end_of(&existing, page_55(&existing)?)?;
    let offset = close.range().start;
    let output = existing.append_child(close, LEAF_TAG, 74, &[])?;
    assert_eq!(
        output,
        [&collision[..offset], EMPTY_LEAF, &collision[offset..]].concat()
    );

    let misplaced = Document::parse(include_bytes!(
        "fixtures/child-insertion/after_page_end.cdx"
    ))?;
    let sibling = misplaced
        .records()
        .iter()
        .find(|record| {
            record.kind()
                == &RecordKind::ObjectStart {
                    tag: LEAF_TAG,
                    id: 74,
                }
        })
        .ok_or("syntactically valid misplaced sibling")?;
    assert_eq!(sibling.range(), 89..95);
    assert_eq!(sibling.parent(), None);
    assert_eq!(source.as_bytes(), IMPLICIT);
    Ok(())
}

#[test]
fn empty_owner_and_repeated_unassigned_ids_work() -> Result {
    let empty_page = [HEADER, OWNER, b"\0\0\0\0"].concat();
    let source = Document::parse(&empty_page)?;
    let anonymous_leaf = b"\x32\xc1\0\0\0\0\0\0";
    let first_close = end_of(&source, 0)?;
    assert_eq!(first_close.range(), 34..36);
    let first_output = source.append_child(first_close, LEAF_TAG, 0, &[])?;
    assert_eq!(
        first_output,
        [HEADER, OWNER, anonymous_leaf, b"\0\0\0\0"].concat()
    );
    let first = Document::parse(&first_output)?;
    let second_output = first.append_child(end_of(&first, 0)?, LEAF_TAG, 0, &[])?;
    assert_eq!(
        second_output,
        [HEADER, OWNER, anonymous_leaf, anonymous_leaf, b"\0\0\0\0"].concat()
    );
    let reopened = Document::parse(&second_output)?;
    assert_eq!(end_of(&reopened, 0)?.range(), 50..52);
    Ok(())
}

#[test]
fn tag_classes_and_property_order_obey_the_binary_grammar() -> Result {
    let empty_page = [HEADER, OWNER, b"\0\0\0\0"].concat();
    let source = Document::parse(&empty_page)?;
    let close = end_of(&source, 0)?;
    for invalid in [0, 1, 0x7fff] {
        assert_eq!(
            source.append_child(close, invalid, 74, &[]),
            Err(Error::InvalidTag)
        );
    }
    for invalid in [0, 0x8000, 0xffff] {
        assert_eq!(
            source.append_child(
                close,
                LEAF_TAG,
                74,
                &[(0x4132, b"valid".as_slice()), (invalid, b"")],
            ),
            Err(Error::InvalidTag)
        );
    }

    let short = vec![0xa7; 65_534];
    let extended = vec![0x59; 65_535];
    let output = source.append_child(
        close,
        LEAF_TAG,
        74,
        &[(0x4134, &short), (0x4131, &extended)],
    )?;
    assert_eq!(
        output,
        [
            HEADER,
            OWNER,
            b"\x32\xc1\x4a\0\0\0\x34\x41\xfe\xff",
            &short,
            b"\x31\x41\xff\xff\xff\xff\0\0",
            &extended,
            b"\0\0\0\0\0\0",
        ]
        .concat()
    );
    assert_eq!(source.as_bytes(), empty_page);
    Ok(())
}

#[test]
fn nesting_budget_counts_the_matched_owners_encoded_ancestry() -> Result {
    let encoded_root = b"\0\x80\x11\0\0\0";
    let nested = b"\x31\xc1\0\0\0\0";
    for depth in [255, 256] {
        let mut raw = [HEADER, encoded_root].concat();
        raw.extend(nested.repeat(depth - 1));
        let at = raw.len();
        raw.resize(at + depth * 2, 0);
        let source = Document::parse(&raw)?;
        let deepest = end_of(&source, depth - 1)?;
        assert_eq!(deepest.range(), at..at + 2);
        let output = source.append_child(deepest, LEAF_TAG, 74, &[]);
        if depth == 255 {
            assert_eq!(output?, [&raw[..at], EMPTY_LEAF, &raw[at..]].concat());
        } else {
            assert_eq!(output, Err(Error::DepthLimit));
            let root = end_of(&source, 0)?;
            let root_at = root.range().start;
            let output = source.append_child(root, LEAF_TAG, 74, &[])?;
            assert_eq!(
                output,
                [&raw[..root_at], EMPTY_LEAF, &raw[root_at..]].concat()
            );
        }
    }
    Ok(())
}

#[test]
fn record_budget_counts_the_child_ends_fields_and_existing_file_end() -> Result {
    let mut raw = [HEADER, OWNER].concat();
    raw.extend(b"\x60\x41\0\0".repeat(100_000 - 7));
    let at = raw.len();
    raw.extend_from_slice(b"\0\0\0\0\0\0");
    let source = Document::parse(&raw)?;
    assert_eq!(source.framing(), Framing::ImplicitDocumentFileEnd);
    assert_eq!(source.records().len(), 99_997);
    let close = end_of(&source, 0)?;
    assert_eq!(close.range(), at..at + 2);
    let output = source.append_child(close, LEAF_TAG, 74, &[(0x4132, &[])])?;
    let leaf = b"\x32\xc1\x4a\0\0\0\x32\x41\0\0\0\0";
    assert_eq!(output, [&raw[..at], leaf, &raw[at..]].concat());
    assert_eq!(Document::parse(&output)?.records().len(), 100_000);
    assert_eq!(
        source.append_child(close, LEAF_TAG, 74, LEAF_PROPERTIES),
        Err(Error::RecordLimit)
    );
    Ok(())
}

#[test]
fn output_size_budget_counts_extended_fields_and_all_terminators() -> Result {
    let empty_page = [HEADER, OWNER, b"\0\0\0\0"].concat();
    let source = Document::parse(&empty_page)?;
    let close = end_of(&source, 0)?;
    let limit = 64 * 1024 * 1024;
    // 38 source bytes, new 6-byte opening, 8-byte property prefix and 2-byte close.
    let at_limit = limit - 54;
    let payload = vec![0xa7; at_limit + 1];
    assert_eq!(
        source.append_child(close, LEAF_TAG, 74, &[(0x4132, &payload)]),
        Err(Error::InputTooLarge)
    );
    let payload = &payload[..at_limit];
    let output = source.append_child(close, LEAF_TAG, 74, &[(0x4132, payload)])?;
    assert_eq!(output.len(), limit);
    assert_eq!(&output[..34], [HEADER, OWNER].concat());
    assert_eq!(
        &output[34..48],
        b"\x32\xc1\x4a\0\0\0\x32\x41\xff\xff\xca\xff\xff\x03"
    );
    assert_eq!(&output[48..limit - 6], payload);
    assert_eq!(&output[limit - 6..], b"\0\0\0\0\0\0");
    assert_eq!(Document::parse(&output)?.records().len(), 6);
    Ok(())
}
