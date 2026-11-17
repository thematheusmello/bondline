//! Independent adapter acceptance, declared after the first engine implementation.
//! Child of chain_creation_wire: its actual clients, deadlines and consumers stay unchanged.
use super::*;

const UI_SETTINGS: &str = "ui.drawing_settings";

fn settings_schema(entries: &[Value], key: &str) -> TestResult {
    assert_eq!(entries.len(), 46);
    let names = entries
        .iter()
        .map(|v| v["name"].as_str().ok_or("name"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(names.len(), 46);
    assert_eq!(
        names,
        OLD_NAMES
            .into_iter()
            .chain([
                "chain.create",
                "chain.create_path",
                "document.set_drawing_settings",
                UI_SETTINGS,
                "ui.chain_tool",
                "ring.sprout",
                "ui.preferences",
                "document.set_document_settings",
            ])
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        entries.get(41).ok_or("old42 final entry")?["name"],
        UI_SETTINGS
    );
    assert_eq!(
        entries.get(42).ok_or("legacy chain position")?["name"],
        "ui.chain_tool"
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
        entries.last().ok_or("last entry")?["name"],
        "document.set_document_settings"
    );
    let entry = entries
        .iter()
        .find(|v| v["name"] == UI_SETTINGS)
        .ok_or("settings command")?;
    if key == "input_schema" {
        assert_eq!(entry["read_only"], false);
    } else {
        assert_eq!(entry["annotations"]["readOnlyHint"], false);
    }
    let schema = &entry[key];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["unevaluatedProperties"], false);
    let variants = schema["oneOf"].as_array().ok_or("two closed actions")?;
    assert_eq!(variants.len(), 2);
    let mut actions = BTreeSet::new();
    for raw in variants {
        let variant = resolve(schema, raw)?;
        assert_eq!(variant["additionalProperties"], false);
        let action = &variant["properties"]["action"];
        let tag = action
            .get("const")
            .and_then(Value::as_str)
            .or_else(|| action.get("enum")?.as_array()?.first()?.as_str())
            .ok_or("action tag")?;
        assert!(actions.insert(tag));
        let fields = match tag {
            "open" => vec!["action", "expected_revision"],
            "cancel" => vec!["action", "expected_revision", "request_id"],
            _ => return Err("unexpected action".into()),
        }
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(
            variant["properties"]
                .as_object()
                .ok_or("properties")?
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            fields
        );
        assert_eq!(
            variant["required"]
                .as_array()
                .ok_or("required")?
                .iter()
                .map(|v| v.as_str().ok_or("required name"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            fields
        );
        assert_eq!(
            variant["properties"]["expected_revision"]["type"],
            "integer"
        );
        assert_eq!(
            variant["properties"]["expected_revision"]["minimum"].as_f64(),
            Some(0.0)
        );
        if tag == "cancel" {
            assert_eq!(variant["properties"]["request_id"]["type"], "integer");
            assert_eq!(
                variant["properties"]["request_id"]["minimum"].as_f64(),
                Some(1.0)
            );
        }
    }
    assert_eq!(
        actions,
        ["open", "cancel"].into_iter().collect::<BTreeSet<_>>()
    );
    Ok(())
}
fn settings_native(length: Option<&str>, angle: Option<&str>) -> TestResult<Vec<u8>> {
    let mut attributes = "CreationProgram=\"Bondline 0.1.0\"".to_owned();
    if let Some(value) = length {
        attributes.push_str(&format!(" BondLength=\"{value}\""));
    }
    if let Some(value) = angle {
        attributes.push_str(&format!(" ChainAngle=\"{value}\""));
    }
    Ok(std::str::from_utf8(PROPANE)?
        .replace("CreationProgram=\"Bondline 0.1.0\"", &attributes)
        .into_bytes())
}
fn no_save_markers(value: &Value) {
    for name in ["file_operation", "ui_file_operation", "close_operation"] {
        assert!(value.get(name).is_none(), "unexpected {name}: {value}");
    }
}
fn without_settings_presentation(mut value: Value) -> TestResult<Value> {
    no_save_markers(&value);
    let object = value.as_object_mut().ok_or("state object")?;
    object.remove("ui_drawing_settings");
    object.remove("ui_drawing_settings_operation");
    Ok(value)
}
fn settings_root_then_chemistry(path: &Path) -> TestResult {
    let mut child = Process::new(python())
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
    child
        .stdin
        .take()
        .ok_or("root stdin")?
        .write_all(&fs::read(path)?)?;
    let read = child.wait_with_output()?;
    assert!(
        read.status.success(),
        "{}",
        String::from_utf8_lossy(&read.stderr)
    );
    let value: Value = serde_json::from_slice(&read.stdout)?;
    assert_eq!(
        value,
        json!({"root":[["CreationProgram","Bondline 0.1.0"],
        ["BondLength","64.25"]],"decimal":{"BondLength":"64.25"}})
    );
    consume(path, &original_propane(), "CCC", "C3H8")
}

#[test]
fn authenticated_settings_open_cancel_wake_without_authoring_then_host_apply_saves_exact_root()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("native-preferences-原-λ.cdxml");
    let output = dir.path("native-preferences-保存-化学.cdxml");
    let source = settings_native(None, Some("137.5"))?;
    fs::write(&input, &source)?;
    let mut session = Engine::default();
    session.set_ui_file_host(true);
    session.dispatch("document.open_cdxml", json!({"path":input}))?;
    let before = serde_json::to_value(session.state())?;
    graph(&before, &original_propane())?;
    assert_eq!(
        before["document"]["drawing_settings"],
        json!({"chain_angle_degrees":137.5})
    );
    assert_eq!(before["revision"], 1);
    assert_eq!(before["undo_depth"], 0);
    assert_eq!(before["file"]["dirty"], false);
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
    // Session credentials are sent only to this local socket, never logged or saved.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    assert!(noticed.try_recv().is_err());
    let opened = socket(
        &mut client,
        request(UI_SETTINGS, json!({"action":"open","expected_revision":1})),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    assert_eq!(opened["result"]["ui_drawing_settings_operation"], 1);
    assert_eq!(
        opened["result"]["ui_drawing_settings"],
        json!({"id":1,"input_revision":1,
        "bond_length_pt":null,"chain_angle_degrees":137.5,"line_width_pt":null,"bond_spacing_percent":null,"current":true})
    );
    assert_eq!(opened["result"]["changed"], false);
    assert_eq!(
        without_settings_presentation(opened["result"].clone())?,
        before
    );
    // Decisive host wakeup despite changed=false; the operation marker is sufficient.
    noticed.recv_timeout(Duration::from_secs(10))?;
    let active = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    assert!(active.get("ui_drawing_settings_operation").is_none());
    assert_eq!(without_settings_presentation(active.clone())?, before);
    for (parameters, code) in [
        (json!({"action":"open","expected_revision":1}), "ui_busy"),
        (
            json!({"action":"cancel","expected_revision":1,"request_id":2}),
            "ui_stale_request",
        ),
    ] {
        let rejected = socket(&mut client, request(UI_SETTINGS, parameters))?;
        assert_eq!(rejected["error"]["code"], code);
        assert_eq!(
            socket(&mut client, request("document.inspect", json!({})))?["result"],
            active
        );
        assert!(noticed.try_recv().is_err());
    }
    let save_busy = socket(
        &mut client,
        request("ui.file", json!({"action":"save","expected_revision":1})),
    )?;
    assert_eq!(save_busy["error"]["code"], "ui_busy");
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        active
    );
    assert!(noticed.try_recv().is_err());
    assert!(!output.exists());
    let canceled = socket(
        &mut client,
        request(
            UI_SETTINGS,
            json!({"action":"cancel","expected_revision":1,"request_id":1}),
        ),
    )?;
    assert!(canceled.get("error").is_none(), "{canceled}");
    assert_eq!(canceled["result"]["ui_drawing_settings_operation"], 1);
    assert!(canceled["result"].get("ui_drawing_settings").is_none());
    assert_eq!(
        without_settings_presentation(canceled["result"].clone())?,
        before
    );
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
    let again = socket(
        &mut client,
        request(UI_SETTINGS, json!({"action":"open","expected_revision":1})),
    )?;
    assert_eq!(again["result"]["ui_drawing_settings"]["id"], 2);
    noticed.recv_timeout(Duration::from_secs(10))?;
    {
        let mut guard = engine.lock().map_err(|_| "engine")?;
        let noop = serde_json::to_value(guard.apply_ui_drawing_settings(2, None, Some(137.5))?)?;
        assert_eq!(noop["changed"], false);
        assert_eq!(noop["ui_drawing_settings_operation"], 2);
        assert!(noop.get("ui_drawing_settings").is_none());
        assert_eq!(without_settings_presentation(noop)?, before);
        assert_eq!(guard.content(), &whole);
        assert!(guard.capture_stamp_matches(&owner));
        assert!(Arc::ptr_eq(
            &model,
            &guard.stored_document()?.model_snapshot()
        ));
    }
    let third = socket(
        &mut client,
        request(UI_SETTINGS, json!({"action":"open","expected_revision":1})),
    )?;
    assert_eq!(third["result"]["ui_drawing_settings"]["id"], 3);
    noticed.recv_timeout(Duration::from_secs(10))?;
    {
        let mut guard = engine.lock().map_err(|_| "engine")?;
        let active = serde_json::to_value(guard.state())?;
        let invalid = guard
            .apply_ui_drawing_settings(3, Some(0.0), None)
            .err()
            .ok_or("invalid length accepted")?;
        assert_eq!(invalid.info().code, "invalid_params");
        assert_eq!(serde_json::to_value(guard.state())?, active);
        assert!(guard.capture_stamp_matches(&owner));
        let applied =
            serde_json::to_value(guard.apply_ui_drawing_settings(3, Some(64.25), None)?)?;
        assert_eq!(applied["changed"], true);
        assert_eq!(applied["revision"], 2);
        assert_eq!(applied["undo_depth"], 1);
        assert_eq!(applied["redo_depth"], 0);
        assert_eq!(applied["ui_drawing_settings_operation"], 3);
        assert!(applied.get("ui_drawing_settings").is_none());
        no_save_markers(&applied);
        assert_eq!(
            applied["document"]["drawing_settings"],
            json!({"bond_length_pt":64.25})
        );
        assert_eq!(applied["document"]["pages"], before["document"]["pages"]);
        assert_eq!(applied["file"]["dirty"], true);
        assert_eq!(applied["source"]["modified"], true);
        assert!(!guard.capture_stamp_matches(&owner));
    }
    // Host Apply is deliberately under the native mutex, not a forged transport result.
    assert_eq!(fs::read(&input)?, source);
    assert!(!output.exists());
    let undo = socket(&mut client, request("history.undo", json!({})))?;
    assert_eq!(undo["result"]["revision"], 3);
    assert_eq!(undo["result"]["document"], before["document"]);
    noticed.recv_timeout(Duration::from_secs(10))?;
    let redo = socket(&mut client, request("history.redo", json!({})))?;
    assert_eq!(redo["result"]["revision"], 4);
    assert_eq!(
        redo["result"]["document"]["drawing_settings"],
        json!({"bond_length_pt":64.25})
    );
    assert_eq!(redo["result"]["undo_depth"], 1);
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
    assert_eq!(fs::read(&output)?, settings_native(Some("64.25"), None)?);
    settings_root_then_chemistry(&output)?;
    noticed.recv_timeout(Duration::from_secs(10))?;
    let reopened = socket(
        &mut client,
        request("document.open_cdxml", json!({"path":output})),
    )?;
    assert!(reopened.get("error").is_none(), "{reopened}");
    assert_eq!(reopened["result"]["revision"], 5);
    assert_eq!(reopened["result"]["undo_depth"], 0);
    assert_eq!(reopened["result"]["file"]["dirty"], false);
    assert_eq!(reopened["result"]["source"]["modified"], false);
    assert_eq!(
        reopened["result"]["document"]["drawing_settings"],
        json!({"bond_length_pt":64.25})
    );
    graph(&reopened["result"], &original_propane())?;
    noticed.recv_timeout(Duration::from_secs(10))?;
    assert_eq!(fs::read(&input)?, source);
    server.shutdown()?;
    Ok(())
}

#[test]
fn actual_cli_discovers_two_closed_settings_actions_but_has_no_native_host() -> TestResult {
    let listed = Process::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let entries: Value = serde_json::from_slice(&listed.stdout)?;
    settings_schema(entries.as_array().ok_or("commands")?, "input_schema")?;
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("headless-preferences-原.cdxml");
    let source = settings_native(Some("48.5"), Some("137.5"))?;
    fs::write(&input, &source)?;
    let open = request("document.open_cdxml", json!({"path":input}));
    let before = cli(&[open.clone(), request("document.inspect", json!({}))])?;
    assert_eq!(before["ui_file"]["available"], false);
    assert!(before.get("ui_drawing_settings").is_none());
    graph(&before, &original_propane())?;
    let output = Process::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("run")
        .arg("--cmd")
        .arg(open.to_string())
        .arg("--cmd")
        .arg(request(UI_SETTINGS, json!({"action":"open","expected_revision":1})).to_string())
        .output()?;
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr)?;
    assert_eq!(error["error"]["code"], "ui_unavailable");
    assert_eq!(fs::read(&input)?, source);
    Ok(())
}

#[test]
fn initialized_stdio_settings_schema_and_refusals_preserve_the_complete_headless_session()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("stdio-preferences-样式.cdxml");
    let source = settings_native(None, Some("137.5"))?;
    fs::write(&input, &source)?;
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    settings_schema(
        listed["result"]["tools"].as_array().ok_or("tools")?,
        "inputSchema",
    )?;
    client.call(3, "document.open_cdxml", json!({"path":input}))?;
    let before = client.call(4, "document.inspect", json!({}))?;
    assert_eq!(before["ui_file"]["available"], false);
    assert_eq!(before["revision"], 1);
    assert_eq!(before["undo_depth"], 0);
    assert_eq!(before["file"]["dirty"], false);
    graph(&before, &original_propane())?;
    assert!(before.get("ui_drawing_settings").is_none());
    for (id, parameters, code) in [
        (
            5,
            json!({"action":"open","expected_revision":1}),
            "ui_unavailable",
        ),
        (7, json!({"action":"open"}), "invalid_params"),
        (
            9,
            json!({"action":"open","expected_revision":1,"request_id":1}),
            "invalid_params",
        ),
        (
            11,
            json!({"action":"cancel","expected_revision":1}),
            "invalid_params",
        ),
        (
            13,
            json!({"action":"open","expected_revision":1,"bond_length_pt":48}),
            "invalid_params",
        ),
        (
            15,
            json!({"action":"open","expected_revision":0}),
            "stale_revision",
        ),
    ] {
        client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
            "params":{"name":UI_SETTINGS,"arguments":parameters}}))?;
        let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(reply["id"], id);
        assert_eq!(reply["result"]["isError"], true);
        assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
        // Stdio remains open after malformed parameters; loopback's disconnect rule is unchanged.
        assert_eq!(client.call(id + 1, "document.inspect", json!({}))?, before);
    }
    assert_eq!(fs::read(&input)?, source);
    client.finish()
}
