//! Explicit selected-notation disclosure, separate from automatic file metadata.

use crate::{Cancellation, Engine, Error, StoredContent, sdf_query::serialized_fits};
use bondline_io::smiles::file::{RecordId, SmilesFileError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{ops::Range, sync::Arc};

pub const MAX_SMILES_RECORD_INSPECT_BYTES: usize = 128 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SmilesRecordInspect {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub record_id: u32,
}

/// Source-local ordinal and whole-file-relative byte ranges, never model IDs or geometry.
#[derive(Clone, Serialize)]
pub struct SmilesRecordInspection {
    pub input_revision: u64,
    pub session_generation: u64,
    pub record_id: u32,
    pub line_number: usize,
    pub full_range: Range<usize>,
    pub notation_range: Range<usize>,
    pub separator_range: Option<Range<usize>>,
    pub application_data_range: Option<Range<usize>>,
    pub line_ending_range: Range<usize>,
    pub atom_count: usize,
    pub bond_count: usize,
    pub notation: String,
}

impl std::fmt::Debug for SmilesRecordInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmilesRecordInspection")
            .field("input_revision", &self.input_revision)
            .field("session_generation", &self.session_generation)
            .field("record_id", &self.record_id)
            .field("line_number", &self.line_number)
            .field("atom_count", &self.atom_count)
            .field("bond_count", &self.bond_count)
            .field("notation_byte_length", &self.notation.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub(super) struct RecordPlan {
    document: Arc<StoredContent>,
    params: SmilesRecordInspect,
    generation: u64,
}

impl Engine {
    pub(super) fn smiles_record_plan(
        &self,
        params: SmilesRecordInspect,
    ) -> Result<RecordPlan, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        let document = self.content().smiles_file()?;
        let id = RecordId::new(params.record_id)
            .ok_or(Error::InvalidSmilesQuery("record_id must be positive"))?;
        if document.record(id).is_none() {
            return Err(SmilesFileError::MissingRecord { record_id: id }.into());
        }
        Ok(RecordPlan {
            document: Arc::clone(&self.document),
            params,
            generation: self.generation,
        })
    }
}

impl RecordPlan {
    pub(super) fn complete(self, cancel: &Cancellation) -> Result<SmilesRecordInspection, Error> {
        cancel.check()?;
        let document = self.document.smiles_file()?;
        let id = RecordId::new(self.params.record_id)
            .ok_or(Error::InvalidSmilesQuery("record_id must be positive"))?;
        let metadata = document
            .record(id)
            .ok_or(SmilesFileError::MissingRecord { record_id: id })?;
        let parsed = document.read_record(id)?;
        cancel.check()?;
        let notation = std::str::from_utf8(parsed.as_bytes())
            .map_err(|_| SmilesFileError::InvalidSpan { record_id: id })?
            .to_owned();
        let result = SmilesRecordInspection {
            input_revision: self.params.expected_revision,
            session_generation: self.generation,
            record_id: self.params.record_id,
            line_number: metadata.line_number(),
            full_range: metadata.full_range(),
            notation_range: metadata.notation_range(),
            separator_range: metadata.separator_range(),
            application_data_range: metadata.application_data_range(),
            line_ending_range: metadata.line_ending_range(),
            atom_count: metadata.atom_count(),
            bond_count: metadata.bond_count(),
            notation,
        };
        if !serialized_fits(&result, MAX_SMILES_RECORD_INSPECT_BYTES)? {
            return Err(Error::SmilesQueryLimit);
        }
        cancel.check()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use serde_json::json;

    #[test]
    fn deferred_work_retains_the_whole_owner_and_completed_reply_releases_it()
    -> Result<(), Box<dyn std::error::Error>> {
        let original = bondline_io::smiles::SmilesFileDocument::parse(
            b"CC private\n[13CH3][OH] \xff\0opaque\n".as_slice().into(),
        )?;
        let mut engine = Engine::from_content(StoredContent::from_smiles_file(original));
        let owner = Arc::downgrade(&engine.document);
        let plan = engine.prepare(Command::parse(
            "smiles.record.inspect",
            json!({"expected_revision":0,"record_id":2}),
        )?)?;
        let current = StoredContent::from_smiles_file(
            bondline_io::smiles::SmilesFileDocument::parse(b"O different".as_slice().into())?,
        );
        engine = Engine::from_content(current);
        assert!(
            owner.upgrade().is_some(),
            "work must retain all source records and opaque bytes"
        );
        let done = plan.complete()?;
        assert!(
            owner.upgrade().is_none(),
            "self-contained reply should not retain an extra whole file"
        );
        let before = serde_json::to_value(engine.state())?;
        let reply = engine.finish(done)?;
        assert_eq!(reply.smiles_record.ok_or("query")?.notation, "[13CH3][OH]");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        Ok(())
    }
}
