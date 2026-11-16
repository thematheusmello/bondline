//! Independent native complete-source SMILES import and order-worker ownership.

use bondline_commands::{Cancellation, Engine, ImportSmilesRecords};
use bondline_mcp::{CommandExecutor, UiSmilesRecordsContext};
use serde_json::json;
use std::{
    fs,
    sync::{Arc, Mutex},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the unchanged isolated filesystem owner")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn native_import_consumes_one_claimed_intent_and_returns_current_postfinish_proof() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("native-records-λ.smi");
    let source = b"CC\tPRIVATE\0\xff\r\n";
    fs::write(&input, source)?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":1,"action":"import_smiles_records"}),
    )?;
    let id = engine.state().ui_file.request.ok_or("import intent")?.id;
    engine.claim_ui_file(id)?;
    let context =
        UiSmilesRecordsContext::capture_import(&engine, id).map_err(|error| error.message)?;
    assert!(context.is_current(&engine));
    let shared = Arc::new(Mutex::new(engine));
    let executor = CommandExecutor::new(Arc::clone(&shared));
    let params: ImportSmilesRecords = serde_json::from_value(json!({
        "expected_revision":1,"selection":{"mode":"all"},"bond_length_pt":64.0,
        "origin_x_pt":96.0,"origin_y_pt":96.0,"page_width_pt":612.0,"page_height_pt":792.0
    }))?;
    let work = executor
        .prepare_ui_smiles_import(&context, params)
        .map_err(|error| error.message)?;
    assert!(executor.has_active_work());
    {
        let engine = shared.lock().map_err(|_| "engine")?;
        assert!(engine.state().ui_file.request.is_none());
        assert!(
            context.is_current(&engine),
            "consumption alone does not stale source context"
        );
        assert_eq!(engine.state().revision, 1);
    }
    let outcome = work
        .complete_blocking(&Cancellation::default())
        .map_err(|error| error.message)?;
    assert!(outcome.result.changed);
    assert_eq!((outcome.result.revision, outcome.result.undo_depth), (2, 0));
    assert!(!outcome.result.file.dirty);
    assert!(outcome.result.file_operation.is_none());
    let mut engine = shared.lock().map_err(|_| "engine")?;
    assert!(outcome.is_current(&engine));
    assert!(!context.is_current(&engine));
    assert!(!engine.finish_ui_file(id, false));
    assert!(!executor.has_active_work());
    assert_eq!(fs::read(input)?, source);
    Ok(())
}

use bondline_commands::{
    BondDrawOrder, BondSetOrder, Command, ErrorInfo, OrderHydrogenPolicy, RenderBackend,
    RenderCapture, SmilesRecordInspect, SmilesRecordSelection,
};
use bondline_mcp::{SharedEngine, UiSmilesRecordsWork};
use serde_json::Value;
use std::{path::Path, sync::mpsc, time::Duration};

const SOURCE: &[u8] = b"CC\tPRIVATE\0\xff\r\nN\tSECOND\0\xff";

fn import_params(revision: u64) -> ImportSmilesRecords {
    ImportSmilesRecords {
        expected_revision: revision,
        selection: SmilesRecordSelection::Record { record_id: 1 },
        bond_length_pt: 64.0,
        origin_x_pt: 96.0,
        origin_y_pt: 96.0,
        page_width_pt: 612.0,
        page_height_pt: 792.0,
    }
}

fn order_params(revision: u64) -> BondSetOrder {
    BondSetOrder {
        expected_revision: revision,
        page_id: 1,
        fragment_id: 1,
        bond_id: 1,
        order: BondDrawOrder::Double,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
    }
}

fn origin(path: &Path, projected: bool) -> TestResult<(SharedEngine, CommandExecutor)> {
    fs::write(path, SOURCE)?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_smiles", json!({"path":path}))?;
    if projected {
        engine.execute(Command::ImportSmilesRecords(import_params(1)))?;
    }
    let shared = Arc::new(Mutex::new(engine));
    Ok((Arc::clone(&shared), CommandExecutor::new(shared)))
}

fn state(shared: &SharedEngine) -> TestResult<Value> {
    let engine = shared.lock().map_err(|_| "engine")?;
    let view = engine.view();
    let tool = engine.bond_tool();
    Ok(
        json!({"state":engine.state(),"view":{"zoom_percent":view.zoom_percent,"center":view.center,"epoch":view.epoch},
        "tool":{"order":tool.order,"epoch":tool.epoch}}),
    )
}

fn dispatch(shared: &SharedEngine, name: &str, params: Value) -> TestResult {
    shared
        .lock()
        .map_err(|_| "engine")?
        .dispatch(name, params)?;
    Ok(())
}

fn context(shared: &SharedEngine, import: bool) -> TestResult<UiSmilesRecordsContext> {
    let mut engine = shared.lock().map_err(|_| "engine")?;
    if import {
        let revision = engine.state().revision;
        engine.dispatch(
            "ui.file",
            json!({"expected_revision":revision,"action":"import_smiles_records"}),
        )?;
        let id = engine.state().ui_file.request.ok_or("import request")?.id;
        engine.claim_ui_file(id)?;
        Ok(UiSmilesRecordsContext::capture_import(&engine, id).map_err(|error| error.message)?)
    } else {
        Ok(UiSmilesRecordsContext::capture_order(&engine, 1, 1, 1)
            .map_err(|error| error.message)?)
    }
}

fn prepare(
    executor: &CommandExecutor,
    context: &UiSmilesRecordsContext,
    import: bool,
    revision: u64,
) -> Result<UiSmilesRecordsWork, ErrorInfo> {
    if import {
        executor.prepare_ui_smiles_import(context, import_params(revision))
    } else {
        executor.prepare_ui_smiles_order(context, order_params(revision))
    }
}

fn changed_context(shared: &SharedEngine, case: &str, input: &Path, other: &Path) -> TestResult {
    let revision = shared.lock().map_err(|_| "engine")?.state().revision;
    match case {
        "view_aba" => {
            dispatch(
                shared,
                "ui.view",
                json!({"action":"set_zoom","percent":150.0}),
            )?;
            dispatch(shared, "ui.view", json!({"action":"fit"}))?;
        }
        "tool_aba" => {
            dispatch(shared, "ui.bond_tool", json!({"order":"double"}))?;
            dispatch(shared, "ui.bond_tool", json!({"order":"single"}))?;
        }
        "close_aba" => {
            dispatch(
                shared,
                "document.close",
                json!({"expected_revision":revision,"action":"request"}),
            )?;
            dispatch(
                shared,
                "document.close",
                json!({"expected_revision":revision,"action":"cancel"}),
            )?;
        }
        "file_aba" => {
            dispatch(
                shared,
                "ui.file",
                json!({"expected_revision":revision,"action":"save_as"}),
            )?;
            let id = shared
                .lock()
                .map_err(|_| "engine")?
                .state()
                .ui_file
                .request
                .ok_or("file request")?
                .id;
            dispatch(
                shared,
                "ui.file",
                json!({"expected_revision":revision,"action":"cancel","request_id":id}),
            )?;
        }
        "association_aba" => {
            dispatch(
                shared,
                "document.save_smiles",
                json!({"path":other,"overwrite":true}),
            )?;
            dispatch(
                shared,
                "document.save_smiles",
                json!({"path":input,"overwrite":true}),
            )?;
        }
        "host_aba" => {
            let mut engine = shared.lock().map_err(|_| "engine")?;
            engine.set_ui_file_host(false);
            engine.set_ui_file_host(true);
        }
        "source" => dispatch(
            shared,
            "document.new",
            json!({"expected_revision":revision,"discard_unsaved":true}),
        )?,
        _ => return Err("unknown original context vector".into()),
    }
    Ok(())
}

#[test]
fn native_import_and_order_share_two_slots_without_consuming_busy_intents_or_save_authority()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("capacity.smi");
    let (shared, executor) = origin(&input, false)?;
    let (stamp, close_epoch) = {
        let engine = shared.lock().map_err(|_| "engine")?;
        (engine.capture_stamp(), engine.state().close_epoch)
    };
    let query = || SmilesRecordInspect {
        expected_revision: 1,
        record_id: 1,
    };
    let one = executor
        .prepare_ui_smiles(&stamp, close_epoch, query())
        .map_err(|error| error.message)?;
    let two = executor
        .prepare_ui_smiles(&stamp, close_epoch, query())
        .map_err(|error| error.message)?;
    let import = context(&shared, true)?;
    let accepted = state(&shared)?;
    assert_eq!(
        prepare(&executor, &import, true, 1)
            .err()
            .ok_or("busy import")?
            .code,
        "busy"
    );
    assert_eq!(state(&shared)?, accepted);
    drop(one);
    let id = shared
        .lock()
        .map_err(|_| "engine")?
        .state()
        .ui_file
        .request
        .ok_or("accepted request")?
        .id;
    assert_eq!(
        executor
            .prepare_ui_file(id, Command::ImportSmilesRecords(import_params(1)))
            .err()
            .ok_or("generic binder import")?
            .code,
        "invalid_params"
    );
    assert_eq!(state(&shared)?, accepted);
    let work = prepare(&executor, &import, true, 1).map_err(|error| error.message)?;
    {
        let engine = shared.lock().map_err(|_| "engine")?;
        assert!(engine.state().ui_file.request.is_none());
        assert!(import.is_current(&engine));
    }
    let consumed = state(&shared)?;
    drop(two);
    drop(work);
    assert!(!executor.has_active_work());
    assert_eq!(state(&shared)?, consumed);
    assert!(
        !shared
            .lock()
            .map_err(|_| "engine")?
            .finish_ui_file(id, false)
    );

    let (shared, executor) = origin(&input, true)?;
    let order = context(&shared, false)?;
    let stamp = shared.lock().map_err(|_| "engine")?.capture_stamp();
    let capture = || RenderCapture {
        page_id: 1,
        width_px: 1024,
        height_px: 1024,
        backend: RenderBackend::Cpu,
        view: None,
    };
    let one = executor
        .prepare_ui_capture(&stamp, capture())
        .map_err(|error| error.message)?;
    let two = executor
        .prepare_ui_capture(&stamp, capture())
        .map_err(|error| error.message)?;
    let before = state(&shared)?;
    assert_eq!(
        prepare(&executor, &order, false, 2)
            .err()
            .ok_or("busy order")?
            .code,
        "busy"
    );
    assert_eq!(state(&shared)?, before);
    drop(one);
    let work = prepare(&executor, &order, false, 2).map_err(|error| error.message)?;
    let output = directory.path("not-a-native-save-λ.smiles");
    let save = shared
        .lock()
        .map_err(|_| "engine")?
        .prepare(Command::parse(
            "document.save_smiles",
            json!({"path":output}),
        )?)?;
    assert!(save.is_deferred(), "chemical work has no Save lease");
    drop(save);
    assert!(!output.exists());
    assert_eq!(state(&shared)?, before);
    drop(two);
    drop(work);
    assert!(!executor.has_active_work());
    assert_eq!(fs::read(input)?, SOURCE);
    Ok(())
}

#[test]
fn explicit_targets_claim_phase_foreign_source_and_pending_forms_are_checked_before_spawn()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("validation.smi");
    let (shared, executor) = origin(&input, false)?;
    let id = {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        engine.dispatch(
            "ui.file",
            json!({"expected_revision":1,"action":"import_smiles_records"}),
        )?;
        let id = engine.state().ui_file.request.ok_or("requested import")?.id;
        let before = serde_json::to_value(engine.state())?;
        assert!(
            UiSmilesRecordsContext::capture_import(&engine, id).is_err(),
            "Requested is not Accepted"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        engine.claim_ui_file(id)?;
        id
    };
    let import = {
        let engine = shared.lock().map_err(|_| "engine")?;
        UiSmilesRecordsContext::capture_import(&engine, id).map_err(|error| error.message)?
    };
    let before = state(&shared)?;
    for revision in [0, 2] {
        assert_eq!(
            prepare(&executor, &import, true, revision)
                .err()
                .ok_or("revision mismatch")?
                .code,
            "stale_revision"
        );
        assert_eq!(state(&shared)?, before);
        assert!(!executor.has_active_work());
    }
    let mut invalid = import_params(1);
    invalid.bond_length_pt = 0.0;
    assert_eq!(
        executor
            .prepare_ui_smiles_import(&import, invalid)
            .err()
            .ok_or("invalid prepared import")?
            .code,
        "invalid_params"
    );
    assert_eq!(state(&shared)?, before);
    assert!(!executor.has_active_work());
    let (foreign, other) = origin(&directory.path("foreign.smi"), false)?;
    let _fresh = context(&foreign, true)?;
    assert!(!import.is_current(&*foreign.lock().map_err(|_| "engine")?));
    let foreign_before = state(&foreign)?;
    assert_eq!(
        prepare(&other, &import, true, 1)
            .err()
            .ok_or("foreign context")?
            .code,
        "stale_revision"
    );
    assert_eq!(state(&foreign)?, foreign_before);
    assert!(!other.has_active_work());

    let (shared, executor) = origin(&input, true)?;
    {
        let engine = shared.lock().map_err(|_| "engine")?;
        for (page, fragment, bond) in [(0, 1, 1), (2, 1, 1), (1, 2, 1), (1, 1, 2)] {
            assert!(UiSmilesRecordsContext::capture_order(&engine, page, fragment, bond).is_err());
        }
    }
    let order = context(&shared, false)?;
    let before = state(&shared)?;
    for (page, fragment, bond) in [(2, 1, 1), (1, 2, 1), (1, 1, 2)] {
        let mut params = order_params(2);
        params.page_id = page;
        params.fragment_id = fragment;
        params.bond_id = bond;
        assert_eq!(
            executor
                .prepare_ui_smiles_order(&order, params)
                .err()
                .ok_or("retargeted gesture")?
                .code,
            "invalid_params"
        );
        assert_eq!(state(&shared)?, before);
        assert!(!executor.has_active_work());
    }
    dispatch(
        &shared,
        "ui.file",
        json!({"expected_revision":2,"action":"save_as"}),
    )?;
    let pending = state(&shared)?;
    assert_eq!(
        UiSmilesRecordsContext::capture_order(&*shared.lock().map_err(|_| "engine")?, 1, 1, 1)
            .err()
            .ok_or("pending form")?
            .code,
        "ui_stale_request"
    );
    assert_eq!(
        prepare(&executor, &order, false, 2)
            .err()
            .ok_or("old gesture during form")?
            .code,
        "ui_stale_request"
    );
    assert_eq!(state(&shared)?, pending);
    assert!(!executor.has_active_work());
    Ok(())
}

#[test]
fn cancellation_failure_and_drop_release_work_without_partial_projection_or_implicit_rearming()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for import in [true, false] {
        let input = directory.path(if import {
            "cancel-import.smi"
        } else {
            "cancel-order.smi"
        });
        let (shared, executor) = origin(&input, !import)?;
        let context = context(&shared, import)?;
        let work = prepare(&executor, &context, import, if import { 1 } else { 2 })
            .map_err(|error| error.message)?;
        let before = state(&shared)?;
        let owner = shared.lock().map_err(|_| "engine")?.content().clone();
        let cancel = Cancellation::default();
        cancel.cancel();
        assert_eq!(
            work.complete_blocking(&cancel)
                .err()
                .ok_or("cancelled work")?
                .code,
            "cancelled"
        );
        assert_eq!(state(&shared)?, before);
        assert_eq!(shared.lock().map_err(|_| "engine")?.content(), &owner);
        assert!(!executor.has_active_work());
        assert!(context.is_current(&*shared.lock().map_err(|_| "engine")?));
        if import {
            assert_eq!(
                prepare(&executor, &context, true, 1)
                    .err()
                    .ok_or("implicit import retry")?
                    .code,
                "ui_stale_request"
            );
        }
        assert_eq!(fs::read(input)?, SOURCE);
    }
    let input = directory.path("all-must-not-project-first.smi");
    let unsupported = b"CC\tKEPT\0\xff\r\n[13CH4]\tUNPROJECTED\0\xff";
    let (shared, executor) = origin(&input, false)?;
    fs::write(&input, unsupported)?;
    dispatch(&shared, "document.open_smiles", json!({"path":input}))?;
    let import = context(&shared, true)?;
    let mut params = import_params(2);
    params.selection = SmilesRecordSelection::All {};
    let work = executor
        .prepare_ui_smiles_import(&import, params)
        .map_err(|error| error.message)?;
    let before = state(&shared)?;
    let owner = shared.lock().map_err(|_| "engine")?.content().clone();
    assert_eq!(
        work.complete_blocking(&Cancellation::default())
            .err()
            .ok_or("partial ALL import")?
            .code,
        "unsupported_content"
    );
    assert_eq!(state(&shared)?, before);
    assert_eq!(shared.lock().map_err(|_| "engine")?.content(), &owner);
    assert!(!executor.has_active_work());
    let retry = context(&shared, true)?;
    let outcome = prepare(&executor, &retry, true, 2)
        .map_err(|error| error.message)?
        .complete_blocking(&Cancellation::default())
        .map_err(|error| error.message)?;
    let engine = shared.lock().map_err(|_| "engine")?;
    assert!(outcome.is_current(&engine));
    assert_eq!(engine.document()?.pages()[0].fragments().len(), 1);
    assert_eq!(engine.state().revision, 3);
    assert!(!engine.state().file.dirty);
    assert_eq!(fs::read(input)?, unsupported);
    Ok(())
}

#[test]
fn context_changes_cannot_retarget_preparation_or_commit_after_off_lock_work() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for import in [true, false] {
        let cases: &[&str] = if import {
            &["close_aba", "association_aba", "host_aba", "source"]
        } else {
            &[
                "view_aba",
                "tool_aba",
                "close_aba",
                "file_aba",
                "association_aba",
                "host_aba",
                "source",
            ]
        };
        for after_prepare in [false, true] {
            for case in cases {
                let input = directory.path(&format!("context-{import}-{after_prepare}-{case}.smi"));
                let other =
                    directory.path(&format!("other-{import}-{after_prepare}-{case}.smiles"));
                let (shared, executor) = origin(&input, !import)?;
                let context = context(&shared, import)?;
                let revision = if import { 1 } else { 2 };
                let work = if after_prepare {
                    Some(
                        prepare(&executor, &context, import, revision)
                            .map_err(|error| error.message)?,
                    )
                } else {
                    None
                };
                changed_context(&shared, case, &input, &other)?;
                let latest = state(&shared)?;
                let owner = shared.lock().map_err(|_| "engine")?.content().clone();
                assert!(
                    !context.is_current(&*shared.lock().map_err(|_| "engine")?),
                    "{case}"
                );
                let error = if let Some(work) = work {
                    work.complete_blocking(&Cancellation::default())
                        .err()
                        .ok_or("stale native finish")?
                } else {
                    prepare(&executor, &context, import, revision)
                        .err()
                        .ok_or("stale native preparation")?
                };
                assert_eq!(
                    error.code,
                    if *case == "source" {
                        "stale_revision"
                    } else {
                        "ui_stale_request"
                    },
                    "{case}"
                );
                assert_eq!(state(&shared)?, latest, "{case}");
                assert_eq!(shared.lock().map_err(|_| "engine")?.content(), &owner);
                assert!(!executor.has_active_work(), "{case}");
            }
        }
    }
    for dirty in [false, true] {
        let input = directory.path(if dirty {
            "dirty-close.smi"
        } else {
            "clean-close.smi"
        });
        let (shared, executor) = origin(&input, true)?;
        if dirty {
            dispatch(
                &shared,
                "bond.set_order",
                json!({"expected_revision":2,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
            )?;
        }
        let context = context(&shared, false)?;
        let revision = if dirty { 3 } else { 2 };
        dispatch(
            &shared,
            "document.close",
            json!({"expected_revision":revision,"action":"request"}),
        )?;
        let before = state(&shared)?;
        assert_eq!(
            prepare(&executor, &context, false, revision)
                .err()
                .ok_or("pending close")?
                .code,
            "ui_stale_request"
        );
        assert_eq!(state(&shared)?, before);
        if !dirty {
            assert!(shared.lock().map_err(|_| "engine")?.complete_close());
            assert_eq!(
                prepare(&executor, &context, false, revision)
                    .err()
                    .ok_or("committed close")?
                    .code,
                "close_committed"
            );
        }
        assert!(!executor.has_active_work());
    }
    Ok(())
}

#[test]
fn postfinish_proof_is_current_only_for_its_exact_engine_surface_and_cannot_revive_after_aba()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for import in [true, false] {
        for case in [
            "view_aba",
            "tool_aba",
            "close_aba",
            "file_aba",
            "association_aba",
            "host_aba",
            "source",
        ] {
            let input = directory.path(&format!("proof-{import}-{case}.smi"));
            let other = directory.path(&format!("proof-other-{import}-{case}.smi"));
            let (shared, executor) = origin(&input, !import)?;
            let context = context(&shared, import)?;
            let outcome = prepare(&executor, &context, import, if import { 1 } else { 2 })
                .map_err(|error| error.message)?
                .complete_blocking(&Cancellation::default())
                .map_err(|error| error.message)?;
            assert!(outcome.is_current(&*shared.lock().map_err(|_| "engine")?));
            let wire = serde_json::to_value(&outcome.result)?;
            for field in [
                "ui_file_context_stamp",
                "proof",
                "file_operation",
                "ui_file_operation",
            ] {
                assert!(wire.get(field).is_none(), "{field}");
            }
            let (foreign, _) = origin(
                &directory.path(&format!("foreign-proof-{import}-{case}.smi")),
                true,
            )?;
            assert!(!outcome.is_current(&*foreign.lock().map_err(|_| "engine")?));
            changed_context(&shared, case, &input, &other)?;
            assert!(
                !outcome.is_current(&*shared.lock().map_err(|_| "engine")?),
                "postfinish {case}"
            );
            assert!(!executor.has_active_work());
        }
    }
    Ok(())
}

#[test]
fn an_existing_running_save_can_coexist_with_order_work_but_cannot_acknowledge_its_new_edit()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("running-input.smi");
    let output = directory.path("running-save-λ.smiles");
    let (shared, executor) = origin(&input, true)?;
    let id = {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        engine.dispatch("ui.file", json!({"expected_revision":2,"action":"save_as"}))?;
        let id = engine.state().ui_file.request.ok_or("save request")?.id;
        engine.claim_ui_file(id)?;
        id
    };
    let save = executor
        .prepare_ui_file(
            id,
            Command::parse("document.save_smiles", json!({"path":output}))?,
        )
        .map_err(|error| error.message)?;
    assert_eq!(
        state(&shared)?["state"]["ui_file"]["request"]["phase"],
        "running"
    );
    let context = context(&shared, false)?;
    let outcome = prepare(&executor, &context, false, 2)
        .map_err(|error| error.message)?
        .complete_blocking(&Cancellation::default())
        .map_err(|error| error.message)?;
    assert_eq!((outcome.result.revision, outcome.result.undo_depth), (3, 1));
    assert!(outcome.result.file.dirty && outcome.result.file_operation.is_none());
    assert!(outcome.is_current(&*shared.lock().map_err(|_| "engine")?));
    let current = shared.lock().map_err(|_| "engine")?.content().clone();
    let receipt = save
        .complete_blocking(&Cancellation::default())
        .map_err(|error| error.message)?;
    assert_eq!(fs::read(output)?, SOURCE);
    let mut engine = shared.lock().map_err(|_| "engine")?;
    assert_eq!(engine.content(), &current);
    assert!(engine.state().file.dirty);
    assert_eq!(
        receipt
            .file_operation
            .ok_or("real old Save receipt")?
            .input_revision,
        2
    );
    assert!(!engine.finish_ui_file(id, false));
    assert!(
        !outcome.is_current(&engine),
        "Save association/baseline changes invalidate an earlier UI outcome"
    );
    assert!(!executor.has_active_work());
    assert_eq!(fs::read(input)?, SOURCE);
    Ok(())
}

#[test]
fn real_worker_commits_before_delivery_and_ui_thread_retirement_remains_a_separate_barrier()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("thread-owned.smi");
    let (shared, executor) = origin(&input, true)?;
    let original = shared.lock().map_err(|_| "engine")?.content().clone();
    let context = context(&shared, false)?;
    assert_eq!(
        shared.lock().map_err(|_| "engine")?.bond_tool().order,
        BondDrawOrder::Single
    );
    let work = prepare(&executor, &context, false, 2).map_err(|error| error.message)?;
    assert!(
        shared.try_lock().is_ok(),
        "preparation retained the Engine mutex"
    );
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (start_tx, start_rx) = mpsc::sync_channel(0);
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let (retire_tx, retire_rx) = mpsc::sync_channel(0);
    let wait = Duration::from_secs(10);
    let thread = std::thread::spawn(move || -> Result<(), String> {
        entered_tx.send(()).map_err(|error| error.to_string())?;
        start_rx
            .recv_timeout(wait)
            .map_err(|error| error.to_string())?;
        let result = work.complete_blocking(&Cancellation::default());
        result_tx.send(result).map_err(|error| error.to_string())?;
        retire_rx
            .recv_timeout(wait)
            .map_err(|error| error.to_string())?;
        Ok(())
    });
    entered_rx.recv_timeout(wait)?;
    assert!(executor.has_active_work());
    assert!(!thread.is_finished());
    start_tx.send(())?;
    let outcome = result_rx
        .recv_timeout(wait)?
        .map_err(|error| error.message)?;
    assert_eq!((outcome.result.revision, outcome.result.undo_depth), (3, 1));
    assert!(!executor.has_active_work());
    assert!(!thread.is_finished());
    {
        let engine = shared.lock().map_err(|_| "engine")?;
        assert!(outcome.is_current(&engine));
        let fragment = &engine.document()?.pages()[0].fragments()[0];
        assert_eq!(
            fragment
                .atoms()
                .iter()
                .map(|atom| atom.explicit_hydrogens)
                .collect::<Vec<_>>(),
            [Some(2), Some(2)]
        );
        assert_eq!(
            serde_json::to_value(fragment.bonds()[0].order)?,
            json!("double")
        );
    }
    retire_tx.send(())?;
    thread
        .join()
        .map_err(|_| "native order thread panicked")?
        .map_err(|error| error.to_string())?;
    dispatch(&shared, "history.undo", json!({}))?;
    assert_eq!(shared.lock().map_err(|_| "engine")?.content(), &original);
    assert!(!outcome.is_current(&*shared.lock().map_err(|_| "engine")?));
    dispatch(&shared, "history.redo", json!({}))?;
    assert!(
        !outcome.is_current(&*shared.lock().map_err(|_| "engine")?),
        "source ABA cannot revive a delivered proof"
    );
    assert_eq!(fs::read(input)?, SOURCE);
    Ok(())
}

#[test]
fn poisoned_engine_refuses_finish_and_new_work_while_releasing_the_registered_permit() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let (shared, executor) = origin(&directory.path("poison.smi"), true)?;
    let context = context(&shared, false)?;
    let before = shared.lock().map_err(|_| "engine")?.content().clone();
    let work = prepare(&executor, &context, false, 2).map_err(|error| error.message)?;
    let target = Arc::clone(&shared);
    let poisoned = std::thread::spawn(move || {
        if let Ok(_guard) = target.lock() {
            std::panic::resume_unwind(Box::new("intentional native records origin poison"));
        }
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
        prepare(&executor, &context, false, 2)
            .err()
            .ok_or("poisoned preparation")?
            .code,
        "engine_unavailable"
    );
    assert!(!executor.has_active_work());
    let engine = shared
        .lock()
        .err()
        .ok_or("real Engine poison")?
        .into_inner();
    assert_eq!(engine.content(), &before);
    assert_eq!(engine.state().revision, 2);
    Ok(())
}

#[test]
fn specialized_smiles_order_context_refuses_otherwise_editable_native_and_source_free_c2()
-> TestResult {
    let mut source_free = Engine::default();
    source_free.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":96,"begin_y_pt":96,"end_x_pt":160,"end_y_pt":96}),
    )?;
    let mut native = Engine::default();
    let native_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../io/tests/fixtures/bondline-v0.cdxml");
    native.dispatch("document.open_cdxml", json!({"path":native_path}))?;
    for mut engine in [source_free, native] {
        engine.set_ui_file_host(true);
        assert!(engine.stored_document()?.is_fully_mapped());
        let fragment = &engine.document()?.pages()[0].fragments()[0];
        assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (2, 1));
        let fragment_id = fragment.id().get();
        let bond_id = fragment.bonds()[0].id.get();
        let before = serde_json::to_value(engine.state())?;
        let owner = engine.content().clone();
        assert_eq!(
            UiSmilesRecordsContext::capture_order(&engine, 1, fragment_id, bond_id)
                .err()
                .ok_or("wrong-source SMILES context")?
                .code,
            "unsupported_content"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &owner);
        let mut params = order_params(engine.state().revision);
        params.fragment_id = fragment_id;
        params.bond_id = bond_id;
        let ordinary = engine.prepare(Command::BondSetOrder(params))?;
        assert!(
            !ordinary.is_deferred(),
            "the existing native edit path stays immediate"
        );
        let result = engine.finish(ordinary.complete()?)?;
        assert!(result.changed);
    }
    Ok(())
}

#[test]
fn running_save_returned_to_accepted_retry_invalidates_captured_and_prepared_order_work()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for already_prepared in [false, true] {
        let input = directory.path(&format!("retry-source-{already_prepared}.smi"));
        let output = directory.path(&format!("not-published-{already_prepared}.smiles"));
        let (shared, executor) = origin(&input, true)?;
        let id = {
            let mut engine = shared.lock().map_err(|_| "engine")?;
            engine.dispatch("ui.file", json!({"expected_revision":2,"action":"save_as"}))?;
            let id = engine.state().ui_file.request.ok_or("Save request")?.id;
            engine.claim_ui_file(id)?;
            id
        };
        let save = executor
            .prepare_ui_file(
                id,
                Command::parse("document.save_smiles", json!({"path":output}))?,
            )
            .map_err(|error| error.message)?;
        let running = state(&shared)?;
        assert_eq!(running["state"]["ui_file"]["request"]["phase"], "running");
        let context = context(&shared, false)?;
        assert!(context.is_current(&*shared.lock().map_err(|_| "engine")?));
        let order = if already_prepared {
            Some(prepare(&executor, &context, false, 2).map_err(|error| error.message)?)
        } else {
            None
        };
        let (owner, stamp) = {
            let engine = shared.lock().map_err(|_| "engine")?;
            (engine.content().clone(), engine.capture_stamp())
        };
        // This real native Save never starts I/O; its guarded retry retains the same request.
        drop(save);
        assert!(!output.exists());
        assert!(
            shared
                .lock()
                .map_err(|_| "engine")?
                .finish_ui_file(id, true)
        );
        let accepted = state(&shared)?;
        assert_eq!(accepted["state"]["ui_file"]["request"]["phase"], "accepted");
        assert_eq!(accepted["state"]["ui_file"]["request"]["id"], id);
        assert_eq!(
            accepted["state"]["ui_file"]["epoch"],
            running["state"]["ui_file"]["epoch"]
        );
        let error = if let Some(order) = order {
            order
                .complete_blocking(&Cancellation::default())
                .err()
                .ok_or("Accepted retry authorized a prepared order commit")?
        } else {
            prepare(&executor, &context, false, 2)
                .err()
                .ok_or("Accepted retry authorized new order work")?
        };
        assert_eq!(error.code, "ui_stale_request");
        assert_eq!(state(&shared)?, accepted);
        let engine = shared.lock().map_err(|_| "engine")?;
        assert!(!context.is_current(&engine));
        assert!(engine.capture_stamp_matches(&stamp));
        assert_eq!(engine.content(), &owner);
        assert_eq!((engine.state().revision, engine.state().undo_depth), (2, 0));
        assert!(!executor.has_active_work());
        assert!(!output.exists());
        assert_eq!(fs::read(input)?, SOURCE);
    }
    Ok(())
}

#[test]
fn a_completed_noop_order_proof_is_stale_when_running_save_returns_to_the_same_retry_form()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("noop-proof-retry.smi");
    let output = directory.path("noop-proof-not-published.smiles");
    let (shared, executor) = origin(&input, true)?;
    let id = {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        engine.dispatch("ui.file", json!({"expected_revision":2,"action":"save_as"}))?;
        let id = engine.state().ui_file.request.ok_or("Save request")?.id;
        engine.claim_ui_file(id)?;
        id
    };
    let save = executor
        .prepare_ui_file(
            id,
            Command::parse("document.save_smiles", json!({"path":output}))?,
        )
        .map_err(|error| error.message)?;
    let context = context(&shared, false)?;
    let (owner, stamp, file_stamp) = {
        let engine = shared.lock().map_err(|_| "engine")?;
        (
            engine.content().clone(),
            engine.capture_stamp(),
            engine.ui_file_context_stamp(),
        )
    };
    let mut params = order_params(2);
    params.order = BondDrawOrder::Single;
    let outcome = executor
        .prepare_ui_smiles_order(&context, params)
        .map_err(|error| error.message)?
        .complete_blocking(&Cancellation::default())
        .map_err(|error| error.message)?;
    assert!(!outcome.result.changed);
    assert_eq!((outcome.result.revision, outcome.result.undo_depth), (2, 0));
    assert!(outcome.is_current(&*shared.lock().map_err(|_| "engine")?));
    let mut expected = state(&shared)?;
    assert_eq!(expected["state"]["ui_file"]["request"]["phase"], "running");
    drop(save);
    assert!(!output.exists());
    assert!(
        shared
            .lock()
            .map_err(|_| "engine")?
            .finish_ui_file(id, true)
    );
    expected["state"]["ui_file"]["request"]["phase"] = json!("accepted");
    assert_eq!(
        state(&shared)?,
        expected,
        "only the same request's phase changed"
    );
    let engine = shared.lock().map_err(|_| "engine")?;
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(engine.ui_file_context_stamp_matches(&file_stamp));
    assert_eq!(engine.content(), &owner);
    assert!(
        !outcome.is_current(&engine),
        "a pending retry form blocks installation of an earlier order outcome"
    );
    assert!(!executor.has_active_work());
    assert!(!output.exists());
    assert_eq!(fs::read(input)?, SOURCE);
    Ok(())
}
