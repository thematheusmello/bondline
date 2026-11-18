//! Edit properties without rebuilding the original record tree.
use crate::{
    Document, Error, MAX_INPUT_BYTES, MAX_RECORDS, Record, RecordKind, write::EncodedLength,
};

/// An absent property to insert immediately before an owned property in the same object.
#[derive(Clone, Copy, Debug)]
pub struct PropertyInsertion<'record, 'value> {
    pub before: &'record Record,
    pub tag: u16,
    pub value: &'value [u8],
}

enum Operation<'input> {
    Insert(u16),
    Replace(&'input [u8]),
}

struct PropertyEdit<'record, 'value, 'input> {
    record: &'record Record,
    operation: Operation<'input>,
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
        self.edit_properties(values, &[])
    }

    /// Insert absent properties alongside replacements in one bounded splice.
    /// Insertions inherit the exact containing object of their property anchor.
    /// An owner may receive each tag once, and only if the tag is absent.
    /// Insertions at one anchor use ascending tag order, before any replacement of that anchor.
    pub fn edit_properties(
        &self,
        values: &[(&Record, &[u8])],
        insertions: &[PropertyInsertion<'_, '_>],
    ) -> Result<Vec<u8>, Error> {
        let edit_count = values
            .len()
            .checked_add(insertions.len())
            .filter(|&count| count <= MAX_RECORDS)
            .ok_or(Error::RecordLimit)?;
        self.records
            .len()
            .checked_add(insertions.len())
            .filter(|&count| count <= MAX_RECORDS)
            .ok_or(Error::RecordLimit)?;
        let mut edits = Vec::new();
        edits
            .try_reserve_exact(edit_count)
            .map_err(|_| Error::Allocation)?;
        for &(record, value) in values {
            edits.push(PropertyEdit {
                record,
                operation: Operation::Replace(self.payload(record)?),
                value,
                encoding: None,
            });
        }
        if !insertions.is_empty() {
            self.validate_insertions(insertions)?;
            for &PropertyInsertion { before, tag, value } in insertions {
                edits.push(PropertyEdit {
                    record: before,
                    operation: Operation::Insert(tag),
                    value,
                    encoding: None,
                });
            }
        }
        edits.sort_unstable_by_key(|edit| {
            let order = match edit.operation {
                Operation::Insert(tag) => tag,
                Operation::Replace(_) => u16::MAX,
            };
            (edit.record.index, order)
        });
        let (mut removed, mut added) = (0_usize, 0_usize);
        let mut previous = None;
        for edit in &mut edits {
            if let Operation::Replace(original) = edit.operation {
                if previous.replace(edit.record.index) == Some(edit.record.index) {
                    return Err(Error::ConflictingEdits);
                }
                if original == edit.value {
                    continue;
                }
                let range = &edit.record.range;
                let old_width = range
                    .end
                    .checked_sub(range.start)
                    .ok_or(Error::InvalidPropertyRange)?;
                removed = removed
                    .checked_add(old_width)
                    .ok_or(Error::InvalidPropertyRange)?;
            }
            let encoding = EncodedLength::new(edit.value.len())?;
            edit.encoding = Some(encoding);
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
        for edit in edits {
            let Some(encoding) = edit.encoding else {
                continue;
            };
            let range = &edit.record.range;
            let (prefix_end, consumed) = match edit.operation {
                Operation::Insert(_) => (range.start, range.start),
                Operation::Replace(_) => (
                    range
                        .start
                        .checked_add(2)
                        .ok_or(Error::InvalidPropertyRange)?,
                    range.end,
                ),
            };
            output.extend_from_slice(
                self.source
                    .get(cursor..prefix_end)
                    .ok_or(Error::InvalidPropertyRange)?,
            );
            if let Operation::Insert(tag) = edit.operation {
                output.extend_from_slice(&tag.to_le_bytes());
            }
            encoding.append(&mut output);
            output.extend_from_slice(edit.value);
            cursor = consumed;
        }
        output.extend_from_slice(
            self.source
                .get(cursor..)
                .ok_or(Error::InvalidPropertyRange)?,
        );
        Ok(output)
    }

    fn validate_insertions(&self, insertions: &[PropertyInsertion<'_, '_>]) -> Result<(), Error> {
        let mut targets = Vec::new();
        targets
            .try_reserve_exact(insertions.len())
            .map_err(|_| Error::Allocation)?;
        for &PropertyInsertion { before, tag, .. } in insertions {
            self.payload(before)?;
            if tag == 0 || tag & 0x8000 != 0 {
                return Err(Error::InvalidTag);
            }
            targets.push((before.parent, tag));
        }
        targets.sort_unstable();
        if targets.windows(2).any(|pair| pair.first() == pair.last())
            || self.records.iter().any(|record| {
                matches!(record.kind, RecordKind::Property { tag, .. }
                    if targets.binary_search(&(record.parent, tag)).is_ok())
            })
        {
            return Err(Error::ConflictingEdits);
        }
        Ok(())
    }
}
