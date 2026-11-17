//! Existing normal-ring weak context must not revive when another form returns to idle.
use bondline_commands::Engine;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn modal_aba(form: &str) -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":120,
        "end_x_pt":120,"end_y_pt":120,"order":"single"}),
    )?;
    engine.dispatch("ui.ring_tool", json!({"size":6}))?;
    let proof = engine.ring_context(1)?;
    assert!(proof.is_current(&engine));
    let before = serde_json::to_value(engine.state())?;
    let whole = engine.content().clone();
    let stamp = engine.capture_stamp();
    let tool = engine.bond_tool();
    let view = engine.view();
    let id = if form == "ui.atom_label" {
        engine.dispatch(
            form,
            json!({"action":"open","expected_revision":1,
            "page_id":1,"fragment_id":1,"atom_id":1}),
        )?;
        engine.ui_atom_label_request().ok_or("Atom owner")?.id
    } else {
        engine.dispatch(form, json!({"action":"open","expected_revision":1}))?;
        engine
            .ui_drawing_settings_request()
            .ok_or("settings owner")?
            .id
    };
    engine.dispatch(
        form,
        json!({"action":"cancel","expected_revision":1,"request_id":id}),
    )?;
    assert!(engine.ui_atom_label_request().is_none());
    assert!(engine.ui_drawing_settings_request().is_none());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &whole);
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.view(), view);
    assert!(
        !proof.is_current(&engine),
        "{form} Open/Cancel must permanently stale the old ring proof"
    );
    assert!(engine.ring_context(1)?.is_current(&engine));
    Ok(())
}

#[test]
fn normal_ring_context_cannot_revive_after_atom_open_cancel() -> TestResult {
    modal_aba("ui.atom_label")
}

#[test]
fn normal_ring_context_cannot_revive_after_settings_open_cancel() -> TestResult {
    modal_aba("ui.drawing_settings")
}
