//! Replace existing property payloads without rebuilding the original record tree.
use crate::{Document, Error, MAX_INPUT_BYTES, MAX_RECORDS, Record, write::EncodedLength};

struct Replacement<'record, 'value> {
    record: &'record Record,
    original: &'record [u8],
    value: &'value [u8],
    encoding: Option<EncodedLength>,
}

impl Document<'_> {
    /// Replace one existing opaque payload using the same source-preserving rules as a batch.
    pub fn replace_property(&self, record: &Record, value: &[u8]) -> Result<Vec<u8>, Error> {
        self.replace_properties(&[(record, value)])
    }

    /// Replace existing opaque payloads in a single output buffer. Handles must
    /// belong to this exact parse and target distinct records, including no-ops.
    /// Input order does not affect the result. Unchanged records retain every byte,
    /// while changed values use the standard short or extended length encoding.
    pub fn replace_properties(&self, values: &[(&Record, &[u8])]) -> Result<Vec<u8>, Error> {
        if values.len() > MAX_RECORDS {
            return Err(Error::RecordLimit);
        }
        let mut replacements = Vec::new();
        replacements
            .try_reserve_exact(values.len())
            .map_err(|_| Error::Allocation)?;
        for &(record, value) in values {
            replacements.push(Replacement {
                record,
                original: self.payload(record)?,
                value,
                encoding: None,
            });
        }
        replacements.sort_unstable_by_key(|edit| edit.record.index);
        let (mut removed, mut added) = (0_usize, 0_usize);
        let mut previous = None;
        for edit in &mut replacements {
            if previous.replace(edit.record.index) == Some(edit.record.index) {
                return Err(Error::ConflictingEdits);
            }
            if edit.original == edit.value {
                continue;
            }
            let encoding = EncodedLength::new(edit.value.len())?;
            edit.encoding = Some(encoding);
            let range = &edit.record.range;
            let old_width = range
                .end
                .checked_sub(range.start)
                .ok_or(Error::InvalidPropertyRange)?;
            removed = removed
                .checked_add(old_width)
                .ok_or(Error::InvalidPropertyRange)?;
            added = added
                .checked_add(encoding.property_prefix_len())
                .and_then(|size| size.checked_add(edit.value.len()))
                .ok_or(Error::InputTooLarge)?;
        }
        let output_size = self
            .source
            .len()
            .checked_sub(removed)
            .ok_or(Error::InvalidPropertyRange)?
            .checked_add(added)
            .filter(|&size| size <= MAX_INPUT_BYTES)
            .ok_or(Error::InputTooLarge)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(output_size)
            .map_err(|_| Error::Allocation)?;
        let mut cursor = 0;
        for edit in replacements {
            let Some(encoding) = edit.encoding else {
                continue;
            };
            let range = &edit.record.range;
            let tag_end = range
                .start
                .checked_add(2)
                .ok_or(Error::InvalidPropertyRange)?;
            output.extend_from_slice(
                self.source
                    .get(cursor..tag_end)
                    .ok_or(Error::InvalidPropertyRange)?,
            );
            encoding.append(&mut output);
            output.extend_from_slice(edit.value);
            cursor = range.end;
        }
        output.extend_from_slice(
            self.source
                .get(cursor..)
                .ok_or(Error::InvalidPropertyRange)?,
        );
        Ok(output)
    }
}
