//! Original deterministic structure export; ordering by authored IDs is not canonicalization.

use super::{
    Limit, MAX_SMILES_ATOMS, MAX_SMILES_BONDS, MAX_SMILES_BRANCHES, MAX_SMILES_BYTES,
    MAX_SMILES_CHAINS, MAX_SMILES_RINGS,
};
use bondline_model::{Atom, BondOrder, Element, Fragment, Radical};
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WriteError {
    #[error("cannot export an empty fragment as a nonempty SMILES notation")]
    Empty,
    #[error("cannot export atom {atom_id}: {feature}")]
    UnsupportedAtom { atom_id: u32, feature: &'static str },
    #[error("cannot export bond {bond_id}: {feature}")]
    UnsupportedBond { bond_id: u32, feature: &'static str },
    #[error(
        "atom {atom_id} requires valence {expected}, but bond sum {bond_order_sum} plus literal H {hydrogens} differs"
    )]
    Valence {
        atom_id: u32,
        expected: u8,
        bond_order_sum: u16,
        hydrogens: u16,
    },
    #[error("generated SMILES exceeds the {kind:?} budget")]
    Limit { kind: Limit },
    #[error("invalid structure export plan: {reason}")]
    Structure { reason: &'static str },
}

impl WriteError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty | Self::Structure { .. } => "invalid_smiles_export",
            Self::UnsupportedAtom { .. } | Self::UnsupportedBond { .. } | Self::Valence { .. } => {
                "unsupported_content"
            }
            Self::Limit { .. } => "resource_limit",
        }
    }
}

type Link = (usize, usize); // Neighbor atom index and source bond index.

struct Plan<'a> {
    atoms: Vec<&'a Atom>,
    orders: Vec<&'static [u8]>,
    roots: Vec<usize>,
    children: Vec<Vec<Link>>,
    openings: Vec<Vec<usize>>,
    closings: Vec<Vec<usize>>,
}

fn structure(reason: &'static str) -> WriteError {
    WriteError::Structure { reason }
}

// Closed normalization/export profile, not periodic-table valence perception.
pub(super) fn target_valence(element: Element) -> Option<u8> {
    match element.get() {
        1 | 9 | 17 | 35 | 53 => Some(1),
        6 => Some(4),
        7 => Some(3),
        8 => Some(2),
        _ => None,
    }
}

fn valence(atom: &Atom) -> Result<u8, WriteError> {
    let unsupported = |feature| WriteError::UnsupportedAtom {
        atom_id: atom.id.get(),
        feature,
    };
    if atom.charge != 0 {
        return Err(unsupported("nonneutral charge"));
    }
    if atom.radical != Radical::None {
        return Err(unsupported("radical state"));
    }
    if atom.explicit_hydrogens.is_none() {
        return Err(unsupported("unspecified literal H count"));
    }
    target_valence(atom.element)
        .ok_or_else(|| unsupported("element outside the closed neutral H/C/N/O/halogen profile"))
}

fn prepare(fragment: &Fragment) -> Result<Plan<'_>, WriteError> {
    let count = fragment.atoms().len();
    if count > MAX_SMILES_ATOMS {
        return Err(WriteError::Limit { kind: Limit::Atoms });
    }
    if fragment.bonds().len() > MAX_SMILES_BONDS {
        return Err(WriteError::Limit { kind: Limit::Bonds });
    }
    if count == 0 {
        return Err(WriteError::Empty);
    }
    let mut atoms = fragment.atoms().iter().collect::<Vec<_>>();
    atoms.sort_unstable_by_key(|atom| atom.id);
    for atom in &atoms {
        valence(atom)?;
    }
    let ids = atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.id, index))
        .collect::<BTreeMap<_, _>>();
    let mut neighbors = vec![Vec::<Link>::new(); count];
    let mut sums = vec![0_u16; count];
    let mut orders = Vec::with_capacity(fragment.bonds().len());
    for (index, bond) in fragment.bonds().iter().enumerate() {
        let (weight, token): (u16, &'static [u8]) = match bond.order {
            BondOrder::Single => (1, b""),
            BondOrder::Double => (2, b"="),
            BondOrder::Triple => (3, b"#"),
            _ => {
                return Err(WriteError::UnsupportedBond {
                    bond_id: bond.id.get(),
                    feature: "bond order outside Single/Double/Triple",
                });
            }
        };
        let left = *ids
            .get(&bond.begin)
            .ok_or_else(|| structure("missing bond begin"))?;
        let right = *ids
            .get(&bond.end)
            .ok_or_else(|| structure("missing bond end"))?;
        for (at, other) in [(left, right), (right, left)] {
            let total = sums
                .get_mut(at)
                .ok_or_else(|| structure("missing valence sum"))?;
            *total = total
                .checked_add(weight)
                .ok_or_else(|| structure("bond order sum overflow"))?;
            neighbors
                .get_mut(at)
                .ok_or_else(|| structure("missing adjacency"))?
                .push((other, index));
        }
        orders.push(token);
    }
    for (index, atom) in atoms.iter().enumerate() {
        let expected = valence(atom)?;
        let hydrogens = atom
            .explicit_hydrogens
            .ok_or_else(|| structure("missing validated H count"))?;
        let sum = *sums
            .get(index)
            .ok_or_else(|| structure("missing valence sum"))?;
        if atom.element.get() == 1
            && (hydrogens != 0
                || sum != 1
                || neighbors
                    .get(index)
                    .ok_or_else(|| structure("missing adjacency"))?
                    .len()
                    != 1)
        {
            return Err(WriteError::UnsupportedAtom {
                atom_id: atom.id.get(),
                feature: "physical H requires literal H0 and one Single bond",
            });
        }
        if u32::from(sum).checked_add(u32::from(hydrogens)) != Some(u32::from(expected)) {
            return Err(WriteError::Valence {
                atom_id: atom.id.get(),
                expected,
                bond_order_sum: sum,
                hydrogens,
            });
        }
    }
    for list in &mut neighbors {
        list.sort_unstable_by_key(|(neighbor, _)| *neighbor);
    }
    let mut children = vec![Vec::new(); count];
    let mut roots = Vec::new();
    let mut ranks = vec![None; count];
    let mut rank_count = 0_usize;
    let mut tree = vec![false; fragment.bonds().len()];
    let mut stack = Vec::new();
    for root in 0..count {
        if ranks
            .get(root)
            .ok_or_else(|| structure("missing visit rank"))?
            .is_some()
        {
            continue;
        }
        if roots.len() >= MAX_SMILES_CHAINS {
            return Err(WriteError::Limit {
                kind: Limit::Chains,
            });
        }
        roots.push(root);
        *ranks
            .get_mut(root)
            .ok_or_else(|| structure("missing visit rank"))? = Some(rank_count);
        rank_count = rank_count
            .checked_add(1)
            .ok_or_else(|| structure("rank overflow"))?;
        stack.push((root, 0_usize));
        while let Some((current, next)) = stack.pop() {
            if let Some(&(child, edge)) = neighbors
                .get(current)
                .ok_or_else(|| structure("missing adjacency"))?
                .get(next)
            {
                stack.push((
                    current,
                    next.checked_add(1)
                        .ok_or_else(|| structure("adjacency cursor overflow"))?,
                ));
                if ranks
                    .get(child)
                    .ok_or_else(|| structure("missing child rank"))?
                    .is_none()
                {
                    *ranks
                        .get_mut(child)
                        .ok_or_else(|| structure("missing child rank"))? = Some(rank_count);
                    rank_count = rank_count
                        .checked_add(1)
                        .ok_or_else(|| structure("rank overflow"))?;
                    *tree
                        .get_mut(edge)
                        .ok_or_else(|| structure("missing tree edge"))? = true;
                    children
                        .get_mut(current)
                        .ok_or_else(|| structure("missing children"))?
                        .push((child, edge));
                    stack.push((child, 0));
                }
            }
        }
    }
    let mut openings = vec![Vec::new(); count];
    let mut closings = vec![Vec::new(); count];
    for (current, adjacent) in neighbors.iter().enumerate() {
        let rank = ranks
            .get(current)
            .copied()
            .flatten()
            .ok_or_else(|| structure("unvisited atom"))?;
        for &(other, edge) in adjacent {
            let other_rank = ranks
                .get(other)
                .copied()
                .flatten()
                .ok_or_else(|| structure("unvisited neighbor"))?;
            if !tree
                .get(edge)
                .copied()
                .ok_or_else(|| structure("missing tree edge"))?
                && rank < other_rank
            {
                openings
                    .get_mut(current)
                    .ok_or_else(|| structure("missing ring openings"))?
                    .push(edge);
                closings
                    .get_mut(other)
                    .ok_or_else(|| structure("missing ring closings"))?
                    .push(edge);
            }
        }
    }
    Ok(Plan {
        atoms,
        orders,
        roots,
        children,
        openings,
        closings,
    })
}

fn append(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), WriteError> {
    output
        .len()
        .checked_add(bytes.len())
        .filter(|length| *length <= MAX_SMILES_BYTES)
        .ok_or(WriteError::Limit { kind: Limit::Bytes })?;
    output.extend_from_slice(bytes);
    Ok(())
}

fn label_token(label: u8) -> String {
    if label < 10 {
        label.to_string()
    } else {
        format!("%{label:02}")
    }
}

enum Event {
    Visit {
        atom: usize,
        incoming: Option<usize>,
        branch: bool,
        depth: usize,
    },
    CloseBranch,
}

/// Export one explicitly authored supported structure as valid, noncanonical SMILES.
/// Every atom must have literal H and satisfy the documented closed finite valence.
/// Authored IDs choose traversal; positions are intentionally not encoded. This is
/// structure export, not a whole-document Save or permission to discard source backing.
pub fn write_fragment(fragment: &Fragment) -> Result<Vec<u8>, WriteError> {
    let plan = prepare(fragment)?;
    let mut output = Vec::new();
    let mut labels = vec![None; plan.orders.len()];
    let mut occupied = [false; MAX_SMILES_RINGS];
    let mut events = Vec::new();
    for (component, &root) in plan.roots.iter().enumerate() {
        if component > 0 {
            append(&mut output, b".")?;
        }
        events.push(Event::Visit {
            atom: root,
            incoming: None,
            branch: false,
            depth: 0,
        });
        while let Some(event) = events.pop() {
            let Event::Visit {
                atom: index,
                incoming,
                branch,
                depth,
            } = event
            else {
                append(&mut output, b")")?;
                continue;
            };
            if branch {
                if depth > MAX_SMILES_BRANCHES {
                    return Err(WriteError::Limit {
                        kind: Limit::Branches,
                    });
                }
                append(&mut output, b"(")?;
            }
            if let Some(edge) = incoming {
                append(
                    &mut output,
                    plan.orders
                        .get(edge)
                        .ok_or_else(|| structure("missing incoming bond"))?,
                )?;
            }
            let atom = plan
                .atoms
                .get(index)
                .ok_or_else(|| structure("missing output atom"))?;
            let symbol = crate::mol::symbols::symbol(atom.element)
                .ok_or_else(|| structure("missing element symbol"))?;
            let isotope = atom
                .isotope
                .map(|mass| mass.get().to_string())
                .unwrap_or_default();
            let token = if atom.element.get() == 1 {
                format!("[{isotope}{symbol}]")
            } else {
                format!(
                    "[{isotope}{symbol}H{}]",
                    atom.explicit_hydrogens
                        .ok_or_else(|| structure("missing output H count"))?
                )
            };
            append(&mut output, token.as_bytes())?;
            // Close first so labels freed at this atom are available for its openings.
            for &edge in plan
                .closings
                .get(index)
                .ok_or_else(|| structure("missing ring closings"))?
            {
                let label = labels
                    .get_mut(edge)
                    .ok_or_else(|| structure("missing closing label"))?
                    .take()
                    .ok_or_else(|| structure("unopened ring closure"))?;
                append(&mut output, label_token(label).as_bytes())?;
                *occupied
                    .get_mut(usize::from(label))
                    .ok_or_else(|| structure("invalid ring label"))? = false;
            }
            for &edge in plan
                .openings
                .get(index)
                .ok_or_else(|| structure("missing ring openings"))?
            {
                let available = occupied
                    .iter()
                    .position(|used| !*used)
                    .ok_or(WriteError::Limit { kind: Limit::Rings })?;
                let label = u8::try_from(available).map_err(|_| structure("invalid ring label"))?;
                append(
                    &mut output,
                    plan.orders
                        .get(edge)
                        .ok_or_else(|| structure("missing ring bond"))?,
                )?;
                append(&mut output, label_token(label).as_bytes())?;
                *occupied
                    .get_mut(available)
                    .ok_or_else(|| structure("invalid ring label"))? = true;
                *labels
                    .get_mut(edge)
                    .ok_or_else(|| structure("missing opening label"))? = Some(label);
            }
            let children = plan
                .children
                .get(index)
                .ok_or_else(|| structure("missing output children"))?;
            for (position, &(child, edge)) in children.iter().enumerate().rev() {
                let branch = position
                    .checked_add(1)
                    .ok_or_else(|| structure("child cursor overflow"))?
                    < children.len();
                let child_depth =
                    depth
                        .checked_add(usize::from(branch))
                        .ok_or(WriteError::Limit {
                            kind: Limit::Branches,
                        })?;
                if child_depth > MAX_SMILES_BRANCHES {
                    return Err(WriteError::Limit {
                        kind: Limit::Branches,
                    });
                }
                if branch {
                    events.push(Event::CloseBranch);
                }
                events.push(Event::Visit {
                    atom: child,
                    incoming: Some(edge),
                    branch,
                    depth: child_depth,
                });
            }
        }
    }
    if labels.iter().any(Option::is_some) || occupied.iter().any(|used| *used) {
        return Err(structure("unclosed generated ring"));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_boundary_append_refuses_before_changing_the_candidate() -> Result<(), WriteError> {
        let mut output = vec![b'C'; MAX_SMILES_BYTES - 1];
        append(&mut output, b"C")?;
        assert_eq!(output.len(), 65_536);
        assert!(matches!(
            append(&mut output, b"C"),
            Err(WriteError::Limit { kind: Limit::Bytes })
        ));
        assert_eq!(output, vec![b'C'; 65_536]);
        Ok(())
    }
}
