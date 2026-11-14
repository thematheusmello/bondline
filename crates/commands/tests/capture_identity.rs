use bondline_commands::{CaptureStamp, Engine};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
use std::{fs, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated file directory")]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn stamps_distinguish_independent_engines_and_history_returning_to_the_same_owner() -> TestResult {
    let mut engine = Engine::default();
    let original = engine.capture_stamp();
    let other = Engine::default();
    assert_ne!(original, other.capture_stamp());
    assert!(!other.capture_stamp_matches(&original));
    let before = serde_json::to_vec(&engine.state())?;
    assert_eq!(original, original.clone());
    assert!(engine.capture_stamp_matches(&original));
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612.0,"height_pt":792.0}),
    )?;
    assert!(!engine.capture_stamp_matches(&original));
    let edited = engine.capture_stamp();
    engine.dispatch("history.undo", json!({}))?;
    assert!(!engine.capture_stamp_matches(&original));
    assert!(!engine.capture_stamp_matches(&edited));
    engine.dispatch("history.redo", json!({}))?;
    assert!(!engine.capture_stamp_matches(&edited));
    assert!(engine.capture_stamp_matches(&engine.capture_stamp()));
    Ok(())
}

#[test]
fn stamp_is_weak_and_debug_does_not_reveal_owner_source_or_address() -> TestResult {
    let (stamp, weak) = {
        let source: Arc<[u8]> =
            Arc::from(b"<CDXML Private='stamp-secret'><page/></CDXML>".as_slice());
        let weak = Arc::downgrade(&source);
        let engine = Engine::from_document(StoredDocument::from_cdxml(source, PageSize::A4)?);
        let stamp: CaptureStamp = engine.capture_stamp();
        assert!(engine.capture_stamp_matches(&stamp));
        (stamp, weak)
    };
    let cloned = stamp.clone();
    assert_eq!(stamp, cloned);
    assert!(
        weak.upgrade().is_none(),
        "stamp may not lease raw source through a strong content owner"
    );
    assert!(!Engine::default().capture_stamp_matches(&stamp));
    let debug = format!("{stamp:?}");
    assert!(debug.starts_with("CaptureStamp"));
    for forbidden in ["stamp-secret", "CDXML", "StoredContent", "0x", "Weak"] {
        assert!(!debug.contains(forbidden), "{debug}");
    }
    Ok(())
}

#[test]
fn close_and_ui_intent_do_not_change_stamp_or_get_acknowledged_by_reading_it() -> TestResult {
    let mut engine = Engine::default();
    let stamp = engine.capture_stamp();
    engine.set_ui_file_host(true);
    engine.dispatch("ui.file", json!({"expected_revision":0,"action":"save_as"}))?;
    let before = serde_json::to_value(engine.state())?;
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(stamp, engine.capture_stamp());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let mut closing = Engine::default();
    let close_stamp = closing.capture_stamp();
    closing.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert!(
        closing.capture_stamp_matches(&close_stamp),
        "close is a separate guard"
    );
    assert!(closing.complete_close());
    let before = serde_json::to_value(closing.state())?;
    assert!(closing.capture_stamp_matches(&close_stamp));
    assert_eq!(serde_json::to_value(closing.state())?, before);
    Ok(())
}

#[test]
fn source_only_sdf_edits_history_and_identical_reopen_invalidate_captured_identity() -> TestResult {
    const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.sdf");
    let another = directory.path("same-bytes.sdf");
    fs::write(&input, SOURCE)?;
    fs::write(&another, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let original = engine.capture_stamp();
    let collection = serde_json::to_value(engine.state())?["collection"].clone();
    engine.dispatch(
        "sdf.field.set_value",
        json!({"expected_revision":1,"record_id":1,"field_index":5,"value_lines":["new"]}),
    )?;
    assert_eq!(
        serde_json::to_value(engine.state())?["collection"],
        collection,
        "same-size opaque value edit has no model/collection geometry signal"
    );
    assert!(!engine.capture_stamp_matches(&original));
    let edited = engine.capture_stamp();
    let before = serde_json::to_vec(&engine.state())?;
    assert_eq!(edited, engine.capture_stamp());
    assert!(engine.capture_stamp_matches(&edited));
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
    assert!(
        !engine.capture_stamp_matches(&original),
        "old owner restored with newer authored revision"
    );
    engine.dispatch("history.redo", json!({}))?;
    assert!(!engine.capture_stamp_matches(&edited));
    engine.dispatch("history.undo", json!({}))?;
    let before_open = engine.capture_stamp();
    engine.dispatch("document.open_sdf", json!({"path":another}))?;
    assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
    assert!(!engine.capture_stamp_matches(&before_open));
    assert!(!engine.capture_stamp_matches(&original));
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::read(&another)?, SOURCE);
    Ok(())
}
