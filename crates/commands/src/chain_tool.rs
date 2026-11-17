//! Transient chain tools and source-qualified numeric presentation.
use crate::{
    BondToolState, CaptureStamp, ChainCreate, CloseState, Command, CommandResult, EffectiveTool,
    Engine, Error, UiFileContextStamp, UiFilePhase,
};
use bondline_model::{PageId, Point2};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

mod pose;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChainKind {
    Acyclic,
    Snaking,
}

impl<'de> Deserialize<'de> for ChainKind {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "acyclic" => Ok(Self::Acyclic),
            "snaking" => Ok(Self::Snaking),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["acyclic", "snaking"],
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum ChainTarget {
    Free {
        #[schemars(range(min = -100000, max = 100000))]
        x_pt: f64,
        #[schemars(range(min = -100000, max = 100000))]
        y_pt: f64,
    },
    Atom {
        #[schemars(range(min = 1))]
        fragment_id: u32,
        #[schemars(range(min = 1))]
        atom_id: u32,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum UiChainTool {
    Select {
        kind: ChainKind,
    },
    Constraints {
        fixed_lengths: bool,
        fixed_angles: bool,
    },
    OpenAdd {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        page_id: u32,
        target: ChainTarget,
    },
    CancelAdd {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        request_id: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChainPreferences {
    pub bond_length_pt: f64,
    pub angle_degrees: f64,
    pub fixed_lengths: bool,
    pub fixed_angles: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ChainToolOperation {
    pub kind: ChainKind,
    pub fixed_lengths: bool,
    pub fixed_angles: bool,
    pub epoch: u64,
    pub changed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct UiAddChainRequest {
    pub id: u64,
    pub input_revision: u64,
    pub page_id: u32,
    pub target: ChainTarget,
    pub count: u8,
    pub bond_length_pt: f64,
    pub angle_degrees: f64,
    pub current: bool,
    #[serde(skip)]
    cancelled: crate::Cancellation,
}

impl UiAddChainRequest {
    pub fn was_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }
}

/// Weak editing context; native paper/layer/device facts remain the host's responsibility.
#[derive(Clone, Debug)]
pub struct ChainContext {
    source: CaptureStamp,
    tool: BondToolState,
    kind: ChainKind,
    page: PageId,
    view_epoch: u64,
    file_epoch: u64,
    close_epoch: u64,
    file_context: UiFileContextStamp,
    add_epoch: u64,
    add_owner: Option<u64>,
    atom_epoch: u64,
    settings_epoch: u64,
}

impl ChainContext {
    pub fn is_current(&self, engine: &Engine) -> bool {
        engine.file_ui_state().available
            && engine.capture_stamp_matches(&self.source)
            && engine.effective_tool() == (EffectiveTool::Chain { kind: self.kind })
            && engine.bond_tool() == self.tool
            && engine.view().epoch == self.view_epoch
            && engine.file_ui.epoch == self.file_epoch
            && engine.file_ui_state().request.is_none()
            && engine.close_state() == CloseState::Open
            && engine.close_policy.epoch == self.close_epoch
            && engine.ui_file_context_stamp_matches(&self.file_context)
            && engine.chain_ui.epoch == self.add_epoch
            && engine.chain_ui.active.as_ref().map(|r| r.view.id) == self.add_owner
            && engine.atom_ui.epoch() == self.atom_epoch
            && engine.drawing_settings_ui.epoch() == self.settings_epoch
            && engine.ui_atom_label_request().is_none()
            && engine.ui_drawing_settings_request().is_none()
            && engine
                .document()
                .is_ok_and(|d| d.pages().iter().any(|p| p.id() == self.page))
    }
}

#[derive(Debug)]
struct AddRequest {
    view: UiAddChainRequest,
    proof: ChainContext,
}

#[derive(Debug)]
pub(super) struct ChainUiPolicy {
    fixed_lengths: bool,
    fixed_angles: bool,
    epoch: u64,
    active: Option<AddRequest>,
}

impl ChainUiPolicy {
    pub(super) fn epoch(&self) -> u64 {
        self.epoch
    }
}

impl Default for ChainUiPolicy {
    fn default() -> Self {
        Self {
            fixed_lengths: true,
            fixed_angles: true,
            epoch: 0,
            active: None,
        }
    }
}

impl Engine {
    pub fn chain_preferences(&self) -> Result<ChainPreferences, Error> {
        let stored = self.stored_document()?;
        crate::chain::require_source(stored)?;
        let settings = stored.model().drawing_settings();
        Ok(ChainPreferences {
            bond_length_pt: settings.bond_length_pt().unwrap_or(48.0),
            angle_degrees: settings.chain_angle_degrees().unwrap_or(120.0),
            fixed_lengths: self.chain_ui.fixed_lengths,
            fixed_angles: self.chain_ui.fixed_angles,
        })
    }

    pub fn chain_context(&self, page_id: u32) -> Result<ChainContext, Error> {
        self.chain_preferences()?;
        let EffectiveTool::Chain { kind } = self.effective_tool() else {
            return Err(Error::UnsupportedChain(
                "select a chain tool before drawing",
            ));
        };
        if !self.file_ui_state().available {
            return Err(Error::UiUnavailable);
        }
        if self.close_state() != CloseState::Open
            || self.file_ui_state().request.is_some()
            || self.ui_atom_label_request().is_some()
            || self.ui_drawing_settings_request().is_some()
            || self.chain_ui.active.is_some()
        {
            return Err(Error::UiBusy);
        }
        let page = crate::chain::page(self.stored_document()?, page_id)?.id();
        Ok(ChainContext {
            source: self.capture_stamp(),
            tool: self.bond_tool(),
            kind,
            page,
            view_epoch: self.view().epoch,
            file_epoch: self.file_ui.epoch,
            close_epoch: self.close_policy.epoch,
            file_context: self.ui_file_context_stamp(),
            add_epoch: self.chain_ui.epoch,
            add_owner: None,
            atom_epoch: self.atom_ui.epoch(),
            settings_epoch: self.drawing_settings_ui.epoch(),
        })
    }

    pub(super) fn ui_chain_tool(&mut self, params: UiChainTool) -> Result<CommandResult, Error> {
        match params {
            UiChainTool::Select { kind } => self.change_chain_tool(
                kind,
                self.chain_ui.fixed_lengths,
                self.chain_ui.fixed_angles,
            ),
            UiChainTool::Constraints {
                fixed_lengths,
                fixed_angles,
            } => {
                let EffectiveTool::Chain { kind } = self.effective_tool() else {
                    return Err(Error::UnsupportedChain(
                        "chain constraints require a chain tool",
                    ));
                };
                self.change_chain_tool(kind, fixed_lengths, fixed_angles)
            }
            UiChainTool::OpenAdd {
                expected_revision,
                page_id,
                target,
            } => {
                if expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                if !self.file_ui_state().available {
                    return Err(Error::UiUnavailable);
                }
                let mut proof = self.chain_context(page_id)?;
                self.chain_target_position(page_id, &target)?;
                let preferences = self.chain_preferences()?;
                let id = self
                    .chain_ui
                    .epoch
                    .checked_add(1)
                    .ok_or(Error::UiEpochExhausted)?;
                proof.add_epoch = id;
                proof.add_owner = Some(id);
                let view = UiAddChainRequest {
                    id,
                    input_revision: self.revision,
                    page_id,
                    target,
                    count: 6,
                    bond_length_pt: preferences.bond_length_pt,
                    angle_degrees: preferences.angle_degrees,
                    current: true,
                    cancelled: crate::Cancellation::default(),
                };
                self.chain_ui.epoch = id;
                self.chain_ui.active = Some(AddRequest { view, proof });
                Ok(CommandResult {
                    ui_add_chain_operation: Some(id),
                    ..self.state()
                })
            }
            UiChainTool::CancelAdd {
                expected_revision,
                request_id,
            } => {
                if expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                if !self.file_ui_state().available {
                    return Err(Error::UiUnavailable);
                }
                let r = self
                    .chain_ui
                    .active
                    .as_ref()
                    .filter(|r| r.view.id == request_id)
                    .ok_or(Error::UiStaleRequest)?;
                r.view.cancelled.cancel();
                self.finish_ui_add_chain(request_id);
                Ok(CommandResult {
                    ui_add_chain_operation: Some(request_id),
                    ..self.state()
                })
            }
        }
    }

    fn change_chain_tool(
        &mut self,
        kind: ChainKind,
        fixed_lengths: bool,
        fixed_angles: bool,
    ) -> Result<CommandResult, Error> {
        if self.close_state() != CloseState::Open
            || self
                .file_ui_state()
                .request
                .is_some_and(|r| r.phase != UiFilePhase::Running)
        {
            return Err(Error::UiBusy);
        }
        self.chain_preferences()?;
        let mode = EffectiveTool::Chain { kind };
        let changed = self.effective_tool() != mode
            || self.chain_ui.fixed_lengths != fixed_lengths
            || self.chain_ui.fixed_angles != fixed_angles;
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
            self.chain_ui.fixed_lengths = fixed_lengths;
            self.chain_ui.fixed_angles = fixed_angles;
        }
        Ok(CommandResult {
            ui_chain_tool_operation: Some(ChainToolOperation {
                kind,
                fixed_lengths,
                fixed_angles,
                epoch: self.bond_tool.epoch,
                changed,
            }),
            ..self.state()
        })
    }

    pub fn ui_add_chain_request(&self) -> Option<UiAddChainRequest> {
        let r = self.chain_ui.active.as_ref()?;
        let mut view = r.view.clone();
        view.current = r.proof.is_current(self);
        Some(view)
    }

    pub fn ui_add_chain_request_current(&self, id: u64) -> bool {
        self.chain_ui
            .active
            .as_ref()
            .is_some_and(|r| r.view.id == id && r.proof.is_current(self))
    }

    pub fn finish_ui_add_chain(&mut self, id: u64) -> bool {
        if self
            .chain_ui
            .active
            .as_ref()
            .is_some_and(|r| r.view.id == id)
        {
            self.chain_ui.active = None;
            true
        } else {
            false
        }
    }

    pub(super) fn detach_add_chain(&mut self) {
        self.chain_ui.active = None;
    }

    pub fn ui_add_chain_candidates(&self, id: u64, count: u8) -> Result<Vec<ChainCreate>, Error> {
        let r = self
            .chain_ui
            .active
            .as_ref()
            .filter(|r| r.view.id == id && r.proof.is_current(self))
            .ok_or(Error::UiStaleRequest)?;
        self.chain_numeric_candidates(r.view.page_id, &r.view.target, count)
    }

    pub fn apply_ui_add_chain(
        &mut self,
        id: u64,
        count: u8,
        candidate_index: usize,
    ) -> Result<CommandResult, Error> {
        let params = self
            .ui_add_chain_candidates(id, count)?
            .get(candidate_index)
            .cloned()
            .ok_or(Error::InvalidChain("numeric pose index is invalid"))?;
        let mut result = self.execute(Command::ChainCreate(params))?;
        self.finish_ui_add_chain(id);
        result.ui_add_chain = None;
        result.ui_add_chain_operation = Some(id);
        Ok(result)
    }

    pub fn chain_repeat_candidates(
        &self,
        proof: &ChainContext,
        page_id: u32,
        target: &ChainTarget,
        count: u8,
    ) -> Result<Vec<ChainCreate>, Error> {
        if !proof.is_current(self) || proof.page.get() != page_id {
            return Err(Error::UiStaleRequest);
        }
        self.chain_numeric_candidates(page_id, target, count)
    }

    pub(super) fn chain_target_position(
        &self,
        page_id: u32,
        target: &ChainTarget,
    ) -> Result<Point2, Error> {
        let stored = self.stored_document()?;
        crate::chain::require_source(stored)?;
        let page = crate::chain::page(stored, page_id)?;
        match *target {
            ChainTarget::Free { x_pt, y_pt } => Ok(Point2::new(x_pt, y_pt)?),
            ChainTarget::Atom {
                fragment_id,
                atom_id,
            } => Ok(crate::chain::attachment(page, fragment_id, atom_id)?.position),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn checked_counters_preserve_noop_and_exact_cancel_identity_at_exhaustion()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))?;
        engine.bond_tool.epoch = u64::MAX;
        let before = serde_json::to_value(engine.state())?;
        let result =
            engine.dispatch("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))?;
        assert!(
            !result
                .ui_chain_tool_operation
                .ok_or("tool receipt")?
                .changed
        );
        assert!(matches!(
            engine.dispatch(
                "ui.chain_tool",
                json!({"action":"constraints","fixed_lengths":false,"fixed_angles":true})
            ),
            Err(Error::UiBondToolEpochExhausted)
        ));
        assert_eq!(serde_json::to_value(engine.state())?, before);
        engine.chain_ui.epoch = u64::MAX - 1;
        let opened=engine.dispatch("ui.chain_tool",json!({"action":"open_add","expected_revision":0,"page_id":1,"target":{"mode":"free","x_pt":144,"y_pt":144}}))?;
        let request = opened.ui_add_chain.ok_or("Add request")?;
        assert_eq!(request.id, u64::MAX);
        assert!(request.current && !request.was_cancelled());
        let before = serde_json::to_value(engine.state())?;
        assert!(matches!(
            engine.dispatch(
                "ui.chain_tool",
                json!({"action":"cancel_add","expected_revision":0,"request_id":u64::MAX-1})
            ),
            Err(Error::UiStaleRequest)
        ));
        assert!(!request.was_cancelled());
        assert_eq!(serde_json::to_value(engine.state())?, before);
        engine.dispatch(
            "ui.chain_tool",
            json!({"action":"cancel_add","expected_revision":0,"request_id":u64::MAX}),
        )?;
        assert!(request.was_cancelled());
        let before = serde_json::to_value(engine.state())?;
        assert!(matches!(engine.dispatch("ui.chain_tool",json!({"action":"open_add","expected_revision":0,"page_id":1,"target":{"mode":"free","x_pt":144,"y_pt":144}})),Err(Error::UiEpochExhausted)));
        assert_eq!(serde_json::to_value(engine.state())?, before);
        Ok(())
    }
}
