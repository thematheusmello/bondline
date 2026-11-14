use bondline_commands::{Cancellation, Command, Engine, registry};
use serde_json::json;
use std::fs;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the isolated test directory")]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = b"CC first\r\n\n \xffignored\n[13CH3][OH] \0\xffPRIVATE\nO last";

#[test]
fn schema_requires_only_revision_and_positive_ordinal_and_is_read_only() -> TestResult {
    let row = registry()
        .into_iter()
        .find(|row| row.name == "smiles.record.inspect")
        .ok_or("query schema")?;
    assert!(row.read_only);
    let schema = row.input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["required"],
        json!(["expected_revision", "record_id"])
    );
    assert_eq!(schema["properties"]["record_id"]["minimum"], 1);
    for params in [
        json!({}),
        json!({"record_id":1}),
        json!({"expected_revision":1}),
        json!({"expected_revision":1,"record_id":-1}),
        json!({"expected_revision":1,"record_id":1.5}),
        json!({"expected_revision":1,"record_id":4294967296_u64}),
        json!({"expected_revision":1,"record_id":1,"include_data":true}),
        json!({"expected_revision":1,"record_id":1,"field_limit":1}),
    ] {
        assert_eq!(
            Command::parse("smiles.record.inspect", params)
                .err()
                .ok_or("closed params")?
                .info()
                .code,
            "invalid_params"
        );
    }
    Ok(())
}

#[test]
fn explicit_nonfirst_record_query_preserves_spelling_ranges_and_the_whole_session() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("records-λ.smi");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    let result = engine.dispatch(
        "smiles.record.inspect",
        json!({"expected_revision":1,"record_id":2}),
    )?;
    assert!(!format!("{result:?}").contains("[13CH3][OH]"));
    let mut wire = serde_json::to_value(result)?;
    let inspection = wire
        .as_object_mut()
        .ok_or("result object")?
        .remove("smiles_record")
        .ok_or("explicit record")?;
    assert_eq!(
        inspection,
        json!({
            "input_revision":1,"session_generation":1,"record_id":2,"line_number":4,
            "full_range":{"start":21,"end":43},
            "notation_range":{"start":21,"end":32},
            "separator_range":{"start":32,"end":33},
            "application_data_range":{"start":33,"end":42},
            "line_ending_range":{"start":42,"end":43},
            "atom_count":2,"bond_count":1,"notation":"[13CH3][OH]"
        })
    );
    assert_eq!(wire, before);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(!serde_json::to_string(&inspection)?.contains("PRIVATE"));
    assert_eq!(fs::read(&input)?, SOURCE);
    Ok(())
}

#[test]
fn invalid_and_stale_ordinals_empty_files_and_wrong_kinds_never_fall_back() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("input.smi");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    assert_eq!(
        engine
            .dispatch(
                "smiles.record.inspect",
                json!({"expected_revision":0,"record_id":1})
            )
            .err()
            .ok_or("drawing kind")?
            .info()
            .code,
        "unsupported_content"
    );
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    for (params, code) in [
        (
            json!({"expected_revision":0,"record_id":2}),
            "stale_revision",
        ),
        (
            json!({"expected_revision":1,"record_id":0}),
            "invalid_params",
        ),
        (json!({"expected_revision":1,"record_id":4}), "not_found"),
        (
            json!({"expected_revision":1,"record_id":4294967295_u32}),
            "not_found",
        ),
    ] {
        assert_eq!(
            engine
                .dispatch("smiles.record.inspect", params)
                .err()
                .ok_or("invalid query")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    fs::write(&input, b"\n \xffignored\r\n")?;
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    assert_eq!(
        engine
            .dispatch(
                "smiles.record.inspect",
                json!({"expected_revision":2,"record_id":1})
            )
            .err()
            .ok_or("empty file has no first molecule")?
            .info()
            .code,
        "not_found"
    );
    let sdf = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/sdf/ordered-fields-v2000.sdf"
    );
    engine.dispatch("document.open_sdf", json!({"path":sdf}))?;
    assert_eq!(
        engine
            .dispatch(
                "smiles.record.inspect",
                json!({"expected_revision":3,"record_id":1})
            )
            .err()
            .ok_or("SDF kind")?
            .info()
            .code,
        "unsupported_content"
    );
    Ok(())
}

#[test]
fn deferred_query_returns_the_captured_record_after_new_open_without_applying_old_state()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("first.smi");
    let next = directory.path("next.smi");
    fs::write(&input, SOURCE)?;
    fs::write(&next, b"O another\n")?;
    let canonical_input = fs::canonicalize(&input)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let query = engine.prepare(Command::parse(
        "smiles.record.inspect",
        json!({"expected_revision":1,"record_id":2}),
    )?)?;
    assert!(query.is_deferred());
    fs::remove_file(&input)?;
    engine.dispatch("document.open_smiles", json!({"path":next}))?;
    let before = serde_json::to_value(engine.state())?;
    let reply = serde_json::to_value(engine.finish(query.complete()?)?)?;
    assert_eq!(reply["revision"], 1);
    assert_eq!(reply["smiles_record"]["input_revision"], 1);
    assert_eq!(reply["smiles_record"]["session_generation"], 1);
    assert_eq!(reply["smiles_record"]["notation"], "[13CH3][OH]");
    assert_eq!(reply["smiles_file"]["record_count"], 3);
    assert_eq!(reply["source"]["byte_length"], SOURCE.len());
    assert_eq!(reply["close_epoch"], before["close_epoch"]);
    assert_eq!(reply["file"]["path"], json!(canonical_input));
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(fs::read(&next)?, b"O another\n");
    Ok(())
}

#[test]
fn cancellation_before_work_and_after_completion_cannot_publish_a_query_reply() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.smi");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    for late in [false, true] {
        let query = engine.prepare(Command::parse(
            "smiles.record.inspect",
            json!({"expected_revision":1,"record_id":2}),
        )?)?;
        let cancel = Cancellation::default();
        if late {
            let done = query.complete_with_cancel(&cancel)?;
            cancel.cancel();
            assert_eq!(
                engine.finish(done).err().ok_or("late cancel")?.info().code,
                "cancelled"
            );
        } else {
            cancel.cancel();
            assert_eq!(
                query
                    .complete_with_cancel(&cancel)
                    .err()
                    .ok_or("early cancel")?
                    .info()
                    .code,
                "cancelled"
            );
        }
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    Ok(())
}

#[test]
fn query_does_not_reserve_writes_acknowledge_close_or_consume_native_file_intent() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("source.smi");
    let output = directory.path("saved.smi");
    fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let query = engine.prepare(Command::parse(
        "smiles.record.inspect",
        json!({"expected_revision":1,"record_id":2}),
    )?)?;
    let saving = engine.prepare(Command::parse(
        "document.save_smiles",
        json!({"path":output}),
    )?)?;
    engine.dispatch("document.close", json!({"expected_revision":1}))?;
    let before = serde_json::to_value(engine.state())?;
    let reply = serde_json::to_value(engine.finish(query.complete()?)?)?;
    assert_eq!(reply["close"], "approved");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(
        engine
            .prepare(Command::parse(
                "document.save_smiles",
                json!({"path":output})
            )?)
            .err()
            .ok_or("save still holds its lease")?
            .info()
            .code,
        "busy"
    );
    drop(saving);
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"cancel"}),
    )?;
    engine.dispatch("document.save_smiles", json!({"path":output}))?;
    assert_eq!(fs::read(&output)?, SOURCE);
    engine.set_ui_file_host(true);
    let offered = engine.dispatch("ui.file", json!({"expected_revision":1,"action":"save_as"}))?;
    let id = offered.ui_file.request.ok_or("native request")?.id;
    engine.claim_ui_file(id)?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .prepare_ui_file(
                id,
                Command::parse(
                    "smiles.record.inspect",
                    json!({"expected_revision":1,"record_id":2})
                )?
            )
            .err()
            .ok_or("query cannot bind a native save")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}
