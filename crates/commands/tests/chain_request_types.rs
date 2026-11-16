//! Public typed/schema contract is separate from the first existing-JSON runtime RED.
use bondline_commands::{
    ChainCreate, ChainPlacement, ChainSide, Command, Engine, OrderHydrogenPolicy,
};
use serde_json::json;
use std::sync::Arc;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn request(count: u8, angle: f64) -> ChainCreate {
    ChainCreate {
        expected_revision: 0,
        page_id: 1,
        count,
        angle_degrees: angle,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement: ChainPlacement::Free {
            begin_x_pt: 64.0,
            begin_y_pt: 112.0,
            end_x_pt: 112.0,
            end_y_pt: 112.0,
            side: ChainSide::Positive,
        },
    }
}
#[test]
fn public_request_types_preserve_exact_closed_wire_fields_and_bounds() -> TestResult {
    let value = serde_json::to_value(request(3, 120.0))?;
    assert_eq!(
        value,
        json!({"expected_revision":0,"page_id":1,"count":3,"angle_degrees":120.0,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free",
        "begin_x_pt":64.0,"begin_y_pt":112.0,"end_x_pt":112.0,"end_y_pt":112.0,"side":"positive"}})
    );
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<ChainCreate>(value.clone())?)?,
        value
    );
    let atom = ChainPlacement::Atom {
        fragment_id: 17,
        atom_id: 29,
        end_x_pt: 192.0,
        end_y_pt: 128.0,
        side: ChainSide::Negative,
    };
    assert_eq!(
        serde_json::to_value(atom)?,
        json!({"mode":"atom","fragment_id":17,"atom_id":29,
        "end_x_pt":192.0,"end_y_pt":128.0,"side":"negative"})
    );
    for bad in [
        serde_json::Value::Null,
        json!({"positive":null}),
        json!(["positive"]),
        json!("other"),
    ] {
        assert!(serde_json::from_value::<ChainSide>(bad).is_err());
    }
    let mut wrapped = value.clone();
    wrapped["hydrogen_policy"] = json!({"rectify_neutral_carbon":null});
    assert!(
        serde_json::from_value::<ChainCreate>(wrapped).is_err(),
        "strict bridge is scoped to the new request"
    );
    let schema = serde_json::to_value(schemars::schema_for!(ChainCreate))?;
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["count"]["minimum"].as_f64(), Some(2.0));
    assert_eq!(
        schema["properties"]["count"]["maximum"].as_f64(),
        Some(64.0)
    );
    assert_eq!(
        schema["properties"]["angle_degrees"]["minimum"].as_f64(),
        Some(1.0)
    );
    assert_eq!(
        schema["properties"]["angle_degrees"]["maximum"].as_f64(),
        Some(179.0)
    );
    Ok(())
}
#[test]
fn typed_nonfinite_or_out_of_range_requests_validate_even_when_count_is_two() -> TestResult {
    let mut engine = Engine::default();
    for (count, angle) in [
        (2, f64::NAN),
        (2, f64::INFINITY),
        (2, f64::NEG_INFINITY),
        (2, 0.0),
        (2, 180.0),
        (0, 120.0),
        (1, 120.0),
        (65, 120.0),
    ] {
        let before = serde_json::to_value(engine.state())?;
        let stamp = engine.capture_stamp();
        let owner = engine.stored_document()?.model_snapshot();
        let error = engine
            .execute(Command::ChainCreate(request(count, angle)))
            .err()
            .ok_or("typed invalid request accepted")?;
        assert_eq!(error.info().code, "invalid_params", "{error}");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.capture_stamp(), stamp);
        assert!(Arc::ptr_eq(
            &owner,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    Ok(())
}
