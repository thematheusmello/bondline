use serde_json::{Value, json};
#[path = "support/native_conversion_intent.rs"]
mod native_conversion_intent;
use std::{
    collections::BTreeSet,
    process::{Command, Output},
};

fn invoke(name: &str, params: Value) -> Result<Output, Box<dyn std::error::Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run",
            "--cmd",
            &json!({"command":name,"params":params}).to_string(),
        ])
        .output()?)
}

#[test]
fn headless_controls_are_typed_bindings_and_never_claim_a_native_dialog()
-> Result<(), Box<dyn std::error::Error>> {
    let output = invoke("ui.controls", json!({}))?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        result["ui_file"],
        json!({"available":false,"epoch":0,"request":null})
    );
    assert_eq!(result["revision"], 0);
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(result["close_epoch"], 0);
    assert_eq!(result["changed"], false);
    assert!(result.get("file_operation").is_none());
    assert!(result.get("ui_file_operation").is_none());
    let controls = result["controls"].as_array().ok_or("controls array")?;
    let ids: BTreeSet<_> = controls
        .iter()
        .map(|control| control["id"].as_str().ok_or("control id"))
        .collect::<Result<_, _>>()?;
    assert_eq!(
        ids,
        BTreeSet::from([
            "file.new",
            "file.open",
            "file.save",
            "file.save_as",
            "file.document_settings",
            "file.preferences",
            "file.convert_smiles",
            "file.import_smiles_records",
            "file.close",
            "edit.undo",
            "edit.redo",
            "edit.paste_smiles",
            "page.a4",
            "page.letter",
            "page.rotate",
            "view.actual",
            "view.fit",
            "view.magnify",
            "view.reduce",
            "view.analysis",
            "tool.bond.single",
            "tool.bond.double",
            "tool.bond.triple",
            "tool.selection",
            "tool.ring.3",
            "tool.ring.4",
            "tool.ring.5",
            "tool.ring.6",
            "tool.ring.7",
            "tool.ring.8",
            "tool.ring.benzene",
            "tool.ring.cyclopentadiene",
            "tool.ring.chair_1",
            "tool.ring.chair_2",
            "edit.select_all",
            "edit.deselect",
            "chain.acyclic",
            "chain.snaking",
            "chain.fixed_lengths",
            "chain.fixed_angles",
        ])
    );
    assert_eq!(
        ids.len(),
        controls.len(),
        "control identities must be unique"
    );
    for control in controls {
        let command = &control["command"];
        bondline_commands::Command::parse(
            command["command"].as_str().ok_or("bound command name")?,
            command["params"].clone(),
        )?;
    }
    for (id, label, key, shift, action) in [
        ("file.new", "New Document", "n", false, "new"),
        ("file.open", "Open…", "o", false, "open"),
        ("file.save", "Save", "s", false, "save"),
        ("file.save_as", "Save As…", "s", true, "save_as"),
    ] {
        let control = controls
            .iter()
            .find(|control| control["id"] == id)
            .ok_or("file control")?;
        assert_eq!(control["label"], label);
        assert_eq!(control["menu"], "File");
        assert_eq!(
            control["shortcut"],
            json!({"key":key,"primary":true,"shift":shift})
        );
        assert_eq!(
            control["command"],
            json!({"command":"ui.file","params":{"expected_revision":0,"action":action}})
        );
        assert_eq!(control["enabled"], false);
        assert_eq!(control["disabled_reason"], "ui_unavailable");
    }
    let settings = controls
        .iter()
        .find(|control| control["id"] == "file.document_settings")
        .ok_or("Document Settings binding")?;
    assert_eq!(settings["label"], "Document Settings…");
    assert_eq!(settings["menu"], "File");
    assert_eq!(settings["shortcut"], Value::Null);
    assert_eq!(
        settings["command"],
        json!({"command":"ui.drawing_settings",
        "params":{"action":"open","expected_revision":0}})
    );
    assert_eq!(settings["enabled"], false);
    assert_eq!(settings["disabled_reason"], "ui_unavailable");
    let preferences = controls
        .iter()
        .find(|control| control["id"] == "file.preferences")
        .ok_or("Preferences binding")?;
    assert_eq!(preferences["label"], "Preferences…");
    assert_eq!(preferences["menu"], "File");
    assert_eq!(preferences["shortcut"], Value::Null);
    assert_eq!(
        preferences["command"],
        json!({"command":"ui.preferences",
        "params":{"action":"open","expected_revision":0}})
    );
    assert_eq!(preferences["enabled"], false);
    assert_eq!(preferences["disabled_reason"], "ui_unavailable");
    let rotate = controls
        .iter()
        .find(|control| control["id"] == "page.rotate")
        .ok_or("rotate control")?;
    assert_eq!(
        rotate["command"]["params"]["width_pt"],
        result["document"]["pages"][0]["size"]["height_pt"]
    );
    assert_eq!(
        rotate["command"]["params"]["height_pt"],
        result["document"]["pages"][0]["size"]["width_pt"]
    );
    Ok(())
}

#[test]
fn cli_presentation_requests_refuse_missing_host_and_reject_unknown_or_inconsistent_fields()
-> Result<(), Box<dyn std::error::Error>> {
    for (params, code) in [
        (
            json!({"expected_revision":0,"action":"open"}),
            "ui_unavailable",
        ),
        (
            json!({"expected_revision":0,"action":"save"}),
            "ui_unavailable",
        ),
        (
            json!({"expected_revision":0,"action":"save_as"}),
            "ui_unavailable",
        ),
        (
            json!({"expected_revision":0,"action":"cancel","request_id":1}),
            "ui_unavailable",
        ),
        (
            json!({"expected_revision":1,"action":"open"}),
            "stale_revision",
        ),
        (
            json!({"expected_revision":0,"action":"open","path":"must-not-open.cdxml"}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":0,"action":"choose"}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":0,"action":"cancel"}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":0,"action":"open","request_id":1}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":0,"action":"save_as","close_epoch":0}),
            "invalid_params",
        ),
    ] {
        let output = invoke("ui.file", params)?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(error["error"]["code"], code);
    }
    let invalid_query = invoke("ui.controls", json!({"force":true}))?;
    assert_eq!(invalid_query.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&invalid_query.stderr)?["error"]["code"],
        "invalid_params"
    );
    Ok(())
}
