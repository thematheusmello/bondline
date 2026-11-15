#![cfg(not(target_arch = "wasm32"))]

use bondline_commands::Engine;
use serde_json::json;
use std::fs;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse the existing atomic isolated file directory"
)]
mod support;
type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("../../cdx/tests/fixtures/external-ethane-indigo-1.47.0.cdx");

#[test]
fn exact_external_file_end_cdx_opens_as_guarded_complete_source() -> TestResult {
    assert_eq!(SOURCE.len(), 258);
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("external-ethane-λ.cdx");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdx", json!({"path":input}))?;
    assert_eq!(engine.state().revision, 1);
    assert_eq!(engine.state().undo_depth, 0);
    assert_eq!(engine.state().redo_depth, 0);
    assert!(!engine.state().file.dirty);
    let stored = engine.stored_document()?;
    assert!(!stored.is_fully_mapped());
    assert_eq!(stored.export_cdx()?, SOURCE);
    let page = stored.model().pages().first().ok_or("page")?;
    assert_eq!(page.fragments().len(), 1);
    let fragment = page.fragments().first().ok_or("fragment")?;
    assert_eq!(fragment.atoms().len(), 2);
    for (atom, x) in fragment.atoms().iter().zip([0.0, 45.0]) {
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.explicit_hydrogens, None);
        let p = atom.position.ok_or("position")?;
        assert_eq!((p.x_pt(), p.y_pt()), (x, 0.0));
    }
    assert_eq!(fragment.bonds().len(), 1);
    assert_eq!(fs::read(input)?, SOURCE);
    Ok(())
}

use bondline_commands::Command;
use serde_json::Value;
use std::{path::Path, process::Command as Process};

fn state(engine: &Engine) -> Result<Value, serde_json::Error> {
    serde_json::to_value(engine.state())
}
fn consume(path: &Path) -> TestResult {
    assert_eq!(fs::read(path)?, SOURCE);
    let output = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../cdx/tests/external_ethane_oracle.py"
        ))
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({"atoms":2,"bonds":1,"formula":"C2H6","smiles":"CC","hydrogens":[3,3],"byte_length":258,"record_count":24,"header_bytes":28,"document_end":[254,256],"file_end":[256,258],"raw_y_x16_16":[[0,0],[0,2949120]]})
    );
    Ok(())
}

#[test]
fn external_file_end_source_guards_failed_import_and_history_preserve_all_bytes() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.cdx");
    let output = directory.path("copy-化学.cdx");
    let sentinel = directory.path("protected.cdx");
    let bad = directory.path("two-file-ends.cdx");
    fs::write(&input, SOURCE)?;
    fs::write(&sentinel, b"preserve existing destination")?;
    let mut extra_end = SOURCE.to_vec();
    extra_end.extend_from_slice(&[0, 0]);
    fs::write(&bad, extra_end)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdx", json!({"path":input}))?;
    let before = state(&engine)?;
    let owner = engine.content().clone();
    let stamp = engine.capture_stamp();
    for (name, params, code) in [
        (
            "document.save_cdx",
            json!({"path":sentinel}),
            "already_exists",
        ),
        (
            "document.save_cdxml",
            json!({"path":sentinel,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "document.save_mol",
            json!({"path":sentinel,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
            "unsupported_content",
        ),
        (
            "render.capture",
            json!({"page_id":1,"backend":"cpu","width_px":1024,"height_px":1024}),
            "unsupported_content",
        ),
    ] {
        assert_eq!(
            engine
                .dispatch(name, params)
                .err()
                .ok_or("unsafe operation succeeded")?
                .info()
                .code,
            code
        );
        assert_eq!(state(&engine)?, before);
        assert_eq!(engine.content(), &owner);
        assert!(engine.capture_stamp_matches(&stamp));
        assert_eq!(fs::read(&sentinel)?, b"preserve existing destination");
    }
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let dirty = state(&engine)?;
    let dirty_owner = engine.content().clone();
    let dirty_stamp = engine.capture_stamp();
    assert_eq!(dirty["undo_depth"], 1);
    assert_eq!(dirty["source"]["modified"], true);
    assert_eq!(dirty["file"]["dirty"], true);
    assert_eq!(
        engine
            .dispatch(
                "document.save_cdx",
                json!({"path":sentinel,"overwrite":true})
            )
            .err()
            .ok_or("edited source saved")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(
        engine
            .dispatch("document.open_cdx", json!({"path":input}))
            .err()
            .ok_or("dirty source discarded")?
            .info()
            .code,
        "unsaved_changes"
    );
    assert_eq!(
        engine
            .dispatch(
                "document.open_cdx",
                json!({"path":bad,"discard_unsaved":true})
            )
            .err()
            .ok_or("multiple FileEnds accepted")?
            .info()
            .code,
        "invalid_document"
    );
    assert_eq!(state(&engine)?, dirty);
    assert_eq!(engine.content(), &dirty_owner);
    assert!(engine.capture_stamp_matches(&dirty_stamp));
    assert_eq!(fs::read(&sentinel)?, b"preserve existing destination");
    engine.dispatch("history.undo", json!({}))?;
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.export_cdx()?, SOURCE);
    let saved = engine.dispatch("document.save_cdx", json!({"path":output}))?;
    assert_eq!(
        saved
            .file_operation
            .as_ref()
            .ok_or("save receipt")?
            .byte_length,
        258
    );
    consume(&output)?;
    engine.dispatch("document.open_cdx", json!({"path":output}))?;
    assert_eq!(engine.state().undo_depth, 0);
    assert_eq!(engine.stored_document()?.export_cdx()?, SOURCE);
    assert_eq!(fs::read(input)?, SOURCE);
    Ok(())
}

#[test]
fn external_file_end_save_reservation_survives_publication_and_new_session() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("lease-input.cdx");
    let output = directory.path("lease-output.cdx");
    let blocked = directory.path("must-not-publish.cdx");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdx", json!({"path":input}))?;
    let before = state(&engine)?;
    let prepared = engine.prepare(Command::parse("document.save_cdx", json!({"path":output}))?)?;
    assert_eq!(
        engine
            .dispatch("document.save_cdx", json!({"path":blocked}))
            .err()
            .ok_or("second Save reserved")?
            .info()
            .code,
        "busy"
    );
    assert!(!output.try_exists()?);
    assert!(!blocked.try_exists()?);
    assert_eq!(state(&engine)?, before);
    let completed = prepared.complete()?;
    consume(&output)?;
    assert_eq!(
        engine
            .dispatch("document.save_cdx", json!({"path":blocked}))
            .err()
            .ok_or("lease released at publication")?
            .info()
            .code,
        "busy"
    );
    assert_eq!(state(&engine)?, before);
    engine.dispatch("document.new", json!({"expected_revision":1}))?;
    let fresh = state(&engine)?;
    assert_eq!(fresh["file"]["path"], Value::Null);
    assert_eq!(fresh["source"], Value::Null);
    let result = engine.finish(completed)?;
    assert_eq!(
        result
            .file_operation
            .as_ref()
            .ok_or("historical receipt")?
            .byte_length,
        258
    );
    assert_eq!(state(&engine)?, fresh);
    assert!(!blocked.try_exists()?);
    assert_eq!(fs::read(&input)?, SOURCE);
    consume(&output)?;
    engine.dispatch("document.open_cdx", json!({"path":output}))?;
    assert_eq!(engine.stored_document()?.export_cdx()?, SOURCE);
    assert!(!engine.stored_document()?.is_fully_mapped());
    Ok(())
}
