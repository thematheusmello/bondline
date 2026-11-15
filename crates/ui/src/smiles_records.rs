//! Explicit readonly SMILES record disclosure through the shared registered executor.
use bondline_commands::{
    Cancellation, CaptureStamp, CloseState, Engine, SmilesRecordInspect, SmilesRecordInspection,
    StoredContent,
};
use bondline_mcp::CommandExecutor;
use eframe::egui;
use std::{
    sync::mpsc::{self, Receiver},
    thread::JoinHandle,
    time::Duration,
};

#[derive(Clone, PartialEq, Eq)]
struct RecordKey {
    stamp: CaptureStamp,
    close_epoch: u64,
    revision: u64,
    ordinal: u32,
    sequence: u64,
}
impl RecordKey {
    fn current(&self, engine: &Engine) -> bool {
        let state = engine.state();
        matches!(engine.content(), StoredContent::SmilesFile(_))
            && state.close == CloseState::Open
            && state.close_epoch == self.close_epoch
            && engine.capture_stamp_matches(&self.stamp)
    }
}
struct RecordJob {
    key: RecordKey,
    cancel: Cancellation,
    result: Receiver<Result<SmilesRecordInspection, String>>,
    worker: JoinHandle<()>,
}
pub(crate) struct SmilesRecordsWorkflow {
    source: Option<CaptureStamp>,
    ordinal: f64,
    number_error: Option<&'static str>,
    sequence: u64,
    desired: Option<RecordKey>,
    pending: Option<RecordKey>,
    job: Option<RecordJob>,
    record: Option<SmilesRecordInspection>,
    error: Option<String>,
    text_id: Option<egui::Id>,
}
impl Default for SmilesRecordsWorkflow {
    fn default() -> Self {
        Self {
            source: None,
            ordinal: 1.0,
            number_error: None,
            sequence: 0,
            desired: None,
            pending: None,
            job: None,
            record: None,
            error: None,
            text_id: None,
        }
    }
}
impl SmilesRecordsWorkflow {
    pub(crate) fn is_busy(&self) -> bool {
        self.pending.is_some() || self.job.is_some()
    }
    fn clear(&mut self, context: &egui::Context) {
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
        self.desired = None;
        self.pending = None;
        self.record = None;
        self.error = None;
        if let Some(id) = self.text_id.take() {
            if let Some(mut state) = egui::TextEdit::load_state(context, id) {
                state.clear_undoer();
                state.store(context, id);
            }
            context.memory_mut(|memory| memory.surrender_focus(id));
        }
    }
    pub(crate) fn sync(&mut self, engine: &Engine, context: &egui::Context) {
        if !matches!(engine.content(), StoredContent::SmilesFile(_)) {
            self.clear(context);
            self.source = None;
            self.ordinal = 1.0;
            self.number_error = None;
        } else {
            let stamp = engine.capture_stamp();
            if self.source.as_ref() != Some(&stamp) {
                self.clear(context);
                self.ordinal = 1.0;
                self.number_error = None;
                self.source = Some(stamp);
            }
            if engine.state().close != CloseState::Open
                || self
                    .desired
                    .as_ref()
                    .is_some_and(|key| !key.current(engine))
            {
                self.clear(context);
            }
        }
        self.retire(Some(engine), context);
    }
    pub(crate) fn deactivate(&mut self, context: &egui::Context) {
        self.clear(context);
        self.retire(None, context);
    }
    fn checked_ordinal(&self) -> Option<u32> {
        (self.number_error.is_none()
            && self.ordinal.is_finite()
            && self.ordinal.fract() == 0.0
            && (1.0..=f64::from(u32::MAX)).contains(&self.ordinal))
        .then_some(self.ordinal as u32)
    }
    fn request(&mut self, engine: &Engine, context: &egui::Context) -> bool {
        self.clear(context);
        if !matches!(engine.content(), StoredContent::SmilesFile(_))
            || engine.state().close != CloseState::Open
        {
            return false;
        }
        let Some(ordinal) = self.checked_ordinal() else {
            self.error = Some(
                "Record number must be a positive whole number within the supported range.".into(),
            );
            return false;
        };
        let Some(sequence) = self.sequence.checked_add(1) else {
            self.error =
                Some("The record request limit was reached. Reopen Bondline to continue.".into());
            return false;
        };
        let stamp = engine.capture_stamp();
        self.source = Some(stamp.clone());
        self.sequence = sequence;
        let key = RecordKey {
            stamp,
            close_epoch: engine.state().close_epoch,
            revision: engine.state().revision,
            ordinal,
            sequence,
        };
        self.desired = Some(key.clone());
        self.pending = Some(key);
        true
    }
    fn retire(&mut self, engine: Option<&Engine>, context: &egui::Context) {
        if self
            .job
            .as_ref()
            .is_some_and(|job| !job.worker.is_finished())
        {
            context.request_repaint_after(Duration::from_millis(40));
            return;
        }
        let Some(job) = self.job.take() else {
            return;
        };
        let RecordJob {
            key,
            cancel,
            result,
            worker,
        } = job;
        let result = match worker.join() {
            Ok(()) => result
                .try_recv()
                .unwrap_or_else(|_| Err("The record worker stopped without a result.".into())),
            Err(_) => Err("The record worker did not complete.".into()),
        };
        if cancel.is_cancelled()
            || self.desired.as_ref() != Some(&key)
            || !engine.is_some_and(|engine| key.current(engine))
        {
            return;
        }
        match result {
            Ok(record)
                if record.input_revision == key.revision && record.record_id == key.ordinal =>
            {
                self.record = Some(record)
            }
            Ok(_) => self.error = Some("The record result did not match this request.".into()),
            Err(message) => self.error = Some(message),
        }
    }
    pub(crate) fn start_pending(&mut self, executor: &CommandExecutor, context: &egui::Context) {
        if self.job.is_some() {
            return;
        }
        let Some(key) = self.pending.take() else {
            return;
        };
        let work = match executor.prepare_ui_smiles(
            &key.stamp,
            key.close_epoch,
            SmilesRecordInspect {
                expected_revision: key.revision,
                record_id: key.ordinal,
            },
        ) {
            Ok(work) => work,
            Err(error) if error.code == "busy" => {
                self.pending = Some(key);
                context.request_repaint_after(Duration::from_millis(40));
                return;
            }
            Err(error) => {
                if matches!(
                    error.code,
                    "stale_revision" | "ui_stale_request" | "close_committed"
                ) {
                    self.clear(context);
                } else {
                    self.error = Some(error.message);
                }
                context.request_repaint();
                return;
            }
        };
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let worker_context = context.clone();
        let (sender, result) = mpsc::sync_channel(1);
        match std::thread::Builder::new()
            .name("bondline-smiles-record".into())
            .spawn(move || {
                let output = work
                    .complete_blocking(&worker_cancel)
                    .map_err(|error| error.message)
                    .and_then(|result| {
                        result
                            .smiles_record
                            .ok_or_else(|| "The record command returned no notation.".to_owned())
                    });
                let _ = sender.send(output);
                worker_context.request_repaint();
            }) {
            Ok(worker) => {
                self.job = Some(RecordJob {
                    key,
                    cancel,
                    result,
                    worker,
                })
            }
            Err(_) => {
                self.error = Some("The record worker could not start. Retry inspection.".into());
                context.request_repaint();
            }
        }
    }
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, engine: &Engine) {
        let count = engine
            .content()
            .smiles_summary()
            .map_or(0, |summary| summary.record_count);
        let enabled =
            engine.state().close == CloseState::Open && engine.state().ui_file.request.is_none();
        ui.separator();
        ui.heading("SMILES record");
        let mut requested = false;
        ui.horizontal_wrapped(|ui| {
            let label = ui.label("Record number");
            let mut value = self.ordinal;
            let mut changes = Vec::new();
            let response = ui
                .add_enabled(
                    enabled,
                    egui::DragValue::from_get_set(|next| {
                        if let Some(next) = next {
                            changes.push(next);
                            if next.is_finite() {
                                value = next;
                            }
                        }
                        value
                    })
                    .speed(1.0)
                    .max_decimals(6)
                    .clamp_existing_to_range(false)
                    .update_while_editing(false),
                )
                .labelled_by(label.id);
            let raw = ui.input(|input| {
                input
                    .events
                    .iter()
                    .filter_map(|event| {
                        if let egui::Event::AccessKitActionRequest(request) = event
                            && request.target.0 == response.id.value()
                            && request.action == egui::accesskit::Action::SetValue
                            && let Some(egui::accesskit::ActionData::NumericValue(value)) =
                                request.data
                        {
                            Some(value)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            });
            if enabled {
                if !raw.is_empty() {
                    changes = raw;
                }
                for value in changes {
                    self.clear(ui.ctx());
                    if value.is_finite() {
                        self.ordinal = value;
                        self.number_error = None;
                    } else {
                        self.number_error = Some("Record number must be finite.");
                        self.error = Some("Record number must be finite.".into());
                    }
                }
            }
            if ui
                .add_enabled(enabled && count > 0, egui::Button::new("Inspect record"))
                .clicked()
            {
                requested = true;
            }
        });
        let ordinal = self.checked_ordinal();
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    enabled
                        && ordinal
                            .is_some_and(|n| n > 1 && usize::try_from(n).is_ok_and(|n| n <= count)),
                    egui::Button::new("Previous record"),
                )
                .clicked()
            {
                self.ordinal -= 1.0;
                requested = true;
            }
            if ui
                .add_enabled(
                    enabled && ordinal.is_some_and(|n| usize::try_from(n).is_ok_and(|n| n < count)),
                    egui::Button::new("Next record"),
                )
                .clicked()
            {
                self.ordinal += 1.0;
                requested = true;
            }
            if (self.desired.is_some() || self.error.is_some() || self.is_busy())
                && ui.button("Close record").clicked()
            {
                self.deactivate(ui.ctx());
            }
        });
        if requested {
            self.request(engine, ui.ctx());
        }
        if count == 0 {
            ui.label("No SMILES records");
            return;
        }
        if self.is_busy() {
            ui.spinner();
            ui.label("Reading record…");
            if ui.button("Cancel inspection").clicked() {
                self.clear(ui.ctx());
                self.error = Some("Inspection cancelled.".into());
            }
        }
        if let Some(message) = &self.error {
            ui.colored_label(
                egui::Color32::DARK_RED,
                format!("Inspection unavailable: {message}"),
            );
            if ui
                .add_enabled(
                    enabled && !self.is_busy(),
                    egui::Button::new("Retry inspection"),
                )
                .clicked()
            {
                self.request(engine, ui.ctx());
            }
        }
        let Some(record) = &self.record else {
            return;
        };
        ui.label(format!(
            "Record {}, line {}",
            record.record_id, record.line_number
        ));
        ui.label(format!(
            "{} {}, {} {}",
            record.atom_count,
            if record.atom_count == 1 {
                "atom"
            } else {
                "atoms"
            },
            record.bond_count,
            if record.bond_count == 1 {
                "bond"
            } else {
                "bonds"
            }
        ));
        ui.label(format!(
            "Notation bytes: {}..{}",
            record.notation_range.start, record.notation_range.end
        ));
        ui.label(format!(
            "Record bytes: {}..{}",
            record.full_range.start, record.full_range.end
        ));
        match &record.application_data_range {
            Some(range) => {
                ui.label(format!("Application data: {} bytes", range.len()));
            }
            None => {
                ui.label("Application data: none");
            }
        }
        let label = ui.label(format!("Record {} notation", record.record_id));
        let mut text = record.notation.as_str();
        egui::ScrollArea::both()
            .id_salt(("smiles-notation", self.sequence))
            .max_height(180.0)
            .show(ui, |ui| {
                let mut output = egui::TextEdit::multiline(&mut text)
                    .desired_rows(3)
                    .desired_width(ui.available_width())
                    .show(ui);
                output.state.clear_undoer();
                let id = output.response.id;
                output.state.store(ui.ctx(), id);
                output.response.labelled_by(label.id);
                ui.ctx()
                    .accesskit_node_builder(id, |builder| builder.set_read_only());
                self.text_id = Some(id);
            });
    }
}
impl Drop for SmilesRecordsWorkflow {
    fn drop(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
    }
}

#[cfg(test)]
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse the original bounded temporary-directory helper for finite workflow gates"
)]
mod support;
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::file_workflow::FileWorkflow;
    use bondline_commands::{Cancellation, Engine, SmilesRecordInspect};
    use bondline_mcp::{CommandExecutor, SharedEngine};
    use eframe::egui;
    use serde_json::json;
    use std::{
        sync::{Arc, Mutex, mpsc},
        time::{Duration, Instant},
    };

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
    const WAIT: Duration = Duration::from_secs(5);
    const SOURCE: &[u8] = b"CC first\r\n\n \xffignored\n[13CH3][OH] \0\xffPRIVATE\nO last";

    struct Gate {
        workflow: SmilesRecordsWorkflow,
        engine: SharedEngine,
        executor: CommandExecutor,
        context: egui::Context,
        release: Option<mpsc::SyncSender<()>>,
    }
    impl Gate {
        fn published(directory: &support::FileTestDirectory) -> TestResult<Self> {
            let input = directory.path("finite-record.smiles");
            std::fs::write(&input, SOURCE)?;
            let mut engine = Engine::default();
            engine.set_ui_file_host(true);
            engine.dispatch("document.open_smiles", json!({"path":input}))?;
            let context = egui::Context::default();
            let mut workflow = SmilesRecordsWorkflow::default();
            workflow.sync(&engine, &context);
            workflow.ordinal = 2.0;
            assert!(workflow.request(&engine, &context));
            let key = workflow.pending.take().ok_or("pending exact query")?;
            let shared = Arc::new(Mutex::new(engine));
            let executor = CommandExecutor::new(Arc::clone(&shared));
            let work = executor
                .prepare_ui_smiles(
                    &key.stamp,
                    key.close_epoch,
                    SmilesRecordInspect {
                        expected_revision: key.revision,
                        record_id: key.ordinal,
                    },
                )
                .map_err(|e| e.message)?;
            let cancel = Cancellation::default();
            let worker_cancel = cancel.clone();
            let (sender, result) = mpsc::sync_channel(1);
            let (published, publication) = mpsc::sync_channel(1);
            let (release, wait) = mpsc::sync_channel(1);
            let worker = std::thread::spawn(move || {
                let output = work
                    .complete_blocking(&worker_cancel)
                    .map_err(|e| e.message)
                    .and_then(|result| {
                        result
                            .smiles_record
                            .ok_or_else(|| "missing real query record".to_owned())
                    });
                let _ = sender.send(output);
                let _ = published.send(());
                let _ = wait.recv_timeout(WAIT);
            });
            workflow.job = Some(RecordJob {
                key,
                cancel,
                result,
                worker,
            });
            let gate = Self {
                workflow,
                engine: shared,
                executor,
                context,
                release: Some(release),
            };
            publication.recv_timeout(WAIT)?;
            assert!(
                !gate.executor.has_active_work(),
                "Core completion already returned"
            );
            assert!(
                gate.workflow
                    .job
                    .as_ref()
                    .is_some_and(|job| !job.worker.is_finished())
            );
            Ok(gate)
        }
        fn release(&mut self) -> TestResult {
            self.release
                .take()
                .ok_or("gate already released")?
                .send(())?;
            Ok(())
        }
    }
    impl Drop for Gate {
        fn drop(&mut self) {
            if let Some(job) = &self.workflow.job {
                job.cancel.cancel();
            }
            if let Some(release) = self.release.take() {
                let _ = release.send(());
            }
            if let Some(job) = self.workflow.job.take() {
                let _ = job.worker.join();
            }
        }
    }

    #[test]
    fn published_record_waits_for_thread_retirement_and_cannot_bypass_native_close() -> TestResult {
        let directory = support::FileTestDirectory::new()?;
        let mut gate = Gate::published(&directory)?;
        let mut files = FileWorkflow::new(directory.path("."));
        {
            let mut engine = gate.engine.lock().map_err(|_| "engine")?;
            gate.workflow.sync(&engine, &gate.context);
            assert!(gate.workflow.is_busy());
            assert!(
                gate.workflow.record.is_none(),
                "published channel is not thread retirement"
            );
            let mut error = None;
            files.request_close(&mut engine, &mut error);
            assert!(error.is_none());
            gate.workflow.sync(&engine, &gate.context);
            assert_eq!(
                engine.state().close,
                bondline_commands::CloseState::Approved
            );
            assert!(!files.complete_close_if_idle(
                &mut engine,
                gate.executor.has_active_work() || gate.workflow.is_busy(),
                false
            ));
            assert!(
                gate.workflow
                    .job
                    .as_ref()
                    .is_some_and(|job| job.cancel.is_cancelled())
            );
            assert_eq!(engine.content().smiles_file()?.as_bytes(), SOURCE);
        }
        gate.release()?;
        let deadline = Instant::now() + WAIT;
        while gate.workflow.is_busy() {
            let engine = gate.engine.lock().map_err(|_| "engine")?;
            gate.workflow.sync(&engine, &gate.context);
            if Instant::now() >= deadline {
                return Err("record worker failed to retire".into());
            }
            drop(engine);
            std::thread::yield_now();
        }
        assert!(
            gate.workflow.record.is_none(),
            "closed intent cannot install late notation"
        );
        let mut engine = gate.engine.lock().map_err(|_| "engine")?;
        assert!(files.complete_close_if_idle(&mut engine, false, false));
        assert_eq!(
            engine.state().close,
            bondline_commands::CloseState::Committed
        );
        assert_eq!(engine.content().smiles_file()?.as_bytes(), SOURCE);
        Ok(())
    }

    #[test]
    fn poisoned_engine_deactivation_retires_cancelled_actual_job_without_installing_text()
    -> TestResult {
        let directory = support::FileTestDirectory::new()?;
        let mut gate = Gate::published(&directory)?;
        let shared = Arc::clone(&gate.engine);
        // Safe test-only unwind while holding the mutex creates a real poisoned Engine.
        let poisoned = std::thread::spawn(move || {
            let Ok(_guard) = shared.lock() else {
                return;
            };
            std::panic::resume_unwind(Box::new("intentional finite lifecycle test poison"));
        })
        .join();
        assert!(poisoned.is_err() && gate.engine.lock().is_err());
        gate.workflow.deactivate(&gate.context);
        assert!(
            gate.workflow.is_busy(),
            "unretired thread still blocks host close"
        );
        assert!(gate.workflow.record.is_none() && gate.workflow.pending.is_none());
        gate.release()?;
        let deadline = Instant::now() + WAIT;
        while gate.workflow.is_busy() {
            gate.workflow.deactivate(&gate.context);
            if Instant::now() >= deadline {
                return Err("engine-free record cleanup did not retire".into());
            }
            std::thread::yield_now();
        }
        assert!(gate.workflow.job.is_none() && gate.workflow.record.is_none());
        assert!(gate.engine.lock().is_err());
        Ok(())
    }
}
