use super::{Client, file_support, mol_support};
use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn change(revision: u64, bond: u32, order: &str) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"bond_id":bond,"order":order,"hydrogen_policy":"rectify_neutral_carbon"})
}
fn tool(client: &mut Client, id: u64, name: &str, args: Value) -> TestResult<Value> {
    let response = client.call(id, "tools/call", json!({"name":name,"arguments":args}))?;
    assert_eq!(response["result"]["isError"], false, "{response}");
    let value = response["result"]["structuredContent"].clone();
    let text: Value = serde_json::from_str(
        response["result"]["content"][0]["text"]
            .as_str()
            .ok_or("text")?,
    )?;
    assert_eq!(text, value);
    Ok(value)
}
fn start() -> TestResult<Client> {
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"bond-order-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    Ok(client)
}
fn assert_pair(value: &Value, order: &str, h: u16) -> TestResult {
    assert_eq!(
        value["document"]["pages"][0]["fragments"],
        json!([{"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":72.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h},
        {"id":2,"element":6,"position":{"x_pt":120.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":order}]}])
    );
    Ok(())
}

#[test]
fn stdio_order_edit_retains_graph_history_source_and_actual_saved_molecules() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    let xml = directory.path("stdio-double-化学.cdxml");
    let double = directory.path("stdio-double-λ.mol");
    let triple = directory.path("stdio-triple-λ.mol");
    let mut c = start()?;
    tool(
        &mut c,
        2,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":144,"end_x_pt":120,"end_y_pt":144}),
    )?;
    let edited = tool(&mut c, 3, "bond.set_order", change(1, 1, "double"))?;
    assert_pair(&edited, "double", 2)?;
    assert_eq!(edited["revision"], 2);
    assert_eq!(edited["undo_depth"], 2);
    let listed = c.call(4, "tools/list", json!({}))?;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    let entry = tools
        .iter()
        .find(|x| x["name"] == "bond.set_order")
        .ok_or("order tool")?;
    assert_eq!(entry["annotations"]["readOnlyHint"], false);
    assert_eq!(entry["inputSchema"]["additionalProperties"], false);
    let mut required = entry["inputSchema"]["required"]
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
    tool(&mut c, 5, "document.save_cdxml", json!({"path":xml}))?;
    let xml_before = std::fs::read(&xml)?;
    tool(
        &mut c,
        6,
        "document.save_mol",
        json!({"path":double,"version":"v2000"}),
    )?;
    let undo = tool(&mut c, 7, "history.undo", json!({}))?;
    assert_pair(&undo, "single", 3)?;
    let redo = tool(&mut c, 8, "history.redo", json!({}))?;
    assert_pair(&redo, "double", 2)?;
    let before = tool(&mut c, 9, "document.inspect", json!({}))?;
    let same = tool(&mut c, 10, "bond.set_order", change(4, 1, "double"))?;
    assert_eq!(same, before);
    let bad = c.call(
        11,
        "tools/call",
        json!({"name":"bond.set_order","arguments":change(3,1,"triple")}),
    )?;
    assert_eq!(bad["result"]["isError"], true);
    assert_eq!(
        bad["result"]["structuredContent"]["error"]["code"],
        "stale_revision"
    );
    let mut missing = change(4, 1, "triple");
    missing.as_object_mut().ok_or("params")?.remove("order");
    let bad = c.call(
        12,
        "tools/call",
        json!({"name":"bond.set_order","arguments":missing}),
    )?;
    assert_eq!(bad["result"]["isError"], true);
    assert_eq!(
        bad["result"]["structuredContent"]["error"]["code"],
        "invalid_params"
    );
    assert_eq!(tool(&mut c, 13, "document.inspect", json!({}))?, before);
    let reopened = tool(&mut c, 14, "document.open_cdxml", json!({"path":xml}))?;
    assert_pair(&reopened, "double", 2)?;
    let changed = tool(&mut c, 15, "bond.set_order", change(5, 1, "triple"))?;
    assert_pair(&changed, "triple", 1)?;
    assert_eq!(changed["source"]["format"], "cdxml");
    assert_eq!(changed["source"]["modified"], true);
    tool(
        &mut c,
        16,
        "document.save_mol",
        json!({"path":triple,"version":"v3000"}),
    )?;
    assert_eq!(std::fs::read(&xml)?, xml_before);
    c.finish()?;
    for (path, smiles, formula, h, weight) in [
        (&double, "C=C", "C2H4", 2, 2.0),
        (&triple, "C#C", "C2H2", 1, 3.0),
    ] {
        let value = mol_support::independently_read_mol(path)?;
        assert_eq!(value["smiles"], smiles);
        assert_eq!(value["formula"], formula);
        let atoms = value["atoms"].as_array().ok_or("atoms")?;
        assert_eq!(atoms.len(), 2);
        for a in atoms {
            assert_eq!(a["hydrogens"], h);
            assert_eq!(a["element"], 6);
        }
        assert_eq!(value["bonds"], json!([{"begin":0,"end":1,"order":weight}]));
    }
    Ok(())
}

#[test]
fn stdio_order_refuses_overvalence_and_actual_mol_authority_without_mutating_state() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    let mol = directory.path("source-v3000.mol");
    std::fs::write(&mol, mol_support::V3000_SOURCE)?;
    let mut c = start()?;
    tool(
        &mut c,
        2,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":144,"end_x_pt":120,"end_y_pt":144,"order":"triple"}),
    )?;
    tool(
        &mut c,
        3,
        "bond.extend",
        json!({"expected_revision":1,"page_id":1,"fragment_id":1,"begin_atom_id":2,"end_x_pt":168,"end_y_pt":144,"order":"single","hydrogen_policy":"rectify_terminal_carbon"}),
    )?;
    let before = tool(&mut c, 4, "document.inspect", json!({}))?;
    let bad = c.call(
        5,
        "tools/call",
        json!({"name":"bond.set_order","arguments":change(2,2,"double")}),
    )?;
    assert_eq!(bad["result"]["isError"], true);
    assert_eq!(
        bad["result"]["structuredContent"]["error"]["code"],
        "unsupported_content"
    );
    assert_eq!(tool(&mut c, 6, "document.inspect", json!({}))?, before);
    let opened = tool(
        &mut c,
        7,
        "document.open_mol",
        json!({"path":mol,"discard_unsaved":true}),
    )?;
    let revision = opened["revision"].as_u64().ok_or("revision")?;
    let before = tool(&mut c, 8, "document.inspect", json!({}))?;
    let bad = c.call(
        9,
        "tools/call",
        json!({"name":"bond.set_order","arguments":change(revision,1,"double")}),
    )?;
    assert_eq!(bad["result"]["isError"], true);
    assert_eq!(
        bad["result"]["structuredContent"]["error"]["code"],
        "unsupported_content"
    );
    assert_eq!(tool(&mut c, 10, "document.inspect", json!({}))?, before);
    assert_eq!(std::fs::read(&mol)?, mol_support::V3000_SOURCE);
    c.finish()?;
    Ok(())
}
