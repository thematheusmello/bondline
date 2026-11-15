//! Original external-byte acceptance through the unchanged shared command transports.
use super::*;

const SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../cdx/tests/fixtures/external-ethane-indigo-1.47.0.cdx"
));

fn projected(value: &Value) -> TestResult {
    assert_eq!(value["file"]["format"], "cdx");
    assert_eq!(value["file"]["dirty"], false);
    assert_eq!(
        value["source"],
        json!({"format":"cdx","byte_length":258,"modified":false})
    );
    assert_eq!(value["undo_depth"], 0);
    assert_eq!(value["redo_depth"], 0);
    let fragments = value["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let fragment = fragments.first().ok_or("fragment")?;
    let atoms = fragment["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 2);
    for (atom, (id, x)) in atoms.iter().zip([(1, 0.0), (2, 45.0)]) {
        assert_eq!(atom["id"], id);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["position"], json!({"x_pt":x,"y_pt":0.0}));
        assert_eq!(atom["explicit_hydrogens"], Value::Null);
    }
    assert_eq!(
        fragment["bonds"],
        json!([{"id":1,"begin":1,"end":2,"order":"single"}])
    );
    Ok(())
}
fn consume(path: &std::path::Path) -> TestResult {
    assert_eq!(fs::read(path)?, SOURCE);
    let output = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../cdx/tests/external_ethane_oracle.py"
        ))
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({"atoms":2,"bonds":1,"formula":"C2H6","smiles":"CC","hydrogens":[3,3],"byte_length":258,"record_count":24,"header_bytes":28,"document_end":[254,256],"file_end":[256,258],"raw_y_x16_16":[[0,0],[0,2949120]]})
    );
    Ok(())
}

#[test]
fn cli_and_stdio_external_cdx_file_end_open_save_reopen_preserve_original_bytes() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("file-end-input-λ.cdx");
    let cli_output = directory.path("cli-copy-化学.cdx");
    let stdio_output = directory.path("stdio-copy-λ.cdx");
    fs::write(&input, SOURCE)?;
    let opened = cli(&[json!({"command":"document.open_cdx","params":{"path":input}})])?;
    projected(&opened)?;
    assert_eq!(opened["revision"], 1);
    let result = cli(&[
        json!({"command":"document.open_cdx","params":{"path":input}}),
        json!({"command":"document.save_cdx","params":{"path":cli_output}}),
        json!({"command":"document.open_cdx","params":{"path":cli_output}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    projected(&result)?;
    assert_eq!(result["revision"], 2);
    assert_eq!(
        result["file"]["path"],
        json!(fs::canonicalize(&cli_output)?)
    );
    consume(&cli_output)?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"cdx-file-end-test","version":"1"}}}))?;
    assert_eq!(
        client.replies.recv_timeout(Duration::from_secs(10))??["id"],
        1
    );
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let opened = client.call(2, "document.open_cdx", json!({"path":input}))?;
    projected(&opened)?;
    let saved = client.call(3, "document.save_cdx", json!({"path":stdio_output}))?;
    projected(&saved)?;
    assert_eq!(saved["file_operation"]["operation"], "save");
    assert_eq!(saved["file_operation"]["byte_length"], 258);
    assert_eq!(saved["file_operation"]["ui_request_id"], Value::Null);
    assert_eq!(saved["file_operation"]["ui_request_current"], Value::Null);
    let reopened = client.call(4, "document.open_cdx", json!({"path":stdio_output}))?;
    projected(&reopened)?;
    assert_eq!(reopened["revision"], 2);
    let stable = client.call(5, "document.inspect", json!({}))?;
    projected(&stable)?;
    assert_eq!(
        stable["file"]["path"],
        json!(fs::canonicalize(&stdio_output)?)
    );
    consume(&stdio_output)?;
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::read(&stdio_output)?, fs::read(&cli_output)?);
    client.finish()
}

#[test]
fn authenticated_external_cdx_file_end_retains_authority_and_exact_saved_source() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("socket-input.cdx");
    let output = directory.path("socket-copy-λ.cdx");
    let cross = directory.path("must-not-convert.cdxml");
    fs::write(&input, SOURCE)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let server = LoopbackServer::bind(0, Arc::clone(&engine), || {})?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    // The fresh credential is only sent to this trusted test connection, never logged.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let opened = socket(
        &mut client,
        json!({"command":"document.open_cdx","params":{"path":input}}),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    projected(&opened["result"])?;
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    let (owner, stamp) = {
        let current = engine.lock().map_err(|_| "engine")?;
        (current.content().clone(), current.capture_stamp())
    };
    assert!(
        !engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .is_fully_mapped()
    );
    for (command, params) in [
        (
            "render.capture",
            json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
        ),
        ("document.save_cdxml", json!({"path":cross})),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
        ),
    ] {
        let refused = socket(&mut client, json!({"command":command,"params":params}))?;
        assert_eq!(refused["error"]["code"], "unsupported_content");
        assert_eq!(
            socket(
                &mut client,
                json!({"command":"document.inspect","params":{}})
            )?["result"],
            before
        );
        {
            let current = engine.lock().map_err(|_| "engine")?;
            assert_eq!(current.content(), &owner);
            assert!(current.capture_stamp_matches(&stamp));
        }
        assert!(!cross.try_exists()?);
    }
    let saved = socket(
        &mut client,
        json!({"command":"document.save_cdx","params":{"path":output}}),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    projected(&saved["result"])?;
    assert_eq!(saved["result"]["file_operation"]["byte_length"], 258);
    assert_eq!(
        saved["result"]["file_operation"]["ui_request_id"],
        Value::Null
    );
    consume(&output)?;
    let reopened = socket(
        &mut client,
        json!({"command":"document.open_cdx","params":{"path":output}}),
    )?;
    assert!(reopened.get("error").is_none(), "{reopened}");
    projected(&reopened["result"])?;
    assert_eq!(reopened["result"]["revision"], 2);
    assert_eq!(
        reopened["result"]["file"]["path"],
        json!(fs::canonicalize(&output)?)
    );
    assert_eq!(fs::read(&input)?, SOURCE);
    drop(client);
    server.shutdown()?;
    Ok(())
}
