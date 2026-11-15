//! Explicit, read-only SDF field pages using the shared registered executor.

use bondline_commands::{
    Cancellation, CaptureStamp, CloseState, Command, Engine, SdfRecordInspect, SdfRecordInspection,
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
struct FieldsKey {
    stamp: CaptureStamp,
    close_epoch: u64,
    revision: u64,
    record_id: u32,
    offset: usize,
    sequence: u64,
}

impl FieldsKey {
    fn current(&self, engine: &Engine) -> bool {
        let state = engine.state();
        matches!(engine.content(), StoredContent::Sdf { .. })
            && state.close == CloseState::Open
            && state.close_epoch == self.close_epoch
            && engine.capture_stamp_matches(&self.stamp)
    }
}

struct FieldsPage {
    record: SdfRecordInspection,
    values: Vec<Option<String>>,
}

impl FieldsPage {
    fn new(record: SdfRecordInspection) -> Self {
        let values = record
            .fields
            .iter()
            .map(|field| field.value_lines.as_ref().map(|lines| lines.join("\n")))
            .collect();
        Self { record, values }
    }
}

struct FieldsJob {
    key: FieldsKey,
    cancel: Cancellation,
    result: Receiver<Result<FieldsPage, String>>,
    worker: JoinHandle<()>,
}

#[derive(Default)]
pub(crate) struct SdfFieldsWorkflow {
    sequence: u64,
    desired: Option<FieldsKey>,
    pending: Option<FieldsKey>,
    job: Option<FieldsJob>,
    page: Option<FieldsPage>,
    error: Option<String>,
    previous_offsets: Vec<usize>,
}

impl SdfFieldsWorkflow {
    pub(crate) fn is_busy(&self) -> bool {
        self.pending.is_some() || self.job.is_some()
    }

    fn clear(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
        self.desired = None;
        self.pending = None;
        self.page = None;
        self.error = None;
    }

    pub(crate) fn inspect(&mut self, engine: &Engine, record_id: u32) {
        self.previous_offsets.clear();
        self.request(engine, record_id, 0);
    }

    fn request(&mut self, engine: &Engine, record_id: u32, offset: usize) -> bool {
        let state = engine.state();
        if !matches!(engine.content(), StoredContent::Sdf { .. }) || state.close != CloseState::Open
        {
            return false;
        }
        let Some(sequence) = self.sequence.checked_add(1) else {
            self.error =
                Some("The field request limit was reached. Reopen Bondline to continue.".into());
            return false;
        };
        self.clear();
        self.sequence = sequence;
        let key = FieldsKey {
            stamp: engine.capture_stamp(),
            close_epoch: state.close_epoch,
            revision: state.revision,
            record_id,
            offset,
            sequence,
        };
        self.desired = Some(key.clone());
        self.pending = Some(key);
        true
    }

    pub(crate) fn sync(&mut self, engine: &Engine, context: &egui::Context) {
        if !matches!(engine.content(), StoredContent::Sdf { .. })
            || engine.state().close != CloseState::Open
            || self
                .desired
                .as_ref()
                .is_some_and(|key| !key.current(engine))
        {
            self.clear();
            self.previous_offsets.clear();
        }
        self.retire(Some(engine), context);
    }

    pub(crate) fn deactivate(&mut self, context: &egui::Context) {
        self.clear();
        self.previous_offsets.clear();
        self.retire(None, context);
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
        let FieldsJob {
            key,
            cancel,
            result,
            worker,
        } = job;
        let result = match worker.join() {
            Ok(()) => result
                .try_recv()
                .unwrap_or_else(|_| Err("The field worker stopped without a result.".into())),
            Err(_) => Err("The field worker did not complete.".into()),
        };
        if cancel.is_cancelled()
            || self.desired.as_ref() != Some(&key)
            || !engine.is_some_and(|engine| key.current(engine))
        {
            return;
        }
        match result {
            Ok(page) => self.page = Some(page),
            Err(message) => self.error = Some(message),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, engine: &Engine) {
        let Some(key) = self.desired.clone() else {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::DARK_RED, error);
            }
            return;
        };
        if !key.current(engine) {
            return;
        }
        ui.separator();
        ui.heading(format!("Record {} fields", key.record_id));
        ui.weak("Read-only field values");
        if ui.small_button("Close fields").clicked() {
            self.deactivate(ui.ctx());
            return;
        }
        if let Some(message) = &self.error {
            ui.colored_label(
                egui::Color32::DARK_RED,
                format!("Fields unavailable: {message}"),
            );
            if ui
                .add_enabled(!self.is_busy(), egui::Button::new("Retry fields"))
                .clicked()
            {
                self.request(engine, key.record_id, key.offset);
            }
            return;
        }
        let Some(page) = &self.page else {
            ui.spinner();
            ui.label("Reading fields…");
            if ui.small_button("Cancel field query").clicked() {
                if let Some(job) = &self.job {
                    job.cancel.cancel();
                }
                self.pending = None;
                self.error = Some("Field query cancelled.".into());
            }
            return;
        };
        let record = &page.record;
        ui.label(format!(
            "{} fields · {}",
            record.field_count, record.mol_version
        ));
        if record.field_count == 0 {
            ui.label("No data fields");
        }
        let mut navigation = None;
        ui.horizontal_wrapped(|ui| {
            if let Some(offset) = self.previous_offsets.last()
                && ui.small_button("Previous fields").clicked()
            {
                navigation = Some((*offset, false));
            }
            if let Some(offset) = record.next_field_offset
                && ui.small_button("Next fields").clicked()
            {
                navigation = Some((offset, true));
            }
        });
        for (field, value) in record.fields.iter().zip(&page.values) {
            let name = field.name.clone().unwrap_or_else(|| {
                field
                    .dt_number
                    .as_ref()
                    .map_or_else(|| "Unnamed".into(), |dt| format!("DT{dt}"))
            });
            let caption: String = name.chars().take(80).collect();
            let number = field.index.saturating_add(1);
            egui::CollapsingHeader::new(format!("Field {number}: {caption}"))
                .id_salt((key.sequence, field.index))
                .show(ui, |ui| {
                    ui.label(&field.header);
                    let unit = if field.value_line_count == 1 {
                        "line"
                    } else {
                        "lines"
                    };
                    ui.label(format!(
                        "Value: {} bytes, {} {unit}",
                        field.value_byte_length, field.value_line_count
                    ));
                    if let Some(value) = value {
                        let label =
                            ui.label(format!("Record {} field {number} value", key.record_id));
                        let mut text = value.as_str();
                        egui::ScrollArea::vertical()
                            .id_salt(("sdf-field-value", key.sequence, field.index))
                            .max_height(120.0)
                            .show(ui, |ui| {
                                let mut output = egui::TextEdit::multiline(&mut text)
                                    .desired_rows(2)
                                    .desired_width(ui.available_width())
                                    .show(ui);
                                // egui records immutable buffers in per-widget history when focused.
                                output.state.clear_undoer();
                                let id = output.response.id;
                                output.state.store(ui.ctx(), id);
                                output.response.labelled_by(label.id);
                                ui.ctx().accesskit_node_builder(id, |builder| {
                                    builder.set_read_only();
                                });
                            });
                    } else {
                        ui.label(
                            "Value omitted by the inspection limit; it is not an empty value.",
                        );
                    }
                });
        }
        if let Some((offset, forward)) = navigation {
            let previous = record.field_offset;
            if self.request(engine, key.record_id, offset) {
                if forward {
                    self.previous_offsets.push(previous);
                } else {
                    self.previous_offsets.pop();
                }
            }
        }
    }

    /// Pending intent is already busy; acquire the shared registration after frame unlock.
    pub(crate) fn start_pending(&mut self, executor: &CommandExecutor, context: &egui::Context) {
        if self.job.is_some() {
            return;
        }
        let Some(key) = self.pending.take() else {
            return;
        };
        let command = Command::SdfRecordInspect(SdfRecordInspect {
            expected_revision: key.revision,
            record_id: key.record_id,
            field_offset: key.offset,
            field_limit: 64,
        });
        let work = match executor.prepare_ui_sdf(&key.stamp, key.close_epoch, command) {
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
                    self.clear();
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
            .name("bondline-fields".into())
            .spawn(move || {
                let output = work
                    .complete_blocking(&worker_cancel)
                    .map_err(|error| error.message)
                    .and_then(|result| {
                        result
                            .sdf_record
                            .ok_or_else(|| "The field command returned no record.".to_owned())
                    })
                    .map(FieldsPage::new);
                let _ = sender.send(output);
                worker_context.request_repaint();
            }) {
            Ok(worker) => {
                self.job = Some(FieldsJob {
                    key,
                    cancel,
                    result,
                    worker,
                })
            }
            Err(_) => {
                self.error = Some("The field worker could not start. Try again.".into());
                context.request_repaint();
            }
        }
    }
}

impl Drop for SdfFieldsWorkflow {
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
    reason = "reuse the original bounded temporary-directory helper"
)]
mod support;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deactivation_retires_a_finished_job_without_an_engine_guard()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Engine::default();
        let key = FieldsKey {
            stamp: engine.capture_stamp(),
            close_epoch: 0,
            revision: 0,
            record_id: 1,
            offset: 0,
            sequence: 1,
        };
        let (sender, result) = mpsc::sync_channel(1);
        drop(sender);
        let worker = std::thread::spawn(|| {});
        while !worker.is_finished() {
            std::thread::yield_now();
        }
        let mut fields = SdfFieldsWorkflow::default();
        fields.job = Some(FieldsJob {
            key,
            cancel: Cancellation::default(),
            result,
            worker,
        });
        fields.deactivate(&egui::Context::default());
        assert!(
            !fields.is_busy(),
            "finished cancelled work must retire even when the Engine is unavailable"
        );
        assert!(fields.job.is_none() && fields.page.is_none() && fields.error.is_none());
        Ok(())
    }

    #[test]
    fn queued_fields_stay_busy_until_thread_retirement_and_cannot_survive_source_replacement()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = support::FileTestDirectory::new()?;
        let source = directory.path("original.sdf");
        std::fs::write(
            &source,
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"),
        )?;
        let mut engine = bondline_commands::Engine::default();
        engine.dispatch("document.open_sdf", serde_json::json!({"path":source}))?;
        let page = engine
            .dispatch(
                "sdf.record.inspect",
                serde_json::json!({"expected_revision":1,"record_id":1}),
            )?
            .sdf_record
            .ok_or("record result")?;
        assert_eq!(
            page.fields.get(2).ok_or("third field")?.name.as_deref(),
            Some("DUP")
        );
        let mut fields = SdfFieldsWorkflow::default();
        fields.request(&engine, 1, 0);
        let key = fields.pending.take().ok_or("pending request")?;
        let (sender, result) = std::sync::mpsc::sync_channel(1);
        let (published, publication) = std::sync::mpsc::sync_channel(1);
        let (release, gate) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            let _ = sender.send(Ok(FieldsPage::new(page)));
            let _ = published.send(());
            let _ = gate.recv_timeout(std::time::Duration::from_secs(5));
        });
        fields.job = Some(FieldsJob {
            key,
            cancel: bondline_commands::Cancellation::default(),
            result,
            worker,
        });
        publication.recv_timeout(std::time::Duration::from_secs(5))?;
        let context = eframe::egui::Context::default();
        fields.sync(&engine, &context);
        assert!(fields.is_busy(), "queued response is not retired work");
        assert!(
            fields.page.is_none(),
            "unfinished worker cannot install fields"
        );

        let replacement = directory.path("replacement.smi");
        std::fs::write(&replacement, b"CN\tprivate replacement\n")?;
        engine.dispatch(
            "document.open_smiles",
            serde_json::json!({"path":replacement}),
        )?;
        let before = serde_json::to_value(engine.state())?;
        fields.sync(&engine, &context);
        assert!(fields.is_busy(), "stale work still blocks host shutdown");
        assert!(fields.desired.is_none() && fields.pending.is_none() && fields.page.is_none());
        assert!(
            fields
                .job
                .as_ref()
                .is_some_and(|job| job.cancel.is_cancelled())
        );
        release.send(())?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while fields.is_busy() {
            fields.sync(&engine, &context);
            if std::time::Instant::now() >= deadline {
                return Err("field worker did not retire".into());
            }
            std::thread::yield_now();
        }
        assert!(fields.page.is_none() && fields.error.is_none());
        assert_eq!(serde_json::to_value(engine.state())?, before);
        Ok(())
    }
}
