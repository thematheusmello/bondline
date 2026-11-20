//! Literal atom/H accounting, independent of valence, topology and presentation.
use bondline_model::{Atom, AtomId, Element, Fragment, GraphError};
use std::{collections::BTreeMap, num::NonZeroU16};

/// Elemental total, including every explicitly resolved attached hydrogen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElementCount {
    pub element: Element,
    pub count: u64,
}

/// An isotope-resolved bin. `None` is unspecified, never a predominant isotope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IsotopeCount {
    pub element: Element,
    pub isotope: Option<NonZeroU16>,
    pub count: u64,
}

/// Numeric composition of supplied model data; not a chemical-validity certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composition {
    elements: Vec<ElementCount>,
    isotopes: Vec<IsotopeCount>,
    charge: i64,
}

impl Composition {
    /// Positive totals, sorted by atomic number (not a formula display order).
    pub fn element_counts(&self) -> &[ElementCount] {
        &self.elements
    }

    /// Sorted by atomic number, then unspecified isotope before increasing mass number.
    pub fn isotope_counts(&self) -> &[IsotopeCount] {
        &self.isotopes
    }

    /// Sum of authored atom formal charges, in units of elementary charge.
    pub const fn total_charge(&self) -> i64 {
        self.charge
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CompositionError {
    #[error("atom {0:?} has an unresolved attached-hydrogen count")]
    UnresolvedHydrogens(AtomId),
    #[error("composition analysis cancelled")]
    Cancelled,
    #[error("composition arithmetic overflow")]
    ArithmeticOverflow,
    #[error(transparent)]
    Model(#[from] GraphError),
}

#[derive(Default)]
struct Counts {
    elements: BTreeMap<u8, ElementCount>,
    isotopes: BTreeMap<(u8, Option<NonZeroU16>), IsotopeCount>,
    charge: i64,
}

impl Counts {
    fn add(
        &mut self,
        element: Element,
        isotope: Option<NonZeroU16>,
        count: u64,
    ) -> Result<(), CompositionError> {
        if count == 0 {
            return Ok(());
        }
        let total = self
            .elements
            .entry(element.get())
            .or_insert(ElementCount { element, count: 0 });
        total.count = total
            .count
            .checked_add(count)
            .ok_or(CompositionError::ArithmeticOverflow)?;
        let total = self
            .isotopes
            .entry((element.get(), isotope))
            .or_insert(IsotopeCount {
                element,
                isotope,
                count: 0,
            });
        total.count = total
            .count
            .checked_add(count)
            .ok_or(CompositionError::ArithmeticOverflow)?;
        Ok(())
    }
}

fn check_cancel(cancelled: &impl Fn() -> bool) -> Result<(), CompositionError> {
    if cancelled() {
        Err(CompositionError::Cancelled)
    } else {
        Ok(())
    }
}

/// Count a structurally validated fragment without modifying it or inferring chemistry.
///
/// Every atom, including physical H, requires a resolved `Some(explicit_hydrogens)`.
/// Each node contributes once to its own isotope bin. Its inline H count adds neutral,
/// isotope-unspecified H independently of any physical H neighbors or parent isotope.
/// Empty/disconnected graphs and the full model resource bounds are supported.
/// No isotope existence, valence, aromaticity, radical or source-completeness claim follows.
pub fn analyze_fragment(
    fragment: &Fragment,
    cancelled: impl Fn() -> bool,
) -> Result<Composition, CompositionError> {
    Composition::from_atoms(fragment.atoms(), cancelled)
}

impl Composition {
    /// Sum resolved authored atoms from one or more fragments. The caller supplies
    /// each selected atom once. Selection boundaries never add or remove hydrogen.
    pub fn from_atoms<'a>(
        atoms: impl IntoIterator<Item = &'a Atom>,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, CompositionError> {
        Self::from_atoms_with_hydrogen_defaults(atoms, |_| None, cancelled)
    }

    /// Use the source's hydrogen convention only where the author omitted a count.
    pub fn from_atoms_with_hydrogen_defaults<'a>(
        atoms: impl IntoIterator<Item = &'a Atom>,
        missing_count: impl Fn(AtomId) -> Option<u16>,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, CompositionError> {
        check_cancel(&cancelled)?;
        let hydrogen = Element::try_from(1)?;
        let mut counts = Counts::default();
        for atom in atoms {
            check_cancel(&cancelled)?;
            let h = atom
                .explicit_hydrogens
                .or_else(|| missing_count(atom.id))
                .ok_or(CompositionError::UnresolvedHydrogens(atom.id))?;
            counts.add(atom.element, atom.isotope, 1)?;
            counts.add(hydrogen, None, u64::from(h))?;
            counts.charge = counts
                .charge
                .checked_add(i64::from(atom.charge))
                .ok_or(CompositionError::ArithmeticOverflow)?;
        }
        let result = Composition {
            elements: counts.elements.into_values().collect(),
            isotopes: counts.isotopes.into_values().collect(),
            charge: counts.charge,
        };
        check_cancel(&cancelled)?;
        Ok(result)
    }
}
