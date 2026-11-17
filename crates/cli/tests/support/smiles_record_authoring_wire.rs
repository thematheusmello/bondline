//! Independent Gate B workflows over the unchanged adapter host helpers.
use super::*;

const NAME: &str = "document.import_smiles_records";
const SMILES_SOURCE: &[u8] = b"CC first\r\nC1.N.C1 \t\0\xffPRIVATE-META\r\nCC duplicate\tname";
const SMILES_EDITED: &[u8] =
    b"CC first\r\n[CH2]=[CH2].[NH3] \t\0\xffPRIVATE-META\r\nCC duplicate\tname";
type AnyResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn import(revision: u64, selection: Value) -> Value {
    json!({"expected_revision":revision,"selection":selection,"bond_length_pt":48.0,
        "origin_x_pt":72.0,"origin_y_pt":72.0,"page_width_pt":210.0*72.0/25.4,
        "page_height_pt":297.0*72.0/25.4})
}
fn edit(revision: u64, fragment: u32, bond: u32) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":fragment,
        "bond_id":bond,"order":"double","hydrogen_policy":"rectify_neutral_carbon"})
}
fn consumer(script: &str, args: &[String], bytes: &[u8]) -> AnyResult<Value> {
    let mut child = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .arg("-B")
        .arg(format!(
            "{}/../io/tests/{script}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn consume_saved(path: &std::path::Path, reimported: &Value) -> TestResult {
    let bytes = fs::read(path)?;
    assert_eq!(bytes, SMILES_EDITED);
    let raw = consumer("smiles_file_raw_oracle.py", &[], &bytes)?;
    assert_eq!(raw["byte_length"], 62);
    assert_eq!(raw["physical_lines"], 3);
    assert_eq!(raw["records"][1]["notation_range"], json!([10, 27]));
    assert_eq!(
        raw["records"][1]["application_data_hex"],
        "00ff505249564154452d4d455441"
    );
    assert_eq!(raw["records"][2]["notation_range"], json!([45, 47]));
    assert_eq!(raw["records"][2]["line_ending_range"], json!([62, 62]));
    for (range, z, orders) in [
        (0..2, vec![6, 6], json!([[0, 1, "SINGLE"]])),
        (10..27, vec![6, 6, 7], json!([[0, 1, "DOUBLE"]])),
        (45..47, vec![6, 6], json!([[0, 1, "SINGLE"]])),
    ] {
        let graph = consumer(
            "smiles_oracle.py",
            &[],
            bytes.get(range).ok_or("literal notation span")?,
        )?;
        assert_eq!(graph["accepted"], true);
        assert_eq!(
            graph["atoms"]
                .as_array()
                .ok_or("atoms")?
                .iter()
                .map(|a| a["element"].as_u64().ok_or("element"))
                .collect::<Result<Vec<_>, _>>()?,
            z
        );
        assert_eq!(graph["bonds"], orders);
    }
    let options = json!({"atoms":[[6,null,2],[6,null,2],[7,null,3]],"bonds":[[1,2,2]],
        "components":[[1,2],[3]],"formula":"C2H7N"});
    let checked = consumer(
        "smiles_authoring_oracle.py",
        &[
            String::from_utf8(bytes[10..27].to_vec())?,
            options.to_string(),
        ],
        &serde_json::to_vec(&reimported["document"]["pages"][0]["fragments"][1])?,
    )?;
    assert_eq!(checked["no_assigned_stereo"], true);
    assert_eq!(checked["no_radicals"], true);
    Ok(())
}

#[test]
fn cli_imports_all_records_edits_saves_and_explicitly_reimports_the_whole_source() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("records-cli-原.smi");
    let output = dir.path("records-cli-化学.data");
    fs::write(&input, SMILES_SOURCE)?;
    // First baseline execution must reach real dynamic dispatch and fail unknown_command.
    let imported = cli(&[
        json!({"command":"document.open_smiles","params":{"path":input}}),
        json!({"command":NAME,"params":import(1,json!({"mode":"all"}))}),
    ])?;
    assert_eq!(imported["revision"], 2);
    assert_eq!(
        imported["smiles_file"]["projected_records"],
        json!([1, 2, 3])
    );
    assert_eq!(imported["file"]["format"], "smiles");
    assert_eq!(imported["file"]["dirty"], false);
    assert_eq!(imported["undo_depth"], 0);
    assert_eq!(
        imported["document"]["pages"][0]["fragments"][1]["bonds"][0]["begin"],
        3
    );
    assert_eq!(
        imported["document"]["pages"][0]["fragments"][1]["bonds"][0]["end"],
        5
    );
    let final_state = cli(&[
        json!({"command":"document.open_smiles","params":{"path":input}}),
        json!({"command":NAME,"params":import(1,json!({"mode":"all"}))}),
        json!({"command":"bond.set_order","params":edit(2,2,2)}),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_smiles","params":{"path":output}}),
        json!({"command":"document.open_smiles","params":{"path":output}}),
        json!({"command":NAME,"params":import(6,json!({"mode":"all"}))}),
    ])?;
    assert_eq!(final_state["revision"], 7);
    assert_eq!(final_state["undo_depth"], 0);
    assert_eq!(final_state["redo_depth"], 0);
    assert_eq!(final_state["file"]["dirty"], false);
    assert_eq!(
        final_state["source"],
        json!({"format":"smiles","byte_length":62,"modified":false})
    );
    assert_eq!(
        final_state["file"]["path"],
        json!(fs::canonicalize(&output)?)
    );
    assert!(final_state.get("file_operation").is_none());
    assert_eq!(fs::read(&input)?, SMILES_SOURCE);
    consume_saved(&output, &final_state)?;
    Ok(())
}

// Initialized stdio and authenticated loopback follow the frozen5,133-byte CLI prefix.
fn readonly(value: &Value, length: usize) {
    assert_eq!(value["document"], Value::Null);
    assert_eq!(
        value["smiles_file"],
        json!({"format":"smiles","byte_length":length,"record_count":3})
    );
    assert_eq!(
        value["source"],
        json!({"format":"smiles","byte_length":length,"modified":false})
    );
    assert_eq!(value["file"]["format"], "smiles");
    assert_eq!(value["file"]["dirty"], false);
}
fn backed(value: &Value, revision: u64, selected: Value, modified: bool) {
    assert_eq!(value["revision"], revision);
    assert!(value["document"].is_object());
    assert_eq!(
        value["smiles_file"],
        json!({"format":"smiles","byte_length":if modified {62} else {52},"record_count":3,"projected_records":selected})
    );
    assert_eq!(
        value["source"],
        json!({"format":"smiles","byte_length":52,"modified":modified})
    );
    assert_eq!(value["file"]["format"], "smiles");
    assert!(value.get("file_operation").is_none());
}
fn source_order(value: &Value, at: usize, ids: [u32; 3], bond: u32, edited: bool) -> TestResult {
    let fragment = &value["document"]["pages"][0]["fragments"][at];
    let h = if edited { 2 } else { 3 };
    assert_eq!(
        fragment["atoms"]
            .as_array()
            .ok_or("atoms")?
            .iter()
            .map(|a| json!([a["id"], a["element"], a["explicit_hydrogens"]]))
            .collect::<Vec<_>>(),
        vec![
            json!([ids[0], 6, h]),
            json!([ids[1], 7, 3]),
            json!([ids[2], 6, h])
        ]
    );
    assert_eq!(
        fragment["bonds"],
        json!([{"id":bond,"begin":ids[0],"end":ids[2],"order":if edited {"double"} else {"single"}}])
    );
    Ok(())
}
fn initialize(client: &mut Client) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"smiles-record-authoring","version":"1"}}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], 1);
    assert!(reply.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
}
fn rejected(client: &mut Client, id: u64, name: &str, args: Value, code: &str) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":args}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], id);
    assert_eq!(reply["result"]["isError"], true);
    assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    Ok(())
}

#[test]
fn initialized_stdio_discovers_closed_import_and_saves_only_the_selected_record_edit() -> TestResult
{
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("records-stdio-原.smi");
    let output = dir.path("records-stdio-化学.data");
    fs::write(&input, SMILES_SOURCE)?;
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 41);
    let names = tools
        .iter()
        .map(|tool| tool["name"].as_str().ok_or("name"))
        .collect::<Result<Vec<_>, _>>()?;
    let unique = names
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(unique.len(), 41);
    for prior in [
        "document.new",
        "document.open_smiles",
        "document.save_smiles",
        "smiles.record.inspect",
        "structure.paste_smiles",
        "bond.create",
        "bond.extend",
        "bond.set_order",
        "sdf.field.set_value",
        "sdf.record.remove",
        "file.convert_smiles_to_cdxml",
        "file.export_smiles_fragment",
    ] {
        assert!(unique.contains(prior));
    }
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == NAME)
        .ok_or("import tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    let mut required = tool["inputSchema"]["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or("field"))
        .collect::<Result<Vec<_>, _>>()?;
    required.sort_unstable();
    assert_eq!(
        required,
        [
            "bond_length_pt",
            "expected_revision",
            "origin_x_pt",
            "origin_y_pt",
            "page_height_pt",
            "page_width_pt",
            "selection"
        ]
    );
    let opened = client.call(3, "document.open_smiles", json!({"path":input}))?;
    readonly(&opened, 52);
    let imported = client.call(4, NAME, import(1, json!({"mode":"record","record_id":2})))?;
    backed(&imported, 2, json!([2]), false);
    assert_eq!(imported["file"]["path"], json!(fs::canonicalize(&input)?));
    assert_eq!(imported["undo_depth"], 0);
    assert_eq!(imported["redo_depth"], 0);
    assert_eq!(imported["file"]["dirty"], false);
    assert_eq!(
        imported["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        1
    );
    source_order(&imported, 0, [1, 2, 3], 1, false)?;
    let before = client.call(5, "document.inspect", json!({}))?;
    rejected(
        &mut client,
        6,
        NAME,
        import(1, json!({"mode":"all"})),
        "stale_revision",
    )?;
    rejected(
        &mut client,
        7,
        NAME,
        import(2, json!({"mode":"all","record_id":2})),
        "invalid_params",
    )?;
    assert_eq!(client.call(8, "document.inspect", json!({}))?, before);
    let edited = client.call(9, "bond.set_order", edit(2, 1, 1))?;
    backed(&edited, 3, json!([2]), true);
    source_order(&edited, 0, [1, 2, 3], 1, true)?;
    assert_eq!(edited["undo_depth"], 1);
    assert_eq!(edited["redo_depth"], 0);
    assert_eq!(edited["file"]["dirty"], true);
    let saved = client.call(10, "document.save_smiles", json!({"path":output}))?;
    assert_eq!(saved["file_operation"]["byte_length"], 62);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["source"]["modified"], true);
    let undone = client.call(11, "history.undo", json!({}))?;
    backed(&undone, 4, json!([2]), false);
    source_order(&undone, 0, [1, 2, 3], 1, false)?;
    assert_eq!(undone["file"]["dirty"], true);
    assert_eq!(undone["redo_depth"], 1);
    let redone = client.call(12, "history.redo", json!({}))?;
    backed(&redone, 5, json!([2]), true);
    source_order(&redone, 0, [1, 2, 3], 1, true)?;
    assert_eq!(redone["file"]["dirty"], false);
    let reopened = client.call(13, "document.open_smiles", json!({"path":output}))?;
    readonly(&reopened, 62);
    assert_eq!(reopened["revision"], 6);
    let all = client.call(14, NAME, import(6, json!({"mode":"all"})))?;
    assert_eq!(all["smiles_file"]["projected_records"], json!([1, 2, 3]));
    assert_eq!(all["revision"], 7);
    assert_eq!(all["undo_depth"], 0);
    assert_eq!(fs::read(&input)?, SMILES_SOURCE);
    consume_saved(&output, &all)?;
    client.finish()
}

fn call_socket(client: &mut BufReader<TcpStream>, name: &str, params: Value) -> AnyResult<Value> {
    let reply = socket(client, json!({"command":name,"params":params}))?;
    assert!(reply.get("error").is_none(), "{reply}");
    Ok(reply["result"].clone())
}

#[test]
fn authenticated_loopback_keeps_complete_source_through_all_edit_history_and_later_refusal()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("records-loopback-原.smi");
    let output = dir.path("records-loopback-化学.data");
    let unsupported = dir.path("records-later-unsupported.smi");
    let retained = dir.path("records-unselected-copy.data");
    const LATER: &[u8] = b"CC selected\r\ns1cccc1 unsupported\t\0\xff";
    fs::write(&input, SMILES_SOURCE)?;
    fs::write(&unsupported, LATER)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let notified = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        notified.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    // Credential goes only to this live trusted socket, never diagnostic output.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let opened = call_socket(&mut client, "document.open_smiles", json!({"path":input}))?;
    readonly(&opened, 52);
    let imported = call_socket(&mut client, NAME, import(1, json!({"mode":"all"})))?;
    backed(&imported, 2, json!([1, 2, 3]), false);
    source_order(&imported, 1, [3, 4, 5], 2, false)?;
    let before = call_socket(&mut client, "document.inspect", json!({}))?;
    let count = changes.load(Ordering::SeqCst);
    let stale = socket(
        &mut client,
        json!({"command":"bond.set_order","params":edit(1,2,2)}),
    )?;
    assert_eq!(stale["error"]["code"], "stale_revision");
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), count);
    let edited = call_socket(&mut client, "bond.set_order", edit(2, 2, 2))?;
    backed(&edited, 3, json!([1, 2, 3]), true);
    source_order(&edited, 1, [3, 4, 5], 2, true)?;
    let undone = call_socket(&mut client, "history.undo", json!({}))?;
    backed(&undone, 4, json!([1, 2, 3]), false);
    assert_eq!(undone["file"]["dirty"], false);
    let redone = call_socket(&mut client, "history.redo", json!({}))?;
    backed(&redone, 5, json!([1, 2, 3]), true);
    assert_eq!(redone["file"]["dirty"], true);
    let saved = call_socket(&mut client, "document.save_smiles", json!({"path":output}))?;
    assert_eq!(saved["file_operation"]["byte_length"], 62);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["source"]["modified"], true);
    readonly(
        &call_socket(&mut client, "document.open_smiles", json!({"path":output}))?,
        62,
    );
    let all = call_socket(&mut client, NAME, import(6, json!({"mode":"all"})))?;
    assert_eq!(all["revision"], 7);
    consume_saved(&output, &all)?;
    let later = call_socket(
        &mut client,
        "document.open_smiles",
        json!({"path":unsupported}),
    )?;
    assert_eq!(later["revision"], 8);
    let before = call_socket(&mut client, "document.inspect", json!({}))?;
    let count = changes.load(Ordering::SeqCst);
    let failure = socket(
        &mut client,
        json!({"command":NAME,"params":import(8,json!({"mode":"all"}))}),
    )?;
    assert_eq!(failure["error"]["code"], "unsupported_content");
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), count);
    let selected = call_socket(
        &mut client,
        NAME,
        import(8, json!({"mode":"record","record_id":1})),
    )?;
    assert_eq!(selected["smiles_file"]["projected_records"], json!([1]));
    assert_eq!(
        selected["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        1
    );
    call_socket(
        &mut client,
        "document.save_smiles",
        json!({"path":retained}),
    )?;
    assert_eq!(fs::read(&retained)?, LATER);
    assert_eq!(fs::read(&unsupported)?, LATER);
    assert_eq!(fs::read(&input)?, SMILES_SOURCE);
    server.shutdown()?;
    Ok(())
}
