//! Actual transports over the shared complete-component converter.
use super::*;

#[path = "../../../../xtask/oracles/components_test_support.rs"]
mod component_reference;
use component_reference::consume;

const CONVERT: &str = "file.convert_smiles_to_cdxml";
const INPUT: &[u8] = b"CC.O.C1CCCCC1\r\n";
const WHOLE: &[u8] = b"CC PRIVATE_FIRST\r\nO\tPRIVATE_SECOND\xff\0\n";

fn params(revision: u64, input: &std::path::Path, output: &std::path::Path) -> Value {
    json!({"expected_revision":revision,"input_path":input,"output_path":output,
        "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
        "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4,
        "profile":"neutral_components_v1"})
}

fn receipt(mut result: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let receipt = result
        .as_object_mut()
        .ok_or("result")?
        .remove("conversion")
        .ok_or("receipt")?;
    assert_eq!(receipt["profile"], "neutral_components_v1");
    assert_eq!(receipt["atom_count"], 9);
    assert_eq!(receipt["bond_count"], 7);
    assert_eq!(receipt["input_byte_length"], INPUT.len());
    assert!(result.get("file_operation").is_none());
    Ok(result)
}

#[test]
fn cli_and_stdio_convert_every_component_without_replacing_complete_source() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let source = directory.path("whole.smi");
    let input = directory.path("components.smi");
    let output = directory.path("stdio-components-λ.cdxml");
    fs::write(&source, WHOLE)?;
    fs::write(&input, INPUT)?;
    let mut client = Client::start()?;
    client.send(
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-11-25","capabilities":{},
            "clientInfo":{"name":"components-acceptance","version":"1"}
        }}),
    )?;
    assert_eq!(
        client.replies.recv_timeout(Duration::from_secs(10))??["id"],
        1
    );
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 38);
    let converter = tools
        .iter()
        .find(|t| t["name"] == CONVERT)
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
    assert_eq!(before["smiles_file"]["record_count"], 2);
    assert!(!before.to_string().contains("PRIVATE_"));
    assert_eq!(
        receipt(client.call(5, CONVERT, params(1, &input, &output))?)?,
        before
    );
    consume(&output, false)?;
    assert_eq!(client.call(6, "document.inspect", json!({}))?, before);

    let cli_output = directory.path("cli-components-λ.cdxml");
    let result = cli(&[
        json!({"command":"document.open_smiles","params":{"path":source}}),
        json!({"command":CONVERT,"params":params(1,&input,&cli_output)}),
    ])?;
    assert_eq!(receipt(result)?, before);
    consume(&cli_output, false)?;
    assert_eq!(fs::read(&cli_output)?, fs::read(&output)?);
    let whole_copy = directory.path("whole-化学.smi");
    client.call(7, "document.save_smiles", json!({"path":whole_copy}))?;
    assert_eq!(fs::read(&whole_copy)?, WHOLE);
    client.call(8, "document.open_cdxml", json!({"path":output}))?;
    let opened = client.call(9, "document.inspect", json!({}))?;
    assert_eq!(opened["revision"], 2);
    assert_eq!(opened["file"]["dirty"], false);
    client.call(
        10,
        "bond.set_order",
        json!({
            "expected_revision":2,"page_id":1,"fragment_id":1,"bond_id":1,
            "order":"double","hydrogen_policy":"rectify_neutral_carbon"
        }),
    )?;
    let saved = directory.path("edited-components-λ.cdxml");
    client.call(11, "document.save_cdxml", json!({"path":saved}))?;
    consume(&saved, true)?;
    client.call(12, "document.open_cdxml", json!({"path":saved}))?;
    let reopened = client.call(13, "document.inspect", json!({}))?;
    assert_eq!(reopened["file"]["format"], "cdxml");
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(fs::read(&source)?, WHOLE);
    assert_eq!(fs::read(&input)?, INPUT);
    client.finish()
}

#[test]
fn authenticated_component_conversion_preserves_dirty_content_and_refuses_republication()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("components.smi");
    let output = directory.path("socket-components-λ.cdxml");
    fs::write(&input, INPUT)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let notified = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        notified.fetch_add(1, Ordering::SeqCst);
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
        json!({"command":"bond.create","params":{
            "page_id":1,"begin_x_pt":72.0,"begin_y_pt":144.0,"end_x_pt":120.0,"end_y_pt":144.0
        }}),
    )?;
    assert!(created.get("error").is_none(), "{created}");
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    assert_eq!(before["file"]["dirty"], true);
    assert_eq!(before["revision"], 1);
    let (content, stamp) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.content().clone(), guard.capture_stamp())
    };
    let notifications = changes.load(Ordering::SeqCst);
    let converted = socket(
        &mut client,
        json!({"command":CONVERT,"params":params(1,&input,&output)}),
    )?;
    assert!(converted.get("error").is_none(), "{converted}");
    assert_eq!(receipt(converted["result"].clone())?, before);
    consume(&output, false)?;
    let bytes = fs::read(&output)?;
    let refused = socket(
        &mut client,
        json!({"command":CONVERT,"params":params(1,&input,&output)}),
    )?;
    assert_eq!(refused["error"]["code"], "already_exists");
    let stale = directory.path("stale-components.cdxml");
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
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &content);
        assert!(guard.capture_stamp_matches(&stamp));
    }
    assert_eq!(changes.load(Ordering::SeqCst), notifications);
    assert_eq!(fs::read(&output)?, bytes);
    assert_eq!(fs::read(&input)?, INPUT);
    drop(client);
    server.shutdown()?;
    Ok(())
}
