//! Independent CDXString declarations precede the typed query.
use bondline_cdx::{Document, Error, RecordKind, StringError, StyleRun};
use serde_json::Value;
use std::{ops::Range, ptr};

#[path = "../../../xtask/oracles/cdx_string_support.rs"]
mod fixture;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn integer(value: &Value) -> Result<usize> {
    Ok(usize::try_from(value.as_u64().ok_or("expected integer")?)?)
}

fn span(value: &Value) -> Result<Range<usize>> {
    Ok(integer(&value[0])?..integer(&value[1])?)
}

fn words(run: &StyleRun) -> [u16; 5] {
    [
        run.start(),
        run.font(),
        run.face(),
        run.size_twentieths(),
        run.color(),
    ]
}

#[test]
fn original_sources_match_the_independent_reader_and_every_raw_run() -> Result {
    let manifest: Value = serde_json::from_str(include_str!(
        "fixtures/raw-strings/data/string-vectors.json"
    ))?;
    for case in manifest["cases"].as_array().ok_or("missing source cases")? {
        let name = case["name"].as_str().ok_or("missing source name")?;
        let raw = fixture::source(name)?;
        let expected = fixture::facts(name)?;
        assert_eq!(raw.len(), integer(&case["bytes"])?);
        assert_eq!(expected["sha256"], case["sha256"]);
        let index = integer(&expected["query_record"])?;
        assert_eq!(fixture::read(&raw, index)?, expected, "{name}");

        let source = Document::parse(&raw)?;
        assert_eq!(
            format!("{:?}", source.framing()),
            expected["framing"].as_str().ok_or("missing framing")?
        );
        let record = source.records().get(index).ok_or("missing record")?;
        let query = &expected["query"];
        if !query["ok"].as_bool().ok_or("missing query expectation")? {
            assert_eq!(
                source.string(record).err(),
                Some(StringError::InvalidPayload { record: index }),
                "{name}"
            );
            assert_eq!(source.to_bytes(), raw);
            continue;
        }

        let view = source.string(record)?;
        assert!(ptr::eq(view.record(), record));
        let text_range = span(&query["text_range"])?;
        let text = raw.get(text_range.clone()).ok_or("invalid text span")?;
        assert_eq!(view.text_range(), text_range);
        assert_eq!(view.text_bytes(), text);
        assert!(ptr::eq(view.text_bytes(), text));

        let styles = span(&query["runs_range"])?;
        let runs = view.runs();
        assert_eq!(runs.len(), integer(&query["run_count"])?);
        let raw_runs = raw.get(styles.clone()).ok_or("invalid style span")?;
        assert_eq!(raw_runs.len(), runs.len() * 10);
        for (ordinal, (run, raw_run)) in runs.zip(raw_runs.chunks_exact(10)).enumerate() {
            let start = styles.start + ordinal * 10;
            assert_eq!(run.range(), start..start + 10, "{name} run {ordinal}");
            assert!(
                words(&run)
                    .into_iter()
                    .flat_map(u16::to_le_bytes)
                    .eq(raw_run.iter().copied()),
                "{name} run {ordinal}"
            );
        }
        for witness in query["runs"].as_array().ok_or("missing run witnesses")? {
            let ordinal = integer(&witness["ordinal"])?;
            let run = view.runs().nth(ordinal).ok_or("missing run witness")?;
            assert_eq!(run.range(), span(&witness["range"])?);
            for (field, value) in ["start", "font", "face", "size_twentieths", "color"]
                .into_iter()
                .zip(words(&run))
            {
                assert_eq!(usize::from(value), integer(&witness[field])?, "{name}");
            }
        }
        assert_eq!(source.to_bytes(), raw);
    }
    Ok(())
}

#[test]
fn string_queries_require_a_property_owned_by_the_exact_document() -> Result {
    let raw = fixture::source("plain_explicit_file_end")?;
    let expected = fixture::facts("plain_explicit_file_end")?;
    let index = integer(&expected["query_record"])?;
    let document = Document::parse(&raw)?;
    let other = Document::parse(&raw)?;
    assert_eq!(
        document.string(&other.records()[index]).err(),
        Some(StringError::Source(Error::InvalidProperty))
    );
    for record in document.records() {
        if !matches!(record.kind(), RecordKind::Property { .. }) {
            assert_eq!(
                document.string(record).err(),
                Some(StringError::Source(Error::InvalidProperty))
            );
        }
    }
    Ok(())
}

#[test]
fn string_text_borrows_the_input_even_after_the_index_is_dropped() -> Result {
    let raw = fixture::source("single_size_240")?;
    let index = integer(&fixture::facts("single_size_240")?["query_record"])?;
    let text = {
        let document = Document::parse(&raw)?;
        let record = document.records().get(index).ok_or("missing record")?;
        document.string(record)?.text_bytes()
    };
    assert_eq!(text, b"Bondline");
    Ok(())
}

#[test]
fn property_replacement_is_independently_readable_without_changing_the_original() -> Result {
    let raw = fixture::source("single_size_240")?;
    let expected = fixture::source("single_size_241")?;
    let index = integer(&fixture::facts("single_size_240")?["query_record"])?;
    let source = Document::parse(&raw)?;
    let destination = Document::parse(&expected)?;
    let record = source.records().get(index).ok_or("missing property")?;
    let changed = destination
        .records()
        .get(index)
        .ok_or("missing changed property")?;
    assert_eq!(
        source.replace_property(record, source.payload(record)?)?,
        raw
    );
    let edited = source.replace_property(record, destination.payload(changed)?)?;
    assert_eq!(edited, expected);
    assert_eq!(
        fixture::read(&edited, index)?,
        fixture::facts("single_size_241")?
    );
    assert_eq!(
        source
            .string(record)?
            .runs()
            .next()
            .ok_or("missing run")?
            .size_twentieths(),
        240
    );
    Ok(())
}
