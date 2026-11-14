use bondline_commands::{Cancellation, Command, Engine};
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated file directory")]
mod support;

const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn capture() -> Result<Command, bondline_commands::Error> {
    Command::parse(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
    )
}

#[test]
fn sdf_capture_is_a_read_only_derived_sheet_and_save_retains_every_record() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.sdf");
    let output = directory.path("saved.sdf");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    let original = engine.content().clone();
    let result = engine.dispatch(
        "render.capture",
        json!({
            "page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"
        }),
    )?;
    assert!(result.capture.is_some());
    let mut wire = serde_json::to_value(result)?;
    wire.as_object_mut()
        .ok_or("result")?
        .remove("capture")
        .ok_or("capture")?;
    assert_eq!(wire, before);
    assert_eq!(engine.content(), &original);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(
        engine.document().is_err(),
        "preview must not become an authored drawing"
    );
    engine.dispatch("document.save_sdf", json!({"path":output}))?;
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::read(&output)?, SOURCE);
    Ok(())
}

#[test]
fn sdf_capture_keeps_its_historical_source_through_field_history_and_new_sessions() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.sdf");
    let next = directory.path("next.smi");
    fs::write(&input, SOURCE)?;
    fs::write(&next, b"O next\n")?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let old = engine.prepare(capture()?)?;
    assert!(old.is_deferred());
    engine.dispatch(
        "sdf.field.set_value",
        json!({"expected_revision":1,"record_id":1,"field_index":5,"value_lines":["changed"]}),
    )?;
    let changed = engine.content().clone();
    let before = serde_json::to_value(engine.state())?;
    let old = engine.finish(old.complete()?)?;
    assert_eq!(old.revision, 1);
    assert_eq!(
        old.collection.as_ref().ok_or("collection")?.byte_length,
        SOURCE.len()
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &changed);
    let pixels = old.capture.ok_or("old capture")?.png_base64;
    for name in [None, Some("history.undo"), Some("history.redo")] {
        if let Some(name) = name {
            engine.dispatch(name, json!({}))?;
        }
        let before = serde_json::to_value(engine.state())?;
        assert_eq!(
            engine
                .execute(capture()?)?
                .capture
                .ok_or("capture")?
                .png_base64,
            pixels
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    assert_eq!(engine.content(), &changed);
    let old = engine.prepare(capture()?)?;
    fs::remove_file(&input)?;
    engine.dispatch(
        "document.open_smiles",
        json!({"path":next,"discard_unsaved":true}),
    )?;
    let revision = engine.state().revision;
    engine.dispatch("document.close", json!({"expected_revision":revision}))?;
    let before = serde_json::to_value(engine.state())?;
    let reply = serde_json::to_value(engine.finish(old.complete()?)?)?;
    assert_eq!(reply["revision"], 4);
    assert!(reply["collection"].is_object());
    assert_eq!(reply["document"], Value::Null);
    assert!(reply.get("smiles_file").is_none());
    assert_eq!(reply["capture"]["png_base64"], pixels);
    assert_eq!(reply["close"], "approved");
    assert_eq!(reply["close_epoch"], before["close_epoch"]);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(fs::read(&next)?, b"O next\n");
    Ok(())
}

#[test]
fn cancelled_sdf_capture_cannot_publish_before_work_or_after_completion() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.sdf");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    for late in [false, true] {
        let work = engine.prepare(capture()?)?;
        let cancel = Cancellation::default();
        let error = if late {
            let done = work.complete_with_cancel(&cancel)?;
            cancel.cancel();
            engine.finish(done).err().ok_or("late cancellation")?
        } else {
            cancel.cancel();
            work.complete_with_cancel(&cancel)
                .err()
                .ok_or("early cancellation")?
        };
        assert_eq!(error.info().code, "cancelled");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(fs::read(&input)?, SOURCE);
    }
    Ok(())
}

#[test]
fn sdf_capture_neither_reserves_a_save_nor_binds_native_file_intent() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.sdf");
    let output = directory.path("saved.sdf");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let work = engine.prepare(capture()?)?;
    let save = engine.prepare(Command::parse("document.save_sdf", json!({"path":output}))?)?;
    let before = serde_json::to_value(engine.state())?;
    engine.finish(work.complete()?)?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(
        engine
            .prepare(Command::parse("document.save_sdf", json!({"path":output}))?)
            .err()
            .ok_or("outstanding save must keep its lease")?
            .info()
            .code,
        "busy"
    );
    engine.finish(save.complete()?)?;
    assert_eq!(fs::read(&output)?, SOURCE);
    engine.set_ui_file_host(true);
    let offered = engine.dispatch("ui.file", json!({"expected_revision":1,"action":"save_as"}))?;
    let id = offered.ui_file.request.ok_or("native file request")?.id;
    engine.claim_ui_file(id)?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .prepare_ui_file(id, capture()?)
            .err()
            .ok_or("capture cannot bind native file intent")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn sdf_preview_never_grants_drawing_edit_or_export_authority() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.sdf");
    let output = directory.path("uncreated.data");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    engine.execute(capture()?)?;
    let before = serde_json::to_value(engine.state())?;
    for (name, params) in [
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":96,"end_y_pt":72}),
        ),
        (
            "file.export_smiles_fragment",
            json!({"expected_revision":1,"page_id":1,"fragment_id":1,"output_path":output}),
        ),
        ("document.save_cdxml", json!({"path":output})),
        ("document.save_cdx", json!({"path":output})),
        ("document.save_mol", json!({"path":output})),
        ("document.save_smiles", json!({"path":output})),
    ] {
        assert_eq!(
            engine
                .dispatch(name, params)
                .err()
                .ok_or("read-only preview")?
                .info()
                .code,
            "unsupported_content",
            "{name}"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(!output.exists());
    }
    Ok(())
}
