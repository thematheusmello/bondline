use bondline_commands::{Engine, registry};
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing confined test directory")]
mod support;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn selected(engine: &mut Engine) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let controls = serde_json::to_value(engine.dispatch("ui.controls", json!({}))?)?;
    Ok(controls["controls"]
        .as_array()
        .ok_or("control metadata")?
        .iter()
        .filter(|control| control["menu"] == "Tools")
        .cloned()
        .collect())
}

#[test]
fn registry_and_bindings_expose_bond_and_selection_tools_without_changing_authored_state()
-> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let controls = selected(&mut engine)?;
    assert_eq!(controls.len(), 4, "missing named drawing tools");
    assert_eq!(controls[3]["id"], "tool.selection");
    assert_eq!(controls[3]["selected"], false);
    for (index, (control, order)) in controls
        .iter()
        .zip(["single", "double", "triple"])
        .enumerate()
    {
        assert_eq!(control["id"], format!("tool.bond.{order}"));
        assert_eq!(
            control["command"],
            json!({"command":"ui.bond_tool","params":{"order":order}})
        );
        assert_eq!(control["selected"], index == 0);
        assert_eq!(control["enabled"], true);
        assert_eq!(control["shortcut"], Value::Null);
    }
    let command = registry()
        .into_iter()
        .find(|row| row.name == "ui.bond_tool")
        .ok_or("tool registry")?;
    assert!(!command.read_only);
    let schema = serde_json::to_value(command.input_schema)?;
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"], json!(["order"]));
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn mode_epoch_records_real_changes_and_noop_without_entering_document_history() -> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    for (order, epoch, changed) in [
        ("single", 0, false),
        ("double", 1, true),
        ("triple", 2, true),
        ("single", 3, true),
        ("single", 3, false),
    ] {
        let result =
            serde_json::to_value(engine.dispatch("ui.bond_tool", json!({"order":order}))?)?;
        assert_eq!(
            result["ui_bond_tool_operation"],
            json!({"order":order,"epoch":epoch,"changed":changed})
        );
        assert_eq!(
            result["changed"], false,
            "tool selection is not an authored edit"
        );
        let active = selected(&mut engine)?;
        assert_eq!(
            active
                .iter()
                .filter(|control| control["selected"] == true)
                .count(),
            1
        );
        assert!(
            active
                .iter()
                .any(|control| control["id"] == format!("tool.bond.{order}")
                    && control["selected"] == true)
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    assert!(engine.state().controls.is_none());
    Ok(())
}

#[test]
fn malformed_tool_parameters_are_atomic_and_context_guards_keep_selection() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
    for params in [
        json!({}),
        json!({"order":null}),
        json!({"order":"aromatic"}),
        json!({"order":2}),
        json!({"order":"triple","extra":true}),
    ] {
        let before = serde_json::to_value(engine.state())?;
        let tools = selected(&mut engine)?;
        let error = engine
            .dispatch("ui.bond_tool", params)
            .err()
            .ok_or("invalid tool accepted")?;
        assert_eq!(error.info().code, "invalid_params");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(selected(&mut engine)?, tools);
    }
    engine.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch("ui.bond_tool", json!({"order":"triple"}))
            .err()
            .ok_or("mode changed during close")?
            .info()
            .code,
        "ui_busy"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(
        selected(&mut engine)?
            .iter()
            .all(|control| control["enabled"] == false)
    );
    engine.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"cancel"}),
    )?;
    engine.set_ui_file_host(true);
    engine.dispatch("ui.file", json!({"expected_revision":0,"action":"open"}))?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch("ui.bond_tool", json!({"order":"triple"}))
            .err()
            .ok_or("mode changed over chooser")?
            .info()
            .code,
        "ui_busy"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn tool_persists_across_history_and_open_but_direct_commands_keep_explicit_default_order()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("clean.cdxml");
    fs::write(&input, Engine::default().stored_document()?.to_cdxml()?)?;
    let mut engine = Engine::default();
    engine.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72.0,"begin_y_pt":144.0,"end_x_pt":120.0,"end_y_pt":144.0}),
    )?;
    let bond = engine.document()?.pages()[0].fragments()[0].bonds()[0];
    assert_eq!(
        serde_json::to_value(bond.order)?,
        "single",
        "direct default must not depend on ambient UI tool"
    );
    engine.dispatch("history.undo", json!({}))?;
    engine.dispatch("history.redo", json!({}))?;
    engine.dispatch(
        "document.open_cdxml",
        json!({"path":input,"discard_unsaved":true}),
    )?;
    assert_eq!(
        selected(&mut engine)?
            .iter()
            .find(|control| control["selected"] == true)
            .ok_or("selected")?["id"],
        "tool.bond.triple"
    );
    let before = serde_json::to_value(engine.state())?;
    let result = engine.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
    assert_eq!(
        serde_json::to_value(result)?["ui_bond_tool_operation"],
        json!({"order":"triple","epoch":1,"changed":false})
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}
