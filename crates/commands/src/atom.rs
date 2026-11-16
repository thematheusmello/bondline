use crate::Error;
use bondline_io::StoredDocument;
use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId, Radical};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Literal elemental authoring; neither valence inference nor neighboring H correction.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AtomSetLabel {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 1))]
    pub fragment_id: u32,
    #[schemars(range(min = 1))]
    pub atom_id: u32,
    #[schemars(range(min = 1, max = 118))]
    pub element: u8,
    #[schemars(range(min = 0, max = 65535))]
    pub explicit_hydrogens: u16,
}

pub(super) fn set_label(
    document: &StoredDocument,
    params: AtomSetLabel,
) -> Result<StoredDocument, Error> {
    let (fragment, target) =
        editable_atom(document, params.page_id, params.fragment_id, params.atom_id)?;
    let element = Element::new(params.element).ok_or(Error::InvalidAtomLabel(
        "element must be in the 1..118 catalog",
    ))?;
    if element.get() == 1 && params.explicit_hydrogens != 0 {
        return Err(Error::InvalidAtomLabel(
            "a physical hydrogen atom cannot carry inline attached hydrogens",
        ));
    }
    let restore_label = target.show_non_terminal_carbon_labels == Some(false);
    if target.element == element
        && target.explicit_hydrogens == Some(params.explicit_hydrogens)
        && !restore_label
    {
        return Ok(document.clone());
    }
    let mut atoms = fragment.atoms().to_vec();
    let target = atoms
        .iter_mut()
        .find(|atom| atom.id.get() == params.atom_id)
        .ok_or(Error::MissingAtom {
            fragment_id: params.fragment_id,
            atom_id: params.atom_id,
        })?;
    target.element = element;
    target.explicit_hydrogens = Some(params.explicit_hydrogens);
    if restore_label {
        target.show_non_terminal_carbon_labels = Some(true);
    }
    let replacement = Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())
        .map_err(bondline_model::Error::from)?;
    let mut candidate = document.clone();
    candidate.model_mut().replace_fragment(
        PageId::new(params.page_id).ok_or(Error::InvalidPageId)?,
        replacement,
    )?;
    Ok(candidate)
}

pub(super) fn editable_atom(
    document: &StoredDocument,
    page: u32,
    fragment: u32,
    atom: u32,
) -> Result<(&Fragment, &Atom), Error> {
    if !document.is_fully_mapped()
        || document
            .source_info()
            .is_some_and(|source| source.format != "cdxml")
    {
        return Err(Error::UnsupportedAtomLabel(
            "source-free or completely mapped native CDXML data is required",
        ));
    }
    let page_id = PageId::new(page).ok_or(Error::InvalidPageId)?;
    let fragment_id =
        FragmentId::new(fragment).ok_or(Error::InvalidAtomLabel("fragment id must be positive"))?;
    let atom_id = AtomId::new(atom).ok_or(Error::InvalidAtomLabel("atom id must be positive"))?;
    let fragment = document
        .model()
        .pages()
        .iter()
        .find(|page| page.id() == page_id)
        .ok_or(bondline_model::Error::MissingPage(page_id.get()))?
        .fragments()
        .iter()
        .find(|fragment| fragment.id() == fragment_id)
        .ok_or(bondline_model::Error::MissingFragment(fragment_id.get()))?;
    let missing = || Error::MissingAtom {
        fragment_id: fragment_id.get(),
        atom_id: atom_id.get(),
    };
    let target = fragment
        .atoms()
        .iter()
        .find(|atom| atom.id == atom_id)
        .ok_or_else(missing)?;
    if target.charge != 0 || target.isotope.is_some() || target.radical != Radical::None {
        return Err(Error::UnsupportedAtomLabel(
            "the target must be neutral, nonisotopic and nonradical",
        ));
    }
    Ok((fragment, target))
}
