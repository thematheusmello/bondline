use serde_json::{Value, json};
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{Receiver, channel},
    thread::JoinHandle,
    time::Duration,
};

struct Client {
    process: Child,
    input: Option<ChildStdin>,
    responses: Receiver<Result<Value, String>>,
    reader: Option<JoinHandle<()>>,
}

impl Client {
    fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = process.stdin.take().ok_or("missing MCP input")?;
        let output = process.stdout.take().ok_or("missing MCP output")?;
        let (sender, responses) = channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let value = line.map_err(|error| error.to_string()).and_then(|line| {
                    serde_json::from_str(&line).map_err(|error| error.to_string())
                });
                if sender.send(value).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            process,
            input: Some(input),
            responses,
            reader: Some(reader),
        })
    }

    fn send(&mut self, value: Value) -> Result<(), Box<dyn std::error::Error>> {
        let input = self.input.as_mut().ok_or("MCP input closed")?;
        serde_json::to_writer(&mut *input, &value)?;
        input.write_all(b"\n")?;
        input.flush()?;
        Ok(())
    }

    fn call(
        &mut self,
        id: u64,
        method: &str,
        params: Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))?;
        let response = self.responses.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(response["id"], id, "unexpected MCP response: {response}");
        Ok(response)
    }

    fn finish(mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.input.take();
        for _ in 0..1000 {
            if let Some(status) = self.process.try_wait()? {
                assert!(status.success(), "MCP did not shut down cleanly: {status}");
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Err("MCP did not exit within ten seconds after EOF".into())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.process.kill();
        let _ = self.process.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[test]
fn public_mcp_wire_messages_discover_edit_and_recover_from_oversized_input()
-> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::start()?;
    let initialize = client.call(
        1,
        "initialize",
        json!({
            "protocolVersion":"2025-11-25", "capabilities":{},
            "clientInfo":{"name":"bondline-wire-test","version":"1.0"}
        }),
    )?;
    assert_eq!(initialize["result"]["serverInfo"]["name"], "bondline");
    assert_eq!(initialize["result"]["protocolVersion"], "2025-11-25");
    assert!(initialize["result"]["capabilities"]["tools"].is_object());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let list = client.call(2, "tools/list", json!({}))?;
    let tools = list["result"]["tools"].as_array().ok_or("missing tools")?;
    let mut names = tools
        .iter()
        .map(|tool| tool["name"].as_str().ok_or("tool name"))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "bond.create",
            "document.close",
            "document.inspect",
            "document.open_cdxml",
            "document.save_cdxml",
            "history.redo",
            "history.undo",
            "page.set_size",
            "render.capture",
            "ui.controls",
            "ui.file"
        ]
    );
    assert!(tools.iter().any(|tool| tool["name"] == "document.close"
        && tool["inputSchema"]["additionalProperties"] == false
        && tool["inputSchema"]["required"] == json!(["expected_revision"])
        && tool["annotations"]["readOnlyHint"] == false));
    assert!(tools.iter().any(|tool| tool["name"] == "ui.controls"
        && tool["inputSchema"]["additionalProperties"] == false
        && tool["annotations"]["readOnlyHint"] == true));
    assert!(tools.iter().any(|tool| tool["name"] == "ui.file"
        && tool["inputSchema"]["additionalProperties"] == false
        && tool["annotations"]["readOnlyHint"] == false));
    assert!(tools.iter().any(|tool| tool["name"] == "page.set_size"
        && tool["inputSchema"]["additionalProperties"] == false));
    let edit = client.call(
        3,
        "tools/call",
        json!({"name":"page.set_size","arguments":{"page_id":1,"width_pt":612,"height_pt":792}}),
    )?;
    assert_eq!(edit["result"]["isError"], false);
    assert_eq!(
        edit["result"]["structuredContent"]["document"]["pages"][0]["size"]["width_pt"],
        612.0
    );
    let undo = client.call(
        4,
        "tools/call",
        json!({"name":"history.undo","arguments":{}}),
    )?;
    let width = undo["result"]["structuredContent"]["document"]["pages"][0]["size"]["width_pt"]
        .as_f64()
        .ok_or("missing page width")?;
    assert!((width - 595.2755905511812).abs() < 1e-10);
    let invalid = client.call(
        5,
        "tools/call",
        json!({"name":"page.set_size","arguments":{"page_id":1,"width_pt":0,"height_pt":72}}),
    )?;
    assert_eq!(invalid["result"]["isError"], true);
    assert_eq!(
        invalid["result"]["structuredContent"]["error"]["code"],
        "invalid_params"
    );
    let unknown = client.call(6, "tools/call", json!({"name":"missing","arguments":{}}))?;
    assert_eq!(unknown["error"]["code"], -32601);
    let input = client.input.as_mut().ok_or("missing input")?;
    input.write_all(&vec![b' '; 1024 * 1024 + 1])?;
    input.write_all(b"\n")?;
    input.flush()?;
    let inspect = client.call(
        7,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    assert_eq!(inspect["result"]["structuredContent"]["revision"], 2);
    assert!(tools.iter().any(|tool| tool["name"] == "render.capture"
        && tool["annotations"]["readOnlyHint"] == true
        && tool["inputSchema"]["properties"]["width_px"]["maximum"] == 2048));
    let capture = client.call(
        8,
        "tools/call",
        json!({"name":"render.capture","arguments":{
            "page_id":1,"width_px":8,"height_px":8,"backend":"cpu"
        }}),
    )?;
    let capture = &capture["result"]["structuredContent"];
    assert_eq!(capture["revision"], 2);
    assert_eq!(capture["changed"], false);
    assert_eq!(capture["redo_depth"], 1);
    assert_eq!(capture["capture"]["mime_type"], "image/png");
    assert!(
        capture["capture"]["png_base64"]
            .as_str()
            .ok_or("missing MCP PNG")?
            .starts_with("iVBORw0KGgo")
    );
    let bond = client.call(
        9,
        "tools/call",
        json!({"name":"bond.create","arguments":{
            "page_id":1,"begin_x_pt":10,"begin_y_pt":20.5,"end_x_pt":50,"end_y_pt":20.5
        }}),
    )?;
    assert_eq!(bond["result"]["isError"], false);
    let created = &bond["result"]["structuredContent"];
    assert_eq!(created["revision"], 3);
    assert_eq!(created["redo_depth"], 0);
    assert_eq!(
        created["document"]["pages"][0]["fragments"][0]["atoms"][0]["explicit_hydrogens"],
        3
    );
    let invalid = client.call(
        10,
        "tools/call",
        json!({"name":"bond.create","arguments":{
            "page_id":1,"begin_x_pt":10,"begin_y_pt":20.5,"end_x_pt":10,"end_y_pt":20.5
        }}),
    )?;
    assert_eq!(invalid["result"]["isError"], true);
    assert_eq!(
        invalid["result"]["structuredContent"]["error"]["code"],
        "invalid_params"
    );
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("mcp.cdxml");
    // Use a fresh known native source so the same independent geometry oracle can inspect the output.
    let original = directory.path("original.cdxml");
    std::fs::write(
        &original,
        include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml"),
    )?;
    let opened = client.call(
        11,
        "tools/call",
        json!({"name":"document.open_cdxml","arguments":{"path":original,"discard_unsaved":true}}),
    )?;
    assert_eq!(opened["result"]["isError"], false);
    assert_eq!(opened["result"]["structuredContent"]["undo_depth"], 0);
    let saved = client.call(
        12,
        "tools/call",
        json!({"name":"document.save_cdxml","arguments":{"path":path}}),
    )?;
    assert_eq!(saved["result"]["isError"], false);
    assert_eq!(saved["result"]["structuredContent"]["file"]["dirty"], false);
    file_support::independently_read_file(&path, 1)?;
    assert_eq!(std::fs::read(&path)?, std::fs::read(&original)?);
    let rejected = client.call(
        13,
        "tools/call",
        json!({"name":"document.save_cdxml","arguments":{"path":path}}),
    )?;
    assert_eq!(rejected["result"]["isError"], true);
    assert_eq!(
        rejected["result"]["structuredContent"]["error"]["code"],
        "already_exists"
    );
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_close_policy_supports_decision_cancel_discard_and_errors_without_ending_session()
-> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::start()?;
    client.call(
        1,
        "initialize",
        json!({
            "protocolVersion":"2025-11-25","capabilities":{},
            "clientInfo":{"name":"close-wire-test","version":"1"}
        }),
    )?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let draw = client.call(
        2,
        "tools/call",
        json!({"name":"bond.create","arguments":{
            "page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200
        }}),
    )?;
    let document = draw["result"]["structuredContent"]["document"].clone();
    assert_eq!(draw["result"]["structuredContent"]["close_epoch"], 0);
    let requested = client.call(
        3,
        "tools/call",
        json!({"name":"document.close","arguments":{"expected_revision":1}}),
    )?;
    let requested = &requested["result"]["structuredContent"];
    assert_eq!(requested["close"], "needs_decision");
    assert_eq!(requested["close_operation"], "needs_decision");
    assert_eq!(requested["close_epoch"], 1);
    assert_eq!(requested["changed"], false);
    assert_eq!(requested["revision"], 1);
    assert_eq!(requested["undo_depth"], 1);
    assert_eq!(requested["document"], document);
    let stale = client.call(
        4,
        "tools/call",
        json!({"name":"document.close","arguments":{"expected_revision":0}}),
    )?;
    assert_eq!(stale["result"]["isError"], true);
    assert_eq!(
        stale["result"]["structuredContent"]["error"]["code"],
        "stale_revision"
    );
    for (id, action, state, epoch) in [(5, "cancel", "open", 2), (6, "discard", "approved", 3)] {
        let response = client.call(
            id,
            "tools/call",
            json!({"name":"document.close","arguments":{
                "expected_revision":1,"action":action
            }}),
        )?;
        assert_eq!(response["result"]["isError"], false);
        let result = &response["result"]["structuredContent"];
        assert_eq!(result["close"], state);
        assert_eq!(result["close_operation"], state);
        assert_eq!(result["close_epoch"], epoch);
        assert_eq!(result["revision"], 1);
        assert_eq!(result["undo_depth"], 1);
        assert_eq!(result["document"], document);
        assert_eq!(result["file"]["dirty"], true);
    }
    let inspect = client.call(
        7,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    let inspect = &inspect["result"]["structuredContent"];
    assert_eq!(inspect["close"], "approved");
    assert_eq!(inspect["close_epoch"], 3);
    assert!(inspect.get("close_operation").is_none());
    for (id, arguments) in [
        (8, json!({"expected_revision":1,"action":"quit"})),
        (9, json!({"expected_revision":1,"force":true})),
        (10, json!({})),
    ] {
        let invalid = client.call(
            id,
            "tools/call",
            json!({"name":"document.close","arguments":arguments}),
        )?;
        assert_eq!(invalid["result"]["isError"], true);
        assert_eq!(
            invalid["result"]["structuredContent"]["error"]["code"],
            "invalid_params"
        );
    }
    let inspected = client.call(
        11,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    assert_eq!(inspected["result"]["structuredContent"], *inspect);
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_file_presentation_is_unavailable_without_a_host_and_cannot_forge_acknowledgements()
-> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::start()?;
    client.call(
        1,
        "initialize",
        json!({
            "protocolVersion":"2025-11-25","capabilities":{},
            "clientInfo":{"name":"ui-contract-wire-test","version":"1"}
        }),
    )?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let queried = client.call(
        2,
        "tools/call",
        json!({"name":"ui.controls","arguments":{}}),
    )?;
    let queried = &queried["result"]["structuredContent"];
    assert_eq!(
        queried["ui_file"],
        json!({"available":false,"epoch":0,"request":null})
    );
    assert_eq!(queried["controls"].as_array().ok_or("controls")?.len(), 9);
    assert!(queried.get("ui_file_operation").is_none());
    for (id, arguments, code) in [
        (
            3,
            json!({"expected_revision":0,"action":"open"}),
            "ui_unavailable",
        ),
        (
            4,
            json!({"expected_revision":0,"action":"save"}),
            "ui_unavailable",
        ),
        (
            5,
            json!({"expected_revision":0,"action":"save_as"}),
            "ui_unavailable",
        ),
        (
            6,
            json!({"expected_revision":0,"action":"cancel","request_id":1}),
            "ui_unavailable",
        ),
        (
            7,
            json!({"expected_revision":0,"action":"open","available":true}),
            "invalid_params",
        ),
        (
            8,
            json!({"expected_revision":0,"action":"open","phase":"accepted"}),
            "invalid_params",
        ),
        (
            9,
            json!({"expected_revision":0,"action":"save","path":"must-not-write.cdxml"}),
            "invalid_params",
        ),
    ] {
        let response = client.call(
            id,
            "tools/call",
            json!({"name":"ui.file","arguments":arguments}),
        )?;
        assert_eq!(response["result"]["isError"], true);
        assert_eq!(
            response["result"]["structuredContent"]["error"]["code"],
            code
        );
    }
    let inspected = client.call(
        10,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    let inspected = &inspected["result"]["structuredContent"];
    assert_eq!(inspected["ui_file"], queried["ui_file"]);
    assert_eq!(inspected["document"], queried["document"]);
    assert_eq!(inspected["revision"], 0);
    assert_eq!(inspected["undo_depth"], 0);
    assert_eq!(inspected["file"]["dirty"], false);
    assert!(inspected.get("controls").is_none());
    assert!(inspected.get("file_operation").is_none());
    assert!(inspected.get("ui_file_operation").is_none());
    client.finish()?;
    Ok(())
}
