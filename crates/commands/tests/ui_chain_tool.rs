//! Independent existing-JSON native-chain declaration; no future Rust API is referenced.
use bondline_commands::Engine;
use serde_json::json;
use std::sync::Arc;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn selecting_acyclic_is_transient_and_needs_no_native_host() -> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let model = engine.stored_document()?.model_snapshot();
    let source = engine.capture_stamp();
    let view = engine.view();
    let order = engine.bond_tool().order;
    let result = engine.dispatch("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))?;
    assert!(!result.changed);
    let mut actual = serde_json::to_value(result)?;
    let receipt = actual
        .as_object_mut()
        .ok_or("command result")?
        .remove("ui_chain_tool_operation")
        .ok_or("chain tool receipt")?;
    assert_eq!(
        receipt,
        json!({"kind":"acyclic","fixed_lengths":true,
        "fixed_angles":true,"epoch":1,"changed":true})
    );
    assert_eq!(
        actual, before,
        "selection must preserve every ordinary session field"
    );
    assert_eq!(engine.content(), &content);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(engine.capture_stamp_matches(&source));
    assert_eq!(engine.view(), view);
    assert_eq!(engine.bond_tool().order, order);
    assert_eq!(engine.bond_tool().epoch, 1);
    assert!(!engine.state().ui_file.available);
    Ok(())
}

// Bounded independent continuation. The initial declaration above remains exact.
fn chain_tool_step(
    engine: &mut Engine,
    params: serde_json::Value,
    kind: &str,
    lengths: bool,
    angles: bool,
    epoch: u64,
    changed: bool,
) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let source = engine.capture_stamp();
    let view = engine.view();
    let remembered_order = engine.bond_tool().order;
    let result = engine.dispatch("ui.chain_tool", params)?;
    assert!(!result.changed);
    let mut actual = serde_json::to_value(result)?;
    let receipt = actual
        .as_object_mut()
        .ok_or("command result")?
        .remove("ui_chain_tool_operation")
        .ok_or("chain tool receipt")?;
    assert_eq!(
        receipt,
        json!({"kind":kind,"fixed_lengths":lengths,
        "fixed_angles":angles,"epoch":epoch,"changed":changed})
    );
    assert_eq!(
        actual, before,
        "tool and constraints must not author or acknowledge Save"
    );
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&source));
    assert_eq!(engine.view(), view);
    assert_eq!(engine.bond_tool().order, remembered_order);
    assert_eq!(engine.bond_tool().epoch, epoch);
    Ok(())
}

#[test]
fn constraints_share_the_tool_epoch_and_survive_mode_new_and_real_open_without_authoring()
-> TestResult {
    let mut engine = Engine::default();
    engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
    let remembered = engine.bond_tool().order;
    chain_tool_step(
        &mut engine,
        json!({"action":"select","kind":"acyclic"}),
        "acyclic",
        true,
        true,
        2,
        true,
    )?;
    chain_tool_step(
        &mut engine,
        json!({"action":"constraints","fixed_lengths":false,"fixed_angles":true}),
        "acyclic",
        false,
        true,
        3,
        true,
    )?;
    chain_tool_step(
        &mut engine,
        json!({"action":"constraints","fixed_lengths":false,"fixed_angles":true}),
        "acyclic",
        false,
        true,
        3,
        false,
    )?;
    chain_tool_step(
        &mut engine,
        json!({"action":"select","kind":"acyclic"}),
        "acyclic",
        false,
        true,
        3,
        false,
    )?;
    chain_tool_step(
        &mut engine,
        json!({"action":"select","kind":"snaking"}),
        "snaking",
        false,
        true,
        4,
        true,
    )?;
    chain_tool_step(
        &mut engine,
        json!({"action":"constraints","fixed_lengths":false,"fixed_angles":false}),
        "snaking",
        false,
        false,
        5,
        true,
    )?;
    chain_tool_step(
        &mut engine,
        json!({"action":"constraints","fixed_lengths":true,"fixed_angles":false}),
        "snaking",
        true,
        false,
        6,
        true,
    )?;
    assert_eq!(engine.state().revision, 0);
    assert_eq!(
        (engine.state().undo_depth, engine.state().redo_depth),
        (0, 0)
    );
    assert!(engine.document()?.drawing_settings().is_empty());
    assert_eq!(engine.bond_tool().order, remembered);

    engine.dispatch("document.new", json!({"expected_revision":0}))?;
    assert_eq!(engine.state().revision, 1);
    chain_tool_step(
        &mut engine,
        json!({"action":"select","kind":"snaking"}),
        "snaking",
        true,
        false,
        6,
        false,
    )?;
    assert!(engine.document()?.drawing_settings().is_empty());
    engine.dispatch(
        "document.open_cdxml",
        json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/bondline-v0.cdxml")}),
    )?;
    assert_eq!(engine.state().revision, 2);
    assert!(!engine.state().file.dirty);
    assert_eq!(
        (engine.state().undo_depth, engine.state().redo_depth),
        (0, 0)
    );
    chain_tool_step(
        &mut engine,
        json!({"action":"select","kind":"snaking"}),
        "snaking",
        true,
        false,
        6,
        false,
    )?;
    chain_tool_step(
        &mut engine,
        json!({"action":"constraints","fixed_lengths":false,"fixed_angles":true}),
        "snaking",
        false,
        true,
        7,
        true,
    )?;
    engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
    assert_eq!(engine.bond_tool().epoch, 8);
    assert_eq!(engine.bond_tool().order, remembered);
    chain_tool_step(
        &mut engine,
        json!({"action":"select","kind":"acyclic"}),
        "acyclic",
        false,
        true,
        9,
        true,
    )?;
    assert!(!engine.state().ui_file.available);
    assert_eq!(engine.state().revision, 2);
    assert!(engine.document()?.drawing_settings().is_empty());
    Ok(())
}
