use serde_json::{Value, json};
use std::process::{Command, Output};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the original isolated file directory")]
mod file_support;
#[path = "support/mol.rs"]
#[allow(
    dead_code,
    reason = "reuse the unchanged actual-file molecular consumer"
)]
mod mol_support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn request(command: &str, params: Value) -> Value {
    json!({"command":command,"params":params})
}
fn create(order: &str) -> Value {
    request(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":144,"end_x_pt":120,"end_y_pt":144,"order":order}),
    )
}
fn change(revision: u64, order: &str) -> Value {
    request(
        "bond.set_order",
        json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"bond_id":1,"order":order,"hydrogen_policy":"rectify_neutral_carbon"}),
    )
}
fn run(requests: &[Value]) -> std::io::Result<Output> {
    let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    process.arg("run");
    for request in requests {
        process.arg("--cmd").arg(request.to_string());
    }
    process.output()
}
fn success(requests: &[Value]) -> TestResult<Value> {
    let output = run(requests)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn graph(result: &Value, order: &str, h: u16) -> TestResult {
    assert_eq!(
        result["document"]["pages"][0]["fragments"],
        json!([{"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":72.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h},
        {"id":2,"element":6,"position":{"x_pt":120.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":order}]}])
    );
    Ok(())
}
fn molecule(path: &std::path::Path, smiles: &str, formula: &str, h: u16, weight: u8) -> TestResult {
    let value = mol_support::independently_read_mol(path)?;
    assert_eq!(value["smiles"], smiles);
    assert_eq!(value["formula"], formula);
    let atoms = value["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 2);
    for (a, p) in atoms.iter().zip([[0.0, -3.0, 0.0], [2.0, -3.0, 0.0]]) {
        assert_eq!(a["hydrogens"], h);
        assert_eq!(a["element"], 6);
        assert_eq!(a["position"], json!(p));
    }
    assert_eq!(
        value["bonds"],
        json!([{"begin":0,"end":1,"order":f64::from(weight)}])
    );
    Ok(())
}

#[test]
fn cli_order_edit_survives_history_native_reopen_and_real_file_consumers() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    let double = directory.path("double-化学.cdxml");
    let double_mol = directory.path("double-化学.mol");
    let triple = directory.path("triple-λ.cdxml");
    let triple_mol = directory.path("triple-λ.mol");
    let value = success(&[
        create("single"),
        change(1, "double"),
        request("document.save_cdxml", json!({"path":double})),
        request(
            "document.save_mol",
            json!({"path":double_mol,"version":"v3000"}),
        ),
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request("document.open_cdxml", json!({"path":double})),
        change(5, "triple"),
        request("document.save_cdxml", json!({"path":triple})),
        request(
            "document.save_mol",
            json!({"path":triple_mol,"version":"v2000"}),
        ),
    ])?;
    graph(&value, "triple", 1)?;
    assert_eq!(value["revision"], 6);
    assert_eq!(value["source"]["format"], "cdxml");
    assert_eq!(value["source"]["modified"], true);
    assert_eq!(value["file"]["dirty"], false);
    molecule(&double_mol, "C=C", "C2H4", 2, 2)?;
    molecule(&triple_mol, "C#C", "C2H2", 1, 3)?;
    graph(
        &success(&[request("document.open_cdxml", json!({"path":double}))])?,
        "double",
        2,
    )?;
    graph(
        &success(&[request("document.open_cdxml", json!({"path":triple}))])?,
        "triple",
        1,
    )?;
    Ok(())
}

#[test]
fn cli_same_order_keeps_redo_and_the_original_revision() -> TestResult {
    let mut requests = vec![
        create("double"),
        request(
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        ),
        request("history.undo", json!({})),
        change(3, "double"),
    ];
    let same = success(&requests)?;
    graph(&same, "double", 2)?;
    assert_eq!(same["changed"], false);
    assert_eq!(same["revision"], 3);
    assert_eq!(same["undo_depth"], 1);
    assert_eq!(same["redo_depth"], 1);
    requests.push(request("history.redo", json!({})));
    let redone = success(&requests)?;
    assert_eq!(redone["revision"], 4);
    assert_eq!(
        redone["document"]["pages"][0]["size"],
        json!({"width_pt":612.0,"height_pt":792.0})
    );
    Ok(())
}

#[test]
fn cli_refusals_report_typed_errors_and_never_publish_later_output() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    for (index, key, value, code) in [
        (0, "expected_revision", json!(0), "stale_revision"),
        (1, "bond_id", json!(0), "invalid_params"),
        (2, "bond_id", json!(99), "not_found"),
        (3, "order", json!("aromatic"), "invalid_params"),
        (
            4,
            "hydrogen_policy",
            json!("rectify_terminal_carbon"),
            "invalid_params",
        ),
        (5, "extra", json!(true), "invalid_params"),
    ] {
        let path = directory.path(&format!("must-not-exist-{index}.cdxml"));
        let mut invalid = change(1, "double");
        invalid["params"][key] = value;
        let output = run(&[
            create("single"),
            invalid,
            request("document.save_cdxml", json!({"path":path})),
        ])?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(error["error"]["code"], code);
        assert!(!path.try_exists()?);
    }
    let source = directory.path("actual-source.mol");
    success(&[
        create("single"),
        request("document.save_mol", json!({"path":source})),
    ])?;
    let raw = std::fs::read(&source)?;
    let output = run(&[
        request("document.open_mol", json!({"path":source})),
        change(1, "double"),
    ])?;
    assert_eq!(output.status.code(), Some(2));
    let error: Value = serde_json::from_slice(&output.stderr)?;
    assert_eq!(error["error"]["code"], "unsupported_content");
    assert_eq!(std::fs::read(&source)?, raw);
    Ok(())
}

#[test]
fn cli_registry_exposes_the_closed_mutating_order_edit_command() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(output.status.success());
    let listed: Value = serde_json::from_slice(&output.stdout)?;
    let commands = listed.as_array().ok_or("commands")?;
    let entry = commands
        .iter()
        .find(|v| v["name"] == "bond.set_order")
        .ok_or("order command missing")?;
    assert_eq!(entry["read_only"], false);
    assert_eq!(entry["input_schema"]["additionalProperties"], false);
    let mut required = entry["input_schema"]["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|x| x.as_str().ok_or("name"))
        .collect::<Result<Vec<_>, _>>()?;
    required.sort_unstable();
    assert_eq!(
        required,
        [
            "bond_id",
            "expected_revision",
            "fragment_id",
            "hydrogen_policy",
            "order",
            "page_id"
        ]
    );
    Ok(())
}
