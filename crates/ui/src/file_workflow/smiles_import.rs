//! Explicit selected/all-record import without replacing or discarding source bytes.

use super::*;
use crate::sdf_draft_text::SdfDraftText;
use bondline_commands::{ImportSmilesRecords, SmilesRecordSelection};
use bondline_mcp::{UiSmilesRecordsContext, UiSmilesRecordsResult};
use bondline_model::PageSize;
use std::thread::JoinHandle;

struct Values {
    all: bool,
    record: SdfDraftText,
    length: SdfDraftText,
    x: SdfDraftText,
    y: SdfDraftText,
    width: SdfDraftText,
    height: SdfDraftText,
}

impl Values {
    fn new() -> Result<Self, String> {
        let text = |value: &str| SdfDraftText::new(value).map_err(|error| error.to_string());
        Ok(Self {
            all: false,
            record: text("1")?,
            length: text("48")?,
            x: text("72")?,
            y: text("72")?,
            width: text(&PageSize::A4.width_pt().to_string())?,
            height: text(&PageSize::A4.height_pt().to_string())?,
        })
    }

    fn params(&self, revision: u64) -> Result<ImportSmilesRecords, String> {
        let number = |value: &SdfDraftText, name: &str| {
            value
                .as_str()
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or_else(|| format!("{name} must be a finite number."))
        };
        let selection = if self.all {
            SmilesRecordSelection::All {}
        } else {
            let value = self.record.as_str();
            let record_id = (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
                .then(|| value.parse::<u32>().ok())
                .flatten()
                .filter(|value| *value > 0)
                .ok_or_else(|| {
                    "Import record number must be a positive 32-bit integer.".to_owned()
                })?;
            SmilesRecordSelection::Record { record_id }
        };
        Ok(ImportSmilesRecords {
            expected_revision: revision,
            selection,
            bond_length_pt: number(&self.length, "Bond length (pt)")?,
            origin_x_pt: number(&self.x, "Origin X (pt)")?,
            origin_y_pt: number(&self.y, "Origin Y (pt)")?,
            page_width_pt: number(&self.width, "Page width (pt)")?,
            page_height_pt: number(&self.height, "Page height (pt)")?,
        })
    }
}

struct Form {
    key: u64,
    accepted: Option<u64>,
    context: UiSmilesRecordsContext,
    revision: u64,
    values: Values,
    detached: bool,
    message: Option<String>,
}

struct Pending {
    context: UiSmilesRecordsContext,
    params: ImportSmilesRecords,
}

struct Job {
    context: UiSmilesRecordsContext,
    cancel: Cancellation,
    user_cancelled: bool,
    result: Receiver<Result<UiSmilesRecordsResult, ErrorInfo>>,
    worker: JoinHandle<()>,
}

#[derive(Clone, Copy)]
enum Action {
    Selection(bool),
    Import,
    Cancel,
}

#[derive(Default)]
pub(super) struct ImportWorkflow {
    form: Option<Form>,
    pending: Option<Pending>,
    job: Option<Job>,
    actions: Vec<(usize, u64, Action)>,
}

impl ImportWorkflow {
    pub(super) fn present(&self) -> bool {
        self.form.is_some()
    }

    pub(super) fn busy(&self) -> bool {
        self.pending.is_some() || self.job.is_some()
    }

    pub(super) fn cancel(&self) {
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
    }
}

fn field_id(key: u64, index: usize) -> egui::Id {
    egui::Id::new(("smiles-import-field", key, index))
}

impl FileWorkflow {
    pub(crate) fn import_present(&self) -> bool {
        self.import.present()
    }

    pub(super) fn open_import(&mut self, id: u64, engine: &mut Engine, error: &mut Option<String>) {
        let opened = (|| {
            Ok::<_, String>(Form {
                key: id,
                accepted: Some(id),
                context: UiSmilesRecordsContext::capture_import(engine, id)
                    .map_err(|failure| failure.message)?,
                revision: engine.state().revision,
                values: Values::new()?,
                detached: false,
                message: None,
            })
        })();
        match opened {
            Ok(form) => self.import.form = Some(form),
            Err(message) => {
                self.cancel_picker(engine, error);
                *error = Some(message);
            }
        }
    }

    pub(crate) fn begin_import_ui(
        &mut self,
        context: &egui::Context,
        engine: Option<&Engine>,
        events: &[egui::Event],
    ) {
        self.import.actions.clear();
        let busy = self.import.busy();
        let Some(form) = &mut self.import.form else {
            return;
        };
        if !busy && !engine.is_some_and(|engine| form.context.is_current(engine)) {
            form.detached = true;
        }
        let key = form.key;
        let can_import =
            !busy && !form.detached && engine.is_some_and(|engine| form.context.is_current(engine));
        let mut actions = Vec::new();
        egui::Window::new("Import SMILES records")
            .id(egui::Id::new(("bondline-smiles-import", key)))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .order(egui::Order::Foreground)
            .default_width(470.0)
            .collapsible(false)
            .resizable(false)
            .show(context, |ui| {
                ui.label("Create an editable drawing while retaining the complete SMILES file. Generated coordinates are not stored in SMILES.");
                ui.add_enabled_ui(can_import, |ui| {
                    ui.horizontal(|ui| {
                        for (all, label) in [(false, "One record"), (true, "All records")] {
                            let response = ui.add(egui::RadioButton::new(form.values.all == all, label));
                            if response.clicked()
                                && let Some(index) = events.iter().position(|event| crate::activation_event(&response, event))
                            {
                                actions.push((index, key, Action::Selection(all)));
                            }
                        }
                    });
                });
                let mut failure = smiles_paste::field(
                    ui, field_id(key, 0), "Import record number", &mut form.values.record,
                    false, !busy && !form.values.all,
                );
                egui::Grid::new(("smiles-import-geometry", key)).num_columns(2).show(ui, |ui| {
                    for (index, (label, text)) in [
                        ("Bond length (pt)", &mut form.values.length),
                        ("Origin X (pt)", &mut form.values.x),
                        ("Origin Y (pt)", &mut form.values.y),
                        ("Page width (pt)", &mut form.values.width),
                        ("Page height (pt)", &mut form.values.height),
                    ].into_iter().enumerate() {
                        failure = smiles_paste::field(ui, field_id(key, index + 1), label, text, false, !busy)
                            .or_else(|| failure.take());
                        ui.end_row();
                    }
                });
                if let Some(failure) = failure {
                    form.message = Some(failure);
                }
                if form.detached {
                    ui.colored_label(egui::Color32::DARK_RED, "The document context changed. Copy these values or Cancel; they cannot be imported here.");
                }
                if let Some(message) = &form.message {
                    ui.colored_label(egui::Color32::DARK_RED, message);
                }
                if busy {
                    ui.spinner();
                    ui.label("Importing SMILES records…");
                }
                ui.horizontal(|ui| {
                    for (label, enabled, action) in [("Import", can_import, Action::Import), ("Cancel", true, Action::Cancel)] {
                        let response = ui.add_enabled(enabled, egui::Button::new(label));
                        if response.clicked()
                            && let Some(index) = events.iter().position(|event| crate::activation_event(&response, event))
                        {
                            actions.push((index, key, action));
                        }
                    }
                });
            });
        if engine.is_none() {
            if actions.iter().any(|(_, _, action)| matches!(action, Action::Cancel))
                || events.iter().any(|event| matches!(event, egui::Event::Key { key: egui::Key::Escape, pressed: true, repeat: false, modifiers, .. } if modifiers.is_none()))
            {
                self.cancel_import(None, context);
            }
        } else {
            self.import.actions = actions;
        }
        if busy {
            context.request_repaint_after(Duration::from_millis(40));
        }
    }

    pub(crate) fn replay_import(
        &mut self,
        index: usize,
        event: &egui::Event,
        engine: &mut Engine,
        error: &mut Option<String>,
        context: &egui::Context,
    ) {
        let Some(key) = self.import.form.as_ref().map(|form| form.key) else {
            return;
        };
        let mut actions = self
            .import
            .actions
            .iter()
            .filter(|(at, owner, _)| *at == index && *owner == key)
            .map(|(_, _, action)| *action)
            .collect::<Vec<_>>();
        if matches!(event, egui::Event::Key { key: egui::Key::Escape, pressed: true, repeat: false, modifiers, .. } if modifiers.is_none())
        {
            actions.push(Action::Cancel);
        }
        for action in actions {
            match action {
                Action::Selection(all) => {
                    if !self.import.busy()
                        && let Some(form) = &mut self.import.form
                        && !form.detached
                        && form.context.is_current(engine)
                    {
                        form.values.all = all;
                    }
                }
                Action::Import => self.queue_import(engine, error),
                Action::Cancel => self.cancel_import(Some(&mut *engine), context),
            }
            context.request_repaint();
        }
    }

    fn queue_import(&mut self, engine: &mut Engine, error: &mut Option<String>) {
        if self.import.busy() {
            return;
        }
        let Some(form) = &mut self.import.form else {
            return;
        };
        if form.detached || !form.context.is_current(engine) {
            form.detached = true;
            return;
        }
        let params = match form.values.params(form.revision) {
            Ok(params) => params,
            Err(message) => {
                form.message = Some(message);
                return;
            }
        };
        if form.accepted.is_none() {
            let request = engine
                .execute(Command::UiFile(UiFile {
                    expected_revision: form.revision,
                    action: UiFileAction::ImportSmilesRecords,
                    request_id: None,
                    close_epoch: None,
                }))
                .and_then(|result| {
                    let id = result
                        .ui_file_operation
                        .ok_or(bondline_commands::Error::UiStaleRequest)?;
                    if !matches!(engine.claim_ui_file(id)?, UiFilePlan::SmilesImport) {
                        return Err(bondline_commands::Error::UiStaleRequest);
                    }
                    Ok(id)
                });
            match request {
                Ok(id) => match UiSmilesRecordsContext::capture_import(engine, id) {
                    Ok(context) => {
                        form.context = context;
                        form.accepted = Some(id);
                    }
                    Err(failure) => {
                        form.message = Some(failure.message);
                        form.detached = true;
                        return;
                    }
                },
                Err(failure) => {
                    form.message = Some(failure.to_string());
                    return;
                }
            }
        }
        self.import.pending = Some(Pending {
            context: form.context.clone(),
            params,
        });
        form.message = None;
        *error = None;
    }

    fn close_import_form(&mut self, context: &egui::Context) {
        if let Some(form) = self.import.form.take() {
            for index in 0..6 {
                let id = field_id(form.key, index);
                context.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(id));
                context.memory_mut(|memory| memory.surrender_focus(id));
            }
            if self.request_id == form.accepted || self.request_id == Some(form.key) {
                self.request_id = None;
            }
        }
        self.import.pending = None;
        self.import.actions.clear();
        context.request_discard("SMILES import form closed");
    }

    fn cancel_import(&mut self, engine: Option<&mut Engine>, context: &egui::Context) {
        self.import.pending = None;
        if let Some(job) = &mut self.import.job {
            job.user_cancelled = true;
            job.cancel.cancel();
            return;
        }
        if let (Some(engine), Some(id)) = (
            engine,
            self.import.form.as_ref().and_then(|form| form.accepted),
        ) && engine
            .state()
            .ui_file
            .request
            .as_ref()
            .is_some_and(|request| {
                request.id == id && request.action == UiFileAction::ImportSmilesRecords
            })
        {
            let _ = engine.execute(Command::UiFile(UiFile {
                expected_revision: engine.state().revision,
                action: UiFileAction::Cancel,
                request_id: Some(id),
                close_epoch: None,
            }));
        }
        self.close_import_form(context);
    }

    pub(crate) fn poll_import(
        &mut self,
        engine: Option<&Engine>,
        context: &egui::Context,
        error: &mut Option<String>,
    ) {
        if let Some(job) = &self.import.job {
            if !engine.is_some_and(|engine| job.context.is_current(engine)) {
                job.cancel.cancel();
            }
            if !job.worker.is_finished() {
                context.request_repaint_after(Duration::from_millis(40));
                return;
            }
        }
        if let Some(job) = self.import.job.take() {
            let joined = job.worker.join();
            if let Some(form) = &mut self.import.form {
                form.accepted = None;
            }
            match job.result.try_recv() {
                Ok(Ok(result)) => {
                    if engine.is_some_and(|engine| result.is_current(engine)) {
                        *error = job.user_cancelled.then(|| {
                            "The records were imported before cancellation completed.".into()
                        });
                        self.close_import_form(context);
                    } else if let Some(form) = &mut self.import.form {
                        form.detached = true;
                        form.message = Some("The records were imported before the document context changed. These values cannot be imported again here.".into());
                    }
                }
                Ok(Err(failure)) => {
                    if job.user_cancelled {
                        self.close_import_form(context);
                    } else if let Some(form) = &mut self.import.form {
                        form.message = Some(failure.message);
                    }
                }
                Err(_) => {
                    if let Some(form) = &mut self.import.form {
                        form.message = Some(
                            if joined.is_err() {
                                "The SMILES import worker did not complete."
                            } else {
                                "The SMILES import worker stopped without a result."
                            }
                            .into(),
                        );
                    }
                }
            }
            context.request_repaint();
        }
        if let Some(form) = &mut self.import.form
            && !engine.is_some_and(|engine| form.context.is_current(engine))
        {
            form.detached = true;
            self.import.pending = None;
        }
    }

    pub(crate) fn start_pending_import(
        &mut self,
        executor: &CommandExecutor,
        context: &egui::Context,
    ) {
        if self.import.job.is_some() {
            return;
        }
        let Some(pending) = self.import.pending.take() else {
            return;
        };
        let work = match executor.prepare_ui_smiles_import(&pending.context, pending.params.clone())
        {
            Ok(work) => work,
            Err(failure) if failure.code == "busy" => {
                self.import.pending = Some(pending);
                context.request_repaint_after(Duration::from_millis(40));
                return;
            }
            Err(failure) => {
                if let Some(form) = &mut self.import.form {
                    form.message = Some(failure.message);
                }
                context.request_repaint();
                return;
            }
        };
        let Some(form) = &mut self.import.form else {
            return;
        };
        form.accepted = None;
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let repaint = context.clone();
        let (sender, result) = mpsc::sync_channel(1);
        match std::thread::Builder::new()
            .name("bondline-smiles-import".into())
            .spawn(move || {
                let _ = sender.send(work.complete_blocking(&worker_cancel));
                repaint.request_repaint();
            }) {
            Ok(worker) => {
                self.import.job = Some(Job {
                    context: pending.context,
                    cancel,
                    user_cancelled: false,
                    result,
                    worker,
                })
            }
            Err(failure) => {
                form.message = Some(format!(
                    "Cannot start SMILES import: {failure}. No worker ran."
                ))
            }
        }
    }

    pub(super) fn import_close(&mut self) {
        self.import.pending = None;
        self.import.cancel();
        if let Some(form) = &mut self.import.form {
            form.detached = true;
        }
    }

    pub(crate) fn deactivate_import(
        &mut self,
        context: &egui::Context,
        error: &mut Option<String>,
    ) {
        self.import_close();
        self.poll_import(None, context, error);
    }
}
