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

#[test]
fn absolute_order_edits_notify_once_and_preserve_mode_noops_and_stale_failures() -> TestResult {
    let mut initial = Engine::default();
    initial.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":148,"end_y_pt":200}),
    )?;
    initial.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
    let engine: SharedEngine = Arc::new(Mutex::new(initial));
    let changes = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
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
    let set = |revision, order| json!({"command":"bond.set_order","params":{"expected_revision":revision,"page_id":1,"fragment_id":1,"bond_id":1,"order":order,"hydrogen_policy":"rectify_neutral_carbon"}});
    let changed = request(&mut client, set(1, "double"))?;
    assert_eq!(changed["result"]["changed"], true);
    assert_eq!(changed["result"]["revision"], 2);
    assert_eq!(changed["result"]["undo_depth"], 2);
    let graph = &changed["result"]["document"]["pages"][0]["fragments"][0];
    assert_eq!(
        graph["bonds"],
        json!([{"id":1,"begin":1,"end":2,"order":"double"}])
    );
    assert_eq!(graph["atoms"][0]["explicit_hydrogens"], 2);
    assert_eq!(graph["atoms"][1]["explicit_hydrogens"], 2);
    let before = request(&mut client, inspect.clone())?;
    assert_eq!(changes.load(Ordering::SeqCst), 1);
    let noop = request(&mut client, set(2, "double"))?;
    assert_eq!(noop["result"]["changed"], false);
    assert_eq!(request(&mut client, inspect.clone())?, before);
    assert_eq!(changes.load(Ordering::SeqCst), 1);
    assert_eq!(
        request(&mut client, set(1, "triple"))?["error"]["code"],
        "stale_revision"
    );
    assert_eq!(request(&mut client, inspect.clone())?, before);
    assert_eq!(changes.load(Ordering::SeqCst), 1);
    let controls = request(&mut client, json!({"command":"ui.controls","params":{}}))?;
    assert_eq!(
        controls["result"]["controls"]
            .as_array()
            .ok_or("controls")?
            .iter()
            .find(|row| row["id"] == "tool.bond.triple")
            .ok_or("Triple")?["selected"],
        true
    );
    let undo = request(&mut client, json!({"command":"history.undo","params":{}}))?;
    assert_eq!(
        undo["result"]["document"]["pages"][0]["fragments"][0]["bonds"][0]["order"],
        "single"
    );
    request(&mut client, inspect.clone())?;
    assert_eq!(changes.load(Ordering::SeqCst), 2);
    let redo = request(&mut client, json!({"command":"history.redo","params":{}}))?;
    assert_eq!(redo["result"]["document"], changed["result"]["document"]);
    request(&mut client, inspect)?;
    assert_eq!(changes.load(Ordering::SeqCst), 3);
    server.shutdown()?;
    Ok(())
}
