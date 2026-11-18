//! Borrowed CDXString layout. The caller chooses which properties use this datatype.
use crate::{Cursor, Document, Error, Record};
use std::{fmt, ops::Range};

const RUN_BYTES: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringError {
    Source(Error),
    InvalidPayload { record: usize },
}

impl fmt::Display for StringError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => write!(output, "CDX string source error: {error}"),
            Self::InvalidPayload { record } => {
                write!(
                    output,
                    "CDX string style block exceeds property at record {record}"
                )
            }
        }
    }
}

impl std::error::Error for StringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            Self::InvalidPayload { .. } => None,
        }
    }
}

/// Five literal words in a style run, with its span in the original CDX source.
#[derive(Debug)]
pub struct StyleRun {
    start: u16,
    font: u16,
    face: u16,
    size_twentieths: u16,
    color: u16,
    range: Range<usize>,
}

impl StyleRun {
    pub fn start(&self) -> u16 {
        self.start
    }
    pub fn font(&self) -> u16 {
        self.font
    }
    pub fn face(&self) -> u16 {
        self.face
    }
    pub fn size_twentieths(&self) -> u16 {
        self.size_twentieths
    }
    pub fn color(&self) -> u16 {
        self.color
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}

/// A bounded view of one owned property, with literal runs and undecoded text.
#[derive(Debug)]
pub struct StringValue<'doc, 'input> {
    record: &'doc Record,
    runs: &'input [[u8; RUN_BYTES]],
    runs_start: usize,
    text: &'input [u8],
}

impl<'doc, 'input> StringValue<'doc, 'input> {
    pub fn record(&self) -> &'doc Record {
        self.record
    }

    pub fn text_bytes(&self) -> &'input [u8] {
        self.text
    }

    pub fn text_range(&self) -> Range<usize> {
        let end = self.record.range.end;
        end - self.text.len()..end
    }

    pub fn runs(&self) -> impl ExactSizeIterator<Item = StyleRun> + '_ {
        self.runs.iter().enumerate().map(|(ordinal, raw)| {
            let &[
                start_lo,
                start_hi,
                font_lo,
                font_hi,
                face_lo,
                face_hi,
                size_lo,
                size_hi,
                color_lo,
                color_hi,
            ] = raw;
            let offset = self.runs_start + ordinal * RUN_BYTES;
            StyleRun {
                start: u16::from_le_bytes([start_lo, start_hi]),
                font: u16::from_le_bytes([font_lo, font_hi]),
                face: u16::from_le_bytes([face_lo, face_hi]),
                size_twentieths: u16::from_le_bytes([size_lo, size_hi]),
                color: u16::from_le_bytes([color_lo, color_hi]),
                range: offset..offset + RUN_BYTES,
            }
        })
    }
}

impl<'input> Document<'input> {
    /// Read the count and fixed-size run block within this property. Run-start units,
    /// font selection, encoding and style validity are outside this raw query.
    pub fn string<'doc>(
        &'doc self,
        record: &'doc Record,
    ) -> Result<StringValue<'doc, 'input>, StringError> {
        let source = self.payload(record).map_err(StringError::Source)?;
        let invalid = || StringError::InvalidPayload {
            record: record.index,
        };
        let mut cursor = Cursor { source, offset: 0 };
        let count = usize::from(cursor.u16().map_err(|_| invalid())?);
        let runs_start = record.range.end - source.len() + cursor.offset;
        let (runs, _) = cursor
            .take(count * RUN_BYTES)
            .map_err(|_| invalid())?
            .as_chunks();
        let text = source.get(cursor.offset..).ok_or_else(invalid)?;
        Ok(StringValue {
            record,
            runs,
            runs_start,
            text,
        })
    }
}
