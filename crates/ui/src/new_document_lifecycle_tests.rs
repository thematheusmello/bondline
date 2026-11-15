//! Finite request qualification for original New-document decisions.
use super::*;
use serde_json::json;
use std::{
    fs,
    sync::{Arc, Mutex, mpsc::SyncSender},
    time::Instant,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn dirty_engine() -> Result<Engine, bondline_commands::Error> {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":640.0,"height_pt":480.0}),
    )?;
    Ok(engine)
}

fn accepted_new(
    workflow: &mut FileWorkflow,
    engine: &mut Engine,
) -> Result<u64, Box<dyn std::error::Error>> {
    let result = engine.dispatch(
        "ui.file",
        json!({"expected_revision":engine.state().revision,"action":"new"}),
    )?;
    let id = result.ui_file_operation.ok_or("new request ID")?;
    let mut error = None;
    assert!(workflow.adopt_request(engine, &mut error).is_none());
    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("accepted request")?
            .phase,
        UiFilePhase::Accepted
    );
    Ok(id)
}

#[test]
fn accepted_dirty_new_cancel_preserves_source_history_and_never_requests_close() -> TestResult {
    let mut engine = dirty_engine()?;
    let original = engine.content().clone();
    let stamp = engine.capture_stamp();
    let revision = engine.state().revision;
    let mut workflow = FileWorkflow::new(std::env::temp_dir());
    let id = accepted_new(&mut workflow, &mut engine)?;
    assert!(workflow.blocks_authoring(&engine.state()));
    let mut error = None;
    workflow.choose_new(id, NewChoice::Cancel, &mut engine, &mut error);
    assert!(error.is_none(), "{error:?}");
    assert_eq!(engine.content(), &original);
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.state().revision, revision);
    assert_eq!(engine.state().undo_depth, 1);
    assert!(engine.state().file.dirty);
    assert_eq!(engine.state().close, CloseState::Open);
    assert_eq!(engine.state().close_epoch, 0);
    assert!(engine.state().ui_file.request.is_none());
    assert!(!workflow.blocks_authoring(&engine.state()) && !workflow.is_busy());
    Ok(())
}

#[test]
fn old_new_id_cannot_discard_a_reopened_request_or_a_replacement_source() -> TestResult {
    let directory = close_tests::Directory::new()?;
    let mut engine = dirty_engine()?;
    let mut workflow = FileWorkflow::new(directory.0.clone());
    let old = accepted_new(&mut workflow, &mut engine)?;
    workflow.choose_new(old, NewChoice::Cancel, &mut engine, &mut None);
    let current = accepted_new(&mut workflow, &mut engine)?;
    assert!(current > old);
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    workflow.choose_new(old, NewChoice::Discard, &mut engine, &mut None);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.capture_stamp_matches(&stamp));

    let source = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
    let path = directory.0.join("new-foreign-source-化学.sdf");
    fs::write(&path, source)?;
    engine.dispatch(
        "document.open_sdf",
        json!({"path":path,"discard_unsaved":true}),
    )?;
    let replacement = serde_json::to_value(engine.state())?;
    workflow.choose_new(current, NewChoice::Discard, &mut engine, &mut None);
    assert_eq!(serde_json::to_value(engine.state())?, replacement);
    assert_eq!(engine.content().sdf()?.as_bytes(), source);
    assert_eq!(fs::read(path)?, source);
    assert!(!workflow.is_busy());
    Ok(())
}

const WAIT: Duration = Duration::from_secs(5);

struct SaveGate {
    workflow: FileWorkflow,
    engine: bondline_mcp::SharedEngine,
    executor: CommandExecutor,
    context: egui::Context,
    release: Option<SyncSender<()>>,
    path: PathBuf,
}

impl SaveGate {
    fn published(directory: &close_tests::Directory) -> Result<Self, Box<dyn std::error::Error>> {
        let path = directory.0.join("save-before-new-化学.cdxml");
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch("document.save_cdxml", json!({"path":path}))?;
        engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":640,"height_pt":480}),
        )?;
        let mut workflow = FileWorkflow::new(directory.0.clone());
        let id = accepted_new(&mut workflow, &mut engine)?;
        let mut error = None;
        workflow.choose_new(id, NewChoice::Save, &mut engine, &mut error);
        assert!(error.is_none(), "{error:?}");
        let intent = workflow
            .adopt_request(&mut engine, &mut error)
            .ok_or("actual associated Save intent")?;
        assert_eq!(intent.format, FileFormat::Cdxml);
        let command = match &intent.action {
            FileAction::Save { path, overwrite } => Command::SaveCdxml(SaveCdxml {
                path: path.clone(),
                overwrite: *overwrite,
            }),
            _ => return Err("expected Save".into()),
        };
        let engine = Arc::new(Mutex::new(engine));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let work = executor
            .prepare_ui_file(intent.request_id, command)
            .map_err(|error| error.message)?;
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let (sender, result) = mpsc::sync_channel(1);
        let (published, publication) = mpsc::sync_channel(1);
        let (release, wait) = mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            let output = work.complete_blocking(&worker_cancel);
            let _ = sender.send(output);
            let _ = published.send(());
            let _ = wait.recv_timeout(WAIT);
        });
        workflow.retain_new_worker(intent.request_id, worker);
        workflow.job = Some(FileJob {
            request_id: intent.request_id,
            format: intent.format,
            action: intent.action,
            result,
            cancel,
            cancelling: false,
            close_save: None,
        });
        let gate = Self {
            workflow,
            engine,
            executor,
            context: egui::Context::default(),
            release: Some(release),
            path,
        };
        publication.recv_timeout(WAIT)?;
        assert!(
            !gate.executor.has_active_work(),
            "registry work finished before the held thread exits"
        );
        let guard = gate.engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.state().revision, 1);
        assert_eq!(guard.state().undo_depth, 1);
        assert!(!guard.state().file.dirty);
        assert_eq!(
            guard
                .state()
                .ui_file
                .request
                .as_ref()
                .ok_or("unconsumed actual Save result")?
                .phase,
            UiFilePhase::Running
        );
        drop(guard);
        independently_check_saved_page(&gate.path)?;
        Ok(gate)
    }

    fn poll(&mut self) -> TestResult {
        let mut guard = self.engine.lock().map_err(|_| "engine")?;
        self.workflow.poll(&mut guard, &mut None);
        Ok(())
    }

    fn release_and_poll(&mut self) -> TestResult {
        self.release
            .take()
            .ok_or("gate already released")?
            .send(())?;
        let deadline = Instant::now() + WAIT;
        while self.workflow.is_busy() {
            self.poll()?;
            if Instant::now() >= deadline {
                return Err("New Save did not retire".into());
            }
            std::thread::yield_now();
        }
        Ok(())
    }
}

impl Drop for SaveGate {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
        let deadline = Instant::now() + WAIT;
        while self.workflow.is_busy() && Instant::now() < deadline {
            self.workflow.deactivate_new(&self.context, &mut None);
            std::thread::yield_now();
        }
    }
}

fn independently_check_saved_page(path: &std::path::Path) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let output = std::process::Command::new(python)
        .args([
            "-B",
            "-c",
            r#"
import sys, xml.etree.ElementTree as ET
p = ET.parse(sys.argv[1]).getroot().findall('page')
assert len(p) == 1
assert float(p[0].attrib['Width']) == 640
assert float(p[0].attrib['Height']) == 480
assert len(list(p[0])) == 0
"#,
        ])
        .arg(path)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn real_save_publication_and_receipt_cannot_create_new_before_thread_retirement() -> TestResult {
    let directory = close_tests::Directory::new()?;
    let mut gate = SaveGate::published(&directory)?;
    gate.poll()?;
    {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        assert!(gate.workflow.is_busy() && gate.workflow.blocks_authoring(&guard.state()));
        assert!(
            !gate
                .workflow
                .complete_close_if_idle(&mut guard, false, false)
        );
        assert_eq!(guard.state().revision, 1);
        assert!(guard.state().file.path.is_some());
        assert_eq!(guard.state().undo_depth, 1);
    }
    gate.release_and_poll()?;
    let guard = gate.engine.lock().map_err(|_| "engine")?;
    assert_eq!(guard.state().revision, 2);
    assert_eq!(guard.state().undo_depth, 0);
    assert!(guard.state().file.path.is_none() && !guard.state().file.dirty);
    assert!(guard.state().ui_file.request.is_none());
    assert!(guard.content().drawing().is_ok());
    assert!(!gate.workflow.blocks_authoring(&guard.state()));
    independently_check_saved_page(&gate.path)?;
    Ok(())
}

#[test]
fn late_cancel_disarms_new_but_keeps_the_successful_save_and_its_actual_result() -> TestResult {
    let directory = close_tests::Directory::new()?;
    let mut gate = SaveGate::published(&directory)?;
    gate.workflow.new_document.disarm();
    gate.workflow
        .job
        .as_ref()
        .ok_or("actual file job")?
        .cancel
        .cancel();
    gate.poll()?;
    assert!(gate.workflow.is_busy());
    gate.release_and_poll()?;
    let guard = gate.engine.lock().map_err(|_| "engine")?;
    assert_eq!(guard.state().revision, 1);
    assert_eq!(guard.state().undo_depth, 1);
    assert!(!guard.state().file.dirty && guard.state().file.path.is_some());
    assert!(guard.state().ui_file.request.is_none());
    assert!(!gate.workflow.blocks_authoring(&guard.state()));
    independently_check_saved_page(&gate.path)?;
    Ok(())
}

#[test]
fn close_disarms_new_and_still_waits_for_the_real_save_thread() -> TestResult {
    let directory = close_tests::Directory::new()?;
    let mut gate = SaveGate::published(&directory)?;
    {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        gate.workflow.request_close(&mut guard, &mut None);
        assert_eq!(guard.state().close, CloseState::Approved);
        assert!(
            !gate
                .workflow
                .complete_close_if_idle(&mut guard, false, false)
        );
    }
    gate.release_and_poll()?;
    let mut guard = gate.engine.lock().map_err(|_| "engine")?;
    assert_eq!(guard.state().revision, 1);
    assert_eq!(guard.state().undo_depth, 1);
    assert!(guard.state().file.path.is_some());
    assert!(
        gate.workflow
            .complete_close_if_idle(&mut guard, false, false)
    );
    assert_eq!(guard.state().close, CloseState::Committed);
    independently_check_saved_page(&gate.path)?;
    Ok(())
}

#[test]
fn source_replacement_after_real_save_publication_never_creates_new_in_the_replacement()
-> TestResult {
    let directory = close_tests::Directory::new()?;
    let mut gate = SaveGate::published(&directory)?;
    let source = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
    let path = directory.0.join("replacement-化学.sdf");
    fs::write(&path, source)?;
    let before = {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        guard.dispatch("document.open_sdf", json!({"path":path}))?;
        serde_json::to_value(guard.state())?
    };
    gate.release_and_poll()?;
    let guard = gate.engine.lock().map_err(|_| "engine")?;
    let after = serde_json::to_value(guard.state())?;
    assert_eq!(after["revision"], before["revision"]);
    assert_eq!(after["collection"], before["collection"]);
    assert_eq!(after["file"], before["file"]);
    assert_eq!(guard.content().sdf()?.as_bytes(), source);
    assert_eq!(fs::read(path)?, source);
    independently_check_saved_page(&gate.path)?;
    Ok(())
}

#[test]
fn poisoned_engine_cleanup_retires_the_new_save_without_authorizing_replacement() -> TestResult {
    let directory = close_tests::Directory::new()?;
    let mut gate = SaveGate::published(&directory)?;
    let owner = Arc::clone(&gate.engine);
    let poisoned = std::thread::spawn(move || {
        let Ok(_held) = owner.lock() else { return };
        std::panic::resume_unwind(Box::new("intentional New Save lifecycle poison"));
    })
    .join();
    assert!(poisoned.is_err());
    let mut error = None;
    gate.workflow.deactivate_new(&gate.context, &mut error);
    assert!(gate.workflow.is_busy());
    gate.release.take().ok_or("release")?.send(())?;
    let deadline = Instant::now() + WAIT;
    while gate.workflow.is_busy() {
        gate.workflow.deactivate_new(&gate.context, &mut error);
        if Instant::now() >= deadline {
            return Err("engine-free New Save retirement timed out".into());
        }
        std::thread::yield_now();
    }
    assert!(error.as_ref().is_some_and(
        |message| message.contains("Save completed") && message.contains("no new document")
    ));
    let guard = gate
        .engine
        .lock()
        .err()
        .ok_or("real Engine poison")?
        .into_inner();
    assert_eq!(guard.state().revision, 1);
    assert!(guard.state().file.path.is_some() && !guard.state().file.dirty);
    assert_eq!(guard.state().undo_depth, 1);
    assert_eq!(
        guard
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("no fabricated ACK under poison")?
            .phase,
        UiFilePhase::Running
    );
    assert!(!gate.executor.has_active_work());
    independently_check_saved_page(&gate.path)?;
    Ok(())
}

#[test]
fn cancelled_new_save_keeps_dirty_source_and_an_ordinary_retry_cannot_resume_new() -> TestResult {
    let directory = close_tests::Directory::new()?;
    let path = directory.0.join("cancelled-save-before-new.cdxml");
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    let original_file = fs::read(&path)?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":640,"height_pt":480}),
    )?;
    let mut workflow = FileWorkflow::new(directory.0.clone());
    let id = accepted_new(&mut workflow, &mut engine)?;
    workflow.choose_new(id, NewChoice::Save, &mut engine, &mut None);
    let intent = workflow
        .adopt_request(&mut engine, &mut None)
        .ok_or("actual Save intent")?;
    let FileAction::Save {
        path: save_path,
        overwrite,
    } = &intent.action
    else {
        return Err("Save action".into());
    };
    let command = Command::SaveCdxml(SaveCdxml {
        path: save_path.clone(),
        overwrite: *overwrite,
    });
    let request_id = intent.request_id;
    let engine = Arc::new(Mutex::new(engine));
    let executor = CommandExecutor::new(Arc::clone(&engine));
    let work = executor
        .prepare_ui_file(request_id, command)
        .map_err(|error| error.message)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    let worker_cancel = cancel.clone();
    let (sender, result) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let _ = sender.send(work.complete_blocking(&worker_cancel));
    });
    workflow.retain_new_worker(request_id, worker);
    workflow.job = Some(FileJob {
        request_id,
        format: FileFormat::Cdxml,
        action: intent.action.clone(),
        result,
        cancel,
        cancelling: true,
        close_save: None,
    });
    let deadline = Instant::now() + WAIT;
    let mut error = None;
    while workflow.is_busy() {
        let mut guard = engine.lock().map_err(|_| "engine")?;
        workflow.poll(&mut guard, &mut error);
        if Instant::now() >= deadline {
            return Err("cancelled New Save did not retire".into());
        }
        std::thread::yield_now();
    }
    assert!(
        error
            .as_ref()
            .is_some_and(|message| message.contains("No new document was created"))
    );
    assert_eq!(
        fs::read(&path)?,
        original_file,
        "cancelled Save never published"
    );
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.state().revision, 1);
        assert_eq!(guard.state().undo_depth, 1);
        assert!(guard.state().file.dirty);
        assert!(
            workflow.chooser.is_open(),
            "ordinary explicit Save retry remains available"
        );
    }
    error = None;
    workflow.chooser.cancel();
    workflow.start(
        FileIntentAction {
            request_id,
            format: FileFormat::Cdxml,
            action: intent.action,
        },
        &executor,
        &egui::Context::default(),
    )?;
    let deadline = Instant::now() + WAIT;
    while workflow.is_busy() {
        let mut guard = engine.lock().map_err(|_| "engine")?;
        workflow.poll(&mut guard, &mut error);
        if Instant::now() >= deadline {
            return Err("ordinary Save retry did not finish".into());
        }
        std::thread::yield_now();
    }
    assert!(error.is_none(), "{error:?}");
    let guard = engine.lock().map_err(|_| "engine")?;
    assert_eq!(
        guard.state().revision,
        1,
        "successful retry must not create New"
    );
    assert_eq!(guard.state().undo_depth, 1);
    assert!(!guard.state().file.dirty && guard.state().file.path.is_some());
    assert!(guard.state().ui_file.request.is_none());
    independently_check_saved_page(&path)?;
    Ok(())
}
