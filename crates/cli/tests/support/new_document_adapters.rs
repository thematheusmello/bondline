//! Additional transport evidence for the same source-qualified New command.
use super::*;

const SMILES: &[u8] = b"CC first\r\n\n \xffignored\n[13CH3][OH] \0\xffPRIVATE\nO last";

fn fresh(value: &Value, revision: u64) -> TestResult {
    assert_eq!(value["changed"], true);
    assert_eq!(value["revision"], revision);
    assert_eq!(value["undo_depth"], 0);
    assert_eq!(value["redo_depth"], 0);
    assert_eq!(
        value["file"],
        json!({"format":"cdxml","path":null,"dirty":false})
    );
    assert_eq!(value["close"], "open");
    for absent in [
        "source",
        "collection",
        "smiles_file",
        "file_operation",
        "ui_file_operation",
    ] {
        assert!(value.get(absent).is_none(), "{absent}: {value}");
    }
    let pages = value["document"]["pages"]
        .as_array()
        .ok_or("Drawing pages")?;
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0]["id"], 1);
    assert_eq!(pages[0]["fragments"], json!([]));
    let width = pages[0]["size"]["width_pt"].as_f64().ok_or("width")?;
    let height = pages[0]["size"]["height_pt"].as_f64().ok_or("height")?;
    assert!((width - 210.0 * 72.0 / 25.4).abs() < 1e-10);
    assert!((height - 297.0 * 72.0 / 25.4).abs() < 1e-10);
    Ok(())
}

fn bond() -> Value {
    json!({"page_id":1,"begin_x_pt":72.0,"begin_y_pt":72.0,"end_x_pt":120.0,"end_y_pt":72.0})
}

fn independently_consume_drawing(path: &std::path::Path) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned interpreter")?;
    let mut child = Command::new(python)
        .args([
            "-B",
            &format!(
                "{}/../commands/tests/smiles_conversion_oracle.py",
                env!("CARGO_MANIFEST_DIR")
            ),
            "2",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(&fs::read(path)?)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn cli_new_leaves_complete_sdf_untouched_and_saves_a_new_independently_readable_drawing()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("new-from-source.sdf");
    let output = dir.path("new-drawing-λ.cdxml");
    fs::write(&input, SOURCE)?;
    let opened = json!({"command":"document.open_sdf","params":{"path":input}});
    let new = json!({"command":"document.new","params":{"expected_revision":1}});
    fresh(&cli(&[opened.clone(), new.clone()])?, 2)?;
    let result = cli(&[
        opened,
        new,
        json!({"command":"bond.create","params":bond()}),
        json!({"command":"document.save_cdxml","params":{"path":output}}),
        json!({"command":"document.open_cdxml","params":{"path":output}}),
    ])?;
    assert_eq!(result["revision"], 4);
    assert_eq!(result["file"]["format"], "cdxml");
    assert_eq!(result["file"]["dirty"], false);
    assert_eq!(result["undo_depth"], 0);
    assert_eq!(
        result["document"]["pages"][0]["fragments"][0]["atoms"]
            .as_array()
            .ok_or("atoms")?
            .len(),
        2
    );
    assert_eq!(fs::read(input)?, SOURCE);
    independently_consume_drawing(&output)?;
    Ok(())
}

#[test]
fn stdio_new_schema_and_smiles_to_drawing_workflow_share_the_registry() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("new-from-records.smiles");
    let output = dir.path("new-stdio-λ.cdxml");
    fs::write(&input, SMILES)?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"new-document-test","version":"1"}}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], 1);
    assert!(reply.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 40);
    let new = tools
        .iter()
        .find(|item| item["name"] == "document.new")
        .ok_or("New tool")?;
    assert_eq!(new["annotations"]["readOnlyHint"], false);
    assert_eq!(new["inputSchema"]["additionalProperties"], false);
    assert_eq!(new["inputSchema"]["required"], json!(["expected_revision"]));
    assert_eq!(
        new["inputSchema"]["properties"]["discard_unsaved"]["default"],
        false
    );
    client.call(3, "document.open_smiles", json!({"path":input}))?;
    let before = client.call(4, "document.inspect", json!({}))?;
    client.send(json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"document.new","arguments":{"expected_revision":0,"discard_unsaved":true}}}))?;
    let stale = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(stale["id"], 5);
    assert_eq!(stale["result"]["isError"], true);
    assert_eq!(
        stale["result"]["structuredContent"]["error"]["code"],
        "stale_revision"
    );
    assert_eq!(client.call(6, "document.inspect", json!({}))?, before);
    fresh(
        &client.call(7, "document.new", json!({"expected_revision":1}))?,
        2,
    )?;
    client.call(8, "bond.create", bond())?;
    client.call(9, "document.save_cdxml", json!({"path":output}))?;
    assert_eq!(fs::read(input)?, SMILES);
    independently_consume_drawing(&output)?;
    client.finish()?;
    Ok(())
}

#[test]
fn authenticated_new_notifies_once_and_dirty_refusal_retains_exact_state() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("new-loopback.sdf");
    fs::write(&input, SOURCE)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let notify = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        notify.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    assert!(
        socket(
            &mut client,
            json!({"command":"document.open_sdf","params":{"path":input}})
        )?
        .get("error")
        .is_none()
    );
    let created = socket(
        &mut client,
        json!({"command":"document.new","params":{"expected_revision":1}}),
    )?;
    fresh(&created["result"], 2)?;
    assert!(
        socket(
            &mut client,
            json!({"command":"bond.create","params":bond()})
        )?
        .get("error")
        .is_none()
    );
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    let refused = socket(
        &mut client,
        json!({"command":"document.new","params":{"expected_revision":3}}),
    )?;
    assert_eq!(refused["error"]["code"], "unsaved_changes");
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?,
        before
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while changes.load(Ordering::SeqCst) < 3 {
        if std::time::Instant::now() >= deadline {
            return Err("New notifications did not arrive".into());
        }
        std::thread::yield_now();
    }
    assert_eq!(changes.load(Ordering::SeqCst), 3);
    fresh(
        &socket(
            &mut client,
            json!({"command":"document.new","params":{"expected_revision":3,"discard_unsaved":true}}),
        )?["result"],
        4,
    )?;
    assert_eq!(fs::read(input)?, SOURCE);
    drop(client);
    server.shutdown()?;
    Ok(())
}
