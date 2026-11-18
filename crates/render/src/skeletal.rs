use crate::{Cancellation, Error};
use bondline_model::{Atom, AtomId, BondOrder, Fragment};
use std::collections::{BTreeMap, BTreeSet};

/// Return only explicitly hidden carbons without an authored label, using current geometry.
///
/// This reuses the existing display decision with literal H, without a resolved-count
/// sidecar. Absence from the set is not proof that a glyph exists: legacy paths differ.
/// The set alone grants no source, chemistry, physical-page or mutation authority.
pub fn hidden_carbon_atoms(
    fragment: &Fragment,
    cancel: &Cancellation,
) -> Result<BTreeSet<AtomId>, Error> {
    cancel.check()?;
    let labels = labels(fragment, None, cancel)?;
    let mut hidden = BTreeSet::new();
    for atom in fragment.atoms() {
        cancel.check()?;
        if atom.authored_label.is_none()
            && matches!(labels.get(&atom.id), Some(CarbonLabel::Hidden))
        {
            hidden.insert(atom.id);
        }
    }
    Ok(hidden)
}

pub(crate) enum CarbonLabel {
    Hidden,
    Visible(u16),
}

pub(crate) fn labels(
    fragment: &Fragment,
    counts: Option<&BTreeMap<AtomId, u16>>,
    cancel: &Cancellation,
) -> Result<BTreeMap<AtomId, CarbonLabel>, Error> {
    let mut labels = BTreeMap::new();
    if !fragment.atoms().iter().any(|atom| {
        atom.element.get() == 6
            && (atom.show_terminal_carbon_labels.is_some()
                || atom.show_non_terminal_carbon_labels.is_some())
    }) {
        return Ok(labels);
    }
    let mut atoms = BTreeMap::new();
    for atom in fragment.atoms() {
        cancel.check()?;
        atoms.insert(atom.id, atom);
    }
    let mut adjoining = BTreeMap::<AtomId, Vec<(AtomId, BondOrder)>>::new();
    for bond in fragment.bonds() {
        cancel.check()?;
        adjoining
            .entry(bond.begin)
            .or_default()
            .push((bond.end, bond.order));
        adjoining
            .entry(bond.end)
            .or_default()
            .push((bond.begin, bond.order));
    }
    for atom in fragment.atoms() {
        cancel.check()?;
        if atom.element.get() != 6 {
            continue;
        }
        let neighbours = adjoining.get(&atom.id).map_or(&[][..], Vec::as_slice);
        let flag = if neighbours.len() <= 1 {
            atom.show_terminal_carbon_labels
        } else {
            atom.show_non_terminal_carbon_labels
        };
        let Some(show) = flag else {
            continue;
        };
        let hydrogens = atom
            .explicit_hydrogens
            .or_else(|| counts.and_then(|counts| counts.get(&atom.id).copied()))
            .ok_or(Error::UnsupportedChemicalContent)?;
        let label = if show {
            CarbonLabel::Visible(hydrogens)
        } else {
            let weight = neighbours.iter().try_fold(0_u32, |sum, (_, order)| {
                cancel.check()?;
                let weight = match order {
                    BondOrder::Single => 1,
                    BondOrder::Double => 2,
                    BondOrder::Triple => 3,
                    _ => return Err(Error::UnsupportedChemicalContent),
                };
                sum.checked_add(weight)
                    .ok_or(Error::UnsupportedChemicalContent)
            })?;
            if weight.checked_add(u32::from(hydrogens)) != Some(4) {
                return Err(Error::UnsupportedChemicalContent);
            }
            let isolated = match neighbours {
                [] => true,
                [(other, _)] => {
                    atoms
                        .get(other)
                        .is_some_and(|other| other.element.get() == 6)
                        && adjoining
                            .get(other)
                            .is_some_and(|neighbours| neighbours.len() == 1)
                }
                _ => false,
            };
            if isolated || mandatory_label(atom, neighbours, &atoms)? {
                CarbonLabel::Visible(hydrogens)
            } else {
                CarbonLabel::Hidden
            }
        };
        labels.insert(atom.id, label);
    }
    Ok(labels)
}

fn mandatory_label(
    atom: &Atom,
    neighbours: &[(AtomId, BondOrder)],
    atoms: &BTreeMap<AtomId, &Atom>,
) -> Result<bool, Error> {
    let [(first, first_order), (second, second_order)] = neighbours else {
        return Ok(false);
    };
    if first_order != second_order {
        return Ok(false);
    }
    if *first_order == BondOrder::Double {
        return Ok(true);
    }
    let origin = atom.position.ok_or(Error::UnsupportedChemicalContent)?;
    let ray = |id| -> Result<[f64; 2], Error> {
        let point = atoms
            .get(id)
            .and_then(|atom| atom.position)
            .ok_or(Error::UnsupportedChemicalContent)?;
        let dx = point.x_pt() - origin.x_pt();
        let dy = point.y_pt() - origin.y_pt();
        let length = dx.hypot(dy);
        if !length.is_finite() || length == 0.0 {
            return Err(Error::InvalidGeometry);
        }
        Ok([dx / length, dy / length])
    };
    let first = ray(first)?;
    let second = ray(second)?;
    // Authored normalized rays keep IUPAC's inclusive 170-degree rule independent of camera rounding.
    Ok(first[0] * second[0] + first[1] * second[1] <= -0.984_807_753_012_208)
}
