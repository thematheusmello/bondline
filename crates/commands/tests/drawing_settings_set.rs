//! Literal command expectations through the existing JSON API, before producer.
use bondline_commands::Engine;
use serde_json::{Value, json};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn explicit_pair_changes_once_without_moving_existing_content() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":144,"begin_y_pt":144,
            "end_x_pt":192,"end_y_pt":144}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let result = engine.dispatch(
        "document.set_drawing_settings",
        json!({"expected_revision":1,"bond_length_pt":24.5,"chain_angle_degrees":120.5}),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 2, 0)
    );
    assert!(result.file.dirty && result.source.is_none());
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    let actual: Value = serde_json::to_value(result)?;
    assert_eq!(actual["document"]["pages"], before["document"]["pages"]);
    assert_eq!(
        actual["document"]["drawing_settings"],
        json!({"bond_length_pt":24.5,"chain_angle_degrees":120.5})
    );
    Ok(())
}
