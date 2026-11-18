//! Fresh source-free sessions preserve the Engine's host and outstanding work ownership.

use crate::{CloseState, CommandResult, Engine, Error, FileFormat, StoredContent, ViewState};
use bondline_io::StoredDocument;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NewDocument {
    pub expected_revision: u64,
    #[serde(default)]
    pub discard_unsaved: bool,
}

impl Engine {
    /// Monotonic New/Open namespace within this Engine, unchanged by ordinary edits.
    /// This is a UI cache key, not globally unique identity or command authority.
    pub fn session_generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn new_document(&mut self, params: NewDocument) -> Result<CommandResult, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if self.close_state() != CloseState::Open {
            return Err(Error::UiBusy);
        }
        if self.document != self.saved && !params.discard_unsaved {
            return Err(Error::UnsavedChanges);
        }
        let revision = self.next_revision()?;
        let selection_epoch = self.selection_epoch_after_edit()?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(Error::RevisionExhausted)?;
        let view_epoch = self
            .view
            .epoch
            .checked_add(1)
            .ok_or(Error::UiViewEpochExhausted)?;
        let document = Arc::new(StoredContent::from(StoredDocument::default()));
        self.document = Arc::clone(&document);
        self.saved = document;
        self.file_path = None;
        self.file_format = FileFormat::Cdxml;
        self.identities = crate::bond::Ids::default();
        self.undo.clear();
        self.redo.clear();
        self.generation = generation;
        self.revision = revision;
        self.clear_selection_after_edit(selection_epoch);
        self.view = ViewState {
            epoch: view_epoch,
            ..ViewState::default()
        };
        self.sdf_view_extent = None;
        self.view_focus = None;
        self.invalidate_close();
        Ok(CommandResult {
            changed: true,
            ..self.state()
        })
    }
}
