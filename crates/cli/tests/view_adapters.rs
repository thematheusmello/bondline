use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{Receiver, channel},
    thread::JoinHandle,
    time::{Duration, Instant},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn request(params: Value) -> Value {
    json!({"command":"ui.view","params":params})
}

fn operation(
    value: &Value,
    percent: Option<f64>,
    center: Option<[f64; 2]>,
    epoch: u64,
    changed: bool,
) {
    assert_eq!(value["changed"], false);
    assert_eq!(
        value["ui_view_operation"],
        json!({
            "zoom_percent":percent,"center_x_pt":center.map(|p|p[0]),
            "center_y_pt":center.map(|p|p[1]),"epoch":epoch,"changed":changed
        })
    );
}

fn without_operation(mut value: Value) -> TestResult<Value> {
    value
        .as_object_mut()
        .ok_or("command result object")?
        .remove("ui_view_operation");
    Ok(value)
}

fn closed_view_schema(schema: &Value) -> TestResult {
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["unevaluatedProperties"], false);
    let variants = schema["oneOf"]
        .as_array()
        .ok_or("tagged view action schemas")?;
    assert_eq!(variants.len(), 7);
    let mut actions = std::collections::BTreeSet::new();
    for variant in variants {
        assert_eq!(variant["type"], "object");
        assert_eq!(variant["additionalProperties"], false);
        let properties = variant["properties"]
            .as_object()
            .ok_or("action properties")?;
        let action = properties["action"]["const"].as_str().ok_or("action tag")?;
        assert!(actions.insert(action), "duplicate action branch");
        let fields: &[&str] = match action {
            "inspect" | "fit" | "actual" => &["action"],
            "set_zoom" => &["action", "percent"],
            "magnify" | "reduce" => &["action", "width_px", "height_px"],
            "pan" => &["action", "center_x_pt", "center_y_pt"],
            _ => return Err("unexpected view action".into()),
        };
        let expected = fields
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            properties
                .keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>(),
            expected
        );
        let required = variant["required"]
            .as_array()
            .ok_or("required action fields")?
            .iter()
            .map(|value| value.as_str().ok_or("required field name"))
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
        assert_eq!(required, expected);
    }
    Ok(())
}

fn run(requests: &[Value]) -> std::io::Result<std::process::Output> {
    let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
    process.arg("run");
    for request in requests {
        process.arg("--cmd").arg(request.to_string());
    }
    process.output()
}

#[test]
fn cli_discovers_and_executes_window_view_without_changing_the_document() -> TestResult {
    let listed = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(listed.status.success());
    let entries: Value = serde_json::from_slice(&listed.stdout)?;
    let entry = entries
        .as_array()
        .ok_or("registry array")?
        .iter()
        .find(|entry| entry["name"] == "ui.view")
        .ok_or("registered ui.view")?;
    assert_eq!(
        entry["read_only"], false,
        "the tool can change window state"
    );
    closed_view_schema(&entry["input_schema"])?;

    let initial = run(&[json!({"command":"document.inspect","params":{}})])?;
    assert!(initial.status.success());
    let initial: Value = serde_json::from_slice(&initial.stdout)?;
    assert!(initial.get("ui_view_operation").is_none());
    let changed = run(&[
        request(json!({"action":"set_zoom","percent":175.5})),
        request(json!({"action":"pan","center_x_pt":72,"center_y_pt":144})),
        request(json!({"action":"inspect"})),
    ])?;
    assert!(
        changed.status.success(),
        "{}",
        String::from_utf8_lossy(&changed.stderr)
    );
    assert!(changed.stderr.is_empty());
    let changed: Value = serde_json::from_slice(&changed.stdout)?;
    operation(&changed, Some(175.5), Some([72.0, 144.0]), 2, false);
    assert_eq!(without_operation(changed)?, initial);
    for params in [
        json!({"action":"inspect","percent":100}),
        json!({"action":"fit","unused":true}),
        json!({"action":"actual","width_px":1}),
        json!({"action":"set_zoom","percent":0}),
    ] {
        let output = run(&[request(params)])?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(error["error"]["code"], "invalid_params");
    }
    Ok(())
}

struct Client {
    process: Child,
    input: Option<ChildStdin>,
    replies: Receiver<Result<Value, String>>,
    reader: Option<JoinHandle<()>>,
}
impl Client {
    fn start() -> TestResult<Self> {
        let mut process = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = process.stdin.take().ok_or("stdio input")?;
        let output = process.stdout.take().ok_or("stdio output")?;
        let (sender, replies) = channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let value = line
                    .map_err(|e| e.to_string())
                    .and_then(|line| serde_json::from_str(&line).map_err(|e| e.to_string()));
                if sender.send(value).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            process,
            input: Some(input),
            replies,
            reader: Some(reader),
        })
    }
    fn send(&mut self, value: Value) -> TestResult {
        let input = self.input.as_mut().ok_or("closed stdio")?;
        serde_json::to_writer(&mut *input, &value)?;
        input.write_all(b"\n")?;
        input.flush()?;
        Ok(())
    }
    fn call(&mut self, id: u64, method: &str, params: Value) -> TestResult<Value> {
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))?;
        let response = self.replies.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(response["id"], id);
        Ok(response)
    }
    fn tool(&mut self, id: u64, name: &str, arguments: Value) -> TestResult<Value> {
        self.call(id, "tools/call", json!({"name":name,"arguments":arguments}))
    }
    fn finish(mut self) -> TestResult {
        self.input.take();
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if let Some(status) = self.process.try_wait()? {
                assert!(status.success(), "stdio exit: {status}");
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Err("stdio did not exit after EOF".into())
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
fn stdio_view_schema_and_exact_results_share_one_unchanged_authored_session() -> TestResult {
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"view-window-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let list = client.call(2, "tools/list", json!({}))?;
    let tool = list["result"]["tools"]
        .as_array()
        .ok_or("tools array")?
        .iter()
        .find(|tool| tool["name"] == "ui.view")
        .ok_or("ui.view MCP tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    closed_view_schema(&tool["inputSchema"])?;
    let initial = client.tool(3, "document.inspect", json!({}))?;
    assert_eq!(initial["result"]["isError"], false);
    let initial = initial["result"]["structuredContent"].clone();
    assert!(initial.get("ui_view_operation").is_none());
    let mut id = 4;
    for (params, percent, center, epoch, changed) in [
        (json!({"action":"inspect"}), None, None, 0, false),
        (json!({"action":"actual"}), Some(100.0), None, 1, true),
        (json!({"action":"actual"}), Some(100.0), None, 1, false),
        (
            json!({"action":"pan","center_x_pt":12,"center_y_pt":24}),
            Some(100.0),
            Some([12.0, 24.0]),
            2,
            true,
        ),
    ] {
        let response = client.tool(id, "ui.view", params)?;
        id += 1;
        assert_eq!(response["result"]["isError"], false, "{response}");
        let result = &response["result"]["structuredContent"];
        operation(result, percent, center, epoch, changed);
        let text: Value = serde_json::from_str(
            response["result"]["content"][0]["text"]
                .as_str()
                .ok_or("MCP text")?,
        )?;
        assert_eq!(&text, result);
        assert_eq!(without_operation(result.clone())?, initial);
    }
    let invalid = client.tool(id, "ui.view", json!({"action":"inspect","percent":100}))?;
    id += 1;
    assert_eq!(invalid["result"]["isError"], true);
    assert_eq!(
        invalid["result"]["structuredContent"]["error"]["code"],
        "invalid_params"
    );
    let after = client.tool(id, "ui.view", json!({"action":"inspect"}))?;
    id += 1;
    operation(
        &after["result"]["structuredContent"],
        Some(100.0),
        Some([12.0, 24.0]),
        2,
        false,
    );
    let normal = client.tool(id, "document.inspect", json!({}))?;
    assert_eq!(normal["result"]["structuredContent"], initial);
    client.finish()
}
