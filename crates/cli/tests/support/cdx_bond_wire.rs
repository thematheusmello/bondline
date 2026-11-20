//! Plain bond metadata and read-only display of complete skeletal CDX drawings.
use super::{Client, cli, socket, support};
use crate::registry_fixture;
use bondline_commands::{Command as EngineCommand, Engine};
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
#[path = "../../../../xtask/oracles/cdx_bond_support.rs"]
#[allow(
    dead_code,
    reason = "shared independent binary, model and actual-file facts"
)]
mod f;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn request(command: &str, params: Value) -> Value {
    json!({"command":command,"params":params})
}
#[test]
fn skeletal_cdx_cli_preserves_omitted_h_after_preview_and_exact_save() -> Result {
    let directory = support::SdfTestDirectory::new()?;
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../io/tests/fixtures/cdx-skeletal");
    let capture = request(
        "render.capture",
        json!({
            "page_id":1,"width_px":512,"height_px":512,"backend":"cpu"
        }),
    );
    let control = cli(&[
        request(
            "document.open_cdxml",
            json!({"path": fixtures.join("optical-control.cdxml")}),
        ),
        capture.clone(),
    ])?;
    let reference = control["capture"]["png_base64"]
        .as_str()
        .ok_or("control PNG")?;
    assert!(!reference.is_empty());
    for (name, expected_h) in [
        ("propane", json!([3, 2, 3])),
        ("missing_h", json!([3, null, 3])),
        ("all_h_missing", json!([null, null, null])),
        ("dormant_true", json!([3, 2, 3])),
    ] {
        let source = fixtures.join(format!("{name}.cdx"));
        let output = directory.path(&format!("{name}-保存.cdx"));
        let saved = cli(&[
            request("document.open_cdx", json!({"path":source})),
            request("document.save_cdx", json!({"path":output})),
            request("document.open_cdx", json!({"path":output})),
            capture.clone(),
        ])?;
        assert_eq!(fs::read(&source)?, fs::read(&output)?);
        assert_eq!(saved["capture"]["png_base64"], reference, "{name}");
        let page = &saved["document"]["pages"][0];
        assert_eq!(page["size"], json!({"width_pt":512.0, "height_pt":512.0}));
        let atoms = page["fragments"][0]["atoms"]
            .as_array()
            .ok_or("carbon atoms")?;
        let h = atoms
            .iter()
            .map(|a| &a["explicit_hydrogens"])
            .collect::<Vec<_>>();
        assert_eq!(json!(h), expected_h);
    }
    Ok(())
}

#[test]
fn skeletal_cdx_cli_moves_scoped_atoms_and_reopens_the_exact_bytes() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../io/tests/fixtures");
    for (source, after, ids, first, second) in [
        (
            "cdx-skeletal-move/aliased_extended.cdx",
            "cdx-skeletal-move/aliased_moved.cdx",
            [4, 2],
            [218.875, 195.25],
            [186.875, 371.25],
        ),
        (
            "cdx-mixed-skeletal-move/twins.cdx",
            "cdx-mixed-skeletal-move/scoped.cdx",
            [3, 2],
            [202.875, 147.25],
            [154.875, 339.25],
        ),
    ] {
        let original = root.join(source);
        let expected = fs::read(root.join(after))?;
        let directory = support::SdfTestDirectory::new()?;
        let dest = directory.path("moved-保存-β.cdx");
        let open = request("document.open_cdx", json!({"path":original}));
        let select = request(
            "ui.selection",
            json!({"expected_revision":1, "action":"set","page_id":1,"atom_ids":ids}),
        );
        let selected = cli(&[open.clone(), select.clone()])?;
        let epoch = selected["ui_selection"]["epoch"]
            .as_u64()
            .ok_or("selection epoch")?;
        let result = cli(&[
            open,
            select,
            request(
                "selection.translate",
                json!({"expected_revision":1, "expected_selection_epoch":epoch, "delta_x_pt":-5.125,"delta_y_pt":3.25}),
            ),
            request("history.undo", json!({})),
            request("history.redo", json!({})),
            request("document.save_cdx", json!({"path":dest})),
            request("document.open_cdx", json!({"path":dest})),
        ])?;
        assert_eq!(fs::read(dest)?, expected);
        let fragments = &result["document"]["pages"][0]["fragments"];
        assert_eq!(
            fragments[0]["atoms"][1]["position"],
            json!({"x_pt":first[0],"y_pt":first[1]})
        );
        assert_eq!(
            fragments[1]["atoms"][0]["position"],
            json!({"x_pt":second[0],"y_pt":second[1]})
        );
        assert_eq!(result["file"]["dirty"], false);
    }
    Ok(())
}

fn exact(value: &Value, name: &str) -> Result {
    assert_eq!(value["document"], f::model(name)?);
    assert_eq!(value["file"]["format"], "cdx");
    assert_eq!(value["file"]["dirty"], false);
    assert_eq!(
        value["source"],
        json!({"format":"cdx","byte_length":f::source(name).len(),"modified":false})
    );
    assert!(value.get("capture").is_none());
    Ok(())
}
fn consume(path: &Path, name: &str) -> Result {
    let raw = fs::read(path)?;
    assert_eq!(raw, f::source(name));
    assert_eq!(f::read(&raw)?, f::facts(name));
    Ok(())
}
fn refusals(revision: u64, order: &str, destination: &Path) -> Vec<(&'static str, Value)> {
    vec![
        (
            "bond.set_order",
            json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"bond_id":1,"order":order,"hydrogen_policy":"rectify_neutral_carbon"}),
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
fn actual_cli_plain_cdx_bonds_keep49_records_direction_side_and_unicode_source_bytes() -> Result {
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
    assert_eq!(rows.as_array().map(Vec::len), Some(51));
    assert_eq!(rows[49]["name"], "ui.caption");
    assert_eq!(rows[50]["name"], "text.create");
    let directory = support::SdfTestDirectory::new()?;
    for name in [
        "ethene",
        "ethyne",
        "auto_right",
        "manual_right",
        "reversed_right",
        "single_absent",
        "single_solid",
    ] {
        let input = directory.path(&format!("plain-cli-{name}-原-λ.cdx"));
        let output = directory.path(&format!("plain-cli-{name}-保存-λ.cdx"));
        fs::write(&input, f::source(name))?;
        consume(&input, name)?;
        let open = request("document.open_cdx", json!({"path":input}));
        let opened = cli(&[open.clone(), request("document.inspect", json!({}))])?;
        exact(&opened, name)?;
        assert_eq!(opened["revision"], 1);
        let reopened = cli(&[
            open,
            request("document.save_cdx", json!({"path":output})),
            request("document.open_cdx", json!({"path":output})),
            request("document.inspect", json!({})),
        ])?;
        exact(&reopened, name)?;
        assert_eq!(reopened["revision"], 2);
        assert_eq!(
            (
                reopened["undo_depth"].as_u64(),
                reopened["redo_depth"].as_u64()
            ),
            (Some(0), Some(0))
        );
        assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
        consume(&input, name)?;
        consume(&output, name)?;
    }
    Ok(())
}

#[test]
fn initialized_mcp_plain_cdx_bonds_keep_exact_discovery_refusals_and_unchanged_save() -> Result {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("plain-mcp-ethyne-原-λ.cdx");
    let output = directory.path("plain-mcp-ethyne-保存-λ.cdx");
    let sentinel = directory.path("plain-mcp-preserved.cdx");
    fs::write(&input, f::source("ethyne"))?;
    fs::write(&sentinel, b"keep destination\n")?;
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"cdx-plain-bond-test","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let rows = listed["result"]["tools"].as_array().ok_or("MCP tools")?;
    let expected: Vec<Value> =
        serde_json::from_str(&registry_fixture::with_cdx_caption_edit(f::METADATA49))?;
    assert_eq!((rows.len(), expected.len()), (51, 49));
    assert_eq!(rows[49]["name"], "ui.caption");
    for (row, expected) in rows.iter().zip(&expected) {
        assert_eq!(row["name"], expected["name"]);
        assert_eq!(row["description"], expected["description"]);
        assert_eq!(row["inputSchema"], expected["input_schema"]);
        assert_eq!(row["annotations"]["readOnlyHint"], expected["read_only"]);
    }
    exact(
        &client.call(3, "document.open_cdx", json!({"path":input}))?,
        "ethyne",
    )?;
    let before = client.call(4, "document.inspect", json!({}))?;
    let mut id = 5;
    for (name, params) in refusals(1, "triple", &sentinel) {
        client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":params}}))?;
        let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(reply["id"], id);
        assert_eq!(reply["result"]["isError"], true, "{name}: {reply}");
        assert_eq!(
            reply["result"]["structuredContent"]["error"]["code"], "unsupported_content",
            "{name}: {reply}"
        );
        id += 1;
        assert_eq!(client.call(id, "document.inspect", json!({}))?, before);
        assert_eq!(fs::read(&sentinel)?, b"keep destination\n");
        id += 1;
    }
    let saved = client.call(id, "document.save_cdx", json!({"path":output}))?;
    exact(&saved, "ethyne")?;
    assert_eq!(saved["revision"], 1);
    assert_eq!(
        saved["file_operation"]["byte_length"],
        f::source("ethyne").len()
    );
    assert_eq!(saved["file_operation"]["ui_request_current"], Value::Null);
    let reopened = client.call(id + 1, "document.open_cdx", json!({"path":output}))?;
    exact(&reopened, "ethyne")?;
    assert_eq!(reopened["revision"], 2);
    assert_eq!(reopened["file"]["path"], json!(fs::canonicalize(&output)?));
    consume(&input, "ethyne")?;
    consume(&output, "ethyne")?;
    client.finish()
}

#[test]
fn authenticated_plain_cdx_bonds_keep_auto_manual_source_owners_and_detached_page_history() -> Result
{
    let directory = support::SdfTestDirectory::new()?;
    let auto = directory.path("plain-socket-auto-原-λ.cdx");
    let manual = directory.path("plain-socket-manual-原-λ.cdx");
    let captured = directory.path("plain-socket-captured-auto-保存-λ.cdx");
    let output = directory.path("plain-socket-manual-保存-λ.cdx");
    let sentinel = directory.path("plain-socket-preserved.cdx");
    assert_ne!(f::source("auto_right"), f::source("manual_right"));
    fs::write(&auto, f::source("auto_right"))?;
    fs::write(&manual, f::source("manual_right"))?;
    fs::write(&sentinel, b"keep destination\n")?;
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
        request("document.open_cdx", json!({"path":auto})),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    exact(&opened["result"], "auto_right")?;
    let (old_source, old_file, pending) = {
        let mut current = engine.lock().map_err(|_| "engine")?;
        (
            current.capture_stamp(),
            current.ui_file_context_stamp(),
            current.prepare(EngineCommand::parse(
                "document.save_cdx",
                json!({"path":captured}),
            )?)?,
        )
    };
    let published = pending.complete()?;
    let switched = socket(
        &mut client,
        request("document.open_cdx", json!({"path":manual})),
    )?;
    assert!(switched.get("error").is_none(), "{switched}");
    exact(&switched["result"], "manual_right")?;
    assert_eq!(opened["result"]["document"], switched["result"]["document"]);
    let before = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let (model, source, file) = {
        let mut current = engine.lock().map_err(|_| "engine")?;
        assert!(!current.capture_stamp_matches(&old_source));
        assert!(!current.ui_file_context_stamp_matches(&old_file));
        let current_source = current.capture_stamp();
        let current_file = current.ui_file_context_stamp();
        let result = serde_json::to_value(current.finish(published)?)?;
        assert_eq!(result["file"]["path"], json!(fs::canonicalize(&manual)?));
        assert_eq!(
            result["file_operation"]["path"],
            json!(fs::canonicalize(&captured)?)
        );
        assert_eq!(result["file_operation"]["input_revision"], 1);
        assert_eq!(
            serde_json::to_value(current.state())?,
            before,
            "captured automatic-source Save cannot ACK manual source"
        );
        assert!(current.capture_stamp_matches(&current_source));
        assert!(current.ui_file_context_stamp_matches(&current_file));
        let stored = current.stored_document()?;
        assert!(!stored.is_fully_mapped());
        assert!(!stored.has_display_projection());
        assert_eq!(stored.to_cdx()?.as_ref(), f::source("manual_right"));
        (
            stored.model_snapshot(),
            current.capture_stamp(),
            current.ui_file_context_stamp(),
        )
    };
    consume(&captured, "auto_right")?;
    let noop = socket(
        &mut client,
        request(
            "page.set_size",
            json!({"page_id":1,"width_pt":210.0*72.0/25.4,"height_pt":297.0*72.0/25.4}),
        ),
    )?;
    assert_eq!(noop["result"], before);
    for (name, params) in refusals(2, "double", &sentinel) {
        let refused = socket(&mut client, request(name, params))?;
        assert_eq!(
            refused["error"]["code"], "unsupported_content",
            "{name}: {refused}"
        );
        assert_eq!(
            socket(&mut client, request("document.inspect", json!({})))?["result"],
            before
        );
        assert_eq!(fs::read(&sentinel)?, b"keep destination\n");
    }
    {
        let current = engine.lock().map_err(|_| "engine")?;
        assert!(Arc::ptr_eq(
            &model,
            &current.stored_document()?.model_snapshot()
        ));
        assert!(current.capture_stamp_matches(&source));
        assert!(current.ui_file_context_stamp_matches(&file));
    }
    let resized = socket(
        &mut client,
        request(
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        ),
    )?;
    assert!(resized.get("error").is_none(), "{resized}");
    let mut expected = f::model("manual_right")?;
    expected["pages"][0]["size"] = json!({"width_pt":612.0,"height_pt":792.0});
    assert_eq!(resized["result"]["document"], expected);
    assert_eq!(resized["result"]["revision"], 3);
    assert_eq!(resized["result"]["file"]["dirty"], true);
    let dirty = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let refused = socket(
        &mut client,
        request(
            "document.save_cdx",
            json!({"path":sentinel,"overwrite":true}),
        ),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        dirty
    );
    assert_eq!(fs::read(&sentinel)?, b"keep destination\n");
    let undone = socket(&mut client, request("history.undo", json!({})))?;
    assert!(undone.get("error").is_none(), "{undone}");
    exact(&undone["result"], "manual_right")?;
    assert_eq!(undone["result"]["revision"], 4);
    assert_eq!(
        (
            undone["result"]["undo_depth"].as_u64(),
            undone["result"]["redo_depth"].as_u64()
        ),
        (Some(0), Some(1))
    );
    let saved = socket(
        &mut client,
        request("document.save_cdx", json!({"path":output})),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    exact(&saved["result"], "manual_right")?;
    assert_eq!(saved["result"]["revision"], 4);
    let reopened = socket(
        &mut client,
        request("document.open_cdx", json!({"path":output})),
    )?;
    assert!(reopened.get("error").is_none(), "{reopened}");
    exact(&reopened["result"], "manual_right")?;
    assert_eq!(reopened["result"]["revision"], 5);
    assert_eq!(
        reopened["result"]["file"]["path"],
        json!(fs::canonicalize(&output)?)
    );
    consume(&auto, "auto_right")?;
    consume(&manual, "manual_right")?;
    consume(&output, "manual_right")?;
    drop(client);
    server.shutdown()?;
    Ok(())
}
