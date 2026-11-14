use crate::cdxml::{ElementalGraph, MappingError};
use crate::mol::{
    MolError, MolFile, MolTransform, Version, v3000, write_v2000, write_v2000_with_header,
};
use bondline_model::{Document, Fragment, FragmentId, PageId, PageSize};
use serde::Serialize;
use std::{borrow::Cow, collections::HashSet, mem::size_of, sync::Arc};

#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error(transparent)]
    Mapping(#[from] MappingError),
    #[error(transparent)]
    CdxMapping(#[from] crate::cdx::MappingError),
    #[error(transparent)]
    Mol(#[from] MolError),
    #[error(transparent)]
    V3000(#[from] v3000::Error),
    #[error("this document has no original CDXML source")]
    NoCdxmlSource,
    #[error("this document has no original MOL source")]
    NoMolSource,
    #[error("this document has no original CDX source")]
    NoCdxSource,
    #[error("the model changed; exporting those changes to CDXML is not yet supported")]
    UnsupportedEdit,
    #[error("the model changed; source-preserving MOL regeneration is not supported")]
    UnsupportedMolEdit,
    #[error("the model changed; source-preserving CDX regeneration is not supported")]
    UnsupportedCdxEdit,
    #[error("saving {from} source as {to} would discard source metadata")]
    CrossFormatSource {
        from: &'static str,
        to: &'static str,
    },
    #[error("MOL cannot preserve this authored layout: {0}")]
    UnsupportedMolLayout(&'static str),
}

impl DocumentError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Mol(error) => error.code(),
            Self::V3000(error) => error.code(),
            Self::CdxMapping(error) => error.code(),
            Self::Mapping(error) => match error {
                MappingError::UnsupportedNodeType
                | MappingError::UnsupportedLabelOnlyAtom
                | MappingError::UnsupportedBondOrder
                | MappingError::UnsupportedLayout
                | MappingError::UnsupportedPageLayout => "unsupported_content",
                _ => "invalid_document",
            },
            Self::NoCdxmlSource
            | Self::NoMolSource
            | Self::NoCdxSource
            | Self::UnsupportedEdit
            | Self::UnsupportedMolEdit
            | Self::UnsupportedCdxEdit
            | Self::CrossFormatSource { .. }
            | Self::UnsupportedMolLayout(_) => "unsupported_content",
        }
    }
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
    Cdx {
        bytes: Arc<[u8]>,
        baseline: Arc<Document>,
    },
    Mol {
        version: Version,
        bytes: Arc<[u8]>,
        baseline: Arc<Document>,
        fully_mapped: bool,
        v3000_mapped: bool,
        transform: MolTransform,
        headers: Arc<[String; 3]>,
    },
}

impl OriginalSource {
    fn bytes(&self) -> &Arc<[u8]> {
        match self {
            Self::Cdxml { bytes, .. } | Self::Cdx { bytes, .. } | Self::Mol { bytes, .. } => bytes,
        }
    }
    fn baseline(&self) -> &Arc<Document> {
        match self {
            Self::Cdxml { baseline, .. }
            | Self::Cdx { baseline, .. }
            | Self::Mol { baseline, .. } => baseline,
        }
    }
    fn fully_mapped(&self) -> bool {
        match self {
            Self::Cdxml { fully_mapped, .. } | Self::Mol { fully_mapped, .. } => *fully_mapped,
            Self::Cdx { .. } => false,
        }
    }
    fn format(&self) -> &'static str {
        match self {
            Self::Cdxml { .. } => "cdxml",
            Self::Cdx { .. } => "cdx",
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
    /// Own the limited elemental view and all original binary content. The preview page
    /// is caller policy; source style, annotations and paper settings remain opaque.
    pub fn from_cdx(bytes: Arc<[u8]>, preview_size: PageSize) -> Result<Self, DocumentError> {
        let graph = crate::cdx::ElementalGraph::parse(&bytes)?;
        let model = Arc::new(graph.preview_document(preview_size)?);
        drop(graph);
        Ok(Self {
            model: Arc::clone(&model),
            original: Some(OriginalSource::Cdx {
                bytes,
                baseline: model,
            }),
        })
    }

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

    /// Validate a complete V2000/V3000 source and own its strict elemental projection. The caller
    /// supplies drawing scale/origin; neither physical units nor a paper size are inferred.
    /// Unknown or unsupported MOL features fail before the caller can replace its session.
    pub fn from_mol(
        bytes: Arc<[u8]>,
        preview_size: PageSize,
        transform: MolTransform,
    ) -> Result<Self, DocumentError> {
        let version = crate::mol::source_version(&bytes)?;
        let (fragment, fully_mapped, v3000_mapped, headers) = match version {
            Version::V2000 => {
                let parsed = MolFile::parse(&bytes)?;
                let fragment = parsed.to_fragment(transform)?;
                let mapped = mol_regenerable(&parsed, &fragment, transform);
                let v3000_mapped = mol_v3000_regenerable(
                    parsed.atoms().iter().map(|atom| (atom.x, atom.y, atom.z)),
                    parsed.header(),
                    &fragment,
                    transform,
                );
                let headers = Arc::new(parsed.header().map(str::to_owned));
                (fragment, mapped, v3000_mapped, headers)
            }
            Version::V3000 => {
                let parsed = v3000::MolFile::parse(&bytes)?;
                let fragment = parsed.to_fragment(transform)?;
                let mapped = mol_v3000_regenerable(
                    parsed.atoms().iter().map(|atom| (atom.x, atom.y, atom.z)),
                    parsed.header(),
                    &fragment,
                    transform,
                );
                let headers = Arc::new(parsed.header().map(str::to_owned));
                (fragment, mapped, mapped, headers)
            }
        };
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
        let model = Arc::new(model);
        Ok(Self {
            model: Arc::clone(&model),
            original: Some(OriginalSource::Mol {
                version,
                bytes,
                baseline: model,
                fully_mapped,
                v3000_mapped,
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

    pub fn mol_version(&self) -> Option<Version> {
        match self.original.as_ref() {
            Some(OriginalSource::Mol { version, .. }) => Some(*version),
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

    /// Only complete unchanged backing can be exported; never substitute its projection.
    pub fn export_cdx(&self) -> Result<&[u8], DocumentError> {
        let Some(OriginalSource::Cdx { bytes, baseline }) = self.original.as_ref() else {
            return Err(DocumentError::NoCdxSource);
        };
        if !Arc::ptr_eq(&self.model, baseline) && self.model != *baseline {
            return Err(DocumentError::UnsupportedCdxEdit);
        }
        Ok(bytes)
    }

    /// No model encoder exists yet. Preserve unchanged imports or explicitly refuse.
    pub fn to_cdx(&self) -> Result<Cow<'_, [u8]>, DocumentError> {
        self.export_cdx().map(Cow::Borrowed)
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
        if let Some(original) = self.original.as_ref()
            && !matches!(original, OriginalSource::Cdxml { .. })
        {
            return Err(DocumentError::CrossFormatSource {
                from: original.format(),
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
        self.to_mol_as(self.mol_version().unwrap_or(Version::V2000), native_frame)
    }

    /// Explicit encoding choice. Existing MOL keeps its captured headers/frame;
    /// unsupported version conversion must never publish a partial projection.
    pub fn to_mol_as(
        &self,
        version: Version,
        native_frame: MolTransform,
    ) -> Result<Cow<'_, [u8]>, DocumentError> {
        if self.mol_version() == Some(Version::V3000) && version == Version::V2000 {
            return Err(DocumentError::CrossFormatSource {
                from: "mol_v3000",
                to: "mol_v2000",
            });
        }
        if self.mol_version() == Some(version)
            && let Ok(bytes) = self.export_mol()
        {
            return Ok(Cow::Borrowed(bytes));
        }
        if version == Version::V3000 {
            match self.original.as_ref() {
                Some(OriginalSource::Mol {
                    version: Version::V2000,
                    fully_mapped,
                    v3000_mapped,
                    ..
                }) if !(*fully_mapped && *v3000_mapped) => {
                    return Err(DocumentError::CrossFormatSource {
                        from: "mol_v2000",
                        to: "mol_v3000",
                    });
                }
                Some(OriginalSource::Mol {
                    v3000_mapped: false,
                    ..
                }) => return Err(DocumentError::UnsupportedMolEdit),
                _ => {}
            }
        }
        let (frame, headers, expected_page) = match self.original.as_ref() {
            Some(OriginalSource::Cdx { .. }) => {
                return Err(DocumentError::CrossFormatSource {
                    from: "cdx",
                    to: "mol",
                });
            }
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
        Ok(Cow::Owned(match (version, headers) {
            (Version::V2000, Some(headers)) => {
                write_v2000_with_header(fragment, headers.each_ref().map(String::as_str), frame)?
            }
            (Version::V2000, None) => write_v2000(fragment, "", frame)?,
            (Version::V3000, Some(headers)) => v3000::write_v3000_with_header(
                fragment,
                headers.each_ref().map(String::as_str),
                frame,
            )?,
            (Version::V3000, None) => v3000::write_v3000(fragment, "", frame)?,
        }))
    }
}

fn mol_v3000_regenerable(
    coordinates: impl ExactSizeIterator<Item = (f64, f64, f64)>,
    headers: [&str; 3],
    fragment: &Fragment,
    frame: MolTransform,
) -> bool {
    let Ok(bytes) = v3000::write_v3000_with_header(fragment, headers, frame) else {
        return false;
    };
    let Ok(regenerated) = v3000::MolFile::parse(&bytes) else {
        return false;
    };
    let Ok(mapped) = regenerated.to_fragment(frame) else {
        return false;
    };
    if coordinates.len() != regenerated.atoms().len()
        || !same_mol_authored_fields(fragment, &mapped)
    {
        return false;
    }
    coordinates
        .zip(regenerated.atoms())
        .all(|((x, y, z), after)| {
            [(x, after.x), (y, after.y), (z, after.z)]
                .into_iter()
                .all(|(first, second)| {
                    (first - second).abs()
                        <= 16.0 * f64::EPSILON * first.abs().max(second.abs()).max(1.0)
                })
        })
}

fn same_mol_authored_fields(fragment: &Fragment, mapped: &Fragment) -> bool {
    fragment.bonds() == mapped.bonds()
        && fragment.atoms().len() == mapped.atoms().len()
        && fragment
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
    same_mol_authored_fields(fragment, &mapped)
}
