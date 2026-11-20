//! Read-only formula for a published drawing/selection scope.
use crate::{Engine, Error};
use bondline_chem::{Composition, CompositionError, FormulaIsotopes};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Bounded synchronous inspection while holding the session lock.
pub const MAX_ANALYSIS_ITEMS: usize = 4096;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisFormula {
    pub expected_revision: u64,
    pub expected_selection_epoch: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UiAnalysis {
    pub expected_revision: u64,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisScope {
    Document,
    Selection,
}

#[derive(Clone, Debug, Serialize)]
pub struct AnalysisReport {
    pub scope: AnalysisScope,
    #[serde(flatten)]
    pub outcome: AnalysisOutcome,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AnalysisOutcome {
    Available {
        atom_count: usize,
        fragment_count: usize,
        partial_fragments: bool,
        formula: String,
    },
    Empty,
    UnsupportedSource,
    UnresolvedHydrogens {
        atom_id: u32,
    },
    LimitExceeded {
        limit: usize,
    },
}

impl Engine {
    pub fn analysis_request(&self) -> AnalysisFormula {
        AnalysisFormula {
            expected_revision: self.revision,
            expected_selection_epoch: self.selection().epoch,
        }
    }

    pub fn analysis_window_open(&self) -> bool {
        self.analysis_window_open
    }

    pub(super) fn analysis_formula(
        &self,
        params: AnalysisFormula,
    ) -> Result<AnalysisReport, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if params.expected_selection_epoch != self.selection().epoch
            || (!self.selection().atom_ids.is_empty() && !self.selected_source_is_current())
        {
            return Err(Error::StaleSelection);
        }
        let selection = self.selection();
        let scope = if selection.atom_ids.is_empty() {
            AnalysisScope::Document
        } else {
            AnalysisScope::Selection
        };
        let report = |outcome| AnalysisReport { scope, outcome };
        let Ok(stored) = self.stored_document() else {
            return Ok(report(AnalysisOutcome::UnsupportedSource));
        };
        if !stored.has_analysis_coverage() {
            return Ok(report(AnalysisOutcome::UnsupportedSource));
        }
        let mut atoms = Vec::new();
        let mut fragment_count = 0;
        let mut partial_fragments = false;
        let mut scanned_atoms = 0usize;
        let mut scanned_fragments = 0usize;
        for page in stored.model().pages() {
            if !selection.atom_ids.is_empty() && selection.page_id != Some(page.id().get()) {
                continue;
            }
            for fragment in page.fragments() {
                scanned_fragments += 1;
                scanned_atoms = scanned_atoms.saturating_add(fragment.atoms().len());
                if scanned_fragments > MAX_ANALYSIS_ITEMS || scanned_atoms > MAX_ANALYSIS_ITEMS {
                    return Ok(report(AnalysisOutcome::LimitExceeded {
                        limit: MAX_ANALYSIS_ITEMS,
                    }));
                }
                let start = atoms.len();
                atoms.extend(fragment.atoms().iter().filter(|atom| {
                    selection.atom_ids.is_empty()
                        || selection.atom_ids.binary_search(&atom.id.get()).is_ok()
                }));
                let included = atoms.len() - start;
                if included > 0 {
                    fragment_count += 1;
                    partial_fragments |= included != fragment.atoms().len();
                }
            }
        }
        if !selection.atom_ids.is_empty() && atoms.len() != selection.atom_ids.len() {
            return Err(Error::StaleSelection);
        }
        if atoms.is_empty() {
            return Ok(report(AnalysisOutcome::Empty));
        }
        let atom_count = atoms.len();
        let formula = match Composition::from_atoms(atoms, || false)
            .and_then(|composition| composition.hill_formula(FormulaIsotopes::Separate))
        {
            Ok(formula) => formula,
            Err(CompositionError::UnresolvedHydrogens(id)) => {
                return Ok(report(AnalysisOutcome::UnresolvedHydrogens {
                    atom_id: id.get(),
                }));
            }
            Err(error) => return Err(Error::Composition(error)),
        };
        Ok(report(AnalysisOutcome::Available {
            atom_count,
            fragment_count,
            partial_fragments,
            formula,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn equal_graph_under_a_different_source_owner_cannot_reuse_an_atom_selection()
    -> Result<(), Error> {
        let mut engine = Engine::default();
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
        )?;
        engine.dispatch(
            "ui.selection",
            json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[1]}),
        )?;
        let request = engine.analysis_request();
        let original = Arc::clone(&engine.document);
        engine.document = Arc::new((*original).clone());
        assert_eq!(engine.document, original);
        assert!(matches!(
            engine.analysis_formula(request),
            Err(Error::StaleSelection)
        ));
        Ok(())
    }
}
