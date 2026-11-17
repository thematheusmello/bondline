//! New public API qualification, separate from the existing-JSON runtime RED.
use bondline_commands::{Command, DocumentSettingsSet, Engine};
use serde_json::{Value, json};
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn open(e: &mut Engine) -> Result<u64, Box<dyn std::error::Error>> {
    let v = serde_json::to_value(e.dispatch(
        "ui.drawing_settings",
        json!({"action":"open","expected_revision":e.state().revision}),
    )?)?;
    Ok(v["ui_drawing_settings"]["id"]
        .as_u64()
        .ok_or("request id")?)
}
#[test]
fn typed_triple_retains_explicit_nulls_and_never_serializes_nonfinite_as_clear() -> TestResult {
    let value = DocumentSettingsSet {
        expected_revision: 7,
        bond_length_pt: None,
        chain_angle_degrees: Some(120.5),
        line_width_pt: Some(2.0),
    };
    assert_eq!(
        serde_json::to_value(Command::DocumentSettingsSet(value))?,
        json!({"command":"document.set_document_settings","params":{
        "expected_revision":7,"bond_length_pt":null,"chain_angle_degrees":120.5,"line_width_pt":2.0}})
    );
    for field in 0..3 {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut values = [Some(24.5), Some(120.5), Some(2.0)];
            values[field] = Some(bad);
            let value = DocumentSettingsSet {
                expected_revision: 0,
                bond_length_pt: values[0],
                chain_angle_degrees: values[1],
                line_width_pt: values[2],
            };
            assert!(
                serde_json::to_value(Command::DocumentSettingsSet(value.clone())).is_err(),
                "field{field} must not silently become null"
            );
            let mut e = Engine::default();
            let before = serde_json::to_value(e.state())?;
            let stamp = e.capture_stamp();
            let error = e
                .execute(Command::DocumentSettingsSet(value))
                .err()
                .ok_or("typed nonfinite accepted")?;
            assert_eq!(error.info().code, "invalid_params");
            assert_eq!(serde_json::to_value(e.state())?, before);
            assert!(e.capture_stamp_matches(&stamp));
        }
    }
    let all_null = DocumentSettingsSet {
        expected_revision: 0,
        bond_length_pt: None,
        chain_angle_degrees: None,
        line_width_pt: None,
    };
    assert_eq!(
        serde_json::to_value(all_null)?,
        json!({"expected_revision":0,
        "bond_length_pt":null,"chain_angle_degrees":null,"line_width_pt":null})
    );
    Ok(())
}
#[test]
fn new_form_apply_is_atomic_and_legacy_pair_apply_preserves_style_and_request_guards() -> TestResult
{
    let mut e = Engine::default();
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":144,"begin_y_pt":300,
        "end_x_pt":224,"end_y_pt":300}),
    )?;
    e.dispatch(
        "document.set_document_settings",
        json!({"expected_revision":1,
        "bond_length_pt":24.5,"chain_angle_degrees":120.5,"line_width_pt":2}),
    )?;
    e.set_ui_file_host(true);
    let id = open(&mut e)?;
    let request = serde_json::to_value(e.ui_drawing_settings_request().ok_or("request")?)?;
    assert_eq!(request["line_width_pt"], 2.0);
    assert_eq!(request["current"], true);
    let pair = e.apply_ui_drawing_settings(id, Some(30.0), Some(90.0))?;
    assert!(pair.changed && pair.file_operation.is_none() && pair.ui_file_operation.is_none());
    assert_eq!(
        serde_json::to_value(e.document()?)?["drawing_style"],
        json!({"line_width_pt":2.0})
    );
    assert!(e.ui_drawing_settings_request().is_none());
    let id = open(&mut e)?;
    let before = serde_json::to_value(e.state())?;
    let stamp = e.capture_stamp();
    let error = e
        .apply_ui_document_settings(id, Some(48.0), Some(120.0), Some(0.0))
        .err()
        .ok_or("invalid width accepted")?;
    assert_eq!(error.info().code, "invalid_params");
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert!(e.capture_stamp_matches(&stamp) && e.ui_drawing_settings_request_current(id));
    let model = e.stored_document()?.model_snapshot();
    let noop = e.apply_ui_document_settings(id, Some(30.0), Some(90.0), Some(2.0))?;
    assert!(!noop.changed && noop.file_operation.is_none() && noop.ui_file_operation.is_none());
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert!(e.ui_drawing_settings_request().is_none());
    let id = open(&mut e)?;
    e.dispatch("ui.view", json!({"action":"actual"}))?;
    e.dispatch("ui.view", json!({"action":"fit"}))?;
    assert!(!e.ui_drawing_settings_request_current(id));
    let before = serde_json::to_value(e.state())?;
    let error = e
        .apply_ui_document_settings(id, None, None, None)
        .err()
        .ok_or("view ABA revived Apply")?;
    assert_eq!(error.info().code, "ui_stale_request");
    assert_eq!(serde_json::to_value(e.state())?, before);
    e.dispatch(
        "ui.drawing_settings",
        json!({"action":"cancel",
        "expected_revision":e.state().revision,"request_id":id}),
    )?;
    assert!(e.ui_drawing_settings_request().is_none());
    let id = open(&mut e)?;
    let applied = e.apply_ui_document_settings(id, None, None, None)?;
    assert!(applied.changed);
    let state = serde_json::to_value(e.state())?;
    assert_eq!(state["document"].get("drawing_settings"), None);
    assert_eq!(state["document"].get("drawing_style"), None);
    assert_eq!(state.get("ui_drawing_settings"), None);
    assert!(applied.file_operation.is_none() && applied.ui_file_operation.is_none());
    let id = open(&mut e)?;
    let request = serde_json::to_value(e.ui_drawing_settings_request().ok_or("request")?)?;
    assert_eq!(request.get("line_width_pt"), Some(&Value::Null));
    assert!(e.finish_ui_drawing_settings(id));
    Ok(())
}
