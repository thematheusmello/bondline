//! Finite real registered Apply gates; no native input or synthetic query result.
use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::SyncSender},
    time::Instant,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse the same original bounded temporary-directory helper"
)]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ORIGINAL: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
const REPLACEMENT: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
const DRAFT: &str = "reviewed\nline two";
const WAIT: Duration = Duration::from_secs(5);

fn edited() -> TestResult<Vec<u8>> {
    Ok([
        ORIGINAL.get(..378).ok_or("literal prefix")?,
        b"reviewed\nline two\n",
        ORIGINAL.get(385..).ok_or("literal suffix")?,
    ]
    .concat())
}

struct ApplyGate {
    editor: SdfEditor,
    engine: bondline_mcp::SharedEngine,
    executor: CommandExecutor,
    context: egui::Context,
    release: Option<SyncSender<()>>,
    input: PathBuf,
}

impl ApplyGate {
    fn published(directory: &support::FileTestDirectory) -> TestResult<Self> {
        let input = directory.path("registered-apply-original-化学.sdf");
        fs::write(&input, ORIGINAL)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_sdf", json!({"path":input}))?;
        let engine = Arc::new(Mutex::new(engine));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let context = egui::Context::default();
        let mut editor = SdfEditor::default();
        {
            let guard = engine.lock().map_err(|_| "engine")?;
            editor.begin(&guard, &context, 1, 2, 0, "second");
            let draft = editor.draft.as_mut().ok_or("real editor draft")?;
            let (_, outcome) = draft.text.edit(|buffer| buffer.replace_with(DRAFT));
            assert!(outcome.changed && outcome.rejection.is_none());
            assert!(editor.apply(&guard));
            assert!(editor.intercept(Target::Control("file.save"), &guard, &context));
            assert!(editor.after_apply.is_some());
        }
        let input_work = editor.pending.take().ok_or("pending real Apply input")?;
        let work = executor
            .prepare_ui_sdf(
                &input_work.stamp,
                input_work.close_epoch,
                Command::SdfFieldSet(input_work.params),
            )
            .map_err(|error| error.message)?;
        assert!(executor.has_active_work());
        let owner = Arc::clone(executor.engine());
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
            cancel,
            result,
            worker,
        });
        // Construct cleanup ownership before any fallible wait or assertion.
        let gate = Self {
            editor,
            engine,
            executor,
            context,
            release: Some(release),
            input,
        };
        publication.recv_timeout(WAIT)?;
        assert!(
            gate.editor
                .job
                .as_ref()
                .is_some_and(|job| !job.worker.is_finished()),
            "the real mutation result must be published while its worker is still gated"
        );
        assert!(!gate.executor.has_active_work());
        assert_eq!(gate.current_source()?, edited()?);
        assert_eq!(gate.state()?["revision"], 2);
        assert_eq!(gate.state()?["undo_depth"], 1);
        assert_eq!(gate.state()?["file"]["dirty"], true);
        assert_eq!(fs::read(&gate.input)?, ORIGINAL);
        Ok(gate)
    }

    fn state(&self) -> TestResult<Value> {
        Ok(serde_json::to_value(
            self.engine.lock().map_err(|_| "engine")?.state(),
        )?)
    }

    fn current_source(&self) -> TestResult<Vec<u8>> {
        Ok(self
            .engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .sdf()?
            .as_bytes()
            .to_vec())
    }

    fn sync(&mut self) -> TestResult {
        let guard = self.engine.lock().map_err(|_| "engine")?;
        self.editor.sync(Some(&guard), &self.context);
        Ok(())
    }

    fn try_host_close(&self) -> TestResult<bool> {
        let mut guard = self.engine.lock().map_err(|_| "engine")?;
        Ok(!self.executor.has_active_work()
            && !self.editor.blocks_close()
            && guard.complete_close())
    }

    fn release_and_retire(&mut self) -> TestResult {
        self.release
            .take()
            .ok_or("worker gate already released")?
            .send(())?;
        let deadline = Instant::now() + WAIT;
        while self.editor.job.is_some() {
            self.sync()?;
            if Instant::now() >= deadline {
                return Err("Apply worker did not retire after its gate was released".into());
            }
            std::thread::yield_now();
        }
        Ok(())
    }
}

impl Drop for ApplyGate {
    fn drop(&mut self) {
        if let Some(job) = &self.editor.job {
            job.cancel.cancel();
        }
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
        // Cleanup is outside any UI frame/Engine guard; the worker gate also has a timeout.
        if let Some(job) = self.editor.job.take() {
            let _ = job.worker.join();
        }
    }
}

#[test]
fn cancelling_after_real_apply_publication_keeps_commit_and_reports_truth_after_retirement()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut gate = ApplyGate::published(&directory)?;
    gate.sync()?;
    assert!(gate.editor.is_busy() && gate.editor.blocks_close());
    assert!(gate.editor.take_effect().is_none());
    {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        guard.dispatch(
            "document.close",
            json!({"expected_revision":2,"action":"request"}),
        )?;
        guard.dispatch(
            "document.close",
            json!({"expected_revision":2,"action":"discard"}),
        )?;
    }
    assert_eq!(gate.state()?["close"], "approved");
    assert!(
        !gate.try_host_close()?,
        "a published result is not a retired UI job"
    );
    gate.editor.actions.push((0, Action::CancelWork));
    {
        let guard = gate.engine.lock().map_err(|_| "engine")?;
        gate.editor.replay(0, &guard, &gate.context);
    }
    assert!(gate.editor.is_busy());
    assert!(gate.editor.after_apply.is_none());
    assert!(
        gate.editor
            .job
            .as_ref()
            .is_some_and(|job| job.cancel.is_cancelled())
    );
    assert!(!gate.try_host_close()?);
    assert_eq!(gate.current_source()?, edited()?);
    gate.release_and_retire()?;
    assert!(!gate.editor.is_busy() && !gate.executor.has_active_work());
    let effect = gate.editor.take_effect().ok_or("committed Apply outcome")?;
    assert_eq!(effect.refresh, Some((1, 0, 2)));
    assert!(
        effect.target.is_none(),
        "the waiting Save was cancelled, not resumed"
    );
    assert_eq!(gate.current_source()?, edited()?);
    assert_eq!(gate.state()?["revision"], 2);
    assert_eq!(gate.state()?["undo_depth"], 1);
    assert_eq!(gate.state()?["file"]["dirty"], true);
    assert!(gate.state()?["ui_file"]["request"].is_null());
    assert_eq!(
        fs::read(&gate.input)?,
        ORIGINAL,
        "late Cancel must not publish the held Save"
    );
    assert!(
        effect
            .warning
            .as_deref()
            .is_some_and(|warning| warning.contains("applied before cancellation")),
        "a real committed Apply needs truthful 'applied before cancellation' information"
    );
    Ok(())
}

#[test]
fn published_historical_apply_preserves_recovery_text_and_never_replays_into_replacement()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let replacement = directory.path("replacement-690.sdf");
    fs::write(&replacement, REPLACEMENT)?;
    let mut gate = ApplyGate::published(&directory)?;
    {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        guard.dispatch(
            "document.open_sdf",
            json!({"path":replacement,"discard_unsaved":true}),
        )?;
    }
    let before = gate.state()?;
    assert_eq!(before["revision"], 3);
    assert_eq!(before["undo_depth"], 0);
    assert_eq!(before["file"]["dirty"], false);
    assert_eq!(gate.current_source()?, REPLACEMENT);
    gate.sync()?;
    assert!(gate.editor.is_busy() && gate.editor.blocks_close());
    assert!(gate.editor.take_effect().is_none());
    gate.release_and_retire()?;
    assert!(!gate.editor.is_busy() && !gate.executor.has_active_work());
    assert!(gate.editor.active() && gate.editor.blocks_close());
    assert_eq!(
        gate.editor
            .draft
            .as_ref()
            .ok_or("historical recovery draft")?
            .text
            .as_str(),
        DRAFT
    );
    assert!(
        gate.editor
            .error
            .as_deref()
            .is_some_and(|status| status.contains("applied to an earlier document"))
    );
    assert!(gate.editor.after_apply.is_none() && gate.editor.take_effect().is_none());
    assert_eq!(
        gate.state()?,
        before,
        "historical success cannot replace current state"
    );
    assert_eq!(gate.current_source()?, REPLACEMENT);
    assert_eq!(fs::read(&replacement)?, REPLACEMENT);
    assert_eq!(fs::read(&gate.input)?, ORIGINAL);
    {
        let guard = gate.engine.lock().map_err(|_| "engine")?;
        assert!(
            !gate.editor.apply(&guard),
            "recovery text cannot silently reapply to new source"
        );
    }
    assert!(gate.editor.pending.is_none());
    assert_eq!(gate.state()?, before);
    assert_eq!(gate.current_source()?, REPLACEMENT);
    assert_eq!(fs::read(replacement)?, REPLACEMENT);
    Ok(())
}
