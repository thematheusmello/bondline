//! Independent same-format bytes through the actual headless executable.
use serde_json::{Value, json};
use std::{fs, process::Command};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated actual-file directory")]
mod files;

const BEFORE: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/mixed_before.cdxml");
const AFTER: &[u8] = include_bytes!("../../io/tests/fixtures/caption-create/mixed_after.cdxml");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/caption-create/cases.json");

fn run_cli(
    requests: impl IntoIterator<Item = (&'static str, Value)>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut cli = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    cli.arg("run");
    for (command, params) in requests {
        cli.arg("--cmd")
            .arg(json!({"command":command,"params":params}).to_string());
    }
    let result = cli.output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

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
    let reopened = run_cli([
        ("document.open_cdxml", json!({"path":input})),
        ("text.create", params),
        ("history.undo", json!({})),
        ("history.redo", json!({})),
        ("document.save_cdxml", json!({"path":output})),
        ("document.open_cdxml", json!({"path":output})),
    ])?;
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

#[test]
fn headless_cdx_creation_reopens_exact_sparse_bytes_and_matches_native_gpu_capture()
-> Result<(), Box<dyn std::error::Error>> {
    let before = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/single_before.cdx");
    let sparse = include_bytes!("../../io/tests/fixtures/cdx-caption-create/after_undo.cdx");
    let witnesses: Value = serde_json::from_str(include_str!(
        "../../io/tests/fixtures/cdx-caption-edit/cases.json"
    ))?;
    let body = "Phase β & q";
    let mut expected = witnesses["cases"]["single_grow"]["before_model"].clone();
    expected["pages"][0]["texts"]
        .as_array_mut()
        .ok_or("original caption list")?
        .push(json!({"id":3,"position":{"x_pt":144.25,"y_pt":216.75},
                    "runs":[{"text":body,"style":{"font_family":"Roboto","size_pt":12.0,
                                                "face":0,"rgb":[0.0,0.0,0.0]}}]}));
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("existing-caption-原-λ.cdx");
    let output = directory.path("new-cdx-caption-保存-β.cdx");
    fs::write(&input, before)?;
    let create = |revision| json!({"expected_revision":revision,"page_id":1,"x_pt":144.25,"y_pt":216.75,"text":body});
    let capture = |backend| json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":backend});
    let cpu = run_cli([
        ("document.open_cdx", json!({"path":input})),
        ("text.create", create(1)),
        ("history.undo", json!({})),
        ("text.create", create(3)),
        ("document.save_cdx", json!({"path":output})),
        ("document.open_cdx", json!({"path":output})),
        ("render.capture", capture("cpu")),
    ])?;
    assert_eq!(fs::read(&input)?, before);
    assert_eq!(fs::read(&output)?, sparse);
    assert_eq!(cpu["document"], expected);
    assert_eq!(cpu["revision"], 5);
    assert_eq!(cpu["undo_depth"], 0);
    assert_eq!(cpu["redo_depth"], 0);
    assert_eq!(cpu["file"]["dirty"], false);
    assert_eq!(
        cpu["source"],
        json!({"format":"cdx","byte_length":sparse.len(),"modified":false})
    );
    let baseline = run_cli([
        ("document.open_cdx", json!({"path":input})),
        ("render.capture", capture("cpu")),
    ])?;
    let gpu = run_cli([
        ("document.open_cdx", json!({"path":output})),
        ("render.capture", capture("gpu")),
    ])?;
    assert_eq!(gpu["document"], expected);
    assert_eq!(gpu["capture"]["backend"], "gpu");
    assert!(
        gpu["capture"]["adapter"]["name"]
            .as_str()
            .is_some_and(|name| !name.is_empty())
    );
    let png = cpu["capture"]["png_base64"]
        .as_str()
        .filter(|png| !png.is_empty())
        .ok_or("created caption PNG")?;
    assert_ne!(baseline["capture"]["png_base64"].as_str(), Some(png));
    assert_eq!(gpu["capture"]["png_base64"].as_str(), Some(png));
    Ok(())
}
