use serde_json::{Value, json};
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
    assert_eq!(tools.len(), 4);
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
    client.finish()?;
    Ok(())
}
