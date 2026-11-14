//! Native file-menu, background execution and unsaved-close lifecycle.

#[cfg(test)]
mod cdx_tests;
#[cfg(test)]
mod close_tests;
#[cfg(test)]
mod mol_tests;
#[cfg(test)]
mod request_tests;
#[cfg(test)]
mod v3000_tests;

use crate::files::{FileAction, FileChooser, FileChooserEvent, FileMode, format_extension};
use bondline_commands::{
    Cancellation, CloseAction, CloseDocument, CloseState, Command, CommandResult, Engine,
    ErrorInfo, FileFormat, MolVersionChoice, OpenCdx, OpenCdxml, OpenMol, SaveCdx, SaveCdxml,
    SaveMol, UiFile, UiFileAction, UiFilePhase, UiFilePlan,
};
use bondline_mcp::CommandExecutor;
use eframe::egui;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Duration,
};

struct FileJob {
    request_id: u64,
    format: FileFormat,
    action: FileAction,
    result: Receiver<Result<CommandResult, ErrorInfo>>,
    cancel: Cancellation,
    cancelling: bool,
    close_save: Option<CloseSave>,
}

pub(crate) struct FileIntentAction {
    request_id: u64,
    format: FileFormat,
    action: FileAction,
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
    request_id: Option<u64>,
}

impl FileWorkflow {
    pub(crate) fn new(directory: PathBuf) -> Self {
        Self {
            chooser: FileChooser::default(),
            directory,
            job: None,
            save_for_close: None,
            request_id: None,
        }
    }

    pub(crate) fn blocks_authoring(&self, state: &CommandResult) -> bool {
        self.chooser.is_open()
            || state.close != CloseState::Open
            || state
                .ui_file
                .request
                .as_ref()
                .is_some_and(|request| request.phase != UiFilePhase::Running)
    }

    pub(crate) fn is_busy(&self) -> bool {
        self.job.is_some()
    }

    fn sync_close(&mut self, state: &CommandResult) {
        if self.request_id.is_some()
            && self.request_id != state.ui_file.request.as_ref().map(|request| request.id)
        {
            self.chooser.cancel();
            self.request_id = None;
            self.save_for_close = None;
        }
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
                .map_or_else(
                    || format!("Untitled.{}", format_extension(state.file.format)),
                    str::to_owned,
                )
        };
        self.chooser
            .begin_with_format(mode, directory, name, state.file.format);
    }

    pub(crate) fn menu(
        &mut self,
        ui: &mut egui::Ui,
        engine: &mut Engine,
        error: &mut Option<String>,
    ) -> Option<FileIntentAction> {
        let state = engine.state();
        self.sync_close(&state);
        let controls = engine.controls();
        let shortcut = crate::control_shortcut(ui.ctx(), &controls, "File");
        let menu = crate::control_menu(ui, &controls, "File");
        let command = shortcut.or(menu);
        if let Some(command) = command {
            *error = engine.execute(command).err().map(|error| error.to_string());
            self.sync_close(&engine.state());
        }
        None
    }

    fn request_file(
        &mut self,
        engine: &mut Engine,
        action: UiFileAction,
        close_epoch: Option<u64>,
        error: &mut Option<String>,
    ) {
        *error = engine
            .execute(Command::UiFile(UiFile {
                expected_revision: engine.state().revision,
                action,
                request_id: None,
                close_epoch,
            }))
            .err()
            .map(|error| error.to_string());
    }

    fn adopt_request(
        &mut self,
        engine: &mut Engine,
        error: &mut Option<String>,
    ) -> Option<FileIntentAction> {
        let state = engine.state();
        self.sync_close(&state);
        let request = state.ui_file.request.as_ref()?;
        if request.phase != UiFilePhase::Requested || self.is_busy() {
            return None;
        }
        let plan = match engine.claim_ui_file(request.id) {
            Ok(plan) => plan,
            Err(failure) => {
                *error = Some(failure.to_string());
                engine.finish_ui_file(request.id, false);
                return None;
            }
        };
        self.request_id = Some(request.id);
        self.save_for_close = request.close_epoch.map(|epoch| CloseSave {
            epoch,
            revision: request.input_revision,
        });
        match plan {
            UiFilePlan::OpenPicker => self.begin(FileMode::Open, &state),
            UiFilePlan::SavePicker => self.begin(FileMode::Save, &state),
            UiFilePlan::SaveExisting { format, params } => {
                return Some(FileIntentAction {
                    request_id: request.id,
                    format,
                    action: FileAction::Save {
                        path: params.path,
                        overwrite: params.overwrite,
                    },
                });
            }
        }
        None
    }

    fn cancel_picker(&mut self, engine: &mut Engine, error: &mut Option<String>) {
        let Some(request_id) = self.request_id else {
            return;
        };
        let command = engine
            .controls()
            .into_iter()
            .find(|control| {
                control.id == "file.cancel"
                    && control.enabled
                    && matches!(&control.command, Command::UiFile(params) if params.request_id == Some(request_id))
            })
            .map(|control| control.command);
        if let Some(command) = command {
            *error = engine.execute(command).err().map(|error| error.to_string());
        }
        self.sync_close(&engine.state());
    }

    pub(crate) fn job_status(&mut self, ui: &mut egui::Ui) {
        if let Some(job) = &mut self.job {
            ui.separator();
            ui.spinner();
            ui.label(if job.cancelling {
                "Cancelling file operation…"
            } else if matches!(job.action, FileAction::Open { .. }) {
                match job.format {
                    FileFormat::Cdxml => "Opening CDXML…",
                    FileFormat::Cdx => "Opening CDX…",
                    FileFormat::MolV2000 | FileFormat::MolV3000 => "Opening MOL…",
                }
            } else {
                match job.format {
                    FileFormat::Cdxml => "Saving CDXML…",
                    FileFormat::Cdx => "Saving CDX…",
                    FileFormat::MolV2000 => "Saving MOL V2000…",
                    FileFormat::MolV3000 => "Saving MOL V3000…",
                }
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
    ) -> Option<FileIntentAction> {
        match choice {
            CloseChoice::Save => {
                let state = engine.state();
                if state.close != CloseState::NeedsDecision {
                    return None;
                }
                self.request_file(engine, UiFileAction::Save, Some(state.close_epoch), error);
                None
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
    ) -> Option<FileIntentAction> {
        if let Some(action) = self.adopt_request(engine, error) {
            return Some(action);
        }
        let state = engine.state();
        if self.chooser.is_open() {
            match self.chooser.show(context, state.file.dirty) {
                Some(FileChooserEvent::Submit(action)) => {
                    return self.request_id.map(|request_id| FileIntentAction {
                        request_id,
                        format: self.chooser.format(),
                        action,
                    });
                }
                Some(FileChooserEvent::Cancel) => self.cancel_picker(engine, error),
                None => {}
            }
            return None;
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
        intent: FileIntentAction,
        executor: &CommandExecutor,
        context: &egui::Context,
    ) -> Result<(), String> {
        if self.is_busy() {
            return Err("A file operation is already running.".to_owned());
        }
        let FileIntentAction {
            request_id,
            format,
            action,
        } = intent;
        let command = match (format, &action) {
            (
                FileFormat::Cdxml,
                FileAction::Open {
                    path,
                    discard_unsaved,
                },
            ) => Command::OpenCdxml(OpenCdxml {
                path: path.clone(),
                discard_unsaved: *discard_unsaved,
            }),
            (
                FileFormat::Cdx,
                FileAction::Open {
                    path,
                    discard_unsaved,
                },
            ) => Command::OpenCdx(OpenCdx {
                path: path.clone(),
                discard_unsaved: *discard_unsaved,
            }),
            (
                FileFormat::MolV2000 | FileFormat::MolV3000,
                FileAction::Open {
                    path,
                    discard_unsaved,
                },
            ) => Command::OpenMol(OpenMol {
                path: path.clone(),
                discard_unsaved: *discard_unsaved,
                points_per_unit: 24.0,
                origin_x_pt: 72.0,
                origin_y_pt: 72.0,
            }),
            (FileFormat::Cdxml, FileAction::Save { path, overwrite }) => {
                Command::SaveCdxml(SaveCdxml {
                    path: path.clone(),
                    overwrite: *overwrite,
                })
            }
            (FileFormat::Cdx, FileAction::Save { path, overwrite }) => Command::SaveCdx(SaveCdx {
                path: path.clone(),
                overwrite: *overwrite,
            }),
            (FileFormat::MolV2000, FileAction::Save { path, overwrite }) => {
                Command::SaveMol(SaveMol {
                    path: path.clone(),
                    overwrite: *overwrite,
                    version: MolVersionChoice::V2000,
                })
            }
            (FileFormat::MolV3000, FileAction::Save { path, overwrite }) => {
                Command::SaveMol(SaveMol {
                    path: path.clone(),
                    overwrite: *overwrite,
                    version: MolVersionChoice::V3000,
                })
            }
        };
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let work = match executor.prepare_ui_file(request_id, command) {
            Ok(work) => work,
            Err(failure) => {
                if let Ok(mut engine) = executor.engine().lock() {
                    self.failed(request_id, format, action, &failure, &mut engine);
                }
                return Err(failure.message);
            }
        };
        let context = context.clone();
        let (sender, result) = mpsc::sync_channel(1);
        if let Err(error) = std::thread::Builder::new()
            .name("bondline-ui-file".to_owned())
            .spawn(move || {
                let result = work.complete_blocking(&worker_cancel);
                let _ = sender.send(result);
                context.request_repaint();
            })
        {
            let message = format!("Cannot start file operation: {error}");
            if let Ok(mut engine) = executor.engine().lock() {
                self.failed(
                    request_id,
                    format,
                    action,
                    &ErrorInfo {
                        code: "worker_failed",
                        message: message.clone(),
                    },
                    &mut engine,
                );
            }
            return Err(message);
        }
        self.job = Some(FileJob {
            request_id,
            format,
            action,
            result,
            cancel,
            cancelling: false,
            close_save: self.save_for_close.take(),
        });
        Ok(())
    }

    fn failed(
        &mut self,
        id: u64,
        format: FileFormat,
        action: FileAction,
        failure: &ErrorInfo,
        engine: &mut Engine,
    ) {
        let retry = failure.code != "save_published" && failure.code != "recovery_required";
        if engine.finish_ui_file(id, retry) && retry {
            self.request_id = Some(id);
            self.chooser
                .failed_with_format(action, format, failure.code, failure.message.clone());
        } else {
            self.sync_close(&engine.state());
        }
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
                let current_completion = engine.finish_ui_file(job.request_id, false);
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
                            && receipt.format == job.format
                            && receipt.ui_request_id == Some(job.request_id)
                            && receipt.ui_request_current == Some(true)
                    });
                    if current_completion
                        && current_receipt
                        && current.close == CloseState::NeedsDecision
                        && current.close_epoch == ticket.epoch
                        && current.revision == ticket.revision
                        && !current.file.dirty
                        && result.revision == current.revision
                        && result.file.path == current.file.path
                        && current.file.format == job.format
                        && result.file.format == current.file.format
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
                self.failed(job.request_id, job.format, job.action, &failure, engine);
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
    fn ordinary_save_reuses_the_known_path_but_save_as_requires_an_explicit_choice()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = super::close_tests::Directory::new()?;
        let path = directory.0.join("known.cdxml");
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch("document.save_cdxml", serde_json::json!({"path":path}))?;
        let canonical = engine.state().file.path.ok_or("saved path")?;
        let mut workflow = FileWorkflow::new(directory.0.clone());
        workflow.request_file(&mut engine, UiFileAction::Save, None, &mut None);
        let intent = workflow
            .adopt_request(&mut engine, &mut None)
            .ok_or("save existing")?;
        assert_eq!(
            intent.action,
            FileAction::Save {
                path: canonical,
                overwrite: true
            }
        );
        assert!(!workflow.chooser.is_open());
        workflow.cancel_picker(&mut engine, &mut None);
        assert!(engine.state().ui_file.request.is_none());
        workflow.request_file(&mut engine, UiFileAction::SaveAs, None, &mut None);
        assert!(workflow.adopt_request(&mut engine, &mut None).is_none());
        assert!(workflow.chooser.is_open());
        assert!(workflow.blocks_authoring(&engine.state()));
        Ok(())
    }

    #[test]
    fn native_open_shortcut_blocks_background_undo_and_drawing_until_escape()
    -> Result<(), Box<dyn std::error::Error>> {
        let context = egui::Context::default();
        crate::configure_context(&context);
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
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
        engine.set_ui_file_host(true);
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
