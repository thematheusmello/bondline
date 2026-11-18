//! Original mixed foreign files through the actual CLI, initialized MCP and loopback.
use super::{Client, cli, socket, support};
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};
#[path = "../../../../xtask/oracles/mixed_label_support.rs"]
#[allow(dead_code, reason = "shared frozen source/model/reader acceptance")]
mod f;
type Result<T = ()> = f::Result<T>;
fn request(name: &str, params: Value) -> Value {
    json!({"command":name,"params":params})
}
fn capture(backend: &str) -> Value {
    json!({"page_id":1,"width_px":1088,"height_px":1088,"backend":backend})
}
fn consume(path: &std::path::Path, case: &str) -> Result {
    let raw = fs::read(path)?;
    assert_eq!(raw, f::source(case).as_bytes());
    assert_eq!(f::read(&raw)?, f::facts(case));
    Ok(())
}
pub(super) fn captures(cpu: &Value, gpu: &Value) -> Result {
    use std::process::{Command, Stdio};
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
import base64,io,json,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
v=json.load(sys.stdin); pixels=[]
for backend in ['cpu','gpu']:
 c=v[backend];assert c['backend']==backend and c['width_px']==c['height_px']==1088
 assert c['mime_type']=='image/png'
 raw=base64.b64decode(c['png_base64'],validate=True)
 im=Image.open(io.BytesIO(raw));assert im.size==(1088,1088) and im.mode=='RGBA'
 pixels.append(im.tobytes())
 assert (51,102,204,255) in im.getdata() and (0,0,0,255) in im.getdata()
assert pixels[0]==pixels[1]
assert v['cpu']['png_base64']==v['gpu']['png_base64']
assert v['cpu']['adapter'] is None and v['gpu']['adapter']['name']
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(
        child.stdin.take().ok_or("PNG input")?,
        &json!({"cpu":cpu["capture"],"gpu":gpu["capture"]}),
    )?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
fn revision(v: &Value) -> Result<u64> {
    v["revision"].as_u64().ok_or_else(|| "revision".into())
}
fn epoch(v: &Value) -> Result<u64> {
    v["ui_selection"]["epoch"]
        .as_u64()
        .ok_or_else(|| "selection epoch".into())
}
fn expected(v: &Value, model: &Value, depths: [u64; 2], dirty: bool) {
    assert_eq!(v["document"], *model);
    assert_eq!(v["undo_depth"], depths[0]);
    assert_eq!(v["redo_depth"], depths[1]);
    assert_eq!(v["file"]["dirty"], dirty);
}

#[test]
fn actual_cli_mixed_edit_move_history_captures_unicode_save_reopen() -> Result {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("mixed-cli-原-λ.cdxml");
    let output = dir.path("mixed-cli-保存-λ.cdxml");
    fs::write(&input, f::source("amino_alcohol"))?;
    let open = request("document.open_cdxml", json!({"path":input}));
    let baseline = cli(&[open.clone(), request("render.capture", capture("cpu"))])?;
    let edit = request("atom.set_authored_label", f::label_params(1, 1, 7, 2));
    let select = request(
        "ui.selection",
        json!({"action":"all","expected_revision":2,"page_id":1}),
    );
    let movement = request("selection.translate", f::motion(2, 1, 24., -12.));
    let steps = [open.clone(), edit.clone(), select.clone(), movement.clone()];
    let mut cpu_steps = steps.to_vec();
    cpu_steps.push(request("render.capture", capture("cpu")));
    let cpu = cli(&cpu_steps)?;
    let mut gpu_steps = steps.to_vec();
    gpu_steps.push(request("render.capture", capture("gpu")));
    let gpu = cli(&gpu_steps)?;
    let model = f::model_for_case(&baseline["document"], "edited_all_p24_m12")?;
    expected(&cpu, &model, [2, 0], true);
    expected(&gpu, &model, [2, 0], true);
    assert_eq!(cpu["revision"], 3);
    captures(&cpu, &gpu)?;
    let result = cli(&[
        open,
        edit,
        select,
        movement,
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request("document.save_cdxml", json!({"path":output})),
        request("document.open_cdxml", json!({"path":output})),
        request("document.inspect", json!({})),
    ])?;
    expected(&result, &model, [0, 0], false);
    assert_eq!(result["revision"], 6);
    assert_eq!(
        result["source"],
        json!({"format":"cdxml","byte_length":f::source("edited_all_p24_m12").len(),"modified":false})
    );
    consume(&output, "edited_all_p24_m12")?;
    consume(&input, "amino_alcohol")?;
    Ok(())
}

fn remote_workflow(mut call: impl FnMut(&str, Value) -> Result<Value>, tag: &str) -> Result {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path(&format!("mixed-{tag}-原-λ.cdxml"));
    let hidden_output = dir.path(&format!("mixed-{tag}-hidden-保存.cdxml"));
    let output = dir.path(&format!("mixed-{tag}-保存-λ.cdxml"));
    fs::write(&input, f::source("amino_alcohol"))?;
    let opened = call("document.open_cdxml", json!({"path":input}))?;
    let selected = call(
        "ui.selection",
        json!({"action":"set","expected_revision":revision(&opened)?,"page_id":1,"atom_ids":[3]}),
    )?;
    let moved = call(
        "selection.translate",
        f::motion(revision(&selected)?, epoch(&selected)?, 12.5, -7.25),
    )?;
    let hidden_model = f::model_for_case(&opened["document"], "hidden_middle_p12_5_m7_25")?;
    expected(&moved, &hidden_model, [1, 0], true);
    let saved = call("document.save_cdxml", json!({"path":hidden_output}))?;
    expected(&saved, &hidden_model, [1, 0], false);
    consume(&hidden_output, "hidden_middle_p12_5_m7_25")?;
    let undone = call("history.undo", json!({}))?;
    expected(&undone, &opened["document"], [0, 1], true);
    let selected = call(
        "ui.selection",
        json!({"action":"set","expected_revision":revision(&undone)?,"page_id":1,"atom_ids":[3]}),
    )?;
    let noop = call(
        "selection.translate",
        f::motion(revision(&selected)?, epoch(&selected)?, 0., 0.),
    )?;
    assert_eq!(noop["revision"], undone["revision"]);
    assert_eq!(noop["redo_depth"], 1);
    assert_eq!(noop["ui_selection"], selected["ui_selection"]);
    let redone = call("history.redo", json!({}))?;
    expected(&redone, &hidden_model, [1, 0], false);
    let base = call("history.undo", json!({}))?;
    let edited = call(
        "atom.set_authored_label",
        f::label_params(revision(&base)?, 1, 7, 2),
    )?;
    let selected = call(
        "ui.selection",
        json!({"action":"all","expected_revision":revision(&edited)?,"page_id":1}),
    )?;
    let moved = call(
        "selection.translate",
        f::motion(revision(&selected)?, epoch(&selected)?, 24., -12.),
    )?;
    let model = f::model_for_case(&opened["document"], "edited_all_p24_m12")?;
    expected(&moved, &model, [2, 0], true);
    let before = call("document.inspect", json!({}))?;
    let cpu = call("render.capture", capture("cpu"))?;
    let gpu = call("render.capture", capture("gpu"))?;
    captures(&cpu, &gpu)?;
    assert_eq!(call("document.inspect", json!({}))?, before);
    call("document.save_cdxml", json!({"path":output}))?;
    consume(&output, "edited_all_p24_m12")?;
    let reopened = call("document.open_cdxml", json!({"path":output}))?;
    expected(&reopened, &model, [0, 0], false);
    assert_eq!(reopened["source"]["modified"], false);
    consume(&input, "amino_alcohol")?;
    consume(&hidden_output, "hidden_middle_p12_5_m7_25")?;
    Ok(())
}

#[test]
fn initialized_stdio_mixed_hidden_motion_noop_redo_compound_save_and_capture() -> Result {
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"mixed-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let mut id = 1;
    remote_workflow(
        |name, args| {
            id += 1;
            client.call(id, name, args)
        },
        "stdio",
    )?;
    client.finish()?;
    Ok(())
}

#[test]
fn authenticated_socket_mixed_hidden_motion_noop_redo_compound_save_and_capture() -> Result {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let server = LoopbackServer::bind(0, Arc::clone(&engine), || {})?;
    let stream = TcpStream::connect(server.address())?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    remote_workflow(
        |name, args| {
            let result = socket(&mut client, request(name, args))?;
            assert!(result.get("error").is_none(), "{result}");
            Ok(result["result"].clone())
        },
        "socket",
    )?;
    drop(client);
    server.shutdown()?;
    Ok(())
}
