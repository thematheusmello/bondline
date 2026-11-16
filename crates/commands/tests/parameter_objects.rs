use bondline_commands::Command;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn every_known_command_requires_an_object_even_when_a_struct_can_read_a_sequence() -> TestResult {
    for command in bondline_commands::registry() {
        for value in [json!(null), json!(false), json!(42), json!(""), json!([])] {
            let error = Command::parse(command.name, value.clone())
                .err()
                .ok_or_else(|| {
                    format!("{} accepted non-object parameters {value}", command.name)
                })?;
            assert_eq!(error.info().code, "invalid_params");
        }
    }
    Ok(())
}

#[test]
fn a_sized_ring_field_sequence_does_not_bypass_the_parameter_object_contract() -> TestResult {
    let sequence = json!([0, 1, 6, "rectify_neutral_carbon", {
        "mode":"free", "begin_x_pt":120, "begin_y_pt":168,
        "end_x_pt":120, "end_y_pt":120, "side":"positive"
    }]);
    let error = Command::parse("ring.create", sequence)
        .err()
        .ok_or("a complete struct sequence was accepted as an object")?;
    assert_eq!(error.info().code, "invalid_params");
    Ok(())
}

#[test]
fn ordinary_objects_and_unknown_command_priority_remain_unchanged() -> TestResult {
    let inspect = Command::parse("document.inspect", json!({}))?;
    assert_eq!(
        serde_json::to_value(inspect)?,
        json!({"command":"document.inspect", "params":{}})
    );
    for value in [json!({}), json!([]), json!(null)] {
        let error = Command::parse("not_registered.command", value)
            .err()
            .ok_or("an unknown command was accepted")?;
        assert_eq!(error.info().code, "unknown_command");
    }
    Ok(())
}
