//! Actual adapters over the same create-only command and existing client framing.
use super::*;

const CONVERT: &str = "file.convert_smiles_to_cdxml";
const INPUT: &[u8] = b"CC1CCCCC1\n";
const WHOLE: &[u8] = b"CC PRIVATE_FIRST\r\nO\tPRIVATE_SECOND\xff\0\n";

fn params(revision: u64, input: &std::path::Path, output: &std::path::Path) -> Value {
    json!({"expected_revision":revision,"input_path":input,"output_path":output,
        "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
        "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4,
        "profile":"neutral_simple_ring_v1"})
}

fn consume(path: &std::path::Path) -> TestResult {
    let output = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_ring_oracle.py"
        ))
        .arg("CC1CCCCC1")
        .arg(
            json!({"atoms":[[6,3],[6,1],[6,2],[6,2],[6,2],[6,2],[6,2]],
            "bonds":[[1,2,1],[2,3,1],[3,4,1],[4,5,1],[5,6,1],[6,7,1],[2,7,1]],
            "formula":"C7H14"})
            .to_string(),
        )
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["formula"], "C7H14");
    assert_eq!(value["atoms"], 7);
    assert_eq!(value["bonds"], 7);
    assert_eq!(value["no_assigned_stereo"], true);
    Ok(())
}

fn receipt(mut state: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let converted = state
        .as_object_mut()
        .ok_or("state")?
        .remove("conversion")
        .ok_or("receipt")?;
    assert_eq!(converted["profile"], "neutral_simple_ring_v1");
    assert_eq!(converted["atom_count"], 7);
    assert_eq!(converted["bond_count"], 7);
    assert!(state.get("file_operation").is_none());
    Ok(state)
}

#[test]
fn cli_and_stdio_ring_conversion_keep_the_complete_source_session() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let source = directory.path("whole.smi");
    let input = directory.path("ring.smi");
    let output = directory.path("stdio-λ.cdxml");
    fs::write(&source, WHOLE)?;
    fs::write(&input, INPUT)?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ring-acceptance","version":"1"}}}))?;
    assert_eq!(
        client.replies.recv_timeout(Duration::from_secs(10))??["id"],
        1
    );
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 54);
    let converter = tools
        .iter()
        .find(|tool| tool["name"] == CONVERT)
        .ok_or("converter")?;
    assert_eq!(converter["annotations"]["readOnlyHint"], false);
    assert_eq!(converter["inputSchema"]["additionalProperties"], false);
    assert_eq!(
        converter["inputSchema"]["required"]
            .as_array()
            .ok_or("required")?
            .len(),
        8
    );
    client.call(3, "document.open_smiles", json!({"path":source}))?;
    let before = client.call(4, "document.inspect", json!({}))?;
    assert_eq!(before["document"], Value::Null);
    assert!(!before.to_string().contains("PRIVATE_"));
    let result = client.call(5, CONVERT, params(1, &input, &output))?;
    assert_eq!(receipt(result)?, before);
    assert_eq!(client.call(6, "document.inspect", json!({}))?, before);
    consume(&output)?;
    let cli_output = directory.path("cli-λ.cdxml");
    let cli_result = cli(&[
        json!({"command":"document.open_smiles","params":{"path":source}}),
        json!({"command":CONVERT,"params":params(1,&input,&cli_output)}),
    ])?;
    assert_eq!(receipt(cli_result)?["smiles_file"]["record_count"], 2);
    consume(&cli_output)?;
    assert_eq!(fs::read(&output)?, fs::read(&cli_output)?);
    let saved = directory.path("whole-copy.smi");
    client.call(7, "document.save_smiles", json!({"path":saved}))?;
    assert_eq!(fs::read(&saved)?, WHOLE);
    client.call(8, "document.open_cdxml", json!({"path":output}))?;
    let opened = client.call(9, "document.inspect", json!({}))?;
    assert_eq!(opened["file"]["format"], "cdxml");
    assert_eq!(opened["file"]["dirty"], false);
    assert_eq!(fs::read(&source)?, WHOLE);
    assert_eq!(fs::read(&input)?, INPUT);
    client.finish()
}

#[test]
fn authenticated_ring_conversion_is_create_only_and_cannot_replace_dirty_content() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("ring.smi");
    let output = directory.path("socket-λ.cdxml");
    fs::write(&input, INPUT)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
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
    let created = socket(
        &mut client,
        json!({"command":"bond.create","params":{"page_id":1,"begin_x_pt":72.0,"begin_y_pt":144.0,"end_x_pt":120.0,"end_y_pt":144.0}}),
    )?;
    assert!(created.get("error").is_none(), "{created}");
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    assert_eq!(before["file"]["dirty"], true);
    assert_eq!(before["revision"], 1);
    let content = engine.lock().map_err(|_| "engine")?.content().clone();
    let prior_changes = changes.load(Ordering::SeqCst);
    let converted = socket(
        &mut client,
        json!({"command":CONVERT,"params":params(1,&input,&output)}),
    )?;
    assert!(converted.get("error").is_none(), "{converted}");
    assert_eq!(receipt(converted["result"].clone())?, before);
    consume(&output)?;
    let bytes = fs::read(&output)?;
    let refused = socket(
        &mut client,
        json!({"command":CONVERT,"params":params(1,&input,&output)}),
    )?;
    assert_eq!(refused["error"]["code"], "already_exists");
    let stale = directory.path("stale.cdxml");
    let refused = socket(
        &mut client,
        json!({"command":CONVERT,"params":params(0,&input,&stale)}),
    )?;
    assert_eq!(refused["error"]["code"], "stale_revision");
    assert!(!stale.try_exists()?);
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?["result"],
        before
    );
    assert_eq!(engine.lock().map_err(|_| "engine")?.content(), &content);
    assert_eq!(changes.load(Ordering::SeqCst), prior_changes);
    assert_eq!(fs::read(&output)?, bytes);
    assert_eq!(fs::read(&input)?, INPUT);
    drop(client);
    server.shutdown()?;
    Ok(())
}
