use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::Value;
#[path = "../../cli/tests/support/cdx.rs"]
mod cdx_support;
#[path = "../../cli/tests/support/chain.rs"]
mod chain_support;
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;
#[path = "../../cli/tests/support/mol.rs"]
mod mol_support;
#[path = "../../../xtask/oracles/sdf_field_test_support.rs"]
mod sdf_field_support;
#[path = "../../../xtask/oracles/sdf_record_test_support.rs"]
mod sdf_record_support;
#[path = "../../../xtask/oracles/sdf_test_support.rs"]
mod sdf_support;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{Ipv4Addr, Shutdown, SocketAddr, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

fn connect(address: SocketAddr) -> Result<BufReader<TcpStream>, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect_timeout(&address, Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    Ok(BufReader::new(stream))
}

fn call(
    client: &mut BufReader<TcpStream>,
    request: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    client.get_mut().write_all(request.as_bytes())?;
    client.get_mut().write_all(b"\n")?;
    let mut response = String::new();
    client.read_line(&mut response)?;
    Ok(serde_json::from_str(&response)?)
}

fn authenticated(
    server: &LoopbackServer,
) -> Result<BufReader<TcpStream>, Box<dyn std::error::Error>> {
    let mut client = connect(server.address())?;
    let response = call(
        &mut client,
        &serde_json::json!({"token":server.token()}).to_string(),
    )?;
    assert_eq!(response, serde_json::json!({"authenticated":true}));
    Ok(client)
}

#[test]
fn authenticated_sdf_record_queries_are_explicit_historical_readonly_without_repaint()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
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
            "socket-query-mixed.sdf"
        } else {
            "socket-query-ordered.sdf"
        });
        std::fs::write(&path, source)?;
        call(
            &mut client,
            &serde_json::json!({"command":"document.open_sdf","params":{"path":path}}).to_string(),
        )?;
        wait_for_changes(&changes, case + 1);
        let before = call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?;
        assert!(before["result"].get("sdf_record").is_none());
        for record_id in [1, 2] {
            let response=call(&mut client,&serde_json::json!({"command":"sdf.record.inspect","params":{"expected_revision":case+1,"record_id":record_id}}).to_string())?;
            let mut result = response["result"].clone();
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
            assert_eq!(result, before["result"]);
        }
        for (params, code) in [
            (
                serde_json::json!({"expected_revision":0,"record_id":1}),
                "stale_revision",
            ),
            (
                serde_json::json!({"expected_revision":case+1,"record_id":3}),
                "not_found",
            ),
            (
                serde_json::json!({"expected_revision":case+1,"record_id":1,"field_limit":65}),
                "invalid_params",
            ),
        ] {
            let response = call(
                &mut client,
                &serde_json::json!({"command":"sdf.record.inspect","params":params}).to_string(),
            )?;
            assert_eq!(response["error"]["code"], code);
            assert_eq!(
                call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?,
                before
            );
        }
        assert_eq!(changes.load(Ordering::SeqCst), case + 1);
        sdf_support::independently_read_sdf(&path, source, mixed)?;
    }
    server.shutdown()?;
    Ok(())
}

#[test]
fn authenticated_field_edit_save_undo_redo_preserves_raw_records_and_notifies_only_effects()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let mut notifications = 0;
    for (case, mixed) in [false, true].into_iter().enumerate() {
        let input = directory.path(if mixed {
            "socket-set-mixed.sdf"
        } else {
            "socket-set-ordered.sdf"
        });
        let output = directory.path(if mixed {
            "socket-set-mixed-λ.data"
        } else {
            "socket-set-ordered-λ.data"
        });
        std::fs::write(&input, sdf_field_support::source(mixed))?;
        let revision = case * 4 + 1;
        let index = sdf_field_support::field_index(mixed);
        call(
            &mut client,
            &serde_json::json!({"command":"document.open_sdf","params":{"path":input}}).to_string(),
        )?;
        notifications += 1;
        wait_for_changes(&changes, notifications);
        let query=call(&mut client,&serde_json::json!({"command":"sdf.record.inspect","params":{"expected_revision":revision,"record_id":1}}).to_string())?;
        assert_eq!(
            query["result"]["sdf_record"]["fields"],
            sdf_record_support::expected_fields(mixed, 1)
        );
        let changed=call(&mut client,&serde_json::json!({"command":"sdf.field.set_value","params":{"expected_revision":revision,"record_id":1,"field_index":index,"value_lines":["revised","multiline"]}}).to_string())?;
        assert_eq!(changed["result"]["changed"], true);
        assert_eq!(changed["result"]["revision"], revision + 1);
        assert_eq!(changed["result"]["file"]["dirty"], true);
        assert_eq!(changed["result"]["source"]["modified"], true);
        assert!(changed["result"].get("sdf_record").is_none());
        notifications += 1;
        wait_for_changes(&changes, notifications);
        let query=call(&mut client,&serde_json::json!({"command":"sdf.record.inspect","params":{"expected_revision":revision+1,"record_id":1}}).to_string())?;
        assert_eq!(
            query["result"]["sdf_record"]["fields"][index]["value_lines"],
            serde_json::json!(["revised", "multiline"])
        );
        let saved = call(
            &mut client,
            &serde_json::json!({"command":"document.save_sdf","params":{"path":output}})
                .to_string(),
        )?;
        assert_eq!(
            saved["result"]["file_operation"]["byte_length"],
            if mixed { 788 } else { 704 }
        );
        assert_eq!(saved["result"]["file"]["dirty"], false);
        assert_eq!(saved["result"]["source"]["modified"], true);
        notifications += 1;
        wait_for_changes(&changes, notifications);
        sdf_field_support::assert_edited_file(&output, mixed)?;
        let undo = call(&mut client, r#"{"command":"history.undo","params":{}}"#)?;
        assert_eq!(undo["result"]["revision"], revision + 2);
        assert_eq!(undo["result"]["file"]["dirty"], true);
        assert_eq!(undo["result"]["source"]["modified"], false);
        notifications += 1;
        wait_for_changes(&changes, notifications);
        let redo = call(&mut client, r#"{"command":"history.redo","params":{}}"#)?;
        assert_eq!(redo["result"]["revision"], revision + 3);
        assert_eq!(redo["result"]["file"]["dirty"], false);
        assert_eq!(redo["result"]["source"]["modified"], true);
        notifications += 1;
        wait_for_changes(&changes, notifications);
        let query=call(&mut client,&serde_json::json!({"command":"sdf.record.inspect","params":{"expected_revision":revision+3,"record_id":1}}).to_string())?;
        assert_eq!(
            query["result"]["sdf_record"]["fields"][index]["value_lines"],
            serde_json::json!(["revised", "multiline"])
        );
        let before = call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?;
        let noop=call(&mut client,&serde_json::json!({"command":"sdf.field.set_value","params":{"expected_revision":revision+3,"record_id":1,"field_index":index,"value_lines":["revised","multiline"]}}).to_string())?;
        assert_eq!(noop, before);
        for (params, code) in [
            (
                serde_json::json!({"expected_revision":revision,"record_id":1,"field_index":index,"value_lines":[]}),
                "stale_revision",
            ),
            (
                serde_json::json!({"expected_revision":revision+3,"record_id":1,"field_index":999,"value_lines":[]}),
                "not_found",
            ),
            (
                serde_json::json!({"expected_revision":revision+3,"record_id":1,"field_index":index,"value_lines":[""]}),
                "invalid_sdf",
            ),
        ] {
            let failed = call(
                &mut client,
                &serde_json::json!({"command":"sdf.field.set_value","params":params}).to_string(),
            )?;
            assert_eq!(failed["error"]["code"], code);
            assert_eq!(
                call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?,
                before
            );
        }
        assert_eq!(changes.load(Ordering::SeqCst), notifications);
        assert_eq!(std::fs::read(input)?, sdf_field_support::source(mixed));
    }
    server.shutdown()?;
    Ok(())
}

fn wait_for_changes(changes: &AtomicUsize, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while changes.load(Ordering::SeqCst) < expected && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(changes.load(Ordering::SeqCst), expected);
}

#[test]
fn authenticated_loopback_sdf_open_save_reopen_and_failure_retention_use_one_session()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let mut notifications = 0;
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
            "socket-mixed.sdf"
        } else {
            "socket-ordered.sd"
        });
        let output = directory.path(if mixed {
            "socket-mixed-λ.data"
        } else {
            "socket-ordered-λ.data"
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
        for (command, params, notify) in [
            ("document.open_sdf", serde_json::json!({"path":input}), true),
            ("document.inspect", serde_json::json!({}), false),
            (
                "document.save_sdf",
                serde_json::json!({"path":output}),
                true,
            ),
            (
                "document.open_sdf",
                serde_json::json!({"path":output}),
                true,
            ),
        ] {
            let response = call(
                &mut client,
                &serde_json::json!({"command":command,"params":params}).to_string(),
            )?;
            let result = &response["result"];
            assert_eq!(result["document"], serde_json::json!(null), "{response}");
            assert_eq!(result["collection"], sdf_support::expected_summary(mixed));
            assert_eq!(result["file"]["format"], "sdf");
            if notify {
                notifications += 1;
                wait_for_changes(&changes, notifications);
            }
        }
        sdf_support::independently_read_sdf(&output, source, mixed)?;
        let baseline = call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?;
        for (command, params, code) in [
            (
                "document.open_sdf",
                serde_json::json!({"path":bad}),
                "invalid_sdf",
            ),
            (
                "document.save_sdf",
                serde_json::json!({"path":output}),
                "already_exists",
            ),
            (
                "document.save_cdxml",
                serde_json::json!({"path":directory.path("must-not-save.cdxml")}),
                "unsupported_content",
            ),
        ] {
            let response = call(
                &mut client,
                &serde_json::json!({"command":command,"params":params}).to_string(),
            )?;
            assert_eq!(response["error"]["code"], code);
            assert_eq!(
                call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?,
                baseline
            );
        }
        assert!(!directory.path("must-not-save.cdxml").exists());
        assert_eq!(std::fs::read(&output)?, source);
    }
    assert_eq!(changes.load(Ordering::SeqCst), notifications);
    server.shutdown()?;
    Ok(())
}

#[test]
fn authenticated_loopback_extends_and_saves_the_same_typed_native_graph()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("loopback-chain.cdxml");
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let executor = bondline_mcp::CommandExecutor::new(Arc::clone(&engine));
    let server = LoopbackServer::bind_with_executor(0, executor, || {})?;
    let mut client = authenticated(&server)?;
    let created = call(
        &mut client,
        &serde_json::json!({"command":"bond.create","params":chain_support::create()}).to_string(),
    )?;
    let extended=call(&mut client,&serde_json::json!({"command":"bond.extend","params":chain_support::extend(1,2,180.0,240.0)}).to_string())?;
    assert_eq!(extended["result"]["revision"], 2);
    let graph = extended["result"]["document"].clone();
    assert_eq!(
        graph["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"],
        2
    );
    let stale=call(&mut client,&serde_json::json!({"command":"bond.extend","params":chain_support::extend(1,3,220.0,280.0)}).to_string())?;
    assert_eq!(stale["error"]["code"], "stale_revision");
    let saved = call(
        &mut client,
        &serde_json::json!({"command":"document.save_cdxml","params":{"path":path}}).to_string(),
    )?;
    assert_eq!(saved["result"]["file"]["dirty"], false);
    chain_support::independently_read(&path, 3)?;
    let undone = call(&mut client, r#"{"command":"history.undo","params":{}}"#)?;
    assert_eq!(undone["result"]["document"], created["result"]["document"]);
    let redone = call(&mut client, r#"{"command":"history.redo","params":{}}"#)?;
    assert_eq!(redone["result"]["document"], graph);
    server.shutdown()?;
    Ok(())
}

#[test]
fn authenticated_cdx_open_and_receipt_only_save_keep_binary_source_and_notify()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let input = directory.path("original.mol");
    let copy = directory.path("copy-λ.cdx");
    std::fs::write(&input, cdx_support::SOURCE)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let opened = call(
        &mut client,
        &serde_json::json!({"command":"document.open_cdx","params":{"path":input}}).to_string(),
    )?;
    cdx_support::assert_state(&opened["result"])?;
    wait_for_changes(&changes, 1);
    let saved = call(
        &mut client,
        &serde_json::json!({"command":"document.save_cdx","params":{"path":copy}}).to_string(),
    )?;
    cdx_support::assert_state(&saved["result"])?;
    assert_eq!(saved["result"]["changed"], false);
    assert_eq!(saved["result"]["revision"], 1);
    assert_eq!(saved["result"]["file_operation"]["operation"], "save");
    assert_eq!(saved["result"]["file_operation"]["format"], "cdx");
    wait_for_changes(&changes, 2);
    cdx_support::independently_read_saved(&copy)?;
    let before = call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?;
    for (command, params, code) in [
        (
            "document.save_cdx",
            serde_json::json!({"path":copy}),
            "already_exists",
        ),
        (
            "document.save_cdxml",
            serde_json::json!({"path":copy,"overwrite":true}),
            "unsupported_content",
        ),
    ] {
        let failed = call(
            &mut client,
            &serde_json::json!({"command":command,"params":params}).to_string(),
        )?;
        assert_eq!(failed["error"]["code"], code);
        assert_eq!(
            call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?,
            before
        );
    }
    // Invalid parameter shapes close that connection; another authenticated client retains the session.
    let mut malformed_client = authenticated(&server)?;
    let rejected=call(&mut malformed_client,&serde_json::json!({"command":"document.open_cdx","params":{"path":input,"unexpected":true}}).to_string())?;
    assert_eq!(rejected["error"]["code"], "invalid_params");
    assert_eq!(
        call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), 2);
    assert_eq!(std::fs::read(copy)?, cdx_support::SOURCE);
    server.shutdown()?;
    Ok(())
}

#[test]
fn authenticated_file_commands_share_native_file_state_and_notify_after_saving()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("native-loopback.cdxml");
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let executor = bondline_mcp::CommandExecutor::new(Arc::clone(&engine));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind_with_executor(0, executor, move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let drawn = call(&mut client,&serde_json::json!({"command":"bond.create","params":{"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0}}).to_string())?;
    assert_eq!(drawn["result"]["file"]["dirty"], true);
    let saved = call(
        &mut client,
        &serde_json::json!({"command":"document.save_cdxml","params":{"path":path}}).to_string(),
    )?;
    assert_eq!(saved["result"]["revision"], 1);
    assert_eq!(
        saved["result"]["file"],
        serde_json::json!({"path":std::fs::canonicalize(&path)?,"dirty":false,"format":"cdxml"})
    );
    wait_for_changes(&changes, 2);
    file_support::independently_read_file(&path, 1)?;
    let opened = call(
        &mut client,
        &serde_json::json!({"command":"document.open_cdxml","params":{"path":path}}).to_string(),
    )?;
    assert_eq!(opened["result"]["revision"], 2);
    assert_eq!(opened["result"]["undo_depth"], 0);
    wait_for_changes(&changes, 3);
    assert_eq!(
        engine.lock().map_err(|_| "engine")?.state().file.path,
        Some(
            std::fs::canonicalize(&path)?
                .to_str()
                .ok_or("UTF-8 path")?
                .to_owned()
        )
    );
    server.shutdown()?;
    Ok(())
}

#[test]
fn authenticated_v3000_exports_preserves_sources_and_repaints_receipt_only_saves()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let native = directory.path("loopback-native-v3.v2000");
    let source = directory.path("loopback-original-v3.v2000");
    let copy = directory.path("loopback-v3-copy.mol");
    let malformed = directory.path("loopback-v3-truncated.mol");
    let unknown = directory.path("loopback-unknown-version.mol");
    std::fs::write(&source, mol_support::V3000_SOURCE)?;
    let original = std::str::from_utf8(mol_support::V3000_SOURCE)?;
    std::fs::write(&malformed, original.replace("M  V30 END CTAB\n", ""))?;
    std::fs::write(&unknown, original.replace(" V3000", " V4000"))?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let drawn = call(
        &mut client,
        r#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}}"#,
    )?;
    assert_eq!(drawn["result"]["revision"], 1);
    wait_for_changes(&changes, 1);
    let exported = call(&mut client,&serde_json::json!({"command":"document.save_mol","params":{"path":native,"version":"v3000"}}).to_string())?;
    assert!(exported.get("error").is_none(), "{exported}");
    assert_eq!(exported["result"]["changed"], false);
    assert_eq!(exported["result"]["revision"], 1);
    assert_eq!(exported["result"]["file"]["format"], "mol_v3000");
    assert_eq!(exported["result"]["file_operation"]["format"], "mol_v3000");
    assert_eq!(exported["result"]["file_operation"]["input_revision"], 1);
    wait_for_changes(&changes, 2);
    let external = mol_support::independently_read_v3000(&native)?;
    assert_eq!(external["rdkit"], "2026.09.2");
    assert_eq!(external["atoms"].as_array().ok_or("atoms")?.len(), 2);
    assert_eq!(
        external["bonds"],
        serde_json::json!([{"begin":0,"end":1,"order":1.0,"direction":"NONE","stereo":"STEREONONE"}])
    );
    for (index, x) in [28.0 / 24.0, 68.0 / 24.0].into_iter().enumerate() {
        let atom = &external["atoms"][index];
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_h"], 3);
        assert!((atom["position"][0].as_f64().ok_or("x")? - x).abs() < 1e-14);
        assert!((atom["position"][1].as_f64().ok_or("y")? + 128.0 / 24.0).abs() < 1e-14);
        assert_eq!(atom["position"][2], 0.0);
    }
    let opened = call(
        &mut client,
        &serde_json::json!({"command":"document.open_mol","params":{"path":source}}).to_string(),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    mol_support::assert_v3000_state(&opened["result"])?;
    assert_eq!(opened["result"]["revision"], 2);
    assert_eq!(opened["result"]["undo_depth"], 0);
    assert_eq!(opened["result"]["file_operation"]["format"], "mol_v3000");
    wait_for_changes(&changes, 3);
    let saved = call(
        &mut client,
        &serde_json::json!({"command":"document.save_mol","params":{"path":copy}}).to_string(),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    mol_support::assert_v3000_state(&saved["result"])?;
    assert_eq!(saved["result"]["changed"], false);
    assert_eq!(saved["result"]["revision"], 2);
    assert_eq!(saved["result"]["file_operation"]["format"], "mol_v3000");
    assert_eq!(saved["result"]["file_operation"]["byte_length"], 344);
    wait_for_changes(&changes, 4);
    assert_eq!(std::fs::read(&copy)?, mol_support::V3000_SOURCE);
    assert_eq!(
        mol_support::independently_read_v3000(&copy)?["source_sha256"],
        "1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380"
    );
    let edited = call(
        &mut client,
        r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}"#,
    )?;
    assert_eq!(edited["result"]["revision"], 3);
    wait_for_changes(&changes, 5);
    let undone = call(&mut client, r#"{"command":"history.undo","params":{}}"#)?;
    assert_eq!(undone["result"]["revision"], 4);
    wait_for_changes(&changes, 6);
    let baseline = call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?;
    assert_eq!(baseline["result"]["redo_depth"], 1);
    mol_support::assert_v3000_state(&baseline["result"])?;
    for (name, params, code) in [
        (
            "document.save_mol",
            serde_json::json!({"path":copy,"overwrite":true,"version":"v2000"}),
            "unsupported_content",
        ),
        (
            "document.open_mol",
            serde_json::json!({"path":malformed}),
            "invalid_mol",
        ),
        (
            "document.open_mol",
            serde_json::json!({"path":unknown}),
            "invalid_mol",
        ),
    ] {
        let rejected = call(
            &mut client,
            &serde_json::json!({"command":name,"params":params}).to_string(),
        )?;
        assert_eq!(rejected["error"]["code"], code, "{rejected}");
        assert_eq!(
            call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?,
            baseline
        );
        assert_eq!(std::fs::read(&copy)?, mol_support::V3000_SOURCE);
    }
    assert_eq!(changes.load(Ordering::SeqCst), 6);
    let saved = call(
        &mut client,
        &serde_json::json!({"command":"document.save_mol","params":{"path":copy,"overwrite":true}})
            .to_string(),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    mol_support::assert_v3000_state(&saved["result"])?;
    assert_eq!(saved["result"]["changed"], false);
    assert_eq!(saved["result"]["revision"], 4);
    assert_eq!(saved["result"]["redo_depth"], 1);
    assert_eq!(saved["result"]["file_operation"]["input_revision"], 4);
    assert_eq!(saved["result"]["file_operation"]["format"], "mol_v3000");
    wait_for_changes(&changes, 7);
    assert_eq!(std::fs::read(&copy)?, mol_support::V3000_SOURCE);
    server.shutdown()?;
    Ok(())
}

#[test]
fn authenticated_mol_io_preserves_bytes_errors_history_and_receipt_repaints()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let source = directory.path("loopback-methyl.mol");
    let destination = directory.path("loopback-copy.mol");
    let malformed = directory.path("loopback-malformed.mol");
    let unsupported = directory.path("loopback-unsupported.mol");
    std::fs::write(&source, mol_support::METHYL)?;
    std::fs::write(&malformed, b"not a MOL file\n")?;
    let extended =
        std::str::from_utf8(mol_support::METHYL)?.replace("M  END", "M  XYZ  1   1   7\nM  END");
    std::fs::write(&unsupported, extended)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let opened = call(
        &mut client,
        &serde_json::json!({"command":"document.open_mol","params":{"path":source}}).to_string(),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    mol_support::assert_methyl_state(&opened["result"], 42.3, 20.3784)?;
    assert_eq!(opened["result"]["revision"], 1);
    wait_for_changes(&changes, 1);
    let saved = call(
        &mut client,
        &serde_json::json!({"command":"document.save_mol","params":{"path":destination}})
            .to_string(),
    )?;
    let saved = &saved["result"];
    mol_support::assert_methyl_state(saved, 42.3, 20.3784)?;
    assert_eq!(saved["revision"], 1);
    assert_eq!(saved["changed"], false);
    assert_eq!(
        saved["file"],
        serde_json::json!({"path":std::fs::canonicalize(&destination)?,"dirty":false,"format":"mol_v2000"})
    );
    assert_eq!(saved["file_operation"]["operation"], "save");
    assert_eq!(saved["file_operation"]["byte_length"], 170);
    assert_eq!(saved["file_operation"]["input_revision"], 1);
    wait_for_changes(&changes, 2);
    assert_eq!(std::fs::read(&destination)?, mol_support::METHYL);
    assert_eq!(
        mol_support::independently_read_mol(&destination)?["smiles"],
        "[CH3]"
    );
    let edited = call(
        &mut client,
        r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}"#,
    )?;
    assert_eq!(edited["result"]["revision"], 2);
    assert_eq!(edited["result"]["file"]["dirty"], true);
    wait_for_changes(&changes, 3);
    let before = call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?;
    for (path, discard, code) in [
        (&source, false, "unsaved_changes"),
        (&malformed, true, "invalid_mol"),
        (&unsupported, true, "unsupported_content"),
    ] {
        let failed = call(&mut client,&serde_json::json!({"command":"document.open_mol","params":{"path":path,"discard_unsaved":discard}}).to_string())?;
        assert_eq!(failed["error"]["code"], code, "{failed}");
        let after = call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?;
        assert_eq!(after, before);
    }
    assert_eq!(std::fs::read(&destination)?, mol_support::METHYL);
    let undone = call(&mut client, r#"{"command":"history.undo","params":{}}"#)?;
    mol_support::assert_methyl_state(&undone["result"], 42.3, 20.3784)?;
    assert_eq!(undone["result"]["file"], saved["file"]);
    assert_eq!(undone["result"]["revision"], 3);
    wait_for_changes(&changes, 4);
    let saved_again = call(&mut client,&serde_json::json!({"command":"document.save_mol","params":{"path":destination,"overwrite":true}}).to_string())?;
    assert_eq!(saved_again["result"]["changed"], false);
    assert_eq!(saved_again["result"]["revision"], 3);
    assert_eq!(saved_again["result"]["file_operation"]["input_revision"], 3);
    assert_eq!(saved_again["result"]["file_operation"]["byte_length"], 170);
    wait_for_changes(&changes, 5);
    assert_eq!(std::fs::read(&destination)?, mol_support::METHYL);
    server.shutdown()?;
    Ok(())
}

#[test]
fn unauthenticated_clients_cannot_inspect_or_edit_the_document()
-> Result<(), Box<dyn std::error::Error>> {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let before = serde_json::to_vec(&engine.lock().map_err(|_| "poisoned engine")?.state())?;
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    for request in [
        r#"{"command":"document.inspect","params":{}}"#,
        r#"{"command":"document.open_mol","params":{"path":"must-not-read.mol"}}"#,
        r#"{"command":"document.open_cdx","params":{"path":"must-not-read.cdx"}}"#,
        r#"{"command":"document.close","params":{"expected_revision":0}}"#,
        r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}"#,
        r#"{"token":"wrong"}"#,
        r#"{"token":null}"#,
        r#"{"token":"wrong","extra":true}"#,
        r#"{"token":"one","token":"two"}"#,
        "[]",
        "{",
    ] {
        let mut client = connect(server.address())?;
        assert_eq!(
            call(&mut client, request)?,
            serde_json::json!({"error":{
                "code":"unauthorized", "message":"valid session token required"
            }})
        );
        match client.read(&mut [0; 1]) {
            Ok(0) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
                ) => {}
            other => return Err(format!("rejected client stayed connected: {other:?}").into()),
        }
    }
    assert_eq!(
        serde_json::to_vec(&engine.lock().map_err(|_| "poisoned engine")?.state())?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), 0);
    server.shutdown()?;
    Ok(())
}

#[test]
fn tokens_are_session_scoped_and_authentication_rejects_ambiguous_objects()
-> Result<(), Box<dyn std::error::Error>> {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let first = LoopbackServer::bind(0, Arc::clone(&engine), || {})?;
    let second = LoopbackServer::bind(0, engine, || {})?;
    assert_eq!(first.token().len(), 64);
    assert!(
        first
            .token()
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    assert_ne!(first.token(), second.token());
    for request in [
        serde_json::json!({"token":first.token()}).to_string(),
        format!("{{\"token\":\"wrong\",\"token\":\"{}\"}}", second.token()),
        serde_json::json!({"token":second.token(),"command":"document.inspect","params":{}})
            .to_string(),
    ] {
        let mut client = connect(second.address())?;
        let response = call(&mut client, &request)?;
        assert_eq!(
            response,
            serde_json::json!({"error":{
                "code":"unauthorized", "message":"valid session token required"
            }})
        );
        assert!(!response.to_string().contains(first.token()));
        assert!(!response.to_string().contains(second.token()));
    }
    let mut client = authenticated(&second)?;
    assert_eq!(
        call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?["result"]["revision"],
        0
    );
    first.shutdown()?;
    second.shutdown()?;
    Ok(())
}

#[test]
fn authentication_compares_decoded_json_strings() -> Result<(), Box<dyn std::error::Error>> {
    let server = LoopbackServer::bind(0, Arc::new(Mutex::new(Engine::default())), || {})?;
    let escaped: String = server
        .token()
        .chars()
        .map(|character| format!("\\u{:04x}", u32::from(character)))
        .collect();
    let mut client = connect(server.address())?;
    assert_eq!(
        call(&mut client, &format!("{{\"token\":\"{escaped}\"}}"))?,
        serde_json::json!({"authenticated":true})
    );
    assert_eq!(
        call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?["result"]["revision"],
        0
    );
    server.shutdown()?;
    Ok(())
}

#[test]
fn independent_socket_clients_share_the_native_engine_and_notify_on_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    assert_eq!(server.address().ip(), Ipv4Addr::LOCALHOST);
    let mut first = authenticated(&server)?;
    let mut second = authenticated(&server)?;
    let edited = call(
        &mut first,
        r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}"#,
    )?;
    assert_eq!(edited["result"]["revision"], 1);
    let inspected = call(&mut second, r#"{"command":"document.inspect","params":{}}"#)?;
    assert_eq!(
        inspected["result"]["document"]["pages"][0]["size"]["width_pt"],
        612.0
    );
    wait_for_changes(&changes, 1);
    call(&mut second, r#"{"command":"history.undo","params":{}}"#)?;
    assert_eq!(
        engine
            .lock()
            .map_err(|_| "poisoned engine")?
            .state()
            .revision,
        2
    );
    wait_for_changes(&changes, 2);
    let invalid = call(
        &mut first,
        r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":0,"height_pt":72}}"#,
    )?;
    assert_eq!(invalid["error"]["code"], "invalid_params");
    assert_eq!(
        engine
            .lock()
            .map_err(|_| "poisoned engine")?
            .state()
            .revision,
        2
    );
    server.shutdown()?;
    Ok(())
}

#[test]
fn socket_capture_reports_the_backend_and_keeps_shared_history_unchanged()
-> Result<(), Box<dyn std::error::Error>> {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let before = serde_json::to_vec(&engine.lock().map_err(|_| "poisoned engine")?.state())?;
    let mut cpu_png = None;
    for backend in ["cpu", "gpu"] {
        let response = call(
            &mut client,
            &serde_json::json!({"command":"render.capture","params":{
                "page_id":1,"width_px":8,"height_px":8,"backend":backend
            }})
            .to_string(),
        )?;
        let result = &response["result"];
        assert_eq!(result["revision"], 0);
        assert_eq!(result["changed"], false);
        let capture = &result["capture"];
        assert_eq!(capture["backend"], backend);
        if backend == "cpu" {
            assert!(capture["adapter"].is_null());
            cpu_png = Some(capture["png_base64"].clone());
        } else {
            assert!(capture["adapter"]["name"].is_string());
            assert!(capture["adapter"]["backend"].is_string());
            assert_eq!(Some(capture["png_base64"].clone()), cpu_png);
        }
    }
    assert_eq!(
        serde_json::to_vec(&engine.lock().map_err(|_| "poisoned engine")?.state())?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), 0);
    server.shutdown()?;
    Ok(())
}

#[test]
fn framing_is_bounded_and_http_or_truncated_messages_cannot_execute_commands()
-> Result<(), Box<dyn std::error::Error>> {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let server = LoopbackServer::bind(0, Arc::clone(&engine), || {})?;
    let mut huge = connect(server.address())?;
    huge.get_mut().write_all(&vec![b' '; 1024 * 1024 + 1])?;
    let mut response = String::new();
    huge.read_line(&mut response)?;
    assert_eq!(
        serde_json::from_str::<Value>(&response)?["error"]["code"],
        "invalid_params"
    );
    let mut truncated = connect(server.address())?;
    truncated.get_mut().write_all(
        br#"{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}"#,
    )?;
    truncated.get_mut().shutdown(Shutdown::Write)?;
    response.clear();
    truncated.read_line(&mut response)?;
    assert_eq!(
        serde_json::from_str::<Value>(&response)?["error"]["code"],
        "invalid_frame"
    );
    let mut http = connect(server.address())?;
    http.get_mut().write_all(b"POST / HTTP/1.1\r\n\r\n{\"command\":\"page.set_size\",\"params\":{\"page_id\":1,\"width_pt\":612,\"height_pt\":792}}\n")?;
    let mut closed = Vec::new();
    match http.read_to_end(&mut closed) {
        Ok(_) => {}
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::UnexpectedEof
            ) => {}
        Err(error) => return Err(error.into()),
    }
    assert_eq!(
        engine
            .lock()
            .map_err(|_| "poisoned engine")?
            .state()
            .revision,
        0
    );
    server.shutdown()?;
    Ok(())
}

#[test]
fn shutdown_cancels_idle_clients_and_releases_the_listener()
-> Result<(), Box<dyn std::error::Error>> {
    let server = LoopbackServer::bind(0, Arc::new(Mutex::new(Engine::default())), || {})?;
    let address = server.address();
    let mut idle = connect(address)?;
    idle.get_mut().write_all(b"{\"command\":")?;
    let start = Instant::now();
    server.shutdown()?;
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(TcpStream::connect_timeout(&address, Duration::from_millis(200)).is_err());
    Ok(())
}

#[test]
fn excess_connections_are_closed_instead_of_spawning_unbounded_tasks()
-> Result<(), Box<dyn std::error::Error>> {
    let server = LoopbackServer::bind(0, Arc::new(Mutex::new(Engine::default())), || {})?;
    let mut clients = Vec::new();
    for _ in 0..16 {
        let mut client = authenticated(&server)?;
        assert_eq!(
            call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?["result"]["revision"],
            0
        );
        clients.push(client);
    }
    let mut extra = connect(server.address())?;
    let _ = extra
        .get_mut()
        .write_all(b"{\"command\":\"document.inspect\",\"params\":{}}\n");
    match extra.read(&mut [0; 1]) {
        Ok(0) => {}
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::NotConnected
            ) => {}
        other => return Err(format!("excess connection was not closed: {other:?}").into()),
    }
    server.shutdown()?;
    Ok(())
}

#[test]
fn authenticated_bond_edits_history_and_captures_share_the_live_model()
-> Result<(), Box<dyn std::error::Error>> {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    call(
        &mut client,
        r#"{"command":"page.set_size","params":{"page_id":1,"width_pt":60,"height_pt":60}}"#,
    )?;
    let created = call(
        &mut client,
        r#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":10,"begin_y_pt":20.5,"end_x_pt":50,"end_y_pt":20.5}}"#,
    )?;
    assert_eq!(created["result"]["revision"], 2);
    assert_eq!(created["result"]["undo_depth"], 2);
    assert_eq!(
        created["result"]["document"]["pages"][0]["fragments"][0]["atoms"][0]["explicit_hydrogens"],
        3
    );
    let undo = call(&mut client, r#"{"command":"history.undo","params":{}}"#)?;
    assert_eq!(
        undo["result"]["document"]["pages"][0]["fragments"],
        serde_json::json!([])
    );
    let redo = call(&mut client, r#"{"command":"history.redo","params":{}}"#)?;
    assert_eq!(redo["result"]["document"], created["result"]["document"]);
    let cpu = call(
        &mut client,
        r#"{"command":"render.capture","params":{"page_id":1,"width_px":80,"height_px":80,"backend":"cpu"}}"#,
    )?;
    let gpu = call(
        &mut client,
        r#"{"command":"render.capture","params":{"page_id":1,"width_px":80,"height_px":80,"backend":"gpu"}}"#,
    )?;
    assert_eq!(
        cpu["result"]["capture"]["png_base64"],
        gpu["result"]["capture"]["png_base64"]
    );
    assert_eq!(gpu["result"]["revision"], 4);
    assert!(gpu["result"]["capture"]["adapter"].is_object());
    wait_for_changes(&changes, 4);
    server.shutdown()?;
    Ok(())
}

#[test]
fn authenticated_close_decisions_repaint_after_reply_and_native_commit_blocks_new_work()
-> Result<(), Box<dyn std::error::Error>> {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let executor = bondline_mcp::CommandExecutor::new(Arc::clone(&engine));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind_with_executor(0, executor.clone(), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let created = call(
        &mut client,
        &serde_json::json!({"command":"bond.create","params":{
            "page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200
        }})
        .to_string(),
    )?;
    let document = created["result"]["document"].clone();
    wait_for_changes(&changes, 1);
    for params in [
        serde_json::json!({"expected_revision":1,"action":"quit"}),
        serde_json::json!({}),
        serde_json::json!({"expected_revision":1,"force":true}),
    ] {
        let mut invalid = authenticated(&server)?;
        let result = call(
            &mut invalid,
            &serde_json::json!({"command":"document.close","params":params}).to_string(),
        )?;
        assert_eq!(result["error"]["code"], "invalid_params");
    }
    let stale = call(
        &mut client,
        r#"{"command":"document.close","params":{"expected_revision":0}}"#,
    )?;
    assert_eq!(stale["error"]["code"], "stale_revision");
    assert_eq!(engine.lock().map_err(|_| "engine")?.state().close_epoch, 0);
    assert_eq!(changes.load(Ordering::SeqCst), 1);
    for (action, state, epoch) in [
        (None, "needs_decision", 1),
        (Some("cancel"), "open", 2),
        (None, "needs_decision", 3),
        (Some("discard"), "approved", 4),
    ] {
        let mut params = serde_json::json!({"expected_revision":1});
        if let Some(action) = action {
            params["action"] = action.into();
        }
        let response = call(
            &mut client,
            &serde_json::json!({"command":"document.close","params":params}).to_string(),
        )?;
        let result = &response["result"];
        assert_eq!(result["close"], state);
        assert_eq!(result["close_operation"], state);
        assert_eq!(result["close_epoch"], epoch);
        assert_eq!(result["changed"], false);
        assert_eq!(result["revision"], 1);
        assert_eq!(result["undo_depth"], 1);
        assert_eq!(result["document"], document);
        assert_eq!(result["file"]["dirty"], true);
        wait_for_changes(&changes, epoch + 1);
    }
    {
        let mut engine = engine.lock().map_err(|_| "engine")?;
        assert!(
            !executor.has_active_work(),
            "reply publication must drain before native acknowledgement"
        );
        assert!(engine.complete_close());
    }
    let capture = call(
        &mut client,
        r#"{"command":"render.capture","params":{"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}}"#,
    )?;
    assert_eq!(capture["error"]["code"], "close_committed");
    let inspected = call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?;
    assert_eq!(inspected["result"]["close"], "committed");
    assert_eq!(inspected["result"]["close_epoch"], 4);
    assert_eq!(inspected["result"]["document"], document);
    assert!(inspected["result"].get("close_operation").is_none());
    assert_eq!(changes.load(Ordering::SeqCst), 5);
    server.shutdown()?;
    Ok(())
}

#[test]
fn authenticated_presentation_requests_require_a_host_and_remain_requested_until_its_acknowledgement()
-> Result<(), Box<dyn std::error::Error>> {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let executor = bondline_mcp::CommandExecutor::new(Arc::clone(&engine));
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let server = LoopbackServer::bind_with_executor(0, executor.clone(), move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut client = authenticated(&server)?;
    let headless = call(&mut client, r#"{"command":"ui.controls","params":{}}"#)?;
    assert_eq!(
        headless["result"]["ui_file"],
        serde_json::json!({"available":false,"epoch":0,"request":null})
    );
    let absent = call(
        &mut client,
        r#"{"command":"ui.file","params":{"expected_revision":0,"action":"open"}}"#,
    )?;
    assert_eq!(absent["error"]["code"], "ui_unavailable");
    assert_eq!(changes.load(Ordering::SeqCst), 0);

    // This test simulates only the trusted host acknowledgement, never displayed pixels.
    engine.lock().map_err(|_| "engine")?.set_ui_file_host(true);
    let requested = call(
        &mut client,
        r#"{"command":"ui.file","params":{"expected_revision":0,"action":"open"}}"#,
    )?;
    let requested = &requested["result"];
    let first = requested["ui_file"]["request"]["id"]
        .as_u64()
        .ok_or("request id")?;
    assert_eq!(requested["ui_file"]["request"]["phase"], "requested");
    assert_eq!(requested["ui_file_operation"], first);
    assert_eq!(requested["changed"], false);
    assert_eq!(requested["revision"], 0);
    assert!(requested.get("file_operation").is_none());
    wait_for_changes(&changes, 1);
    let controls = call(&mut client, r#"{"command":"ui.controls","params":{}}"#)?;
    let controls = controls["result"]["controls"]
        .as_array()
        .ok_or("controls")?;
    let cancel = controls
        .iter()
        .find(|control| control["id"] == "file.cancel")
        .ok_or("cancel binding")?;
    assert_eq!(
        cancel["command"],
        serde_json::json!({"command":"ui.file","params":{
            "expected_revision":0,"action":"cancel","request_id":first
        }})
    );
    assert_eq!(cancel["enabled"], true);
    let open = controls
        .iter()
        .find(|control| control["id"] == "file.open")
        .ok_or("open binding")?;
    assert_eq!(open["enabled"], false);
    assert_eq!(open["disabled_reason"], "ui_busy");
    {
        let mut engine = engine.lock().map_err(|_| "engine")?;
        assert_eq!(
            engine
                .state()
                .ui_file
                .request
                .as_ref()
                .ok_or("still requested")?
                .phase,
            bondline_commands::UiFilePhase::Requested
        );
        assert!(matches!(
            engine.claim_ui_file(first)?,
            bondline_commands::UiFilePlan::OpenPicker
        ));
    }
    let accepted = call(&mut client, r#"{"command":"document.inspect","params":{}}"#)?;
    assert_eq!(
        accepted["result"]["ui_file"]["request"]["phase"],
        "accepted"
    );
    assert!(accepted["result"].get("ui_file_operation").is_none());
    assert!(accepted["result"].get("file_operation").is_none());
    let cancelled = call(
        &mut client,
        &serde_json::json!({"command":"ui.file","params":{
            "expected_revision":0,"action":"cancel","request_id":first
        }})
        .to_string(),
    )?;
    assert_eq!(cancelled["result"]["ui_file"]["request"], Value::Null);
    assert_eq!(cancelled["result"]["ui_file_operation"], 2);
    wait_for_changes(&changes, 2);
    let next = call(
        &mut client,
        r#"{"command":"ui.file","params":{"expected_revision":0,"action":"open"}}"#,
    )?;
    let second = next["result"]["ui_file"]["request"]["id"]
        .as_u64()
        .ok_or("replacement id")?;
    assert!(second > first);
    wait_for_changes(&changes, 3);
    let old_cancel = call(
        &mut client,
        &serde_json::json!({"command":"ui.file","params":{
            "expected_revision":0,"action":"cancel","request_id":first
        }})
        .to_string(),
    )?;
    assert_eq!(old_cancel["error"]["code"], "ui_stale_request");
    {
        let mut engine = engine.lock().map_err(|_| "engine")?;
        assert_eq!(
            engine
                .claim_ui_file(first)
                .err()
                .ok_or("old host claim")?
                .info()
                .code,
            "ui_stale_request"
        );
        engine.claim_ui_file(second)?;
    }
    let close = call(
        &mut client,
        r#"{"command":"document.close","params":{"expected_revision":0}}"#,
    )?;
    assert_eq!(close["result"]["close"], "approved");
    wait_for_changes(&changes, 4);
    assert_eq!(
        executor
            .prepare_ui_file(
                second,
                bondline_commands::Command::parse(
                    "document.open_cdxml",
                    serde_json::json!({"path":"must-not-open-after-close.cdxml"})
                )?
            )
            .err()
            .ok_or("obsolete picker survived close")?
            .code,
        "ui_stale_request"
    );
    assert!(!executor.has_active_work());
    let cancelled_close = call(
        &mut client,
        r#"{"command":"document.close","params":{"expected_revision":0,"action":"cancel"}}"#,
    )?;
    assert_eq!(cancelled_close["result"]["close"], "open");
    assert_eq!(cancelled_close["result"]["ui_file"]["request"], Value::Null);
    assert_eq!(cancelled_close["result"]["document"], requested["document"]);
    wait_for_changes(&changes, 5);
    server.shutdown()?;
    Ok(())
}

#[test]
fn prepared_native_file_work_finishes_only_its_origin_and_same_revision_path_aba_stays_stale()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("original-profile.cdxml");
    let other = directory.path("other-name.cdxml");
    std::fs::write(
        &path,
        include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml"),
    )?;
    let origin: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let unrelated: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let executor = bondline_mcp::CommandExecutor::new(Arc::clone(&origin));
    let unrelated_before =
        serde_json::to_value(unrelated.lock().map_err(|_| "unrelated engine")?.state())?;
    let request = {
        let mut engine = origin.lock().map_err(|_| "origin")?;
        engine.set_ui_file_host(true);
        let requested = engine.dispatch(
            "ui.file",
            serde_json::json!({"expected_revision":0,"action":"open"}),
        )?;
        let id = requested.ui_file.request.as_ref().ok_or("request")?.id;
        engine.claim_ui_file(id)?;
        id
    };
    let work = executor
        .prepare_ui_file(
            request,
            bondline_commands::Command::parse(
                "document.open_cdxml",
                serde_json::json!({"path":path}),
            )?,
        )
        .map_err(|error| error.message)?;
    assert!(
        executor.has_active_work(),
        "prepared native work must be registered before std::thread::spawn"
    );
    let result = std::thread::spawn(move || {
        work.complete_blocking(&bondline_commands::Cancellation::default())
    })
    .join()
    .map_err(|_| "native test worker panicked")?
    .map_err(|error| error.message)?;
    assert_eq!(
        result.file.path,
        Some(
            std::fs::canonicalize(&path)?
                .to_str()
                .ok_or("path")?
                .to_owned()
        )
    );
    assert_eq!(
        result
            .file_operation
            .as_ref()
            .ok_or("actual open receipt")?
            .operation,
        "open"
    );
    assert!(!executor.has_active_work());
    assert_eq!(
        serde_json::to_value(unrelated.lock().map_err(|_| "unrelated engine")?.state())?,
        unrelated_before
    );
    let (save_id, actual) = {
        let mut engine = origin.lock().map_err(|_| "origin")?;
        assert_eq!(engine.state().revision, 1);
        assert!(engine.finish_ui_file(request, false));
        let requested = engine.dispatch(
            "ui.file",
            serde_json::json!({"expected_revision":1,"action":"save"}),
        )?;
        let id = requested.ui_file.request.as_ref().ok_or("save request")?.id;
        let bondline_commands::UiFilePlan::SaveExisting {
            format,
            params: save,
        } = engine.claim_ui_file(id)?
        else {
            return Err("known-path Save did not bind its existing path".into());
        };
        assert_eq!(format, bondline_commands::FileFormat::Cdxml);
        (id, bondline_commands::Command::SaveCdxml(save))
    };
    {
        let mut engine = origin.lock().map_err(|_| "origin")?;
        engine.dispatch("document.save_cdxml", serde_json::json!({"path":other}))?;
        engine.dispatch(
            "document.save_cdxml",
            serde_json::json!({"path":path,"overwrite":true}),
        )?;
        assert_eq!(
            engine.state().revision,
            1,
            "Save As path changes do not advance authored revision"
        );
    }
    let before = serde_json::to_value(origin.lock().map_err(|_| "origin")?.state())?;
    assert_eq!(
        executor
            .prepare_ui_file(save_id, actual)
            .err()
            .ok_or("path ABA resurrected accepted Save")?
            .code,
        "ui_stale_request"
    );
    assert_eq!(
        serde_json::to_value(origin.lock().map_err(|_| "origin")?.state())?,
        before
    );
    assert!(!executor.has_active_work());
    file_support::independently_read_file(&path, 1)?;
    Ok(())
}
