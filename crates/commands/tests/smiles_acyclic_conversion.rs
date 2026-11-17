use bondline_commands::Command;
use serde_json::json;

#[test]
fn acyclic_profile_is_an_explicit_typed_conversion_choice() -> Result<(), Box<dyn std::error::Error>>
{
    let command = Command::parse(
        "file.convert_smiles_to_cdxml",
        json!({
            "expected_revision":0,"input_path":"not-read.smi","output_path":"not-written.cdxml",
            "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
            "page_width_pt":595.2755905511812,"page_height_pt":841.8897637795276,
            "profile":"neutral_acyclic_v1"
        }),
    )?;
    let Command::ConvertSmilesToCdxml(params) = command else {
        return Err("conversion routed to a different command".into());
    };
    assert_eq!(
        serde_json::to_value(params)?["profile"],
        "neutral_acyclic_v1"
    );
    Ok(())
}

use bondline_commands::{Cancellation, Engine};
use serde_json::Value;
use std::{fs, path::Path, process::Command as Process};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated file directory")]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const NAME: &str = "file.convert_smiles_to_cdxml";
const PROFILE: &str = "neutral_acyclic_v1";
const COMPLETE_SOURCE: &[u8] =
    b" ignored source line\r\nCC PRIVATE_FIRST\r\nO\tPRIVATE_SECOND\xff\0\n";

fn params(revision: u64, input: &Path, output: &Path) -> Value {
    json!({"expected_revision":revision,"input_path":input,"output_path":output,
        "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
        "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4,
        "profile":PROFILE})
}

fn consume(path: &Path, notation: &str, atoms: Value, bonds: Value, formula: &str) -> TestResult {
    let result = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_acyclic_oracle.py"
        ))
        .arg(notation)
        .arg(
            json!({"atoms":atoms,"bonds":bonds,"formula":formula,"length":"48",
            "origin":[72,72],"page":[210.0*72.0/25.4,297.0*72.0/25.4]})
            .to_string(),
        )
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        result.status.success(),
        "independent {notation} consumer: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout)?,
        json!({"atoms":atoms.as_array().ok_or("literal atoms")?.len(),
            "bonds":bonds.as_array().ok_or("literal bonds")?.len(),
            "formula":formula,"stereo_none":true,"physical_h":0,
            "no_assigned_stereo":true,
            "bond_stereo":vec!["STEREONONE"; bonds.as_array().ok_or("literal bonds")?.len()]})
    );
    Ok(())
}

fn without_receipt(mut result: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let receipt = result
        .as_object_mut()
        .ok_or("result")?
        .remove("conversion")
        .ok_or("conversion receipt")?;
    assert_eq!(receipt["profile"], PROFILE);
    assert!(result.get("file_operation").is_none());
    Ok(result)
}

#[test]
fn the_optional_closed_profile_preserves_the_old_parameter_and_file_contract() -> TestResult {
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 44);
    let entries = registry
        .iter()
        .filter(|entry| entry.name == NAME)
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    assert!(!entries[0].read_only);
    let schema = entries[0].input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"].as_array().ok_or("required")?.len(), 8);
    assert!(
        !schema["required"]
            .as_array()
            .ok_or("required")?
            .contains(&json!("profile"))
    );
    let property = &schema["properties"]["profile"];
    let options = if let Some(reference) = property["$ref"].as_str() {
        schema
            .pointer(reference.strip_prefix('#').ok_or("local enum reference")?)
            .ok_or("profile enum")?
    } else {
        property
    };
    let mut options = options["enum"].as_array().ok_or("profile choices")?.clone();
    options.sort_by_key(Value::to_string);
    assert_eq!(
        options,
        vec![
            json!(PROFILE),
            json!("neutral_carbon_path_v1"),
            json!("neutral_components_v1"),
            json!("neutral_simple_ring_v1")
        ]
    );
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("old-input.smi");
    fs::write(&input, b"CC\n")?;
    let mut old = params(0, &input, &directory.path("old.cdxml"));
    old.as_object_mut().ok_or("params")?.remove("profile");
    let Command::ConvertSmilesToCdxml(typed) = Command::parse(NAME, old.clone())? else {
        return Err("wrong command".into());
    };
    assert_eq!(
        serde_json::to_value(typed)?,
        old,
        "old serialization gains no field"
    );
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let omitted = serde_json::to_value(engine.dispatch(NAME, old.clone())?)?;
    assert_eq!(omitted["conversion"]["profile"], "neutral_carbon_path_v1");
    assert_eq!(omitted["conversion"]["input_revision"], 0);
    assert_eq!(omitted["conversion"]["atom_count"], 2);
    assert_eq!(omitted["conversion"]["bond_count"], 1);
    assert!(omitted.get("file_operation").is_none());
    let mut explicit = old.clone();
    explicit["profile"] = json!("neutral_carbon_path_v1");
    explicit["output_path"] = json!(directory.path("explicit-old.cdxml"));
    let result = serde_json::to_value(engine.dispatch(NAME, explicit)?)?;
    assert_eq!(result["conversion"]["profile"], "neutral_carbon_path_v1");
    assert_eq!(
        fs::read(directory.path("old.cdxml"))?,
        fs::read(directory.path("explicit-old.cdxml"))?
    );
    for bad in [
        Value::Null,
        json!(true),
        json!(0),
        json!(""),
        json!("acyclic"),
        json!("NEUTRAL_ACYCLIC_V1"),
    ] {
        let mut invalid = params(0, &input, &directory.path("never-created.cdxml"));
        invalid["profile"] = bad;
        assert_eq!(
            engine
                .dispatch(NAME, invalid)
                .err()
                .ok_or("invalid profile accepted")?
                .info()
                .code,
            "invalid_params"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    assert!(!directory.path("never-created.cdxml").try_exists()?);
    assert_eq!(fs::read(input)?, b"CC\n");
    Ok(())
}

#[test]
fn branched_hetero_carbonyl_and_triple_outputs_do_not_replace_whole_session_owners() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for smiles_owner in [false, true] {
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        let source = directory.path(if smiles_owner {
            "whole.smi"
        } else {
            "whole.cdxml"
        });
        if smiles_owner {
            fs::write(&source, COMPLETE_SOURCE)?;
            engine.dispatch("document.open_smiles", json!({"path":source}))?;
            assert!(engine.state().document.is_none());
        } else {
            fs::write(
                &source,
                include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml"),
            )?;
            engine.dispatch("document.open_cdxml", json!({"path":source}))?;
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
        let revision = engine.state().revision;
        engine.dispatch(
            "document.close",
            json!({"expected_revision":revision,"action":"request"}),
        )?;
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        let source_bytes = fs::read(&source)?;
        for (case, notation, atoms, bonds, formula) in [
            (
                "branch",
                "CC(C)O",
                json!([[6, 3], [6, 1], [6, 3], [8, 1]]),
                json!([[1, 2, 1], [2, 3, 1], [2, 4, 1]]),
                "C3H8O",
            ),
            (
                "carbonyl",
                "CC(=O)O",
                json!([[6, 3], [6, 0], [8, 0], [8, 1]]),
                json!([[1, 2, 1], [2, 3, 2], [2, 4, 1]]),
                "C2H4O2",
            ),
            (
                "triple",
                "CC#N",
                json!([[6, 3], [6, 0], [7, 0]]),
                json!([[1, 2, 1], [2, 3, 3]]),
                "C2H3N",
            ),
        ] {
            let input = directory.path(&format!("{smiles_owner}-{case}.smi"));
            let output = directory.path(&format!("{smiles_owner}-{case}-化学.mol"));
            let raw = format!("{notation}\r\n");
            fs::write(&input, raw.as_bytes())?;
            let result =
                serde_json::to_value(engine.dispatch(NAME, params(revision, &input, &output))?)?;
            assert_eq!(result["conversion"]["profile"], PROFILE);
            assert_eq!(result["conversion"]["input_revision"], revision);
            assert_eq!(result["conversion"]["session_generation"], 1);
            assert_eq!(
                result["conversion"]["input_path"],
                json!(fs::canonicalize(&input)?)
            );
            assert_eq!(result["conversion"]["input_byte_length"], raw.len());
            assert_eq!(
                result["conversion"]["atom_count"],
                atoms.as_array().ok_or("atoms")?.len()
            );
            assert_eq!(
                result["conversion"]["bond_count"],
                bonds.as_array().ok_or("bonds")?.len()
            );
            assert_eq!(
                result["conversion"]["output"]["byte_length"],
                fs::read(&output)?.len()
            );
            assert_eq!(
                result["conversion"]["output"]["path"],
                json!(fs::canonicalize(&output)?)
            );
            assert!(result["conversion"]["output"]["directory_synced"].is_boolean());
            assert_eq!(without_receipt(result)?, before);
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert_eq!(engine.content(), &content);
            assert_eq!(fs::read(&source)?, source_bytes);
            assert_eq!(fs::read(input)?, raw.as_bytes());
            consume(&output, notation, atoms, bonds, formula)?;
        }
        if smiles_owner {
            assert!(!before.to_string().contains("PRIVATE_"));
            engine.dispatch(
                "document.close",
                json!({"expected_revision":revision,"action":"cancel"}),
            )?;
            let saved = directory.path("whole-source-copy.smi");
            engine.dispatch("document.save_smiles", json!({"path":saved}))?;
            assert_eq!(fs::read(saved)?, COMPLETE_SOURCE);
        }
    }
    Ok(())
}

#[test]
fn explicit_open_of_a_converted_carbon_branch_keeps_edit_history_and_native_save_real() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("carbon-branch.smi");
    let output = directory.path("carbon-branch-λ.cdxml");
    fs::write(&input, b"CC(C)C\n")?;
    let mut engine = Engine::default();
    let empty = serde_json::to_value(engine.state())?;
    engine.dispatch(NAME, params(0, &input, &output))?;
    assert_eq!(serde_json::to_value(engine.state())?, empty);
    consume(
        &output,
        "CC(C)C",
        json!([[6, 3], [6, 1], [6, 3], [6, 3]]),
        json!([[1, 2, 1], [2, 3, 1], [2, 4, 1]]),
        "C4H10",
    )?;
    let original = fs::read(&output)?;
    engine.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert!(engine.stored_document()?.is_fully_mapped());
    assert_eq!(engine.state().revision, 1);
    assert_eq!(engine.state().undo_depth, 0);
    assert!(!engine.state().file.dirty);
    let capture_before = serde_json::to_value(engine.state())?;
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
    assert_eq!(serde_json::to_value(engine.state())?, capture_before);
    let edit = json!({"expected_revision":1,"page_id":1,"fragment_id":1,"bond_id":1,
        "order":"double","hydrogen_policy":"rectify_neutral_carbon"});
    engine.dispatch("bond.set_order", edit)?;
    assert_eq!(engine.state().revision, 2);
    assert_eq!(engine.state().undo_depth, 1);
    assert!(engine.state().file.dirty);
    assert!(engine.state().source.ok_or("source")?.modified);
    let edited = engine.stored_document()?.to_cdxml()?.into_owned();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), original);
    assert!(!engine.state().file.dirty);
    assert!(!engine.state().source.ok_or("source")?.modified);
    assert_eq!(engine.state().redo_depth, 1);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), edited);
    let saved = directory.path("edited-化学.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":saved}))?;
    assert_eq!(fs::read(&saved)?, edited);
    assert!(!engine.state().file.dirty);
    consume(
        &saved,
        "C=C(C)C",
        json!([[6, 2], [6, 0], [6, 3], [6, 3]]),
        json!([[1, 2, 2], [2, 3, 1], [2, 4, 1]]),
        "C4H8",
    )?;
    let mol = directory.path("edited.v3000");
    engine.dispatch("document.save_mol", json!({"path":mol,"version":"v3000"}))?;
    let consumer = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/mol_oracle.py"
        ))
        .stdin(fs::File::open(&mol)?)
        .output()?;
    assert!(
        consumer.status.success(),
        "{}",
        String::from_utf8_lossy(&consumer.stderr)
    );
    let consumed: Value = serde_json::from_slice(&consumer.stdout)?;
    assert_eq!(consumed["formula"], "C4H8");
    assert_eq!(
        consumed["atoms"]
            .as_array()
            .ok_or("MOL atoms")?
            .iter()
            .map(|a| a["hydrogens"].clone())
            .collect::<Vec<_>>(),
        vec![json!(2), json!(0), json!(3), json!(3)]
    );
    assert_eq!(
        consumed["bonds"],
        json!([{"begin":0,"end":1,"order":2.0},{"begin":1,"end":2,"order":1.0},{"begin":1,"end":3,"order":1.0}])
    );
    engine.dispatch("document.open_cdxml", json!({"path":saved}))?;
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), edited);
    assert!(engine.stored_document()?.is_fully_mapped());
    assert_eq!(engine.state().undo_depth, 0);
    assert_eq!(engine.state().redo_depth, 0);
    assert!(!engine.state().file.dirty);
    assert!(!engine.state().source.ok_or("source")?.modified);
    assert_eq!(fs::read(&input)?, b"CC(C)C\n");
    Ok(())
}

#[test]
fn profile_refusals_cancellation_and_historical_publication_keep_the_shared_save_lease()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("lease.smi");
    let output = directory.path("lease.cdxml");
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    for (raw, code) in [
        (b"C1CC1\n".as_slice(), "unsupported_content"),
        (b"CC.O\n".as_slice(), "unsupported_content"),
        (b"c1ccccc1\n".as_slice(), "unsupported_content"),
        (b"[13CH3]C\n".as_slice(), "unsupported_content"),
        (b"[NH4+]\n".as_slice(), "unsupported_content"),
        (b"C(\n".as_slice(), "invalid_smiles"),
        (b"CC private-data\n".as_slice(), "invalid_smiles"),
    ] {
        fs::write(&input, raw)?;
        assert_eq!(
            engine
                .dispatch(NAME, params(0, &input, &output))
                .err()
                .ok_or("unsupported input accepted")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(fs::read(&input)?, raw);
        assert!(!output.try_exists()?);
    }
    fs::write(&input, b"CO\n")?;
    let mut old = params(0, &input, &output);
    old.as_object_mut().ok_or("params")?.remove("profile");
    assert_eq!(
        engine
            .dispatch(NAME, old)
            .err()
            .ok_or("default profile widened")?
            .info()
            .code,
        "unsupported_content"
    );
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
            .ok_or("cancel accepted")?
            .info()
            .code,
        "cancelled"
    );
    assert!(!output.try_exists()?);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    fs::write(&output, b"existing unrelated bytes")?;
    assert_eq!(
        engine
            .dispatch(NAME, params(0, &input, &output))
            .err()
            .ok_or("destination clobbered")?
            .info()
            .code,
        "already_exists"
    );
    assert_eq!(fs::read(&output)?, b"existing unrelated bytes");
    let published = directory.path("historical.cdxml");
    let completed = engine
        .prepare(Command::parse(NAME, params(0, &input, &published))?)?
        .complete()?;
    engine.dispatch("document.new", json!({"expected_revision":0}))?;
    let fresh = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    assert_eq!(
        engine
            .prepare(Command::parse(
                "document.save_cdxml",
                json!({"path":directory.path("still-blocked.cdxml")})
            )?)
            .err()
            .ok_or("published lease escaped")?
            .info()
            .code,
        "busy"
    );
    let finished = serde_json::to_value(engine.finish(completed)?)?;
    assert_eq!(finished["conversion"]["input_revision"], 0);
    assert_eq!(finished["conversion"]["session_generation"], 0);
    assert_eq!(without_receipt(finished)?, fresh);
    assert_eq!(engine.content(), &content);
    assert_eq!(serde_json::to_value(engine.state())?, fresh);
    consume(
        &published,
        "CO",
        json!([[6, 3], [8, 1]]),
        json!([[1, 2, 1]]),
        "CH4O",
    )?;
    engine.set_ui_file_host(true);
    let request = engine
        .dispatch("ui.file", json!({"expected_revision":1,"action":"save_as"}))?
        .ui_file_operation
        .ok_or("request")?;
    engine.claim_ui_file(request)?;
    let qualified = serde_json::to_value(engine.state())?;
    let rejected = directory.path("not-native-save.cdxml");
    assert_eq!(
        engine
            .prepare_ui_file(request, Command::parse(NAME, params(1, &input, &rejected))?)
            .err()
            .ok_or("conversion bound to Save")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(serde_json::to_value(engine.state())?, qualified);
    assert!(!rejected.try_exists()?);
    assert!(
        !engine.finish_ui_file(request, false),
        "conversion cannot acknowledge a Save"
    );
    drop(engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":directory.path("lease-released.cdxml")}),
    )?)?);
    Ok(())
}
