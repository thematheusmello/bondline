use bondline_commands::Engine;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn native_conversion_intent_and_control_preserve_the_current_session() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let controls = engine.controls();
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let result = engine.dispatch(
        "ui.file",
        json!({"expected_revision":0,"action":"convert_smiles"}),
    )?;
    assert!(!result.changed);
    assert_eq!(result.revision, 0);
    assert_eq!((result.undo_depth, result.redo_depth), (0, 0));
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(result.file_operation.is_none() && result.conversion.is_none());
    let request = result.ui_file.request.ok_or("conversion intent")?;
    assert_eq!(serde_json::to_value(request)?["action"], "convert_smiles");
    let control = controls
        .iter()
        .find(|control| control.id == "file.convert_smiles")
        .ok_or("central conversion control")?;
    assert_eq!(control.menu, "File");
    assert_eq!(control.label, "Convert SMILES file…");
    assert!(control.enabled && control.shortcut.is_none());
    assert_eq!(
        serde_json::to_value(&control.command)?,
        json!({"command":"ui.file","params":{"expected_revision":0,"action":"convert_smiles"}})
    );
    assert_eq!(controls.len(), 31);
    Ok(())
}

use bondline_commands::{Cancellation, Command, ConvertSmilesToCdxml, UiFilePlan};
use serde_json::Value;
use std::{fs, path::Path};

#[path = "../../cli/tests/support/conversion.rs"]
#[allow(
    dead_code,
    reason = "reuse the unchanged actual-output chemistry consumer"
)]
mod conversion_support;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse the existing bounded temporary-directory owner"
)]
mod support;

const SDF: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

fn conversion(revision: u64, input: &Path, output: &Path) -> TestResult<ConvertSmilesToCdxml> {
    Ok(serde_json::from_value(conversion_support::params(
        revision, input, output,
    ))?)
}

fn state(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.state())?)
}

fn request(engine: &mut Engine, action: &str, claim: bool) -> TestResult<u64> {
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":engine.state().revision,"action":action}),
    )?;
    let id = engine.state().ui_file.request.ok_or("native intent")?.id;
    if claim {
        engine.claim_ui_file(id)?;
    }
    Ok(id)
}

fn consumed(mut accepted: Value) -> Value {
    accepted["ui_file"]["request"] = Value::Null;
    accepted
}

fn refused(engine: &mut Engine, id: u64, params: ConvertSmilesToCdxml, code: &str) -> TestResult {
    let before = state(engine)?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    assert_eq!(
        engine
            .prepare_ui_smiles_conversion(id, params)
            .err()
            .ok_or("native conversion unexpectedly accepted")?
            .info()
            .code,
        code
    );
    assert_eq!(state(engine)?, before);
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    Ok(())
}

fn save_plan(
    engine: &mut Engine,
    path: &Path,
) -> Result<bondline_commands::Prepared, bondline_commands::Error> {
    let name = if engine.content().collection().is_some() {
        "document.save_sdf"
    } else {
        "document.save_cdxml"
    };
    engine.prepare(Command::parse(name, json!({"path":path}))?)
}

#[test]
fn accepted_conversion_publishes_real_cdxml_without_saving_the_open_dirty_sdf() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let source = directory.path("current-化学.sdf");
    let input = directory.path("input-λ.smi");
    let output = directory.path("converted-化学.misleading");
    fs::write(&source, SDF)?;
    fs::write(&input, b"CC\n")?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_sdf", json!({"path":source}))?;
    engine.dispatch(
        "sdf.field.set_value",
        json!({"expected_revision":1,"record_id":1,"field_index":2,"value_lines":["reviewed"]}),
    )?;
    let original = engine.content().clone();
    let stamp = engine.capture_stamp();
    let revision = engine.state().revision;
    let generation = engine.session_generation();
    let id = request(&mut engine, "convert_smiles", false)?;
    assert!(matches!(
        engine.claim_ui_file(id)?,
        UiFilePlan::SmilesConversion
    ));
    let accepted = state(&engine)?;
    assert_eq!(accepted["ui_file"]["request"]["phase"], "accepted");
    let epoch = accepted["ui_file"]["epoch"].clone();
    let prepared =
        engine.prepare_ui_smiles_conversion(id, conversion(revision, &input, &output)?)?;
    assert!(prepared.is_deferred());
    let unchanged = consumed(accepted);
    assert_eq!(state(&engine)?, unchanged);
    assert_eq!(engine.state().ui_file.epoch, epoch.as_u64().ok_or("epoch")?);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(
        !output.try_exists()?,
        "preparation performed filesystem work"
    );
    let completed = prepared.complete()?;
    conversion_support::independent(&output, 2)?;
    assert_eq!(fs::read(&input)?, b"CC\n");
    assert_eq!(fs::read(&source)?, SDF);
    assert_eq!(
        save_plan(&mut engine, &directory.path("busy.sdf"))
            .err()
            .ok_or("published conversion released the Save reservation early")?
            .info()
            .code,
        "busy"
    );
    assert!(!directory.path("busy.sdf").try_exists()?);
    let result = engine.finish(completed)?;
    let receipt = result
        .conversion
        .as_ref()
        .ok_or("real conversion receipt")?;
    assert_eq!(receipt.profile, "neutral_carbon_path_v1");
    assert_eq!(receipt.input_revision, revision);
    assert_eq!(receipt.session_generation, generation);
    assert_eq!((receipt.atom_count, receipt.bond_count), (2, 1));
    assert_eq!(receipt.input_byte_length, 3);
    assert_eq!(
        receipt.input_path,
        fs::canonicalize(&input)?.to_string_lossy()
    );
    assert_eq!(
        receipt.output.path,
        fs::canonicalize(&output)?.to_string_lossy()
    );
    assert_eq!(receipt.output.byte_length, fs::read(&output)?.len());
    assert!(!result.changed && result.file_operation.is_none());
    let mut wire = serde_json::to_value(result)?;
    wire.as_object_mut().ok_or("result")?.remove("conversion");
    assert_eq!(wire, unchanged);
    assert_eq!(state(&engine)?, unchanged);
    assert_eq!(engine.content(), &original);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(engine.state().file.dirty);
    assert_eq!(
        (engine.state().undo_depth, engine.state().redo_depth),
        (1, 0)
    );
    assert!(!engine.finish_ui_file(id, false));
    assert_eq!(state(&engine)?, unchanged);
    drop(save_plan(&mut engine, &directory.path("released.sdf"))?);
    assert_eq!(bondline_commands::registry().len(), 41);
    Ok(())
}

#[test]
fn failed_preparation_retains_the_exact_accepted_intent_and_old_generic_binder_refuses()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("not-read.smi");
    let output = directory.path("not-created.cdxml");
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    refused(
        &mut engine,
        1,
        conversion(0, &input, &output)?,
        "ui_stale_request",
    )?;
    let id = request(&mut engine, "convert_smiles", false)?;
    refused(&mut engine, id, conversion(0, &input, &output)?, "ui_busy")?;
    engine.claim_ui_file(id)?;
    refused(
        &mut engine,
        0,
        conversion(0, &input, &output)?,
        "ui_stale_request",
    )?;
    refused(
        &mut engine,
        id + 1,
        conversion(0, &input, &output)?,
        "ui_stale_request",
    )?;
    refused(
        &mut engine,
        id,
        conversion(1, &input, &output)?,
        "stale_revision",
    )?;
    let mut bad = conversion(0, &input, &output)?;
    bad.bond_length_pt = 0.0;
    refused(&mut engine, id, bad, "invalid_params")?;
    let before = state(&engine)?;
    let generic = engine.prepare_ui_file(
        id,
        Command::ConvertSmilesToCdxml(conversion(0, &input, &output)?),
    );
    assert_eq!(
        generic
            .err()
            .ok_or("generic file binder accepted conversion")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(state(&engine)?, before);
    let save = save_plan(&mut engine, &directory.path("held.cdxml"))?;
    refused(&mut engine, id, conversion(0, &input, &output)?, "busy")?;
    drop(save);
    let prepared = engine.prepare_ui_smiles_conversion(id, conversion(0, &input, &output)?)?;
    assert_eq!(state(&engine)?, consumed(before));
    drop(prepared);
    assert!(!input.try_exists()? && !output.try_exists()?);
    assert!(!engine.finish_ui_file(id, false));
    drop(save_plan(&mut engine, &directory.path("lease-free.cdxml"))?);
    let wrong = request(&mut engine, "open", true)?;
    refused(
        &mut engine,
        wrong,
        conversion(0, &input, &output)?,
        "invalid_params",
    )?;
    Ok(())
}

#[test]
fn old_conversion_intents_cannot_survive_revision_source_close_or_host_aba() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("untouched.smi");
    let output = directory.path("untouched.cdxml");
    for transition in 0..6 {
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        let id = request(&mut engine, "convert_smiles", true)?;
        match transition {
            0 => {
                engine.dispatch(
                    "page.set_size",
                    json!({"page_id":1,"width_pt":612,"height_pt":792}),
                )?;
                engine.dispatch("history.undo", json!({}))?;
            }
            1 => {
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":0,"action":"request"}),
                )?;
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":0,"action":"cancel"}),
                )?;
            }
            2 => {
                engine.dispatch("document.new", json!({"expected_revision":0}))?;
            }
            3 => {
                engine.set_ui_file_host(false);
                engine.set_ui_file_host(true);
            }
            4 => {
                engine.dispatch(
                    "ui.file",
                    json!({"expected_revision":0,"action":"cancel","request_id":id}),
                )?;
            }
            _ => {
                engine.dispatch(
                    "document.save_cdxml",
                    json!({"path":directory.path("associated.cdxml")}),
                )?;
            }
        }
        let revision = engine.state().revision;
        refused(
            &mut engine,
            id,
            conversion(revision, &input, &output)?,
            "ui_stale_request",
        )?;
        assert!(!input.try_exists()? && !output.try_exists()?);
    }
    Ok(())
}

#[test]
fn accepted_conversion_can_publish_historically_without_acknowledging_or_cleaning_new() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("historical.smi");
    fs::write(&input, b"CC\n")?;
    for drop_result in [false, true] {
        let output = directory.path(if drop_result {
            "dropped.cdxml"
        } else {
            "finished.cdxml"
        });
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        let id = request(&mut engine, "convert_smiles", true)?;
        let prepared = engine.prepare_ui_smiles_conversion(id, conversion(0, &input, &output)?)?;
        engine.dispatch("document.new", json!({"expected_revision":0}))?;
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}),
        )?;
        let before = state(&engine)?;
        let content = engine.content().clone();
        let cancel = Cancellation::default();
        let completed = prepared.complete_with_cancel(&cancel)?;
        cancel.cancel();
        conversion_support::independent(&output, 2)?;
        assert_eq!(
            save_plan(&mut engine, &directory.path("still-busy.cdxml"))
                .err()
                .ok_or("completed conversion released reservation")?
                .info()
                .code,
            "busy"
        );
        if drop_result {
            drop(completed);
        } else {
            let mut result = serde_json::to_value(engine.finish(completed)?)?;
            let receipt = result
                .as_object_mut()
                .ok_or("result")?
                .remove("conversion")
                .ok_or("receipt")?;
            assert_eq!(receipt["input_revision"], 0);
            assert_eq!(receipt["session_generation"], 0);
            assert_eq!(result, before);
        }
        assert_eq!(state(&engine)?, before);
        assert_eq!(engine.content(), &content);
        assert!(!engine.finish_ui_file(id, false));
        assert_eq!(state(&engine)?, before);
        assert!(engine.state().file.path.is_none() && engine.state().file.dirty);
        assert_eq!(
            (engine.state().undo_depth, engine.state().redo_depth),
            (1, 0)
        );
        drop(save_plan(
            &mut engine,
            &directory.path("lease-released.cdxml"),
        )?);
    }
    Ok(())
}

#[test]
fn failed_or_cancelled_native_conversion_never_saves_or_replaces_the_active_session() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("conversion.smi");
    for scenario in 0..3 {
        let output = directory.path(&format!("refused-{scenario}.cdxml"));
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        )?;
        fs::write(
            &input,
            if scenario == 0 {
                b"CC=O\n".as_slice()
            } else {
                b"CC\n".as_slice()
            },
        )?;
        if scenario == 2 {
            fs::write(&output, b"preserve existing unrelated bytes")?;
        }
        let id = request(&mut engine, "convert_smiles", true)?;
        let prepared = engine.prepare_ui_smiles_conversion(id, conversion(1, &input, &output)?)?;
        let before = state(&engine)?;
        let content = engine.content().clone();
        let cancel = Cancellation::default();
        if scenario == 1 {
            cancel.cancel();
        }
        let code = prepared
            .complete_with_cancel(&cancel)
            .err()
            .ok_or("refused conversion succeeded")?
            .info()
            .code;
        assert_eq!(
            code,
            match scenario {
                0 => "unsupported_content",
                1 => "cancelled",
                _ => "already_exists",
            }
        );
        assert_eq!(state(&engine)?, before);
        assert_eq!(engine.content(), &content);
        assert!(!engine.finish_ui_file(id, false));
        assert_eq!(state(&engine)?, before);
        if scenario == 2 {
            assert_eq!(fs::read(&output)?, b"preserve existing unrelated bytes");
        } else {
            assert!(!output.try_exists()?);
        }
        drop(save_plan(
            &mut engine,
            &directory.path("free-after-error.cdxml"),
        )?);
        assert_eq!(engine.state().undo_depth, 1);
        assert!(engine.state().file.dirty && engine.state().file.path.is_none());
    }
    Ok(())
}
