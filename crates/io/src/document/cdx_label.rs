//! Synchronize one existing atom label and certify the entire edited CDX.
use super::{DocumentError, OriginalSource, StoredDocument};
use crate::cdx::{MappingError, label_edit::LabelFields};
use bondline_cdx::Document;
use bondline_model::{AtomId, Element, Fragment, FragmentId, PageId};
use std::sync::Arc;

impl StoredDocument {
    /// The current preview owns the fields needed to edit this atom's boxed label.
    pub fn can_edit_cdx_atom_label(&self, atom: AtomId) -> bool {
        self.has_cdx_atom_label_preview()
            && self
                .export_cdx()
                .ok()
                .and_then(|bytes| Document::parse(bytes).ok())
                .is_some_and(|source| LabelFields::find(&source, atom).is_some())
    }

    /// Edit one qualified label's element, explicit H count, text and bounds together.
    /// An omitted Element remains absent on carbon and is inserted for another element.
    pub fn with_cdx_atom_label(
        &self,
        page_id: PageId,
        fragment_id: FragmentId,
        atom_id: AtomId,
        element: Element,
        explicit_hydrogens: u16,
    ) -> Result<Self, DocumentError> {
        if !matches!(self.original, Some(OriginalSource::Cdx { .. })) {
            return Err(DocumentError::NoCdxSource);
        }
        if !self.has_cdx_atom_label_preview() {
            return Err(DocumentError::UnsupportedCdxEdit);
        }
        let page = self
            .model
            .pages()
            .iter()
            .find(|page| page.id() == page_id)
            .ok_or(bondline_model::Error::MissingPage(page_id.get()))
            .map_err(MappingError::from)?;
        let fragment = page
            .fragments()
            .iter()
            .find(|fragment| fragment.id() == fragment_id)
            .ok_or(bondline_model::Error::MissingFragment(fragment_id.get()))
            .map_err(MappingError::from)?;
        let (index, atom) = fragment
            .atoms()
            .iter()
            .enumerate()
            .find(|(_, atom)| atom.id == atom_id)
            .ok_or(DocumentError::UnsupportedCdxLabel(
                "an atom in the specified fragment",
            ))?;
        let source = Document::parse(self.export_cdx()?).map_err(MappingError::from)?;
        let fields =
            LabelFields::find(&source, atom_id).ok_or(DocumentError::UnsupportedCdxLabel(
                "an explicit H count and existing boxed Text and style records",
            ))?;
        if element.get() == 1 && explicit_hydrogens != 0 {
            return Err(DocumentError::UnsupportedCdxLabel(
                "zero attached hydrogens on a physical hydrogen atom",
            ));
        }
        if atom.element == element && atom.explicit_hydrogens == Some(explicit_hydrogens) {
            return Ok(self.clone());
        }

        let mut target = atom.clone();
        target.element = element;
        target.explicit_hydrogens = Some(explicit_hydrogens);
        let layout_error = || DocumentError::UnsupportedCdxLabel("a representable boxed placement");
        let label = bondline_render::position_atom_label(&target).map_err(|_| layout_error())?;
        let coordinates = label.bounding_box().ok_or_else(layout_error)?.map(fixed);
        let [Some(left), Some(top), Some(right), Some(bottom)] = coordinates else {
            return Err(layout_error());
        };
        let raw_bounds = [top, left, bottom, right].map(i32::to_le_bytes);
        let [run] = label.runs() else {
            return Err(layout_error());
        };
        let output = fields.replace(
            &source,
            element,
            explicit_hydrogens,
            run.text(),
            raw_bounds.as_flattened(),
        )?;
        let bounds = [left, top, right, bottom].map(|value| f64::from(value) / FIXED_SCALE);
        target.authored_label = Some(Arc::new(
            label
                .with_bounding_box(Some(bounds))
                .map_err(|_| layout_error())?,
        ));

        let mut atoms = fragment.atoms().to_vec();
        *atoms.get_mut(index).ok_or_else(layout_error)? = target;
        let fragment = Fragment::new(fragment_id, atoms, fragment.bonds().to_vec())
            .map_err(MappingError::from)?;
        let mut expected = self.model.as_ref().clone();
        expected
            .replace_fragment(page_id, fragment)
            .map_err(MappingError::from)?;
        self.finish_cdx_edit(
            output,
            expected,
            page.size(),
            DocumentError::UnsupportedCdxLabel(
                "complete agreement, page bounds and bond clearance after the edit",
            ),
        )
    }
}

const FIXED_SCALE: f64 = 65_536.0;

fn fixed(value: f64) -> Option<i32> {
    let scaled = (value * FIXED_SCALE).round_ties_even();
    (f64::from(i32::MIN)..=f64::from(i32::MAX))
        .contains(&scaled)
        .then_some(scaled as i32)
}
