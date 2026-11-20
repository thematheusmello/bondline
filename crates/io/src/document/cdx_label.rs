//! Synchronize a chemical label with its Node and certify the entire edited CDX.
use super::{CdxPreview, DocumentError, OriginalSource, StoredDocument};
use crate::cdx::{MappingError, label_edit::LabelFields};
use bondline_cdx::Document;
use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId, TextStyle};
use std::sync::Arc;

impl StoredDocument {
    fn cdx_label_source(&self) -> Option<(&[u8], CdxPreview)> {
        self.cdx_source_for(|profile| {
            matches!(
                profile,
                CdxPreview::AtomLabels | CdxPreview::Mixed | CdxPreview::SkeletalLabels
            )
        })
    }

    /// The exact current CDX page supports targeting existing chemical labels.
    pub fn has_cdx_label_edit_profile(&self) -> bool {
        self.cdx_label_source().is_some()
    }

    /// The current preview owns the fields needed to edit this atom's boxed label.
    pub fn can_edit_cdx_atom_label(&self, atom: AtomId) -> bool {
        self.cdx_label_source()
            .and_then(|(bytes, _)| Document::parse(bytes).ok())
            .is_some_and(|source| LabelFields::find(&source, atom).is_some())
    }

    /// Target only a hidden carbon that can acquire a direct Text with the drawing's resources.
    pub fn can_create_cdx_atom_label(&self, id: AtomId) -> bool {
        let Some((bytes, _)) = self.cdx_skeletal_source() else {
            return false;
        };
        self.model
            .pages()
            .iter()
            .flat_map(|p| p.fragments())
            .flat_map(|f| f.atoms())
            .any(|atom| atom.id == id && hidden_label_target(atom))
            && Document::parse(bytes)
                .ok()
                .is_some_and(|doc| crate::cdx::has_label_creation_font(&doc))
    }

    /// Add the label and numeric atom properties to the same Node, then re-import the whole page.
    pub fn with_new_cdx_atom_label(
        &self,
        page: PageId,
        fragment: FragmentId,
        atom: AtomId,
        element: Element,
        hydrogens: u16,
        style: TextStyle,
    ) -> Result<Self, DocumentError> {
        self.change_cdx_atom_label(page, fragment, atom, element, hydrogens, Some(style))
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
        self.change_cdx_atom_label(
            page_id,
            fragment_id,
            atom_id,
            element,
            explicit_hydrogens,
            None,
        )
    }

    fn change_cdx_atom_label(
        &self,
        page_id: PageId,
        fragment_id: FragmentId,
        atom_id: AtomId,
        element: Element,
        explicit_hydrogens: u16,
        creation: Option<TextStyle>,
    ) -> Result<Self, DocumentError> {
        if !matches!(self.original, Some(OriginalSource::Cdx { .. })) {
            return Err(DocumentError::NoCdxSource);
        }
        let (bytes, profile) = if creation.is_some() {
            self.cdx_skeletal_source()
        } else {
            self.cdx_label_source()
        }
        .ok_or(DocumentError::UnsupportedCdxEdit)?;
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
        if creation.is_some() && (element.get() == 6 || !hidden_label_target(atom)) {
            return Err(DocumentError::UnsupportedCdxLabel(
                "a new heteroatom at a qualified hidden carbon",
            ));
        }
        let source = Document::parse(bytes).map_err(MappingError::from)?;
        let fields = if creation.is_some() {
            None
        } else {
            Some(
                LabelFields::find(&source, atom_id).ok_or(DocumentError::UnsupportedCdxLabel(
                    "an explicit H count and existing boxed Text and style records",
                ))?,
            )
        };
        if element.get() == 1 && explicit_hydrogens != 0 {
            return Err(DocumentError::UnsupportedCdxLabel(
                "zero attached hydrogens on a physical hydrogen atom",
            ));
        }
        if fields.is_some()
            && atom.element == element
            && atom.explicit_hydrogens == Some(explicit_hydrogens)
        {
            return Ok(self.clone());
        }

        let mut target = atom.clone();
        target.element = element;
        target.explicit_hydrogens = Some(explicit_hydrogens);
        let layout_error = || DocumentError::UnsupportedCdxLabel("a representable boxed placement");
        let label = match creation {
            Some(style) => bondline_render::create_atom_label(&target, style),
            None => bondline_render::position_atom_label(&target),
        }
        .map_err(|_| layout_error())?;
        let coordinates = label.bounding_box().ok_or_else(layout_error)?.map(fixed);
        let [Some(left), Some(top), Some(right), Some(bottom)] = coordinates else {
            return Err(layout_error());
        };
        let raw_bounds = [top, left, bottom, right].map(i32::to_le_bytes);
        let [run] = label.runs() else {
            return Err(layout_error());
        };
        let (output, profile) = if let Some(fields) = fields {
            (
                fields.replace(
                    &source,
                    element,
                    explicit_hydrogens,
                    run.text(),
                    raw_bounds.as_flattened(),
                    profile == CdxPreview::SkeletalLabels,
                )?,
                profile,
            )
        } else {
            let remaining_hidden = page
                .fragments()
                .iter()
                .flat_map(|f| f.atoms())
                .any(|a| a.id != atom_id && a.authored_label.is_none());
            (
                crate::cdx::label_edit::insert_label(
                    &source,
                    atom_id,
                    element,
                    explicit_hydrogens,
                    run,
                    raw_bounds.as_flattened(),
                )?,
                if remaining_hidden {
                    CdxPreview::SkeletalLabels
                } else {
                    CdxPreview::AtomLabels
                },
            )
        };
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
            profile,
            DocumentError::UnsupportedCdxLabel(
                "complete agreement, page bounds and bond clearance after the edit",
            ),
        )
    }
}

fn hidden_label_target(atom: &Atom) -> bool {
    atom.element.get() == 6
        && atom.authored_label.is_none()
        && atom.show_terminal_carbon_labels.is_none()
        && atom.show_non_terminal_carbon_labels.is_none()
}

const FIXED_SCALE: f64 = 65_536.0;

fn fixed(value: f64) -> Option<i32> {
    let scaled = (value * FIXED_SCALE).round_ties_even();
    (f64::from(i32::MIN)..=f64::from(i32::MAX))
        .contains(&scaled)
        .then_some(scaled as i32)
}
