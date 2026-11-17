//! Independent actual CLI transient ring-tool acceptance, using only JSON dispatch.
use serde_json::{Value, json};
use std::process::Command;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn cli_ring_tool_returns_an_explicit_transient_receipt_without_authoring() -> TestResult {
    let run = |request: Value| -> TestResult<Value> {
        let out = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
            .arg("run")
            .arg("--cmd")
            .arg(request.to_string())
            .output()?;
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(serde_json::from_slice(&out.stdout)?)
    };
    let before = run(json!({"command":"document.inspect","params":{}}))?;
    assert!(before.get("ui_ring_tool_operation").is_none());
    // Baseline compiles against existing APIs and reaches a real CLI UnknownCommand.
    let selected = run(json!({"command":"ui.ring_tool","params":{"size":6}}))?;
    assert_eq!(
        selected["ui_ring_tool_operation"],
        json!({"size":6,"epoch":1,"changed":true})
    );
    assert_eq!(selected["changed"], false);
    for key in [
        "document",
        "source",
        "file",
        "revision",
        "undo_depth",
        "redo_depth",
        "close",
        "close_epoch",
        "ui_file",
    ] {
        assert_eq!(selected[key], before[key], "ring tool authored {key}");
    }
    assert!(selected.get("file_operation").is_none());
    assert!(selected.get("close_operation").is_none());
    Ok(())
}

// Additive declarations preserve the exact frozen 1,400-byte initial CLI case.
use super::{Client, cli, socket, support};
use bondline_commands::Engine;
use bondline_mcp::{CommandExecutor, SharedEngine, control::LoopbackServer};
use std::{
    collections::BTreeSet,
    fs,
    io::BufReader,
    net::TcpStream,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

const NAME: &str = "ui.ring_tool";
const OLD_NAMES: [&str; 37] = [
    "document.inspect",
    "document.new",
    "document.close",
    "ui.file",
    "ui.controls",
    "ui.view",
    "ui.bond_tool",
    "document.open_cdxml",
    "document.save_cdxml",
    "document.open_cdx",
    "document.save_cdx",
    "document.open_mol",
    "document.save_mol",
    "document.open_sdf",
    "document.save_sdf",
    "document.open_smiles",
    "document.import_smiles_records",
    "document.save_smiles",
    "file.convert_smiles_to_cdxml",
    "file.export_smiles_fragment",
    "sdf.record.inspect",
    "smiles.record.inspect",
    "structure.paste_smiles",
    "sdf.field.set_value",
    "sdf.record.remove",
    "page.set_size",
    "bond.create",
    "bond.extend",
    "bond.set_order",
    "history.undo",
    "history.redo",
    "render.capture",
    "atom.set_label",
    "ui.atom_label",
    "ui.selection",
    "selection.translate",
    "ring.create",
];
fn ring(size: u8) -> Value {
    json!({"command":NAME,"params":{"size":size}})
}
fn same_authored(actual: &Value, before: &Value) {
    for key in [
        "document",
        "source",
        "file",
        "revision",
        "undo_depth",
        "redo_depth",
        "close",
        "close_epoch",
        "ui_file",
    ] {
        assert_eq!(actual[key], before[key], "tool changed {key}");
    }
}
fn operation(value: &Value, size: u8, epoch: u64, changed: bool) {
    assert_eq!(value["changed"], false);
    assert_eq!(
        value["ui_ring_tool_operation"],
        json!({"size":size,"epoch":epoch,"changed":changed})
    );
    assert!(value.get("file_operation").is_none());
    assert!(value.get("close_operation").is_none());
}
fn names(entries: &[Value]) -> TestResult {
    let actual = entries
        .iter()
        .map(|v| v["name"].as_str().ok_or("command name"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(entries.len(), 43);
    assert_eq!(
        actual,
        OLD_NAMES
            .into_iter()
            .chain([
                NAME,
                "chain.create",
                "chain.create_path",
                "document.set_drawing_settings",
                "ui.drawing_settings",
                "ui.chain_tool",
            ])
            .collect::<BTreeSet<_>>()
    );
    Ok(())
}
fn schema(value: &Value) -> TestResult {
    let legacy = legacy_ui_ring_tool_schema(value)?;
    let value = &legacy;
    assert_eq!(value["type"], "object");
    assert_eq!(value["additionalProperties"], false);
    assert_eq!(value["required"], json!(["size"]));
    let fields = value["properties"].as_object().ok_or("properties")?;
    assert_eq!(
        fields.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["size"]
    );
    assert_eq!(fields["size"]["type"], "integer");
    assert_eq!(fields["size"]["minimum"].as_f64(), Some(3.0));
    assert_eq!(fields["size"]["maximum"].as_f64(), Some(8.0));
    Ok(())
}
fn controls(value: &Value, ring: Option<u8>, bond: Option<&str>, selection: bool) -> TestResult {
    let all = value["controls"].as_array().ok_or("controls")?;
    assert_eq!(all.len(), 38);
    for (size, label) in [
        (3, "Cyclopropane"),
        (4, "Cyclobutane"),
        (5, "Cyclopentane"),
        (6, "Cyclohexane"),
        (7, "Cycloheptane"),
        (8, "Cyclooctane"),
    ] {
        let id = format!("tool.ring.{size}");
        let entry = all.iter().find(|v| v["id"] == id).ok_or("ring control")?;
        assert_eq!(entry["label"], label);
        assert_eq!(entry["menu"], "Tools");
        assert_eq!(entry["shortcut"], Value::Null);
        assert_eq!(entry["selected"], ring == Some(size));
        assert_eq!(entry["enabled"], true);
        assert_eq!(
            entry["command"],
            json!({"command":NAME,"params":{"size":size}})
        );
    }
    for order in ["single", "double", "triple"] {
        let id = format!("tool.bond.{order}");
        assert_eq!(
            all.iter().find(|v| v["id"] == id).ok_or("bond control")?["selected"],
            bond == Some(order)
        );
    }
    assert_eq!(
        all.iter()
            .find(|v| v["id"] == "tool.selection")
            .ok_or("selection control")?["selected"],
        selection
    );
    assert!(
        value.get("ui_ring_tool_operation").is_none(),
        "normal controls must not replay a receipt"
    );
    Ok(())
}

#[test]
fn cli_discovery_and_mode_transitions_preserve_native_authoring_and_remembered_bond_order()
-> TestResult {
    let out = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(out.status.success());
    let entries: Value = serde_json::from_slice(&out.stdout)?;
    let entries = entries.as_array().ok_or("commands")?;
    names(entries)?;
    let entry = entries
        .iter()
        .find(|v| v["name"] == NAME)
        .ok_or("ring tool command")?;
    assert_eq!(entry["read_only"], false);
    schema(&entry["input_schema"])?;
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/bondline-v0.cdxml"
    );
    let raw = fs::read(path)?;
    let seed = vec![
        json!({"command":"document.open_cdxml","params":{"path":path}}),
        json!({"command":"bond.set_order","params":{"expected_revision":1,"page_id":1,"fragment_id":1,
            "bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}}),
    ];
    let before = cli(&seed)?;
    assert_eq!(before["revision"], 2);
    assert_eq!(before["undo_depth"], 1);
    assert_eq!(before["file"]["dirty"], true);
    let mut requests = seed;
    requests.push(json!({"command":"ui.selection","params":{"action":"set","expected_revision":2,"page_id":1,"atom_ids":[2,1]}}));
    requests.push(ring(6));
    let selected = cli(&requests)?;
    operation(&selected, 6, 2, true);
    same_authored(&selected, &before);
    assert_eq!(selected["ui_selection"]["active"], false);
    assert_eq!(selected["ui_selection"]["atom_ids"], json!([]));
    assert_eq!(selected["ui_selection"]["page_id"], Value::Null);
    requests.push(ring(6));
    let noop = cli(&requests)?;
    operation(&noop, 6, 2, false);
    same_authored(&noop, &before);
    assert_eq!(noop["ui_selection"], selected["ui_selection"]);
    requests.push(json!({"command":"ui.bond_tool","params":{"order":"single"}}));
    let bond = cli(&requests)?;
    assert_eq!(
        bond["ui_bond_tool_operation"],
        json!({"order":"single","epoch":3,"changed":true})
    );
    assert!(bond.get("ui_ring_tool_operation").is_none());
    same_authored(&bond, &before);
    let mut shown = requests.clone();
    shown.push(json!({"command":"ui.controls","params":{}}));
    controls(&cli(&shown)?, None, Some("single"), false)?;
    requests.extend([
        ring(8),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.new","params":{"expected_revision":4,"discard_unsaved":true}}),
        json!({"command":"document.open_cdxml","params":{"path":path}}),
    ]);
    let mut nooped = requests.clone();
    nooped.push(ring(8));
    operation(&cli(&nooped)?, 8, 4, false);
    requests.push(json!({"command":"ui.controls","params":{}}));
    let current = cli(&requests)?;
    controls(&current, Some(8), None, false)?;
    assert_eq!(current["revision"], 6);
    assert_eq!(current["undo_depth"], 0);
    assert_eq!(current["redo_depth"], 0);
    assert_eq!(current["file"]["dirty"], false);
    assert_eq!(current["source"]["modified"], false);
    assert_eq!(fs::read(path)?, raw);
    Ok(())
}

fn initialize(client: &mut Client) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25",
        "capabilities":{},"clientInfo":{"name":"ring-tool-acceptance","version":"1"}}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], 1);
    assert!(reply.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
}
fn rejected(client: &mut Client, id: u64, params: Value, code: &str) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":NAME,"arguments":params}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], id);
    assert_eq!(reply["result"]["isError"], true);
    assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    Ok(())
}
#[test]
fn initialized_stdio_validates_closed_sizes_modes_noops_close_and_source_before_receipt()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let path = directory.path("ring-tool-readonly.sdf");
    const SDF: &[u8] = include_bytes!("../../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
    fs::write(&path, SDF)?;
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    names(tools)?;
    let tool = tools
        .iter()
        .find(|v| v["name"] == NAME)
        .ok_or("ring tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    schema(&tool["inputSchema"])?;
    client.call(
        3,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
    )?;
    let before = client.call(4, "document.inspect", json!({}))?;
    assert_eq!(before["ui_file"]["available"], false);
    for (i, params) in [
        json!({}),
        json!({"size":2}),
        json!({"size":9}),
        json!({"size":6,"expected_revision":1}),
    ]
    .into_iter()
    .enumerate()
    {
        rejected(&mut client, 10 + i as u64 * 2, params, "invalid_params")?;
        assert_eq!(
            client.call(11 + i as u64 * 2, "document.inspect", json!({}))?,
            before
        );
    }
    let selected = client.call(20, NAME, json!({"size":6}))?;
    operation(&selected, 6, 1, true);
    same_authored(&selected, &before);
    operation(&client.call(21, NAME, json!({"size":6}))?, 6, 1, false);
    controls(
        &client.call(22, "ui.controls", json!({}))?,
        Some(6),
        None,
        false,
    )?;
    client.call(
        23,
        "ui.selection",
        json!({"action":"activate","expected_revision":1}),
    )?;
    controls(
        &client.call(24, "ui.controls", json!({}))?,
        None,
        None,
        true,
    )?;
    operation(&client.call(25, NAME, json!({"size":6}))?, 6, 3, true);
    client.call(
        26,
        "ui.selection",
        json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[2,1]}),
    )?;
    let changed = client.call(27, NAME, json!({"size":5}))?;
    operation(&changed, 5, 5, true);
    same_authored(&changed, &before);
    assert_eq!(changed["ui_selection"]["active"], false);
    assert_eq!(changed["ui_selection"]["atom_ids"], json!([]));
    client.call(
        28,
        "ui.selection",
        json!({"action":"clear","expected_revision":1}),
    )?;
    operation(&client.call(29, NAME, json!({"size":5}))?, 5, 5, false);
    controls(
        &client.call(30, "ui.controls", json!({}))?,
        Some(5),
        None,
        false,
    )?;
    let bond = client.call(31, "ui.bond_tool", json!({"order":"single"}))?;
    assert_eq!(
        bond["ui_bond_tool_operation"],
        json!({"order":"single","epoch":6,"changed":true})
    );
    operation(&client.call(32, NAME, json!({"size":8}))?, 8, 7, true);
    client.call(33, "history.undo", json!({}))?;
    operation(&client.call(34, NAME, json!({"size":8}))?, 8, 7, false);
    client.call(35, "history.redo", json!({}))?;
    operation(&client.call(36, NAME, json!({"size":8}))?, 8, 7, false);
    let state = client.call(37, "document.inspect", json!({}))?;
    assert!(state.get("ui_ring_tool_operation").is_none());
    let closed = client.call(
        38,
        "document.close",
        json!({"expected_revision":3,"action":"request"}),
    )?;
    assert_eq!(closed["close"], "needs_decision");
    let stable = client.call(39, "document.inspect", json!({}))?;
    rejected(&mut client, 40, json!({"size":8}), "ui_busy")?;
    assert_eq!(client.call(41, "document.inspect", json!({}))?, stable);
    client.call(
        42,
        "document.close",
        json!({"expected_revision":3,"action":"cancel"}),
    )?;
    operation(&client.call(43, NAME, json!({"size":8}))?, 8, 7, false);
    client.call(
        44,
        "document.open_sdf",
        json!({"path":path,"discard_unsaved":true}),
    )?;
    let readonly = client.call(45, "document.inspect", json!({}))?;
    assert_eq!(readonly["document"], Value::Null);
    rejected(&mut client, 46, json!({"size":8}), "unsupported_content")?;
    assert_eq!(client.call(47, "document.inspect", json!({}))?, readonly);
    let source_controls = client.call(48, "ui.controls", json!({}))?;
    let all = source_controls["controls"]
        .as_array()
        .ok_or("SDF controls")?;
    assert_eq!(all.len(), 15);
    let settings = all
        .iter()
        .find(|row| row["id"] == "file.document_settings")
        .ok_or("Document Settings remains visible but unavailable")?;
    assert_eq!(settings["enabled"], false);
    assert_eq!(settings["disabled_reason"], "ui_unavailable");
    assert!(!all.iter().any(|v| {
        v["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("tool.ring."))
    }));
    assert_eq!(fs::read(&path)?, SDF);
    client.finish()
}

fn call_socket(client: &mut BufReader<TcpStream>, name: &str, params: Value) -> TestResult<Value> {
    let reply = socket(client, json!({"command":name,"params":params}))?;
    assert!(reply.get("error").is_none(), "{reply}");
    Ok(reply["result"].clone())
}
fn publication(
    noticed: &mpsc::Receiver<bool>,
    ack: &mpsc::Sender<()>,
    retired: &mpsc::Receiver<bool>,
) -> TestResult {
    assert!(
        !noticed.recv_timeout(Duration::from_secs(10))?,
        "reply activity must be released before repaint"
    );
    ack.send(())?;
    assert!(
        retired.recv_timeout(Duration::from_secs(10))?,
        "callback must receive the explicit post-response ACK"
    );
    Ok(())
}
#[test]
fn authenticated_ring_mode_and_size_repaint_only_after_reply_publication_and_real_change()
-> TestResult {
    let mut session = Engine::default();
    session.set_ui_file_host(true);
    session.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
    )?;
    session.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[2,1]}),
    )?;
    let before = serde_json::to_value(session.state())?;
    let content = session.content().clone();
    let owner = session.capture_stamp();
    let engine: SharedEngine = Arc::new(Mutex::new(session));
    let executor = CommandExecutor::new(Arc::clone(&engine));
    let observer = executor.clone();
    let (notify, noticed) = mpsc::channel();
    let (ack, acks) = mpsc::channel();
    let acks = Mutex::new(acks);
    let (retire, retired) = mpsc::channel();
    let server = LoopbackServer::bind_with_executor(0, executor, move || {
        let _ = notify.send(observer.has_active_work());
        let acknowledged = acks
            .lock()
            .ok()
            .is_some_and(|r| r.recv_timeout(Duration::from_secs(10)).is_ok());
        let _ = retire.send(acknowledged);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    // The session credential is sent only to its own listener and never printed/stored.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let selected = call_socket(&mut client, NAME, json!({"size":6}))?;
    operation(&selected, 6, 2, true);
    same_authored(&selected, &before);
    assert_eq!(selected["ui_selection"]["active"], false);
    assert_eq!(selected["ui_selection"]["atom_ids"], json!([]));
    // Complete wire reply is readable while this callback waits for the explicit ACK.
    publication(&noticed, &ack, &retired)?;
    let changed = call_socket(&mut client, NAME, json!({"size":5}))?;
    operation(&changed, 5, 3, true);
    same_authored(&changed, &before);
    publication(&noticed, &ack, &retired)?;
    let stable = call_socket(&mut client, "document.inspect", json!({}))?;
    assert!(stable.get("ui_ring_tool_operation").is_none());
    let noop = call_socket(&mut client, NAME, json!({"size":5}))?;
    operation(&noop, 5, 3, false);
    same_authored(&noop, &stable);
    let failed = socket(&mut client, json!({"command":NAME,"params":{"size":9}}))?;
    assert_eq!(failed["error"]["code"], "invalid_params");
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        stable
    );
    let bond = call_socket(&mut client, "ui.bond_tool", json!({"order":"single"}))?;
    assert_eq!(
        bond["ui_bond_tool_operation"],
        json!({"order":"single","epoch":4,"changed":true})
    );
    publication(&noticed, &ack, &retired)?;
    operation(
        &call_socket(&mut client, NAME, json!({"size":5}))?,
        5,
        5,
        true,
    );
    publication(&noticed, &ack, &retired)?;
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &content);
        assert!(guard.capture_stamp_matches(&owner));
    }
    let request = {
        let mut guard = engine.lock().map_err(|_| "engine")?;
        guard
            .dispatch("ui.file", json!({"expected_revision":1,"action":"open"}))?
            .ui_file_operation
            .ok_or("file request")?
    };
    let busy = call_socket(&mut client, "document.inspect", json!({}))?;
    let failed = socket(&mut client, json!({"command":NAME,"params":{"size":5}}))?;
    assert_eq!(failed["error"]["code"], "ui_busy");
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        busy
    );
    engine.lock().map_err(|_| "engine")?.dispatch(
        "ui.file",
        json!({"expected_revision":1,"action":"cancel","request_id":request}),
    )?;
    operation(
        &call_socket(&mut client, NAME, json!({"size":5}))?,
        5,
        5,
        false,
    );
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("ring-ineligible-保存.smi");
    const RAW: &[u8] = b"CC first\r\nO\tPRIVATE_RING\0\xff\n";
    fs::write(&input, RAW)?;
    {
        let mut guard = engine.lock().map_err(|_| "engine")?;
        guard.dispatch(
            "document.open_smiles",
            json!({"path":input,"discard_unsaved":true}),
        )?;
        let revision = guard.state().revision;
        guard.dispatch("document.import_smiles_records",json!({"expected_revision":revision,"selection":{"mode":"all"},
            "bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,"page_width_pt":612,"page_height_pt":792}))?;
        assert!(guard.stored_document()?.has_smiles_projection());
    }
    let (source, stamp) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.content().clone(), guard.capture_stamp())
    };
    let before = call_socket(&mut client, "document.inspect", json!({}))?;
    let failed = socket(&mut client, json!({"command":NAME,"params":{"size":5}}))?;
    assert_eq!(failed["error"]["code"], "unsupported_content");
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        before
    );
    let controls = call_socket(&mut client, "ui.controls", json!({}))?;
    let all = controls["controls"].as_array().ok_or("retained controls")?;
    assert_eq!(all.len(), 21);
    let settings = all
        .iter()
        .find(|row| row["id"] == "file.document_settings")
        .ok_or("Document Settings remains visible but unavailable")?;
    assert_eq!(settings["enabled"], false);
    assert_eq!(settings["disabled_reason"], "unsupported_content");
    assert!(!all.iter().any(|v| {
        v["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("tool.ring."))
    }));
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &source);
        assert!(guard.capture_stamp_matches(&stamp));
    }
    assert_eq!(fs::read(&input)?, RAW);
    drop(client);
    server.shutdown()?;
    assert!(noticed.try_recv().is_err());
    assert!(retired.try_recv().is_err());
    Ok(())
}

// Test-only positive schema extension: returns only the preserved legacy object.
// All original size assertions following this call remain unchanged and meaningful.
fn legacy_ui_ring_tool_schema(
    root: &serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use serde_json::{Value, json};
    assert_eq!(root["type"], "object");
    let branches = root
        .get("anyOf")
        .or_else(|| root.get("oneOf"))
        .and_then(Value::as_array)
        .ok_or("three closed ring-tool alternatives")?;
    assert_eq!(branches.len(), 3);
    let mut legacy = None;
    let mut named = false;
    let mut chair = false;
    for branch in branches {
        let branch = if let Some(reference) = branch.get("$ref").and_then(Value::as_str) {
            root.pointer(reference.strip_prefix('#').ok_or("nonlocal schema")?)
                .ok_or("missing branch")?
        } else {
            branch
        };
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let fields = branch["properties"].as_object().ok_or("fields")?;
        let is_chair = if fields.contains_key("size") {
            false
        } else {
            let kind = fields.get("kind").ok_or("kind selector")?;
            let kind = if let Some(reference) = kind.get("$ref").and_then(Value::as_str) {
                root.pointer(reference.strip_prefix('#').ok_or("nonlocal chair kind")?)
                    .ok_or("missing chair kind")?
            } else {
                kind
            };
            assert_eq!(kind["type"], "string");
            kind["enum"] == json!(["cyclohexane_chair_1", "cyclohexane_chair_2"])
        };
        if fields.contains_key("size") {
            assert!(legacy.is_none());
            assert_eq!(
                fields.keys().map(String::as_str).collect::<Vec<_>>(),
                vec!["size"]
            );
            legacy = Some(branch.clone());
        } else if is_chair {
            assert!(!chair);
            chair = true;
            assert_eq!(
                fields.keys().map(String::as_str).collect::<Vec<_>>(),
                vec!["kind"]
            );
            assert_eq!(branch["required"], json!(["kind"]));
        } else {
            assert!(!named);
            named = true;
            assert_eq!(
                fields.keys().map(String::as_str).collect::<Vec<_>>(),
                vec!["kind"]
            );
            assert_eq!(branch["required"], json!(["kind"]));
            let kind = &fields["kind"];
            let kind = if let Some(reference) = kind.get("$ref").and_then(Value::as_str) {
                root.pointer(reference.strip_prefix('#').ok_or("nonlocal kind")?)
                    .ok_or("missing kind")?
            } else {
                kind
            };
            assert_eq!(kind["type"], "string");
            assert_eq!(kind["enum"], json!(["benzene", "cyclopentadiene"]));
        }
    }
    assert!(named);
    assert!(chair);
    legacy.ok_or_else(|| "missing original size object".into())
}

#[path = "ring_tool/named_ring_wire.rs"]
mod named_ring_wire;
