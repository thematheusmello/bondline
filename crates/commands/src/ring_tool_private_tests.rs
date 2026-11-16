//! Independent private epoch/type checks; Root owns the cfg-only bridge and execution.
use crate::{Command, EffectiveTool, Engine, SelectRingTool};
use serde_json::{Value, json};
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn pair() -> TestResult<Engine> {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,
        "begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
    )?;
    Ok(engine)
}
fn refused(engine: &mut Engine, command: Command, code: &str) -> TestResult {
    let before: Value = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let owner = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let current = Arc::clone(&engine.document);
    let saved = Arc::clone(&engine.saved);
    let ids = (
        engine.identities.atom,
        engine.identities.bond,
        engine.identities.fragment,
    );
    let mode = engine.effective_tool();
    let tool = engine.bond_tool();
    let view = engine.view();
    let generation = engine.session_generation();
    let file = engine.ui_file_context_stamp();
    let error = engine
        .execute(command)
        .err()
        .ok_or("exhausted/invalid mode change accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), owner);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(Arc::ptr_eq(&current, &engine.document));
    assert!(Arc::ptr_eq(&saved, &engine.saved));
    assert_eq!(
        (
            engine.identities.atom,
            engine.identities.bond,
            engine.identities.fragment
        ),
        ids
    );
    assert_eq!(engine.effective_tool(), mode);
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.view(), view);
    assert_eq!(engine.session_generation(), generation);
    assert!(engine.ui_file_context_stamp_matches(&file));
    Ok(())
}

#[test]
fn exhausted_single_tool_epoch_preserves_validated_ring_noops_and_every_mode_switch_atomically()
-> TestResult {
    let mut engine = pair()?;
    engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
    engine.bond_tool.epoch = u64::MAX;
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.capture_stamp();
    let mode = engine.effective_tool();
    let context = engine.ring_context(1)?;
    let result = engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
    assert_eq!(
        serde_json::to_value(result)?["ui_ring_tool_operation"],
        json!({"size":6,"epoch":u64::MAX,"changed":false})
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.capture_stamp(), owner);
    assert_eq!(engine.effective_tool(), mode);
    assert!(context.is_current(&engine));
    let revision = engine.state().revision;
    for command in [
        Command::UiRingToolSelect(SelectRingTool { size: 7 }),
        Command::parse("ui.bond_tool", json!({"order":"single"}))?,
        Command::parse(
            "ui.selection",
            json!({"action":"activate","expected_revision":revision}),
        )?,
    ] {
        refused(&mut engine, command, "ui_epoch_exhausted")?;
        assert!(context.is_current(&engine));
    }
    for size in [0, 2, 9, 255] {
        refused(
            &mut engine,
            Command::UiRingToolSelect(SelectRingTool { size }),
            "invalid_params",
        )?;
    }
    let cleared = engine.dispatch(
        "ui.selection",
        json!({"action":"clear","expected_revision":revision}),
    )?;
    assert_eq!(
        serde_json::to_value(cleared)?["ui_selection_operation"]["changed"],
        false
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(context.is_current(&engine));
    assert_eq!(engine.effective_tool(), EffectiveTool::Ring { size: 6 });
    Ok(())
}

#[test]
fn leaving_selection_preflights_selection_epoch_and_an_inactive_cache_is_not_authority()
-> TestResult {
    let mut engine = pair()?;
    engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[1,2]}),
    )?;
    assert_eq!(engine.effective_tool(), EffectiveTool::Selection);
    engine.selection.state.epoch = u64::MAX;
    let before = serde_json::to_value(engine.state())?;
    let context = engine.selection_context()?;
    let noop = engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[2,1]}),
    )?;
    assert_eq!(
        serde_json::to_value(noop)?["ui_selection_operation"]["changed"],
        false
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(context.is_current(&engine));
    // A remembered size6 is not a Ring no-op while Selection is the actual mode.
    refused(
        &mut engine,
        Command::UiRingToolSelect(SelectRingTool { size: 6 }),
        "ui_selection_epoch_exhausted",
    )?;
    refused(
        &mut engine,
        Command::parse("ui.bond_tool", json!({"order":"single"}))?,
        "ui_selection_epoch_exhausted",
    )?;
    assert_eq!(engine.effective_tool(), EffectiveTool::Selection);
    assert!(context.is_current(&engine));
    assert_eq!(engine.selection().atom_ids, [1, 2]);
    let mut inactive = pair()?;
    inactive.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
    inactive.selection.state.epoch = u64::MAX;
    assert!(!inactive.selection().active);
    let result = inactive.execute(Command::UiRingToolSelect(SelectRingTool { size: 7 }))?;
    assert_eq!(
        serde_json::to_value(result)?["ui_ring_tool_operation"],
        json!({"size":7,"epoch":2,"changed":true})
    );
    assert_eq!(inactive.effective_tool(), EffectiveTool::Ring { size: 7 });
    assert_eq!(inactive.selection().epoch, u64::MAX);
    assert!(!inactive.selection().active && inactive.selection().atom_ids.is_empty());
    Ok(())
}
