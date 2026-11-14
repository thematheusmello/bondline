use bondline_commands::{CloseAction, CloseDocument, CloseState, Command, Engine, NoParams};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn close(
    engine: &mut Engine,
    action: CloseAction,
) -> Result<bondline_commands::CommandResult, bondline_commands::Error> {
    engine.execute(Command::Close(CloseDocument {
        expected_revision: engine.state().revision,
        action,
    }))
}

fn edit(engine: &mut Engine, width: f64) -> TestResult {
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":width,"height_pt":600}),
    )?;
    Ok(())
}

fn authored_state(engine: &Engine) -> Result<Value, serde_json::Error> {
    let mut state = serde_json::to_value(engine.state())?;
    if let Some(object) = state.as_object_mut() {
        object.remove("close");
        object.remove("close_operation");
        object.remove("close_epoch");
    }
    Ok(state)
}

#[test]
fn refreshing_runtime_close_metadata_keeps_the_original_authored_result() -> TestResult {
    let mut engine = Engine::default();
    let old_edit = engine.prepare(Command::parse(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?)?;
    close(&mut engine, CloseAction::Discard)?;
    edit(&mut engine, 500.0)?;
    close(&mut engine, CloseAction::Cancel)?;
    let result = engine.finish(old_edit.complete()?)?;
    assert!(
        result.changed,
        "refreshing close state must retain the original operation's authored-change result"
    );
    assert_eq!(result.revision, 1);
    assert_eq!(
        result
            .document
            .as_ref()
            .ok_or("drawing result")?
            .pages()
            .first()
            .ok_or("result page")?
            .size()
            .width_pt(),
        400.0
    );
    assert_eq!(engine.state().revision, 2);
    assert_eq!(
        engine
            .document()?
            .pages()
            .first()
            .ok_or("current page")?
            .size()
            .width_pt(),
        500.0
    );
    assert_eq!(result.close, CloseState::Open);
    assert_eq!(result.close_epoch, 2);
    assert_eq!(result.close_operation, None);
    Ok(())
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "bondline-close-tests-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
    fn path(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self
            .0
            .join(name)
            .into_os_string()
            .into_string()
            .map_err(|_| "test path is not UTF-8")?)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn close_registry_has_closed_typed_parameters_and_explicit_lifecycle_results() -> TestResult {
    let info = bondline_commands::registry()
        .into_iter()
        .find(|info| info.name == "document.close")
        .ok_or("missing close command")?;
    assert!(!info.read_only);
    let schema = serde_json::to_value(info.input_schema)?;
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"], json!(["expected_revision"]));
    assert_eq!(schema["properties"]["action"]["default"], "request");
    let command = Command::parse("document.close", json!({"expected_revision":0}))?;
    assert!(matches!(
        command,
        Command::Close(CloseDocument {
            expected_revision: 0,
            action: CloseAction::Request
        })
    ));
    assert_eq!(command.name(), "document.close");
    for params in [
        json!({}),
        json!({"expected_revision":-1}),
        json!({"expected_revision":0.5}),
        json!({"expected_revision":0,"action":"approve"}),
        json!({"expected_revision":0,"action":null}),
        json!({"expected_revision":0,"extra":true}),
    ] {
        assert_eq!(
            Command::parse("document.close", params)
                .err()
                .ok_or("accepted invalid close params")?
                .info()
                .code,
            "invalid_params"
        );
    }
    assert_eq!(
        serde_json::to_value([
            CloseState::Open,
            CloseState::NeedsDecision,
            CloseState::Approved,
            CloseState::Committed
        ])?,
        json!(["open", "needs_decision", "approved", "committed"])
    );
    let mut engine = Engine::default();
    assert_eq!(engine.state().close, CloseState::Open);
    assert!(
        serde_json::to_value(engine.state())?
            .get("close_operation")
            .is_none()
    );
    let result = close(&mut engine, CloseAction::Request)?;
    assert_eq!(result.close, CloseState::Approved);
    assert_eq!(result.close_operation, Some(CloseState::Approved));
    assert!(!result.changed);
    assert!(engine.state().close_operation.is_none());
    Ok(())
}

#[test]
fn lifecycle_preserves_complete_source_history_and_saved_baseline() -> TestResult {
    let source = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
    let stored = bondline_io::StoredDocument::from_cdxml(
        source.as_slice().into(),
        bondline_model::PageSize::A4,
    )?;
    let mut engine = Engine::from_document(stored);
    let clean = engine.stored_document()?.clone();
    let initial = authored_state(&engine)?;
    assert!(!engine.complete_close());
    assert_eq!(
        close(&mut engine, CloseAction::Request)?.close,
        CloseState::Approved
    );
    assert_eq!(
        close(&mut engine, CloseAction::Cancel)?.close,
        CloseState::Open
    );
    assert_eq!(authored_state(&engine)?, initial);
    assert_eq!(engine.stored_document()?, &clean);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), source);
    edit(&mut engine, 400.0)?;
    let dirty = engine.stored_document()?.clone();
    let before = authored_state(&engine)?;
    assert_eq!(
        close(&mut engine, CloseAction::Request)?.close,
        CloseState::NeedsDecision
    );
    assert!(!engine.complete_close());
    assert_eq!(
        close(&mut engine, CloseAction::Discard)?.close,
        CloseState::Approved
    );
    assert_eq!(authored_state(&engine)?, before);
    assert_eq!(engine.stored_document()?, &dirty);
    close(&mut engine, CloseAction::Cancel)?;
    engine.execute(Command::Undo(NoParams {}))?;
    assert!(
        !engine.state().file.dirty,
        "close actions must retain the exact clean baseline"
    );
    assert_eq!(engine.stored_document()?, &clean);
    engine.execute(Command::Redo(NoParams {}))?;
    assert_eq!(engine.stored_document()?, &dirty);
    assert!(engine.state().file.dirty);
    Ok(())
}

#[test]
fn stale_actions_are_effect_free_and_edits_invalidate_approval_without_freezing() -> TestResult {
    let mut engine = Engine::default();
    close(&mut engine, CloseAction::Request)?;
    edit(&mut engine, 400.0)?;
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    let before = serde_json::to_value(engine.state())?;
    for action in [
        CloseAction::Request,
        CloseAction::Discard,
        CloseAction::Cancel,
    ] {
        let error = engine
            .execute(Command::Close(CloseDocument {
                expected_revision: 0,
                action,
            }))
            .err()
            .ok_or("accepted stale action")?;
        assert_eq!(error.info().code, "stale_revision");
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    close(&mut engine, CloseAction::Discard)?;
    engine.execute(Command::Undo(NoParams {}))?;
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    engine.execute(Command::Redo(NoParams {}))?;
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    close(&mut engine, CloseAction::Request)?;
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    close(&mut engine, CloseAction::Cancel)?;
    assert_eq!(engine.state().close, CloseState::Open);
    Ok(())
}

#[test]
fn host_commit_is_idempotent_and_all_later_commands_except_inspect_are_refused() -> TestResult {
    let mut engine = Engine::default();
    close(&mut engine, CloseAction::Request)?;
    let before = authored_state(&engine)?;
    assert!(engine.complete_close());
    assert!(engine.complete_close());
    assert_eq!(engine.state().close, CloseState::Committed);
    assert_eq!(authored_state(&engine)?, before);
    for (name, params) in [
        (
            "document.close",
            json!({"expected_revision":0,"action":"cancel"}),
        ),
        (
            "document.open_cdxml",
            json!({"path":"never-open-this.cdxml"}),
        ),
        (
            "document.save_cdxml",
            json!({"path":"never-save-this.cdxml"}),
        ),
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":400,"height_pt":600}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":0,"begin_y_pt":0,"end_x_pt":30,"end_y_pt":0}),
        ),
        (
            "bond.extend",
            json!({"expected_revision":0,"page_id":1,"fragment_id":1,"begin_atom_id":2,
                "end_x_pt":180,"end_y_pt":240,"hydrogen_policy":"rectify_terminal_carbon"}),
        ),
        ("history.undo", json!({})),
        ("history.redo", json!({})),
        (
            "render.capture",
            json!({"page_id":1,"width_px":32,"height_px":32,"backend":"cpu"}),
        ),
    ] {
        assert_eq!(
            engine
                .dispatch(name, params)
                .err()
                .ok_or("accepted post-commit command")?
                .info()
                .code,
            "close_committed",
            "{name}"
        );
        assert_eq!(authored_state(&engine)?, before);
        assert_eq!(engine.state().close, CloseState::Committed);
    }
    let result = engine.execute(Command::Inspect(NoParams {}))?;
    assert_eq!(result.close, CloseState::Committed);
    assert_eq!(result.close_operation, None);
    assert!(!result.changed);
    Ok(())
}

#[test]
fn stale_immediate_close_and_capture_results_report_current_lifecycle() -> TestResult {
    let mut engine = Engine::default();
    let request = engine.prepare(Command::Close(CloseDocument {
        expected_revision: 0,
        action: CloseAction::Request,
    }))?;
    let capture = engine.prepare(Command::parse(
        "render.capture",
        json!({"page_id":1,"width_px":32,"height_px":32,"backend":"cpu"}),
    )?)?;
    close(&mut engine, CloseAction::Cancel)?;
    let old_request = engine.finish(request.complete()?)?;
    assert_eq!(old_request.close, CloseState::Open);
    assert_eq!(old_request.close_operation, Some(CloseState::Open));
    let old_capture = engine.finish(capture.complete()?)?;
    assert_eq!(old_capture.close, CloseState::Open);
    assert!(old_capture.capture.is_some());
    assert!(old_capture.close_operation.is_none());
    let old_cancel = engine.prepare(Command::Close(CloseDocument {
        expected_revision: 0,
        action: CloseAction::Cancel,
    }))?;
    close(&mut engine, CloseAction::Request)?;
    assert!(engine.complete_close());
    let result = engine.finish(old_cancel.complete()?)?;
    assert_eq!(result.close, CloseState::Committed);
    assert_eq!(
        result.close_operation,
        Some(CloseState::Committed),
        "a late reply cannot falsely acknowledge stopping committed shutdown"
    );
    Ok(())
}

#[test]
fn saving_a_dirty_request_does_not_approve_it_until_a_renewed_current_request() -> TestResult {
    let directory = Directory::new()?;
    let path = directory.path("saved.cdxml")?;
    let mut engine = Engine::default();
    edit(&mut engine, 400.0)?;
    close(&mut engine, CloseAction::Request)?;
    let saved = engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    assert!(saved.file_operation.is_some());
    assert!(!saved.file.dirty);
    assert_eq!(saved.close, CloseState::NeedsDecision);
    assert!(!engine.complete_close());
    assert_eq!(
        close(&mut engine, CloseAction::Request)?.close,
        CloseState::Approved
    );
    assert!(engine.complete_close());
    Ok(())
}

#[test]
fn path_and_old_save_baseline_changes_permanently_invalidate_approval() -> TestResult {
    let directory = Directory::new()?;
    let path = directory.path("first.cdxml")?;
    let other = directory.path("second.cdxml")?;
    let mut engine = Engine::default();
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    engine.dispatch("document.save_cdxml", json!({"path":other}))?;
    let old = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":path,"overwrite":true}),
    )?)?;
    close(&mut engine, CloseAction::Request)?;
    assert_eq!(engine.state().close, CloseState::Approved);
    engine.finish(old.complete()?)?;
    assert_eq!(
        engine.state().close,
        CloseState::NeedsDecision,
        "restoring the old path must not resurrect approval"
    );
    engine.dispatch(
        "document.save_cdxml",
        json!({"path":other,"overwrite":true}),
    )?;
    assert_eq!(
        engine.state().close,
        CloseState::NeedsDecision,
        "restoring the approved path must not resurrect invalidated approval"
    );
    close(&mut engine, CloseAction::Cancel)?;
    edit(&mut engine, 400.0)?;
    engine.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
    engine.dispatch("history.undo", json!({}))?;
    let older_baseline = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":path,"overwrite":true}),
    )?)?;
    engine.dispatch("history.redo", json!({}))?;
    close(&mut engine, CloseAction::Request)?;
    assert_eq!(engine.state().close, CloseState::Approved);
    engine.finish(older_baseline.complete()?)?;
    assert!(engine.state().file.dirty);
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    engine.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
    assert!(!engine.state().file.dirty);
    assert_eq!(
        engine.state().close,
        CloseState::NeedsDecision,
        "a later clean baseline cannot resurrect invalidated approval"
    );
    Ok(())
}

#[test]
fn old_open_cannot_resurrect_committed_session_and_published_save_is_truthful() -> TestResult {
    let directory = Directory::new()?;
    let path = directory.path("existing.cdxml")?;
    let output = directory.path("published.cdxml")?;
    fs::write(
        &path,
        include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml"),
    )?;
    let mut engine = Engine::default();
    let old_open = engine
        .prepare(Command::parse("document.open_cdxml", json!({"path":path}))?)?
        .complete()?;
    edit(&mut engine, 400.0)?;
    let published = engine
        .prepare(Command::parse(
            "document.save_cdxml",
            json!({"path":output}),
        )?)?
        .complete()?;
    let bytes = fs::read(&output)?;
    close(&mut engine, CloseAction::Discard)?;
    let before = authored_state(&engine)?;
    let stored = engine.stored_document()?.clone();
    assert!(engine.complete_close());
    assert_eq!(
        engine
            .finish(old_open)
            .err()
            .ok_or("old open resurrected session")?
            .info()
            .code,
        "close_committed"
    );
    let receipt = engine.finish(published)?;
    assert_eq!(receipt.close, CloseState::Committed);
    assert!(!receipt.changed);
    let operation = receipt.file_operation.ok_or("lost published receipt")?;
    assert_eq!(operation.operation, "save");
    assert_eq!(operation.input_revision, 1);
    assert_eq!(operation.byte_length, bytes.len());
    assert_eq!(fs::read(output)?, bytes);
    assert_eq!(authored_state(&engine)?, before);
    assert_eq!(engine.stored_document()?, &stored);
    assert!(engine.state().file.dirty);
    assert!(engine.state().file.path.is_none());
    Ok(())
}

#[test]
fn every_successful_close_action_has_a_new_identity_at_the_same_authored_revision() -> TestResult {
    let mut engine = Engine::default();
    edit(&mut engine, 400.0)?;
    let revision = engine.state().revision;
    let first = close(&mut engine, CloseAction::Request)?;
    assert_eq!(first.close_epoch, 1);
    let cancel = close(&mut engine, CloseAction::Cancel)?;
    let second = close(&mut engine, CloseAction::Request)?;
    let repeated = close(&mut engine, CloseAction::Request)?;
    assert_eq!(
        [cancel.close_epoch, second.close_epoch, repeated.close_epoch],
        [2, 3, 4]
    );
    assert_eq!(first.revision, revision);
    assert_eq!(second.revision, revision);
    assert_eq!(first.close, second.close);
    assert_ne!(
        first.close_epoch, second.close_epoch,
        "old save continuation must distinguish Cancel followed by a new request"
    );
    let before = serde_json::to_value(engine.state())?;
    assert!(
        engine
            .execute(Command::Close(CloseDocument {
                expected_revision: 0,
                action: CloseAction::Cancel
            }))
            .is_err()
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn actual_baseline_changes_invalidate_discard_but_identical_saves_preserve_approval() -> TestResult
{
    let directory = Directory::new()?;
    let path = directory.path("baseline.cdxml")?;
    let mut engine = Engine::default();
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    close(&mut engine, CloseAction::Request)?;
    let epoch = engine.state().close_epoch;
    engine.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
    assert_eq!(
        engine.state().close,
        CloseState::Approved,
        "a logically identical baseline/path is not invalidation"
    );
    assert_eq!(engine.state().close_epoch, epoch);
    edit(&mut engine, 400.0)?;
    let save = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":path,"overwrite":true}),
    )?)?;
    close(&mut engine, CloseAction::Discard)?;
    engine.finish(save.complete()?)?;
    assert!(!engine.state().file.dirty);
    assert_eq!(
        engine.state().close,
        CloseState::NeedsDecision,
        "even a discard approval must be renewed when its saved baseline changes"
    );
    assert!(!engine.complete_close());
    close(&mut engine, CloseAction::Request)?;
    assert!(engine.complete_close());
    Ok(())
}
