//! Qualified replacement decisions reuse the existing native file request and Save.

use super::*;
use bondline_commands::{CaptureStamp, NewDocument};
use std::thread::JoinHandle;

#[derive(Clone, Copy)]
pub(super) enum NewChoice {
    Save,
    Discard,
    Cancel,
}

enum Phase {
    Decision(u64),
    Saving(NewSave),
}

struct NewSave {
    request_id: u64,
    stamp: CaptureStamp,
    revision: u64,
    close_epoch: u64,
    armed: bool,
    worker: Option<JoinHandle<()>>,
    worker_failed: bool,
}

#[derive(Default)]
pub(super) struct NewWorkflow {
    phase: Option<Phase>,
    actions: Vec<(usize, u64, NewChoice)>,
}

impl NewWorkflow {
    pub(super) fn pending(&self) -> bool {
        self.phase.is_some()
    }

    pub(super) fn decision(&self) -> bool {
        matches!(self.phase, Some(Phase::Decision(_)))
    }

    pub(super) fn disarm(&mut self) {
        match &mut self.phase {
            Some(Phase::Saving(save)) => save.armed = false,
            _ => self.phase = None,
        }
    }
}

impl FileWorkflow {
    pub(crate) fn new_decision_open(&self) -> bool {
        self.new_document.decision()
    }

    pub(super) fn accepted_new(
        &mut self,
        id: u64,
        engine: &mut Engine,
        error: &mut Option<String>,
    ) {
        if engine.state().file.dirty {
            self.new_document.phase = Some(Phase::Decision(id));
        } else {
            *error = engine
                .create_ui_document(id, false)
                .err()
                .map(|failure| failure.to_string());
            self.sync_close(&engine.state());
        }
    }

    /// Called at the raw command event so later Drawing events use the new graph.
    pub(crate) fn adopt_new(&mut self, engine: &mut Engine, error: &mut Option<String>) {
        if engine
            .state()
            .ui_file
            .request
            .as_ref()
            .is_some_and(|request| {
                request.action == UiFileAction::New && request.phase == UiFilePhase::Requested
            })
        {
            let _ = self.adopt_request(engine, error);
        }
    }

    pub(super) fn sync_new(&mut self, engine: &Engine) {
        let state = engine.state();
        match &mut self.new_document.phase {
            Some(Phase::Decision(id)) => {
                if state.close != CloseState::Open
                    || !state.ui_file.request.as_ref().is_some_and(|request| {
                        request.id == *id
                            && request.action == UiFileAction::New
                            && request.phase == UiFilePhase::Accepted
                    })
                {
                    self.new_document.phase = None;
                }
            }
            Some(Phase::Saving(save)) => {
                if state.close != CloseState::Open
                    || state.close_epoch != save.close_epoch
                    || state.revision != save.revision
                    || !engine.capture_stamp_matches(&save.stamp)
                    || state.ui_file.epoch != save.request_id
                    || !state.ui_file.request.as_ref().is_some_and(|request| {
                        request.id == save.request_id && request.action == UiFileAction::Save
                    })
                {
                    save.armed = false;
                    if save.worker.is_none() && self.job.is_none() {
                        self.new_document.phase = None;
                    }
                }
            }
            None => {}
        }
    }

    pub(crate) fn begin_new_ui(&mut self, context: &egui::Context, events: &[egui::Event]) {
        self.new_document.actions.clear();
        let Some(Phase::Decision(id)) = self.new_document.phase else {
            return;
        };
        let mut actions = Vec::new();
        egui::Window::new("New document")
            .id(egui::Id::new(("bondline-new-document", id)))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .order(egui::Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .show(context, |ui| {
                ui.label("Save your changes before creating a new document?");
                ui.horizontal(|ui| {
                    for (caption, choice) in [
                        ("Save", NewChoice::Save),
                        ("Discard Changes", NewChoice::Discard),
                        ("Cancel", NewChoice::Cancel),
                    ] {
                        let response = ui.button(caption);
                        if response.clicked()
                            && let Some(index) = events
                                .iter()
                                .position(|event| crate::activation_event(&response, event))
                        {
                            actions.push((index, id, choice));
                        }
                    }
                });
            });
        self.new_document.actions = actions;
    }

    pub(crate) fn replay_new(
        &mut self,
        index: usize,
        event: &egui::Event,
        engine: &mut Engine,
        error: &mut Option<String>,
        context: &egui::Context,
    ) {
        let mut actions = self
            .new_document
            .actions
            .iter()
            .filter(|(at, _, _)| *at == index)
            .map(|(_, id, choice)| (*id, *choice))
            .collect::<Vec<_>>();
        if matches!(event, egui::Event::Key { key: egui::Key::Escape, pressed:true, repeat:false, modifiers, .. } if modifiers.is_none())
            && let Some(Phase::Decision(id)) = self.new_document.phase
        {
            actions.push((id, NewChoice::Cancel));
        }
        for (id, choice) in actions {
            self.choose_new(id, choice, engine, error);
            context.request_discard("New document decision changed");
        }
    }

    pub(super) fn choose_new(
        &mut self,
        id: u64,
        choice: NewChoice,
        engine: &mut Engine,
        error: &mut Option<String>,
    ) {
        if !matches!(self.new_document.phase, Some(Phase::Decision(current)) if current == id) {
            return;
        }
        if !matches!(engine.claim_ui_file(id), Ok(UiFilePlan::NewDocument)) {
            self.new_document.phase = None;
            *error =
                Some("The New document request is obsolete; no new document was created.".into());
            self.sync_close(&engine.state());
            return;
        }
        match choice {
            NewChoice::Discard => {
                self.new_document.phase = None;
                *error = engine
                    .create_ui_document(id, true)
                    .err()
                    .map(|failure| format!("{failure}; no new document was created."));
                self.sync_close(&engine.state());
            }
            NewChoice::Cancel => {
                self.new_document.phase = None;
                self.cancel_picker(engine, error);
            }
            NewChoice::Save => {
                let stamp = engine.capture_stamp();
                let state = engine.state();
                self.new_document.phase = None;
                self.cancel_picker(engine, error);
                if error.is_some() {
                    return;
                }
                self.request_file(engine, UiFileAction::Save, None, error);
                if error.is_some() {
                    return;
                }
                if let Some(request) = engine.state().ui_file.request {
                    self.new_document.phase = Some(Phase::Saving(NewSave {
                        request_id: request.id,
                        stamp,
                        revision: state.revision,
                        close_epoch: state.close_epoch,
                        armed: true,
                        worker: None,
                        worker_failed: false,
                    }));
                }
            }
        }
    }

    pub(super) fn retain_new_worker(&mut self, id: u64, worker: JoinHandle<()>) {
        if let Some(Phase::Saving(save)) = &mut self.new_document.phase
            && save.request_id == id
        {
            save.worker = Some(worker);
        }
    }

    pub(super) fn new_worker_retired(&mut self, id: u64) -> bool {
        let Some(Phase::Saving(save)) = &mut self.new_document.phase else {
            return true;
        };
        if save.request_id != id {
            return true;
        }
        if save
            .worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
        {
            return false;
        }
        if let Some(worker) = save.worker.take()
            && worker.join().is_err()
        {
            save.armed = false;
            save.worker_failed = true;
        }
        true
    }

    pub(super) fn abort_new_save(&mut self) -> bool {
        if matches!(self.new_document.phase, Some(Phase::Saving(_))) {
            self.new_document.phase = None;
            true
        } else {
            false
        }
    }

    pub(super) fn finish_new_save(
        &mut self,
        job: &FileJob,
        result: &CommandResult,
        acknowledged: bool,
        engine: &mut Engine,
        error: &mut Option<String>,
    ) {
        let id = job.request_id;
        let format = job.format;
        let action = &job.action;
        if !matches!(&self.new_document.phase, Some(Phase::Saving(save)) if save.request_id == id) {
            return;
        }
        let Some(Phase::Saving(save)) = self.new_document.phase.take() else {
            return;
        };
        let state = engine.state();
        let current = save.request_id == id
            && save.armed
            && acknowledged
            && engine.capture_stamp_matches(&save.stamp)
            && state.close == CloseState::Open
            && state.close_epoch == save.close_epoch
            && state.revision == save.revision
            && result.revision == save.revision
            && state.ui_file.epoch == id
            && state.ui_file.request.is_none()
            && !state.file.dirty
            && result.file.path == state.file.path
            && result.file.format == state.file.format
            && state.file.format == format
            && matches!(action, FileAction::Save { .. })
            && result.file_operation.as_ref().is_some_and(|receipt| {
                receipt.operation == "save"
                    && receipt.input_revision == save.revision
                    && receipt.format == format
                    && state.file.path.as_deref() == Some(receipt.path.as_str())
                    && receipt.ui_request_id == Some(id)
                    && receipt.ui_request_current == Some(true)
            });
        if current {
            *error = engine
                .execute(Command::NewDocument(NewDocument {
                    expected_revision: save.revision,
                    discard_unsaved: false,
                }))
                .err()
                .map(|failure| format!("{failure}; no new document was created."));
        } else {
            *error = Some(if save.worker_failed {
                "The Save worker did not retire normally. No new document was created; check the saved destination."
            } else { "The Save result or New document context changed. No new document was created." }.into());
        }
    }

    pub(super) fn cancel_new_for_close(&mut self, engine: &mut Engine, error: &mut Option<String>) {
        if self.new_document.pending() && self.job.is_none() {
            self.new_document.phase = None;
            self.cancel_picker(engine, error);
        } else {
            self.new_document.disarm();
        }
    }

    /// A poisoned Engine cannot authorize New or acknowledge a receipt; still retire its worker.
    pub(crate) fn deactivate_new(&mut self, context: &egui::Context, error: &mut Option<String>) {
        let id = match &self.new_document.phase {
            Some(Phase::Saving(save)) => save.request_id,
            Some(Phase::Decision(_)) => {
                self.new_document.phase = None;
                return;
            }
            None => return,
        };
        self.new_document.disarm();
        if let Some(job) = &mut self.job
            && job.request_id == id
        {
            job.cancel.cancel();
            job.cancelling = true;
        }
        if !self.new_worker_retired(id) {
            context.request_repaint_after(Duration::from_millis(40));
            return;
        }
        self.new_document.phase = None;
        self.chooser.cancel();
        self.request_id = None;
        if self.job.as_ref().is_some_and(|job| job.request_id == id)
            && let Some(job) = self.job.take()
        {
            *error = Some(match job.result.try_recv() {
                Ok(Ok(_)) => "Save completed before the Engine became unavailable; no new document was created.".into(),
                Ok(Err(failure)) => format!("{} No new document was created.", failure.message),
                Err(_) => "The Save worker stopped without a result. No new document was created; check the destination.".into(),
            });
        }
    }
}
