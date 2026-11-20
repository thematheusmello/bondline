//! Read-only formula for a published drawing/selection scope.
use crate::{CommandResult, Engine, Error, Prepared};
use bondline_chem::{
    Composition, CompositionError, ElementWeight, FormulaIsotopes, MassUnavailable, MolecularMasses,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Bounded synchronous inspection while holding the session lock.
pub const MAX_ANALYSIS_ITEMS: usize = 4096;
/// Only the selected connection table is decoded while the session lock is held.
pub(super) const MAX_ANALYSIS_RECORD_BYTES: usize = 1024 * 1024;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdf_record_id: Option<std::num::NonZeroU32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smiles_record_id: Option<std::num::NonZeroU32>,
}

#[derive(Clone, Copy)]
pub(super) enum Record {
    Sdf(std::num::NonZeroU32),
    Smiles(std::num::NonZeroU32),
}

impl AnalysisProperties {
    pub(super) fn record(&self) -> Result<Option<Record>, Error> {
        match (self.sdf_record_id, self.smiles_record_id) {
            (Some(id), None) => Ok(Some(Record::Sdf(id))),
            (None, Some(id)) => Ok(Some(Record::Smiles(id))),
            (None, None) => Ok(None),
            (Some(_), Some(_)) => Err(Error::InvalidParams(
                <serde_json::Error as serde::de::Error>::custom("choose one record source"),
            )),
        }
    }
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
    SdfRecord,
    SmilesRecord,
}

#[derive(Clone, Debug, Serialize)]
pub struct AnalysisReport {
    pub scope: AnalysisScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_id: Option<std::num::NonZeroU32>,
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
    RecordByteLimit {
        limit: usize,
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
            sdf_record_id: None,
            smiles_record_id: None,
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
        record: Option<Record>,
    ) -> Result<Prepared, Error> {
        Ok(Prepared::immediate(CommandResult {
            analysis: Some(self.analysis_report(revision, epoch, fields, record)?),
            ..self.state()
        }))
    }

    fn analysis_report(
        &self,
        revision: u64,
        epoch: u64,
        fields: Fields,
        record: Option<Record>,
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
        let (scope, record_id) = match record {
            Some(Record::Sdf(id)) => (AnalysisScope::SdfRecord, Some(id)),
            Some(Record::Smiles(id)) => (AnalysisScope::SmilesRecord, Some(id)),
            None if selection.atom_ids.is_empty() => (AnalysisScope::Document, None),
            None => (AnalysisScope::Selection, None),
        };
        let report = |outcome| AnalysisReport {
            scope,
            record_id,
            outcome,
        };
        if let Some(Record::Smiles(id)) = record {
            let file = match self.content() {
                crate::StoredContent::SmilesFile(file) => file,
                crate::StoredContent::Drawing(_) => {
                    return Err(Error::WrongContent {
                        required: "smiles",
                        actual: "drawing",
                    });
                }
                crate::StoredContent::Sdf { .. } => {
                    return Err(Error::WrongContent {
                        required: "smiles",
                        actual: "sdf",
                    });
                }
            };
            let raw = file.read_record(id.into())?;
            if raw.atoms().len() > MAX_ANALYSIS_ITEMS {
                return Ok(report(AnalysisOutcome::LimitExceeded {
                    limit: MAX_ANALYSIS_ITEMS,
                }));
            }
            let Some(molecule) = raw.record_composition(&Default::default())? else {
                return Ok(report(AnalysisOutcome::UnsupportedSource));
            };
            return Ok(report(fields.outcome(
                molecule.composition,
                molecule.atom_count,
                molecule.component_count,
                false,
                std::num::NonZeroUsize::new(molecule.defaulted_hydrogen_atoms),
            )?));
        }
        let molecule;
        let stored = if let Some(Record::Sdf(record_id)) = record {
            let sdf = self.content().sdf()?;
            let id = bondline_io::sdf::RecordId::new(record_id.get())
                .ok_or(bondline_io::sdf::SdfError::InvalidSpan)?;
            let bytes = sdf.mol_bytes(id)?;
            if bytes.len() > MAX_ANALYSIS_RECORD_BYTES {
                return Ok(report(AnalysisOutcome::RecordByteLimit {
                    limit: MAX_ANALYSIS_RECORD_BYTES,
                }));
            }
            let Ok(record) = bondline_io::StoredDocument::from_mol(
                bytes.into(),
                bondline_model::PageSize::A4,
                crate::files::native_mol_transform()?,
            ) else {
                return Ok(report(AnalysisOutcome::UnsupportedSource));
            };
            molecule = record;
            &molecule
        } else if let Ok(document) = self.stored_document() {
            document
        } else {
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
            stored.analysis_hydrogen_counts(&Default::default())?
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
        Ok(report(fields.outcome(
            composition,
            atom_count,
            fragment_count,
            partial_fragments,
            defaulted_hydrogen_atoms,
        )?))
    }
}

impl Fields {
    fn outcome(
        self,
        composition: Composition,
        atom_count: usize,
        fragment_count: usize,
        partial_fragments: bool,
        defaulted_hydrogen_atoms: Option<std::num::NonZeroUsize>,
    ) -> Result<AnalysisOutcome, Error> {
        let (masses, elemental_analysis) = match self {
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
        Ok(AnalysisOutcome::Available {
            atom_count,
            fragment_count,
            partial_fragments,
            defaulted_hydrogen_atoms,
            formula: composition.hill_formula(FormulaIsotopes::Separate)?,
            masses,
            elemental_analysis,
        })
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
                Fields::Formula,
                None
            ),
            Err(Error::StaleSelection)
        ));
        Ok(())
    }
}
