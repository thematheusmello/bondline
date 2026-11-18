//! Existing public serde API: the baseline reaches a runtime unknown-field error.
use bondline_model::{Atom, Bond, BondOrder, Fragment, FragmentId};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn directed_right_position_survives_model_json_and_missing_position_stays_omitted() -> TestResult {
    let specified = json!({"id":41,"begin":29,"end":11,"order":"double","double_position":"right"});
    let bond: Bond = serde_json::from_value(specified.clone())?;
    assert_eq!(
        (bond.id.get(), bond.begin.get(), bond.end.get()),
        (41, 29, 11)
    );
    assert_eq!(bond.order, BondOrder::Double);
    assert_eq!(serde_json::to_value(bond)?, specified);
    let atoms = json!([
        {"id":29,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,
         "charge":0,"radical":"none","explicit_hydrogens":2},
        {"id":11,"element":6,"position":{"x_pt":164.0,"y_pt":200.0},"isotope":null,
         "charge":0,"radical":"none","explicit_hydrogens":2}
    ]);
    let fragment = Fragment::new(
        FragmentId::new(17).ok_or("fragment id")?,
        serde_json::from_value::<Vec<Atom>>(atoms.clone())?,
        vec![bond],
    )?;
    assert_eq!(
        serde_json::to_value(fragment)?,
        json!({"id":17,"atoms":atoms,"bonds":[specified]})
    );
    let old = json!({"id":41,"begin":29,"end":11,"order":"double"});
    let unspecified: Bond = serde_json::from_value(old.clone())?;
    assert_eq!(serde_json::to_value(unspecified)?, old);
    Ok(())
}

// The entire frozen 1,493-byte first declaration remains the prefix above.
use bondline_model::GraphError;
use serde_json::Value;

fn two_atoms() -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
    Ok(serde_json::from_value(json!([
        {"id":29,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,
         "charge":0,"radical":"none","explicit_hydrogens":null},
        {"id":11,"element":6,"position":{"x_pt":164.0,"y_pt":200.0},"isotope":null,
         "charge":0,"radical":"none","explicit_hydrogens":null}
    ]))?)
}

#[test]
fn every_position_default_and_dormant_property_keeps_exact_direction_and_data() -> TestResult {
    let atoms = two_atoms()?;
    let original_atoms = serde_json::to_value(&atoms)?;
    for order in ["double", "single", "triple", "aromatic"] {
        for position in [
            None,
            Some("unspecified"),
            Some("center"),
            Some("right"),
            Some("left"),
        ] {
            for (begin, end) in [(29, 11), (11, 29)] {
                let mut input = json!({"id":41,"begin":begin,"end":end,"order":order});
                if let Some(position) = position {
                    input["double_position"] = json!(position);
                }
                let bond: Bond = serde_json::from_value(input.clone())?;
                let mut expected = input;
                if position == Some("unspecified") {
                    expected
                        .as_object_mut()
                        .ok_or("bond object")?
                        .remove("double_position");
                }
                assert_eq!(serde_json::to_value(bond)?, expected);
                let fragment = Fragment::new(
                    FragmentId::new(17).ok_or("fragment")?,
                    atoms.clone(),
                    vec![bond],
                )?;
                assert_eq!(
                    serde_json::to_value(&fragment)?,
                    json!({"id":17,"atoms":original_atoms,"bonds":[expected]})
                );
                assert_eq!(serde_json::to_value(fragment.atoms())?, original_atoms);
            }
        }
    }
    Ok(())
}

#[test]
fn invalid_position_values_and_extra_bond_fields_remain_closed_errors() -> TestResult {
    let valid = json!({"id":41,"begin":29,"end":11,"order":"double","double_position":"right"});
    let before: Bond = serde_json::from_value(valid.clone())?;
    for invalid in [
        Value::Null,
        json!(true),
        json!(0),
        json!(1.0),
        json!([]),
        json!({}),
        json!("Right"),
        json!("RIGHT"),
        json!("auto"),
        json!(""),
    ] {
        let mut candidate = valid.clone();
        candidate["double_position"] = invalid;
        assert!(serde_json::from_value::<Bond>(candidate).is_err());
        assert_eq!(serde_json::to_value(before)?, valid);
    }
    let mut extra = valid.clone();
    extra["automatic"] = json!(true);
    assert!(serde_json::from_value::<Bond>(extra).is_err());
    for field in ["id", "begin", "end", "order"] {
        let mut missing = valid.clone();
        missing.as_object_mut().ok_or("bond")?.remove(field);
        assert!(serde_json::from_value::<Bond>(missing).is_err());
    }
    assert_eq!(serde_json::to_value(before)?, valid);
    Ok(())
}

#[test]
fn placement_data_does_not_bypass_structural_fragment_validation() -> TestResult {
    let atoms = two_atoms()?;
    let right = json!({"id":41,"begin":29,"end":11,"order":"double","double_position":"right"});
    let bond: Bond = serde_json::from_value(right.clone())?;
    let mut self_bond = right.clone();
    self_bond["end"] = json!(29);
    let mut missing = right.clone();
    missing["end"] = json!(99);
    let mut reverse_duplicate = right.clone();
    reverse_duplicate["id"] = json!(42);
    reverse_duplicate["begin"] = json!(11);
    reverse_duplicate["end"] = json!(29);
    for (bonds, expected) in [
        (
            vec![serde_json::from_value(self_bond)?],
            GraphError::SelfBond(41),
        ),
        (
            vec![serde_json::from_value(missing)?],
            GraphError::MissingAtom(99),
        ),
        (vec![bond, bond], GraphError::DuplicateBond(41)),
        (
            vec![bond, serde_json::from_value(reverse_duplicate)?],
            GraphError::DuplicateConnection,
        ),
    ] {
        let error = Fragment::new(FragmentId::new(17).ok_or("fragment")?, atoms.clone(), bonds)
            .err()
            .ok_or("invalid connection accepted")?;
        assert_eq!(error, expected);
    }
    assert_eq!(serde_json::to_value(bond)?, right);
    assert_eq!(
        serde_json::to_value(two_atoms()?)?,
        serde_json::to_value(atoms)?
    );
    Ok(())
}
