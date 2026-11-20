//! Frozen creation workflows through the actual CLI, initialized MCP and loopback.
use super::{Client, cli, mixed_label_wire::captures, socket, support};
use crate::registry_fixture;
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    net::TcpStream,
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
#[path = "../../../../xtask/oracles/created_label_support.rs"]
#[allow(
    dead_code,
    reason = "shared immutable source/model/actual-file vectors"
)]
mod f;
type Result<T = ()> = f::Result<T>;
const CREATE: &str = "atom.create_authored_label";
const EDIT: &str = "atom.set_authored_label";
fn request(name: &str, params: Value) -> Value {
    json!({"command":name,"params":params})
}
fn capture(backend: &str) -> Value {
    json!({"page_id":1,"width_px":1088,"height_px":1088,"backend":backend})
}
fn revision(v: &Value) -> Result<u64> {
    v["revision"].as_u64().ok_or_else(|| "revision".into())
}
fn consume(path: &std::path::Path, name: &str) -> Result {
    let raw = fs::read(path)?;
    assert_eq!(raw, f::source(name).as_bytes());
    assert_eq!(f::read(&raw)?, f::facts(name));
    Ok(())
}
fn schema(v: &Value) {
    assert_eq!(v["type"], "object");
    assert_eq!(v["additionalProperties"], false);
    assert_eq!(
        v["required"],
        json!([
            "expected_revision",
            "page_id",
            "fragment_id",
            "atom_id",
            "element",
            "explicit_hydrogens",
            "style"
        ])
    );
    let style = &v["$defs"]["AtomLabelStyle"];
    assert_eq!(style["type"], "object");
    assert_eq!(style["additionalProperties"], false);
    assert_eq!(
        style["required"],
        json!(["font_family", "size_pt", "face", "rgb"])
    );
}
fn discovery(rows: &[Value], mcp: bool) -> Result {
    let old: Vec<Value> =
        serde_json::from_str(&registry_fixture::with_cdx_caption_edit(f::METADATA48))?;
    assert_eq!((old.len(), rows.len()), (48, 53));
    for (row, expected) in rows.iter().take(48).zip(&old) {
        if mcp {
            assert_eq!(row["name"], expected["name"]);
            assert_eq!(row["description"], expected["description"]);
            assert_eq!(row["inputSchema"], expected["input_schema"]);
            assert_eq!(row["annotations"]["readOnlyHint"], expected["read_only"]);
        } else {
            assert_eq!(row, expected);
        }
    }
    assert_eq!(rows[48]["name"], CREATE);
    if mcp {
        assert_eq!(rows[48]["annotations"]["readOnlyHint"], false);
        schema(&rows[48]["inputSchema"]);
    } else {
        assert_eq!(rows[48]["read_only"], false);
        schema(&rows[48]["input_schema"]);
    }
    Ok(())
}
fn state(
    v: &Value,
    baseline: &Value,
    name: &str,
    depths: [u64; 2],
    flags: [bool; 2],
    original_len: usize,
) -> Result {
    assert_eq!(v["document"], f::model_for_case(baseline, name)?);
    assert_eq!(v["undo_depth"], depths[0]);
    assert_eq!(v["redo_depth"], depths[1]);
    assert_eq!(v["file"]["dirty"], flags[0]);
    assert_eq!(v["file"]["format"], "cdxml");
    assert_eq!(
        v["source"],
        json!({"format":"cdxml","byte_length":original_len,"modified":flags[1]})
    );
    Ok(())
}

#[test]
fn actual_cli_creation_keeps_discovery_prefix_and_explicit_style_value_dense_id_files() -> Result {
    let listing = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listing.status.success(),
        "{}",
        String::from_utf8_lossy(&listing.stderr)
    );
    let rows: Value = serde_json::from_slice(&listing.stdout)?;
    discovery(rows.as_array().ok_or("command rows")?, false)?;
    let expected = registry_fixture::with_cdx_caption_edit(f::METADATA48);
    let prefix = expected.trim().strip_suffix(']').ok_or("old array")?;
    assert!(
        listing.stdout.starts_with(prefix.as_bytes()),
        "actual CLI serialized old48 records must remain byte-exact"
    );
    assert_eq!(listing.stdout.get(prefix.len()), Some(&b','));
    let directory = support::SdfTestDirectory::new()?;
    for (base, after, z, h, appearance, second) in [
        ("base", "created_3_nh", 7, 1, f::black_style(), false),
        ("base", "created_3_blue14", 6, 2, f::blue_style(), false),
        (
            "dense_base",
            "dense_created_3_2",
            6,
            2,
            f::black_style(),
            true,
        ),
    ] {
        let input = directory.path(&format!("created-cli-{after}-原-λ.cdxml"));
        let output = directory.path(&format!("created-cli-{after}-保存-λ.cdxml"));
        fs::write(&input, f::source(base))?;
        let open = request("document.open_cdxml", json!({"path":input}));
        let before = cli(std::slice::from_ref(&open))?;
        let mut edits = vec![open, request(CREATE, f::params(1, 3, z, h, appearance))];
        let depth = if second {
            edits.push(request(CREATE, f::params(2, 2, 6, 2, f::black_style())));
            2
        } else {
            1
        };
        let mut cpu_calls = edits.clone();
        cpu_calls.push(request("render.capture", capture("cpu")));
        let mut gpu_calls = edits.clone();
        gpu_calls.push(request("render.capture", capture("gpu")));
        let cpu = cli(&cpu_calls)?;
        let gpu = cli(&gpu_calls)?;
        state(
            &cpu,
            &before["document"],
            after,
            [depth, 0],
            [true, true],
            f::source(base).len(),
        )?;
        state(
            &gpu,
            &before["document"],
            after,
            [depth, 0],
            [true, true],
            f::source(base).len(),
        )?;
        assert_eq!(revision(&cpu)?, depth + 1);
        captures(&cpu, &gpu)?;
        edits.push(request("document.save_cdxml", json!({"path":output})));
        edits.push(request("document.open_cdxml", json!({"path":output})));
        edits.push(request("document.inspect", json!({})));
        let reopened = cli(&edits)?;
        state(
            &reopened,
            &before["document"],
            after,
            [0, 0],
            [false, false],
            f::source(after).len(),
        )?;
        assert_eq!(revision(&reopened)?, depth + 2);
        assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
        consume(&output, after)?;
        consume(&input, base)?;
    }
    Ok(())
}

fn workflow(mut call: impl FnMut(&str, Value) -> Result<Value>, tag: &str) -> Result {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path(&format!("created-{tag}-原-λ.cdxml"));
    let partial = directory.path(&format!("created-{tag}-partial-保存.cdxml"));
    let output = directory.path(&format!("created-{tag}-all-保存-λ.cdxml"));
    fs::write(&input, f::source("base"))?;
    let opened = call("document.open_cdxml", json!({"path":input}))?;
    let baseline = opened["document"].clone();
    let original_len = f::source("base").len();
    state(
        &opened,
        &baseline,
        "base",
        [0, 0],
        [false, false],
        original_len,
    )?;
    let created = call(
        CREATE,
        f::params(revision(&opened)?, 3, 6, 2, f::black_style()),
    )?;
    state(
        &created,
        &baseline,
        "created_3_black",
        [1, 0],
        [true, true],
        original_len,
    )?;
    assert!(created["changed"].as_bool().ok_or("creation changed")?);
    let changed = call(EDIT, f::label_params(revision(&created)?, 3, 7, 1))?;
    state(
        &changed,
        &baseline,
        "created_3_old48_nh",
        [2, 0],
        [true, true],
        original_len,
    )?;
    let undo = call("history.undo", json!({}))?;
    state(
        &undo,
        &baseline,
        "created_3_black",
        [1, 1],
        [true, true],
        original_len,
    )?;
    let before_noop = call("document.inspect", json!({}))?;
    let noop = call(EDIT, f::label_params(revision(&undo)?, 3, 6, 2))?;
    assert_eq!(
        noop, before_noop,
        "same created CH2 must keep flags, redo and every persistent owner"
    );
    let redo = call("history.redo", json!({}))?;
    state(
        &redo,
        &baseline,
        "created_3_old48_nh",
        [2, 0],
        [true, true],
        original_len,
    )?;
    let undo = call("history.undo", json!({}))?;
    state(
        &undo,
        &baseline,
        "created_3_black",
        [1, 1],
        [true, true],
        original_len,
    )?;
    let saved_partial = call("document.save_cdxml", json!({"path":partial}))?;
    state(
        &saved_partial,
        &baseline,
        "created_3_black",
        [1, 1],
        [false, true],
        original_len,
    )?;
    assert_eq!(revision(&saved_partial)?, revision(&undo)?);
    consume(&partial, "created_3_black")?;
    let second = call(
        CREATE,
        f::params(revision(&saved_partial)?, 2, 6, 2, f::black_style()),
    )?;
    state(
        &second,
        &baseline,
        "created_3_2_black",
        [2, 0],
        [true, true],
        original_len,
    )?;
    let all = call(
        CREATE,
        f::params(revision(&second)?, 4, 6, 2, f::black_style()),
    )?;
    state(
        &all,
        &baseline,
        "created_all_black",
        [3, 0],
        [true, true],
        original_len,
    )?;
    let selected = call(
        "ui.selection",
        json!({"action":"all","expected_revision":revision(&all)?,"page_id":1}),
    )?;
    assert_eq!(selected["changed"], false);
    assert_eq!(selected["document"], all["document"]);
    assert_eq!(selected["ui_selection"]["atom_ids"], json!([1, 2, 3, 4, 5]));
    let epoch = selected["ui_selection"]["epoch"]
        .as_u64()
        .ok_or("selection epoch")?;
    let moved = call(
        "selection.translate",
        f::motion(revision(&selected)?, epoch, 24., -12.),
    )?;
    state(
        &moved,
        &baseline,
        "created_all_p24_m12",
        [4, 0],
        [true, true],
        original_len,
    )?;
    let before_capture = call("document.inspect", json!({}))?;
    let cpu = call("render.capture", capture("cpu"))?;
    let gpu = call("render.capture", capture("gpu"))?;
    captures(&cpu, &gpu)?;
    assert_eq!(call("document.inspect", json!({}))?, before_capture);
    let saved = call("document.save_cdxml", json!({"path":output}))?;
    state(
        &saved,
        &baseline,
        "created_all_p24_m12",
        [4, 0],
        [false, true],
        original_len,
    )?;
    assert_eq!(revision(&saved)?, revision(&moved)?);
    consume(&output, "created_all_p24_m12")?;
    let reopened = call("document.open_cdxml", json!({"path":output}))?;
    state(
        &reopened,
        &baseline,
        "created_all_p24_m12",
        [0, 0],
        [false, false],
        f::source("created_all_p24_m12").len(),
    )?;
    assert_eq!(revision(&reopened)?, revision(&moved)? + 1);
    assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
    consume(&input, "base")?;
    consume(&partial, "created_3_black")?;
    Ok(())
}

#[test]
fn initialized_stdio_creation_edit_noop_history_all_labels_move_and_exact_unicode_files() -> Result
{
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"created-label-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    discovery(
        listed["result"]["tools"].as_array().ok_or("MCP tools")?,
        true,
    )?;
    let mut id = 2;
    workflow(
        |name, p| {
            id += 1;
            client.call(id, name, p)
        },
        "stdio",
    )?;
    client.finish()
}

#[test]
fn authenticated_socket_creation_edit_noop_history_all_labels_move_and_exact_unicode_files()
-> Result {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let server = LoopbackServer::bind(0, Arc::clone(&engine), || {})?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    workflow(
        |name, p| {
            let v = socket(&mut client, request(name, p))?;
            assert!(v.get("error").is_none(), "{v}");
            Ok(v["result"].clone())
        },
        "socket",
    )?;
    drop(client);
    server.shutdown()?;
    Ok(())
}
