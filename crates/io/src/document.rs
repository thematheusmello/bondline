use crate::cdxml::{ElementalGraph, MappingError};
use crate::mol::{MolError, MolFile, MolTransform, write_v2000, write_v2000_with_header};
use bondline_model::{Document, Fragment, FragmentId, PageId, PageSize};
use serde::Serialize;
use std::{borrow::Cow, collections::HashSet, mem::size_of, sync::Arc};

#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error(transparent)]
    Mapping(#[from] MappingError),
    #[error(transparent)]
    Mol(#[from] MolError),
    #[error("this document has no original CDXML source")]
    NoCdxmlSource,
    #[error("this document has no original MOL source")]
    NoMolSource,
    #[error("the model changed; exporting those changes to CDXML is not yet supported")]
    UnsupportedEdit,
    #[error("the model changed; source-preserving MOL regeneration is not supported")]
    UnsupportedMolEdit,
    #[error("saving {from} source as {to} would discard source metadata")]
    CrossFormatSource {
        from: &'static str,
        to: &'static str,
    },
    #[error("MOL cannot preserve this authored layout: {0}")]
    UnsupportedMolLayout(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceInfo {
    pub format: &'static str,
    pub byte_length: usize,
    pub modified: bool,
}

#[derive(Clone, PartialEq)]
enum OriginalSource {
    Cdxml {
        bytes: Arc<[u8]>,
        baseline: Arc<Document>,
        fully_mapped: bool,
    },
    Mol {
        bytes: Arc<[u8]>,
        baseline: Arc<Document>,
        fully_mapped: bool,
        transform: MolTransform,
        headers: Arc<[String; 3]>,
    },
}

impl OriginalSource {
    fn bytes(&self) -> &Arc<[u8]> {
        match self {
            Self::Cdxml { bytes, .. } | Self::Mol { bytes, .. } => bytes,
        }
    }
    fn baseline(&self) -> &Arc<Document> {
        match self {
            Self::Cdxml { baseline, .. } | Self::Mol { baseline, .. } => baseline,
        }
    }
    fn fully_mapped(&self) -> bool {
        match self {
            Self::Cdxml { fully_mapped, .. } | Self::Mol { fully_mapped, .. } => *fully_mapped,
        }
    }
    fn format(&self) -> &'static str {
        match self {
            Self::Cdxml { .. } => "cdxml",
            Self::Mol { .. } => "mol",
        }
    }
}

/// Owned application snapshot: derived model plus the complete original interchange data.
/// Explicit Poster extents are read; other printer/style settings remain opaque.
#[derive(Clone, PartialEq, Default)]
pub struct StoredDocument {
    model: Arc<Document>,
    original: Option<OriginalSource>,
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
            original: Some(OriginalSource::Cdxml {
                bytes,
                baseline: model,
                fully_mapped,
            }),
        })
    }

    /// Own a complete V2000 source and its strict elemental projection. The caller
    /// supplies drawing scale/origin; neither physical units nor a paper size are inferred.
    /// Unknown or unsupported MOL features fail before the caller can replace its session.
    pub fn from_mol(
        bytes: Arc<[u8]>,
        preview_size: PageSize,
        transform: MolTransform,
    ) -> Result<Self, DocumentError> {
        let parsed = MolFile::parse(&bytes)?;
        let fragment = parsed.to_fragment(transform)?;
        let fully_mapped = mol_regenerable(&parsed, &fragment, transform);
        let headers = Arc::new(parsed.header().map(str::to_owned));
        let mut model = Document::default();
        let page = PageId::new(1).ok_or(MolError::CountLimit)?;
        model
            .set_page_size(page, preview_size)
            .map_err(MolError::Model)?;
        if !fragment.atoms().is_empty() || !fragment.bonds().is_empty() {
            model
                .append_fragments(page, vec![fragment])
                .map_err(MolError::Model)?;
        }
        drop(parsed);
        let model = Arc::new(model);
        Ok(Self {
            model: Arc::clone(&model),
            original: Some(OriginalSource::Mol {
                bytes,
                baseline: model,
                fully_mapped,
                transform,
                headers,
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
            .is_none_or(OriginalSource::fully_mapped)
    }

    /// Imported MOL drawing frame. Saving that source always keeps this frame.
    pub fn mol_transform(&self) -> Option<MolTransform> {
        match self.original.as_ref() {
            Some(OriginalSource::Mol { transform, .. }) => Some(*transform),
            _ => None,
        }
    }

    /// Approximate retained storage, deduplicating source/model/fragment allocations.
    /// Includes baselines and reserved vectors; excludes Arc/allocator headers and caches.
    pub fn retained_bytes<'a>(documents: impl IntoIterator<Item = &'a Self>) -> usize {
        let mut snapshots = HashSet::new();
        let mut sources = HashSet::new();
        let mut header_arrays = HashSet::new();
        let mut models = Vec::new();
        let mut bytes = 0_usize;
        for document in documents {
            if !snapshots.insert(std::ptr::from_ref(document)) {
                continue;
            }
            bytes = bytes.saturating_add(size_of::<Self>());
            models.push(document.model.as_ref());
            if let Some(original) = &document.original {
                if sources.insert(original.bytes().as_ptr()) {
                    bytes = bytes.saturating_add(original.bytes().len());
                }
                if let OriginalSource::Mol { headers, .. } = original
                    && header_arrays.insert(Arc::as_ptr(headers))
                {
                    bytes = bytes.saturating_add(size_of::<[String; 3]>());
                    for header in headers.iter() {
                        bytes = bytes.saturating_add(header.capacity());
                    }
                }
                models.push(original.baseline().as_ref());
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
            format: original.format(),
            byte_length: original.bytes().len(),
            modified: !Arc::ptr_eq(&self.model, original.baseline())
                && self.model != *original.baseline(),
        })
    }

    /// Byte-exact passthrough only. Never silently export stale XML after a model edit.
    pub fn export_cdxml(&self) -> Result<&[u8], DocumentError> {
        let Some(OriginalSource::Cdxml {
            bytes, baseline, ..
        }) = self.original.as_ref()
        else {
            return Err(DocumentError::NoCdxmlSource);
        };
        if !Arc::ptr_eq(&self.model, baseline) && self.model != *baseline {
            return Err(DocumentError::UnsupportedEdit);
        }
        Ok(bytes)
    }

    /// Borrow exact unchanged source, or encode a new native elemental document.
    /// Only a proven complete profile may encode edits; unknown source is never discarded.
    pub fn to_cdxml(&self) -> Result<Cow<'_, [u8]>, DocumentError> {
        if matches!(self.original, Some(OriginalSource::Mol { .. })) {
            return Err(DocumentError::CrossFormatSource {
                from: "mol",
                to: "cdxml",
            });
        }
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

    /// Byte-exact passthrough only; no model change can export stale original MOL.
    pub fn export_mol(&self) -> Result<&[u8], DocumentError> {
        let Some(OriginalSource::Mol {
            bytes, baseline, ..
        }) = self.original.as_ref()
        else {
            return Err(DocumentError::NoMolSource);
        };
        if !Arc::ptr_eq(&self.model, baseline) && self.model != *baseline {
            return Err(DocumentError::UnsupportedMolEdit);
        }
        Ok(bytes)
    }

    /// Preserve an unchanged source exactly, or regenerate a completely mapped MOL
    /// with its retained headers/frame. `native_frame` applies only to source-free or
    /// proven native-profile CDXML output. Layout/partition loss is explicitly refused.
    pub fn to_mol(&self, native_frame: MolTransform) -> Result<Cow<'_, [u8]>, DocumentError> {
        if let Ok(bytes) = self.export_mol() {
            return Ok(Cow::Borrowed(bytes));
        }
        let (frame, headers, expected_page) = match self.original.as_ref() {
            Some(OriginalSource::Mol {
                fully_mapped: false,
                ..
            }) => return Err(DocumentError::UnsupportedMolEdit),
            Some(OriginalSource::Mol {
                transform,
                headers,
                baseline,
                ..
            }) => {
                let page = baseline
                    .pages()
                    .first()
                    .ok_or(DocumentError::UnsupportedMolLayout("missing original page"))?;
                (*transform, Some(headers), (page.id(), page.size()))
            }
            Some(OriginalSource::Cdxml {
                fully_mapped: false,
                ..
            }) => {
                return Err(DocumentError::CrossFormatSource {
                    from: "cdxml",
                    to: "mol",
                });
            }
            Some(OriginalSource::Cdxml {
                fully_mapped: true, ..
            })
            | None => (
                native_frame,
                None,
                (PageId::new(1).ok_or(MolError::CountLimit)?, PageSize::A4),
            ),
        };
        if self.model.pages().len() != 1 {
            return Err(DocumentError::UnsupportedMolLayout("more than one page"));
        }
        let page = self
            .model
            .pages()
            .first()
            .ok_or(DocumentError::UnsupportedMolLayout("missing page"))?;
        if (page.id(), page.size()) != expected_page {
            return Err(DocumentError::UnsupportedMolLayout(
                "page identity or dimensions changed",
            ));
        }
        if page.fragments().len() > 1 {
            return Err(DocumentError::UnsupportedMolLayout(
                "more than one authored fragment",
            ));
        }
        let empty = Fragment::new(
            FragmentId::new(1).ok_or(MolError::CountLimit)?,
            vec![],
            vec![],
        )
        .map_err(MolError::Graph)?;
        let fragment = page.fragments().first().map_or(&empty, Arc::as_ref);
        Ok(Cow::Owned(match headers {
            Some(headers) => {
                write_v2000_with_header(fragment, headers.each_ref().map(String::as_str), frame)?
            }
            None => write_v2000(fragment, "", frame)?,
        }))
    }
}

fn mol_regenerable(raw: &MolFile<'_>, fragment: &Fragment, frame: MolTransform) -> bool {
    let Ok(bytes) = write_v2000_with_header(fragment, raw.header(), frame) else {
        return false;
    };
    let Ok(regenerated) = MolFile::parse(&bytes) else {
        return false;
    };
    let Ok(mapped) = regenerated.to_fragment(frame) else {
        return false;
    };
    if raw.atoms().len() != regenerated.atoms().len()
        || fragment.bonds() != mapped.bonds()
        || fragment.atoms().len() != mapped.atoms().len()
    {
        return false;
    }
    for (before, after) in raw.atoms().iter().zip(regenerated.atoms()) {
        for (first, second) in [
            (before.x, after.x),
            (before.y, after.y),
            (before.z, after.z),
        ] {
            if (first - second).abs()
                > 0.00005 + 16.0 * f64::EPSILON * first.abs().max(second.abs())
            {
                return false;
            }
        }
    }
    fragment
        .atoms()
        .iter()
        .zip(mapped.atoms())
        .all(|(before, after)| {
            before.id == after.id
                && before.element == after.element
                && before.isotope == after.isotope
                && before.charge == after.charge
                && before.radical == after.radical
                && before.explicit_hydrogens == after.explicit_hydrogens
        })
}
