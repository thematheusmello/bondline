#![cfg(not(target_arch = "wasm32"))]

use bondline_commands::{Cancellation, Command, Engine, registry};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command as Process, Stdio},
};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code)]
mod file_support;
use file_support::FileTestDirectory;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const LEGACY: &[u8] = include_bytes!("../../cdx/tests/fixtures/original-ethane-explicit.cdx");
const HEADER: &[u8] =
    include_bytes!("../../cdx/tests/fixtures/original-ethane-header-document.cdx");
const METADATA: &[u8] = include_bytes!("../../io/tests/fixtures/cdx/metadata-header.cdx");
const METADATA_LEGACY: &[u8] = include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx");

fn state(engine: &Engine) -> Result<Value, serde_json::Error> {
    serde_json::to_value(engine.state())
}

fn oracle(path: &Path, script: &str, arguments: &[&str]) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Process::new(python)
        .arg("-B")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(script),
        )
        .args(arguments)
        .stdin(fs::File::open(path)?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn cdx_commands_have_closed_typed_parameters_with_no_implicit_permission() -> TestResult {
    for (name, permission) in [
        ("document.open_cdx", "discard_unsaved"),
        ("document.save_cdx", "overwrite"),
    ] {
        let commands = registry();
        let info = commands
            .iter()
            .find(|info| info.name == name)
            .ok_or("missing CDX command")?;
        assert!(!info.read_only);
        let schema = serde_json::to_value(&info.input_schema)?;
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], json!(["path"]));
        let properties = schema["properties"].as_object().ok_or("properties")?;
        assert_eq!(
            properties
                .keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from(["path", permission])
        );
        assert_eq!(properties["path"]["type"], "string");
        assert_eq!(properties[permission]["type"], "boolean");
        assert_eq!(properties[permission]["default"], false);
        assert_eq!(
            Command::parse(name, json!({"path":"unused.cdx","unexpected":true}))
                .err()
                .ok_or("open schema")?
                .info()
                .code,
            "invalid_params"
        );
    }
    Ok(())
}

#[test]
fn registered_cdx_open_save_retains_four_original_vectors_and_independent_consumers() -> TestResult
{
    for (bytes, script, arguments) in [
        (
            LEGACY,
            "xtask/oracles/cdx_oracle.py",
            &["verify", "1", "explicit"][..],
        ),
        (
            HEADER,
            "xtask/oracles/cdx_header_oracle.py",
            &["verify", "1"][..],
        ),
        (
            METADATA_LEGACY,
            "crates/io/tests/cdx_metadata_oracle.py",
            &["verify", "explicit"][..],
        ),
        (
            METADATA,
            "crates/io/tests/cdx_metadata_oracle.py",
            &["verify", "header"][..],
        ),
    ] {
        let directory = FileTestDirectory::new()?;
        let input = directory.path("original-λ.mol");
        let copy = directory.path("saved-λ.cdx");
        fs::write(&input, bytes)?;
        let mut engine = Engine::default();
        let opened = engine.dispatch("document.open_cdx", json!({"path":input}))?;
        let wire = serde_json::to_value(opened)?;
        assert_eq!(wire["revision"], 1);
        assert_eq!(wire["undo_depth"], 0);
        assert_eq!(wire["redo_depth"], 0);
        assert_eq!(
            wire["file"],
            json!({"format":"cdx","path":input.canonicalize()?,"dirty":false})
        );
        assert_eq!(
            wire["source"],
            json!({"format":"cdx","byte_length":bytes.len(),"modified":false})
        );
        assert_eq!(wire["file_operation"]["format"], "cdx");
        assert!(!serde_json::to_string(&wire)?.contains("Original metadata advisory"));
        let page = engine.document()?.pages().first().ok_or("page")?;
        assert_eq!(page.size(), bondline_model::PageSize::A4);
        let fragment = page.fragments().first().ok_or("fragment")?;
        assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (2, 1));
        assert!(
            fragment
                .atoms()
                .iter()
                .all(|atom| atom.explicit_hydrogens.is_none())
        );
        assert!(!engine.stored_document()?.is_fully_mapped());
        assert_eq!(engine.stored_document()?.export_cdx()?, bytes);
        let saved = engine.dispatch("document.save_cdx", json!({"path":copy}))?;
        assert_eq!(saved.revision, 1);
        assert!(!saved.file.dirty);
        let wire = serde_json::to_value(saved)?;
        assert_eq!(wire["file_operation"]["operation"], "save");
        assert_eq!(wire["file_operation"]["format"], "cdx");
        assert_eq!(wire["file_operation"]["byte_length"], bytes.len());
        assert_eq!(fs::read(&copy)?, bytes);
        oracle(&copy, script, arguments)?;
        assert_eq!(fs::read(input)?, bytes);
        let mut reopened = Engine::default();
        reopened.dispatch("document.open_cdx", json!({"path":copy}))?;
        assert_eq!(reopened.stored_document()?, engine.stored_document()?);
    }
    Ok(())
}

#[test]
fn cdx_failures_preserve_snapshot_history_destination_and_release_save_slot() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let input = directory.path("original.cdx");
    let destination = directory.path("keep.cdx");
    fs::write(&input, METADATA)?;
    fs::write(&destination, b"keep destination")?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdx", json!({"path":input}))?;
    let before = state(&engine)?;
    for (command, params, code) in [
        (
            "document.save_cdx",
            json!({"path":destination}),
            "already_exists",
        ),
        (
            "document.save_cdxml",
            json!({"path":destination,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "document.save_mol",
            json!({"path":destination,"overwrite":true}),
            "unsupported_content",
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":10,"begin_y_pt":10,"end_x_pt":20,"end_y_pt":20}),
            "unsupported_content",
        ),
        (
            "render.capture",
            json!({"page_id":1,"backend":"cpu","width_px":80,"height_px":80}),
            "unsupported_content",
        ),
    ] {
        assert_eq!(
            engine
                .dispatch(command, params)
                .err()
                .ok_or("unsafe operation succeeded")?
                .info()
                .code,
            code
        );
        assert_eq!(state(&engine)?, before);
        assert_eq!(fs::read(&destination)?, b"keep destination");
    }
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let edited = state(&engine)?;
    assert_eq!(
        engine
            .dispatch(
                "document.save_cdx",
                json!({"path":destination,"overwrite":true})
            )
            .err()
            .ok_or("stale export")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(state(&engine)?, edited);
    assert_eq!(
        engine
            .dispatch("document.open_cdx", json!({"path":input}))
            .err()
            .ok_or("discarded dirty document")?
            .info()
            .code,
        "unsaved_changes"
    );
    engine.dispatch("history.undo", json!({}))?;
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.export_cdx()?, METADATA);
    engine.dispatch(
        "document.save_cdx",
        json!({"path":destination,"overwrite":true}),
    )?;
    assert_eq!(fs::read(&destination)?, METADATA);
    let native = directory.path("native.cdx");
    assert_eq!(
        Engine::default()
            .dispatch("document.save_cdx", json!({"path":native}))
            .err()
            .ok_or("invented encoder")?
            .info()
            .code,
        "unsupported_content"
    );
    assert!(!native.exists());
    Ok(())
}

#[test]
fn source_family_initialization_and_failed_open_preserve_complete_context() -> TestResult {
    let stored =
        bondline_io::StoredDocument::from_cdx(METADATA.into(), bondline_model::PageSize::A4)?;
    let mut engine = Engine::from_document(stored);
    let initial = state(&engine)?;
    assert_eq!(
        initial["file"],
        json!({"format":"cdx","path":null,"dirty":false})
    );
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    let before = state(&engine)?;
    assert_eq!(before["redo_depth"], 1);
    let directory = FileTestDirectory::new()?;
    let invalid = directory.path("bad.cdx");
    fs::write(&invalid, b"not a CDX document")?;
    assert_eq!(
        engine
            .dispatch("document.open_cdx", json!({"path":invalid}))
            .err()
            .ok_or("bad CDX opened")?
            .info()
            .code,
        "invalid_document"
    );
    assert_eq!(state(&engine)?, before);
    assert_eq!(engine.stored_document()?.export_cdx()?, METADATA);
    Ok(())
}

#[test]
fn trusted_cdx_known_save_binds_format_and_current_receipt_without_consuming_bad_selection()
-> TestResult {
    let directory = FileTestDirectory::new()?;
    let path = directory.path("associated.mol");
    fs::write(&path, METADATA)?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_cdx", json!({"path":path}))?;
    let request = engine.dispatch("ui.file", json!({"expected_revision":1,"action":"save"}))?;
    let id = request.ui_file_operation.ok_or("request")?;
    let plan = engine.claim_ui_file(id)?;
    let bondline_commands::UiFilePlan::SaveExisting { format, params } = plan else {
        return Err("known Save picker".into());
    };
    assert_eq!(format, bondline_commands::FileFormat::Cdx);
    let accepted = state(&engine)?;
    let wrong = Command::parse(
        "document.save_cdxml",
        json!({"path":params.path,"overwrite":true}),
    )?;
    assert_eq!(
        engine
            .prepare_ui_file(id, wrong)
            .err()
            .ok_or("wrong-format known Save")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(state(&engine)?, accepted);
    let actual = Command::parse(
        "document.save_cdx",
        json!({"path":params.path,"overwrite":true}),
    )?;
    let completed = engine.prepare_ui_file(id, actual)?.complete()?;
    let result = engine.finish(completed)?;
    let receipt = result.file_operation.ok_or("receipt")?;
    assert_eq!(receipt.format, bondline_commands::FileFormat::Cdx);
    assert_eq!(receipt.ui_request_id, Some(id));
    assert_eq!(receipt.ui_request_current, Some(true));
    assert_eq!(receipt.input_revision, 1);
    assert!(engine.finish_ui_file(id, false));
    assert!(engine.state().ui_file.request.is_none());
    assert_eq!(fs::read(path)?, METADATA);
    Ok(())
}

#[test]
fn cdx_deferred_work_retains_revision_and_the_shared_save_reservation() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let input = directory.path("original.cdx");
    let copy = directory.path("saved.cdx");
    fs::write(&input, HEADER)?;
    let mut engine = Engine::default();
    let pending = engine.prepare(Command::parse("document.open_cdx", json!({"path":input}))?)?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let edited = state(&engine)?;
    assert_eq!(
        engine
            .finish(pending.complete()?)
            .err()
            .ok_or("stale open applied")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(state(&engine)?, edited);
    engine.dispatch(
        "document.open_cdx",
        json!({"path":input,"discard_unsaved":true}),
    )?;
    let prepared = engine.prepare(Command::parse("document.save_cdx", json!({"path":copy}))?)?;
    assert_eq!(
        engine
            .dispatch("document.save_cdxml", json!({"path":copy}))
            .err()
            .ok_or("overlapping save")?
            .info()
            .code,
        "busy"
    );
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        prepared
            .complete_with_cancel(&cancel)
            .err()
            .ok_or("ignored cancel")?
            .info()
            .code,
        "cancelled"
    );
    assert!(!copy.exists());
    let completed = engine
        .prepare(Command::parse("document.save_cdx", json!({"path":copy}))?)?
        .complete()?;
    assert_eq!(
        engine
            .dispatch("document.save_cdx", json!({"path":copy,"overwrite":true}))
            .err()
            .ok_or("released published lease")?
            .info()
            .code,
        "busy"
    );
    let saved = engine.finish(completed)?;
    assert!(!saved.file.dirty);
    assert_eq!(fs::read(copy)?, HEADER);
    Ok(())
}
