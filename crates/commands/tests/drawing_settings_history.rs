use bondline_commands::Engine;
use proptest::prelude::*;
use serde_json::json;
use std::sync::Arc;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn supported_pairs_keep_all_geometry_and_exact_undo_redo_snapshots(
        length in prop::option::of(1_u32..=1_600_000),
        angle in prop::option::of(0_u16..=1780),
    ) {
        let mut engine = Engine::default();
        engine.dispatch("bond.create",json!({"page_id":1,
            "begin_x_pt":144,"begin_y_pt":144,"end_x_pt":192,"end_y_pt":144}))?;
        let before = engine.content().clone();
        let old_model = engine.stored_document()?.model_snapshot();
        let state = serde_json::to_value(engine.state())?;
        let length = length.map(|value| f64::from(value)/16.0);
        let angle = angle.map(|value| 1.0+f64::from(value)/10.0);
        let result = engine.dispatch("document.set_drawing_settings",json!({
            "expected_revision":1,"bond_length_pt":length,"chain_angle_degrees":angle}))?;
        prop_assert_eq!(engine.document()?.pages(),old_model.pages());
        let settings=engine.document()?.drawing_settings();
        prop_assert_eq!(settings.bond_length_pt(),length);
        prop_assert_eq!(settings.chain_angle_degrees(),angle);
        if result.changed {
            let after=engine.content().clone();
            let new_model=engine.stored_document()?.model_snapshot();
            engine.dispatch("history.undo",json!({}))?;
            prop_assert_eq!(engine.content(),&before);
            prop_assert!(Arc::ptr_eq(&engine.stored_document()?.model_snapshot(),&old_model));
            engine.dispatch("history.redo",json!({}))?;
            prop_assert_eq!(engine.content(),&after);
            prop_assert!(Arc::ptr_eq(&engine.stored_document()?.model_snapshot(),&new_model));
        } else {
            prop_assert!(length.is_none() && angle.is_none());
            prop_assert_eq!(serde_json::to_value(engine.state())?,state);
            prop_assert!(Arc::ptr_eq(&engine.stored_document()?.model_snapshot(),&old_model));
        }
    }
}
