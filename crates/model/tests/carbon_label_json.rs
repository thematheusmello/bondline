//! Existing Atom serde API: explicit false reaches an old runtime unknown-field error.
use bondline_model::{Atom, Radical};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn explicit_false_is_owned_data_and_legacy_absence_stays_omitted() -> TestResult {
    let literal = json!({"id":29,"element":6,"position":{"x_pt":100.25,"y_pt":200.5},
        "isotope":13,"charge":-1,"radical":"doublet","explicit_hydrogens":0,
        "show_non_terminal_carbon_labels":false});
    let atom: Atom = serde_json::from_value(literal.clone())?;
    assert_eq!(atom.id.get(), 29);
    assert_eq!(atom.element.get(), 6);
    let p = atom.position.ok_or("position")?;
    assert_eq!([p.x_pt(), p.y_pt()], [100.25, 200.5]);
    assert_eq!(atom.isotope.map(|mass| mass.get()), Some(13));
    assert_eq!(atom.charge, -1);
    assert_eq!(atom.radical, Radical::Doublet);
    assert_eq!(atom.explicit_hydrogens, Some(0));
    assert_eq!(serde_json::to_value(&atom)?, literal);
    let mut old = literal;
    old.as_object_mut()
        .ok_or("atom")?
        .remove("show_non_terminal_carbon_labels");
    let unspecified: Atom = serde_json::from_value(old.clone())?;
    assert_eq!(serde_json::to_value(unspecified)?, old);
    Ok(())
}

// Keep the exact frozen1,232-byte initial serde declaration as the complete prefix.
use bondline_model::{Bond, Document, Fragment, FragmentId, GraphError, PageId};
use serde_json::Value;

fn marked(flag: Value) -> Value {
    json!({"id":29,"element":6,"position":{"x_pt":100.25,"y_pt":200.5},
        "isotope":13,"charge":-1,"radical":"doublet","explicit_hydrogens":0,
        "show_non_terminal_carbon_labels":flag})
}
fn other() -> Value {
    json!({"id":77,"element":8,"position":{"x_pt":148.25,"y_pt":200.5},
        "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":1})
}
fn edge() -> Value {
    json!({"id":41,"begin":77,"end":29,"order":"single"})
}

#[test]
fn absent_null_and_both_booleans_keep_distinct_presence_and_all_fragment_data() -> TestResult {
    for input_flag in [
        None,
        Some(Value::Null),
        Some(json!(false)),
        Some(json!(true)),
    ] {
        let mut literal = marked(Value::Null);
        match &input_flag {
            Some(flag) => literal["show_non_terminal_carbon_labels"] = flag.clone(),
            None => {
                literal
                    .as_object_mut()
                    .ok_or("atom")?
                    .remove("show_non_terminal_carbon_labels");
            }
        }
        let atom: Atom = serde_json::from_value(literal.clone())?;
        let mut expected = literal;
        if input_flag.as_ref().is_none_or(Value::is_null) {
            expected
                .as_object_mut()
                .ok_or("atom")?
                .remove("show_non_terminal_carbon_labels");
        }
        assert_eq!(serde_json::to_value(&atom)?, expected);
        let fragment = Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            vec![atom, serde_json::from_value(other())?],
            vec![serde_json::from_value::<Bond>(edge())?],
        )?;
        assert_eq!(
            serde_json::to_value(&fragment)?,
            json!({"id":17,"atoms":[expected,other()],"bonds":[edge()]})
        );
        let mut document = Document::default();
        let page = PageId::new(1).ok_or("page")?;
        document.append_fragments(page, vec![fragment.clone()])?;
        let old = document.clone();
        let mut atoms = fragment.atoms().to_vec();
        atoms.first_mut().ok_or("atom")?.position =
            Some(bondline_model::Point2::new(124.25, 200.5)?);
        document.replace_fragment(
            page,
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
        )?;
        let mut moved = expected.clone();
        moved["position"]["x_pt"] = json!(124.25);
        assert_eq!(
            serde_json::to_value(&old)?["pages"][0]["fragments"][0]["atoms"][0],
            expected
        );
        assert_eq!(
            serde_json::to_value(&document)?["pages"][0]["fragments"][0]["atoms"][0],
            moved
        );
        assert_eq!(
            serde_json::to_value(&document)?["pages"][0]["fragments"][0]["atoms"][1],
            other()
        );
    }
    // The model retains this datum even when its eventual display role is dormant.
    let mut oxygen = other();
    oxygen["show_non_terminal_carbon_labels"] = json!(false);
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<Atom>(oxygen.clone())?)?,
        oxygen
    );
    Ok(())
}

#[test]
fn label_flag_accepts_actual_booleans_only_and_does_not_open_atom_json() -> TestResult {
    let literal = marked(json!(false));
    let atom: Atom = serde_json::from_value(literal.clone())?;
    for value in [
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
        assert!(serde_json::from_value::<Atom>(marked(value)).is_err());
        assert_eq!(serde_json::to_value(&atom)?, literal);
    }
    for name in [
        "ShowNonTerminalCarbonLabels",
        "show_terminal_carbon_labels",
        "unexpected",
    ] {
        let mut input = literal.clone();
        input[name] = json!(true);
        assert!(serde_json::from_value::<Atom>(input).is_err());
    }
    for name in ["id", "element"] {
        let mut input = literal.clone();
        input[name] = json!(0);
        assert!(serde_json::from_value::<Atom>(input).is_err());
    }
    assert_eq!(serde_json::to_value(atom)?, literal);
    Ok(())
}

#[test]
fn specified_atom_flag_does_not_bypass_fragment_connection_or_identity_validation() -> TestResult {
    let atom: Atom = serde_json::from_value(marked(json!(true)))?;
    let other: Atom = serde_json::from_value(other())?;
    let bond: Bond = serde_json::from_value(edge())?;
    let mut self_bond = edge();
    self_bond["begin"] = json!(29);
    self_bond["end"] = json!(29);
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
            vec![atom.clone(), other.clone()],
            vec![serde_json::from_value(self_bond)?],
            GraphError::SelfBond(41),
        ),
        (
            vec![atom.clone(), other.clone()],
            vec![serde_json::from_value(missing)?],
            GraphError::MissingAtom(99),
        ),
        (
            vec![atom.clone(), other.clone()],
            vec![bond, bond],
            GraphError::DuplicateBond(41),
        ),
        (
            vec![atom.clone(), other],
            vec![bond, serde_json::from_value(duplicate)?],
            GraphError::DuplicateConnection,
        ),
    ] {
        let error = Fragment::new(FragmentId::new(17).ok_or("fragment")?, atoms, bonds)
            .err()
            .ok_or("invalid fragment accepted")?;
        assert_eq!(error, expected);
    }
    assert_eq!(serde_json::to_value(atom)?, marked(json!(true)));
    Ok(())
}
