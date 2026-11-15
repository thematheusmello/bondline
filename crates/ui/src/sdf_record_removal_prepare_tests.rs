//! Independent pre-spawn file-context regression, using the real registered worker.
use super::*;
use serde_json::json;
use std::{
    fs,
    sync::{Arc, Mutex},
    time::Instant,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse the existing isolated temporary-directory helper"
)]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

struct RetiringEditor(SdfEditor);

impl Drop for RetiringEditor {
    fn drop(&mut self) {
        if let Some(job) = self.0.job.take() {
            job.cancel.cancel();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !job.worker.is_finished() && Instant::now() < deadline {
                std::thread::yield_now();
            }
            if job.worker.is_finished() {
                let _ = job.worker.join();
            }
        }
    }
}

#[test]
fn queued_removal_refuses_a_new_file_request_before_reservation_or_worker_spawn() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("preparation-context-化学.sdf");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    engine.set_ui_file_host(true);
    let engine = Arc::new(Mutex::new(engine));
    let executor = CommandExecutor::new(Arc::clone(&engine));
    let context = egui::Context::default();
    let mut editor = RetiringEditor(SdfEditor::default());
    {
        let frame_guard = engine.lock().map_err(|_| "frame Engine guard")?;
        assert!(editor.0.confirm_remove(&frame_guard, &context, 1));
        let sequence = editor.0.removal.as_ref().ok_or("confirmation")?.sequence;
        editor.0.actions.push((0, Action::ConfirmRemove(sequence)));
        editor.0.replay(0, &frame_guard, &context);
        assert!(editor.0.pending.is_some() && editor.0.removal.is_none());
        assert!(editor.0.is_busy() && editor.0.holds_files());
        assert!(editor.0.job.is_none() && !executor.has_active_work());
        assert_eq!(frame_guard.content().sdf()?.as_bytes(), SOURCE);
        assert_eq!(frame_guard.state().revision, 1);
        assert_eq!(frame_guard.state().undo_depth, 0);
    }
    // This real shared command interposes after frame unlock and before preparation.
    let before = {
        let mut guard = engine.lock().map_err(|_| "interposed Engine guard")?;
        guard.dispatch("ui.file", json!({"expected_revision":1,"action":"save"}))?;
        let state = serde_json::to_value(guard.state())?;
        assert_eq!(state["ui_file"]["epoch"], 1);
        assert_eq!(state["ui_file"]["request"]["phase"], "requested");
        assert_eq!(state["ui_file"]["request"]["action"], "save");
        state
    };
    editor.0.start_pending(&executor, &context);
    assert!(
        editor.0.job.is_none(),
        "obsolete confirmation file context must refuse before worker spawn"
    );
    assert!(
        !executor.has_active_work(),
        "no worker reservation may remain"
    );
    assert!(editor.0.pending.is_none());
    let message = editor
        .0
        .error
        .as_deref()
        .ok_or("truthful stale-context refusal")?;
    assert!(
        message.contains("context") || message.contains("obsolete") || message.contains("stale"),
        "{message}"
    );
    let guard = engine.lock().map_err(|_| "final Engine guard")?;
    assert_eq!(serde_json::to_value(guard.state())?, before);
    assert_eq!(guard.content().sdf()?.as_bytes(), SOURCE);
    assert_eq!(fs::read(input)?, SOURCE);
    Ok(())
}
