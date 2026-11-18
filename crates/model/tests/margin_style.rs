use bondline_model::{Document, DrawingStyle, MAX_POSITION_PT};
use serde_json::json;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn margin_is_independent_checked_optional_point_data_and_preserves_old_absence() -> TestResult {
    let original = Document::default();
    assert_eq!(original.drawing_style().margin_width_pt(), None);
    assert_eq!(serde_json::to_value(original.drawing_style())?, json!({}));
    for margin in [0.0, f64::from_bits(1), 0.125, 2.0, MAX_POSITION_PT] {
        let style = DrawingStyle::new(Some(1.5))?
            .with_bond_spacing_percent(Some(15.0))?
            .with_margin_width(Some(margin))?;
        assert_eq!(style.margin_width_pt(), Some(margin));
        assert!(!style.is_empty());
        let expected =
            json!({"line_width_pt":1.5,"bond_spacing_percent":15.0,"margin_width_pt":margin});
        assert_eq!(serde_json::to_value(style)?, expected);
        assert_eq!(serde_json::from_value::<DrawingStyle>(expected)?, style);
        assert_eq!(style.with_line_width(None)?.margin_width_pt(), Some(margin));
        assert_eq!(
            style.with_bond_spacing_percent(None)?.margin_width_pt(),
            Some(margin)
        );
        let cleared = style.with_margin_width(None)?;
        assert_eq!(cleared.line_width_pt(), Some(1.5));
        assert_eq!(cleared.bond_spacing_percent(), Some(15.0));
        assert_eq!(cleared.margin_width_pt(), None);
        let mut document = original.clone();
        document.set_drawing_style(DrawingStyle::default().with_margin_width(Some(margin))?);
        assert_eq!(document.pages(), original.pages());
        assert_eq!(document.drawing_settings(), original.drawing_settings());
        assert!(
            serde_json::to_value(document)?
                .get("drawing_style")
                .is_some()
        );
    }
    for raw in ["{}", r#"{"margin_width_pt":null}"#] {
        let style: DrawingStyle = serde_json::from_str(raw)?;
        assert!(style.is_empty());
        assert_eq!(style.margin_width_pt(), None);
        assert_eq!(serde_json::to_string(&style)?, "{}");
    }
    Ok(())
}

#[test]
fn invalid_margin_and_nonobject_unknown_or_duplicate_input_are_rejected() -> TestResult {
    for value in [
        -1.0,
        -f64::from_bits(1),
        MAX_POSITION_PT + 1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert!(
            DrawingStyle::default()
                .with_margin_width(Some(value))
                .is_err()
        );
    }
    for raw in [
        "[]",
        "null",
        "2",
        r#"{"margin_width_pt":"2"}"#,
        r#"{"margin_width_pt":true}"#,
        r#"{"margin_width_pt":-1}"#,
        r#"{"margin_width_pt":100001}"#,
        r#"{"margin_width_pt":1,"margin_width_pt":2}"#,
        r#"{"margin_width_pt":2,"future":1}"#,
    ] {
        assert!(serde_json::from_str::<DrawingStyle>(raw).is_err(), "{raw}");
    }
    let zero: DrawingStyle = serde_json::from_str(r#"{"margin_width_pt":-0.0}"#)?;
    assert_eq!(zero.margin_width_pt(), Some(0.0));
    assert!(!zero.is_empty());
    Ok(())
}
