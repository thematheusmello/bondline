use crate::{Document, Error, MAX_DEPTH, MAX_INPUT_BYTES, MAX_RECORDS, PREFIX};
use std::fmt;

/// New canonical explicit-root syntax. This never regenerates an imported document
/// and does not validate property types, object IDs, containment or chemical meaning.
pub struct Writer {
    bytes: Vec<u8>,
    depth: usize,
    records: usize,
}

impl fmt::Debug for Writer {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output
            .debug_struct("Writer")
            .field("byte_length", &self.bytes.len())
            .field("open_objects", &self.depth)
            .field("records", &self.records)
            .finish()
    }
}

impl Writer {
    /// Start the standard header and explicit Document with unassigned ID0.
    pub fn new() -> Result<Self, Error> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(36).map_err(|_| Error::Allocation)?;
        bytes.extend_from_slice(PREFIX);
        bytes.extend_from_slice(&[0; 16]);
        bytes.extend_from_slice(&0x8000_u16.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        Ok(Self {
            bytes,
            depth: 1,
            records: 1,
        })
    }

    pub fn start_object(&mut self, tag: u16, id: u32) -> Result<(), Error> {
        if tag & 0x8000 == 0 {
            return Err(Error::InvalidTag);
        }
        let depth = self
            .depth
            .checked_add(1)
            .filter(|depth| *depth <= MAX_DEPTH)
            .ok_or(Error::DepthLimit)?;
        let records = self.reserve_record(6, depth)?;
        self.bytes.extend_from_slice(&tag.to_le_bytes());
        self.bytes.extend_from_slice(&id.to_le_bytes());
        self.depth = depth;
        self.records = records;
        Ok(())
    }

    pub fn property(&mut self, tag: u16, value: &[u8]) -> Result<(), Error> {
        if tag == 0 || tag & 0x8000 != 0 {
            return Err(Error::InvalidTag);
        }
        let length = u32::try_from(value.len()).map_err(|_| Error::InputTooLarge)?;
        let short = u16::try_from(value.len())
            .ok()
            .filter(|length| *length < u16::MAX);
        let prefix = if short.is_some() { 4_usize } else { 8 };
        let additional = prefix
            .checked_add(value.len())
            .ok_or(Error::InputTooLarge)?;
        let records = self.reserve_record(additional, self.depth)?;
        self.bytes.extend_from_slice(&tag.to_le_bytes());
        self.bytes
            .extend_from_slice(&short.unwrap_or(u16::MAX).to_le_bytes());
        if short.is_none() {
            self.bytes.extend_from_slice(&length.to_le_bytes());
        }
        self.bytes.extend_from_slice(value);
        self.records = records;
        Ok(())
    }

    pub fn end_object(&mut self) -> Result<(), Error> {
        let depth = self
            .depth
            .checked_sub(1)
            .filter(|depth| *depth > 0)
            .ok_or(Error::ObjectBalance)?;
        let records = self.reserve_record(2, depth)?;
        self.bytes.extend_from_slice(&[0, 0]);
        self.records = records;
        self.depth = depth;
        Ok(())
    }

    pub fn finish(mut self) -> Result<Vec<u8>, Error> {
        if self.depth != 1 {
            return Err(Error::ObjectBalance);
        }
        self.reserve_record(2, 0)?;
        self.bytes.extend_from_slice(&[0, 0]);
        Document::parse(&self.bytes)?;
        Ok(self.bytes)
    }

    fn reserve_record(&mut self, additional: usize, depth: usize) -> Result<usize, Error> {
        // Reserve every pending terminator in both budgets before changing the tree.
        self.bytes
            .len()
            .checked_add(additional)
            .and_then(|size| depth.checked_mul(2).and_then(|ends| size.checked_add(ends)))
            .filter(|size| *size <= MAX_INPUT_BYTES)
            .ok_or(Error::InputTooLarge)?;
        let records = self.records.checked_add(1).ok_or(Error::RecordLimit)?;
        records
            .checked_add(depth)
            .filter(|count| *count <= MAX_RECORDS)
            .ok_or(Error::RecordLimit)?;
        self.bytes
            .try_reserve(additional)
            .map_err(|_| Error::Allocation)?;
        Ok(records)
    }
}
