//! CDX Node labels through the actual CLI and authenticated loopback adapter.
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

const RAW: &[u8] = include_bytes!("../../../io/tests/fixtures/cdx-node-labels/connected_pair.cdx");
const WITNESS: &str = include_str!("../../../io/tests/fixtures/cdx-node-labels/cases.json");

fn request(command: &str, params: Value) -> Value {
    json!({"command":command,"params":params})
}

#[test]
fn cdx_node_label_wire_keeps_the_complete_model_and_guards_exact_save() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("atom-wire-原-λ.cdx");
    let cli_output = directory.path("atom-cli-保存-λ.cdx");
    let socket_output = directory.path("atom-socket-保存-β.cdx");
    fs::write(&input, RAW)?;
    let original: Value = serde_json::from_str(WITNESS)?;
    let model = &original["connected_pair"]["model"];
    let open = request("document.open_cdx", json!({"path":input}));
    let inspect = request("document.inspect", json!({}));
    let headless = cli(&[
        open.clone(),
        request("document.save_cdx", json!({"path":cli_output})),
        request("document.open_cdx", json!({"path":cli_output})),
        inspect.clone(),
    ])?;
    assert_eq!(&headless["document"], model);
    assert_eq!(headless["file"]["dirty"], false);
    assert_eq!(fs::read(&cli_output)?, RAW);

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
    assert_eq!(opened["result"]["revision"], 1);
    let before = socket(&mut client, inspect.clone())?;
    for denied in [
        request("render.capture", capture()),
        request(
            "atom.set_label",
            json!({"expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":1,
                   "element":7,"explicit_hydrogens":2}),
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
    let save = request("document.save_cdx", json!({"path":socket_output}));
    assert_eq!(socket(&mut client, save)?["result"]["file"]["dirty"], false);
    assert_eq!(fs::read(&socket_output)?, RAW);
    assert_eq!(fs::read(&input)?, RAW);
    let reopened = socket(
        &mut client,
        request("document.open_cdx", json!({"path":socket_output})),
    )?;
    assert_eq!(&reopened["result"]["document"], model);
    assert_eq!(reopened["result"]["undo_depth"], 0);
    drop(client);
    server.shutdown()?;
    Ok(())
}
