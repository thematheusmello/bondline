//! Independent locked-policy wire declarations, after the first engine/adapter draft.
//! Child of chain_creation_wire: use its actual clients, handshakes, deadlines and oracles.
use super::*;

const CHAIN_UI: &str = "ui.chain_tool";
const EMPTY_NATIVE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="612" Height="792"></page></CDXML>"#;

fn tag<'a>(root: &'a Value, variant: &'a Value, field: &str) -> TestResult<&'a str> {
    let property = resolve(root, &variant["properties"][field])?;
    if let Some(value) = property.get("const").and_then(Value::as_str) {
        return Ok(value);
    }
    let values = property["enum"].as_array().ok_or("single literal tag")?;
    assert_eq!(values.len(), 1);
    values
        .first()
        .and_then(Value::as_str)
        .ok_or_else(|| "string tag".into())
}
fn closed_fields(variant: &Value, fields: &[&str]) -> TestResult {
    let expected = fields.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(variant["additionalProperties"], false);
    assert_eq!(
        variant["properties"]
            .as_object()
            .ok_or("properties")?
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(
        variant["required"]
            .as_array()
            .ok_or("required")?
            .iter()
            .map(|v| v.as_str().ok_or("required key"))
            .collect::<Result<BTreeSet<_>, _>>()?,
        expected
    );
    Ok(())
}
fn chain_ui_discovery(entries: &[Value], schema_key: &str) -> TestResult {
    assert_eq!(entries.len(), 47);
    let names = entries
        .iter()
        .map(|v| v["name"].as_str().ok_or("command name"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(names.len(), 47);
    assert_eq!(
        names,
        OLD_NAMES
            .into_iter()
            .chain([
                "chain.create",
                "chain.create_path",
                "document.set_drawing_settings",
                "ui.drawing_settings",
                CHAIN_UI,
                "ring.sprout",
                "ui.preferences",
                "document.set_document_settings",
                "text.set_run",
            ])
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        entries.get(42).ok_or("legacy chain position")?["name"],
        CHAIN_UI
    );
    assert_eq!(
        entries.get(43).ok_or("preserved sprout index")?["name"],
        "ring.sprout"
    );
    assert_eq!(
        entries.get(44).ok_or("preserved preferences entry")?["name"],
        "ui.preferences"
    );
    assert_eq!(
        entries.get(45).ok_or("preserved settings position")?["name"],
        "document.set_document_settings"
    );
    let entry = entries
        .iter()
        .find(|v| v["name"] == CHAIN_UI)
        .ok_or("chain UI command")?;
    if schema_key == "input_schema" {
        assert_eq!(entry["read_only"], false);
    } else {
        assert_eq!(entry["annotations"]["readOnlyHint"], false);
    }
    let root = &entry[schema_key];
    assert_eq!(root["type"], "object");
    assert_eq!(root["unevaluatedProperties"], false);
    let variants = root["oneOf"].as_array().ok_or("four actions")?;
    assert_eq!(variants.len(), 4);
    let mut actions = BTreeSet::new();
    for value in variants {
        let branch = resolve(root, value)?;
        let action = tag(root, branch, "action")?;
        assert!(actions.insert(action));
        match action {
            "select" => {
                closed_fields(branch, &["action", "kind"])?;
                let kind = resolve(root, &branch["properties"]["kind"])?;
                assert_eq!(kind["type"], "string");
                assert_eq!(
                    kind["enum"]
                        .as_array()
                        .ok_or("kinds")?
                        .iter()
                        .map(|v| v.as_str().ok_or("kind"))
                        .collect::<Result<BTreeSet<_>, _>>()?,
                    ["acyclic", "snaking"].into_iter().collect::<BTreeSet<_>>()
                );
            }
            "constraints" => {
                closed_fields(branch, &["action", "fixed_lengths", "fixed_angles"])?;
                assert_eq!(branch["properties"]["fixed_lengths"]["type"], "boolean");
                assert_eq!(branch["properties"]["fixed_angles"]["type"], "boolean");
            }
            "open_add" => {
                closed_fields(
                    branch,
                    &["action", "expected_revision", "page_id", "target"],
                )?;
                assert_eq!(branch["properties"]["expected_revision"]["type"], "integer");
                assert_eq!(
                    branch["properties"]["expected_revision"]["minimum"].as_f64(),
                    Some(0.0)
                );
                assert_eq!(branch["properties"]["page_id"]["type"], "integer");
                assert_eq!(
                    branch["properties"]["page_id"]["minimum"].as_f64(),
                    Some(1.0)
                );
                let target = resolve(root, &branch["properties"]["target"])?;
                assert_eq!(target["type"], "object");
                assert_eq!(target["unevaluatedProperties"], false);
                let alternatives = target["oneOf"].as_array().ok_or("two targets")?;
                assert_eq!(alternatives.len(), 2);
                let mut modes = BTreeSet::new();
                for raw in alternatives {
                    let target_branch = resolve(root, raw)?;
                    let mode = tag(root, target_branch, "mode")?;
                    assert!(modes.insert(mode));
                    match mode {
                        "free" => {
                            closed_fields(target_branch, &["mode", "x_pt", "y_pt"])?;
                            for axis in ["x_pt", "y_pt"] {
                                assert_eq!(target_branch["properties"][axis]["type"], "number");
                                assert_eq!(
                                    target_branch["properties"][axis]["minimum"].as_f64(),
                                    Some(-100000.0)
                                );
                                assert_eq!(
                                    target_branch["properties"][axis]["maximum"].as_f64(),
                                    Some(100000.0)
                                );
                            }
                        }
                        "atom" => {
                            closed_fields(target_branch, &["mode", "fragment_id", "atom_id"])?;
                            for id in ["fragment_id", "atom_id"] {
                                assert_eq!(target_branch["properties"][id]["type"], "integer");
                                assert_eq!(
                                    target_branch["properties"][id]["minimum"].as_f64(),
                                    Some(1.0)
                                );
                            }
                        }
                        _ => return Err("unexpected target mode".into()),
                    }
                }
                assert_eq!(modes, ["free", "atom"].into_iter().collect::<BTreeSet<_>>());
            }
            "cancel_add" => {
                closed_fields(branch, &["action", "expected_revision", "request_id"])?;
                assert_eq!(branch["properties"]["expected_revision"]["type"], "integer");
                assert_eq!(branch["properties"]["request_id"]["type"], "integer");
                assert_eq!(
                    branch["properties"]["request_id"]["minimum"].as_f64(),
                    Some(1.0)
                );
            }
            _ => return Err("unexpected chain action".into()),
        }
    }
    assert_eq!(
        actions,
        ["select", "constraints", "open_add", "cancel_add"]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    Ok(())
}
fn stripped(mut value: Value) -> TestResult<Value> {
    for marker in ["file_operation", "ui_file_operation", "close_operation"] {
        assert!(
            value.get(marker).is_none(),
            "false publication {marker}: {value}"
        );
    }
    let object = value.as_object_mut().ok_or("state")?;
    object.remove("ui_chain_tool_operation");
    object.remove("ui_add_chain_operation");
    object.remove("ui_add_chain");
    Ok(value)
}
fn selected(result: &Value, kind: &str, lengths: bool, angles: bool, epoch: u64, changed: bool) {
    assert_eq!(result["changed"], false);
    assert_eq!(
        result["ui_chain_tool_operation"],
        json!({"kind":kind,
        "fixed_lengths":lengths,"fixed_angles":angles,"epoch":epoch,"changed":changed})
    );
}
fn open_add() -> Value {
    json!({"action":"open_add","expected_revision":1,"page_id":1,
        "target":{"mode":"free","x_pt":144.0,"y_pt":144.0}})
}
fn expected_added_chain() -> Value {
    json!({"id":1,"atoms":[
        atom(1,[144.0,144.0],3,(Some(false),Some(false))),
        atom(2,[144.0+24.0*3.0_f64.sqrt(),120.0],2,(Some(false),Some(false))),
        atom(3,[144.0+48.0*3.0_f64.sqrt(),144.0],3,(Some(false),Some(false)))],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"},
                 {"id":2,"begin":2,"end":3,"order":"single"}]})
}
fn consume_actual_chain(path: &Path) -> TestResult {
    let mut process = Process::new(python())
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../io/tests/drawing_settings_root.py"
            ),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut input = process.stdin.take().ok_or("root consumer stdin")?;
    std::io::Write::write_all(&mut input, &fs::read(path)?)?;
    drop(input);
    let output = process.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)?,
        json!({"root":[["CreationProgram","Bondline 0.1.0"]],"decimal":{}})
    );
    consume(path, &expected_added_chain(), "CCC", "C3H8")
}

#[test]
fn actual_cli_chain_tools_and_constraints_are_headless_and_never_author() -> TestResult {
    let listed = Process::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    chain_ui_discovery(
        serde_json::from_slice::<Value>(&listed.stdout)?
            .as_array()
            .ok_or("commands")?,
        "input_schema",
    )?;
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("headless-chain-原-λ.cdxml");
    fs::write(&input, PROPANE)?;
    let open = request("document.open_cdxml", json!({"path":input}));
    let before = cli(&[open.clone(), request("document.inspect", json!({}))])?;
    graph(&before, &original_propane())?;
    assert_eq!(before["ui_file"]["available"], false);
    let choose = request(CHAIN_UI, json!({"action":"select","kind":"acyclic"}));
    let first = cli(&[open.clone(), choose.clone()])?;
    selected(&first, "acyclic", true, true, 1, true);
    assert_eq!(stripped(first)?, before);
    let duplicate = cli(&[open.clone(), choose.clone(), choose.clone()])?;
    selected(&duplicate, "acyclic", true, true, 1, false);
    assert_eq!(stripped(duplicate)?, before);
    let constraints = request(
        CHAIN_UI,
        json!({"action":"constraints","fixed_lengths":false,"fixed_angles":true}),
    );
    let select_snaking = request(CHAIN_UI, json!({"action":"select","kind":"snaking"}));
    let changed = cli(&[
        open.clone(),
        choose.clone(),
        constraints.clone(),
        select_snaking.clone(),
    ])?;
    selected(&changed, "snaking", false, true, 3, true);
    assert_eq!(stripped(changed)?, before);
    let noop = cli(&[
        open.clone(),
        choose.clone(),
        constraints.clone(),
        select_snaking,
        constraints,
    ])?;
    selected(&noop, "snaking", false, true, 3, false);
    assert_eq!(stripped(noop)?, before);
    for parameters in [
        open_add(),
        json!({"action":"cancel_add","expected_revision":1,"request_id":1}),
    ] {
        let output = Process::new(env!("CARGO_BIN_EXE_bondline-cli"))
            .arg("run")
            .arg("--cmd")
            .arg(open.to_string())
            .arg("--cmd")
            .arg(choose.to_string())
            .arg("--cmd")
            .arg(request(CHAIN_UI, parameters).to_string())
            .output()?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stderr)?["error"]["code"],
            "ui_unavailable"
        );
    }
    assert_eq!(fs::read(&input)?, PROPANE);
    Ok(())
}

#[test]
fn initialized_stdio_chain_schema_and_host_source_refusals_preserve_complete_content() -> TestResult
{
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("stdio-chain-原-化学.cdxml");
    fs::write(&input, PROPANE)?;
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    chain_ui_discovery(
        listed["result"]["tools"].as_array().ok_or("tools")?,
        "inputSchema",
    )?;
    client.call(3, "document.open_cdxml", json!({"path":input}))?;
    let before = client.call(4, "document.inspect", json!({}))?;
    assert_eq!(before["revision"], 1);
    assert_eq!(before["undo_depth"], 0);
    assert_eq!(before["file"]["dirty"], false);
    let result = client.call(5, CHAIN_UI, json!({"action":"select","kind":"acyclic"}))?;
    selected(&result, "acyclic", true, true, 1, true);
    assert_eq!(stripped(result)?, before);
    let result = client.call(
        6,
        CHAIN_UI,
        json!({"action":"constraints","fixed_lengths":false,"fixed_angles":false}),
    )?;
    selected(&result, "acyclic", false, false, 2, true);
    assert_eq!(stripped(result)?, before);
    for (id, parameters, code) in [
        (7, open_add(), "ui_unavailable"),
        (
            9,
            json!({"action":"cancel_add","expected_revision":1,"request_id":1}),
            "ui_unavailable",
        ),
        (
            11,
            json!({"action":"select","kind":"linear"}),
            "invalid_params",
        ),
        (
            13,
            json!({"action":"select","kind":"acyclic","expected_revision":1}),
            "invalid_params",
        ),
        (
            15,
            json!({"action":"constraints","fixed_lengths":true}),
            "invalid_params",
        ),
        (
            17,
            json!({"action":"open_add","expected_revision":1,"page_id":1,
            "target":{"mode":"free","x_pt":144,"y_pt":144,"atom_id":1}}),
            "invalid_params",
        ),
        (
            19,
            json!({"action":"cancel_add","expected_revision":1}),
            "invalid_params",
        ),
        (
            21,
            json!({"action":"open_add","expected_revision":0,"page_id":1,
            "target":{"mode":"free","x_pt":144,"y_pt":144}}),
            "stale_revision",
        ),
    ] {
        client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
            "params":{"name":CHAIN_UI,"arguments":parameters}}))?;
        let rejected = client.replies.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(rejected["id"], id);
        assert_eq!(rejected["result"]["isError"], true);
        assert_eq!(
            rejected["result"]["structuredContent"]["error"]["code"],
            code
        );
        assert_eq!(client.call(id + 1, "document.inspect", json!({}))?, before);
    }
    let opaque_path = dir.path("stdio-chain-opaque-原.cdxml");
    let opaque = std::str::from_utf8(PROPANE)?.replace("<CDXML ", "<CDXML Private=\"retain-all\" ");
    fs::write(&opaque_path, opaque.as_bytes())?;
    client.call(23, "document.open_cdxml", json!({"path":opaque_path}))?;
    let retained = client.call(24, "document.inspect", json!({}))?;
    assert_eq!(retained["revision"], 2);
    client.send(
        json!({"jsonrpc":"2.0","id":25,"method":"tools/call","params":{
        "name":CHAIN_UI,"arguments":{"action":"select","kind":"acyclic"}}}),
    )?;
    let refused = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(refused["id"], 25);
    assert_eq!(refused["result"]["isError"], true);
    assert_eq!(
        refused["result"]["structuredContent"]["error"]["code"],
        "unsupported_content"
    );
    assert_eq!(client.call(26, "document.inspect", json!({}))?, retained);
    let copy = dir.path("stdio-chain-opaque-保持.cdxml");
    client.call(27, "document.save_cdxml", json!({"path":copy}))?;
    assert_eq!(fs::read(&copy)?, opaque.as_bytes());
    assert_eq!(fs::read(&opaque_path)?, opaque.as_bytes());
    assert_eq!(fs::read(&input)?, PROPANE);
    client.finish()
}

#[test]
fn authenticated_add_chain_wakes_without_authoring_then_host_apply_history_and_unicode_save()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("native-chain-empty-原-λ.cdxml");
    let output = dir.path("native-chain-保存-化学.cdxml");
    fs::write(&input, EMPTY_NATIVE)?;
    let mut session = Engine::default();
    session.set_ui_file_host(true);
    session.dispatch("document.open_cdxml", json!({"path":input}))?;
    let before = serde_json::to_value(session.state())?;
    assert_eq!(before["revision"], 1);
    assert_eq!(before["undo_depth"], 0);
    assert_eq!(before["file"]["dirty"], false);
    assert!(before["document"].get("drawing_settings").is_none());
    assert_eq!(before["document"]["pages"][0]["fragments"], json!([]));
    let whole = session.content().clone();
    let owner = session.capture_stamp();
    let model = session.stored_document()?.model_snapshot();
    let engine: SharedEngine = Arc::new(Mutex::new(session));
    let (notify, noticed) = std::sync::mpsc::channel();
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        let _ = notify.send(());
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    // Actual credentials stay only on this local socket; never log/store them.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    assert!(noticed.try_recv().is_err());
    let chosen = socket(
        &mut client,
        request(CHAIN_UI, json!({"action":"select","kind":"acyclic"})),
    )?;
    assert!(chosen.get("error").is_none(), "{chosen}");
    selected(&chosen["result"], "acyclic", true, true, 1, true);
    assert_eq!(stripped(chosen["result"].clone())?, before);
    noticed.recv_timeout(Duration::from_secs(10))?;
    let duplicate = socket(
        &mut client,
        request(CHAIN_UI, json!({"action":"select","kind":"acyclic"})),
    )?;
    selected(&duplicate["result"], "acyclic", true, true, 1, false);
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        before
    );
    assert!(noticed.try_recv().is_err());
    let opened = socket(&mut client, request(CHAIN_UI, open_add()))?;
    assert!(opened.get("error").is_none(), "{opened}");
    assert_eq!(opened["result"]["ui_add_chain_operation"], 1);
    assert_eq!(
        opened["result"]["ui_add_chain"],
        json!({"id":1,"input_revision":1,
        "page_id":1,"target":{"mode":"free","x_pt":144.0,"y_pt":144.0},"count":6,
        "bond_length_pt":48.0,"angle_degrees":120.0,"current":true})
    );
    assert_eq!(opened["result"]["changed"], false);
    assert_eq!(stripped(opened["result"].clone())?, before);
    // Presentation itself must wake a host despite no authored revision.
    noticed.recv_timeout(Duration::from_secs(10))?;
    let active = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    assert!(active.get("ui_add_chain_operation").is_none());
    for (parameters, code) in [
        (open_add(), "ui_busy"),
        (
            json!({"action":"cancel_add","expected_revision":1,"request_id":2}),
            "ui_stale_request",
        ),
    ] {
        let rejected = socket(&mut client, request(CHAIN_UI, parameters))?;
        assert_eq!(rejected["error"]["code"], code);
        assert_eq!(
            socket(&mut client, request("document.inspect", json!({})))?["result"],
            active
        );
        assert!(noticed.try_recv().is_err());
    }
    let canceled = socket(
        &mut client,
        request(
            CHAIN_UI,
            json!({"action":"cancel_add","expected_revision":1,"request_id":1}),
        ),
    )?;
    assert!(canceled.get("error").is_none(), "{canceled}");
    assert_eq!(canceled["result"]["ui_add_chain_operation"], 1);
    assert_eq!(canceled["result"]["changed"], false);
    assert!(canceled["result"].get("ui_add_chain").is_none());
    assert_eq!(stripped(canceled["result"].clone())?, before);
    noticed.recv_timeout(Duration::from_secs(10))?;
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        before
    );
    assert!(noticed.try_recv().is_err());
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &whole);
        assert!(guard.capture_stamp_matches(&owner));
        assert!(Arc::ptr_eq(
            &model,
            &guard.stored_document()?.model_snapshot()
        ));
    }
    let next = socket(&mut client, request(CHAIN_UI, open_add()))?;
    assert_eq!(next["result"]["ui_add_chain"]["id"], 2);
    assert_eq!(next["result"]["ui_add_chain"]["count"], 6);
    noticed.recv_timeout(Duration::from_secs(10))?;
    {
        let mut guard = engine.lock().map_err(|_| "engine")?;
        let unchanged = serde_json::to_value(guard.state())?;
        assert_eq!(
            guard
                .apply_ui_add_chain(2, 1, 0)
                .err()
                .ok_or("invalid count accepted")?
                .info()
                .code,
            "invalid_params"
        );
        assert_eq!(serde_json::to_value(guard.state())?, unchanged);
        let applied = serde_json::to_value(guard.apply_ui_add_chain(2, 3, 0)?)?;
        assert_eq!(applied["changed"], true);
        assert_eq!(applied["revision"], 2);
        assert_eq!(applied["undo_depth"], 1);
        assert_eq!(applied["redo_depth"], 0);
        assert_eq!(applied["ui_add_chain_operation"], 2);
        assert!(applied.get("ui_add_chain").is_none());
        assert_eq!(applied["file"]["dirty"], true);
        assert_eq!(applied["source"]["modified"], true);
        assert!(applied["document"].get("drawing_settings").is_none());
        graph(&applied, &expected_added_chain())?;
        let _ = stripped(applied)?;
        assert!(!guard.capture_stamp_matches(&owner));
    }
    assert!(!output.exists());
    assert_eq!(fs::read(&input)?, EMPTY_NATIVE);
    let undo = socket(&mut client, request("history.undo", json!({})))?;
    assert_eq!(undo["result"]["revision"], 3);
    assert_eq!(undo["result"]["document"], before["document"]);
    assert_eq!(undo["result"]["undo_depth"], 0);
    assert_eq!(undo["result"]["redo_depth"], 1);
    noticed.recv_timeout(Duration::from_secs(10))?;
    let redo = socket(&mut client, request("history.redo", json!({})))?;
    assert_eq!(redo["result"]["revision"], 4);
    assert_eq!(redo["result"]["undo_depth"], 1);
    graph(&redo["result"], &expected_added_chain())?;
    noticed.recv_timeout(Duration::from_secs(10))?;
    let saved = socket(
        &mut client,
        request("document.save_cdxml", json!({"path":output})),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    assert_eq!(saved["result"]["revision"], 4);
    assert_eq!(saved["result"]["file"]["dirty"], false);
    assert_eq!(
        saved["result"]["file"]["path"],
        json!(fs::canonicalize(&output)?)
    );
    assert!(saved["result"].get("ui_file_operation").is_none());
    noticed.recv_timeout(Duration::from_secs(10))?;
    consume_actual_chain(&output)?;
    assert_eq!(fs::read(&input)?, EMPTY_NATIVE);
    let reopened = socket(
        &mut client,
        request("document.open_cdxml", json!({"path":output})),
    )?;
    assert!(reopened.get("error").is_none(), "{reopened}");
    assert_eq!(reopened["result"]["revision"], 5);
    assert_eq!(reopened["result"]["undo_depth"], 0);
    assert_eq!(reopened["result"]["file"]["dirty"], false);
    assert_eq!(reopened["result"]["source"]["modified"], false);
    assert!(reopened["result"].get("ui_add_chain").is_none());
    graph(&reopened["result"], &expected_added_chain())?;
    noticed.recv_timeout(Duration::from_secs(10))?;
    let retained_tool = socket(
        &mut client,
        request(CHAIN_UI, json!({"action":"select","kind":"acyclic"})),
    )?;
    selected(&retained_tool["result"], "acyclic", true, true, 1, false);
    let _ = socket(&mut client, request("document.inspect", json!({})))?;
    assert!(noticed.try_recv().is_err());
    server.shutdown()?;
    Ok(())
}
