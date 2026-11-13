use bondline_commands::{Command, Engine};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command as ProcessCommand, Stdio},
};
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_support;
use file_support::FileTestDirectory;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const CH3: &[u8] = include_bytes!("../../io/tests/fixtures/mol/rdkit-CH3.mol");

fn bond(engine: &mut Engine, y: f64) -> TestResult {
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":y,"end_x_pt":140,"end_y_pt":y}),
    )?;
    Ok(())
}
fn snapshot(engine: &Engine) -> Result<Value, serde_json::Error> {
    serde_json::to_value(engine.state())
}
fn read_mol(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = ProcessCommand::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/mol_oracle.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(&fs::read(path)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn real_mol_open_and_exact_save_keep_complete_headers_source_and_format() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let input = directory.path("methyl-λ.mol");
    let copy = directory.path("copy.mol");
    let mut lines = std::str::from_utf8(CH3)?
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    *lines.get_mut(2).ok_or("comment")? = "Bondline synthetic private header".to_owned();
    let bytes = format!("{}\n", lines.join("\n")).into_bytes();
    fs::write(&input, &bytes)?;
    let mut engine = Engine::default();
    let opened = engine.dispatch("document.open_mol", json!({"path":input}))?;
    assert_eq!(opened.revision, 1);
    assert_eq!(opened.undo_depth, 0);
    assert_eq!(opened.redo_depth, 0);
    let serialized = serde_json::to_value(&opened)?;
    assert_eq!(
        serialized["file"],
        json!({"path":fs::canonicalize(&input)?,"dirty":false,"format":"mol_v2000"})
    );
    assert_eq!(
        serialized["source"],
        json!({"format":"mol","byte_length":bytes.len(),"modified":false})
    );
    let atom = opened
        .document
        .pages()
        .first()
        .ok_or("page")?
        .fragments()
        .first()
        .ok_or("fragment")?
        .atoms()
        .first()
        .ok_or("atom")?;
    assert_eq!(atom.element.get(), 6);
    assert_eq!(atom.explicit_hydrogens, Some(3));
    assert_eq!(atom.radical, bondline_model::Radical::Doublet);
    let point = atom.position.ok_or("position")?;
    assert!((point.x_pt() - 42.3).abs() < 1e-10);
    assert!((point.y_pt() - 20.3784).abs() < 1e-10);
    assert!(!serde_json::to_string(&opened)?.contains("synthetic private header"));
    let saved = engine.dispatch("document.save_mol", json!({"path":copy}))?;
    assert_eq!(saved.revision, 1);
    assert_eq!(saved.undo_depth, 0);
    assert_eq!(fs::read(&copy)?, bytes);
    let wire = serde_json::to_value(saved)?;
    assert_eq!(wire["file"]["format"], "mol_v2000");
    assert_eq!(wire["file"]["dirty"], false);
    assert_eq!(wire["file_operation"]["operation"], "save");
    assert_eq!(wire["file_operation"]["format"], "mol_v2000");
    assert_eq!(wire["file_operation"]["input_revision"], 1);
    assert_eq!(read_mol(&copy)?["formula"], "CH3");
    Ok(())
}

#[test]
fn native_bond_mol_save_and_reopen_use_the_declared_frame_and_real_external_consumer() -> TestResult
{
    let directory = FileTestDirectory::new()?;
    let path = directory.path("native.mol");
    let mut engine = Engine::default();
    bond(&mut engine, 200.0)?;
    let document = engine.document().clone();
    let saved = engine.dispatch("document.save_mol", json!({"path":path}))?;
    assert_eq!(*saved.document, document);
    assert_eq!(saved.revision, 1);
    assert_eq!(saved.undo_depth, 1);
    assert!(!saved.file.dirty);
    assert!(saved.source.is_none());
    let external = read_mol(&path)?;
    assert_eq!(external["smiles"], "CC");
    assert_eq!(external["formula"], "C2H6");
    assert_eq!(
        external["atoms"][0]["position"],
        json!([1.1667, -5.3333, 0.0])
    );
    assert_eq!(
        external["atoms"][1]["position"],
        json!([2.8333, -5.3333, 0.0])
    );
    let mut reopened = Engine::default();
    let result = reopened.dispatch("document.open_mol", json!({"path":path}))?;
    let atoms = result
        .document
        .pages()
        .first()
        .ok_or("page")?
        .fragments()
        .first()
        .ok_or("fragment")?
        .atoms();
    assert_eq!(atoms.len(), 2);
    assert!(atoms.iter().all(|a| a.explicit_hydrogens == Some(3)));
    let point = atoms.first().ok_or("atom")?.position.ok_or("position")?;
    assert!((point.x_pt() - 100.0).abs() <= 0.0013);
    assert!((point.y_pt() - 200.0).abs() <= 0.0013);
    Ok(())
}

#[test]
fn mol_refusals_do_not_replace_authored_state_or_publish_lossy_files() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let input = directory.path("input.mol");
    fs::write(&input, CH3)?;
    let bad = directory.path("bad.mol");
    fs::write(&bad, b"malformed")?;
    let mut engine = Engine::default();
    bond(&mut engine, 200.0)?;
    let before = snapshot(&engine)?;
    assert_eq!(
        engine
            .dispatch("document.open_mol", json!({"path":input}))
            .err()
            .ok_or("discarded dirty doc")?
            .info()
            .code,
        "unsaved_changes"
    );
    assert_eq!(snapshot(&engine)?, before);
    assert_eq!(
        engine
            .dispatch(
                "document.open_mol",
                json!({"path":bad,"discard_unsaved":true})
            )
            .err()
            .ok_or("loaded malformed")?
            .info()
            .code,
        "invalid_mol"
    );
    assert_eq!(snapshot(&engine)?, before);
    engine.dispatch(
        "document.open_mol",
        json!({"path":input,"discard_unsaved":true}),
    )?;
    let before = snapshot(&engine)?;
    let output = directory.path("must-not-drop.cdxml");
    assert_eq!(
        engine
            .dispatch("document.save_cdxml", json!({"path":output}))
            .err()
            .ok_or("dropped MOL headers")?
            .info()
            .code,
        "unsupported_content"
    );
    assert!(!output.exists());
    assert_eq!(snapshot(&engine)?, before);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":500}),
    )?;
    let before = snapshot(&engine)?;
    let output = directory.path("must-not-drop-page.mol");
    assert_eq!(
        engine
            .dispatch("document.save_mol", json!({"path":output}))
            .err()
            .ok_or("dropped page layout")?
            .info()
            .code,
        "unsupported_content"
    );
    assert!(!output.exists());
    assert_eq!(snapshot(&engine)?, before);
    Ok(())
}

#[test]
fn malformed_or_nonpositive_mol_frames_are_rejected_before_io() -> TestResult {
    let mut engine = Engine::default();
    let before = snapshot(&engine)?;
    for params in [
        json!({"path":"must-not-read.mol","points_per_unit":0}),
        json!({"path":"must-not-read.mol","points_per_unit":-1}),
        json!({"path":"must-not-read.mol","origin_x_pt":100001}),
        json!({"path":"must-not-read.mol","unexpected":true}),
    ] {
        assert_eq!(
            engine
                .dispatch("document.open_mol", params)
                .err()
                .ok_or("invalid frame accepted")?
                .info()
                .code,
            "invalid_params"
        );
        assert_eq!(snapshot(&engine)?, before);
    }
    Ok(())
}

#[test]
fn same_path_format_changes_invalidate_close_and_picker_but_same_format_save_is_noop() -> TestResult
{
    let directory = FileTestDirectory::new()?;
    let path = directory.path("format.data");
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    bond(&mut engine, 200.0)?;
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    file_support::independently_read_file(&path, 1)?;
    engine.dispatch("document.close", json!({"expected_revision":1}))?;
    engine.dispatch("document.save_mol", json!({"path":path,"overwrite":true}))?;
    assert_eq!(
        engine.state().close,
        bondline_commands::CloseState::NeedsDecision
    );
    assert!(!engine.complete_close());
    assert_eq!(snapshot(&engine)?["file"]["format"], "mol_v2000");
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"cancel"}),
    )?;
    let id = engine
        .dispatch("ui.file", json!({"expected_revision":1,"action":"open"}))?
        .ui_file
        .request
        .ok_or("request")?
        .id;
    engine.claim_ui_file(id)?;
    engine.dispatch("document.save_mol", json!({"path":path,"overwrite":true}))?;
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .as_ref()
            .ok_or("noop cancelled picker")?
            .id,
        id
    );
    engine.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
    assert!(engine.state().ui_file.request.is_none());
    assert_eq!(snapshot(&engine)?["file"]["format"], "cdxml");
    Ok(())
}

#[test]
fn stale_mol_open_is_rejected_and_old_save_receipt_keeps_newer_edits_dirty() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let input = directory.path("input.mol");
    fs::write(&input, CH3)?;
    let mut engine = Engine::default();
    let prepared = engine.prepare(Command::parse("document.open_mol", json!({"path":input}))?)?;
    bond(&mut engine, 200.0)?;
    let before = snapshot(&engine)?;
    assert_eq!(
        engine
            .finish(prepared.complete()?)
            .err()
            .ok_or("stale open applied")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(snapshot(&engine)?, before);
    let output = directory.path("older.mol");
    let prepared = engine.prepare(Command::parse("document.save_mol", json!({"path":output}))?)?;
    bond(&mut engine, 280.0)?;
    let result = engine.finish(prepared.complete()?)?;
    assert_eq!(result.revision, 2);
    assert_eq!(result.undo_depth, 2);
    assert!(result.file.dirty);
    assert_eq!(
        serde_json::to_value(result)?["file_operation"]["input_revision"],
        1
    );
    assert_eq!(read_mol(&output)?["smiles"], "CC");
    Ok(())
}

#[test]
fn known_mol_save_uses_its_associated_format_and_trusted_request_receipt() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let path = directory.path("misleading.cdxml");
    let mut engine = Engine::default();
    bond(&mut engine, 200.0)?;
    engine.dispatch("document.save_mol", json!({"path":path}))?;
    engine.set_ui_file_host(true);
    let id = engine
        .dispatch("ui.file", json!({"expected_revision":1,"action":"save"}))?
        .ui_file
        .request
        .ok_or("request")?
        .id;
    let bondline_commands::UiFilePlan::SaveExisting { format, params } =
        engine.claim_ui_file(id)?
    else {
        return Err("known MOL save opened a picker".into());
    };
    assert_eq!(format, bondline_commands::FileFormat::MolV2000);
    let before = snapshot(&engine)?;
    assert_eq!(
        engine
            .prepare_ui_file(id, Command::SaveCdxml(params.clone()))
            .err()
            .ok_or("extension overrode the associated format")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(snapshot(&engine)?, before);
    let prepared = engine.prepare_ui_file(
        id,
        Command::parse("document.save_mol", serde_json::to_value(params)?)?,
    )?;
    let result = serde_json::to_value(engine.finish(prepared.complete()?)?)?;
    assert_eq!(result["file_operation"]["ui_request_id"], id);
    assert_eq!(result["file_operation"]["ui_request_current"], true);
    assert_eq!(result["file_operation"]["format"], "mol_v2000");
    assert_eq!(read_mol(&path)?["smiles"], "CC");
    Ok(())
}

#[test]
fn format_and_baseline_aba_never_restore_a_running_request_receipt() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let path = directory.path("shared.data");
    let mut engine = Engine::default();
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    bond(&mut engine, 200.0)?;
    engine.set_ui_file_host(true);
    let id = engine
        .dispatch("ui.file", json!({"expected_revision":1,"action":"save"}))?
        .ui_file
        .request
        .ok_or("request")?
        .id;
    let bondline_commands::UiFilePlan::SaveExisting { params, .. } = engine.claim_ui_file(id)?
    else {
        return Err("known save opened a picker".into());
    };
    let prepared = engine.prepare_ui_file(id, Command::SaveCdxml(params))?;
    for name in ["document.save_mol", "document.save_cdxml"] {
        let direct =
            serde_json::to_value(engine.dispatch(name, json!({"path":path,"overwrite":true}))?)?;
        assert!(direct["file_operation"].get("ui_request_id").is_none());
        assert!(direct["file_operation"].get("ui_request_current").is_none());
    }
    let result = serde_json::to_value(engine.finish(prepared.complete()?)?)?;
    assert_eq!(result["revision"], 1);
    assert_eq!(result["file"]["format"], "cdxml");
    assert_eq!(result["file"]["dirty"], false);
    assert_eq!(result["file_operation"]["ui_request_id"], id);
    assert_eq!(result["file_operation"]["ui_request_current"], false);
    assert!(!engine.finish_ui_file(id, true));
    assert!(engine.state().ui_file.request.is_none());
    Ok(())
}
