//! Independent post-producer adapter acceptance; no native form/default claim.
#[path = "document_settings_wire.rs"]
mod document_settings;

use super::{
    Client, OLD_NAMES, PROPANE, TestResult, cli, consume, graph, initialize, original_propane,
    python, request, socket, support,
};
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    io::{BufReader, Write as _},
    net::TcpStream,
    path::Path,
    process::{Command as Process, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

const SET: &str = "document.set_drawing_settings";
fn set(revision: u64, length: Value, angle: Value) -> Value {
    json!({"expected_revision":revision,"bond_length_pt":length,"chain_angle_degrees":angle})
}
// Original parent propane graph/page with independently chosen root values.
// No product encoder or downloaded example supplies these bytes.
fn native(length: Option<&str>, angle: Option<&str>) -> TestResult<Vec<u8>> {
    let mut root = "CreationProgram=\"Bondline 0.1.0\"".to_owned();
    if let Some(value) = length {
        root.push_str(&format!(" BondLength=\"{value}\""));
    }
    if let Some(value) = angle {
        root.push_str(&format!(" ChainAngle=\"{value}\""));
    }
    Ok(std::str::from_utf8(PROPANE)?
        .replace("CreationProgram=\"Bondline 0.1.0\"", &root)
        .into_bytes())
}
fn propene() -> Value {
    let mut graph = original_propane();
    graph["atoms"][0]["explicit_hydrogens"] = json!(2);
    graph["atoms"][1]["explicit_hydrogens"] = json!(1);
    graph["bonds"][0]["order"] = json!("double");
    graph
}
fn modeled(state: &Value, expected: Value) {
    if expected.is_null() {
        assert!(state["document"].get("drawing_settings").is_none());
    } else {
        assert_eq!(state["document"]["drawing_settings"], expected);
    }
}
fn root_then_chemistry(
    path: &Path,
    length: Option<&str>,
    angle: Option<&str>,
    expected: &Value,
    smiles: &str,
    formula: &str,
) -> TestResult {
    let mut child = Process::new(python())
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../io/tests/drawing_settings_root.py"
            ),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("root reader stdin")?
        .write_all(&fs::read(path)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: Value = serde_json::from_slice(&output.stdout)?;
    let mut attributes = vec![json!(["CreationProgram", "Bondline 0.1.0"])];
    let mut numbers = json!({});
    if let Some(value) = length {
        attributes.push(json!(["BondLength", value]));
        numbers["BondLength"] = json!(value);
    }
    if let Some(value) = angle {
        attributes.push(json!(["ChainAngle", value]));
        numbers["ChainAngle"] = json!(value);
    }
    assert_eq!(observed, json!({"root":attributes,"decimal":numbers}));
    // Existing parent's raw both-flag/directed integer/Decimal checker then its
    // unchanged independent integer+sanitized chemistry consumer.
    consume(path, expected, smiles, formula)
}

#[test]
fn actual_cli_styled_native_edit_history_unicode_save_and_registered_mol_loss_refusal() -> TestResult
{
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("样式-propane-λ.cdxml");
    let output = dir.path("propene-样式-保存.cdxml");
    let source = native(Some("48.5"), Some("137.5"))?;
    fs::write(&input, &source)?;
    let open = request("document.open_cdxml", json!({"path":input}));
    let edit = request(
        "bond.set_order",
        json!({"expected_revision":1,"page_id":1,"fragment_id":1,
        "bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
    );
    let replace = request(SET, set(2, json!(64.25), Value::Null));
    let changed = cli(&[open.clone(), edit.clone(), replace.clone()])?;
    graph(&changed, &propene())?;
    modeled(&changed, json!({"bond_length_pt":64.25}));
    assert_eq!(changed["revision"], 3);
    assert_eq!(changed["undo_depth"], 2);
    assert_eq!(changed["file"]["dirty"], true);
    assert!(changed.get("file_operation").is_none() && changed.get("ui_file_operation").is_none());
    let undone = cli(&[
        open.clone(),
        edit.clone(),
        replace.clone(),
        request("history.undo", json!({})),
    ])?;
    graph(&undone, &propene())?;
    modeled(
        &undone,
        json!({"bond_length_pt":48.5,"chain_angle_degrees":137.5}),
    );
    assert_eq!(undone["undo_depth"], 1);
    assert_eq!(undone["redo_depth"], 1);
    let saved = cli(&[
        open,
        edit,
        replace,
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request("document.save_cdxml", json!({"path":output})),
        request("document.inspect", json!({})),
    ])?;
    graph(&saved, &propene())?;
    modeled(&saved, json!({"bond_length_pt":64.25}));
    assert_eq!(saved["revision"], 5);
    assert_eq!(saved["undo_depth"], 2);
    assert_eq!(saved["redo_depth"], 0);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(
        saved["source"],
        json!({"format":"cdxml","byte_length":source.len(),"modified":true})
    );
    root_then_chemistry(&output, Some("64.25"), None, &propene(), "C=CC", "C3H6")?;
    let reopened = cli(&[request("document.open_cdxml", json!({"path":output}))])?;
    graph(&reopened, &propene())?;
    modeled(&reopened, json!({"bond_length_pt":64.25}));
    assert_eq!(reopened["source"]["modified"], false);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(fs::read(&input)?, source);

    // The original source-free bond command has A4 paper and no label flags.
    // Its registered success prevents unrelated metadata/page guards masking loss.
    let create = request(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":96,
        "end_x_pt":120,"end_y_pt":96}),
    );
    let styled = cli(&[
        create.clone(),
        request(SET, set(1, json!(24.5), json!(120.5))),
    ])?;
    modeled(
        &styled,
        json!({"bond_length_pt":24.5,"chain_angle_degrees":120.5}),
    );
    assert_eq!(styled["revision"], 2);
    for version in ["v2000", "v3000"] {
        let existing = dir.path(&format!("keep-old-{version}-λ.mol"));
        let absent = dir.path(&format!("absent-{version}-样式.mol"));
        let plain = cli(&[
            create.clone(),
            request(
                "document.save_mol",
                json!({"path":existing,"version":version}),
            ),
        ])?;
        modeled(&plain, Value::Null);
        assert_eq!(plain["file"]["dirty"], false);
        assert_eq!(plain["file"]["format"], format!("mol_{version}"));
        let atoms = plain["document"]["pages"][0]["fragments"][0]["atoms"]
            .as_array()
            .ok_or("control atoms")?;
        assert_eq!(atoms.len(), 2);
        for atom in atoms {
            assert_eq!(atom["explicit_hydrogens"], 3);
            assert!(atom.get("show_terminal_carbon_labels").is_none());
            assert!(atom.get("show_non_terminal_carbon_labels").is_none());
        }
        let old_mol = fs::read(&existing)?;
        assert!(!old_mol.is_empty());
        for path in [&existing, &absent] {
            let mut command = Process::new(env!("CARGO_BIN_EXE_bondline-cli"));
            command.arg("run");
            for value in [
                create.clone(),
                request(SET, set(1, json!(24.5), json!(120.5))),
                request(
                    "document.save_mol",
                    json!({"path":path,"version":version,"overwrite":true}),
                ),
            ] {
                command.arg("--cmd").arg(value.to_string());
            }
            let result = command.output()?;
            assert!(!result.status.success());
            assert!(String::from_utf8_lossy(&result.stderr).contains("unsupported_content"));
            assert!(!String::from_utf8_lossy(&result.stdout).contains("\"file_operation\""));
            assert_eq!(fs::read(&existing)?, old_mol);
            assert!(!absent.exists());
        }
        let cleared = cli(&[
            create.clone(),
            request(SET, set(1, json!(24.5), json!(120.5))),
            request(SET, set(2, Value::Null, Value::Null)),
            request(
                "document.save_mol",
                json!({"path":absent,"version":version}),
            ),
        ])?;
        modeled(&cleared, Value::Null);
        assert_eq!(cleared["revision"], 3);
        assert_eq!(cleared["file"]["dirty"], false);
        assert!(absent.exists());
        assert_eq!(fs::read(&existing)?, old_mol);
    }
    Ok(())
}

fn rejected(client: &mut Client, id: u64, parameters: Value, code: &str) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":SET,"arguments":parameters}}))?;
    let response = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(response["id"], id);
    assert_eq!(response["result"]["isError"], true);
    assert_eq!(
        response["result"]["structuredContent"]["error"]["code"],
        code
    );
    assert!(
        response["result"]["structuredContent"]
            .get("file_operation")
            .is_none()
    );
    Ok(())
}

#[test]
fn initialized_stdio_schema_required_nulls_noops_source_and_whole_history() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("stdio-styled-原.cdxml");
    let output = dir.path("stdio-cleared-保存.cdxml");
    let smiles = dir.path("whole-原.smi");
    let source = native(Some("48.5"), Some("137.5"))?;
    fs::write(&input, &source)?;
    fs::write(&smiles, b"CC original\r\nO opaque\0\xff")?;
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let response = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(response["id"], 2);
    let tools = response["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 50);
    assert_eq!(
        tools
            .iter()
            .map(|t| t["name"].as_str().ok_or("name"))
            .collect::<Result<BTreeSet<_>, _>>()?,
        OLD_NAMES
            .into_iter()
            .chain([
                super::NAME,
                "chain.create_path",
                SET,
                "ui.drawing_settings",
                "ui.chain_tool",
                "ring.sprout",
                "ui.preferences",
                "document.set_document_settings",
                "text.set_run",
                "atom.set_authored_label",
                "atom.create_authored_label",
                "ui.caption",
            ])
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        tools.get(40).ok_or("legacy settings position")?["name"],
        SET
    );
    let entry = tools
        .iter()
        .find(|v| v["name"] == SET)
        .ok_or("new command")?;
    assert_eq!(entry["annotations"]["readOnlyHint"], false);
    let schema = &entry["inputSchema"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    let keys = ["expected_revision", "bond_length_pt", "chain_angle_degrees"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        schema["required"]
            .as_array()
            .ok_or("required")?
            .iter()
            .map(|v| v.as_str().ok_or("key"))
            .collect::<Result<BTreeSet<_>, _>>()?,
        keys
    );
    assert_eq!(
        schema["properties"]
            .as_object()
            .ok_or("properties")?
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        keys
    );
    for field in ["bond_length_pt", "chain_angle_degrees"] {
        assert_eq!(
            schema["properties"][field]["type"]
                .as_array()
                .ok_or("nullable types")?
                .iter()
                .map(|v| v.as_str().ok_or("type"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            ["number", "null"].into_iter().collect::<BTreeSet<_>>()
        );
    }
    assert_eq!(
        schema["properties"]["bond_length_pt"]["exclusiveMinimum"].as_f64(),
        Some(0.0)
    );
    assert_eq!(
        schema["properties"]["bond_length_pt"]["maximum"].as_f64(),
        Some(100000.0)
    );
    assert_eq!(
        schema["properties"]["chain_angle_degrees"]["minimum"].as_f64(),
        Some(1.0)
    );
    assert_eq!(
        schema["properties"]["chain_angle_degrees"]["maximum"].as_f64(),
        Some(179.0)
    );
    client.call(3, "document.open_cdxml", json!({"path":input}))?;
    let before = client.call(4, "document.inspect", json!({}))?;
    graph(&before, &original_propane())?;
    let noop = client.call(5, SET, set(1, json!(48.5), json!(137.5)))?;
    assert_eq!(noop, before);
    let mut missing = set(1, Value::Null, Value::Null);
    missing
        .as_object_mut()
        .ok_or("object")?
        .remove("bond_length_pt");
    let mut extra = set(1, Value::Null, Value::Null);
    extra["unused"] = json!(true);
    for (i, (value, code)) in [
        (missing, "invalid_params"),
        (extra, "invalid_params"),
        (set(1, json!(0), Value::Null), "invalid_params"),
        (set(0, json!(0), json!(180)), "stale_revision"),
    ]
    .into_iter()
    .enumerate()
    {
        rejected(&mut client, 10 + i as u64 * 2, value, code)?;
        assert_eq!(
            client.call(11 + i as u64 * 2, "document.inspect", json!({}))?,
            before
        );
    }
    let partial = client.call(30, SET, set(1, Value::Null, json!(137.5)))?;
    modeled(&partial, json!({"chain_angle_degrees":137.5}));
    assert_eq!(partial["revision"], 2);
    let empty = client.call(31, SET, set(2, Value::Null, Value::Null))?;
    modeled(&empty, Value::Null);
    assert_eq!(empty["revision"], 3);
    let undo = client.call(32, "history.undo", json!({}))?;
    modeled(&undo, json!({"chain_angle_degrees":137.5}));
    let original = client.call(33, "history.undo", json!({}))?;
    modeled(
        &original,
        json!({"bond_length_pt":48.5,"chain_angle_degrees":137.5}),
    );
    assert_eq!(original["source"]["modified"], false);
    assert_eq!(original["file"]["dirty"], false);
    client.call(34, "history.redo", json!({}))?;
    let cleared = client.call(35, "history.redo", json!({}))?;
    modeled(&cleared, Value::Null);
    assert_eq!(cleared["revision"], 7);
    let stable = client.call(36, "document.inspect", json!({}))?;
    assert_eq!(
        client.call(37, SET, set(7, Value::Null, Value::Null))?,
        stable
    );
    client.call(38, "document.save_cdxml", json!({"path":output}))?;
    root_then_chemistry(&output, None, None, &original_propane(), "CCC", "C3H8")?;
    assert_eq!(fs::read(&output)?, native(None, None)?);
    let reopened = client.call(39, "document.open_cdxml", json!({"path":output}))?;
    modeled(&reopened, Value::Null);
    assert_eq!(reopened["source"]["modified"], false);
    assert_eq!(reopened["undo_depth"], 0);
    client.call(40, "document.open_smiles", json!({"path":smiles}))?;
    let foreign = client.call(41, "document.inspect", json!({}))?;
    rejected(
        &mut client,
        42,
        set(9, Value::Null, Value::Null),
        "unsupported_content",
    )?;
    rejected(
        &mut client,
        43,
        set(8, Value::Null, Value::Null),
        "stale_revision",
    )?;
    assert_eq!(client.call(44, "document.inspect", json!({}))?, foreign);
    assert_eq!(fs::read(&smiles)?, b"CC original\r\nO opaque\0\xff");
    assert_eq!(fs::read(&input)?, source);
    client.finish()
}

#[test]
fn authenticated_settings_publish_only_real_edits_and_save_exact_root_data() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("loopback-样式-原.cdxml");
    let output = dir.path("loopback-样式-保存.cdxml");
    let source = native(Some("48.5"), Some("137.5"))?;
    fs::write(&input, &source)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let changed = Arc::clone(&callbacks);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        changed.fetch_add(1, Ordering::SeqCst);
    })?;
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
        request("document.open_cdxml", json!({"path":input})),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    let before = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let notifications = callbacks.load(Ordering::SeqCst);
    let (whole, stamp, owner) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (
            guard.content().clone(),
            guard.capture_stamp(),
            guard.stored_document()?.model_snapshot(),
        )
    };
    let noop = socket(&mut client, request(SET, set(1, json!(48.5), json!(137.5))))?;
    assert_eq!(noop["result"], before);
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        before
    );
    assert_eq!(callbacks.load(Ordering::SeqCst), notifications);
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &whole);
        assert!(guard.capture_stamp_matches(&stamp));
        assert!(Arc::ptr_eq(
            &owner,
            &guard.stored_document()?.model_snapshot()
        ));
    }
    let edited = socket(
        &mut client,
        request(SET, set(1, Value::Null, json!(150.25))),
    )?;
    assert!(edited.get("error").is_none(), "{edited}");
    assert_eq!(edited["result"]["changed"], true);
    assert_eq!(edited["result"]["revision"], 2);
    assert_eq!(edited["result"]["undo_depth"], 1);
    modeled(&edited["result"], json!({"chain_angle_degrees":150.25}));
    assert_eq!(
        edited["result"]["document"]["pages"],
        before["document"]["pages"]
    );
    assert!(
        edited["result"].get("file_operation").is_none()
            && edited["result"].get("ui_file_operation").is_none()
    );
    let after = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    assert_eq!(callbacks.load(Ordering::SeqCst), notifications + 1);
    let stale = socket(&mut client, request(SET, set(1, Value::Null, Value::Null)))?;
    assert_eq!(stale["error"]["code"], "stale_revision");
    let noop = socket(
        &mut client,
        request(SET, set(2, Value::Null, json!(150.25))),
    )?;
    assert_eq!(noop["result"], after);
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        after
    );
    assert_eq!(callbacks.load(Ordering::SeqCst), notifications + 1);
    let saved = socket(
        &mut client,
        request("document.save_cdxml", json!({"path":output})),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    assert_eq!(saved["result"]["file"]["dirty"], false);
    assert_eq!(fs::read(&output)?, native(None, Some("150.25"))?);
    root_then_chemistry(
        &output,
        None,
        Some("150.25"),
        &original_propane(),
        "CCC",
        "C3H8",
    )?;
    assert_eq!(fs::read(&input)?, source);
    let opaque = dir.path("opaque-root-原.cdxml");
    let raw = String::from_utf8(native(None, Some("150.25"))?)?
        .replace("<CDXML ", "<CDXML Private=\"settings-opaque\" ");
    fs::write(&opaque, raw.as_bytes())?;
    socket(
        &mut client,
        request("document.open_cdxml", json!({"path":opaque})),
    )?;
    let before = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let count = callbacks.load(Ordering::SeqCst);
    let refused = socket(
        &mut client,
        request(
            SET,
            set(
                before["revision"].as_u64().ok_or("revision")?,
                Value::Null,
                json!(150.25),
            ),
        ),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        before
    );
    assert_eq!(callbacks.load(Ordering::SeqCst), count);
    let guard = engine.lock().map_err(|_| "engine")?;
    assert_eq!(guard.stored_document()?.export_cdxml()?, raw.as_bytes());
    drop(guard);
    server.shutdown()?;
    Ok(())
}
