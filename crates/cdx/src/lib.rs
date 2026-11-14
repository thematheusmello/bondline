//! Standalone CDX source preservation, property edits and bounded new-tree syntax.
//! No model, native I/O or chemical interpretation.

use std::{fmt, ops::Range};

mod write;
pub use write::Writer;

pub const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_RECORDS: usize = 100_000;
/// Maximum simultaneously open encoded object records; the implied root is not encoded.
pub const MAX_DEPTH: usize = 256;
pub const HEADER_BYTES: usize = 28;
const PREFIX: &[u8] = b"VjCD0100\x04\x03\x02\x01";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Framing {
    ImplicitDocument,
    ExplicitDocument,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LengthEncoding {
    Short,
    Extended,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordKind {
    ObjectStart {
        tag: u16,
        id: u32,
    },
    Property {
        tag: u16,
        length: LengthEncoding,
        payload: Range<usize>,
    },
    ObjectEnd {
        start: Option<usize>,
    },
}

/// A handle owned by one parsed document. Its fields cannot be forged or mutated.
#[derive(Debug)]
pub struct Record {
    index: usize,
    parent: Option<usize>,
    range: Range<usize>,
    kind: RecordKind,
}

impl Record {
    pub fn kind(&self) -> &RecordKind {
        &self.kind
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    /// Opening-record index of the containing object. For an end, this is the matched
    /// opening's parent; `ObjectEnd::start` identifies the object being closed.
    pub fn parent(&self) -> Option<usize> {
        self.parent
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InputTooLarge,
    InvalidHeader,
    Truncated { offset: usize },
    TrailingData,
    DepthLimit,
    RecordLimit,
    InvalidProperty,
    InvalidPropertyRange,
    InvalidTag,
    ObjectBalance,
    Allocation,
}

impl fmt::Display for Error {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge => output.write_str("CDX input or output exceeds 64 MiB"),
            Self::InvalidHeader => output.write_str(
                "CDX requires the current 28-byte VjCD0100 header and little-endian marker",
            ),
            Self::Truncated { offset } => write!(output, "truncated CDX record at byte {offset}"),
            Self::TrailingData => output.write_str("CDX has bytes after its root terminator"),
            Self::DepthLimit => {
                output.write_str("CDX exceeds 256 simultaneously open object records")
            }
            Self::RecordLimit => output.write_str("CDX exceeds 100000 records"),
            Self::InvalidProperty => output
                .write_str("property operations require a property record owned by this document"),
            Self::InvalidPropertyRange => {
                output.write_str("CDX property ranges are inconsistent with the source")
            }
            Self::InvalidTag => output.write_str(
                "CDX object tags need bit15 set; property tags must be nonzero with bit15 clear",
            ),
            Self::ObjectBalance => output.write_str(
                "object begin/end calls must balance inside the automatically written document root",
            ),
            Self::Allocation => output.write_str("unable to allocate a bounded CDX buffer"),
        }
    }
}

impl std::error::Error for Error {}

/// An immutable borrowed source plus a flat index. Destruction never recurses over input.
pub struct Document<'input> {
    source: &'input [u8],
    header: &'input [u8],
    framing: Framing,
    records: Vec<Record>,
}

impl fmt::Debug for Document<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output
            .debug_struct("Document")
            .field("byte_length", &self.source.len())
            .field("record_count", &self.records.len())
            .finish()
    }
}

impl<'input> Document<'input> {
    pub fn parse(input: &'input [u8]) -> Result<Self, Error> {
        if input.len() > MAX_INPUT_BYTES {
            return Err(Error::InputTooLarge);
        }
        let header = input
            .get(..HEADER_BYTES)
            .filter(|_| input.starts_with(PREFIX))
            .ok_or(Error::InvalidHeader)?;
        let framing = if input.get(HEADER_BYTES..HEADER_BYTES + 2) == Some(&[0, 0x80]) {
            Framing::ExplicitDocument
        } else {
            Framing::ImplicitDocument
        };
        let mut cursor = Cursor {
            source: input,
            offset: HEADER_BYTES,
        };
        let mut records = Vec::new();
        let mut stack = Vec::new();
        loop {
            let start = cursor.offset;
            let tag = cursor.u16()?;
            let kind;
            let parent;
            if tag == 0 {
                let opening = stack.pop();
                parent = stack.last().copied();
                kind = RecordKind::ObjectEnd { start: opening };
                push_record(&mut records, parent, start..cursor.offset, kind)?;
                if opening.is_none() || (framing == Framing::ExplicitDocument && stack.is_empty()) {
                    if cursor.offset != input.len() {
                        return Err(Error::TrailingData);
                    }
                    return Ok(Self {
                        source: input,
                        header,
                        framing,
                        records,
                    });
                }
            } else if tag & 0x8000 != 0 {
                if stack.len() >= MAX_DEPTH {
                    return Err(Error::DepthLimit);
                }
                let id = cursor.u32()?;
                parent = stack.last().copied();
                kind = RecordKind::ObjectStart { tag, id };
                let index = records.len();
                push_record(&mut records, parent, start..cursor.offset, kind)?;
                stack.try_reserve(1).map_err(|_| Error::Allocation)?;
                stack.push(index);
            } else {
                parent = stack.last().copied();
                let short = cursor.u16()?;
                let (length, count) = if short == u16::MAX {
                    (
                        LengthEncoding::Extended,
                        usize::try_from(cursor.u32()?).map_err(|_| Error::InputTooLarge)?,
                    )
                } else {
                    (LengthEncoding::Short, usize::from(short))
                };
                let payload_start = cursor.offset;
                cursor.take(count)?;
                kind = RecordKind::Property {
                    tag,
                    length,
                    payload: payload_start..cursor.offset,
                };
                push_record(&mut records, parent, start..cursor.offset, kind)?;
            }
        }
    }

    pub fn as_bytes(&self) -> &'input [u8] {
        self.source
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.source.to_vec()
    }
    pub fn header(&self) -> &'input [u8] {
        self.header
    }
    pub fn framing(&self) -> Framing {
        self.framing
    }
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    pub fn payload(&self, record: &Record) -> Result<&'input [u8], Error> {
        let owned = self
            .records
            .get(record.index)
            .filter(|owned| std::ptr::eq(*owned, record))
            .ok_or(Error::InvalidProperty)?;
        let RecordKind::Property {
            length, payload, ..
        } = &owned.kind
        else {
            return Err(Error::InvalidProperty);
        };
        let prefix = match length {
            LengthEncoding::Short => 4,
            LengthEncoding::Extended => 8,
        };
        if owned.range.start < HEADER_BYTES
            || owned.range.start.checked_add(prefix) != Some(payload.start)
            || payload.start > payload.end
            || payload.end != owned.range.end
        {
            return Err(Error::InvalidPropertyRange);
        }
        self.source
            .get(payload.clone())
            .ok_or(Error::InvalidPropertyRange)
    }

    /// Replace only an existing opaque payload. Untouched bytes and the parsed source
    /// remain exact. Changed lengths use the documented threshold; no-ops preserve even
    /// a noncanonical extended prefix from the original input.
    pub fn replace_property(&self, record: &Record, value: &[u8]) -> Result<Vec<u8>, Error> {
        let old = self.payload(record)?;
        let prefix_size = if value.len() < usize::from(u16::MAX) {
            4_usize
        } else {
            8
        };
        let old_size = record
            .range
            .end
            .checked_sub(record.range.start)
            .ok_or(Error::InvalidPropertyRange)?;
        let output_size = self
            .source
            .len()
            .checked_sub(old_size)
            .and_then(|size| size.checked_add(prefix_size))
            .and_then(|size| size.checked_add(value.len()))
            .filter(|size| *size <= MAX_INPUT_BYTES)
            .ok_or(Error::InputTooLarge)?;
        if value == old {
            return copy_source(self.source);
        }
        let tag_end = record
            .range
            .start
            .checked_add(2)
            .ok_or(Error::InvalidPropertyRange)?;
        let before = self
            .source
            .get(..tag_end)
            .ok_or(Error::InvalidPropertyRange)?;
        let after = self
            .source
            .get(record.range.end..)
            .ok_or(Error::InvalidPropertyRange)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(output_size)
            .map_err(|_| Error::Allocation)?;
        output.extend_from_slice(before);
        if prefix_size == 4 {
            let length = u16::try_from(value.len()).map_err(|_| Error::InputTooLarge)?;
            output.extend_from_slice(&length.to_le_bytes());
        } else {
            let length = u32::try_from(value.len()).map_err(|_| Error::InputTooLarge)?;
            output.extend_from_slice(&u16::MAX.to_le_bytes());
            output.extend_from_slice(&length.to_le_bytes());
        }
        output.extend_from_slice(value);
        output.extend_from_slice(after);
        Ok(output)
    }
}

fn copy_source(source: &[u8]) -> Result<Vec<u8>, Error> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(source.len())
        .map_err(|_| Error::Allocation)?;
    output.extend_from_slice(source);
    Ok(output)
}

fn push_record(
    records: &mut Vec<Record>,
    parent: Option<usize>,
    range: Range<usize>,
    kind: RecordKind,
) -> Result<(), Error> {
    if records.len() >= MAX_RECORDS {
        return Err(Error::RecordLimit);
    }
    records.try_reserve(1).map_err(|_| Error::Allocation)?;
    records.push(Record {
        index: records.len(),
        parent,
        range,
        kind,
    });
    Ok(())
}

struct Cursor<'input> {
    source: &'input [u8],
    offset: usize,
}

impl<'input> Cursor<'input> {
    fn take(&mut self, count: usize) -> Result<&'input [u8], Error> {
        let end = self.offset.checked_add(count).ok_or(Error::Truncated {
            offset: self.offset,
        })?;
        let bytes = self.source.get(self.offset..end).ok_or(Error::Truncated {
            offset: self.offset,
        })?;
        self.offset = end;
        Ok(bytes)
    }
    fn u16(&mut self) -> Result<u16, Error> {
        let bytes = self.take(2)?.try_into().map_err(|_| Error::Truncated {
            offset: self.offset,
        })?;
        Ok(u16::from_le_bytes(bytes))
    }
    fn u32(&mut self) -> Result<u32, Error> {
        let bytes = self.take(4)?.try_into().map_err(|_| Error::Truncated {
            offset: self.offset,
        })?;
        Ok(u32::from_le_bytes(bytes))
    }
}
