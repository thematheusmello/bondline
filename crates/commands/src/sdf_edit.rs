//! Deferred immutable candidates for one existing ordered field occurrence.

use crate::{
    Cancellation, CloseState, CommandResult, Engine, Error, MAX_SDF_INSPECT_VALUE_BYTES,
    StoredContent,
};
use bondline_io::sdf::{RecordId, SdfError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SdfFieldSet {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub record_id: u32,
    pub field_index: usize,
    pub value_lines: Vec<String>,
}

impl std::fmt::Debug for SdfFieldSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdfFieldSet")
            .field("expected_revision", &self.expected_revision)
            .field("record_id", &self.record_id)
            .field("field_index", &self.field_index)
            .field("value_line_count", &self.value_lines.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub(super) struct FieldPlan {
    input: Arc<StoredContent>,
    params: SdfFieldSet,
    generation: u64,
}

#[derive(Debug)]
pub(super) struct FieldCandidate {
    input: Arc<StoredContent>,
    candidate: StoredContent,
    revision: u64,
    generation: u64,
    cancel: Cancellation,
}

impl Engine {
    pub(super) fn sdf_field_plan(&self, params: SdfFieldSet) -> Result<FieldPlan, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        let current = self.content().sdf()?;
        let id = RecordId::new(params.record_id)
            .ok_or(Error::InvalidSdfField("record_id must be positive"))?;
        let record = current.record(id).ok_or(SdfError::MissingRecord {
            record_id: params.record_id,
        })?;
        if params.field_index >= record.field_count() {
            return Err(SdfError::MissingField {
                record_id: params.record_id,
                field_index: params.field_index,
            }
            .into());
        }
        Ok(FieldPlan {
            input: Arc::clone(&self.document),
            params,
            generation: self.generation,
        })
    }

    pub(super) fn finish_sdf_field(
        &mut self,
        completed: FieldCandidate,
    ) -> Result<CommandResult, Error> {
        if self.close_state() == CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        completed.cancel.check()?;
        if completed.revision != self.revision
            || completed.generation != self.generation
            || !Arc::ptr_eq(&completed.input, &self.document)
        {
            return Err(Error::StaleRevision);
        }
        let changed = self.commit(completed.candidate)?;
        Ok(CommandResult {
            changed,
            ..self.state()
        })
    }
}

impl FieldPlan {
    pub(super) fn complete(self, cancel: &Cancellation) -> Result<FieldCandidate, Error> {
        cancel.check()?;
        let mut authored_bytes = 0_usize;
        for line in &self.params.value_lines {
            authored_bytes = authored_bytes
                .checked_add(line.len())
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= MAX_SDF_INSPECT_VALUE_BYTES)
                .ok_or(Error::SdfFieldInputLimit)?;
        }
        if !crate::sdf_query::serialized_fits(
            &self.params.value_lines,
            MAX_SDF_INSPECT_VALUE_BYTES,
        )? {
            return Err(Error::SdfFieldInputLimit);
        }
        cancel.check()?;
        let id = RecordId::new(self.params.record_id)
            .ok_or(Error::InvalidSdfField("record_id must be positive"))?;
        let lines = self
            .params
            .value_lines
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let candidate = match self.input.as_ref() {
            StoredContent::Sdf { original, current } => StoredContent::Sdf {
                original: original.clone(),
                current: current.with_field_value(id, self.params.field_index, &lines)?,
            },
            StoredContent::Drawing(_) => {
                return Err(Error::WrongContent {
                    required: "sdf",
                    actual: "drawing",
                });
            }
        };
        cancel.check()?;
        Ok(FieldCandidate {
            input: self.input,
            candidate,
            revision: self.params.expected_revision,
            generation: self.generation,
            cancel: cancel.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use bondline_io::sdf::SdfDocument;
    use serde_json::json;

    #[test]
    fn revision_limit_refuses_changed_candidates_but_keeps_noop_and_debug_private()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
        let document = SdfDocument::parse(bytes.as_slice().into())?;
        let mut engine = Engine::from_content(StoredContent::from_sdf(document));
        engine.revision = u64::MAX;
        engine.dispatch(
            "document.close",
            json!({"expected_revision":u64::MAX,"action":"request"}),
        )?;
        let before = serde_json::to_value(engine.state())?;
        let command = Command::parse(
            "sdf.field.set_value",
            json!({"expected_revision":u64::MAX,"record_id":1,"field_index":5,"value_lines":["PRIVATE_FIELD_TEST_VALUE"]}),
        )?;
        assert!(!format!("{command:?}").contains("PRIVATE_FIELD_TEST_VALUE"));
        let prepared = engine.prepare(command)?;
        assert!(!format!("{prepared:?}").contains("PRIVATE_FIELD_TEST_VALUE"));
        let completed = prepared.complete()?;
        assert!(!format!("{completed:?}").contains("PRIVATE_FIELD_TEST_VALUE"));
        assert_eq!(
            engine
                .finish(completed)
                .err()
                .ok_or("revision overflow")?
                .info()
                .code,
            "revision_exhausted"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content().sdf()?.as_bytes(), bytes);
        let noop=engine.dispatch("sdf.field.set_value",json!({"expected_revision":u64::MAX,"record_id":1,"field_index":5,"value_lines":["two"]}))?;
        assert!(!noop.changed);
        assert_eq!(noop.revision, u64::MAX);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(engine.complete_close());
        Ok(())
    }
}
