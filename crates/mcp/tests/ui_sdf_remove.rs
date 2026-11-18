use bondline_commands::{Cancellation, Command, Engine, SdfRecordRemove};
use bondline_mcp::{CommandExecutor, SharedEngine, UiSdfRemoveContext};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, Mutex},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the original isolated filesystem helper")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

fn open(path: &std::path::Path) -> TestResult<(SharedEngine, CommandExecutor)> {
    fs::write(path, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    let shared = Arc::new(Mutex::new(engine));
    Ok((Arc::clone(&shared), CommandExecutor::new(shared)))
}

fn remove() -> SdfRecordRemove {
    SdfRecordRemove {
        expected_revision: 1,
        record_id: 1,
    }
}

fn state(shared: &SharedEngine) -> TestResult<Value> {
    Ok(serde_json::to_value(
        shared.lock().map_err(|_| "engine")?.state(),
    )?)
}

#[test]
fn native_record_removal_reserves_before_spawn_and_commits_the_exact_remaining_source() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("native-record-remove.sdf");
    let (shared, executor) = open(&path)?;
    let before = state(&shared)?;
    let context = {
        let engine = shared.lock().map_err(|_| "engine")?;
        UiSdfRemoveContext::capture(&engine)
    };
    let work = executor
        .prepare_ui_sdf_remove(&context, remove())
        .map_err(|e| e.message)?;
    assert!(executor.has_active_work());
    assert!(shared.try_lock().is_ok());
    assert_eq!(state(&shared)?, before);
    let worker = std::thread::spawn(move || work.complete_blocking(&Cancellation::default()));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !worker.is_finished() {
        if std::time::Instant::now() >= deadline {
            return Err("native removal worker did not retire within five seconds".into());
        }
        std::thread::yield_now();
    }
    let reply = worker
        .join()
        .map_err(|_| "native removal worker panicked")?
        .map_err(|e| e.message)?;
    assert!(reply.changed);
    assert_eq!(reply.revision, 2);
    assert!(reply.document.is_none());
    assert_eq!(reply.collection.ok_or("collection")?.record_count, 1);
    assert_eq!(reply.undo_depth, 1);
    assert!(reply.file.dirty);
    assert!(!executor.has_active_work());
    let engine = shared.lock().map_err(|_| "engine")?;
    assert_eq!(
        engine.content().sdf()?.as_bytes(),
        SOURCE.get(462..).ok_or("literal suffix")?
    );
    assert_eq!(fs::read(path)?, SOURCE, "removal is not file publication");
    Ok(())
}

#[test]
fn native_removal_shares_capacity_and_rejects_cancelled_or_changed_close_context() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let (shared, executor) = open(&directory.path("bounded-remove.sdf"))?;
    let before = state(&shared)?;
    let (stamp, epoch, context) = {
        let engine = shared.lock().map_err(|_| "engine")?;
        (
            engine.capture_stamp(),
            engine.state().close_epoch,
            UiSdfRemoveContext::capture(&engine),
        )
    };
    let query = Command::parse(
        "sdf.record.inspect",
        json!({"expected_revision":1,"record_id":2}),
    )?;
    let query = executor
        .prepare_ui_sdf(&stamp, epoch, query)
        .map_err(|e| e.message)?;
    let work = executor
        .prepare_ui_sdf_remove(&context, remove())
        .map_err(|e| e.message)?;
    let full = executor
        .prepare_ui_sdf_remove(&context, remove())
        .err()
        .ok_or("unbounded native work")?;
    assert_eq!(full.code, "busy");
    drop(query);
    let cancel = Cancellation::default();
    cancel.cancel();
    let refused = work
        .complete_blocking(&cancel)
        .err()
        .ok_or("cancelled removal committed")?;
    assert_eq!(refused.code, "cancelled");
    assert!(!executor.has_active_work());
    assert_eq!(state(&shared)?, before);
    {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        engine.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":"request"}),
        )?;
        engine.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":"cancel"}),
        )?;
    }
    let current = state(&shared)?;
    let refused = executor
        .prepare_ui_sdf_remove(&context, remove())
        .err()
        .ok_or("old close context reused")?;
    assert_eq!(refused.code, "ui_stale_request");
    assert_eq!(state(&shared)?, current);
    assert!(!executor.has_active_work());
    assert_eq!(fs::read(directory.path("bounded-remove.sdf"))?, SOURCE);
    Ok(())
}

#[test]
fn removal_preparation_binds_file_request_epoch_and_association_without_weakening_queries()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("file-context.sdf");
    let (shared, executor) = open(&input)?;
    let (stamp, epoch, context) = {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        engine.set_ui_file_host(true);
        (
            engine.capture_stamp(),
            engine.state().close_epoch,
            UiSdfRemoveContext::capture(&engine),
        )
    };
    let generic = executor.prepare_ui_sdf(&stamp, epoch, Command::SdfRecordRemove(remove()));
    assert_eq!(
        generic.err().ok_or("unbound native removal")?.code,
        "invalid_params"
    );
    let request = {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        engine.dispatch("ui.file", json!({"expected_revision":1,"action":"save"}))?;
        engine.state().ui_file.request.ok_or("request")?.id
    };
    for cancel in [false, true] {
        if cancel {
            shared.lock().map_err(|_| "engine")?.dispatch(
                "ui.file",
                json!({"expected_revision":1,"action":"cancel","request_id":request}),
            )?;
        }
        let before = state(&shared)?;
        let error = executor
            .prepare_ui_sdf_remove(&context, remove())
            .err()
            .ok_or("stale file context accepted")?;
        assert_eq!(error.code, "ui_stale_request");
        assert_eq!(state(&shared)?, before);
        assert!(!executor.has_active_work());
    }
    let fresh = {
        let engine = shared.lock().map_err(|_| "engine")?;
        UiSdfRemoveContext::capture(&engine)
    };
    let other = directory.path("associated-化学.sdf");
    shared
        .lock()
        .map_err(|_| "engine")?
        .dispatch("document.save_sdf", json!({"path":other}))?;
    let before = state(&shared)?;
    let error = executor
        .prepare_ui_sdf_remove(&fresh, remove())
        .err()
        .ok_or("changed association accepted")?;
    assert_eq!(error.code, "ui_stale_request");
    assert_eq!(state(&shared)?, before);
    assert!(!executor.has_active_work());
    let query = executor
        .prepare_ui_sdf(
            &stamp,
            epoch,
            Command::parse(
                "sdf.record.inspect",
                json!({"expected_revision":1,"record_id":2}),
            )?,
        )
        .map_err(|e| e.message)?;
    assert!(
        query
            .complete_blocking(&Cancellation::default())
            .map_err(|e| e.message)?
            .sdf_record
            .is_some()
    );
    assert_eq!(state(&shared)?, before);
    assert_eq!(fs::read(input)?, SOURCE);
    assert_eq!(fs::read(other)?, SOURCE);
    Ok(())
}
