use bondline_model::Point2;

#[test]
fn model_positions_use_the_published_point_units_and_axis_order()
-> Result<(), Box<dyn std::error::Error>> {
    // CDX-COORDS: CDXML "72 144" is one inch right and two inches down.
    let point = Point2::new(72.0, 144.0)?;
    assert_eq!(point.x_pt(), 72.0);
    assert_eq!(point.y_pt(), 144.0);
    assert_eq!(
        serde_json::to_value(point)?,
        serde_json::json!({"x_pt":72.0,"y_pt":144.0})
    );
    assert_eq!(Point2::new(-72.0, 0.0)?.x_pt(), -72.0);
    Ok(())
}

#[test]
fn positions_are_finite_bounded_and_validated_during_deserialization()
-> Result<(), Box<dyn std::error::Error>> {
    for invalid in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        100_001.0,
        -100_001.0,
    ] {
        assert!(Point2::new(invalid, 0.0).is_err());
        assert!(Point2::new(0.0, invalid).is_err());
    }
    assert!(Point2::new(-100_000.0, 100_000.0).is_ok());
    for json in [
        r#"{"x_pt":100001,"y_pt":0}"#,
        r#"{"x_pt":0,"y_pt":-100001}"#,
        r#"{"x_pt":1,"y_pt":2,"z_pt":3}"#,
    ] {
        assert!(serde_json::from_str::<Point2>(json).is_err());
    }
    let origin: Point2 = serde_json::from_str(r#"{"x_pt":0,"y_pt":0}"#)?;
    assert_eq!(origin, Point2::new(0.0, 0.0)?);
    Ok(())
}
