//! One SDF draft or record removal and its registered authoring transaction.

use crate::sdf_draft_text::SdfDraftText;
use bondline_commands::{
    Cancellation, CaptureStamp, CloseAction, CloseDocument, CloseState, Command, CommandResult,
    Engine, ErrorInfo, FileFormat, SdfFieldSet, SdfRecordRemove, UiFileAction, UiFilePhase,
};
use bondline_mcp::{CommandExecutor, UiSdfRemoveContext};
use eframe::egui;
use std::{
    sync::mpsc::{self, Receiver},
    thread::JoinHandle,
    time::Duration,
};

pub(super) enum Target {
    Record(u32),
    Next,
    Previous(usize),
    CloseFields,
    RemoveRecord(u32),
    Control(&'static str),
    ExternalFile { action: UiFileAction, id: u64 },
}

struct Held {
    target: Target,
    stamp: CaptureStamp,
    path: Option<String>,
    format: FileFormat,
    close_epoch: u64,
    file_epoch: u64,
}

impl Held {
    fn new(target: Target, engine: &Engine) -> Self {
        let state = engine.state();
        Self {
            target,
            stamp: engine.capture_stamp(),
            path: state.file.path,
            format: state.file.format,
            close_epoch: state.close_epoch,
            file_epoch: state.ui_file.epoch,
        }
    }

    fn destination_current(&self, engine: &Engine) -> bool {
        let state = engine.state();
        state.file.path == self.path
            && state.file.format == self.format
            && state.close_epoch == self.close_epoch
            && state.ui_file.epoch == self.file_epoch
    }
}

struct Draft {
    stamp: CaptureStamp,
    revision: u64,
    record: u32,
    field: usize,
    offset: usize,
    original: String,
    text: SdfDraftText,
    source_name: String,
}

impl Draft {
    fn changed(&self) -> bool {
        self.original != self.text.as_str()
    }
}

struct ApplyInput {
    stamp: CaptureStamp,
    close_epoch: u64,
    params: SdfFieldSet,
}

struct RemovalConfirmation {
    held: Held,
    record: u32,
    sequence: u64,
}

struct RemoveInput {
    context: UiSdfRemoveContext,
    params: SdfRecordRemove,
}

enum MutationInput {
    Field(ApplyInput),
    Remove(RemoveInput),
}

#[derive(Clone, Copy)]
enum MutationKind {
    Field,
    Remove,
}

impl MutationInput {
    fn kind(&self) -> MutationKind {
        match self {
            Self::Field(_) => MutationKind::Field,
            Self::Remove(_) => MutationKind::Remove,
        }
    }
}

struct Applied {
    result: CommandResult,
    stamp: Option<CaptureStamp>,
}

struct ApplyJob {
    kind: MutationKind,
    cancel: Cancellation,
    result: Receiver<Result<Applied, ErrorInfo>>,
    worker: JoinHandle<()>,
}

pub(super) struct Effect {
    pub refresh: Option<(u32, usize, usize)>,
    pub records_invalidated: bool,
    pub target: Option<Target>,
    pub warning: Option<String>,
}

#[derive(Clone, Copy)]
enum Action {
    Apply,
    End,
    CancelWork,
    DecideApply,
    DecideDiscard,
    DecideCancel,
    ConfirmRemove(u64),
    KeepRecord(u64),
}

#[derive(Default)]
pub(super) struct SdfEditor {
    draft: Option<Draft>,
    pending: Option<MutationInput>,
    job: Option<ApplyJob>,
    decision: Option<Held>,
    after_apply: Option<Held>,
    effect: Option<Effect>,
    error: Option<String>,
    actions: Vec<(usize, Action)>,
    removal: Option<RemovalConfirmation>,
    removal_sequence: u64,
}

pub(super) fn widget_id() -> egui::Id {
    egui::Id::new("bondline-sdf-field-draft")
}

fn undoer() -> egui::util::undoer::Undoer<(egui::text::CCursorRange, String)> {
    egui::util::undoer::Undoer::with_settings(egui::util::undoer::Settings {
        max_undos: 8,
        ..Default::default()
    })
}

fn reset_widget(context: &egui::Context) {
    context.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(widget_id()));
    context.memory_mut(|memory| memory.surrender_focus(widget_id()));
}

impl SdfEditor {
    pub fn active(&self) -> bool {
        self.draft.is_some()
    }
    pub fn is_busy(&self) -> bool {
        self.pending.is_some() || self.job.is_some()
    }
    pub fn blocks_close(&self) -> bool {
        self.is_busy()
            || self.removal.is_some()
            || self.decision.is_some()
            || self.after_apply.is_some()
            || self.draft.as_ref().is_some_and(Draft::changed)
    }
    pub fn holds_files(&self) -> bool {
        self.blocks_close()
    }
    pub fn begin_frame(&mut self) {
        self.actions.clear();
    }
    pub fn take_effect(&mut self) -> Option<Effect> {
        self.effect.take()
    }

    pub fn begin(
        &mut self,
        engine: &Engine,
        context: &egui::Context,
        record: u32,
        field: usize,
        offset: usize,
        value: &str,
    ) {
        let state = engine.state();
        if self.active()
            || self.is_busy()
            || self.removal.is_some()
            || state.close != CloseState::Open
            || state.ui_file.request.is_some()
        {
            return;
        }
        match SdfDraftText::new(value) {
            Ok(text) => {
                reset_widget(context);
                let mut widget = egui::text_edit::TextEditState::default();
                widget.set_undoer(undoer());
                widget.store(context, widget_id());
                self.draft = Some(Draft {
                    stamp: engine.capture_stamp(),
                    revision: state.revision,
                    record,
                    field,
                    offset,
                    original: value.to_owned(),
                    text,
                    source_name: state.file.path.unwrap_or_else(|| "SDF collection".into()),
                });
                self.error = None;
                context.request_repaint();
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    fn end(&mut self, context: &egui::Context) {
        self.draft = None;
        self.error = None;
        reset_widget(context);
    }

    pub fn confirm_remove(
        &mut self,
        engine: &Engine,
        context: &egui::Context,
        record: u32,
    ) -> bool {
        let state = engine.state();
        let valid = state.close == CloseState::Open
            && state.ui_file.request.is_none()
            && state.collection.as_ref().is_some_and(|collection| {
                collection.record_count > 1
                    && collection.records.iter().any(|item| item.id == record)
            });
        if self.active() || self.is_busy() || !valid {
            return false;
        }
        let Some(sequence) = self.removal_sequence.checked_add(1) else {
            self.error =
                Some("The removal request limit was reached. Reopen Bondline to continue.".into());
            return false;
        };
        self.removal_sequence = sequence;
        self.removal = Some(RemovalConfirmation {
            held: Held::new(Target::RemoveRecord(record), engine),
            record,
            sequence,
        });
        self.error = None;
        context.request_repaint();
        true
    }

    fn removal_current(removal: &RemovalConfirmation, engine: &Engine) -> bool {
        let state = engine.state();
        engine.capture_stamp_matches(&removal.held.stamp)
            && removal.held.destination_current(engine)
            && state.close == CloseState::Open
            && state.ui_file.request.is_none()
            && state.collection.as_ref().is_some_and(|collection| {
                collection.record_count > 1
                    && collection
                        .records
                        .iter()
                        .any(|item| item.id == removal.record)
            })
    }

    fn queue_remove(&mut self, sequence: u64, engine: &Engine) {
        let Some(removal) = self.removal.as_ref() else {
            return;
        };
        if removal.sequence != sequence || self.is_busy() {
            return;
        }
        if !Self::removal_current(removal, engine) {
            self.removal = None;
            self.error = Some(
                "The removal context changed. Choose the record again; nothing was removed.".into(),
            );
            return;
        }
        self.pending = Some(MutationInput::Remove(RemoveInput {
            context: UiSdfRemoveContext::capture(engine),
            params: SdfRecordRemove {
                expected_revision: engine.state().revision,
                record_id: removal.record,
            },
        }));
        self.removal = None;
        self.error = None;
        // Ordinals and query text cannot remain authoritative while a removal is pending.
        self.effect = Some(Effect {
            refresh: None,
            records_invalidated: true,
            target: None,
            warning: None,
        });
    }

    /// A held intent contains no old command parameters or file publication authority.
    pub fn intercept(&mut self, target: Target, engine: &Engine, context: &egui::Context) -> bool {
        self.removal = None;
        if self.is_busy() {
            if self.after_apply.is_none() {
                self.after_apply = Some(Held::new(target, engine));
            } else {
                self.error = Some(
                    "An action is already waiting for Apply. Try again after it finishes.".into(),
                );
            }
            context.request_repaint();
            return true;
        }
        if self.draft.as_ref().is_some_and(Draft::changed) {
            if self.decision.is_none() {
                self.decision = Some(Held::new(target, engine));
            }
            context.request_repaint();
            return true;
        }
        if self.active() {
            self.end(context);
        }
        false
    }

    pub fn intercept_command(
        &mut self,
        command: &Command,
        engine: &Engine,
        context: &egui::Context,
    ) -> bool {
        if self.removal.is_some()
            && !matches!(command, Command::UiView(_) | Command::UiBondToolSelect(_))
        {
            self.removal = None;
        }
        if !self.active() && !self.is_busy() {
            return false;
        }
        let target = match command {
            Command::UiFile(params) => match params.action {
                UiFileAction::New => Some(Target::Control("file.new")),
                UiFileAction::ConvertSmiles => Some(Target::Control("file.convert_smiles")),
                UiFileAction::Open => Some(Target::Control("file.open")),
                UiFileAction::Save => Some(Target::Control("file.save")),
                UiFileAction::SaveAs => Some(Target::Control("file.save_as")),
                UiFileAction::Cancel => None,
            },
            Command::Close(params) if params.action == CloseAction::Request => {
                Some(Target::Control("file.close"))
            }
            // TextEdit owns local undo while its transaction is open, even after a same-frame focus change.
            Command::Undo(_) | Command::Redo(_) => return true,
            Command::UiView(_) | Command::UiBondToolSelect(_) => return false,
            _ => {
                self.error = Some("Finish the field edit before changing the document.".into());
                return true;
            }
        };
        target.is_some_and(|target| self.intercept(target, engine, context))
    }

    pub fn guard_external(&mut self, engine: &mut Engine, context: &egui::Context) {
        if !self.active() && !self.is_busy() && self.removal.is_none() {
            return;
        }
        let state = engine.state();
        if !matches!(state.close, CloseState::Open | CloseState::Committed) {
            if self.blocks_close() {
                match engine.execute(Command::Close(CloseDocument {
                    expected_revision: state.revision,
                    action: CloseAction::Cancel,
                })) {
                    Ok(_) => {
                        self.intercept(Target::Control("file.close"), engine, context);
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            } else {
                self.end(context);
            }
        }
        let state = engine.state();
        if let Some(request) = state.ui_file.request
            && request.phase == UiFilePhase::Requested
            && self.decision.is_none()
            && self.after_apply.is_none()
        {
            self.intercept(
                Target::ExternalFile {
                    action: request.action,
                    id: request.id,
                },
                engine,
                context,
            );
        }
    }

    fn apply(&mut self, engine: &Engine) -> bool {
        if self.is_busy() {
            return false;
        }
        let Some(draft) = &self.draft else {
            return false;
        };
        if !engine.capture_stamp_matches(&draft.stamp) || engine.state().close != CloseState::Open {
            self.error = Some(
                "The source changed. Copy or discard this draft; it cannot be applied here.".into(),
            );
            return false;
        }
        let value = draft.text.as_str();
        let value_lines = if value.is_empty() {
            Vec::new()
        } else {
            value.split('\n').map(str::to_owned).collect()
        };
        self.pending = Some(MutationInput::Field(ApplyInput {
            stamp: draft.stamp.clone(),
            close_epoch: engine.state().close_epoch,
            params: SdfFieldSet {
                expected_revision: draft.revision,
                record_id: draft.record,
                field_index: draft.field,
                value_lines,
            },
        }));
        self.error = None;
        true
    }

    pub fn replay(&mut self, index: usize, engine: &Engine, context: &egui::Context) {
        let actions = self
            .actions
            .iter()
            .filter(|(at, _)| *at == index)
            .map(|(_, action)| *action)
            .collect::<Vec<_>>();
        for action in actions {
            match action {
                Action::Apply if self.decision.is_none() => {
                    self.apply(engine);
                }
                Action::End if !self.is_busy() => {
                    self.end(context);
                    self.decision = None;
                }
                Action::CancelWork => {
                    let removal = self
                        .pending
                        .as_ref()
                        .is_some_and(|input| matches!(input.kind(), MutationKind::Remove))
                        || self
                            .job
                            .as_ref()
                            .is_some_and(|job| matches!(job.kind, MutationKind::Remove));
                    self.pending = None;
                    self.after_apply = None;
                    self.error = Some(if let Some(job) = &self.job {
                        job.cancel.cancel();
                        if removal {
                            "Removal cancellation requested; its actual result is still required."
                                .into()
                        } else {
                            "Apply cancellation requested; its actual result is still required."
                                .into()
                        }
                    } else if removal {
                        "Removal was cancelled before work started; nothing was removed.".into()
                    } else {
                        "Apply was cancelled before work started; the draft was kept.".into()
                    });
                }
                Action::DecideApply => {
                    if self.apply(engine) {
                        self.after_apply = self.decision.take();
                    }
                }
                Action::DecideDiscard => {
                    if let Some(held) = self.decision.take() {
                        if engine.capture_stamp_matches(&held.stamp)
                            && held.destination_current(engine)
                        {
                            self.end(context);
                            self.effect = Some(Effect {
                                refresh: None,
                                records_invalidated: false,
                                target: Some(held.target),
                                warning: None,
                            });
                        } else {
                            self.error =
                                Some("The pending action changed. The draft was kept.".into());
                        }
                    }
                }
                Action::DecideCancel => {
                    if let Some(Held {
                        target: Target::ExternalFile { id, .. },
                        ..
                    }) = self.decision.take()
                        && engine
                            .state()
                            .ui_file
                            .request
                            .as_ref()
                            .is_some_and(|request| request.id == id)
                    {
                        // File cancellation is issued by the parent with fresh current parameters.
                        self.effect = Some(Effect {
                            refresh: None,
                            records_invalidated: false,
                            target: Some(Target::Control("file.cancel")),
                            warning: None,
                        });
                    }
                    self.decision = None;
                }
                Action::ConfirmRemove(sequence) => self.queue_remove(sequence, engine),
                Action::KeepRecord(sequence) => {
                    if self
                        .removal
                        .as_ref()
                        .is_some_and(|removal| removal.sequence == sequence)
                    {
                        self.removal = None;
                        context.request_discard("SDF record confirmation was dismissed");
                    }
                }
                _ => {}
            }
            context.request_repaint();
        }
    }

    pub fn show(
        &mut self,
        context: &egui::Context,
        engine: Option<&Engine>,
        events: &[egui::Event],
    ) {
        let busy = self.is_busy();
        let mut actions = Vec::new();
        if let Some(draft) = &mut self.draft {
            let current = engine.is_some_and(|engine| engine.capture_stamp_matches(&draft.stamp));
            let can_apply = current
                && engine.is_some_and(|engine| engine.state().close == CloseState::Open)
                && !busy;
            egui::Window::new("SDF field value").id(egui::Id::new("bondline-sdf-editor"))
                .default_size([440.0, 260.0]).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false).show(context, |ui| {
                    ui.weak(&draft.source_name);
                    ui.label("Unapplied text is not saved until Apply succeeds and the document is saved.");
                    if !current { ui.colored_label(egui::Color32::DARK_RED,
                        "The source changed. Copy or discard this draft; it cannot be applied here."); }
                    let label = ui.label(format!("Record {} field {} draft", draft.record, draft.field.saturating_add(1)));
                    let prior = egui::TextEdit::load_state(context, widget_id()).unwrap_or_default();
                    let prior_cursor = prior.cursor;
                    let (mut output, outcome) = draft.text.edit(|text| {
                        egui::ScrollArea::vertical().id_salt("sdf-draft-scroll").max_height(230.0)
                            .show(ui, |ui| {
                                ui.add_enabled_ui(!busy, |ui| egui::TextEdit::multiline(text)
                                    .id(widget_id()).desired_rows(6).desired_width(ui.available_width()).show(ui)).inner
                            }).inner
                    });
                    output.response.labelled_by(label.id);
                    if outcome.rejection.is_some() {
                        output.state.cursor = prior_cursor;
                        output.state.set_undoer(undoer());
                        self.error = draft.text.last_rejection().map(|error| error.to_string());
                    } else if outcome.changed { self.error = None; }
                    output.state.store(context, widget_id());
                    if let Some(error) = &self.error { ui.colored_label(egui::Color32::DARK_RED, error); }
                    ui.horizontal_wrapped(|ui| {
                        for (caption, enabled, action) in [
                            ("Apply field value", can_apply, Action::Apply),
                            ("Cancel field edit", !busy, Action::End),
                            ("Cancel applying", busy, Action::CancelWork),
                        ] {
                            if caption == "Cancel applying" && !busy { continue; }
                            let response = ui.add_enabled(enabled, egui::Button::new(caption));
                            if response.clicked() && let Some(index) = events.iter().position(|event| crate::activation_event(&response, event)) {
                                actions.push((index, action));
                            }
                        }
                        if ui.button("Copy draft").clicked() { context.copy_text(draft.text.as_str().to_owned()); }
                    });
                    if busy { ui.spinner(); ui.label("Applying field value…"); }
                });
        }
        if self.decision.is_some() {
            let can_apply = !busy
                && self.draft.as_ref().is_some_and(|draft| {
                    engine.is_some_and(|engine| {
                        engine.capture_stamp_matches(&draft.stamp)
                            && engine.state().close == CloseState::Open
                    })
                });
            egui::Window::new("Unapplied field value")
                .id(egui::Id::new("bondline-sdf-draft-decision"))
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .order(egui::Order::Foreground)
                .collapsible(false)
                .resizable(false)
                .show(context, |ui| {
                    ui.label("Apply this field value before continuing?");
                    ui.horizontal(|ui| {
                        for (caption, enabled, action) in [
                            ("Apply", can_apply, Action::DecideApply),
                            ("Discard", !busy, Action::DecideDiscard),
                            ("Cancel", !busy, Action::DecideCancel),
                        ] {
                            let response = ui.add_enabled(enabled, egui::Button::new(caption));
                            if response.clicked()
                                && let Some(index) = events
                                    .iter()
                                    .position(|event| crate::activation_event(&response, event))
                            {
                                actions.push((index, action));
                            }
                        }
                    });
                });
            if let Some(index) = events.iter().position(|event| {
                matches!(
                    event,
                    egui::Event::Key {
                        key: egui::Key::Escape,
                        pressed: true,
                        repeat: false,
                        ..
                    }
                )
            }) {
                actions.push((index, Action::DecideCancel));
            }
        }
        if let Some(removal) = &self.removal {
            let sequence = removal.sequence;
            let current = engine.is_some_and(|engine| Self::removal_current(removal, engine));
            egui::Window::new("Remove SDF record")
                .id(egui::Id::new(("bondline-sdf-remove", sequence)))
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .order(egui::Order::Foreground)
                .collapsible(false)
                .resizable(false)
                .show(context, |ui| {
                    ui.label(format!(
                        "Remove record {} and all of its structure and data fields?",
                        removal.record
                    ));
                    ui.label("This changes the collection in memory. Save is a separate action.");
                    ui.horizontal(|ui| {
                        for (caption, enabled, action) in [
                            (
                                "Confirm record removal",
                                current && !busy,
                                Action::ConfirmRemove(sequence),
                            ),
                            ("Keep record", !busy, Action::KeepRecord(sequence)),
                        ] {
                            let response = ui.add_enabled(enabled, egui::Button::new(caption));
                            if response.clicked()
                                && let Some(index) = events
                                    .iter()
                                    .position(|event| crate::activation_event(&response, event))
                            {
                                actions.push((index, action));
                            }
                        }
                    });
                });
            if let Some(index) = events.iter().position(|event| {
                matches!(
                    event,
                    egui::Event::Key {
                        key: egui::Key::Escape,
                        pressed: true,
                        repeat: false,
                        ..
                    }
                )
            }) {
                actions.push((index, Action::KeepRecord(sequence)));
            }
        }
        if self.draft.is_none() && (busy || self.error.is_some()) {
            egui::Window::new("SDF record removal")
                .id(egui::Id::new("bondline-sdf-removal-status"))
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(context, |ui| {
                    if busy {
                        ui.spinner();
                        ui.label("Removing record…");
                    }
                    if let Some(error) = &self.error {
                        ui.label(error);
                    }
                    let (caption, action) = if busy {
                        ("Cancel removal", Action::CancelWork)
                    } else {
                        ("Close removal message", Action::End)
                    };
                    let response = ui.button(caption);
                    if response.clicked()
                        && let Some(index) = events
                            .iter()
                            .position(|event| crate::activation_event(&response, event))
                    {
                        actions.push((index, action));
                    }
                });
        }
        self.actions.extend(actions);
        if busy {
            context.request_repaint_after(Duration::from_millis(40));
        }
    }

    pub fn sync(&mut self, engine: Option<&Engine>, context: &egui::Context) {
        if self.removal.as_ref().is_some_and(|removal| {
            !engine.is_some_and(|engine| Self::removal_current(removal, engine))
        }) {
            self.removal = None;
            self.error = Some(
                "The removal context changed. Choose the record again; nothing was removed.".into(),
            );
        }
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
        let removal = matches!(job.kind, MutationKind::Remove);
        let cancellation_requested = job.cancel.is_cancelled();
        let result = match job.worker.join() {
            Ok(()) => job.result.try_recv().unwrap_or_else(|_| {
                Err(ErrorInfo {
                    code: "worker_failed",
                    message: if removal { "The removal worker stopped without a result. Its outcome could not be verified; inspect the current collection before trying again.".into() }
                        else { "The Apply worker stopped without a result; the draft was kept.".into() },
                })
            }),
            Err(_) => Err(ErrorInfo {
                code: "worker_failed",
                message: if removal { "The removal worker did not complete. Its outcome could not be verified; inspect the current collection before trying again.".into() }
                    else { "The Apply worker did not complete; the draft was kept.".into() },
            }),
        };
        // Unlike queries, a successfully committed edit is never discarded as cancellation.
        match result {
            Ok(applied) => {
                let current = engine.is_some_and(|engine| {
                    applied
                        .stamp
                        .as_ref()
                        .is_some_and(|stamp| engine.capture_stamp_matches(stamp))
                });
                if !current {
                    self.error = Some(if removal {
                        "The record was removed from an earlier document. The current document was not replaced; no waiting action ran.".into()
                    } else {
                        "The value was applied to an earlier document. Its text was kept for recovery; no waiting action ran.".into()
                    });
                    self.after_apply = None;
                    return;
                }
                let held = self.after_apply.take();
                let mut warning = cancellation_requested.then(|| {
                    if removal {
                        "The record was removed before cancellation completed.".to_owned()
                    } else if applied.result.changed {
                        "The value was applied before cancellation completed.".to_owned()
                    } else {
                        "Apply completed without changes before cancellation completed.".to_owned()
                    }
                });
                let target = held.and_then(|held| {
                    if engine.is_some_and(|engine| held.destination_current(engine)) { Some(held.target) }
                    else { warning = Some(if removal { "The waiting action's file or close context changed. The record was removed; the waiting action did not run.".into() }
                        else { "The waiting action's file or close context changed. The field was applied; the waiting action did not run.".into() }); None }
                });
                if removal {
                    self.effect = Some(Effect {
                        refresh: None,
                        records_invalidated: true,
                        target,
                        warning,
                    });
                } else if let Some(draft) = &self.draft {
                    self.effect = Some(Effect {
                        refresh: Some((draft.record, draft.offset, draft.field)),
                        records_invalidated: false,
                        target,
                        warning,
                    });
                }
                self.end(context);
            }
            Err(error) => {
                self.after_apply = None;
                self.error = Some(if removal {
                    format!(
                        "{} No waiting Save/Open/close action was started.",
                        error.message
                    )
                } else {
                    format!(
                        "{} No waiting Save/Open/close action was started; the draft was kept.",
                        error.message
                    )
                });
            }
        }
    }

    pub fn start_pending(&mut self, executor: &CommandExecutor, context: &egui::Context) {
        if self.job.is_some() {
            return;
        }
        let Some(input) = self.pending.take() else {
            return;
        };
        let kind = input.kind();
        let removal = matches!(kind, MutationKind::Remove);
        let prepared = match &input {
            MutationInput::Field(input) => executor.prepare_ui_sdf(
                &input.stamp,
                input.close_epoch,
                Command::SdfFieldSet(input.params.clone()),
            ),
            MutationInput::Remove(input) => {
                executor.prepare_ui_sdf_remove(&input.context, input.params.clone())
            }
        };
        let work = match prepared {
            Ok(work) => work,
            Err(error) if error.code == "busy" => {
                self.pending = Some(input);
                context.request_repaint_after(Duration::from_millis(40));
                return;
            }
            Err(error) => {
                self.after_apply = None;
                self.error = Some(if removal {
                    format!(
                        "{} No removal or waiting action was started.",
                        error.message
                    )
                } else {
                    format!(
                        "{} No waiting action was started; the draft was kept.",
                        error.message
                    )
                });
                context.request_repaint();
                return;
            }
        };
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let owner = std::sync::Arc::clone(executor.engine());
        let worker_context = context.clone();
        let (sender, result) = mpsc::sync_channel(1);
        match std::thread::Builder::new()
            .name("bondline-sdf-apply".into())
            .spawn(move || {
                let output = work.complete_blocking(&worker_cancel).map(|result| {
                    let stamp = owner.lock().ok().and_then(|engine| {
                        (engine.state().revision == result.revision).then(|| engine.capture_stamp())
                    });
                    Applied { result, stamp }
                });
                let _ = sender.send(output);
                worker_context.request_repaint();
            }) {
            Ok(worker) => {
                self.job = Some(ApplyJob {
                    kind,
                    cancel,
                    result,
                    worker,
                })
            }
            Err(_) => {
                self.after_apply = None;
                self.error = Some(if removal {
                    "The removal worker could not start; nothing was removed.".into()
                } else {
                    "The Apply worker could not start; the draft was kept.".into()
                });
            }
        }
    }
}

impl Drop for SdfEditor {
    fn drop(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
    }
}

pub(super) fn resume_command(target: Target, engine: &Engine) -> Result<Option<Command>, String> {
    let id =
        match target {
            Target::Control(id) => id,
            Target::ExternalFile { action, id } => {
                let state = engine.state();
                if state.ui_file.request.as_ref().is_some_and(|request| {
                    request.id == id && request.phase == UiFilePhase::Requested
                }) {
                    return Ok(None);
                }
                match action {
                    UiFileAction::New => "file.new",
                    UiFileAction::ConvertSmiles => "file.convert_smiles",
                    UiFileAction::Open => "file.open",
                    UiFileAction::Save => "file.save",
                    UiFileAction::SaveAs => "file.save_as",
                    UiFileAction::Cancel => return Ok(None),
                }
            }
            _ => return Ok(None),
        };
    crate::control_command(engine, id).map(Some).ok_or_else(|| {
        "The waiting action is no longer available. No file work or close was started.".into()
    })
}
#[cfg(test)]
#[path = "sdf_editor_lifecycle_tests.rs"]
mod lifecycle_tests;

#[cfg(test)]
#[path = "sdf_record_removal_lifecycle_tests.rs"]
mod record_removal_tests;

#[cfg(test)]
#[path = "sdf_record_removal_prepare_tests.rs"]
mod removal_prepare_tests;
