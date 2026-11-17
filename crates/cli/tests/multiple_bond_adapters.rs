use serde_json::{Value, json};
use std::{collections::BTreeSet, process::Command};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated file directory")]
mod file_support;
#[path = "support/mol.rs"]
#[allow(
    dead_code,
    reason = "reuse only the existing generic external MOL consumer"
)]
mod mol_support;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn request(name: &str, params: Value) -> Value {
    json!({"command":name,"params":params})
}
fn create(order: Option<&str>) -> Value {
    let mut params =
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72});
    if let Some(order) = order {
        params["order"] = json!(order);
    }
    request("bond.create", params)
}
fn extend(order: Value) -> Value {
    request(
        "bond.extend",
        json!({"expected_revision":1,"page_id":1,"fragment_id":1,"begin_atom_id":2,"end_x_pt":168,"end_y_pt":72,"hydrogen_policy":"rectify_terminal_carbon","order":order}),
    )
}
fn cli(requests: &[Value]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    command.arg("run");
    for request in requests {
        command.arg("--cmd").arg(request.to_string());
    }
    let output = command.output()?;
    assert!(
        output.status.success(),
        "actual CLI operation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn graph(value: &Value, h: &[u16], orders: &[&str]) -> TestResult {
    let fragments = value["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let fragment = &fragments[0];
    assert_eq!(fragment["id"], 1);
    let atoms = fragment["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), h.len());
    for (i, (atom, &h)) in atoms.iter().zip(h).enumerate() {
        assert_eq!(atom["id"], i + 1);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], h);
        assert_eq!(
            atom["position"],
            json!({"x_pt":72.0+48.0*i as f64,"y_pt":72.0})
        );
    }
    assert_eq!(
        fragment["bonds"],
        json!(
            orders
                .iter()
                .enumerate()
                .map(|(i, &order)| json!({"id":i+1,"begin":i+1,"end":i+2,"order":order}))
                .collect::<Vec<_>>()
        )
    );
    Ok(())
}

#[test]
fn baseline_cli_multiple_orders_round_trip_and_list_typed_commands() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    for (order, weight) in [("double", 2_u16), ("triple", 3)] {
        let pair = cli(&[create(Some(order))])?;
        graph(&pair, &[4 - weight, 4 - weight], &[order])?;
        assert_eq!(
            (pair["revision"].clone(), pair["undo_depth"].clone()),
            (json!(1), json!(1))
        );
        let xml = directory.path(&format!("cli-{order}-λ.cdxml"));
        let mol = directory.path(&format!("cli-{order}-λ.mol"));
        let state = cli(&[
            create(None),
            extend(json!(order)),
            request("document.save_cdxml", json!({"path":xml})),
            request("document.save_mol", json!({"path":mol})),
            request("history.undo", json!({})),
            request("history.redo", json!({})),
            request("document.open_cdxml", json!({"path":xml})),
            request("document.inspect", json!({})),
        ])?;
        graph(&state, &[3, 3 - weight, 4 - weight], &["single", order])?;
        assert_eq!(state["revision"], 5);
        assert_eq!(state["undo_depth"], 0);
        assert_eq!(state["redo_depth"], 0);
        assert_eq!(state["source"]["format"], "cdxml");
        assert_eq!(state["source"]["modified"], false);
        assert_eq!(state["file"]["dirty"], false);
        assert!(state.get("ui_bond_tool_operation").is_none());
        let external = mol_support::independently_read_mol(&mol)?;
        assert_eq!(
            external["formula"],
            if weight == 2 { "C3H6" } else { "C3H4" }
        );
        assert_eq!(
            external["smiles"],
            if weight == 2 { "C=CC" } else { "C#CC" }
        );
        assert_eq!(
            external["atoms"]
                .as_array()
                .ok_or("external atoms")?
                .iter()
                .map(|atom| atom["hydrogens"].clone())
                .collect::<Vec<_>>(),
            vec![json!(3), json!(3 - weight), json!(4 - weight)]
        );
        assert_eq!(
            external["bonds"],
            json!([{"begin":0,"end":1,"order":1.0},{"begin":1,"end":2,"order":f64::from(weight)}])
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(output.status.success());
    let commands: Vec<Value> = serde_json::from_slice(&output.stdout)?;
    let names = commands
        .iter()
        .map(|value| value["name"].as_str().ok_or("name"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(commands.len(), 43);
    assert_eq!(commands.len(), names.len());
    for name in ["bond.create", "bond.extend", "ui.bond_tool"] {
        let command = commands
            .iter()
            .find(|value| value["name"] == name)
            .ok_or("registered order command")?;
        assert_eq!(command["input_schema"]["additionalProperties"], false);
        assert!(command["input_schema"]["properties"].get("order").is_some());
    }
    Ok(())
}

#[test]
fn cli_invalid_order_cannot_publish_a_changed_molecule() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    for (index, value) in [Value::Null, json!("quadruple")].into_iter().enumerate() {
        let path = directory.path(&format!("saved-before-error-{index}.mol"));
        let mut command = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
        command.arg("run");
        for request in [
            create(None),
            request("document.save_mol", json!({"path":path})),
            extend(value),
        ] {
            command.arg("--cmd").arg(request.to_string());
        }
        let output = command.output()?;
        assert!(!output.status.success());
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(error["error"]["code"], "invalid_params");
        let source = mol_support::independently_read_mol(&path)?;
        assert_eq!(source["smiles"], "CC");
        assert_eq!(source["formula"], "C2H6");
        assert_eq!(source["bonds"], json!([{"begin":0,"end":1,"order":1.0}]));
    }
    Ok(())
}
