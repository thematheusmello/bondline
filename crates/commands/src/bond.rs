use crate::Error;
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, Point2, Radical,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Closed authoring choices; absent order retains the original Single command behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BondDrawOrder {
    #[default]
    Single,
    Double,
    Triple,
}

impl BondDrawOrder {
    fn is_single(&self) -> bool {
        *self == Self::Single
    }

    fn weight(self) -> u16 {
        match self {
            Self::Single => 1,
            Self::Double => 2,
            Self::Triple => 3,
        }
    }

    fn model_order(self) -> BondOrder {
        match self {
            Self::Single => BondOrder::Single,
            Self::Double => BondOrder::Double,
            Self::Triple => BondOrder::Triple,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BondCreate {
    #[serde(default, skip_serializing_if = "BondDrawOrder::is_single")]
    pub order: BondDrawOrder,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExtendHydrogenPolicy {
    RectifyTerminalCarbon,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BondExtend {
    #[serde(default, skip_serializing_if = "BondDrawOrder::is_single")]
    pub order: BondDrawOrder,
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 1))]
    pub fragment_id: u32,
    #[schemars(range(min = 1))]
    pub begin_atom_id: u32,
    #[schemars(range(min = -100000, max = 100000))]
    pub end_x_pt: f64,
    #[schemars(range(min = -100000, max = 100000))]
    pub end_y_pt: f64,
    /// Required opt-in to replace available literal terminal-carbon H with a new bond.
    pub hydrogen_policy: ExtendHydrogenPolicy,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderHydrogenPolicy {
    RectifyNeutralCarbon,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BondSetOrder {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 1))]
    pub fragment_id: u32,
    #[schemars(range(min = 1))]
    pub bond_id: u32,
    pub order: BondDrawOrder,
    pub hydrogen_policy: OrderHydrogenPolicy,
}

fn bond_weight(order: BondOrder) -> Option<u16> {
    match order {
        BondOrder::Single => Some(1),
        BondOrder::Double => Some(2),
        BondOrder::Triple => Some(3),
        _ => None,
    }
}

pub(super) fn set_order(
    document: &StoredDocument,
    params: BondSetOrder,
) -> Result<StoredDocument, Error> {
    if !document.is_fully_mapped()
        || document
            .source_info()
            .is_some_and(|source| source.format != "cdxml")
    {
        return Err(Error::UnsupportedBondOrderSource);
    }
    let OrderHydrogenPolicy::RectifyNeutralCarbon = params.hydrogen_policy;
    let page_id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
    let fragment_id = FragmentId::new(params.fragment_id).ok_or(Error::InvalidBondTarget)?;
    let bond_id = BondId::new(params.bond_id).ok_or(Error::InvalidBondTarget)?;
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
    let missing_bond = || Error::MissingBond {
        fragment_id: fragment_id.get(),
        bond_id: bond_id.get(),
    };
    let target = fragment
        .bonds()
        .iter()
        .find(|bond| bond.id == bond_id)
        .ok_or_else(missing_bond)?;
    let old_weight = bond_weight(target.order).ok_or(Error::UnsupportedBondOrder)?;
    let mut hydrogens = [0; 2];
    for (result, id) in hydrogens.iter_mut().zip([target.begin, target.end]) {
        let atom =
            fragment
                .atoms()
                .iter()
                .find(|atom| atom.id == id)
                .ok_or(Error::MissingAtom {
                    fragment_id: fragment_id.get(),
                    atom_id: id.get(),
                })?;
        if atom.element.get() != 6
            || atom.charge != 0
            || atom.isotope.is_some()
            || atom.radical != Radical::None
        {
            return Err(Error::UnsupportedBondOrder);
        }
        let used = fragment
            .bonds()
            .iter()
            .filter(|bond| bond.begin == id || bond.end == id)
            .try_fold(0_u16, |sum, bond| {
                sum.checked_add(bond_weight(bond.order)?)
                    .filter(|sum| *sum <= 4)
            })
            .ok_or(Error::UnsupportedBondOrder)?;
        if atom.explicit_hydrogens != Some(4 - used) {
            return Err(Error::UnsupportedBondOrder);
        }
        let new_used = used
            .checked_sub(old_weight)
            .and_then(|sum| sum.checked_add(params.order.weight()))
            .filter(|sum| *sum <= 4)
            .ok_or(Error::UnsupportedBondOrder)?;
        *result = 4 - new_used;
    }
    // Validate both endpoints even for no-ops; keep the original graph allocation.
    if target.order == params.order.model_order() {
        return Ok(document.clone());
    }
    let mut atoms = fragment.atoms().to_vec();
    let [begin_hydrogens, end_hydrogens] = hydrogens;
    for atom in &mut atoms {
        if atom.id == target.begin {
            atom.explicit_hydrogens = Some(begin_hydrogens);
        } else if atom.id == target.end {
            atom.explicit_hydrogens = Some(end_hydrogens);
        }
    }
    let mut bonds = fragment.bonds().to_vec();
    bonds
        .iter_mut()
        .find(|bond| bond.id == bond_id)
        .ok_or_else(missing_bond)?
        .order = params.order.model_order();
    let replacement =
        Fragment::new(fragment_id, atoms, bonds).map_err(bondline_model::Error::from)?;
    let mut candidate = document.clone();
    candidate
        .model_mut()
        .replace_fragment(page_id, replacement)?;
    Ok(candidate)
}

#[derive(Clone, Copy, Debug, Default)]
// Session floors outlive history so new branches cannot reuse removed identities.
pub(super) struct Ids {
    pub(super) atom: u32,
    pub(super) bond: u32,
    pub(super) fragment: u32,
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
    let maximum = maxima(document, floor);
    let atom_max = maximum.atom;
    let bond_max = maximum.bond;
    let fragment_max = maximum.fragment;
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
    first.explicit_hydrogens = Some(4 - params.order.weight());
    second.explicit_hydrogens = Some(4 - params.order.weight());
    let fragment = Fragment::new(
        id,
        vec![first, second],
        vec![Bond {
            id: bond,
            begin: a,
            end: b,
            order: params.order.model_order(),
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

fn maxima(document: &StoredDocument, mut maximum: Ids) -> Ids {
    for fragment in document
        .model()
        .pages()
        .iter()
        .flat_map(|page| page.fragments())
    {
        maximum.fragment = maximum.fragment.max(fragment.id().get());
        for atom in fragment.atoms() {
            maximum.atom = maximum.atom.max(atom.id.get());
        }
        for bond in fragment.bonds() {
            maximum.bond = maximum.bond.max(bond.id.get());
        }
    }
    maximum
}

pub(super) fn extend(
    document: &StoredDocument,
    params: BondExtend,
    floor: Ids,
) -> Result<(StoredDocument, Ids), Error> {
    if !document.is_fully_mapped()
        || document
            .source_info()
            .is_some_and(|source| source.format != "cdxml")
    {
        return Err(Error::UnsupportedExtensionSource);
    }
    let ExtendHydrogenPolicy::RectifyTerminalCarbon = params.hydrogen_policy;
    let page_id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
    let fragment_id = FragmentId::new(params.fragment_id).ok_or(Error::InvalidBondTarget)?;
    let begin = AtomId::new(params.begin_atom_id).ok_or(Error::InvalidBondTarget)?;
    let page = document
        .model()
        .pages()
        .iter()
        .find(|page| page.id() == page_id)
        .ok_or(bondline_model::Error::MissingPage(page_id.get()))?;
    let fragment = page
        .fragments()
        .iter()
        .find(|fragment| fragment.id() == fragment_id)
        .ok_or(bondline_model::Error::MissingFragment(fragment_id.get()))?;
    let missing = || Error::MissingAtom {
        fragment_id: fragment_id.get(),
        atom_id: begin.get(),
    };
    let atom = fragment
        .atoms()
        .iter()
        .find(|atom| atom.id == begin)
        .ok_or_else(missing)?;
    if fragment.atoms().iter().any(|atom| {
        atom.element.get() != 6
            || atom.position.is_none()
            || atom.charge != 0
            || atom.isotope.is_some()
            || atom.radical != Radical::None
    }) || fragment.bonds().iter().any(|bond| {
        !matches!(
            bond.order,
            BondOrder::Single | BondOrder::Double | BondOrder::Triple
        )
    }) {
        return Err(Error::UnsupportedBondExtension);
    }
    let mut incident = fragment
        .bonds()
        .iter()
        .filter(|bond| bond.begin == begin || bond.end == begin);
    let existing_weight = match incident.next().map(|bond| bond.order) {
        Some(BondOrder::Single) => 1,
        Some(BondOrder::Double) => 2,
        Some(BondOrder::Triple) => 3,
        _ => return Err(Error::UnsupportedBondExtension),
    };
    let available_hydrogens = 4 - existing_weight;
    if incident.next().is_some()
        || atom.explicit_hydrogens != Some(available_hydrogens)
        || params.order.weight() > available_hydrogens
    {
        return Err(Error::UnsupportedBondExtension);
    }
    let end = Point2::new(params.end_x_pt, params.end_y_pt)?;
    if atom.position == Some(end) {
        return Err(Error::CoincidentEndpoints);
    }
    if page
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.atoms())
        .any(|atom| atom.position == Some(end))
    {
        return Err(Error::OccupiedEndpoint);
    }
    let mut maximum = maxima(document, floor);
    maximum.atom = maximum
        .atom
        .checked_add(1)
        .ok_or(Error::IdentityExhausted)?;
    maximum.bond = maximum
        .bond
        .checked_add(1)
        .ok_or(Error::IdentityExhausted)?;
    let id = AtomId::new(maximum.atom).ok_or(Error::IdentityExhausted)?;
    let bond_id = BondId::new(maximum.bond).ok_or(Error::IdentityExhausted)?;
    let carbon = Element::try_from(6).map_err(bondline_model::Error::from)?;
    let mut atoms = fragment.atoms().to_vec();
    atoms
        .iter_mut()
        .find(|atom| atom.id == begin)
        .ok_or_else(missing)?
        .explicit_hydrogens = Some(available_hydrogens - params.order.weight());
    let mut added = Atom::new(id, carbon, Some(end));
    added.explicit_hydrogens = Some(4 - params.order.weight());
    atoms.push(added);
    let mut bonds = fragment.bonds().to_vec();
    bonds.push(Bond {
        id: bond_id,
        begin,
        end: id,
        order: params.order.model_order(),
    });
    let fragment = Fragment::new(fragment_id, atoms, bonds).map_err(bondline_model::Error::from)?;
    let mut candidate = document.clone();
    candidate.model_mut().replace_fragment(page_id, fragment)?;
    Ok((candidate, maximum))
}

#[cfg(test)]
mod tests {
    use super::{BondDrawOrder, BondExtend, ExtendHydrogenPolicy};
    use crate::{Command, Engine, Error};

    fn parameters(revision: u64, end_x: f64) -> BondExtend {
        BondExtend {
            order: BondDrawOrder::Single,
            expected_revision: revision,
            page_id: 1,
            fragment_id: 1,
            begin_atom_id: 2,
            end_x_pt: end_x,
            end_y_pt: 240.0,
            hydrogen_policy: ExtendHydrogenPolicy::RectifyTerminalCarbon,
        }
    }

    #[test]
    fn order_edit_noop_and_revision_exhaustion_do_not_allocate_or_replace_content()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::default();
        engine.dispatch("bond.create", serde_json::json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":148,"end_y_pt":200}))?;
        engine.revision = u64::MAX;
        engine.identities = super::Ids {
            atom: u32::MAX,
            bond: u32::MAX,
            fragment: u32::MAX,
        };
        let content = std::sync::Arc::clone(&engine.document);
        let before = serde_json::to_value(engine.state())?;
        let request = |order| serde_json::json!({"expected_revision":u64::MAX,"page_id":1,"fragment_id":1,"bond_id":1,"order":order,"hydrogen_policy":"rectify_neutral_carbon"});
        assert!(
            !engine
                .dispatch("bond.set_order", request("single"))?
                .changed
        );
        assert!(matches!(
            engine.dispatch("bond.set_order", request("double")),
            Err(Error::RevisionExhausted)
        ));
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(std::sync::Arc::ptr_eq(&content, &engine.document));
        assert_eq!(
            (
                engine.identities.atom,
                engine.identities.bond,
                engine.identities.fragment
            ),
            (u32::MAX, u32::MAX, u32::MAX)
        );
        Ok(())
    }

    #[test]
    fn typed_nonfinite_input_and_revision_exhaustion_leave_identity_floors_and_snapshots_untouched()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::default();
        engine.dispatch("bond.create",serde_json::json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0}))?;
        let before = engine.stored_document()?.clone();
        let floor = (
            engine.identities.atom,
            engine.identities.bond,
            engine.identities.fragment,
        );
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                engine
                    .execute(Command::BondExtend(parameters(1, invalid)))
                    .err()
                    .ok_or("nonfinite accepted")?
                    .info()
                    .code,
                "invalid_params"
            );
            assert_eq!(engine.stored_document()?, &before);
            assert_eq!(engine.state().revision, 1);
            assert_eq!(
                (
                    engine.identities.atom,
                    engine.identities.bond,
                    engine.identities.fragment
                ),
                floor
            );
        }
        engine.revision = u64::MAX;
        let state = serde_json::to_value(engine.state())?;
        assert!(matches!(
            engine.execute(Command::BondExtend(parameters(u64::MAX, 180.0))),
            Err(Error::RevisionExhausted)
        ));
        assert_eq!(serde_json::to_value(engine.state())?, state);
        assert_eq!(engine.stored_document()?, &before);
        assert_eq!(
            (
                engine.identities.atom,
                engine.identities.bond,
                engine.identities.fragment
            ),
            floor
        );
        Ok(())
    }
}
