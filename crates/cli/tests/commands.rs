use serde_json::{Value, json};
use std::process::Command;
#[path = "support/cdx.rs"]
mod cdx_support;
#[path = "support/chain.rs"]
mod chain_support;
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;
#[path = "support/mol.rs"]
mod mol_support;

#[test]
fn executable_extends_saves_undoes_reopens_and_extends_the_same_native_chain()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("cli-propane.cdxml");
    let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    process.arg("run");
    for request in [
        json!({"command":"bond.create","params":chain_support::create()}),
        json!({"command":"bond.extend","params":chain_support::extend(1,2,180.0,240.0)}),
        json!({"command":"document.save_cdxml","params":{"path":path}}),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
    ] {
        process.arg("--cmd").arg(request.to_string());
    }
    let output = process.output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        (result["revision"].as_u64(), result["undo_depth"].as_u64()),
        (Some(4), Some(2))
    );
    assert_eq!(result["file"]["dirty"], false);
    assert_eq!(
        result["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        1
    );
    chain_support::independently_read(&path, 3)?;
    let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    process.arg("run");
    for request in [
        json!({"command":"document.open_cdxml","params":{"path":path}}),
        json!({"command":"bond.extend","params":chain_support::extend(1,3,220.0,280.0)}),
        json!({"command":"document.save_cdxml","params":{"path":path,"overwrite":true}}),
    ] {
        process.arg("--cmd").arg(request.to_string());
    }
    let output = process.output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["source"]["format"], "cdxml");
    assert_eq!(result["source"]["modified"], true);
    assert_eq!(result["file"]["dirty"], false);
    chain_support::independently_read(&path, 4)?;
    Ok(())
}

#[test]
fn executable_lists_schemas_and_runs_a_stateful_command_sequence()
-> Result<(), Box<dyn std::error::Error>> {
    let binary = env!("CARGO_BIN_EXE_bondline-cli");
    let list = Command::new(binary).arg("commands").output()?;
    assert!(list.status.success());
    let commands: Value = serde_json::from_slice(&list.stdout)?;
    let commands = commands.as_array().ok_or("expected command array")?;
    let mut names = commands
        .iter()
        .map(|info| info["name"].as_str().ok_or("command name"))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "bond.create",
            "bond.extend",
            "document.close",
            "document.inspect",
            "document.open_cdx",
            "document.open_cdxml",
            "document.open_mol",
            "document.save_cdx",
            "document.save_cdxml",
            "document.save_mol",
            "history.redo",
            "history.undo",
            "page.set_size",
            "render.capture",
            "ui.controls",
            "ui.file"
        ]
    );
    for (name, permission) in [
        ("document.open_cdx", "discard_unsaved"),
        ("document.save_cdx", "overwrite"),
    ] {
        let info = commands
            .iter()
            .find(|info| info["name"] == name)
            .ok_or("CDX command")?;
        assert_eq!(info["read_only"], false);
        let schema = &info["input_schema"];
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], json!(["path"]));
        let properties = schema["properties"].as_object().ok_or("CDX properties")?;
        assert_eq!(
            properties
                .keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from(["path", permission])
        );
        assert_eq!(properties[permission]["type"], "boolean");
        assert_eq!(properties[permission]["default"], false);
    }
    assert!(
        commands
            .iter()
            .any(|command| command["name"] == "page.set_size"
                && command["input_schema"]["properties"]["width_pt"]["exclusiveMinimum"] == 0.0)
    );
    for name in ["document.open_mol", "document.save_mol"] {
        let command = commands
            .iter()
            .find(|info| info["name"] == name)
            .ok_or("MOL command")?;
        assert_eq!(command["read_only"], false);
        let schema = &command["input_schema"];
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], json!(["path"]));
        let properties = schema["properties"].as_object().ok_or("MOL properties")?;
        let keys: std::collections::BTreeSet<_> = properties.keys().map(String::as_str).collect();
        if name == "document.open_mol" {
            assert_eq!(
                keys,
                std::collections::BTreeSet::from([
                    "path",
                    "discard_unsaved",
                    "points_per_unit",
                    "origin_x_pt",
                    "origin_y_pt"
                ])
            );
            assert_eq!(properties["discard_unsaved"]["default"], false);
            for (key, default) in [
                ("points_per_unit", 24.0),
                ("origin_x_pt", 72.0),
                ("origin_y_pt", 72.0),
            ] {
                assert_eq!(properties[key]["type"], "number");
                assert_eq!(properties[key]["default"].as_f64(), Some(default));
            }
        } else {
            assert_eq!(
                keys,
                std::collections::BTreeSet::from(["path", "overwrite", "version"])
            );
            assert_eq!(properties["overwrite"]["default"], false);
            assert_eq!(properties["version"]["default"], "auto");
            assert_eq!(
                schema["$defs"]["MolVersionChoice"]["enum"],
                json!(["auto", "v2000", "v3000"])
            );
        }
    }
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
        state["file"],
        json!({"path":null,"dirty":true,"format":"cdxml"})
    );
    assert_eq!(
        state["document"]["pages"][0],
        json!({"id":1,"size":{"width_pt":612.0,"height_pt":792.0},"fragments":[]})
    );
    Ok(())
}

#[test]
fn executable_cdx_open_save_uses_complete_source_and_refuses_lossy_exports()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("input-λ.mol");
    let copy = directory.path("copy.cdx");
    std::fs::write(&input, cdx_support::SOURCE)?;
    let open = json!({"command":"document.open_cdx","params":{"path":input}}).to_string();
    let save = json!({"command":"document.save_cdx","params":{"path":copy}}).to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args(["run", "--cmd", &open, "--cmd", &save])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    cdx_support::assert_state(&result)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["file_operation"]["format"], "cdx");
    assert_eq!(result["file_operation"]["operation"], "save");
    cdx_support::independently_read_saved(&copy)?;
    for (command, params, code) in [
        ("document.save_cdx", json!({"path":copy}), "already_exists"),
        (
            "document.save_cdxml",
            json!({"path":copy,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "document.save_mol",
            json!({"path":copy,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "document.open_cdx",
            json!({"path":input,"version":"auto"}),
            "invalid_params",
        ),
    ] {
        let request = json!({"command":command,"params":params}).to_string();
        let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
            .args(["run", "--cmd", &open, "--cmd", &request])
            .output()?;
        assert!(!output.status.success());
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(error["error"]["code"], code);
        assert_eq!(std::fs::read(&copy)?, cdx_support::SOURCE);
    }
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
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.open_mol","params":{"path":"unused.mol","unexpected":true}}"#,
            ],
            "invalid_params",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.save_mol","params":{"path":"unused.mol","format":"v3000"}}"#,
            ],
            "invalid_params",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.open_mol","params":{"path":"unused.mol","points_per_unit":0}}"#,
            ],
            "invalid_params",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.save_mol","params":{"path":"unused.mol","version":"v4000"}}"#,
            ],
            "invalid_params",
        ),
        (
            vec![
                "run",
                "--cmd",
                r#"{"command":"document.save_mol","params":{"path":"unused.mol","version":null}}"#,
            ],
            "invalid_params",
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
        json!({"path":std::fs::canonicalize(&path)?,"dirty":false,"format":"cdxml"})
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

#[test]
fn executable_opens_and_saves_real_mol_with_source_and_headers_intact()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let source = directory.path("methyl-source.mol");
    let destination = directory.path("methyl-copy.mol");
    let bytes = mol_support::METHYL;
    std::fs::write(&source, bytes)?;
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run",
            "--cmd",
            &json!({"command":"document.open_mol","params":{"path":source}}).to_string(),
            "--cmd",
            &json!({"command":"document.save_mol","params":{"path":destination}}).to_string(),
        ])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(
        result["file"],
        json!({
            "path":std::fs::canonicalize(&destination)?,"dirty":false,"format":"mol_v2000"
        })
    );
    mol_support::assert_methyl_state(&result, 42.3, 20.3784)?;
    assert_eq!(result["file_operation"]["operation"], "save");
    assert_eq!(result["file_operation"]["input_revision"], 1);
    assert_eq!(std::fs::read(&destination)?, bytes);
    let external = mol_support::independently_read_mol(&destination)?;
    assert_eq!(
        external["source_sha256"],
        "afd2352697ca20c0d580c4981fe8bd24da36c4e1661e9d2aee501a40364fd42d"
    );
    assert_eq!(external["smiles"], "[CH3]");
    assert_eq!(external["formula"], "CH3");
    assert_eq!(external["atoms"][0]["hydrogens"], 3);
    assert_eq!(external["atoms"][0]["radical_electrons"], 1);
    Ok(())
}

#[test]
fn executable_exports_native_cc_to_mol_and_preserves_association_through_history()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("native-ethane.mol");
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli")).args([
        "run", "--cmd", r#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":108,"end_y_pt":72}}"#,
        "--cmd", &json!({"command":"document.save_mol","params":{"path":path}}).to_string(),
        "--cmd", r#"{"command":"history.undo","params":{}}"#,
        "--cmd", r#"{"command":"history.redo","params":{}}"#,
    ]).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["revision"], 3);
    assert_eq!(result["undo_depth"], 1);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(
        result["file"],
        json!({"path":std::fs::canonicalize(&path)?,"dirty":false,"format":"mol_v2000"})
    );
    assert!(result.get("source").is_none());
    let external = mol_support::independently_read_mol(&path)?;
    assert_eq!(external["smiles"], "CC");
    assert_eq!(external["formula"], "C2H6");
    assert_eq!(external["atoms"].as_array().ok_or("atoms")?.len(), 2);
    assert_eq!(external["bonds"], json!([{"begin":0,"end":1,"order":1.0}]));
    for (index, position) in [json!([0.0, 0.0, 0.0]), json!([1.5, 0.0, 0.0])]
        .into_iter()
        .enumerate()
    {
        assert_eq!(external["atoms"][index]["position"], position);
        assert_eq!(external["atoms"][index]["hydrogens"], 3);
        assert_eq!(external["atoms"][index]["radical_electrons"], 0);
    }
    let reopened = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run",
            "--cmd",
            &json!({"command":"document.open_mol","params":{"path":path}}).to_string(),
        ])
        .output()?;
    assert!(
        reopened.status.success(),
        "{}",
        String::from_utf8_lossy(&reopened.stderr)
    );
    let reopened: Value = serde_json::from_slice(&reopened.stdout)?;
    assert_eq!(reopened["document"], result["document"]);
    assert_eq!(reopened["file"], result["file"]);
    assert_eq!(reopened["undo_depth"], 0);
    Ok(())
}

#[test]
fn executable_exports_explicit_v3000_at_full_precision_and_auto_keeps_its_version()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("explicit-v3.v2000");
    let copy = directory.path("auto-v3.copy");
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli")).args([
        "run", "--cmd", r#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}}"#,
        "--cmd", &json!({"command":"document.save_mol","params":{"path":path,"version":"v3000"}}).to_string(),
        "--cmd", &json!({"command":"document.save_mol","params":{"path":copy}}).to_string(),
    ]).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["undo_depth"], 1);
    assert_eq!(result["changed"], false);
    assert_eq!(
        result["file"],
        json!({"path":std::fs::canonicalize(&copy)?,"dirty":false,"format":"mol_v3000"})
    );
    assert_eq!(result["file_operation"]["format"], "mol_v3000");
    assert_eq!(result["file_operation"]["input_revision"], 1);
    assert!(result.get("source").is_none());
    assert_eq!(std::fs::read(&path)?, std::fs::read(&copy)?);
    let external = mol_support::independently_read_v3000(&copy)?;
    assert_eq!(external["rdkit"], "2026.09.2");
    assert_eq!(external["atoms"].as_array().ok_or("atoms")?.len(), 2);
    assert_eq!(
        external["bonds"],
        json!([{"begin":0,"end":1,"order":1.0,"direction":"NONE","stereo":"STEREONONE"}])
    );
    for (index, x) in [28.0 / 24.0, 68.0 / 24.0].into_iter().enumerate() {
        let atom = &external["atoms"][index];
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_h"], 3);
        assert_eq!(atom["radical_electrons"], 0);
        assert!((atom["position"][0].as_f64().ok_or("x")? - x).abs() < 1e-14);
        assert!((atom["position"][1].as_f64().ok_or("y")? + 128.0 / 24.0).abs() < 1e-14);
        assert_eq!(atom["position"][2], 0.0);
    }
    let reopened = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run",
            "--cmd",
            &json!({"command":"document.open_mol","params":{"path":copy}}).to_string(),
        ])
        .output()?;
    assert!(
        reopened.status.success(),
        "{}",
        String::from_utf8_lossy(&reopened.stderr)
    );
    let reopened: Value = serde_json::from_slice(&reopened.stdout)?;
    assert_eq!(reopened["file"], result["file"]);
    assert_eq!(reopened["undo_depth"], 0);
    for (atom, x) in reopened["document"]["pages"][0]["fragments"][0]["atoms"]
        .as_array()
        .ok_or("atoms")?
        .iter()
        .zip([100.0, 140.0])
    {
        assert!((atom["position"]["x_pt"].as_f64().ok_or("x")? - x).abs() < 1e-10);
        assert!((atom["position"]["y_pt"].as_f64().ok_or("y")? - 200.0).abs() < 1e-10);
        assert_eq!(atom["explicit_hydrogens"], 3);
    }
    Ok(())
}

#[test]
fn executable_opens_actual_v3000_and_default_save_preserves_all_original_source_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let source = directory.path("original-v3.v2000");
    let copy = directory.path("original-v3-copy.mol");
    std::fs::write(&source, mol_support::V3000_SOURCE)?;
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .args([
            "run",
            "--cmd",
            &json!({"command":"document.open_mol","params":{"path":source}}).to_string(),
            "--cmd",
            &json!({"command":"document.save_mol","params":{"path":copy}}).to_string(),
        ])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    mol_support::assert_v3000_state(&result)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(result["changed"], false);
    assert_eq!(result["file"]["path"], json!(std::fs::canonicalize(&copy)?));
    assert_eq!(result["file_operation"]["format"], "mol_v3000");
    assert_eq!(result["file_operation"]["input_revision"], 1);
    assert_eq!(result["file_operation"]["byte_length"], 344);
    assert_eq!(std::fs::read(&copy)?, mol_support::V3000_SOURCE);
    let external = mol_support::independently_read_v3000(&copy)?;
    assert_eq!(
        external["source_sha256"],
        "1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380"
    );
    assert_eq!(external["atoms"][0]["position"], json!([-1.25, 0.5, 0.0]));
    assert_eq!(external["atoms"][1]["position"], json!([0.25, 0.5, 0.0]));
    assert_eq!(external["atoms"][0]["explicit_h"], 3);
    assert_eq!(external["atoms"][1]["explicit_h"], 3);
    assert_eq!(external["bonds"][0]["order"], 1.0);
    Ok(())
}
