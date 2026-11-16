//! Transport discovery/presentation checks; these do not claim a native dialog.
use super::*;

#[test]
fn stdio_discovers_converter_intent_but_cannot_show_a_headless_dialog() -> TestResult {
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"native-conversion-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(
        tools.len(),
        32,
        "presentation adds no second converter command"
    );
    let file = tools
        .iter()
        .find(|tool| tool["name"] == "ui.file")
        .ok_or("file tool")?;
    assert_eq!(file["annotations"]["readOnlyHint"], false);
    assert_eq!(file["inputSchema"]["additionalProperties"], false);
    assert!(
        file["inputSchema"]["$defs"]["UiFileAction"]["enum"]
            .as_array()
            .ok_or("file action enum")?
            .contains(&json!("convert_smiles"))
    );
    let converter = tools
        .iter()
        .find(|tool| tool["name"] == "file.convert_smiles_to_cdxml")
        .ok_or("converter")?;
    assert_eq!(converter["annotations"]["readOnlyHint"], false);
    assert_eq!(converter["inputSchema"]["additionalProperties"], false);
    assert_eq!(
        converter["inputSchema"]["required"]
            .as_array()
            .ok_or("eight required params")?
            .len(),
        8
    );
    let before = client.call(3, "document.inspect", json!({}))?;
    for (id, arguments, code) in [
        (
            4,
            json!({"expected_revision":0,"action":"convert_smiles"}),
            "ui_unavailable",
        ),
        (
            5,
            json!({"expected_revision":0,"action":"convert_smiles","input_path":"not-read.smi"}),
            "invalid_params",
        ),
        (
            6,
            json!({"expected_revision":0,"action":"convert_smiles","close_epoch":0}),
            "invalid_params",
        ),
    ] {
        client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"ui.file","arguments":arguments}}))?;
        let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(reply["id"], id);
        assert_eq!(reply["result"]["isError"], true);
        assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    }
    assert_eq!(client.call(7, "document.inspect", json!({}))?, before);
    client.finish()
}

#[test]
fn authenticated_converter_presentation_and_cancel_preserve_the_complete_source() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("conversion-origin-λ.sdf");
    fs::write(&input, SOURCE)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    engine
        .lock()
        .map_err(|_| "engine poisoned")?
        .set_ui_file_host(true);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), || {})?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let opened = socket(
        &mut client,
        json!({"command":"document.open_sdf","params":{"path":input}}),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    let controls = socket(&mut client, json!({"command":"ui.controls","params":{}}))?;
    let controls = controls["result"]["controls"]
        .as_array()
        .ok_or("controls")?;
    assert_eq!(controls.len(), 14);
    let control = controls
        .iter()
        .find(|control| control["id"] == "file.convert_smiles")
        .ok_or("converter control")?;
    assert_eq!(control["enabled"], true);
    assert_eq!(control["menu"], "File");
    assert_eq!(control["label"], "Convert SMILES file…");
    assert_eq!(control["shortcut"], Value::Null);
    assert_eq!(
        control["command"],
        json!({"command":"ui.file","params":{"expected_revision":1,"action":"convert_smiles"}})
    );
    let requested = socket(&mut client, control["command"].clone())?;
    assert!(requested.get("error").is_none(), "{requested}");
    let state = &requested["result"];
    let request = &state["ui_file"]["request"];
    assert_eq!(request["action"], "convert_smiles");
    assert_eq!(request["phase"], "requested");
    assert_eq!(request["input_revision"], 1);
    let id = request["id"].as_u64().ok_or("presentation id")?;
    for key in [
        "revision",
        "document",
        "collection",
        "source",
        "file",
        "undo_depth",
        "redo_depth",
        "close",
        "close_epoch",
    ] {
        assert_eq!(state[key], before[key], "{key}");
    }
    assert_eq!(
        state["ui_file_operation"], id,
        "presentation returns only its issued epoch"
    );
    for key in ["conversion", "file_operation"] {
        assert!(state.get(key).is_none(), "presentation cannot create {key}");
    }
    let cancelled = socket(
        &mut client,
        json!({"command":"ui.file","params":{"expected_revision":1,"action":"cancel","request_id":id}}),
    )?;
    assert!(cancelled.get("error").is_none(), "{cancelled}");
    assert_eq!(cancelled["result"]["ui_file"]["request"], Value::Null);
    assert_eq!(cancelled["result"]["ui_file"]["epoch"], id + 1);
    for key in [
        "revision",
        "document",
        "collection",
        "source",
        "file",
        "undo_depth",
        "redo_depth",
        "close",
        "close_epoch",
    ] {
        assert_eq!(cancelled["result"][key], before[key], "{key}");
    }
    assert_eq!(fs::read(input)?, SOURCE);
    drop(client);
    server.shutdown()?;
    Ok(())
}
