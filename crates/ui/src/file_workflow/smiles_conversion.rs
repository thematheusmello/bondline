//! One restricted conversion form and its registered create-only worker.

use super::*;
use crate::sdf_draft_text::SdfDraftText;
use bondline_commands::{CaptureStamp, ConversionReceipt, ConvertSmilesToCdxml};
use std::thread::JoinHandle;

#[cfg(test)]
#[path = "../conversion_worker_tests.rs"]
mod worker_tests;

const MAX_TEXT_BYTES: usize = 32_768;

#[derive(Clone)]
struct Context {
    stamp: CaptureStamp,
    revision: u64,
    close_epoch: u64,
    file_epoch: u64,
    path: Option<String>,
    format: FileFormat,
}

impl Context {
    fn capture(engine: &Engine) -> Self {
        let state = engine.state();
        Self {
            stamp: engine.capture_stamp(),
            revision: state.revision,
            close_epoch: state.close_epoch,
            file_epoch: state.ui_file.epoch,
            path: state.file.path,
            format: state.file.format,
        }
    }

    fn current(&self, engine: &Engine) -> bool {
        let state = engine.state();
        state.close == CloseState::Open
            && state.close_epoch == self.close_epoch
            && engine.capture_stamp_matches(&self.stamp)
            && state.ui_file.epoch == self.file_epoch
            && state.file.path == self.path
            && state.file.format == self.format
    }
}

struct Values {
    input: SdfDraftText,
    output: SdfDraftText,
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
            input: text("")?,
            output: text("")?,
            length: text("48")?,
            x: text("72")?,
            y: text("72")?,
            width: text(&bondline_model::PageSize::A4.width_pt().to_string())?,
            height: text(&bondline_model::PageSize::A4.height_pt().to_string())?,
        })
    }

    fn params(&self, revision: u64) -> Result<ConvertSmilesToCdxml, String> {
        let path = |text: &SdfDraftText, name: &str| {
            let value = text.as_str();
            if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.contains('\0') {
                Err(format!(
                    "{name} must be a nonempty path without NUL, at most 32768 bytes."
                ))
            } else {
                Ok(value.to_owned())
            }
        };
        let number = |text: &SdfDraftText, name: &str| {
            text.as_str()
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or_else(|| format!("{name} must be a finite number."))
        };
        Ok(ConvertSmilesToCdxml {
            expected_revision: revision,
            input_path: path(&self.input, "Input file")?,
            output_path: path(&self.output, "Output file")?,
            bond_length_pt: number(&self.length, "Bond length (pt)")?,
            origin_x_pt: number(&self.x, "Origin X (pt)")?,
            origin_y_pt: number(&self.y, "Origin Y (pt)")?,
            page_width_pt: number(&self.width, "Page width (pt)")?,
            page_height_pt: number(&self.height, "Page height (pt)")?,
        })
    }
}

enum Outcome {
    Success {
        receipt: ConversionReceipt,
        historical: bool,
        cancelled: bool,
    },
    Failure(String),
}

struct Form {
    widget_key: u64,
    accepted: Option<u64>,
    context: Context,
    values: Values,
    outcome: Option<Outcome>,
    stale: bool,
}

#[derive(Clone, Copy)]
enum Browse {
    Input,
    Output,
}

struct Pending {
    id: u64,
    params: ConvertSmilesToCdxml,
}

struct Job {
    context: Context,
    cancel: Cancellation,
    result: Receiver<Result<CommandResult, ErrorInfo>>,
    worker: JoinHandle<()>,
}

#[derive(Clone, Copy)]
enum Action {
    Convert,
    BrowseInput,
    BrowseOutput,
    Cancel,
    Close,
}

#[derive(Default)]
pub(super) struct ConversionWorkflow {
    form: Option<Form>,
    pending: Option<Pending>,
    job: Option<Job>,
    browsing: Option<Browse>,
    actions: Vec<(usize, u64, Action)>,
}

impl ConversionWorkflow {
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
    egui::Id::new(("conversion-field", key, index))
}

fn reset_fields(context: &egui::Context, key: u64) {
    for index in 0..7 {
        let id = field_id(key, index);
        context.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(id));
        context.memory_mut(|memory| memory.surrender_focus(id));
    }
}

fn field(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: &str,
    text: &mut SdfDraftText,
    enabled: bool,
) -> Option<String> {
    let label = ui.label(label);
    let previous = text.as_str().to_owned();
    let prior = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
    let mut error = None;
    let (mut output, outcome) = text.edit(|buffer| {
        if enabled {
            let requested = ui.input(|input| {
                input
                    .accesskit_action_requests(id, egui::accesskit::Action::SetValue)
                    .last()
                    .and_then(|request| match &request.data {
                        Some(egui::accesskit::ActionData::Value(value)) => {
                            Some(if value.len() > MAX_TEXT_BYTES || value.contains('\0') {
                                Err("Field value exceeds 32768 UTF-8 bytes or contains NUL."
                                    .to_owned())
                            } else {
                                Ok(value.to_string())
                            })
                        }
                        _ => None,
                    })
            });
            match requested {
                Some(Ok(value)) => buffer.replace_with(&value),
                Some(Err(message)) => error = Some(message),
                None => {}
            }
        }
        let output = ui
            .add_enabled_ui(enabled, |ui| {
                egui::TextEdit::singleline(buffer)
                    .id(id)
                    .char_limit(usize::MAX)
                    .desired_width(ui.available_width())
                    .show(ui)
            })
            .inner;
        if enabled {
            if buffer.as_str().len() > MAX_TEXT_BYTES || buffer.as_str().contains('\0') {
                error = Some("Field value exceeds 32768 UTF-8 bytes or contains NUL.".into());
            }
            if error.is_some() {
                buffer.replace_with(&previous);
            }
        }
        output
    });
    if outcome.changed || error.is_some() || outcome.rejection.is_some() {
        ui.ctx().request_repaint();
    }
    output.response.labelled_by(label.id);
    if error.is_some() || outcome.rejection.is_some() {
        output.state.cursor = prior.cursor;
    }
    output.state.clear_undoer();
    output.state.store(ui.ctx(), id);
    ui.ctx().accesskit_node_builder(id, |node| {
        node.set_value(text.as_str());
        if enabled {
            node.add_action(egui::accesskit::Action::SetValue);
        }
    });
    ui.end_row();
    error.or_else(|| outcome.rejection.map(|error| error.to_string()))
}

impl FileWorkflow {
    pub(super) fn open_conversion(
        &mut self,
        id: u64,
        engine: &mut Engine,
        error: &mut Option<String>,
    ) {
        match Values::new() {
            Ok(values) => {
                self.conversion.form = Some(Form {
                    widget_key: id,
                    accepted: Some(id),
                    context: Context::capture(engine),
                    values,
                    outcome: None,
                    stale: false,
                })
            }
            Err(failure) => {
                self.cancel_picker(engine, error);
                *error = Some(failure);
            }
        }
    }

    pub(crate) fn begin_conversion_ui(
        &mut self,
        context: &egui::Context,
        engine: Option<&Engine>,
        events: &[egui::Event],
    ) {
        self.conversion.actions.clear();
        if self.conversion.job.is_none()
            && self.conversion.form.as_ref().is_some_and(|form| {
                form.accepted.is_some()
                    && !engine.is_some_and(|engine| form.context.current(engine))
            })
        {
            self.conversion.pending = None;
            self.close_conversion_form(context);
            return;
        }
        let busy = self.conversion.busy();
        let browsing = self.conversion.browsing.is_some();
        let Some(form) = &mut self.conversion.form else {
            return;
        };
        let key = form.widget_key;
        let enabled = !busy && !browsing && !form.stale && engine.is_some();
        let mut actions = Vec::new();
        let mut open = true;
        egui::Window::new("Convert SMILES file").id(egui::Id::new(("bondline-conversion",key)))
            .anchor(egui::Align2::CENTER_CENTER,egui::Vec2::ZERO).order(egui::Order::Foreground)
            .default_width(620.0).collapsible(false).resizable(false).open(&mut open).show(context, |ui| {
                ui.label("Supports one connected chain of 1–12 neutral carbon atoms with single bonds.");
                ui.label("The current document stays open. The output must be a new CDXML file; existing files are never overwritten.");
                let mut field_error = None;
                egui::Grid::new(("conversion-parameters",key)).num_columns(2).show(ui, |ui| {
                    for (index,(label,text)) in [("Input file",&mut form.values.input),("Output file",&mut form.values.output),
                        ("Bond length (pt)",&mut form.values.length),("Origin X (pt)",&mut form.values.x),("Origin Y (pt)",&mut form.values.y),
                        ("Page width (pt)",&mut form.values.width),("Page height (pt)",&mut form.values.height)].into_iter().enumerate() {
                        field_error = field(ui,field_id(key,index),label,text,enabled).or_else(|| field_error.take());
                    }
                });
                if let Some(error) = field_error { form.outcome = Some(Outcome::Failure(error)); }
                if form.stale { ui.colored_label(egui::Color32::DARK_RED,"The document or file context changed. Close this form and start a new conversion request."); }
                if let Some(outcome) = &form.outcome {
                    match outcome {
                        Outcome::Failure(message) => { ui.heading("Conversion failed"); ui.colored_label(egui::Color32::DARK_RED,message); }
                        Outcome::Success {receipt,historical,cancelled} => {
                            ui.heading("Conversion complete"); ui.label(&receipt.output.path);
                            ui.label(format!("{} {}, {} {}",receipt.atom_count,if receipt.atom_count==1 {"atom"} else {"atoms"},receipt.bond_count,if receipt.bond_count==1 {"bond"} else {"bonds"}));
                            ui.label("Close this form, then use File → Open to edit the created CDXML file.");
                            if *historical { ui.label("The document context changed while converting. The new file was created; the current document was not replaced."); }
                            if *cancelled { ui.label("The new file was created before cancellation completed."); }
                        }
                    }
                }
                if busy { ui.spinner(); ui.label("Converting…"); }
                ui.horizontal_wrapped(|ui| {
                    for (caption,allowed,action) in [("Browse input",enabled,Action::BrowseInput),("Browse output",enabled,Action::BrowseOutput),
                        ("Convert",enabled && !matches!(form.outcome,Some(Outcome::Success {..})),Action::Convert),
                        ("Cancel conversion",true,Action::Cancel),("Close conversion",!busy,Action::Close)] {
                        let response = ui.add_enabled(allowed,egui::Button::new(caption));
                        if response.clicked() && let Some(index) = events.iter().position(|event| crate::activation_event(&response,event)) { actions.push((index,key,action)); }
                    }
                });
            });
        if !open && let Some(index) = events.len().checked_sub(1) {
            actions.push((index, key, Action::Close));
        }
        let engine_free_close = engine.is_none()
            && !busy
            && actions
                .iter()
                .any(|(_, _, action)| matches!(action, Action::Close | Action::Cancel));
        self.conversion.actions = actions;
        if busy {
            context.request_repaint_after(Duration::from_millis(40));
        }
        if engine_free_close || (engine.is_none() && !busy && !open) {
            self.close_conversion_form(context);
        }
    }

    pub(crate) fn replay_conversion(
        &mut self,
        index: usize,
        event: &egui::Event,
        engine: &mut Engine,
        error: &mut Option<String>,
        context: &egui::Context,
    ) {
        let Some(key) = self.conversion.form.as_ref().map(|form| form.widget_key) else {
            return;
        };
        let mut actions = self
            .conversion
            .actions
            .iter()
            .filter(|(at, owner, _)| *at == index && *owner == key)
            .map(|(_, _, action)| *action)
            .collect::<Vec<_>>();
        if matches!(event,egui::Event::Key {key:egui::Key::Escape,pressed:true,repeat:false,modifiers,..} if modifiers.is_none())
        {
            actions.push(Action::Cancel);
        }
        for action in actions {
            match action {
                Action::Convert => {
                    self.rearm_conversion(engine);
                    let parameters = self
                        .conversion
                        .form
                        .as_ref()
                        .map(|form| form.values.params(form.context.revision));
                    match parameters {
                        Some(Ok(params)) => {
                            self.queue_conversion(params, engine, context);
                            if self.conversion.pending.is_some() {
                                *error = None;
                            }
                        }
                        Some(Err(message)) => {
                            if let Some(form) = &mut self.conversion.form {
                                form.outcome = Some(Outcome::Failure(message));
                            }
                        }
                        None => {}
                    }
                }
                Action::BrowseInput => self.browse_conversion(Browse::Input),
                Action::BrowseOutput => self.browse_conversion(Browse::Output),
                Action::Cancel | Action::Close => self.cancel_conversion(engine, error, context),
            }
            context.request_repaint();
        }
    }

    fn rearm_conversion(&mut self, engine: &mut Engine) {
        let Some(form) = &mut self.conversion.form else {
            return;
        };
        if form.accepted.is_some()
            || self.conversion.pending.is_some()
            || self.conversion.job.is_some()
        {
            return;
        }
        if form.stale || !form.context.current(engine) || engine.state().ui_file.request.is_some() {
            form.stale = true;
            return;
        }
        match engine.execute(Command::UiFile(UiFile {
            expected_revision: form.context.revision,
            action: UiFileAction::ConvertSmiles,
            request_id: None,
            close_epoch: None,
        })) {
            Ok(result) => {
                if let Some(id) = result.ui_file_operation {
                    match engine.claim_ui_file(id) {
                        Ok(UiFilePlan::SmilesConversion) => {
                            form.accepted = Some(id);
                            form.context = Context::capture(engine);
                            self.request_id = Some(id);
                        }
                        _ => {
                            form.stale = true;
                            form.outcome = Some(Outcome::Failure(
                                "The conversion request could not be accepted.".into(),
                            ));
                        }
                    }
                }
            }
            Err(error) => form.outcome = Some(Outcome::Failure(error.to_string())),
        }
    }

    pub(super) fn queue_conversion(
        &mut self,
        mut params: ConvertSmilesToCdxml,
        engine: &Engine,
        context: &egui::Context,
    ) {
        if self.conversion.busy() {
            return;
        }
        let Some(form) = &mut self.conversion.form else {
            return;
        };
        if form.stale || !form.context.current(engine) || form.accepted.is_none() {
            form.outcome=Some(Outcome::Failure("The conversion context is obsolete. Close this form and start a new conversion request.".into()));
            form.stale = true;
            return;
        }
        let Some(id) = form.accepted else { return };
        params.expected_revision = form.context.revision;
        self.conversion.pending = Some(Pending { id, params });
        form.outcome = None;
        context.request_repaint();
    }

    fn browse_conversion(&mut self, which: Browse) {
        if self.conversion.busy() {
            return;
        }
        let Some(form) = &self.conversion.form else {
            return;
        };
        if form.stale {
            return;
        }
        let (mode, format, title, value) = match which {
            Browse::Input => (
                FileMode::Open,
                FileFormat::Smiles,
                "Choose SMILES input",
                form.values.input.as_str(),
            ),
            Browse::Output => (
                FileMode::Save,
                FileFormat::Cdxml,
                "Choose new CDXML output",
                form.values.output.as_str(),
            ),
        };
        let path = std::path::Path::new(value);
        let directory = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map_or_else(|| self.directory.clone(), std::path::Path::to_owned);
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_owned();
        self.chooser
            .begin_selection(mode, directory, name, format, title);
        self.conversion.browsing = Some(which);
    }

    pub(super) fn conversion_picker(&mut self, context: &egui::Context) -> bool {
        let Some(which) = self.conversion.browsing else {
            return false;
        };
        match self.chooser.show(context, false) {
            Some(FileChooserEvent::Submit(action)) => {
                self.conversion.browsing = None;
                let path = match action {
                    FileAction::Open { path, .. } | FileAction::Save { path, .. } => path,
                };
                if let Some(form) = &mut self.conversion.form {
                    match SdfDraftText::new(&path) {
                        Ok(value) => match which {
                            Browse::Input => form.values.input = value,
                            Browse::Output => form.values.output = value,
                        },
                        Err(error) => form.outcome = Some(Outcome::Failure(error.to_string())),
                    }
                }
            }
            Some(FileChooserEvent::Cancel) => self.conversion.browsing = None,
            None => {}
        }
        true
    }

    fn close_conversion_form(&mut self, context: &egui::Context) {
        if let Some(form) = self.conversion.form.take() {
            reset_fields(context, form.widget_key);
        }
        self.conversion.browsing = None;
        self.chooser.cancel();
        context.request_discard("Conversion form closed");
    }

    fn cancel_conversion(
        &mut self,
        engine: &mut Engine,
        error: &mut Option<String>,
        context: &egui::Context,
    ) {
        self.conversion.pending = None;
        if let Some(job) = &self.conversion.job {
            job.cancel.cancel();
            return;
        }
        if let Some(id) = self.conversion.form.as_ref().and_then(|form| form.accepted)
            && engine
                .state()
                .ui_file
                .request
                .as_ref()
                .is_some_and(|request| {
                    request.id == id && request.action == UiFileAction::ConvertSmiles
                })
        {
            self.cancel_picker(engine, error);
        }
        self.close_conversion_form(context);
    }

    pub(super) fn sync_conversion(&mut self, engine: Option<&Engine>, error: &mut Option<String>) {
        if self
            .conversion
            .job
            .as_ref()
            .is_some_and(|job| !job.worker.is_finished())
        {
            return;
        }
        if let Some(job) = self.conversion.job.take() {
            let cancelled = job.cancel.is_cancelled();
            let joined = job.worker.join();
            let result = job.result.try_recv();
            if let Some(form) = &mut self.conversion.form {
                form.accepted = None;
                form.outcome=Some(match result {
                    Ok(Ok(result)) => match result.conversion {
                        Some(receipt) => Outcome::Success {receipt,historical:!engine.is_some_and(|engine|job.context.current(engine)),cancelled},
                        None=>Outcome::Failure("The conversion returned no publication receipt. Check the destination before retrying.".into()),
                    },
                    Ok(Err(failure))=>Outcome::Failure(format!("{} Check the destination before retrying.",failure.message)),
                    Err(_)=>Outcome::Failure(if joined.is_err() {"The conversion worker did not complete. Check the destination before retrying."}else{"The conversion worker stopped without a result. Check the destination before retrying."}.into()),
                });
            }
        }
        if let Some(form) = &mut self.conversion.form {
            if !engine.is_some_and(|engine| form.context.current(engine)) {
                form.stale = true;
                self.conversion.pending = None;
                self.conversion.browsing = None;
            }
            if let Some(Outcome::Failure(message)) = &form.outcome {
                *error = Some(message.clone());
            }
        }
    }

    pub(crate) fn start_pending_conversion(
        &mut self,
        executor: &CommandExecutor,
        context: &egui::Context,
    ) {
        if self.conversion.job.is_some() {
            return;
        }
        let Some(pending) = self.conversion.pending.take() else {
            return;
        };
        let work = match executor.prepare_ui_smiles_conversion(pending.id, pending.params) {
            Ok(work) => work,
            Err(failure) if failure.code == "busy" => {
                // Busy may mean the shared budget or Save lease; keep one explicit intent.
                if let Some(form) = &mut self.conversion.form {
                    form.outcome = Some(Outcome::Failure(failure.message));
                }
                context.request_repaint();
                return;
            }
            Err(failure) => {
                if let Some(form) = &mut self.conversion.form {
                    form.stale = matches!(
                        failure.code,
                        "stale_revision"
                            | "ui_stale_request"
                            | "close_committed"
                            | "ui_unavailable"
                    );
                    form.outcome = Some(Outcome::Failure(failure.message));
                }
                context.request_repaint();
                return;
            }
        };
        let Some(form) = &mut self.conversion.form else {
            return;
        };
        form.accepted = None;
        let captured = form.context.clone();
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let worker_context = context.clone();
        let (sender, result) = mpsc::sync_channel(1);
        match std::thread::Builder::new()
            .name("bondline-smiles-conversion".into())
            .spawn(move || {
                let output = work.complete_blocking(&worker_cancel);
                let _ = sender.send(output);
                worker_context.request_repaint();
            }) {
            Ok(worker) => {
                self.conversion.job = Some(Job {
                    context: captured,
                    cancel,
                    result,
                    worker,
                })
            }
            Err(failure) => {
                form.outcome = Some(Outcome::Failure(format!(
                    "Cannot start conversion: {failure}. No conversion worker ran."
                )))
            }
        }
    }

    pub(super) fn conversion_close(&mut self, engine: &mut Engine, error: &mut Option<String>) {
        self.conversion.pending = None;
        if let Some(job) = &self.conversion.job {
            job.cancel.cancel();
            return;
        }
        if self.conversion.present() {
            if self
                .conversion
                .form
                .as_ref()
                .and_then(|form| form.accepted)
                .is_some()
            {
                self.cancel_picker(engine, error);
            }
            self.conversion.form = None;
            self.conversion.browsing = None;
        }
    }

    pub(crate) fn deactivate_conversion(
        &mut self,
        context: &egui::Context,
        error: &mut Option<String>,
    ) {
        self.conversion.pending = None;
        if let Some(job) = &self.conversion.job {
            job.cancel.cancel();
        }
        self.sync_conversion(None, error);
        if self.conversion.busy() {
            context.request_repaint_after(Duration::from_millis(40));
        }
    }
}

#[cfg(test)]
#[path = "../native_conversion_review_tests.rs"]
mod owner_drop_tests;
