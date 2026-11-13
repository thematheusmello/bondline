use serde_json::{Value, json};
use std::process::Command;

#[test]
fn executable_lists_schemas_and_runs_a_stateful_command_sequence()
-> Result<(), Box<dyn std::error::Error>> {
    let binary = env!("CARGO_BIN_EXE_bondline-cli");
    let list = Command::new(binary).arg("commands").output()?;
    assert!(list.status.success());
    let commands: Value = serde_json::from_slice(&list.stdout)?;
    let commands = commands.as_array().ok_or("expected command array")?;
    assert_eq!(commands.len(), 4);
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
        json!({"id":1,"size":{"width_pt":612.0,"height_pt":792.0}})
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
