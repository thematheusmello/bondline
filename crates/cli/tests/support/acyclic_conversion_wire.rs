//! Actual adapters, reusing the existing stdio client and authenticated socket framing.
use super::*;

const ACYCLIC: &str = "file.convert_smiles_to_cdxml";
const SOURCE_FILE: &[u8] = b"CC PRIVATE_FIRST\r\nO\tPRIVATE_SECOND\xff\0\n";

fn acyclic_params(revision: u64, input: &std::path::Path, output: &std::path::Path) -> Value {
    json!({"expected_revision":revision,"input_path":input,"output_path":output,
        "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
        "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4,
        "profile":"neutral_acyclic_v1"})
}

fn inspect_without_conversion(mut value: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let receipt = value
        .as_object_mut()
        .ok_or("state")?
        .remove("conversion")
        .ok_or("receipt")?;
    assert_eq!(receipt["profile"], "neutral_acyclic_v1");
    assert!(value.get("file_operation").is_none());
    Ok(value)
}

fn consume_acyclic(
    path: &std::path::Path,
    notation: &str,
    atoms: Value,
    bonds: Value,
    formula: &str,
) -> TestResult {
    let result = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_acyclic_oracle.py"
        ))
        .arg(notation)
        .arg(
            json!({"atoms":atoms,"bonds":bonds,"formula":formula,"length":"48",
            "origin":[72,72],"page":[210.0*72.0/25.4,297.0*72.0/25.4]})
            .to_string(),
        )
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout)?,
        json!({"atoms":atoms.as_array().ok_or("atoms")?.len(),
        "bonds":bonds.as_array().ok_or("bonds")?.len(),"formula":formula,"stereo_none":true,"physical_h":0,
            "no_assigned_stereo":true,
            "bond_stereo":vec!["STEREONONE"; bonds.as_array().ok_or("literal bonds")?.len()]})
    );
    Ok(())
}

#[test]
fn stdio_acyclic_profile_is_discovered_and_never_implicitly_replaces_a_complete_source()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let source = directory.path("whole-source-λ.smi");
    let input = directory.path("acid.smi");
    let output = directory.path("acid-化学.cdxml");
    fs::write(&source, SOURCE_FILE)?;
    fs::write(&input, b"CC(=O)O\n")?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"acyclic-acceptance","version":"1"}}}))?;
    assert_eq!(
        client.replies.recv_timeout(Duration::from_secs(10))??["id"],
        1
    );
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 50);
    let converters = tools
        .iter()
        .filter(|tool| tool["name"] == ACYCLIC)
        .collect::<Vec<_>>();
    assert_eq!(converters.len(), 1);
    assert_eq!(converters[0]["annotations"]["readOnlyHint"], false);
    assert_eq!(converters[0]["inputSchema"]["additionalProperties"], false);
    assert_eq!(
        converters[0]["inputSchema"]["required"]
            .as_array()
            .ok_or("required")?
            .len(),
        8
    );
    assert!(
        converters[0]["inputSchema"]["properties"]
            .get("profile")
            .is_some()
    );
    client.call(3, "document.open_smiles", json!({"path":source}))?;
    let before = client.call(4, "document.inspect", json!({}))?;
    assert_eq!(before["document"], Value::Null);
    assert_eq!(before["smiles_file"]["record_count"], 2);
    assert!(!before.to_string().contains("PRIVATE_"));
    let converted = client.call(5, ACYCLIC, acyclic_params(1, &input, &output))?;
    assert_eq!(converted["conversion"]["input_revision"], 1);
    assert_eq!(converted["conversion"]["session_generation"], 1);
    assert_eq!(converted["conversion"]["atom_count"], 4);
    assert_eq!(converted["conversion"]["bond_count"], 3);
    assert_eq!(
        converted["conversion"]["output"]["path"],
        json!(fs::canonicalize(&output)?)
    );
    assert_eq!(
        converted["conversion"]["output"]["byte_length"],
        fs::read(&output)?.len()
    );
    assert_eq!(inspect_without_conversion(converted)?, before);
    assert_eq!(client.call(6, "document.inspect", json!({}))?, before);
    consume_acyclic(
        &output,
        "CC(=O)O",
        json!([[6, 3], [6, 0], [8, 0], [8, 1]]),
        json!([[1, 2, 1], [2, 3, 2], [2, 4, 1]]),
        "C2H4O2",
    )?;
    let mut invalid = acyclic_params(1, &input, &directory.path("invalid.cdxml"));
    invalid["profile"] = Value::Null;
    client.send(json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":ACYCLIC,"arguments":invalid}}))?;
    let refused = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(refused["id"], 7);
    assert_eq!(refused["result"]["isError"], true);
    assert_eq!(
        refused["result"]["structuredContent"]["error"]["code"],
        "invalid_params"
    );
    assert_eq!(client.call(8, "document.inspect", json!({}))?, before);
    assert!(!directory.path("invalid.cdxml").try_exists()?);
    let copied = directory.path("whole-copy.smi");
    client.call(9, "document.save_smiles", json!({"path":copied}))?;
    assert_eq!(fs::read(&copied)?, SOURCE_FILE);
    client.call(10, "document.open_smiles", json!({"path":copied}))?;
    let reopened = client.call(11, "document.inspect", json!({}))?;
    assert_eq!(reopened["document"], Value::Null);
    assert_eq!(reopened["smiles_file"]["record_count"], 2);
    assert!(!reopened.to_string().contains("PRIVATE_"));
    let drawing = client.call(12, "document.open_cdxml", json!({"path":output}))?;
    assert!(drawing["document"].is_object());
    assert_eq!(
        drawing["document"]["pages"][0]["fragments"][0]["atoms"]
            .as_array()
            .ok_or("atoms")?
            .len(),
        4
    );
    assert_eq!(drawing["file"]["dirty"], false);
    assert_eq!(fs::read(&source)?, SOURCE_FILE);
    assert_eq!(fs::read(&input)?, b"CC(=O)O\n");
    client.finish()
}

#[test]
fn authenticated_acyclic_publication_retains_dirty_state_and_obeys_the_shared_save_lease()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("branch.smi");
    let output = directory.path("branch-λ.cdxml");
    fs::write(&input, b"CC(C)C\n")?;
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
        json!({"command":"bond.create","params":{"page_id":1,
        "begin_x_pt":72.0,"begin_y_pt":144.0,"end_x_pt":120.0,"end_y_pt":144.0}}),
    )?;
    assert!(created.get("error").is_none(), "{created}");
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    assert_eq!(before["revision"], 1);
    assert_eq!(before["file"]["dirty"], true);
    let content = engine
        .lock()
        .map_err(|_| "engine poisoned")?
        .content()
        .clone();
    let prior_changes = changes.load(Ordering::SeqCst);
    let converted = socket(
        &mut client,
        json!({"command":ACYCLIC,"params":acyclic_params(1,&input,&output)}),
    )?;
    assert!(converted.get("error").is_none(), "{converted}");
    assert_eq!(converted["result"]["conversion"]["input_revision"], 1);
    assert_eq!(
        inspect_without_conversion(converted["result"].clone())?,
        before
    );
    assert_eq!(
        changes.load(Ordering::SeqCst),
        prior_changes,
        "conversion must not request a repaint"
    );
    consume_acyclic(
        &output,
        "CC(C)C",
        json!([[6, 3], [6, 1], [6, 3], [6, 3]]),
        json!([[1, 2, 1], [2, 3, 1], [2, 4, 1]]),
        "C4H10",
    )?;
    let stale = directory.path("stale.cdxml");
    let refused = socket(
        &mut client,
        json!({"command":ACYCLIC,"params":acyclic_params(0,&input,&stale)}),
    )?;
    assert_eq!(refused["error"]["code"], "stale_revision");
    assert!(!stale.try_exists()?);
    let held = engine.lock().map_err(|_| "engine poisoned")?.prepare(
        bondline_commands::Command::parse(
            "document.save_cdxml",
            json!({"path":directory.path("held.cdxml")}),
        )?,
    )?;
    let blocked = directory.path("blocked.cdxml");
    let refused = socket(
        &mut client,
        json!({"command":ACYCLIC,"params":acyclic_params(1,&input,&blocked)}),
    )?;
    assert_eq!(refused["error"]["code"], "busy");
    assert!(!blocked.try_exists()?);
    assert!(!directory.path("held.cdxml").try_exists()?);
    drop(held);
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?["result"],
        before
    );
    assert_eq!(
        engine.lock().map_err(|_| "engine poisoned")?.content(),
        &content
    );
    let conflict = socket(
        &mut client,
        json!({"command":ACYCLIC,"params":acyclic_params(1,&input,&output)}),
    )?;
    assert_eq!(conflict["error"]["code"], "already_exists");
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?["result"],
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), prior_changes);
    assert_eq!(fs::read(input)?, b"CC(C)C\n");
    drop(client);
    server.shutdown()?;
    Ok(())
}
