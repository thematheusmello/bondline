//! Transient native ring mode and weak gesture authority; preview never authors.
use crate::{
    BondToolState, CaptureStamp, CloseState, CommandResult, EffectiveTool, Engine, Error,
    RingCreate, UiFileContextStamp, UiFilePhase,
};
use bondline_model::PageId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectRingTool {
    #[schemars(range(min = 3, max = 8))]
    pub size: u8,
}

#[cfg(test)]
#[path = "ring_tool_private_tests.rs"]
mod private_tests;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct RingToolOperation {
    pub size: u8,
    pub epoch: u64,
    pub changed: bool,
}

/// Rust-only weak proof; physical canvas and layer facts remain in the native host.
#[derive(Clone, Debug)]
pub struct RingContext {
    source: CaptureStamp,
    size: u8,
    page: PageId,
    tool: BondToolState,
    view_epoch: u64,
    close_epoch: u64,
    file_epoch: u64,
    file_context: UiFileContextStamp,
}

impl RingContext {
    pub fn is_current(&self, engine: &Engine) -> bool {
        engine.capture_stamp_matches(&self.source)
            && engine.effective_tool() == (EffectiveTool::Ring { size: self.size })
            && engine.bond_tool() == self.tool
            && engine.view().epoch == self.view_epoch
            && engine.close_state() == CloseState::Open
            && engine.close_policy.epoch == self.close_epoch
            && engine.file_ui.epoch == self.file_epoch
            && engine.file_ui_state().request.is_none()
            && engine.ui_file_context_stamp_matches(&self.file_context)
            && engine
                .document()
                .is_ok_and(|document| document.pages().iter().any(|page| page.id() == self.page))
    }
}

impl Engine {
    fn require_ring_source(&self) -> Result<(), Error> {
        if !self.selection_available() {
            return Err(Error::UnsupportedRing(
                "source-free or completely mapped native CDXML is required",
            ));
        }
        Ok(())
    }

    pub(super) fn select_ring_tool(
        &mut self,
        params: SelectRingTool,
    ) -> Result<CommandResult, Error> {
        if self.close_state() != CloseState::Open
            || self
                .file_ui_state()
                .request
                .is_some_and(|request| request.phase != UiFilePhase::Running)
        {
            return Err(Error::UiBusy);
        }
        self.require_ring_source()?;
        if !(3..=8).contains(&params.size) {
            return Err(Error::InvalidRing("ring size must be 3 through 8"));
        }
        let mode = EffectiveTool::Ring { size: params.size };
        let changed = self.effective_tool != mode;
        if changed {
            let selection_epoch = self.selection_epoch_for_bond_tool()?;
            let epoch = self
                .bond_tool
                .epoch
                .checked_add(1)
                .ok_or(Error::UiBondToolEpochExhausted)?;
            self.deactivate_selection(selection_epoch);
            self.bond_tool.epoch = epoch;
            self.effective_tool = mode;
        }
        Ok(CommandResult {
            ui_ring_tool_operation: Some(RingToolOperation {
                size: params.size,
                epoch: self.bond_tool.epoch,
                changed,
            }),
            ..self.state()
        })
    }

    pub fn ring_context(&self, page_id: u32) -> Result<RingContext, Error> {
        self.require_ring_source()?;
        let EffectiveTool::Ring { size } = self.effective_tool else {
            return Err(Error::UnsupportedRing("select a ring tool before drawing"));
        };
        if self.close_state() != CloseState::Open || self.file_ui_state().request.is_some() {
            return Err(Error::UiBusy);
        }
        let page = PageId::new(page_id).ok_or(Error::InvalidPageId)?;
        if !self
            .document()?
            .pages()
            .iter()
            .any(|candidate| candidate.id() == page)
        {
            return Err(bondline_model::Error::MissingPage(page_id).into());
        }
        Ok(RingContext {
            source: self.capture_stamp(),
            size,
            page,
            tool: self.bond_tool(),
            view_epoch: self.view().epoch,
            close_epoch: self.close_policy.epoch,
            file_epoch: self.file_ui.epoch,
            file_context: self.ui_file_context_stamp(),
        })
    }

    /// Explicit headless candidate, independent of the current native tool or host.
    pub fn preview_ring_create(
        &self,
        params: &RingCreate,
    ) -> Result<bondline_io::StoredDocument, Error> {
        if self.close_state() == CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        crate::ring::create(self.stored_document()?, params.clone(), self.identities)
            .map(|(document, _)| document)
    }
}
