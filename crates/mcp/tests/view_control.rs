use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn connect(address: SocketAddr) -> TestResult<BufReader<TcpStream>> {
    let stream = TcpStream::connect_timeout(&address, Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    Ok(BufReader::new(stream))
}
fn call(client: &mut BufReader<TcpStream>, request: Value) -> TestResult<Value> {
    serde_json::to_writer(client.get_mut(), &request)?;
    client.get_mut().write_all(b"\n")?;
    client.get_mut().flush()?;
    let mut line = String::new();
    assert!(
        client.read_line(&mut line)? > 0,
        "response before connection EOF"
    );
    Ok(serde_json::from_str(&line)?)
}
fn authenticated(server: &LoopbackServer) -> TestResult<BufReader<TcpStream>> {
    let mut client = connect(server.address())?;
    assert_eq!(
        call(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    Ok(client)
}
fn query(client: &mut BufReader<TcpStream>, params: Value) -> TestResult<Value> {
    call(client, json!({"command":"ui.view","params":params}))
}
fn wait_for_count(count: &AtomicUsize, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && count.load(Ordering::SeqCst) < expected {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        count.load(Ordering::SeqCst),
        expected,
        "changed view must wake the native host"
    );
}

#[test]
fn authenticated_view_is_shared_but_only_real_changes_notify_and_no_authoring_state_changes()
-> TestResult {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let initial = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let count = Arc::new(AtomicUsize::new(0));
    let on_change = Arc::clone(&count);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        on_change.fetch_add(1, Ordering::SeqCst);
    })?;
    let mut unauthenticated = connect(server.address())?;
    let error = call(
        &mut unauthenticated,
        json!({"command":"ui.view","params":{"action":"actual"}}),
    )?;
    assert_eq!(error["error"]["code"], "unauthorized");
    let mut first = authenticated(&server)?;
    let mut second = authenticated(&server)?;
    for (params, percent, center, epoch, changed, notifications) in [
        (json!({"action":"inspect"}), None, None, 0, false, 0),
        (json!({"action":"actual"}), Some(100.0), None, 1, true, 1),
        (json!({"action":"actual"}), Some(100.0), None, 1, false, 1),
        (
            json!({"action":"pan","center_x_pt":72,"center_y_pt":144}),
            Some(100.0),
            Some([72.0, 144.0]),
            2,
            true,
            2,
        ),
    ] {
        let response = query(&mut first, params)?;
        assert!(response.get("error").is_none(), "{response}");
        let expected = json!({"zoom_percent":percent,"center_x_pt":center.map(|p|p[0]),"center_y_pt":center.map(|p|p[1]),"epoch":epoch,"changed":changed});
        assert_eq!(response["result"]["ui_view_operation"], expected);
        assert_eq!(response["result"]["changed"], false);
        // A second request on the same ordered connection retires the preceding callback.
        let normal = call(
            &mut first,
            json!({"command":"document.inspect","params":{}}),
        )?;
        assert_eq!(normal["result"], initial);
        wait_for_count(&count, notifications);
        let shared = query(&mut second, json!({"action":"inspect"}))?;
        let mut inspected = expected;
        inspected["changed"] = json!(false);
        assert_eq!(shared["result"]["ui_view_operation"], inspected);
        assert!(
            engine
                .lock()
                .map_err(|_| "engine")?
                .capture_stamp_matches(&stamp)
        );
    }
    let invalid = query(&mut first, json!({"action":"set_zoom","percent":0}))?;
    assert_eq!(invalid["error"]["code"], "invalid_params");
    let normal = call(
        &mut first,
        json!({"command":"document.inspect","params":{}}),
    )?;
    assert_eq!(normal["result"], initial);
    assert_eq!(count.load(Ordering::SeqCst), 2);
    server.shutdown()?;
    Ok(())
}

#[path = "view_control/sdf.rs"]
mod sdf;
