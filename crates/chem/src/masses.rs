//! Relative molecular weight and calculated monoisotopic mass from resolved atoms.
use crate::{Composition, IsotopeCount};
use elements_rs::{Element, Isotope, IsotopicComposition, RelativeAtomicMass};

/// CODATA 2022 electron rest mass in unified atomic mass units.
const ELECTRON_MASS_DA: f64 = 0.000_548_579_909_044_1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error, serde::Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum MassUnavailable {
    #[error("no standard atomic weight for element {atomic_number}")]
    NoStandardAtomicWeight { atomic_number: u8 },
    #[error("no natural predominant isotope for element {atomic_number}")]
    NoNaturalIsotope { atomic_number: u8 },
    #[error("no tabulated mass for mass number {mass_number} of element {atomic_number}")]
    UnknownIsotope { atomic_number: u8, mass_number: u16 },
    #[error("no mass data for element {atomic_number}")]
    UnknownElement { atomic_number: u8 },
}

/// Each quantity may be unavailable independently. Molecular weight is dimensionless.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MolecularMasses {
    pub molecular_weight: Result<f64, MassUnavailable>,
    pub exact_mass_da: Result<f64, MassUnavailable>,
}

#[derive(Clone, Copy)]
enum Quantity {
    MolecularWeight,
    ExactMass,
}

impl Quantity {
    fn atomic_mass(self, bin: &IsotopeCount) -> Result<f64, MassUnavailable> {
        let atomic_number = bin.element.get();
        let element = Element::try_from(atomic_number)
            .map_err(|_| MassUnavailable::UnknownElement { atomic_number })?;
        if let Some(number) = bin.isotope {
            let mass_number = number.get();
            return Isotope::try_from((element, mass_number))
                .map(|isotope| isotope.relative_atomic_mass())
                .map_err(|_| MassUnavailable::UnknownIsotope {
                    atomic_number,
                    mass_number,
                });
        }
        // CIAAW does not assign these elements a standard atomic weight. The
        // data library also supplies representative fallback isotopes/weights.
        let natural_weight = !matches!(atomic_number, 43 | 61 | 84..=89 | 93..=118);
        match self {
            Self::MolecularWeight if natural_weight => Ok(element.standard_atomic_weight()),
            Self::MolecularWeight => Err(MassUnavailable::NoStandardAtomicWeight { atomic_number }),
            Self::ExactMass => {
                let isotope = element.most_abundant_isotope();
                if natural_weight && isotope.isotopic_composition().is_some_and(|p| p > 0.0) {
                    Ok(isotope.relative_atomic_mass())
                } else {
                    Err(MassUnavailable::NoNaturalIsotope { atomic_number })
                }
            }
        }
    }
}

impl Composition {
    /// Use the labelled nuclide for either calculation. Unlabelled atoms use the
    /// dataset's conventional weight or most abundant naturally occurring nuclide.
    /// Only exact mass includes the net charge correction; binding energies are omitted.
    pub fn molecular_masses(&self) -> MolecularMasses {
        let sum = |quantity: Quantity| {
            self.isotope_counts().iter().try_fold(0.0, |sum, bin| {
                Ok(sum + bin.count as f64 * quantity.atomic_mass(bin)?)
            })
        };
        MolecularMasses {
            molecular_weight: sum(Quantity::MolecularWeight),
            exact_mass_da: sum(Quantity::ExactMass)
                .map(|mass| mass - self.total_charge() as f64 * ELECTRON_MASS_DA),
        }
    }
}
