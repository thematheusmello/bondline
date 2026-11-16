//! Public field qualification is separate from the existing-serde runtime RED.
use bondline_model::{Atom, AtomId, Element};

#[test]
fn new_atom_defaults_to_none_and_public_flag_keeps_both_boolean_values()
-> Result<(), Box<dyn std::error::Error>> {
    let mut atom = Atom::new(AtomId::new(29).ok_or("atom")?, Element::try_from(6)?, None);
    assert_eq!(atom.show_non_terminal_carbon_labels, None);
    for flag in [false, true] {
        atom.show_non_terminal_carbon_labels = Some(flag);
        assert_eq!(atom.show_non_terminal_carbon_labels, Some(flag));
        assert_eq!(
            serde_json::to_value(&atom)?["show_non_terminal_carbon_labels"],
            flag
        );
    }
    atom.show_non_terminal_carbon_labels = None;
    assert!(
        serde_json::to_value(&atom)?
            .get("show_non_terminal_carbon_labels")
            .is_none()
    );
    Ok(())
}
