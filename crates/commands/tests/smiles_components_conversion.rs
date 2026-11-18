use bondline_commands::Command;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn neutral_components_profile_uses_the_existing_conversion_command() -> TestResult {
    let command = Command::parse(
        "file.convert_smiles_to_cdxml",
        json!({
            "expected_revision":0,
            "input_path":"components-before.smi",
            "output_path":"components-not-created.cdxml",
            "bond_length_pt":48.0,
            "origin_x_pt":72.0,
            "origin_y_pt":72.0,
            "page_width_pt":595.0,
            "page_height_pt":842.0,
            "profile":"neutral_components_v1"
        }),
    )?;
    assert_eq!(command.name(), "file.convert_smiles_to_cdxml");
    Ok(())
}

use bondline_commands::Engine;
use serde_json::Value;
use std::{fs, path::Path};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod support;

const NAME: &str = "file.convert_smiles_to_cdxml";
const PROFILE: &str = "neutral_components_v1";
const INPUT: &[u8] = b"CC.O.C1CCCCC1\r\n";
const WHOLE: &[u8] = b"CC PRIVATE_FIRST\r\nO\tPRIVATE_SECOND\xff\0\n";

fn params(revision: u64, input: &Path, output: &Path) -> Value {
    json!({"expected_revision":revision,"input_path":input,"output_path":output,
        "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
        "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4,
        "profile":PROFILE})
}

#[path = "../../../xtask/oracles/components_test_support.rs"]
mod component_reference;
use component_reference::consume;

#[test]
fn all_components_survive_create_only_conversion_explicit_open_edit_history_and_save() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let source = directory.path("complete.smi");
    let input = directory.path("components.smi");
    let output = directory.path("components-λ.cdxml");
    fs::write(&source, WHOLE)?;
    fs::write(&input, INPUT)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":source}))?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(before["document"], Value::Null);
    assert_eq!(before["smiles_file"]["record_count"], 2);
    assert!(!before.to_string().contains("PRIVATE_"));

    let mut result = serde_json::to_value(engine.dispatch(NAME, params(1, &input, &output))?)?;
    let receipt = result
        .as_object_mut()
        .ok_or("result")?
        .remove("conversion")
        .ok_or("receipt")?;
    assert_eq!(result, before);
    assert_eq!(receipt["profile"], PROFILE);
    assert_eq!(receipt["input_revision"], 1);
    assert_eq!(receipt["input_byte_length"], INPUT.len());
    assert_eq!(receipt["atom_count"], 9);
    assert_eq!(receipt["bond_count"], 7);
    assert_eq!(receipt["output"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(receipt["output"]["byte_length"], fs::read(&output)?.len());
    assert!(result.get("file_operation").is_none());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(fs::read(&source)?, WHOLE);
    assert_eq!(fs::read(&input)?, INPUT);
    consume(&output, false)?;
    let single = fs::read(&output)?;

    engine.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert_eq!(engine.state().revision, 2);
    assert!(engine.stored_document()?.is_fully_mapped());
    let opened = engine.content().clone();
    let opened_stamp = engine.capture_stamp();
    let opened_state = serde_json::to_value(engine.state())?;
    let cpu = engine.dispatch(
        "render.capture",
        json!({
            "page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"
        }),
    )?;
    let gpu = engine.dispatch(
        "render.capture",
        json!({
            "page_id":1,"width_px":1024,"height_px":1024,"backend":"gpu"
        }),
    )?;
    assert_eq!(
        cpu.capture.ok_or("CPU PNG")?.png_base64,
        gpu.capture.ok_or("GPU PNG")?.png_base64
    );
    assert_eq!(serde_json::to_value(engine.state())?, opened_state);
    assert!(engine.capture_stamp_matches(&opened_stamp));
    engine.dispatch(
        "bond.set_order",
        json!({
            "expected_revision":2,"page_id":1,"fragment_id":1,"bond_id":1,
            "order":"double","hydrogen_policy":"rectify_neutral_carbon"
        }),
    )?;
    assert!(engine.state().file.dirty);
    assert_eq!(engine.state().undo_depth, 1);
    let edited_model = engine.document()?.clone();
    let edited = engine.stored_document()?.to_cdxml()?.into_owned();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &opened);
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), single);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), edited);
    let saved = directory.path("edited-化学.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":saved}))?;
    assert_eq!(fs::read(&saved)?, edited);
    consume(&saved, true)?;
    engine.dispatch("document.open_cdxml", json!({"path":saved}))?;
    assert_eq!(engine.document()?, &edited_model);
    assert!(engine.stored_document()?.is_fully_mapped());
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.state().undo_depth, 0);
    assert_eq!(fs::read(&output)?, single);
    assert_eq!(fs::read(&source)?, WHOLE);
    assert_eq!(fs::read(&input)?, INPUT);
    Ok(())
}

#[test]
fn later_unsupported_components_and_existing_outputs_publish_nothing_and_keep_state() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("components.smi");
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({
            "page_id":1,"begin_x_pt":72.0,"begin_y_pt":144.0,
            "end_x_pt":120.0,"end_y_pt":144.0
        }),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let oversized = vec!["C"; 65].join(".");
    for (index, (notation, code)) in [
        ("CC.c1ccccc1", "unsupported_content"),
        ("CC.[13CH4]", "unsupported_content"),
        ("CC.[NH4+]", "unsupported_content"),
        ("CC.C1=CCCCC1", "unsupported_content"),
        ("CC.C1CCCCCCCC1", "unsupported_content"),
        (oversized.as_str(), "resource_limit"),
    ]
    .into_iter()
    .enumerate()
    {
        fs::write(&input, notation)?;
        let output = directory.path(&format!("never-{index}.cdxml"));
        assert_eq!(
            engine
                .dispatch(NAME, params(1, &input, &output))
                .err()
                .ok_or("accepted unsupported member")?
                .info()
                .code,
            code
        );
        assert!(!output.try_exists()?);
        assert_eq!(fs::read(&input)?, notation.as_bytes());
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert!(engine.capture_stamp_matches(&stamp));
    }

    fs::write(&input, INPUT)?;
    for profile in [
        "neutral_carbon_path_v1",
        "neutral_acyclic_v1",
        "neutral_simple_ring_v1",
    ] {
        let output = directory.path(&format!("legacy-{profile}.cdxml"));
        let mut old = params(1, &input, &output);
        old["profile"] = json!(profile);
        assert_eq!(
            engine
                .dispatch(NAME, old)
                .err()
                .ok_or("legacy profile widened")?
                .info()
                .code,
            "unsupported_content"
        );
        assert!(!output.try_exists()?);
    }
    let output = directory.path("sentinel.cdxml");
    fs::write(&output, b"original destination")?;
    assert_eq!(
        engine
            .dispatch(NAME, params(1, &input, &output))
            .err()
            .ok_or("overwrote existing destination")?
            .info()
            .code,
        "already_exists"
    );
    assert_eq!(fs::read(&output)?, b"original destination");
    let stale = directory.path("stale.cdxml");
    assert_eq!(
        engine
            .dispatch(NAME, params(0, &input, &stale))
            .err()
            .ok_or("accepted stale revision")?
            .info()
            .code,
        "stale_revision"
    );
    assert!(!stale.try_exists()?);
    let mut extra = params(1, &input, &stale);
    extra["component_gap_pt"] = json!(48.0);
    assert_eq!(
        engine
            .dispatch(NAME, extra)
            .err()
            .ok_or("accepted undeclared packing parameter")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    let retry = directory.path("retry-λ.cdxml");
    engine.dispatch(NAME, params(1, &input, &retry))?;
    consume(&retry, false)?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(fs::read(&input)?, INPUT);
    Ok(())
}
