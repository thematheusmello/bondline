//! Independent chair acceptance through actual CLI, initialized stdio and authenticated loopback.
//! Child of unsaturated_wire: the existing raw/style/Expat/Decimal/RDKit consumer is unchanged.
use super::super::{names, schema_node};
use super::{
    Client, Engine, LoopbackServer, SharedEngine, TestResult, atom, cli, consume, graph,
    initialize, socket, support,
};
use bondline_mcp::CommandExecutor;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    io::BufReader,
    net::TcpStream,
    process::Command,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

const ONE: &str = "cyclohexane_chair_1";
const TWO: &str = "cyclohexane_chair_2";

fn chair(revision: u64, kind: &str, placement: Value) -> Value {
    json!({"expected_revision":revision,"page_id":1,"kind":kind,
        "hydrogen_policy":"rectify_neutral_carbon","placement":placement})
}
fn free(revision: u64, kind: &str) -> Value {
    chair(
        revision,
        kind,
        json!({"mode":"free","begin_x_pt":160,"begin_y_pt":160,
        "end_x_pt":224,"end_y_pt":160,"side":"positive"}),
    )
}
fn create(params: Value) -> Value {
    json!({"command":"ring.create","params":params})
}
fn expected(two: bool, shared_edge: bool) -> Value {
    // Literal closed projected-chair coordinates from the locked original 3D construction.
    // This neither calls the product geometry nor iteratively turns regular-ring edges.
    let h = 2.0_f64.sqrt() / 8.0;
    let q = 3.0_f64.sqrt() / 8.0;
    let g = 4.0 / 17.0_f64.sqrt();
    // Supporting end-bonds: One source indices[1,0,5,4,3,2], Two[3,2,1,0,5,4].
    // Same 3D seed/camera/projected polygons; only the unshipped reference edge changed.
    let projected = if two {
        [
            [-1.0, g * h],
            [-0.5, -g * (h + q)],
            [0.5, g * (h - q)],
            [1.0, -g * h],
            [0.5, g * (h + q)],
            [-0.5, g * (q - h)],
        ]
    } else {
        [
            [0.5, -g * (h + q)],
            [1.0, g * h],
            [0.5, g * (q - h)],
            [-0.5, g * (q + h)],
            [-1.0, -g * h],
            [-0.5, g * (h - q)],
        ]
    };
    let edge_x = 0.5;
    let edge_y = if two { -1.0 } else { 1.0 } * g * (2.0 * h + q);
    let divisor = (7.0 + 6.0_f64.sqrt()) / 17.0;
    let points = projected.map(|p| {
        let dx = p[0] - projected[0][0];
        let dy = p[1] - projected[0][1];
        [
            160.0 + 64.0 * (dx * edge_x + dy * edge_y) / divisor,
            160.0 + 64.0 * (dy * edge_x - dx * edge_y) / divisor,
        ]
    });
    let atoms = points
        .into_iter()
        .enumerate()
        .map(|(i, point)| atom(i + 1, point, 2, !shared_edge || i >= 2))
        .collect::<Vec<_>>();
    json!({"id":1,"atoms":atoms,"bonds":[
        {"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"single"},
        {"id":3,"begin":3,"end":4,"order":"single"},
        {"id":4,"begin":4,"end":5,"order":"single"},
        {"id":5,"begin":5,"end":6,"order":"single"},
        {"id":6,"begin":6,"end":1,"order":"single"}]})
}
fn operation(value: &Value, epoch: u64, changed: bool) {
    assert_eq!(value["changed"], false);
    assert_eq!(
        value["ui_ring_tool_operation"],
        json!({"size":6,"epoch":epoch,"changed":changed})
    );
    assert!(value.get("file_operation").is_none());
    assert!(value.get("close_operation").is_none());
}
fn same_session(result: &Value, before: &Value) -> TestResult {
    let mut ordinary = result.clone();
    assert!(
        ordinary
            .as_object_mut()
            .ok_or("state")?
            .remove("ui_ring_tool_operation")
            .is_some()
    );
    assert_eq!(
        &ordinary, before,
        "tool selection is not an authored edit or Save"
    );
    Ok(())
}
fn controls(value: &Value, selected: &str) -> TestResult {
    let rows = value["controls"].as_array().ok_or("controls")?;
    assert_eq!(rows.len(), 40);
    for (kind, id, label) in [
        (ONE, "tool.ring.chair_1", "Cyclohexane chair 1"),
        (TWO, "tool.ring.chair_2", "Cyclohexane chair 2"),
    ] {
        let row = rows.iter().find(|r| r["id"] == id).ok_or("chair control")?;
        assert_eq!(row["label"], label);
        assert_eq!(row["menu"], "Tools");
        assert_eq!(row["enabled"], true);
        assert_eq!(
            row["shortcut"],
            Value::Null,
            "9/0 are contextual bond hotkeys"
        );
        assert_eq!(row["selected"], kind == selected);
        assert_eq!(
            row["command"],
            json!({"command":"ui.ring_tool","params":{"kind":kind}})
        );
    }
    let active = rows
        .iter()
        .filter(|r| {
            r["selected"] == true && r["id"].as_str().is_some_and(|id| id.starts_with("tool."))
        })
        .map(|r| r["id"].as_str().ok_or("selected ID"))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        active,
        vec![if selected == ONE {
            "tool.ring.chair_1"
        } else {
            "tool.ring.chair_2"
        }]
    );
    Ok(())
}
fn schema(root: &Value, creation: bool) -> TestResult {
    assert_eq!(root["type"], "object");
    assert!(root.get("additionalProperties").is_none());
    let branches = root["anyOf"]
        .as_array()
        .ok_or("three closed ring branches")?;
    assert_eq!(branches.len(), 3);
    let mut seen = BTreeSet::new();
    for raw in branches {
        let branch = schema_node(root, raw)?;
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let p = branch["properties"]
            .as_object()
            .ok_or("branch properties")?;
        let mut required = BTreeSet::new();
        if creation {
            required.extend([
                "expected_revision",
                "page_id",
                "hydrogen_policy",
                "placement",
            ]);
            assert_eq!(
                schema_node(root, &p["hydrogen_policy"])?["enum"],
                json!(["rectify_neutral_carbon"])
            );
            assert_eq!(
                schema_node(root, &p["placement"])?["oneOf"]
                    .as_array()
                    .ok_or("placements")?
                    .len(),
                3
            );
        }
        let category;
        let mut optional = BTreeSet::new();
        if p.contains_key("size") {
            category = "sized";
            required.insert("size");
            assert_eq!(p["size"]["minimum"].as_f64(), Some(3.0));
            assert_eq!(p["size"]["maximum"].as_f64(), Some(8.0));
        } else {
            required.insert("kind");
            let kind = schema_node(root, &p["kind"])?;
            assert_eq!(kind["type"], "string");
            if kind["enum"] == json!(["benzene", "cyclopentadiene"]) {
                category = "named";
                if creation {
                    optional.insert("phase");
                    assert_eq!(
                        schema_node(root, &p["phase"])?["enum"],
                        json!(["double_first", "single_first"])
                    );
                }
            } else {
                category = "chair";
                assert_eq!(kind["enum"], json!([ONE, TWO]));
                assert!(!p.contains_key("size") && !p.contains_key("phase"));
            }
        }
        assert!(
            seen.insert(category),
            "each old/new branch occurs exactly once"
        );
        assert_eq!(
            branch["required"]
                .as_array()
                .ok_or("required")?
                .iter()
                .map(|v| v.as_str().ok_or("required key"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            required
        );
        required.extend(optional);
        assert_eq!(
            p.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            required
        );
    }
    assert_eq!(seen, ["sized", "named", "chair"].into_iter().collect());
    Ok(())
}
fn discovery(entries: &[Value], key: &str) -> TestResult {
    names(entries)?; // Original literal historical36 + the nine current additions =45.
    for (name, creation) in [("ring.create", true), ("ui.ring_tool", false)] {
        let row = entries
            .iter()
            .find(|r| r["name"] == name)
            .ok_or("ring metadata")?;
        if key == "input_schema" {
            assert_eq!(row["read_only"], false);
        } else {
            assert_eq!(row["annotations"]["readOnlyHint"], false);
        }
        schema(&row[key], creation)?;
    }
    Ok(())
}
fn rejected(client: &mut Client, id: u64, name: &str, params: Value, code: &str) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
        "params":{"name":name,"arguments":params}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], id);
    assert_eq!(reply["result"]["isError"], true);
    assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    Ok(())
}

#[test]
fn cli_chair_one_is_one_real_edit_with_history_unicode_native_save_and_editable_reopen()
-> TestResult {
    let listing = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listing.status.success(),
        "{}",
        String::from_utf8_lossy(&listing.stderr)
    );
    discovery(
        serde_json::from_slice::<Value>(&listing.stdout)?
            .as_array()
            .ok_or("commands")?,
        "input_schema",
    )?;
    let directory = support::SdfTestDirectory::new()?;
    let path = directory.path("chair-one-cli-化学-λ.cdxml");
    let copy = directory.path("chair-one-cli-再保存.cdxml");
    let wanted = expected(false, false);
    let created = cli(&[create(free(0, ONE))])?;
    graph(&created, &wanted)?;
    assert_eq!(created["revision"], 1);
    assert_eq!(created["undo_depth"], 1);
    assert_eq!(created["redo_depth"], 0);
    assert_eq!(created["changed"], true);
    assert_eq!(created["file"]["dirty"], true);
    assert_eq!(created["source"], Value::Null);
    assert!(created.get("file_operation").is_none());
    let saved = cli(&[
        create(free(0, ONE)),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":path}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    graph(&saved, &wanted)?;
    assert_eq!(saved["revision"], 3);
    assert_eq!(saved["undo_depth"], 1);
    assert_eq!(saved["redo_depth"], 0);
    assert_eq!(saved["file"]["dirty"], false);
    consume(&path, &wanted, "C1CCCCC1", "C6H12")?;
    let bytes = fs::read(&path)?;
    let reopened = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":path}}),
        json!({"command":"bond.set_order","params":{"expected_revision":1,"page_id":1,"fragment_id":1,
            "bond_id":1,"order":"single","hydrogen_policy":"rectify_neutral_carbon"}}),
        json!({"command":"document.save_cdxml","params":{"path":copy}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    graph(&reopened, &wanted)?;
    assert_eq!(
        reopened["revision"], 1,
        "a supported same-order edit is an exact no-op"
    );
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["redo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(
        reopened["source"],
        json!({"format":"cdxml","byte_length":bytes.len(),"modified":false})
    );
    assert_eq!(fs::read(&path)?, bytes);
    assert_eq!(fs::read(&copy)?, bytes);
    consume(&copy, &wanted, "C1CCCCC1", "C6H12")?;
    Ok(())
}

#[test]
fn initialized_stdio_chair_two_has_closed_schema_exact_mode_noops_and_whole_history() -> TestResult
{
    let directory = support::SdfTestDirectory::new()?;
    let path = directory.path("chair-two-stdio-保存.cdxml");
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listing = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listing["id"], 2);
    discovery(
        listing["result"]["tools"].as_array().ok_or("tools")?,
        "inputSchema",
    )?;
    let before = client.call(3, "document.inspect", json!({}))?;
    let one = client.call(4, "ui.ring_tool", json!({"kind":ONE}))?;
    operation(&one, 1, true);
    same_session(&one, &before)?;
    let noop = client.call(5, "ui.ring_tool", json!({"kind":ONE}))?;
    operation(&noop, 1, false);
    same_session(&noop, &before)?;
    let sized = client.call(6, "ui.ring_tool", json!({"size":6}))?;
    operation(&sized, 2, true);
    same_session(&sized, &before)?;
    let named = client.call(7, "ui.ring_tool", json!({"kind":"benzene"}))?;
    operation(&named, 3, true);
    same_session(&named, &before)?;
    let two = client.call(8, "ui.ring_tool", json!({"kind":TWO}))?;
    operation(&two, 4, true);
    same_session(&two, &before)?;
    controls(&client.call(9, "ui.controls", json!({}))?, TWO)?;
    for (id, name, params) in [
        (10, "ui.ring_tool", json!({"kind":ONE,"size":6})),
        (12, "ui.ring_tool", json!({"kind":Value::Null})),
        (14, "ring.create", {
            let mut v = free(0, TWO);
            v["phase"] = json!("double_first");
            v
        }),
        (16, "ring.create", {
            let mut v = free(0, TWO);
            v["size"] = json!(6);
            v
        }),
    ] {
        rejected(&mut client, id, name, params, "invalid_params")?;
        assert_eq!(client.call(id + 1, "document.inspect", json!({}))?, before);
    }
    controls(&client.call(18, "ui.controls", json!({}))?, TWO)?;
    let wanted = expected(true, false);
    let created = client.call(19, "ring.create", free(0, TWO))?;
    graph(&created, &wanted)?;
    assert_eq!(created["revision"], 1);
    assert_eq!(created["undo_depth"], 1);
    assert!(created.get("file_operation").is_none());
    let stable = client.call(20, "document.inspect", json!({}))?;
    rejected(
        &mut client,
        21,
        "ring.create",
        free(0, ONE),
        "stale_revision",
    )?;
    assert_eq!(client.call(22, "document.inspect", json!({}))?, stable);
    let saved = client.call(23, "document.save_cdxml", json!({"path":path}))?;
    assert_eq!(saved["file"]["dirty"], false);
    consume(&path, &wanted, "C1CCCCC1", "C6H12")?;
    let bytes = fs::read(&path)?;
    let undo = client.call(24, "history.undo", json!({}))?;
    assert_eq!(undo["document"]["pages"][0]["fragments"], json!([]));
    assert_eq!(undo["file"]["dirty"], true);
    let redo = client.call(25, "history.redo", json!({}))?;
    graph(&redo, &wanted)?;
    assert_eq!(redo["revision"], 3);
    assert_eq!(redo["undo_depth"], 1);
    assert_eq!(redo["redo_depth"], 0);
    assert_eq!(redo["file"]["dirty"], false);
    let reopened = client.call(26, "document.open_cdxml", json!({"path":path}))?;
    graph(&reopened, &wanted)?;
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["source"]["modified"], false);
    assert_eq!(fs::read(&path)?, bytes);
    client.finish()
}

#[test]
fn authenticated_chair_selection_repaint_shared_edge_and_opaque_source_refusal_are_truthful()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("chair-edge-input-原.cdxml");
    let output = directory.path("chair-edge-socket-保存.cdxml");
    let opaque = directory.path("chair-opaque-原.cdxml");
    let copy = directory.path("chair-opaque-保存.cdxml");
    let opaque_bytes = String::from_utf8(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/fixtures/bondline-v0.cdxml"
        ))
        .to_vec(),
    )?
    .replace("<CDXML ", "<CDXML Private=\"chair-whole-source\" ");
    fs::write(&opaque, opaque_bytes.as_bytes())?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let executor = CommandExecutor::new(Arc::clone(&engine));
    let observer = executor.clone();
    let (notify, noticed) = mpsc::channel();
    let (ack, acks) = mpsc::channel();
    let acks = Mutex::new(acks);
    let (retire, retired) = mpsc::channel();
    let server = LoopbackServer::bind_with_executor(0, executor, move || {
        let _ = notify.send(observer.has_active_work());
        let acknowledged = acks
            .lock()
            .ok()
            .is_some_and(|r| r.recv_timeout(Duration::from_secs(10)).is_ok());
        let _ = retire.send(acknowledged);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let call =
        |client: &mut BufReader<TcpStream>, name: &str, params: Value| -> TestResult<Value> {
            let result = socket(client, json!({"command":name,"params":params}))?;
            assert!(result.get("error").is_none(), "{result}");
            Ok(result["result"].clone())
        };
    let publication = || -> TestResult {
        assert!(
            !noticed.recv_timeout(Duration::from_secs(10))?,
            "release reply activity before repaint"
        );
        ack.send(())?;
        assert!(retired.recv_timeout(Duration::from_secs(10))?);
        Ok(())
    };
    call(
        &mut client,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":160,"begin_y_pt":160,
        "end_x_pt":224,"end_y_pt":160}),
    )?;
    publication()?;
    call(&mut client, "document.save_cdxml", json!({"path":input}))?;
    publication()?;
    call(&mut client, "document.open_cdxml", json!({"path":input}))?;
    publication()?;
    let original = fs::read(&input)?;
    let before = call(&mut client, "document.inspect", json!({}))?;
    assert_eq!(before["revision"], 2);
    assert_eq!(before["undo_depth"], 0);
    assert_eq!(before["file"]["dirty"], false);
    let (content, owner, model) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (
            guard.content().clone(),
            guard.capture_stamp(),
            guard.stored_document()?.model_snapshot(),
        )
    };
    let one = call(&mut client, "ui.ring_tool", json!({"kind":ONE}))?;
    operation(&one, 1, true);
    same_session(&one, &before)?;
    publication()?;
    let context = engine.lock().map_err(|_| "engine")?.ring_context(1)?;
    let noop = call(&mut client, "ui.ring_tool", json!({"kind":ONE}))?;
    operation(&noop, 1, false);
    same_session(&noop, &before)?;
    assert_eq!(call(&mut client, "document.inspect", json!({}))?, before);
    assert!(
        noticed.try_recv().is_err(),
        "same-mode no-op must not repaint"
    );
    let two = call(&mut client, "ui.ring_tool", json!({"kind":TWO}))?;
    operation(&two, 2, true);
    same_session(&two, &before)?;
    publication()?;
    let one = call(&mut client, "ui.ring_tool", json!({"kind":ONE}))?;
    operation(&one, 3, true);
    same_session(&one, &before)?;
    publication()?;
    controls(&call(&mut client, "ui.controls", json!({}))?, ONE)?;
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &content);
        assert!(guard.capture_stamp_matches(&owner));
        assert!(Arc::ptr_eq(
            &model,
            &guard.stored_document()?.model_snapshot()
        ));
        assert!(
            !context.is_current(&guard),
            "kind ABA cannot revive old gesture authority"
        );
    }
    let attached = call(
        &mut client,
        "ring.create",
        chair(
            2,
            ONE,
            json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"positive"}),
        ),
    )?;
    let wanted = expected(false, true);
    graph(&attached, &wanted)?;
    assert_eq!(attached["revision"], 3);
    assert_eq!(attached["undo_depth"], 1);
    assert_eq!(attached["file"]["dirty"], true);
    assert_eq!(
        attached["source"],
        json!({"format":"cdxml","byte_length":original.len(),"modified":true})
    );
    assert!(attached.get("file_operation").is_none());
    assert_eq!(
        attached["document"]["pages"][0]["fragments"][0]["bonds"][0],
        before["document"]["pages"][0]["fragments"][0]["bonds"][0]
    );
    publication()?;
    let saved = call(&mut client, "document.save_cdxml", json!({"path":output}))?;
    assert_eq!(saved["file"]["dirty"], false);
    publication()?;
    consume(&output, &wanted, "C1CCCCC1", "C6H12")?;
    let undo = call(&mut client, "history.undo", json!({}))?;
    assert_eq!(undo["document"], before["document"]);
    assert_eq!(undo["source"]["modified"], false);
    publication()?;
    let redo = call(&mut client, "history.redo", json!({}))?;
    graph(&redo, &wanted)?;
    assert_eq!(redo["revision"], 5);
    assert_eq!(redo["file"]["dirty"], false);
    publication()?;
    assert_eq!(fs::read(&input)?, original);
    call(&mut client, "document.open_cdxml", json!({"path":opaque}))?;
    publication()?;
    let stable = call(&mut client, "document.inspect", json!({}))?;
    let (whole, stamp) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.content().clone(), guard.capture_stamp())
    };
    for (name, params) in [
        ("ui.ring_tool", json!({"kind":ONE})),
        (
            "ring.create",
            free(stable["revision"].as_u64().ok_or("revision")?, TWO),
        ),
    ] {
        let refused = socket(&mut client, json!({"command":name,"params":params}))?;
        assert_eq!(refused["error"]["code"], "unsupported_content");
        assert_eq!(call(&mut client, "document.inspect", json!({}))?, stable);
    }
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &whole);
        assert!(guard.capture_stamp_matches(&stamp));
    }
    assert!(noticed.try_recv().is_err());
    call(&mut client, "document.save_cdxml", json!({"path":copy}))?;
    publication()?;
    assert_eq!(fs::read(&opaque)?, opaque_bytes.as_bytes());
    assert_eq!(fs::read(&copy)?, opaque_bytes.as_bytes());
    assert_eq!(fs::read(&input)?, original);
    drop(client);
    server.shutdown()?;
    Ok(())
}
