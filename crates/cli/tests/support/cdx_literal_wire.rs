//! Independent literal CDX data through the real, unchanged command transports.
use super::{Client, cli, socket, support};
use crate::registry_fixture;
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    net::TcpStream,
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
#[path = "../../../../xtask/oracles/cdx_literal_support.rs"]
#[allow(
    dead_code,
    reason = "shared frozen bytes, scalar facts and independent reader"
)]
mod f;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn request(name: &str, params: Value) -> Value {
    json!({"command":name,"params":params})
}
fn checked(value: &Value, name: &str, revision: u64, depths: [u64; 2], dirty: bool) -> Result {
    assert_eq!(value["revision"], revision);
    assert_eq!(value["undo_depth"], depths[0]);
    assert_eq!(value["redo_depth"], depths[1]);
    assert_eq!(value["file"]["dirty"], dirty);
    assert_eq!(value["file"]["format"], "cdx");
    assert_eq!(
        value["source"],
        json!({"format":"cdx","byte_length":f::source(name).len(),"modified":dirty})
    );
    assert!(value.get("capture").is_none());
    let mut expected = f::model(name)?;
    if dirty {
        // The sole allowed model-only edit below supplies this exact page size.
        expected["pages"][0]["size"] = json!({"width_pt":612.0,"height_pt":792.0});
    }
    assert_eq!(
        value["document"], expected,
        "{name}: complete independent literal-data model"
    );
    Ok(())
}
fn consume(path: &Path, name: &str) -> Result {
    let raw = fs::read(path)?;
    assert_eq!(
        raw,
        f::source(name),
        "unchanged source presence, widths and opaque bytes"
    );
    assert_eq!(f::read(&raw)?, f::facts(name));
    Ok(())
}
fn discovery(rows: &[Value], mcp: bool) -> Result {
    let expected: Vec<Value> =
        serde_json::from_str(&registry_fixture::with_cdx_caption_edit(f::METADATA49))?;
    assert_eq!((rows.len(), expected.len()), (55, 49));
    assert_eq!(rows[49]["name"], "ui.caption");
    for (row, expected) in rows.iter().zip(&expected) {
        if mcp {
            assert_eq!(row["name"], expected["name"]);
            assert_eq!(row["description"], expected["description"]);
            assert_eq!(row["inputSchema"], expected["input_schema"]);
            assert_eq!(row["annotations"]["readOnlyHint"], expected["read_only"]);
        } else {
            assert_eq!(row, expected);
        }
    }
    Ok(())
}
fn refusals(revision: u64, destination: &Path) -> Vec<(&'static str, Value)> {
    vec![
        (
            "atom.set_label",
            json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":1,"element":6,"explicit_hydrogens":3}),
        ),
        (
            "atom.set_authored_label",
            json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":1,"element":6,"explicit_hydrogens":3}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
        ),
        (
            "render.capture",
            json!({"page_id":1,"width_px":1088,"height_px":1088,"backend":"cpu"}),
        ),
        (
            "render.capture",
            json!({"page_id":1,"width_px":1088,"height_px":1088,"backend":"gpu"}),
        ),
        (
            "document.save_cdxml",
            json!({"path":destination,"overwrite":true}),
        ),
        (
            "document.save_mol",
            json!({"path":destination,"overwrite":true}),
        ),
    ]
}

#[test]
fn actual_cli_literal_cdx_keeps49_records_and_independently_read_unicode_copies() -> Result {
    let listing = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listing.status.success(),
        "{}",
        String::from_utf8_lossy(&listing.stderr)
    );
    let previous = registry_fixture::with_cdx_caption_edit(f::METADATA49);
    let prefix = previous.strip_suffix("]\n").ok_or("frozen array end")?;
    assert!(
        listing.stdout.starts_with(prefix.as_bytes()),
        "the original 49 discovery records retain their exact bytes"
    );
    assert_eq!(listing.stdout.get(prefix.len()), Some(&b','));
    let rows: Value = serde_json::from_slice(&listing.stdout)?;
    discovery(rows.as_array().ok_or("commands")?, false)?;
    let directory = support::SdfTestDirectory::new()?;
    // The two legacy-charge cases deliberately disagree with the qualified RDKit
    // consumer. Their expected values come from the property-specific SDK rule.
    for name in [
        "methylammonium_explicit",
        "methyl_doublet",
        "deuteron",
        "charge4_low_one",
        "charge4_low_noise_max",
    ] {
        let input = directory.path(&format!("literal-cli-{name}-原-λ.cdx"));
        let output = directory.path(&format!("literal-cli-{name}-保存-λ.cdx"));
        fs::write(&input, f::source(name))?;
        consume(&input, name)?;
        let open = request("document.open_cdx", json!({"path":input}));
        let opened = cli(&[open.clone(), request("document.inspect", json!({}))])?;
        checked(&opened, name, 1, [0, 0], false)?;
        let reopened = cli(&[
            open,
            request("document.save_cdx", json!({"path":output})),
            request("document.open_cdx", json!({"path":output})),
            request("document.inspect", json!({})),
        ])?;
        checked(&reopened, name, 2, [0, 0], false)?;
        assert_eq!(reopened["document"], opened["document"]);
        assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
        consume(&input, name)?;
        consume(&output, name)?;
    }
    Ok(())
}

#[test]
fn initialized_stdio_literal_cdx_refuses_authoring_capture_and_cross_format_loss() -> Result {
    let name = "methyl_doublet";
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("literal-stdio-原-λ.cdx");
    let output = directory.path("literal-stdio-保存-λ.cdx");
    let sentinel = directory.path("literal-stdio-preserved-destination.mol");
    fs::write(&input, f::source(name))?;
    fs::write(&sentinel, b"retain destination\n")?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"cdx-literal-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    discovery(
        listed["result"]["tools"].as_array().ok_or("MCP tools")?,
        true,
    )?;
    let opened = client.call(3, "document.open_cdx", json!({"path":input}))?;
    checked(&opened, name, 1, [0, 0], false)?;
    let stable = client.call(4, "document.inspect", json!({}))?;
    let mut id = 5;
    for (command, params) in refusals(1, &sentinel) {
        client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":command,"arguments":params}}))?;
        let refused = client.replies.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(refused["id"], id);
        assert_eq!(refused["result"]["isError"], true, "{command}: {refused}");
        assert_eq!(
            refused["result"]["structuredContent"]["error"]["code"], "unsupported_content",
            "{command}: {refused}"
        );
        id += 1;
        assert_eq!(client.call(id, "document.inspect", json!({}))?, stable);
        assert_eq!(fs::read(&sentinel)?, b"retain destination\n");
        id += 1;
    }
    let saved = client.call(id, "document.save_cdx", json!({"path":output}))?;
    checked(&saved, name, 1, [0, 0], false)?;
    assert_eq!(
        saved["file_operation"]["byte_length"],
        f::source(name).len()
    );
    assert_eq!(saved["file_operation"]["ui_request_id"], Value::Null);
    assert_eq!(saved["file_operation"]["ui_request_current"], Value::Null);
    id += 1;
    let reopened = client.call(id, "document.open_cdx", json!({"path":output}))?;
    checked(&reopened, name, 2, [0, 0], false)?;
    assert_eq!(reopened["document"], stable["document"]);
    assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
    consume(&input, name)?;
    consume(&output, name)?;
    client.finish()
}

#[test]
fn authenticated_literal_cdx_preserves_noop_owners_and_restores_source_after_page_undo() -> Result {
    let name = "methylammonium_explicit";
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("literal-socket-原-λ.cdx");
    let output = directory.path("literal-socket-保存-λ.cdx");
    let sentinel = directory.path("literal-socket-preserved-destination.cdx");
    fs::write(&input, f::source(name))?;
    fs::write(&sentinel, b"retain destination\n")?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let server = LoopbackServer::bind(0, Arc::clone(&engine), || {})?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let opened = socket(
        &mut client,
        request("document.open_cdx", json!({"path":input})),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    checked(&opened["result"], name, 1, [0, 0], false)?;
    let before = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let (content, model, stamp, file) = {
        let current = engine.lock().map_err(|_| "engine")?;
        let stored = current.stored_document()?;
        assert!(!stored.is_fully_mapped());
        assert!(!stored.has_display_projection());
        (
            current.content().clone(),
            stored.model_snapshot(),
            current.capture_stamp(),
            current.ui_file_context_stamp(),
        )
    };
    let size = &before["document"]["pages"][0]["size"];
    let noop = socket(
        &mut client,
        request(
            "page.set_size",
            json!({"page_id":1,"width_pt":size["width_pt"],"height_pt":size["height_pt"]}),
        ),
    )?;
    assert_eq!(noop["result"], before);
    let mut cases = refusals(1, &sentinel);
    cases.push(("document.save_cdx", json!({"path":sentinel})));
    for (command, params) in cases {
        let refused = socket(&mut client, request(command, params))?;
        let code = if command == "document.save_cdx" {
            "already_exists"
        } else {
            "unsupported_content"
        };
        assert_eq!(refused["error"]["code"], code, "{command}: {refused}");
        assert_eq!(
            socket(&mut client, request("document.inspect", json!({})))?["result"],
            before
        );
        assert_eq!(fs::read(&sentinel)?, b"retain destination\n");
    }
    {
        let current = engine.lock().map_err(|_| "engine")?;
        assert_eq!(current.content(), &content);
        assert!(Arc::ptr_eq(
            &current.stored_document()?.model_snapshot(),
            &model
        ));
        assert!(current.capture_stamp_matches(&stamp));
        assert!(current.ui_file_context_stamp_matches(&file));
    }
    let changed = socket(
        &mut client,
        request(
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        ),
    )?;
    assert!(changed.get("error").is_none(), "{changed}");
    checked(&changed["result"], name, 2, [1, 0], true)?;
    let dirty = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    for (command, params, code) in [
        (
            "document.save_cdx",
            json!({"path":sentinel,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "document.open_cdx",
            json!({"path":input}),
            "unsaved_changes",
        ),
    ] {
        let refused = socket(&mut client, request(command, params))?;
        assert_eq!(refused["error"]["code"], code);
        assert_eq!(
            socket(&mut client, request("document.inspect", json!({})))?["result"],
            dirty
        );
        assert_eq!(fs::read(&sentinel)?, b"retain destination\n");
    }
    let undone = socket(&mut client, request("history.undo", json!({})))?;
    assert!(undone.get("error").is_none(), "{undone}");
    checked(&undone["result"], name, 3, [0, 1], false)?;
    assert_eq!(undone["result"]["document"], before["document"]);
    let saved = socket(
        &mut client,
        request("document.save_cdx", json!({"path":output})),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    checked(&saved["result"], name, 3, [0, 1], false)?;
    assert_eq!(
        saved["result"]["file_operation"]["byte_length"],
        f::source(name).len()
    );
    consume(&output, name)?;
    let reopened = socket(
        &mut client,
        request("document.open_cdx", json!({"path":output})),
    )?;
    assert!(reopened.get("error").is_none(), "{reopened}");
    checked(&reopened["result"], name, 4, [0, 0], false)?;
    assert_eq!(reopened["result"]["document"], before["document"]);
    assert_eq!(
        reopened["result"]["file"]["path"],
        json!(fs::canonicalize(&output)?)
    );
    consume(&input, name)?;
    consume(&output, name)?;
    let absent = directory.path("literal-socket-absent.cdx");
    let zeros = directory.path("literal-socket-defaults.cdx");
    fs::write(&absent, f::source("absent_implied"))?;
    fs::write(&zeros, f::source("scalar_zero_implied"))?;
    let first = socket(
        &mut client,
        request("document.open_cdx", json!({"path":absent})),
    )?;
    assert!(first.get("error").is_none(), "{first}");
    let (owner, file) = {
        let current = engine.lock().map_err(|_| "engine")?;
        (current.capture_stamp(), current.ui_file_context_stamp())
    };
    let second = socket(
        &mut client,
        request("document.open_cdx", json!({"path":zeros})),
    )?;
    assert!(second.get("error").is_none(), "{second}");
    assert_eq!(first["result"]["document"], second["result"]["document"]);
    {
        let current = engine.lock().map_err(|_| "engine")?;
        assert!(!current.capture_stamp_matches(&owner));
        assert!(!current.ui_file_context_stamp_matches(&file));
    }
    consume(&absent, "absent_implied")?;
    consume(&zeros, "scalar_zero_implied")?;
    drop(client);
    server.shutdown()?;
    Ok(())
}
