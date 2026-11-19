#![cfg(not(target_arch = "wasm32"))]

use bondline_commands::Engine;
use bondline_mcp::{CommandExecutor, control::LoopbackServer};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated actual-file directory")]
mod files;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const BEFORE: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/multi_before.cdx");
const AFTER: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/multi_after.cdx");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/cdx-caption-edit/cases.json");

fn exchange(client: &mut BufReader<TcpStream>, packet: Value) -> TestResult<Value> {
    serde_json::to_writer(client.get_mut(), &packet)?;
    client.get_mut().write_all(b"\n")?;
    client.get_mut().flush()?;
    let mut line = String::new();
    assert!(
        client.read_line(&mut line)? > 0,
        "response before connection EOF"
    );
    Ok(serde_json::from_str(&line)?)
}

fn call(client: &mut BufReader<TcpStream>, command: &str, params: Value) -> TestResult<Value> {
    exchange(client, json!({"command":command,"params":params}))
}

#[test]
fn attached_caption_operations_publish_repaints_then_apply_and_save_exact_source() -> TestResult {
    let directory = files::FileTestDirectory::new()?;
    let input = directory.path("caption-control-原-λ.cdx");
    let output = directory.path("caption-control-保存-β.cdx");
    fs::write(&input, BEFORE)?;
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let case = &witnesses["cases"]["multi_shift"];
    let params = &case["params"];
    let mut session = Engine::default();
    session.set_ui_file_host(true);
    session.dispatch("document.open_cdx", json!({"path":input}))?;
    let engine = Arc::new(Mutex::new(session));
    let executor = CommandExecutor::new(Arc::clone(&engine));
    let count = Arc::new(AtomicUsize::new(0));
    let active_at_callback = Arc::new(AtomicBool::new(false));
    let count_at_callback = Arc::clone(&count);
    let active = Arc::clone(&active_at_callback);
    let observed = executor.clone();
    let server = LoopbackServer::bind_with_executor(0, executor, move || {
        active.fetch_or(observed.has_active_work(), Ordering::SeqCst);
        count_at_callback.fetch_add(1, Ordering::SeqCst);
    })?;
    let timeout = Duration::from_secs(5);
    let stream = TcpStream::connect_timeout(&server.address(), timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        exchange(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );

    let inspect = |client: &mut BufReader<TcpStream>, notifications| {
        let state = call(client, "document.inspect", json!({}))?;
        // The connection completes the prior publication callback before reading this request.
        assert_eq!(count.load(Ordering::SeqCst), notifications);
        assert!(!active_at_callback.load(Ordering::SeqCst));
        Ok::<_, Box<dyn std::error::Error>>(state)
    };
    let baseline = inspect(&mut client, 0)?;
    assert_eq!(baseline["result"]["document"], case["before_model"]);
    let revision = baseline["result"]["revision"]
        .as_u64()
        .ok_or("source revision")?;
    let unchanged = |reply: &Value| {
        assert!(reply.get("error").is_none(), "{reply}");
        for field in [
            "changed",
            "revision",
            "document",
            "source",
            "file",
            "undo_depth",
            "redo_depth",
        ] {
            assert_eq!(reply["result"][field], baseline["result"][field], "{field}");
        }
    };
    let selected = call(&mut client, "ui.caption", json!({"action":"select"}))?;
    unchanged(&selected);
    assert!(
        selected["result"]["ui_caption_operation"]
            .as_u64()
            .is_some()
    );
    assert_eq!(inspect(&mut client, 1)?, baseline);

    let open = json!({"action":"open","expected_revision":revision,
                      "page_id":params["page_id"],"text_id":params["text_id"],
                      "run_index":params["run_index"]});
    let opened = call(&mut client, "ui.caption", open.clone())?;
    unchanged(&opened);
    let id = opened["result"]["ui_caption_operation"]
        .as_u64()
        .ok_or("caption request")?;
    assert_eq!(
        opened["result"]["ui_caption"],
        json!({"id":id,"input_revision":revision,"page_id":1,"text_id":2,
               "run_index":0,"run_count":3,"text":"AV","current":true})
    );
    let presented = inspect(&mut client, 2)?;
    assert_eq!(
        presented["result"]["ui_caption"],
        opened["result"]["ui_caption"]
    );
    let canceled = call(
        &mut client,
        "ui.caption",
        json!({"action":"cancel","expected_revision":revision,"request_id":id}),
    )?;
    unchanged(&canceled);
    assert_eq!(canceled["result"]["ui_caption_operation"], id);
    assert!(canceled["result"]["ui_caption"].is_null());
    assert_eq!(inspect(&mut client, 3)?, baseline);

    let apply = |id| {
        json!({"action":"apply","expected_revision":revision,
                           "request_id":id,"text":params["text"]})
    };
    let stale = call(&mut client, "ui.caption", apply(id))?;
    assert_eq!(stale["error"]["code"], "ui_stale_request");
    assert_eq!(inspect(&mut client, 3)?, baseline);
    assert_eq!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .export_cdx()?,
        BEFORE
    );
    let reopened = call(&mut client, "ui.caption", open)?;
    unchanged(&reopened);
    let next = reopened["result"]["ui_caption_operation"]
        .as_u64()
        .ok_or("new caption request")?;
    assert!(next > id);
    inspect(&mut client, 4)?;
    let applied = call(&mut client, "ui.caption", apply(next))?;
    assert_eq!(applied["result"]["ui_caption_operation"], next);
    assert!(applied["result"]["ui_caption"].is_null());
    assert_eq!(applied["result"]["changed"], true);
    assert_eq!(applied["result"]["revision"], revision + 1);
    assert_eq!(applied["result"]["undo_depth"], 1);
    assert_eq!(applied["result"]["file"]["dirty"], true);
    assert_eq!(applied["result"]["document"], case["after_model"]);
    assert_eq!(
        inspect(&mut client, 5)?["result"]["document"],
        case["after_model"]
    );

    let saved = call(&mut client, "document.save_cdx", json!({"path":output}))?;
    assert_eq!(saved["result"]["file"]["dirty"], false);
    assert_eq!(
        saved["result"]["source"],
        json!({"format":"cdx","byte_length":BEFORE.len(),"modified":true})
    );
    inspect(&mut client, 6)?;
    assert_eq!(fs::read(&input)?, BEFORE);
    assert_eq!(fs::read(&output)?, AFTER);
    drop(client);
    server.shutdown()?;
    Ok(())
}
