//! Explicit bounded field disclosure, separate from automatic collection metadata.

use crate::{Cancellation, Engine, Error};
use bondline_io::sdf::{DataField, RecordId, SdfDocument, SdfError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

pub const MAX_SDF_RECORD_PAGE_BYTES: usize = 1024 * 1024;
pub const MAX_SDF_INSPECT_VALUE_BYTES: usize = 64 * 1024;

fn default_field_limit() -> u8 {
    64
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SdfRecordInspect {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub record_id: u32,
    #[serde(default)]
    pub field_offset: usize,
    #[serde(default = "default_field_limit")]
    #[schemars(range(min = 1, max = 64))]
    pub field_limit: u8,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SdfValueOmission {
    ValueByteLimit,
    ValueJsonLimit,
}

#[derive(Clone, Serialize)]
pub struct SdfFieldInspection {
    pub index: usize,
    pub header: String,
    pub name: Option<String>,
    pub dt_number: Option<String>,
    /// Retained source value bytes including line endings, excluding the blank terminator.
    pub value_byte_length: usize,
    pub value_line_count: usize,
    pub value_lines: Option<Vec<String>>,
    pub omitted_reason: Option<SdfValueOmission>,
}

impl std::fmt::Debug for SdfFieldInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdfFieldInspection")
            .field("index", &self.index)
            .field("value_byte_length", &self.value_byte_length)
            .field("value_line_count", &self.value_line_count)
            .field("omitted_reason", &self.omitted_reason)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Serialize)]
pub struct SdfRecordInspection {
    pub input_revision: u64,
    pub session_generation: u64,
    pub record_id: u32,
    pub mol_version: &'static str,
    pub record_byte_length: usize,
    pub mol_byte_length: usize,
    pub field_count: usize,
    pub field_offset: usize,
    pub field_limit: u8,
    pub fields: Vec<SdfFieldInspection>,
    pub next_field_offset: Option<usize>,
    pub page_limited: bool,
}

impl std::fmt::Debug for SdfRecordInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdfRecordInspection")
            .field("input_revision", &self.input_revision)
            .field("session_generation", &self.session_generation)
            .field("record_id", &self.record_id)
            .field("field_count", &self.field_count)
            .field("field_offset", &self.field_offset)
            .field("returned_fields", &self.fields.len())
            .field("next_field_offset", &self.next_field_offset)
            .field("page_limited", &self.page_limited)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub(super) struct RecordPlan {
    document: SdfDocument,
    params: SdfRecordInspect,
    generation: u64,
}

impl Engine {
    pub(super) fn sdf_record_plan(&self, params: SdfRecordInspect) -> Result<RecordPlan, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        let document = self.content().sdf()?;
        let id = RecordId::new(params.record_id)
            .ok_or(Error::InvalidSdfQuery("record_id must be positive"))?;
        if !(1..=64).contains(&params.field_limit) {
            return Err(Error::InvalidSdfQuery("field_limit must be in 1..64"));
        }
        let record = document.record(id).ok_or(SdfError::MissingRecord {
            record_id: params.record_id,
        })?;
        if params.field_offset > record.field_count() {
            return Err(Error::InvalidSdfQuery(
                "field_offset exceeds the record's field count",
            ));
        }
        Ok(RecordPlan {
            document: document.clone(),
            params,
            generation: self.generation,
        })
    }
}

impl RecordPlan {
    pub(super) fn complete(self, cancel: &Cancellation) -> Result<SdfRecordInspection, Error> {
        cancel.check()?;
        let source = self.document.read()?;
        cancel.check()?;
        let record_id = RecordId::new(self.params.record_id)
            .ok_or(Error::InvalidSdfQuery("record_id must be positive"))?;
        let metadata = self
            .document
            .record(record_id)
            .ok_or(SdfError::MissingRecord {
                record_id: self.params.record_id,
            })?;
        let index = usize::try_from(self.params.record_id)
            .ok()
            .and_then(|id| id.checked_sub(1))
            .ok_or(SdfError::InvalidSpan)?;
        let record = source.records().get(index).ok_or(SdfError::InvalidSpan)?;
        let mut page = SdfRecordInspection {
            input_revision: self.params.expected_revision,
            session_generation: self.generation,
            record_id: self.params.record_id,
            mol_version: match record.mol_version() {
                bondline_io::mol::Version::V2000 => "v2000",
                bondline_io::mol::Version::V3000 => "v3000",
            },
            record_byte_length: metadata.range().len(),
            mol_byte_length: metadata.mol_range().len(),
            field_count: record.fields().len(),
            field_offset: self.params.field_offset,
            field_limit: self.params.field_limit,
            fields: Vec::new(),
            next_field_offset: None,
            page_limited: false,
        };
        for (index, field) in record
            .fields()
            .iter()
            .enumerate()
            .skip(self.params.field_offset)
            .take(usize::from(self.params.field_limit))
        {
            cancel.check()?;
            page.fields.push(inspect_field(index, field)?);
            let next = index.checked_add(1).ok_or(SdfError::FieldLimit)?;
            page.next_field_offset = (next < page.field_count).then_some(next);
            if serialized_fits(&page, MAX_SDF_RECORD_PAGE_BYTES)? {
                continue;
            }
            page.fields.pop();
            if page.fields.is_empty() {
                return Err(Error::SdfQueryPageLimit);
            }
            // true is shorter JSON than false, and the preceding next offset is this index.
            page.next_field_offset = Some(index);
            page.page_limited = true;
            break;
        }
        cancel.check()?;
        Ok(page)
    }
}

fn inspect_field(index: usize, field: &DataField<'_>) -> Result<SdfFieldInspection, Error> {
    let omitted_reason = if field.raw_value().len() > MAX_SDF_INSPECT_VALUE_BYTES {
        Some(SdfValueOmission::ValueByteLimit)
    } else if !serialized_fits(field.value_lines(), MAX_SDF_INSPECT_VALUE_BYTES)? {
        Some(SdfValueOmission::ValueJsonLimit)
    } else {
        None
    };
    Ok(SdfFieldInspection {
        index,
        header: field.header().to_owned(),
        name: field.name().map(str::to_owned),
        dt_number: field.dt_number().map(str::to_owned),
        value_byte_length: field.raw_value().len(),
        value_line_count: field.value_lines().len(),
        value_lines: omitted_reason.is_none().then(|| {
            field
                .value_lines()
                .iter()
                .map(|line| (*line).to_owned())
                .collect()
        }),
        omitted_reason,
    })
}

/// Count real compact JSON without allocating an over-budget output buffer.
fn serialized_fits(value: &(impl Serialize + ?Sized), limit: usize) -> Result<bool, Error> {
    struct Budget {
        bytes: usize,
        limit: usize,
        exceeded: bool,
    }
    impl Write for Budget {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let next = self.bytes.checked_add(bytes.len());
            match next {
                Some(next) if next <= self.limit => {
                    self.bytes = next;
                    Ok(bytes.len())
                }
                _ => {
                    self.exceeded = true;
                    Err(io::Error::other("SDF query JSON limit"))
                }
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut budget = Budget {
        bytes: 0,
        limit,
        exceeded: false,
    };
    match serde_json::to_writer(&mut budget, value) {
        Ok(()) => Ok(true),
        Err(_) if budget.exceeded => Ok(false),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, StoredContent};
    use serde_json::json;

    #[test]
    fn explicit_query_preserves_dirty_source_saved_history_and_close_decision()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
        let original = SdfDocument::parse(bytes.as_slice().into())?;
        let current = original.with_field_value(
            RecordId::new(1).ok_or("record")?,
            5,
            &["revised", "multiline"],
        )?;
        let baseline = StoredContent::from_sdf(original.clone());
        let candidate = StoredContent::Sdf { original, current };
        let mut engine = Engine::from_content(baseline.clone());
        engine.commit(candidate.clone())?;
        engine.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":"request"}),
        )?;
        let before = serde_json::to_value(engine.state())?;
        let prepared = engine.prepare(Command::parse(
            "sdf.record.inspect",
            json!({"expected_revision":1,"record_id":1,"field_offset":5,"field_limit":1}),
        )?)?;
        assert!(prepared.is_deferred());
        assert_eq!(serde_json::to_value(engine.state())?, before);
        let result = serde_json::to_value(engine.finish(prepared.complete()?)?)?;
        assert_eq!(
            result["sdf_record"]["fields"][0]["value_lines"],
            json!(["revised", "multiline"])
        );
        assert_eq!(result["sdf_record"]["input_revision"], 1);
        assert_eq!(result["sdf_record"]["session_generation"], 0);
        assert_eq!(result["close"], "needs_decision");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &candidate);
        assert_eq!(engine.saved.as_ref(), &baseline);
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.content(), &baseline);
        assert!(!engine.state().file.dirty);
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.content(), &candidate);
        assert!(engine.state().file.dirty);
        assert!(engine.state().source.ok_or("source")?.modified);
        Ok(())
    }
}
