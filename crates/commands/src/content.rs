use crate::Error;
use bondline_io::{SourceInfo, StoredDocument, sdf::SdfDocument, smiles::SmilesFileDocument};
use serde::Serialize;
use std::collections::HashSet;

/// Complete authored session data; collections never substitute a drawing projection.
#[derive(Clone, Debug, PartialEq)]
pub enum StoredContent {
    Drawing(StoredDocument),
    Sdf {
        original: SdfDocument,
        current: SdfDocument,
    },
    SmilesFile(SmilesFileDocument),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecordSummary {
    pub id: u32,
    pub mol_version: &'static str,
    pub byte_length: usize,
    pub mol_byte_length: usize,
    pub field_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CollectionSummary {
    pub format: &'static str,
    pub byte_length: usize,
    pub record_count: usize,
    pub records: Vec<RecordSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SmilesFileSummary {
    pub format: &'static str,
    pub byte_length: usize,
    pub record_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected_records: Option<Vec<u32>>,
}

impl From<StoredDocument> for StoredContent {
    fn from(document: StoredDocument) -> Self {
        Self::Drawing(document)
    }
}

impl StoredContent {
    pub fn from_smiles_file(document: SmilesFileDocument) -> Self {
        Self::SmilesFile(document)
    }

    pub fn from_sdf(document: SdfDocument) -> Self {
        Self::Sdf {
            original: document.clone(),
            current: document,
        }
    }

    pub fn drawing(&self) -> Result<&StoredDocument, Error> {
        match self {
            Self::Drawing(document) => Ok(document),
            Self::Sdf { .. } => Err(Error::WrongContent {
                required: "drawing",
                actual: "sdf",
            }),
            Self::SmilesFile(_) => Err(Error::WrongContent {
                required: "drawing",
                actual: "smiles",
            }),
        }
    }

    pub fn sdf(&self) -> Result<&SdfDocument, Error> {
        match self {
            Self::Sdf { current, .. } => Ok(current),
            Self::Drawing(_) => Err(Error::WrongContent {
                required: "sdf",
                actual: "drawing",
            }),
            Self::SmilesFile(_) => Err(Error::WrongContent {
                required: "sdf",
                actual: "smiles",
            }),
        }
    }

    pub fn smiles_file(&self) -> Result<&SmilesFileDocument, Error> {
        match self {
            Self::SmilesFile(document) => Ok(document),
            Self::Drawing(document)
                if document
                    .source_info()
                    .is_some_and(|source| source.format == "smiles") =>
            {
                Ok(document.smiles_file()?)
            }
            Self::Drawing(_) => Err(Error::WrongContent {
                required: "smiles",
                actual: "drawing",
            }),
            Self::Sdf { .. } => Err(Error::WrongContent {
                required: "smiles",
                actual: "sdf",
            }),
        }
    }

    pub fn smiles_summary(&self) -> Option<SmilesFileSummary> {
        let (document, projected_records) = match self {
            Self::SmilesFile(document) => (document, None),
            Self::Drawing(stored) => (
                stored.smiles_file().ok()?,
                Some(
                    stored
                        .smiles_records()?
                        .iter()
                        .map(|record| record.record_id().get())
                        .collect(),
                ),
            ),
            Self::Sdf { .. } => return None,
        };
        Some(SmilesFileSummary {
            format: "smiles",
            byte_length: document.as_bytes().len(),
            record_count: document.records().len(),
            projected_records,
        })
    }

    pub fn source_info(&self) -> Option<SourceInfo> {
        match self {
            Self::Drawing(document) => document.source_info(),
            Self::Sdf { original, current } => Some(SourceInfo {
                format: "sdf",
                byte_length: original.as_bytes().len(),
                modified: current != original,
            }),
            Self::SmilesFile(document) => Some(SourceInfo {
                format: "smiles",
                byte_length: document.as_bytes().len(),
                modified: false,
            }),
        }
    }

    pub fn collection(&self) -> Option<CollectionSummary> {
        let Self::Sdf { current, .. } = self else {
            return None;
        };
        Some(CollectionSummary {
            format: "sdf",
            byte_length: current.as_bytes().len(),
            record_count: current.records().len(),
            records: current
                .records()
                .iter()
                .map(|record| RecordSummary {
                    id: record.id().get(),
                    mol_version: match record.mol_version() {
                        bondline_io::mol::Version::V2000 => "v2000",
                        bondline_io::mol::Version::V3000 => "v3000",
                    },
                    byte_length: record.range().len(),
                    mol_byte_length: record.mol_range().len(),
                    field_count: record.field_count(),
                })
                .collect(),
        })
    }

    /// Count each outer inline snapshot once and deduplicate all nested payloads.
    /// Inner wrapper sizes are already part of Self; temporary counters are excluded.
    pub fn retained_bytes<'a>(contents: impl IntoIterator<Item = &'a Self>) -> usize {
        let mut snapshots = HashSet::new();
        let mut drawings = Vec::new();
        let mut collections = Vec::new();
        let mut smiles_files = Vec::new();
        let mut bytes = 0_usize;
        for content in contents {
            if !snapshots.insert(std::ptr::from_ref(content)) {
                continue;
            }
            bytes = bytes.saturating_add(size_of::<Self>());
            match content {
                Self::Drawing(document) => drawings.push(document),
                Self::Sdf { original, current } => {
                    collections.push(original);
                    collections.push(current);
                }
                Self::SmilesFile(document) => smiles_files.push(document),
            }
        }
        let drawing_inline = drawings.len().saturating_mul(size_of::<StoredDocument>());
        let sdf_inline = collections.len().saturating_mul(size_of::<SdfDocument>());
        let smiles_inline = smiles_files
            .len()
            .saturating_mul(size_of::<SmilesFileDocument>());
        bytes
            .saturating_add(
                StoredDocument::retained_bytes_with_smiles_files(drawings, smiles_files)
                    .saturating_sub(drawing_inline.saturating_add(smiles_inline)),
            )
            .saturating_add(SdfDocument::retained_bytes(collections).saturating_sub(sdf_inline))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bondline_io::{
        StoredDocument,
        sdf::{RecordId, RecordInfo, SdfDocument},
    };

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");

    #[test]
    fn immutable_smiles_files_share_source_and_index_without_undercharging_history() -> TestResult {
        use bondline_io::smiles::file::RecordInfo as SmilesRecordInfo;
        let source: &[u8] = b"CC PRIVATE\r\nO second";
        let first = StoredContent::from_smiles_file(SmilesFileDocument::parse(source.into())?);
        let expected =
            size_of::<StoredContent>() + source.len() + 2 * size_of::<SmilesRecordInfo>();
        assert_eq!(StoredContent::retained_bytes([&first, &first]), expected);
        let shared = first.clone();
        assert_eq!(
            StoredContent::retained_bytes([&first, &shared]),
            expected + size_of::<StoredContent>()
        );
        let separate = StoredContent::from_smiles_file(SmilesFileDocument::parse(source.into())?);
        assert_eq!(first, separate);
        assert_eq!(
            StoredContent::retained_bytes([&first, &separate]),
            2 * expected
        );
        let other = StoredContent::from_smiles_file(SmilesFileDocument::parse(
            b"CC DIFFERENT\r\nO second".as_slice().into(),
        )?);
        assert_ne!(first, other, "metadata remains part of whole-file identity");
        assert_eq!(first.smiles_file()?.as_bytes(), source);
        assert_eq!(
            first.drawing().err().ok_or("not drawing")?.info().code,
            "unsupported_content"
        );
        assert_eq!(
            first.sdf().err().ok_or("not SDF")?.info().code,
            "unsupported_content"
        );
        assert!(first.collection().is_none());
        assert!(!format!("{first:?}").contains("PRIVATE"));
        assert_eq!(first.smiles_summary().ok_or("summary")?.record_count, 2);
        Ok(())
    }

    #[test]
    fn collection_ownership_has_no_dummy_drawing_and_accounts_inline_storage_once() -> TestResult {
        let document = SdfDocument::parse(SOURCE.into())?;
        let content = StoredContent::from_sdf(document);
        let expected = size_of::<StoredContent>() + 690 + 2 * size_of::<RecordInfo>();
        assert_eq!(StoredContent::retained_bytes([&content]), expected);
        assert_eq!(
            StoredContent::retained_bytes([&content, &content]),
            expected
        );
        let shared = content.clone();
        assert_eq!(
            StoredContent::retained_bytes([&content, &shared]),
            expected + size_of::<StoredContent>()
        );
        assert_eq!(
            content.drawing().err().ok_or("not a drawing")?.info().code,
            "unsupported_content"
        );
        assert_eq!(content.sdf()?.as_bytes(), SOURCE);
        let separate = StoredContent::from_sdf(SdfDocument::parse(SOURCE.into())?);
        assert_eq!(content, separate);
        assert_eq!(
            StoredContent::retained_bytes([&content, &separate]),
            2 * expected
        );
        let drawing = StoredContent::from(StoredDocument::default());
        assert!(drawing.collection().is_none());
        assert_eq!(
            drawing.sdf().err().ok_or("not SDF")?.info().code,
            "unsupported_content"
        );
        assert_eq!(
            StoredContent::retained_bytes([&drawing]),
            size_of::<StoredContent>() + StoredDocument::retained_bytes([drawing.drawing()?])
                - size_of::<StoredDocument>()
        );
        Ok(())
    }

    #[test]
    fn original_current_identity_is_source_sensitive_and_shared_current_is_counted_once()
    -> TestResult {
        let original = SdfDocument::parse(SOURCE.into())?;
        let other_original =
            original.with_field_value(RecordId::new(1).ok_or("record")?, 0, &["FIRST"])?;
        let current = original.with_field_value(RecordId::new(1).ok_or("record")?, 5, &["TWO"])?;
        let first = StoredContent::Sdf {
            original: original.clone(),
            current: current.clone(),
        };
        let second = StoredContent::Sdf {
            original: other_original,
            current,
        };
        assert_ne!(
            first, second,
            "identical current files must not erase distinct source originals"
        );
        assert_eq!(first.collection(), second.collection());
        assert_eq!(
            first.source_info(),
            Some(bondline_io::SourceInfo {
                format: "sdf",
                byte_length: 690,
                modified: true
            })
        );
        let expected = 2 * size_of::<StoredContent>() + 3 * 690 + 3 * 2 * size_of::<RecordInfo>();
        assert_eq!(StoredContent::retained_bytes([&first, &second]), expected);
        let text = format!("{first:?}");
        for value in ["<DUP>", "alpha", "numbered", "M  END"] {
            assert!(!text.contains(value));
        }
        assert_eq!(original.as_bytes(), SOURCE);
        Ok(())
    }
}
