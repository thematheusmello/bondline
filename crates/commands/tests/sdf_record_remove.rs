//! Independent literal record removal through the existing shared command interface.
use bondline_commands::Engine;
use serde_json::json;
use std::fs;

#[path = "../../../xtask/oracles/sdf_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only existing isolated directory and external consumers"
)]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const MIXED: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

#[test]
fn explicit_second_record_removal_keeps_the_complete_literal_first_record() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("remove-second.sdf");
    fs::write(&input, MIXED)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let result = engine.dispatch(
        "sdf.record.remove",
        json!({"expected_revision":1,"record_id":2}),
    )?;
    assert!(result.changed);
    assert_eq!(result.revision, 2);
    assert_eq!(
        engine.content().sdf()?.as_bytes(),
        MIXED.get(..462).ok_or("literal record1 span")?
    );
    assert_eq!(fs::read(input)?, MIXED);
    Ok(())
}

#[path = "../../../xtask/oracles/sdf_record_test_support.rs"]
mod fields;

const ORDERED: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");

fn remove(revision: u64, record_id: u32) -> serde_json::Value {
    json!({"expected_revision":revision,"record_id":record_id})
}

fn opened(path: &std::path::Path, bytes: &[u8]) -> TestResult<Engine> {
    fs::write(path, bytes)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    Ok(engine)
}

fn assert_snapshot(engine: &Engine, original_bytes: &[u8], current_bytes: &[u8]) -> TestResult {
    let bondline_commands::StoredContent::Sdf { original, current } = engine.content() else {
        return Err("whole SDF content required".into());
    };
    assert_eq!(original.as_bytes(), original_bytes);
    assert_eq!(current.as_bytes(), current_bytes);
    assert!(engine.document().is_err());
    assert!(engine.stored_document().is_err());
    let state = serde_json::to_value(engine.state())?;
    assert!(state["document"].is_null());
    assert!(state.get("sdf_record").is_none());
    assert_eq!(state["source"]["byte_length"], original_bytes.len());
    assert_eq!(state["source"]["modified"], original_bytes != current_bytes);
    assert_eq!(state["file"]["format"], "sdf");
    Ok(())
}

fn retained(
    mixed: bool,
    removed: u32,
) -> TestResult<(&'static [u8], &'static str, usize, usize, u32)> {
    let result = match (mixed, removed) {
        (true, 1) => (MIXED.get(462..), "v2000", 282, 1, 2),
        (true, 2) => (MIXED.get(..462), "v3000", 344, 6, 1),
        (false, 1) => (ORDERED.get(424..), "v2000", 261, 0, 2),
        (false, 2) => (ORDERED.get(..424), "v2000", 282, 7, 1),
        _ => return Err("literal removal scenario".into()),
    };
    Ok((
        result.0.ok_or("literal retained span")?,
        result.1,
        result.2,
        result.3,
        result.4,
    ))
}

fn consume_retained(
    bytes: &[u8],
    version: &str,
    mol_len: usize,
    expected_fields: &serde_json::Value,
) -> TestResult {
    let mol = bytes.get(..mol_len).ok_or("literal MOL span")?;
    if version == "v2000" {
        let raw = support::describe_source("sdf_oracle.py", bytes)?;
        let record = &raw["records"][0];
        assert_eq!(raw["records"].as_array().ok_or("raw records")?.len(), 1);
        assert_eq!(record["range"], json!([0, bytes.len()]));
        assert_eq!(record["mol_range"], json!([0, mol_len]));
        let actual = record["fields"].as_array().ok_or("ordered raw fields")?;
        let expected = expected_fields.as_array().ok_or("literal fields")?;
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected) {
            for key in ["header", "name", "dt_number", "value_lines"] {
                assert_eq!(actual[key], expected[key], "retained {key}");
            }
        }
        assert_eq!(record["rdkit"]["smiles"], "CC");
        let molecular = support::describe_source("mol_oracle.py", mol)?;
        assert_eq!(molecular["smiles"], "CC");
        assert_eq!(molecular["formula"], "C2H6");
    } else {
        let molecular = support::describe_source("v3000_oracle.py", mol)?;
        assert_eq!(molecular["source_bytes"], 344);
        assert_eq!(molecular["atoms"].as_array().ok_or("V3000 atoms")?.len(), 2);
        for atom in molecular["atoms"].as_array().ok_or("V3000 atoms")? {
            assert_eq!(atom["element"], 6);
            assert_eq!(atom["charge"], 0);
            assert_eq!(atom["isotope"], 0);
        }
        assert_eq!(molecular["bonds"].as_array().ok_or("V3000 bonds")?.len(), 1);
        assert_eq!(molecular["bonds"][0]["begin"], 0);
        assert_eq!(molecular["bonds"][0]["end"], 1);
        assert_eq!(molecular["bonds"][0]["order"], 1.0);
    }
    Ok(())
}

#[test]
fn removal_schema_is_closed_mutating_and_adds_only_the_twenty_ninth_command() -> TestResult {
    let names = bondline_commands::registry()
        .into_iter()
        .map(|info| info.name)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        names,
        [
            "atom.set_label",
            "chain.create",
            "chain.create_path",
            "document.inspect",
            "document.import_smiles_records",
            "document.new",
            "document.close",
            "ui.file",
            "ui.controls",
            "ui.drawing_settings",
            "ui.atom_label",
            "ui.selection",
            "selection.translate",
            "ui.bond_tool",
            "ui.ring_tool",
            "ui.view",
            "document.open_cdxml",
            "document.save_cdxml",
            "document.open_cdx",
            "document.save_cdx",
            "document.open_mol",
            "document.save_mol",
            "document.open_sdf",
            "document.save_sdf",
            "document.open_smiles",
            "document.save_smiles",
            "document.set_drawing_settings",
            "file.convert_smiles_to_cdxml",
            "file.export_smiles_fragment",
            "sdf.record.inspect",
            "smiles.record.inspect",
            "structure.paste_smiles",
            "sdf.field.set_value",
            "sdf.record.remove",
            "page.set_size",
            "bond.create",
            "bond.extend",
            "bond.set_order",
            "history.undo",
            "history.redo",
            "render.capture",
            "ring.create",
            "ui.chain_tool",
            "ring.sprout",
            "ui.preferences",
            "document.set_document_settings",
        ]
        .into_iter()
        .collect()
    );
    let info = bondline_commands::registry()
        .into_iter()
        .find(|info| info.name == "sdf.record.remove")
        .ok_or("remove schema")?;
    assert!(!info.read_only);
    let schema = info.input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["required"],
        json!(["expected_revision", "record_id"])
    );
    assert_eq!(
        schema["properties"].as_object().ok_or("properties")?.len(),
        2
    );
    assert_eq!(schema["properties"]["record_id"]["minimum"], 1);
    assert!(bondline_commands::Command::parse("sdf.record.remove", remove(1, 1)).is_ok());
    for params in [
        json!({}),
        json!({"expected_revision":1}),
        json!({"record_id":1}),
        json!({"expected_revision":1,"record_id":-1}),
        json!({"expected_revision":1,"record_id":"1"}),
        json!({"expected_revision":1,"record_id":null}),
        json!({"expected_revision":1,"record_id":1,"name":"DUP"}),
    ] {
        assert_eq!(
            bondline_commands::Command::parse("sdf.record.remove", params)
                .err()
                .ok_or("closed removal params")?
                .info()
                .code,
            "invalid_params"
        );
    }
    Ok(())
}

#[test]
fn either_literal_record_can_be_saved_reopened_and_restored_without_data_reconstruction()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    for mixed in [false, true] {
        let source = if mixed { MIXED } else { ORDERED };
        for removed in [1, 2] {
            let input = directory.path(&format!("input-{mixed}-{removed}.sdf"));
            let output = directory.path(&format!("retained-{mixed}-{removed}-λ.data"));
            let mut engine = opened(&input, source)?;
            let original = engine.content().clone();
            let (expected, version, mol_len, field_count, kept_id) = retained(mixed, removed)?;
            let result = engine.dispatch("sdf.record.remove", remove(1, removed))?;
            assert_eq!(
                (
                    result.changed,
                    result.revision,
                    result.undo_depth,
                    result.redo_depth
                ),
                (true, 2, 1, 0)
            );
            assert!(result.file.dirty);
            assert!(result.file_operation.is_none());
            assert_eq!(
                serde_json::to_value(&result)?["collection"],
                json!({
                    "format":"sdf", "byte_length":expected.len(), "record_count":1,
                    "records":[{"id":1,"mol_version":version,"byte_length":expected.len(),
                        "mol_byte_length":mol_len,"field_count":field_count}]
                })
            );
            assert_snapshot(&engine, source, expected)?;
            let record = engine
                .content()
                .sdf()?
                .records()
                .first()
                .ok_or("retained record")?;
            assert_eq!(record.id().get(), 1);
            assert_eq!(record.range(), 0..expected.len());
            assert_eq!(record.mol_range(), 0..mol_len);
            let page = serde_json::to_value(engine.dispatch("sdf.record.inspect", remove(2, 1))?)?;
            fields::assert_json_budget(&page["sdf_record"], None)?;
            let expected_fields = fields::expected_fields(mixed, kept_id);
            assert_eq!(page["sdf_record"]["fields"], expected_fields);
            assert_snapshot(&engine, source, expected)?;
            let saved = engine.dispatch("document.save_sdf", json!({"path":output}))?;
            let receipt = saved.file_operation.ok_or("actual retained Save")?;
            assert_eq!(receipt.format, bondline_commands::FileFormat::Sdf);
            assert_eq!(receipt.input_revision, 2);
            assert_eq!(receipt.byte_length, expected.len());
            assert!(receipt.ui_request_id.is_none() && receipt.ui_request_current.is_none());
            assert!(!saved.file.dirty);
            assert!(saved.source.ok_or("original source")?.modified);
            let actual = fs::read(&output)?;
            assert_eq!(actual, expected);
            consume_retained(&actual, version, mol_len, &expected_fields)?;
            let undo = engine.dispatch("history.undo", json!({}))?;
            assert_eq!((undo.revision, undo.undo_depth, undo.redo_depth), (3, 0, 1));
            assert!(undo.file.dirty);
            assert!(!undo.source.ok_or("restored source")?.modified);
            assert_eq!(engine.content(), &original);
            let redo = engine.dispatch("history.redo", json!({}))?;
            assert_eq!((redo.revision, redo.undo_depth, redo.redo_depth), (4, 1, 0));
            assert!(!redo.file.dirty);
            assert!(redo.source.ok_or("removed source")?.modified);
            assert_snapshot(&engine, source, expected)?;
            let mut reopened = Engine::default();
            reopened.dispatch("document.open_sdf", json!({"path":output}))?;
            assert_snapshot(&reopened, expected, expected)?;
            assert_eq!(reopened.state().undo_depth, 0);
            assert!(!reopened.state().file.dirty);
            assert_eq!(fs::read(input)?, source);
        }
    }
    Ok(())
}

#[test]
fn wrong_kind_targets_revision_and_sole_record_refusals_preserve_every_snapshot() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let mut drawing = Engine::default();
    let before = serde_json::to_value(drawing.state())?;
    let content = drawing.content().clone();
    assert_eq!(
        drawing
            .dispatch("sdf.record.remove", remove(0, 1))
            .err()
            .ok_or("wrong content")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(serde_json::to_value(drawing.state())?, before);
    assert_eq!(drawing.content(), &content);
    let mut engine = opened(&directory.path("refusals.sdf"), MIXED)?;
    for (params, code) in [
        (remove(0, 1), "stale_revision"),
        (remove(1, 0), "invalid_params"),
        (remove(1, 3), "not_found"),
        (remove(1, u32::MAX), "not_found"),
    ] {
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        assert_eq!(
            engine
                .dispatch("sdf.record.remove", params)
                .err()
                .ok_or("removal refusal")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
    }
    engine.dispatch("sdf.record.remove", remove(1, 2))?;
    for (id, code) in [(2, "not_found"), (1, "invalid_sdf")] {
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        assert_eq!(
            engine
                .dispatch("sdf.record.remove", remove(2, id))
                .err()
                .ok_or("nonempty-file/target refusal")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert_snapshot(&engine, MIXED, MIXED.get(..462).ok_or("first record")?)?;
    }
    Ok(())
}

#[test]
fn deferred_removal_guards_revision_source_cancel_and_committed_close() -> TestResult {
    use bondline_commands::{Cancellation, CloseState, Command};
    let directory = support::SdfTestDirectory::new()?;
    let path = directory.path("deferred.sdf");
    let mut engine = opened(&path, MIXED)?;
    let query = engine.prepare(Command::parse("sdf.record.inspect", remove(1, 2))?)?;
    let first = engine.prepare(Command::parse("sdf.record.remove", remove(1, 1))?)?;
    let second = engine.prepare(Command::parse("sdf.record.remove", remove(1, 2))?)?;
    assert!(first.is_deferred() && second.is_deferred());
    assert_snapshot(&engine, MIXED, MIXED)?;
    let first = first.complete()?;
    let second = second.complete()?;
    engine.finish(second)?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .finish(first)
            .err()
            .ok_or("reversed candidate")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let historical = serde_json::to_value(engine.finish(query.complete()?)?)?;
    assert_eq!(historical["sdf_record"]["input_revision"], 1);
    assert_eq!(historical["sdf_record"]["record_id"], 2);
    assert_eq!(
        historical["sdf_record"]["fields"],
        fields::expected_fields(true, 2)
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);

    for after_work in [false, true] {
        let mut current = opened(&directory.path(&format!("cancel-{after_work}.sdf")), MIXED)?;
        let before = serde_json::to_value(current.state())?;
        let snapshot = current.content().clone();
        let work = current.prepare(Command::parse("sdf.record.remove", remove(1, 1))?)?;
        let cancel = Cancellation::default();
        let error = if after_work {
            let completed = work.complete_with_cancel(&cancel)?;
            cancel.cancel();
            current.finish(completed).err().ok_or("cancelled finish")?
        } else {
            cancel.cancel();
            work.complete_with_cancel(&cancel)
                .err()
                .ok_or("cancelled worker")?
        };
        assert_eq!(error.info().code, "cancelled");
        assert_eq!(serde_json::to_value(current.state())?, before);
        assert_eq!(current.content(), &snapshot);
    }
    let mut current = opened(&directory.path("source-aba.sdf"), MIXED)?;
    let old = current
        .prepare(Command::parse("sdf.record.remove", remove(1, 1))?)?
        .complete()?;
    let other = directory.path("same-bytes.sdf");
    fs::write(&other, MIXED)?;
    current.dispatch("document.open_sdf", json!({"path":other}))?;
    let before = serde_json::to_value(current.state())?;
    assert_eq!(
        current
            .finish(old)
            .err()
            .ok_or("same-byte Open ABA")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(current.state())?, before);
    assert_snapshot(&current, MIXED, MIXED)?;

    let foreign = opened(&directory.path("foreign-origin.sdf"), MIXED)?
        .prepare(Command::parse("sdf.record.remove", remove(1, 1))?)?
        .complete()?;
    let mut receiver = opened(&directory.path("foreign-receiver.sdf"), MIXED)?;
    let before = serde_json::to_value(receiver.state())?;
    assert_eq!(
        receiver
            .finish(foreign)
            .err()
            .ok_or("foreign owner")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(receiver.state())?, before);

    let mut allowed = opened(&directory.path("approved-close.sdf"), MIXED)?;
    allowed.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    assert_eq!(allowed.state().close, CloseState::Approved);
    let edit = allowed.prepare(Command::parse("sdf.record.remove", remove(1, 2))?)?;
    assert!(allowed.finish(edit.complete()?)?.changed);
    assert_eq!(allowed.state().close, CloseState::NeedsDecision);
    assert!(!allowed.complete_close());

    let mut closed = opened(&directory.path("committed-close.sdf"), MIXED)?;
    let late = closed
        .prepare(Command::parse("sdf.record.remove", remove(1, 2))?)?
        .complete()?;
    closed.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    assert!(closed.complete_close());
    let before = serde_json::to_value(closed.state())?;
    assert_eq!(
        closed
            .finish(late)
            .err()
            .ok_or("committed close")?
            .info()
            .code,
        "close_committed"
    );
    assert_eq!(serde_json::to_value(closed.state())?, before);
    assert_snapshot(&closed, MIXED, MIXED)?;
    Ok(())
}

#[test]
fn captured_save_publishes_its_whole_source_and_keeps_the_same_lease_through_removal() -> TestResult
{
    use bondline_commands::Command;
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("captured-input.sdf");
    let output = directory.path("captured-output.sdf");
    let mut engine = opened(&input, ORDERED)?;
    let save = engine.prepare(Command::parse("document.save_sdf", json!({"path":output}))?)?;
    engine.dispatch("sdf.record.remove", remove(1, 1))?;
    let expected = ORDERED.get(424..).ok_or("literal second record")?;
    let completed = save.complete()?;
    support::independently_read_sdf(&output, ORDERED, false)?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .prepare(Command::parse(
                "document.save_sdf",
                json!({"path":output,"overwrite":true})
            )?)
            .err()
            .ok_or("published Save still reserved")?
            .info()
            .code,
        "busy"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let saved = engine.finish(completed)?;
    let receipt = saved.file_operation.ok_or("captured receipt")?;
    assert_eq!(receipt.byte_length, 690);
    assert_eq!(receipt.input_revision, 1);
    assert!(receipt.ui_request_id.is_none() && receipt.ui_request_current.is_none());
    assert!(saved.file.dirty && saved.source.ok_or("original source")?.modified);
    assert_snapshot(&engine, ORDERED, expected)?;
    let result = engine.dispatch("document.save_sdf", json!({"path":output,"overwrite":true}))?;
    assert_eq!(
        result.file_operation.ok_or("current receipt")?.byte_length,
        266
    );
    assert!(!result.file.dirty);
    assert!(result.source.ok_or("retained original")?.modified);
    let actual = fs::read(output)?;
    assert_eq!(actual, expected);
    consume_retained(&actual, "v2000", 261, &json!([]))?;
    assert_eq!(fs::read(input)?, ORDERED);
    Ok(())
}
