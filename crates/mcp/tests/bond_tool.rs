use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn request(client: &mut BufReader<TcpStream>, value: Value) -> TestResult<Value> {
    serde_json::to_writer(client.get_mut(), &value)?;
    client.get_mut().write_all(b"\n")?;
    client.get_mut().flush()?;
    let mut line = String::new();
    assert_ne!(client.read_line(&mut line)?, 0);
    Ok(serde_json::from_str(&line)?)
}

#[test]
fn tool_changes_notify_after_publication_without_authored_changes_or_noop_repaints() -> TestResult {
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        count.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        request(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let inspect = json!({"command":"document.inspect","params":{}});
    let before = request(&mut client, inspect.clone())?;
    for (order, epoch, changed, notifications) in [
        ("double", 1, true, 1),
        ("double", 1, false, 1),
        ("triple", 2, true, 2),
    ] {
        let response = request(
            &mut client,
            json!({"command":"ui.bond_tool","params":{"order":order}}),
        )?;
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(response["result"]["changed"], false);
        assert_eq!(
            response["result"]["ui_bond_tool_operation"],
            json!({"order":order,"epoch":epoch,"changed":changed})
        );
        // The next request is processed after the preceding publication callback.
        assert_eq!(request(&mut client, inspect.clone())?, before);
        assert_eq!(
            changes.load(Ordering::SeqCst),
            notifications,
            "tool selection did not notify exactly once"
        );
    }
    server.shutdown()?;
    Ok(())
}
