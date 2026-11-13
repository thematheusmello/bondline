//! Source-preserving V2000 MOL syntax and an explicit, strict elemental model mapping.
//! Numeric fields follow BIOVIA Chemistry 2026 CTfile Formats, printed pages 62–70/90.

mod parse;
mod symbols;
pub mod v3000;
mod write;
pub use write::{write_v2000, write_v2000_with_header};

use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Point2, Radical,
};
use std::num::NonZeroU16;

pub const MAX_MOL_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_MOL_ITEMS: usize = 999;
pub const MAX_MOL_LINE_BYTES: usize = 4096;
pub const MAX_MOL_LINES: usize = 100_000;

#[derive(Debug, thiserror::Error)]
pub enum MolError {
    #[error("MOL input or output exceeds 64 MiB")]
    InputTooLarge,
    #[error("MOL requires printable ASCII with LF or CRLF line endings")]
    InvalidEncoding,
    #[error("MOL line {line} exceeds 4096 bytes")]
    LineTooLong { line: usize },
    #[error("MOL exceeds 100000 lines")]
    LineLimit,
    #[error("missing {field} at MOL line {line}")]
    Truncated { line: usize, field: &'static str },
    #[error("invalid {field} at MOL line {line}")]
    InvalidField { line: usize, field: &'static str },
    #[error("unsupported MOL feature {feature} at line {line}; original bytes are retained")]
    Unsupported { line: usize, feature: &'static str },
    #[error("MOL is missing its M  END terminator")]
    MissingEnd,
    #[error("unexpected data after MOL terminator at line {line}")]
    TrailingData { line: usize },
    #[error("V2000 permits at most 999 atoms and 999 bonds")]
    CountLimit,
    #[error("MOL transform needs a finite positive scale")]
    InvalidTransform,
    #[error("MOL coordinate cannot fit a 10-character field with four decimal places")]
    CoordinateRange,
    #[error(transparent)]
    Model(#[from] bondline_model::Error),
    #[error(transparent)]
    Graph(#[from] bondline_model::GraphError),
}

impl MolError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InputTooLarge | Self::LineTooLong { .. } | Self::LineLimit | Self::CountLimit => {
                "resource_limit"
            }
            Self::Unsupported { .. } | Self::CoordinateRange => "unsupported_content",
            Self::InvalidTransform => "invalid_params",
            _ => "invalid_mol",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedFeature {
    pub line: usize,
    pub feature: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MolAtom<'a> {
    pub symbol: &'a str,
    pub element: Option<Element>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub mass_difference: i8,
    pub isotope: Option<NonZeroU16>,
    pub charge: i8,
    pub radical: Radical,
    /// None is unmarked; Some(0) is encoded by V2000 valence 15.
    pub declared_valence: Option<u8>,
    line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MolBond {
    pub begin: u16,
    pub end: u16,
    pub order: u8,
    line: usize,
}

/// Complete original bytes plus parsed authored fields. Unknown properties stay in
/// source and are explicitly refused by the model mapping, never silently dropped.
pub struct MolFile<'a> {
    source: &'a [u8],
    header: [&'a str; 3],
    atoms: Vec<MolAtom<'a>>,
    bonds: Vec<MolBond>,
    unsupported: Vec<UnsupportedFeature>,
    isotope_override: bool,
}

impl std::fmt::Debug for MolFile<'_> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("MolFile")
            .field("byte_length", &self.source.len())
            .field("atom_count", &self.atoms.len())
            .field("bond_count", &self.bonds.len())
            .field("unsupported", &self.unsupported)
            .finish()
    }
}

/// Caller-chosen conversion of raw CTfile coordinates to drawing points.
/// CTfile x/y increase right/up; the model increases right/down. No unit or scale
/// is inferred from the drawing, molecule, header, or reference application.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MolTransform {
    points_per_unit: f64,
    origin: Point2,
}

impl MolTransform {
    pub fn new(points_per_unit: f64, origin: Point2) -> Result<Self, MolError> {
        if !points_per_unit.is_finite() || points_per_unit <= 0.0 {
            return Err(MolError::InvalidTransform);
        }
        Ok(Self {
            points_per_unit,
            origin,
        })
    }
    pub fn points_per_unit(self) -> f64 {
        self.points_per_unit
    }
    pub fn origin(self) -> Point2 {
        self.origin
    }
    fn import(self, x: f64, y: f64) -> Result<Point2, MolError> {
        Ok(Point2::new(
            self.origin.x_pt() + x * self.points_per_unit,
            self.origin.y_pt() - y * self.points_per_unit,
        )?)
    }
    fn export(self, position: Point2) -> (f64, f64) {
        (
            (position.x_pt() - self.origin.x_pt()) / self.points_per_unit,
            (self.origin.y_pt() - position.y_pt()) / self.points_per_unit,
        )
    }
}

impl<'a> MolFile<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.source
    }
    pub fn header(&self) -> [&'a str; 3] {
        self.header
    }
    pub fn atoms(&self) -> &[MolAtom<'a>] {
        &self.atoms
    }
    pub fn bonds(&self) -> &[MolBond] {
        &self.bonds
    }
    pub fn unsupported(&self) -> &[UnsupportedFeature] {
        &self.unsupported
    }

    /// Strict elemental graph projection with fresh 1-based application identities.
    /// Valence-to-H arithmetic only decodes an explicit authored valence; it never
    /// infers an element's usual valence or perceives chemistry/stereochemistry.
    pub fn to_fragment(&self, transform: MolTransform) -> Result<Fragment, MolError> {
        if let Some(feature) = self.unsupported.first() {
            return Err(MolError::Unsupported {
                line: feature.line,
                feature: feature.feature,
            });
        }
        let mut totals = vec![Some(0_u16); self.atoms.len()];
        let mut bonds = Vec::with_capacity(self.bonds.len());
        for (index, raw) in self.bonds.iter().enumerate() {
            let order = match raw.order {
                1 => BondOrder::Single,
                2 => BondOrder::Double,
                3 => BondOrder::Triple,
                4 => BondOrder::Aromatic,
                _ => {
                    return Err(MolError::Unsupported {
                        line: raw.line,
                        feature: "query bond order",
                    });
                }
            };
            for endpoint in [raw.begin, raw.end] {
                let index = usize::from(endpoint)
                    .checked_sub(1)
                    .ok_or(MolError::InvalidField {
                        line: raw.line,
                        field: "bond endpoint",
                    })?;
                let total = totals.get_mut(index).ok_or(MolError::InvalidField {
                    line: raw.line,
                    field: "bond endpoint",
                })?;
                *total = if raw.order == 4 {
                    None
                } else {
                    total.and_then(|value| value.checked_add(u16::from(raw.order)))
                };
            }
            bonds.push(Bond {
                id: BondId::new(u32::try_from(index + 1).map_err(|_| MolError::CountLimit)?)
                    .ok_or(MolError::CountLimit)?,
                begin: AtomId::new(u32::from(raw.begin)).ok_or(MolError::CountLimit)?,
                end: AtomId::new(u32::from(raw.end)).ok_or(MolError::CountLimit)?,
                order,
            });
        }
        let mut atoms = Vec::with_capacity(self.atoms.len());
        for (index, raw) in self.atoms.iter().enumerate() {
            if raw.z != 0.0 {
                return Err(MolError::Unsupported {
                    line: raw.line,
                    feature: "3D coordinates",
                });
            }
            if raw.mass_difference != 0 && !self.isotope_override {
                return Err(MolError::Unsupported {
                    line: raw.line,
                    feature: "isotope mass difference without a verified reference mass",
                });
            }
            let element = raw.element.ok_or(MolError::Unsupported {
                line: raw.line,
                feature: "query or non-element atom symbol",
            })?;
            let mut atom = Atom::new(
                AtomId::new(u32::try_from(index + 1).map_err(|_| MolError::CountLimit)?)
                    .ok_or(MolError::CountLimit)?,
                element,
                Some(transform.import(raw.x, raw.y)?),
            );
            atom.charge = raw.charge;
            atom.isotope = raw.isotope;
            atom.radical = raw.radical;
            atom.explicit_hydrogens =
                match raw.declared_valence {
                    None => None,
                    Some(valence) => {
                        let total =
                            totals
                                .get(index)
                                .copied()
                                .flatten()
                                .ok_or(MolError::Unsupported {
                                    line: raw.line,
                                    feature: "declared valence on aromatic bonds",
                                })?;
                        Some(u16::from(valence).checked_sub(total).ok_or(
                            MolError::InvalidField {
                                line: raw.line,
                                field: "declared valence below bond-order sum",
                            },
                        )?)
                    }
                };
            atoms.push(atom);
        }
        Ok(Fragment::new(
            FragmentId::new(1).ok_or(MolError::CountLimit)?,
            atoms,
            bonds,
        )?)
    }
}
