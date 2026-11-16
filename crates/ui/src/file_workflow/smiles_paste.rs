//! A bounded notation draft and one registered native authoring transaction.

use super::*;
use crate::sdf_draft_text::SdfDraftText;
use bondline_commands::{CaptureStamp, PasteSmiles, UiFileContextStamp};
use bondline_mcp::UiSmilesPasteContext;
use std::{sync::Arc, thread::JoinHandle};

const FIELD_COUNT: usize = 4;

#[derive(Clone)]
struct Facts {
    close_epoch: u64,
    view_epoch: u64,
    path: Option<String>,
    format: FileFormat,
}

impl Facts {
    fn capture(engine: &Engine) -> Self {
        let state = engine.state();
        Self {
            close_epoch: state.close_epoch,
            view_epoch: engine.view().epoch,
            path: state.file.path,
            format: state.file.format,
        }
    }

    fn current(&self, engine: &Engine) -> bool {
        let state = engine.state();
        state.close == CloseState::Open
            && state.close_epoch == self.close_epoch
            && engine.view().epoch == self.view_epoch
            && state.file.path == self.path
            && state.file.format == self.format
    }
}

struct Applied {
    stamp: CaptureStamp,
    file_context: UiFileContextStamp,
    facts: Facts,
    file_epoch: u64,
}

impl Applied {
    fn current(&self, engine: &Engine) -> bool {
        engine.capture_stamp_matches(&self.stamp)
            && engine.ui_file_context_stamp_matches(&self.file_context)
            && self.facts.current(engine)
            && engine.state().ui_file.epoch == self.file_epoch
    }
}

struct Values {
    notation: SdfDraftText,
    length: SdfDraftText,
    x: SdfDraftText,
    y: SdfDraftText,
}

impl Values {
    fn new() -> Result<Self, String> {
        let text = |value: &str| SdfDraftText::new(value).map_err(|error| error.to_string());
        Ok(Self {
            notation: text("")?,
            length: text("48")?,
            x: text("72")?,
            y: text("72")?,
        })
    }

    fn params(&self, revision: u64, page: u32) -> Result<PasteSmiles, String> {
        let number = |text: &SdfDraftText, name: &str| {
            text.as_str()
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or_else(|| format!("{name} must be a finite number."))
        };
        Ok(PasteSmiles {
            expected_revision: revision,
            page_id: page,
            notation: self.notation.as_str().to_owned(),
            bond_length_pt: number(&self.length, "Bond length (pt)")?,
            origin_x_pt: number(&self.x, "Origin X (pt)")?,
            origin_y_pt: number(&self.y, "Origin Y (pt)")?,
        })
    }
}

struct Form {
    key: u64,
    accepted: Option<u64>,
    context: UiSmilesPasteContext,
    facts: Facts,
    revision: u64,
    page: u32,
    values: Values,
    detached: bool,
    message: Option<String>,
}

struct Pending {
    context: UiSmilesPasteContext,
    params: PasteSmiles,
}

struct Finished {
    result: Result<CommandResult, ErrorInfo>,
    applied: Option<Applied>,
}

struct Job {
    context: UiSmilesPasteContext,
    cancel: Cancellation,
    user_cancelled: bool,
    result: Receiver<Finished>,
    worker: JoinHandle<()>,
}

#[derive(Clone, Copy)]
enum Action {
    Apply,
    Cancel,
}

#[derive(Default)]
pub(super) struct PasteWorkflow {
    form: Option<Form>,
    pending: Option<Pending>,
    job: Option<Job>,
    actions: Vec<(usize, u64, Action)>,
}

impl PasteWorkflow {
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
    egui::Id::new(("smiles-paste-field", key, index))
}

fn undoer() -> egui::util::undoer::Undoer<(egui::text::CCursorRange, String)> {
    egui::util::undoer::Undoer::with_settings(egui::util::undoer::Settings {
        max_undos: 8,
        ..Default::default()
    })
}

fn reset_fields(context: &egui::Context, key: u64) {
    for index in 0..FIELD_COUNT {
        let id = field_id(key, index);
        context.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(id));
        context.memory_mut(|memory| memory.surrender_focus(id));
    }
}

pub(super) fn field(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: &str,
    text: &mut SdfDraftText,
    multiline: bool,
    enabled: bool,
) -> Option<String> {
    let label = ui.label(label);
    let prior = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_else(|| {
        let mut state = egui::text_edit::TextEditState::default();
        state.set_undoer(undoer());
        state
    });
    prior.clone().store(ui.ctx(), id);
    let mut error = None;
    let (mut output, outcome) = text.edit(|buffer| {
        if enabled {
            let requested = ui.input(|input| {
                input
                    .accesskit_action_requests(id, egui::accesskit::Action::SetValue)
                    .last()
                    .and_then(|request| match &request.data {
                        Some(egui::accesskit::ActionData::Value(value)) => {
                            Some(if value.len() > bondline_commands::MAX_SMILES_PASTE_BYTES {
                                Err("Draft text exceeds 65536 UTF-8 bytes.".to_owned())
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
        ui.add_enabled_ui(enabled, |ui| {
            let editor = if multiline {
                egui::TextEdit::multiline(buffer).desired_rows(5)
            } else {
                egui::TextEdit::singleline(buffer)
            };
            editor
                .id(id)
                .char_limit(usize::MAX)
                .desired_width(ui.available_width())
                .show(ui)
        })
        .inner
    });
    output.response.labelled_by(label.id);
    if outcome.rejection.is_some() || error.is_some() {
        output.state.cursor = prior.cursor;
        output.state.set_undoer(undoer());
    }
    if outcome.changed || outcome.rejection.is_some() || error.is_some() {
        ui.ctx().request_repaint();
    }
    output.state.store(ui.ctx(), id);
    ui.ctx().accesskit_node_builder(id, |node| {
        node.set_value(text.as_str());
        if enabled {
            node.add_action(egui::accesskit::Action::SetValue);
        }
    });
    error.or_else(|| outcome.rejection.map(|error| error.to_string()))
}

impl FileWorkflow {
    pub(crate) fn paste_present(&self) -> bool {
        self.paste.present()
    }

    pub(super) fn open_paste(&mut self, id: u64, engine: &mut Engine, error: &mut Option<String>) {
        let opened = (|| {
            let page = engine
                .document()
                .map_err(|failure| failure.to_string())?
                .pages()
                .first()
                .ok_or_else(|| "The drawing has no page for SMILES paste.".to_owned())?
                .id()
                .get();
            let context = UiSmilesPasteContext::capture(engine, id, page)
                .map_err(|failure| failure.message)?;
            let values = Values::new()?;
            Ok::<_, String>(Form {
                key: id,
                accepted: Some(id),
                context,
                facts: Facts::capture(engine),
                revision: engine.state().revision,
                page,
                values,
                detached: false,
                message: None,
            })
        })();
        match opened {
            Ok(form) => self.paste.form = Some(form),
            Err(message) => {
                self.cancel_picker(engine, error);
                *error = Some(message);
            }
        }
    }

    pub(crate) fn begin_paste_ui(
        &mut self,
        context: &egui::Context,
        engine: Option<&Engine>,
        events: &[egui::Event],
    ) {
        self.paste.actions.clear();
        let busy = self.paste.busy();
        let Some(form) = &mut self.paste.form else {
            return;
        };
        if !busy && !engine.is_some_and(|engine| form.context.is_current(engine)) {
            form.detached = true;
        }
        let key = form.key;
        let can_apply =
            !busy && !form.detached && engine.is_some_and(|engine| form.context.is_current(engine));
        let mut actions = Vec::new();
        egui::Window::new("Paste SMILES")
            .id(egui::Id::new(("bondline-smiles-paste", key)))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .order(egui::Order::Foreground).default_width(500.0)
            .collapsible(false).resizable(false).show(context, |ui| {
                ui.label("Paste one complete supported neutral SMILES notation. Apply appends the entire structure as one edit.");
                let mut failure = field(ui, field_id(key,0), "SMILES", &mut form.values.notation, true, !busy);
                egui::Grid::new(("smiles-paste-geometry",key)).num_columns(2).show(ui, |ui| {
                    for (index,(label,text)) in [("Bond length (pt)",&mut form.values.length),
                        ("Origin X (pt)",&mut form.values.x),("Origin Y (pt)",&mut form.values.y)].into_iter().enumerate() {
                        failure = field(ui,field_id(key,index+1),label,text,false,!busy).or_else(|| failure.take());
                        ui.end_row();
                    }
                });
                if let Some(failure) = failure { form.message = Some(failure); }
                if form.detached {
                    ui.colored_label(egui::Color32::DARK_RED,"The document or view changed. Copy this draft or Cancel; it cannot be applied here.");
                }
                if let Some(message) = &form.message { ui.colored_label(egui::Color32::DARK_RED,message); }
                if busy { ui.spinner(); ui.label("Applying SMILES…"); }
                ui.horizontal(|ui| {
                    for (label,enabled,action) in [("Apply",can_apply,Action::Apply),("Cancel",true,Action::Cancel)] {
                        let response=ui.add_enabled(enabled,egui::Button::new(label));
                        if response.clicked() && let Some(index)=events.iter().position(|event|crate::activation_event(&response,event)) {
                            actions.push((index,key,action));
                        }
                    }
                });
            });
        if engine.is_none() {
            let cancel = actions.iter().any(|(_,_,action)|matches!(action,Action::Cancel))
                || events.iter().any(|event|matches!(event,egui::Event::Key {key:egui::Key::Escape,pressed:true,repeat:false,modifiers,..} if modifiers.is_none()));
            if cancel {
                self.cancel_paste(None, context);
            }
        } else {
            self.paste.actions = actions;
        }
        if busy {
            context.request_repaint_after(Duration::from_millis(40));
        }
    }

    pub(crate) fn replay_paste(
        &mut self,
        index: usize,
        event: &egui::Event,
        engine: &mut Engine,
        error: &mut Option<String>,
        context: &egui::Context,
    ) {
        let Some(key) = self.paste.form.as_ref().map(|form| form.key) else {
            return;
        };
        let mut actions = self
            .paste
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
                Action::Cancel => self.cancel_paste(Some(&mut *engine), context),
                Action::Apply => self.queue_paste(engine, error),
            }
            context.request_repaint();
        }
    }

    fn queue_paste(&mut self, engine: &mut Engine, error: &mut Option<String>) {
        if self.paste.busy() {
            return;
        }
        let Some(form) = &mut self.paste.form else {
            return;
        };
        if form.detached || !form.context.is_current(engine) {
            form.detached = true;
            return;
        }
        let params = match form.values.params(form.revision, form.page) {
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
                    action: UiFileAction::PasteSmiles,
                    request_id: None,
                    close_epoch: None,
                }))
                .and_then(|result| {
                    let id = result
                        .ui_file_operation
                        .ok_or(bondline_commands::Error::UiStaleRequest)?;
                    if !matches!(engine.claim_ui_file(id)?, UiFilePlan::SmilesPaste) {
                        return Err(bondline_commands::Error::UiStaleRequest);
                    }
                    Ok(id)
                });
            match request {
                Ok(id) => match UiSmilesPasteContext::capture(engine, id, form.page) {
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
        self.paste.pending = Some(Pending {
            context: form.context.clone(),
            params,
        });
        form.message = None;
        *error = None;
    }

    fn close_paste_form(&mut self, context: &egui::Context) {
        if let Some(form) = self.paste.form.take() {
            reset_fields(context, form.key);
            if self.request_id == form.accepted || self.request_id == Some(form.key) {
                self.request_id = None;
            }
        }
        self.paste.pending = None;
        self.paste.actions.clear();
        context.request_discard("SMILES paste form closed");
    }

    fn cancel_paste(&mut self, engine: Option<&mut Engine>, context: &egui::Context) {
        self.paste.pending = None;
        if let Some(job) = &mut self.paste.job {
            job.user_cancelled = true;
            job.cancel.cancel();
            return;
        }
        if let (Some(engine), Some(id)) = (
            engine,
            self.paste.form.as_ref().and_then(|form| form.accepted),
        ) && engine
            .state()
            .ui_file
            .request
            .as_ref()
            .is_some_and(|request| request.id == id && request.action == UiFileAction::PasteSmiles)
        {
            let revision = engine.state().revision;
            let _ = engine.execute(Command::UiFile(UiFile {
                expected_revision: revision,
                action: UiFileAction::Cancel,
                request_id: Some(id),
                close_epoch: None,
            }));
        }
        self.close_paste_form(context);
    }

    pub(crate) fn poll_paste(
        &mut self,
        engine: Option<&Engine>,
        context: &egui::Context,
        error: &mut Option<String>,
    ) {
        if let Some(job) = &self.paste.job {
            if !engine.is_some_and(|engine| job.context.is_current(engine)) {
                job.cancel.cancel();
            }
            if !job.worker.is_finished() {
                context.request_repaint_after(Duration::from_millis(40));
                return;
            }
        }
        if let Some(job) = self.paste.job.take() {
            let joined = job.worker.join();
            let finished = job.result.try_recv();
            if let Some(form) = &mut self.paste.form {
                form.accepted = None;
            }
            match finished {
                Ok(Finished {
                    result: Ok(_),
                    applied,
                }) => {
                    if applied
                        .as_ref()
                        .is_some_and(|applied| engine.is_some_and(|engine| applied.current(engine)))
                    {
                        *error = job.user_cancelled.then(|| {
                            "The structure was applied before cancellation completed.".into()
                        });
                        self.close_paste_form(context);
                    } else if let Some(form) = &mut self.paste.form {
                        form.detached = true;
                        form.message=Some("The structure was applied before the document context changed. This draft cannot be applied again.".into());
                    }
                }
                Ok(Finished {
                    result: Err(failure),
                    ..
                }) => {
                    if job.user_cancelled {
                        self.close_paste_form(context);
                    } else if let Some(form) = &mut self.paste.form {
                        form.message = Some(failure.message);
                    }
                }
                Err(_) => {
                    if let Some(form) = &mut self.paste.form {
                        form.message = Some(
                            if joined.is_err() {
                                "The SMILES worker did not complete."
                            } else {
                                "The SMILES worker stopped without a result."
                            }
                            .into(),
                        );
                    }
                }
            }
            context.request_repaint();
        }
        if let Some(form) = &mut self.paste.form
            && !engine.is_some_and(|engine| form.context.is_current(engine))
        {
            form.detached = true;
            self.paste.pending = None;
        }
    }

    pub(crate) fn start_pending_paste(
        &mut self,
        executor: &CommandExecutor,
        context: &egui::Context,
    ) {
        if self.paste.job.is_some() {
            return;
        }
        let Some(pending) = self.paste.pending.take() else {
            return;
        };
        let work = match executor.prepare_ui_smiles_paste(&pending.context, pending.params.clone())
        {
            Ok(work) => work,
            Err(failure) if failure.code == "busy" => {
                self.paste.pending = Some(pending);
                context.request_repaint_after(Duration::from_millis(40));
                return;
            }
            Err(failure) => {
                if let Some(form) = &mut self.paste.form {
                    form.message = Some(failure.message);
                }
                context.request_repaint();
                return;
            }
        };
        let Some(form) = &mut self.paste.form else {
            return;
        };
        form.accepted = None;
        let facts = form.facts.clone();
        let origin = Arc::clone(executor.engine());
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let repaint = context.clone();
        let (sender, result) = mpsc::sync_channel(1);
        match std::thread::Builder::new()
            .name("bondline-smiles-paste".into())
            .spawn(move || {
                let result = work.complete_blocking(&worker_cancel);
                let applied = result.as_ref().ok().and_then(|result| {
                    let engine = origin.lock().ok()?;
                    let state = engine.state();
                    let stored = engine.stored_document().ok()?;
                    let returned = result.document.as_ref()?;
                    let file_context = result.ui_file_context_stamp.as_ref()?;
                    (result.revision == state.revision
                        && Arc::ptr_eq(returned, &stored.model_snapshot())
                        && engine.ui_file_context_stamp_matches(file_context)
                        && facts.current(&engine)
                        && result.ui_file.epoch == state.ui_file.epoch)
                        .then(|| Applied {
                            stamp: engine.capture_stamp(),
                            file_context: file_context.clone(),
                            facts,
                            file_epoch: state.ui_file.epoch,
                        })
                });
                let _ = sender.send(Finished { result, applied });
                repaint.request_repaint();
            }) {
            Ok(worker) => {
                self.paste.job = Some(Job {
                    context: pending.context,
                    cancel,
                    user_cancelled: false,
                    result,
                    worker,
                })
            }
            Err(failure) => {
                form.message = Some(format!(
                    "Cannot start SMILES paste: {failure}. No worker ran."
                ))
            }
        }
    }

    pub(super) fn paste_close(&mut self) {
        self.paste.pending = None;
        self.paste.cancel();
        if let Some(form) = &mut self.paste.form {
            form.detached = true;
        }
    }

    pub(crate) fn deactivate_paste(&mut self, context: &egui::Context, error: &mut Option<String>) {
        self.paste_close();
        self.poll_paste(None, context, error);
    }
}
