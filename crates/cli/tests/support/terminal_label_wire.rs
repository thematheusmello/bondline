//! Existing real adapters; no new command, channel policy, client or consumer.
use super::*;
use std::process::Command as Process;
#[path = "../../../../xtask/oracles/terminal_label_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse original fixed source/properties and unchanged chemical consumer"
)]
mod literal;

fn label(revision: u64) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,
    "atom_id":1,"element":6,"explicit_hydrogens":3})
}
fn restored() -> literal::Flags {
    let mut f = literal::ALL_FALSE;
    f[0] = (Some(true), Some(true));
    f
}
fn check(value: &Value, flags: &literal::Flags) {
    assert_eq!(
        value["document"]["pages"][0]["size"],
        json!({"width_pt":420.0,"height_pt":280.0})
    );
    assert_eq!(
        value["document"]["pages"][0]["fragments"],
        json!([literal::fragment(flags)])
    );
    assert_eq!(value["source"]["format"], "cdxml");
    assert!(value.get("collection").is_none());
}

#[test]
fn cli_restores_both_terminal_flags_once_and_saves_exact_unicode_properties_without_changing_registry()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("terminal-cli-original-炭-λ.cdxml");
    let output = directory.path("terminal-cli-restored-化学-λ.cdxml");
    fs::write(&input, literal::HIDDEN)?;
    let listed = Process::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(listed.status.success());
    let entries: Value = serde_json::from_slice(&listed.stdout)?;
    let entries = entries.as_array().ok_or("commands")?;
    assert_eq!(entries.len(), 42);
    let spec = &entries
        .iter()
        .find(|v| v["name"] == "atom.set_label")
        .ok_or("label command")?["input_schema"];
    let fields = [
        "expected_revision",
        "page_id",
        "fragment_id",
        "atom_id",
        "element",
        "explicit_hydrogens",
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(spec["additionalProperties"], false);
    assert_eq!(
        spec["properties"]
            .as_object()
            .ok_or("properties")?
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        fields
    );
    assert_eq!(
        spec["required"]
            .as_array()
            .ok_or("required")?
            .iter()
            .map(|v| v.as_str().ok_or("field"))
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?,
        fields
    );
    let opened = json!({"command":"document.open_cdxml","params":{"path":input}});
    let changed = cli(&[
        opened.clone(),
        json!({"command":"atom.set_label","params":label(1)}),
    ])?;
    check(&changed, &restored());
    assert_eq!(changed["changed"], true);
    assert_eq!(changed["revision"], 2);
    assert_eq!(changed["undo_depth"], 1);
    assert_eq!(changed["redo_depth"], 0);
    assert_eq!(changed["file"]["dirty"], true);
    assert_eq!(changed["source"]["byte_length"], literal::HIDDEN.len());
    assert_eq!(changed["source"]["modified"], true);
    assert!(changed.get("file_operation").is_none() && changed.get("ui_file_operation").is_none());
    let result = cli(&[
        opened,
        json!({"command":"atom.set_label","params":label(1)}),
        json!({"command":"atom.set_label","params":label(2)}),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":output}}),
        json!({"command":"document.open_cdxml","params":{"path":output}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    check(&result, &restored());
    assert_eq!(result["revision"], 5);
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(result["file"]["dirty"], false);
    assert_eq!(result["source"]["modified"], false);
    assert_eq!(result["file"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(fs::read(&input)?, literal::HIDDEN);
    literal::consume(&input, &literal::ALL_FALSE)?;
    literal::consume(&output, &restored())?;
    Ok(())
}

fn call_socket(
    client: &mut BufReader<TcpStream>,
    name: &str,
    params: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let reply = socket(client, json!({"command":name,"params":params}))?;
    assert!(reply.get("error").is_none(), "{reply}");
    Ok(reply["result"].clone())
}
#[test]
fn initialized_stdio_and_authenticated_socket_share_restoration_noop_history_source_and_changed_only_repaint()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("terminal-transports-original-λ.cdxml");
    let output = directory.path("terminal-stdio-restored-化学.cdxml");
    fs::write(&input, literal::HIDDEN)?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25",
        "capabilities":{},"clientInfo":{"name":"terminal-label-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let opened = client.call(2, "document.open_cdxml", json!({"path":input}))?;
    check(&opened, &literal::ALL_FALSE);
    let changed = client.call(3, "atom.set_label", label(1))?;
    check(&changed, &restored());
    assert_eq!(changed["revision"], 2);
    assert_eq!(changed["undo_depth"], 1);
    let before = client.call(4, "document.inspect", json!({}))?;
    assert_eq!(client.call(5, "atom.set_label", label(2))?, before);
    client.send(json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"atom.set_label","arguments":label(1)}}))?;
    let stale = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(stale["id"], 6);
    assert_eq!(
        stale["result"]["structuredContent"]["error"]["code"],
        "stale_revision"
    );
    assert_eq!(client.call(7, "document.inspect", json!({}))?, before);
    let undone = client.call(8, "history.undo", json!({}))?;
    check(&undone, &literal::ALL_FALSE);
    assert_eq!(undone["file"]["dirty"], false);
    let redone = client.call(9, "history.redo", json!({}))?;
    check(&redone, &restored());
    assert_eq!(redone["undo_depth"], 1);
    client.call(10, "document.save_cdxml", json!({"path":output}))?;
    let reopened = client.call(11, "document.open_cdxml", json!({"path":output}))?;
    check(&reopened, &restored());
    assert_eq!(reopened["undo_depth"], 0);
    literal::consume(&output, &restored())?;
    client.finish()?;

    let output = directory.path("terminal-socket-restored-炭-λ.cdxml");
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        observed.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    call_socket(&mut client, "document.open_cdxml", json!({"path":input}))?;
    let count = changes.load(Ordering::SeqCst);
    let changed = call_socket(&mut client, "atom.set_label", label(1))?;
    check(&changed, &restored());
    assert_eq!(changed["changed"], true);
    assert_eq!(changes.load(Ordering::SeqCst), count + 1);
    let before = call_socket(&mut client, "document.inspect", json!({}))?;
    let count = changes.load(Ordering::SeqCst);
    assert_eq!(
        call_socket(&mut client, "atom.set_label", label(2))?,
        before
    );
    let stale = socket(
        &mut client,
        json!({"command":"atom.set_label","params":label(1)}),
    )?;
    assert_eq!(stale["error"]["code"], "stale_revision");
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), count);
    call_socket(&mut client, "history.undo", json!({}))?;
    let redone = call_socket(&mut client, "history.redo", json!({}))?;
    check(&redone, &restored());
    call_socket(&mut client, "document.save_cdxml", json!({"path":output}))?;
    let reopened = call_socket(&mut client, "document.open_cdxml", json!({"path":output}))?;
    check(&reopened, &restored());
    assert_eq!(reopened["file"]["dirty"], false);
    literal::consume(&output, &restored())?;
    assert_eq!(fs::read(&input)?, literal::HIDDEN);
    server.shutdown()?;
    Ok(())
}
