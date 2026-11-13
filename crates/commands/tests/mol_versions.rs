use bondline_commands::{Command, Engine, registry};
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
const V3: &[u8] = include_bytes!("../../io/tests/fixtures/v3000/noncontiguous-cc.mol");

fn snapshot(engine: &Engine) -> Result<Value, serde_json::Error> {
    serde_json::to_value(engine.state())
}
fn bond(engine: &mut Engine) -> TestResult {
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}),
    )?;
    Ok(())
}
fn stamp(path: &Path) -> TestResult {
    assert!(
        std::str::from_utf8(&fs::read(path)?)?
            .lines()
            .nth(3)
            .ok_or("stamp")?
            .ends_with("V3000")
    );
    Ok(())
}
fn external(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
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
            "/../io/tests/v3000_oracle.py"
        ))
        .arg("describe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle input")?
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
fn open_mol_decodes_actual_v3000_and_default_save_preserves_version_bytes_and_state() -> TestResult
{
    let directory = FileTestDirectory::new()?;
    let input = directory.path("v3-with-misleading.cdxml");
    let copy = directory.path("copy.mol");
    fs::write(&input, V3)?;
    let mut engine = Engine::default();
    let opened =
        serde_json::to_value(engine.dispatch("document.open_mol", json!({"path":input}))?)?;
    assert_eq!(opened["file"]["format"], "mol_v3000");
    assert_eq!(opened["file"]["dirty"], false);
    assert_eq!(
        opened["source"],
        json!({"format":"mol","byte_length":V3.len(),"modified":false})
    );
    assert_eq!(
        opened["document"]["pages"][0]["fragments"][0]["atoms"][0]["position"],
        json!({"x_pt":42.0,"y_pt":60.0})
    );
    assert_eq!(
        opened["document"]["pages"][0]["fragments"][0]["atoms"][0]["explicit_hydrogens"],
        3
    );
    assert!(!serde_json::to_string(&opened)?.contains("Original specification-derived vector"));
    let saved = serde_json::to_value(engine.dispatch("document.save_mol", json!({"path":copy}))?)?;
    assert_eq!(saved["file"]["format"], "mol_v3000");
    assert_eq!(saved["file_operation"]["format"], "mol_v3000");
    assert_eq!(saved["revision"], 1);
    assert_eq!(fs::read(&copy)?, V3);
    let before = snapshot(&engine)?;
    let lossy = directory.path("downgrade.mol");
    assert_eq!(
        engine
            .dispatch("document.save_mol", json!({"path":lossy,"version":"v2000"}))
            .err()
            .ok_or("silently downgraded")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(snapshot(&engine)?, before);
    assert!(!lossy.exists());
    Ok(())
}

#[test]
fn native_v3000_save_reopens_with_full_precision_and_auto_keeps_association() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let path = directory.path("native.mol");
    let copy = directory.path("copy.mol");
    let mut engine = Engine::default();
    bond(&mut engine)?;
    let model = engine.document().clone();
    let saved = serde_json::to_value(
        engine.dispatch("document.save_mol", json!({"path":path,"version":"v3000"}))?,
    )?;
    assert_eq!(saved["file"]["format"], "mol_v3000");
    assert_eq!(saved["file"]["dirty"], false);
    assert!(saved.get("source").is_none());
    stamp(&path)?;
    let consumed = external(&path)?;
    assert_eq!(consumed["atoms"][0]["element"], 6);
    assert_eq!(consumed["atoms"][0]["explicit_h"], 3);
    let raw_x = consumed["atoms"][0]["position"][0]
        .as_f64()
        .ok_or("raw x")?;
    assert!(
        (raw_x - 28.0 / 24.0).abs() < 1e-14,
        "V3000 must not use V2000 four-decimal coordinates"
    );
    engine.dispatch("document.save_mol", json!({"path":copy}))?;
    stamp(&copy)?;
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_mol", json!({"path":copy}))?;
    assert_eq!(snapshot(&reopened)?["file"]["format"], "mol_v3000");
    let before = model
        .pages()
        .first()
        .ok_or("before page")?
        .fragments()
        .first()
        .ok_or("before fragment")?;
    let after = reopened
        .document()
        .pages()
        .first()
        .ok_or("after page")?
        .fragments()
        .first()
        .ok_or("after fragment")?;
    for (expected, actual) in before.atoms().iter().zip(after.atoms()) {
        let expected = expected.position.ok_or("before position")?;
        let actual = actual.position.ok_or("after position")?;
        assert!((actual.x_pt() - expected.x_pt()).abs() < 1e-10);
        assert!((actual.y_pt() - expected.y_pt()).abs() < 1e-10);
    }
    Ok(())
}

#[test]
fn save_versions_have_closed_schema_and_unknown_versions_never_reach_io() -> TestResult {
    let schema = serde_json::to_value(
        registry()
            .into_iter()
            .find(|item| item.name == "document.save_mol")
            .ok_or("registry")?,
    )?;
    assert_eq!(
        schema["input_schema"]["properties"]["version"]["default"],
        "auto"
    );
    assert_eq!(schema["input_schema"]["additionalProperties"], false);
    let mut engine = Engine::default();
    let before = snapshot(&engine)?;
    for params in [
        json!({"path":"unused.mol","version":"v4000"}),
        json!({"path":"unused.mol","version":null}),
        json!({"path":"unused.mol","encoding":"v3000"}),
    ] {
        assert_eq!(
            engine
                .dispatch("document.save_mol", params)
                .err()
                .ok_or("bad version accepted")?
                .info()
                .code,
            "invalid_params"
        );
        assert_eq!(snapshot(&engine)?, before);
    }
    Ok(())
}

#[test]
fn same_path_version_changes_invalidate_picker_and_known_save_cannot_downgrade() -> TestResult {
    let directory = FileTestDirectory::new()?;
    let path = directory.path("association.data");
    let mut engine = Engine::default();
    bond(&mut engine)?;
    engine.dispatch("document.save_mol", json!({"path":path,"version":"v3000"}))?;
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
        return Err("known save opened picker".into());
    };
    assert_eq!(serde_json::to_value(format)?, "mol_v3000");
    let before = snapshot(&engine)?;
    assert_eq!(
        engine
            .prepare_ui_file(
                id,
                Command::parse(
                    "document.save_mol",
                    json!({"path":params.path,"overwrite":true,"version":"v2000"})
                )?
            )
            .err()
            .ok_or("known save version changed")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(snapshot(&engine)?, before);
    let prepared = engine.prepare_ui_file(
        id,
        Command::parse(
            "document.save_mol",
            json!({"path":params.path,"overwrite":true}),
        )?,
    )?;
    let saved = serde_json::to_value(engine.finish(prepared.complete()?)?)?;
    assert_eq!(saved["file_operation"]["format"], "mol_v3000");
    assert_eq!(saved["file_operation"]["ui_request_current"], true);
    engine.finish_ui_file(id, false);
    let id = engine
        .dispatch("ui.file", json!({"expected_revision":1,"action":"open"}))?
        .ui_file
        .request
        .ok_or("open request")?
        .id;
    engine.claim_ui_file(id)?;
    engine.dispatch("document.save_mol", json!({"path":path,"overwrite":true}))?;
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .ok_or("auto no-op cancelled picker")?
            .id,
        id
    );
    engine.dispatch(
        "document.save_mol",
        json!({"path":path,"overwrite":true,"version":"v2000"}),
    )?;
    assert!(engine.state().ui_file.request.is_none());
    assert_eq!(snapshot(&engine)?["file"]["format"], "mol_v2000");
    let xml = directory.path("native-after-versions.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":xml}))?;
    file_support::independently_read_file(&xml, 1)?;
    Ok(())
}
