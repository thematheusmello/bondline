//! Independent native preparation/worker ownership tests; never a second paste engine.

use bondline_commands::{Cancellation, Engine, PasteSmiles, UiFilePlan};
use bondline_mcp::{CommandExecutor, UiSmilesPasteContext};
use serde_json::json;
use std::sync::{Arc, Mutex};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn native_paste_claims_one_intent_and_prepares_the_real_registered_edit() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":0,"action":"paste_smiles"}),
    )?;
    let id = engine.state().ui_file.request.ok_or("paste intent")?.id;
    assert!(matches!(engine.claim_ui_file(id)?, UiFilePlan::SmilesPaste));
    let context = UiSmilesPasteContext::capture(&engine, id, 1).map_err(|e| e.message)?;
    assert!(context.is_current(&engine));
    let shared = Arc::new(Mutex::new(engine));
    let executor = CommandExecutor::new(Arc::clone(&shared));
    let params: PasteSmiles = serde_json::from_value(json!({
        "expected_revision":0,"page_id":1,"notation":"c1ccccc1",
        "bond_length_pt":64.0,"origin_x_pt":96.0,"origin_y_pt":96.0
    }))?;
    let work = executor
        .prepare_ui_smiles_paste(&context, params)
        .map_err(|e| e.message)?;
    assert!(executor.has_active_work());
    {
        let engine = shared.lock().map_err(|_| "engine")?;
        assert!(engine.state().ui_file.request.is_none());
        assert!(
            context.is_current(&engine),
            "consuming intent is not a source change"
        );
        assert_eq!(engine.state().revision, 0);
    }
    let result = work
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert!(result.changed);
    assert_eq!((result.revision, result.undo_depth), (1, 1));
    assert!(result.file.dirty);
    assert!(result.file_operation.is_none());
    assert!(!executor.has_active_work());
    Ok(())
}

use bondline_commands::{Command, RenderBackend, RenderCapture};
use bondline_mcp::SharedEngine;
use serde_json::Value;
use std::{fs, path::Path, sync::mpsc, time::Duration};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the unchanged isolated filesystem owner")]
mod support;

fn origin() -> (SharedEngine, CommandExecutor) {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let shared = Arc::new(Mutex::new(engine));
    (Arc::clone(&shared), CommandExecutor::new(shared))
}

fn params(revision: u64, page: u32, notation: &str) -> TestResult<PasteSmiles> {
    Ok(serde_json::from_value(json!({
        "expected_revision":revision,"page_id":page,"notation":notation,
        "bond_length_pt":64.0,"origin_x_pt":96.0,"origin_y_pt":96.0
    }))?)
}

fn request(shared: &SharedEngine) -> TestResult<(u64, UiSmilesPasteContext)> {
    let mut engine = shared.lock().map_err(|_| "engine")?;
    let revision = engine.state().revision;
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":revision,"action":"paste_smiles"}),
    )?;
    let id = engine.state().ui_file.request.ok_or("paste intent")?.id;
    assert!(matches!(engine.claim_ui_file(id)?, UiFilePlan::SmilesPaste));
    let context = UiSmilesPasteContext::capture(&engine, id, 1).map_err(|e| e.message)?;
    Ok((id, context))
}

fn state(shared: &SharedEngine) -> TestResult<Value> {
    let engine = shared.lock().map_err(|_| "engine")?;
    let view = engine.view();
    Ok(json!({"state":engine.state(),"view":{
        "zoom_percent":view.zoom_percent,"center":view.center,"epoch":view.epoch
    }}))
}

fn consumed(mut accepted: Value) -> Value {
    accepted["state"]["ui_file"]["request"] = Value::Null;
    accepted
}

fn dispatch(shared: &SharedEngine, command: &str, params: Value) -> TestResult {
    shared
        .lock()
        .map_err(|_| "engine")?
        .dispatch(command, params)?;
    Ok(())
}

fn capture() -> RenderCapture {
    RenderCapture {
        page_id: 1,
        width_px: 64,
        height_px: 64,
        backend: RenderBackend::Cpu,
        view: None,
    }
}

fn rejected(
    shared: &SharedEngine,
    executor: &CommandExecutor,
    context: &UiSmilesPasteContext,
    params: PasteSmiles,
    code: &str,
) -> TestResult {
    let before = state(shared)?;
    let error = executor
        .prepare_ui_smiles_paste(context, params)
        .err()
        .ok_or("unexpected native paste acceptance")?;
    assert_eq!(error.code, code);
    assert_eq!(state(shared)?, before);
    Ok(())
}

#[test]
fn shared_capacity_is_reserved_before_consumption_and_never_claims_save_authority() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let (shared, executor) = origin();
    let stamp = shared.lock().map_err(|_| "engine")?.capture_stamp();
    let one = executor
        .prepare_ui_capture(&stamp, capture())
        .map_err(|e| e.message)?;
    let two = executor
        .prepare_ui_capture(&stamp, capture())
        .map_err(|e| e.message)?;
    let (id, context) = request(&shared)?;
    let accepted = state(&shared)?;
    rejected(
        &shared,
        &executor,
        &context,
        params(0, 1, "c1ccccc1")?,
        "busy",
    )?;
    assert_eq!(
        state(&shared)?,
        accepted,
        "full capacity cannot consume the intent"
    );
    drop(one);
    let generic = executor.prepare_ui_file(id, Command::PasteSmiles(params(0, 1, "c1ccccc1")?));
    assert_eq!(
        generic
            .err()
            .ok_or("generic file binder accepted paste")?
            .code,
        "invalid_params"
    );
    assert_eq!(state(&shared)?, accepted);
    let clone = context.clone();
    let work = executor
        .clone()
        .prepare_ui_smiles_paste(&clone, params(0, 1, "c1ccccc1")?)
        .map_err(|e| e.message)?;
    let expected = consumed(accepted);
    assert_eq!(state(&shared)?, expected);
    rejected(
        &shared,
        &executor,
        &context,
        params(0, 1, "c1ccccc1")?,
        "ui_stale_request",
    )?;
    drop(two);
    let output = directory.path("must-not-publish-λ.cdxml");
    let save = shared
        .lock()
        .map_err(|_| "engine")?
        .prepare(Command::parse(
            "document.save_cdxml",
            json!({"path":output}),
        )?)?;
    assert!(
        save.is_deferred(),
        "paste cannot take the independent Save lease"
    );
    drop(save);
    assert!(!output.try_exists()?);
    assert!(executor.has_active_work());
    drop(work);
    assert!(!executor.has_active_work());
    assert_eq!(state(&shared)?, expected);
    let mut engine = shared.lock().map_err(|_| "engine")?;
    assert!(context.is_current(&engine));
    assert!(
        !engine.finish_ui_file(id, false),
        "no completed Save acknowledgement exists"
    );
    Ok(())
}

#[test]
fn invalid_explicit_params_and_foreign_context_cannot_consume_or_leak_native_work() -> TestResult {
    let (shared, executor) = origin();
    let (_, context) = request(&shared)?;
    let before = state(&shared)?;
    for _ in 0..3 {
        rejected(
            &shared,
            &executor,
            &context,
            params(1, 1, "c1ccccc1")?,
            "stale_revision",
        )?;
        rejected(
            &shared,
            &executor,
            &context,
            params(0, 2, "c1ccccc1")?,
            "invalid_params",
        )?;
        assert!(!executor.has_active_work());
    }
    let (foreign, other) = origin();
    let (_, actual) = request(&foreign)?;
    {
        let engine = foreign.lock().map_err(|_| "engine")?;
        assert!(!context.is_current(&engine));
    }
    rejected(
        &foreign,
        &other,
        &context,
        params(0, 1, "c1ccccc1")?,
        "stale_revision",
    )?;
    assert!(!other.has_active_work());
    let work = other
        .prepare_ui_smiles_paste(&actual, params(0, 1, "c1ccccc1")?)
        .map_err(|e| e.message)?;
    drop(work);
    assert!(!other.has_active_work());
    assert_eq!(state(&shared)?, before);
    Ok(())
}

#[test]
fn cancelled_or_failed_work_requires_explicit_rearming_and_keeps_source_history_exact() -> TestResult
{
    let (shared, executor) = origin();
    let (id, context) = request(&shared)?;
    let expected = consumed(state(&shared)?);
    let work = executor
        .prepare_ui_smiles_paste(&context, params(0, 1, "c1ccccc1")?)
        .map_err(|e| e.message)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        work.complete_blocking(&cancel)
            .err()
            .ok_or("cancelled paste committed")?
            .code,
        "cancelled"
    );
    assert!(!executor.has_active_work());
    assert_eq!(state(&shared)?, expected);
    {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        assert!(context.is_current(&engine));
        assert!(!engine.finish_ui_file(id, false));
    }
    rejected(
        &shared,
        &executor,
        &context,
        params(0, 1, "c1ccccc1")?,
        "ui_stale_request",
    )?;
    let (_, retry) = request(&shared)?;
    let expected = consumed(state(&shared)?);
    let work = executor
        .prepare_ui_smiles_paste(&retry, params(0, 1, "[NH4+]")?)
        .map_err(|e| e.message)?;
    assert_eq!(
        work.complete_blocking(&Cancellation::default())
            .err()
            .ok_or("charged paste committed")?
            .code,
        "unsupported_content"
    );
    assert_eq!(state(&shared)?, expected);
    assert!(!executor.has_active_work());
    {
        let engine = shared.lock().map_err(|_| "engine")?;
        assert!(
            !context.is_current(&engine),
            "rearming advances the file-intent epoch"
        );
        assert!(retry.is_current(&engine));
    }
    let (_, fresh) = request(&shared)?;
    let result = executor
        .prepare_ui_smiles_paste(&fresh, params(0, 1, "c1ccccc1")?)
        .map_err(|e| e.message)?
        .complete_blocking(&Cancellation::default())
        .map_err(|e| e.message)?;
    assert_eq!((result.revision, result.undo_depth), (1, 1));
    assert!(result.changed && result.file_operation.is_none());
    let engine = shared.lock().map_err(|_| "engine")?;
    assert!(
        !fresh.is_current(&engine),
        "a real authoring commit changes source identity"
    );
    Ok(())
}

fn change_context(shared: &SharedEngine, case: &str, path: &Path) -> TestResult {
    match case {
        "view_aba" => {
            dispatch(
                shared,
                "ui.view",
                json!({"action":"set_zoom","percent":150.0}),
            )?;
            dispatch(shared, "ui.view", json!({"action":"fit"}))?;
        }
        "close_aba" => {
            dispatch(
                shared,
                "document.close",
                json!({"expected_revision":0,"action":"request"}),
            )?;
            dispatch(
                shared,
                "document.close",
                json!({"expected_revision":0,"action":"cancel"}),
            )?;
        }
        "file_epoch" => dispatch(
            shared,
            "ui.file",
            json!({"expected_revision":0,"action":"open"}),
        )?,
        "association" => dispatch(shared, "document.save_cdxml", json!({"path":path}))?,
        "source_aba" => {
            dispatch(
                shared,
                "bond.create",
                json!({"page_id":1,"begin_x_pt":72.0,
                "begin_y_pt":480.0,"end_x_pt":120.0,"end_y_pt":480.0}),
            )?;
            dispatch(shared, "history.undo", json!({}))?;
        }
        _ => return Err("unknown original context vector".into()),
    }
    Ok(())
}

#[test]
fn consumed_context_predicate_latches_view_close_file_and_source_changes_before_new_preparation()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for case in [
        "view_aba",
        "close_aba",
        "file_epoch",
        "association",
        "source_aba",
    ] {
        let (shared, executor) = origin();
        let (_, context) = request(&shared)?;
        let work = executor
            .prepare_ui_smiles_paste(&context, params(0, 1, "c1ccccc1")?)
            .map_err(|e| e.message)?;
        drop(work);
        {
            let engine = shared.lock().map_err(|_| "engine")?;
            assert!(
                context.is_current(&engine),
                "intent consumption alone is not stale"
            );
        }
        change_context(
            &shared,
            case,
            &directory.path(&format!("before-{case}.cdxml")),
        )?;
        {
            let engine = shared.lock().map_err(|_| "engine")?;
            assert!(!context.is_current(&engine), "stale {case}");
        }
        rejected(
            &shared,
            &executor,
            &context,
            params(0, 1, "c1ccccc1")?,
            if case == "source_aba" {
                "stale_revision"
            } else {
                "ui_stale_request"
            },
        )?;
        assert!(!executor.has_active_work());
    }
    Ok(())
}

#[test]
fn prepared_work_revalidates_full_ui_context_after_off_lock_completion_before_any_commit()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for case in [
        "view_aba",
        "close_aba",
        "file_epoch",
        "association",
        "source_aba",
    ] {
        let (shared, executor) = origin();
        let (id, context) = request(&shared)?;
        let work = executor
            .prepare_ui_smiles_paste(&context, params(0, 1, "c1ccccc1")?)
            .map_err(|e| e.message)?;
        change_context(
            &shared,
            case,
            &directory.path(&format!("finish-{case}.cdxml")),
        )?;
        let latest = state(&shared)?;
        let error = work
            .complete_blocking(&Cancellation::default())
            .err()
            .ok_or("stale native presentation committed an authoring edit")?;
        assert_eq!(
            error.code,
            if case == "source_aba" {
                "stale_revision"
            } else {
                "ui_stale_request"
            },
            "{case}"
        );
        assert_eq!(
            state(&shared)?,
            latest,
            "finish changed current {case} state"
        );
        assert!(!executor.has_active_work());
        let mut engine = shared.lock().map_err(|_| "engine")?;
        assert!(!engine.finish_ui_file(id, false));
    }
    Ok(())
}

#[test]
fn pending_approved_and_committed_close_states_never_authorize_new_native_work() -> TestResult {
    for dirty in [false, true] {
        let (shared, executor) = origin();
        if dirty {
            dispatch(
                &shared,
                "bond.create",
                json!({"page_id":1,"begin_x_pt":72.0,
                "begin_y_pt":480.0,"end_x_pt":120.0,"end_y_pt":480.0}),
            )?;
        }
        let revision = if dirty { 1 } else { 0 };
        let (_, context) = request(&shared)?;
        let work = executor
            .prepare_ui_smiles_paste(&context, params(revision, 1, "c1ccccc1")?)
            .map_err(|e| e.message)?;
        drop(work);
        dispatch(
            &shared,
            "document.close",
            json!({"expected_revision":revision,"action":"request"}),
        )?;
        let current = state(&shared)?;
        assert_eq!(
            current["state"]["close"],
            if dirty { "needs_decision" } else { "approved" }
        );
        rejected(
            &shared,
            &executor,
            &context,
            params(revision, 1, "c1ccccc1")?,
            "ui_stale_request",
        )?;
        if !dirty {
            assert!(!executor.has_active_work());
            assert!(shared.lock().map_err(|_| "engine")?.complete_close());
            rejected(
                &shared,
                &executor,
                &context,
                params(0, 1, "c1ccccc1")?,
                "close_committed",
            )?;
        }
    }
    Ok(())
}

#[test]
fn real_native_source_worker_reports_commit_then_retires_without_any_save_ack() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("native-origin-λ.cdxml");
    let (shared, executor) = origin();
    dispatch(
        &shared,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72.0,
        "begin_y_pt":480.0,"end_x_pt":120.0,"end_y_pt":480.0}),
    )?;
    dispatch(&shared, "document.save_cdxml", json!({"path":input}))?;
    dispatch(&shared, "document.open_cdxml", json!({"path":input}))?;
    let original = fs::read(&input)?;
    let content = shared.lock().map_err(|_| "engine")?.content().clone();
    let (id, context) = request(&shared)?;
    let work = executor
        .prepare_ui_smiles_paste(&context, params(2, 1, "c1ccccc1")?)
        .map_err(|e| e.message)?;
    assert!(
        shared.try_lock().is_ok(),
        "preparation retained the origin mutex"
    );
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (start_tx, start_rx) = mpsc::sync_channel(0);
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let (retire_tx, retire_rx) = mpsc::sync_channel(0);
    let wait = Duration::from_secs(10);
    let worker = std::thread::spawn(move || -> Result<(), String> {
        entered_tx.send(()).map_err(|e| e.to_string())?;
        start_rx.recv_timeout(wait).map_err(|e| e.to_string())?;
        let result = work.complete_blocking(&Cancellation::default());
        result_tx.send(result).map_err(|e| e.to_string())?;
        retire_rx.recv_timeout(wait).map_err(|e| e.to_string())?;
        Ok(())
    });
    entered_rx.recv_timeout(wait)?;
    assert!(executor.clone().has_active_work());
    assert!(!worker.is_finished());
    start_tx.send(())?;
    let reply = result_rx.recv_timeout(wait)?.map_err(|e| e.message)?;
    assert_eq!((reply.revision, reply.undo_depth), (3, 1));
    assert!(reply.changed && reply.file.dirty && reply.file_operation.is_none());
    assert!(
        !executor.has_active_work(),
        "registered method has now returned"
    );
    assert!(
        !worker.is_finished(),
        "UI must separately retain this actual caller thread"
    );
    retire_tx.send(())?;
    worker
        .join()
        .map_err(|_| "native worker panicked")?
        .map_err(|e| e.to_string())?;
    assert!(fs::read(&input)? == original);
    {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        assert!(!context.is_current(&engine));
        assert!(!engine.finish_ui_file(id, false));
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.content(), &content);
        assert!(!engine.state().file.dirty);
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.state().undo_depth, 1);
        assert!(engine.state().file.dirty);
    }
    assert_eq!(fs::read(input)?, original);
    Ok(())
}

#[test]
fn poisoned_origin_refuses_finish_and_fresh_preparation_without_leaking_activity() -> TestResult {
    let (shared, executor) = origin();
    let (_, context) = request(&shared)?;
    let work = executor
        .prepare_ui_smiles_paste(&context, params(0, 1, "c1ccccc1")?)
        .map_err(|e| e.message)?;
    let before = state(&shared)?;
    let owner = Arc::clone(&shared);
    let poisoned = std::thread::spawn(move || {
        let Ok(_held) = owner.lock() else { return };
        std::panic::resume_unwind(Box::new("intentional native paste origin poison"));
    })
    .join();
    assert!(poisoned.is_err());
    assert_eq!(
        work.complete_blocking(&Cancellation::default())
            .err()
            .ok_or("poisoned finish")?
            .code,
        "engine_unavailable"
    );
    assert!(!executor.has_active_work());
    assert_eq!(
        executor
            .prepare_ui_smiles_paste(&context, params(0, 1, "c1ccccc1")?)
            .err()
            .ok_or("poisoned preparation")?
            .code,
        "engine_unavailable"
    );
    assert!(!executor.has_active_work());
    let engine = shared.lock().err().ok_or("real mutex poison")?.into_inner();
    assert_eq!(serde_json::to_value(engine.state())?, before["state"]);
    Ok(())
}

#[test]
fn named_association_a_to_b_to_a_never_revives_consumed_native_paste_authority() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let a = directory.path("association-A-λ.cdxml");
    let b = directory.path("association-B-化学.cdxml");
    let (shared, executor) = origin();
    dispatch(
        &shared,
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72.0,"begin_y_pt":480.0,
            "end_x_pt":120.0,"end_y_pt":480.0}),
    )?;
    dispatch(&shared, "document.save_cdxml", json!({"path":a}))?;
    dispatch(&shared, "document.open_cdxml", json!({"path":a}))?;
    let original = fs::read(&a)?;
    let (owner, stamp) = {
        let engine = shared.lock().map_err(|_| "engine")?;
        assert_eq!(engine.state().revision, 2);
        assert!(!engine.state().file.dirty);
        (engine.content().clone(), engine.capture_stamp())
    };
    let initial = state(&shared)?;
    let fragments = initial["state"]["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("original fragments")?;
    assert_eq!(fragments.len(), 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("original atoms")?;
    assert_eq!(atoms.len(), 2);
    assert!(
        atoms
            .iter()
            .all(|atom| atom["element"] == 6 && atom["explicit_hydrogens"] == 3)
    );
    assert_eq!(fragments[0]["bonds"][0]["order"], "single");

    let (id, context) = request(&shared)?;
    let work = executor
        .prepare_ui_smiles_paste(&context, params(2, 1, "c1ccccc1")?)
        .map_err(|e| e.message)?;
    let consumed = state(&shared)?;
    {
        let engine = shared.lock().map_err(|_| "engine")?;
        assert!(context.is_current(&engine));
        assert!(engine.state().ui_file.request.is_none());
    }
    dispatch(&shared, "document.save_cdxml", json!({"path":b}))?;
    {
        let engine = shared.lock().map_err(|_| "engine")?;
        assert_eq!(engine.content(), &owner);
        assert!(engine.capture_stamp_matches(&stamp));
        assert!(
            !context.is_current(&engine),
            "association B is a different context"
        );
    }
    dispatch(
        &shared,
        "document.save_cdxml",
        json!({"path":a,"overwrite":true}),
    )?;
    let latest = state(&shared)?;
    assert_eq!(
        latest, consumed,
        "the public values returned to their original association"
    );
    assert_eq!(
        latest["state"]["file"]["path"],
        json!(fs::canonicalize(&a)?)
    );
    assert_eq!(latest["state"]["file"]["format"], "cdxml");
    assert_eq!(fs::read(&a)?, original);
    assert_eq!(fs::read(&b)?, original);
    {
        let engine = shared.lock().map_err(|_| "engine")?;
        assert_eq!(engine.content(), &owner);
        assert!(
            engine.capture_stamp_matches(&stamp),
            "model owner/revision/generation are unchanged"
        );
        assert!(
            !context.is_current(&engine),
            "A to B to A cannot revive old paste authority"
        );
    }
    assert!(executor.has_active_work());
    let error = work
        .complete_blocking(&Cancellation::default())
        .err()
        .ok_or("association ABA allowed an obsolete paste to commit")?;
    assert_eq!(error.code, "ui_stale_request");
    assert_eq!(state(&shared)?, latest);
    assert!(!executor.has_active_work());
    {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        assert!(
            !engine.finish_ui_file(id, false),
            "paste has no Save completion authority"
        );
        assert_eq!(engine.content(), &owner);
    }
    assert_eq!(fs::read(&a)?, original);
    assert_eq!(fs::read(&b)?, original);
    Ok(())
}
