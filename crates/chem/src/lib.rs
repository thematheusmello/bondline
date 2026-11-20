//! Model-only chemistry with explicit source semantics and bounded authoring policies.

#![forbid(unsafe_code)]

mod composition;
mod formula;
mod masses;
pub use composition::{
    Composition, CompositionError, ElementCount, IsotopeCount, analyze_fragment,
};
pub use formula::FormulaIsotopes;
pub use masses::{ElementWeight, MassUnavailable, MolecularMasses};

use bondline_model::{Atom, AtomId, BondId, BondOrder, Fragment, GraphError, Radical};

/// Attached H on one unbracketed, nonaromatic OpenSMILES organic-subset atom.
/// The caller sums every actual incident bond order, including ring closures.
pub fn opensmiles_implicit_hydrogens(
    element: bondline_model::Element,
    occupied: u16,
) -> Option<u16> {
    let normal: &[u16] = match element.get() {
        5 => &[3],
        6 => &[4],
        7 | 15 => &[3, 5],
        8 => &[2],
        16 => &[2, 4, 6],
        9 | 17 | 35 | 53 => &[1],
        _ => return None,
    };
    // The specification assigns zero above the largest normal valence.
    Some(
        normal
            .iter()
            .find_map(|valence| valence.checked_sub(occupied))
            .unwrap_or(0),
    )
}

pub const MAX_AUTHORING_ATOMS: usize = 64;
pub const MAX_AUTHORING_BONDS: usize = 128;
const MAX_SEARCH_VISITS: usize = 100_000;
const MAX_SEARCH_DEPTH: usize = 64;

/// Source hydrogen intent; an omitted bracket count is `Count(0)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hydrogens {
    Implicit,
    Count(u16),
}

/// Source markers aligned with the input fragment's atom array, independent of IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtomSemantics {
    pub aromatic: bool,
    pub hydrogens: Hydrogens,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AuthoringError {
    #[error("unsupported neutral authoring graph: {0}")]
    Unsupported(&'static str),
    #[error("neutral authoring resource limit: {0}")]
    Limit(&'static str),
    #[error("neutral authoring cancelled")]
    Cancelled,
    #[error("invalid authoring graph: {0}")]
    InvalidDocument(&'static str),
    #[error(transparent)]
    Model(#[from] GraphError),
}

impl AuthoringError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unsupported(_) => "unsupported_content",
            Self::Limit(_) | Self::Model(GraphError::Limit(_)) => "resource_limit",
            Self::Cancelled => "cancelled",
            Self::InvalidDocument(_) | Self::Model(_) => "invalid_document",
        }
    }
}

fn check_cancel(cancelled: &impl Fn() -> bool) -> Result<(), AuthoringError> {
    if cancelled() {
        Err(AuthoringError::Cancelled)
    } else {
        Ok(())
    }
}

fn item<T>(items: &[T], index: usize) -> Result<&T, AuthoringError> {
    items.get(index).ok_or(AuthoringError::InvalidDocument(
        "inconsistent graph indexing",
    ))
}

fn item_mut<T>(items: &mut [T], index: usize) -> Result<&mut T, AuthoringError> {
    items.get_mut(index).ok_or(AuthoringError::InvalidDocument(
        "inconsistent graph indexing",
    ))
}

fn weight(order: BondOrder) -> Result<u16, AuthoringError> {
    match order {
        BondOrder::Single | BondOrder::Aromatic => Ok(1),
        BondOrder::Double => Ok(2),
        BondOrder::Triple => Ok(3),
        _ => Err(AuthoringError::Unsupported(
            "bond kind outside Single/Double/Triple/Aromatic",
        )),
    }
}

#[derive(Clone, Copy)]
struct Neighbor {
    atom: usize,
    bond: usize,
    atom_id: AtomId,
    bond_id: BondId,
}

fn adjacency(
    fragment: &Fragment,
    semantics: &[AtomSemantics],
    cancelled: &impl Fn() -> bool,
) -> Result<Vec<Vec<Neighbor>>, AuthoringError> {
    let mut adjacency = vec![Vec::new(); fragment.atoms().len()];
    for (bond_index, bond) in fragment.bonds().iter().enumerate() {
        check_cancel(cancelled)?;
        weight(bond.order)?;
        let mut endpoints = [0; 2];
        for (slot, id) in endpoints.iter_mut().zip([bond.begin, bond.end]) {
            *slot = fragment
                .atoms()
                .iter()
                .position(|atom| atom.id == id)
                .ok_or(AuthoringError::InvalidDocument("missing bond endpoint"))?;
            if bond.order == BondOrder::Aromatic && !item(semantics, *slot)?.aromatic {
                return Err(AuthoringError::Unsupported(
                    "aromatic bond needs two marked atoms",
                ));
            }
        }
        let [begin, end] = endpoints;
        for (at, other, other_id) in [(begin, end, bond.end), (end, begin, bond.begin)] {
            item_mut(&mut adjacency, at)?.push(Neighbor {
                atom: other,
                bond: bond_index,
                atom_id: other_id,
                bond_id: bond.id,
            });
        }
    }
    for neighbors in &mut adjacency {
        neighbors.sort_unstable_by_key(|edge| (edge.atom_id, edge.bond_id));
    }
    Ok(adjacency)
}

struct State {
    hydrogens: u16,
    demand: bool,
    sp2: bool,
}

fn state(
    atom: &Atom,
    semantics: AtomSemantics,
    neighbors: &[Neighbor],
    fragment: &Fragment,
) -> Result<State, AuthoringError> {
    if atom.charge != 0 || atom.radical != Radical::None {
        return Err(AuthoringError::Unsupported("charge or radical state"));
    }
    let mut base = 0_u16;
    let mut doubles = 0_u16;
    let mut triples = 0_u16;
    for edge in neighbors {
        let order = item(fragment.bonds(), edge.bond)?.order;
        base = base
            .checked_add(weight(order)?)
            .ok_or(AuthoringError::Limit("bond sum"))?;
        doubles += u16::from(order == BondOrder::Double);
        triples += u16::from(order == BondOrder::Triple);
    }
    let degree = u16::try_from(neighbors.len()).map_err(|_| AuthoringError::Limit("degree"))?;
    let target = match atom.element.get() {
        1 | 9 | 17 | 35 | 53 if !semantics.aromatic => 1_u16,
        6 => 4,
        7 => 3,
        8 => 2,
        _ => {
            return Err(AuthoringError::Unsupported(
                "element outside the closed neutral profile",
            ));
        }
    };
    if semantics.aromatic && (!(2..=3).contains(&degree) || triples != 0 || doubles > 1) {
        return Err(AuthoringError::Unsupported(
            "marked atom cannot have an sp2 state",
        ));
    }
    let hydrogens = match semantics.hydrogens {
        Hydrogens::Count(count) => count,
        Hydrogens::Implicit if semantics.aromatic && atom.element.get() == 6 => 3_u16
            .checked_sub(degree)
            .ok_or(AuthoringError::Unsupported("aromatic carbon degree"))?,
        Hydrogens::Implicit if semantics.aromatic => 0,
        Hydrogens::Implicit => target.checked_sub(base).ok_or(AuthoringError::Unsupported(
            "bond sum exceeds neutral valence",
        ))?,
    };
    if atom.element.get() == 1
        && (semantics.hydrogens != Hydrogens::Count(0) || degree != 1 || base != 1)
    {
        return Err(AuthoringError::Unsupported(
            "physical H requires H0 and one Single bond",
        ));
    }
    let used = base
        .checked_add(hydrogens)
        .ok_or(AuthoringError::Unsupported(
            "H count exceeds neutral valence",
        ))?;
    let demand = target.checked_sub(used).ok_or(AuthoringError::Unsupported(
        "H and bonds exceed neutral valence",
    ))?;
    if demand > u16::from(semantics.aromatic) {
        return Err(AuthoringError::Unsupported(
            "H and bonds do not fit neutral valence",
        ));
    }
    if semantics.aromatic {
        match atom.element.get() {
            6 if doubles + demand != 1 => {
                return Err(AuthoringError::Unsupported(
                    "aromatic carbon needs exactly one Double",
                ));
            }
            8 if degree != 2 || hydrogens != 0 || doubles != 0 || demand != 0 => {
                return Err(AuthoringError::Unsupported(
                    "aromatic oxygen needs a degree-two H0 donor state",
                ));
            }
            _ => {}
        }
    }
    let sp2 = triples == 0
        && match atom.element.get() {
            6 => doubles + demand == 1,
            7 => doubles + demand == 1 || used == 3,
            8 => degree == 2 && hydrogens == 0 && doubles == 0,
            _ => false,
        };
    Ok(State {
        hydrogens,
        demand: demand == 1,
        sp2,
    })
}

// Removing a vertex leaves two of its neighbors connected exactly when it lies
// on a cycle. Unlike a two-core test, this excludes paths joining separate cycles.
fn on_sp2_cycle(
    vertex: usize,
    adjacency: &[Vec<Neighbor>],
    states: &[State],
    cancelled: &impl Fn() -> bool,
) -> Result<bool, AuthoringError> {
    if !item(states, vertex)?.sp2 {
        return Ok(false);
    }
    let mut visited = vec![false; states.len()];
    *item_mut(&mut visited, vertex)? = true;
    let mut queue = Vec::with_capacity(states.len());
    for seed in item(adjacency, vertex)? {
        check_cancel(cancelled)?;
        if *item(&visited, seed.atom)? || !item(states, seed.atom)?.sp2 {
            continue;
        }
        queue.clear();
        queue.push(seed.atom);
        *item_mut(&mut visited, seed.atom)? = true;
        let mut cursor = 0;
        let mut adjacent_to_vertex = 0;
        while cursor < queue.len() {
            check_cancel(cancelled)?;
            let at = *item(&queue, cursor)?;
            cursor += 1;
            for edge in item(adjacency, at)? {
                if edge.atom == vertex {
                    adjacent_to_vertex += 1;
                    if adjacent_to_vertex == 2 {
                        return Ok(true);
                    }
                } else if item(states, edge.atom)?.sp2 && !*item(&visited, edge.atom)? {
                    *item_mut(&mut visited, edge.atom)? = true;
                    queue.push(edge.atom);
                }
            }
        }
    }
    Ok(false)
}

struct Matching<'a, C> {
    adjacency: &'a [Vec<Neighbor>],
    fragment: &'a Fragment,
    order: Vec<(AtomId, usize)>,
    needed: Vec<bool>,
    doubles: Vec<bool>,
    visits: usize,
    cancelled: &'a C,
}

impl<C: Fn() -> bool> Matching<'_, C> {
    fn complete(&mut self, depth: usize) -> Result<bool, AuthoringError> {
        check_cancel(self.cancelled)?;
        if depth > MAX_SEARCH_DEPTH || self.visits >= MAX_SEARCH_VISITS {
            return Err(AuthoringError::Limit("Kekule search"));
        }
        self.visits += 1;
        let mut next = None;
        for &(_, at) in &self.order {
            if *item(&self.needed, at)? {
                next = Some(at);
                break;
            }
        }
        let Some(at) = next else { return Ok(true) };
        let count = item(self.adjacency, at)?.len();
        for index in 0..count {
            check_cancel(self.cancelled)?;
            let edge = *item(item(self.adjacency, at)?, index)?;
            if item(self.fragment.bonds(), edge.bond)?.order != BondOrder::Aromatic
                || !*item(&self.needed, edge.atom)?
            {
                continue;
            }
            *item_mut(&mut self.needed, at)? = false;
            *item_mut(&mut self.needed, edge.atom)? = false;
            *item_mut(&mut self.doubles, edge.bond)? = true;
            if self.complete(depth + 1)? {
                return Ok(true);
            }
            *item_mut(&mut self.needed, at)? = true;
            *item_mut(&mut self.needed, edge.atom)? = true;
            *item_mut(&mut self.doubles, edge.bond)? = false;
        }
        Ok(false)
    }
}

/// Resolve the closed neutral policy and supplied aromatic markers to fixed H
/// and integer bonds. Source semantics are validated, not perceived or canonicalized.
/// The input is immutable; only the returned graph's H and Aromatic orders change.
pub fn normalize_neutral_kekule(
    fragment: &Fragment,
    semantics: &[AtomSemantics],
    cancelled: impl Fn() -> bool,
) -> Result<Fragment, AuthoringError> {
    check_cancel(&cancelled)?;
    if fragment.atoms().len() > MAX_AUTHORING_ATOMS || fragment.bonds().len() > MAX_AUTHORING_BONDS
    {
        return Err(AuthoringError::Limit("64 atoms / 128 bonds"));
    }
    if semantics.len() != fragment.atoms().len() {
        return Err(AuthoringError::InvalidDocument("atom sidecar length"));
    }
    for (atom, source) in fragment.atoms().iter().zip(semantics) {
        check_cancel(&cancelled)?;
        let specified = match source.hydrogens {
            Hydrogens::Implicit => None,
            Hydrogens::Count(count) => Some(count),
        };
        if specified != atom.explicit_hydrogens {
            return Err(AuthoringError::InvalidDocument(
                "atom sidecar H correspondence",
            ));
        }
    }
    let adjacency = adjacency(fragment, semantics, &cancelled)?;
    let mut states = Vec::with_capacity(fragment.atoms().len());
    for ((atom, &source), neighbors) in fragment.atoms().iter().zip(semantics).zip(&adjacency) {
        check_cancel(&cancelled)?;
        states.push(state(atom, source, neighbors, fragment)?);
    }
    for (at, source) in semantics.iter().enumerate() {
        if source.aromatic && !on_sp2_cycle(at, &adjacency, &states, &cancelled)? {
            return Err(AuthoringError::Unsupported(
                "marked atom needs an sp2 cycle",
            ));
        }
    }
    let mut order = fragment
        .atoms()
        .iter()
        .enumerate()
        .map(|(at, atom)| (atom.id, at))
        .collect::<Vec<_>>();
    order.sort_unstable();
    let mut matching = Matching {
        adjacency: &adjacency,
        fragment,
        order,
        needed: states.iter().map(|state| state.demand).collect(),
        doubles: vec![false; fragment.bonds().len()],
        visits: 0,
        cancelled: &cancelled,
    };
    if !matching.complete(0)? {
        return Err(AuthoringError::Unsupported("no complete Kekule matching"));
    }
    let mut atoms = fragment.atoms().to_vec();
    for (atom, state) in atoms.iter_mut().zip(states) {
        check_cancel(&cancelled)?;
        atom.explicit_hydrogens = Some(state.hydrogens);
    }
    let mut bonds = fragment.bonds().to_vec();
    for (at, bond) in bonds.iter_mut().enumerate() {
        check_cancel(&cancelled)?;
        if bond.order == BondOrder::Aromatic {
            bond.order = if *item(&matching.doubles, at)? {
                BondOrder::Double
            } else {
                BondOrder::Single
            };
        }
    }
    check_cancel(&cancelled)?;
    Ok(Fragment::new(fragment.id(), atoms, bonds)?)
}
