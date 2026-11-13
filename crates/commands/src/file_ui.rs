//! One bounded native file-presentation intent, separate from document persistence.

use crate::{
    CloseAction, CloseDocument, CloseState, Command, CommandResult, Engine, Error, Prepared,
    SaveCdxml,
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
    SaveExisting(SaveCdxml),
}

#[derive(Debug)]
struct ActiveRequest {
    view: UiFileRequest,
    generation: u64,
    close_epoch: u64,
    valid: bool,
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
                    .map_or(UiFilePlan::SavePicker, |path| {
                        UiFilePlan::SaveExisting(SaveCdxml {
                            path: path.clone(),
                            overwrite: true,
                        })
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
            (UiFileAction::Open, Command::OpenCdxml(_)) => true,
            (UiFileAction::SaveAs, Command::SaveCdxml(_)) => true,
            (UiFileAction::Save, Command::SaveCdxml(params)) => active
                .view
                .input_path
                .as_ref()
                .is_none_or(|path| path == &params.path && params.overwrite),
            _ => false,
        };
        if !matches {
            return Err(Error::InvalidUiFileParameters(
                "selection does not match the accepted file request",
            ));
        }
        let prepared = self.prepare(actual)?;
        if let Some(active) = &mut self.file_ui.active {
            active.view.phase = UiFilePhase::Running;
        }
        Ok(prepared)
    }

    /// Consume a native result or retain its same picker for a guarded retry.
    /// A false retry result forbids reopening an obsolete picker. Running publication
    /// is never cancelled or rolled back by this presentation acknowledgement.
    pub fn finish_ui_file(&mut self, id: u64, retry: bool) -> bool {
        let Some(active) = &self.file_ui.active else {
            return false;
        };
        if active.view.id != id {
            return false;
        }
        let can_retry = self.file_ui.available && self.file_ui_context_matches(active);
        if retry && can_retry {
            if let Some(active) = &mut self.file_ui.active {
                active.view.phase = UiFilePhase::Accepted;
            }
            true
        } else {
            self.file_ui.active = None;
            !retry
        }
    }
}
