use crate::Error;
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, Point2,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BondCreate {
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = -100000, max = 100000))]
    pub begin_x_pt: f64,
    #[schemars(range(min = -100000, max = 100000))]
    pub begin_y_pt: f64,
    #[schemars(range(min = -100000, max = 100000))]
    pub end_x_pt: f64,
    #[schemars(range(min = -100000, max = 100000))]
    pub end_y_pt: f64,
}

#[derive(Clone, Copy, Debug, Default)]
// Session floors outlive history so new branches cannot reuse removed identities.
pub(super) struct Ids {
    atom: u32,
    bond: u32,
    fragment: u32,
}

pub(super) fn create(
    document: &StoredDocument,
    params: BondCreate,
    floor: Ids,
) -> Result<(StoredDocument, Ids), Error> {
    if !document.is_fully_mapped() {
        return Err(Error::SourceBackedEdit);
    }
    let page = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
    let begin = Point2::new(params.begin_x_pt, params.begin_y_pt)?;
    let end = Point2::new(params.end_x_pt, params.end_y_pt)?;
    if begin == end {
        return Err(Error::CoincidentEndpoints);
    }
    let mut atom_max = floor.atom;
    let mut bond_max = floor.bond;
    let mut fragment_max = floor.fragment;
    for fragment in document
        .model()
        .pages()
        .iter()
        .flat_map(|page| page.fragments())
    {
        fragment_max = fragment_max.max(fragment.id().get());
        for atom in fragment.atoms() {
            atom_max = atom_max.max(atom.id.get());
        }
        for bond in fragment.bonds() {
            bond_max = bond_max.max(bond.id.get());
        }
    }
    let a = AtomId::new(atom_max.checked_add(1).ok_or(Error::IdentityExhausted)?)
        .ok_or(Error::IdentityExhausted)?;
    let b = AtomId::new(atom_max.checked_add(2).ok_or(Error::IdentityExhausted)?)
        .ok_or(Error::IdentityExhausted)?;
    let bond = BondId::new(bond_max.checked_add(1).ok_or(Error::IdentityExhausted)?)
        .ok_or(Error::IdentityExhausted)?;
    let id = FragmentId::new(
        fragment_max
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?,
    )
    .ok_or(Error::IdentityExhausted)?;
    let carbon = Element::try_from(6).map_err(bondline_model::Error::from)?;
    let mut first = Atom::new(a, carbon, Some(begin));
    let mut second = Atom::new(b, carbon, Some(end));
    first.explicit_hydrogens = Some(3);
    second.explicit_hydrogens = Some(3);
    let fragment = Fragment::new(
        id,
        vec![first, second],
        vec![Bond {
            id: bond,
            begin: a,
            end: b,
            order: BondOrder::Single,
        }],
    )
    .map_err(bondline_model::Error::from)?;
    let mut candidate = document.clone();
    candidate
        .model_mut()
        .append_fragments(page, vec![fragment])?;
    Ok((
        candidate,
        Ids {
            atom: b.get(),
            bond: bond.get(),
            fragment: id.get(),
        },
    ))
}
