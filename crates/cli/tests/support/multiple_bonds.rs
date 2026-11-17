use super::{Client, file_support, mol_support};
use serde_json::{Value, json};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn tool(client: &mut Client, id: u64, name: &str, args: Value) -> TestResult<Value> {
    let result = client.call(id, "tools/call", json!({"name":name,"arguments":args}))?;
    assert_eq!(
        result["result"]["isError"], false,
        "actual stdio operation failed: {result}"
    );
    let wire = result["result"]["structuredContent"].clone();
    let text: Value = serde_json::from_str(
        result["result"]["content"][0]["text"]
            .as_str()
            .ok_or("MCP text")?,
    )?;
    assert_eq!(text, wire);
    Ok(wire)
}

#[test]
fn baseline_stdio_multiple_orders_discovery_selection_history_and_saved_source() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    for (order, weight, selected) in [("double", 2_u16, "triple"), ("triple", 3, "double")] {
        let mut client = Client::start()?;
        client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"multiple-wire-test","version":"1"}}))?;
        client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
        let before = tool(&mut client, 2, "document.inspect", json!({}))?;
        let mode = tool(&mut client, 3, "ui.bond_tool", json!({"order":selected}))?;
        assert_eq!(mode["ui_bond_tool_operation"]["order"], selected);
        assert_eq!(mode["ui_bond_tool_operation"]["changed"], true);
        let epoch = mode["ui_bond_tool_operation"]["epoch"]
            .as_u64()
            .ok_or("tool epoch")?;
        assert_eq!(tool(&mut client, 4, "document.inspect", json!({}))?, before);
        let same = tool(&mut client, 5, "ui.bond_tool", json!({"order":selected}))?;
        assert_eq!(
            same["ui_bond_tool_operation"],
            json!({"order":selected,"epoch":epoch,"changed":false})
        );
        let list = client.call(6, "tools/list", json!({}))?;
        let tools = list["result"]["tools"].as_array().ok_or("tools")?;
        assert_eq!(tools.len(), 42);
        for name in ["bond.create", "bond.extend", "ui.bond_tool"] {
            let entry = tools
                .iter()
                .find(|tool| tool["name"] == name)
                .ok_or("ordered tool")?;
            assert_eq!(entry["inputSchema"]["additionalProperties"], false);
            assert!(entry["inputSchema"]["properties"].get("order").is_some());
        }
        let controls = tool(&mut client, 7, "ui.controls", json!({}))?;
        for name in ["single", "double", "triple"] {
            let id = format!("tool.bond.{name}");
            let control = controls["controls"]
                .as_array()
                .ok_or("controls")?
                .iter()
                .find(|control| control["id"] == id)
                .ok_or("selected tool")?;
            assert_eq!(control["selected"], name == selected);
        }
        let pair = tool(
            &mut client,
            8,
            "bond.create",
            json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72,"order":order}),
        )?;
        assert_eq!(
            pair["document"]["pages"][0]["fragments"][0]["bonds"][0]["order"],
            order
        );
        for atom in pair["document"]["pages"][0]["fragments"][0]["atoms"]
            .as_array()
            .ok_or("atoms")?
        {
            assert_eq!(atom["explicit_hydrogens"], 4 - weight);
        }
        let before = tool(&mut client, 9, "document.inspect", json!({}))?;
        let bad=client.call(10,"tools/call",json!({"name":"bond.extend","arguments":{"expected_revision":1,"page_id":1,"fragment_id":1,"begin_atom_id":2,"end_x_pt":168,"end_y_pt":72,"hydrogen_policy":"rectify_terminal_carbon","order":null}}))?;
        assert_eq!(bad["result"]["isError"], true);
        assert_eq!(
            bad["result"]["structuredContent"]["error"]["code"],
            "invalid_params"
        );
        assert_eq!(
            tool(&mut client, 11, "document.inspect", json!({}))?,
            before
        );
        let chain = tool(
            &mut client,
            12,
            "bond.extend",
            json!({"expected_revision":1,"page_id":1,"fragment_id":1,"begin_atom_id":2,"end_x_pt":168,"end_y_pt":72,"hydrogen_policy":"rectify_terminal_carbon","order":"single"}),
        )?;
        assert_eq!(
            (chain["revision"].clone(), chain["undo_depth"].clone()),
            (json!(2), json!(2))
        );
        let graph = chain["document"].clone();
        assert_eq!(
            graph["pages"][0]["fragments"][0]["bonds"],
            json!([{"id":1,"begin":1,"end":2,"order":order},{"id":2,"begin":2,"end":3,"order":"single"}])
        );
        assert_eq!(
            graph["pages"][0]["fragments"][0]["atoms"]
                .as_array()
                .ok_or("chain atoms")?
                .iter()
                .map(|atom| atom["explicit_hydrogens"].clone())
                .collect::<Vec<_>>(),
            vec![json!(4 - weight), json!(3 - weight), json!(3)]
        );
        let xml = directory.path(&format!("stdio-{order}-λ.cdxml"));
        let mol = directory.path(&format!("stdio-{order}-λ.mol"));
        tool(&mut client, 13, "document.save_cdxml", json!({"path":xml}))?;
        let saved = tool(&mut client, 14, "document.save_mol", json!({"path":mol}))?;
        assert_eq!(saved["file"]["dirty"], false);
        let undo = tool(&mut client, 15, "history.undo", json!({}))?;
        assert_eq!(undo["document"], pair["document"]);
        assert_eq!(undo["file"]["dirty"], true);
        let redo = tool(&mut client, 16, "history.redo", json!({}))?;
        assert_eq!(redo["document"], graph);
        assert_eq!(redo["file"]["dirty"], false);
        let reopened = tool(&mut client, 17, "document.open_cdxml", json!({"path":xml}))?;
        assert_eq!(reopened["document"], graph);
        assert_eq!(reopened["source"]["format"], "cdxml");
        assert_eq!(reopened["source"]["modified"], false);
        let inspected = tool(&mut client, 18, "document.inspect", json!({}))?;
        assert_eq!(inspected["document"], graph);
        assert!(inspected.get("ui_bond_tool_operation").is_none());
        let external = mol_support::independently_read_mol(&mol)?;
        assert_eq!(
            external["smiles"],
            if weight == 2 { "C=CC" } else { "C#CC" }
        );
        assert_eq!(
            external["formula"],
            if weight == 2 { "C3H6" } else { "C3H4" }
        );
        assert_eq!(
            external["atoms"]
                .as_array()
                .ok_or("external atoms")?
                .iter()
                .map(|atom| atom["hydrogens"].clone())
                .collect::<Vec<_>>(),
            vec![json!(4 - weight), json!(3 - weight), json!(3)]
        );
        assert_eq!(
            external["bonds"],
            json!([{"begin":0,"end":1,"order":f64::from(weight)},{"begin":1,"end":2,"order":1.0}])
        );
        client.finish()?;
    }
    Ok(())
}
