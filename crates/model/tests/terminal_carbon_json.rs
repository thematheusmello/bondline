//! Existing Atom serde API only: this must reach the intended unknown-field runtime RED.
use bondline_model::Atom;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn terminal_false_is_independent_owned_data_and_not_a_hydrogen_override() -> TestResult {
    let literal = json!({"id":29,"element":6,"position":{"x_pt":72.5,"y_pt":96.25},
        "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3,
        "show_terminal_carbon_labels":false,"show_non_terminal_carbon_labels":true});
    let atom: Atom = serde_json::from_value(literal.clone())?;
    assert_eq!(atom.id.get(), 29);
    assert_eq!(atom.element.get(), 6);
    assert_eq!(atom.explicit_hydrogens, Some(3));
    assert_eq!(atom.show_non_terminal_carbon_labels, Some(true));
    assert_eq!(serde_json::to_value(&atom)?, literal);
    Ok(())
}

// Additive declarations preserve the exact first862 bytes and use no future public field.
use bondline_model::{Bond, Document, Fragment, FragmentId, GraphError, PageId, Point2};
use serde_json::Value;
use std::sync::Arc;

fn literal(terminal: Option<Value>, nonterminal: Option<Value>) -> Value {
    let mut value = json!({"id":29,"element":6,"position":{"x_pt":72.5,"y_pt":96.25},
        "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3});
    if let Some(flag) = terminal {
        value["show_terminal_carbon_labels"] = flag;
    }
    if let Some(flag) = nonterminal {
        value["show_non_terminal_carbon_labels"] = flag;
    }
    value
}

fn other() -> Value {
    json!({"id":77,"element":8,"position":{"x_pt":120.5,"y_pt":96.25},
        "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":1})
}

fn edge() -> Value {
    json!({"id":41,"begin":77,"end":29,"order":"single"})
}

#[test]
fn both_tri_states_are_independent_through_graph_copies_without_inference_or_metadata_loss()
-> TestResult {
    let states = [
        None,
        Some(Value::Null),
        Some(json!(false)),
        Some(json!(true)),
    ];
    for terminal in &states {
        for nonterminal in &states {
            let input = literal(terminal.clone(), nonterminal.clone());
            let mut expected = input.clone();
            for name in [
                "show_terminal_carbon_labels",
                "show_non_terminal_carbon_labels",
            ] {
                if expected.get(name).is_some_and(Value::is_null) {
                    expected.as_object_mut().ok_or("atom")?.remove(name);
                }
            }
            let atom: Atom = serde_json::from_value(input)?;
            assert_eq!(serde_json::to_value(&atom)?, expected);
            assert_eq!(atom.explicit_hydrogens, Some(3));
            let fragment = Fragment::new(
                FragmentId::new(17).ok_or("fragment")?,
                vec![atom, serde_json::from_value(other())?],
                vec![serde_json::from_value::<Bond>(edge())?],
            )?;
            assert_eq!(
                serde_json::to_value(&fragment)?,
                json!({"id":17,
                "atoms":[expected.clone(),other()],"bonds":[edge()]})
            );
            let mut document = Document::default();
            let page = PageId::new(1).ok_or("page")?;
            document.append_fragments(page, vec![fragment.clone()])?;
            let old = document.clone();
            let owner = Arc::clone(&old.pages()[0].fragments()[0]);
            let mut atoms = fragment.atoms().to_vec();
            atoms[0].position = Some(Point2::new(96.5, 96.25)?);
            document.replace_fragment(
                page,
                Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
            )?;
            let mut moved = expected.clone();
            moved["position"]["x_pt"] = json!(96.5);
            assert_eq!(
                serde_json::to_value(&old)?["pages"][0]["fragments"][0]["atoms"][0],
                expected
            );
            assert_eq!(
                serde_json::to_value(&document)?["pages"][0]["fragments"][0],
                json!({"id":17,"atoms":[moved,other()],"bonds":[edge()]})
            );
            assert!(Arc::ptr_eq(&owner, &old.pages()[0].fragments()[0]));
            assert!(!Arc::ptr_eq(&owner, &document.pages()[0].fragments()[0]));
        }
    }
    for (element, h) in [(1, 0), (8, 1)] {
        let mut value = literal(Some(json!(false)), Some(json!(true)));
        value["element"] = json!(element);
        value["explicit_hydrogens"] = json!(h);
        assert_eq!(
            serde_json::to_value(serde_json::from_value::<Atom>(value.clone())?)?,
            value,
            "a dormant display datum cannot rewrite an element or literal H"
        );
    }
    Ok(())
}

#[test]
fn actual_boolean_fields_leave_closed_atom_and_graph_identity_rules_intact() -> TestResult {
    let value = literal(Some(json!(false)), Some(json!(true)));
    let atom: Atom = serde_json::from_value(value.clone())?;
    for name in [
        "show_terminal_carbon_labels",
        "show_non_terminal_carbon_labels",
    ] {
        for invalid in [
            json!(0),
            json!(1),
            json!(0.0),
            json!("false"),
            json!("true"),
            json!("yes"),
            json!("no"),
            json!(""),
            json!([]),
            json!({}),
        ] {
            let mut input = value.clone();
            input[name] = invalid;
            assert!(serde_json::from_value::<Atom>(input).is_err());
            assert_eq!(serde_json::to_value(&atom)?, value);
        }
    }
    for name in [
        "ShowTerminalCarbonLabels",
        "show_terminal_labels",
        "show_terminal_carbon_label",
        "ShowNonTerminalCarbonLabels",
        "unexpected",
    ] {
        let mut input = value.clone();
        input[name] = json!(true);
        assert!(serde_json::from_value::<Atom>(input).is_err());
    }
    for name in ["id", "element"] {
        let mut input = value.clone();
        input[name] = json!(0);
        assert!(serde_json::from_value::<Atom>(input).is_err());
    }
    let oxygen: Atom = serde_json::from_value(other())?;
    let bond: Bond = serde_json::from_value(edge())?;
    let mut self_edge = edge();
    self_edge["begin"] = json!(29);
    self_edge["end"] = json!(29);
    let mut missing = edge();
    missing["end"] = json!(99);
    let mut duplicate = edge();
    duplicate["id"] = json!(42);
    duplicate["begin"] = json!(29);
    duplicate["end"] = json!(77);
    for (atoms, bonds, expected) in [
        (
            vec![atom.clone(), atom.clone()],
            vec![],
            GraphError::DuplicateAtom(29),
        ),
        (
            vec![atom.clone(), oxygen.clone()],
            vec![serde_json::from_value(self_edge)?],
            GraphError::SelfBond(41),
        ),
        (
            vec![atom.clone(), oxygen.clone()],
            vec![serde_json::from_value(missing)?],
            GraphError::MissingAtom(99),
        ),
        (
            vec![atom.clone(), oxygen.clone()],
            vec![bond, bond],
            GraphError::DuplicateBond(41),
        ),
        (
            vec![atom.clone(), oxygen],
            vec![bond, serde_json::from_value(duplicate)?],
            GraphError::DuplicateConnection,
        ),
    ] {
        let error = Fragment::new(FragmentId::new(17).ok_or("fragment")?, atoms, bonds)
            .err()
            .ok_or("invalid graph accepted")?;
        assert_eq!(error, expected);
    }
    assert_eq!(serde_json::to_value(atom)?, value);
    Ok(())
}
