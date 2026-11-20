//! Independent source-only declarations, child of sdf_capture_adapters.rs.
use super::{Client, cli, socket, support};
use crate::registry_fixture;
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufReader, Write},
    net::TcpStream,
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const OLD47: &str = include_str!("atom-child-old47.json");
const SOURCE: &str = r#"<CDXML CreationProgram='Original child-text wire fixture'><fonttable><font id='42' name='Roboto'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='0.2' g='0.4' b='0.8'/></colortable><page id='90' DrawingSpace='Poster' Width='256' Height='256'><fragment id='51'><n id='73' Element='8' NumHydrogens='1' p='72 96'><t id='21' p='71.5 96.25' Justification='Left' LabelJustification='Auto' LabelAlignment='Right'><s font='42' size='12.5' face='96' color='4'>OH &amp; μ</s></t></n></fragment><t id='88' p='72 144' Justification='Left'><s font='42' size='12' face='0' color='3'>caption</s></t></page></CDXML>"#;
fn request(name: &str, params: Value) -> Value {
    json!({"command":name,"params":params})
}
fn expected_label() -> Value {
    json!({"position":{"x_pt":71.5,"y_pt":96.25},"justification":"left","label_justification":"auto","label_alignment":"right",
    "runs":[{"text":"OH & μ","style":{"font_family":"Roboto","size_pt":12.5,"face":96,"rgb":[0.2,0.4,0.8]}}]})
}
fn expected_facts() -> Value {
    json!([
        {"parent":"n","position":[71.5,96.25],"justification":"Left","label_justification":"Auto","label_alignment":"Right",
         "runs":[{"text":"OH & μ","font_family":"Roboto","size_pt":12.5,"face":96,"rgb":[0.2,0.4,0.8]}]},
        {"parent":"page","position":[72.0,144.0],"justification":"Left","label_justification":null,"label_alignment":null,
         "runs":[{"text":"caption","font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.0,0.0,0.0]}]}
    ])
}
fn facts(raw: &[u8]) -> TestResult<Value> {
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
            include_str!("../../../io/tests/atom_label_xml_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("independent XML stdin")?
        .write_all(raw)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "independent child-text XML: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn checked_file(path: &Path) -> TestResult {
    let bytes = fs::read(path)?;
    assert_eq!(bytes, SOURCE.as_bytes());
    assert_eq!(facts(&bytes)?, expected_facts());
    Ok(())
}
fn checked(value: &Value, revision: u64) {
    assert_eq!(value["revision"], revision);
    assert_eq!(value["undo_depth"], 0);
    assert_eq!(value["redo_depth"], 0);
    assert_eq!(value["file"]["dirty"], false);
    assert_eq!(value["file"]["format"], "cdxml");
    assert_eq!(
        value["source"],
        json!({"format":"cdxml","byte_length":SOURCE.len(),"modified":false})
    );
    let atom = &value["document"]["pages"][0]["fragments"][0]["atoms"][0];
    assert_eq!(atom["id"], 1);
    assert_eq!(atom["element"], 8);
    assert_eq!(atom["explicit_hydrogens"], 1);
    assert_eq!(atom["position"], json!({"x_pt":72.0,"y_pt":96.0}));
    assert_eq!(atom["authored_label"], expected_label());
    assert_eq!(value["document"]["pages"][0]["texts"][0]["id"], 1);
    assert_eq!(
        value["document"]["pages"][0]["texts"][0]["runs"][0]["text"],
        "caption"
    );
    assert!(value.get("capture").is_none());
}
fn metadata(rows: &[Value], mcp: bool) -> TestResult {
    let expected: Vec<Value> =
        serde_json::from_str(&registry_fixture::with_cdx_caption_edit(OLD47))?;
    assert_eq!((expected.len(), rows.len()), (47, 53));
    for (row, old) in rows.iter().zip(&expected) {
        if mcp {
            assert_eq!(row["name"], old["name"]);
            assert_eq!(row["description"], old["description"]);
            assert_eq!(row["annotations"]["readOnlyHint"], old["read_only"]);
            assert_eq!(row["inputSchema"], old["input_schema"]);
        } else {
            assert_eq!(row, old);
        }
    }
    Ok(())
}
fn refusals(revision: u64) -> Vec<(&'static str, Value)> {
    vec![
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":256.0,"height_pt":256.0}),
        ),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":512.0,"height_pt":512.0}),
        ),
        (
            "atom.set_label",
            json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":1,"element":8,"explicit_hydrogens":1}),
        ),
        (
            "text.set_run",
            json!({"expected_revision":revision,"page_id":1,"text_id":1,"run_index":0,"text":"caption"}),
        ),
        (
            "text.set_run",
            json!({"expected_revision":revision,"page_id":1,"text_id":1,"run_index":0,"text":"wrong capability"}),
        ),
        (
            "render.capture",
            json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
        ),
    ]
}

#[test]
fn actual_cli_owned_child_text_and_caption_survive_unicode_unchanged_save_and_reopen() -> TestResult
{
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("child-text-cli-λ.cdxml");
    let output = dir.path("child-text-cli-編集.cdxml");
    fs::write(&input, SOURCE)?;
    checked_file(&input)?;
    let listed = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let rows: Value = serde_json::from_slice(&listed.stdout)?;
    metadata(rows.as_array().ok_or("command rows")?, false)?;
    let opened = cli(&[
        request("document.open_cdxml", json!({"path":input})),
        request("document.inspect", json!({})),
    ])?;
    checked(&opened, 1);
    let reopened = cli(&[
        request("document.open_cdxml", json!({"path":input})),
        request("document.save_cdxml", json!({"path":output})),
        request("document.open_cdxml", json!({"path":output})),
        request("document.inspect", json!({})),
    ])?;
    checked(&reopened, 2);
    assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(opened["document"], reopened["document"]);
    checked_file(&input)?;
    checked_file(&output)?;
    Ok(())
}

#[test]
fn initialized_stdio_child_text_refusals_preserve_owned_data_and_exact_unicode_file_output()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("child-text-stdio-λ.cdxml");
    let output = dir.path("child-text-stdio-編集.cdxml");
    fs::write(&input, SOURCE)?;
    checked_file(&input)?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"child-text-wire-test","version":"1"}}}))?;
    let init = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(init["id"], 1);
    assert!(init.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    metadata(
        listed["result"]["tools"].as_array().ok_or("MCP tools")?,
        true,
    )?;
    let opened = client.call(3, "document.open_cdxml", json!({"path":input}))?;
    checked(&opened, 1);
    let stable = client.call(4, "document.inspect", json!({}))?;
    let mut next = 5_u64;
    for (name, params) in refusals(1) {
        client.send(json!({"jsonrpc":"2.0","id":next,"method":"tools/call","params":{"name":name,"arguments":params}}))?;
        let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(reply["id"], next);
        assert_eq!(reply["result"]["isError"], true, "{name}");
        assert_eq!(
            reply["result"]["structuredContent"]["error"]["code"], "unsupported_content",
            "{name}: {reply}"
        );
        next += 1;
        assert_eq!(
            client.call(next, "document.inspect", json!({}))?,
            stable,
            "{name} mutated state"
        );
        next += 1;
    }
    let saved = client.call(next, "document.save_cdxml", json!({"path":output}))?;
    checked(&saved, 1);
    next += 1;
    assert_eq!(saved["file_operation"]["byte_length"], SOURCE.len());
    checked_file(&output)?;
    let reopened = client.call(next, "document.open_cdxml", json!({"path":output}))?;
    checked(&reopened, 2);
    assert_eq!(reopened["document"], stable["document"]);
    checked_file(&input)?;
    checked_file(&output)?;
    client.finish()
}

#[test]
fn authenticated_loopback_child_text_refusals_do_not_authorize_edit_render_or_lossy_destination()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("child-text-socket-λ.cdxml");
    let output = dir.path("child-text-socket-編集.cdxml");
    let mol = dir.path("child-text-sentinel.mol");
    fs::write(&input, SOURCE)?;
    fs::write(&mol, b"keep authored child text\n")?;
    checked_file(&input)?;
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
    checked(&opened["result"], 1);
    let stable = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let (content, model, stamp, file) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert!(!guard.stored_document()?.is_fully_mapped());
        (
            guard.content().clone(),
            guard.stored_document()?.model_snapshot(),
            guard.capture_stamp(),
            guard.ui_file_context_stamp(),
        )
    };
    let mut cases = refusals(1);
    cases.push(("document.save_mol", json!({"path":mol,"overwrite":true})));
    for (name, params) in cases {
        let refused = socket(&mut client, request(name, params))?;
        assert_eq!(
            refused["error"]["code"], "unsupported_content",
            "{name}: {refused}"
        );
        assert_eq!(
            socket(&mut client, request("document.inspect", json!({})))?["result"],
            stable
        );
    }
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &content);
        assert!(Arc::ptr_eq(
            &model,
            &guard.stored_document()?.model_snapshot()
        ));
        assert!(guard.capture_stamp_matches(&stamp));
        assert!(guard.ui_file_context_stamp_matches(&file));
    }
    assert_eq!(fs::read(&mol)?, b"keep authored child text\n");
    let saved = socket(
        &mut client,
        request("document.save_cdxml", json!({"path":output})),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    checked(&saved["result"], 1);
    checked_file(&output)?;
    let reopened = socket(
        &mut client,
        request("document.open_cdxml", json!({"path":output})),
    )?;
    assert!(reopened.get("error").is_none(), "{reopened}");
    checked(&reopened["result"], 2);
    assert_eq!(reopened["result"]["document"], stable["document"]);
    checked_file(&input)?;
    checked_file(&output)?;
    drop(client);
    server.shutdown()?;
    Ok(())
}
