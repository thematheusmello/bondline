//! Separate public-type declaration; missing DoublePosition is an API compiler RED.
use bondline_model::DoublePosition;
use serde_json::json;

#[test]
fn public_position_enum_has_four_exact_values_and_unspecified_default()
-> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(DoublePosition::default(), DoublePosition::Unspecified);
    for (position, spelling) in [
        (DoublePosition::Unspecified, "unspecified"),
        (DoublePosition::Center, "center"),
        (DoublePosition::Right, "right"),
        (DoublePosition::Left, "left"),
    ] {
        assert_eq!(serde_json::to_value(position)?, json!(spelling));
        assert_eq!(
            serde_json::from_value::<DoublePosition>(json!(spelling))?,
            position
        );
    }
    Ok(())
}
