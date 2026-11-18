//! Source-preserving bridge to the bounded neutral integer-bond authoring policy.

use super::{BondOrder, Hydrogens, NormalizeError, SmilesDocument};
use bondline_chem::{AtomSemantics, MAX_AUTHORING_ATOMS, MAX_AUTHORING_BONDS};
use bondline_model::{Atom, AtomId, Bond, BondId, Fragment, FragmentId, GraphError};
use bondline_render::Cancellation;
use std::num::NonZeroU16;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AuthoringError {
    #[error("SMILES authoring cancelled")]
    Cancelled,
    #[error("SMILES authoring exceeds the whole-graph budget")]
    Limit,
    #[error("SMILES authoring refuses {0}")]
    Unsupported(&'static str),
    #[error(transparent)]
    Normalize(#[from] NormalizeError),
    #[error(transparent)]
    Chemistry(#[from] bondline_chem::AuthoringError),
    #[error(transparent)]
    Model(#[from] GraphError),
}

impl AuthoringError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::Limit | Self::Model(GraphError::Limit(_)) => "resource_limit",
            Self::Unsupported(_) => "unsupported_content",
            Self::Normalize(error) => error.code(),
            Self::Chemistry(error) => error.code(),
            Self::Model(_) => "invalid_document",
        }
    }
}

fn identity(index: usize) -> Result<u32, AuthoringError> {
    index
        .checked_add(1)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(AuthoringError::Limit)
}

fn check(cancel: &Cancellation) -> Result<(), AuthoringError> {
    if cancel.is_cancelled() {
        Err(AuthoringError::Cancelled)
    } else {
        Ok(())
    }
}

impl SmilesDocument {
    /// Derive one complete neutral fixed-H Kekulé graph without rewriting source or IR.
    /// Source aromatic markers are input semantics, not an aromaticity-perception result.
    /// Positions remain absent; layout and editable-session authority are separate gates.
    pub fn normalize_authoring_fragment(
        &self,
        cancel: &Cancellation,
    ) -> Result<Fragment, AuthoringError> {
        check(cancel)?;
        if self.atoms().len() > MAX_AUTHORING_ATOMS || self.bonds().len() > MAX_AUTHORING_BONDS {
            return Err(AuthoringError::Limit);
        }
        if !self.atoms().iter().any(|atom| atom.aromatic)
            && !self
                .bonds()
                .iter()
                .any(|bond| bond.order == BondOrder::Aromatic)
        {
            let fragment = self.normalize_explicit_h_fragment()?;
            check(cancel)?;
            return Ok(fragment);
        }
        let (fragment, semantics) = self.authoring_input(cancel)?;
        Ok(bondline_chem::normalize_neutral_kekule(
            &fragment,
            &semantics,
            || cancel.is_cancelled(),
        )?)
    }

    pub(super) fn authoring_input(
        &self,
        cancel: &Cancellation,
    ) -> Result<(Fragment, Vec<AtomSemantics>), AuthoringError> {
        check(cancel)?;
        if self.atoms().len() > MAX_AUTHORING_ATOMS || self.bonds().len() > MAX_AUTHORING_BONDS {
            return Err(AuthoringError::Limit);
        }
        let mut atoms = Vec::with_capacity(self.atoms().len());
        let mut semantics = Vec::with_capacity(self.atoms().len());
        for (index, source) in self.atoms().iter().enumerate() {
            check(cancel)?;
            let mut atom = Atom::new(
                AtomId::new(identity(index)?).ok_or(AuthoringError::Limit)?,
                source.element,
                None,
            );
            atom.isotope = match source.isotope {
                Some(0) => return Err(AuthoringError::Unsupported("isotope zero")),
                Some(mass) => NonZeroU16::new(mass),
                None => None,
            };
            atom.charge = source.charge;
            let hydrogens = match source.hydrogens {
                Hydrogens::Implicit => bondline_chem::Hydrogens::Implicit,
                Hydrogens::Bracket(count) => {
                    atom.explicit_hydrogens = Some(u16::from(count));
                    bondline_chem::Hydrogens::Count(u16::from(count))
                }
            };
            semantics.push(AtomSemantics {
                aromatic: source.aromatic,
                hydrogens,
            });
            atoms.push(atom);
        }
        let mut bonds = Vec::with_capacity(self.bonds().len());
        for (index, source) in self.bonds().iter().enumerate() {
            check(cancel)?;
            bonds.push(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(identity(index)?).ok_or(AuthoringError::Limit)?,
                begin: AtomId::new(identity(source.atoms[0])?).ok_or(AuthoringError::Limit)?,
                end: AtomId::new(identity(source.atoms[1])?).ok_or(AuthoringError::Limit)?,
                order: match source.order {
                    BondOrder::Single => bondline_model::BondOrder::Single,
                    BondOrder::Double => bondline_model::BondOrder::Double,
                    BondOrder::Triple => bondline_model::BondOrder::Triple,
                    BondOrder::Aromatic => bondline_model::BondOrder::Aromatic,
                },
            });
        }
        let fragment = Fragment::new(
            FragmentId::new(1).ok_or(AuthoringError::Limit)?,
            atoms,
            bonds,
        )?;
        Ok((fragment, semantics))
    }
}
