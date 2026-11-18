use bondline_model::{Document, DrawingStyle, MAX_POSITION_PT};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn optional_point_width_is_validated_copy_data_without_an_imported_default() -> TestResult {
    fn copy<T: Copy + Default + PartialEq>(value: T) -> T {
        value
    }
    let mut document = Document::default();
    let original = document.clone();
    assert!(document.drawing_style().is_empty());
    assert!(
        serde_json::to_value(&document)?
            .get("drawing_style")
            .is_none()
    );
    for width in [f64::from_bits(1), 0.5, 2.0, MAX_POSITION_PT] {
        let style = copy(DrawingStyle::new(Some(width))?);
        assert_eq!(style.line_width_pt(), Some(width));
        assert!(!style.is_empty());
        assert_eq!(serde_json::to_value(style)?, json!({"line_width_pt":width}));
        assert_eq!(
            serde_json::from_value::<DrawingStyle>(json!({"line_width_pt":width}))?,
            style
        );
        document.set_drawing_style(style);
        assert_eq!(document.drawing_style(), style);
        assert_eq!(document.pages(), original.pages());
        assert_eq!(document.drawing_settings(), original.drawing_settings());
    }
    for empty in [json!({}), json!({"line_width_pt":null})] {
        let style = serde_json::from_value::<DrawingStyle>(empty)?;
        assert!(style.is_empty());
        assert_eq!(style.line_width_pt(), None);
        assert_eq!(serde_json::to_value(style)?, json!({}));
        document.set_drawing_style(style);
        assert_eq!(document, original);
    }
    Ok(())
}

#[test]
fn invalid_widths_and_nonobject_or_unknown_json_are_rejected() -> TestResult {
    for width in [
        0.0,
        -0.0,
        -1.0,
        MAX_POSITION_PT + 1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert!(DrawingStyle::new(Some(width)).is_err(), "{width}");
    }
    for value in [
        json!([]),
        json!([0.5]),
        json!(null),
        json!(0.5),
        json!({"other":1}),
        json!({"line_width_pt":"0.5"}),
        json!({"line_width_pt":true}),
        json!({"line_width_pt":-1}),
        json!({"line_width_pt":100001}),
    ] {
        assert!(
            serde_json::from_value::<DrawingStyle>(value.clone()).is_err(),
            "{value}"
        );
    }
    Ok(())
}
