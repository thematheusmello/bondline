use bondline_commands::{Cancellation, CloseState, Command, Engine, RenderBackend, RenderCapture};
use bondline_mcp::{CommandExecutor, SharedEngine, UiCaptureWork};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, Mutex},
};

#[path = "../../../xtask/oracles/sdf_test_support.rs"]
#[allow(dead_code, reason = "reuse unchanged complete-source test consumers")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

fn params() -> RenderCapture {
    RenderCapture {
        page_id: 1,
        width_px: 1024,
        height_px: 1024,
        backend: RenderBackend::Cpu,
    }
}
fn empty() -> (SharedEngine, CommandExecutor) {
    let engine = Arc::new(Mutex::new(Engine::default()));
    let executor = CommandExecutor::new(Arc::clone(&engine));
    (engine, executor)
}
fn state(engine: &SharedEngine) -> TestResult<Value> {
    Ok(serde_json::to_value(
        engine.lock().map_err(|_| "engine")?.state(),
    )?)
}
fn open(path: &std::path::Path) -> TestResult<(SharedEngine, CommandExecutor)> {
    fs::write(path, SOURCE)?;
    let (engine, executor) = empty();
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("document.open_sdf", json!({"path":path}))?;
    Ok((engine, executor))
}

#[test]
fn ui_capture_reserves_before_spawning_and_drop_releases_the_close_barrier() -> TestResult {
    let (engine, executor) = empty();
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let work: UiCaptureWork = executor
        .prepare_ui_capture(&stamp, params())
        .map_err(|e| e.message)?;
    assert!(executor.has_active_work());
    assert!(executor.clone().has_active_work());
    {
        // Preparation returned without retaining the Engine mutex.
        let mut engine = engine
            .try_lock()
            .map_err(|_| "preparation held the Engine")?;
        engine.dispatch("document.close", json!({"expected_revision":0}))?;
        assert_eq!(engine.state().close, CloseState::Approved);
        let closed_if_idle = if executor.has_active_work() {
            false
        } else {
            engine.complete_close()
        };
        assert!(!closed_if_idle);
    }
    drop(work);
    assert!(!executor.has_active_work());
    assert!(engine.lock().map_err(|_| "engine")?.complete_close());
    let (engine, executor) = empty();
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let before = state(&engine)?;
    let work = executor
        .prepare_ui_capture(&stamp, params())
        .map_err(|e| e.message)?;
    let result = std::thread::spawn(move || {
        work.complete_blocking(&Cancellation::default())
            .map_err(|e| e.message)
    })
    .join()
    .map_err(|_| "capture worker panicked")??;
    assert!(result.capture.is_some());
    assert!(!executor.has_active_work());
    assert_eq!(state(&engine)?, before);
    Ok(())
}

#[test]
fn ui_capture_shares_two_slots_with_registry_work_and_errors_release_capacity() -> TestResult {
    let (engine, executor) = empty();
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let before = state(&engine)?;
    let first = executor
        .prepare_ui_capture(&stamp, params())
        .map_err(|e| e.message)?;
    let second = executor
        .clone()
        .prepare_ui_capture(&stamp, params())
        .map_err(|e| e.message)?;
    assert_eq!(
        executor
            .prepare_ui_capture(&stamp, params())
            .err()
            .ok_or("third capture")?
            .code,
        "busy"
    );
    assert_eq!(
        executor
            .execute_blocking(Command::Capture(params()), &Cancellation::default())
            .err()
            .ok_or("registry escaped shared worker budget")?
            .code,
        "busy"
    );
    assert_eq!(state(&engine)?, before);
    drop(first);
    let mut invalid = params();
    invalid.width_px = 0;
    assert_eq!(
        executor
            .prepare_ui_capture(&stamp, invalid)
            .err()
            .ok_or("invalid capture")?
            .code,
        "invalid_params"
    );
    let replacement = executor
        .prepare_ui_capture(&stamp, params())
        .map_err(|e| e.message)?;
    drop(second);
    drop(replacement);
    assert!(!executor.has_active_work());
    // Valid work that fails during actual rendering must also release its permit.
    let mut tiny = params();
    tiny.width_px = 8;
    tiny.height_px = 8;
    engine.lock().map_err(|_| "engine")?.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":96,"end_y_pt":72}),
    )?;
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let before = state(&engine)?;
    let work = executor
        .prepare_ui_capture(&stamp, tiny)
        .map_err(|e| e.message)?;
    assert_eq!(
        work.complete_blocking(&Cancellation::default())
            .err()
            .ok_or("tiny chemical labels")?
            .code,
        "unsupported_scale"
    );
    assert!(!executor.has_active_work());
    assert_eq!(state(&engine)?, before);
    Ok(())
}

#[test]
fn ui_capture_rejects_foreign_stale_and_nonopen_contexts_without_reserving_work() -> TestResult {
    let (engine, executor) = empty();
    let (foreign, foreign_executor) = empty();
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    assert_eq!(
        foreign_executor
            .prepare_ui_capture(&stamp, params())
            .err()
            .ok_or("foreign source accepted")?
            .code,
        "stale_revision"
    );
    assert!(!foreign_executor.has_active_work());
    assert_eq!(state(&foreign)?["revision"], 0);
    engine.lock().map_err(|_| "engine")?.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":96,"end_y_pt":72}),
    )?;
    assert_eq!(
        executor
            .prepare_ui_capture(&stamp, params())
            .err()
            .ok_or("stale source accepted")?
            .code,
        "stale_revision"
    );
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("history.undo", json!({}))?;
    assert_eq!(
        executor
            .prepare_ui_capture(&stamp, params())
            .err()
            .ok_or("A-to-B-to-A revived a stale capture")?
            .code,
        "stale_revision"
    );
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("history.redo", json!({}))?;
    let current = engine.lock().map_err(|_| "engine")?.capture_stamp();
    for (action, close) in [
        ("request", CloseState::NeedsDecision),
        ("discard", CloseState::Approved),
    ] {
        engine.lock().map_err(|_| "engine")?.dispatch(
            "document.close",
            json!({"expected_revision":3,"action":action}),
        )?;
        assert_eq!(engine.lock().map_err(|_| "engine")?.state().close, close);
        let before = state(&engine)?;
        assert_eq!(
            executor
                .prepare_ui_capture(&current, params())
                .err()
                .ok_or("nonOpen capture")?
                .code,
            "ui_stale_request"
        );
        assert!(!executor.has_active_work());
        assert_eq!(state(&engine)?, before);
    }
    assert!(engine.lock().map_err(|_| "engine")?.complete_close());
    assert_eq!(
        executor
            .prepare_ui_capture(&current, params())
            .err()
            .ok_or("committed capture")?
            .code,
        "close_committed"
    );
    assert!(!executor.has_active_work());
    Ok(())
}

#[test]
fn cancelled_ui_capture_has_no_result_state_change_or_leaked_permit() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("source.sdf");
    let (engine, executor) = open(&input)?;
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let before = state(&engine)?;
    let work = executor
        .prepare_ui_capture(&stamp, params())
        .map_err(|e| e.message)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        work.complete_blocking(&cancel)
            .err()
            .ok_or("cancelled capture published")?
            .code,
        "cancelled"
    );
    assert!(!executor.has_active_work());
    assert_eq!(state(&engine)?, before);
    assert_eq!(fs::read(&input)?, SOURCE);
    Ok(())
}

#[test]
fn ui_capture_keeps_whole_historical_source_without_save_lease_file_binding_or_ack() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("original.sdf");
    let output = directory.path("saved.sdf");
    let next = directory.path("next.smi");
    fs::write(&next, b"O next\n")?;
    let (engine, executor) = open(&input)?;
    let expected = executor
        .execute_blocking(Command::Capture(params()), &Cancellation::default())
        .map_err(|e| e.message)?
        .capture
        .ok_or("reference capture")?
        .png_base64;
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let work = executor
        .prepare_ui_capture(&stamp, params())
        .map_err(|e| e.message)?;
    let save = engine
        .lock()
        .map_err(|_| "engine")?
        .prepare(Command::parse("document.save_sdf", json!({"path":output}))?)?;
    engine.lock().map_err(|_| "engine")?.dispatch(
        "sdf.field.set_value",
        json!({"expected_revision":1,"record_id":1,"field_index":5,"value_lines":["changed"]}),
    )?;
    let before = state(&engine)?;
    let result = work
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert_eq!(result.revision, 1);
    assert!(
        result.capture.ok_or("capture")?.png_base64 == expected,
        "historical PNG changed after a field-only edit"
    );
    assert!(!result.changed);
    assert!(result.document.is_none());
    assert!(result.file_operation.is_none());
    assert!(result.ui_file_operation.is_none());
    assert_eq!(state(&engine)?, before);
    assert!(!executor.has_active_work());
    assert_eq!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .prepare(Command::parse("document.save_sdf", json!({"path":output}))?)
            .err()
            .ok_or("capture released someone else's Save lease")?
            .info()
            .code,
        "busy"
    );
    let completed_save = save.complete()?;
    engine
        .lock()
        .map_err(|_| "engine")?
        .finish(completed_save)?;
    support::independently_read_sdf(&output, SOURCE, true)?;
    assert_eq!(fs::read(&input)?, SOURCE);
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let work = executor
        .prepare_ui_capture(&stamp, params())
        .map_err(|e| e.message)?;
    fs::remove_file(&input)?;
    engine.lock().map_err(|_| "engine")?.dispatch(
        "document.open_smiles",
        json!({"path":next,"discard_unsaved":true}),
    )?;
    let revision = engine.lock().map_err(|_| "engine")?.state().revision;
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("document.close", json!({"expected_revision":revision}))?;
    let before = state(&engine)?;
    let result = work
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert_eq!(result.revision, 2);
    assert!(
        result.capture.ok_or("historical capture")?.png_base64 == expected,
        "historical PNG changed after its source session was replaced"
    );
    assert!(result.collection.is_some());
    assert!(result.smiles_file.is_none());
    assert_eq!(result.close, CloseState::Approved);
    assert_eq!(state(&engine)?, before);
    assert!(!executor.has_active_work());
    engine.lock().map_err(|_| "engine")?.dispatch(
        "document.close",
        json!({"expected_revision":revision,"action":"cancel"}),
    )?;
    let stamp = engine.lock().map_err(|_| "engine")?.capture_stamp();
    let before = state(&engine)?;
    assert_eq!(
        executor
            .prepare_ui_capture(&stamp, params())
            .err()
            .ok_or("SMILES file became a drawing capture")?
            .code,
        "unsupported_content"
    );
    assert!(!executor.has_active_work());
    assert_eq!(state(&engine)?, before);
    // A capture is not one of the accepted native File commands and cannot grant
    // trusted completion acknowledgement, even after a successful PNG reply.
    let (fresh, fresh_executor) = empty();
    let mut guard = fresh.lock().map_err(|_| "engine")?;
    guard.set_ui_file_host(true);
    let id = guard
        .dispatch("ui.file", json!({"expected_revision":0,"action":"save_as"}))?
        .ui_file
        .request
        .ok_or("request")?
        .id;
    guard.claim_ui_file(id)?;
    let before = serde_json::to_value(guard.state())?;
    let stamp = guard.capture_stamp();
    drop(guard);
    let work = fresh_executor
        .prepare_ui_capture(&stamp, params())
        .map_err(|e| e.message)?;
    assert!(
        work.complete_blocking(&Cancellation::default())
            .map_err(|e| e.message)?
            .capture
            .is_some()
    );
    let mut guard = fresh.lock().map_err(|_| "engine")?;
    assert_eq!(
        serde_json::to_value(guard.state())?,
        before,
        "capture acknowledged or changed the accepted File intent"
    );
    assert_eq!(
        guard
            .prepare_ui_file(id, Command::Capture(params()))
            .err()
            .ok_or("capture bound File intent")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(serde_json::to_value(guard.state())?, before);
    assert!(!guard.finish_ui_file(id, false));
    drop(guard);
    assert!(!fresh_executor.has_active_work());
    Ok(())
}
