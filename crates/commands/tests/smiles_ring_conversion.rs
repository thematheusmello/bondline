use bondline_commands::Command;
use serde_json::json;

#[test]
fn simple_ring_profile_is_an_explicit_typed_conversion_choice()
-> Result<(), Box<dyn std::error::Error>> {
    let command = Command::parse(
        "file.convert_smiles_to_cdxml",
        json!({
            "expected_revision":0,"input_path":"not-read.smi","output_path":"not-written.cdxml",
            "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
            "page_width_pt":595.2755905511812,"page_height_pt":841.8897637795276,
            "profile":"neutral_simple_ring_v1"
        }),
    )?;
    let Command::ConvertSmilesToCdxml(params) = command else {
        return Err("conversion routed to a different command".into());
    };
    assert_eq!(
        serde_json::to_value(params)?["profile"],
        "neutral_simple_ring_v1"
    );
    Ok(())
}

use bondline_commands::{Cancellation, Engine};
use serde_json::Value;
use std::{fs, path::Path, process::Command as Process};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const NAME: &str = "file.convert_smiles_to_cdxml";
const PROFILE: &str = "neutral_simple_ring_v1";
const INPUT: &[u8] = b"CC1CCCCC1\r\n";
const COMPLETE_SOURCE: &[u8] = b" ignored\r\nCC PRIVATE_FIRST\r\nO\tPRIVATE_SECOND\xff\0\n";

fn params(revision: u64, input: &Path, output: &Path) -> Value {
    json!({"expected_revision":revision,"input_path":input,"output_path":output,
        "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
        "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4,
        "profile":PROFILE})
}

fn consume(path: &Path, edited: bool) -> TestResult {
    let (notation, atoms, formula, first_order) = if edited {
        (
            "C=C1CCCCC1",
            json!([[6, 2], [6, 0], [6, 2], [6, 2], [6, 2], [6, 2], [6, 2]]),
            "C7H12",
            2,
        )
    } else {
        (
            "CC1CCCCC1",
            json!([[6, 3], [6, 1], [6, 2], [6, 2], [6, 2], [6, 2], [6, 2]]),
            "C7H14",
            1,
        )
    };
    let output = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_ring_oracle.py"
        ))
        .arg(notation)
        .arg(
            json!({"atoms":atoms,"formula":formula,
            "bonds":[[1,2,first_order],[2,3,1],[3,4,1],[4,5,1],[5,6,1],[6,7,1],[2,7,1]],
            "length":"48","origin":[72,72],"page":[210.0*72.0/25.4,297.0*72.0/25.4]})
            .to_string(),
        )
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["atoms"], 7);
    assert_eq!(value["bonds"], 7);
    assert_eq!(value["formula"], formula);
    assert_eq!(value["no_assigned_stereo"], true);
    Ok(())
}

#[test]
fn ring_conversion_preserves_each_complete_content_owner_and_only_explicit_open_replaces_it()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for kind in ["drawing", "smiles", "sdf"] {
        let mut engine = Engine::default();
        let active = directory.path(&format!("active-{kind}"));
        match kind {
            "drawing" => {
                fs::write(
                    &active,
                    include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml"),
                )?;
                engine.dispatch("document.open_cdxml", json!({"path":active}))?;
                engine.dispatch(
                    "page.set_size",
                    json!({"page_id":1,"width_pt":700.0,"height_pt":900.0}),
                )?;
                engine.dispatch(
                    "page.set_size",
                    json!({"page_id":1,"width_pt":710.0,"height_pt":910.0}),
                )?;
                engine.dispatch("history.undo", json!({}))?;
                assert!(engine.state().file.dirty);
                assert_eq!(engine.state().redo_depth, 1);
            }
            "smiles" => {
                fs::write(&active, COMPLETE_SOURCE)?;
                engine.dispatch("document.open_smiles", json!({"path":active}))?;
                assert!(engine.state().document.is_none());
            }
            "sdf" => {
                fs::write(
                    &active,
                    include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"),
                )?;
                engine.dispatch("document.open_sdf", json!({"path":active}))?;
                assert!(engine.state().document.is_none());
            }
            _ => return Err("undeclared owner".into()),
        }
        let original = fs::read(&active)?;
        let content = engine.content().clone();
        let stamp = engine.capture_stamp();
        let before = serde_json::to_value(engine.state())?;
        let input = directory.path(&format!("input-{kind}.smi"));
        let output = directory.path(&format!("ring-{kind}-λ.not-xml"));
        fs::write(&input, INPUT)?;
        let mut result = serde_json::to_value(
            engine.dispatch(NAME, params(engine.state().revision, &input, &output))?,
        )?;
        let receipt = result
            .as_object_mut()
            .ok_or("state")?
            .remove("conversion")
            .ok_or("receipt")?;
        assert_eq!(receipt["profile"], PROFILE);
        assert_eq!(receipt["input_revision"], before["revision"]);
        assert_eq!(receipt["atom_count"], 7);
        assert_eq!(receipt["bond_count"], 7);
        assert_eq!(receipt["input_byte_length"], INPUT.len());
        assert_eq!(receipt["output"]["path"], json!(fs::canonicalize(&output)?));
        assert_eq!(receipt["output"]["byte_length"], fs::read(&output)?.len());
        assert!(result.get("file_operation").is_none());
        assert_eq!(result, before);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert!(engine.capture_stamp_matches(&stamp));
        assert_eq!(fs::read(&active)?, original);
        assert_eq!(fs::read(&input)?, INPUT);
        consume(&output, false)?;
        if kind == "drawing" {
            let error = engine
                .dispatch("document.open_cdxml", json!({"path":output}))
                .err()
                .ok_or("dirty Open accepted")?;
            assert_eq!(error.info().code, "unsaved_changes");
            assert_eq!(engine.content(), &content);
        }
        engine.dispatch(
            "document.open_cdxml",
            json!({"path":output,"discard_unsaved":true}),
        )?;
        assert!(engine.stored_document()?.is_fully_mapped());
        assert!(!engine.state().file.dirty);
        assert_eq!(
            engine.document()?.pages()[0].fragments()[0].atoms().len(),
            7
        );
    }
    Ok(())
}

#[test]
fn the_derived_ring_is_renderable_editable_and_preserves_history_and_independent_files()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("methyl.smi");
    let output = directory.path("methyl-λ.cdxml");
    fs::write(&input, INPUT)?;
    let mut engine = Engine::default();
    engine.dispatch(NAME, params(0, &input, &output))?;
    let single = fs::read(&output)?;
    consume(&output, false)?;
    engine.dispatch("document.open_cdxml", json!({"path":output}))?;
    let before = serde_json::to_value(engine.state())?;
    let cpu = engine.dispatch(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
    )?;
    let gpu = engine.dispatch(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"gpu"}),
    )?;
    assert_eq!(
        cpu.capture.ok_or("CPU PNG")?.png_base64,
        gpu.capture.ok_or("GPU PNG")?.png_base64
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    engine.dispatch(
        "bond.set_order",
        json!({"expected_revision":1,"page_id":1,"fragment_id":1,"bond_id":1,
        "order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
    )?;
    assert!(engine.state().file.dirty);
    assert_eq!(engine.state().undo_depth, 1);
    let edited = engine.stored_document()?.to_cdxml()?.into_owned();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), single);
    assert!(!engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), edited);
    let saved = directory.path("edited-化学.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":saved}))?;
    assert_eq!(fs::read(&saved)?, edited);
    consume(&saved, true)?;
    engine.dispatch("document.open_cdxml", json!({"path":saved}))?;
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), edited);
    assert!(engine.stored_document()?.is_fully_mapped());
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.state().undo_depth, 0);
    assert_eq!(engine.state().redo_depth, 0);
    assert_eq!(fs::read(&input)?, INPUT);
    assert_eq!(fs::read(&output)?, single);
    Ok(())
}

#[test]
fn ring_refusals_and_captured_publication_obey_the_shared_save_lifetime() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("bounded.smi");
    let output = directory.path("absent.cdxml");
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    for (raw, code) in [
        ("CC\n", "unsupported_content"),
        ("C1CCCCC1.O\n", "unsupported_content"),
        ("C1CCC2CCCCC2C1\n", "unsupported_content"),
        ("C1CCCCCCCC1\n", "unsupported_content"),
        ("C1=CCCCC1\n", "unsupported_content"),
        ("c1ccccc1\n", "unsupported_content"),
        ("[13CH2]1CCCCC1\n", "unsupported_content"),
        ("C1(\n", "invalid_smiles"),
        ("C1CCCCC1 opaque\n", "invalid_smiles"),
    ] {
        fs::write(&input, raw)?;
        assert_eq!(
            engine
                .dispatch(NAME, params(0, &input, &output))
                .err()
                .ok_or("refused source accepted")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(fs::read(&input)?, raw.as_bytes());
        assert!(!output.try_exists()?);
    }
    fs::write(&input, INPUT)?;
    for profile in ["neutral_carbon_path_v1", "neutral_acyclic_v1"] {
        let mut old = params(0, &input, &output);
        old["profile"] = json!(profile);
        assert_eq!(
            engine
                .dispatch(NAME, old)
                .err()
                .ok_or("old profile widened")?
                .info()
                .code,
            "unsupported_content"
        );
    }
    assert_eq!(
        engine
            .prepare(Command::parse(NAME, params(1, &input, &output))?)
            .err()
            .ok_or("stale accepted")?
            .info()
            .code,
        "stale_revision"
    );
    let prepared = engine.prepare(Command::parse(NAME, params(0, &input, &output))?)?;
    assert!(prepared.is_deferred());
    assert_eq!(
        engine
            .prepare(Command::parse(
                "document.save_cdxml",
                json!({"path":directory.path("blocked.cdxml")})
            )?)
            .err()
            .ok_or("lease escaped")?
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
            .ok_or("cancel ignored")?
            .info()
            .code,
        "cancelled"
    );
    assert!(!output.try_exists()?);
    fs::write(&output, b"retain original sentinel")?;
    assert_eq!(
        engine
            .dispatch(NAME, params(0, &input, &output))
            .err()
            .ok_or("existing output overwritten")?
            .info()
            .code,
        "already_exists"
    );
    assert_eq!(fs::read(&output)?, b"retain original sentinel");
    let published = directory.path("historical.cdxml");
    let completed = engine
        .prepare(Command::parse(NAME, params(0, &input, &published))?)?
        .complete()?;
    engine.dispatch("document.new", json!({"expected_revision":0}))?;
    let fresh = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let mut result = serde_json::to_value(engine.finish(completed)?)?;
    let receipt = result
        .as_object_mut()
        .ok_or("state")?
        .remove("conversion")
        .ok_or("receipt")?;
    assert_eq!(receipt["profile"], PROFILE);
    assert_eq!(receipt["input_revision"], 0);
    assert_eq!(receipt["session_generation"], 0);
    assert_eq!(result, fresh);
    assert_eq!(engine.content(), &content);
    assert_eq!(serde_json::to_value(engine.state())?, fresh);
    consume(&published, false)?;
    assert_eq!(fs::read(&input)?, INPUT);
    Ok(())
}
