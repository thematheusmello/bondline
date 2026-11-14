//! Immutable complete SMILES file ownership, separate from single-notation syntax.

use super::{Limit, MAX_SMILES_BYTES, SmilesDocument, SmilesError};
use bondline_render::Cancellation;
use std::{collections::HashSet, num::NonZeroU32, ops::Range, sync::Arc};

pub const MAX_FILE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_PHYSICAL_LINES: usize = 1_000_000;
pub const MAX_RECORDS: usize = 65_536;

/// Positive ordinal within one immutable file, not a chemical or cross-source identity.
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
impl std::fmt::Display for RecordId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SmilesFileError {
    #[error("SMILES file exceeds the 64 MiB input budget at byte {offset}")]
    InputLimit { offset: usize },
    #[error("SMILES file physical-line limit exceeded at line {line_number}, byte {offset}")]
    LineLimit { line_number: usize, offset: usize },
    #[error("SMILES file meaningful-record limit exceeded at line {line_number}, byte {offset}")]
    RecordLimit { line_number: usize, offset: usize },
    #[error(
        "SMILES file record {record_id}, physical line {line_number}, absolute byte {offset}: {error}"
    )]
    Notation {
        line_number: usize,
        record_id: RecordId,
        offset: usize,
        #[source]
        error: SmilesError,
    },
    #[error(
        "SMILES file parsing cancelled at physical line {line_number}, byte {offset}, record {record_id:?}"
    )]
    Cancelled {
        line_number: usize,
        record_id: Option<RecordId>,
        offset: usize,
    },
    #[error("SMILES file has no record {record_id}")]
    MissingRecord { record_id: RecordId },
    #[error("invalid retained SMILES file span for record {record_id}")]
    InvalidSpan { record_id: RecordId },
}
impl SmilesFileError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InputLimit { .. } | Self::LineLimit { .. } | Self::RecordLimit { .. } => {
                "resource_limit"
            }
            Self::Notation { error, .. } => error.code(),
            Self::Cancelled { .. } => "cancelled",
            Self::MissingRecord { .. } => "not_found",
            Self::InvalidSpan { .. } => "invalid_smiles_file",
        }
    }
    pub fn line_number(&self) -> Option<usize> {
        match self {
            Self::LineLimit { line_number, .. }
            | Self::RecordLimit { line_number, .. }
            | Self::Notation { line_number, .. }
            | Self::Cancelled { line_number, .. } => Some(*line_number),
            _ => None,
        }
    }
    pub fn record_id(&self) -> Option<RecordId> {
        match self {
            Self::Notation { record_id, .. }
            | Self::MissingRecord { record_id }
            | Self::InvalidSpan { record_id } => Some(*record_id),
            Self::Cancelled { record_id, .. } => *record_id,
            Self::RecordLimit { .. } => u32::try_from(MAX_RECORDS)
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(RecordId::new),
            _ => None,
        }
    }
    /// Absolute file byte offset; the nested notation error retains its local offset.
    pub fn offset(&self) -> Option<usize> {
        match self {
            Self::InputLimit { offset }
            | Self::LineLimit { offset, .. }
            | Self::RecordLimit { offset, .. }
            | Self::Notation { offset, .. }
            | Self::Cancelled { offset, .. } => Some(*offset),
            _ => None,
        }
    }
}

/// Whole-file-relative spans and literal counts only; no strings or persistent parsed IR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordInfo {
    id: RecordId,
    line_number: usize,
    full_range: Range<usize>,
    notation_range: Range<usize>,
    separator_range: Option<Range<usize>>,
    application_data_range: Option<Range<usize>>,
    line_ending_range: Range<usize>,
    atom_count: usize,
    bond_count: usize,
}
impl RecordInfo {
    pub fn id(&self) -> RecordId {
        self.id
    }
    pub fn line_number(&self) -> usize {
        self.line_number
    }
    pub fn full_range(&self) -> Range<usize> {
        self.full_range.clone()
    }
    pub fn notation_range(&self) -> Range<usize> {
        self.notation_range.clone()
    }
    pub fn separator_range(&self) -> Option<Range<usize>> {
        self.separator_range.clone()
    }
    pub fn application_data_range(&self) -> Option<Range<usize>> {
        self.application_data_range.clone()
    }
    pub fn line_ending_range(&self) -> Range<usize> {
        self.line_ending_range.clone()
    }
    pub fn atom_count(&self) -> usize {
        self.atom_count
    }
    pub fn bond_count(&self) -> usize {
        self.bond_count
    }
}

/// Complete immutable file; clones share source and compact metadata, equality includes all bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct SmilesFileDocument {
    source: Arc<[u8]>,
    records: Arc<[RecordInfo]>,
}
impl std::fmt::Debug for SmilesFileDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmilesFileDocument")
            .field("byte_length", &self.source.len())
            .field("record_count", &self.records.len())
            .finish()
    }
}
impl SmilesFileDocument {
    pub fn parse(source: Arc<[u8]>) -> Result<Self, SmilesFileError> {
        Self::parse_with_cancel(source, &Cancellation::default())
    }
    /// Validate every meaningful record before success. Cancellation is cooperative
    /// between physical lines and around each bounded original notation parser call.
    pub fn parse_with_cancel(
        source: Arc<[u8]>,
        cancel: &Cancellation,
    ) -> Result<Self, SmilesFileError> {
        let records = index_records(&source, &mut || cancel.is_cancelled())?;
        Ok(Self {
            source,
            records: records.into(),
        })
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.source
    }
    pub fn records(&self) -> &[RecordInfo] {
        &self.records
    }
    pub fn record(&self, id: RecordId) -> Option<&RecordInfo> {
        self.records
            .get(usize::try_from(id.get().checked_sub(1)?).ok()?)
    }
    /// Reparse only this notation into a temporary independent owner. Returned IR offsets
    /// are local to notation; RecordInfo spans remain relative to the complete file.
    pub fn read_record(&self, id: RecordId) -> Result<SmilesDocument, SmilesFileError> {
        let record = self
            .record(id)
            .ok_or(SmilesFileError::MissingRecord { record_id: id })?;
        let bytes = self
            .source
            .get(record.notation_range())
            .ok_or(SmilesFileError::InvalidSpan { record_id: id })?;
        SmilesDocument::parse(bytes.into()).map_err(|error| {
            notation_error(record.line_number, id, record.notation_range.start, error)
        })
    }
    /// Retained inline/source/index bytes with shared allocations counted once. Excludes
    /// Arc/allocator headers, temporary reads and external copies; arithmetic saturates.
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

fn notation_error(
    line_number: usize,
    record_id: RecordId,
    start: usize,
    error: SmilesError,
) -> SmilesFileError {
    SmilesFileError::Notation {
        line_number,
        record_id,
        offset: start.saturating_add(error.offset()),
        error,
    }
}
fn check_cancel(
    cancelled: &mut impl FnMut() -> bool,
    line_number: usize,
    record_id: Option<RecordId>,
    offset: usize,
) -> Result<(), SmilesFileError> {
    if cancelled() {
        Err(SmilesFileError::Cancelled {
            line_number,
            record_id,
            offset,
        })
    } else {
        Ok(())
    }
}
fn index_records(
    source: &[u8],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<RecordInfo>, SmilesFileError> {
    check_cancel(cancelled, 1, None, 0)?;
    if source.len() > MAX_FILE_BYTES {
        return Err(SmilesFileError::InputLimit {
            offset: MAX_FILE_BYTES,
        });
    }
    let mut records = Vec::new();
    let mut position = 0_usize;
    let mut physical_lines = 0_usize;
    for line in source.split_inclusive(|byte| *byte == b'\n') {
        let number = physical_lines.saturating_add(1);
        check_cancel(cancelled, number, None, position)?;
        if number > MAX_PHYSICAL_LINES {
            return Err(SmilesFileError::LineLimit {
                line_number: number,
                offset: position,
            });
        }
        physical_lines = number;
        let start = position;
        let full_end = start
            .checked_add(line.len())
            .ok_or(SmilesFileError::InputLimit {
                offset: MAX_FILE_BYTES,
            })?;
        // Strip only a recognized ending from the borrowed view, never the retained bytes.
        let body = line
            .strip_suffix(b"\r\n")
            .or_else(|| line.strip_suffix(b"\n"))
            .unwrap_or(line);
        position = full_end;
        if body
            .first()
            .is_none_or(|byte| matches!(*byte, b' ' | b'\t' | b'\r'))
        {
            continue;
        }
        if records.len() >= MAX_RECORDS {
            return Err(SmilesFileError::RecordLimit {
                line_number: number,
                offset: start,
            });
        }
        let id = u32::try_from(records.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .and_then(RecordId::new)
            .ok_or(SmilesFileError::RecordLimit {
                line_number: number,
                offset: start,
            })?;
        let notation_length = body
            .iter()
            .position(|byte| matches!(*byte, b' ' | b'\t'))
            .unwrap_or(body.len());
        let notation = body
            .get(..notation_length)
            .ok_or(SmilesFileError::InvalidSpan { record_id: id })?;
        check_cancel(cancelled, number, Some(id), start)?;
        // Refuse before copying an oversized token; the unchanged parser owns this same limit.
        if notation.len() > MAX_SMILES_BYTES {
            return Err(notation_error(
                number,
                id,
                start,
                SmilesError::Limit {
                    offset: MAX_SMILES_BYTES,
                    kind: Limit::Bytes,
                },
            ));
        }
        let parsed = SmilesDocument::parse(notation.into());
        check_cancel(cancelled, number, Some(id), start)?;
        let parsed = parsed.map_err(|error| notation_error(number, id, start, error))?;
        let notation_end = start
            .checked_add(notation_length)
            .ok_or(SmilesFileError::InvalidSpan { record_id: id })?;
        let content_end = start
            .checked_add(body.len())
            .ok_or(SmilesFileError::InvalidSpan { record_id: id })?;
        let (separator_range, application_data_range) = if notation_length < body.len() {
            let separator_length = body
                .get(notation_length..)
                .ok_or(SmilesFileError::InvalidSpan { record_id: id })?
                .iter()
                .take_while(|byte| matches!(**byte, b' ' | b'\t'))
                .count();
            let separator_end = notation_end
                .checked_add(separator_length)
                .ok_or(SmilesFileError::InvalidSpan { record_id: id })?;
            (
                Some(notation_end..separator_end),
                Some(separator_end..content_end),
            )
        } else {
            (None, None)
        };
        records.push(RecordInfo {
            id,
            line_number: number,
            full_range: start..full_end,
            notation_range: start..notation_end,
            separator_range,
            application_data_range,
            line_ending_range: content_end..full_end,
            atom_count: parsed.atoms().len(),
            bond_count: parsed.bonds().len(),
        });
    }
    check_cancel(cancelled, physical_lines.max(1), None, source.len())?;
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_is_observed_before_each_line_and_around_notation_parsing()
    -> Result<(), Box<dyn std::error::Error>> {
        for (checkpoint, line, record, offset) in [
            (3, 1, Some(1), 0),
            (4, 1, Some(1), 0),
            (5, 2, None, 3),
            (6, 2, Some(2), 3),
            (7, 2, Some(2), 3),
        ] {
            let mut count = 0;
            let error = index_records(b"CC\nN\n", &mut || {
                count += 1;
                count == checkpoint
            })
            .err()
            .ok_or("missed cooperative checkpoint")?;
            assert_eq!(error.code(), "cancelled");
            assert_eq!(error.line_number(), Some(line));
            assert_eq!(error.record_id().map(RecordId::get), record);
            assert_eq!(error.offset(), Some(offset));
        }
        let mut count = 0;
        let error = index_records(b" \xff\n\tC(\n", &mut || {
            count += 1;
            count == 3
        })
        .err()
        .ok_or("ignored line missed cancellation")?;
        assert_eq!(error.line_number(), Some(2));
        assert_eq!(error.offset(), Some(3));
        Ok(())
    }
}
