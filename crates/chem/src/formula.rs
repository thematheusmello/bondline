//! Plain-text Hill formulas for a resolved composition.
use crate::{Composition, CompositionError};
use bondline_model::GraphError;

/// Choose whether numbered isotopes share their element's count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormulaIsotopes {
    Combined,
    /// Preserve mass numbers in bracketed terms such as `[13C]2` and `[2H]`.
    Separate,
}

impl Composition {
    /// Format exact counts in Hill order. An empty composition produces an empty string.
    ///
    /// With carbon, C comes first and H second; otherwise every symbol sorts alphabetically.
    /// Within each element, the unspecified isotope precedes increasing mass numbers.
    /// A net charge uses a final sign, followed by its magnitude when greater than one.
    pub fn hill_formula(&self, isotopes: FormulaIsotopes) -> Result<String, CompositionError> {
        let has_carbon = self
            .element_counts()
            .iter()
            .any(|row| row.element.get() == 6);
        let mut elements = Vec::with_capacity(self.element_counts().len());
        for row in self.element_counts() {
            let symbol = row
                .element
                .symbol()
                .ok_or(GraphError::InvalidElement(row.element.get()))?;
            let priority = match (has_carbon, symbol) {
                (true, "C") => 0,
                (true, "H") => 1,
                _ => 2,
            };
            elements.push((priority, symbol, row));
        }
        elements.sort_unstable_by_key(|&(priority, symbol, _)| (priority, symbol));

        let mut formula = String::new();
        for (_, symbol, row) in elements {
            match isotopes {
                FormulaIsotopes::Combined => {
                    formula.push_str(symbol);
                    append_count(&mut formula, row.count);
                }
                FormulaIsotopes::Separate => {
                    // Composition already groups isotope bins by atomic number and mass.
                    let bins = self.isotope_counts();
                    let start = bins.partition_point(|bin| bin.element.get() < row.element.get());
                    for bin in bins[start..]
                        .iter()
                        .take_while(|bin| bin.element == row.element)
                    {
                        if let Some(mass) = bin.isotope {
                            formula.push('[');
                            formula.push_str(&mass.to_string());
                            formula.push_str(symbol);
                            formula.push(']');
                        } else {
                            formula.push_str(symbol);
                        }
                        append_count(&mut formula, bin.count);
                    }
                }
            }
        }
        let charge = self.total_charge();
        if charge != 0 {
            formula.push(if charge > 0 { '+' } else { '-' });
            append_count(&mut formula, charge.unsigned_abs());
        }
        Ok(formula)
    }
}

fn append_count(output: &mut String, count: u64) {
    if count > 1 {
        output.push_str(&count.to_string());
    }
}
