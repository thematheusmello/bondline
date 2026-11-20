#![cfg(not(target_arch = "wasm32"))]

use bondline_commands::Engine;
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated actual-file directory")]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const BEFORE: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/multi_before.cdx");
const AFTER: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-caption-edit/multi_after.cdx");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/cdx-caption-edit/cases.json");
const ATOM_LABELS: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-label-preview/water.cdx");

#[test]
fn mixed_page_run_edit_keeps_exact_history_and_the_chemical_source() -> TestResult {
    let before =
        include_bytes!("../../io/tests/fixtures/cdx-mixed-caption-edit/extended_before.cdx");
    let prefix = include_bytes!("../../io/tests/fixtures/cdx-mixed-caption-edit/after_prefix.cdx");
    let both = include_bytes!("../../io/tests/fixtures/cdx-mixed-caption-edit/after_both.cdx");
    let tail = include_bytes!("../../io/tests/fixtures/cdx-mixed-caption-edit/after_tail.cdx");
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("mixed-runs-原.cdx");
    let output = directory.path("mixed-runs-保存-β.cdx");
    fs::write(&input, before)?;
    let mut engine = Engine::default();
    let start = engine.dispatch("document.open_cdx", json!({"path":input}))?;
    let run = |revision: u64, index: usize, body: &str| json!({"expected_revision":revision,"page_id":1,"text_id":19,"run_index":index,"text":body});
    let original = engine.content().clone();
    let stamp = engine.capture_stamp();
    let unchanged = engine.dispatch("text.set_run", run(start.revision, 0, "H "))?;
    assert!(!unchanged.changed && !unchanged.file.dirty);
    assert_eq!(
        (unchanged.revision, unchanged.undo_depth),
        (start.revision, 0)
    );
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.stored_document()?.export_cdx()?, before);
    let first = engine.dispatch("text.set_run", run(start.revision, 0, "H H "))?;
    assert!(first.changed && first.file.dirty && first.source.ok_or("source")?.modified);
    assert_eq!(engine.stored_document()?.export_cdx()?, prefix);
    assert!(engine.stored_document()?.has_cdx_mixed_preview());
    assert!(!engine.stored_document()?.can_create_caption());
    assert!(!engine.selection_available());
    let undo = engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.stored_document()?.export_cdx()?, before);
    assert!(!undo.file.dirty && !undo.source.ok_or("restored source")?.modified);
    assert_eq!((undo.undo_depth, undo.redo_depth), (0, 1));
    let stamp = engine.capture_stamp();
    for params in [
        run(start.revision, 0, "H H "),
        run(undo.revision, 2, "H H "),
        run(undo.revision, 0, "λ "),
        {
            let mut p = run(undo.revision, 0, "H H ");
            p["text_id"] = json!(73);
            p
        },
    ] {
        assert!(engine.dispatch("text.set_run", params).is_err());
        assert_eq!(engine.content(), &original);
        assert!(engine.capture_stamp_matches(&stamp));
        assert_eq!(engine.state().redo_depth, 1);
    }
    let redo = engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document()?.export_cdx()?, prefix);
    let second = engine.dispatch("text.set_run", run(redo.revision, 1, "λéπΩ"))?;
    assert_eq!((second.undo_depth, second.redo_depth), (2, 0));
    assert_eq!(engine.stored_document()?.export_cdx()?, both);
    engine.dispatch("document.save_cdx", json!({"path":output}))?;
    assert_eq!(fs::read(&input)?, before);
    assert_eq!(fs::read(&output)?, both);
    let reopened = engine.dispatch("document.open_cdx", json!({"path":output}))?;
    assert_eq!(reopened.document, second.document);
    assert!(!reopened.file.dirty);
    assert_eq!((reopened.undo_depth, reopened.redo_depth), (0, 0));
    engine.dispatch("text.set_run", run(reopened.revision, 0, "H "))?;
    assert_eq!(engine.stored_document()?.export_cdx()?, tail);
    Ok(())
}

#[test]
fn cdx_caption_edit_keeps_exact_noop_history_and_file_provenance() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("caption-原-λ.cdx");
    let output = directory.path("caption-保存-β.cdx");
    fs::write(&input, BEFORE)?;
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let case = &witnesses["cases"]["multi_shift"];

    let mut engine = Engine::default();
    let opened = engine.dispatch("document.open_cdx", json!({"path":input}))?;
    assert_eq!(serde_json::to_value(opened.document)?, case["before_model"]);
    let original = engine.content().clone();
    let original_stamp = engine.capture_stamp();
    let mut edit = case["params"].clone();
    edit["expected_revision"] = json!(1);
    let changed = engine.dispatch("text.set_run", edit.clone())?;
    assert!(changed.changed && changed.file.dirty);
    assert_eq!(
        (changed.revision, changed.undo_depth, changed.redo_depth),
        (2, 1, 0)
    );
    assert_eq!(serde_json::to_value(changed.document)?, case["after_model"]);
    assert_eq!(engine.stored_document()?.export_cdx()?, AFTER);
    assert!(engine.stored_document()?.has_display_projection());
    assert!(!engine.stored_document()?.is_fully_mapped());
    assert!(!engine.selection_available());
    let edited = engine.content().clone();

    let undone = engine.dispatch("history.undo", json!({}))?;
    assert!(!undone.file.dirty && !undone.source.ok_or("restored source")?.modified);
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.stored_document()?.export_cdx()?, BEFORE);
    assert!(!engine.capture_stamp_matches(&original_stamp));
    let stamp = engine.capture_stamp();
    let file_context = engine.ui_file_context_stamp();
    let mut noop = edit.clone();
    noop["expected_revision"] = json!(3);
    noop["text"] = json!("AV");
    let unchanged = engine.dispatch("text.set_run", noop)?;
    assert!(!unchanged.changed && !unchanged.file.dirty);
    assert_eq!(
        (
            unchanged.revision,
            unchanged.undo_depth,
            unchanged.redo_depth
        ),
        (3, 0, 1)
    );
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(engine.ui_file_context_stamp_matches(&file_context));
    assert_eq!(engine.stored_document()?.export_cdx()?, BEFORE);

    let redone = engine.dispatch("history.redo", json!({}))?;
    assert!(redone.file.dirty && redone.source.ok_or("edited source")?.modified);
    assert_eq!(engine.content(), &edited);
    assert_eq!(engine.stored_document()?.export_cdx()?, AFTER);
    let saved = engine.dispatch("document.save_cdx", json!({"path":output}))?;
    assert!(!saved.file.dirty);
    let source = saved.source.ok_or("retained original source")?;
    assert_eq!((source.byte_length, source.modified), (BEFORE.len(), true));
    let receipt = saved.file_operation.ok_or("save receipt")?;
    assert_eq!(
        (receipt.input_revision, receipt.byte_length),
        (4, AFTER.len())
    );
    assert_eq!(fs::read(&input)?, BEFORE);
    assert_eq!(fs::read(&output)?, AFTER);
    let reopened = engine.dispatch("document.open_cdx", json!({"path":output}))?;
    assert_eq!(
        serde_json::to_value(reopened.document)?,
        case["after_model"]
    );
    assert!(!reopened.file.dirty);
    let source = reopened.source.ok_or("reopened source")?;
    assert_eq!((source.byte_length, source.modified), (AFTER.len(), false));
    assert_eq!((reopened.undo_depth, reopened.redo_depth), (0, 0));

    let unrelated = directory.path("unrelated-atom-labels.cdx");
    fs::write(&unrelated, ATOM_LABELS)?;
    let other = engine.dispatch("document.open_cdx", json!({"path":unrelated}))?;
    assert!(engine.stored_document()?.has_cdx_atom_label_preview());
    let before_refusal = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    edit["expected_revision"] = json!(other.revision);
    let error = engine
        .dispatch("text.set_run", edit)
        .err()
        .ok_or("direct-page caption authority leaked to an atom-label page")?;
    assert_eq!(error.info().code, "unsupported_content");
    assert_eq!(serde_json::to_value(engine.state())?, before_refusal);
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.stored_document()?.export_cdx()?, ATOM_LABELS);
    Ok(())
}
