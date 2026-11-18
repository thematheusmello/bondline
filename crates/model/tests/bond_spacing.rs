//! Existing-API runtime RED: optional spacing must be authored model data.
use bondline_model::DrawingStyle;
use serde_json::json;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn explicit_percent_is_not_an_unknown_style_field_or_an_invented_default() -> TestResult {
    let style: DrawingStyle = serde_json::from_value(json!({"bond_spacing_percent":18.0}))?;
    assert_eq!(
        serde_json::to_value(style)?,
        json!({"bond_spacing_percent":18.0})
    );
    assert!(!style.is_empty());
    assert_eq!(style.line_width_pt(), None);
    assert_eq!(serde_json::to_value(DrawingStyle::new(None)?)?, json!({}));
    Ok(())
}

#[test]
fn independent_preserving_setters_keep_absence_zero_and_both_fields() -> TestResult {
    for width in [None, Some(0.5), Some(2.0), Some(100000.0)] {
        let original = DrawingStyle::new(width)?;
        assert_eq!(original.bond_spacing_percent(), None);
        for spacing in [
            None,
            Some(0.0),
            Some(f64::from_bits(1)),
            Some(18.0),
            Some(3276.7),
        ] {
            let style = original.with_bond_spacing_percent(spacing)?;
            assert_eq!(style.line_width_pt(), width);
            assert_eq!(style.bond_spacing_percent(), spacing);
            assert_eq!(style.is_empty(), width.is_none() && spacing.is_none());
            assert_eq!(style.with_bond_spacing_percent(None)?, original);
            let changed_width = style.with_line_width(Some(3.0))?;
            assert_eq!(changed_width.line_width_pt(), Some(3.0));
            assert_eq!(changed_width.bond_spacing_percent(), spacing);
            assert_eq!(changed_width.with_line_width(width)?, style);
            assert_eq!(
                serde_json::from_value::<DrawingStyle>(serde_json::to_value(style)?)?,
                style
            );
        }
    }
    let empty: DrawingStyle =
        serde_json::from_str(r#"{"line_width_pt":null,"bond_spacing_percent":null}"#)?;
    assert_eq!(empty, DrawingStyle::default());
    assert_eq!(serde_json::to_value(empty)?, json!({}));
    let explicit_zero = DrawingStyle::default().with_bond_spacing_percent(Some(0.0))?;
    assert_eq!(
        serde_json::to_value(explicit_zero)?,
        json!({"bond_spacing_percent":0.0})
    );
    let mut document = bondline_model::Document::default();
    let pages = document.pages().to_vec();
    document.set_drawing_style(explicit_zero);
    assert_eq!(document.pages(), pages);
    assert_eq!(
        serde_json::to_value(document)?["drawing_style"],
        json!({"bond_spacing_percent":0.0})
    );
    Ok(())
}

#[test]
fn percentages_have_checked_signed_domain_and_closed_map_only_json() -> TestResult {
    let style = DrawingStyle::new(Some(0.75))?.with_bond_spacing_percent(Some(18.0))?;
    for value in [
        -f64::from_bits(1),
        -1.0,
        3276.7000001,
        6553.5,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert!(
            style.with_bond_spacing_percent(Some(value)).is_err(),
            "{value}"
        );
    }
    for value in [0.0, -1.0, 100001.0, f64::NAN, f64::INFINITY] {
        assert!(style.with_line_width(Some(value)).is_err(), "{value}");
    }
    for value in [
        json!([]),
        json!([0.5, 18.0]),
        json!(null),
        json!(18),
        json!({"other":1}),
        json!({"bond_spacing_percent":"18"}),
        json!({"bond_spacing_percent":true}),
        json!({"bond_spacing_percent":-1}),
        json!({"bond_spacing_percent":3276.71}),
        json!({"line_width_pt":0.5,"bond_spacing_percent":18,"BondSpacingAbs":3}),
    ] {
        assert!(
            serde_json::from_value::<DrawingStyle>(value.clone()).is_err(),
            "{value}"
        );
    }
    assert!(
        serde_json::from_str::<DrawingStyle>(
            r#"{"bond_spacing_percent":1,"bond_spacing_percent":2}"#
        )
        .is_err()
    );
    assert_eq!(style.line_width_pt(), Some(0.75));
    assert_eq!(style.bond_spacing_percent(), Some(18.0));
    Ok(())
}
