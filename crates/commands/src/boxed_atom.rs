//! A complete qualified child label changes numeric data, literal text and bounds together.
use crate::{AtomSetLabel, Error};
use bondline_io::StoredDocument;
use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId};
use std::sync::Arc;

pub(super) fn editable_atom(
    document: &StoredDocument,
    page: u32,
    fragment: u32,
    atom: u32,
) -> Result<(&Fragment, &Atom), Error> {
    if !document.has_boxed_atom_labels() {
        return Err(Error::UnsupportedAtomLabel(
            "complete box-positioned literal label source is required",
        ));
    }
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
    bondline_render::validate_atom_label(atom)?;
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
    if atom.element == element && atom.explicit_hydrogens == Some(params.explicit_hydrogens) {
        return Ok(document.clone());
    }
    let mut atoms = fragment.atoms().to_vec();
    let target = atoms
        .iter_mut()
        .find(|value| value.id == atom.id)
        .ok_or(Error::MissingAtom {
            fragment_id: fragment.id().get(),
            atom_id: atom.id.get(),
        })?;
    target.element = element;
    target.explicit_hydrogens = Some(params.explicit_hydrogens);
    target.authored_label = Some(Arc::new(bondline_render::position_atom_label(target)?));
    let replacement = Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())
        .map_err(bondline_model::Error::from)?;
    let mut candidate = document.clone();
    let page_id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
    candidate
        .model_mut()
        .replace_fragment(page_id, replacement)?;
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
