use bondline_commands::Engine;
use serde_json::json;
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated filesystem helper")]
mod support;

#[test]
fn session_namespace_changes_on_new_and_open_but_not_edits_history_or_navigation()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    assert_eq!(engine.session_generation(), 0);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.session_generation(), 0);
    engine.dispatch(
        "document.new",
        json!({"expected_revision":3,"discard_unsaved":true}),
    )?;
    assert_eq!(engine.session_generation(), 1);
    engine.dispatch("document.new", json!({"expected_revision":4}))?;
    assert_eq!(engine.session_generation(), 2);
    engine.dispatch("ui.view", json!({"action":"actual"}))?;
    engine.dispatch("ui.bond_tool", json!({"order":"triple"}))?;
    assert_eq!(engine.session_generation(), 2);

    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("namespace.sdf");
    let source = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
    fs::write(&input, source)?;
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    assert_eq!(engine.session_generation(), 3);
    engine.dispatch(
        "sdf.record.remove",
        json!({"expected_revision":6,"record_id":1}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.session_generation(), 3);
    engine.dispatch("document.new", json!({"expected_revision":8}))?;
    assert_eq!(engine.session_generation(), 4);
    assert_eq!(fs::read(input)?, source);
    Ok(())
}
