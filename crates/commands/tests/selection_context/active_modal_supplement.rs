//! Child of commands/tests/selection_context.rs; separate pre-fix supplement.
//! Adopt as tests/selection_context/active_modal_supplement.rs and append:
//! #[path = "selection_context/active_modal_supplement.rs"]
//! mod active_modal_supplement;
use super::*;

#[test]
fn active_atom_and_settings_modals_refuse_selection_context_until_exact_cancel() -> TestResult {
    for modal in ["atom", "settings"] {
        let directory = support::FileTestDirectory::new()?;
        let (mut engine, path) = named_pair(&directory)?;
        engine.set_ui_file_host(true);
        let before = wire(&engine)?;
        let content = engine.content().clone();
        let model = engine.stored_document()?.model_snapshot();
        let source = engine.capture_stamp();
        let file = engine.ui_file_context_stamp();
        let selected = engine.selection().clone();
        let tool = engine.bond_tool();
        let mode = engine.effective_tool();
        let view = engine.view();
        let bytes = fs::read(&path)?;
        let proof = engine.selection_context()?;
        assert!(proof.is_current(&engine));
        let revision = engine.state().revision;
        let (command, field, params) = if modal == "atom" {
            (
                "ui.atom_label",
                "ui_atom_label",
                json!({"action":"open", "expected_revision":revision,
                "page_id":1, "fragment_id":1, "atom_id":1}),
            )
        } else {
            (
                "ui.drawing_settings",
                "ui_drawing_settings",
                json!({"action":"open", "expected_revision":revision}),
            )
        };
        let opened = serde_json::to_value(engine.dispatch(command, params)?)?;
        let id = opened[field]["id"].as_u64().ok_or("actual modal request")?;
        assert!(id > 0);
        assert_eq!(opened[field]["current"], true);
        let active = wire(&engine)?;
        let error = engine
            .selection_context()
            .err()
            .ok_or("an active native modal must refuse a new selection context")?;
        assert_eq!(error.info().code, "ui_busy", "{modal}: {error}");
        assert!(
            !proof.is_current(&engine),
            "active {modal} must invalidate the older proof"
        );
        assert_eq!(
            wire(&engine)?,
            active,
            "context refusal cannot mutate the request or session"
        );
        for key in [
            "document",
            "revision",
            "undo_depth",
            "redo_depth",
            "source",
            "file",
            "ui_selection",
            "ui_file",
            "close",
            "close_epoch",
        ] {
            assert_eq!(active[key], before[key], "opening {modal} changed {key}");
        }
        assert_eq!(engine.content(), &content);
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
        assert!(engine.capture_stamp_matches(&source));
        assert!(engine.ui_file_context_stamp_matches(&file));
        assert_eq!(engine.selection(), &selected);
        assert_eq!(engine.bond_tool(), tool);
        assert_eq!(engine.effective_tool(), mode);
        assert_eq!(engine.view(), view);
        engine.dispatch(
            command,
            json!({"action":"cancel", "expected_revision":revision, "request_id":id}),
        )?;
        assert_eq!(wire(&engine)?, before);
        assert!(
            !proof.is_current(&engine),
            "Cancel cannot revive the prior proof"
        );
        assert!(
            engine.selection_context()?.is_current(&engine),
            "exact Cancel permits a fresh proof"
        );
        assert_eq!(engine.content(), &content);
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
        assert!(engine.capture_stamp_matches(&source));
        assert!(engine.ui_file_context_stamp_matches(&file));
        assert_eq!(engine.selection(), &selected);
        assert_eq!(engine.bond_tool(), tool);
        assert_eq!(engine.effective_tool(), mode);
        assert_eq!(engine.view(), view);
        assert_eq!(fs::read(&path)?, bytes);
    }
    Ok(())
}
