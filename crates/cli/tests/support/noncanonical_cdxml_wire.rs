//! Existing transport helpers; no listener/handshake/deadline or registry changes.
use super::*;
use std::path::Path;
const ORIGINAL: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<CDXML CreationProgram="Bondline 0.1.0">
 <page id='91' Width='612.00' Height="792.000" DrawingSpace='Poster'>
  <fragment id="400">
   <b id='900' E='207' Order='1' B='101' />
   <n Radical='None' id='101' p='100.00 200.000' NumHydrogens='3' Charge='0' Element='6' NodeType='Element'/>
   <n Element="6" NumHydrogens="3" NodeType="Element" p="140 200" id="207" Radical="None" Charge="0"></n>
  </fragment>
 </page>
</CDXML>
"#;
const DOUBLE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<CDXML CreationProgram="Bondline 0.1.0">
 <page id='91' Width='612.00' Height="792.000" DrawingSpace='Poster'>
  <fragment id="400">
   <b id='900' E='207' Order='2' B='101' />
   <n Radical='None' id='101' p='100.00 200.000' NumHydrogens='2' Charge='0' Element='6' NodeType='Element'/>
   <n Element="6" NumHydrogens="2" NodeType="Element" p="140 200" id="207" Radical="None" Charge="0"></n>
  </fragment>
 </page>
</CDXML>
"#;
fn order(revision: u64) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"})
}
fn checked(
    value: &Value,
    revision: u64,
    depths: [u64; 2],
    h: u8,
    dirty: bool,
    modified: bool,
) -> TestResult {
    assert_eq!(value["revision"], revision);
    assert_eq!(value["undo_depth"], depths[0]);
    assert_eq!(value["redo_depth"], depths[1]);
    assert_eq!(value["file"]["format"], "cdxml");
    assert_eq!(value["file"]["dirty"], dirty);
    assert_eq!(
        value["source"],
        json!({"format":"cdxml","byte_length":464,"modified":modified})
    );
    assert_eq!(
        value["document"]["pages"][0]["size"],
        json!({"width_pt":612.0,"height_pt":792.0})
    );
    let f = &value["document"]["pages"][0]["fragments"][0];
    assert_eq!(f["id"], 1);
    assert_eq!(
        f["atoms"],
        json!([
        {"id":1,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h},
        {"id":2,"element":6,"position":{"x_pt":140.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h}])
    );
    assert_eq!(
        f["bonds"],
        json!([{"id":1,"begin":1,"end":2,"order":if h==2{"double"}else{"single"}}])
    );
    Ok(())
}
fn consume(
    path: &Path,
    expected: &[u8],
    h: u8,
    weight: &str,
    smiles: &str,
    formula: &str,
) -> TestResult {
    assert_eq!(fs::read(path)?, expected);
    let graph = json!([{"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h},
        {"id":2,"element":6,"position":{"x_pt":140.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":weight}]}]);
    let expected = serde_json::to_string(
        &json!({"graph":graph,"smiles":smiles,"formula":formula,"flags":[null,null],"double_positions":[null]}),
    )?;
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../commands/tests/ring_create/named_ring_oracle.py"
            ),
            &expected,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(&fs::read(path)?)?;
    let out = child.wait_with_output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let facts: Value = serde_json::from_slice(&out.stdout)?;
    assert_eq!(
        facts,
        json!({"atoms":2,"bonds":1,"formula":formula,"literal_integer_orders":true,"literal_direction":true,"literal_flags_and_double_positions":true,"no_assigned_stereo":true})
    );
    Ok(())
}

#[test]
fn actual_cli_and_initialized_stdio_preserve_three_values_and_real_unicode_files() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("noncanonical-λ.cdxml");
    let manual = directory.path("manual-double.cdxml");
    let cli_output = directory.path("cli-double-化学.cdxml");
    let stdio_output = directory.path("stdio-double-λ.cdxml");
    fs::write(&input, ORIGINAL)?;
    fs::write(&manual, DOUBLE)?;
    consume(&input, ORIGINAL, 3, "single", "CC", "C2H6")?;
    consume(&manual, DOUBLE, 2, "double", "C=C", "C2H4")?;
    let result = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":input}}),
        json!({"command":"bond.set_order","params":order(1)}),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":cli_output}}),
        json!({"command":"document.open_cdxml","params":{"path":cli_output}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    checked(&result, 5, [0, 0], 2, false, false)?;
    assert_eq!(
        result["file"]["path"],
        json!(fs::canonicalize(&cli_output)?)
    );
    consume(&cli_output, DOUBLE, 2, "double", "C=C", "C2H4")?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"preserved-cdxml-test","version":"1"}}}))?;
    assert_eq!(
        client.replies.recv_timeout(Duration::from_secs(10))??["id"],
        1
    );
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    checked(
        &client.call(2, "document.open_cdxml", json!({"path":input}))?,
        1,
        [0, 0],
        3,
        false,
        false,
    )?;
    checked(
        &client.call(3, "bond.set_order", order(1))?,
        2,
        [1, 0],
        2,
        true,
        true,
    )?;
    checked(
        &client.call(4, "history.undo", json!({}))?,
        3,
        [0, 1],
        3,
        false,
        false,
    )?;
    checked(
        &client.call(5, "history.redo", json!({}))?,
        4,
        [1, 0],
        2,
        true,
        true,
    )?;
    let saved = client.call(6, "document.save_cdxml", json!({"path":stdio_output}))?;
    checked(&saved, 4, [1, 0], 2, false, true)?;
    assert_eq!(saved["file_operation"]["byte_length"], 464);
    assert_eq!(saved["file_operation"]["ui_request_id"], Value::Null);
    assert_eq!(saved["file_operation"]["ui_request_current"], Value::Null);
    checked(
        &client.call(7, "document.open_cdxml", json!({"path":stdio_output}))?,
        5,
        [0, 0],
        2,
        false,
        false,
    )?;
    consume(&stdio_output, DOUBLE, 2, "double", "C=C", "C2H4")?;
    assert_eq!(fs::read(&input)?, ORIGINAL);
    assert_eq!(fs::read(&stdio_output)?, fs::read(&cli_output)?);
    client.finish()
}

#[test]
fn authenticated_loopback_saves_preserved_source_but_never_grants_opaque_or_whole_mol_authority()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("socket-noncanonical.cdxml");
    let output = directory.path("socket-double-化学.cdxml");
    let opaque = directory.path("socket-opaque.cdxml");
    let sentinel = directory.path("keep.mol");
    let unknown = std::str::from_utf8(ORIGINAL)?.replacen(
        "<CDXML CreationProgram=",
        "<CDXML Future='retained &amp; unknown' CreationProgram=",
        1,
    );
    fs::write(&input, ORIGINAL)?;
    fs::write(&opaque, unknown.as_bytes())?;
    fs::write(
        &sentinel,
        b"keep existing destination
",
    )?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
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
        json!({"command":"document.open_cdxml","params":{"path":input}}),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    checked(&opened["result"], 1, [0, 0], 3, false, false)?;
    let changed = socket(
        &mut client,
        json!({"command":"bond.set_order","params":order(1)}),
    )?;
    assert!(changed.get("error").is_none(), "{changed}");
    checked(&changed["result"], 2, [1, 0], 2, true, true)?;
    let saved = socket(
        &mut client,
        json!({"command":"document.save_cdxml","params":{"path":output}}),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    checked(&saved["result"], 2, [1, 0], 2, false, true)?;
    assert_eq!(
        saved["result"]["file_operation"]["ui_request_id"],
        Value::Null
    );
    consume(&output, DOUBLE, 2, "double", "C=C", "C2H4")?;
    let reopened = socket(
        &mut client,
        json!({"command":"document.open_cdxml","params":{"path":output}}),
    )?;
    assert!(reopened.get("error").is_none(), "{reopened}");
    checked(&reopened["result"], 3, [0, 0], 2, false, false)?;
    // A4 removes the old layout reason: refusal below must retain the new source-class boundary.
    let a4 = {
        let native = Engine::default();
        let page = native.document()?.pages().first().ok_or("default page")?;
        [page.size().width_pt(), page.size().height_pt()]
    };
    let resized = socket(
        &mut client,
        json!({"command":"page.set_size","params":{"page_id":1,"width_pt":a4[0],"height_pt":a4[1]}}),
    )?;
    assert!(resized.get("error").is_none(), "{resized}");
    assert!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .is_fully_mapped()
    );
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    let (owner, stamp) = {
        let e = engine.lock().map_err(|_| "engine")?;
        (e.content().clone(), e.capture_stamp())
    };
    let denied = socket(
        &mut client,
        json!({"command":"document.save_mol","params":{"path":sentinel,"overwrite":true}}),
    )?;
    assert_eq!(denied["error"]["code"], "unsupported_content");
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?["result"],
        before
    );
    {
        let e = engine.lock().map_err(|_| "engine")?;
        assert_eq!(e.content(), &owner);
        assert!(e.capture_stamp_matches(&stamp));
    }
    assert_eq!(
        fs::read(&sentinel)?,
        b"keep existing destination
"
    );
    let undone = socket(&mut client, json!({"command":"history.undo","params":{}}))?;
    assert!(undone.get("error").is_none(), "{undone}");
    checked(&undone["result"], 5, [0, 1], 2, false, false)?;
    let read_only = socket(
        &mut client,
        json!({"command":"document.open_cdxml","params":{"path":opaque}}),
    )?;
    assert!(read_only.get("error").is_none(), "{read_only}");
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    let revision = before["revision"].as_u64().ok_or("current revision")?;
    let denied = socket(
        &mut client,
        json!({"command":"bond.set_order","params":order(revision)}),
    )?;
    assert_eq!(denied["error"]["code"], "unsupported_content");
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?["result"],
        before
    );
    assert_eq!(fs::read(&opaque)?, unknown.as_bytes());
    assert_eq!(fs::read(&input)?, ORIGINAL);
    assert_eq!(fs::read(&output)?, DOUBLE);
    drop(client);
    server.shutdown()?;
    Ok(())
}
