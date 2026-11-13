use crate::cdxml::{ElementalGraph, MappingError};
use bondline_model::{Document, PageSize};
use serde::Serialize;
use std::sync::Arc;

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
}

/// Owned application snapshot: derived model plus the complete original interchange data.
/// Preview dimensions are explicit; source paper/style settings are not interpreted here.
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
        Ok(Self {
            model: Arc::clone(&model),
            original: Some(OriginalCdxml {
                bytes,
                baseline: model,
            }),
        })
    }

    pub fn model(&self) -> &Document {
        &self.model
    }

    pub fn model_snapshot(&self) -> Arc<Document> {
        Arc::clone(&self.model)
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
}
