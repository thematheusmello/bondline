use super::*;
use serde_json::json;

#[test]
fn exhausted_edit_counters_preserve_noops_and_refuse_real_settings_changes_atomically()
-> Result<(), Box<dyn std::error::Error>> {
    for revision_exhausted in [true, false] {
        let mut engine = Engine::default();
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":144,"begin_y_pt":144,
                "end_x_pt":192,"end_y_pt":144}),
        )?;
        engine.dispatch(
            "ui.selection",
            json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[1,2]}),
        )?;
        if revision_exhausted {
            engine.revision = u64::MAX;
        } else {
            engine.selection.state.epoch = u64::MAX;
        }
        let before = serde_json::to_value(engine.state())?;
        let current = Arc::clone(&engine.document);
        let saved = Arc::clone(&engine.saved);
        let model = engine.stored_document()?.model_snapshot();
        let owner = engine.capture_stamp();
        let file = engine.ui_file_context_stamp();
        let focus = engine.view_focus;
        let identities = engine.identities;
        let revision = engine.revision;
        assert!(
            !engine
                .dispatch(
                    "document.set_drawing_settings",
                    json!({"expected_revision":revision,
                        "bond_length_pt":null,"chain_angle_degrees":null}),
                )?
                .changed
        );
        let error = engine
            .dispatch(
                "document.set_drawing_settings",
                json!({"expected_revision":revision,
                    "bond_length_pt":24.5,"chain_angle_degrees":120.5}),
            )
            .err()
            .ok_or("exhausted counter accepted an edit")?;
        assert_eq!(
            error.info().code,
            if revision_exhausted {
                "revision_exhausted"
            } else {
                "ui_selection_epoch_exhausted"
            }
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(Arc::ptr_eq(&engine.document, &current));
        assert!(Arc::ptr_eq(&engine.saved, &saved));
        assert!(Arc::ptr_eq(
            &engine.stored_document()?.model_snapshot(),
            &model
        ));
        assert!(engine.capture_stamp_matches(&owner));
        assert!(engine.ui_file_context_stamp_matches(&file));
        assert_eq!(engine.view_focus, focus);
        assert_eq!(
            (
                engine.identities.atom,
                engine.identities.bond,
                engine.identities.fragment
            ),
            (identities.atom, identities.bond, identities.fragment)
        );
    }
    Ok(())
}
