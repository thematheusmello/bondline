//! Child of commands/tests/selection_context.rs; public commands/proofs only.
use super::*;

#[test]
fn hidden_atom_settings_and_preferences_cycles_stale_selection_without_other_context_changes()
-> TestResult {
    for modal in ["atom", "settings", "preferences"] {
        let directory = support::FileTestDirectory::new()?;
        let (mut engine, path) = named_pair(&directory)?;
        engine.set_ui_file_host(true);
        let raw = fs::read(&path)?;
        let before = wire(&engine)?;
        let content = engine.content().clone();
        let source = engine.capture_stamp();
        let model = engine.stored_document()?.model_snapshot();
        let file = engine.ui_file_context_stamp();
        let tool = engine.bond_tool();
        let mode = engine.effective_tool();
        let view = engine.view();
        let selected = engine.selection().clone();
        let proof = engine.selection_context()?;
        assert!(
            proof.is_current(&engine),
            "ordinary no-loss proof must begin current"
        );
        engine.dispatch("ui.view", json!({"action":"inspect"}))?;
        assert!(proof.is_current(&engine), "inspection is a no-loss control");
        let revision = engine.state().revision;
        let (command, field, params) = match modal {
            "atom" => (
                "ui.atom_label",
                "ui_atom_label",
                json!({"action":"open","expected_revision":revision,
                "page_id":1,"fragment_id":1,"atom_id":1}),
            ),
            "settings" => (
                "ui.drawing_settings",
                "ui_drawing_settings",
                json!({"action":"open","expected_revision":revision}),
            ),
            "preferences" => (
                "ui.preferences",
                "ui_preferences",
                json!({"action":"open","expected_revision":revision}),
            ),
            _ => return Err("modal".into()),
        };
        let opened = serde_json::to_value(engine.dispatch(command, params)?)?;
        let id = opened[field]["id"].as_u64().ok_or("actual request ID")?;
        assert_eq!(opened[field]["current"], true);
        engine.dispatch(
            command,
            json!({"action":"cancel","expected_revision":revision,"request_id":id}),
        )?;
        assert_eq!(
            wire(&engine)?,
            before,
            "hidden cycle changes no normal session fields"
        );
        assert_eq!(engine.content(), &content);
        assert!(engine.capture_stamp_matches(&source));
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
        assert!(engine.ui_file_context_stamp_matches(&file));
        assert_eq!(engine.bond_tool(), tool);
        assert_eq!(engine.effective_tool(), mode);
        assert_eq!(engine.view(), view);
        assert_eq!(engine.selection(), &selected);
        assert_eq!(fs::read(path)?, raw);
        assert!(
            !proof.is_current(&engine),
            "hidden {modal} Open→Cancel must permanently stale the old proof"
        );
        assert!(
            engine.selection_context()?.is_current(&engine),
            "fresh proof remains available"
        );
    }
    Ok(())
}
