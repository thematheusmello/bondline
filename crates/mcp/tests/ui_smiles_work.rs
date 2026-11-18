use bondline_commands::{
    Cancellation, CaptureStamp, Command, Engine, RenderBackend, RenderCapture, SmilesRecordInspect,
};
use bondline_mcp::{CommandExecutor, SharedEngine};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, Mutex},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse the original bounded temporary-directory helper"
)]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = b"CC first\r\n\n \xffignored\n[13CH3][OH] \0\xffPRIVATE\nO last";

fn open(
    directory: &support::FileTestDirectory,
    name: &str,
) -> TestResult<(SharedEngine, CommandExecutor)> {
    let path = directory.path(name);
    fs::write(&path, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":path}))?;
    let shared = Arc::new(Mutex::new(engine));
    Ok((Arc::clone(&shared), CommandExecutor::new(shared)))
}
fn context(engine: &SharedEngine) -> TestResult<(CaptureStamp, u64)> {
    let engine = engine.lock().map_err(|_| "engine")?;
    Ok((engine.capture_stamp(), engine.state().close_epoch))
}
fn state(engine: &SharedEngine) -> TestResult<Value> {
    Ok(serde_json::to_value(
        engine.lock().map_err(|_| "engine")?.state(),
    )?)
}
fn query(revision: u64, record: u32) -> SmilesRecordInspect {
    SmilesRecordInspect {
        expected_revision: revision,
        record_id: record,
    }
}
fn close(engine: &SharedEngine, action: &str) -> TestResult {
    let mut engine = engine.lock().map_err(|_| "engine")?;
    let revision = engine.state().revision;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":revision,"action":action}),
    )?;
    Ok(())
}
fn refused(
    executor: &CommandExecutor,
    engine: &SharedEngine,
    stamp: &CaptureStamp,
    epoch: u64,
    params: SmilesRecordInspect,
    code: &str,
) -> TestResult {
    let before = state(engine)?;
    assert_eq!(
        executor
            .prepare_ui_smiles(stamp, epoch, params)
            .err()
            .ok_or("UI SMILES work unexpectedly accepted")?
            .code,
        code
    );
    assert_eq!(state(engine)?, before);
    Ok(())
}
fn expected() -> Value {
    json!({"input_revision":1,"session_generation":1,"record_id":2,"line_number":4,
        "full_range":{"start":21,"end":43},"notation_range":{"start":21,"end":32},
        "separator_range":{"start":32,"end":33},"application_data_range":{"start":33,"end":42},
        "line_ending_range":{"start":42,"end":43},"atom_count":2,"bond_count":1,"notation":"[13CH3][OH]"})
}

#[test]
fn native_smiles_query_registers_before_spawn_and_keeps_exact_selected_historical_source()
-> TestResult {
    assert_eq!(SOURCE.len(), 49);
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("records-化学.smiles");
    let (engine, executor) = open(&directory, "records-化学.smiles")?;
    let before = state(&engine)?;
    let original = engine.lock().map_err(|_| "engine")?.content().clone();
    let (stamp, epoch) = context(&engine)?;
    let work = executor
        .prepare_ui_smiles(&stamp, epoch, query(1, 2))
        .map_err(|e| e.message)?;
    assert!(executor.has_active_work() && executor.clone().has_active_work());
    assert!(
        engine.try_lock().is_ok(),
        "prepare retained the origin mutex"
    );
    assert_eq!(state(&engine)?, before);
    let result = std::thread::spawn(move || work.complete_blocking(&Cancellation::default()))
        .join()
        .map_err(|_| "SMILES worker panicked")?
        .map_err(|e| e.message)?;
    assert_eq!(
        serde_json::to_value(result.smiles_record.as_ref().ok_or("explicit record")?)?,
        expected()
    );
    assert!(!format!("{result:?}").contains("[13CH3][OH]"));
    assert!(!format!("{result:?}").contains("PRIVATE"));
    let mut wire = serde_json::to_value(&result)?;
    wire.as_object_mut()
        .ok_or("result object")?
        .remove("smiles_record")
        .ok_or("selected record")?;
    assert_eq!(wire, before);
    assert_eq!(state(&engine)?, before);
    assert_eq!(engine.lock().map_err(|_| "engine")?.content(), &original);
    assert!(!executor.has_active_work());
    assert_eq!(fs::read(&input)?, SOURCE);
    // Accepted work owns all source; neither pathname deletion nor another Open can substitute it.
    let (stamp, epoch) = context(&engine)?;
    let historical = executor
        .prepare_ui_smiles(&stamp, epoch, query(1, 2))
        .map_err(|e| e.message)?;
    fs::remove_file(input)?;
    let replacement = directory.path("replacement.smi");
    fs::write(&replacement, b"O new source\n")?;
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("document.open_smiles", json!({"path":replacement}))?;
    close(&engine, "request")?;
    let current = state(&engine)?;
    assert_eq!(current["revision"], 2);
    assert_eq!(current["close"], "approved");
    let result = std::thread::spawn(move || historical.complete_blocking(&Cancellation::default()))
        .join()
        .map_err(|_| "historical worker panicked")?
        .map_err(|e| e.message)?;
    assert_eq!(
        serde_json::to_value(result.smiles_record.ok_or("historical record")?)?,
        expected()
    );
    assert_eq!(result.revision, 1);
    assert_eq!(
        result.close_epoch,
        current["close_epoch"].as_u64().ok_or("close epoch")?
    );
    assert_eq!(state(&engine)?, current);
    assert_eq!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .smiles_file()?
            .as_bytes(),
        b"O new source\n"
    );
    assert_eq!(fs::read(replacement)?, b"O new source\n");
    assert!(!executor.has_active_work());
    Ok(())
}

#[test]
fn native_smiles_context_guards_refuse_before_work_and_never_leak_reservations() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let (engine, executor) = open(&directory, "guarded.smi")?;
    let (foreign, _) = open(&directory, "foreign.smi")?;
    let (foreign_stamp, _) = context(&foreign)?;
    let (stamp, epoch) = context(&engine)?;
    assert_eq!(
        engine.lock().map_err(|_| "engine")?.content(),
        foreign.lock().map_err(|_| "foreign")?.content()
    );
    refused(
        &executor,
        &engine,
        &foreign_stamp,
        epoch,
        query(1, 2),
        "stale_revision",
    )?;
    refused(
        &executor,
        &engine,
        &stamp,
        epoch,
        query(0, 2),
        "stale_revision",
    )?;
    refused(
        &executor,
        &engine,
        &stamp,
        epoch,
        query(1, 0),
        "invalid_params",
    )?;
    refused(&executor, &engine, &stamp, epoch, query(1, 4), "not_found")?;
    assert!(!executor.has_active_work());
    engine.lock().map_err(|_| "engine")?.dispatch(
        "document.open_smiles",
        json!({"path":directory.path("guarded.smi")}),
    )?;
    let (_, current_epoch) = context(&engine)?;
    refused(
        &executor,
        &engine,
        &stamp,
        current_epoch,
        query(2, 2),
        "stale_revision",
    )?;
    let (stamp, epoch) = context(&engine)?;
    close(&engine, "request")?;
    refused(
        &executor,
        &engine,
        &stamp,
        context(&engine)?.1,
        query(2, 2),
        "ui_stale_request",
    )?;
    close(&engine, "cancel")?;
    assert!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&stamp)
    );
    refused(
        &executor,
        &engine,
        &stamp,
        epoch,
        query(2, 2),
        "ui_stale_request",
    )?;
    let (stamp, epoch) = context(&engine)?;
    drop(
        executor
            .prepare_ui_smiles(&stamp, epoch, query(2, 2))
            .map_err(|e| e.message)?,
    );
    assert!(!executor.has_active_work());
    close(&engine, "request")?;
    assert!(engine.lock().map_err(|_| "engine")?.complete_close());
    refused(
        &executor,
        &engine,
        &stamp,
        context(&engine)?.1,
        query(2, 2),
        "close_committed",
    )?;
    assert!(!executor.has_active_work());
    let drawing = Arc::new(Mutex::new(Engine::default()));
    let drawing_executor = CommandExecutor::new(Arc::clone(&drawing));
    let (stamp, epoch) = context(&drawing)?;
    refused(
        &drawing_executor,
        &drawing,
        &stamp,
        epoch,
        query(0, 1),
        "unsupported_content",
    )?;
    assert!(!drawing_executor.has_active_work());
    Ok(())
}

#[test]
fn native_smiles_work_shares_two_slots_cancels_and_cannot_release_or_acknowledge_saves()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let shared = Arc::new(Mutex::new(Engine::default()));
    let executor = CommandExecutor::new(Arc::clone(&shared));
    let (stamp, _) = context(&shared)?;
    let capture = executor
        .prepare_ui_capture(
            &stamp,
            RenderCapture {
                page_id: 1,
                width_px: 8,
                height_px: 8,
                backend: RenderBackend::Cpu,
                view: None,
            },
        )
        .map_err(|e| e.message)?;
    let input = directory.path("slots.smi");
    fs::write(&input, SOURCE)?;
    shared
        .lock()
        .map_err(|_| "engine")?
        .dispatch("document.open_smiles", json!({"path":input}))?;
    let before = state(&shared)?;
    let (stamp, epoch) = context(&shared)?;
    let second = executor
        .clone()
        .prepare_ui_smiles(&stamp, epoch, query(1, 2))
        .map_err(|e| e.message)?;
    refused(&executor, &shared, &stamp, epoch, query(1, 2), "busy")?;
    assert_eq!(
        executor
            .execute_blocking(
                Command::SmilesRecordInspect(query(1, 2)),
                &Cancellation::default()
            )
            .err()
            .ok_or("registry escaped shared two slots")?
            .code,
        "busy"
    );
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        second
            .complete_blocking(&cancel)
            .err()
            .ok_or("cancelled work completed")?
            .code,
        "cancelled"
    );
    assert!(
        executor.has_active_work(),
        "the original capture still owns its slot"
    );
    let replacement = executor
        .prepare_ui_smiles(&stamp, epoch, query(1, 2))
        .map_err(|e| e.message)?;
    drop(capture);
    drop(replacement);
    assert!(!executor.has_active_work());
    assert_eq!(state(&shared)?, before);
    let output = directory.path("saved-whole.smiles");
    let save = shared
        .lock()
        .map_err(|_| "engine")?
        .prepare(Command::parse(
            "document.save_smiles",
            json!({"path":output}),
        )?)?;
    let work = executor
        .prepare_ui_smiles(&stamp, epoch, query(1, 2))
        .map_err(|e| e.message)?;
    let result = work
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert!(result.smiles_record.is_some() && !result.changed);
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    assert_eq!(
        shared
            .lock()
            .map_err(|_| "engine")?
            .prepare(Command::parse(
                "document.save_smiles",
                json!({"path":output})
            )?)
            .err()
            .ok_or("query released another Save's lease")?
            .info()
            .code,
        "busy"
    );
    let done = save.complete()?;
    shared.lock().map_err(|_| "engine")?.finish(done)?;
    assert_eq!(fs::read(output)?, SOURCE);
    assert_eq!(fs::read(input)?, SOURCE);
    let mut guard = shared.lock().map_err(|_| "engine")?;
    guard.set_ui_file_host(true);
    let id = guard
        .dispatch("ui.file", json!({"expected_revision":1,"action":"save_as"}))?
        .ui_file
        .request
        .ok_or("native file request")?
        .id;
    guard.claim_ui_file(id)?;
    let stamp = guard.capture_stamp();
    let epoch = guard.state().close_epoch;
    let before = serde_json::to_value(guard.state())?;
    drop(guard);
    let work = executor
        .prepare_ui_smiles(&stamp, epoch, query(1, 2))
        .map_err(|e| e.message)?;
    work.complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert_eq!(state(&shared)?, before);
    let mut guard = shared.lock().map_err(|_| "engine")?;
    assert_eq!(
        guard
            .prepare_ui_file(id, Command::SmilesRecordInspect(query(1, 2)))
            .err()
            .ok_or("query bound file intent")?
            .info()
            .code,
        "invalid_params"
    );
    assert!(!guard.finish_ui_file(id, false));
    let mut consumed = before;
    consumed["ui_file"]["request"] = Value::Null;
    assert_eq!(serde_json::to_value(guard.state())?, consumed);
    drop(guard);
    assert!(!executor.has_active_work());
    Ok(())
}
