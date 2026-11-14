use serde_json::{Value, json};
#[path = "support/cdx.rs"]
mod cdx_support;
#[path = "support/chain.rs"]
mod chain_support;
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;
#[path = "support/mol.rs"]
mod mol_support;
#[path = "../../../xtask/oracles/sdf_record_test_support.rs"]
mod sdf_record_support;
#[path = "../../../xtask/oracles/sdf_test_support.rs"]
mod sdf_support;
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
fn stdio_explicit_sdf_record_queries_disclose_fields_without_mutation_or_automatic_replay()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sdf-query-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let tools = client.call(2, "tools/list", json!({}))?;
    let tool = tools["result"]["tools"]
        .as_array()
        .ok_or("tools")?
        .iter()
        .find(|tool| tool["name"] == "sdf.record.inspect")
        .ok_or("SDF query")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], true);
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    let mut id = 3;
    for (case, (source, mixed)) in [
        (
            include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf").as_slice(),
            false,
        ),
        (
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            true,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let path = directory.path(if mixed {
            "stdio-query-mixed.sdf"
        } else {
            "stdio-query-ordered.sdf"
        });
        std::fs::write(&path, source)?;
        client.call(
            id,
            "tools/call",
            json!({"name":"document.open_sdf","arguments":{"path":path}}),
        )?;
        id += 1;
        let before = client.call(
            id,
            "tools/call",
            json!({"name":"document.inspect","arguments":{}}),
        )?;
        id += 1;
        assert!(
            before["result"]["structuredContent"]
                .get("sdf_record")
                .is_none()
        );
        for record_id in [1, 2] {
            let response=client.call(id,"tools/call",json!({"name":"sdf.record.inspect","arguments":{"expected_revision":case+1,"record_id":record_id}}))?;
            id += 1;
            assert_eq!(response["result"]["isError"], false, "{response}");
            let mut result = response["result"]["structuredContent"].clone();
            assert_eq!(
                result["sdf_record"]["fields"],
                sdf_record_support::expected_fields(mixed, record_id)
            );
            assert_eq!(result["sdf_record"]["input_revision"], case + 1);
            assert_eq!(result["sdf_record"]["session_generation"], case + 1);
            sdf_record_support::assert_json_budget(&result["sdf_record"], None)?;
            result
                .as_object_mut()
                .ok_or("query result")?
                .remove("sdf_record");
            assert_eq!(result, before["result"]["structuredContent"]);
        }
        let invalid=client.call(id,"tools/call",json!({"name":"sdf.record.inspect","arguments":{"expected_revision":case+1,"record_id":3}}))?;
        id += 1;
        assert_eq!(invalid["result"]["isError"], true);
        assert_eq!(
            invalid["result"]["structuredContent"]["error"]["code"],
            "not_found"
        );
        let after = client.call(
            id,
            "tools/call",
            json!({"name":"document.inspect","arguments":{}}),
        )?;
        id += 1;
        assert_eq!(
            after["result"]["structuredContent"],
            before["result"]["structuredContent"]
        );
        sdf_support::independently_read_sdf(&path, source, mixed)?;
    }
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_sdf_files_preserve_the_complete_collection_and_recover_from_file_refusals()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sdf-file-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let mut id = 2;
    for (source, mixed) in [
        (
            include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf").as_slice(),
            false,
        ),
        (
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            true,
        ),
    ] {
        let input = directory.path(if mixed {
            "stdio-mixed.sdf"
        } else {
            "stdio-ordered.sd"
        });
        let output = directory.path(if mixed {
            "stdio-mixed-λ.data"
        } else {
            "stdio-ordered-λ.data"
        });
        let bad = directory.path("bad-later.sdf");
        std::fs::write(&input, source)?;
        std::fs::write(
            &bad,
            [
                &source[..if mixed { 462 } else { 424 }],
                b"bad second record\n",
            ]
            .concat(),
        )?;
        for (name, arguments) in [
            ("document.open_sdf", json!({"path":input})),
            ("document.inspect", json!({})),
            ("document.save_sdf", json!({"path":output})),
            ("document.open_sdf", json!({"path":output})),
        ] {
            let response =
                client.call(id, "tools/call", json!({"name":name,"arguments":arguments}))?;
            id += 1;
            assert_eq!(response["result"]["isError"], false, "{response}");
            let result = &response["result"]["structuredContent"];
            assert_eq!(result["document"], json!(null));
            assert_eq!(result["collection"], sdf_support::expected_summary(mixed));
            assert_eq!(result["file"]["format"], "sdf");
            assert_eq!(result["file"]["dirty"], false);
        }
        sdf_support::independently_read_sdf(&output, source, mixed)?;
        let baseline = client.call(
            id,
            "tools/call",
            json!({"name":"document.inspect","arguments":{}}),
        )?;
        id += 1;
        for (name, arguments, code) in [
            ("document.open_sdf", json!({"path":bad}), "invalid_sdf"),
            (
                "document.save_sdf",
                json!({"path":output}),
                "already_exists",
            ),
            (
                "document.save_mol",
                json!({"path":directory.path("must-not-save.mol")}),
                "unsupported_content",
            ),
        ] {
            let response =
                client.call(id, "tools/call", json!({"name":name,"arguments":arguments}))?;
            id += 1;
            assert_eq!(response["result"]["isError"], true, "{response}");
            assert_eq!(
                response["result"]["structuredContent"]["error"]["code"],
                code
            );
            let inspect = client.call(
                id,
                "tools/call",
                json!({"name":"document.inspect","arguments":{}}),
            )?;
            id += 1;
            assert_eq!(
                inspect["result"]["structuredContent"],
                baseline["result"]["structuredContent"]
            );
        }
        assert!(!directory.path("must-not-save.mol").exists());
        assert_eq!(std::fs::read(&output)?, source);
    }
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_tools_extend_the_native_graph_with_typed_policy_and_saved_consumer_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("mcp-chain.cdxml");
    let mut client = Client::start()?;
    client.call(
        1,
        "initialize",
        json!({"protocolVersion":"2025-11-25","capabilities":{},
        "clientInfo":{"name":"bondline-extension-test","version":"1.0"}}),
    )?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let tools = client.call(2, "tools/list", json!({}))?;
    let tool = tools["result"]["tools"]
        .as_array()
        .ok_or("tools")?
        .iter()
        .find(|tool| tool["name"] == "bond.extend")
        .ok_or("extend tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    let required = tool["inputSchema"]["required"]
        .as_array()
        .ok_or("required")?;
    assert!(
        required.contains(&json!("hydrogen_policy"))
            && required.contains(&json!("expected_revision"))
    );
    let created = client.call(
        3,
        "tools/call",
        json!({"name":"bond.create","arguments":chain_support::create()}),
    )?;
    assert_eq!(created["result"]["isError"], false);
    let extended = client.call(
        4,
        "tools/call",
        json!({"name":"bond.extend","arguments":chain_support::extend(1,2,180.0,240.0)}),
    )?;
    assert_eq!(extended["result"]["isError"], false);
    let state = extended["result"]["structuredContent"].clone();
    assert_eq!(state["revision"], 2);
    assert_eq!(
        state["document"]["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"],
        2
    );
    let stale = client.call(
        5,
        "tools/call",
        json!({"name":"bond.extend","arguments":chain_support::extend(1,3,220.0,280.0)}),
    )?;
    assert_eq!(stale["result"]["isError"], true);
    assert_eq!(
        stale["result"]["structuredContent"]["error"]["code"],
        "stale_revision"
    );
    let inspected = client.call(
        6,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    assert_eq!(
        inspected["result"]["structuredContent"]["document"],
        state["document"]
    );
    let saved = client.call(
        7,
        "tools/call",
        json!({"name":"document.save_cdxml","arguments":{"path":path}}),
    )?;
    assert_eq!(saved["result"]["isError"], false);
    chain_support::independently_read(&path, 3)?;
    let undone = client.call(
        8,
        "tools/call",
        json!({"name":"history.undo","arguments":{}}),
    )?;
    assert_eq!(
        undone["result"]["structuredContent"]["document"],
        created["result"]["structuredContent"]["document"]
    );
    let redone = client.call(
        9,
        "tools/call",
        json!({"name":"history.redo","arguments":{}}),
    )?;
    assert_eq!(
        redone["result"]["structuredContent"]["document"],
        state["document"]
    );
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_cdx_retains_binary_source_and_state_across_file_and_schema_refusals()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let source = directory.path("original-λ.mol");
    let copy = directory.path("copy.cdx");
    let malformed = directory.path("malformed.cdx");
    std::fs::write(&source, cdx_support::SOURCE)?;
    std::fs::write(&malformed, b"not CDX")?;
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"cdx-wire-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let opened = client.call(
        2,
        "tools/call",
        json!({"name":"document.open_cdx","arguments":{"path":source}}),
    )?;
    assert_eq!(opened["result"]["isError"], false, "{opened}");
    cdx_support::assert_state(&opened["result"]["structuredContent"])?;
    let saved = client.call(
        3,
        "tools/call",
        json!({"name":"document.save_cdx","arguments":{"path":copy}}),
    )?;
    assert_eq!(saved["result"]["isError"], false, "{saved}");
    let saved = &saved["result"]["structuredContent"];
    cdx_support::assert_state(saved)?;
    assert_eq!(saved["file_operation"]["format"], "cdx");
    cdx_support::independently_read_saved(&copy)?;
    let baseline = client.call(
        4,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    let mut id = 5;
    for (name, arguments, code) in [
        (
            "document.open_cdx",
            json!({"path":source,"unknown":true}),
            "invalid_params",
        ),
        ("document.save_cdx", json!({"path":copy}), "already_exists"),
        (
            "document.save_mol",
            json!({"path":copy,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "document.open_cdx",
            json!({"path":malformed}),
            "invalid_document",
        ),
        (
            "render.capture",
            json!({"page_id":1,"backend":"cpu","width_px":80,"height_px":80}),
            "unsupported_content",
        ),
    ] {
        let failure = client.call(id, "tools/call", json!({"name":name,"arguments":arguments}))?;
        assert_eq!(failure["result"]["isError"], true, "{failure}");
        assert_eq!(
            failure["result"]["structuredContent"]["error"]["code"],
            code
        );
        let current = client.call(
            id + 1,
            "tools/call",
            json!({"name":"document.inspect","arguments":{}}),
        )?;
        assert_eq!(
            current["result"]["structuredContent"],
            baseline["result"]["structuredContent"]
        );
        id += 2;
    }
    client.call(
        id,
        "tools/call",
        json!({"name":"page.set_size","arguments":{"page_id":1,"width_pt":612,"height_pt":792}}),
    )?;
    let failed = client.call(
        id + 1,
        "tools/call",
        json!({"name":"document.open_cdx","arguments":{"path":source}}),
    )?;
    assert_eq!(
        failed["result"]["structuredContent"]["error"]["code"],
        "unsaved_changes"
    );
    let restored = client.call(
        id + 2,
        "tools/call",
        json!({"name":"history.undo","arguments":{}}),
    )?;
    cdx_support::assert_state(&restored["result"]["structuredContent"])?;
    let saved = client.call(
        id + 3,
        "tools/call",
        json!({"name":"document.save_cdx","arguments":{"path":copy,"overwrite":true}}),
    )?;
    assert_eq!(saved["result"]["isError"], false, "{saved}");
    cdx_support::independently_read_saved(&copy)?;
    client.finish()?;
    Ok(())
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
            "bond.extend",
            "document.close",
            "document.inspect",
            "document.open_cdx",
            "document.open_cdxml",
            "document.open_mol",
            "document.open_sdf",
            "document.save_cdx",
            "document.save_cdxml",
            "document.save_mol",
            "document.save_sdf",
            "history.redo",
            "history.undo",
            "page.set_size",
            "render.capture",
            "sdf.record.inspect",
            "ui.controls",
            "ui.file"
        ]
    );
    for (name, permission) in [
        ("document.open_cdx", "discard_unsaved"),
        ("document.save_cdx", "overwrite"),
        ("document.open_sdf", "discard_unsaved"),
        ("document.save_sdf", "overwrite"),
    ] {
        let tool = tools
            .iter()
            .find(|tool| tool["name"] == name)
            .ok_or("CDX tool")?;
        assert_eq!(tool["annotations"]["readOnlyHint"], false);
        let schema = &tool["inputSchema"];
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], json!(["path"]));
        let properties = schema["properties"].as_object().ok_or("CDX properties")?;
        assert_eq!(
            properties
                .keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from(["path", permission])
        );
        assert_eq!(properties[permission]["type"], "boolean");
        assert_eq!(properties[permission]["default"], false);
    }
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
    for name in ["document.open_mol", "document.save_mol"] {
        let tool = tools
            .iter()
            .find(|tool| tool["name"] == name)
            .ok_or("MOL tool")?;
        assert_eq!(tool["annotations"]["readOnlyHint"], false);
        let schema = &tool["inputSchema"];
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], json!(["path"]));
        let properties = schema["properties"].as_object().ok_or("MOL properties")?;
        let keys: std::collections::BTreeSet<_> = properties.keys().map(String::as_str).collect();
        if name == "document.open_mol" {
            assert_eq!(
                keys,
                std::collections::BTreeSet::from([
                    "path",
                    "discard_unsaved",
                    "points_per_unit",
                    "origin_x_pt",
                    "origin_y_pt"
                ])
            );
            assert_eq!(properties["discard_unsaved"]["default"], false);
            for (key, default) in [
                ("points_per_unit", 24.0),
                ("origin_x_pt", 72.0),
                ("origin_y_pt", 72.0),
            ] {
                assert_eq!(properties[key]["type"], "number");
                assert_eq!(properties[key]["default"].as_f64(), Some(default));
            }
        } else {
            assert_eq!(
                keys,
                std::collections::BTreeSet::from(["path", "overwrite", "version"])
            );
            assert_eq!(properties["overwrite"]["default"], false);
            assert_eq!(properties["version"]["default"], "auto");
            assert_eq!(
                schema["$defs"]["MolVersionChoice"]["enum"],
                json!(["auto", "v2000", "v3000"])
            );
        }
    }
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
fn stdio_v3000_preserves_original_bytes_and_redo_across_rejected_versions_and_imports()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let source = directory.path("stdio-original-v3.v2000");
    let copy = directory.path("stdio-v3-copy.mol");
    let malformed = directory.path("stdio-v3-truncated.mol");
    let unknown = directory.path("stdio-unknown-version.mol");
    let unpublished = directory.path("stdio-invalid-selector.mol");
    std::fs::write(&source, mol_support::V3000_SOURCE)?;
    let original = std::str::from_utf8(mol_support::V3000_SOURCE)?;
    std::fs::write(&malformed, original.replace("M  V30 END CTAB\n", ""))?;
    std::fs::write(&unknown, original.replace(" V3000", " V4000"))?;
    let mut client = Client::start()?;
    client.call(1, "initialize", json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"v3000-wire-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let opened = client.call(
        2,
        "tools/call",
        json!({"name":"document.open_mol","arguments":{"path":source}}),
    )?;
    assert_eq!(opened["result"]["isError"], false, "{opened}");
    mol_support::assert_v3000_state(&opened["result"]["structuredContent"])?;
    let saved = client.call(
        3,
        "tools/call",
        json!({"name":"document.save_mol","arguments":{"path":copy}}),
    )?;
    assert_eq!(saved["result"]["isError"], false, "{saved}");
    mol_support::assert_v3000_state(&saved["result"]["structuredContent"])?;
    assert_eq!(
        saved["result"]["structuredContent"]["file_operation"]["format"],
        "mol_v3000"
    );
    assert_eq!(std::fs::read(&copy)?, mol_support::V3000_SOURCE);
    let external = mol_support::independently_read_v3000(&copy)?;
    assert_eq!(
        external["source_sha256"],
        "1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380"
    );
    assert_eq!(external["atoms"][0]["explicit_h"], 3);
    assert_eq!(external["atoms"][1]["explicit_h"], 3);
    assert_eq!(external["bonds"][0]["order"], 1.0);
    let edited = client.call(
        4,
        "tools/call",
        json!({"name":"page.set_size","arguments":{"page_id":1,"width_pt":612,"height_pt":792}}),
    )?;
    assert_eq!(edited["result"]["isError"], false, "{edited}");
    let undone = client.call(
        5,
        "tools/call",
        json!({"name":"history.undo","arguments":{}}),
    )?;
    assert_eq!(undone["result"]["isError"], false, "{undone}");
    let baseline = client.call(
        6,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    let baseline = &baseline["result"]["structuredContent"];
    assert_eq!(baseline["revision"], 3);
    assert_eq!(baseline["undo_depth"], 0);
    assert_eq!(baseline["redo_depth"], 1);
    mol_support::assert_v3000_state(baseline)?;
    let mut id = 7;
    for (name, arguments, code) in [
        (
            "document.save_mol",
            json!({"path":copy,"overwrite":true,"version":"v2000"}),
            "unsupported_content",
        ),
        (
            "document.open_mol",
            json!({"path":malformed}),
            "invalid_mol",
        ),
        ("document.open_mol", json!({"path":unknown}), "invalid_mol"),
        (
            "document.save_mol",
            json!({"path":unpublished,"version":"v4000"}),
            "invalid_params",
        ),
        (
            "document.save_mol",
            json!({"path":unpublished,"version":null}),
            "invalid_params",
        ),
    ] {
        let rejected = client.call(id, "tools/call", json!({"name":name,"arguments":arguments}))?;
        assert_eq!(rejected["result"]["isError"], true, "{rejected}");
        assert_eq!(
            rejected["result"]["structuredContent"]["error"]["code"], code,
            "{rejected}"
        );
        let inspected = client.call(
            id + 1,
            "tools/call",
            json!({"name":"document.inspect","arguments":{}}),
        )?;
        assert_eq!(&inspected["result"]["structuredContent"], baseline);
        assert_eq!(std::fs::read(&copy)?, mol_support::V3000_SOURCE);
        id += 2;
    }
    assert!(!unpublished.try_exists()?);
    let saved = client.call(
        id,
        "tools/call",
        json!({"name":"document.save_mol","arguments":{"path":copy,"overwrite":true}}),
    )?;
    assert_eq!(saved["result"]["isError"], false, "{saved}");
    let saved = &saved["result"]["structuredContent"];
    mol_support::assert_v3000_state(saved)?;
    assert_eq!(saved["changed"], false);
    assert_eq!(saved["revision"], 3);
    assert_eq!(saved["redo_depth"], 1);
    assert_eq!(saved["file_operation"]["format"], "mol_v3000");
    assert_eq!(saved["file_operation"]["input_revision"], 3);
    assert_eq!(saved["file_operation"]["byte_length"], 344);
    assert_eq!(std::fs::read(&copy)?, mol_support::V3000_SOURCE);
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_mol_round_trip_and_failed_imports_preserve_source_history_and_file_state()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let source = directory.path("stdio-methyl.mol");
    let destination = directory.path("stdio-copy.mol");
    let malformed = directory.path("malformed.mol");
    let unsupported = directory.path("unsupported.mol");
    let cross_format = directory.path("must-not-publish.cdxml");
    let changed_layout = directory.path("must-not-publish.mol");
    std::fs::write(&source, mol_support::METHYL)?;
    std::fs::write(&malformed, b"not a MOL file\n")?;
    let extended =
        std::str::from_utf8(mol_support::METHYL)?.replace("M  END", "M  XYZ  1   1   7\nM  END");
    std::fs::write(&unsupported, extended)?;
    let mut client = Client::start()?;
    client.call(
        1,
        "initialize",
        json!({
            "protocolVersion":"2025-11-25","capabilities":{},
            "clientInfo":{"name":"mol-wire-test","version":"1"}
        }),
    )?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let opened = client.call(
        2,
        "tools/call",
        json!({"name":"document.open_mol","arguments":{
            "path":source,"points_per_unit":20.0,"origin_x_pt":100.0,"origin_y_pt":200.0
        }}),
    )?;
    assert_eq!(opened["result"]["isError"], false, "{opened}");
    mol_support::assert_methyl_state(&opened["result"]["structuredContent"], 75.25, 156.982)?;
    let saved = client.call(
        3,
        "tools/call",
        json!({"name":"document.save_mol","arguments":{"path":destination}}),
    )?;
    assert_eq!(saved["result"]["isError"], false, "{saved}");
    let saved = &saved["result"]["structuredContent"];
    mol_support::assert_methyl_state(saved, 75.25, 156.982)?;
    assert_eq!(saved["revision"], 1);
    assert_eq!(
        saved["file"],
        json!({"path":std::fs::canonicalize(&destination)?,"dirty":false,"format":"mol_v2000"})
    );
    assert_eq!(saved["file_operation"]["operation"], "save");
    assert_eq!(saved["file_operation"]["input_revision"], 1);
    assert_eq!(std::fs::read(&destination)?, mol_support::METHYL);
    assert_eq!(
        mol_support::independently_read_mol(&destination)?["smiles"],
        "[CH3]"
    );
    let baseline = client.call(
        4,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    let baseline = &baseline["result"]["structuredContent"];
    let mut id = 5;
    for (name, arguments, code) in [
        (
            "document.open_mol",
            json!({"path":source,"unexpected":true}),
            "invalid_params",
        ),
        (
            "document.open_mol",
            json!({"path":source,"points_per_unit":0}),
            "invalid_params",
        ),
        (
            "document.save_mol",
            json!({"path":destination}),
            "already_exists",
        ),
        (
            "document.save_cdxml",
            json!({"path":cross_format}),
            "unsupported_content",
        ),
    ] {
        let rejected = client.call(id, "tools/call", json!({"name":name,"arguments":arguments}))?;
        assert_eq!(rejected["result"]["isError"], true, "{rejected}");
        assert_eq!(
            rejected["result"]["structuredContent"]["error"]["code"],
            code
        );
        let inspected = client.call(
            id + 1,
            "tools/call",
            json!({"name":"document.inspect","arguments":{}}),
        )?;
        assert_eq!(&inspected["result"]["structuredContent"], baseline);
        id += 2;
    }
    assert!(!cross_format.try_exists()?);
    let edited = client.call(
        id,
        "tools/call",
        json!({"name":"page.set_size","arguments":{
            "page_id":1,"width_pt":612,"height_pt":792
        }}),
    )?;
    assert_eq!(edited["result"]["isError"], false, "{edited}");
    assert_eq!(edited["result"]["structuredContent"]["file"]["dirty"], true);
    let dirty = client.call(
        id + 1,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    let dirty = &dirty["result"]["structuredContent"];
    assert_eq!(dirty["revision"], 2);
    assert_eq!(dirty["undo_depth"], 1);
    id += 2;
    for (name, arguments, code) in [
        (
            "document.open_mol",
            json!({"path":source}),
            "unsaved_changes",
        ),
        (
            "document.open_mol",
            json!({"path":malformed,"discard_unsaved":true}),
            "invalid_mol",
        ),
        (
            "document.open_mol",
            json!({"path":unsupported,"discard_unsaved":true}),
            "unsupported_content",
        ),
        (
            "document.save_mol",
            json!({"path":changed_layout}),
            "unsupported_content",
        ),
    ] {
        let rejected = client.call(id, "tools/call", json!({"name":name,"arguments":arguments}))?;
        assert_eq!(rejected["result"]["isError"], true, "{rejected}");
        assert_eq!(
            rejected["result"]["structuredContent"]["error"]["code"],
            code
        );
        let inspected = client.call(
            id + 1,
            "tools/call",
            json!({"name":"document.inspect","arguments":{}}),
        )?;
        assert_eq!(&inspected["result"]["structuredContent"], dirty);
        id += 2;
    }
    assert!(!changed_layout.try_exists()?);
    assert_eq!(std::fs::read(&destination)?, mol_support::METHYL);
    let reopened = client.call(
        id,
        "tools/call",
        json!({"name":"document.open_mol","arguments":{
            "path":source,"discard_unsaved":true
        }}),
    )?;
    assert_eq!(reopened["result"]["isError"], false, "{reopened}");
    let reopened = &reopened["result"]["structuredContent"];
    mol_support::assert_methyl_state(reopened, 42.3, 20.3784)?;
    assert_eq!(reopened["revision"], 3);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
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
