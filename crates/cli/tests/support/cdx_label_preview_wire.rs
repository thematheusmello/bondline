//! Render and Save through the actual CLI and authenticated command adapter.
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

fn request(command: &str, params: Value) -> Value {
    json!({"command":command,"params":params})
}

#[test]
fn cdx_label_preview_wire_matches_real_cpu_gpu_and_preserves_readonly_save() -> TestResult {
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
    let timeout = Duration::from_secs(5);
    let stream = TcpStream::connect_timeout(&server.address(), timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let opened = socket(&mut client, open)?;
    assert_eq!(&opened["result"]["document"], model);
    let inspect = request("document.inspect", json!({}));
    let before = socket(&mut client, inspect.clone())?;
    for denied in [
        request(
            "atom.set_authored_label",
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
