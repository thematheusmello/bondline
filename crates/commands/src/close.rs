//! Transient close intent is shared by every adapter, separate from authored history.

use crate::{CommandResult, Engine, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CloseAction {
    #[default]
    Request,
    Discard,
    Cancel,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CloseDocument {
    pub expected_revision: u64,
    #[serde(default)]
    pub action: CloseAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseState {
    Open,
    NeedsDecision,
    Approved,
    Committed,
}

#[derive(Debug)]
struct Approval {
    revision: u64,
    path: Option<String>,
    discard: bool,
    approved: bool,
}

impl Approval {
    fn matches(&self, revision: u64, path: Option<&str>, dirty: bool) -> bool {
        self.revision == revision && self.path.as_deref() == path && (self.discard || !dirty)
    }
}

#[derive(Debug, Default)]
pub(super) struct ClosePolicy {
    approval: Option<Approval>,
    committed: bool,
    pub(super) epoch: u64,
}

impl ClosePolicy {
    fn state(&self, revision: u64, path: Option<&str>, dirty: bool) -> CloseState {
        if self.committed {
            return CloseState::Committed;
        }
        match &self.approval {
            None => CloseState::Open,
            Some(approval) if approval.approved && approval.matches(revision, path, dirty) => {
                CloseState::Approved
            }
            Some(_) => CloseState::NeedsDecision,
        }
    }

    fn apply(
        &mut self,
        action: CloseAction,
        revision: u64,
        path: Option<&str>,
        dirty: bool,
    ) -> Result<(), Error> {
        let epoch = self
            .epoch
            .checked_add(1)
            .ok_or(Error::CloseEpochExhausted)?;
        self.approval = match action {
            CloseAction::Cancel => None,
            CloseAction::Request | CloseAction::Discard => Some(Approval {
                revision,
                path: path.map(str::to_owned),
                discard: action == CloseAction::Discard,
                approved: action == CloseAction::Discard || !dirty,
            }),
        };
        self.epoch = epoch;
        Ok(())
    }
}

impl Engine {
    pub(super) fn close_state(&self) -> CloseState {
        self.close_policy.state(
            self.revision,
            self.file_path.as_deref(),
            self.document != self.saved,
        )
    }

    pub(super) fn invalidate_close(&mut self) {
        self.invalidate_file_ui();
        if let Some(approval) = &mut self.close_policy.approval {
            approval.approved = false;
        }
    }

    pub(super) fn close(&mut self, params: CloseDocument) -> Result<CommandResult, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        self.close_policy.apply(
            params.action,
            self.revision,
            self.file_path.as_deref(),
            self.document != self.saved,
        )?;
        self.invalidate_file_ui();
        let state = self.state();
        Ok(CommandResult {
            close_operation: Some(state.close),
            ..state
        })
    }

    /// Trusted host acknowledgement after all registered workers and UI jobs finish.
    /// This does not initiate a user close action or change authored state/history.
    pub fn complete_close(&mut self) -> bool {
        match self.close_state() {
            CloseState::Approved => {
                self.close_policy.committed = true;
                true
            }
            CloseState::Committed => true,
            CloseState::Open | CloseState::NeedsDecision => false,
        }
    }

    pub(super) fn refresh_close_result(&self, result: &mut CommandResult) {
        result.close = self.close_state();
        result.close_epoch = self.close_policy.epoch;
        result.ui_file = self.file_ui_state();
        if result.controls.is_some() {
            result.controls = Some(self.controls());
        }
        if result.close_operation.is_some() {
            result.close_operation = Some(result.close);
        }
    }
}
