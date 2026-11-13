//! Native file-menu, background execution and unsaved-close lifecycle.

#[cfg(test)]
mod close_tests;

use crate::files::{FileAction, FileChooser, FileMode};
use bondline_commands::{
    Cancellation, CloseAction, CloseDocument, CloseState, Command, CommandResult, Engine,
    ErrorInfo, OpenCdxml, SaveCdxml,
};
use bondline_mcp::CommandExecutor;
use eframe::egui;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Duration,
};

struct FileJob {
    action: FileAction,
    result: Receiver<Result<CommandResult, ErrorInfo>>,
    cancel: Cancellation,
    cancelling: bool,
    close_save: Option<CloseSave>,
}

#[derive(Clone, Copy)]
struct CloseSave {
    revision: u64,
    epoch: u64,
}

enum CloseChoice {
    Save,
    Discard,
    Cancel,
}

pub(crate) struct FileWorkflow {
    chooser: FileChooser,
    directory: PathBuf,
    job: Option<FileJob>,
    save_for_close: Option<CloseSave>,
}

impl FileWorkflow {
    pub(crate) fn new(directory: PathBuf) -> Self {
        Self {
            chooser: FileChooser::default(),
            directory,
            job: None,
            save_for_close: None,
        }
    }

    pub(crate) fn blocks_authoring(&self, state: &CommandResult) -> bool {
        self.chooser.is_open() || state.close != CloseState::Open
    }

    pub(crate) fn is_busy(&self) -> bool {
        self.job.is_some()
    }

    fn sync_close(&mut self, state: &CommandResult) {
        if matches!(state.close, CloseState::Approved | CloseState::Committed) {
            self.chooser.cancel();
        }
        let current = |ticket: CloseSave| {
            state.close == CloseState::NeedsDecision
                && ticket.epoch == state.close_epoch
                && ticket.revision == state.revision
        };
        if self.save_for_close.is_some_and(|ticket| !current(ticket)) {
            self.save_for_close = None;
            self.chooser.cancel();
        }
        if let Some(job) = &mut self.job
            && job.close_save.is_some_and(|ticket| !current(ticket))
        {
            job.close_save = None;
        }
    }

    fn begin(&mut self, mode: FileMode, state: &CommandResult) {
        let path = state.file.path.as_deref().map(PathBuf::from);
        let directory = path
            .as_ref()
            .and_then(|path| path.parent())
            .map_or_else(|| self.directory.clone(), std::path::Path::to_owned);
        let name = if mode == FileMode::Open {
            String::new()
        } else {
            path.as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("Untitled.cdxml")
                .to_owned()
        };
        self.chooser.begin(mode, directory, name);
    }

    fn request_save(&mut self, state: &CommandResult, save_as: bool) -> Option<FileAction> {
        if !save_as && let Some(path) = &state.file.path {
            return Some(FileAction::Save {
                path: path.clone(),
                overwrite: true,
            });
        }
        self.begin(FileMode::Save, state);
        None
    }

    pub(crate) fn menu(
        &mut self,
        ui: &mut egui::Ui,
        engine: &mut Engine,
        error: &mut Option<String>,
    ) -> Option<FileAction> {
        let state = engine.state();
        self.sync_close(&state);
        let mut action = None;
        let available = !self.is_busy() && !self.blocks_authoring(&state);
        let modifier = if cfg!(target_os = "macos") {
            "Cmd"
        } else {
            "Ctrl"
        };
        if !self.blocks_authoring(&state)
            && ui.input_mut(|input| {
                input.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::W,
                ))
            })
        {
            self.request_close(engine, error);
        } else if available {
            let selected = ui.input_mut(|input| {
                if input.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::S,
                )) {
                    Some(2)
                } else if input.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::S,
                )) {
                    Some(1)
                } else if input.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::O,
                )) {
                    Some(0)
                } else {
                    None
                }
            });
            match selected {
                Some(0) => self.begin(FileMode::Open, &state),
                Some(1) => action = self.request_save(&state, false),
                Some(2) => action = self.request_save(&state, true),
                _ => {}
            }
        }
        ui.menu_button("File", |ui| {
            for (index, label, shortcut) in [
                (0, "Open…", format!("{modifier}+O")),
                (1, "Save", format!("{modifier}+S")),
                (2, "Save As…", format!("{modifier}+Shift+S")),
            ] {
                if ui
                    .add_enabled(available, egui::Button::new(label).shortcut_text(shortcut))
                    .clicked()
                {
                    match index {
                        0 => self.begin(FileMode::Open, &state),
                        1 => action = self.request_save(&state, false),
                        _ => action = self.request_save(&state, true),
                    }
                    ui.close_menu();
                }
            }
            ui.separator();
            if ui
                .add_enabled(
                    !self.blocks_authoring(&engine.state()),
                    egui::Button::new("Close Window").shortcut_text(format!("{modifier}+W")),
                )
                .clicked()
            {
                self.request_close(engine, error);
                ui.close_menu();
            }
        });
        action
    }

    pub(crate) fn job_status(&mut self, ui: &mut egui::Ui) {
        if let Some(job) = &mut self.job {
            ui.separator();
            ui.spinner();
            ui.label(if job.cancelling {
                "Cancelling file operation…"
            } else if matches!(job.action, FileAction::Open { .. }) {
                "Opening CDXML…"
            } else {
                "Saving CDXML…"
            });
            if ui
                .add_enabled(!job.cancelling, egui::Button::new("Cancel Operation"))
                .clicked()
            {
                job.cancel.cancel();
                job.cancelling = true;
            }
            ui.ctx().request_repaint_after(Duration::from_millis(40));
        }
    }

    pub(crate) fn request_close(&mut self, engine: &mut Engine, error: &mut Option<String>) {
        self.close_command(engine, CloseAction::Request, error);
    }

    fn close_command(
        &mut self,
        engine: &mut Engine,
        action: CloseAction,
        error: &mut Option<String>,
    ) {
        *error = engine
            .execute(Command::Close(CloseDocument {
                expected_revision: engine.state().revision,
                action,
            }))
            .err()
            .map(|error| error.to_string());
        self.sync_close(&engine.state());
    }

    fn choose_close(
        &mut self,
        choice: CloseChoice,
        engine: &mut Engine,
        error: &mut Option<String>,
    ) -> Option<FileAction> {
        match choice {
            CloseChoice::Save => {
                let state = engine.state();
                if state.close != CloseState::NeedsDecision {
                    return None;
                }
                self.save_for_close = Some(CloseSave {
                    revision: state.revision,
                    epoch: state.close_epoch,
                });
                self.request_save(&state, false)
            }
            CloseChoice::Discard => {
                self.close_command(engine, CloseAction::Discard, error);
                None
            }
            CloseChoice::Cancel => {
                self.close_command(engine, CloseAction::Cancel, error);
                None
            }
        }
    }

    /// Caller holds the shared Engine mutex; activity includes remote workers/reply publication.
    pub(crate) fn complete_close_if_idle(
        &self,
        engine: &mut Engine,
        active_work: bool,
        pending_file_action: bool,
    ) -> bool {
        !pending_file_action && !self.is_busy() && !active_work && engine.complete_close()
    }

    pub(crate) fn keep_open_after_file_failure(
        &mut self,
        engine: &mut Engine,
        error: &mut Option<String>,
    ) {
        if engine.state().close == CloseState::Approved {
            let mut cancellation_error = None;
            self.close_command(engine, CloseAction::Cancel, &mut cancellation_error);
            if let Some(failure) = cancellation_error {
                let original = error.take().unwrap_or_default();
                *error = Some(format!("{original}; close cancellation failed: {failure}"));
            }
        }
    }

    pub(crate) fn dialogs(
        &mut self,
        context: &egui::Context,
        engine: &mut Engine,
        error: &mut Option<String>,
        active_work: bool,
    ) -> Option<FileAction> {
        let state = engine.state();
        self.sync_close(&state);
        if self.chooser.is_open() {
            let close_save = self.save_for_close.is_some();
            let action = self.chooser.show(context, state.file.dirty);
            if close_save && !self.chooser.is_open() && action.is_none() {
                self.close_command(engine, CloseAction::Cancel, error);
            }
            return action;
        }
        if matches!(state.close, CloseState::Open | CloseState::Committed) {
            return None;
        }
        let busy = self.is_busy() || active_work;
        if busy {
            context.request_repaint_after(Duration::from_millis(40));
        }
        if state.close == CloseState::Approved && !busy {
            return None;
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.close_command(engine, CloseAction::Cancel, error);
            return None;
        }
        let mut action = None;
        egui::Window::new("Unsaved changes").id(egui::Id::new("bondline-unsaved-close"))
            .anchor(egui::Align2::CENTER_CENTER,egui::Vec2::ZERO).order(egui::Order::Foreground)
            .collapsible(false).resizable(false).show(context,|ui| {
                if busy {
                    ui.label("Document work is still running. The window will stay open until its result is known.");
                    if ui.button("Keep Window Open").clicked() {self.close_command(engine,CloseAction::Cancel,error);}
                    return;
                }
                ui.label(if state.file.dirty {"Save your changes before closing Bondline?"} else {"Close this document?"});
                ui.horizontal(|ui| {
                    for (label,choice) in [("Cancel",CloseChoice::Cancel),("Discard Changes",CloseChoice::Discard)] {
                        if ui.button(label).clicked() {action = self.choose_close(choice,engine,error);}
                    }
                    if ui.button(if state.file.dirty {"Save"} else {"Close"}).clicked() {
                        if state.file.dirty {action=self.choose_close(CloseChoice::Save,engine,error);} else {self.request_close(engine,error);}
                    }
                });
            });
        action
    }

    pub(crate) fn start(
        &mut self,
        action: FileAction,
        executor: &CommandExecutor,
        context: &egui::Context,
    ) -> Result<(), String> {
        if self.is_busy() {
            return Err("A file operation is already running.".to_owned());
        }
        let command = match &action {
            FileAction::Open {
                path,
                discard_unsaved,
            } => Command::OpenCdxml(OpenCdxml {
                path: path.clone(),
                discard_unsaved: *discard_unsaved,
            }),
            FileAction::Save { path, overwrite } => Command::SaveCdxml(SaveCdxml {
                path: path.clone(),
                overwrite: *overwrite,
            }),
        };
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let worker_executor = executor.clone();
        let context = context.clone();
        let (sender, result) = mpsc::sync_channel(1);
        if let Err(error) = std::thread::Builder::new()
            .name("bondline-ui-file".to_owned())
            .spawn(move || {
                let result = worker_executor.execute_blocking(command, &worker_cancel);
                let _ = sender.send(result);
                context.request_repaint();
            })
        {
            let message = format!("Cannot start file operation: {error}");
            self.chooser
                .failed(action, "worker_failed", message.clone());
            return Err(message);
        }
        self.job = Some(FileJob {
            action,
            result,
            cancel,
            cancelling: false,
            close_save: self.save_for_close.take(),
        });
        Ok(())
    }

    pub(crate) fn poll(&mut self, engine: &mut Engine, error: &mut Option<String>) {
        let current = engine.state();
        self.sync_close(&current);
        let Some(job) = &self.job else {
            return;
        };
        let completed = match job.result.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err(ErrorInfo {code:"worker_failed",message:"File worker stopped before reporting its result. Check the destination before retrying.".to_owned()}),
        };
        let Some(job) = self.job.take() else {
            return;
        };
        match completed {
            Ok(result) => {
                *error = None;
                if let Some(path) = &result.file.path
                    && let Some(parent) = std::path::Path::new(path).parent()
                {
                    self.directory = parent.to_owned();
                }
                if let Some(ticket) = job.close_save {
                    let current_receipt = result.file_operation.as_ref().is_some_and(|receipt| {
                        current.file.path.as_deref() == Some(receipt.path.as_str())
                            && matches!(job.action, FileAction::Save { .. })
                            && receipt.operation == "save"
                            && receipt.input_revision == current.revision
                    });
                    if current_receipt
                        && current.close == CloseState::NeedsDecision
                        && current.close_epoch == ticket.epoch
                        && current.revision == ticket.revision
                        && !current.file.dirty
                        && result.revision == current.revision
                        && result.file.path == current.file.path
                    {
                        self.request_close(engine, error);
                    } else {
                        *error = Some("The document changed during the file operation. Review the current changes before closing.".to_owned());
                    }
                }
            }
            Err(failure) => {
                *error = Some(failure.message.clone());
                self.keep_open_after_file_failure(engine, error);
                self.save_for_close = job.close_save;
                self.chooser
                    .failed(job.action, failure.code, failure.message);
            }
        }
    }
}

impl Drop for FileWorkflow {
    fn drop(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bondline_commands::Engine;

    fn edit(engine: &mut Engine) -> Result<(), bondline_commands::Error> {
        engine.dispatch(
            "page.set_size",
            serde_json::json!({"page_id":1,"width_pt":640.0,"height_pt":480.0}),
        )?;
        Ok(())
    }

    #[test]
    fn ordinary_save_reuses_the_known_path_but_save_as_requires_an_explicit_choice() {
        let mut state = Engine::default().state();
        state.file.path = Some("/tmp/known.cdxml".to_owned());
        state.file.dirty = true;
        let mut workflow = FileWorkflow::new(std::env::temp_dir());
        assert_eq!(
            workflow.request_save(&state, false),
            Some(FileAction::Save {
                path: "/tmp/known.cdxml".to_owned(),
                overwrite: true
            })
        );
        assert!(!workflow.chooser.is_open());
        assert!(workflow.request_save(&state, true).is_none());
        assert!(workflow.chooser.is_open());
        assert!(workflow.blocks_authoring(&state));
    }

    #[test]
    fn native_open_shortcut_blocks_background_undo_and_drawing_until_escape()
    -> Result<(), Box<dyn std::error::Error>> {
        let context = egui::Context::default();
        crate::configure_context(&context);
        let mut engine = Engine::default();
        edit(&mut engine)?;
        let revision = engine.state().revision;
        let mut workflow = FileWorkflow::new(std::env::temp_dir());
        let mut drawing = crate::DrawingState::default();
        let shortcut = |key| egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        };
        let point = egui::pos2(420.0, 300.0);
        let pointer = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        for events in [
            vec![],
            vec![shortcut(egui::Key::O)],
            vec![shortcut(egui::Key::Z)],
            vec![
                egui::Event::PointerMoved(point),
                pointer(point, true),
                egui::Event::PointerMoved(point + egui::vec2(80.0, 30.0)),
                pointer(point + egui::vec2(80.0, 30.0), false),
            ],
        ] {
            context.begin_pass(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 700.0),
                )),
                events,
                ..Default::default()
            });
            let mut error = None;
            assert!(
                crate::show_workspace_with_files(
                    &context,
                    &mut engine,
                    &mut error,
                    &mut drawing,
                    Some(&mut workflow),
                    false,
                )
                .is_none()
            );
            let _ = context.end_pass();
            assert!(error.is_none());
        }
        assert!(workflow.chooser.is_open());
        assert_eq!(engine.state().revision, revision);
        assert!(
            engine
                .document()
                .pages()
                .first()
                .ok_or("page")?
                .fragments()
                .is_empty()
        );
        context.begin_pass(egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: Some(egui::Key::Escape),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        });
        crate::show_workspace_with_files(
            &context,
            &mut engine,
            &mut None,
            &mut drawing,
            Some(&mut workflow),
            false,
        );
        let _ = context.end_pass();
        assert!(!workflow.chooser.is_open());
        Ok(())
    }

    #[test]
    fn close_shortcut_and_escape_preserve_the_dirty_document()
    -> Result<(), Box<dyn std::error::Error>> {
        let context = egui::Context::default();
        crate::configure_context(&context);
        let mut engine = Engine::default();
        edit(&mut engine)?;
        let original = engine.stored_document().clone();
        let revision = engine.state().revision;
        let mut workflow = FileWorkflow::new(std::env::temp_dir());
        let mut drawing = crate::DrawingState::default();
        for (key, modifiers, blocked) in [
            (egui::Key::W, egui::Modifiers::COMMAND, true),
            (egui::Key::Escape, egui::Modifiers::NONE, false),
        ] {
            context.begin_pass(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 700.0),
                )),
                events: vec![egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed: true,
                    repeat: false,
                    modifiers,
                }],
                ..Default::default()
            });
            crate::show_workspace_with_files(
                &context,
                &mut engine,
                &mut None,
                &mut drawing,
                Some(&mut workflow),
                false,
            );
            let _ = context.end_pass();
            assert_eq!(workflow.blocks_authoring(&engine.state()), blocked);
            assert!(!workflow.complete_close_if_idle(&mut engine, false, false));
            assert_eq!(engine.state().revision, revision);
            assert_eq!(engine.stored_document(), &original);
        }
        Ok(())
    }
}
