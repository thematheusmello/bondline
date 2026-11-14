//! Opt-in materialization of a restricted SMILES hydrogen policy into fixed counts.

use super::{BondOrder, Hydrogens, Limit, SmilesDocument, write::target_valence};
use bondline_model::{Atom, AtomId, Bond, BondId, Fragment, FragmentId, GraphError};
use std::num::NonZeroU16;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NormalizeError {
    #[error("SMILES normalization refuses byte {offset}: {feature}")]
    Unsupported {
        offset: usize,
        feature: &'static str,
    },
    #[error(
        "SMILES atom at byte {offset} does not fit target {expected}: bond sum {bond_order_sum}, source H {hydrogens:?}"
    )]
    Valence {
        offset: usize,
        expected: u8,
        bond_order_sum: u16,
        hydrogens: Hydrogens,
    },
    #[error("SMILES normalization exceeds the {kind:?} budget")]
    Limit { kind: Limit },
    #[error(transparent)]
    Model(#[from] GraphError),
}

impl NormalizeError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unsupported { .. } | Self::Valence { .. } => "unsupported_content",
            Self::Limit { .. } | Self::Model(GraphError::Limit(_)) => "resource_limit",
            Self::Model(_) => "invalid_document",
        }
    }
}

fn identity(index: usize, kind: Limit) -> Result<u32, NormalizeError> {
    index
        .checked_add(1)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(NormalizeError::Limit { kind })
}

impl SmilesDocument {
    /// Materialize this closed neutral subset's H counts as fixed model values.
    /// Bare C/N/O/halogens resolve only within C4/N3/O2/halogen1; bracket H stays exact.
    /// Physical H remains a node with H0 and one Single bond. Other/radical-ambiguous
    /// contexts refuse. This is not live inference or a chemical-validity certificate.
    /// Source/IR stay untouched. Output IDs are dense and local to the new Fragment;
    /// positions are None. This supplies no layout, application import or Save authority.
    pub fn normalize_explicit_h_fragment(&self) -> Result<Fragment, NormalizeError> {
        // The immutable owner already enforces the complete source/graph budgets.
        let mut totals = vec![(0_u16, 0_u16); self.atoms().len()];
        let mut bonds = Vec::with_capacity(self.bonds().len());
        for (index, raw) in self.bonds().iter().enumerate() {
            let (weight, order) = match raw.order {
                BondOrder::Single => (1, bondline_model::BondOrder::Single),
                BondOrder::Double => (2, bondline_model::BondOrder::Double),
                BondOrder::Triple => (3, bondline_model::BondOrder::Triple),
            };
            let [begin, end] = raw.atoms;
            for endpoint in raw.atoms {
                let id = identity(endpoint, Limit::Atoms)?;
                let (sum, degree) = totals
                    .get_mut(endpoint)
                    .ok_or(GraphError::MissingAtom(id))?;
                *sum = sum
                    .checked_add(weight)
                    .ok_or(NormalizeError::Limit { kind: Limit::Bonds })?;
                *degree = degree
                    .checked_add(1)
                    .ok_or(NormalizeError::Limit { kind: Limit::Bonds })?;
            }
            bonds.push(Bond {
                id: BondId::new(identity(index, Limit::Bonds)?)
                    .ok_or(NormalizeError::Limit { kind: Limit::Bonds })?,
                begin: AtomId::new(identity(begin, Limit::Atoms)?)
                    .ok_or(NormalizeError::Limit { kind: Limit::Atoms })?,
                end: AtomId::new(identity(end, Limit::Atoms)?)
                    .ok_or(NormalizeError::Limit { kind: Limit::Atoms })?,
                order,
            });
        }
        let mut atoms = Vec::with_capacity(self.atoms().len());
        for (index, raw) in self.atoms().iter().enumerate() {
            if raw.charge != 0 {
                return Err(NormalizeError::Unsupported {
                    offset: raw
                        .charge_range
                        .as_ref()
                        .map_or(raw.range.start, |range| range.start),
                    feature: "nonneutral charge",
                });
            }
            let isotope = match raw.isotope {
                Some(0) => {
                    return Err(NormalizeError::Unsupported {
                        offset: raw
                            .isotope_range
                            .as_ref()
                            .map_or(raw.range.start, |range| range.start),
                        feature: "isotope zero is not representable by the positive-isotope model",
                    });
                }
                Some(mass) => NonZeroU16::new(mass),
                None => None,
            };
            let expected = target_valence(raw.element).ok_or(NormalizeError::Unsupported {
                offset: raw.symbol_range.start,
                feature: "element outside the closed neutral H/C/N/O/halogen profile",
            })?;
            let id = AtomId::new(identity(index, Limit::Atoms)?)
                .ok_or(NormalizeError::Limit { kind: Limit::Atoms })?;
            let (sum, degree) = totals
                .get(index)
                .copied()
                .ok_or(GraphError::MissingAtom(id.get()))?;
            if raw.element.get() == 1
                && (raw.hydrogens != Hydrogens::Bracket(0) || sum != 1 || degree != 1)
            {
                return Err(NormalizeError::Unsupported {
                    offset: raw.range.start,
                    feature: "physical H requires bracket H0 and one Single bond",
                });
            }
            let mismatch = || NormalizeError::Valence {
                offset: raw.range.start,
                expected,
                bond_order_sum: sum,
                hydrogens: raw.hydrogens,
            };
            let hydrogens = match raw.hydrogens {
                Hydrogens::Implicit => u16::from(expected).checked_sub(sum).ok_or_else(mismatch)?,
                Hydrogens::Bracket(count) => {
                    if sum.checked_add(u16::from(count)) != Some(u16::from(expected)) {
                        return Err(mismatch());
                    }
                    u16::from(count)
                }
            };
            let mut atom = Atom::new(id, raw.element, None);
            atom.isotope = isotope;
            atom.explicit_hydrogens = Some(hydrogens);
            atoms.push(atom);
        }
        Ok(Fragment::new(
            FragmentId::new(1).ok_or(NormalizeError::Limit { kind: Limit::Atoms })?,
            atoms,
            bonds,
        )?)
    }
}
