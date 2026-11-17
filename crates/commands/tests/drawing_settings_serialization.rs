use bondline_commands::{Command, DrawingSettingsSet};
use serde_json::json;

#[test]
fn nonfinite_typed_values_cannot_serialize_as_explicit_clear_requests()
-> Result<(), Box<dyn std::error::Error>> {
    for (length, angle) in [(None, None), (Some(24.5), None), (None, Some(120.5))] {
        let command = Command::DrawingSettingsSet(DrawingSettingsSet {
            expected_revision: 7,
            bond_length_pt: length,
            chain_angle_degrees: angle,
        });
        assert_eq!(
            serde_json::to_value(command)?,
            json!({
            "command":"document.set_drawing_settings","params":{
                "expected_revision":7,"bond_length_pt":length,"chain_angle_degrees":angle}})
        );
    }
    for (length, angle) in [
        (Some(f64::NAN), Some(120.5)),
        (Some(f64::INFINITY), None),
        (Some(f64::NEG_INFINITY), None),
        (Some(24.5), Some(f64::NAN)),
        (None, Some(f64::INFINITY)),
        (None, Some(f64::NEG_INFINITY)),
    ] {
        let command = Command::DrawingSettingsSet(DrawingSettingsSet {
            expected_revision: 7,
            bond_length_pt: length,
            chain_angle_degrees: angle,
        });
        let actual = serde_json::to_value(command);
        assert!(
            actual.is_err(),
            "non-finite values became clear requests: {actual:?}"
        );
    }
    Ok(())
}
