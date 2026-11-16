//! Transient selection and one atomic source-qualified translation.
use crate::{
    BondToolState, CaptureStamp, CloseState, CommandResult, Engine, Error, UiFileContextStamp,
    UiFilePhase,
};
use bondline_model::{Fragment, MAX_GRAPH_ITEMS, Page, PageId, Point2};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum UiSelection {
    Activate {
        expected_revision: u64,
    },
    Set {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        page_id: u32,
        #[schemars(schema_with = "atom_ids_schema")]
        atom_ids: Vec<u32>,
    },
    All {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        page_id: u32,
    },
    Clear {
        expected_revision: u64,
    },
    Inspect {
        expected_revision: u64,
    },
}

fn atom_ids_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({"type":"array","maxItems":MAX_GRAPH_ITEMS,
        "items":{"type":"integer","minimum":1,"maximum":u32::MAX}})
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectionTranslate {
    pub expected_revision: u64,
    pub expected_selection_epoch: u64,
    pub delta_x_pt: f64,
    pub delta_y_pt: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SelectionState {
    pub active: bool,
    pub epoch: u64,
    pub page_id: Option<u32>,
    pub atom_ids: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct SelectionOperation {
    pub epoch: u64,
    pub changed: bool,
}

#[derive(Debug, Default)]
pub(super) struct SelectionPolicy {
    pub(super) state: SelectionState,
    owner: Option<CaptureStamp>,
}

/// Rust-only weak authority for one native gesture; canvas/DPI remain UI qualifiers.
#[derive(Clone, Debug)]
pub struct SelectionContext {
    source: CaptureStamp,
    selection_epoch: u64,
    tool: BondToolState,
    view_epoch: u64,
    close_epoch: u64,
    file_epoch: u64,
    file_context: UiFileContextStamp,
}

impl SelectionContext {
    pub fn is_current(&self, engine: &Engine) -> bool {
        engine.capture_stamp_matches(&self.source)
            && engine.selection().active
            && engine.selection().epoch == self.selection_epoch
            && engine.bond_tool() == self.tool
            && engine.view().epoch == self.view_epoch
            && engine.close_state() == CloseState::Open
            && engine.close_policy.epoch == self.close_epoch
            && engine.file_ui.epoch == self.file_epoch
            && engine.file_ui_state().request.is_none()
            && engine.ui_file_context_stamp_matches(&self.file_context)
    }
}

impl Engine {
    pub fn selection_context(&self) -> Result<SelectionContext, Error> {
        self.require_selection_source()?;
        if !self.selection().active {
            return Err(Error::StaleSelection);
        }
        if self.close_state() != CloseState::Open || self.file_ui_state().request.is_some() {
            return Err(Error::UiBusy);
        }
        Ok(SelectionContext {
            source: self.capture_stamp(),
            selection_epoch: self.selection().epoch,
            tool: self.bond_tool(),
            view_epoch: self.view().epoch,
            close_epoch: self.close_policy.epoch,
            file_epoch: self.file_ui.epoch,
            file_context: self.ui_file_context_stamp(),
        })
    }

    pub fn selection(&self) -> &SelectionState {
        &self.selection.state
    }

    pub fn selection_available(&self) -> bool {
        self.stored_document().is_ok_and(|stored| {
            stored.is_fully_mapped()
                && stored
                    .source_info()
                    .is_none_or(|source| source.format == "cdxml")
        })
    }

    fn require_selection_source(&self) -> Result<(), Error> {
        if !self.selection_available() {
            return Err(Error::UnsupportedSelection(
                "source-free or completely mapped native CDXML is required",
            ));
        }
        Ok(())
    }

    fn selection_page(&self, page_id: u32) -> Result<&Page, Error> {
        self.require_selection_source()?;
        let id = PageId::new(page_id).ok_or(Error::InvalidPageId)?;
        self.document()?
            .pages()
            .iter()
            .find(|page| page.id() == id)
            .ok_or_else(|| bondline_model::Error::MissingPage(page_id).into())
    }

    fn validated_selection_ids(&self, page_id: u32, mut ids: Vec<u32>) -> Result<Vec<u32>, Error> {
        let page = self.selection_page(page_id)?;
        if ids.len() > MAX_GRAPH_ITEMS || ids.contains(&0) {
            return Err(Error::InvalidSelection(
                "positive atom IDs within the selection bound are required",
            ));
        }
        ids.sort_unstable();
        if ids.windows(2).any(|pair| pair.first() == pair.get(1)) {
            return Err(Error::InvalidSelection(
                "duplicate atom IDs are not allowed",
            ));
        }
        if !ids.is_empty() {
            let available: BTreeSet<_> = page
                .fragments()
                .iter()
                .flat_map(|fragment| fragment.atoms().iter().map(|atom| atom.id.get()))
                .collect();
            if let Some(id) = ids.iter().find(|id| !available.contains(id)) {
                return Err(Error::MissingSelectionAtom(*id));
            }
        }
        Ok(ids)
    }

    fn replace_selection(
        &mut self,
        active: bool,
        page_id: Option<u32>,
        atom_ids: Vec<u32>,
    ) -> Result<bool, Error> {
        let old = &self.selection.state;
        let changed = old.active != active
            || old.page_id != page_id
            || old.atom_ids != atom_ids
            || (!atom_ids.is_empty()
                && self
                    .selection
                    .owner
                    .as_ref()
                    .is_none_or(|stamp| !self.capture_stamp_matches(stamp)));
        if !changed {
            return Ok(false);
        }
        let epoch = old
            .epoch
            .checked_add(1)
            .ok_or(Error::UiSelectionEpochExhausted)?;
        let tool_epoch = if old.active != active {
            self.bond_tool
                .epoch
                .checked_add(1)
                .ok_or(Error::UiBondToolEpochExhausted)?
        } else {
            self.bond_tool.epoch
        };
        self.selection.owner = (!atom_ids.is_empty()).then(|| self.capture_stamp());
        self.selection.state = SelectionState {
            active,
            epoch,
            page_id,
            atom_ids,
        };
        self.bond_tool.epoch = tool_epoch;
        Ok(true)
    }

    pub(super) fn ui_selection(&mut self, params: UiSelection) -> Result<CommandResult, Error> {
        let revision = match &params {
            UiSelection::Activate { expected_revision }
            | UiSelection::Set {
                expected_revision, ..
            }
            | UiSelection::All {
                expected_revision, ..
            }
            | UiSelection::Clear { expected_revision }
            | UiSelection::Inspect { expected_revision } => *expected_revision,
        };
        if revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if !matches!(params, UiSelection::Inspect { .. })
            && (self.close_state() != CloseState::Open
                || self
                    .file_ui_state()
                    .request
                    .is_some_and(|request| request.phase != UiFilePhase::Running))
        {
            return Err(Error::UiBusy);
        }
        let changed = match params {
            UiSelection::Inspect { .. } => false,
            UiSelection::Clear { .. } => {
                self.replace_selection(self.selection.state.active, None, vec![])?
            }
            UiSelection::Activate { .. } => {
                self.require_selection_source()?;
                self.replace_selection(
                    true,
                    self.selection.state.page_id,
                    self.selection.state.atom_ids.clone(),
                )?
            }
            UiSelection::Set {
                page_id, atom_ids, ..
            } => {
                let ids = self.validated_selection_ids(page_id, atom_ids)?;
                self.replace_selection(true, (!ids.is_empty()).then_some(page_id), ids)?
            }
            UiSelection::All { page_id, .. } => {
                let page = self.selection_page(page_id)?;
                let mut ids = Vec::new();
                for atom in page
                    .fragments()
                    .iter()
                    .flat_map(|fragment| fragment.atoms())
                {
                    if ids.len() == MAX_GRAPH_ITEMS {
                        return Err(Error::InvalidSelection("too many selected atoms"));
                    }
                    ids.push(atom.id.get());
                }
                ids.sort_unstable();
                self.replace_selection(true, (!ids.is_empty()).then_some(page_id), ids)?
            }
        };
        Ok(CommandResult {
            ui_selection: Some(self.selection.state.clone()),
            ui_selection_operation: Some(SelectionOperation {
                epoch: self.selection.state.epoch,
                changed,
            }),
            ..self.state()
        })
    }

    pub(super) fn selection_epoch_after_edit(&self) -> Result<Option<u64>, Error> {
        if self.selection.state.atom_ids.is_empty() {
            return Ok(None);
        }
        self.selection
            .state
            .epoch
            .checked_add(1)
            .map(Some)
            .ok_or(Error::UiSelectionEpochExhausted)
    }

    pub(super) fn clear_selection_after_edit(&mut self, epoch: Option<u64>) {
        if let Some(epoch) = epoch {
            self.selection.state.epoch = epoch;
            self.selection.state.page_id = None;
            self.selection.state.atom_ids.clear();
            self.selection.owner = None;
        }
    }

    pub(super) fn selection_epoch_for_bond_tool(&self) -> Result<Option<u64>, Error> {
        if !self.selection.state.active {
            return Ok(None);
        }
        self.selection
            .state
            .epoch
            .checked_add(1)
            .map(Some)
            .ok_or(Error::UiSelectionEpochExhausted)
    }

    pub(super) fn deactivate_selection(&mut self, epoch: Option<u64>) {
        if let Some(epoch) = epoch {
            self.selection.state = SelectionState {
                epoch,
                ..SelectionState::default()
            };
            self.selection.owner = None;
        }
    }

    /// Build the same immutable candidate as authoring, without publishing or committing it.
    pub fn preview_selection_translation(
        &self,
        params: &SelectionTranslate,
    ) -> Result<bondline_io::StoredDocument, Error> {
        if self.close_state() == CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if !params.delta_x_pt.is_finite() || !params.delta_y_pt.is_finite() {
            return Err(Error::InvalidSelection("translation deltas must be finite"));
        }
        self.require_selection_source()?;
        let selected = &self.selection.state;
        if !selected.active
            || selected.atom_ids.is_empty()
            || selected.epoch != params.expected_selection_epoch
            || self
                .selection
                .owner
                .as_ref()
                .is_none_or(|owner| !self.capture_stamp_matches(owner))
        {
            return Err(Error::StaleSelection);
        }
        let page_id = selected.page_id.ok_or(Error::StaleSelection)?;
        let page = self.selection_page(page_id)?;
        let mut replacements = Vec::new();
        for fragment in page.fragments() {
            if !fragment
                .atoms()
                .iter()
                .any(|atom| selected.atom_ids.binary_search(&atom.id.get()).is_ok())
            {
                continue;
            }
            let mut atoms = fragment.atoms().to_vec();
            let mut changed = false;
            for atom in &mut atoms {
                if selected.atom_ids.binary_search(&atom.id.get()).is_ok() {
                    let old = atom.position.ok_or(Error::UnsupportedSelection(
                        "a selected atom has no authored position",
                    ))?;
                    let new = Point2::new(
                        old.x_pt() + params.delta_x_pt,
                        old.y_pt() + params.delta_y_pt,
                    )?;
                    changed |= new != old;
                    atom.position = Some(new);
                }
            }
            if changed {
                replacements.push(
                    Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())
                        .map_err(bondline_model::Error::from)?,
                );
            }
        }
        let mut candidate = self.stored_document()?.clone();
        if !replacements.is_empty() {
            candidate.model_mut().replace_fragments(
                PageId::new(page_id).ok_or(Error::InvalidPageId)?,
                replacements,
            )?;
        }
        Ok(candidate)
    }

    pub(super) fn translate_selection(
        &mut self,
        params: SelectionTranslate,
    ) -> Result<CommandResult, Error> {
        let candidate = self.preview_selection_translation(&params)?;
        let ids = self.selection.state.atom_ids.clone();
        let page_id = self.selection.state.page_id;
        let changed = self.commit(candidate)?;
        if changed {
            self.selection.state.page_id = page_id;
            self.selection.state.atom_ids = ids;
            self.selection.owner = Some(self.capture_stamp());
        }
        Ok(CommandResult {
            changed,
            ui_selection_operation: Some(SelectionOperation {
                epoch: self.selection.state.epoch,
                changed,
            }),
            ..self.state()
        })
    }
}

#[cfg(test)]
use crate::Command;
#[cfg(test)]
#[path = "selection_epoch_tests.rs"]
mod epoch_tests;
