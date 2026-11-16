//! Independent private atomicity cases; Root owns the cfg-only child bridge.
use super::*;
use serde_json::{Value, json};
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn selected_pair_with_redo() -> TestResult<Engine> {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,
        "begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
    )?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":3,
        "page_id":1,"atom_ids":[1,2]}),
    )?;
    Ok(engine)
}

#[test]
fn exhausted_selection_epoch_keeps_valid_noops_and_every_changed_path_is_atomic() -> TestResult {
    let mut engine = selected_pair_with_redo()?;
    engine.selection.state.epoch = u64::MAX;
    let before: Value = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let current = Arc::clone(&engine.document);
    let saved = Arc::clone(&engine.saved);
    let generation = engine.session_generation();
    let tool = engine.bond_tool();
    let view = engine.view();
    let file = engine.ui_file_context_stamp();
    for (name, params) in [
        (
            "ui.selection",
            json!({"action":"set","expected_revision":3,
            "page_id":1,"atom_ids":[2,1]}),
        ),
        (
            "selection.translate",
            json!({"expected_revision":3,
            "expected_selection_epoch":u64::MAX,"delta_x_pt":0,"delta_y_pt":0}),
        ),
        (
            "atom.set_label",
            json!({"expected_revision":3,"page_id":1,"fragment_id":1,
            "atom_id":2,"element":6,"explicit_hydrogens":3}),
        ),
    ] {
        assert!(!engine.dispatch(name, params)?.changed);
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    for (name, params) in [
        (
            "ui.selection",
            json!({"action":"set","expected_revision":3,
            "page_id":1,"atom_ids":[1]}),
        ),
        (
            "ui.selection",
            json!({"action":"clear","expected_revision":3}),
        ),
        ("ui.bond_tool", json!({"order":"single"})),
        (
            "atom.set_label",
            json!({"expected_revision":3,"page_id":1,"fragment_id":1,
            "atom_id":2,"element":8,"explicit_hydrogens":1}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":72,
            "end_x_pt":268,"end_y_pt":72}),
        ),
        ("history.undo", json!({})),
        ("history.redo", json!({})),
        (
            "document.new",
            json!({"expected_revision":3,"discard_unsaved":true}),
        ),
    ] {
        let error = engine
            .dispatch(name, params)
            .err()
            .ok_or("epoch-exhausted change accepted")?;
        assert_eq!(
            error.info().code,
            "ui_selection_epoch_exhausted",
            "{name}: {error}"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert_eq!(engine.capture_stamp(), stamp);
        assert!(Arc::ptr_eq(&engine.document, &current));
        assert!(Arc::ptr_eq(&engine.saved, &saved));
        assert!(Arc::ptr_eq(
            &engine.stored_document()?.model_snapshot(),
            &model
        ));
        assert_eq!(engine.session_generation(), generation);
        assert_eq!(engine.bond_tool(), tool);
        assert_eq!(engine.view(), view);
        assert!(engine.ui_file_context_stamp_matches(&file));
    }
    // Restore only the injected test counter, then prove refused creation spent no IDs.
    engine.selection.state.epoch = 41;
    engine.dispatch(
        "ui.selection",
        json!({"action":"clear","expected_revision":3}),
    )?;
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":72,
        "end_x_pt":268,"end_y_pt":72}),
    )?;
    let fragment = &engine.document()?.pages()[0].fragments()[1];
    assert_eq!(fragment.id().get(), 2);
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|atom| atom.id.get())
            .collect::<Vec<_>>(),
        [3, 4]
    );
    assert_eq!(fragment.bonds()[0].id.get(), 2);
    Ok(())
}

#[test]
fn typed_ieee_nonfinite_displacements_refuse_before_mutation() -> TestResult {
    let mut engine = selected_pair_with_redo()?;
    let before: Value = serde_json::to_value(engine.state())?;
    let epoch = before["ui_selection"]["epoch"]
        .as_u64()
        .ok_or("selection epoch")?;
    let owner = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let saved = Arc::clone(&engine.saved);
    for (x, y) in [
        (f64::NAN, 0.0),
        (f64::INFINITY, 0.0),
        (f64::NEG_INFINITY, 0.0),
        (0.0, f64::NAN),
        (0.0, f64::INFINITY),
        (0.0, f64::NEG_INFINITY),
    ] {
        let error = engine
            .execute(Command::SelectionTranslate(SelectionTranslate {
                expected_revision: 3,
                expected_selection_epoch: epoch,
                delta_x_pt: x,
                delta_y_pt: y,
            }))
            .err()
            .ok_or("nonfinite translation accepted")?;
        assert_eq!(error.info().code, "invalid_params");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.capture_stamp(), owner);
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
        assert!(Arc::ptr_eq(&saved, &engine.saved));
    }
    Ok(())
}
