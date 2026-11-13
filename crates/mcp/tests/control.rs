use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::Value;
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;
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

fn wait_for_changes(changes: &AtomicUsize, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while changes.load(Ordering::SeqCst) < expected && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(changes.load(Ordering::SeqCst), expected);
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
        serde_json::json!({"path":std::fs::canonicalize(&path)?,"dirty":false})
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
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
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
