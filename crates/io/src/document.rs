use crate::cdxml::{ElementalGraph, MappingError};
use bondline_model::{Document, PageSize};
use serde::Serialize;
use std::{borrow::Cow, collections::HashSet, mem::size_of, sync::Arc};

#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error(transparent)]
    Mapping(#[from] MappingError),
    #[error("this document has no original CDXML source")]
    NoCdxmlSource,
    #[error("the model changed; exporting those changes to CDXML is not yet supported")]
    UnsupportedEdit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceInfo {
    pub format: &'static str,
    pub byte_length: usize,
    pub modified: bool,
}

#[derive(Clone, PartialEq)]
struct OriginalCdxml {
    bytes: Arc<[u8]>,
    baseline: Arc<Document>,
    fully_mapped: bool,
}

/// Owned application snapshot: derived model plus the complete original interchange data.
/// Explicit Poster extents are read; other printer/style settings remain opaque.
#[derive(Clone, PartialEq, Default)]
pub struct StoredDocument {
    model: Arc<Document>,
    original: Option<OriginalCdxml>,
}

impl std::fmt::Debug for StoredDocument {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StoredDocument")
            .field("page_count", &self.model.pages().len())
            .field("source", &self.source_info())
            .finish()
    }
}

impl StoredDocument {
    pub fn from_cdxml(bytes: Arc<[u8]>, preview_size: PageSize) -> Result<Self, DocumentError> {
        let graph = ElementalGraph::parse(&bytes)?;
        let model = Arc::new(graph.preview_document(preview_size)?);
        drop(graph);
        // A producer name is insufficient: every source byte must match our complete profile.
        // shortcut: current canonical profile only; broaden with complete mappings and migrations.
        let fully_mapped = crate::cdxml::write_document(&model)
            .is_ok_and(|encoded| encoded.as_slice() == bytes.as_ref());
        Ok(Self {
            model: Arc::clone(&model),
            original: Some(OriginalCdxml {
                bytes,
                baseline: model,
                fully_mapped,
            }),
        })
    }

    pub fn model(&self) -> &Document {
        &self.model
    }

    pub fn model_snapshot(&self) -> Arc<Document> {
        Arc::clone(&self.model)
    }

    /// Every authored field is represented by the native elemental codec.
    /// Noncanonical or extended imports retain source but cannot be regenerated safely.
    pub fn is_fully_mapped(&self) -> bool {
        self.original
            .as_ref()
            .is_none_or(|source| source.fully_mapped)
    }

    /// Approximate retained storage, deduplicating source/model/fragment allocations.
    /// Includes baselines and reserved vectors; excludes Arc/allocator headers and caches.
    pub fn retained_bytes<'a>(documents: impl IntoIterator<Item = &'a Self>) -> usize {
        let mut snapshots = HashSet::new();
        let mut sources = HashSet::new();
        let mut models = Vec::new();
        let mut bytes = 0_usize;
        for document in documents {
            if !snapshots.insert(std::ptr::from_ref(document)) {
                continue;
            }
            bytes = bytes.saturating_add(size_of::<Self>());
            models.push(document.model.as_ref());
            if let Some(original) = &document.original {
                if sources.insert(original.bytes.as_ptr()) {
                    bytes = bytes.saturating_add(original.bytes.len());
                }
                models.push(original.baseline.as_ref());
            }
        }
        bytes.saturating_add(Document::retained_bytes(models))
    }

    /// Detached model edits preserve source data, but cannot be exported until synchronized.
    pub fn model_mut(&mut self) -> &mut Document {
        Arc::make_mut(&mut self.model)
    }

    pub fn source_info(&self) -> Option<SourceInfo> {
        self.original.as_ref().map(|original| SourceInfo {
            format: "cdxml",
            byte_length: original.bytes.len(),
            modified: !Arc::ptr_eq(&self.model, &original.baseline)
                && self.model != original.baseline,
        })
    }

    /// Byte-exact passthrough only. Never silently export stale XML after a model edit.
    pub fn export_cdxml(&self) -> Result<&[u8], DocumentError> {
        let original = self.original.as_ref().ok_or(DocumentError::NoCdxmlSource)?;
        if !Arc::ptr_eq(&self.model, &original.baseline) && self.model != original.baseline {
            return Err(DocumentError::UnsupportedEdit);
        }
        Ok(&original.bytes)
    }

    /// Borrow exact unchanged source, or encode a new native elemental document.
    /// Only a proven complete profile may encode edits; unknown source is never discarded.
    pub fn to_cdxml(&self) -> Result<Cow<'_, [u8]>, DocumentError> {
        match self.export_cdxml() {
            Ok(bytes) => Ok(Cow::Borrowed(bytes)),
            Err(DocumentError::NoCdxmlSource | DocumentError::UnsupportedEdit)
                if self.is_fully_mapped() =>
            {
                Ok(Cow::Owned(crate::cdxml::write_document(&self.model)?))
            }
            Err(error) => Err(error),
        }
    }
}
