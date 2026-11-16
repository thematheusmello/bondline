//! No-loss comparison control; no parser, writer or geometry assumptions.
use super::*;
use bondline_model::{Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn original(flag: Option<bool>) -> Result<Fragment, Box<dyn std::error::Error>> {
    let first_id = AtomId::new(29).ok_or("first atom")?;
    let second_id = AtomId::new(11).ok_or("second atom")?;
    let mut first = Atom::new(first_id, Element::try_from(6)?, None);
    let mut second = Atom::new(second_id, Element::try_from(6)?, None);
    first.explicit_hydrogens = Some(3);
    second.explicit_hydrogens = Some(3);
    first.show_non_terminal_carbon_labels = flag;
    Ok(Fragment::new(
        FragmentId::new(17).ok_or("fragment")?,
        vec![first, second],
        vec![Bond {
            id: BondId::new(41).ok_or("bond")?,
            begin: second_id,
            end: first_id,
            order: BondOrder::Single,
            double_position: bondline_model::DoublePosition::Unspecified,
        }],
    )?)
}

#[test]
fn mol_authored_comparison_distinguishes_every_flag_presence_and_value_without_mutation()
-> TestResult {
    for first in [None, Some(false), Some(true)] {
        for second in [None, Some(false), Some(true)] {
            let left = original(first)?;
            let right = original(second)?;
            let before_left = left.clone();
            let before_right = right.clone();
            assert_eq!(left.id(), right.id());
            assert_eq!(left.bonds(), right.bonds());
            // Qualify that the new datum is the ONLY difference between inputs.
            let mut corresponding = left.atoms().to_vec();
            corresponding[0].show_non_terminal_carbon_labels = second;
            assert_eq!(corresponding, right.atoms());
            assert_eq!(
                same_mol_authored_fields(&left, &right),
                first == second,
                "MOL comparison must retain None/no/yes distinction: {first:?} -> {second:?}"
            );
            assert_eq!(left, before_left);
            assert_eq!(right, before_right);
        }
    }
    Ok(())
}
