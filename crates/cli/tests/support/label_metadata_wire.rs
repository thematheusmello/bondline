//! Explicit run workflows preserve unused scalar metadata through actual adapters.
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
#[path = "../../../../xtask/oracles/label_metadata_support.rs"]
#[allow(dead_code, reason = "shared frozen source/model/actual-file evidence")]
mod f;
type Result<T = ()> = f::Result<T>;
const EDIT: &str = "atom.set_authored_label";
const CREATE: &str = "atom.create_authored_label";

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
fn discovery(rows: &[Value], mcp: bool) -> Result {
    let expected: Vec<Value> =
        serde_json::from_str(&registry_fixture::with_cdx_caption_edit(f::METADATA49))?;
    assert_eq!((rows.len(), expected.len()), (50, 49));
    assert_eq!(rows[49]["name"], "ui.caption");
    for (row, expected) in rows.iter().zip(&expected) {
        if mcp {
            assert_eq!(row["name"], expected["name"]);
            assert_eq!(row["description"], expected["description"]);
            assert_eq!(row["inputSchema"], expected["input_schema"]);
            assert_eq!(row["annotations"]["readOnlyHint"], expected["read_only"]);
        } else {
            assert_eq!(row, expected);
        }
    }
    Ok(())
}
fn state(
    v: &Value,
    baseline: &Value,
    name: &str,
    depths: [u64; 2],
    flags: [bool; 2],
    source_len: usize,
) -> Result {
    assert_eq!(v["document"], f::model_for_case(baseline, name)?);
    assert_eq!(v["undo_depth"], depths[0]);
    assert_eq!(v["redo_depth"], depths[1]);
    assert_eq!(v["file"]["dirty"], flags[0]);
    assert_eq!(v["file"]["format"], "cdxml");
    assert_eq!(
        v["source"],
        json!({"format":"cdxml","byte_length":source_len,"modified":flags[1]})
    );
    Ok(())
}

#[test]
fn actual_cli_metadata_keeps_all49_records_and_compound_unicode_file_bytes() -> Result {
    let listing = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listing.status.success(),
        "{}",
        String::from_utf8_lossy(&listing.stderr)
    );
    let previous = registry_fixture::with_cdx_caption_edit(f::METADATA49);
    let prefix = previous.strip_suffix("]\n").ok_or("frozen array end")?;
    assert!(
        listing.stdout.starts_with(prefix.as_bytes()),
        "the original 49 discovery records retain their exact bytes"
    );
    assert_eq!(listing.stdout.get(prefix.len()), Some(&b','));
    let rows: Value = serde_json::from_slice(&listing.stdout)?;
    discovery(rows.as_array().ok_or("command rows")?, false)?;
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("metadata-cli-原-λ.cdxml");
    let control = directory.path("metadata-cli-original-control.cdxml");
    let output = directory.path("metadata-cli-保存-λ.cdxml");
    fs::write(&input, f::source("mixed_a"))?;
    fs::write(
        &control,
        include_str!("../../../io/tests/fixtures/created-labels/base.cdxml"),
    )?;
    let open = request("document.open_cdxml", json!({"path":input}));
    let baseline = cli(&[open.clone(), request("render.capture", capture("cpu"))])?;
    let plain = cli(&[
        request("document.open_cdxml", json!({"path":control})),
        request("render.capture", capture("cpu")),
    ])?;
    assert_eq!(baseline["document"], plain["document"]);
    assert_eq!(
        baseline["capture"], plain["capture"],
        "unused metadata/font resource cannot affect explicit-run CPU pixels"
    );
    let edits = [
        open,
        request(EDIT, f::label_params(1, 1, 7, 2)),
        request(CREATE, f::create_params(2, 3, 6, 2, f::black_style())),
        request(
            "ui.selection",
            json!({"action":"all","expected_revision":3,"page_id":1}),
        ),
        request("selection.translate", f::motion(3, 1, 24., -12.)),
    ];
    let mut cpu_steps = edits.to_vec();
    cpu_steps.push(request("render.capture", capture("cpu")));
    let cpu = cli(&cpu_steps)?;
    let mut gpu_steps = edits.to_vec();
    gpu_steps.push(request("render.capture", capture("gpu")));
    let gpu = cli(&gpu_steps)?;
    for value in [&cpu, &gpu] {
        state(
            value,
            &baseline["document"],
            "compound",
            [3, 0],
            [true, true],
            f::source("mixed_a").len(),
        )?;
        assert_eq!(revision(value)?, 4);
    }
    captures(&cpu, &gpu)?;
    let mut saved_steps = edits.to_vec();
    saved_steps.extend([
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request("document.save_cdxml", json!({"path":output})),
        request("document.open_cdxml", json!({"path":output})),
        request("document.inspect", json!({})),
    ]);
    let reopened = cli(&saved_steps)?;
    state(
        &reopened,
        &baseline["document"],
        "compound",
        [0, 0],
        [false, false],
        f::source("compound").len(),
    )?;
    assert_eq!(revision(&reopened)?, 7);
    assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
    consume(&input, "mixed_a")?;
    consume(&output, "compound")?;
    Ok(())
}

fn workflow(mut call: impl FnMut(&str, Value) -> Result<Value>, tag: &str) -> Result {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path(&format!("metadata-{tag}-原-λ.cdxml"));
    let output = directory.path(&format!("metadata-{tag}-保存-λ.cdxml"));
    fs::write(&input, f::source("mixed_a"))?;
    let opened = call("document.open_cdxml", json!({"path":input}))?;
    let baseline = opened["document"].clone();
    let source_len = f::source("mixed_a").len();
    state(
        &opened,
        &baseline,
        "mixed_a",
        [0, 0],
        [false, false],
        source_len,
    )?;
    let edited = call(EDIT, f::label_params(revision(&opened)?, 1, 7, 2))?;
    state(
        &edited,
        &baseline,
        "edited_endpoint",
        [1, 0],
        [true, true],
        source_len,
    )?;
    let undone = call("history.undo", json!({}))?;
    state(
        &undone,
        &baseline,
        "mixed_a",
        [0, 1],
        [false, false],
        source_len,
    )?;
    let before_noop = call("document.inspect", json!({}))?;
    let noop = call(EDIT, f::label_params(revision(&undone)?, 1, 8, 1))?;
    assert_eq!(
        noop, before_noop,
        "unused defaults cannot alter a certified no-op"
    );
    let redone = call("history.redo", json!({}))?;
    state(
        &redone,
        &baseline,
        "edited_endpoint",
        [1, 0],
        [true, true],
        source_len,
    )?;
    let created = call(
        CREATE,
        f::create_params(revision(&redone)?, 3, 6, 2, f::black_style()),
    )?;
    assert_eq!(created["changed"], true);
    assert_eq!(created["undo_depth"], 2);
    assert_eq!(created["redo_depth"], 0);
    let selected = call(
        "ui.selection",
        json!({"action":"all","expected_revision":revision(&created)?,"page_id":1}),
    )?;
    assert_eq!(selected["changed"], false);
    assert_eq!(selected["document"], created["document"]);
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
        "compound",
        [3, 0],
        [true, true],
        source_len,
    )?;
    let undo_move = call("history.undo", json!({}))?;
    assert_eq!(undo_move["document"], created["document"]);
    assert_eq!(
        (
            undo_move["undo_depth"].as_u64(),
            undo_move["redo_depth"].as_u64()
        ),
        (Some(2), Some(1))
    );
    let redo_move = call("history.redo", json!({}))?;
    state(
        &redo_move,
        &baseline,
        "compound",
        [3, 0],
        [true, true],
        source_len,
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
        "compound",
        [3, 0],
        [false, true],
        source_len,
    )?;
    assert_eq!(revision(&saved)?, revision(&redo_move)?);
    consume(&output, "compound")?;
    let reopened = call("document.open_cdxml", json!({"path":output}))?;
    state(
        &reopened,
        &baseline,
        "compound",
        [0, 0],
        [false, false],
        f::source("compound").len(),
    )?;
    assert_eq!(revision(&reopened)?, revision(&saved)? + 1);
    assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
    consume(&input, "mixed_a")?;
    consume(&output, "compound")?;
    Ok(())
}

#[test]
fn initialized_stdio_metadata_preserves_noop_history_compound_files_and_explicit_pixels() -> Result
{
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"label-metadata-test","version":"1"}}}))?;
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
        |name, params| {
            id += 1;
            client.call(id, name, params)
        },
        "stdio",
    )?;
    client.finish()
}

#[test]
fn authenticated_socket_metadata_preserves_noop_history_compound_files_and_explicit_pixels()
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
        |name, params| {
            let response = socket(&mut client, request(name, params))?;
            assert!(response.get("error").is_none(), "{response}");
            Ok(response["result"].clone())
        },
        "socket",
    )?;
    drop(client);
    server.shutdown()?;
    Ok(())
}
