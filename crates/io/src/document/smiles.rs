//! Complete SMILES file authority is separate from the generated drawing projection.

use super::{DocumentError, OriginalSource, StoredDocument};
use crate::smiles::{
    ProjectionError, RecordSelection, SmilesFileDocument, SmilesRecordMapping, projection,
    validate_authoring_page,
};
use bondline_model::{Document, Fragment, PageId, PageSize, Point2};
use bondline_render::Cancellation;
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub(super) struct SmilesBacking {
    pub(super) original: SmilesFileDocument,
    pub(super) current: SmilesFileDocument,
    pub(super) baseline: Arc<Document>,
    pub(super) mapped: Arc<Document>,
    pub(super) original_maps: Arc<[SmilesRecordMapping]>,
    pub(super) maps: Arc<[SmilesRecordMapping]>,
}

impl StoredDocument {
    /// Derive only the explicit record selection while owning every original source byte.
    /// The resulting drawing is not a completely mapped native-format document.
    pub fn from_smiles_records(
        source: SmilesFileDocument,
        selection: RecordSelection,
        length: f64,
        origin: Point2,
        page: PageSize,
        cancel: &Cancellation,
    ) -> Result<Self, DocumentError> {
        let (model, maps) = projection::project(&source, selection, length, origin, page, cancel)?;
        let model = Arc::new(model);
        let backing = SmilesBacking {
            original: source.clone(),
            current: source,
            baseline: Arc::clone(&model),
            mapped: Arc::clone(&model),
            original_maps: Arc::clone(&maps),
            maps,
        };
        Ok(Self {
            model,
            original: Some(OriginalSource::Smiles(Arc::new(backing))),
        })
    }

    /// Current complete file for source Save. Detached raw-model edits never grant this authority.
    pub fn smiles_file(&self) -> Result<&SmilesFileDocument, DocumentError> {
        Ok(&self.mapped_smiles()?.current)
    }

    /// Source-relative mappings are metadata; callers must check current projection authority.
    pub fn smiles_records(&self) -> Option<&[SmilesRecordMapping]> {
        match &self.original {
            Some(OriginalSource::Smiles(backing)) => Some(&backing.maps),
            _ => None,
        }
    }

    pub fn has_smiles_projection(&self) -> bool {
        self.mapped_smiles().is_ok()
    }

    fn mapped_smiles(&self) -> Result<&SmilesBacking, ProjectionError> {
        let Some(OriginalSource::Smiles(backing)) = &self.original else {
            return Err(ProjectionError::Unsupported(
                "document has no SMILES backing",
            ));
        };
        if !Arc::ptr_eq(&self.model, &backing.mapped) && self.model != backing.mapped {
            return Err(ProjectionError::Unsupported(
                "drawing differs from synchronized SMILES projection",
            ));
        }
        Ok(backing)
    }

    /// Change only one mapped fragment's literal H and integer bond orders, atomically with
    /// its current notation and all shifted mappings. Membership/geometry/metadata stay exact.
    pub fn with_smiles_fragment(
        &self,
        page_id: PageId,
        replacement: Fragment,
        cancel: &Cancellation,
    ) -> Result<Self, DocumentError> {
        self.replace_smiles_fragment(page_id, replacement, cancel)
            .map_err(Into::into)
    }

    fn replace_smiles_fragment(
        &self,
        page_id: PageId,
        replacement: Fragment,
        cancel: &Cancellation,
    ) -> Result<Self, ProjectionError> {
        projection::check(cancel)?;
        let backing = self.mapped_smiles()?;
        let page = self
            .model
            .pages()
            .iter()
            .find(|page| page.id() == page_id)
            .ok_or(bondline_model::Error::MissingPage(page_id.get()))?;
        let previous = page
            .fragments()
            .iter()
            .find(|fragment| fragment.id() == replacement.id())
            .ok_or(bondline_model::Error::MissingFragment(
                replacement.id().get(),
            ))?;
        let selected = backing
            .maps
            .iter()
            .find(|mapping| mapping.fragment_id() == replacement.id())
            .ok_or(ProjectionError::Unsupported(
                "fragment has no complete record mapping",
            ))?;
        if previous.atoms().len() != replacement.atoms().len()
            || previous.bonds().len() != replacement.bonds().len()
            || previous
                .atoms()
                .iter()
                .zip(replacement.atoms())
                .any(|(old, new)| {
                    old.id != new.id
                        || old.element != new.element
                        || old.position != new.position
                        || old.isotope != new.isotope
                        || old.charge != new.charge
                        || old.radical != new.radical
                })
            || previous
                .bonds()
                .iter()
                .zip(replacement.bonds())
                .any(|(old, new)| old.id != new.id || old.begin != new.begin || old.end != new.end)
        {
            return Err(ProjectionError::Unsupported(
                "fragment membership, order, geometry or metadata changed",
            ));
        }
        if previous.as_ref() == &replacement {
            return Ok(self.clone());
        }
        let written = crate::smiles::write::write_fragment_mapped(&replacement)?;
        let current =
            backing
                .current
                .with_record_notation(selected.record_id(), &written.bytes, cancel)?;
        let mut model = self.model.as_ref().clone();
        model.replace_fragment(page_id, replacement)?;
        let page = model
            .pages()
            .iter()
            .find(|page| page.id() == page_id)
            .ok_or(bondline_model::Error::MissingPage(page_id.get()))?;
        validate_authoring_page(page, cancel)?;
        let mut maps = Vec::with_capacity(backing.maps.len());
        for original in backing.maps.iter() {
            projection::check(cancel)?;
            let fragment = page
                .fragments()
                .iter()
                .find(|fragment| fragment.id() == original.fragment_id())
                .ok_or(ProjectionError::Invalid("mapped fragment disappeared"))?;
            let atom_ids = if original.record_id() == selected.record_id() {
                written.atom_ids.clone()
            } else {
                original.atoms().iter().map(|atom| atom.atom_id).collect()
            };
            maps.push(projection::mapping(
                &current,
                original.record_id(),
                fragment,
                &atom_ids,
                cancel,
            )?);
        }
        projection::check(cancel)?;
        let model = Arc::new(model);
        let next = SmilesBacking {
            original: backing.original.clone(),
            current,
            baseline: Arc::clone(&backing.baseline),
            mapped: Arc::clone(&model),
            original_maps: Arc::clone(&backing.original_maps),
            maps: maps.into(),
        };
        Ok(Self {
            model,
            original: Some(OriginalSource::Smiles(Arc::new(next))),
        })
    }
}
