//! Source-only declaration, child of sdf_capture_adapters.rs. Root owns execution.
use super::{Client, cli, socket, support};
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
    sync::{Arc, Mutex},
    time::Duration,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const NAME: &str = "text.set_run";
const OLD46: &str = include_str!("caption-run-old46.json");
// Original literals: source IDs90/21 and font42 are not application text/page IDs1/1.
// White palette entry2 is explicit; there is no unsupported root bgcolor property.
const ORIGINAL: &str = r#"<CDXML CreationProgram="Independent caption wire fixture"><fonttable><font id="42" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id='90' DrawingSpace="Poster" Width="256.00" Height='256'><t id="21" p='72 96' Justification="Left"><s font='42' size="12.00" face="0" color="4">Phase A</s><s font="42" size='16' face="0" color="3"> μmol</s></t></page></CDXML>"#;
const EDITED: &str = r#"<CDXML CreationProgram="Independent caption wire fixture"><fonttable><font id="42" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id='90' DrawingSpace="Poster" Width="256.00" Height='256'><t id="21" p='72 96' Justification="Left"><s font='42' size="12.00" face="0" color="4">Phase B &amp; μ</s><s font="42" size='16' face="0" color="3"> μmol</s></t></page></CDXML>"#;
const CHANGED: &str = "Phase B & μ";
fn request(name: &str, params: Value) -> Value {
    json!({"command":name,"params":params})
}
fn edit(revision: u64, text: &str) -> Value {
    json!({"expected_revision":revision,"page_id":1,"text_id":1,"run_index":0,"text":text})
}
fn capture() -> Value {
    json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"})
}
fn model(text: &str) -> Value {
    json!({"pages":[{"id":1,"size":{"width_pt":256.0,"height_pt":256.0},"fragments":[],
        "texts":[{"id":1,"position":{"x_pt":72.0,"y_pt":96.0},"runs":[
        {"text":text,"style":{"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.2,0.4,0.8]}},
        {"text":" μmol","style":{"font_family":"Roboto","size_pt":16.0,"face":0,"rgb":[0.0,0.0,0.0]}}]}]}]})
}
fn checked(
    v: &Value,
    text: &str,
    revision: u64,
    depths: [u64; 2],
    flags: [bool; 2],
    source_len: usize,
) {
    assert_eq!(v["document"], model(text));
    assert_eq!(v["revision"], revision);
    assert_eq!(
        (v["undo_depth"].clone(), v["redo_depth"].clone()),
        (json!(depths[0]), json!(depths[1]))
    );
    assert_eq!(v["file"]["format"], "cdxml");
    assert_eq!(v["file"]["dirty"], flags[0]);
    assert_eq!(
        v["source"],
        json!({"format":"cdxml","byte_length":source_len,"modified":flags[1]})
    );
}
fn metadata(rows: &[Value], mcp: bool) -> TestResult {
    let old: Vec<Value> = serde_json::from_str(OLD46)?;
    assert_eq!((old.len(), rows.len()), (46, 47));
    for (actual, expected) in rows.iter().zip(&old) {
        if mcp {
            assert_eq!(actual["name"], expected["name"]);
            assert_eq!(actual["description"], expected["description"]);
            assert_eq!(actual["annotations"]["readOnlyHint"], expected["read_only"]);
            assert_eq!(
                actual["inputSchema"], expected["input_schema"],
                "{}",
                expected["name"]
            );
        } else {
            assert_eq!(actual, expected, "all old46 metadata remain exact");
        }
    }
    let row = rows.get(46).ok_or("new47th row")?;
    assert_eq!(row["name"], NAME);
    if mcp {
        assert_eq!(row["annotations"]["readOnlyHint"], false);
    } else {
        assert_eq!(row["read_only"], false);
    }
    let schema = &row[if mcp { "inputSchema" } else { "input_schema" }];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    let keys = BTreeSet::from([
        "expected_revision",
        "page_id",
        "text_id",
        "run_index",
        "text",
    ]);
    let required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or("key"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(required, keys);
    assert_eq!(
        schema["properties"]
            .as_object()
            .ok_or("properties")?
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        keys
    );
    assert_eq!(schema["properties"]["text"]["type"], "string");
    for name in ["expected_revision", "page_id", "text_id", "run_index"] {
        assert_eq!(schema["properties"][name]["type"], "integer");
    }
    Ok(())
}
fn python() -> std::ffi::OsString {
    std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    })
}
fn facts(raw: &[u8]) -> TestResult<Value> {
    let mut child=Process::new(python()).args(["-X","utf8","-B","-c",r#"
import json,sys,xml.parsers.expat as expat
from decimal import Decimal
raw=sys.stdin.buffer.read(1048577);assert 0<len(raw)<=1048576
fonts=[];colors=[];texts=[];stack=[];current=None;run=None;order=[];page=None
def start(name,attrs):
 global current,run,page
 order.append(name);stack.append(name)
 if name=='CDXML':assert attrs=={'CreationProgram':'Independent caption wire fixture'}
 if name=='font':fonts.append([int(attrs['id']),attrs['name']])
 if name=='color':
  rgb=[Decimal(attrs[k]) for k in ('r','g','b')];assert all(v.is_finite() and 0<=v<=1 for v in rgb);colors.append(list(map(str,rgb)))
 if name=='page':page={'id':int(attrs['id']),'width':str(Decimal(attrs['Width'])),'height':str(Decimal(attrs['Height'])),'space':attrs['DrawingSpace']}
 if name=='t':
  current={'id':int(attrs['id']),'p':[str(Decimal(v)) for v in attrs['p'].split()],'justification':attrs['Justification'],'runs':[]};texts.append(current)
 if name=='s':
  font=int(attrs['font']);assert font in dict(fonts)
  size=Decimal(attrs['size']);assert size.is_finite() and size>0
  face=int(attrs['face']);color=int(attrs['color']);assert 0<=face<=65535 and 2<=color<len(colors)+2
  run={'text':'','font':font,'size':str(size),'face':face,'color':color};current['runs'].append(run)
def end(name):
 global run
 assert stack.pop()==name
 if name=='s':run=None
def chars(text):
 if run is not None:run['text']+=text
 else:assert not text.strip(' \t\r\n')
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.ExternalEntityRefHandler=lambda *unused:0
p.StartElementHandler=start;p.EndElementHandler=end;p.CharacterDataHandler=chars;p.Parse(raw,True)
assert order==['CDXML','fonttable','font','colortable','color','color','color','page','t','s','s']
assert colors[0]==['1','1','1']
print(json.dumps({'fonts':fonts,'colors':colors,'page':page,'texts':texts},ensure_ascii=False,separators=(',',':')))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child
        .stdin
        .take()
        .ok_or("caption reader stdin")?
        .write_all(raw)?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(format!(
            "independent caption reader: {}",
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}
fn expected_facts(text: &str) -> Value {
    json!({"fonts":[[42,"Roboto"]],"colors":[["1","1","1"],["0","0","0"],["0.2","0.4","0.8"]],
        "page":{"id":90,"width":"256.00","height":"256","space":"Poster"},
        "texts":[{"id":21,"p":["72","96"],"justification":"Left","runs":[
        {"text":text,"font":42,"size":"12.00","face":0,"color":4},
        {"text":" μmol","font":42,"size":"16","face":0,"color":3}]}]})
}
fn checked_file(path: &Path, expected: &str, text: &str) -> TestResult {
    let raw = fs::read(path)?;
    assert_eq!(
        raw,
        expected.as_bytes(),
        "complete source bytes before any external/render check"
    );
    assert_eq!(facts(&raw)?, expected_facts(text));
    Ok(())
}
fn qualify(dir: &support::SdfTestDirectory) -> TestResult {
    let original = dir.path("manual-caption-λ.cdxml");
    let edited = dir.path("manual-caption-edited-化学.cdxml");
    fs::write(&original, ORIGINAL)?;
    fs::write(&edited, EDITED)?;
    checked_file(&original, ORIGINAL, "Phase A")?;
    checked_file(&edited, EDITED, CHANGED)?;
    let positive = facts(ORIGINAL.as_bytes())?;
    for (key, wrong) in [
        ("text", json!("Phase B")),
        ("font", json!(43)),
        ("size", json!("13")),
        ("face", json!(1)),
        ("color", json!(3)),
    ] {
        let mut bad = expected_facts("Phase A");
        bad["texts"][0]["runs"][0][key] = wrong;
        assert_ne!(positive, bad);
    }
    let missing = ORIGINAL.replacen("font='42'", "font='43'", 1);
    assert!(facts(missing.as_bytes()).is_err());
    let mut reversed = expected_facts("Phase A");
    reversed["texts"][0]["runs"]
        .as_array_mut()
        .ok_or("runs")?
        .reverse();
    assert_ne!(positive, reversed);
    Ok(())
}
fn png(value: &Value) -> TestResult {
    assert_eq!(value["capture"]["backend"], "cpu");
    assert_eq!(value["changed"], false);
    let mut child=Process::new(python()).args(["-X","utf8","-B","-c",r#"
import base64,io,json,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
c=json.load(sys.stdin);assert c['mime_type']=='image/png' and c['width_px']==1024 and c['height_px']==1024
raw=base64.b64decode(c['png_base64'],validate=True)
with Image.open(io.BytesIO(raw)) as im:im.verify()
with Image.open(io.BytesIO(raw)) as im:
 assert im.format=='PNG' and im.mode=='RGBA' and im.size==(1024,1024)
 assert any(pixel==(51,102,204,255) for pixel in im.get_flattened_data()),'explicit blue caption ink missing'
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(child.stdin.take().ok_or("PNG stdin")?, &value["capture"])?;
    let out = child.wait_with_output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(())
}
fn denied(client: &mut Client, id: u64, name: &str, params: Value, code: &str) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":params}}))?;
    let response = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(response["id"], id);
    assert_eq!(response["result"]["isError"], true);
    assert_eq!(
        response["result"]["structuredContent"]["error"]["code"],
        code
    );
    Ok(())
}

#[test]
fn actual_cli_caption_metadata_history_unicode_save_reopen_and_capture() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    qualify(&dir)?;
    let listed = Process::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let rows: Value = serde_json::from_slice(&listed.stdout)?;
    metadata(rows.as_array().ok_or("CLI commands")?, false)?;
    let input = dir.path("caption-cli-λ.cdxml");
    let output = dir.path("caption-cli-編集.cdxml");
    fs::write(&input, ORIGINAL)?;
    checked_file(&input, ORIGINAL, "Phase A")?;
    let state = cli(&[
        request("document.open_cdxml", json!({"path":input})),
        request(NAME, edit(1, CHANGED)),
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request("document.save_cdxml", json!({"path":output})),
        request("document.open_cdxml", json!({"path":output})),
        request("document.inspect", json!({})),
    ])?;
    checked(&state, CHANGED, 5, [0, 0], [false, false], EDITED.len());
    assert_eq!(state["file"]["path"], json!(fs::canonicalize(&output)?));
    checked_file(&input, ORIGINAL, "Phase A")?;
    checked_file(&output, EDITED, CHANGED)?;
    let captured = cli(&[
        request("document.open_cdxml", json!({"path":output})),
        request("ui.view", json!({"action":"actual"})),
        request("render.capture", capture()),
    ])?;
    checked(&captured, CHANGED, 1, [0, 0], [false, false], EDITED.len());
    png(&captured)?;
    Ok(())
}

#[test]
fn initialized_stdio_caption_noop_errors_history_and_actual_source_output() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    qualify(&dir)?;
    let input = dir.path("caption-stdio-λ.cdxml");
    let output = dir.path("caption-stdio-編集.cdxml");
    fs::write(&input, ORIGINAL)?;
    checked_file(&input, ORIGINAL, "Phase A")?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"caption-wire-test","version":"1"}}}))?;
    let init = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(init["id"], 1);
    assert!(init.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    metadata(listed["result"]["tools"].as_array().ok_or("tools")?, true)?;
    checked(
        &client.call(3, "document.open_cdxml", json!({"path":input}))?,
        "Phase A",
        1,
        [0, 0],
        [false, false],
        ORIGINAL.len(),
    );
    client.call(4, "ui.view", json!({"action":"actual"}))?;
    let before = client.call(5, "document.inspect", json!({}))?;
    denied(&mut client, 6, NAME, edit(0, CHANGED), "stale_revision")?;
    let mut extra = edit(1, CHANGED);
    extra["color"] = json!(3);
    denied(&mut client, 7, NAME, extra, "invalid_params")?;
    let mut missing = edit(1, CHANGED);
    missing.as_object_mut().ok_or("params")?.remove("run_index");
    denied(&mut client, 8, NAME, missing, "invalid_params")?;
    assert_eq!(client.call(9, "document.inspect", json!({}))?, before);
    checked(
        &client.call(10, NAME, edit(1, CHANGED))?,
        CHANGED,
        2,
        [1, 0],
        [true, true],
        ORIGINAL.len(),
    );
    checked(
        &client.call(11, "history.undo", json!({}))?,
        "Phase A",
        3,
        [0, 1],
        [false, false],
        ORIGINAL.len(),
    );
    let undone = client.call(12, "document.inspect", json!({}))?;
    let noop = client.call(13, NAME, edit(3, "Phase A"))?;
    assert_eq!(noop, undone);
    assert_eq!(noop["changed"], false);
    checked(
        &client.call(14, "history.redo", json!({}))?,
        CHANGED,
        4,
        [1, 0],
        [true, true],
        ORIGINAL.len(),
    );
    let saved = client.call(15, "document.save_cdxml", json!({"path":output}))?;
    checked(&saved, CHANGED, 4, [1, 0], [false, true], ORIGINAL.len());
    assert_eq!(saved["file_operation"]["byte_length"], EDITED.len());
    assert_eq!(saved["file_operation"]["ui_request_id"], Value::Null);
    assert_eq!(saved["file_operation"]["ui_request_current"], Value::Null);
    checked_file(&output, EDITED, CHANGED)?;
    checked(
        &client.call(16, "document.open_cdxml", json!({"path":output}))?,
        CHANGED,
        5,
        [0, 0],
        [false, false],
        EDITED.len(),
    );
    let current = client.call(17, "document.inspect", json!({}))?;
    let captured = client.call(18, "render.capture", capture())?;
    png(&captured)?;
    let mut without_capture = captured;
    without_capture
        .as_object_mut()
        .ok_or("capture result")?
        .remove("capture");
    assert_eq!(without_capture, current);
    denied(
        &mut client,
        19,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":40,"begin_y_pt":160,"end_x_pt":80,"end_y_pt":160}),
        "unsupported_content",
    )?;
    assert_eq!(client.call(20, "document.inspect", json!({}))?, current);
    checked_file(&input, ORIGINAL, "Phase A")?;
    checked_file(&output, EDITED, CHANGED)?;
    client.finish()
}

#[test]
fn authenticated_caption_capture_keeps_chemical_and_opaque_source_guards() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    qualify(&dir)?;
    let input = dir.path("caption-socket-λ.cdxml");
    let output = dir.path("caption-socket-編集.cdxml");
    let opaque = dir.path("caption-opaque.cdxml");
    let copy = dir.path("caption-opaque-copy.cdxml");
    let mol = dir.path("caption-loss.mol");
    let unknown = ORIGINAL.replacen("<CDXML ", "<CDXML Future='retain &amp; unknown' ", 1);
    fs::write(&input, ORIGINAL)?;
    fs::write(&opaque, unknown.as_bytes())?;
    fs::write(&mol, b"keep caption destination\n")?;
    checked_file(&input, ORIGINAL, "Phase A")?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let server = LoopbackServer::bind(0, Arc::clone(&engine), || {})?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    // The fresh credential is used only on this known test connection, never logged.
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
        "Phase A",
        1,
        [0, 0],
        [false, false],
        ORIGINAL.len(),
    );
    assert!(
        !engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .is_fully_mapped()
    );
    let viewed = socket(&mut client, request("ui.view", json!({"action":"actual"})))?;
    assert!(viewed.get("error").is_none(), "{viewed}");
    let edited = socket(&mut client, request(NAME, edit(1, CHANGED)))?;
    assert!(edited.get("error").is_none(), "{edited}");
    checked(
        &edited["result"],
        CHANGED,
        2,
        [1, 0],
        [true, true],
        ORIGINAL.len(),
    );
    assert!(
        !engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .is_fully_mapped()
    );
    let saved = socket(
        &mut client,
        request("document.save_cdxml", json!({"path":output})),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    checked(
        &saved["result"],
        CHANGED,
        2,
        [1, 0],
        [false, true],
        ORIGINAL.len(),
    );
    checked_file(&output, EDITED, CHANGED)?;
    let stable = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let (owner, stamp) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.content().clone(), guard.capture_stamp())
    };
    for (name, params) in [
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":40,"begin_y_pt":160,"end_x_pt":80,"end_y_pt":160}),
        ),
        ("document.save_mol", json!({"path":mol,"overwrite":true})),
    ] {
        let refused = socket(&mut client, request(name, params))?;
        assert_eq!(refused["error"]["code"], "unsupported_content");
        assert_eq!(
            socket(&mut client, request("document.inspect", json!({})))?["result"],
            stable
        );
    }
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &owner);
        assert!(guard.capture_stamp_matches(&stamp));
    }
    assert_eq!(fs::read(&mol)?, b"keep caption destination\n");
    let undone = socket(&mut client, request("history.undo", json!({})))?;
    checked(
        &undone["result"],
        "Phase A",
        3,
        [0, 1],
        [true, false],
        ORIGINAL.len(),
    );
    let redone = socket(&mut client, request("history.redo", json!({})))?;
    checked(
        &redone["result"],
        CHANGED,
        4,
        [1, 0],
        [false, true],
        ORIGINAL.len(),
    );
    let reopened = socket(
        &mut client,
        request("document.open_cdxml", json!({"path":output})),
    )?;
    assert!(reopened.get("error").is_none(), "{reopened}");
    checked(
        &reopened["result"],
        CHANGED,
        5,
        [0, 0],
        [false, false],
        EDITED.len(),
    );
    let captured = socket(&mut client, request("render.capture", capture()))?;
    assert!(captured.get("error").is_none(), "{captured}");
    png(&captured["result"])?;
    let opened = socket(
        &mut client,
        request("document.open_cdxml", json!({"path":opaque})),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    let before = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let refused = socket(
        &mut client,
        request(
            NAME,
            edit(
                before["revision"].as_u64().ok_or("revision")?,
                "must refuse",
            ),
        ),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        before
    );
    let copied = socket(
        &mut client,
        request("document.save_cdxml", json!({"path":copy})),
    )?;
    assert!(copied.get("error").is_none(), "{copied}");
    assert_eq!(fs::read(&copy)?, unknown.as_bytes());
    assert_eq!(fs::read(&opaque)?, unknown.as_bytes());
    checked_file(&input, ORIGINAL, "Phase A")?;
    checked_file(&output, EDITED, CHANGED)?;
    drop(client);
    server.shutdown()?;
    Ok(())
}
