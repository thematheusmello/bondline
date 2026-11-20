//! Read-only attached hydrogen counts for the published BIOVIA MOL valence convention.
use crate::{StoredDocument, display::DisplayError};
use bondline_model::{Atom, AtomId, BondOrder, Fragment, Radical};
use bondline_render::Cancellation;
use std::collections::BTreeMap;

impl StoredDocument {
    /// Consult only the current, completely represented source's valence convention.
    pub fn analysis_hydrogen_counts(
        &self,
        cancel: &Cancellation,
    ) -> Result<BTreeMap<AtomId, u16>, DisplayError> {
        if cancel.is_cancelled() {
            return Err(DisplayError::Cancelled);
        }
        // Certified CDXML neutral carbon uses the same tetravalent count on each
        // complete fragment. Other XML sources have no grant to derive missing H.
        if self.mol_version().is_none() && !self.has_cdxml_carbon_connectivity() {
            return self.cdx_skeletal_hydrogen_counts(cancel);
        }
        if !self.has_analysis_coverage() {
            return Ok(BTreeMap::new());
        }
        let [page] = self.model().pages() else {
            return Ok(BTreeMap::new());
        };
        let mut counts = BTreeMap::new();
        for fragment in page.fragments() {
            if cancel.is_cancelled() {
                return Err(DisplayError::Cancelled);
            }
            append_defaults(fragment, cancel, &mut counts)?;
        }
        Ok(counts)
    }
}

fn append_defaults(
    fragment: &Fragment,
    cancel: &Cancellation,
    counts: &mut BTreeMap<AtomId, u16>,
) -> Result<(), DisplayError> {
    let mut occupied = BTreeMap::new();
    for bond in fragment.bonds() {
        if cancel.is_cancelled() {
            return Err(DisplayError::Cancelled);
        }
        let order = match bond.order {
            BondOrder::Single => Some(1),
            BondOrder::Double => Some(2),
            BondOrder::Triple => Some(3),
            _ => None,
        };
        for id in [bond.begin, bond.end] {
            let sum = occupied.entry(id).or_insert(Some(0_u16));
            *sum = sum.and_then(|old| old.checked_add(order?));
        }
    }
    for atom in fragment
        .atoms()
        .iter()
        .filter(|a| a.explicit_hydrogens.is_none())
    {
        if cancel.is_cancelled() {
            return Err(DisplayError::Cancelled);
        }
        if let Some(sum) = occupied.get(&atom.id).copied().unwrap_or(Some(0))
            && let Some(h) = hydrogen_count(atom, sum)
        {
            counts.insert(atom.id, h);
        }
    }
    Ok(())
}

fn hydrogen_count(atom: &Atom, occupied: u16) -> Option<u16> {
    let number = atom.element.get();
    if number == 1 {
        // A physical hydrogen is itself counted by the composition, including D/T.
        return match (atom.charge, atom.radical, occupied) {
            (0, Radical::None, 0 | 1) | (0, Radical::Doublet, 0) | (-1 | 1, Radical::None, 0) => {
                Some(0)
            }
            _ => None,
        };
    }
    let vacancies = match atom.radical {
        Radical::None => 0,
        Radical::Doublet => 1,
        Radical::Singlet | Radical::Triplet => return None,
    };
    let occupied = occupied.checked_add(vacancies)?;
    let isoelectronic = match (number, atom.charge) {
        (_, 0) => number,
        // Period-two nonmetals and their same-row isoelectronic neighbours.
        (5..=9, -1 | 1) => u8::try_from(i16::from(number) - i16::from(atom.charge)).ok()?,
        (17 | 35 | 53 | 85, -1) => return (occupied == 0).then_some(0),
        _ => return None,
    };
    let allowed: &[u16] = match isoelectronic {
        2 | 10 | 18 | 36 | 54 | 86 => &[0],
        5 | 7 => &[3],
        6 | 14 => &[4],
        8 => &[2],
        9 | 35 => &[1],
        15 | 33 => &[3, 5],
        16 | 34 | 52 => &[2, 4, 6],
        17 | 53 | 85 => &[1, 3, 5, 7],
        _ => return None,
    };
    allowed
        .iter()
        .copied()
        .find_map(|v| v.checked_sub(occupied))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bondline_model::Element;

    #[test]
    fn default_valence_steps_and_unsupported_states_remain_distinct()
    -> Result<(), Box<dyn std::error::Error>> {
        let id = AtomId::new(1).ok_or("atom ID")?;
        for (element, charge, spin, used, expected) in [
            (16, 0, Radical::None, 3, Some(1)),
            (16, 0, Radical::None, 5, Some(1)),
            (16, 0, Radical::None, 7, None),
            (17, 0, Radical::None, 4, Some(1)),
            (35, 0, Radical::None, 2, None),
            (7, 1, Radical::None, 2, Some(2)),
            (8, -1, Radical::None, 1, Some(0)),
            (6, 0, Radical::Doublet, 0, Some(3)),
            (6, 0, Radical::Singlet, 0, None),
            (6, 0, Radical::Triplet, 0, None),
            (6, 3, Radical::None, 0, None),
            (26, 0, Radical::None, 0, None),
            (1, 0, Radical::None, 1, Some(0)),
            (1, 1, Radical::None, 1, None),
            (1, 0, Radical::Triplet, 0, None),
        ] {
            let mut atom = Atom::new(id, Element::try_from(element)?, None);
            atom.charge = charge;
            atom.radical = spin;
            assert_eq!(
                hydrogen_count(&atom, used),
                expected,
                "{element}/{charge}/{spin:?}/{used}"
            );
        }
        Ok(())
    }
}
