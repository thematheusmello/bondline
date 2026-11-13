use crate::Point2;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    num::{NonZeroU16, NonZeroU32},
};

// shortcut: cap each graph item category at 100000; revise with large-document memory budgets.
pub const MAX_GRAPH_ITEMS: usize = 100_000;

macro_rules! identity {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(NonZeroU32);
        impl $name {
            pub const fn new(value: u32) -> Option<Self> {
                match NonZeroU32::new(value) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }
    };
}

identity!(AtomId);
identity!(BondId);
identity!(FragmentId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Element(u8);

impl Element {
    pub const fn new(atomic_number: u8) -> Option<Self> {
        if atomic_number >= 1 && atomic_number <= 118 {
            Some(Self(atomic_number))
        } else {
            None
        }
    }
    pub const fn get(self) -> u8 {
        self.0
    }
}
impl TryFrom<u8> for Element {
    type Error = GraphError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(GraphError::InvalidElement(value))
    }
}
impl From<Element> for u8 {
    fn from(value: Element) -> Self {
        value.get()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Radical {
    #[default]
    None,
    Singlet,
    Doublet,
    Triplet,
}

/// One authored elemental atom. No valence, isotope-existence or stereo inference is performed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Atom {
    pub id: AtomId,
    pub element: Element,
    pub position: Option<Point2>,
    pub isotope: Option<NonZeroU16>,
    #[serde(default)]
    pub charge: i8,
    #[serde(default)]
    pub radical: Radical,
    pub explicit_hydrogens: Option<u16>,
}

impl Atom {
    pub fn new(id: AtomId, element: Element, position: Option<Point2>) -> Self {
        Self {
            id,
            element,
            position,
            isotope: None,
            charge: 0,
            radical: Radical::None,
            explicit_hydrogens: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BondOrder {
    Single,
    Double,
    Triple,
    Quadruple,
    Quintuple,
    Sextuple,
    Half,
    Aromatic,
    TwoAndHalf,
    ThreeAndHalf,
    FourAndHalf,
    FiveAndHalf,
    Dative,
    Ionic,
    Hydrogen,
    ThreeCenter,
}

/// Begin/end ordering is authored data, including direction-dependent bond kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bond {
    pub id: BondId,
    pub begin: AtomId,
    pub end: AtomId,
    pub order: BondOrder,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GraphError {
    #[error("atomic number {0} is outside the supported 1..118 catalog")]
    InvalidElement(u8),
    #[error("graph exceeds 100000 {0}")]
    Limit(&'static str),
    #[error("atom id {0} is already used")]
    DuplicateAtom(u32),
    #[error("bond id {0} is already used")]
    DuplicateBond(u32),
    #[error("fragment id {0} is already used")]
    DuplicateFragment(u32),
    #[error("bond endpoint atom {0} does not exist in its fragment")]
    MissingAtom(u32),
    #[error("bond {0} cannot connect an atom to itself")]
    SelfBond(u32),
    #[error("the current graph model permits only one bond object per atom pair")]
    DuplicateConnection,
}

/// Structurally validated connectivity. Cycles, disconnected and empty fragments are allowed.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Fragment {
    id: FragmentId,
    atoms: Vec<Atom>,
    bonds: Vec<Bond>,
}

impl Fragment {
    pub fn new(id: FragmentId, atoms: Vec<Atom>, bonds: Vec<Bond>) -> Result<Self, GraphError> {
        if atoms.len() > MAX_GRAPH_ITEMS {
            return Err(GraphError::Limit("atoms"));
        }
        if bonds.len() > MAX_GRAPH_ITEMS {
            return Err(GraphError::Limit("bonds"));
        }
        let mut atom_ids = BTreeSet::new();
        for atom in &atoms {
            if !atom_ids.insert(atom.id) {
                return Err(GraphError::DuplicateAtom(atom.id.get()));
            }
        }
        let mut bond_ids = BTreeSet::new();
        let mut pairs = BTreeSet::new();
        for bond in &bonds {
            if !bond_ids.insert(bond.id) {
                return Err(GraphError::DuplicateBond(bond.id.get()));
            }
            if bond.begin == bond.end {
                return Err(GraphError::SelfBond(bond.id.get()));
            }
            for endpoint in [bond.begin, bond.end] {
                if !atom_ids.contains(&endpoint) {
                    return Err(GraphError::MissingAtom(endpoint.get()));
                }
            }
            if !pairs.insert((bond.begin.min(bond.end), bond.begin.max(bond.end))) {
                return Err(GraphError::DuplicateConnection);
            }
        }
        Ok(Self { id, atoms, bonds })
    }
    pub fn id(&self) -> FragmentId {
        self.id
    }
    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
    pub fn bonds(&self) -> &[Bond] {
        &self.bonds
    }
}
