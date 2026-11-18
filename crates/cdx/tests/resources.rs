//! Original literal table vectors declared before the resource query exists.
use bondline_cdx::{Document, Record, RecordKind, ResourceError};
use serde_json::{Value, json};
use std::ptr;

#[path = "../../../xtask/oracles/cdx_resource_support.rs"]
mod f;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const FONT: u16 = 0x0100;
const COLOR: u16 = 0x0300;
const HEADER: &[u8] = b"VjCD0100\x04\x03\x02\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

fn property(tag: u16, payload: &[u8]) -> Vec<u8> {
    assert!(payload.len() < usize::from(u16::MAX));
    let mut raw = tag.to_le_bytes().to_vec();
    raw.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    raw.extend_from_slice(payload);
    raw
}

fn object(tag: u16, id: u32, children: &[Vec<u8>]) -> Vec<u8> {
    let mut raw = tag.to_le_bytes().to_vec();
    raw.extend_from_slice(&id.to_le_bytes());
    for child in children {
        raw.extend_from_slice(child);
    }
    raw.extend_from_slice(&[0, 0]);
    raw
}

fn implied(fields: &[Vec<u8>]) -> Vec<u8> {
    let mut raw = HEADER.to_vec();
    for field in fields {
        raw.extend_from_slice(field);
    }
    raw.extend_from_slice(&[0, 0]);
    raw
}

fn fonts(platform: u16, entries: &[(u16, u16, &[u8])]) -> Vec<u8> {
    let mut raw = platform.to_le_bytes().to_vec();
    raw.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    for &(id, charset, name) in entries {
        raw.extend_from_slice(&id.to_le_bytes());
        raw.extend_from_slice(&charset.to_le_bytes());
        raw.extend_from_slice(&(name.len() as u16).to_le_bytes());
        raw.extend_from_slice(name);
    }
    raw
}

fn colors(entries: &[[u16; 3]]) -> Vec<u8> {
    let mut raw = (entries.len() as u16).to_le_bytes().to_vec();
    for rgb in entries {
        for value in rgb {
            raw.extend_from_slice(&value.to_le_bytes());
        }
    }
    raw
}

fn record<'doc>(document: &'doc Document<'_>, tag: u16) -> Result<&'doc Record> {
    document
        .records()
        .iter()
        .find(|record| matches!(record.kind(), RecordKind::Property { tag: actual, .. } if *actual == tag))
        .ok_or_else(|| "table record absent".into())
}

#[test]
fn absent_resources_have_no_font_and_only_the_four_published_color_defaults() -> Result {
    let raw = implied(&[]);
    let document = Document::parse(&raw)?;
    let view = document.resources()?;
    assert!(view.font_table().is_none());
    assert!(view.color_table().is_none());
    assert_eq!(view.font_platform(), None);
    assert!(view.fonts().is_empty() && view.colors().is_empty());
    for id in [0, 7, u16::MAX] {
        assert!(
            matches!(view.font_by_id(id), Err(ResourceError::MissingFont(actual)) if actual == id)
        );
    }
    for (index, rgb) in [
        (0, [0; 3]),
        (1, [u16::MAX; 3]),
        (2, [u16::MAX; 3]),
        (3, [0; 3]),
    ] {
        assert_eq!(view.color(index)?, rgb);
    }
    for index in [4, 65_536, u32::MAX] {
        assert!(
            matches!(view.color(index), Err(ResourceError::MissingColor(actual)) if actual == index)
        );
    }
    assert_eq!(document.to_bytes(), raw);
    Ok(())
}

#[test]
fn present_empty_tables_never_activate_absent_table_defaults() -> Result {
    for platform in [0, 1] {
        let raw = implied(&[
            property(COLOR, &colors(&[])),
            property(FONT, &fonts(platform, &[])),
        ]);
        let document = Document::parse(&raw)?;
        let view = document.resources()?;
        assert_eq!(view.font_platform(), Some(platform));
        assert!(view.fonts().is_empty() && view.colors().is_empty());
        let font = record(&document, FONT)?;
        let color = record(&document, COLOR)?;
        assert!(view.font_table().is_some_and(|r| ptr::eq(r, font)));
        assert!(view.color_table().is_some_and(|r| ptr::eq(r, color)));
        assert!(matches!(
            view.font_by_id(0),
            Err(ResourceError::MissingFont(0))
        ));
        assert_eq!(view.color(0)?, [0; 3]);
        assert_eq!(view.color(1)?, [u16::MAX; 3]);
        for index in [2, 3, 4, 65_536] {
            assert!(
                matches!(view.color(index), Err(ResourceError::MissingColor(actual)) if actual == index)
            );
        }
        assert_eq!(document.as_bytes(), raw);
        assert_eq!(document.to_bytes(), raw);
    }
    Ok(())
}

#[test]
fn owner_and_duplicate_table_checks_precede_either_malformed_payload() -> Result {
    for tag in [FONT, COLOR] {
        let raw = implied(&[property(tag, &[]), property(tag, &[])]);
        let document = Document::parse(&raw)?;
        assert!(
            matches!(document.resources(), Err(ResourceError::DuplicateTable { tag: actual, record: 1 }) if actual == tag)
        );
        assert_eq!(document.to_bytes(), raw);
    }
    let raw = implied(&[
        property(FONT, &[]),
        object(0x8001, 90, &[property(COLOR, &[])]),
    ]);
    let document = Document::parse(&raw)?;
    assert!(matches!(
        document.resources(),
        Err(ResourceError::UnsupportedTable {
            tag: COLOR,
            record: 2
        })
    ));
    assert_eq!(document.to_bytes(), raw);
    Ok(())
}

#[test]
fn sparse_unsigned_font_fields_and_names_are_borrowed_with_original_entry_spans() -> Result {
    let entries: &[(u16, u16, &[u8])] = &[
        (65535, 0, b"Original Sans"),
        (0, 32768, b""),
        (7, 65535, b"\xff\0\x80"),
    ];
    let raw = implied(&[
        property(FONT, &fonts(1, entries)),
        object(0x8001, 90, &[object(0x8003, 51, &[object(0x8004, 7, &[])])]),
    ]);
    let document = Document::parse(&raw)?;
    let view = document.resources()?;
    assert_eq!(view.font_platform(), Some(1));
    assert_eq!(
        view.fonts().iter().map(|f| f.id()).collect::<Vec<_>>(),
        [65535, 0, 7]
    );
    let table = record(&document, FONT)?;
    let mut start = HEADER.len() + 4 + 4;
    for (ordinal, (font, &(id, charset, name))) in view.fonts().iter().zip(entries).enumerate() {
        let end = start + 6 + name.len();
        assert_eq!(
            (font.id(), font.charset(), font.ordinal()),
            (id, charset, ordinal)
        );
        assert_eq!(font.range(), start..end);
        assert!(ptr::eq(font.record(), table));
        assert!(ptr::eq(view.font_by_id(id)?, font));
        assert_eq!(font.name_bytes(), name);
        assert!(ptr::eq(
            font.name_bytes().as_ptr(),
            raw[start + 6..end].as_ptr()
        ));
        start = end;
    }
    assert!(matches!(
        view.font_by_id(1),
        Err(ResourceError::MissingFont(1))
    ));
    assert_eq!(document.to_bytes(), raw);
    Ok(())
}

#[test]
fn stored_color_indices_do_not_replace_reserved_values_or_invent_a_short_foreground() -> Result {
    for entries in [
        &[[123, 32768, 65534]][..],
        &[[65535, 0, 123], [13, 29, 41]][..],
    ] {
        let raw = implied(&[property(COLOR, &colors(entries))]);
        let document = Document::parse(&raw)?;
        let view = document.resources()?;
        let table = record(&document, COLOR)?;
        assert_eq!(view.color(0)?, [0; 3]);
        assert_eq!(view.color(1)?, [u16::MAX; 3]);
        for (offset, color) in view.colors().iter().enumerate() {
            let index = u32::try_from(offset)? + 2;
            let start = HEADER.len() + 4 + 2 + offset * 6;
            assert_eq!((color.index(), color.rgb()), (index, entries[offset]));
            assert_eq!(color.range(), start..start + 6);
            assert!(ptr::eq(color.record(), table));
            assert_eq!(view.color(index)?, entries[offset]);
        }
        let next = u32::try_from(entries.len())? + 2;
        assert!(
            matches!(view.color(next), Err(ResourceError::MissingColor(actual)) if actual == next)
        );
        assert_eq!(document.to_bytes(), raw);
    }
    Ok(())
}

#[test]
fn unknown_user_and_object_tags_are_not_masked_into_resource_tables() -> Result {
    let raw = implied(&[
        property(0x4100, &[255]),
        property(0x4300, &[254]),
        object(0x8100, 7, &[]),
        object(0x8300, 8, &[]),
    ]);
    let document = Document::parse(&raw)?;
    let view = document.resources()?;
    assert!(view.font_table().is_none() && view.color_table().is_none());
    assert!(view.fonts().is_empty() && view.colors().is_empty());
    assert_eq!(view.color(2)?, [u16::MAX; 3]);
    assert_eq!(document.to_bytes(), raw);
    Ok(())
}

#[test]
fn encoded_document_resource_ownership_uses_record_zero_not_its_object_id() -> Result {
    for id in [7, u32::MAX] {
        let mut raw = HEADER.to_vec();
        raw.extend_from_slice(&object(
            0x8000,
            id,
            &[
                property(FONT, &fonts(0, &[])),
                property(COLOR, &colors(&[])),
            ],
        ));
        let document = Document::parse(&raw)?;
        assert!(
            matches!(document.records().first().map(Record::kind), Some(RecordKind::ObjectStart { tag: 0x8000, id: actual }) if *actual == id)
        );
        assert_eq!(record(&document, FONT)?.parent(), Some(0));
        let view = document.resources()?;
        assert_eq!(view.font_platform(), Some(0));
        assert!(view.fonts().is_empty() && view.colors().is_empty());
        assert!(matches!(view.color(2), Err(ResourceError::MissingColor(2))));
        assert_eq!(document.to_bytes(), raw);
    }
    Ok(())
}

fn integer(value: &Value) -> Result<usize> {
    Ok(usize::try_from(
        value.as_u64().ok_or("expected unsigned integer")?,
    )?)
}

fn range(value: &Value) -> Result<std::ops::Range<usize>> {
    Ok(integer(&value[0])?..integer(&value[1])?)
}

fn error_value(error: ResourceError) -> Value {
    match error {
        ResourceError::Source(source) => json!({"kind":"Source","message":source.to_string()}),
        ResourceError::UnsupportedTable { tag, record } => {
            json!({"kind":"UnsupportedTable","tag":tag,"record":record})
        }
        ResourceError::DuplicateTable { tag, record } => {
            json!({"kind":"DuplicateTable","tag":tag,"record":record})
        }
        ResourceError::InvalidTable { tag, record } => {
            json!({"kind":"InvalidTable","tag":tag,"record":record})
        }
        ResourceError::UnsupportedPlatform { record, value } => {
            json!({"kind":"UnsupportedPlatform","record":record,"value":value})
        }
        ResourceError::DuplicateFontId {
            record,
            id,
            ordinal,
        } => json!({"kind":"DuplicateFontId","record":record,"id":id,"ordinal":ordinal}),
        ResourceError::MissingFont(id) => json!({"kind":"MissingFont","id":id}),
        ResourceError::MissingColor(index) => json!({"kind":"MissingColor","index":index}),
        ResourceError::Allocation => json!({"kind":"Allocation"}),
    }
}

fn handle(document: &Document<'_>, record: Option<&Record>) -> Result<Value> {
    let Some(record) = record else {
        return Ok(Value::Null);
    };
    let index = document
        .records()
        .iter()
        .position(|r| ptr::eq(r, record))
        .ok_or("foreign table record")?;
    let RecordKind::Property { tag, payload, .. } = record.kind() else {
        return Err("table must be property".into());
    };
    Ok(
        json!({"record":index,"tag":tag,"range":[record.range().start,record.range().end],"payload_range":[payload.start,payload.end]}),
    )
}

fn check_font(
    font: &bondline_cdx::Font<'_, '_>,
    expected: &Value,
    document: &Document<'_>,
) -> Result {
    assert_eq!(usize::from(font.id()), integer(&expected["id"])?);
    assert_eq!(usize::from(font.charset()), integer(&expected["charset"])?);
    assert_eq!(font.ordinal(), integer(&expected["ordinal"])?);
    assert_eq!(font.range(), range(&expected["range"])?);
    assert!(ptr::eq(
        font.record(),
        document
            .records()
            .get(integer(&expected["record"])?)
            .ok_or("font record")?
    ));
    let name_range = range(&expected["name_range"])?;
    let original_name = document.as_bytes().get(name_range).ok_or("original name")?;
    assert_eq!(
        font.name_bytes().len(),
        integer(&expected["name"]["bytes"])?
    );
    assert_eq!(font.name_bytes(), original_name);
    assert!(ptr::eq(font.name_bytes().as_ptr(), original_name.as_ptr()));
    Ok(())
}

fn check_view(document: &Document<'_>, query: &Value) -> Result {
    let view = document.resources()?;
    assert_eq!(handle(document, view.font_table())?, query["font_table"]);
    assert_eq!(handle(document, view.color_table())?, query["color_table"]);
    assert_eq!(json!(view.font_platform()), query["font_platform"]);
    assert_eq!(view.fonts().len(), integer(&query["font_count"])?);
    assert_eq!(view.colors().len(), integer(&query["color_count"])?);
    for font in query["fonts"].as_array().ok_or("font descriptors")? {
        check_font(
            view.fonts()
                .get(integer(&font["ordinal"])?)
                .ok_or("font ordinal")?,
            font,
            document,
        )?;
    }
    for color in query["colors"].as_array().ok_or("color descriptors")? {
        let ordinal = integer(&color["index"])? - 2;
        let actual = view.colors().get(ordinal).ok_or("color ordinal")?;
        assert_eq!(json!(actual.index()), color["index"]);
        assert_eq!(json!(actual.rgb()), color["rgb"]);
        assert_eq!(actual.range(), range(&color["range"])?);
        assert!(ptr::eq(
            actual.record(),
            document
                .records()
                .get(integer(&color["record"])?)
                .ok_or("color record")?
        ));
    }
    for lookup in query["font_lookups"].as_array().ok_or("font lookups")? {
        let id = u16::try_from(integer(&lookup["id"])?)?;
        if lookup.get("error").is_some() {
            assert_eq!(
                error_value(view.font_by_id(id).err().ok_or("missing font accepted")?),
                lookup["error"]
            );
        } else {
            let actual = view.font_by_id(id)?;
            check_font(actual, &lookup["font"], document)?;
            assert!(ptr::eq(
                actual,
                view.fonts()
                    .get(actual.ordinal())
                    .ok_or("lookup source ordinal")?
            ));
        }
    }
    for lookup in query["color_lookups"].as_array().ok_or("color lookups")? {
        let index = u32::try_from(integer(&lookup["index"])?)?;
        if lookup.get("error").is_some() {
            assert_eq!(
                error_value(view.color(index).err().ok_or("missing color accepted")?),
                lookup["error"]
            );
        } else {
            assert_eq!(json!(view.color(index)?), lookup["rgb"]);
        }
    }
    Ok(())
}

#[test]
fn all_frozen_original_sources_match_the_independent_full_record_and_query_facts() -> Result {
    let facts: Value =
        serde_json::from_str(include_str!("fixtures/resources/resource-facts.json"))?;
    let mut positives = 0;
    let mut negatives = 0;
    for (name, expected) in facts.as_object().ok_or("original facts")? {
        let raw = f::source(name);
        assert_eq!(f::read(raw)?, *expected, "independent source {name}");
        let document = Document::parse(raw)?;
        assert_eq!(
            format!("{:?}", document.framing()),
            expected["framing"].as_str().ok_or("framing")?
        );
        let query = &expected["query"];
        if query["ok"] == true {
            positives += 1;
            check_view(&document, query)?;
        } else {
            negatives += 1;
            assert_eq!(
                error_value(document.resources().err().ok_or("invalid table accepted")?),
                query["error"],
                "{name}"
            );
        }
        assert!(ptr::eq(document.as_bytes().as_ptr(), raw.as_ptr()));
        assert_eq!(document.as_bytes(), raw);
        assert_eq!(document.to_bytes(), raw);
        for record in document.records().iter().filter(|r| {
            matches!(
                r.kind(),
                RecordKind::Property {
                    tag: FONT | COLOR,
                    ..
                }
            )
        }) {
            assert_eq!(
                document.replace_property(record, document.payload(record)?)?,
                raw,
                "query does not alter same-payload writer {name}"
            );
        }
    }
    assert_eq!((positives, negatives), (17, 31));
    Ok(())
}

#[test]
fn every_nonempty_positive_and_every_query_error_is_individually_checked() -> Result {
    for name in [
        "tables_implicit",
        "tables_explicit",
        "tables_header",
        "tables_implicit_file_end",
        "tables_explicit_file_end",
        "tables_reordered",
        "tables_extended",
        "palette_one",
        "palette_nondefault",
        "max_name",
        "max_fonts",
        "max_colors",
    ] {
        let document = Document::parse(f::source(name))?;
        check_view(&document, &f::facts(name)["query"])?;
    }
    let facts: Value =
        serde_json::from_str(include_str!("fixtures/resources/resource-facts.json"))?;
    for (name, expected) in facts.as_object().ok_or("facts")? {
        if expected["query"]["ok"] == false {
            let document = Document::parse(f::source(name))?;
            assert_eq!(
                error_value(document.resources().err().ok_or("query error missing")?),
                expected["query"]["error"],
                "{name}"
            );
        }
    }
    Ok(())
}

#[test]
fn all_maximum_font_and_color_entries_preserve_source_order_raw_fields_and_u32_indices() -> Result {
    let document = Document::parse(f::source("max_fonts"))?;
    let view = document.resources()?;
    let facts = f::facts("max_fonts");
    let start = integer(&facts["query"]["font_entries_range"][0])?;
    let ids = [0, u16::MAX]
        .into_iter()
        .chain((1..u16::MAX).filter(|&id| id != 32768));
    assert_eq!(view.fonts().len(), 65535);
    for (ordinal, (font, id)) in view.fonts().iter().zip(ids).enumerate() {
        assert_eq!(
            (font.id(), font.charset(), font.ordinal()),
            (id, [0, 32768, 65535][ordinal % 3], ordinal)
        );
        assert!(font.name_bytes().is_empty());
        assert_eq!(font.range(), start + ordinal * 6..start + (ordinal + 1) * 6);
    }
    assert!(matches!(
        view.font_by_id(32768),
        Err(ResourceError::MissingFont(32768))
    ));
    let document = Document::parse(f::source("max_colors"))?;
    let view = document.resources()?;
    let facts = f::facts("max_colors");
    let start = integer(&facts["query"]["color_entries_range"][0])?;
    assert_eq!(view.colors().len(), 65535);
    for (ordinal, color) in view.colors().iter().enumerate() {
        let i = u32::try_from(ordinal)?;
        let rgb = [
            ((73 * i + 11) & 65535) as u16,
            ((257 * i + 32768) & 65535) as u16,
            (65535 - i) as u16,
        ];
        assert_eq!((color.index(), color.rgb()), (i + 2, rgb));
        assert_eq!(
            color.range(),
            start + ordinal * 6..start + (ordinal + 1) * 6
        );
        assert_eq!(view.color(i + 2)?, rgb);
    }
    assert_eq!(view.colors().last().ok_or("last color")?.index(), 65536);
    assert!(matches!(
        view.color(65537),
        Err(ResourceError::MissingColor(65537))
    ));
    Ok(())
}

#[test]
fn equal_logical_tables_keep_distinct_record_owners_and_foreign_handles_cannot_edit() -> Result {
    let first = Document::parse(f::source("tables_implicit"))?;
    let other = Document::parse(f::source("tables_extended"))?;
    let a = first.resources()?;
    let b = other.resources()?;
    for (left, right) in a.fonts().iter().zip(b.fonts()) {
        assert_eq!(
            (left.id(), left.charset(), left.name_bytes()),
            (right.id(), right.charset(), right.name_bytes())
        );
        assert!(!ptr::eq(left.record(), right.record()));
        assert!(matches!(
            first.replace_property(right.record(), &[]),
            Err(bondline_cdx::Error::InvalidProperty)
        ));
    }
    assert_eq!(
        a.colors().iter().map(|c| c.rgb()).collect::<Vec<_>>(),
        b.colors().iter().map(|c| c.rgb()).collect::<Vec<_>>()
    );
    assert_ne!(first.as_bytes(), other.as_bytes());
    Ok(())
}

#[test]
fn generic_palette_replacement_is_independently_readable_without_changing_the_source() -> Result {
    let original = f::source("tables_implicit");
    let expected = f::facts("tables_implicit");
    let document = Document::parse(original)?;
    let record = record(&document, COLOR)?;
    let mut replacement = document.payload(record)?.to_vec();
    replacement
        .get_mut(2..8)
        .ok_or("first RGB")?
        .copy_from_slice(&[1, 0, 2, 0, 3, 0]);
    let mut exact = original.to_vec();
    let start = integer(&expected["query"]["color_entries_range"][0])?;
    exact
        .get_mut(start..start + 6)
        .ok_or("independent first RGB span")?
        .copy_from_slice(&[1, 0, 2, 0, 3, 0]);
    let edited = document.replace_property(record, &replacement)?;
    assert_eq!(edited, exact);
    let external = f::read(&edited)?;
    assert_eq!(external["query"]["colors"][0]["rgb"], json!([1, 2, 3]));
    assert_eq!(external["query"]["fonts"], expected["query"]["fonts"]);
    let reparsed = Document::parse(&edited)?;
    assert_eq!(reparsed.resources()?.color(2)?, [1, 2, 3]);
    assert_eq!(document.as_bytes(), original);
    assert_eq!(document.to_bytes(), original);
    Ok(())
}
