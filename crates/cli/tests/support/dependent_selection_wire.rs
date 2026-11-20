//! Independent adapter declarations; adopt beneath sdf_capture_adapters.rs unchanged.
use super::{Client, cli, socket, support};
use crate::registry_fixture;
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
#[path = "../../../../xtask/oracles/dependent_label_support.rs"]
#[allow(dead_code, reason = "shared frozen source/history/App acceptance")]
mod f;
fn request(name: &str, p: Value) -> Value {
    json!({"command":name,"params":p})
}
fn inspect() -> Value {
    request("document.inspect", json!({}))
}
fn capture() -> Value {
    json!({"page_id":1,"width_px":1088,"height_px":1088,"backend":"cpu"})
}
fn all(v: &Value) -> TestResult<Value> {
    Ok(
        json!({"action":"all","expected_revision":v["revision"].as_u64().ok_or("revision")?,"page_id":1}),
    )
}
fn movement(v: &Value, dx: f64, dy: f64) -> TestResult<Value> {
    Ok(f::params(
        v["revision"].as_u64().ok_or("revision")?,
        v["ui_selection"]["epoch"]
            .as_u64()
            .ok_or("selection epoch")?,
        dx,
        dy,
    ))
}
fn ids(base: &str) -> Vec<u32> {
    if base == "boxed" {
        vec![1]
    } else {
        vec![1, 2, 3]
    }
}
fn same_authored(v: &Value, before: &Value) {
    for name in [
        "document",
        "source",
        "file",
        "revision",
        "undo_depth",
        "redo_depth",
        "close",
        "close_epoch",
        "ui_file",
    ] {
        assert_eq!(v[name], before[name], "unchanged {name}");
    }
}
fn checked(
    v: &Value,
    doc: &Value,
    revision: u64,
    depths: [u64; 2],
    flags: [bool; 2],
    source_len: usize,
) {
    assert_eq!(v["document"], *doc);
    assert_eq!(v["revision"], revision);
    assert_eq!(v["undo_depth"], depths[0]);
    assert_eq!(v["redo_depth"], depths[1]);
    assert_eq!(v["file"]["dirty"], flags[0]);
    assert_eq!(v["file"]["format"], "cdxml");
    assert_eq!(
        v["source"],
        json!({"format":"cdxml","byte_length":source_len,"modified":flags[1]})
    );
}
fn metadata(rows: &[Value], mcp: bool) -> TestResult {
    let old: Vec<Value> =
        serde_json::from_str(&registry_fixture::with_cdx_caption_edit(f::METADATA48))?;
    assert_eq!((old.len(), rows.len()), (48, 55));
    for (actual, expected) in rows.iter().zip(&old) {
        if mcp {
            assert_eq!(actual["name"], expected["name"]);
            assert_eq!(actual["description"], expected["description"]);
            assert_eq!(actual["annotations"]["readOnlyHint"], expected["read_only"]);
            assert_eq!(actual["inputSchema"], expected["input_schema"]);
        } else {
            assert_eq!(actual, expected);
        }
    }
    Ok(())
}
fn consume(path: &std::path::Path, name: &str) -> TestResult {
    let raw = fs::read(path)?;
    assert_eq!(raw, f::source(name).as_bytes());
    assert_eq!(f::read(&raw)?, f::facts(name));
    Ok(())
}
// The reference is the actual baseline CPU capture, shifted independently by Pillow.
// Each colour's predeclared IoU >= .99 allows boundary pixels from f32 rounding only;
// all ink must also stay inside that colour's independently translated one-pixel hull.
fn translated_pixels(
    before: &Value,
    after: &Value,
    dx_px: i32,
    dy_px: i32,
    bonds: bool,
) -> TestResult {
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
data=json.load(sys.stdin)
def pixels(c):
 assert c['mime_type']=='image/png' and c['backend']=='cpu' and c['adapter'] is None
 assert c['width_px']==c['height_px']==1088
 raw=base64.b64decode(c['png_base64'],validate=True)
 with Image.open(io.BytesIO(raw)) as im: im.verify()
 with Image.open(io.BytesIO(raw)) as im:
  assert im.mode=='RGBA' and im.size==(1088,1088)
  return list(im.get_flattened_data())
before,after=map(pixels,(data['before'],data['after']))
for colour,present in [((51,102,204,255),True),((0,0,0,255),data['bonds'])]:
 old={(i%1088,i//1088) for i,p in enumerate(before) if p==colour}
 new={(i%1088,i//1088) for i,p in enumerate(after) if p==colour}
 if not present:
  assert not old and not new
  continue
 assert len(old)>20 and len(new)>20
 expected={(x+data['dx'],y+data['dy']) for x,y in old}
 assert all(0<=x<1088 and 0<=y<1088 for x,y in expected)
 assert len(expected&new)/len(expected|new)>=.99
 hull={(x+a,y+b) for x,y in expected for a in (-1,0,1) for b in (-1,0,1)}
 assert new<=hull
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(
        child.stdin.take().ok_or("PNG stdin")?,
        &json!({"before":before["capture"],"after":after["capture"],"dx":dx_px,"dy":dy_px,"bonds":bonds}),
    )?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
#[test]
fn actual_cli_both_label_profiles_move_all_capture_history_unicode_save_and_reopen() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let listed = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(listed.status.success());
    let rows: Value = serde_json::from_slice(&listed.stdout)?;
    metadata(rows.as_array().ok_or("rows")?, false)?;
    for (base, after) in [
        ("boxed", "boxed_all_p24_m12"),
        ("connected", "connected_all_p24_m12"),
    ] {
        let input = dir.path(&format!("dependent-{base}-cli-original-λ.cdxml"));
        let output = dir.path(&format!("dependent-{base}-cli-保存.cdxml"));
        fs::write(&input, f::source(base))?;
        consume(&input, base)?;
        let open = request("document.open_cdxml", json!({"path":input}));
        let baseline = cli(&[open.clone(), request("render.capture", capture())])?;
        let select = request(
            "ui.selection",
            json!({"action":"all","expected_revision":1,"page_id":1}),
        );
        let translate = request("selection.translate", f::params(1, 1, 24., -12.));
        let moved = cli(&[
            open.clone(),
            select.clone(),
            translate.clone(),
            request("render.capture", capture()),
        ])?;
        let expected = f::model_after(&baseline["document"], &ids(base), 24., -12.)?;
        checked(
            &moved,
            &expected,
            2,
            [1, 0],
            [true, true],
            f::source(base).len(),
        );
        translated_pixels(&baseline, &moved, 48, -24, base == "connected")?;
        let final_state = cli(&[
            open,
            select,
            translate,
            request("history.undo", json!({})),
            request("history.redo", json!({})),
            request("document.save_cdxml", json!({"path":output})),
            request("document.open_cdxml", json!({"path":output})),
            inspect(),
        ])?;
        checked(
            &final_state,
            &expected,
            5,
            [0, 0],
            [false, false],
            f::source(after).len(),
        );
        assert_eq!(
            final_state["file"]["path"],
            json!(fs::canonicalize(&output)?)
        );
        consume(&output, after)?;
        consume(&input, base)?;
    }
    Ok(())
}
#[test]
fn initialized_stdio_both_profiles_reuse_exact48_and_keep_nudge_noop_redo_and_file_owners()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"dependent-label-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    metadata(listed["result"]["tools"].as_array().ok_or("tools")?, true)?;
    for (index, (base, after)) in [
        ("boxed", "boxed_nudge_p25_m2"),
        ("connected", "connected_nudge_p25_m2"),
    ]
    .into_iter()
    .enumerate()
    {
        let id = 10 + u64::try_from(index)? * 100;
        let input = dir.path(&format!("dependent-{base}-stdio-λ.cdxml"));
        let output = dir.path(&format!("dependent-{base}-stdio-編集.cdxml"));
        fs::write(&input, f::source(base))?;
        let opened = client.call(id, "document.open_cdxml", json!({"path":input}))?;
        let revision = opened["revision"].as_u64().ok_or("revision")?;
        let baseline = client.call(id + 1, "render.capture", capture())?;
        let selected = client.call(id + 2, "ui.selection", all(&opened)?)?;
        same_authored(&selected, &opened);
        assert_eq!(selected["changed"], false);
        assert_eq!(selected["ui_selection"]["atom_ids"], json!(ids(base)));
        let drag = client.call(
            id + 3,
            "selection.translate",
            movement(&selected, 24., -12.)?,
        )?;
        let right = client.call(id + 4, "selection.translate", movement(&drag, 1., 0.)?)?;
        let down = client.call(id + 5, "selection.translate", movement(&right, 0., 10.)?)?;
        let expected = f::model_after(&opened["document"], &ids(base), 25., -2.)?;
        checked(
            &down,
            &expected,
            revision + 3,
            [3, 0],
            [true, true],
            f::source(base).len(),
        );
        let rendered = client.call(id + 6, "render.capture", capture())?;
        translated_pixels(&baseline, &rendered, 50, -4, base == "connected")?;
        let undone = client.call(id + 7, "history.undo", json!({}))?;
        assert_eq!(undone["document"], right["document"]);
        assert_eq!(undone["redo_depth"], 1);
        let reselected = client.call(id + 8, "ui.selection", all(&undone)?)?;
        let noop = client.call(
            id + 9,
            "selection.translate",
            movement(&reselected, 0., 0.)?,
        )?;
        same_authored(&noop, &undone);
        assert_eq!(noop["changed"], false);
        assert_eq!(noop["ui_selection"], reselected["ui_selection"]);
        let redone = client.call(id + 10, "history.redo", json!({}))?;
        assert_eq!(redone["document"], expected);
        assert_eq!(redone["redo_depth"], 0);
        assert_eq!(redone["undo_depth"], 3);
        let saved = client.call(id + 11, "document.save_cdxml", json!({"path":output}))?;
        checked(
            &saved,
            &expected,
            revision + 5,
            [3, 0],
            [false, true],
            f::source(base).len(),
        );
        consume(&output, after)?;
        let reopened = client.call(id + 12, "document.open_cdxml", json!({"path":output}))?;
        checked(
            &reopened,
            &expected,
            revision + 6,
            [0, 0],
            [false, false],
            f::source(after).len(),
        );
        consume(&input, base)?;
    }
    client.finish()
}
#[test]
fn authenticated_socket_subset_moves_preserve_arc_noops_refuse_other_authors_and_keep_saved_baseline_separate()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    for (base, after) in [
        ("boxed", "boxed_sub_p12_5_m7_25"),
        ("connected", "connected_sub_p12_5_m7_25"),
    ] {
        let input = dir.path(&format!("dependent-{base}-socket-λ.cdxml"));
        let output = dir.path(&format!("dependent-{base}-socket-保存.cdxml"));
        let mol = dir.path(&format!("dependent-{base}-keep.mol"));
        fs::write(&input, f::source(base))?;
        fs::write(&mol, b"keep authored label\n")?;
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
        let selected = socket(
            &mut client,
            request(
                "ui.selection",
                json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[1]}),
            ),
        )?;
        assert!(selected.get("error").is_none(), "{selected}");
        same_authored(&selected["result"], &opened["result"]);
        let stable = socket(&mut client, inspect())?["result"].clone();
        let model = engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .model_snapshot();
        let noop = socket(
            &mut client,
            request(
                "selection.translate",
                movement(&selected["result"], 0., 0.)?,
            ),
        )?;
        assert!(noop.get("error").is_none(), "{noop}");
        same_authored(&noop["result"], &stable);
        assert_eq!(noop["result"]["ui_selection"], stable["ui_selection"]);
        assert_eq!(noop["result"]["changed"], false);
        for (name, p) in [
            ("ui.ring_tool", json!({"size":6})),
            ("ui.ring_tool", json!({"kind":"cyclohexane_chair_1"})),
            ("ui.chain_tool", json!({"action":"select","kind":"acyclic"})),
            (
                "atom.set_label",
                json!({"expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":1,"element":8,"explicit_hydrogens":if base=="boxed"{1}else{0}}),
            ),
            (
                "page.set_size",
                json!({"page_id":1,"width_pt":512,"height_pt":512}),
            ),
            ("document.save_mol", json!({"path":mol,"overwrite":true})),
        ] {
            let error = socket(&mut client, request(name, p))?;
            assert_eq!(
                error["error"]["code"], "unsupported_content",
                "{name}: {error}"
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
        assert_eq!(fs::read(&mol)?, b"keep authored label\n");
        let moved = socket(
            &mut client,
            request(
                "selection.translate",
                movement(&selected["result"], 12.5, -7.25)?,
            ),
        )?;
        assert!(moved.get("error").is_none(), "{moved}");
        let expected = f::model_after(&opened["result"]["document"], &[1], 12.5, -7.25)?;
        checked(
            &moved["result"],
            &expected,
            2,
            [1, 0],
            [true, true],
            f::source(base).len(),
        );
        let saved = socket(
            &mut client,
            request("document.save_cdxml", json!({"path":output})),
        )?;
        assert!(saved.get("error").is_none(), "{saved}");
        checked(
            &saved["result"],
            &expected,
            2,
            [1, 0],
            [false, true],
            f::source(base).len(),
        );
        consume(&output, after)?;
        let reopened = socket(
            &mut client,
            request("document.open_cdxml", json!({"path":output})),
        )?;
        assert!(reopened.get("error").is_none(), "{reopened}");
        checked(
            &reopened["result"],
            &expected,
            3,
            [0, 0],
            [false, false],
            f::source(after).len(),
        );
        consume(&input, base)?;
        drop(client);
        server.shutdown()?;
    }
    Ok(())
}
