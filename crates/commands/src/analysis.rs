//! Read-only formula for a published drawing/selection scope.
use crate::{CommandResult, Engine, Error, Prepared};
use bondline_chem::{
    Composition, CompositionError, ElementWeight, FormulaIsotopes, MassUnavailable, MolecularMasses,
};
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
pub struct AnalysisProperties {
    pub expected_revision: u64,
    pub expected_selection_epoch: u64,
    #[serde(default)]
    pub include_elemental_analysis: bool,
}

pub(super) enum Fields {
    Formula,
    Masses,
    ElementalAnalysis,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum ElementalAnalysis {
    Available { elements: Vec<ElementWeight> },
    Unavailable(MassUnavailable),
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

/// Available mass values are JSON numbers. On missing data each quantity
/// reports its own structured reason while the resolved formula stays available.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(untagged)]
pub enum AnalysisMass {
    Value(f64),
    Unavailable(MassUnavailable),
}

impl From<Result<f64, MassUnavailable>> for AnalysisMass {
    fn from(value: Result<f64, MassUnavailable>) -> Self {
        match value {
            Ok(mass) => Self::Value(mass),
            Err(reason) => Self::Unavailable(reason),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct AnalysisMasses {
    pub molecular_weight: AnalysisMass,
    pub exact_mass_da: AnalysisMass,
}

impl From<MolecularMasses> for AnalysisMasses {
    fn from(value: MolecularMasses) -> Self {
        Self {
            molecular_weight: value.molecular_weight.into(),
            exact_mass_da: value.exact_mass_da.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AnalysisOutcome {
    Available {
        atom_count: usize,
        fragment_count: usize,
        partial_fragments: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        defaulted_hydrogen_atoms: Option<std::num::NonZeroUsize>,
        formula: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        masses: Option<AnalysisMasses>,
        #[serde(skip_serializing_if = "Option::is_none")]
        elemental_analysis: Option<ElementalAnalysis>,
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

    pub fn analysis_properties_request(
        &self,
        include_elemental_analysis: bool,
    ) -> AnalysisProperties {
        AnalysisProperties {
            expected_revision: self.revision,
            expected_selection_epoch: self.selection().epoch,
            include_elemental_analysis,
        }
    }

    pub fn analysis_window_open(&self) -> bool {
        self.analysis_window_open
    }

    pub(super) fn prepare_analysis(
        &self,
        revision: u64,
        epoch: u64,
        fields: Fields,
    ) -> Result<Prepared, Error> {
        Ok(Prepared::immediate(CommandResult {
            analysis: Some(self.analysis_report(revision, epoch, fields)?),
            ..self.state()
        }))
    }

    fn analysis_report(
        &self,
        revision: u64,
        epoch: u64,
        fields: Fields,
    ) -> Result<AnalysisReport, Error> {
        if revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if epoch != self.selection().epoch
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
        let defaulted_hydrogen_atoms = std::num::NonZeroUsize::new(
            atoms
                .iter()
                .filter(|atom| atom.explicit_hydrogens.is_none())
                .count(),
        );
        let hydrogens = if defaulted_hydrogen_atoms.is_some() {
            stored.cdx_skeletal_hydrogen_counts(&Default::default())?
        } else {
            Default::default()
        };
        let composition = match Composition::from_atoms_with_hydrogen_defaults(
            atoms,
            |id| hydrogens.get(&id).copied(),
            || false,
        ) {
            Ok(composition) => composition,
            Err(CompositionError::UnresolvedHydrogens(id)) => {
                return Ok(report(AnalysisOutcome::UnresolvedHydrogens {
                    atom_id: id.get(),
                }));
            }
            Err(error) => return Err(Error::Composition(error)),
        };
        let (masses, elemental_analysis) = match fields {
            Fields::Formula => (None, None),
            Fields::Masses => (Some(composition.molecular_masses().into()), None),
            Fields::ElementalAnalysis => {
                let (masses, rows) = composition.masses_with_elemental_analysis();
                let analysis = match rows {
                    Ok(elements) => ElementalAnalysis::Available { elements },
                    Err(reason) => ElementalAnalysis::Unavailable(reason),
                };
                (Some(masses.into()), Some(analysis))
            }
        };
        Ok(report(AnalysisOutcome::Available {
            atom_count,
            fragment_count,
            partial_fragments,
            defaulted_hydrogen_atoms,
            formula: composition.hill_formula(FormulaIsotopes::Separate)?,
            masses,
            elemental_analysis,
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
            engine.analysis_report(
                request.expected_revision,
                request.expected_selection_epoch,
                Fields::Formula
            ),
            Err(Error::StaleSelection)
        ));
        Ok(())
    }
}
