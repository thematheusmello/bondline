use super::*;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn exhausted(counter: &str, expected_code: &str) -> TestResult {
    let mut engine = Engine::default();
    match counter {
        "revision" => engine.revision = u64::MAX,
        "generation" => engine.generation = u64::MAX,
        "view" => engine.view.epoch = u64::MAX,
        _ => return Err("unknown counter declaration".into()),
    }
    let before = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    let view = engine.view();
    let generation = engine.generation;
    let current = Arc::clone(&engine.document);
    let saved = Arc::clone(&engine.saved);
    let error = engine
        .dispatch("document.new", json!({"expected_revision":engine.revision}))
        .err()
        .ok_or("exhausted New succeeded")?;
    assert_eq!(error.info().code, expected_code);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.view(), view);
    assert_eq!(engine.generation, generation);
    assert!(Arc::ptr_eq(&engine.document, &current));
    assert!(Arc::ptr_eq(&engine.saved, &saved));
    Ok(())
}

#[test]
fn new_refuses_exhausted_revision_without_replacing_any_state() -> TestResult {
    exhausted("revision", "revision_exhausted")
}

#[test]
fn new_refuses_exhausted_generation_without_replacing_any_state() -> TestResult {
    exhausted("generation", "revision_exhausted")
}

#[test]
fn new_refuses_exhausted_view_epoch_without_replacing_any_state() -> TestResult {
    exhausted("view", "ui_epoch_exhausted")
}
