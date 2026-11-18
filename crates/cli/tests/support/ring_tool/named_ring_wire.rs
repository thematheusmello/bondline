//! Independent post-draft named native-mode acceptance over the existing real adapters.
//! Reuses the existing process/socket clients, framing, deadlines and publication barrier.
use super::{
    Client, NAME, TestResult, call_socket, cli, initialize, operation, publication, rejected,
    schema, socket, support,
};
use bondline_commands::Engine;
use bondline_mcp::{CommandExecutor, SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    net::TcpStream,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

fn named_controls(value: &Value, selected: &str) -> TestResult {
    let entries = value["controls"].as_array().ok_or("controls")?;
    assert_eq!(entries.len(), 39);
    for (kind, label) in [
        ("benzene", "Benzene"),
        ("cyclopentadiene", "Cyclopentadiene"),
    ] {
        let id = format!("tool.ring.{kind}");
        let row = entries
            .iter()
            .find(|row| row["id"] == id)
            .ok_or("named control")?;
        assert_eq!(row["label"], label);
        assert_eq!(row["menu"], "Tools");
        assert_eq!(row["enabled"], true);
        assert_eq!(row["shortcut"], Value::Null);
        assert_eq!(row["selected"], kind == selected);
        assert_eq!(
            row["command"],
            json!({"command":NAME,"params":{"kind":kind}})
        );
    }
    let selected_ids = entries
        .iter()
        .filter(|row| row["menu"] == "Tools" && row["selected"] == true)
        .map(|row| row["id"].as_str().ok_or("selected control ID"))
        .collect::<Result<Vec<_>, _>>()?;
    let expected = format!("tool.ring.{selected}");
    assert_eq!(selected_ids, vec![expected.as_str()]);
    assert!(value.get("ui_ring_tool_operation").is_none());
    Ok(())
}

fn exact_unchanged_result(value: &Value, before: &Value) -> TestResult {
    let mut ordinary = value.clone();
    assert!(
        ordinary
            .as_object_mut()
            .ok_or("state")?
            .remove("ui_ring_tool_operation")
            .is_some()
    );
    assert_eq!(
        &ordinary, before,
        "tool selection must leave every ordinary session field exact"
    );
    Ok(())
}

#[test]
fn cli_and_initialized_stdio_preserve_the_exact_receipt_and_distinguish_named_from_same_size()
-> TestResult {
    let seed = json!({"command":"bond.create","params":{
        "page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}});
    let before = cli(&[
        seed.clone(),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    assert_eq!(before["revision"], 1);
    assert_eq!(before["undo_depth"], 1);
    assert_eq!(before["file"]["dirty"], true);
    let mut requests = vec![
        seed,
        json!({"command":NAME,"params":{"kind":"cyclopentadiene"}}),
        json!({"command":NAME,"params":{"size":5}}),
        json!({"command":NAME,"params":{"kind":"benzene"}}),
        json!({"command":NAME,"params":{"kind":"benzene"}}),
    ];
    let selected = cli(&requests)?;
    operation(&selected, 6, 3, false);
    exact_unchanged_result(&selected, &before)?;
    requests.push(json!({"command":"ui.controls","params":{}}));
    named_controls(&cli(&requests)?, "benzene")?;

    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 48);
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == NAME)
        .ok_or("named-tool discovery")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    // The reused schema helper retains every old size assertion and validates the named alternative.
    schema(&tool["inputSchema"])?;
    client.call(
        3,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
    )?;
    let before = client.call(4, "document.inspect", json!({}))?;
    assert_eq!(before["ui_file"]["available"], false);
    let cp = client.call(5, NAME, json!({"kind":"cyclopentadiene"}))?;
    operation(&cp, 5, 1, true);
    exact_unchanged_result(&cp, &before)?;
    let noop = client.call(6, NAME, json!({"kind":"cyclopentadiene"}))?;
    operation(&noop, 5, 1, false);
    exact_unchanged_result(&noop, &before)?;
    let single = client.call(7, NAME, json!({"size":5}))?;
    operation(&single, 5, 2, true);
    exact_unchanged_result(&single, &before)?;
    let cp = client.call(8, NAME, json!({"kind":"cyclopentadiene"}))?;
    operation(&cp, 5, 3, true);
    exact_unchanged_result(&cp, &before)?;
    named_controls(
        &client.call(9, "ui.controls", json!({}))?,
        "cyclopentadiene",
    )?;
    for (i, params) in [
        json!({"kind":"benzene","size":6}),
        json!({"kind":{"benzene":null}}),
        json!({"kind":"benzene","phase":"single_first"}),
    ]
    .into_iter()
    .enumerate()
    {
        rejected(&mut client, 20 + i as u64 * 2, params, "invalid_params")?;
        assert_eq!(
            client.call(21 + i as u64 * 2, "document.inspect", json!({}))?,
            before
        );
    }
    named_controls(
        &client.call(30, "ui.controls", json!({}))?,
        "cyclopentadiene",
    )?;
    client.finish()
}

#[test]
fn authenticated_named_modes_publish_only_real_changes_and_revalidate_complete_source_before_noop()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("named-mode-原.cdxml");
    let readonly = directory.path("named-mode-opaque-保存.smi");
    const NATIVE: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/bondline-v0.cdxml"
    ));
    const RAW: &[u8] = b"CC first\r\nO\tPRIVATE_NAMED\0\xff\n";
    fs::write(&input, NATIVE)?;
    fs::write(&readonly, RAW)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
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
    // The fresh credential is sent only to its own listener, never logged or retained.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    call_socket(&mut client, "document.open_cdxml", json!({"path":input}))?;
    publication(&noticed, &ack, &retired)?;
    let before = call_socket(&mut client, "document.inspect", json!({}))?;
    assert_eq!(before["revision"], 1);
    assert_eq!(before["undo_depth"], 0);
    assert_eq!(before["file"]["dirty"], false);
    let (content, owner, model) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (
            guard.content().clone(),
            guard.capture_stamp(),
            guard.stored_document()?.model_snapshot(),
        )
    };
    let bz = call_socket(&mut client, NAME, json!({"kind":"benzene"}))?;
    operation(&bz, 6, 1, true);
    exact_unchanged_result(&bz, &before)?;
    publication(&noticed, &ack, &retired)?;
    let first = engine.lock().map_err(|_| "engine")?.ring_context(1)?;
    let noop = call_socket(&mut client, NAME, json!({"kind":"benzene"}))?;
    operation(&noop, 6, 1, false);
    exact_unchanged_result(&noop, &before)?;
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert!(first.is_current(&guard));
    }
    let cp = call_socket(&mut client, NAME, json!({"kind":"cyclopentadiene"}))?;
    operation(&cp, 5, 2, true);
    exact_unchanged_result(&cp, &before)?;
    publication(&noticed, &ack, &retired)?;
    let bz = call_socket(&mut client, NAME, json!({"kind":"benzene"}))?;
    operation(&bz, 6, 3, true);
    exact_unchanged_result(&bz, &before)?;
    publication(&noticed, &ack, &retired)?;
    named_controls(
        &call_socket(&mut client, "ui.controls", json!({}))?,
        "benzene",
    )?;
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        before
    );
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert!(
            !first.is_current(&guard),
            "kind A-B-A must not revive the first weak context"
        );
        assert_eq!(guard.content(), &content);
        assert!(guard.capture_stamp_matches(&owner));
        assert!(Arc::ptr_eq(
            &model,
            &guard.stored_document()?.model_snapshot()
        ));
    }
    // Reply ordering is the barrier: no timer-based absence assumption or callback polling.
    assert!(noticed.try_recv().is_err());
    assert!(retired.try_recv().is_err());
    call_socket(
        &mut client,
        "document.open_smiles",
        json!({"path":readonly}),
    )?;
    publication(&noticed, &ack, &retired)?;
    let raw = call_socket(&mut client, "document.inspect", json!({}))?;
    assert_eq!(raw["document"], Value::Null);
    let (raw_content, raw_owner) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.content().clone(), guard.capture_stamp())
    };
    // Remembered Benzene may not bypass whole-source validation merely because it looks like a no-op.
    let failed = socket(
        &mut client,
        json!({"command":NAME,"params":{"kind":"benzene"}}),
    )?;
    assert_eq!(failed["error"]["code"], "unsupported_content");
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        raw
    );
    let controls = call_socket(&mut client, "ui.controls", json!({}))?;
    assert_eq!(controls["ui_file"]["available"], false);
    let rows = controls["controls"]
        .as_array()
        .ok_or("raw source controls")?;
    assert_eq!(rows.len(), 12);
    let settings = rows
        .iter()
        .find(|row| row["id"] == "file.document_settings")
        .ok_or("Document Settings remains visible but unavailable")?;
    assert_eq!(settings["enabled"], false);
    assert_eq!(settings["disabled_reason"], "ui_unavailable");
    assert!(!rows.iter().any(|row| {
        row["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("tool.ring."))
    }));
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &raw_content);
        assert!(guard.capture_stamp_matches(&raw_owner));
        assert!(!first.is_current(&guard));
    }
    assert_eq!(fs::read(&input)?, NATIVE);
    assert_eq!(fs::read(&readonly)?, RAW);
    drop(client);
    server.shutdown()?;
    assert!(noticed.try_recv().is_err());
    assert!(retired.try_recv().is_err());
    Ok(())
}
