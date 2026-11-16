//! New public-field qualification stays separate from existing-API runtime declarations.
use bondline_model::{Atom, AtomId, Element};

#[test]
fn public_terminal_field_defaults_none_and_never_changes_the_other_flag_or_literal_h()
-> Result<(), Box<dyn std::error::Error>> {
    let mut atom = Atom::new(AtomId::new(29).ok_or("atom")?, Element::try_from(6)?, None);
    atom.explicit_hydrogens = Some(4);
    atom.show_non_terminal_carbon_labels = Some(true);
    assert_eq!(atom.show_terminal_carbon_labels, None);
    for flag in [false, true] {
        atom.show_terminal_carbon_labels = Some(flag);
        assert_eq!(atom.show_terminal_carbon_labels, Some(flag));
        assert_eq!(atom.show_non_terminal_carbon_labels, Some(true));
        assert_eq!(atom.explicit_hydrogens, Some(4));
        let encoded = serde_json::to_string(&atom)?;
        assert!(
            encoded
                .find("show_terminal_carbon_labels")
                .ok_or("terminal")?
                < encoded
                    .find("show_non_terminal_carbon_labels")
                    .ok_or("nonterminal")?
        );
        assert_eq!(
            serde_json::to_value(&atom)?["show_terminal_carbon_labels"],
            flag
        );
    }
    atom.show_terminal_carbon_labels = None;
    assert!(
        serde_json::to_value(&atom)?
            .get("show_terminal_carbon_labels")
            .is_none()
    );
    assert_eq!(atom.show_non_terminal_carbon_labels, Some(true));
    assert_eq!(atom.explicit_hydrogens, Some(4));
    Ok(())
}
