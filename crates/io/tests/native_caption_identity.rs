//! Independent native caption bytes; lexical foreign input keeps ordinal model IDs.
use bondline_io::StoredDocument;
use bondline_model::{PageSize, Point2, StyledText, TextId, TextRun, TextStyle};
use serde_json::Value;
use std::{fs, path::Path, sync::Arc};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const CASES: &str = include_str!("fixtures/caption-create/cases.json");

fn source(name: &str) -> TestResult<Vec<u8>> {
    Ok(fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/caption-create")
            .join(name),
    )?)
}

fn read(bytes: &[u8]) -> TestResult<StoredDocument> {
    Ok(StoredDocument::from_cdxml(Arc::from(bytes), PageSize::A4)?)
}

fn caption(id: u32, text: &str, y: f64) -> TestResult<StyledText> {
    Ok(StyledText::new(
        TextId::new(id).ok_or("positive Text ID")?,
        Point2::new(144.25, y)?,
        vec![TextRun::new(
            text.into(),
            TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
        )?],
    )?)
}

fn raw_text_ids(bytes: &[u8]) -> TestResult<Vec<u32>> {
    let source = bondline_cdxml::Document::parse(bytes)?;
    source
        .xml()
        .descendants()
        .filter(|node| {
            node.has_tag_name("t") && node.parent().is_some_and(|p| p.has_tag_name("page"))
        })
        .map(|node| Ok(node.attribute("id").ok_or("raw Text ID")?.parse()?))
        .collect()
}

#[test]
fn native_creation_writes_dense_and_burned_text_ids_as_independent_source_bytes() -> TestResult {
    let witnesses: Value = serde_json::from_str(CASES)?;
    for name in ["first", "after_undo", "mixed_append"] {
        let case = &witnesses["cases"][name];
        let before = source(case["before"].as_str().ok_or("before file")?)?;
        let after = source(case["after"].as_str().ok_or("after file")?)?;
        let mut document = read(&before)?;
        assert_eq!(
            serde_json::to_value(document.model())?,
            case["before_model"],
            "{name}"
        );
        let text: StyledText = serde_json::from_value(
            case["after_model"]["pages"][0]["texts"]
                .as_array()
                .and_then(|texts| texts.last())
                .ok_or("independent appended text")?
                .clone(),
        )?;
        let page = document.model().pages().first().ok_or("Page")?.id();
        document.model_mut().append_texts(page, vec![text])?;
        assert_eq!(
            serde_json::to_value(document.model())?,
            case["after_model"],
            "{name}"
        );
        assert_eq!(document.to_cdxml()?.as_ref(), after, "{name}");
    }
    Ok(())
}

#[test]
fn exact_native_sparse_source_reopens_with_its_complete_independent_text_identity() -> TestResult {
    let witnesses: Value = serde_json::from_str(CASES)?;
    for name in ["first", "after_undo", "mixed_append"] {
        let case = &witnesses["cases"][name];
        let bytes = source(case["after"].as_str().ok_or("after file")?)?;
        let document = read(&bytes)?;
        assert_eq!(
            serde_json::to_value(document.model())?,
            case["after_model"],
            "{name}"
        );
        assert!(document.is_fully_mapped(), "{name}");
        assert_eq!(document.export_cdxml()?, bytes, "{name}");
    }
    Ok(())
}

#[test]
fn producer_text_or_lexical_trivia_never_grants_sparse_identity_to_foreign_source() -> TestResult {
    let native = String::from_utf8(source("after_undo.cdxml")?)?;
    assert_eq!(raw_text_ids(native.as_bytes())?, [3]);
    for foreign in [
        native.replacen("Bondline 0.1.0", "Independent fixture", 1),
        native.replacen("<fonttable>", "<!-- retained original --> <fonttable>", 1),
        native.replacen("Width=\"512\"", "Width=\"512.0\"", 1),
    ] {
        assert_ne!(foreign, native);
        let document = read(foreign.as_bytes())?;
        let text = document
            .model()
            .pages()
            .first()
            .and_then(|page| page.texts().first())
            .ok_or("Text")?;
        assert_eq!(text.id().get(), 1);
        assert!(!document.is_fully_mapped());
        assert_eq!(document.export_cdxml()?, foreign.as_bytes());
    }
    Ok(())
}

#[test]
fn unrepresentable_existing_api_ids_keep_whole_document_legacy_export_without_collisions()
-> TestResult {
    let mut document = read(&source("blank.cdxml")?)?;
    let page = document.model().pages().first().ok_or("Page")?.id();
    document
        .model_mut()
        .append_texts(page, vec![caption(u32::MAX, "Phase β & q", 216.75)?])?;
    assert_eq!(document.to_cdxml()?.as_ref(), source("first.cdxml")?);
    document
        .model_mut()
        .append_texts(page, vec![caption(1, "Other", 260.0)?])?;
    let bytes = document.to_cdxml()?;
    assert_eq!(raw_text_ids(&bytes)?, [2, 3]);
    let reopened = read(&bytes)?;
    let texts = reopened.model().pages().first().ok_or("Page")?.texts();
    assert_eq!(texts.len(), 2);
    assert_eq!(
        (texts[0].id().get(), texts[0].runs()[0].text()),
        (1, "Phase β & q")
    );
    assert_eq!(
        (texts[1].id().get(), texts[1].runs()[0].text()),
        (2, "Other")
    );
    Ok(())
}

#[test]
fn maximum_representable_native_wire_id_keeps_its_checked_positive_text_id() -> TestResult {
    let bytes = String::from_utf8(source("after_undo.cdxml")?)?.replacen(
        "<t id=\"3\"",
        "<t id=\"4294967295\"",
        1,
    );
    assert_eq!(raw_text_ids(bytes.as_bytes())?, [u32::MAX]);
    let document = read(bytes.as_bytes())?;
    let text = document
        .model()
        .pages()
        .first()
        .and_then(|page| page.texts().first())
        .ok_or("Text")?;
    assert_eq!(text.id().get(), u32::MAX - 1);
    assert!(document.can_create_caption());
    assert_eq!(document.to_cdxml()?.as_ref(), bytes.as_bytes());
    Ok(())
}

#[test]
fn creation_admission_requires_exact_native_source_and_representable_plain_model_ids() -> TestResult
{
    let free = StoredDocument::default();
    assert!(free.can_create_caption());
    for name in [
        "blank.cdxml",
        "first.cdxml",
        "after_undo.cdxml",
        "mixed_before.cdxml",
        "mixed_after.cdxml",
    ] {
        let document = read(&source(name)?)?;
        assert!(document.is_fully_mapped(), "{name}");
        assert!(document.can_create_caption(), "{name}");
    }

    let blank = String::from_utf8(source("blank.cdxml")?)?;
    let preserved = blank.replacen("<page", "<!-- preserved lexical source --> <page", 1);
    let preserved = read(preserved.as_bytes())?;
    assert!(preserved.is_fully_mapped());
    assert!(!preserved.can_create_caption());

    let first = String::from_utf8(source("first.cdxml")?)?;
    let foreign = first.replacen(
        "<fonttable>",
        "<!-- literal foreign caption --> <fonttable>",
        1,
    );
    let foreign = read(foreign.as_bytes())?;
    assert!(foreign.has_caption_projection());
    assert!(!foreign.can_create_caption());

    let mut overflow = free.clone();
    let page = overflow.model().pages().first().ok_or("Page")?.id();
    overflow
        .model_mut()
        .append_texts(page, vec![caption(u32::MAX, "H", 216.75)?])?;
    assert!(overflow.is_fully_mapped());
    assert!(!overflow.can_create_caption());

    let boxed = read(include_bytes!("fixtures/dependent-labels/boxed.cdxml"))?;
    let mut labelled_free = free;
    *labelled_free.model_mut() = boxed.model().clone();
    assert!(labelled_free.model().has_authored_atom_labels());
    assert!(!labelled_free.can_create_caption());
    Ok(())
}
