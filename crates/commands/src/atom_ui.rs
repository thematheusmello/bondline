//! Exact-target native presentation; the current source selects the registered label edit.
use crate::{
    AtomSetLabel, BondToolState, CaptureStamp, CloseState, Command, CommandResult, Engine, Error,
    UiFileContextStamp,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum UiAtomLabel {
    Open {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        page_id: u32,
        #[schemars(range(min = 1))]
        fragment_id: u32,
        #[schemars(range(min = 1))]
        atom_id: u32,
    },
    Cancel {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        request_id: u64,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct UiAtomLabelRequest {
    pub id: u64,
    pub input_revision: u64,
    pub page_id: u32,
    pub fragment_id: u32,
    pub atom_id: u32,
    pub element: u8,
    pub explicit_hydrogens: Option<u16>,
    pub current: bool,
}

#[derive(Debug)]
struct Request {
    view: UiAtomLabelRequest,
    owner: CaptureStamp,
    file_context: UiFileContextStamp,
    file_epoch: u64,
    close_epoch: u64,
    view_epoch: u64,
    tool: BondToolState,
}

#[derive(Debug, Default)]
pub(super) struct AtomLabelUiPolicy {
    epoch: u64,
    active: Option<Request>,
}

impl AtomLabelUiPolicy {
    pub(super) fn has_request(&self) -> bool {
        self.active.is_some()
    }

    pub(super) fn epoch(&self) -> u64 {
        self.epoch
    }
}

impl Engine {
    pub fn ui_atom_label_request(&self) -> Option<UiAtomLabelRequest> {
        let active = self.atom_ui.active.as_ref()?;
        let mut request = active.view.clone();
        request.current = self.ui_atom_label_request_current(request.id);
        Some(request)
    }

    pub fn ui_atom_label_request_current(&self, id: u64) -> bool {
        self.atom_ui.active.as_ref().is_some_and(|active| {
            active.view.id == id
                && self.file_ui_state().available
                && self.capture_stamp_matches(&active.owner)
                && self.ui_file_context_stamp_matches(&active.file_context)
                && self.file_ui.epoch == active.file_epoch
                && self.file_ui_state().request.is_none()
                && !self.preferences_ui.has_request()
                && self.close_state() == CloseState::Open
                && self.close_policy.epoch == active.close_epoch
                && self.view().epoch == active.view_epoch
                && self.bond_tool() == active.tool
        })
    }

    /// Retire only the matching host presentation, including during committed shutdown.
    /// This neither changes authored data nor acknowledges Save or close.
    pub fn finish_ui_atom_label(&mut self, id: u64) -> bool {
        if self
            .atom_ui
            .active
            .as_ref()
            .is_some_and(|active| active.view.id == id)
        {
            self.atom_ui.active = None;
            true
        } else {
            false
        }
    }

    pub(super) fn detach_atom_label(&mut self) {
        self.atom_ui.active = None;
    }

    pub(super) fn ui_atom_label(&mut self, params: UiAtomLabel) -> Result<CommandResult, Error> {
        let expected_revision = match &params {
            UiAtomLabel::Open {
                expected_revision, ..
            }
            | UiAtomLabel::Cancel {
                expected_revision, ..
            } => *expected_revision,
        };
        if expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if !self.file_ui_state().available {
            return Err(Error::UiUnavailable);
        }
        let id = match params {
            UiAtomLabel::Open {
                page_id,
                fragment_id,
                atom_id,
                ..
            } => {
                if self.atom_ui.active.is_some()
                    || self.preferences_ui.has_request()
                    || self.ui_drawing_settings_request().is_some()
                    || self.ui_add_chain_request().is_some()
                    || self.close_state() != CloseState::Open
                    || self.file_ui_state().request.is_some()
                {
                    return Err(Error::UiBusy);
                }
                let document = self.stored_document()?;
                let (_, atom) = if document.has_boxed_atom_labels() {
                    crate::boxed_atom::editable_atom(document, page_id, fragment_id, atom_id)
                } else {
                    crate::atom::editable_atom(document, page_id, fragment_id, atom_id)
                }?;
                let id = self
                    .atom_ui
                    .epoch
                    .checked_add(1)
                    .ok_or(Error::UiEpochExhausted)?;
                let request = Request {
                    view: UiAtomLabelRequest {
                        id,
                        input_revision: self.revision,
                        page_id,
                        fragment_id,
                        atom_id,
                        element: atom.element.get(),
                        explicit_hydrogens: atom.explicit_hydrogens,
                        current: true,
                    },
                    owner: self.capture_stamp(),
                    file_context: self.ui_file_context_stamp(),
                    file_epoch: self.file_ui.epoch,
                    close_epoch: self.close_policy.epoch,
                    view_epoch: self.view().epoch,
                    tool: self.bond_tool(),
                };
                self.atom_ui.epoch = id;
                self.atom_ui.active = Some(request);
                id
            }
            UiAtomLabel::Cancel { request_id, .. } => {
                if !self.finish_ui_atom_label(request_id) {
                    return Err(Error::UiStaleRequest);
                }
                request_id
            }
        };
        Ok(CommandResult {
            ui_atom_label_operation: Some(id),
            ..self.state()
        })
    }

    /// Revalidate and dispatch under the native frame's same Engine guard.
    pub fn apply_ui_atom_label(
        &mut self,
        id: u64,
        element: u8,
        explicit_hydrogens: u16,
    ) -> Result<CommandResult, Error> {
        if !self.ui_atom_label_request_current(id) {
            return Err(Error::UiStaleRequest);
        }
        let request = self.atom_ui.active.as_ref().ok_or(Error::UiStaleRequest)?;
        let params = AtomSetLabel {
            expected_revision: request.view.input_revision,
            page_id: request.view.page_id,
            fragment_id: request.view.fragment_id,
            atom_id: request.view.atom_id,
            element,
            explicit_hydrogens,
        };
        let command = if self.stored_document()?.has_boxed_atom_labels() {
            Command::AtomSetAuthoredLabel(params)
        } else {
            Command::AtomSetLabel(params)
        };
        let mut result = self.execute(command)?;
        self.finish_ui_atom_label(id);
        result.ui_atom_label = None;
        result.ui_atom_label_operation = Some(id);
        Ok(result)
    }
}
