//! Child of the existing CLI adapter host: no new client, socket or timeout framework.
use super::*;

#[test]
fn stdio_discovers_atom_presentation_but_does_not_fabricate_an_attached_native_host() -> TestResult
{
    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"native-atom-presentation","version":"1"}}}))?;
    let initialized = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 36);
    let found = tools
        .iter()
        .filter(|tool| tool["name"] == "ui.atom_label")
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["inputSchema"]["type"], "object");
    assert_eq!(found[0]["annotations"]["readOnlyHint"], false);
    client.call(
        3,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72.0,
        "begin_y_pt":72.0,"end_x_pt":120.0,"end_y_pt":72.0}),
    )?;
    let before = client.call(4, "document.inspect", json!({}))?;
    assert_eq!(before["ui_file"]["available"], false);
    client.send(
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{
        "name":"ui.atom_label","arguments":{"action":"open","expected_revision":1,
            "page_id":1,"fragment_id":1,"atom_id":2}}}),
    )?;
    let rejected = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(rejected["id"], 5);
    assert_eq!(rejected["result"]["isError"], true);
    assert_eq!(
        rejected["result"]["structuredContent"]["error"]["code"],
        "ui_unavailable"
    );
    let after = client.call(6, "document.inspect", json!({}))?;
    assert_eq!(after, before);
    assert!(after.get("ui_atom_label").is_none());
    client.finish()
}

#[test]
fn authenticated_atom_presentation_wakes_the_host_without_claiming_authored_change_or_display()
-> TestResult {
    let mut session = Engine::default();
    session.set_ui_file_host(true);
    session.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72.0,
        "begin_y_pt":72.0,"end_x_pt":120.0,"end_y_pt":72.0}),
    )?;
    let before = serde_json::to_value(session.state())?;
    let owner = session.capture_stamp();
    let engine: SharedEngine = Arc::new(Mutex::new(session));
    let (notify, noticed) = channel();
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        let _ = notify.send(());
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    // The live credential is sent only on this connection and never printed or stored.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    assert!(noticed.try_recv().is_err());
    let reply = socket(
        &mut client,
        json!({"command":"ui.atom_label","params":{
        "action":"open","expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":2}}),
    )?;
    assert!(reply.get("error").is_none(), "{reply}");
    let result = &reply["result"];
    assert_eq!(result["changed"], false);
    let id = result["ui_atom_label_operation"]
        .as_u64()
        .ok_or("request ID")?;
    assert_eq!(
        result["ui_atom_label"],
        json!({"id":id,"input_revision":1,
        "page_id":1,"fragment_id":1,"atom_id":2,"element":6,"explicit_hydrogens":3,"current":true})
    );
    assert!(result.get("file_operation").is_none());
    noticed.recv_timeout(Duration::from_secs(10))?;
    let state = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    assert!(state.get("error").is_none());
    let mut current = state["result"].clone();
    assert!(current.get("ui_atom_label_operation").is_none());
    current
        .as_object_mut()
        .ok_or("state")?
        .remove("ui_atom_label");
    assert_eq!(current, before);
    let failed = socket(
        &mut client,
        json!({"command":"ui.atom_label","params":{
        "action":"cancel","expected_revision":1,"request_id":id+1}}),
    )?;
    assert_eq!(failed["error"]["code"], "ui_stale_request");
    assert!(noticed.try_recv().is_err());
    let canceled = socket(
        &mut client,
        json!({"command":"ui.atom_label","params":{
        "action":"cancel","expected_revision":1,"request_id":id}}),
    )?;
    assert!(canceled.get("error").is_none());
    assert_eq!(canceled["result"]["ui_atom_label_operation"], id);
    assert_eq!(canceled["result"]["changed"], false);
    assert!(canceled["result"].get("ui_atom_label").is_none());
    noticed.recv_timeout(Duration::from_secs(10))?;
    assert!(noticed.try_recv().is_err());
    assert!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&owner)
    );
    assert_eq!(
        serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
        before
    );
    server.shutdown()?;
    Ok(())
}
