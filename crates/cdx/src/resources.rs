//! Checked logical table data, without font codecs, style resolution or model authority.
use crate::{Cursor, Document, Error, Record, RecordKind};
use std::{fmt, ops::Range};

const FONT_TABLE: u16 = 0x0100;
const COLOR_TABLE: u16 = 0x0300;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceError {
    Source(Error),
    UnsupportedTable {
        tag: u16,
        record: usize,
    },
    DuplicateTable {
        tag: u16,
        record: usize,
    },
    InvalidTable {
        tag: u16,
        record: usize,
    },
    UnsupportedPlatform {
        record: usize,
        value: u16,
    },
    DuplicateFontId {
        record: usize,
        id: u16,
        ordinal: usize,
    },
    MissingFont(u16),
    MissingColor(u32),
    Allocation,
}

impl fmt::Display for ResourceError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => write!(output, "CDX resource source error: {error}"),
            Self::UnsupportedTable { tag, record } => write!(
                output,
                "unsupported CDX table {tag:#06x} owner at record {record}"
            ),
            Self::DuplicateTable { tag, record } => {
                write!(output, "duplicate CDX table {tag:#06x} at record {record}")
            }
            Self::InvalidTable { tag, record } => {
                write!(output, "invalid CDX table {tag:#06x} at record {record}")
            }
            Self::UnsupportedPlatform { record, value } => write!(
                output,
                "unsupported CDX font platform {value} at record {record}"
            ),
            Self::DuplicateFontId {
                record,
                id,
                ordinal,
            } => write!(
                output,
                "duplicate CDX font id {id} at ordinal {ordinal} in record {record}"
            ),
            Self::MissingFont(id) => write!(output, "CDX declared font id {id} is not defined"),
            Self::MissingColor(index) => write!(output, "CDX color index {index} is not defined"),
            Self::Allocation => {
                output.write_str("unable to allocate bounded CDX resource descriptors")
            }
        }
    }
}

impl std::error::Error for ResourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            _ => None,
        }
    }
}

impl From<Error> for ResourceError {
    fn from(error: Error) -> Self {
        Self::Source(error)
    }
}

/// A declared logical font; the exact name bytes do not imply a character codec.
#[derive(Debug)]
pub struct Font<'doc, 'input> {
    id: u16,
    charset: u16,
    name: &'input [u8],
    ordinal: usize,
    range: Range<usize>,
    record: &'doc Record,
}

impl<'doc, 'input> Font<'doc, 'input> {
    pub fn id(&self) -> u16 {
        self.id
    }
    pub fn charset(&self) -> u16 {
        self.charset
    }
    pub fn name_bytes(&self) -> &'input [u8] {
        self.name
    }
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn record(&self) -> &'doc Record {
        self.record
    }
}

/// One source-ordered RGB16 entry, whose implicit index starts at two.
#[derive(Debug)]
pub struct Color<'doc> {
    index: u32,
    rgb: [u16; 3],
    range: Range<usize>,
    record: &'doc Record,
}

impl<'doc> Color<'doc> {
    pub fn index(&self) -> u32 {
        self.index
    }
    pub fn rgb(&self) -> [u16; 3] {
        self.rgb
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn record(&self) -> &'doc Record {
        self.record
    }
}

/// Checked table descriptors. Success does not resolve styled-run references or rendering.
#[derive(Debug)]
pub struct Resources<'doc, 'input> {
    font_table: Option<&'doc Record>,
    color_table: Option<&'doc Record>,
    platform: Option<u16>,
    fonts: Vec<Font<'doc, 'input>>,
    font_index: Vec<(u16, usize)>,
    colors: Vec<Color<'doc>>,
}

impl<'doc, 'input> Resources<'doc, 'input> {
    pub fn font_table(&self) -> Option<&'doc Record> {
        self.font_table
    }
    pub fn color_table(&self) -> Option<&'doc Record> {
        self.color_table
    }
    pub fn font_platform(&self) -> Option<u16> {
        self.platform
    }
    pub fn fonts(&self) -> &[Font<'doc, 'input>] {
        &self.fonts
    }
    pub fn colors(&self) -> &[Color<'doc>] {
        &self.colors
    }

    /// Lookup by the table's declared ID, not a claim about a style-run font field.
    pub fn font_by_id(&self, id: u16) -> Result<&Font<'doc, 'input>, ResourceError> {
        let index = self
            .font_index
            .binary_search_by_key(&id, |entry| entry.0)
            .map_err(|_| ResourceError::MissingFont(id))?;
        let ordinal = self
            .font_index
            .get(index)
            .map(|entry| entry.1)
            .ok_or(ResourceError::MissingFont(id))?;
        self.fonts
            .get(ordinal)
            .ok_or(ResourceError::MissingFont(id))
    }

    pub fn color(&self, index: u32) -> Result<[u16; 3], ResourceError> {
        match index {
            0 => Ok([0; 3]),
            1 => Ok([u16::MAX; 3]),
            2 if self.color_table.is_none() => Ok([u16::MAX; 3]),
            3 if self.color_table.is_none() => Ok([0; 3]),
            _ => index
                .checked_sub(2)
                .and_then(|offset| usize::try_from(offset).ok())
                .and_then(|offset| self.colors.get(offset))
                .map(Color::rgb)
                .ok_or(ResourceError::MissingColor(index)),
        }
    }
}

impl<'input> Document<'input> {
    /// Read document-level resource tables; extraction errors never alter syntax or source.
    pub fn resources(&self) -> Result<Resources<'_, 'input>, ResourceError> {
        let mut font_table = None;
        let mut color_table = None;
        let owner = self.framing().has_encoded_document().then_some(0);
        for (index, record) in self.records().iter().enumerate() {
            let RecordKind::Property { tag, .. } = record.kind() else {
                continue;
            };
            let table = match *tag {
                FONT_TABLE => &mut font_table,
                COLOR_TABLE => &mut color_table,
                _ => continue,
            };
            if record.parent() != owner {
                return Err(ResourceError::UnsupportedTable {
                    tag: *tag,
                    record: index,
                });
            }
            if table.replace(record).is_some() {
                return Err(ResourceError::DuplicateTable {
                    tag: *tag,
                    record: index,
                });
            }
        }
        let font_layout = font_table
            .map(|record| validate_fonts(self, record))
            .transpose()?;
        let color_count = color_table
            .map(|record| validate_colors(self, record))
            .transpose()?;
        let mut resources = Resources {
            font_table,
            color_table,
            platform: font_layout.map(|layout| layout.0),
            fonts: Vec::new(),
            font_index: Vec::new(),
            colors: Vec::new(),
        };
        // Both complete table bodies are checked before any descriptor allocation.
        if let (Some(record), Some((_, count))) = (font_table, font_layout) {
            resources
                .fonts
                .try_reserve_exact(usize::from(count))
                .map_err(|_| ResourceError::Allocation)?;
            resources
                .font_index
                .try_reserve_exact(usize::from(count))
                .map_err(|_| ResourceError::Allocation)?;
            let mut cursor = table_cursor(self, record)?;
            let invalid = || ResourceError::InvalidTable {
                tag: FONT_TABLE,
                record: record.index,
            };
            cursor.take(4).map_err(|_| invalid())?;
            for ordinal in 0..usize::from(count) {
                let start = cursor.offset;
                let id = cursor.u16().map_err(|_| invalid())?;
                let charset = cursor.u16().map_err(|_| invalid())?;
                let length = usize::from(cursor.u16().map_err(|_| invalid())?);
                let name = cursor.take(length).map_err(|_| invalid())?;
                resources.fonts.push(Font {
                    id,
                    charset,
                    name,
                    ordinal,
                    range: start..cursor.offset,
                    record,
                });
                resources.font_index.push((id, ordinal));
            }
            resources.font_index.sort_unstable_by_key(|entry| entry.0);
        }
        if let (Some(record), Some(count)) = (color_table, color_count) {
            resources
                .colors
                .try_reserve_exact(usize::from(count))
                .map_err(|_| ResourceError::Allocation)?;
            let mut cursor = table_cursor(self, record)?;
            let invalid = || ResourceError::InvalidTable {
                tag: COLOR_TABLE,
                record: record.index,
            };
            cursor.take(2).map_err(|_| invalid())?;
            for ordinal in 0..u32::from(count) {
                let start = cursor.offset;
                let rgb = [
                    cursor.u16().map_err(|_| invalid())?,
                    cursor.u16().map_err(|_| invalid())?,
                    cursor.u16().map_err(|_| invalid())?,
                ];
                let index = ordinal.checked_add(2).ok_or_else(invalid)?;
                resources.colors.push(Color {
                    index,
                    rgb,
                    range: start..cursor.offset,
                    record,
                });
            }
        }
        Ok(resources)
    }
}

fn validate_fonts(source: &Document<'_>, record: &Record) -> Result<(u16, u16), ResourceError> {
    let mut cursor = table_cursor(source, record)?;
    let invalid = || ResourceError::InvalidTable {
        tag: FONT_TABLE,
        record: record.index,
    };
    let platform = cursor.u16().map_err(|_| invalid())?;
    let count = cursor.u16().map_err(|_| invalid())?;
    if platform > 1 {
        return Err(ResourceError::UnsupportedPlatform {
            record: record.index,
            value: platform,
        });
    }
    let minimum = usize::from(count).checked_mul(6).ok_or_else(invalid)?;
    if cursor
        .source
        .len()
        .checked_sub(cursor.offset)
        .filter(|remaining| *remaining >= minimum)
        .is_none()
    {
        return Err(invalid());
    }
    let mut ids = [0_u64; 1024];
    for ordinal in 0..usize::from(count) {
        let id = cursor.u16().map_err(|_| invalid())?;
        cursor.u16().map_err(|_| invalid())?;
        let length = usize::from(cursor.u16().map_err(|_| invalid())?);
        cursor.take(length).map_err(|_| invalid())?;
        let word = ids.get_mut(usize::from(id) / 64).ok_or_else(invalid)?;
        let mask = 1_u64 << (id % 64);
        if *word & mask != 0 {
            return Err(ResourceError::DuplicateFontId {
                record: record.index,
                id,
                ordinal,
            });
        }
        *word |= mask;
    }
    if cursor.offset != cursor.source.len() {
        return Err(invalid());
    }
    Ok((platform, count))
}

fn validate_colors(source: &Document<'_>, record: &Record) -> Result<u16, ResourceError> {
    let mut cursor = table_cursor(source, record)?;
    let invalid = || ResourceError::InvalidTable {
        tag: COLOR_TABLE,
        record: record.index,
    };
    let count = cursor.u16().map_err(|_| invalid())?;
    let remaining = usize::from(count).checked_mul(6).ok_or_else(invalid)?;
    cursor.take(remaining).map_err(|_| invalid())?;
    if cursor.offset != cursor.source.len() {
        return Err(invalid());
    }
    Ok(count)
}

fn table_cursor<'input>(
    source: &Document<'input>,
    record: &Record,
) -> Result<Cursor<'input>, ResourceError> {
    source.payload(record)?;
    let RecordKind::Property { payload, .. } = record.kind() else {
        return Err(Error::InvalidProperty.into());
    };
    Ok(Cursor {
        source: source
            .as_bytes()
            .get(..payload.end)
            .ok_or(Error::InvalidPropertyRange)?,
        offset: payload.start,
    })
}
