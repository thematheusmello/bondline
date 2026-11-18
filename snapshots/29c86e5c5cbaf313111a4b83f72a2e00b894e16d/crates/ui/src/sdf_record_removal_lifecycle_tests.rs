//! Finite confirmation and real registered-removal lifecycle acceptance.
use super::*;
use serde_json::json;
use std::{
    fs,
    sync::{Arc, Mutex, mpsc::SyncSender},
    time::Instant,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse the unchanged atomic temporary-directory helper"
)]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

#[test]
fn removal_confirmation_rejects_old_nonce_and_close_aba_without_changing_source() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("confirmation-source.sdf");
    fs::write(&path, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    let before = serde_json::to_value(engine.state())?;
    let context = egui::Context::default();
    let mut editor = SdfEditor::default();
    assert!(editor.confirm_remove(&engine, &context, 1));
    let first = editor
        .removal
        .as_ref()
        .ok_or("first confirmation")?
        .sequence;
    assert!(editor.blocks_close() && editor.holds_files());
    assert!(!editor.is_busy());
    editor.actions.push((0, Action::KeepRecord(first)));
    editor.replay(0, &engine, &context);
    assert!(editor.removal.is_none() && editor.pending.is_none());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    editor.begin_frame();
    assert!(editor.confirm_remove(&engine, &context, 2));
    let second = editor
        .removal
        .as_ref()
        .ok_or("second confirmation")?
        .sequence;
    assert!(second > first);
    editor.actions.push((0, Action::ConfirmRemove(first)));
    editor.replay(0, &engine, &context);
    assert!(editor.pending.is_none());
    assert_eq!(
        editor
            .removal
            .as_ref()
            .ok_or("live second confirmation")?
            .sequence,
        second
    );
    for action in ["request", "cancel"] {
        engine.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":action}),
        )?;
    }
    editor.sync(Some(&engine), &context);
    assert!(editor.removal.is_none() && editor.pending.is_none());
    assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
    assert_eq!(engine.state().revision, 1);
    assert_eq!(engine.state().undo_depth, 0);
    assert_eq!(fs::read(path)?, SOURCE);
    Ok(())
}

#[test]
fn sole_record_cannot_open_confirmation_or_reserve_a_mutation() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("single-record.sdf");
    let single = SOURCE.get(462..).ok_or("literal sole survivor")?;
    assert_eq!(single.len(), 315);
    fs::write(&path, single)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    assert_eq!(
        engine.state().collection.ok_or("collection")?.record_count,
        1
    );
    let before = serde_json::to_value(engine.state())?;
    let mut editor = SdfEditor::default();
    assert!(!editor.confirm_remove(&engine, &egui::Context::default(), 1));
    assert!(editor.removal.is_none() && editor.pending.is_none());
    assert!(!editor.is_busy());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content().sdf()?.as_bytes(), single);
    Ok(())
}

const WAIT: Duration = Duration::from_secs(5);

struct RemovalGate {
    editor: SdfEditor,
    engine: bondline_mcp::SharedEngine,
    executor: CommandExecutor,
    context: egui::Context,
    release: Option<SyncSender<()>>,
    input: std::path::PathBuf,
}

impl RemovalGate {
    fn published(directory: &support::FileTestDirectory) -> TestResult<Self> {
        let input = directory.path("finite-record-removal-化学.sdf");
        fs::write(&input, SOURCE)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_sdf", json!({"path":input}))?;
        let engine = Arc::new(Mutex::new(engine));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let context = egui::Context::default();
        let mut editor = SdfEditor::default();
        {
            let guard = engine.lock().map_err(|_| "engine")?;
            assert!(editor.confirm_remove(&guard, &context, 1));
            let sequence = editor.removal.as_ref().ok_or("confirmation")?.sequence;
            editor.actions.push((0, Action::ConfirmRemove(sequence)));
            editor.replay(0, &guard, &context);
            assert!(editor.is_busy() && !editor.active());
            let effect = editor.take_effect().ok_or("pending invalidation")?;
            assert!(effect.records_invalidated && effect.refresh.is_none());
            assert!(editor.intercept(Target::Control("file.save"), &guard, &context));
        }
        let MutationInput::Remove(input_work) =
            editor.pending.take().ok_or("real removal input")?
        else {
            return Err("expected removal input".into());
        };
        let work = executor
            .prepare_ui_sdf(
                &input_work.stamp,
                input_work.close_epoch,
                Command::SdfRecordRemove(input_work.params),
            )
            .map_err(|error| error.message)?;
        let owner = Arc::clone(&engine);
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let (sender, result) = mpsc::sync_channel(1);
        let (published, publication) = mpsc::sync_channel(1);
        let (release, wait) = mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            let output = work.complete_blocking(&worker_cancel).map(|result| {
                let stamp = owner.lock().ok().and_then(|engine| {
                    (engine.state().revision == result.revision).then(|| engine.capture_stamp())
                });
                Applied { result, stamp }
            });
            let _ = sender.send(output);
            let _ = published.send(());
            let _ = wait.recv_timeout(WAIT);
        });
        editor.job = Some(ApplyJob {
            kind: MutationKind::Remove,
            cancel,
            result,
            worker,
        });
        let gate = Self {
            editor,
            engine,
            executor,
            context,
            release: Some(release),
            input,
        };
        publication.recv_timeout(WAIT)?;
        assert!(!gate.executor.has_active_work());
        assert!(gate.editor.is_busy() && gate.editor.blocks_close());
        assert_eq!(
            gate.engine
                .lock()
                .map_err(|_| "engine")?
                .content()
                .sdf()?
                .as_bytes(),
            SOURCE.get(462..).ok_or("literal survivor")?
        );
        assert_eq!(fs::read(&gate.input)?, SOURCE);
        Ok(gate)
    }

    fn release_and_retire(&mut self, poisoned: bool) -> TestResult {
        self.release.take().ok_or("already released")?.send(())?;
        let deadline = Instant::now() + WAIT;
        while self.editor.job.is_some() {
            if poisoned {
                self.editor.sync(None, &self.context);
            } else {
                let guard = self.engine.lock().map_err(|_| "engine")?;
                self.editor.sync(Some(&guard), &self.context);
            }
            if Instant::now() >= deadline {
                return Err("real removal worker did not retire".into());
            }
            std::thread::yield_now();
        }
        Ok(())
    }
}

impl Drop for RemovalGate {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
        if let Some(job) = self.editor.job.take() {
            let _ = job.worker.join();
        }
    }
}

#[test]
fn published_removal_waits_for_retirement_and_late_cancel_cannot_hide_the_commit() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut gate = RemovalGate::published(&directory)?;
    {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        gate.editor.sync(Some(&guard), &gate.context);
        assert!(gate.editor.take_effect().is_none());
        for action in ["request", "discard"] {
            guard.dispatch(
                "document.close",
                json!({"expected_revision":2,"action":action}),
            )?;
        }
        assert_eq!(guard.state().close, CloseState::Approved);
        assert!(
            gate.editor.blocks_close(),
            "publication is not UI retirement"
        );
        gate.editor.actions.push((0, Action::CancelWork));
        gate.editor.replay(0, &guard, &gate.context);
        assert!(
            gate.editor
                .job
                .as_ref()
                .is_some_and(|job| job.cancel.is_cancelled())
        );
        assert!(gate.editor.is_busy() && gate.editor.after_apply.is_none());
    }
    gate.release_and_retire(false)?;
    let effect = gate.editor.take_effect().ok_or("truthful removal effect")?;
    assert!(effect.records_invalidated && effect.refresh.is_none() && effect.target.is_none());
    assert!(
        effect
            .warning
            .as_ref()
            .is_some_and(|warning| warning.contains("removed before cancellation"))
    );
    let guard = gate.engine.lock().map_err(|_| "engine")?;
    assert_eq!(
        guard.content().sdf()?.as_bytes(),
        SOURCE.get(462..).ok_or("literal survivor")?
    );
    assert_eq!(guard.state().revision, 2);
    assert_eq!(guard.state().undo_depth, 1);
    assert!(!gate.editor.blocks_close() && !gate.executor.has_active_work());
    assert_eq!(fs::read(&gate.input)?, SOURCE);
    Ok(())
}

#[test]
fn historical_removal_never_replays_save_or_installs_deleted_ordinals_in_replacement() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let mut gate = RemovalGate::published(&directory)?;
    let replacement = directory.path("identical-source-new-session.sdf");
    fs::write(&replacement, SOURCE)?;
    {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        guard.dispatch(
            "document.open_sdf",
            json!({"path":replacement,"discard_unsaved":true}),
        )?;
    }
    gate.release_and_retire(false)?;
    assert!(gate.editor.after_apply.is_none() && gate.editor.take_effect().is_none());
    assert!(
        gate.editor
            .error
            .as_ref()
            .is_some_and(|error| error.contains("removed from an earlier document"))
    );
    let guard = gate.engine.lock().map_err(|_| "engine")?;
    assert_eq!(guard.content().sdf()?.as_bytes(), SOURCE);
    assert_eq!(guard.state().revision, 3);
    assert_eq!(guard.state().undo_depth, 0);
    assert!(!guard.state().file.dirty);
    assert_eq!(fs::read(replacement)?, SOURCE);
    assert_eq!(fs::read(&gate.input)?, SOURCE);
    Ok(())
}

#[test]
fn poisoned_engine_still_retires_the_real_removal_and_reports_its_committed_outcome() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let mut gate = RemovalGate::published(&directory)?;
    let owner = Arc::clone(&gate.engine);
    let poisoned = std::thread::spawn(move || {
        let Ok(_held) = owner.lock() else { return };
        std::panic::resume_unwind(Box::new("intentional removal lifecycle poison"));
    })
    .join();
    assert!(poisoned.is_err() && gate.engine.lock().is_err());
    gate.editor.sync(None, &gate.context);
    assert!(gate.editor.is_busy());
    gate.release_and_retire(true)?;
    assert!(!gate.editor.is_busy() && !gate.executor.has_active_work());
    assert!(gate.editor.after_apply.is_none() && gate.editor.take_effect().is_none());
    assert!(
        gate.editor
            .error
            .as_ref()
            .is_some_and(|error| error.contains("record was removed"))
    );
    let guard = gate
        .engine
        .lock()
        .err()
        .ok_or("poison retained")?
        .into_inner();
    assert_eq!(
        guard.content().sdf()?.as_bytes(),
        SOURCE.get(462..).ok_or("literal survivor")?
    );
    assert_eq!(guard.state().revision, 2);
    assert_eq!(fs::read(&gate.input)?, SOURCE);
    Ok(())
}
