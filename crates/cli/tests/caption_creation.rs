//! Independent same-format bytes through the actual headless executable.
use serde_json::{Value, json};
use std::{fs, process::Command};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated actual-file directory")]
mod files;

const BEFORE: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/mixed_before.cdxml");
const AFTER: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/mixed_after.cdxml");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/caption-create/cases.json");

#[test]
fn headless_creation_round_trips_the_independent_mixed_unicode_caption()
-> Result<(), Box<dyn std::error::Error>> {
    let case: Value = serde_json::from_str(WITNESSES)?;
    let mixed = &case["cases"]["mixed_append"];
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("original-原-λ.cdxml");
    let output = directory.path("created-caption-保存-β.cdxml");
    fs::write(&input, BEFORE)?;
    let mut params = mixed["params"].clone();
    params["expected_revision"] = json!(1);
    let mut cli = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    cli.arg("run");
    for (command, params) in [
        ("document.open_cdxml", json!({"path":input})),
        ("text.create", params),
        ("history.undo", json!({})),
        ("history.redo", json!({})),
        ("document.save_cdxml", json!({"path":output})),
        ("document.open_cdxml", json!({"path":output})),
    ] {
        cli.arg("--cmd")
            .arg(json!({"command":command,"params":params}).to_string());
    }
    let result = cli.output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let reopened: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(reopened["document"], mixed["after_model"]);
    assert_eq!(reopened["revision"], 5);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["redo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(
        reopened["source"],
        json!({"format":"cdxml","byte_length":AFTER.len(),"modified":false})
    );
    assert_eq!(fs::read(&input)?, BEFORE);
    assert_eq!(fs::read(&output)?, AFTER);
    Ok(())
}
