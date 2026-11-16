use bondline_commands::Engine;
#[path = "support/native_conversion_wire.rs"]
mod native_conversion_wire;
#[path = "support/new_document_adapters.rs"]
mod new_document;
#[path = "support/sdf_remove_adapters.rs"]
mod record_remove;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, channel},
    },
    thread::JoinHandle,
    time::Duration,
};
#[path = "../../../xtask/oracles/sdf_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse the existing mixed-source directory, summary and independent consumers"
)]
mod support;
type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
fn capture() -> Value {
    json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"})
}
fn summary(value: &Value) {
    assert_eq!(value["document"], Value::Null);
    assert_eq!(value["collection"], support::expected_summary(true));
    assert_eq!(
        value["source"],
        json!({"format":"sdf","byte_length":777,"modified":false})
    );
    assert_eq!(value["revision"], 1);
    assert_eq!(value["undo_depth"], 0);
    assert_eq!(value["redo_depth"], 0);
    assert_eq!(value["file"]["dirty"], false);
    assert_eq!(value["file"]["format"], "sdf");
}
fn png(value: &Value) -> TestResult {
    summary(value);
    assert_eq!(value["changed"], false);
    assert_eq!(value["capture"]["backend"], "cpu");
    assert_eq!(value["capture"]["adapter"], Value::Null);
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut process = Command::new(python)
        .args([
            "-B",
            "-c",
            r#"
import base64,io,json,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
c=json.load(sys.stdin)
assert c['mime_type']=='image/png' and c['width_px']==1024 and c['height_px']==1024
b=base64.b64decode(c['png_base64'],validate=True)
with Image.open(io.BytesIO(b)) as im: im.verify()
with Image.open(io.BytesIO(b)) as im:
 assert im.format=='PNG' and im.mode=='RGBA' and im.size==(1024,1024)
 assert len(im.tobytes())==1024*1024*4
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(process.stdin.take().ok_or("PNG stdin")?, &value["capture"])?;
    let out = process.wait_with_output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(())
}
fn cli(requests: &[Value]) -> Result<Value, Box<dyn std::error::Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    command.arg("run");
    for request in requests {
        command.arg("--cmd").arg(request.to_string());
    }
    let out = command.output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(serde_json::from_slice(&out.stdout)?)
}
#[test]
fn cli_complete_sdf_capture_then_save_keeps_every_source_byte() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("capture-cli-λ.sdf");
    let output = dir.path("capture-cli-copy.data");
    fs::write(&input, SOURCE)?;
    let open = json!({"command":"document.open_sdf","params":{"path":input}});
    let render = json!({"command":"render.capture","params":capture()});
    let value = cli(&[open.clone(), render.clone()])?;
    png(&value)?;
    assert_eq!(value["file"]["path"], json!(fs::canonicalize(&input)?));
    let saved = cli(&[
        open,
        render,
        json!({"command":"document.save_sdf","params":{"path":output}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    summary(&saved);
    assert!(saved.get("capture").is_none());
    assert_eq!(saved["file"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::read(&output)?, SOURCE);
    support::independently_read_sdf(&output, SOURCE, true)?;
    Ok(())
}

struct Client {
    child: Child,
    input: Option<ChildStdin>,
    replies: Receiver<Result<Value, String>>,
    reader: Option<JoinHandle<()>>,
}
impl Client {
    fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child.stdin.take().ok_or("MCP stdin")?;
        let output = child.stdout.take().ok_or("MCP stdout")?;
        let (sender, replies) = channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let parsed = line
                    .map_err(|e| e.to_string())
                    .and_then(|line| serde_json::from_str(&line).map_err(|e| e.to_string()));
                if sender.send(parsed).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input: Some(input),
            replies,
            reader: Some(reader),
        })
    }
    fn send(&mut self, value: Value) -> TestResult {
        let input = self.input.as_mut().ok_or("closed stdin")?;
        serde_json::to_writer(&mut *input, &value)?;
        input.write_all(b"\n")?;
        input.flush()?;
        Ok(())
    }
    fn call(
        &mut self,
        id: u64,
        name: &str,
        args: Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":args}}))?;
        let value = self.replies.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(value["id"], id);
        assert_eq!(value["result"]["isError"], false, "{value}");
        Ok(value["result"]["structuredContent"].clone())
    }
    fn finish(mut self) -> TestResult {
        self.input.take();
        for _ in 0..1000 {
            if let Some(status) = self.child.try_wait()? {
                assert!(status.success());
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Err("MCP failed to close after EOF within ten seconds".into())
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
#[test]
fn stdio_complete_sdf_capture_is_explicit_not_cached_and_does_not_mutate_state() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("capture-stdio.sdf");
    let output = dir.path("capture-stdio-copy.data");
    fs::write(&input, SOURCE)?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sdf-capture-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.call(2, "document.open_sdf", json!({"path":input}))?;
    let before = client.call(3, "document.inspect", json!({}))?;
    let result = client.call(4, "render.capture", capture())?;
    png(&result)?;
    let mut state = result.clone();
    state.as_object_mut().ok_or("state")?.remove("capture");
    assert_eq!(state, before);
    assert_eq!(client.call(5, "document.inspect", json!({}))?, before);
    let saved = client.call(6, "document.save_sdf", json!({"path":output}))?;
    summary(&saved);
    assert!(saved.get("capture").is_none());
    assert_eq!(saved["file_operation"]["byte_length"], 777);
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::read(&output)?, SOURCE);
    support::independently_read_sdf(&output, SOURCE, true)?;
    client.finish()?;
    Ok(())
}
fn socket(
    client: &mut BufReader<TcpStream>,
    value: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    serde_json::to_writer(client.get_mut(), &value)?;
    client.get_mut().write_all(b"\n")?;
    client.get_mut().flush()?;
    let mut line = String::new();
    assert_ne!(client.read_line(&mut line)?, 0, "unexpected control EOF");
    Ok(serde_json::from_str(&line)?)
}
#[test]
fn authenticated_complete_sdf_capture_does_not_repaint_or_change_the_session() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("capture-socket.sdf");
    let output = dir.path("capture-socket-copy.data");
    fs::write(&input, SOURCE)?;
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
    // The credential is sent only on this trusted test connection, never printed or saved.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let open = socket(
        &mut client,
        json!({"command":"document.open_sdf","params":{"path":input}}),
    )?;
    assert!(open.get("error").is_none());
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    let result = socket(
        &mut client,
        json!({"command":"render.capture","params":capture()}),
    )?;
    assert!(result.get("error").is_none(), "{result}");
    png(&result["result"])?;
    let mut state = result["result"].clone();
    state.as_object_mut().ok_or("state")?.remove("capture");
    assert_eq!(state, before["result"]);
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), 1);
    let saved = socket(
        &mut client,
        json!({"command":"document.save_sdf","params":{"path":output}}),
    )?;
    assert!(saved.get("error").is_none());
    summary(&saved["result"]);
    assert!(saved["result"].get("capture").is_none());
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::read(&output)?, SOURCE);
    support::independently_read_sdf(&output, SOURCE, true)?;
    server.shutdown()?;
    Ok(())
}

#[path = "support/acyclic_conversion_wire.rs"]
mod acyclic_conversion_wire;

#[path = "support/ring_conversion_wire.rs"]
mod ring_conversion_wire;

#[path = "support/cdx_file_end_wire.rs"]
mod cdx_file_end_wire;

#[path = "support/components_conversion_wire.rs"]
mod components_conversion_wire;

#[path = "support/smiles_paste_wire.rs"]
mod smiles_paste_wire;
#[path = "support/smiles_record_authoring_wire.rs"]
mod smiles_record_authoring_wire;
