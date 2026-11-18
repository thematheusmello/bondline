//! Independent literal paths; reuse the parent module's clients and consumers unchanged.
use super::{
    Client, PROPANE, TestResult, atom, cli, consume, graph, initialize, original_propane, request,
    socket, support,
};
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

const PATH: &str = "chain.create_path";

fn path_params(revision: u64, placement: Value) -> Value {
    json!({"expected_revision":revision,"page_id":1,
        "hydrogen_policy":"rectify_neutral_carbon","placement":placement})
}
fn free_path(revision: u64) -> Value {
    path_params(
        revision,
        json!({"mode":"free","points":[
        {"x_pt":144,"y_pt":144},{"x_pt":192,"y_pt":144},
        {"x_pt":192,"y_pt":192},{"x_pt":240,"y_pt":192},
        {"x_pt":240,"y_pt":144}]}),
    )
}
fn pentane() -> Value {
    json!({"id":1,"atoms":[
        atom(1,[144.0,144.0],3,(Some(false),Some(false))),
        atom(2,[192.0,144.0],2,(Some(false),Some(false))),
        atom(3,[192.0,192.0],2,(Some(false),Some(false))),
        atom(4,[240.0,192.0],2,(Some(false),Some(false))),
        atom(5,[240.0,144.0],3,(Some(false),Some(false)))],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"single"},
        {"id":3,"begin":3,"end":4,"order":"single"},
        {"id":4,"begin":4,"end":5,"order":"single"}]})
}
fn attached_path(revision: u64) -> Value {
    path_params(
        revision,
        json!({"mode":"atom","fragment_id":1,"atom_id":2,
        "new_points":[{"x_pt":208,"y_pt":208},{"x_pt":256,"y_pt":208},
        {"x_pt":256,"y_pt":256},{"x_pt":304,"y_pt":256}]}),
    )
}
fn methylhexane() -> Value {
    json!({"id":1,"atoms":[
        atom(1,[144.0,144.0],3,(Some(true),Some(false))),
        atom(2,[208.0,144.0],1,(Some(true),Some(true))),
        atom(3,[240.0,96.0],3,(Some(false),None)),
        atom(4,[208.0,208.0],2,(Some(false),Some(false))),
        atom(5,[256.0,208.0],2,(Some(false),Some(false))),
        atom(6,[256.0,256.0],2,(Some(false),Some(false))),
        atom(7,[304.0,256.0],3,(Some(false),Some(false)))],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"single"},
        {"id":3,"begin":2,"end":4,"order":"single"},
        {"id":4,"begin":4,"end":5,"order":"single"},
        {"id":5,"begin":5,"end":6,"order":"single"},
        {"id":6,"begin":6,"end":7,"order":"single"}]})
}

#[test]
fn actual_cli_mixed_turn_path_is_one_edit_with_literal_files_and_history() -> TestResult {
    let created = cli(&[request(PATH, free_path(0))])?;
    graph(&created, &pentane())?;
    assert_eq!(created["changed"], true);
    assert_eq!(created["revision"], 1);
    assert_eq!(created["undo_depth"], 1);
    assert_eq!(created["redo_depth"], 0);
    assert!(created["source"].is_null() && created["file"]["path"].is_null());
    assert_eq!(created["file"]["dirty"], true);
    assert!(created.get("file_operation").is_none() && created.get("ui_file_operation").is_none());
    let dir = support::SdfTestDirectory::new()?;
    let output = dir.path("path-五炭-λ.cdxml");
    let undone = cli(&[
        request(PATH, free_path(0)),
        request("history.undo", json!({})),
    ])?;
    assert_eq!(undone["document"]["pages"][0]["fragments"], json!([]));
    assert_eq!(undone["revision"], 2);
    assert_eq!(undone["undo_depth"], 0);
    assert_eq!(undone["redo_depth"], 1);
    assert_eq!(undone["file"]["dirty"], false);
    let saved = cli(&[
        request(PATH, free_path(0)),
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request("document.save_cdxml", json!({"path":output})),
        request("document.inspect", json!({})),
    ])?;
    graph(&saved, &pentane())?;
    assert_eq!(saved["revision"], 3);
    assert_eq!(saved["undo_depth"], 1);
    assert_eq!(saved["redo_depth"], 0);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["file"]["path"], json!(fs::canonicalize(&output)?));
    // Unmodified parent consume() checks both flags/Decimal/raw directed integer
    // properties first, then the unchanged external RDKit named-ring consumer.
    consume(&output, &pentane(), "CCCCC", "C5H12")?;
    let bytes = fs::read(&output)?;
    let reopened = cli(&[request("document.open_cdxml", json!({"path":output}))])?;
    graph(&reopened, &pentane())?;
    assert_eq!(
        reopened["source"],
        json!({"format":"cdxml","byte_length":bytes.len(),"modified":false})
    );
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(fs::read(&output)?, bytes);
    Ok(())
}

#[test]
fn initialized_stdio_attaches_four_literal_points_without_rewriting_old_propane() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("原propane-λ.cdxml");
    let output = dir.path("七炭-path-保存.cdxml");
    fs::write(&input, PROPANE)?;
    let mut client = Client::start()?;
    initialize(&mut client)?;
    let opened = client.call(2, "document.open_cdxml", json!({"path":input}))?;
    graph(&opened, &original_propane())?;
    let created = client.call(3, PATH, attached_path(1))?;
    graph(&created, &methylhexane())?;
    assert_eq!(created["changed"], true);
    assert_eq!(created["revision"], 2);
    assert_eq!(created["undo_depth"], 1);
    assert_eq!(created["redo_depth"], 0);
    assert_eq!(
        created["source"],
        json!({"format":"cdxml","byte_length":PROPANE.len(),"modified":true})
    );
    assert_eq!(
        created["document"]["pages"][0]["fragments"][0]["bonds"][0],
        opened["document"]["pages"][0]["fragments"][0]["bonds"][0]
    );
    assert_eq!(
        created["document"]["pages"][0]["fragments"][0]["atoms"][0],
        opened["document"]["pages"][0]["fragments"][0]["atoms"][0]
    );
    assert_eq!(
        created["document"]["pages"][0]["fragments"][0]["atoms"][2],
        opened["document"]["pages"][0]["fragments"][0]["atoms"][2]
    );
    let saved = client.call(4, "document.save_cdxml", json!({"path":output}))?;
    assert_eq!(saved["file"]["dirty"], false);
    consume(&output, &methylhexane(), "CC(C)CCCC", "C7H16")?;
    let undo = client.call(5, "history.undo", json!({}))?;
    graph(&undo, &original_propane())?;
    assert_eq!(undo["source"]["modified"], false);
    assert_eq!(undo["file"]["dirty"], true);
    let redo = client.call(6, "history.redo", json!({}))?;
    graph(&redo, &methylhexane())?;
    assert_eq!(redo["file"]["dirty"], false);
    let reopened = client.call(7, "document.open_cdxml", json!({"path":output}))?;
    graph(&reopened, &methylhexane())?;
    assert_eq!(reopened["source"]["modified"], false);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(fs::read(&input)?, PROPANE);
    client.finish()
}

#[test]
fn authenticated_path_refusals_publish_nothing_and_opaque_sources_stay_complete() -> TestResult {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let changes = Arc::clone(&callbacks);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        changes.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let created = socket(&mut client, request(PATH, free_path(0)))?;
    assert!(created.get("error").is_none(), "{created}");
    graph(&created["result"], &pentane())?;
    let before = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    assert_eq!(callbacks.load(Ordering::SeqCst), 1);
    let (whole, stamp) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.content().clone(), guard.capture_stamp())
    };
    let mut extra = free_path(1);
    extra["placement"]["points"][0]["unused"] = json!(true);
    let duplicate = path_params(
        1,
        json!({"mode":"free","points":[
        {"x_pt":400,"y_pt":400},{"x_pt":400,"y_pt":400}]}),
    );
    let occupied = path_params(
        1,
        json!({"mode":"free","points":[
        {"x_pt":144,"y_pt":144},{"x_pt":96,"y_pt":144}]}),
    );
    for (params, code, disconnect) in [
        (extra, "invalid_params", true),
        (duplicate, "invalid_params", false),
        (occupied, "invalid_params", false),
        (free_path(0), "stale_revision", false),
    ] {
        let refused = socket(&mut client, request(PATH, params))?;
        assert_eq!(refused["error"]["code"], code);
        if disconnect {
            let mut end = String::new();
            assert_eq!(std::io::BufRead::read_line(&mut client, &mut end)?, 0);
            let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
            stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            stream.set_write_timeout(Some(Duration::from_secs(5)))?;
            client = BufReader::new(stream);
            assert_eq!(
                socket(&mut client, json!({"token":server.token()}))?,
                json!({"authenticated":true})
            );
        }
        assert_eq!(
            socket(&mut client, request("document.inspect", json!({})))?["result"],
            before
        );
        assert_eq!(callbacks.load(Ordering::SeqCst), 1);
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &whole);
        assert!(guard.capture_stamp_matches(&stamp));
    }
    let dir = support::SdfTestDirectory::new()?;
    let input = dir.path("opaque-path-原.cdxml");
    let output = dir.path("opaque-path-保存.cdxml");
    let raw = std::str::from_utf8(PROPANE)?.replace("<CDXML ", "<CDXML Private=\"path-source\" ");
    fs::write(&input, raw.as_bytes())?;
    let opened = socket(
        &mut client,
        request(
            "document.open_cdxml",
            json!({"path":input,"discard_unsaved":true}),
        ),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    let before = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let count = callbacks.load(Ordering::SeqCst);
    let (whole, stamp) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.content().clone(), guard.capture_stamp())
    };
    let refused = socket(
        &mut client,
        request(
            PATH,
            free_path(before["revision"].as_u64().ok_or("revision")?),
        ),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        before
    );
    assert_eq!(callbacks.load(Ordering::SeqCst), count);
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &whole);
        assert!(guard.capture_stamp_matches(&stamp));
        assert_eq!(guard.stored_document()?.export_cdxml()?, raw.as_bytes());
    }
    let saved = socket(
        &mut client,
        request("document.save_cdxml", json!({"path":output})),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    assert_eq!(fs::read(&input)?, raw.as_bytes());
    assert_eq!(fs::read(&output)?, raw.as_bytes());
    server.shutdown()?;
    Ok(())
}
