//! Explicit graph-only export retains its declared context loss; it is not Save.
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::{DrawingSettings, PageSize};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write as _,
    process::{Command, Stdio},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the unchanged isolated-directory helper"
)]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

#[test]
fn explicit_fragment_export_declares_settings_loss_without_changing_whole_session() -> TestResult {
    let mut stored = StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    stored
        .model_mut()
        .set_drawing_settings(DrawingSettings::new(Some(30.0), Some(120.0))?);
    let mut engine = Engine::from_document(stored);
    let state = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let snapshot = engine.stored_document()?.model_snapshot();
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("explicit-graph-only-λ.smi");
    let result = engine.dispatch(
        "file.export_smiles_fragment",
        json!({
            "expected_revision":0,"page_id":1,"fragment_id":1,"output_path":path
        }),
    )?;
    let result = serde_json::to_value(result)?;
    assert_eq!(result["changed"], false);
    assert!(result.get("file_operation").is_none());
    assert_eq!(
        result["smiles_export"]["scope"],
        "selected_fragment_chemical_graph"
    );
    assert_eq!(
        result["smiles_export"]["not_encoded"],
        json!(["geometry", "document_context"])
    );
    assert_eq!(result["smiles_export"]["atom_count"], 2);
    assert_eq!(result["smiles_export"]["bond_count"], 1);
    assert_eq!(engine.content(), &content);
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert!(std::sync::Arc::ptr_eq(
        &snapshot,
        &engine.stored_document()?.model_snapshot()
    ));
    let actual = fs::read(&path)?;
    assert_eq!(actual, b"[CH3][CH3]\n");

    // Existing unchanged external reader. The source literal independently fixes
    // ethane; this consumer does not establish preservation of drawing settings.
    let mut child = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_write_oracle.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(actual.strip_suffix(b"\n").ok_or("one record LF")?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["notation"]["source_ascii"], "[CH3][CH3]");
    assert_eq!(value["notation"]["rdkit"], "2026.09.2");
    assert_eq!(value["meaning"]["formula"], "C2H6");
    assert_eq!(value["meaning"]["smiles"], "CC");
    assert_eq!(value["meaning"]["radicals"], json!([0, 0]));
    assert_eq!(value["meaning"]["conformers"], 0);
    Ok(())
}
