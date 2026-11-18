//! New Rust API qualification is separate from the initial JSON runtime RED.
use bondline_commands::{Command, DrawingSettingsSet, Engine};
use serde_json::json;
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn typed_full_pair_retains_nulls_and_rejects_nonfinite_domains_without_mutation() -> TestResult {
    let request = DrawingSettingsSet {
        expected_revision: 0,
        bond_length_pt: None,
        chain_angle_degrees: Some(120.5),
    };
    assert_eq!(
        serde_json::to_value(&request)?,
        json!({
        "expected_revision":0,"bond_length_pt":null,"chain_angle_degrees":120.5})
    );
    let decoded: DrawingSettingsSet = serde_json::from_value(serde_json::to_value(&request)?)?;
    assert_eq!(
        (
            decoded.expected_revision,
            decoded.bond_length_pt,
            decoded.chain_angle_degrees
        ),
        (0, None, Some(120.5))
    );
    for missing in [
        json!({"bond_length_pt":null,"chain_angle_degrees":null}),
        json!({"expected_revision":0,"chain_angle_degrees":null}),
        json!({"expected_revision":0,"bond_length_pt":null}),
    ] {
        assert!(serde_json::from_value::<DrawingSettingsSet>(missing).is_err());
    }
    let mut engine = Engine::default();
    for (length, angle) in [
        (Some(f64::NAN), None),
        (Some(f64::INFINITY), None),
        (Some(f64::NEG_INFINITY), None),
        (None, Some(f64::NAN)),
        (None, Some(f64::INFINITY)),
        (None, Some(f64::NEG_INFINITY)),
        (Some(0.0), Some(120.0)),
        (Some(100_001.0), None),
        (None, Some(0.0)),
        (None, Some(180.0)),
    ] {
        let state = serde_json::to_value(engine.state())?;
        let stamp = engine.capture_stamp();
        let model = engine.stored_document()?.model_snapshot();
        let request = DrawingSettingsSet {
            expected_revision: 0,
            bond_length_pt: length,
            chain_angle_degrees: angle,
        };
        let error = engine
            .execute(Command::DrawingSettingsSet(request))
            .err()
            .ok_or("typed invalid settings accepted")?;
        assert_eq!(error.info().code, "invalid_params", "{error}");
        assert_eq!(serde_json::to_value(engine.state())?, state);
        assert_eq!(engine.capture_stamp(), stamp);
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    Ok(())
}
