//! Source-preserving SDF V2000/V3000 records and ordered data items.
//!
//! BIOVIA Chemistry 2026 CTfile Formats, printed pages 97–99, defines the framing.
//! ASCII/size budgets, unambiguous header identities, balanced registry parentheses
//! and strict complete-file EOF are
//! this implementation's subset policies. Extra empty lines are accepted only after
//! a completed data item, accommodating external empty-value writers without trimming
//! spaces-only data. Original bytes are never normalized or reconstructed from a map.

use crate::mol::{MolError, MolFile, Version, source_version, v3000};
use std::ops::Range;

mod document;
pub use document::{RecordId, RecordInfo, SdfDocument};

pub const MAX_SDF_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_SDF_RECORDS: usize = 1024;
pub const MAX_SDF_FIELDS: usize = 65_536;
pub const MAX_SDF_LINES: usize = 1_000_000;
pub const MAX_SDF_LINE_BYTES: usize = 4096;
pub const MAX_SDF_VALUE_BYTES: usize = 200;

#[derive(Debug, thiserror::Error)]
pub enum SdfError {
    #[error("SDF input or output exceeds 64 MiB")]
    ByteLimit,
    #[error("SDF exceeds 1024 records")]
    RecordLimit,
    #[error("SDF exceeds 65536 total data fields")]
    FieldLimit,
    #[error("SDF exceeds 1000000 lines")]
    LineLimit,
    #[error("SDF line {line} exceeds {limit} bytes")]
    LineTooLong { line: usize, limit: usize },
    #[error("this SDF profile supports ASCII only (byte {offset})")]
    UnsupportedEncoding { offset: usize },
    #[error("invalid ASCII control byte at SDF byte {offset}")]
    InvalidControl { offset: usize },
    #[error("invalid line ending at SDF line {line}")]
    InvalidLineEnding { line: usize },
    #[error("an SDfile requires at least one complete molecule record")]
    Empty,
    #[error("record {record} is missing M  END")]
    MissingMolEnd { record: usize },
    #[error("record {record} is missing its $$$$ delimiter")]
    MissingDelimiter { record: usize },
    #[error("data field at line {line} is missing its blank terminator")]
    MissingFieldTerminator { line: usize },
    #[error("unexpected SDF data or padding at line {line}")]
    UnexpectedLine { line: usize },
    #[error("invalid SDF data header at line {line}: {reason}")]
    InvalidHeader { line: usize, reason: &'static str },
    #[error("unsupported SDF data header at line {line}: {reason}")]
    UnsupportedHeader { line: usize, reason: &'static str },
    #[error("data-only datfiles are not supported by this molecule SDF codec")]
    DataOnly,
    #[error("raw MOL writer input contains padding after M  END")]
    MolPadding,
    #[error("MOL block for SDF record {record} contains a $$$$ delimiter at line {line}")]
    DelimiterInMol { record: usize, line: usize },
    #[error("writer data lines must be nonempty single lines and cannot begin with $$$$")]
    InvalidValue,
    #[error("invalid source span")]
    InvalidSpan,
    #[error("MOL syntax in SDF record {record}: {source}")]
    Mol {
        record: usize,
        #[source]
        source: MolError,
    },
    #[error("V3000 MOL syntax in SDF record {record}: {source}")]
    MolV3000 {
        record: usize,
        #[source]
        source: v3000::Error,
    },
    #[error("SDF record {record} uses {version:?}; this writer accepts V2000 only")]
    MolVersion { record: usize, version: Version },
}

impl SdfError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ByteLimit
            | Self::RecordLimit
            | Self::FieldLimit
            | Self::LineLimit
            | Self::LineTooLong { .. } => "resource_limit",
            Self::UnsupportedEncoding { .. }
            | Self::UnsupportedHeader { .. }
            | Self::DataOnly
            | Self::MolVersion { .. } => "unsupported_content",
            Self::Mol { source, .. } => match source.code() {
                "invalid_mol" => "invalid_sdf",
                other => other,
            },
            Self::MolV3000 { source, .. } => match source.code() {
                "invalid_mol" => "invalid_sdf",
                other => other,
            },
            _ => "invalid_sdf",
        }
    }
}

pub struct SdfFile<'a> {
    source: &'a [u8],
    records: Vec<SdfRecord<'a>>,
}

pub struct SdfRecord<'a> {
    range: Range<usize>,
    mol_range: Range<usize>,
    delimiter_range: Range<usize>,
    mol_bytes: &'a [u8],
    mol_version: Version,
    fields: Vec<DataField<'a>>,
}

pub struct DataField<'a> {
    range: Range<usize>,
    header_range: Range<usize>,
    value_range: Range<usize>,
    header: &'a str,
    name: Option<&'a str>,
    dt_number: Option<&'a str>,
    raw_value: &'a [u8],
    value_lines: Vec<&'a str>,
}

impl std::fmt::Debug for SdfFile<'_> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("SdfFile")
            .field("byte_length", &self.source.len())
            .field("record_count", &self.records.len())
            .finish()
    }
}
impl std::fmt::Debug for SdfRecord<'_> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("SdfRecord")
            .field("range", &self.range)
            .field("mol_version", &self.mol_version)
            .field("field_count", &self.fields.len())
            .finish()
    }
}
impl std::fmt::Debug for DataField<'_> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("DataField")
            .field("range", &self.range)
            .field("value_lines", &self.value_lines.len())
            .finish()
    }
}

impl<'a> SdfRecord<'a> {
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn mol_range(&self) -> Range<usize> {
        self.mol_range.clone()
    }
    pub fn delimiter_range(&self) -> Range<usize> {
        self.delimiter_range.clone()
    }
    pub fn mol_bytes(&self) -> &'a [u8] {
        self.mol_bytes
    }
    /// Actual embedded MOL encoding; an SDfile can contain both versions.
    pub fn mol_version(&self) -> Version {
        self.mol_version
    }
    pub fn fields(&self) -> &[DataField<'a>] {
        &self.fields
    }
}
impl<'a> DataField<'a> {
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    /// Includes the original header line ending; header() itself excludes that ending.
    pub fn header_range(&self) -> Range<usize> {
        self.header_range.clone()
    }
    pub fn value_range(&self) -> Range<usize> {
        self.value_range.clone()
    }
    pub fn header(&self) -> &'a str {
        self.header
    }
    pub fn name(&self) -> Option<&'a str> {
        self.name
    }
    pub fn dt_number(&self) -> Option<&'a str> {
        self.dt_number
    }
    pub fn raw_value(&self) -> &'a [u8] {
        self.raw_value
    }
    pub fn value_lines(&self) -> &[&'a str] {
        &self.value_lines
    }
}

#[derive(Clone, Copy)]
struct Line {
    start: usize,
    content_end: usize,
    end: usize,
    number: usize,
}
impl Line {
    fn text(self, source: &str) -> Result<&str, SdfError> {
        source
            .get(self.start..self.content_end)
            .ok_or(SdfError::InvalidSpan)
    }
    fn range(self) -> Range<usize> {
        self.start..self.end
    }
}
struct Cursor<'a> {
    source: &'a str,
    offset: usize,
    lines: usize,
}
impl<'a> Cursor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            lines: 0,
        }
    }
    fn next(&mut self) -> Result<Option<Line>, SdfError> {
        if self.offset == self.source.len() {
            return Ok(None);
        }
        self.lines = self.lines.checked_add(1).ok_or(SdfError::LineLimit)?;
        if self.lines > MAX_SDF_LINES {
            return Err(SdfError::LineLimit);
        }
        let rest = self
            .source
            .get(self.offset..)
            .ok_or(SdfError::InvalidSpan)?;
        let newline = rest.bytes().position(|byte| byte == b'\n');
        let end = match newline {
            Some(index) => self
                .offset
                .checked_add(index)
                .and_then(|value| value.checked_add(1))
                .ok_or(SdfError::ByteLimit)?,
            None => self.source.len(),
        };
        let mut content_end = if newline.is_some() {
            end.checked_sub(1).ok_or(SdfError::InvalidSpan)?
        } else {
            end
        };
        if content_end > self.offset && self.source.as_bytes().get(content_end - 1) == Some(&b'\r')
        {
            if newline.is_none() {
                return Err(SdfError::InvalidLineEnding { line: self.lines });
            }
            content_end -= 1;
        }
        let line = Line {
            start: self.offset,
            content_end,
            end,
            number: self.lines,
        };
        let value = line.text(self.source)?;
        if value.contains('\r') {
            return Err(SdfError::InvalidLineEnding { line: self.lines });
        }
        if value.len() > MAX_SDF_LINE_BYTES {
            return Err(SdfError::LineTooLong {
                line: self.lines,
                limit: MAX_SDF_LINE_BYTES,
            });
        }
        self.offset = end;
        Ok(Some(line))
    }
}

fn ascii(bytes: &[u8]) -> Result<(), SdfError> {
    for (offset, byte) in bytes.iter().enumerate() {
        if !byte.is_ascii() {
            return Err(SdfError::UnsupportedEncoding { offset });
        }
        if byte.is_ascii_control() && !matches!(byte, b'\t' | b'\r' | b'\n') {
            return Err(SdfError::InvalidControl { offset });
        }
    }
    Ok(())
}

fn word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}
fn header_ids(header: &str, line: usize) -> Result<(Option<&str>, Option<&str>), SdfError> {
    if !header.starts_with('>') || header.contains(['\r', '\n']) {
        return Err(SdfError::InvalidHeader {
            line,
            reason: "header must begin with > and occupy one line",
        });
    }
    if header.len() > MAX_SDF_LINE_BYTES {
        return Err(SdfError::LineTooLong {
            line,
            limit: MAX_SDF_LINE_BYTES,
        });
    }
    ascii(header.as_bytes())?;
    let bytes = header.as_bytes();
    let mut index = 1;
    let mut parentheses = 0_usize;
    let mut name = None;
    let mut dt_number = None;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'(' {
            parentheses = parentheses.saturating_add(1);
            index += 1;
            continue;
        }
        if byte == b')' {
            if parentheses == 0 {
                return Err(SdfError::UnsupportedHeader {
                    line,
                    reason: "unbalanced optional registry metadata",
                });
            }
            parentheses -= 1;
            index += 1;
            continue;
        }
        if parentheses > 0 {
            index += 1;
            continue;
        }
        if byte == b'<' {
            let start = index + 1;
            let relative = bytes
                .get(start..)
                .ok_or(SdfError::InvalidSpan)?
                .iter()
                .position(|value| *value == b'>')
                .ok_or(SdfError::InvalidHeader {
                    line,
                    reason: "unclosed field name",
                })?;
            let end = start.checked_add(relative).ok_or(SdfError::InvalidSpan)?;
            let candidate = header.get(start..end).ok_or(SdfError::InvalidSpan)?;
            if !candidate
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
                || !candidate.bytes().all(word)
            {
                return Err(SdfError::InvalidHeader {
                    line,
                    reason: "name must start alphabetic and contain only alphanumeric or underscore",
                });
            }
            if name.replace(candidate).is_some() {
                return Err(SdfError::UnsupportedHeader {
                    line,
                    reason: "multiple field names",
                });
            }
            index = end + 1;
            continue;
        }
        if byte == b'>' {
            return Err(SdfError::InvalidHeader {
                line,
                reason: "unmatched > in header",
            });
        }
        if byte == b'D'
            && bytes.get(index + 1) == Some(&b'T')
            && !bytes.get(index - 1).copied().is_some_and(word)
        {
            let start = index + 2;
            let mut end = start;
            while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                end += 1;
            }
            if end > start && !bytes.get(end).copied().is_some_and(word) {
                let candidate = header.get(start..end).ok_or(SdfError::InvalidSpan)?;
                if dt_number.replace(candidate).is_some() {
                    return Err(SdfError::UnsupportedHeader {
                        line,
                        reason: "multiple DT field numbers",
                    });
                }
                index = end;
                continue;
            }
        }
        index += 1;
    }
    if parentheses != 0 {
        return Err(SdfError::UnsupportedHeader {
            line,
            reason: "unbalanced optional registry metadata",
        });
    }
    if name.is_none() && dt_number.is_none() {
        return Err(SdfError::InvalidHeader {
            line,
            reason: "missing field name or DT field number",
        });
    }
    Ok((name, dt_number))
}

impl<'a> SdfFile<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.source
    }
    pub fn records(&self) -> &[SdfRecord<'a>] {
        &self.records
    }

    pub fn parse(source: &'a [u8]) -> Result<Self, SdfError> {
        if source.len() > MAX_SDF_BYTES {
            return Err(SdfError::ByteLimit);
        }
        ascii(source)?;
        let text =
            std::str::from_utf8(source).map_err(|_| SdfError::UnsupportedEncoding { offset: 0 })?;
        let mut cursor = Cursor::new(text);
        let mut records = Vec::new();
        let mut field_count = 0_usize;
        while let Some(first) = cursor.next()? {
            if records.len() >= MAX_SDF_RECORDS {
                return Err(SdfError::RecordLimit);
            }
            let record_number = records.len() + 1;
            let mut current = first;
            let mut mol_lines = 0_usize;
            let mol_end = loop {
                let content = current.text(text)?;
                if content.starts_with("$$$$") {
                    return Err(SdfError::MissingMolEnd {
                        record: record_number,
                    });
                }
                if mol_lines >= 4 && content.trim_end() == "M  END" {
                    break current.end;
                }
                mol_lines = mol_lines.checked_add(1).ok_or(SdfError::LineLimit)?;
                current = match cursor.next()? {
                    Some(line) => line,
                    None if first.text(text)?.starts_with('>') => return Err(SdfError::DataOnly),
                    None => {
                        return Err(SdfError::MissingMolEnd {
                            record: record_number,
                        });
                    }
                };
            };
            let mol_range = first.start..mol_end;
            let mol_bytes = source.get(mol_range.clone()).ok_or(SdfError::InvalidSpan)?;
            let mol_version = validate_mol(mol_bytes, record_number, false)?;
            let mut fields = Vec::new();
            let delimiter = loop {
                let line = cursor.next()?.ok_or(SdfError::MissingDelimiter {
                    record: record_number,
                })?;
                let content = line.text(text)?;
                if content.starts_with("$$$$") {
                    break line;
                }
                if content.is_empty() {
                    if fields.is_empty() {
                        return Err(SdfError::UnexpectedLine { line: line.number });
                    }
                    continue;
                }
                let (name, dt_number) = header_ids(content, line.number)?;
                field_count = field_count.checked_add(1).ok_or(SdfError::FieldLimit)?;
                if field_count > MAX_SDF_FIELDS {
                    return Err(SdfError::FieldLimit);
                }
                let mut values = Vec::new();
                let blank = loop {
                    let value = cursor
                        .next()?
                        .ok_or(SdfError::MissingFieldTerminator { line: line.number })?;
                    let value_text = value.text(text)?;
                    if value_text.is_empty() {
                        break value;
                    }
                    if value_text.starts_with("$$$$") {
                        return Err(SdfError::MissingFieldTerminator { line: line.number });
                    }
                    if value_text.len() > MAX_SDF_VALUE_BYTES {
                        return Err(SdfError::LineTooLong {
                            line: value.number,
                            limit: MAX_SDF_VALUE_BYTES,
                        });
                    }
                    values.push(value_text);
                };
                let value_range = line.end..blank.start;
                fields.push(DataField {
                    range: line.start..blank.end,
                    header_range: line.range(),
                    value_range: value_range.clone(),
                    header: content,
                    name,
                    dt_number,
                    raw_value: source.get(value_range).ok_or(SdfError::InvalidSpan)?,
                    value_lines: values,
                });
            };
            records.push(SdfRecord {
                range: first.start..delimiter.end,
                mol_range,
                delimiter_range: delimiter.range(),
                mol_bytes,
                mol_version,
                fields,
            });
        }
        if records.is_empty() {
            return Err(SdfError::Empty);
        }
        Ok(Self { source, records })
    }
}

/// A new data item. Header metadata and duplicate items are caller-owned ordered data,
/// not a dictionary. Empty value is an empty slice; blank value lines are invalid.
#[derive(Clone, Copy, Debug)]
pub struct DataItemInput<'a> {
    pub header: &'a str,
    pub value_lines: &'a [&'a str],
}
#[derive(Clone, Copy, Debug)]
pub struct SdfRecordInput<'a> {
    pub mol_bytes: &'a [u8],
    pub items: &'a [DataItemInput<'a>],
}

fn add_bytes(total: &mut usize, count: usize) -> Result<(), SdfError> {
    *total = total
        .checked_add(count)
        .filter(|value| *value <= MAX_SDF_BYTES)
        .ok_or(SdfError::ByteLimit)?;
    Ok(())
}
fn add_lines(total: &mut usize, count: usize) -> Result<(), SdfError> {
    *total = total
        .checked_add(count)
        .filter(|value| *value <= MAX_SDF_LINES)
        .ok_or(SdfError::LineLimit)?;
    Ok(())
}

fn validate_mol(bytes: &[u8], record: usize, v2000_only: bool) -> Result<Version, SdfError> {
    let version = source_version(bytes).map_err(|source| SdfError::Mol { record, source })?;
    if v2000_only && version != Version::V2000 {
        return Err(SdfError::MolVersion { record, version });
    }
    match version {
        Version::V2000 => {
            MolFile::parse(bytes).map_err(|source| SdfError::Mol { record, source })?;
        }
        Version::V3000 => {
            v3000::MolFile::parse(bytes).map_err(|source| SdfError::MolV3000 { record, source })?;
        }
    }
    Ok(version)
}

/// Construct new SDF framing while preserving each complete MOL payload and its
/// actual V2000/V3000 encoding. This does not convert or project molecules. Supplied
/// bytes retain their EOLs; only a missing final LF is appended. New SDF lines use LF.
pub fn write(records: &[SdfRecordInput<'_>]) -> Result<Vec<u8>, SdfError> {
    write_records(records, false)
}

/// Construct canonical SDF framing with one blank per item and exact $$$$ delimiters.
/// Supplied V2000 MOL bytes are validated and retained, including CRLF and unknown
/// properties; only a missing final LF is appended. New wrapper lines use LF, so mixed
/// line endings are deliberate. This is separate from exact source passthrough.
pub fn write_v2000(records: &[SdfRecordInput<'_>]) -> Result<Vec<u8>, SdfError> {
    write_records(records, true)
}

fn write_records(records: &[SdfRecordInput<'_>], v2000_only: bool) -> Result<Vec<u8>, SdfError> {
    if records.is_empty() {
        return Err(SdfError::Empty);
    }
    if records.len() > MAX_SDF_RECORDS {
        return Err(SdfError::RecordLimit);
    }
    let mut byte_count = 0_usize;
    let mut line_count = 0_usize;
    let mut field_count = 0_usize;
    for (index, record) in records.iter().enumerate() {
        if record.mol_bytes.len() > MAX_SDF_BYTES {
            return Err(SdfError::ByteLimit);
        }
        ascii(record.mol_bytes)?;
        validate_mol(record.mol_bytes, index + 1, v2000_only)?;
        let text = std::str::from_utf8(record.mol_bytes)
            .map_err(|_| SdfError::UnsupportedEncoding { offset: 0 })?;
        let mut cursor = Cursor::new(text);
        let mut lines = 0_usize;
        let mut end = None;
        while let Some(line) = cursor.next()? {
            if line.text(text)?.starts_with("$$$$") {
                return Err(SdfError::DelimiterInMol {
                    record: index + 1,
                    line: line.number,
                });
            }
            if lines >= 4 && line.text(text)?.trim_end() == "M  END" {
                end = Some(line.end);
            }
            lines = lines.checked_add(1).ok_or(SdfError::LineLimit)?;
        }
        if end != Some(record.mol_bytes.len()) {
            return Err(SdfError::MolPadding);
        }
        add_bytes(&mut byte_count, record.mol_bytes.len())?;
        if !record.mol_bytes.ends_with(b"\n") {
            add_bytes(&mut byte_count, 1)?;
        }
        add_lines(&mut line_count, lines)?;
        field_count = field_count
            .checked_add(record.items.len())
            .filter(|count| *count <= MAX_SDF_FIELDS)
            .ok_or(SdfError::FieldLimit)?;
        for item in record.items {
            header_ids(item.header, 0)?;
            add_bytes(&mut byte_count, item.header.len())?;
            add_bytes(&mut byte_count, 2)?;
            add_lines(&mut line_count, 2)?;
            for value in item.value_lines {
                ascii(value.as_bytes())?;
                if value.is_empty() || value.contains(['\r', '\n']) || value.starts_with("$$$$") {
                    return Err(SdfError::InvalidValue);
                }
                if value.len() > MAX_SDF_VALUE_BYTES {
                    return Err(SdfError::LineTooLong {
                        line: 0,
                        limit: MAX_SDF_VALUE_BYTES,
                    });
                }
                add_bytes(&mut byte_count, value.len())?;
                add_bytes(&mut byte_count, 1)?;
                add_lines(&mut line_count, 1)?;
            }
        }
        add_bytes(&mut byte_count, 5)?;
        add_lines(&mut line_count, 1)?;
    }
    let mut output = Vec::with_capacity(byte_count);
    for record in records {
        output.extend_from_slice(record.mol_bytes);
        if !record.mol_bytes.ends_with(b"\n") {
            output.push(b'\n');
        }
        for item in record.items {
            output.extend_from_slice(item.header.as_bytes());
            output.push(b'\n');
            for value in item.value_lines {
                output.extend_from_slice(value.as_bytes());
                output.push(b'\n');
            }
            output.push(b'\n');
        }
        output.extend_from_slice(b"$$$$\n");
    }
    Ok(output)
}
