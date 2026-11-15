//! Finite gates around real registered conversion/publication and thread retirement.
use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, Mutex, mpsc::SyncSender},
    time::Instant,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const WAIT: Duration = Duration::from_secs(5);
const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

struct ConversionGate {
    workflow: FileWorkflow,
    engine: bondline_mcp::SharedEngine,
    executor: CommandExecutor,
    context: egui::Context,
    release: Option<SyncSender<()>>,
    output: PathBuf,
    source: Vec<u8>,
    state: Value,
}

impl ConversionGate {
    fn published(directory: &super::super::close_tests::Directory) -> TestResult<Self> {
        let input = directory.0.join("gated-input-化学.smi");
        let output = directory.0.join("gated-output-化学.cdxml");
        let current = directory.0.join("gated-source-化学.sdf");
        fs::write(&input, b"CCC\n")?;
        fs::write(&current, SOURCE)?;
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch("document.open_sdf", json!({"path":current}))?;
        engine.dispatch(
            "sdf.field.set_value",
            json!({"expected_revision":1,"record_id":1,"field_index":2,
                "value_lines":["reviewed","line two"]}),
        )?;
        let source = [
            SOURCE.get(..378).ok_or("literal source prefix")?,
            b"reviewed\nline two\n",
            SOURCE.get(385..).ok_or("literal source suffix")?,
        ]
        .concat();
        assert_eq!(source.len(), 788);
        assert_eq!(engine.content().sdf()?.as_bytes(), source);
        let params = ConvertSmilesToCdxml {
            profile: Default::default(),
            expected_revision: 2,
            input_path: input.to_string_lossy().into_owned(),
            output_path: output.to_string_lossy().into_owned(),
            bond_length_pt: 48.0,
            origin_x_pt: 72.0,
            origin_y_pt: 72.0,
            page_width_pt: 210.0 * 72.0 / 25.4,
            page_height_pt: 297.0 * 72.0 / 25.4,
        };
        let mut workflow = FileWorkflow::new(directory.0.clone());
        engine.dispatch(
            "ui.file",
            json!({"expected_revision":2,"action":"convert_smiles"}),
        )?;
        assert!(workflow.adopt_request(&mut engine, &mut None).is_none());
        let form = workflow.conversion.form.as_mut().ok_or("conversion form")?;
        let id = form.accepted.ok_or("accepted conversion")?;
        let captured = form.context.clone();
        let engine = Arc::new(Mutex::new(engine));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let work = executor
            .prepare_ui_smiles_conversion(id, params)
            .map_err(|error| error.message)?;
        assert!(executor.has_active_work());
        form.accepted = None;
        let state = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        assert!(state["ui_file"]["request"].is_null());
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
        workflow.conversion.job = Some(Job {
            context: captured,
            cancel,
            result,
            worker,
        });
        let gate = Self {
            workflow,
            engine,
            executor,
            context: egui::Context::default(),
            release: Some(release),
            output,
            source,
            state,
        };
        publication.recv_timeout(WAIT)?;
        assert!(!gate.executor.has_active_work());
        assert!(
            gate.workflow
                .conversion
                .job
                .as_ref()
                .is_some_and(|job| !job.worker.is_finished())
        );
        assert!(
            gate.workflow
                .conversion
                .form
                .as_ref()
                .is_some_and(|form| form.outcome.is_none())
        );
        assert_eq!(fs::read(input)?, b"CCC\n");
        assert_eq!(fs::read(current)?, SOURCE);
        independently_check_output(&gate.output)?;
        gate.assert_source_unchanged()?;
        Ok(gate)
    }

    fn assert_source_unchanged(&self) -> TestResult {
        let guard = self.engine.lock().map_err(|_| "engine")?;
        assert_eq!(serde_json::to_value(guard.state())?, self.state);
        assert_eq!(guard.content().sdf()?.as_bytes(), self.source);
        Ok(())
    }

    fn poll(&mut self) -> TestResult {
        let guard = self.engine.lock().map_err(|_| "engine")?;
        self.workflow.sync_conversion(Some(&guard), &mut None);
        Ok(())
    }

    fn retire(&mut self) -> TestResult {
        self.release
            .take()
            .ok_or("gate already released")?
            .send(())?;
        let deadline = Instant::now() + WAIT;
        while self.workflow.conversion.job.is_some() {
            if self.engine.is_poisoned() {
                self.workflow
                    .deactivate_conversion(&self.context, &mut None);
            } else {
                self.poll()?;
            }
            if Instant::now() >= deadline {
                return Err("conversion thread did not retire".into());
            }
            std::thread::yield_now();
        }
        Ok(())
    }

    fn assert_success(&self, historical: bool, cancelled: bool) -> TestResult {
        let Some(Outcome::Success {
            receipt,
            historical: actual_history,
            cancelled: actual_cancel,
        }) = self
            .workflow
            .conversion
            .form
            .as_ref()
            .and_then(|form| form.outcome.as_ref())
        else {
            return Err("real published conversion receipt was discarded".into());
        };
        assert_eq!(*actual_history, historical);
        assert_eq!(*actual_cancel, cancelled);
        assert_eq!(receipt.profile, "neutral_carbon_path_v1");
        assert_eq!(receipt.input_revision, 2);
        assert_eq!(receipt.atom_count, 3);
        assert_eq!(receipt.bond_count, 2);
        assert_eq!(receipt.input_byte_length, 4);
        assert_eq!(
            PathBuf::from(&receipt.output.path),
            fs::canonicalize(&self.output)?
        );
        assert_eq!(receipt.output.byte_length, fs::read(&self.output)?.len());
        assert!(!self.workflow.is_busy() && !self.executor.has_active_work());
        Ok(())
    }
}

impl Drop for ConversionGate {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
        let deadline = Instant::now() + WAIT;
        while self.workflow.conversion.job.is_some() && Instant::now() < deadline {
            self.workflow
                .deactivate_conversion(&self.context, &mut None);
            std::thread::yield_now();
        }
    }
}

fn independently_check_output(path: &std::path::Path) -> TestResult {
    use std::io::Write;
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../io/tests/smiles_layout_oracle.py");
    let mut child = std::process::Command::new(python)
        .arg("-B")
        .arg(script)
        .arg(r#"[["72","72"],["113.56921938165306","96"],["155.1384387633061","72"]]"#)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("external consumer stdin")?
        .write_all(&fs::read(path)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)?,
        json!({"smiles":"CCC","formula":"C3H8","atoms":3,"bonds":2})
    );
    Ok(())
}

#[test]
fn published_conversion_waits_for_actual_thread_retirement_without_save_ack_or_source_edit()
-> TestResult {
    let directory = super::super::close_tests::Directory::new()?;
    let mut gate = ConversionGate::published(&directory)?;
    gate.poll()?;
    {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        assert!(gate.workflow.is_busy() && gate.workflow.blocks_authoring(&guard.state()));
        assert!(
            !gate
                .workflow
                .complete_close_if_idle(&mut guard, false, false)
        );
        assert!(
            !guard.finish_ui_file(1, false),
            "conversion cannot create trusted Save ACK"
        );
    }
    gate.retire()?;
    gate.assert_success(false, false)?;
    gate.assert_source_unchanged()?;
    Ok(())
}

#[test]
fn late_cancel_and_source_replacement_keep_the_real_historical_publication() -> TestResult {
    let directory = super::super::close_tests::Directory::new()?;
    let mut gate = ConversionGate::published(&directory)?;
    let replacement = {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        guard.dispatch(
            "document.new",
            json!({"expected_revision":2,"discard_unsaved":true}),
        )?;
        gate.workflow
            .cancel_conversion(&mut guard, &mut None, &gate.context);
        serde_json::to_value(guard.state())?
    };
    gate.poll()?;
    assert!(gate.workflow.is_busy());
    gate.retire()?;
    gate.assert_success(true, true)?;
    let mut guard = gate.engine.lock().map_err(|_| "engine")?;
    assert_eq!(serde_json::to_value(guard.state())?, replacement);
    assert!(guard.content().drawing().is_ok());
    assert!(guard.state().file.path.is_none() && !guard.state().file.dirty);
    assert_eq!(guard.state().undo_depth, 0);
    assert!(!guard.finish_ui_file(1, false));
    Ok(())
}

#[test]
fn close_waits_for_the_conversion_thread_and_retains_its_published_receipt() -> TestResult {
    let directory = super::super::close_tests::Directory::new()?;
    let mut gate = ConversionGate::published(&directory)?;
    {
        let mut guard = gate.engine.lock().map_err(|_| "engine")?;
        gate.workflow.request_close(&mut guard, &mut None);
        assert_eq!(guard.state().close, CloseState::NeedsDecision);
        gate.workflow
            .choose_close(CloseChoice::Discard, &mut guard, &mut None);
        assert_eq!(guard.state().close, CloseState::Approved);
        assert!(
            !gate
                .workflow
                .complete_close_if_idle(&mut guard, false, false)
        );
        assert_eq!(guard.content().sdf()?.as_bytes(), gate.source);
    }
    gate.retire()?;
    gate.assert_success(true, true)?;
    let mut guard = gate.engine.lock().map_err(|_| "engine")?;
    assert_eq!(guard.state().revision, 2);
    assert_eq!(guard.state().undo_depth, 1);
    assert!(guard.state().file.dirty);
    assert_eq!(guard.content().sdf()?.as_bytes(), gate.source);
    assert!(
        gate.workflow
            .complete_close_if_idle(&mut guard, false, false)
    );
    assert_eq!(guard.state().close, CloseState::Committed);
    Ok(())
}

#[test]
fn poisoned_engine_drains_published_conversion_without_losing_success_or_authorizing_save()
-> TestResult {
    let directory = super::super::close_tests::Directory::new()?;
    let mut gate = ConversionGate::published(&directory)?;
    let owner = Arc::clone(&gate.engine);
    assert!(
        std::thread::spawn(move || {
            let Ok(_held) = owner.lock() else { return };
            std::panic::resume_unwind(Box::new("intentional conversion lifecycle poison"));
        })
        .join()
        .is_err()
    );
    gate.workflow
        .deactivate_conversion(&gate.context, &mut None);
    assert!(gate.workflow.is_busy());
    gate.retire()?;
    gate.assert_success(true, true)?;
    let mut guard = gate
        .engine
        .lock()
        .err()
        .ok_or("real Engine poison")?
        .into_inner();
    assert_eq!(serde_json::to_value(guard.state())?, gate.state);
    assert_eq!(guard.content().sdf()?.as_bytes(), gate.source);
    assert!(!guard.finish_ui_file(1, false));
    Ok(())
}
