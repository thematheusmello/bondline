//! Adopt as a child of sdf_capture_adapters.rs; use its unchanged process/socket lifecycle.
use super::{Client, cli, socket, support};
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    net::TcpStream,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[path = "../../../../xtask/oracles/boxed_label_support.rs"]
#[allow(dead_code, reason = "shared actual-file/FontTools literals")]
mod boxed;
fn request(name: &str, p: Value) -> Value {
    json!({"command":name,"params":p})
}
fn inspect() -> Value {
    request("document.inspect", json!({}))
}
fn capture() -> Value {
    json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"})
}
fn metadata(rows: &[Value], mcp: bool) -> TestResult {
    let old: Vec<Value> = serde_json::from_str(include_str!("atom-child-old47.json"))?;
    assert_eq!((old.len(), rows.len()), (47, 48));
    for (row, expected) in rows.iter().zip(&old) {
        if mcp {
            assert_eq!(row["name"], expected["name"]);
            assert_eq!(row["description"], expected["description"]);
            assert_eq!(row["annotations"]["readOnlyHint"], expected["read_only"]);
            assert_eq!(row["inputSchema"], expected["input_schema"]);
        } else {
            assert_eq!(row, expected);
        }
    }
    assert_eq!(rows[47]["name"], "atom.set_authored_label");
    let key = if mcp { "inputSchema" } else { "input_schema" };
    let atom = old
        .iter()
        .find(|r| r["name"] == "atom.set_label")
        .ok_or("old atom metadata")?;
    assert_eq!(rows[47][key], atom["input_schema"]);
    Ok(())
}
fn checked(
    v: &Value,
    e: u8,
    h: u16,
    rev: u64,
    depths: [u64; 2],
    flags: [bool; 2],
    source_len: usize,
) {
    assert_eq!(v["revision"], rev);
    assert_eq!(v["undo_depth"], depths[0]);
    assert_eq!(v["redo_depth"], depths[1]);
    assert_eq!(v["file"]["dirty"], flags[0]);
    assert_eq!(v["file"]["format"], "cdxml");
    assert_eq!(
        v["source"],
        json!({"format":"cdxml","byte_length":source_len,"modified":flags[1]})
    );
    let atom = &v["document"]["pages"][0]["fragments"][0]["atoms"][0];
    assert_eq!(atom["id"], 1);
    assert_eq!(atom["element"], e);
    assert_eq!(atom["explicit_hydrogens"], h);
    assert_eq!(
        atom["position"],
        json!({"x_pt":boxed::POINT[0],"y_pt":boxed::POINT[1]})
    );
    assert_eq!(atom["authored_label"], boxed::label(e, h));
}
fn consume(path: &std::path::Path, e: u8, h: u16) -> TestResult {
    let raw = fs::read(path)?;
    let expected = if e == 8 {
        boxed::SOURCE.to_owned()
    } else {
        boxed::edited()
    };
    assert_eq!(raw, expected.as_bytes());
    assert_eq!(boxed::read(&raw)?, boxed::facts(e, h));
    Ok(())
}
fn png(v: &Value) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-X",
            "utf8",
            "-B",
            "-c",
            r#"
import base64,io,json,sys
from PIL import Image,__version__
assert __version__=='12.3.0'
c=json.load(sys.stdin);assert c['mime_type']=='image/png' and c['backend']=='cpu'
assert c['width_px']==c['height_px']==1024
raw=base64.b64decode(c['png_base64'],validate=True)
with Image.open(io.BytesIO(raw)) as im: im.verify()
with Image.open(io.BytesIO(raw)) as im:
 assert im.mode=='RGBA' and im.size==(1024,1024)
 assert sum(p==(51,102,204,255) for p in im.get_flattened_data())>20
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(child.stdin.take().ok_or("PNG stdin")?, &v["capture"])?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
#[test]
fn actual_cli_command48_capture_history_unicode_save_and_reopen_keep_complete_boxed_source()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("boxed-cli-original-λ.cdxml");
    let output = dir.path("boxed-cli-保存.cdxml");
    fs::write(&input, boxed::SOURCE)?;
    consume(&input, 8, 1)?;
    let listed = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(listed.status.success());
    let rows: Value = serde_json::from_slice(&listed.stdout)?;
    metadata(rows.as_array().ok_or("rows")?, false)?;
    let open = request("document.open_cdxml", json!({"path":input}));
    let edit = request("atom.set_authored_label", boxed::params(1, 7, 2));
    let rendered = cli(&[
        open.clone(),
        edit.clone(),
        request("render.capture", capture()),
    ])?;
    checked(
        &rendered,
        7,
        2,
        2,
        [1, 0],
        [true, true],
        boxed::SOURCE.len(),
    );
    png(&rendered)?;
    let after = cli(&[
        open,
        edit,
        request("history.undo", json!({})),
        request("atom.set_authored_label", boxed::params(3, 8, 1)),
        request("history.redo", json!({})),
        request("document.save_cdxml", json!({"path":output})),
        request("document.open_cdxml", json!({"path":output})),
        inspect(),
    ])?;
    checked(
        &after,
        7,
        2,
        5,
        [0, 0],
        [false, false],
        boxed::edited().len(),
    );
    consume(&input, 8, 1)?;
    consume(&output, 7, 2)?;
    assert_eq!(after["file"]["path"], json!(fs::canonicalize(&output)?));
    Ok(())
}
#[test]
fn initialized_stdio_command48_preserves_old47_noops_history_and_actual_unicode_file_bytes()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("boxed-stdio-original-λ.cdxml");
    let output = dir.path("boxed-stdio-編集.cdxml");
    fs::write(&input, boxed::SOURCE)?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"boxed-label-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    metadata(listed["result"]["tools"].as_array().ok_or("tools")?, true)?;
    let opened = client.call(3, "document.open_cdxml", json!({"path":input}))?;
    checked(
        &opened,
        8,
        1,
        1,
        [0, 0],
        [false, false],
        boxed::SOURCE.len(),
    );
    let no = client.call(4, "atom.set_authored_label", boxed::params(1, 8, 1))?;
    assert_eq!(no["changed"], false);
    checked(&no, 8, 1, 1, [0, 0], [false, false], boxed::SOURCE.len());
    let changed = client.call(5, "atom.set_authored_label", boxed::params(1, 7, 2))?;
    checked(&changed, 7, 2, 2, [1, 0], [true, true], boxed::SOURCE.len());
    let rendered = client.call(6, "render.capture", capture())?;
    checked(
        &rendered,
        7,
        2,
        2,
        [1, 0],
        [true, true],
        boxed::SOURCE.len(),
    );
    png(&rendered)?;
    let undone = client.call(7, "history.undo", json!({}))?;
    checked(
        &undone,
        8,
        1,
        3,
        [0, 1],
        [false, false],
        boxed::SOURCE.len(),
    );
    let no = client.call(8, "atom.set_authored_label", boxed::params(3, 8, 1))?;
    assert_eq!(no["changed"], false);
    checked(&no, 8, 1, 3, [0, 1], [false, false], boxed::SOURCE.len());
    let redone = client.call(9, "history.redo", json!({}))?;
    checked(&redone, 7, 2, 4, [1, 0], [true, true], boxed::SOURCE.len());
    let saved = client.call(10, "document.save_cdxml", json!({"path":output}))?;
    checked(&saved, 7, 2, 4, [1, 0], [false, true], boxed::SOURCE.len());
    consume(&output, 7, 2)?;
    let reopened = client.call(11, "document.open_cdxml", json!({"path":output}))?;
    checked(
        &reopened,
        7,
        2,
        5,
        [0, 0],
        [false, false],
        boxed::edited().len(),
    );
    consume(&input, 8, 1)?;
    client.finish()
}
#[test]
fn authenticated_loopback_command48_keeps_refusals_capture_and_current_saved_source_baselines_distinct()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("boxed-socket-original-λ.cdxml");
    let output = dir.path("boxed-socket-保存.cdxml");
    let mol = dir.path("boxed-sentinel.mol");
    fs::write(&input, boxed::SOURCE)?;
    fs::write(&mol, b"keep literal label\n")?;
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
        request("document.open_cdxml", json!({"path":input})),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    checked(
        &opened["result"],
        8,
        1,
        1,
        [0, 0],
        [false, false],
        boxed::SOURCE.len(),
    );
    let stable = socket(&mut client, inspect())?["result"].clone();
    let model = engine
        .lock()
        .map_err(|_| "engine")?
        .stored_document()?
        .model_snapshot();
    for (name, p) in [
        ("atom.set_label", boxed::params(1, 8, 1)),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":512,"height_pt":512}),
        ),
        (
            "text.set_run",
            json!({"expected_revision":1,"page_id":1,"text_id":1,"run_index":0,"text":"OH"}),
        ),
        ("document.save_mol", json!({"path":mol,"overwrite":true})),
    ] {
        let failed = socket(&mut client, request(name, p))?;
        assert_eq!(
            failed["error"]["code"], "unsupported_content",
            "{name}: {failed}"
        );
        assert_eq!(socket(&mut client, inspect())?["result"], stable);
    }
    assert!(Arc::ptr_eq(
        &model,
        &engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .model_snapshot()
    ));
    assert_eq!(fs::read(&mol)?, b"keep literal label\n");
    let changed = socket(
        &mut client,
        request("atom.set_authored_label", boxed::params(1, 7, 2)),
    )?;
    assert!(changed.get("error").is_none(), "{changed}");
    checked(
        &changed["result"],
        7,
        2,
        2,
        [1, 0],
        [true, true],
        boxed::SOURCE.len(),
    );
    let captured = socket(&mut client, request("render.capture", capture()))?;
    assert!(captured.get("error").is_none(), "{captured}");
    png(&captured["result"])?;
    let saved = socket(
        &mut client,
        request("document.save_cdxml", json!({"path":output})),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    checked(
        &saved["result"],
        7,
        2,
        2,
        [1, 0],
        [false, true],
        boxed::SOURCE.len(),
    );
    consume(&output, 7, 2)?;
    let reopened = socket(
        &mut client,
        request("document.open_cdxml", json!({"path":output})),
    )?;
    assert!(reopened.get("error").is_none(), "{reopened}");
    checked(
        &reopened["result"],
        7,
        2,
        3,
        [0, 0],
        [false, false],
        boxed::edited().len(),
    );
    consume(&input, 8, 1)?;
    drop(client);
    server.shutdown()?;
    Ok(())
}
