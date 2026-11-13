//! Atomic-number/symbol facts from the IUPAC periodic table, 4 May 2022.
//! https://iupac.org/wp-content/uploads/2022/05/IUPAC_Periodic_Table_A3-04May22.pdf
use bondline_model::Element;

const SYMBOLS: [&str; 118] = [
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
];
pub(super) fn element(symbol: &str) -> Option<Element> {
    SYMBOLS
        .iter()
        .position(|candidate| *candidate == symbol)
        .and_then(|index| u8::try_from(index + 1).ok())
        .and_then(Element::new)
}
pub(super) fn symbol(element: Element) -> Option<&'static str> {
    SYMBOLS
        .get(usize::from(element.get()).checked_sub(1)?)
        .copied()
}
