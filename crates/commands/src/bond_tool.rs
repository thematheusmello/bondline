//! Transient drawing order; authored commands always receive an explicit/default order.

use crate::{BondDrawOrder, CloseState, CommandResult, Engine, Error, UiFilePhase};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectBondTool {
    pub order: BondDrawOrder,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BondToolState {
    pub order: BondDrawOrder,
    pub epoch: u64,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct BondToolOperation {
    pub order: BondDrawOrder,
    pub epoch: u64,
    pub changed: bool,
}

impl Engine {
    pub fn bond_tool(&self) -> BondToolState {
        self.bond_tool
    }

    pub(super) fn select_bond_tool(
        &mut self,
        params: SelectBondTool,
    ) -> Result<CommandResult, Error> {
        if self.close_state() != CloseState::Open
            || self
                .file_ui_state()
                .request
                .is_some_and(|request| request.phase != UiFilePhase::Running)
        {
            return Err(Error::UiBusy);
        }
        let stored = self.stored_document()?;
        if !stored.is_fully_mapped() && !stored.has_smiles_projection() {
            return Err(Error::UnsupportedDocumentPreview);
        }
        let changed = self.bond_tool.order != params.order;
        if changed {
            let epoch = self
                .bond_tool
                .epoch
                .checked_add(1)
                .ok_or(Error::UiBondToolEpochExhausted)?;
            self.bond_tool = BondToolState {
                order: params.order,
                epoch,
            };
        }
        Ok(CommandResult {
            ui_bond_tool_operation: Some(BondToolOperation {
                order: self.bond_tool.order,
                epoch: self.bond_tool.epoch,
                changed,
            }),
            ..self.state()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_epoch_keeps_noop_and_refuses_mode_change_without_touching_state()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::default();
        engine.bond_tool.epoch = u64::MAX;
        let before = serde_json::to_value(engine.state())?;
        let tool = engine.bond_tool();
        let result = engine.select_bond_tool(SelectBondTool {
            order: BondDrawOrder::Single,
        })?;
        assert!(!result.ui_bond_tool_operation.ok_or("tool result")?.changed);
        assert!(matches!(
            engine.select_bond_tool(SelectBondTool {
                order: BondDrawOrder::Double,
            }),
            Err(Error::UiBondToolEpochExhausted)
        ));
        assert_eq!(engine.bond_tool(), tool);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        Ok(())
    }
}
