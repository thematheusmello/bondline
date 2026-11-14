use serde_json::{Value, json};
use std::process::{Command, Output};

fn run(commands: &[Value]) -> Result<Output, Box<dyn std::error::Error>> {
    let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    process.arg("run");
    for command in commands {
        process.arg("--cmd").arg(command.to_string());
    }
    Ok(process.output()?)
}

fn close(action: Option<&str>, expected_revision: u64) -> Value {
    let mut params = json!({"expected_revision":expected_revision});
    if let Some(action) = action {
        params["action"] = action.into();
    }
    json!({"command":"document.close","params":params})
}

fn draw() -> Value {
    json!({"command":"bond.create","params":{
        "page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200
    }})
}

#[test]
fn close_discovery_and_cli_policy_preserve_authored_revision_and_history()
-> Result<(), Box<dyn std::error::Error>> {
    let list = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(list.status.success());
    let commands: Value = serde_json::from_slice(&list.stdout)?;
    let commands = commands.as_array().ok_or("command array")?;
    assert_eq!(commands.len(), 24);
    let info = commands
        .iter()
        .find(|info| info["name"] == "document.close")
        .ok_or("close discovery")?;
    assert_eq!(info["read_only"], false);
    assert_eq!(info["input_schema"]["additionalProperties"], false);
    assert_eq!(
        info["input_schema"]["required"],
        json!(["expected_revision"])
    );
    assert_eq!(
        info["input_schema"]["properties"]["action"]["default"],
        "request"
    );

    for (commands, state, revision, dirty, epoch) in [
        (vec![close(None, 0)], "approved", 0, false, 1),
        (vec![draw(), close(None, 1)], "needs_decision", 1, true, 1),
        (
            vec![draw(), close(None, 1), close(Some("cancel"), 1)],
            "open",
            1,
            true,
            2,
        ),
        (
            vec![draw(), close(None, 1), close(Some("discard"), 1)],
            "approved",
            1,
            true,
            2,
        ),
    ] {
        let output = run(&commands)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(result["close"], state);
        assert_eq!(result["close_operation"], state);
        assert_eq!(result["close_epoch"], epoch);
        assert_eq!(result["changed"], false);
        assert_eq!(result["revision"], revision);
        assert_eq!(result["undo_depth"], revision);
        assert_eq!(result["redo_depth"], 0);
        assert_eq!(result["file"]["dirty"], dirty);
    }
    Ok(())
}

#[test]
fn cli_close_errors_are_typed_and_inspect_preserves_unacknowledged_policy_state()
-> Result<(), Box<dyn std::error::Error>> {
    for (commands, expected) in [
        (vec![draw(), close(None, 0)], "stale_revision"),
        (vec![close(Some("quit"), 0)], "invalid_params"),
        (
            vec![json!({"command":"document.close","params":{}})],
            "invalid_params",
        ),
        (
            vec![json!({"command":"document.close","params":{"expected_revision":0,"force":true}})],
            "invalid_params",
        ),
    ] {
        let output = run(&commands)?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(error["error"]["code"], expected);
    }
    let output = run(&[
        close(None, 0),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["close"], "approved");
    assert!(result.get("close_operation").is_none());
    Ok(())
}
