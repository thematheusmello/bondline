use serde_json::{Value, json};
#[path = "support/bond_order.rs"]
mod bond_order;
#[path = "support/cdx.rs"]
mod cdx_support;
#[path = "support/chain.rs"]
mod chain_support;
#[path = "support/conversion.rs"]
mod conversion_support;
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;
#[path = "support/mol.rs"]
mod mol_support;
#[path = "support/multiple_bonds.rs"]
mod multiple_bonds;
#[path = "../../../xtask/oracles/sdf_field_test_support.rs"]
mod sdf_field_support;
#[path = "../../../xtask/oracles/sdf_record_test_support.rs"]
mod sdf_record_support;
#[path = "../../../xtask/oracles/sdf_test_support.rs"]
mod sdf_support;
#[path = "support/smiles_export.rs"]
mod smiles_export_support;
#[path = "support/smiles_file.rs"]
mod smiles_file_support;
#[path = "support/smiles_record.rs"]
mod smiles_record_support;
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
fn stdio_smiles_record_query_is_bounded_exact_private_and_not_cached()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("stdio-record-化学.smi");
    let output = directory.path("stdio-record-copy.data");
    std::fs::write(&input, smiles_record_support::SOURCE)?;
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"smiles-record-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let listed = client.call(2, "tools/list", json!({}))?;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 31);
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "smiles.record.inspect")
        .ok_or("SMILES query tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], true);
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    assert_eq!(
        tool["inputSchema"]["required"],
        json!(["expected_revision", "record_id"])
    );
    let properties = tool["inputSchema"]["properties"]
        .as_object()
        .ok_or("query properties")?;
    assert_eq!(
        properties
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from(["expected_revision", "record_id"])
    );
    assert_eq!(properties["record_id"]["minimum"], 1);
    let wrong = client.call(
        3,
        "tools/call",
        json!({"name":"smiles.record.inspect","arguments":smiles_record_support::params(0,2)}),
    )?;
    assert_eq!(wrong["result"]["isError"], true);
    assert_eq!(
        wrong["result"]["structuredContent"]["error"]["code"],
        "unsupported_content"
    );
    let opened = client.call(
        4,
        "tools/call",
        json!({"name":"document.open_smiles","arguments":{"path":input}}),
    )?;
    assert_eq!(opened["result"]["isError"], false);
    let before = client.call(
        5,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    let before = &before["result"]["structuredContent"];
    smiles_record_support::assert_summary(before, 1);
    assert!(before.get("smiles_record").is_none());
    let mut id = 6;
    for record in [2, 3] {
        let result=client.call(id,"tools/call",json!({"name":"smiles.record.inspect","arguments":smiles_record_support::params(1,record)}))?;
        id += 1;
        assert_eq!(result["result"]["isError"], false, "{result}");
        let value = &result["result"]["structuredContent"];
        smiles_record_support::assert_query(value, 1, 1, record)?;
        let text: Value = serde_json::from_str(
            result["result"]["content"][0]["text"]
                .as_str()
                .ok_or("MCP text")?,
        )?;
        assert_eq!(&text, value);
        let mut state = value.clone();
        state
            .as_object_mut()
            .ok_or("result object")?
            .remove("smiles_record");
        assert_eq!(&state, before);
        if record == 2 {
            smiles_record_support::consume_notation(value)?;
        }
    }
    for (params, code) in [
        (
            json!({"expected_revision":0,"record_id":2}),
            "stale_revision",
        ),
        (json!({"expected_revision":1,"record_id":4}), "not_found"),
        (
            json!({"expected_revision":1,"record_id":0}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":1,"record_id":2,"graph":true}),
            "invalid_params",
        ),
        (json!({"record_id":2}), "invalid_params"),
    ] {
        let result = client.call(
            id,
            "tools/call",
            json!({"name":"smiles.record.inspect","arguments":params}),
        )?;
        id += 1;
        assert_eq!(result["result"]["isError"], true);
        assert_eq!(result["result"]["structuredContent"]["error"]["code"], code);
        let after = client.call(
            id,
            "tools/call",
            json!({"name":"document.inspect","arguments":{}}),
        )?;
        id += 1;
        assert_eq!(&after["result"]["structuredContent"], before);
    }
    let saved = client.call(
        id,
        "tools/call",
        json!({"name":"document.save_smiles","arguments":{"path":output}}),
    )?;
    id += 1;
    assert_eq!(saved["result"]["isError"], false);
    assert_eq!(
        saved["result"]["structuredContent"]["file_operation"]["byte_length"],
        82
    );
    assert!(
        saved["result"]["structuredContent"]
            .get("smiles_record")
            .is_none()
    );
    let reopened = client.call(
        id,
        "tools/call",
        json!({"name":"document.open_smiles","arguments":{"path":output}}),
    )?;
    id += 1;
    assert_eq!(reopened["result"]["isError"], false);
    let result = client.call(
        id,
        "tools/call",
        json!({"name":"smiles.record.inspect","arguments":smiles_record_support::params(2,2)}),
    )?;
    id += 1;
    smiles_record_support::assert_query(&result["result"]["structuredContent"], 2, 2, 2)?;
    let after = client.call(
        id,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    id += 1;
    smiles_record_support::assert_summary(&after["result"]["structuredContent"], 2);
    assert!(
        after["result"]["structuredContent"]
            .get("smiles_record")
            .is_none()
    );
    assert!(
        !after["result"]["structuredContent"]
            .to_string()
            .contains("[13CH2]")
    );
    assert_eq!(std::fs::read(&input)?, smiles_record_support::SOURCE);
    assert_eq!(std::fs::read(&output)?, smiles_record_support::SOURCE);

    // An existing valid lexical boundary is disclosed verbatim, not normalized.
    // The128KiB contract bounds smiles_record only; MCP's text copy is separate.
    let long_notation = format!("[{}C]", "0".repeat(65533));
    assert_eq!(long_notation.len(), 65536);
    let mut long_source = b"C PRIVATE_FIRST\n".to_vec();
    assert_eq!(long_source.len(), 16);
    long_source.extend_from_slice(long_notation.as_bytes());
    long_source.extend_from_slice(b"\tPRIVATE_SECOND");
    assert_eq!(long_source.len(), 65567);
    let long_input = directory.path("stdio-record-long.smi");
    std::fs::write(&long_input, &long_source)?;
    let opened = client.call(
        id,
        "tools/call",
        json!({"name":"document.open_smiles","arguments":{"path":long_input}}),
    )?;
    id += 1;
    assert_eq!(opened["result"]["isError"], false);
    let result = client.call(
        id,
        "tools/call",
        json!({"name":"smiles.record.inspect","arguments":smiles_record_support::params(3,2)}),
    )?;
    id += 1;
    assert_eq!(result["result"]["isError"], false);
    let record = &result["result"]["structuredContent"]["smiles_record"];
    assert_eq!(
        record,
        &json!({"input_revision":3,"session_generation":3,"record_id":2,"line_number":2,
        "full_range":{"start":16,"end":65567},"notation_range":{"start":16,"end":65552},
        "separator_range":{"start":65552,"end":65553},"application_data_range":{"start":65553,"end":65567},
        "line_ending_range":{"start":65567,"end":65567},"atom_count":1,"bond_count":0,"notation":long_notation})
    );
    assert!(serde_json::to_vec(record)?.len() <= 128 * 1024);
    smiles_record_support::assert_private(&result["result"]["structuredContent"]);
    let after = client.call(
        id,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    assert!(
        after["result"]["structuredContent"]
            .get("smiles_record")
            .is_none()
    );
    assert_eq!(std::fs::read(&long_input)?, long_source);
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_smiles_file_commands_keep_every_record_and_opaque_byte()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("stdio-smiles-λ.smi");
    let output = directory.path("stdio-smiles-copy.data");
    std::fs::write(&input, smiles_file_support::SOURCE)?;
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"smiles-file-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    for (id, name, path, revision) in [
        (2, "document.open_smiles", &input, 1),
        (3, "document.save_smiles", &output, 1),
        (4, "document.open_smiles", &output, 2),
    ] {
        let result = client.call(
            id,
            "tools/call",
            json!({"name":name,"arguments":{"path":path}}),
        )?;
        assert_eq!(result["result"]["isError"], false, "{result}");
        smiles_file_support::assert_file(&result["result"]["structuredContent"], revision);
    }
    assert_eq!(std::fs::read(&output)?, smiles_file_support::SOURCE);
    assert_eq!(std::fs::read(&input)?, smiles_file_support::SOURCE);
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_export_selection_receipt_and_file_bytes_are_explicit_and_do_not_change_the_session()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("stdio-export-native.cdxml");
    let output = directory.path("stdio-export-λ.data");
    std::fs::write(&input, smiles_export_support::SOURCE)?;
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"export-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let listed = client.call(2, "tools/list", json!({}))?;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 31);
    let tool = tools
        .iter()
        .find(|t| t["name"] == "file.export_smiles_fragment")
        .ok_or("export tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    assert_eq!(
        tool["inputSchema"]["required"]
            .as_array()
            .ok_or("required")?
            .len(),
        4
    );
    let opened = client.call(
        3,
        "tools/call",
        json!({"name":"document.open_cdxml","arguments":{"path":input}}),
    )?;
    let mut before = opened["result"]["structuredContent"].clone();
    before
        .as_object_mut()
        .ok_or("state")?
        .remove("file_operation");
    before["changed"] = json!(false);
    let result=client.call(4,"tools/call",json!({"name":"file.export_smiles_fragment","arguments":smiles_export_support::params(1,&output)}))?;
    assert_eq!(result["result"]["isError"], false);
    smiles_export_support::assert_export(&result["result"]["structuredContent"], &output)?;
    let inspected = client.call(
        5,
        "tools/call",
        json!({"name":"document.inspect","arguments":{}}),
    )?;
    assert_eq!(inspected["result"]["structuredContent"], before);
    let refused=client.call(6,"tools/call",json!({"name":"file.export_smiles_fragment","arguments":smiles_export_support::params(1,&output)}))?;
    assert_eq!(
        refused["result"]["structuredContent"]["error"]["code"],
        "already_exists"
    );
    smiles_export_support::consume(&output)?;
    assert_eq!(std::fs::read(&input)?, smiles_export_support::SOURCE);
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_conversion_discovery_publication_and_editable_native_workflow_are_real()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("stdio-conversion.smi");
    let output = directory.path("stdio-conversion-λ.cdxml");
    std::fs::write(&input, b"CC\n")?;
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"conversion-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let list = client.call(2, "tools/list", json!({}))?;
    let tools = list["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 31);
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "file.convert_smiles_to_cdxml")
        .ok_or("conversion tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    assert_eq!(
        tool["inputSchema"]["required"]
            .as_array()
            .ok_or("required")?
            .len(),
        8
    );
    let converted=client.call(3,"tools/call",json!({"name":"file.convert_smiles_to_cdxml","arguments":conversion_support::params(0,&input,&output)}))?;
    assert_eq!(converted["result"]["isError"], false);
    conversion_support::assert_conversion(
        &converted["result"]["structuredContent"],
        &input,
        &output,
    )?;
    conversion_support::independent(&output, 2)?;
    let conflict=client.call(4,"tools/call",json!({"name":"file.convert_smiles_to_cdxml","arguments":conversion_support::params(0,&input,&output)}))?;
    assert_eq!(conflict["result"]["isError"], true);
    assert_eq!(
        conflict["result"]["structuredContent"]["error"]["code"],
        "already_exists"
    );
    for (at, (name, arguments, count, revision, dirty)) in [
        ("document.open_cdxml", json!({"path":output}), 2, 1, false),
        ("bond.extend", conversion_support::extend(1), 3, 2, true),
        ("history.undo", json!({}), 2, 3, false),
        ("history.redo", json!({}), 3, 4, true),
        (
            "document.save_cdxml",
            json!({"path":output,"overwrite":true}),
            3,
            4,
            false,
        ),
        ("document.open_cdxml", json!({"path":output}), 3, 5, false),
    ]
    .into_iter()
    .enumerate()
    {
        let response = client.call(
            5 + at as u64,
            "tools/call",
            json!({"name":name,"arguments":arguments}),
        )?;
        assert_eq!(response["result"]["isError"], false, "{response}");
        let state = &response["result"]["structuredContent"];
        conversion_support::assert_chain(state, count)?;
        assert_eq!(state["revision"], revision);
        assert_eq!(state["file"]["dirty"], dirty);
        assert!(state.get("conversion").is_none());
    }
    conversion_support::independent(&output, 3)?;
    client.finish()?;
    Ok(())
}

#[test]
fn stdio_field_edit_save_undo_redo_uses_the_same_complete_source_and_error_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let mut client = Client::start()?;
    client.call(1,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sdf-set-test","version":"1"}}))?;
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let tools = client.call(2, "tools/list", json!({}))?;
    let tool = tools["result"]["tools"]
        .as_array()
        .ok_or("tools")?
        .iter()
        .find(|tool| tool["name"] == "sdf.field.set_value")
        .ok_or("setter")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    let mut id = 3;
    for (case, mixed) in [false, true].into_iter().enumerate() {
        let input = directory.path(if mixed {
            "stdio-set-mixed.sdf"
        } else {
            "stdio-set-ordered.sdf"
        });
        let output = directory.path(if mixed {
            "stdio-set-mixed-λ.data"
        } else {
            "stdio-set-ordered-λ.data"
        });
        std::fs::write(&input, sdf_field_support::source(mixed))?;
        let revision = case * 4 + 1;
        let index = sdf_field_support::field_index(mixed);
        client.call(
            id,
            "tools/call",
            json!({"name":"document.open_sdf","arguments":{"path":input}}),
        )?;
        id += 1;
        let query=client.call(id,"tools/call",json!({"name":"sdf.record.inspect","arguments":{"expected_revision":revision,"record_id":1}}))?;
        id += 1;
        assert_eq!(
            query["result"]["structuredContent"]["sdf_record"]["fields"],
            sdf_record_support::expected_fields(mixed, 1)
        );
        let changed=client.call(id,"tools/call",json!({"name":"sdf.field.set_value","arguments":{"expected_revision":revision,"record_id":1,"field_index":index,"value_lines":["revised","multiline"]}}))?;
        id += 1;
        assert_eq!(changed["result"]["isError"], false, "{changed}");
        let changed = &changed["result"]["structuredContent"];
        assert_eq!(changed["changed"], true);
        assert_eq!(changed["revision"], revision + 1);
        assert_eq!(changed["file"]["dirty"], true);
        assert_eq!(changed["source"]["modified"], true);
        assert!(changed.get("sdf_record").is_none());
        let query=client.call(id,"tools/call",json!({"name":"sdf.record.inspect","arguments":{"expected_revision":revision+1,"record_id":1}}))?;
        id += 1;
        assert_eq!(
            query["result"]["structuredContent"]["sdf_record"]["fields"][index]["value_lines"],
            json!(["revised", "multiline"])
        );
        let saved = client.call(
            id,
            "tools/call",
            json!({"name":"document.save_sdf","arguments":{"path":output}}),
        )?;
        id += 1;
        assert_eq!(saved["result"]["isError"], false, "{saved}");
        assert_eq!(
            saved["result"]["structuredContent"]["file_operation"]["byte_length"],
            if mixed { 788 } else { 704 }
        );
        assert_eq!(saved["result"]["structuredContent"]["file"]["dirty"], false);
        sdf_field_support::assert_edited_file(&output, mixed)?;
        let undo = client.call(
            id,
            "tools/call",
            json!({"name":"history.undo","arguments":{}}),
        )?;
        id += 1;
        assert_eq!(
            undo["result"]["structuredContent"]["revision"],
            revision + 2
        );
        assert_eq!(undo["result"]["structuredContent"]["file"]["dirty"], true);
        assert_eq!(
            undo["result"]["structuredContent"]["source"]["modified"],
            false
        );
        let redo = client.call(
            id,
            "tools/call",
            json!({"name":"history.redo","arguments":{}}),
        )?;
        id += 1;
        assert_eq!(
            redo["result"]["structuredContent"]["revision"],
            revision + 3
        );
        assert_eq!(redo["result"]["structuredContent"]["file"]["dirty"], false);
        assert_eq!(
            redo["result"]["structuredContent"]["source"]["modified"],
            true
        );
        let query=client.call(id,"tools/call",json!({"name":"sdf.record.inspect","arguments":{"expected_revision":revision+3,"record_id":1}}))?;
        id += 1;
        assert_eq!(
            query["result"]["structuredContent"]["sdf_record"]["fields"][index]["value_lines"],
            json!(["revised", "multiline"])
        );
        let before = client.call(
            id,
            "tools/call",
            json!({"name":"document.inspect","arguments":{}}),
        )?;
        id += 1;
        for (params, code) in [
            (
                json!({"expected_revision":revision,"record_id":1,"field_index":index,"value_lines":[]}),
                "stale_revision",
            ),
            (
                json!({"expected_revision":revision+3,"record_id":3,"field_index":0,"value_lines":[]}),
                "not_found",
            ),
            (
                json!({"expected_revision":revision+3,"record_id":1,"field_index":index,"value_lines":[""]}),
                "invalid_sdf",
            ),
        ] {
            let failure = client.call(
                id,
                "tools/call",
                json!({"name":"sdf.field.set_value","arguments":params}),
            )?;
            id += 1;
            assert_eq!(failure["result"]["isError"], true);
            assert_eq!(
                failure["result"]["structuredContent"]["error"]["code"],
                code
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
        }
        let noop=client.call(id,"tools/call",json!({"name":"sdf.field.set_value","arguments":{"expected_revision":revision+3,"record_id":1,"field_index":index,"value_lines":["revised","multiline"]}}))?;
        id += 1;
        assert_eq!(
            noop["result"]["structuredContent"],
            before["result"]["structuredContent"]
        );
        assert_eq!(std::fs::read(input)?, sdf_field_support::source(mixed));
    }
    client.finish()?;
    Ok(())
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
            "bond.set_order",
            "document.close",
            "document.inspect",
            "document.new",
            "document.open_cdx",
            "document.open_cdxml",
            "document.open_mol",
            "document.open_sdf",
            "document.open_smiles",
            "document.save_cdx",
            "document.save_cdxml",
            "document.save_mol",
            "document.save_sdf",
            "document.save_smiles",
            "file.convert_smiles_to_cdxml",
            "file.export_smiles_fragment",
            "history.redo",
            "history.undo",
            "page.set_size",
            "render.capture",
            "sdf.field.set_value",
            "sdf.record.inspect",
            "sdf.record.remove",
            "smiles.record.inspect",
            "structure.paste_smiles",
            "ui.bond_tool",
            "ui.controls",
            "ui.file",
            "ui.view"
        ]
    );
    for (name, permission) in [
        ("document.open_cdx", "discard_unsaved"),
        ("document.save_cdx", "overwrite"),
        ("document.open_sdf", "discard_unsaved"),
        ("document.save_sdf", "overwrite"),
        ("document.open_smiles", "discard_unsaved"),
        ("document.save_smiles", "overwrite"),
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
    assert_eq!(queried["controls"].as_array().ok_or("controls")?.len(), 18);
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
