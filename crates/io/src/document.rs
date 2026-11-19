#[cfg(test)]
mod carbon_label_tests;
mod cdx_caption;
mod cdx_label;
mod smiles;

use crate::cdx::Preview as CdxPreview;
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
    #[error(transparent)]
    SmilesFile(#[from] crate::smiles::SmilesFileError),
    #[error(transparent)]
    SmilesProjection(#[from] crate::smiles::ProjectionError),
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
    #[error("CDX atom label editing requires {0}")]
    UnsupportedCdxLabel(&'static str),
    #[error("CDX caption editing requires {0}")]
    UnsupportedCdxCaption(&'static str),
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
            Self::SmilesFile(error) => error.code(),
            Self::SmilesProjection(error) => error.code(),
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
            | Self::UnsupportedCdxLabel(_)
            | Self::UnsupportedCdxCaption(_)
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum CdxmlProfile {
    Opaque,
    Canonical,
    Preserved,
    Caption,
    Boxed,
    Connected,
    Mixed,
    Flagged,
    LabelDefaults,
}

#[derive(Clone, PartialEq)]
struct CdxEdit {
    bytes: Arc<[u8]>,
    model: Arc<Document>,
}

#[derive(Clone, PartialEq)]
enum OriginalSource {
    Smiles(Arc<smiles::SmilesBacking>),
    Cdxml {
        bytes: Arc<[u8]>,
        baseline: Arc<Document>,
        profile: CdxmlProfile,
    },
    Cdx {
        bytes: Arc<[u8]>,
        baseline: Arc<Document>,
        preview: CdxPreview,
        edit: Option<CdxEdit>,
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
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Cdxml { bytes, .. } | Self::Cdx { bytes, .. } | Self::Mol { bytes, .. } => bytes,
            Self::Smiles(backing) => backing.original.as_bytes(),
        }
    }
    fn baseline(&self) -> &Arc<Document> {
        match self {
            Self::Cdxml { baseline, .. }
            | Self::Cdx { baseline, .. }
            | Self::Mol { baseline, .. } => baseline,
            Self::Smiles(backing) => &backing.baseline,
        }
    }
    fn current_cdx(&self) -> Option<(&[u8], &Arc<Document>, CdxPreview)> {
        let Self::Cdx {
            bytes,
            baseline,
            preview,
            edit,
        } = self
        else {
            return None;
        };
        let (bytes, model) = match edit {
            Some(edit) => (&edit.bytes, &edit.model),
            None => (bytes, baseline),
        };
        Some((bytes, model, *preview))
    }
    fn fully_mapped(&self, current: &Arc<Document>) -> bool {
        match self {
            Self::Cdxml {
                profile, baseline, ..
            } => match profile {
                CdxmlProfile::Canonical => true,
                CdxmlProfile::Opaque
                | CdxmlProfile::Caption
                | CdxmlProfile::Boxed
                | CdxmlProfile::Connected
                | CdxmlProfile::Mixed
                | CdxmlProfile::Flagged
                | CdxmlProfile::LabelDefaults => false,
                CdxmlProfile::Preserved => {
                    Arc::ptr_eq(current, baseline)
                        || crate::cdxml::supports_delta(baseline, current)
                }
            },
            Self::Mol { fully_mapped, .. } => *fully_mapped,
            Self::Cdx { .. } | Self::Smiles(_) => false,
        }
    }
    fn format(&self) -> &'static str {
        match self {
            Self::Cdxml { .. } => "cdxml",
            Self::Cdx { .. } => "cdx",
            Self::Mol { .. } => "mol",
            Self::Smiles(_) => "smiles",
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
    /// Own the binary source and supported view. Certified display reads explicit Poster
    /// dimensions, while the data-only elemental projection uses the caller's page.
    pub fn from_cdx(bytes: Arc<[u8]>, preview_size: PageSize) -> Result<Self, DocumentError> {
        let (model, preview) = crate::cdx::project_document(&bytes, preview_size)?;
        let model = Arc::new(model);
        Ok(Self {
            model: Arc::clone(&model),
            original: Some(OriginalSource::Cdx {
                bytes,
                baseline: model,
                preview,
                edit: None,
            }),
        })
    }

    pub fn from_cdxml(bytes: Arc<[u8]>, preview_size: PageSize) -> Result<Self, DocumentError> {
        let graph = ElementalGraph::parse(&bytes)?;
        let model = Arc::new(graph.preview_document(preview_size)?);
        drop(graph);
        // Producer text alone grants no authority; every source field needs a complete proof.
        let profile = if crate::cdxml::is_flagged(&bytes, &model) {
            CdxmlProfile::Flagged
        } else if crate::cdxml::write_document(&model)
            .is_ok_and(|encoded| encoded.as_slice() == bytes.as_ref())
        {
            CdxmlProfile::Canonical
        } else if crate::cdxml::is_preserved(&bytes, &model) {
            CdxmlProfile::Preserved
        } else if crate::cdxml::is_caption(&bytes, &model) {
            CdxmlProfile::Caption
        } else if crate::cdxml::is_boxed(&bytes, &model) {
            CdxmlProfile::Boxed
        } else if crate::cdxml::is_connected(&bytes, &model) {
            CdxmlProfile::Connected
        } else if crate::cdxml::is_mixed(&bytes, &model) {
            CdxmlProfile::Mixed
        } else if crate::cdxml::is_label_defaults(&bytes, &model) {
            CdxmlProfile::LabelDefaults
        } else {
            CdxmlProfile::Opaque
        };
        Ok(Self {
            model: Arc::clone(&model),
            original: Some(OriginalSource::Cdxml {
                bytes,
                baseline: model,
                profile,
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

    /// Complete chemical authoring/display capability, beyond raw data codec coverage.
    /// Authored child labels need layout/agreement proofs; opaque extensions stay guarded.
    pub fn is_fully_mapped(&self) -> bool {
        !self.model.has_authored_atom_labels()
            && self
                .original
                .as_ref()
                .is_none_or(|source| source.fully_mapped(&self.model))
    }

    /// Native plain-caption creation may be attempted without changing source authority.
    /// Recheck the candidate's representable IDs and final source/page before committing it.
    pub fn can_create_caption(&self) -> bool {
        matches!(
            self.original.as_ref(),
            None | Some(OriginalSource::Cdxml {
                profile: CdxmlProfile::Canonical,
                ..
            })
        ) && self.is_fully_mapped()
            && crate::cdxml::native_caption_id_base(&self.model).is_some()
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
        Self::retained_bytes_with_smiles_files(documents, std::iter::empty())
    }

    /// Include raw SMILES owners in the same source/index accountant as mapped drawings.
    /// External file wrapper sizes are included; nested wrappers already belong to backing.
    pub fn retained_bytes_with_smiles_files<'a>(
        documents: impl IntoIterator<Item = &'a Self>,
        raw_smiles_files: impl IntoIterator<Item = &'a crate::smiles::SmilesFileDocument>,
    ) -> usize {
        let mut snapshots = HashSet::new();
        let mut sources = HashSet::new();
        let mut header_arrays = HashSet::new();
        let mut backings = HashSet::new();
        let mut mapping_arrays = HashSet::new();
        let mut files = raw_smiles_files.into_iter().collect::<Vec<_>>();
        let mut nested_inline = 0_usize;
        let mut models = Vec::new();
        let mut bytes = 0_usize;
        for document in documents {
            if !snapshots.insert(std::ptr::from_ref(document)) {
                continue;
            }
            bytes = bytes.saturating_add(size_of::<Self>());
            models.push(document.model.as_ref());
            if let Some(original) = &document.original {
                if let OriginalSource::Smiles(backing) = original {
                    if backings.insert(Arc::as_ptr(backing)) {
                        bytes = bytes.saturating_add(size_of::<smiles::SmilesBacking>());
                        files.extend([&backing.original, &backing.current]);
                        nested_inline = nested_inline
                            .saturating_add(2 * size_of::<crate::smiles::SmilesFileDocument>());
                        models.push(backing.mapped.as_ref());
                        for maps in [&backing.original_maps, &backing.maps] {
                            if mapping_arrays.insert(maps.as_ptr()) {
                                bytes =
                                    bytes.saturating_add(maps.len().saturating_mul(size_of::<
                                        crate::smiles::SmilesRecordMapping,
                                    >(
                                    )));
                                for mapping in maps.iter() {
                                    bytes = bytes.saturating_add(mapping.reserved_bytes());
                                }
                            }
                        }
                    }
                } else if sources.insert(original.bytes().as_ptr()) {
                    bytes = bytes.saturating_add(original.bytes().len());
                }
                if let OriginalSource::Cdx {
                    edit: Some(edit), ..
                } = original
                {
                    if sources.insert(edit.bytes.as_ptr()) {
                        bytes = bytes.saturating_add(edit.bytes.len());
                    }
                    models.push(edit.model.as_ref());
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
        bytes
            .saturating_add(Document::retained_bytes(models))
            .saturating_add(
                crate::smiles::SmilesFileDocument::retained_bytes(files)
                    .saturating_sub(nested_inline),
            )
    }

    /// Detached model edits preserve source data, but cannot be exported until synchronized.
    pub fn model_mut(&mut self) -> &mut Document {
        Arc::make_mut(&mut self.model)
    }

    /// Source-qualified caption edits/display, without chemical authoring permission.
    pub fn has_caption_projection(&self) -> bool {
        matches!(self.original.as_ref(),Some(OriginalSource::Cdxml{profile:CdxmlProfile::Caption,baseline,..})
            if Arc::ptr_eq(&self.model,baseline) || crate::cdxml::supports_caption_delta(baseline,&self.model))
    }

    /// CDX caption proof for the exact source-certified model.
    pub fn has_cdx_caption_preview(&self) -> bool {
        self.has_cdx_preview(CdxPreview::Caption)
    }

    /// CDX atom-label display proof for the exact source-certified model.
    pub fn has_cdx_atom_label_preview(&self) -> bool {
        self.has_cdx_preview(CdxPreview::AtomLabels)
    }

    fn has_cdx_preview(&self, required: CdxPreview) -> bool {
        self.original
            .as_ref()
            .and_then(OriginalSource::current_cdx)
            .is_some_and(|(_, model, preview)| {
                preview == required && (Arc::ptr_eq(&self.model, model) || self.model == *model)
            })
    }

    /// Re-import a qualified edit and retain original and current snapshots together.
    fn finish_cdx_edit(
        &self,
        output: Vec<u8>,
        expected: Document,
        page_size: PageSize,
        mismatch: DocumentError,
    ) -> Result<Self, DocumentError> {
        let Some(OriginalSource::Cdx {
            bytes,
            baseline,
            preview,
            ..
        }) = &self.original
        else {
            return Err(DocumentError::NoCdxSource);
        };
        let (projected, profile) = crate::cdx::project_document(&output, page_size)?;
        if profile != *preview || projected != expected {
            return Err(mismatch);
        }
        let (model, edit) = if expected == **baseline && output == bytes.as_ref() {
            (Arc::clone(baseline), None)
        } else {
            let model = Arc::new(expected);
            let edit = CdxEdit {
                bytes: Arc::from(output),
                model: Arc::clone(&model),
            };
            (model, Some(edit))
        };
        Ok(Self {
            model,
            original: Some(OriginalSource::Cdx {
                bytes: Arc::clone(bytes),
                baseline: Arc::clone(baseline),
                preview: *preview,
                edit,
            }),
        })
    }

    /// Closed source/model coverage for the box-positioned literal label workflow.
    /// Full chemical authoring stays false; view/raster and final-edit guards still apply.
    pub fn has_boxed_atom_labels(&self) -> bool {
        match self.original.as_ref() {
            None => crate::cdxml::boxed_model_supported(&self.model),
            Some(OriginalSource::Cdxml {
                profile: CdxmlProfile::Canonical,
                baseline,
                ..
            }) => crate::cdxml::supports_boxed_delta(baseline, &self.model),
            Some(OriginalSource::Cdxml {
                profile: CdxmlProfile::Boxed,
                baseline,
                ..
            }) => {
                Arc::ptr_eq(&self.model, baseline)
                    || crate::cdxml::supports_boxed_delta(baseline, &self.model)
            }
            _ => false,
        }
    }

    /// Complete foreign connected-source coverage; native conventions remain separate.
    /// This grants only the synchronized existing-label workflow, never full authoring.
    pub fn has_connected_atom_labels(&self) -> bool {
        matches!(self.original.as_ref(), Some(OriginalSource::Cdxml {
            profile: CdxmlProfile::Connected, baseline, ..
        }) if Arc::ptr_eq(&self.model, baseline)
            || crate::cdxml::supports_connected_delta(baseline, &self.model))
    }

    /// Complete mixed-source coverage; every unlabelled atom is currently glyphless.
    /// Existing printed labels and point movement remain separate from full authoring.
    pub fn has_mixed_atom_labels(&self) -> bool {
        matches!(self.original.as_ref(), Some(OriginalSource::Cdxml {
            profile: CdxmlProfile::Mixed, baseline, ..
        }) if Arc::ptr_eq(&self.model, baseline)
            || crate::cdxml::supports_mixed_delta(baseline, &self.model))
    }

    /// Complete printed labels with retained flags and current hidden source atoms.
    /// New runs must resolve against original resources; no foreign defaults are inferred.
    pub fn has_flagged_atom_labels(&self) -> bool {
        matches!(self.original.as_ref(), Some(OriginalSource::Cdxml {
            profile: CdxmlProfile::Mixed | CdxmlProfile::Flagged, bytes, baseline,
        }) if crate::cdxml::supports_flagged_delta(bytes, baseline, &self.model))
    }

    /// Explicit runs beside immutable, source-checked Label* metadata.
    /// Defaults never supply missing run fields or grant full chemical authoring.
    pub fn has_label_defaults_projection(&self) -> bool {
        matches!(self.original.as_ref(), Some(OriginalSource::Cdxml {
            profile: CdxmlProfile::LabelDefaults, bytes, baseline,
        }) if Arc::ptr_eq(&self.model, baseline)
            || crate::cdxml::supports_label_defaults_delta(bytes, baseline, &self.model))
    }

    /// Source coverage to attempt display; scale, glyph and geometry checks still apply.
    pub fn has_display_projection(&self) -> bool {
        self.is_fully_mapped()
            || self.has_smiles_projection()
            || self.has_caption_projection()
            || self.has_cdx_caption_preview()
            || self.has_cdx_atom_label_preview()
            || self.has_boxed_atom_labels()
            || self.has_connected_atom_labels()
            || self.has_mixed_atom_labels()
            || self.has_flagged_atom_labels()
            || self.has_label_defaults_projection()
    }

    pub fn source_info(&self) -> Option<SourceInfo> {
        self.original.as_ref().map(|original| SourceInfo {
            format: original.format(),
            byte_length: original.bytes().len(),
            modified: (!Arc::ptr_eq(&self.model, original.baseline())
                && self.model != *original.baseline())
                || matches!(original, OriginalSource::Smiles(backing) if backing.current!=backing.original)
                || matches!(original, OriginalSource::Cdx { edit: Some(_), .. }),
        })
    }

    /// Borrow the exact CDX bytes certified for the current model.
    pub fn export_cdx(&self) -> Result<&[u8], DocumentError> {
        let (bytes, model, _) = self
            .original
            .as_ref()
            .and_then(OriginalSource::current_cdx)
            .ok_or(DocumentError::NoCdxSource)?;
        if !Arc::ptr_eq(&self.model, model) && self.model != *model {
            return Err(DocumentError::UnsupportedCdxEdit);
        }
        Ok(bytes)
    }

    /// Preserve original or source-edited CDX without transcoding other formats.
    pub fn to_cdx(&self) -> Result<Cow<'_, [u8]>, DocumentError> {
        if matches!(self.original.as_ref(), Some(OriginalSource::Smiles(_))) {
            return Err(DocumentError::CrossFormatSource {
                from: "smiles",
                to: "cdx",
            });
        }
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
            Err(DocumentError::NoCdxmlSource) => {
                Ok(Cow::Owned(crate::cdxml::write_document(&self.model)?))
            }
            Err(DocumentError::UnsupportedEdit) => match self.original.as_ref() {
                Some(OriginalSource::Cdxml {
                    profile: CdxmlProfile::Canonical,
                    ..
                }) => Ok(Cow::Owned(crate::cdxml::write_document(&self.model)?)),
                Some(OriginalSource::Cdxml {
                    profile: CdxmlProfile::Preserved,
                    bytes,
                    baseline,
                }) => Ok(Cow::Owned(crate::cdxml::write_preserved(
                    bytes,
                    baseline,
                    &self.model,
                )?)),
                Some(OriginalSource::Cdxml {
                    profile: CdxmlProfile::Caption,
                    bytes,
                    baseline,
                }) => Ok(Cow::Owned(crate::cdxml::write_caption_preserved(
                    bytes,
                    baseline,
                    &self.model,
                )?)),
                Some(OriginalSource::Cdxml {
                    profile: CdxmlProfile::Boxed,
                    bytes,
                    baseline,
                }) => Ok(Cow::Owned(crate::cdxml::write_boxed_preserved(
                    bytes,
                    baseline,
                    &self.model,
                )?)),
                Some(OriginalSource::Cdxml {
                    profile: CdxmlProfile::Connected,
                    bytes,
                    baseline,
                }) => Ok(Cow::Owned(crate::cdxml::write_connected_preserved(
                    bytes,
                    baseline,
                    &self.model,
                )?)),
                Some(OriginalSource::Cdxml {
                    profile: CdxmlProfile::Mixed,
                    bytes,
                    baseline,
                }) => Ok(Cow::Owned(crate::cdxml::write_mixed_preserved(
                    bytes,
                    baseline,
                    &self.model,
                )?)),
                Some(OriginalSource::Cdxml {
                    profile: CdxmlProfile::Flagged,
                    bytes,
                    baseline,
                }) => match crate::cdxml::write_flagged_preserved(bytes, baseline, &self.model) {
                    Ok(output) => Ok(Cow::Owned(output)),
                    Err(DocumentError::UnsupportedEdit)
                        if crate::cdxml::write_document(baseline)
                            .is_ok_and(|native| native.as_slice() == bytes.as_ref()) =>
                    {
                        // Preserve the prior exact-native data codec, never edit authority.
                        Ok(Cow::Owned(crate::cdxml::write_document(&self.model)?))
                    }
                    Err(error) => Err(error),
                },
                Some(OriginalSource::Cdxml {
                    profile: CdxmlProfile::LabelDefaults,
                    bytes,
                    baseline,
                }) => Ok(Cow::Owned(crate::cdxml::write_label_defaults_preserved(
                    bytes,
                    baseline,
                    &self.model,
                )?)),
                _ => Err(DocumentError::UnsupportedEdit),
            },
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
        if self.model.has_authored_atom_labels() {
            return Err(DocumentError::UnsupportedMolLayout(
                "authored atom text cannot be represented",
            ));
        }
        if !self.model.drawing_settings().is_empty() {
            return Err(DocumentError::UnsupportedMolLayout(
                "drawing settings cannot be represented",
            ));
        }
        if self
            .model
            .pages()
            .iter()
            .any(|page| !page.texts().is_empty())
        {
            return Err(DocumentError::UnsupportedMolLayout(
                "caption text cannot be represented",
            ));
        }
        if !self.model.drawing_style().is_empty() {
            return Err(DocumentError::UnsupportedMolLayout(
                "drawing style cannot be represented",
            ));
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
            Some(OriginalSource::Smiles(_)) => {
                return Err(DocumentError::CrossFormatSource {
                    from: "smiles",
                    to: "mol",
                });
            }
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
                profile:
                    CdxmlProfile::Opaque
                    | CdxmlProfile::Preserved
                    | CdxmlProfile::Caption
                    | CdxmlProfile::Boxed
                    | CdxmlProfile::Connected
                    | CdxmlProfile::Mixed
                    | CdxmlProfile::Flagged
                    | CdxmlProfile::LabelDefaults,
                ..
            }) => {
                return Err(DocumentError::CrossFormatSource {
                    from: "cdxml",
                    to: "mol",
                });
            }
            Some(OriginalSource::Cdxml {
                profile: CdxmlProfile::Canonical,
                ..
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
                    && before.show_terminal_carbon_labels == after.show_terminal_carbon_labels
                    && before.show_non_terminal_carbon_labels
                        == after.show_non_terminal_carbon_labels
                    && before.authored_label == after.authored_label
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
