use bondline_commands::Engine;
use serde_json::{Value, json};

#[test]
fn native_settings_request_is_addressable_and_never_authors_or_acknowledges_a_save()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let before = serde_json::to_value(engine.state())?;
    let opened = serde_json::to_value(engine.dispatch(
        "ui.drawing_settings",
        json!({"action":"open","expected_revision":0}),
    )?)?;
    for key in [
        "document",
        "revision",
        "undo_depth",
        "redo_depth",
        "file",
        "source",
    ] {
        assert_eq!(opened[key], before[key], "opening changed {key}");
    }
    let request = &opened["ui_drawing_settings"];
    assert_eq!(request["id"], 1);
    assert_eq!(request["input_revision"], 0);
    assert_eq!(request["bond_length_pt"], Value::Null);
    assert_eq!(request["chain_angle_degrees"], Value::Null);
    assert_eq!(request["current"], true);
    assert_eq!(opened["file_operation"], Value::Null);
    assert!(
        engine
            .controls()
            .iter()
            .any(|c| c.id == "file.document_settings" && !c.enabled)
    );
    let cancelled = serde_json::to_value(engine.dispatch(
        "ui.drawing_settings",
        json!({"action":"cancel","expected_revision":0,"request_id":1}),
    )?)?;
    assert!(cancelled.get("ui_drawing_settings").is_none());
    for key in [
        "document",
        "revision",
        "undo_depth",
        "redo_depth",
        "file",
        "source",
    ] {
        assert_eq!(cancelled[key], before[key], "cancel changed {key}");
    }
    assert!(
        engine
            .controls()
            .iter()
            .any(|c| c.id == "file.document_settings" && c.enabled)
    );
    let next = serde_json::to_value(engine.dispatch(
        "ui.drawing_settings",
        json!({"action":"open","expected_revision":0}),
    )?)?;
    assert_eq!(next["ui_drawing_settings"]["id"], 2);
    Ok(())
}
