#![cfg(not(target_arch = "wasm32"))]

use bondline_commands::{Command, Engine};
use serde_json::{Value, json};
use std::{fs, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated actual-file directory")]
mod support;
use support::FileTestDirectory;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const WATER: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-label-preview/water.cdx");
const EDITED: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-label-edit/water_to_nh.cdx");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/cdx-label-edit/cases.json");

fn open_water(directory: &FileTestDirectory) -> TestResult<Engine> {
    let input = directory.path("water-原-λ.cdx");
    fs::write(&input, WATER)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdx", json!({"path":input}))?;
    Ok(engine)
}

fn label_params(revision: u64, element: u8, hydrogens: u16) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":2,
           "element":element,"explicit_hydrogens":hydrogens})
}

fn form_params(revision: u64) -> Value {
    json!({"action":"open","expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":2})
}

#[test]
fn cdx_label_edit_preserves_exact_source_through_noop_history_save_and_reopen() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let output = directory.path("water-edited-保存-β.cdx");
    let mut engine = open_water(&directory)?;
    let original = engine.content().clone();
    let original_model = engine.stored_document()?.model_snapshot();
    let original_stamp = engine.capture_stamp();

    let changed = engine.dispatch("atom.set_authored_label", label_params(1, 7, 1))?;
    assert!(changed.changed && changed.file.dirty);
    assert_eq!(
        (changed.revision, changed.undo_depth, changed.redo_depth),
        (2, 1, 0)
    );
    let source = changed.source.ok_or("modified source")?;
    assert_eq!(
        (source.format, source.byte_length, source.modified),
        ("cdx", WATER.len(), true)
    );
    let edited = engine.content().clone();
    let edited_model = engine.stored_document()?.model_snapshot();
    let witness: Value = serde_json::from_str(WITNESSES)?;
    assert_eq!(
        serde_json::to_value(&edited_model)?,
        witness["cases"]["water_to_nh"]["after_model"]
    );
    let stored = engine.stored_document()?;
    assert_eq!(stored.to_cdx()?.as_ref(), EDITED);
    assert!(stored.has_cdx_atom_label_preview());
    assert!(!stored.is_fully_mapped());
    assert!(!engine.selection_available());
    assert!(stored.to_cdxml().is_err());

    let undone = engine.dispatch("history.undo", json!({}))?;
    assert!(!undone.file.dirty);
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.stored_document()?.to_cdx()?.as_ref(), WATER);
    assert!(Arc::ptr_eq(
        &original_model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(!engine.capture_stamp_matches(&original_stamp));
    let undo_stamp = engine.capture_stamp();
    let file_context = engine.ui_file_context_stamp();
    let noop = engine.dispatch("atom.set_authored_label", label_params(3, 8, 0))?;
    assert!(!noop.changed && !noop.file.dirty);
    assert_eq!((noop.revision, noop.undo_depth, noop.redo_depth), (3, 0, 1));
    assert!(engine.capture_stamp_matches(&undo_stamp));
    assert!(engine.ui_file_context_stamp_matches(&file_context));

    let redone = engine.dispatch("history.redo", json!({}))?;
    assert!(redone.file.dirty);
    assert_eq!(engine.content(), &edited);
    assert_eq!(engine.stored_document()?.to_cdx()?.as_ref(), EDITED);
    let saved = engine.dispatch("document.save_cdx", json!({"path":output}))?;
    assert!(!saved.file.dirty);
    let source = saved.source.ok_or("retained original source")?;
    assert_eq!((source.byte_length, source.modified), (WATER.len(), true));
    let receipt = saved.file_operation.ok_or("save receipt")?;
    assert_eq!(
        (receipt.input_revision, receipt.byte_length),
        (4, EDITED.len())
    );
    assert_eq!(fs::read(&output)?, EDITED);
    assert_eq!(fs::read(directory.path("water-原-λ.cdx"))?, WATER);

    let reopened = engine.dispatch("document.open_cdx", json!({"path":output}))?;
    assert_eq!(
        (
            reopened.undo_depth,
            reopened.redo_depth,
            reopened.file.dirty
        ),
        (0, 0, false)
    );
    let source = reopened.source.ok_or("reopened source")?;
    assert_eq!((source.byte_length, source.modified), (EDITED.len(), false));
    assert_eq!(engine.stored_document()?.model_snapshot(), edited_model);
    assert!(engine.stored_document()?.has_cdx_atom_label_preview());
    Ok(())
}

#[test]
fn cdx_label_native_form_reuses_the_command_and_rejects_an_aba_owner_after_a_captured_save()
-> TestResult {
    let directory = FileTestDirectory::new()?;
    let output = directory.path("water-captured-原-λ.cdx");
    let mut engine = open_water(&directory)?;
    engine.set_ui_file_host(true);
    let pending = engine.prepare(Command::parse("document.save_cdx", json!({"path":output}))?)?;

    let form = engine
        .dispatch("ui.atom_label", form_params(1))?
        .ui_atom_label
        .ok_or("existing atom label form")?;
    assert_eq!((form.element, form.explicit_hydrogens), (8, Some(0)));
    assert!(form.current && form.creation_style.is_none());
    let changed = engine.apply_ui_atom_label(form.id, 7, 1)?;
    assert!(changed.changed && changed.ui_atom_label.is_none());
    assert_eq!(engine.stored_document()?.to_cdx()?.as_ref(), EDITED);
    let edited_model = engine.stored_document()?.model_snapshot();

    let saved = engine.finish(pending.complete()?)?;
    assert!(saved.file.dirty);
    let receipt = saved.file_operation.ok_or("captured save receipt")?;
    assert_eq!(
        (receipt.input_revision, receipt.byte_length),
        (1, WATER.len())
    );
    assert_eq!(fs::read(&output)?, WATER);
    assert_eq!(engine.stored_document()?.to_cdx()?.as_ref(), EDITED);

    let form = engine
        .dispatch("ui.atom_label", form_params(2))?
        .ui_atom_label
        .ok_or("edited atom label form")?;
    assert_eq!((form.element, form.explicit_hydrogens), (7, Some(1)));
    assert!(!engine.dispatch("history.undo", json!({}))?.file.dirty);
    assert!(engine.dispatch("history.redo", json!({}))?.file.dirty);
    assert!(Arc::ptr_eq(
        &edited_model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(!engine.ui_atom_label_request_current(form.id));
    let before = serde_json::to_value(engine.state())?;
    let error = engine
        .apply_ui_atom_label(form.id, 8, 0)
        .err()
        .ok_or("stale form accepted the ABA owner")?;
    assert_eq!(error.info().code, "ui_stale_request");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}
