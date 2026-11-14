//! One bounded native file-presentation intent, separate from document persistence.

use crate::{
    CloseAction, CloseDocument, CloseState, Command, CommandResult, Engine, Error, FileFormat,
    Prepared, SaveCdxml,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiFileAction {
    Open,
    Save,
    SaveAs,
    Cancel,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UiFile {
    pub expected_revision: u64,
    pub action: UiFileAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_epoch: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFilePhase {
    Requested,
    Accepted,
    Running,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UiFileRequest {
    pub id: u64,
    pub action: UiFileAction,
    pub input_revision: u64,
    pub input_path: Option<String>,
    pub close_epoch: Option<u64>,
    pub phase: UiFilePhase,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UiFileState {
    pub available: bool,
    pub epoch: u64,
    pub request: Option<UiFileRequest>,
}

/// Acceptance by a native host is not a claim that pixels have been displayed.
#[derive(Clone, Debug)]
pub enum UiFilePlan {
    OpenPicker,
    SavePicker,
    SaveExisting {
        format: FileFormat,
        params: SaveCdxml,
    },
}

#[derive(Debug)]
struct ActiveRequest {
    view: UiFileRequest,
    generation: u64,
    close_epoch: u64,
    valid: bool,
    completed_current: bool,
}

#[derive(Debug, Default)]
pub(super) struct FileUiPolicy {
    available: bool,
    pub(super) epoch: u64,
    active: Option<ActiveRequest>,
}

impl Engine {
    pub(super) fn file_ui_state(&self) -> UiFileState {
        UiFileState {
            available: self.file_ui.available,
            epoch: self.file_ui.epoch,
            request: self
                .file_ui
                .active
                .as_ref()
                .map(|active| active.view.clone()),
        }
    }

    /// Register or detach the actual native host; headless engines default to absent.
    /// Detach prevents an obsolete picker from being adopted by a later host.
    pub fn set_ui_file_host(&mut self, available: bool) {
        if self.file_ui.available != available {
            self.invalidate_file_ui();
            self.file_ui.available = available;
        }
    }

    pub(super) fn invalidate_file_ui(&mut self) {
        if let Some(active) = &mut self.file_ui.active {
            if active.view.phase == UiFilePhase::Running {
                // Published work must still finish truthfully, but cannot reopen an old picker.
                active.valid = false;
                active.completed_current = false;
            } else {
                self.file_ui.active = None;
            }
        }
    }

    fn file_ui_context_matches(&self, active: &ActiveRequest) -> bool {
        active.valid
            && active.generation == self.generation
            && active.view.input_revision == self.revision
            && active.view.input_path == self.file_path
            && active.close_epoch == self.close_policy.epoch
            && match active.view.close_epoch {
                Some(epoch) => {
                    epoch == self.close_policy.epoch
                        && self.close_state() == CloseState::NeedsDecision
                }
                None => self.close_state() == CloseState::Open,
            }
    }

    fn current_ui_file(&self, id: u64) -> Result<&ActiveRequest, Error> {
        if self.close_state() == CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        if !self.file_ui.available {
            return Err(Error::UiUnavailable);
        }
        self.file_ui
            .active
            .as_ref()
            .filter(|active| active.view.id == id && self.file_ui_context_matches(active))
            .ok_or(Error::UiStaleRequest)
    }

    pub(super) fn ui_file(&mut self, params: UiFile) -> Result<CommandResult, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if (params.action == UiFileAction::Cancel) != params.request_id.is_some()
            || (params.close_epoch.is_some() && params.action != UiFileAction::Save)
        {
            return Err(Error::InvalidUiFileParameters(
                "request_id is required only for Cancel; close_epoch is allowed only for Save",
            ));
        }
        if !self.file_ui.available {
            return Err(Error::UiUnavailable);
        }
        let next = self
            .file_ui
            .epoch
            .checked_add(1)
            .ok_or(Error::UiEpochExhausted)?;
        if let Some(id) = params.request_id {
            let active = self.current_ui_file(id)?;
            if active.view.phase == UiFilePhase::Running {
                return Err(Error::UiBusy);
            }
            let close_epoch = active.view.close_epoch;
            if close_epoch == Some(self.close_policy.epoch)
                && self.close_state() == CloseState::NeedsDecision
            {
                // close() checks its own epoch before mutation. A failed cancellation
                // must not dismiss the chooser or consume the file-request epoch.
                self.close(CloseDocument {
                    expected_revision: self.revision,
                    action: CloseAction::Cancel,
                })?;
            }
            self.file_ui.active = None;
        } else {
            if self.file_ui.active.is_some() {
                return Err(Error::UiBusy);
            }
            match params.close_epoch {
                Some(epoch)
                    if epoch == self.close_policy.epoch
                        && self.close_state() == CloseState::NeedsDecision => {}
                Some(_) => return Err(Error::UiStaleRequest),
                None if self.close_state() == CloseState::Open => {}
                None => return Err(Error::UiBusy),
            }
            self.file_ui.active = Some(ActiveRequest {
                view: UiFileRequest {
                    id: next,
                    action: params.action,
                    input_revision: self.revision,
                    input_path: self.file_path.clone(),
                    close_epoch: params.close_epoch,
                    phase: UiFilePhase::Requested,
                },
                generation: self.generation,
                close_epoch: self.close_policy.epoch,
                valid: true,
                completed_current: false,
            });
        }
        self.file_ui.epoch = next;
        Ok(CommandResult {
            ui_file_operation: Some(next),
            ..self.state()
        })
    }

    /// Native acknowledgement of one current intent. The request stays addressable
    /// so a remote Cancel can dismiss a picker that the host has already accepted.
    pub fn claim_ui_file(&mut self, id: u64) -> Result<UiFilePlan, Error> {
        let active = self.current_ui_file(id)?;
        if active.view.phase == UiFilePhase::Running {
            return Err(Error::UiBusy);
        }
        let plan = match active.view.action {
            UiFileAction::Open => UiFilePlan::OpenPicker,
            UiFileAction::Save => {
                active
                    .view
                    .input_path
                    .as_ref()
                    .map_or(UiFilePlan::SavePicker, |path| UiFilePlan::SaveExisting {
                        format: self.file_format,
                        params: SaveCdxml {
                            path: path.clone(),
                            overwrite: true,
                        },
                    })
            }
            UiFileAction::SaveAs => UiFilePlan::SavePicker,
            UiFileAction::Cancel => return Err(Error::UiStaleRequest),
        };
        if let Some(active) = &mut self.file_ui.active {
            active.view.phase = UiFilePhase::Accepted;
        }
        Ok(plan)
    }

    /// Validate a host selection and prepare its actual file command under the same
    /// engine lock. Adapters must reserve their worker before calling this method.
    /// No request is consumed when validation or ordinary command preparation fails.
    pub fn prepare_ui_file(&mut self, id: u64, actual: Command) -> Result<Prepared, Error> {
        let active = self.current_ui_file(id)?;
        if active.view.phase != UiFilePhase::Accepted {
            return Err(Error::UiBusy);
        }
        let matches = match (&active.view.action, &actual) {
            (
                UiFileAction::Open,
                Command::OpenCdxml(_)
                | Command::OpenCdx(_)
                | Command::OpenMol(_)
                | Command::OpenSdf(_),
            ) => true,
            (
                UiFileAction::SaveAs,
                Command::SaveCdxml(_)
                | Command::SaveCdx(_)
                | Command::SaveMol(_)
                | Command::SaveSdf(_),
            ) => true,
            (UiFileAction::Save, Command::SaveCdxml(params)) => {
                active.view.input_path.as_ref().is_none_or(|path| {
                    self.file_format == FileFormat::Cdxml
                        && path == &params.path
                        && params.overwrite
                })
            }
            (UiFileAction::Save, Command::SaveMol(params)) => {
                let format = self.mol_save_format(params.version)?;
                active.view.input_path.as_ref().is_none_or(|path| {
                    self.file_format == format && path == &params.path && params.overwrite
                })
            }
            (UiFileAction::Save, Command::SaveCdx(params)) => {
                active.view.input_path.as_ref().is_none_or(|path| {
                    self.file_format == FileFormat::Cdx && path == &params.path && params.overwrite
                })
            }
            (UiFileAction::Save, Command::SaveSdf(params)) => {
                active.view.input_path.as_ref().is_none_or(|path| {
                    self.file_format == FileFormat::Sdf && path == &params.path && params.overwrite
                })
            }
            _ => false,
        };
        if !matches {
            return Err(Error::InvalidUiFileParameters(
                "selection does not match the accepted file request",
            ));
        }
        let mut prepared = self.prepare(actual)?;
        prepared.bind_ui_request(id)?;
        if let Some(active) = &mut self.file_ui.active {
            active.view.phase = UiFilePhase::Running;
        }
        Ok(prepared)
    }

    pub(super) fn ui_file_request_is_current(&self, id: u64) -> bool {
        self.current_ui_file(id)
            .is_ok_and(|active| active.view.phase == UiFilePhase::Running)
    }

    pub(super) fn mark_ui_file_completed(&mut self, id: Option<u64>, current: Option<bool>) {
        if current == Some(true)
            && let Some(active) = &mut self.file_ui.active
            && Some(active.view.id) == id
        {
            // Its own publication may change the baseline; later changes invalidate this.
            active.completed_current = true;
        }
    }

    /// Consume a native result or retain its same picker for a guarded retry.
    /// A false retry result forbids reopening an obsolete picker. Running publication
    /// is never cancelled or rolled back by this presentation acknowledgement. Success
    /// returns true only while the applied completion has not subsequently been invalidated.
    pub fn finish_ui_file(&mut self, id: u64, retry: bool) -> bool {
        let Some(active) = &self.file_ui.active else {
            return false;
        };
        if active.view.id != id {
            return false;
        }
        let can_retry = self.file_ui.available && self.file_ui_context_matches(active);
        let can_acknowledge = self.file_ui.available && active.completed_current;
        if retry && can_retry {
            if let Some(active) = &mut self.file_ui.active {
                active.view.phase = UiFilePhase::Accepted;
            }
            true
        } else {
            self.file_ui.active = None;
            !retry && can_acknowledge
        }
    }
}
