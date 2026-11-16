//! Deferred complete-source SMILES projection and synchronized chemical edits.

use crate::{
    BondSetOrder, Cancellation, CloseState, CommandResult, Engine, Error, StoredContent,
    UiFileContextStamp, ViewState,
};
use bondline_io::{
    StoredDocument,
    smiles::{RecordSelection, file::RecordId},
};
use bondline_model::{PageSize, Point2};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum SmilesRecordSelection {
    Record {
        #[schemars(range(min = 1))]
        record_id: u32,
    },
    All {},
}

impl SmilesRecordSelection {
    fn selection(&self) -> Result<RecordSelection, Error> {
        Ok(match self {
            Self::Record { record_id } => RecordSelection::Record(
                RecordId::new(*record_id)
                    .ok_or(Error::InvalidSmilesImport("record_id must be positive"))?,
            ),
            Self::All {} => RecordSelection::All,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportSmilesRecords {
    pub expected_revision: u64,
    pub selection: SmilesRecordSelection,
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub bond_length_pt: f64,
    pub origin_x_pt: f64,
    pub origin_y_pt: f64,
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub page_width_pt: f64,
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub page_height_pt: f64,
}

#[derive(Debug)]
enum Operation {
    Import(ImportSmilesRecords),
    Order(BondSetOrder),
}

#[derive(Debug)]
pub(super) struct RecordsPlan {
    input: Arc<StoredContent>,
    operation: Operation,
    revision: u64,
    generation: u64,
    close_epoch: u64,
    file_context: UiFileContextStamp,
}

#[derive(Debug)]
enum Candidate {
    Import(StoredDocument),
    Edit(StoredDocument),
}

#[derive(Debug)]
pub(super) struct RecordsCandidate {
    input: Arc<StoredContent>,
    candidate: Candidate,
    revision: u64,
    generation: u64,
    close_epoch: u64,
    file_context: UiFileContextStamp,
    cancel: Cancellation,
}

impl Engine {
    fn records_plan(&self, revision: u64, operation: Operation) -> Result<RecordsPlan, Error> {
        if revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if self.close_state() != CloseState::Open {
            return Err(Error::UiBusy);
        }
        Ok(RecordsPlan {
            input: Arc::clone(&self.document),
            operation,
            revision,
            generation: self.generation,
            close_epoch: self.close_policy.epoch,
            file_context: self.ui_file_context_stamp(),
        })
    }

    pub(super) fn smiles_import_plan(
        &self,
        params: ImportSmilesRecords,
    ) -> Result<RecordsPlan, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        let selection = params.selection.selection()?;
        if !params.bond_length_pt.is_finite() || params.bond_length_pt <= 0.0 {
            return Err(Error::InvalidSmilesImport(
                "bond length must be finite and positive",
            ));
        }
        Point2::new(params.origin_x_pt, params.origin_y_pt)?;
        PageSize::new(params.page_width_pt, params.page_height_pt)?;
        let StoredContent::SmilesFile(source) = self.content() else {
            return Err(Error::WrongContent {
                required: "unprojected smiles file",
                actual: if matches!(self.content(), StoredContent::Sdf { .. }) {
                    "sdf"
                } else {
                    "drawing"
                },
            });
        };
        if let RecordSelection::Record(id) = selection {
            source
                .record(id)
                .ok_or(bondline_io::smiles::SmilesFileError::MissingRecord { record_id: id })?;
        }
        self.records_plan(params.expected_revision, Operation::Import(params))
    }

    pub(super) fn smiles_order_plan(&self, params: BondSetOrder) -> Result<RecordsPlan, Error> {
        self.records_plan(params.expected_revision, Operation::Order(params))
    }

    pub(super) fn finish_smiles_records(
        &mut self,
        completed: RecordsCandidate,
    ) -> Result<CommandResult, Error> {
        completed.cancel.check()?;
        if self.close_state() == CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        if completed.revision != self.revision
            || completed.generation != self.generation
            || !Arc::ptr_eq(&completed.input, &self.document)
        {
            return Err(Error::StaleRevision);
        }
        if self.close_state() != CloseState::Open
            || self.close_policy.epoch != completed.close_epoch
            || !self.ui_file_context_stamp_matches(&completed.file_context)
        {
            return Err(Error::UiStaleRequest);
        }
        let changed = match completed.candidate {
            Candidate::Edit(document) => self.commit(document)?,
            Candidate::Import(document) => {
                let revision = self.next_revision()?;
                let generation = self
                    .generation
                    .checked_add(1)
                    .ok_or(Error::RevisionExhausted)?;
                let view_epoch = self
                    .view
                    .epoch
                    .checked_add(1)
                    .ok_or(Error::UiViewEpochExhausted)?;
                let document = Arc::new(StoredContent::Drawing(document));
                self.document = Arc::clone(&document);
                self.saved = document;
                self.identities = crate::bond::Ids::default();
                self.undo.clear();
                self.redo.clear();
                self.generation = generation;
                self.revision = revision;
                self.view = ViewState {
                    epoch: view_epoch,
                    ..ViewState::default()
                };
                self.sdf_view_extent = None;
                self.view_focus = None;
                self.invalidate_close();
                true
            }
        };
        Ok(CommandResult {
            changed,
            ..self.state()
        })
    }
}

impl RecordsPlan {
    pub(super) fn complete(self, cancel: &Cancellation) -> Result<RecordsCandidate, Error> {
        cancel.check()?;
        let candidate = match self.operation {
            Operation::Import(params) => {
                let StoredContent::SmilesFile(source) = self.input.as_ref() else {
                    return Err(Error::InvalidSmilesImport(
                        "captured complete source is unavailable",
                    ));
                };
                Candidate::Import(StoredDocument::from_smiles_records(
                    source.clone(),
                    params.selection.selection()?,
                    params.bond_length_pt,
                    Point2::new(params.origin_x_pt, params.origin_y_pt)?,
                    PageSize::new(params.page_width_pt, params.page_height_pt)?,
                    cancel,
                )?)
            }
            Operation::Order(params) => Candidate::Edit(crate::bond::set_order_with_cancel(
                self.input.drawing()?,
                params,
                cancel,
            )?),
        };
        cancel.check()?;
        Ok(RecordsCandidate {
            input: self.input,
            candidate,
            revision: self.revision,
            generation: self.generation,
            close_epoch: self.close_epoch,
            file_context: self.file_context,
            cancel: cancel.clone(),
        })
    }
}
