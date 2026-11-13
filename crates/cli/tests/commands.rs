use serde_json::{Value, json};
use std::process::Command;
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;

#[test]
fn executable_lists_schemas_and_runs_a_stateful_command_sequence()
-> Result<(), Box<dyn std::error::Error>> {
    let binary = env!("CARGO_BIN_EXE_bondline-cli");
    let list = Command::new(binary).arg("commands").output()?;
    assert!(list.status.success());
    let commands: Value = serde_json::from_slice(&list.stdout)?;
    let commands = commands.as_array().ok_or("expected command array")?;
    assert_eq!(commands.len(), 8);
    assert!(
        commands
            .iter()
            .any(|command| command["name"] == "page.set_size"
                && command["input_schema"]["properties"]["width_pt"]["exclusiveMinimum"] == 0.0)
    );
    let result = Command::new(binary)
        .args([
            "run",
            "--cmd",
            r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}"#,
            "--cmd",
            r#"{"command":"history.undo","params":{}}"#,
            "--cmd",
            r#"{"command":"history.redo","params":{}}"#,
        ])
        .output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let state: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(state["revision"], 3);
    assert_eq!(
        state["document"]["pages"][0],
        json!({"id":1,"size":{"width_pt":612.0,"height_pt":792.0},"fragments":[]})
    );
    Ok(())
}

#[test]
fn executable_returns_a_png_capture_from_the_same_authored_revision()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run", "--cmd",
            r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":72,"height_pt":144}}"#,
            "--cmd",
            r#"{"command":"render.capture","params":{"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}}"#,
        ]).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["changed"], false);
    assert_eq!(result["undo_depth"], 1);
    assert_eq!(result["capture"]["width_px"], 8);
    assert_eq!(result["capture"]["height_px"], 8);
    assert_eq!(result["capture"]["backend"], "cpu");
    assert_eq!(result["capture"]["adapter"], Value::Null);
    assert!(
        result["capture"]["png_base64"]
            .as_str()
            .ok_or("missing PNG")?
            .starts_with("iVBORw0KGgo")
    );
    Ok(())
}

#[test]
fn executable_reports_consistent_errors_without_a_success_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let binary = env!("CARGO_BIN_EXE_bondline-cli");
    for (arguments, code) in [
        (
            vec!["run", "--cmd", r#"{"command":"history.undo","params":{}}"#],
            "empty_history",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":-1,"height_pt":72}}"#,
            ],
            "invalid_params",
        ),
        (
            vec!["run", "--cmd", r#"{"command":"missing","params":{}}"#],
            "unknown_command",
        ),
        (vec!["commands", "extra"], "usage"),
        (vec!["run"], "usage"),
    ] {
        let result = Command::new(binary).args(arguments).output()?;
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        let error: Value = serde_json::from_slice(&result.stderr)?;
        assert_eq!(error["error"]["code"], code);
    }
    assert!(
        Command::new(binary)
            .arg("--help")
            .output()?
            .status
            .success()
    );
    Ok(())
}

#[test]
fn executable_creates_and_captures_a_labeled_single_bond() -> Result<(), Box<dyn std::error::Error>>
{
    let result=Command::new(env!("CARGO_BIN_EXE_bondline-cli")).args([
        "run","--cmd",r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":60,"height_pt":60}}"#,
        "--cmd",r#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":10,"begin_y_pt":20.5,"end_x_pt":50,"end_y_pt":20.5}}"#,
        "--cmd",r#"{"command":"render.capture","params":{"page_id":1,"width_px":80,"height_px":80,"backend":"cpu"}}"#,
    ]).output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let state: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(state["revision"], 2);
    assert_eq!(state["undo_depth"], 2);
    let fragment = &state["document"]["pages"][0]["fragments"][0];
    assert_eq!(fragment["atoms"].as_array().ok_or("atoms")?.len(), 2);
    assert_eq!(fragment["atoms"][0]["explicit_hydrogens"], 3);
    assert_eq!(fragment["bonds"][0]["order"], "single");
    assert_eq!(state["capture"]["backend"], "cpu");
    assert!(
        state["capture"]["png_base64"]
            .as_str()
            .ok_or("png")?
            .starts_with("iVBORw0KGgo")
    );
    Ok(())
}

#[test]
fn executable_saves_reopens_and_edits_real_cdxml_across_processes()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("drawing.cdxml");
    let binary = env!("CARGO_BIN_EXE_bondline-cli");
    let draw = json!({"command":"bond.create","params":{"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0}}).to_string();
    let save = json!({"command":"document.save_cdxml","params":{"path":path}}).to_string();
    let output = Command::new(binary)
        .args(["run", "--cmd", &draw, "--cmd", &save])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let saved: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(saved["revision"], 1);
    assert_eq!(
        saved["file"],
        json!({"path":std::fs::canonicalize(&path)?,"dirty":false})
    );
    file_support::independently_read_file(&path, 1)?;
    let open = json!({"command":"document.open_cdxml","params":{"path":path}}).to_string();
    let draw_more = json!({"command":"bond.create","params":{"page_id":1,"begin_x_pt":100.0,"begin_y_pt":300.0,"end_x_pt":140.0,"end_y_pt":300.0}}).to_string();
    let overwrite =
        json!({"command":"document.save_cdxml","params":{"path":path,"overwrite":true}})
            .to_string();
    let output = Command::new(binary)
        .args([
            "run", "--cmd", &open, "--cmd", &draw_more, "--cmd", &overwrite,
        ])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let edited: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(edited["revision"], 2);
    assert_eq!(edited["undo_depth"], 1);
    assert_eq!(edited["file"]["dirty"], false);
    file_support::independently_read_file(&path, 2)?;
    let output = Command::new(binary)
        .args(["run", "--cmd", &open])
        .output()?;
    assert!(output.status.success());
    let reopened: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(reopened["document"], edited["document"]);
    assert_eq!(reopened["undo_depth"], 0);
    Ok(())
}
