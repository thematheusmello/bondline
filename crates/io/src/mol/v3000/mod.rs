//! Source-preserving V3000 syntax and a strict, independently tested elemental subset.
//! BIOVIA Chemistry 2026 CTfile Formats, printed pages 7–13 and 32, is normative.
//! Unknown syntax stays in the original bytes; a successful syntax parse is not
//! permission to regenerate unmodeled chemistry from a partial projection.

mod parse;
mod tokens;
mod write;
use super::MolTransform;
pub use super::UnsupportedFeature;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Radical,
};
use std::{collections::BTreeMap, num::NonZeroU16, ops::Range};
pub use write::{write_v3000, write_v3000_with_header};

pub const MAX_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_ITEMS: usize = 100_000;
pub const MAX_LINES: usize = 1_000_000;
pub const MAX_PHYSICAL_LINE: usize = 80;
pub const MAX_LOGICAL_BYTES: usize = 65536;
pub const MAX_SEGMENTS: usize = 1024;
pub const MAX_TOKENS: usize = 8192;
pub const MAX_TOTAL_TOKENS: usize = 4_000_000;
pub const MAX_DEPTH: usize = 256;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("V3000 resource limit: {0}")]
    Resource(&'static str),
    #[error("invalid V3000 {field} at line {line}")]
    Syntax { line: usize, field: &'static str },
    #[error("unsupported V3000 {feature} at line {line}; retain the original bytes")]
    Unsupported { line: usize, feature: &'static str },
    #[error("V3000 coordinates must remain finite through the drawing transform")]
    CoordinateRange,
    #[error(transparent)]
    Transform(#[from] super::MolError),
    #[error(transparent)]
    Graph(#[from] bondline_model::GraphError),
}
impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Resource(_) => "resource_limit",
            Self::Unsupported { .. } | Self::CoordinateRange => "unsupported_content",
            Self::Transform(error) => error.code(),
            _ => "invalid_mol",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Text(String),
    List(Vec<String>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub key: String,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtomRecord {
    pub source_index: u32,
    pub symbol: String,
    pub element: Option<Element>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub mapping: u32,
    pub charge: i8,
    pub radical: Radical,
    pub isotope: Option<NonZeroU16>,
    /// VAL absent/0 is unspecified; VAL=-1 is explicitly zero.
    pub declared_valence: Option<u8>,
    pub attributes: Vec<Attribute>,
    pub line: usize,
    /// Includes all original physical continuation lines and their endings.
    pub range: Range<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BondRecord {
    pub source_index: u32,
    pub begin: u32,
    pub end: u32,
    pub order: u8,
    pub attributes: Vec<Attribute>,
    pub line: usize,
    pub range: Range<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub name: String,
    pub parent: Option<usize>,
    pub line: usize,
    pub range: Range<usize>,
}

pub struct MolFile<'a> {
    source: &'a [u8],
    header: [&'a str; 3],
    atoms: Vec<AtomRecord>,
    bonds: Vec<BondRecord>,
    blocks: Vec<Block>,
    unsupported: Vec<UnsupportedFeature>,
}
impl std::fmt::Debug for MolFile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V3000MolFile")
            .field("byte_length", &self.source.len())
            .field("atoms", &self.atoms.len())
            .field("bonds", &self.bonds.len())
            .field("unsupported", &self.unsupported)
            .finish()
    }
}
impl<'a> MolFile<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.source
    }
    pub fn header(&self) -> [&'a str; 3] {
        self.header
    }
    pub fn atoms(&self) -> &[AtomRecord] {
        &self.atoms
    }
    pub fn bonds(&self) -> &[BondRecord] {
        &self.bonds
    }
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }
    pub fn unsupported(&self) -> &[UnsupportedFeature] {
        &self.unsupported
    }

    /// Fresh dense application identities are independent from positive source IDs.
    /// This only decodes declared valence minus integer bond-order sums; no normal
    /// valence, HCOUNT query, aromaticity, spin inference or stereo is perceived.
    pub fn to_fragment(&self, frame: MolTransform) -> Result<Fragment, Error> {
        if let Some(feature) = self.unsupported.first() {
            return Err(Error::Unsupported {
                line: feature.line,
                feature: feature.feature,
            });
        }
        let mut ids = BTreeMap::new();
        let mut totals = vec![0_u32; self.atoms.len()];
        for (index, atom) in self.atoms.iter().enumerate() {
            let id =
                AtomId::new(u32::try_from(index + 1).map_err(|_| Error::Resource("atom IDs"))?)
                    .ok_or(Error::Resource("atom IDs"))?;
            ids.insert(atom.source_index, (index, id));
        }
        let mut bonds = Vec::with_capacity(self.bonds.len());
        for (index, bond) in self.bonds.iter().enumerate() {
            let order = match bond.order {
                1 => BondOrder::Single,
                2 => BondOrder::Double,
                3 => BondOrder::Triple,
                _ => {
                    return Err(Error::Unsupported {
                        line: bond.line,
                        feature: "query or extended bond type",
                    });
                }
            };
            let mut endpoints = Vec::with_capacity(2);
            for endpoint in [bond.begin, bond.end] {
                let (position, id) = ids.get(&endpoint).copied().ok_or(Error::Syntax {
                    line: bond.line,
                    field: "bond endpoint",
                })?;
                let total = totals.get_mut(position).ok_or(Error::Syntax {
                    line: bond.line,
                    field: "bond endpoint",
                })?;
                *total = total
                    .checked_add(u32::from(bond.order))
                    .ok_or(Error::Resource("bond-order sum"))?;
                endpoints.push(id);
            }
            bonds.push(Bond {
                id: BondId::new(u32::try_from(index + 1).map_err(|_| Error::Resource("bond IDs"))?)
                    .ok_or(Error::Resource("bond IDs"))?,
                begin: *endpoints.first().ok_or(Error::Syntax {
                    line: bond.line,
                    field: "bond begin",
                })?,
                end: *endpoints.get(1).ok_or(Error::Syntax {
                    line: bond.line,
                    field: "bond end",
                })?,
                order,
            });
        }
        let mut atoms = Vec::with_capacity(self.atoms.len());
        for (index, raw) in self.atoms.iter().enumerate() {
            let (_, id) = ids.get(&raw.source_index).copied().ok_or(Error::Syntax {
                line: raw.line,
                field: "atom ID",
            })?;
            let element = raw.element.ok_or(Error::Unsupported {
                line: raw.line,
                feature: "query or non-element atom",
            })?;
            let mut atom = Atom::new(id, element, Some(frame.import(raw.x, raw.y)?));
            atom.charge = raw.charge;
            atom.radical = raw.radical;
            atom.isotope = raw.isotope;
            atom.explicit_hydrogens = match raw.declared_valence {
                None => None,
                Some(value) => Some(
                    u16::try_from(
                        u32::from(value)
                            .checked_sub(*totals.get(index).ok_or(Error::Syntax {
                                line: raw.line,
                                field: "valence",
                            })?)
                            .ok_or(Error::Syntax {
                                line: raw.line,
                                field: "declared valence below bond-order sum",
                            })?,
                    )
                    .map_err(|_| Error::Syntax {
                        line: raw.line,
                        field: "valence",
                    })?,
                ),
            };
            atoms.push(atom);
        }
        Ok(Fragment::new(
            FragmentId::new(1).ok_or(Error::Resource("fragment ID"))?,
            atoms,
            bonds,
        )?)
    }
}
