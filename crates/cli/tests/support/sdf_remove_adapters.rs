//! Additional wire acceptance over the existing unchanged transport helpers.
use super::*;

fn assert_remaining(value: &Value) {
    assert_eq!(value["document"], Value::Null);
    assert_eq!(value["collection"]["record_count"], 1);
    assert_eq!(value["collection"]["byte_length"], 315);
    assert_eq!(value["collection"]["records"][0]["id"], 1);
    assert_eq!(
        value["source"],
        json!({"format":"sdf","byte_length":777,"modified":true})
    );
    assert_eq!(value["file"]["format"], "sdf");
    assert_eq!(value["undo_depth"], 1);
}

fn consume(path: &std::path::Path) -> TestResult {
    let bytes = fs::read(path)?;
    assert_eq!(bytes, SOURCE.get(462..).ok_or("literal suffix")?);
    for (script, input) in [
        ("sdf_oracle.py", bytes.as_slice()),
        ("mol_oracle.py", bytes.get(..282).ok_or("literal MOL span")?),
    ] {
        let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned consumer interpreter")?;
        let mut child = Command::new(python)
            .args([
                "-B",
                &format!("{}/../io/tests/{script}", env!("CARGO_MANIFEST_DIR")),
                "describe",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("consumer stdin")?
            .write_all(input)?;
        let result = child.wait_with_output()?;
        assert!(
            result.status.success(),
            "{script}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: Value = serde_json::from_slice(&result.stdout)?;
        if script == "sdf_oracle.py" {
            assert_eq!(report["records"].as_array().ok_or("records")?.len(), 1);
            assert_eq!(report["records"][0]["fields"][0]["name"], "ORDINARY");
            assert_eq!(
                report["records"][0]["fields"][0]["value_lines"],
                json!(["second record"])
            );
        } else {
            assert_eq!(report["smiles"], "CC");
            assert_eq!(report["formula"], "C2H6");
        }
    }
    Ok(())
}

#[test]
fn cli_removal_saves_only_the_literal_survivor_and_keeps_original_input() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("remove-cli.sdf");
    let output = dir.path("remove-cli-化学.sdf");
    fs::write(&input, SOURCE)?;
    let value = cli(&[
        json!({"command":"document.open_sdf","params":{"path":input}}),
        json!({"command":"sdf.record.remove","params":{"expected_revision":1,"record_id":1}}),
        json!({"command":"document.save_sdf","params":{"path":output}}),
        json!({"command":"sdf.record.inspect","params":{"expected_revision":2,"record_id":1}}),
    ])?;
    assert_remaining(&value);
    assert_eq!(value["file"]["dirty"], false);
    assert_eq!(value["sdf_record"]["record_id"], 1);
    assert_eq!(
        value["sdf_record"]["fields"][0]["value_lines"],
        json!(["second record"])
    );
    assert_eq!(fs::read(&input)?, SOURCE);
    consume(&output)?;
    Ok(())
}

#[test]
fn stdio_sdk_discovers_mutating_removal_and_restores_exact_history_before_save() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("remove-stdio.sdf");
    let output = dir.path("remove-stdio-化学.sdf");
    fs::write(&input, SOURCE)?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sdf-record-remove-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 48);
    let remove = tools
        .iter()
        .find(|tool| tool["name"] == "sdf.record.remove")
        .ok_or("remove tool")?;
    assert_eq!(remove["inputSchema"]["additionalProperties"], false);
    assert_eq!(remove["annotations"]["readOnlyHint"], false);
    client.call(3, "document.open_sdf", json!({"path":input}))?;
    let removed = client.call(
        4,
        "sdf.record.remove",
        json!({"expected_revision":1,"record_id":1}),
    )?;
    assert_remaining(&removed);
    assert_eq!(removed["file"]["dirty"], true);
    let undo = client.call(5, "history.undo", json!({}))?;
    assert_eq!(undo["collection"]["byte_length"], 777);
    assert_eq!(undo["collection"]["record_count"], 2);
    assert_eq!(undo["file"]["dirty"], false);
    let redo = client.call(6, "history.redo", json!({}))?;
    assert_remaining(&redo);
    let saved = client.call(7, "document.save_sdf", json!({"path":output}))?;
    assert_remaining(&saved);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(fs::read(&input)?, SOURCE);
    consume(&output)?;
    client.finish()?;
    Ok(())
}

#[test]
fn authenticated_loopback_removal_notifies_and_refuses_to_empty_the_file() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("remove-loopback.sdf");
    let output = dir.path("remove-loopback-化学.sdf");
    fs::write(&input, SOURCE)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        count.fetch_add(1, Ordering::SeqCst);
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
        .get("result")
        .is_some()
    );
    let removed = socket(
        &mut client,
        json!({"command":"sdf.record.remove","params":{"expected_revision":1,"record_id":1}}),
    )?;
    assert_remaining(&removed["result"]);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while changes.load(Ordering::SeqCst) < 2 {
        if std::time::Instant::now() >= deadline {
            return Err("removal did not notify the host".into());
        }
        std::thread::yield_now();
    }
    assert_eq!(changes.load(Ordering::SeqCst), 2);
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    let refused = socket(
        &mut client,
        json!({"command":"sdf.record.remove","params":{"expected_revision":2,"record_id":1}}),
    )?;
    assert_eq!(refused["error"]["code"], "invalid_sdf");
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?,
        before
    );
    let saved = socket(
        &mut client,
        json!({"command":"document.save_sdf","params":{"path":output}}),
    )?;
    assert_remaining(&saved["result"]);
    assert_eq!(saved["result"]["file"]["dirty"], false);
    assert_eq!(fs::read(&input)?, SOURCE);
    consume(&output)?;
    drop(client);
    server.shutdown()?;
    Ok(())
}
