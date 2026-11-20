//! Chemical composition from one notation graph, independent of drawing preparation.
use super::{BondOrder, Hydrogens, SmilesDocument};
use bondline_chem::{Composition, CompositionError, opensmiles_implicit_hydrogens};
use bondline_model::{Atom, AtomId};
use bondline_render::Cancellation;
use std::num::NonZeroU16;

#[derive(Debug)]
pub struct RecordComposition {
    pub composition: Composition,
    pub atom_count: usize,
    pub component_count: usize,
    pub defaulted_hydrogen_atoms: usize,
}

// The parser owns both endpoints of each edge. Union by representative counts
// connected components without building or laying out a Fragment for each one.
fn representative(parents: &mut [usize], mut index: usize) -> usize {
    while parents[index] != index {
        parents[index] = parents[parents[index]];
        index = parents[index];
    }
    index
}

impl SmilesDocument {
    /// Read one complete record. Unresolved isotope zero and aromatic notation
    /// retain their source representation and produce no chemical result.
    pub fn record_composition(
        &self,
        cancel: &Cancellation,
    ) -> Result<Option<RecordComposition>, CompositionError> {
        let check = || {
            if cancel.is_cancelled() {
                Err(CompositionError::Cancelled)
            } else {
                Ok(())
            }
        };
        check()?;
        let atom_count = self.atoms().len();
        let mut parents: Vec<_> = (0..atom_count).collect();
        let mut occupied = vec![0_u16; atom_count];
        let mut component_count = atom_count;
        for bond in self.bonds() {
            check()?;
            let weight = match bond.order {
                BondOrder::Single => 1,
                BondOrder::Double => 2,
                BondOrder::Triple => 3,
                BondOrder::Aromatic => return Ok(None),
            };
            let [begin, end] = bond.atoms;
            for id in [begin, end] {
                occupied[id] = occupied[id]
                    .checked_add(weight)
                    .ok_or(CompositionError::ArithmeticOverflow)?;
            }
            let a = representative(&mut parents, begin);
            let b = representative(&mut parents, end);
            if a != b {
                parents[b] = a;
                component_count -= 1;
            }
        }
        let mut atoms = Vec::with_capacity(atom_count);
        let mut defaulted_hydrogen_atoms = 0;
        for (index, source) in self.atoms().iter().enumerate() {
            check()?;
            if source.aromatic || source.isotope == Some(0) {
                return Ok(None);
            }
            let count = match source.hydrogens {
                Hydrogens::Bracket(h) => u16::from(h),
                Hydrogens::Implicit => {
                    let Some(h) = opensmiles_implicit_hydrogens(source.element, occupied[index])
                    else {
                        return Ok(None);
                    };
                    defaulted_hydrogen_atoms += 1;
                    h
                }
            };
            let id = u32::try_from(index + 1)
                .ok()
                .and_then(AtomId::new)
                .ok_or(CompositionError::ArithmeticOverflow)?;
            let mut atom = Atom::new(id, source.element, None);
            atom.charge = source.charge;
            atom.isotope = source.isotope.and_then(NonZeroU16::new);
            atom.explicit_hydrogens = Some(count);
            atoms.push(atom);
        }
        let composition = Composition::from_atoms(&atoms, || cancel.is_cancelled())?;
        Ok(Some(RecordComposition {
            composition,
            atom_count,
            component_count,
            defaulted_hydrogen_atoms,
        }))
    }
}
