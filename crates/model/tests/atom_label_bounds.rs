use bondline_model::{AtomLabel, Point2, TextRun, TextStyle};
use serde_json::json;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn optional_raw_text_bounds_survive_checked_copies_and_keep_legacy_absence_exact() -> TestResult {
    let label = AtomLabel::new(
        None,
        None,
        None,
        None,
        vec![TextRun::new(
            "OH".into(),
            TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
        )?],
    )?;
    let old = serde_json::to_value(&label)?;
    assert!(old.get("bounding_box").is_none());
    assert_eq!(label.bounding_box(), None);
    let bounds = [80.69140625, 91.3515625, 95.796875, 100.1171875];
    let boxed = label.with_bounding_box(Some(bounds))?;
    assert_eq!(boxed.bounding_box(), Some(bounds));
    assert_eq!(
        boxed.with_run_text(0, "NH2".into())?.bounding_box(),
        Some(bounds)
    );
    assert_eq!(
        boxed
            .with_position(Some(Point2::new(1.0, 2.0)?))?
            .bounding_box(),
        Some(bounds)
    );
    let mut expected = old.clone();
    expected["bounding_box"] = json!(bounds);
    assert_eq!(serde_json::to_value(&boxed)?, expected);
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<AtomLabel>(expected)?)?,
        serde_json::to_value(&boxed)?
    );
    assert_eq!(serde_json::to_value(boxed.with_bounding_box(None)?)?, old);
    Ok(())
}

#[test]
fn text_bounds_reject_unordered_nonfinite_or_unbounded_data_without_losing_the_source() -> TestResult
{
    let label = AtomLabel::new(
        None,
        None,
        None,
        None,
        vec![TextRun::new(
            "O".into(),
            TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
        )?],
    )?;
    for bounds in [
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0, 0.0],
        [-100001.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 100001.0, 1.0],
        [0.0, 0.0, f64::INFINITY, 1.0],
        [0.0, f64::NAN, 1.0, 1.0],
    ] {
        assert!(matches!(
            label.with_bounding_box(Some(bounds)),
            Err(bondline_model::TextError::InvalidBounds)
        ));
        assert_eq!(label.bounding_box(), None);
    }
    assert_eq!(
        label
            .with_bounding_box(Some([-100000.0, -100000.0, 100000.0, 100000.0]))?
            .bounding_box(),
        Some([-100000.0, -100000.0, 100000.0, 100000.0])
    );
    assert_eq!(
        label.with_bounding_box(Some([0.0; 4]))?.bounding_box(),
        Some([0.0; 4])
    );
    let base = serde_json::to_value(&label)?;
    for value in [
        json!([0, 0, 1]),
        json!([0, 0, 1, 1, 2]),
        json!([0, 0, -1, 1]),
        json!({"left":0,"top":0,"right":1,"bottom":1}),
        json!("0 0 1 1"),
    ] {
        let mut candidate = base.clone();
        candidate["bounding_box"] = value;
        assert!(serde_json::from_value::<AtomLabel>(candidate).is_err());
    }
    let mut null = base.clone();
    null["bounding_box"] = serde_json::Value::Null;
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<AtomLabel>(null)?)?,
        base
    );
    Ok(())
}
