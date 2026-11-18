use bondline_model::{
    Atom, AtomId, AtomLabel, Document, Element, Fragment, FragmentId, LabelAlignment, Point2,
    TextJustification, TextRun, TextStyle,
};
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn run(text: String) -> TestResult<TextRun> {
    Ok(TextRun::new(
        text,
        TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
    )?)
}

fn label(text: String) -> TestResult<Arc<AtomLabel>> {
    Ok(Arc::new(AtomLabel::new(
        None,
        None,
        None,
        None,
        vec![run(text)?],
    )?))
}

fn fragment(id: u32, first: u32, count: u32, label: Arc<AtomLabel>) -> TestResult<Fragment> {
    let atoms = (first..first + count)
        .map(|id| {
            let mut atom = Atom::new(AtomId::new(id).ok_or("atom")?, Element::try_from(6)?, None);
            atom.authored_label = Some(Arc::clone(&label));
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    Ok(Fragment::new(
        FragmentId::new(id).ok_or("fragment")?,
        atoms,
        vec![],
    )?)
}

fn caption(id: u32, text: String) -> TestResult<bondline_model::StyledText> {
    Ok(bondline_model::StyledText::new(
        bondline_model::TextId::new(id).ok_or("text")?,
        Point2::new(0.0, 0.0)?,
        vec![run(text)?],
    )?)
}

#[test]
fn authored_node_label_is_owned_separately_from_chemical_fields_and_captions()
-> Result<(), Box<dyn std::error::Error>> {
    let label = AtomLabel::new(
        Some(Point2::new(71.5, 96.25)?),
        Some(TextJustification::Left),
        Some(TextJustification::Auto),
        Some(LabelAlignment::Right),
        vec![TextRun::new(
            "OH".into(),
            TextStyle::new("Roboto".into(), 12.5, 96, [0.2, 0.4, 0.8])?,
        )?],
    )?;
    let mut atom = Atom::new(
        AtomId::new(1).ok_or("atom")?,
        Element::new(8).ok_or("element")?,
        Some(Point2::new(72.0, 96.0)?),
    );
    atom.explicit_hydrogens = Some(1);
    atom.authored_label = Some(Arc::new(label.clone()));
    let mut doc = Document::default();
    let page = doc.pages()[0].id();
    doc.append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    assert!(doc.has_authored_atom_labels());
    assert!(doc.pages()[0].texts().is_empty());
    assert_eq!(label.position(), Some(Point2::new(71.5, 96.25)?));
    assert_eq!(label.label_alignment(), Some(LabelAlignment::Right));
    assert_eq!(doc.pages()[0].fragments()[0].atoms()[0].element.get(), 8);
    assert_eq!(label.with_run_text(0, "N".into())?.runs()[0].text(), "N");
    assert_eq!(label.runs()[0].text(), "OH");
    Ok(())
}

#[test]
fn label_json_is_a_closed_map_with_distinct_string_enums_and_optional_raw_position() -> TestResult {
    use serde_json::{Value, json};
    let base = json!({"position":{"x_pt":-100000.0,"y_pt":100000.0},"justification":"full","label_justification":"best","label_alignment":"above","runs":[{"text":" λ\n&<😀 ","style":{"font_family":"Other","size_pt":0.25,"face":65535,"rgb":[0.0,0.25,1.0]}}]});
    let parsed: AtomLabel = serde_json::from_value(base.clone())?;
    assert_eq!(serde_json::to_value(parsed)?, base);
    for key in [
        "position",
        "justification",
        "label_justification",
        "label_alignment",
    ] {
        let mut omitted = base.clone();
        omitted.as_object_mut().ok_or("object")?.remove(key);
        let parsed: AtomLabel = serde_json::from_value(omitted.clone())?;
        omitted[key] = Value::Null;
        assert_eq!(serde_json::to_value(parsed)?, omitted);
    }
    for (key, value) in [
        ("position", json!([0, 0])),
        ("position", json!({"x_pt":0,"y_pt":0,"z":0})),
        ("position", json!({"x_pt":100001,"y_pt":0})),
        ("justification", json!("other")),
        ("justification", json!(-1)),
        ("justification", json!({"right":null})),
        ("label_justification", json!(5)),
        ("label_alignment", json!("full")),
        ("label_alignment", json!({"auto":null})),
        ("runs", json!([])),
        ("runs", Value::Null),
        ("extra", json!(true)),
    ] {
        let mut invalid = base.clone();
        invalid[key] = value;
        assert!(
            serde_json::from_value::<AtomLabel>(invalid.clone()).is_err(),
            "{invalid}"
        );
    }
    for invalid in [
        json!([]),
        json!([null, null, null, null, base["runs"]]),
        Value::Null,
        json!("label"),
    ] {
        assert!(serde_json::from_value::<AtomLabel>(invalid).is_err());
    }
    for spelling in [
        "right", "left", "center", "full", "above", "below", "auto", "best",
    ] {
        let mut value = base.clone();
        value["justification"] = json!(spelling);
        value["label_justification"] = json!(spelling);
        let parsed: AtomLabel = serde_json::from_value(value.clone())?;
        assert_eq!(serde_json::to_value(parsed)?, value);
    }
    for spelling in ["auto", "left", "center", "right", "above", "below", "best"] {
        let mut value = base.clone();
        value["label_alignment"] = json!(spelling);
        let parsed: AtomLabel = serde_json::from_value(value.clone())?;
        assert_eq!(serde_json::to_value(parsed)?, value);
    }
    let atom = Atom::new(AtomId::new(1).ok_or("atom")?, Element::try_from(6)?, None);
    let exact = r#"{"id":1,"element":6,"position":null,"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":null}"#;
    assert_eq!(serde_json::to_string(&atom)?, exact);
    assert_eq!(serde_json::from_str::<Atom>(exact)?, atom);
    let mut with_label = serde_json::to_value(&atom)?;
    with_label["authored_label"] = base;
    assert!(
        serde_json::from_value::<Atom>(with_label.clone())?
            .authored_label
            .is_some()
    );
    with_label["authored_label"]["runs"] = json!([]);
    assert!(serde_json::from_value::<Atom>(with_label).is_err());
    Ok(())
}

#[test]
fn labels_reuse_run_validation_and_bound_checked_copies_without_chemical_inference() -> TestResult {
    use bondline_model::{MAX_TEXT_BYTES, TextError};
    let empty = run(String::new())?;
    assert!(AtomLabel::new(None, None, None, None, vec![]).is_err());
    let full_runs = AtomLabel::new(None, None, None, None, vec![empty.clone(); 4096])?;
    assert!(matches!(
        AtomLabel::new(None, None, None, None, vec![empty; 4097]),
        Err(TextError::RunLimit)
    ));
    assert!(matches!(
        full_runs.with_run_text(4096, String::new()),
        Err(TextError::MissingRun(4096))
    ));
    assert!(matches!(
        full_runs.with_run_text(0, "\0".into()),
        Err(TextError::InvalidCharacter)
    ));
    let half = "λ".repeat(MAX_TEXT_BYTES / 4);
    let exact = AtomLabel::new(
        None,
        Some(TextJustification::Right),
        None,
        Some(LabelAlignment::Auto),
        vec![run(half.clone())?, run(half)?],
    )?;
    assert_eq!(
        exact
            .runs()
            .iter()
            .map(|run| run.text().len())
            .sum::<usize>(),
        MAX_TEXT_BYTES
    );
    assert!(matches!(
        exact.with_run_text(1, format!("{}x", exact.runs()[1].text())),
        Err(TextError::TextLimit)
    ));
    let mut too_long = exact.runs().to_vec();
    too_long.push(run("x".into())?);
    assert!(matches!(
        AtomLabel::new(None, None, None, None, too_long),
        Err(TextError::TextLimit)
    ));
    let moved = exact.with_position(Some(Point2::new(10.0, 20.0)?))?;
    assert_eq!(moved.runs(), exact.runs());
    assert_eq!(moved.justification(), Some(TextJustification::Right));
    assert_eq!(moved.label_alignment(), Some(LabelAlignment::Auto));
    assert_eq!(exact.position(), None);
    assert_eq!(moved.with_position(None)?, exact);
    Ok(())
}

#[test]
fn fragment_budgets_count_every_authored_label_even_when_payloads_share_one_arc() -> TestResult {
    use bondline_model::{GraphError, MAX_TEXT_BYTES, TextError};
    let many = Arc::new(AtomLabel::new(
        None,
        None,
        None,
        None,
        vec![run(String::new())?; 4000],
    )?);
    assert_eq!(fragment(1, 1, 25, Arc::clone(&many))?.atoms().len(), 25);
    let atoms = (1..=26)
        .map(|id| {
            let mut atom = Atom::new(AtomId::new(id).ok_or("atom")?, Element::try_from(6)?, None);
            atom.authored_label = Some(Arc::clone(&many));
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert!(matches!(
        Fragment::new(FragmentId::new(1).ok_or("fragment")?, atoms, vec![]),
        Err(GraphError::Text(TextError::DocumentRunLimit))
    ));
    let full = label("a".repeat(MAX_TEXT_BYTES))?;
    assert!(fragment(1, 1, 64, Arc::clone(&full)).is_ok());
    let atoms = (1..=65)
        .map(|id| {
            let mut atom = Atom::new(AtomId::new(id).ok_or("atom")?, Element::try_from(6)?, None);
            atom.authored_label = Some(Arc::clone(&full));
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert!(matches!(
        Fragment::new(FragmentId::new(1).ok_or("fragment")?, atoms, vec![]),
        Err(GraphError::Text(TextError::DocumentTextLimit))
    ));
    Ok(())
}

#[test]
fn caption_and_label_run_budgets_share_one_atomic_append_and_replace_boundary() -> TestResult {
    use bondline_model::{Error, PageId, TextError};
    let page = PageId::new(1).ok_or("page")?;
    let many = Arc::new(AtomLabel::new(
        None,
        None,
        None,
        None,
        vec![run(String::new())?; 4000],
    )?);
    let mut doc = Document::default();
    doc.append_fragments(page, vec![fragment(1, 1, 25, Arc::clone(&many))?])?;
    let before = doc.clone();
    assert!(matches!(
        doc.append_texts(page, vec![caption(1, String::new())?]),
        Err(Error::Text(TextError::DocumentRunLimit))
    ));
    assert_eq!(doc, before);
    assert!(Arc::ptr_eq(
        &doc.pages()[0].fragments()[0],
        &before.pages()[0].fragments()[0]
    ));
    let mut doc = Document::default();
    doc.append_texts(page, vec![caption(1, String::new())?])?;
    let before = doc.clone();
    assert!(matches!(
        doc.append_fragments(page, vec![fragment(1, 1, 25, Arc::clone(&many))?]),
        Err(Error::Text(TextError::DocumentRunLimit))
    ));
    assert_eq!(doc, before);
    assert!(Arc::ptr_eq(
        &doc.pages()[0].texts()[0],
        &before.pages()[0].texts()[0]
    ));
    doc.append_fragments(
        page,
        vec![
            fragment(1, 1, 24, Arc::clone(&many))?,
            fragment(
                2,
                25,
                1,
                Arc::new(AtomLabel::new(
                    None,
                    None,
                    None,
                    None,
                    vec![run(String::new())?; 3999],
                )?),
            )?,
        ],
    )?;
    let before = doc.clone();
    assert!(matches!(
        doc.replace_fragment(page, fragment(2, 25, 1, many)?),
        Err(Error::Text(TextError::DocumentRunLimit))
    ));
    assert_eq!(doc, before);
    Ok(())
}

#[test]
fn combined_object_limit_is_distinct_from_existing_graph_limits() -> TestResult {
    use bondline_model::{Error, PageId, TextError};
    let page = PageId::new(1).ok_or("page")?;
    let mut doc = Document::default();
    doc.append_fragments(page, vec![fragment(1, 1, 100000, label(String::new())?)?])?;
    let before = doc.clone();
    assert!(matches!(
        doc.append_texts(page, vec![caption(1, String::new())?]),
        Err(Error::Text(TextError::TextCountLimit))
    ));
    assert_eq!(doc, before);
    Ok(())
}

#[test]
fn combined_utf8_limit_is_checked_on_both_replacement_routes_before_mutation() -> TestResult {
    use bondline_model::{Error, MAX_TEXT_BYTES, PageId, TextError};
    let page = PageId::new(1).ok_or("page")?;
    let full = label("λ".repeat(MAX_TEXT_BYTES / 2))?;
    let mut doc = Document::default();
    doc.append_texts(page, vec![caption(1, String::new())?])?;
    doc.append_fragments(
        page,
        vec![
            fragment(1, 1, 32, Arc::clone(&full))?,
            fragment(2, 33, 32, Arc::clone(&full))?,
        ],
    )?;
    let before = doc.clone();
    assert!(matches!(
        doc.replace_text(page, caption(1, "x".into())?),
        Err(Error::Text(TextError::DocumentTextLimit))
    ));
    assert!(matches!(
        doc.replace_fragment(page, fragment(2, 33, 33, Arc::clone(&full))?),
        Err(Error::Text(TextError::DocumentTextLimit))
    ));
    assert_eq!(doc, before);
    doc.replace_fragments(
        page,
        vec![
            fragment(2, 32, 33, Arc::clone(&full))?,
            fragment(1, 1, 31, full)?,
        ],
    )?;
    assert_eq!(doc.pages()[0].fragments()[0].atoms().len(), 31);
    assert_eq!(doc.pages()[0].fragments()[1].atoms().len(), 33);
    assert!(doc.has_authored_atom_labels());
    Ok(())
}

fn allocated_label(
    font_capacity: usize,
    text_capacity: usize,
    run_capacity: usize,
) -> TestResult<(Arc<AtomLabel>, usize)> {
    use std::mem::size_of;
    let mut family = String::with_capacity(font_capacity);
    family.push_str("Roboto");
    let mut text = String::with_capacity(text_capacity);
    text.push_str("OH");
    let mut runs = Vec::with_capacity(run_capacity);
    let payload = size_of::<AtomLabel>()
        + family.capacity()
        + text.capacity()
        + runs.capacity() * size_of::<TextRun>();
    runs.push(TextRun::new(
        text,
        TextStyle::new(family, 12.0, 0, [0.0; 3])?,
    )?);
    Ok((
        Arc::new(AtomLabel::new(None, None, None, None, runs)?),
        payload,
    ))
}

#[test]
fn retained_bytes_count_reserved_label_storage_once_across_fragments_and_snapshots() -> TestResult {
    use bondline_model::PageId;
    let page = PageId::new(1).ok_or("page")?;
    let (small, small_size) = allocated_label(6, 2, 1)?;
    let (large, large_size) = allocated_label(2048, 8192, 128)?;
    let mut small_doc = Document::default();
    small_doc.append_fragments(page, vec![fragment(1, 1, 2, small)?])?;
    let mut shared = Document::default();
    shared.append_fragments(page, vec![fragment(1, 1, 2, Arc::clone(&large))?])?;
    assert_eq!(small_doc, shared);
    assert_eq!(
        Document::retained_bytes([&shared]) - Document::retained_bytes([&small_doc]),
        large_size - small_size
    );
    let once = Document::retained_bytes([&shared]);
    assert_eq!(Document::retained_bytes([&shared, &shared]), once);
    let clone = shared.clone();
    assert_eq!(
        Document::retained_bytes([&shared, &clone]) - once,
        Document::retained_bytes([&Document::default()]) + std::mem::size_of::<Arc<Fragment>>()
    );
    let mut shared_two = Document::default();
    shared_two.append_fragments(
        page,
        vec![
            fragment(1, 1, 1, Arc::clone(&large))?,
            fragment(2, 2, 1, Arc::clone(&large))?,
        ],
    )?;
    let mut distinct = Document::default();
    distinct.append_fragments(
        page,
        vec![
            fragment(1, 1, 1, Arc::clone(&large))?,
            fragment(2, 2, 1, allocated_label(2048, 8192, 128)?.0)?,
        ],
    )?;
    assert_eq!(shared_two, distinct);
    assert_eq!(
        Document::retained_bytes([&distinct]) - Document::retained_bytes([&shared_two]),
        large_size
    );
    let original = shared_two.clone();
    shared_two.replace_fragment(page, fragment(1, 1, 1, large)?)?;
    assert!(!Arc::ptr_eq(
        &original.pages()[0].fragments()[0],
        &shared_two.pages()[0].fragments()[0]
    ));
    assert!(Arc::ptr_eq(
        original.pages()[0].fragments()[0].atoms()[0]
            .authored_label
            .as_ref()
            .ok_or("label")?,
        shared_two.pages()[0].fragments()[0].atoms()[0]
            .authored_label
            .as_ref()
            .ok_or("label")?
    ));
    assert!(
        Document::retained_bytes([&shared_two, &original])
            < Document::retained_bytes([&shared_two]) + Document::retained_bytes([&original])
    );
    let mut shared_snapshot = shared.clone();
    shared_snapshot.replace_fragment(
        page,
        fragment(
            1,
            1,
            2,
            shared.pages()[0].fragments()[0].atoms()[0]
                .authored_label
                .clone()
                .ok_or("label")?,
        )?,
    )?;
    let mut separate_snapshot = shared.clone();
    separate_snapshot.replace_fragment(
        page,
        fragment(1, 1, 2, allocated_label(2048, 8192, 128)?.0)?,
    )?;
    assert_eq!(shared_snapshot, separate_snapshot);
    assert_eq!(
        Document::retained_bytes([&shared, &separate_snapshot])
            - Document::retained_bytes([&shared, &shared_snapshot]),
        large_size
    );
    Ok(())
}
