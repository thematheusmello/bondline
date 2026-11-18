//! Render and round-trip qualified CDX labels through the actual command adapters.
use super::{TestResult, capture, cli, socket, support};
use bondline_commands::Engine;
use bondline_mcp::control::LoopbackServer;
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

const RAW: &[u8] = include_bytes!("../../../io/tests/fixtures/cdx-label-preview/water.cdx");
const WITNESSES: &str = include_str!("../../../io/tests/fixtures/cdx-label-preview/cases.json");
const EDITED: &[u8] = include_bytes!("../../../io/tests/fixtures/cdx-label-edit/water_to_nh.cdx");
const EDIT_WITNESSES: &str = include_str!("../../../io/tests/fixtures/cdx-label-edit/cases.json");

fn request(command: &str, params: Value) -> Value {
    json!({"command":command,"params":params})
}

fn connect(server: &LoopbackServer) -> Result<BufReader<TcpStream>, Box<dyn std::error::Error>> {
    let timeout = Duration::from_secs(5);
    let stream = TcpStream::connect_timeout(&server.address(), timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    Ok(client)
}

#[test]
fn cdx_label_preview_wire_matches_real_cpu_gpu_and_preserves_unchanged_save() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("preview-source-原-λ.cdx");
    let output = directory.path("preview-copy-再開-β.cdx");
    fs::write(&input, RAW)?;
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let model = &witnesses["renderable"]["water"]["model"];
    let open = request("document.open_cdx", json!({"path":input}));
    let cpu_request = request("render.capture", capture());
    let mut gpu = capture();
    gpu["backend"] = json!("gpu");
    let cpu = cli(&[open.clone(), cpu_request.clone()])?;
    let gpu = cli(&[open.clone(), request("render.capture", gpu)])?;
    assert_eq!(&cpu["document"], model);
    assert_eq!(&gpu["document"], model);
    assert_eq!(gpu["capture"]["backend"], "gpu");
    let expected_png = cpu["capture"]["png_base64"].as_str().ok_or("missing PNG")?;
    assert!(!expected_png.is_empty());
    assert_eq!(gpu["capture"]["png_base64"].as_str(), Some(expected_png));

    let server = LoopbackServer::bind(0, Arc::new(Mutex::new(Engine::default())), || {})?;
    let mut client = connect(&server)?;
    let opened = socket(&mut client, open)?;
    assert_eq!(&opened["result"]["document"], model);
    let inspect = request("document.inspect", json!({}));
    let before = socket(&mut client, inspect.clone())?;
    for denied in [
        request(
            "atom.set_label",
            json!({"expected_revision":1,"page_id":1,"fragment_id":1,
                  "atom_id":2,"element":7,"explicit_hydrogens":1}),
        ),
        request(
            "page.set_size",
            json!({"page_id":1,"width_pt":612.0,"height_pt":792.0}),
        ),
    ] {
        let reply = socket(&mut client, denied)?;
        assert_eq!(reply["error"]["code"], "unsupported_content", "{reply}");
    }
    assert_eq!(socket(&mut client, inspect)?, before);
    let captured = socket(&mut client, cpu_request.clone())?;
    assert_eq!(
        captured["result"]["capture"]["png_base64"].as_str(),
        Some(expected_png)
    );
    let saved = socket(
        &mut client,
        request("document.save_cdx", json!({"path":output})),
    )?;
    assert_eq!(saved["result"]["file"]["dirty"], false);
    assert_eq!(fs::read(&input)?, RAW);
    assert_eq!(fs::read(&output)?, RAW);
    let reopened = socket(
        &mut client,
        request("document.open_cdx", json!({"path":output})),
    )?;
    assert_eq!(&reopened["result"]["document"], model);
    let captured = socket(&mut client, cpu_request)?;
    assert_eq!(
        captured["result"]["capture"]["png_base64"].as_str(),
        Some(expected_png)
    );
    assert_eq!(captured["result"]["undo_depth"], 0);
    drop(client);
    server.shutdown()?;
    Ok(())
}

#[test]
fn cdx_authored_label_edit_round_trips_exactly_through_cli_and_authenticated_loopback() -> TestResult
{
    let witnesses: Value = serde_json::from_str(EDIT_WITNESSES)?;
    label_edit_round_trip(RAW, EDITED, &witnesses["cases"]["water_to_nh"])
}

#[test]
fn cdx_implicit_element_round_trips_exactly_through_cli_and_authenticated_loopback() -> TestResult {
    let witnesses: Value = serde_json::from_str(include_str!(
        "../../../io/tests/fixtures/cdx-implicit-element/cases.json"
    ))?;
    label_edit_round_trip(
        include_bytes!("../../../io/tests/fixtures/cdx-implicit-element/cyanide_implicit.cdx"),
        include_bytes!("../../../io/tests/fixtures/cdx-implicit-element/cyanide_si.cdx"),
        &witnesses["cases"]["cyanide_c_to_si"],
    )
}

fn label_edit_round_trip(original: &[u8], expected: &[u8], witness: &Value) -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("label-source-原-λ.cdx");
    let cli_output = directory.path("label-cli-保存-λ.cdx");
    let socket_output = directory.path("label-loopback-保存-β.cdx");
    fs::write(&input, original)?;
    let mut params = witness["params"].clone();
    params["expected_revision"] = json!(1);
    let open = request("document.open_cdx", json!({"path":input}));
    let edit = request("atom.set_authored_label", params);
    let undo = request("history.undo", json!({}));
    let redo = request("history.redo", json!({}));
    let render = request("render.capture", capture());
    let reopened = cli(&[
        open.clone(),
        edit.clone(),
        undo.clone(),
        redo.clone(),
        request("document.save_cdx", json!({"path":cli_output})),
        request("document.open_cdx", json!({"path":cli_output})),
        render.clone(),
    ])?;
    assert_eq!(reopened["document"], witness["after_model"]);
    assert_eq!(reopened["revision"], 5);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["redo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(
        reopened["source"],
        json!({"format":"cdx","byte_length":expected.len(),"modified":false})
    );
    assert_eq!(fs::read(&cli_output)?, expected);
    let expected_png = reopened["capture"]["png_base64"]
        .as_str()
        .ok_or("missing edited PNG")?;
    assert!(!expected_png.is_empty());

    let server = LoopbackServer::bind(0, Arc::new(Mutex::new(Engine::default())), || {})?;
    let mut client = connect(&server)?;
    let opened = socket(&mut client, open)?;
    assert_eq!(opened["result"]["document"], witness["before_model"]);
    let edited = socket(&mut client, edit)?;
    assert_eq!(edited["result"]["document"], witness["after_model"]);
    assert_eq!(edited["result"]["changed"], true);
    assert_eq!(edited["result"]["undo_depth"], 1);
    assert_eq!(edited["result"]["file"]["dirty"], true);
    let selection = request(
        "ui.selection",
        json!({"action":"all","expected_revision":2,"page_id":1}),
    );
    let crossed = directory.path("label-lossy.cdxml");
    let cross_format = request("document.save_cdxml", json!({"path":crossed}));
    for denied in [selection, cross_format] {
        let reply = socket(&mut client, denied)?;
        assert_eq!(reply["error"]["code"], "unsupported_content", "{reply}");
    }
    assert!(!crossed.exists());
    let undone = socket(&mut client, undo)?;
    assert_eq!(undone["result"]["document"], witness["before_model"]);
    assert_eq!(undone["result"]["file"]["dirty"], false);
    let redone = socket(&mut client, redo)?;
    assert_eq!(redone["result"]["document"], witness["after_model"]);
    assert_eq!(redone["result"]["file"]["dirty"], true);
    let saved = socket(
        &mut client,
        request("document.save_cdx", json!({"path":socket_output})),
    )?;
    assert_eq!(saved["result"]["file"]["dirty"], false);
    assert_eq!(
        saved["result"]["source"],
        json!({"format":"cdx","byte_length":original.len(),"modified":true})
    );
    assert_eq!(fs::read(&input)?, original);
    assert_eq!(fs::read(&socket_output)?, expected);
    let opened = socket(
        &mut client,
        request("document.open_cdx", json!({"path":socket_output})),
    )?;
    assert_eq!(opened["result"]["document"], witness["after_model"]);
    assert_eq!(opened["result"]["undo_depth"], 0);
    let rendered = socket(&mut client, render)?;
    assert_eq!(
        rendered["result"]["capture"]["png_base64"].as_str(),
        Some(expected_png)
    );
    drop(client);
    server.shutdown()?;
    Ok(())
}
