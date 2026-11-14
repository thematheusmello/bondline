use bondline_commands::{Command, Engine};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command as Process};
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

fn extend(revision: u64, atom: u32, x: f64, y: f64) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"begin_atom_id":atom,
        "end_x_pt":x,"end_y_pt":y,"hydrogen_policy":"rectify_terminal_carbon"})
}

fn independent(path: &Path, count: u32, paper: &str) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Process::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/terminal_chain_oracle.py"
        ))
        .args([count.to_string(), paper.to_owned()])
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn saved_native_propane_reopens_extends_to_butane_and_retains_source_and_saved_baselines()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("native-chain.cdxml");
    fs::write(&path, NATIVE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":path}))?;
    let before = engine.stored_document()?.clone();
    engine.dispatch("bond.extend", extend(1, 2, 180.0, 240.0))?;
    let propane = engine.stored_document()?.clone();
    let saved = engine.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
    assert!(!saved.file.dirty);
    assert!(saved.source.ok_or("source")?.modified);
    let bytes = fs::read(&path)?;
    independent(&path, 3, "letter")?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document()?, &before);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), NATIVE);
    assert!(engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document()?, &propane);
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), bytes);
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_cdxml", json!({"path":path}))?;
    assert_eq!(reopened.document()?, engine.document()?);
    assert!(!reopened.state().source.ok_or("reopened source")?.modified);
    reopened.dispatch("bond.extend", extend(1, 3, 220.0, 280.0))?;
    reopened.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
    independent(&path, 4, "letter")?;
    Ok(())
}

#[test]
fn in_flight_saves_keep_their_immutable_input_and_stale_open_cannot_replace_an_extension()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("captured-ethane.cdxml");
    let newer = directory.path("propane.cdxml");
    let mut engine = Engine::default();
    engine.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0}))?;
    let save = engine.prepare(Command::parse("document.save_cdxml", json!({"path":path}))?)?;
    engine.dispatch("bond.extend", extend(1, 2, 180.0, 240.0))?;
    let completed = save.complete()?;
    assert_eq!(
        engine
            .prepare(Command::parse(
                "document.save_cdxml",
                json!({"path":newer})
            )?)
            .err()
            .ok_or("save slot released early")?
            .info()
            .code,
        "busy"
    );
    assert!(!newer.try_exists()?);
    support::independently_read_file(&path, 1)?;
    let receipt = engine.finish(completed)?;
    assert_eq!(receipt.file_operation.ok_or("receipt")?.input_revision, 1);
    assert!(receipt.file.dirty);
    engine.dispatch("document.save_cdxml", json!({"path":newer}))?;
    independent(&newer, 3, "a4")?;
    let prepared = engine.prepare(Command::parse(
        "document.open_cdxml",
        json!({"path":path,"discard_unsaved":true}),
    )?)?;
    engine.dispatch("bond.extend", extend(2, 3, 220.0, 280.0))?;
    let before = engine.stored_document()?.clone();
    let state = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .finish(prepared.complete()?)
            .err()
            .ok_or("stale open applied")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(engine.stored_document()?, &before);
    assert_eq!(serde_json::to_value(engine.state())?, state);
    Ok(())
}

#[test]
fn source_free_mol_association_does_not_turn_into_mol_source_authority() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("native.mol");
    let mut engine = Engine::default();
    engine.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0}))?;
    engine.dispatch("document.save_mol", json!({"path":path}))?;
    assert!(engine.state().source.is_none());
    assert_eq!(
        engine.state().file.format,
        bondline_commands::FileFormat::MolV2000
    );
    engine.dispatch("bond.extend", extend(1, 2, 180.0, 240.0))?;
    engine.dispatch("document.save_mol", json!({"path":path,"overwrite":true}))?;
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_mol", json!({"path":path}))?;
    assert_eq!(reopened.state().source.ok_or("MOL backing")?.format, "mol");
    let before = reopened.stored_document()?.clone();
    assert_eq!(
        reopened
            .dispatch("bond.extend", extend(1, 3, 220.0, 280.0))
            .err()
            .ok_or("MOL was treated as native")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(reopened.stored_document()?, &before);
    Ok(())
}
