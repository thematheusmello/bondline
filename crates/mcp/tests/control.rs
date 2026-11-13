use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::Value;
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
    let mut first = connect(server.address())?;
    let mut second = connect(server.address())?;
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
    assert_eq!(changes.load(Ordering::SeqCst), 1);
    call(&mut second, r#"{"command":"history.undo","params":{}}"#)?;
    assert_eq!(
        engine
            .lock()
            .map_err(|_| "poisoned engine")?
            .state()
            .revision,
        2
    );
    assert_eq!(changes.load(Ordering::SeqCst), 2);
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
        let mut client = connect(server.address())?;
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
