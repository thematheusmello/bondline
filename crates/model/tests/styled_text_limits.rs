use bondline_model::{Document, PageId, Point2, StyledText, TextId, TextRun, TextStyle};
use serde_json::{Value, json};
use std::{mem::size_of, sync::Arc};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn style() -> TestResult<TextStyle> {
    Ok(TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?)
}

fn caption(id: u32, value: &str) -> TestResult<StyledText> {
    Ok(StyledText::new(
        TextId::new(id).ok_or("text id")?,
        Point2::new(72.0, 96.0)?,
        vec![TextRun::new(value.into(), style()?)?],
    )?)
}

fn text_json() -> Value {
    json!({"id":19,"position":{"x_pt":-72.0,"y_pt":100000.0},"runs":[
        {"text":"x\nλ\t😀","style":{"font_family":"Other family","size_pt":0.5,"face":65535,"rgb":[0.0,1.0,0.25]}}
    ]})
}

#[test]
fn style_is_literal_validated_data_without_renderer_font_or_face_defaults() -> TestResult {
    for size in [f64::from_bits(1), 0.25, 12.0, 100_000.0] {
        let value = TextStyle::new("Uninstalled family".into(), size, u16::MAX, [0.0, 0.5, 1.0])?;
        assert_eq!(value.font_family(), "Uninstalled family");
        assert_eq!(value.size_pt(), size);
        assert_eq!(value.face(), 65535);
        assert_eq!(value.rgb(), [0.0, 0.5, 1.0]);
    }
    assert!(TextStyle::new("λ".repeat(128), 12.0, 0, [0.0; 3]).is_ok());
    for family in [String::new(), "λ".repeat(129), "A\0B".into()] {
        assert!(TextStyle::new(family, 12.0, 0, [0.0; 3]).is_err());
    }
    for size in [
        0.0,
        -0.0,
        -1.0,
        100_001.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert!(TextStyle::new("Roboto".into(), size, 0, [0.0; 3]).is_err());
    }
    for channel in [
        -f64::from_bits(1),
        -1.0,
        1.0000000000000002,
        f64::NAN,
        f64::INFINITY,
    ] {
        for index in 0..3 {
            let mut rgb = [0.0; 3];
            rgb[index] = channel;
            assert!(TextStyle::new("Roboto".into(), 12.0, 0, rgb).is_err());
        }
    }
    Ok(())
}

#[test]
fn xml_characters_and_utf8_byte_limits_are_checked_without_normalization() -> TestResult {
    let literal = "\t\n\r !<&>\u{7f}\u{85}λ\u{d7ff}\u{e000}\u{fffd}\u{10000}\u{10ffff}";
    assert_eq!(TextRun::new(literal.into(), style()?)?.text(), literal);
    assert_eq!(TextRun::new(String::new(), style()?)?.text(), "");
    for invalid in [
        '\0', '\u{1}', '\u{8}', '\u{b}', '\u{c}', '\u{e}', '\u{1f}', '\u{fffe}', '\u{ffff}',
    ] {
        assert!(TextRun::new(format!("before{invalid}after"), style()?).is_err());
        let mut value = text_json();
        value["runs"][0]["text"] = json!(format!("{invalid}"));
        assert!(serde_json::from_value::<StyledText>(value).is_err());
    }
    let exact = "λ".repeat(524_288);
    assert_eq!(
        TextRun::new(exact.clone(), style()?)?.text().len(),
        1_048_576
    );
    assert!(TextRun::new(format!("{exact}a"), style()?).is_err());
    let first = TextRun::new("a".repeat(524_288), style()?)?;
    let second = TextRun::new("b".repeat(524_288), style()?)?;
    assert!(
        StyledText::new(
            TextId::new(1).ok_or("id")?,
            Point2::new(0.0, 0.0)?,
            vec![first.clone(), second.clone()]
        )
        .is_ok()
    );
    assert!(
        StyledText::new(
            TextId::new(1).ok_or("id")?,
            Point2::new(0.0, 0.0)?,
            vec![first, second.with_text(format!("{}c", second.text()))?]
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn caption_and_run_json_are_closed_maps_and_cannot_bypass_validation() -> TestResult {
    let value = text_json();
    let parsed: StyledText = serde_json::from_value(value.clone())?;
    assert_eq!(serde_json::to_value(&parsed)?, value);
    assert_eq!(
        serde_json::from_value::<TextId>(json!(u32::MAX))?.get(),
        u32::MAX
    );
    for id in [
        json!(0),
        json!(-1),
        json!(4294967296_u64),
        json!("19"),
        json!({"19":null}),
    ] {
        assert!(serde_json::from_value::<TextId>(id).is_err());
    }
    for invalid in [
        json!([]),
        json!([19, {"x_pt":0,"y_pt":0}, []]),
        json!(null),
        json!(1),
        json!("text"),
    ] {
        assert!(serde_json::from_value::<StyledText>(invalid).is_err());
    }
    for key in ["id", "position", "runs"] {
        let mut invalid = value.clone();
        invalid.as_object_mut().ok_or("object")?.remove(key);
        assert!(serde_json::from_value::<StyledText>(invalid).is_err());
    }
    for (pointer, replacement) in [
        ("/extra", json!(true)),
        ("/id", json!(0)),
        ("/runs", json!([])),
        ("/runs/0/extra", json!(1)),
        ("/runs/0/text", json!(null)),
        ("/runs/0/style", json!(["Roboto", 12, 0, [0, 0, 0]])),
        ("/runs/0/style/size_pt", json!(0)),
        ("/runs/0/style/face", json!(65536)),
        ("/runs/0/style/rgb", json!([0, 0, 2])),
        ("/runs/0/style/extra", json!(1)),
    ] {
        let mut invalid = value.clone();
        if let Some(slot) = invalid.pointer_mut(pointer) {
            *slot = replacement;
        } else if pointer == "/extra" {
            invalid["extra"] = replacement;
        } else if pointer == "/runs/0/extra" {
            invalid["runs"][0]["extra"] = replacement;
        } else {
            invalid["runs"][0]["style"]["extra"] = replacement;
        }
        assert!(
            serde_json::from_value::<StyledText>(invalid.clone()).is_err(),
            "{invalid}"
        );
    }
    assert!(serde_json::from_value::<TextRun>(json!(["x", value["runs"][0]["style"]])).is_err());
    assert!(serde_json::from_str::<TextRun>(r#"{"text":"x","text":"y","style":{"font_family":"Roboto","size_pt":12,"face":0,"rgb":[0,0,0]}}"#).is_err());
    Ok(())
}

#[test]
fn checked_copies_preserve_ids_styles_positions_and_original_runs() -> TestResult {
    let original = StyledText::new(
        TextId::new(9).ok_or("id")?,
        Point2::new(10.0, 20.0)?,
        vec![
            TextRun::new("first".into(), style()?)?,
            TextRun::new(
                "second".into(),
                TextStyle::new("Roboto".into(), 18.0, 96, [1.0, 0.0, 0.0])?,
            )?,
        ],
    )?;
    let changed = original.with_run_text(1, "changed λ".into())?;
    assert_eq!(changed.id(), original.id());
    assert_eq!(changed.position(), original.position());
    assert_eq!(changed.runs()[0], original.runs()[0]);
    assert_eq!(changed.runs()[1].style(), original.runs()[1].style());
    assert_eq!(original.runs()[1].text(), "second");
    assert!(original.with_run_text(2, "bad index".into()).is_err());
    assert!(original.with_run_text(0, "bad\0".into()).is_err());
    let moved = original.with_position(Point2::new(-100_000.0, 100_000.0)?)?;
    assert_eq!(moved.runs(), original.runs());
    assert_eq!(moved.id(), original.id());
    assert_eq!(original.position(), Point2::new(10.0, 20.0)?);
    assert!(StyledText::new(original.id(), original.position(), vec![]).is_err());
    let empty = TextRun::new(String::new(), style()?)?;
    assert!(
        StyledText::new(
            original.id(),
            original.position(),
            vec![empty.clone(); 4096]
        )
        .is_ok()
    );
    assert!(StyledText::new(original.id(), original.position(), vec![empty; 4097]).is_err());
    Ok(())
}

#[test]
fn append_and_replacement_are_atomic_ordered_and_share_unrelated_payloads() -> TestResult {
    let page = PageId::new(1).ok_or("page")?;
    let missing_page = PageId::new(2).ok_or("page")?;
    let mut document = Document::default();
    document.append_texts(
        page,
        vec![caption(u32::MAX, "first")?, caption(7, "second")?],
    )?;
    let before = document.clone();
    assert!(
        document
            .append_texts(page, vec![caption(8, "new")?, caption(7, "duplicate")?])
            .is_err()
    );
    assert_eq!(document, before);
    assert!(
        document
            .append_texts(missing_page, vec![caption(8, "new")?])
            .is_err()
    );
    assert!(
        document
            .replace_text(missing_page, caption(7, "changed")?)
            .is_err()
    );
    assert!(document.replace_text(page, caption(8, "absent")?).is_err());
    assert_eq!(document, before);
    document.replace_text(page, caption(7, "second")?)?;
    assert!(
        Arc::ptr_eq(
            &document.pages()[0].texts()[1],
            &before.pages()[0].texts()[1]
        ),
        "logical no-op retains the shared payload"
    );
    document.replace_text(page, caption(7, "changed")?)?;
    assert_eq!(
        document.pages()[0]
            .texts()
            .iter()
            .map(|t| t.id().get())
            .collect::<Vec<_>>(),
        [u32::MAX, 7]
    );
    assert!(Arc::ptr_eq(
        &document.pages()[0].texts()[0],
        &before.pages()[0].texts()[0]
    ));
    assert!(!Arc::ptr_eq(
        &document.pages()[0].texts()[1],
        &before.pages()[0].texts()[1]
    ));
    assert_eq!(before.pages()[0].texts()[1].runs()[0].text(), "second");
    assert_eq!(
        document.pages()[0].fragments(),
        before.pages()[0].fragments()
    );
    Ok(())
}

#[test]
fn document_aggregate_run_and_text_limits_refuse_without_partial_append_or_replace() -> TestResult {
    let page = PageId::new(1).ok_or("page")?;
    let point = Point2::new(0.0, 0.0)?;
    let empty = TextRun::new(String::new(), style()?)?;
    let mut document = Document::default();
    let texts = (1..=25)
        .map(|id| {
            StyledText::new(
                TextId::new(id).ok_or("id")?,
                point,
                vec![empty.clone(); 4000],
            )
            .map_err(Into::into)
        })
        .collect::<TestResult<Vec<_>>>()?;
    document.append_texts(page, texts)?;
    let before = document.clone();
    assert!(document.append_texts(page, vec![caption(26, "")?]).is_err());
    assert_eq!(document, before);
    let bigger = StyledText::new(TextId::new(1).ok_or("id")?, point, vec![empty; 4001])?;
    assert!(document.replace_text(page, bigger).is_err());
    assert_eq!(document, before);
    let mut document = Document::default();
    let full = "a".repeat(1_048_576);
    for id in 1..=64 {
        document.append_texts(page, vec![caption(id, &full)?])?;
    }
    let before = document.clone();
    assert!(
        document
            .append_texts(page, vec![caption(65, "b")?])
            .is_err()
    );
    assert_eq!(document, before);
    // Exactly 64MiB remains valid after a same-sized replacement.
    document.replace_text(page, caption(64, &full)?)?;
    assert_eq!(document, before);
    Ok(())
}

fn allocated_caption(
    font_capacity: usize,
    text_capacity: usize,
    run_capacity: usize,
) -> TestResult<(StyledText, usize)> {
    let mut family = String::with_capacity(font_capacity);
    family.push_str("Roboto");
    let mut value = String::with_capacity(text_capacity);
    value.push_str("caption");
    let mut runs = Vec::with_capacity(run_capacity);
    let allocated = family.capacity() + value.capacity() + runs.capacity() * size_of::<TextRun>();
    runs.push(TextRun::new(
        value,
        TextStyle::new(family, 12.0, 0, [0.0; 3])?,
    )?);
    Ok((
        StyledText::new(TextId::new(1).ok_or("id")?, Point2::new(72.0, 96.0)?, runs)?,
        allocated,
    ))
}

#[test]
fn history_accounting_counts_reserved_strings_runs_and_unique_text_arcs() -> TestResult {
    let page = PageId::new(1).ok_or("page")?;
    let (small, small_allocated) = allocated_caption(6, 7, 1)?;
    let (large, large_allocated) = allocated_caption(2048, 8192, 128)?;
    let mut first = Document::default();
    let mut second = Document::default();
    first.append_texts(page, vec![small])?;
    second.append_texts(page, vec![large])?;
    assert_eq!(first, second, "capacity is not authored content");
    assert_eq!(
        Document::retained_bytes([&second]) - Document::retained_bytes([&first]),
        large_allocated - small_allocated
    );
    let once = Document::retained_bytes([&second]);
    assert_eq!(Document::retained_bytes([&second, &second]), once);
    let clone = second.clone();
    assert!(Arc::ptr_eq(
        &second.pages()[0].texts()[0],
        &clone.pages()[0].texts()[0]
    ));
    assert_eq!(
        Document::retained_bytes([&second, &clone]) - once,
        Document::retained_bytes([&Document::default()]) + size_of::<Arc<StyledText>>()
    );
    second.append_texts(page, vec![caption(2, "unchanged shared caption")?])?;
    let original = second.clone();
    second.replace_text(page, caption(1, "changed")?)?;
    assert!(Arc::ptr_eq(
        &second.pages()[0].texts()[1],
        &original.pages()[0].texts()[1]
    ));
    assert!(
        Document::retained_bytes([&second, &original])
            < Document::retained_bytes([&second]) + Document::retained_bytes([&original])
    );
    Ok(())
}
