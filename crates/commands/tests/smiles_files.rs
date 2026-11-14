use bondline_commands::{Command, Engine, registry};
use serde_json::json;
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated test directory")]
mod support;

const SOURCE: &[u8] =
    b"CC ethane\r\n\n \tignored PRIVATE_IGNORED\nc1ccccc1\taromatic\xff\n[13CH3][OH]\tPRIVATE_DATA";

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn file_commands_have_closed_parameters_and_do_not_offer_graph_or_layout_conversion() -> TestResult
{
    for (name, option) in [
        ("document.open_smiles", "discard_unsaved"),
        ("document.save_smiles", "overwrite"),
    ] {
        assert!(Command::parse(name, json!({"path":"molecules.smi"})).is_ok());
        assert!(Command::parse(name, json!({"path":"molecules.smi",option:true})).is_ok());
        for params in [
            json!({}),
            json!({"path":3}),
            json!({"path":"x",option:"yes"}),
            json!({"path":"x","record_id":1}),
            json!({"path":"x","points_per_unit":24}),
            json!({"path":"x","unexpected":true}),
        ] {
            assert_eq!(
                Command::parse(name, params)
                    .err()
                    .ok_or("closed params")?
                    .info()
                    .code,
                "invalid_params"
            );
        }
        let row = registry()
            .into_iter()
            .find(|row| row.name == name)
            .ok_or("schema")?;
        assert!(!row.read_only);
        assert_eq!(row.input_schema.as_value()["additionalProperties"], false);
        assert_eq!(row.input_schema.as_value()["required"], json!(["path"]));
    }
    Ok(())
}

#[test]
fn open_and_save_keep_every_smiles_record_and_opaque_byte_without_a_dummy_drawing()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("original-\u{03bb}.smi");
    let output = directory.path("copy-\u{5316}\u{5b66}.data");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    let result =
        serde_json::to_value(engine.dispatch("document.open_smiles", json!({"path":input}))?)?;
    assert_eq!(result["document"], serde_json::Value::Null);
    assert!(result.get("collection").is_none());
    assert_eq!(
        result["smiles_file"],
        json!({
            "format":"smiles","byte_length":SOURCE.len(),"record_count":3,
        })
    );
    assert_eq!(
        result["source"],
        json!({"format":"smiles","byte_length":SOURCE.len(),"modified":false})
    );
    assert_eq!(result["revision"], 1);
    assert_eq!(result["file"]["format"], "smiles");
    assert_eq!(result["file"]["dirty"], false);
    for secret in ["PRIVATE_DATA", "PRIVATE_IGNORED", "c1ccccc1"] {
        assert!(!format!("{:?}", engine.content()).contains(secret));
        assert!(!serde_json::to_string(&result)?.contains(secret));
    }
    let saved =
        serde_json::to_value(engine.dispatch("document.save_smiles", json!({"path":output}))?)?;
    assert_eq!(saved["revision"], 1);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["smiles_file"], result["smiles_file"]);
    assert_eq!(saved["source"], result["source"]);
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::read(&output)?, SOURCE);
    let mut reopened = Engine::default();
    let result =
        serde_json::to_value(reopened.dispatch("document.open_smiles", json!({"path":output}))?)?;
    assert_eq!(result["document"], serde_json::Value::Null);
    assert_eq!(result["smiles_file"], saved["smiles_file"]);
    assert_eq!(reopened.content(), engine.content());
    Ok(())
}

#[test]
fn any_later_record_failure_keeps_the_complete_prior_drawing_or_smiles_file() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("valid.smi");
    let bad = directory.path("bad.smi");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    for file in [false, true] {
        if file {
            engine.dispatch("document.open_smiles", json!({"path":input}))?;
        }
        let before = engine.content().clone();
        let state = serde_json::to_value(engine.state())?;
        for source in [
            b"CC valid\nC1 PRIVATE\n".as_slice(),
            b"CC valid\n[C@H] PRIVATE\n",
            b"CC valid\n\xff PRIVATE\n",
        ] {
            fs::write(&bad, source)?;
            assert!(
                engine
                    .dispatch(
                        "document.open_smiles",
                        json!({"path":bad,"discard_unsaved":true})
                    )
                    .is_err()
            );
            assert_eq!(engine.content(), &before);
            assert_eq!(serde_json::to_value(engine.state())?, state);
        }
    }
    Ok(())
}

#[test]
fn zero_record_files_preserve_ignored_content_instead_of_creating_a_molecule() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for (index, source) in [b"".as_slice(), b"\n\r\n \tPRIVATE\xff\0\n\rprivate\n"]
        .into_iter()
        .enumerate()
    {
        let input = directory.path(&format!("empty-{index}.smi"));
        let output = directory.path(&format!("copy-{index}.smi"));
        fs::write(&input, source)?;
        let mut engine = Engine::default();
        let opened =
            serde_json::to_value(engine.dispatch("document.open_smiles", json!({"path":input}))?)?;
        assert_eq!(opened["document"], serde_json::Value::Null);
        assert_eq!(opened["smiles_file"]["record_count"], 0);
        engine.dispatch("document.save_smiles", json!({"path":output}))?;
        assert_eq!(fs::read(&output)?, source);
    }
    Ok(())
}

#[test]
fn drawing_and_other_format_operations_cannot_substitute_a_first_record() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.smi");
    let output = directory.path("must-not-exist.data");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    assert_eq!(
        engine
            .dispatch("document.save_smiles", json!({"path":output}))
            .err()
            .ok_or("drawing is not SMILES")?
            .info()
            .code,
        "unsupported_content"
    );
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    for (name, params) in [
        (
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        ),
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":96,"end_y_pt":72}),
        ),
        (
            "render.capture",
            json!({"page_id":1,"width_px":32,"height_px":32,"backend":"cpu"}),
        ),
        (
            "file.export_smiles_fragment",
            json!({"expected_revision":1,"page_id":1,"fragment_id":1,"output_path":output}),
        ),
        (
            "sdf.record.inspect",
            json!({"expected_revision":1,"record_id":1,"field_offset":0,"field_limit":1}),
        ),
        ("document.save_cdxml", json!({"path":output})),
        ("document.save_cdx", json!({"path":output})),
        ("document.save_mol", json!({"path":output})),
        ("document.save_sdf", json!({"path":output})),
    ] {
        let error = engine
            .dispatch(name, params)
            .err()
            .ok_or("unsupported whole SMILES content")?;
        assert_eq!(error.info().code, "unsupported_content", "{name}: {error}");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(!output.exists());
    }
    Ok(())
}

#[test]
fn dirty_guard_stale_open_and_committed_close_preserve_the_whole_session() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.smi");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    let plan = engine.prepare(Command::parse(
        "document.open_smiles",
        json!({"path":input}),
    )?)?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch("document.open_smiles", json!({"path":input}))
            .err()
            .ok_or("dirty")?
            .info()
            .code,
        "unsaved_changes"
    );
    assert_eq!(
        engine
            .finish(plan.complete()?)
            .err()
            .ok_or("stale")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let plan = engine.prepare(Command::parse(
        "document.open_smiles",
        json!({"path":input,"discard_unsaved":true}),
    )?)?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"discard"}),
    )?;
    assert!(engine.complete_close());
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .finish(plan.complete()?)
            .err()
            .ok_or("closed")?
            .info()
            .code,
        "close_committed"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn completed_save_holds_the_shared_write_lease_and_does_not_associate_a_new_generation()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.smi");
    let other = directory.path("new-generation.smi");
    let output = directory.path("captured.smi");
    fs::write(&input, SOURCE)?;
    fs::write(&other, b"CO other\n")?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let plan = engine.prepare(Command::parse(
        "document.save_smiles",
        json!({"path":output}),
    )?)?;
    let done = plan.complete()?;
    assert_eq!(fs::read(&output)?, SOURCE);
    engine.dispatch("document.open_smiles", json!({"path":other}))?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .prepare(Command::parse(
                "document.save_smiles",
                json!({"path":output,"overwrite":true})
            )?)
            .err()
            .ok_or("shared lease")?
            .info()
            .code,
        "busy"
    );
    let result = serde_json::to_value(engine.finish(done)?)?;
    assert_eq!(result["file"], before["file"]);
    assert_eq!(result["source"], before["source"]);
    assert_eq!(result["smiles_file"], before["smiles_file"]);
    assert_eq!(result["file_operation"]["input_revision"], 1);
    assert_eq!(result["file_operation"]["byte_length"], SOURCE.len());
    let retry = engine.prepare(Command::parse(
        "document.save_smiles",
        json!({"path":output,"overwrite":true}),
    )?)?;
    drop(retry);
    Ok(())
}
