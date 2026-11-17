//! Transient native ring mode and weak gesture authority; preview never authors.
use crate::{
    BondToolState, CaptureStamp, ChairKind, CloseState, CommandResult, EffectiveTool, Engine,
    Error, RingCreate, RingCreateRequest, RingKind, UiFileContextStamp, UiFilePhase,
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectNamedRingTool {
    #[serde(deserialize_with = "crate::ring::present_kind")]
    #[schemars(with = "RingKind")]
    pub kind: RingKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectChairRingTool {
    #[serde(deserialize_with = "crate::ring::present_chair_kind")]
    #[schemars(with = "ChairKind")]
    pub kind: ChairKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("type" = "object"))]
pub enum SelectRingToolRequest {
    Sized(SelectRingTool),
    Named(SelectNamedRingTool),
    Chair(SelectChairRingTool),
}

impl SelectRingToolRequest {
    pub const fn size(&self) -> u8 {
        match self {
            Self::Sized(params) => params.size,
            Self::Named(params) => params.kind.size(),
            Self::Chair(_) => 6,
        }
    }

    pub const fn kind(&self) -> Option<RingKind> {
        match self {
            Self::Sized(_) => None,
            Self::Named(params) => Some(params.kind),
            Self::Chair(_) => None,
        }
    }

    pub const fn chair_kind(&self) -> Option<ChairKind> {
        match self {
            Self::Chair(params) => Some(params.kind),
            _ => None,
        }
    }
}

impl From<SelectRingTool> for SelectRingToolRequest {
    fn from(params: SelectRingTool) -> Self {
        Self::Sized(params)
    }
}

impl From<SelectNamedRingTool> for SelectRingToolRequest {
    fn from(params: SelectNamedRingTool) -> Self {
        Self::Named(params)
    }
}

impl From<SelectChairRingTool> for SelectRingToolRequest {
    fn from(params: SelectChairRingTool) -> Self {
        Self::Chair(params)
    }
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
    mode: EffectiveTool,
    page: PageId,
    tool: BondToolState,
    view_epoch: u64,
    close_epoch: u64,
    file_epoch: u64,
    file_context: UiFileContextStamp,
    atom_epoch: u64,
    settings_epoch: u64,
    add_epoch: u64,
}

impl RingContext {
    pub fn is_current(&self, engine: &Engine) -> bool {
        engine.capture_stamp_matches(&self.source)
            && engine.effective_tool() == self.mode
            && engine.bond_tool() == self.tool
            && engine.view().epoch == self.view_epoch
            && engine.close_state() == CloseState::Open
            && engine.close_policy.epoch == self.close_epoch
            && engine.file_ui.epoch == self.file_epoch
            && engine.file_ui_state().request.is_none()
            && engine.ui_file_context_stamp_matches(&self.file_context)
            && engine.atom_ui.epoch() == self.atom_epoch
            && engine.drawing_settings_ui.epoch() == self.settings_epoch
            && engine.chain_ui.epoch() == self.add_epoch
            && engine.ui_atom_label_request().is_none()
            && engine.ui_drawing_settings_request().is_none()
            && engine.ui_add_chain_request().is_none()
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
        params: SelectRingToolRequest,
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
        let size = params.size();
        if !(3..=8).contains(&size) {
            return Err(Error::InvalidRing("ring size must be 3 through 8"));
        }
        let mode = match params {
            SelectRingToolRequest::Sized(_) => EffectiveTool::Ring { size },
            SelectRingToolRequest::Named(params) => EffectiveTool::NamedRing { kind: params.kind },
            SelectRingToolRequest::Chair(params) => EffectiveTool::ChairRing { kind: params.kind },
        };
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
                size,
                epoch: self.bond_tool.epoch,
                changed,
            }),
            ..self.state()
        })
    }

    pub fn ring_context(&self, page_id: u32) -> Result<RingContext, Error> {
        self.require_ring_source()?;
        let mode = self.effective_tool;
        if !matches!(
            mode,
            EffectiveTool::Ring { .. }
                | EffectiveTool::NamedRing { .. }
                | EffectiveTool::ChairRing { .. }
        ) {
            return Err(Error::UnsupportedRing("select a ring tool before drawing"));
        }
        if self.close_state() != CloseState::Open
            || self.file_ui_state().request.is_some()
            || self.ui_atom_label_request().is_some()
            || self.ui_drawing_settings_request().is_some()
            || self.ui_add_chain_request().is_some()
        {
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
            mode,
            page,
            tool: self.bond_tool(),
            view_epoch: self.view().epoch,
            close_epoch: self.close_policy.epoch,
            file_epoch: self.file_ui.epoch,
            file_context: self.ui_file_context_stamp(),
            atom_epoch: self.atom_ui.epoch(),
            settings_epoch: self.drawing_settings_ui.epoch(),
            add_epoch: self.chain_ui.epoch(),
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

    /// Explicit request preview; named and sized candidates share the real builder.
    pub fn preview_ring_request(
        &self,
        params: &RingCreateRequest,
    ) -> Result<bondline_io::StoredDocument, Error> {
        if self.close_state() == CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        if params.expected_revision() != self.revision {
            return Err(Error::StaleRevision);
        }
        crate::ring::create_request(self.stored_document()?, params.clone(), self.identities)
            .map(|(document, _)| document)
    }

    /// The complete connector-plus-cycle candidate, without state changes or ID reservation.
    pub fn preview_ring_sprout(
        &self,
        params: &crate::RingSprout,
    ) -> Result<bondline_io::StoredDocument, Error> {
        if self.close_state() == CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        crate::ring::create_sprout(self.stored_document()?, params.clone(), self.identities)
            .map(|(document, _)| document)
    }
}
