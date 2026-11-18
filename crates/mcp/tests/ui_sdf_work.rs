use bondline_commands::{Cancellation, Command, Engine};
use bondline_mcp::{CommandExecutor, SharedEngine};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, Mutex},
};

#[path = "../../../xtask/oracles/sdf_record_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse unchanged independent ordered/DT field literals"
)]
mod expected;
#[path = "../../../xtask/oracles/sdf_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse unchanged isolated directories and complete-source consumers"
)]
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
fn state(engine: &SharedEngine) -> TestResult<Value> {
    Ok(serde_json::to_value(
        engine.lock().map_err(|_| "engine")?.state(),
    )?)
}
fn query(revision: u64, record: u32) -> TestResult<Command> {
    Ok(Command::parse(
        "sdf.record.inspect",
        json!({"expected_revision":revision,"record_id":record}),
    )?)
}

#[test]
fn native_sdf_query_reserves_before_spawn_and_returns_literal_ordered_duplicate_dt_fields()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("ui-query-化学.sdf");
    let (engine, executor) = open(&input)?;
    let before = state(&engine)?;
    let (stamp, epoch) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.capture_stamp(), guard.state().close_epoch)
    };
    let work = executor
        .prepare_ui_sdf(&stamp, epoch, query(1, 1)?)
        .map_err(|e| e.message)?;
    assert!(executor.has_active_work());
    assert!(executor.clone().has_active_work());
    assert!(
        engine.try_lock().is_ok(),
        "preparation must release the originating Engine lock"
    );
    assert_eq!(
        state(&engine)?,
        before,
        "prepare cannot disclose/commit an edit"
    );
    let result = work
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    let value = serde_json::to_value(&result)?;
    assert_eq!(value["sdf_record"]["input_revision"], 1);
    assert_eq!(value["sdf_record"]["record_id"], 1);
    assert_eq!(
        value["sdf_record"]["fields"],
        expected::expected_fields(true, 1)
    );
    assert_eq!(value["sdf_record"]["mol_byte_length"], 344);
    assert_eq!(value["sdf_record"]["record_byte_length"], 462);
    assert!(!result.changed);
    assert!(
        result.document.is_none() && result.capture.is_none() && result.file_operation.is_none()
    );
    assert_eq!(state(&engine)?, before);
    assert!(!executor.has_active_work());
    assert_eq!(fs::read(input)?, SOURCE);
    Ok(())
}
fn context(engine: &SharedEngine) -> TestResult<(bondline_commands::CaptureStamp, u64)> {
    let guard = engine.lock().map_err(|_| "engine")?;
    Ok((guard.capture_stamp(), guard.state().close_epoch))
}
fn set(revision: u64, lines: Value) -> TestResult<Command> {
    Ok(Command::parse(
        "sdf.field.set_value",
        json!({"expected_revision":revision,"record_id":1,"field_index":2,"value_lines":lines}),
    )?)
}
fn prepared(
    executor: &CommandExecutor,
    engine: &SharedEngine,
    command: Command,
) -> TestResult<bondline_mcp::UiSdfWork> {
    let (stamp, epoch) = context(engine)?;
    Ok(executor
        .prepare_ui_sdf(&stamp, epoch, command)
        .map_err(|e| e.message)?)
}
fn source(engine: &SharedEngine) -> TestResult<Vec<u8>> {
    Ok(engine
        .lock()
        .map_err(|_| "engine")?
        .content()
        .sdf()?
        .as_bytes()
        .to_vec())
}
fn edited() -> Vec<u8> {
    [&SOURCE[..378], b"reviewed\nline two\n", &SOURCE[385..]].concat()
}
fn refusal(
    executor: &CommandExecutor,
    engine: &SharedEngine,
    stamp: &bondline_commands::CaptureStamp,
    epoch: u64,
    command: Command,
    expected_code: &str,
) -> TestResult {
    let before = state(engine)?;
    assert_eq!(
        executor
            .prepare_ui_sdf(stamp, epoch, command)
            .err()
            .ok_or("unexpected accepted UI SDF work")?
            .code,
        expected_code
    );
    assert_eq!(state(engine)?, before);
    Ok(())
}
fn close(engine: &SharedEngine, action: &str) -> TestResult {
    let mut guard = engine.lock().map_err(|_| "engine")?;
    let revision = guard.state().revision;
    guard.dispatch(
        "document.close",
        json!({"expected_revision":revision,"action":action}),
    )?;
    Ok(())
}

#[test]
fn native_existing_duplicate_edit_has_literal_source_history_noop_and_independent_saved_consumers()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("native-field-input.sdf");
    let output = directory.path("native-field-output-化学.sdf");
    let (engine, executor) = open(&input)?;
    let before = state(&engine)?;
    let work = prepared(&executor, &engine, set(1, json!(["reviewed", "line two"]))?)?;
    assert_eq!(
        state(&engine)?,
        before,
        "prepare must not commit the candidate"
    );
    let result = work
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 1, 0)
    );
    assert!(result.document.is_none() && result.file.dirty);
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    assert_eq!(source(&engine)?, edited());
    let current = state(&engine)?;
    assert_eq!(
        current["source"],
        json!({"format":"sdf","byte_length":777,"modified":true})
    );
    assert_eq!(current["collection"]["byte_length"], 788);
    let query_result = prepared(&executor, &engine, query(2, 1)?)?
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    let page = serde_json::to_value(query_result.sdf_record.ok_or("explicit selected fields")?)?;
    let mut fields = expected::expected_fields(true, 1);
    fields[2]["value_lines"] = json!(["reviewed", "line two"]);
    fields[2]["value_byte_length"] = json!(18);
    fields[2]["value_line_count"] = json!(2);
    assert_eq!(page["fields"], fields);
    expected::assert_json_budget(&page, None)?;
    let (stamp, _) = context(&engine)?;
    let pointer = engine
        .lock()
        .map_err(|_| "engine")?
        .content()
        .sdf()?
        .as_bytes()
        .as_ptr() as usize;
    let noop = prepared(&executor, &engine, set(2, json!(["reviewed", "line two"]))?)?
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert!(!noop.changed);
    assert_eq!(state(&engine)?, current);
    assert!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&stamp)
    );
    assert_eq!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .sdf()?
            .as_bytes()
            .as_ptr() as usize,
        pointer
    );
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("history.undo", json!({}))?;
    assert_eq!(source(&engine)?, SOURCE);
    assert_eq!(state(&engine)?["redo_depth"], 1);
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("history.redo", json!({}))?;
    assert_eq!(source(&engine)?, edited());
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("document.save_sdf", json!({"path":output}))?;
    let bytes = fs::read(&output)?;
    assert_eq!(
        bytes,
        edited(),
        "full literal splice retains every other source byte"
    );
    assert_eq!(bytes.len(), 788);
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let measured=std::process::Command::new(python).args(["-B","-c",r#"
import hashlib,sys
from pathlib import Path
b=Path(sys.argv[1]).read_bytes()
assert len(b)==788
assert hashlib.sha256(b).hexdigest()=='e63cb9adf5cbc62b5b0be94992246c0d10b94cda405e49b9514993e7d9f9c361'
assert b[378:396]==b'reviewed\nline two\n'
assert b[344:354]==b'> <EMPTY>\n' and b[354:355]==b'\n'
assert b[355:363]==b'> <DUP>\n' and b[363:369]==b'first\n'
assert b[370:378]==b'> <DUP>\n'
assert b[397:422]==b'> 17(REF-42)<MASS>DT0009\n'
assert b[422:437]==b'carbon\nisotope\n'
assert b[438:447]==b'> DT0012\n' and b[447:451]==b'raw\n'
assert b[452:463]==b'> <SPACES>\n' and b[463:467]==b'   \n'
assert b[468:473]==b'$$$$\n'
"#]).arg(&output).output()?;
    assert!(
        measured.status.success(),
        "independent raw/hash check: {}",
        String::from_utf8_lossy(&measured.stderr)
    );
    let first = support::describe_source("v3000_oracle.py", &bytes[..344])?;
    let second = support::describe_source("mol_oracle.py", &bytes[473..755])?;
    assert_eq!(first["atoms"].as_array().ok_or("V3000 atoms")?.len(), 2);
    assert_eq!(first["bonds"].as_array().ok_or("V3000 bonds")?.len(), 1);
    assert_eq!(second["smiles"], "CC");
    assert_eq!(second["formula"], "C2H6");
    assert_eq!(fs::read(input)?, SOURCE);
    assert!(!executor.has_active_work());
    Ok(())
}

#[test]
fn native_sdf_work_shares_capture_slots_and_drop_error_or_cancel_releases_each_reservation()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let (engine, executor) = open(&directory.path("slots.sdf"))?;
    let before = state(&engine)?;
    let (stamp, epoch) = context(&engine)?;
    let first = prepared(&executor, &engine, query(1, 1)?)?;
    let capture = executor
        .clone()
        .prepare_ui_capture(
            &stamp,
            bondline_commands::RenderCapture {
                page_id: 1,
                width_px: 1024,
                height_px: 1024,
                backend: bondline_commands::RenderBackend::Cpu,
                view: None,
            },
        )
        .map_err(|e| e.message)?;
    refusal(&executor, &engine, &stamp, epoch, query(1, 999)?, "busy")?;
    assert_eq!(
        executor
            .execute_blocking(query(1, 1)?, &Cancellation::default())
            .err()
            .ok_or("registry worker escaped two slots")?
            .code,
        "busy"
    );
    drop(first);
    refusal(
        &executor,
        &engine,
        &stamp,
        epoch,
        query(1, 999)?,
        "not_found",
    )?;
    let replacement = prepared(&executor, &engine, query(1, 1)?)?;
    drop(capture);
    drop(replacement);
    assert!(!executor.has_active_work());
    let oversized = prepared(&executor, &engine, set(1, json!(["x".repeat(65536)]))?)?;
    assert_eq!(
        oversized
            .complete_blocking(&Cancellation::default())
            .err()
            .ok_or("oversized candidate completed")?
            .code,
        "resource_limit"
    );
    assert!(!executor.has_active_work());
    for command in [query(1, 1)?, set(1, json!(["reviewed", "line two"]))?] {
        let work = prepared(&executor, &engine, command)?;
        let cancel = Cancellation::default();
        cancel.cancel();
        assert_eq!(
            work.complete_blocking(&cancel)
                .err()
                .ok_or("cancelled UI work completed")?
                .code,
            "cancelled"
        );
        assert!(!executor.has_active_work());
    }
    assert_eq!(state(&engine)?, before);
    assert_eq!(source(&engine)?, SOURCE);
    Ok(())
}

#[test]
fn family_identity_kind_and_close_epoch_guards_refuse_without_immediate_mutation_or_leaks()
-> TestResult {
    let drawing = Arc::new(Mutex::new(Engine::default()));
    drawing.lock().map_err(|_| "engine")?.set_ui_file_host(true);
    let executor = CommandExecutor::new(Arc::clone(&drawing));
    let (stamp, epoch) = context(&drawing)?;
    for command in [
        Command::parse(
            "page.set_size",
            json!({"page_id":1,"width_pt":100,"height_pt":100}),
        )?,
        Command::parse("document.inspect", json!({}))?,
        Command::parse(
            "render.capture",
            json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
        )?,
        Command::parse("ui.file", json!({"expected_revision":0,"action":"open"}))?,
    ] {
        refusal(
            &executor,
            &drawing,
            &stamp,
            epoch,
            command,
            "invalid_params",
        )?;
        assert!(!executor.has_active_work());
    }
    refusal(
        &executor,
        &drawing,
        &stamp,
        epoch,
        query(0, 1)?,
        "unsupported_content",
    )?;
    let directory = support::SdfTestDirectory::new()?;
    let (engine, executor) = open(&directory.path("identity.sdf"))?;
    let (foreign, _) = open(&directory.path("foreign.sdf"))?;
    let (foreign_stamp, _) = context(&foreign)?;
    let (stamp, epoch) = context(&engine)?;
    refusal(
        &executor,
        &engine,
        &foreign_stamp,
        epoch,
        query(1, 1)?,
        "stale_revision",
    )?;
    close(&engine, "request")?;
    close(&engine, "cancel")?;
    assert!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&stamp)
    );
    refusal(
        &executor,
        &engine,
        &stamp,
        epoch,
        query(1, 1)?,
        "ui_stale_request",
    )?;
    drop(prepared(&executor, &engine, query(1, 1)?)?);
    engine.lock().map_err(|_| "engine")?.dispatch(
        "sdf.field.set_value",
        json!({"expected_revision":1,"record_id":1,"field_index":0,"value_lines":["changed"]}),
    )?;
    let current_epoch = context(&engine)?.1;
    refusal(
        &executor,
        &engine,
        &stamp,
        current_epoch,
        query(2, 1)?,
        "stale_revision",
    )?;
    let (stamp, _) = context(&engine)?;
    close(&engine, "request")?;
    assert_eq!(state(&engine)?["close"], "needs_decision");
    refusal(
        &executor,
        &engine,
        &stamp,
        context(&engine)?.1,
        query(2, 1)?,
        "ui_stale_request",
    )?;
    close(&engine, "discard")?;
    assert_eq!(state(&engine)?["close"], "approved");
    refusal(
        &executor,
        &engine,
        &stamp,
        context(&engine)?.1,
        query(2, 1)?,
        "ui_stale_request",
    )?;
    assert!(!executor.has_active_work());
    assert!(engine.lock().map_err(|_| "engine")?.complete_close());
    refusal(
        &executor,
        &engine,
        &stamp,
        context(&engine)?.1,
        query(2, 1)?,
        "close_committed",
    )?;
    assert!(!executor.has_active_work());
    Ok(())
}

#[test]
fn accepted_work_blocks_host_close_and_earlier_field_commit_invalidates_approval() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    for edit in [false, true] {
        let (engine, executor) = open(&directory.path(if edit {
            "close-edit.sdf"
        } else {
            "close-query.sdf"
        }))?;
        let work = prepared(
            &executor,
            &engine,
            if edit {
                set(1, json!(["reviewed", "line two"]))?
            } else {
                query(1, 1)?
            },
        )?;
        close(&engine, "request")?;
        assert_eq!(state(&engine)?["close"], "approved");
        {
            let mut guard = engine.try_lock().map_err(|_| "preparation retained lock")?;
            let host_committed = if executor.has_active_work() {
                false
            } else {
                guard.complete_close()
            };
            assert!(
                !host_committed,
                "prepared work must remain in the host close busy gate"
            );
        }
        let result = work
            .complete_blocking(&Cancellation::default())
            .map_err(|e| e.message)?;
        assert!(!executor.has_active_work());
        if edit {
            assert!(result.changed);
            assert_eq!(state(&engine)?["close"], "needs_decision");
            assert!(!engine.lock().map_err(|_| "engine")?.complete_close());
            assert_eq!(source(&engine)?, edited());
        } else {
            assert!(!result.changed);
            assert!(result.sdf_record.is_some());
            assert!(engine.lock().map_err(|_| "engine")?.complete_close());
        }
    }
    let (engine, executor) = open(&directory.path("close-drop.sdf"))?;
    let work = prepared(&executor, &engine, query(1, 1)?)?;
    close(&engine, "request")?;
    drop(work);
    assert!(!executor.has_active_work());
    assert!(engine.lock().map_err(|_| "engine")?.complete_close());
    Ok(())
}

#[test]
fn historical_query_keeps_complete_source_while_stale_native_field_work_cannot_apply() -> TestResult
{
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("historical.sdf");
    let next = directory.path("replacement.smi");
    fs::write(&next, b"O current\n")?;
    let (engine, executor) = open(&input)?;
    let query_work = prepared(&executor, &engine, query(1, 2)?)?;
    let edit_work = prepared(&executor, &engine, set(1, json!(["reviewed", "line two"]))?)?;
    fs::remove_file(input)?;
    engine
        .lock()
        .map_err(|_| "engine")?
        .dispatch("document.open_smiles", json!({"path":next}))?;
    let before = state(&engine)?;
    assert_eq!(
        edit_work
            .complete_blocking(&Cancellation::default())
            .err()
            .ok_or("stale native edit applied")?
            .code,
        "stale_revision"
    );
    assert_eq!(state(&engine)?, before);
    close(&engine, "request")?;
    let closed = state(&engine)?;
    let result = query_work
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    let page = serde_json::to_value(result.sdf_record.ok_or("historical query payload")?)?;
    assert_eq!(page["input_revision"], 1);
    assert_eq!(page["record_id"], 2);
    assert_eq!(page["fields"], expected::expected_fields(true, 2));
    assert_eq!(
        state(&engine)?,
        closed,
        "historical payload does not replace current content/lifecycle"
    );
    close(&engine, "cancel")?;
    let (stamp, epoch) = context(&engine)?;
    refusal(
        &executor,
        &engine,
        &stamp,
        epoch,
        query(2, 1)?,
        "unsupported_content",
    )?;
    assert!(!executor.has_active_work());
    Ok(())
}

#[test]
fn native_sdf_work_neither_takes_a_save_lease_nor_acknowledges_a_file_picker() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let (engine, executor) = open(&directory.path("lease.sdf"))?;
    let path = directory.path("pending-save.sdf");
    let query_work = prepared(&executor, &engine, query(1, 1)?)?;
    let edit_work = prepared(&executor, &engine, set(1, json!(["reviewed", "line two"]))?)?;
    let save = engine
        .lock()
        .map_err(|_| "engine")?
        .prepare(Command::parse("document.save_sdf", json!({"path":path}))?)?;
    query_work
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    drop(edit_work);
    assert_eq!(
        engine
            .lock()
            .map_err(|_| "engine")?
            .prepare(Command::parse("document.save_sdf", json!({"path":path}))?)
            .err()
            .ok_or("UI query released another Save lease")?
            .info()
            .code,
        "busy"
    );
    drop(save);
    assert!(!path.exists());
    let mut guard = engine.lock().map_err(|_| "engine")?;
    guard.set_ui_file_host(true);
    let id = guard
        .dispatch("ui.file", json!({"expected_revision":1,"action":"save_as"}))?
        .ui_file
        .request
        .ok_or("picker request")?
        .id;
    guard.claim_ui_file(id)?;
    drop(guard);
    let before = state(&engine)?;
    let result = prepared(&executor, &engine, query(1, 1)?)?
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert!(result.ui_file_operation.is_none() && result.file_operation.is_none());
    assert_eq!(state(&engine)?, before);
    let mut guard = engine.lock().map_err(|_| "engine")?;
    assert_eq!(
        guard
            .prepare_ui_file(id, query(1, 1)?)
            .err()
            .ok_or("SDF query bound a native File request")?
            .info()
            .code,
        "invalid_params"
    );
    assert!(!guard.finish_ui_file(id, false));
    drop(guard);
    assert!(!executor.has_active_work());
    Ok(())
}
