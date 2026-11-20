//! Immutable ownership of a complete validated SDfile, without a molecular projection.

use super::{SdfError, SdfFile, add_bytes, add_lines, validate_value_line};
use crate::mol::Version;
use std::{collections::HashSet, num::NonZeroU32, ops::Range, sync::Arc};

/// Positive ordinal identity inside an immutable source; not a molecular or field ID.
/// Reopening a different source does not promise cross-document identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordId(NonZeroU32);

impl RecordId {
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

/// Source-relative metadata only; ordered fields and complete MOL bytes stay in source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordInfo {
    id: RecordId,
    range: Range<usize>,
    mol_range: Range<usize>,
    delimiter_range: Range<usize>,
    mol_version: Version,
    field_count: usize,
}

impl RecordInfo {
    pub fn id(&self) -> RecordId {
        self.id
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn mol_range(&self) -> Range<usize> {
        self.mol_range.clone()
    }

    pub fn delimiter_range(&self) -> Range<usize> {
        self.delimiter_range.clone()
    }

    pub fn mol_version(&self) -> Version {
        self.mol_version
    }

    pub fn field_count(&self) -> usize {
        self.field_count
    }
}

/// Validated whole-source ownership. Clones share both bytes and immutable metadata;
/// equality includes every source byte, not only molecular/field projections.
#[derive(Clone, PartialEq, Eq)]
pub struct SdfDocument {
    source: Arc<[u8]>,
    records: Arc<[RecordInfo]>,
}

impl std::fmt::Debug for SdfDocument {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("SdfDocument")
            .field("byte_length", &self.source.len())
            .field("record_count", &self.records.len())
            .finish()
    }
}

impl SdfDocument {
    /// Validate every record using the existing syntax/profile limits before owning it.
    /// Syntax-valid opaque MOL data is retained without requiring model projection.
    pub fn parse(source: Arc<[u8]>) -> Result<Self, SdfError> {
        let records = {
            let file = SdfFile::parse(&source)?;
            file.records()
                .iter()
                .enumerate()
                .map(|(index, record)| {
                    let id = u32::try_from(index)
                        .ok()
                        .and_then(|index| index.checked_add(1))
                        .and_then(RecordId::new)
                        .ok_or(SdfError::RecordLimit)?;
                    Ok(RecordInfo {
                        id,
                        range: record.range(),
                        mol_range: record.mol_range(),
                        delimiter_range: record.delimiter_range(),
                        mol_version: record.mol_version(),
                        field_count: record.fields().len(),
                    })
                })
                .collect::<Result<Vec<_>, SdfError>>()?
        };
        Ok(Self {
            source,
            records: records.into(),
        })
    }

    /// Exact unchanged bytes, including every record, item, delimiter and line ending.
    pub fn as_bytes(&self) -> &[u8] {
        &self.source
    }

    pub fn records(&self) -> &[RecordInfo] {
        &self.records
    }

    pub fn record(&self, id: RecordId) -> Option<&RecordInfo> {
        let index = usize::try_from(id.get().checked_sub(1)?).ok()?;
        self.records.get(index)
    }

    /// The original connection table of one record, excluding its data fields.
    pub fn mol_bytes(&self, id: RecordId) -> Result<&[u8], SdfError> {
        let record = self.record(id).ok_or(SdfError::MissingRecord {
            record_id: id.get(),
        })?;
        self.source
            .get(record.mol_range())
            .ok_or(SdfError::InvalidSpan)
    }

    /// A bounded borrowed read of the retained source. Reparse avoids self-reference;
    /// the returned view's temporary allocations are not owned by this snapshot.
    pub fn read(&self) -> Result<SdfFile<'_>, SdfError> {
        SdfFile::parse(&self.source)
    }

    /// Remove one complete record, retaining every other source byte unchanged.
    /// The result is reparsed so its positive ordinal IDs and spans describe the new
    /// source; IDs are not stable across removal. An SDfile must retain one record.
    pub fn without_record(&self, record_id: RecordId) -> Result<Self, SdfError> {
        let record = self.record(record_id).ok_or(SdfError::MissingRecord {
            record_id: record_id.get(),
        })?;
        if self.records.len() == 1 {
            return Err(SdfError::Empty);
        }
        let range = record.range();
        let prefix = self
            .source
            .get(..range.start)
            .ok_or(SdfError::InvalidSpan)?;
        let suffix = self.source.get(range.end..).ok_or(SdfError::InvalidSpan)?;
        let byte_count = prefix
            .len()
            .checked_add(suffix.len())
            .ok_or(SdfError::InvalidSpan)?;
        let mut output = Vec::with_capacity(byte_count);
        output.extend_from_slice(prefix);
        output.extend_from_slice(suffix);
        Self::parse(output.into())
    }

    /// Replace one existing ordered value occurrence, retaining all other source bytes.
    /// The field index is zero-based; an empty slice authors an empty value.
    /// Identical logical lines share the original snapshot, including its line endings;
    /// changed value lines use LF while the original blank terminator stays untouched.
    pub fn with_field_value(
        &self,
        record_id: RecordId,
        field_index: usize,
        value_lines: &[&str],
    ) -> Result<Self, SdfError> {
        let index = usize::try_from(record_id.get())
            .ok()
            .and_then(|number| number.checked_sub(1))
            .ok_or(SdfError::MissingRecord {
                record_id: record_id.get(),
            })?;
        let metadata = self.records.get(index).ok_or(SdfError::MissingRecord {
            record_id: record_id.get(),
        })?;
        if field_index >= metadata.field_count {
            return Err(SdfError::MissingField {
                record_id: record_id.get(),
                field_index,
            });
        }
        let file = self.read()?;
        let field = file
            .records()
            .get(index)
            .and_then(|record| record.fields().get(field_index))
            .ok_or(SdfError::InvalidSpan)?;
        if field.value_lines() == value_lines {
            return Ok(self.clone());
        }
        let mut line_count = self.source.iter().filter(|byte| **byte == b'\n').count();
        if !self.source.ends_with(b"\n") {
            add_lines(&mut line_count, 1)?;
        }
        line_count = line_count
            .checked_sub(field.value_lines().len())
            .ok_or(SdfError::InvalidSpan)?;
        add_lines(&mut line_count, value_lines.len())?;
        let mut value_bytes = 0_usize;
        for line in value_lines {
            validate_value_line(line)?;
            add_bytes(&mut value_bytes, line.len())?;
            add_bytes(&mut value_bytes, 1)?;
        }
        let range = field.value_range();
        let old_length = range
            .end
            .checked_sub(range.start)
            .ok_or(SdfError::InvalidSpan)?;
        let mut byte_count = self
            .source
            .len()
            .checked_sub(old_length)
            .ok_or(SdfError::InvalidSpan)?;
        add_bytes(&mut byte_count, value_bytes)?;
        let prefix = self
            .source
            .get(..range.start)
            .ok_or(SdfError::InvalidSpan)?;
        let suffix = self.source.get(range.end..).ok_or(SdfError::InvalidSpan)?;
        let mut output = Vec::with_capacity(byte_count);
        output.extend_from_slice(prefix);
        for line in value_lines {
            output.extend_from_slice(line.as_bytes());
            output.push(b'\n');
        }
        output.extend_from_slice(suffix);
        Self::parse(output.into())
    }

    /// Retained inline/source/index bytes, shared allocations counted once.
    /// Excludes Arc/allocator headers, temporary reads and external copies; saturates.
    pub fn retained_bytes<'a>(documents: impl IntoIterator<Item = &'a Self>) -> usize {
        let mut snapshots = HashSet::new();
        let mut sources = HashSet::new();
        let mut indices = HashSet::new();
        let mut bytes = 0_usize;
        for document in documents {
            if !snapshots.insert(std::ptr::from_ref(document)) {
                continue;
            }
            bytes = bytes.saturating_add(size_of::<Self>());
            if sources.insert(document.source.as_ptr()) {
                bytes = bytes.saturating_add(document.source.len());
            }
            if indices.insert(document.records.as_ptr()) {
                bytes = bytes.saturating_add(
                    document
                        .records
                        .len()
                        .saturating_mul(size_of::<RecordInfo>()),
                );
            }
        }
        bytes
    }
}
