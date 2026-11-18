#![cfg(not(target_arch = "wasm32"))]

use bondline_commands::Engine;
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated actual-file directory")]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const ORIGINAL: &[u8] =
    include_bytes!("../../io/tests/fixtures/cdx-implicit-element/cyanide_implicit.cdx");
const EDITED: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-implicit-element/cyanide_si.cdx");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/cdx-implicit-element/cases.json");

#[test]
fn cdx_implicit_element_noop_undo_and_save_preserve_the_exact_source_baselines() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("cyanide-原-λ.cdx");
    let output = directory.path("cyanide-保存-β.cdx");
    let crossed = directory.path("cyanide-lossy.cdxml");
    fs::write(&input, ORIGINAL)?;
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let case = &witnesses["cases"]["cyanide_c_to_si"];

    let mut engine = Engine::default();
    let opened = engine.dispatch("document.open_cdx", json!({"path":input}))?;
    assert_eq!(serde_json::to_value(opened.document)?, case["before_model"]);
    let original = engine.content().clone();
    let stamp = engine.capture_stamp();
    let file_context = engine.ui_file_context_stamp();
    let mut params = case["params"].clone();
    params["expected_revision"] = json!(1);
    let mut noop = params.clone();
    noop["element"] = json!(6);
    let unchanged = engine.dispatch("atom.set_authored_label", noop)?;
    assert!(!unchanged.changed && !unchanged.file.dirty);
    assert_eq!(
        (
            unchanged.revision,
            unchanged.undo_depth,
            unchanged.redo_depth
        ),
        (1, 0, 0)
    );
    assert!(!unchanged.source.ok_or("original source")?.modified);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(engine.ui_file_context_stamp_matches(&file_context));
    assert_eq!(engine.stored_document()?.export_cdx()?, ORIGINAL);

    let changed = engine.dispatch("atom.set_authored_label", params.clone())?;
    assert!(changed.changed && changed.file.dirty);
    assert_eq!(changed.undo_depth, 1);
    assert_eq!(serde_json::to_value(changed.document)?, case["after_model"]);
    assert_eq!(engine.stored_document()?.export_cdx()?, EDITED);
    let edited = engine.content().clone();
    let before_refusals = serde_json::to_value(engine.state())?;
    params["expected_revision"] = json!(2);
    for (command, arguments) in [
        ("atom.set_label", params),
        (
            "ui.selection",
            json!({"action":"all","expected_revision":2,"page_id":1}),
        ),
        ("document.save_cdxml", json!({"path":crossed})),
    ] {
        let error = engine
            .dispatch(command, arguments)
            .err()
            .ok_or("unsupported command accepted the edited CDX")?;
        assert_eq!(error.info().code, "unsupported_content", "{command}");
    }
    assert!(!crossed.exists());
    assert_eq!(serde_json::to_value(engine.state())?, before_refusals);

    let undone = engine.dispatch("history.undo", json!({}))?;
    assert!(!undone.file.dirty && !undone.source.ok_or("restored source")?.modified);
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.stored_document()?.export_cdx()?, ORIGINAL);
    assert!(!engine.capture_stamp_matches(&stamp));
    let redone = engine.dispatch("history.redo", json!({}))?;
    assert!(redone.file.dirty && redone.source.ok_or("edited source")?.modified);
    assert_eq!(engine.content(), &edited);
    assert_eq!(engine.stored_document()?.export_cdx()?, EDITED);

    let saved = engine.dispatch("document.save_cdx", json!({"path":output}))?;
    assert!(!saved.file.dirty);
    let source = saved.source.ok_or("retained original source")?;
    assert_eq!(
        (source.byte_length, source.modified),
        (ORIGINAL.len(), true)
    );
    let receipt = saved.file_operation.ok_or("save receipt")?;
    assert_eq!(
        (receipt.input_revision, receipt.byte_length),
        (4, EDITED.len())
    );
    assert_eq!(fs::read(&input)?, ORIGINAL);
    assert_eq!(fs::read(&output)?, EDITED);

    let reopened = engine.dispatch("document.open_cdx", json!({"path":output}))?;
    assert_eq!(
        serde_json::to_value(reopened.document)?,
        case["after_model"]
    );
    assert!(!reopened.file.dirty);
    let source = reopened.source.ok_or("reopened source")?;
    assert_eq!((source.byte_length, source.modified), (EDITED.len(), false));
    assert_eq!((reopened.undo_depth, reopened.redo_depth), (0, 0));
    Ok(())
}
