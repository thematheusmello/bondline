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
        self.calculate_masses(|_, _| {})
    }

    /// Return both descriptors and each element's share of the same molecular
    /// weight. Isotopes of one element share a row. Rows are sorted by atomic
    /// number. Missing data invalidate the whole denominator and all shares.
    pub fn masses_with_elemental_analysis(
        &self,
    ) -> (MolecularMasses, Result<Vec<ElementWeight>, MassUnavailable>) {
        let mut weights: Vec<(bondline_model::Element, f64)> = Vec::new();
        let masses = self.calculate_masses(|bin, contribution| {
            // Composition keeps isotope bins contiguous and in atomic-number order.
            if let Some((element, value)) = weights.last_mut()
                && *element == bin.element
            {
                *value += contribution;
            } else {
                weights.push((bin.element, contribution));
            }
        });
        let rows = masses.molecular_weight.and_then(|total| {
            weights
                .into_iter()
                .map(|(element, weight_contribution)| {
                    let atomic_number = element.get();
                    Ok(ElementWeight {
                        atomic_number,
                        symbol: element
                            .symbol()
                            .ok_or(MassUnavailable::UnknownElement { atomic_number })?,
                        weight_contribution,
                        // All positive bins have finite positive weight. Empty
                        // compositions produce zero rows, hence no division.
                        percent_by_weight: weight_contribution / total * 100.0,
                    })
                })
                .collect()
        });
        (masses, rows)
    }

    fn calculate_masses(&self, mut weight_bin: impl FnMut(&IsotopeCount, f64)) -> MolecularMasses {
        let molecular_weight = self.isotope_counts().iter().try_fold(0.0, |sum, bin| {
            let weight = bin.count as f64 * Quantity::MolecularWeight.atomic_mass(bin)?;
            weight_bin(bin, weight);
            Ok(sum + weight)
        });
        let neutral_exact = self.isotope_counts().iter().try_fold(0.0, |sum, bin| {
            Ok(sum + bin.count as f64 * Quantity::ExactMass.atomic_mass(bin)?)
        });
        MolecularMasses {
            molecular_weight,
            exact_mass_da: neutral_exact
                .map(|mass| mass - self.total_charge() as f64 * ELECTRON_MASS_DA),
        }
    }
}

/// A row of elemental analysis, averaged over isotope-unspecified atoms.
/// Both numerical values are unrounded. Weight contribution is dimensionless.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct ElementWeight {
    pub atomic_number: u8,
    pub symbol: &'static str,
    pub weight_contribution: f64,
    pub percent_by_weight: f64,
}
