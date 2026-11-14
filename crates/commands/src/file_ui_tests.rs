use crate::{Command, Engine, UiFile, UiFileAction, UiFilePhase, UiFilePlan};
use serde_json::json;
use std::sync::Arc;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn request(engine: &mut Engine, action: UiFileAction) -> Result<u64, crate::Error> {
    let result = engine.execute(Command::UiFile(UiFile {
        expected_revision: engine.state().revision,
        action,
        request_id: None,
        close_epoch: None,
    }))?;
    result.ui_file_operation.ok_or(crate::Error::UiStaleRequest)
}

#[test]
fn no_host_cannot_claim_a_dialog_and_requests_preserve_all_document_storage() -> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let error = request(&mut engine, UiFileAction::Open)
        .err()
        .ok_or("host absent accepted")?;
    assert_eq!(error.info().code, "ui_unavailable");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let source = Arc::clone(&engine.document);
    let saved = Arc::clone(&engine.saved);
    engine.set_ui_file_host(true);
    let id = request(&mut engine, UiFileAction::Open)?;
    assert_eq!(id, 1);
    assert!(Arc::ptr_eq(&engine.document, &source));
    assert!(Arc::ptr_eq(&engine.saved, &saved));
    assert_eq!(engine.state().revision, 0);
    assert_eq!(engine.state().undo_depth, 0);
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("request")?
            .phase,
        UiFilePhase::Requested
    );
    assert!(matches!(engine.claim_ui_file(id)?, UiFilePlan::OpenPicker));
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("request")?
            .phase,
        UiFilePhase::Accepted
    );
    Ok(())
}

#[test]
fn cancellation_and_new_request_never_reuse_an_identity_at_the_same_revision() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let first = request(&mut engine, UiFileAction::SaveAs)?;
    engine.claim_ui_file(first)?;
    let cancelled = engine.execute(Command::UiFile(UiFile {
        expected_revision: 0,
        action: UiFileAction::Cancel,
        request_id: Some(first),
        close_epoch: None,
    }))?;
    assert_eq!(cancelled.ui_file_operation, Some(2));
    assert!(cancelled.ui_file.request.is_none());
    let next = request(&mut engine, UiFileAction::Open)?;
    assert_eq!(next, 3);
    assert_eq!(engine.state().revision, 0);
    assert_eq!(
        engine
            .claim_ui_file(first)
            .err()
            .ok_or("stale claim")?
            .info()
            .code,
        "ui_stale_request"
    );
    assert!(!engine.finish_ui_file(first, false));
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("new request")?
            .id,
        next
    );
    Ok(())
}

#[test]
fn stale_selection_cannot_discard_a_newer_edit_and_noop_edits_keep_the_picker() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let first = request(&mut engine, UiFileAction::Open)?;
    engine.claim_ui_file(first)?;
    let size = engine.document()?.pages().first().ok_or("page")?.size();
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":size.width_pt(),"height_pt":size.height_pt()}),
    )?;
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("noop removed request")?
            .id,
        first
    );
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    assert!(engine.state().ui_file.request.is_none());
    let before = serde_json::to_value(engine.state())?;
    let error = engine
        .prepare_ui_file(
            first,
            Command::parse(
                "document.open_cdxml",
                json!({"path":"old.cdxml","discard_unsaved":true}),
            )?,
        )
        .err()
        .ok_or("old picker discarded edit")?;
    assert_eq!(error.info().code, "ui_stale_request");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn selection_is_validated_before_running_and_epoch_exhaustion_is_atomic() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let id = request(&mut engine, UiFileAction::SaveAs)?;
    engine.claim_ui_file(id)?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .prepare_ui_file(id, Command::parse("history.undo", json!({}))?)
            .err()
            .ok_or("wrong command accepted")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let prepared = engine.prepare_ui_file(
        id,
        Command::parse("document.save_cdxml", json!({"path":"chosen.cdxml"}))?,
    )?;
    assert!(prepared.is_deferred());
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("running request")?
            .phase,
        UiFilePhase::Running
    );
    assert_eq!(
        engine
            .execute(Command::UiFile(UiFile {
                expected_revision: 0,
                action: UiFileAction::Cancel,
                request_id: Some(id),
                close_epoch: None
            }))
            .err()
            .ok_or("running cancel claimed rollback")?
            .info()
            .code,
        "ui_busy"
    );
    assert!(engine.finish_ui_file(id, true));
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("retry request")?
            .phase,
        UiFilePhase::Accepted
    );
    engine.file_ui.epoch = u64::MAX;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .execute(Command::UiFile(UiFile {
                expected_revision: 0,
                action: UiFileAction::Cancel,
                request_id: Some(id),
                close_epoch: None
            }))
            .err()
            .ok_or("epoch wrapped")?
            .info()
            .code,
        "ui_epoch_exhausted"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn named_save_cannot_replace_its_path_or_lower_its_overwrite_guard() -> TestResult {
    let mut engine = Engine {
        file_path: Some("/original.cdxml".to_owned()),
        ..Engine::default()
    };
    engine.set_ui_file_host(true);
    let id = request(&mut engine, UiFileAction::Save)?;
    let UiFilePlan::SaveExisting { format, params } = engine.claim_ui_file(id)? else {
        return Err("named Save opened an unguarded picker".into());
    };
    assert_eq!(params.path, "/original.cdxml");
    assert_eq!(format, crate::FileFormat::Cdxml);
    assert!(params.overwrite);
    for actual in [
        json!({"path":"/other.cdxml","overwrite":true}),
        json!({"path":"/original.cdxml","overwrite":false}),
    ] {
        let before = serde_json::to_value(engine.state())?;
        let error = engine
            .prepare_ui_file(id, Command::parse("document.save_cdxml", actual)?)
            .err()
            .ok_or("named Save guard changed")?;
        assert_eq!(error.info().code, "invalid_params");
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    assert_eq!(
        engine
            .prepare_ui_file(
                id,
                Command::parse(
                    "document.save_mol",
                    json!({"path":"/original.cdxml","overwrite":true})
                )?,
            )
            .err()
            .ok_or("named Save changed associated format")?
            .info()
            .code,
        "invalid_params"
    );
    assert!(
        engine
            .prepare_ui_file(id, Command::SaveCdxml(params))?
            .is_deferred()
    );
    assert!(
        !engine.finish_ui_file(id, false),
        "dropping prepared work cannot acknowledge a completed publication"
    );
    assert!(engine.state().ui_file.request.is_none());
    Ok(())
}

#[test]
fn close_specific_picker_cancel_only_cancels_its_current_close_epoch() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    let closing = engine.dispatch("document.close", json!({"expected_revision":1}))?;
    assert_eq!(closing.close, crate::CloseState::NeedsDecision);
    let requested = engine.execute(Command::UiFile(UiFile {
        expected_revision: 1,
        action: UiFileAction::Save,
        request_id: None,
        close_epoch: Some(closing.close_epoch),
    }))?;
    let id = requested.ui_file_operation.ok_or("request identity")?;
    engine.claim_ui_file(id)?;
    engine.execute(Command::UiFile(UiFile {
        expected_revision: 1,
        action: UiFileAction::Cancel,
        request_id: Some(id),
        close_epoch: None,
    }))?;
    assert_eq!(engine.state().close, crate::CloseState::Open);
    assert_eq!(engine.state().close_epoch, closing.close_epoch + 1);
    assert!(engine.state().file.dirty);
    let closing = engine.dispatch("document.close", json!({"expected_revision":1}))?;
    let newer = engine.execute(Command::UiFile(UiFile {
        expected_revision: 1,
        action: UiFileAction::Save,
        request_id: None,
        close_epoch: Some(closing.close_epoch),
    }))?;
    let newer_id = newer.ui_file_operation.ok_or("new request identity")?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .execute(Command::UiFile(UiFile {
                expected_revision: 1,
                action: UiFileAction::Cancel,
                request_id: Some(id),
                close_epoch: None
            }))
            .err()
            .ok_or("old cancel dismissed newer close")?
            .info()
            .code,
        "ui_stale_request"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("newer picker")?
            .id,
        newer_id
    );
    Ok(())
}

#[test]
fn ordinary_close_invalidates_picker_even_when_cancel_returns_to_the_same_open_state() -> TestResult
{
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let id = request(&mut engine, UiFileAction::Open)?;
    engine.claim_ui_file(id)?;
    engine.dispatch("document.close", json!({"expected_revision":0}))?;
    assert!(engine.state().ui_file.request.is_none());
    engine.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"cancel"}),
    )?;
    assert_eq!(engine.state().close, crate::CloseState::Open);
    assert_eq!(
        engine
            .claim_ui_file(id)
            .err()
            .ok_or("close ABA revived chooser")?
            .info()
            .code,
        "ui_stale_request"
    );
    Ok(())
}

#[test]
fn running_work_can_finish_but_edit_undo_aba_cannot_reopen_its_old_picker() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let id = request(&mut engine, UiFileAction::SaveAs)?;
    engine.claim_ui_file(id)?;
    let _prepared = engine.prepare_ui_file(
        id,
        Command::parse("document.save_cdxml", json!({"path":"chosen.cdxml"}))?,
    )?;
    let source = Arc::clone(&engine.document);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    assert!(Arc::ptr_eq(&engine.document, &source));
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("running identity lost")?
            .phase,
        UiFilePhase::Running
    );
    assert!(!engine.finish_ui_file(id, true));
    assert!(engine.state().ui_file.request.is_none());
    Ok(())
}

#[test]
fn close_counter_exhaustion_does_not_partially_cancel_its_file_picker() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    engine.close_policy.epoch = u64::MAX - 1;
    engine.dispatch("document.close", json!({"expected_revision":1}))?;
    let id = engine
        .execute(Command::UiFile(UiFile {
            expected_revision: 1,
            action: UiFileAction::Save,
            request_id: None,
            close_epoch: Some(u64::MAX),
        }))?
        .ui_file_operation
        .ok_or("request")?;
    let before = serde_json::to_value(engine.state())?;
    let error = engine
        .execute(Command::UiFile(UiFile {
            expected_revision: 1,
            action: UiFileAction::Cancel,
            request_id: Some(id),
            close_epoch: None,
        }))
        .err()
        .ok_or("partial cancel")?;
    assert_eq!(error.info().code, "close_epoch_exhausted");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn opaque_source_and_history_survive_presentation_and_host_detachment() -> TestResult {
    const SOURCE: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");
    let stored =
        bondline_io::StoredDocument::from_cdxml(Arc::from(SOURCE), bondline_model::PageSize::A4)?;
    let mut engine = Engine::from_document(stored);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    let source = Arc::clone(&engine.document);
    let saved = Arc::clone(&engine.saved);
    let redo = engine.redo.clone();
    engine.set_ui_file_host(true);
    let id = request(&mut engine, UiFileAction::SaveAs)?;
    engine.claim_ui_file(id)?;
    engine.set_ui_file_host(false);
    engine.set_ui_file_host(true);
    assert!(engine.state().ui_file.request.is_none());
    assert_eq!(
        engine
            .claim_ui_file(id)
            .err()
            .ok_or("detached chooser revived")?
            .info()
            .code,
        "ui_stale_request"
    );
    assert!(Arc::ptr_eq(&engine.document, &source));
    assert!(Arc::ptr_eq(&engine.saved, &saved));
    assert!(
        redo.iter()
            .zip(&engine.redo)
            .all(|(a, b)| Arc::ptr_eq(a, b))
    );
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), SOURCE);
    Ok(())
}

#[test]
fn controls_use_typed_registry_bindings_and_contextual_enablement_without_a_host_claim()
-> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let result = engine.dispatch("ui.controls", json!({}))?;
    assert!(!result.changed);
    let rows = result.controls.ok_or("controls missing")?;
    assert_eq!(rows.len(), 12);
    assert!(
        rows.iter()
            .filter(|control| control.menu == "File")
            .all(|control| {
                !control.enabled && control.disabled_reason == Some("ui_unavailable")
            }),
        "a headless catalog must not enable native File-window controls"
    );
    for control in &rows {
        let serialized = serde_json::to_value(&control.command)?;
        let name = serialized["command"].as_str().ok_or("command name")?;
        crate::Command::parse(name, serialized["params"].clone())?;
        assert!(crate::registry().iter().any(|info| info.name == name));
    }
    let open = rows
        .iter()
        .find(|control| control.id == "file.open")
        .ok_or("Open")?;
    assert!(!open.enabled);
    assert_eq!(open.disabled_reason, Some("ui_unavailable"));
    assert_eq!(open.label, "Open…");
    assert_eq!(
        open.shortcut.ok_or("Open shortcut")?.key,
        crate::ControlKey::O
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    engine.set_ui_file_host(true);
    let open = engine
        .controls()
        .into_iter()
        .find(|control| control.id == "file.open")
        .ok_or("Open")?;
    assert!(open.enabled);
    engine.execute(open.command)?;
    assert!(
        engine
            .controls()
            .iter()
            .filter(|control| control.menu == "File")
            .all(|control| !control.enabled)
    );
    let cancel = engine
        .controls()
        .into_iter()
        .find(|control| control.id == "file.cancel")
        .ok_or("Cancel binding")?;
    assert!(cancel.enabled);
    engine.execute(cancel.command)?;
    assert!(engine.state().ui_file.request.is_none());
    Ok(())
}

#[test]
fn ui_parameters_are_closed_and_rejected_commands_preserve_request_identity() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    for params in [
        json!({"expected_revision":0,"action":"open","extra":true}),
        json!({"expected_revision":0,"action":"cancel"}),
        json!({"expected_revision":0,"action":"open","request_id":7}),
        json!({"expected_revision":0,"action":"save_as","close_epoch":0}),
    ] {
        let before = serde_json::to_value(engine.state())?;
        assert_eq!(
            engine
                .dispatch("ui.file", params)
                .err()
                .ok_or("invalid UI params accepted")?
                .info()
                .code,
            "invalid_params"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    let stale = engine
        .dispatch("ui.file", json!({"expected_revision":1,"action":"open"}))
        .err()
        .ok_or("stale revision accepted")?;
    assert_eq!(stale.info().code, "stale_revision");
    assert_eq!(engine.state().ui_file.epoch, 0);
    Ok(())
}

struct Directory(std::path::PathBuf);
impl Directory {
    fn new() -> Result<Self, std::io::Error> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "bondline-file-ui-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)?;
        Ok(Self(path))
    }
    fn path(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.0
            .join(name)
            .into_os_string()
            .into_string()
            .map_err(|_| "test path is not UTF-8".into())
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn same_save_keeps_request_but_path_and_saved_baseline_aba_never_revive_it() -> TestResult {
    let directory = Directory::new()?;
    let first_path = directory.path("first.cdxml")?;
    let second_path = directory.path("second.cdxml")?;
    let save = |path: &str| {
        Command::SaveCdxml(crate::SaveCdxml {
            path: path.to_owned(),
            overwrite: true,
        })
    };
    let mut engine = Engine::default();
    engine.execute(save(&first_path))?;
    engine.set_ui_file_host(true);
    let id = request(&mut engine, UiFileAction::Save)?;
    engine.claim_ui_file(id)?;
    engine.execute(save(&first_path))?;
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("same save invalidated chooser")?
            .id,
        id
    );
    engine.execute(save(&second_path))?;
    engine.execute(save(&first_path))?;
    assert_eq!(engine.state().revision, 0);
    assert!(engine.state().ui_file.request.is_none());
    assert_eq!(
        engine
            .claim_ui_file(id)
            .err()
            .ok_or("path ABA revived chooser")?
            .info()
            .code,
        "ui_stale_request"
    );

    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":500,"height_pt":600}),
    )?;
    engine.execute(save(&first_path))?;
    let baseline = Arc::clone(&engine.saved);
    engine.dispatch("history.undo", json!({}))?;
    let old_save = engine.prepare(save(&first_path))?;
    engine.dispatch("history.redo", json!({}))?;
    let id = request(&mut engine, UiFileAction::SaveAs)?;
    engine.claim_ui_file(id)?;
    let path = engine.file_path.clone();
    let stale_receipt = engine.finish(old_save.complete()?)?;
    assert_eq!(
        stale_receipt
            .file_operation
            .ok_or("old save receipt")?
            .input_revision,
        3
    );
    assert!(engine.state().file.dirty);
    assert_eq!(engine.file_path, path);
    assert!(engine.state().ui_file.request.is_none());
    engine.execute(save(&first_path))?;
    assert_eq!(engine.saved, baseline);
    assert_eq!(engine.state().revision, 4);
    assert_eq!(
        engine
            .claim_ui_file(id)
            .err()
            .ok_or("baseline ABA revived chooser")?
            .info()
            .code,
        "ui_stale_request"
    );
    Ok(())
}

#[test]
fn old_presentation_response_refreshes_current_request_without_replaying_its_operation()
-> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let old = engine.prepare(Command::UiFile(UiFile {
        expected_revision: 0,
        action: UiFileAction::Open,
        request_id: None,
        close_epoch: None,
    }))?;
    let id = engine
        .state()
        .ui_file
        .request
        .as_ref()
        .ok_or("old request")?
        .id;
    engine.execute(Command::UiFile(UiFile {
        expected_revision: 0,
        action: UiFileAction::Cancel,
        request_id: Some(id),
        close_epoch: None,
    }))?;
    let current = request(&mut engine, UiFileAction::SaveAs)?;
    let response = engine.finish(old.complete()?)?;
    assert!(!response.changed);
    assert_eq!(response.ui_file_operation, Some(id));
    assert_eq!(response.ui_file.epoch, 3);
    assert_eq!(
        response.ui_file.request.ok_or("current request")?.id,
        current
    );
    Ok(())
}

#[test]
fn old_close_specific_save_reports_publication_without_approving_a_new_close_request() -> TestResult
{
    let directory = Directory::new()?;
    let path = directory.path("close.cdxml")?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    let closing = engine.dispatch("document.close", json!({"expected_revision":1}))?;
    let id = engine
        .execute(Command::UiFile(UiFile {
            expected_revision: 1,
            action: UiFileAction::Save,
            request_id: None,
            close_epoch: Some(closing.close_epoch),
        }))?
        .ui_file_operation
        .ok_or("file request")?;
    engine.claim_ui_file(id)?;
    let save = engine.prepare_ui_file(
        id,
        Command::SaveCdxml(crate::SaveCdxml {
            path: path.clone(),
            overwrite: false,
        }),
    )?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"cancel"}),
    )?;
    let later = engine.dispatch("document.close", json!({"expected_revision":1}))?;
    let result = engine.finish(save.complete()?)?;
    assert_eq!(
        result
            .file_operation
            .ok_or("published save receipt")?
            .input_revision,
        1
    );
    assert!(std::fs::metadata(path)?.len() > 0);
    assert!(!result.file.dirty);
    assert_eq!(result.close, crate::CloseState::NeedsDecision);
    assert_eq!(result.close_epoch, later.close_epoch);
    assert_ne!(result.close_epoch, closing.close_epoch);
    assert!(!engine.complete_close());
    assert!(!engine.finish_ui_file(id, true));
    assert!(engine.state().ui_file.request.is_none());
    Ok(())
}
