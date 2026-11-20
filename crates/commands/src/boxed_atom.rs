//! A complete qualified child label changes numeric data, literal text and bounds together.
use crate::{AtomSetLabel, Error};
use bondline_io::StoredDocument;
use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId, TextStyle};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// An explicit authoring choice, independent of unresolved imported-file defaults.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AtomLabelStyle {
    #[schemars(length(min = 1, max = 256))]
    pub font_family: String,
    #[schemars(extend("exclusiveMinimum" = 0.0, "maximum" = 100000.0))]
    pub size_pt: f64,
    pub face: u16,
    #[schemars(extend("items" = {"type":"number", "format":"double", "minimum":0.0, "maximum":1.0}))]
    pub rgb: [f64; 3],
}

impl AtomLabelStyle {
    pub(super) fn native_preset() -> Self {
        Self {
            font_family: "Roboto".into(),
            size_pt: 12.0,
            face: 0,
            rgb: [0.0; 3],
        }
    }

    fn checked(self) -> Result<TextStyle, Error> {
        TextStyle::new(self.font_family, self.size_pt, self.face, self.rgb)
            .map_err(bondline_model::Error::from)
            .map_err(Error::from)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AtomCreateLabel {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 1))]
    pub fragment_id: u32,
    #[schemars(range(min = 1))]
    pub atom_id: u32,
    #[schemars(range(min = 1, max = 118))]
    pub element: u8,
    pub explicit_hydrogens: u16,
    pub style: AtomLabelStyle,
}

pub(super) fn supports_edit(document: &StoredDocument) -> bool {
    document.has_boxed_atom_labels()
        || document.has_connected_atom_labels()
        || document.has_mixed_atom_labels()
        || document.has_flagged_atom_labels()
        || document.has_label_defaults_projection()
        || document.has_cdx_label_edit_profile()
}

pub(super) fn editable_atom(
    document: &StoredDocument,
    page: u32,
    fragment: u32,
    atom: u32,
) -> Result<(&Fragment, &Atom), Error> {
    if !supports_edit(document) {
        return Err(Error::UnsupportedAtomLabel(
            "complete box-positioned literal label source is required",
        ));
    }
    let (fragment, atom) = atom_at(document, page, fragment, atom)?;
    bondline_render::validate_atom_label(atom)?;
    Ok((fragment, atom))
}

pub(super) fn creation_atom(
    document: &StoredDocument,
    page: u32,
    fragment: u32,
    atom: u32,
) -> Result<(&Fragment, &Atom), Error> {
    if !document.has_mixed_atom_labels()
        && !document.has_flagged_atom_labels()
        && !document.has_label_defaults_projection()
    {
        return Err(Error::UnsupportedAtomLabel(
            "complete imported skeletal label source is required for creation",
        ));
    }
    let (fragment, atom) = atom_at(document, page, fragment, atom)?;
    if atom.authored_label.is_some()
        || !bondline_render::hidden_carbon_atoms(fragment, &Default::default())?.contains(&atom.id)
    {
        return Err(Error::UnsupportedAtomLabel(
            "creation requires an explicitly hidden carbon without an existing child label",
        ));
    }
    Ok((fragment, atom))
}

pub(super) fn atom_at(
    document: &StoredDocument,
    page: u32,
    fragment: u32,
    atom: u32,
) -> Result<(&Fragment, &Atom), Error> {
    let page = PageId::new(page).ok_or(Error::InvalidPageId)?;
    let fragment = FragmentId::new(fragment).ok_or(Error::InvalidFragmentId)?;
    let atom = AtomId::new(atom).ok_or(Error::InvalidAtomLabel("atom id must be positive"))?;
    let fragment = document
        .model()
        .pages()
        .iter()
        .find(|value| value.id() == page)
        .ok_or(bondline_model::Error::MissingPage(page.get()))?
        .fragments()
        .iter()
        .find(|value| value.id() == fragment)
        .ok_or(bondline_model::Error::MissingFragment(fragment.get()))?;
    let atom = fragment
        .atoms()
        .iter()
        .find(|value| value.id == atom)
        .ok_or(Error::MissingAtom {
            fragment_id: fragment.id().get(),
            atom_id: atom.get(),
        })?;
    Ok((fragment, atom))
}

pub(super) fn set_label(
    document: &StoredDocument,
    params: AtomSetLabel,
) -> Result<StoredDocument, Error> {
    let (fragment, atom) =
        editable_atom(document, params.page_id, params.fragment_id, params.atom_id)?;
    let element = Element::new(params.element).ok_or(Error::InvalidAtomLabel(
        "element must be in the 1..118 catalog",
    ))?;
    if element.get() == 1 && params.explicit_hydrogens != 0 {
        return Err(Error::InvalidAtomLabel(
            "a physical hydrogen atom cannot carry inline attached hydrogens",
        ));
    }
    if document.has_cdx_label_edit_profile() {
        return Ok(document.with_cdx_atom_label(
            PageId::new(params.page_id).ok_or(Error::InvalidPageId)?,
            fragment.id(),
            atom.id,
            element,
            params.explicit_hydrogens,
        )?);
    }
    if atom.element == element && atom.explicit_hydrogens == Some(params.explicit_hydrogens) {
        return Ok(document.clone());
    }
    let mut target = atom.clone();
    target.element = element;
    target.explicit_hydrogens = Some(params.explicit_hydrogens);
    target.authored_label = Some(Arc::new(bondline_render::position_atom_label(&target)?));
    finish_label(document, params.page_id, fragment, target)
}

pub(super) fn create_label(
    document: &StoredDocument,
    params: AtomCreateLabel,
) -> Result<StoredDocument, Error> {
    let (fragment, atom) =
        creation_atom(document, params.page_id, params.fragment_id, params.atom_id)?;
    let element = Element::new(params.element).ok_or(Error::InvalidAtomLabel(
        "element must be in the 1..118 catalog",
    ))?;
    if element.get() == 1 && params.explicit_hydrogens != 0 {
        return Err(Error::InvalidAtomLabel(
            "a physical hydrogen atom cannot carry inline attached hydrogens",
        ));
    }
    let mut target = atom.clone();
    target.element = element;
    target.explicit_hydrogens = Some(params.explicit_hydrogens);
    target.authored_label = Some(Arc::new(bondline_render::create_atom_label(
        &target,
        params.style.checked()?,
    )?));
    finish_label(document, params.page_id, fragment, target)
}

fn finish_label(
    document: &StoredDocument,
    page: u32,
    fragment: &Fragment,
    target: Atom,
) -> Result<StoredDocument, Error> {
    let mut atoms = fragment.atoms().to_vec();
    let slot = atoms
        .iter_mut()
        .find(|atom| atom.id == target.id)
        .ok_or(Error::MissingAtom {
            fragment_id: fragment.id().get(),
            atom_id: target.id.get(),
        })?;
    *slot = target;
    let replacement = Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())
        .map_err(bondline_model::Error::from)?;
    let mut candidate = document.clone();
    let page_id = PageId::new(page).ok_or(Error::InvalidPageId)?;
    candidate
        .model_mut()
        .replace_fragment(page_id, replacement)?;
    if !candidate.has_boxed_atom_labels()
        && !candidate.has_connected_atom_labels()
        && !candidate.has_mixed_atom_labels()
        && !candidate.has_flagged_atom_labels()
        && !candidate.has_label_defaults_projection()
    {
        return Err(Error::UnsupportedAtomLabel(
            "the final candidate must retain complete literal label source coverage",
        ));
    }
    candidate.to_cdxml()?;
    let page = candidate
        .model()
        .pages()
        .iter()
        .find(|page| page.id() == page_id)
        .ok_or(Error::InvalidPageId)?;
    bondline_io::smiles::validate_authoring_page_with_settings(
        page,
        candidate.model().drawing_style(),
        candidate.model().drawing_settings(),
        &bondline_render::Cancellation::default(),
    )
    .map_err(|error| match error {
        bondline_io::smiles::LayoutError::Render(error) => Error::Render(error),
        other => Error::SmilesLayout(other),
    })?;
    Ok(candidate)
}
