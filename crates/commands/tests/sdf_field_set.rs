use bondline_commands::{Command, Engine, registry};
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/sdf_record_test_support.rs"]
mod expected;
#[path = "../../../xtask/oracles/sdf_field_test_support.rs"]
mod field;
#[path = "../../../xtask/oracles/sdf_test_support.rs"]
mod sdf_support;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn set(revision: u64, index: usize, lines: Value) -> Value {
    json!({"expected_revision":revision,"record_id":1,"field_index":index,"value_lines":lines})
}

fn inspected(engine: &mut Engine, revision: u64) -> Result<Value, Box<dyn std::error::Error>> {
    let result = serde_json::to_value(engine.dispatch(
        "sdf.record.inspect",
        json!({"expected_revision":revision,"record_id":1}),
    )?)?;
    expected::assert_json_budget(&result["sdf_record"], None)?;
    Ok(result["sdf_record"]["fields"].clone())
}

#[test]
fn setter_schema_is_closed_mutating_and_retains_all_nineteen_prior_commands() -> TestResult {
    let names = registry()
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
            "document.open_smiles",
            "document.save_sdf",
            "document.save_smiles",
            "file.convert_smiles_to_cdxml",
            "file.export_smiles_fragment",
            "sdf.record.inspect",
            "sdf.record.remove",
            "smiles.record.inspect",
            "structure.paste_smiles",
            "sdf.field.set_value",
            "page.set_size",
            "bond.create",
            "bond.extend",
            "bond.set_order",
            "history.undo",
            "history.redo",
            "render.capture",
            "ring.create",
        ]
        .into_iter()
        .collect()
    );
    let info = registry()
        .into_iter()
        .find(|info| info.name == "sdf.field.set_value")
        .ok_or("setter schema")?;
    assert!(!info.read_only);
    let schema = info.input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["required"],
        json!([
            "expected_revision",
            "record_id",
            "field_index",
            "value_lines"
        ])
    );
    assert_eq!(schema["properties"]["record_id"]["minimum"], 1);
    assert_eq!(schema["properties"]["value_lines"]["type"], "array");
    assert_eq!(
        schema["properties"]["value_lines"]["items"]["type"],
        "string"
    );
    assert!(Command::parse("sdf.field.set_value", set(1, 5, json!([]))).is_ok());
    for params in [
        json!({}),
        json!({"expected_revision":1,"record_id":1,"field_index":5}),
        json!({"expected_revision":1,"record_id":1,"field_index":-1,"value_lines":[]}),
        json!({"expected_revision":1,"record_id":1,"field_index":5,"value_lines":"x"}),
        json!({"expected_revision":1,"record_id":1,"field_index":5,"value_lines":[1]}),
        json!({"expected_revision":1,"record_id":1,"field_index":5,"value_lines":[],"name":"DUP"}),
    ] {
        assert_eq!(
            Command::parse("sdf.field.set_value", params)
                .err()
                .ok_or("closed setter params")?
                .info()
                .code,
            "invalid_params"
        );
    }
    Ok(())
}

#[test]
fn complete_set_save_undo_redo_workflow_changes_only_the_literal_second_duplicate() -> TestResult {
    let directory = sdf_support::SdfTestDirectory::new()?;
    for mixed in [false, true] {
        let source = field::source(mixed);
        let input = directory.path(if mixed {
            "mixed-input.sdf"
        } else {
            "ordered-input.sd"
        });
        let output = directory.path(if mixed {
            "mixed-edited-λ.data"
        } else {
            "ordered-edited-λ.data"
        });
        fs::write(&input, source)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_sdf", json!({"path":input}))?;
        assert_eq!(
            serde_json::to_value(engine.state())?["collection"],
            sdf_support::expected_summary(mixed)
        );
        assert_eq!(
            inspected(&mut engine, 1)?,
            expected::expected_fields(mixed, 1)
        );
        let set_result = engine.dispatch(
            "sdf.field.set_value",
            set(
                1,
                field::field_index(mixed),
                json!(["revised", "multiline"]),
            ),
        )?;
        assert_eq!(
            (
                set_result.changed,
                set_result.revision,
                set_result.undo_depth,
                set_result.redo_depth
            ),
            (true, 2, 1, 0)
        );
        assert!(set_result.file.dirty);
        assert_eq!(
            set_result.source,
            Some(bondline_io::SourceInfo {
                format: "sdf",
                byte_length: source.len(),
                modified: true
            })
        );
        assert_eq!(
            engine.content().sdf()?.as_bytes(),
            field::edited_bytes(mixed)
        );
        let mut fields = expected::expected_fields(mixed, 1);
        let target = &mut fields[field::field_index(mixed)];
        target["value_lines"] = json!(["revised", "multiline"]);
        target["value_byte_length"] = json!(18);
        target["value_line_count"] = json!(2);
        assert_eq!(inspected(&mut engine, 2)?, fields);
        assert!(
            serde_json::to_value(engine.state())?
                .get("sdf_record")
                .is_none()
        );
        let saved = engine.dispatch("document.save_sdf", json!({"path":output}))?;
        assert_eq!(
            saved.file_operation.ok_or("save receipt")?.byte_length,
            if mixed { 788 } else { 704 }
        );
        assert!(!saved.file.dirty);
        assert!(saved.source.ok_or("source")?.modified);
        field::assert_edited_file(&output, mixed)?;
        let undo = engine.dispatch("history.undo", json!({}))?;
        assert_eq!((undo.revision, undo.undo_depth, undo.redo_depth), (3, 0, 1));
        assert!(undo.file.dirty);
        assert!(!undo.source.ok_or("original source")?.modified);
        assert_eq!(engine.content().sdf()?.as_bytes(), source);
        assert_eq!(
            inspected(&mut engine, 3)?,
            expected::expected_fields(mixed, 1)
        );
        let redo = engine.dispatch("history.redo", json!({}))?;
        assert_eq!((redo.revision, redo.undo_depth, redo.redo_depth), (4, 1, 0));
        assert!(!redo.file.dirty);
        assert!(redo.source.ok_or("changed source")?.modified);
        assert_eq!(
            engine.content().sdf()?.as_bytes(),
            field::edited_bytes(mixed)
        );
        assert_eq!(inspected(&mut engine, 4)?, fields);
        let mut reopened = Engine::default();
        reopened.dispatch("document.open_sdf", json!({"path":output}))?;
        assert_eq!(
            reopened.content().sdf()?.as_bytes(),
            field::edited_bytes(mixed)
        );
        assert!(!reopened.state().source.ok_or("reopened source")?.modified);
        assert_eq!(fs::read(&input)?, source);
        sdf_support::independently_read_sdf(&input, source, mixed)?;
    }
    Ok(())
}

#[test]
fn invalid_targets_values_and_both_input_budgets_preserve_complete_state() -> TestResult {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let input = directory.path("invalid-input.sdf");
    fs::write(&input, field::source(false))?;
    let mut engine = Engine::default();
    assert_eq!(
        engine
            .dispatch("sdf.field.set_value", set(0, 5, json!(["x"])))
            .err()
            .ok_or("wrong kind")?
            .info()
            .code,
        "unsupported_content"
    );
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    let mut raw_over = vec!["a".repeat(200); 326];
    raw_over.push("a".repeat(10));
    let escaped_over = vec!["\t\"\\".repeat(60); 181];
    for (params, code) in [
        (set(0, 5, json!(["x"])), "stale_revision"),
        (
            json!({"expected_revision":1,"record_id":0,"field_index":5,"value_lines":[]}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":1,"record_id":3,"field_index":0,"value_lines":[]}),
            "not_found",
        ),
        (set(1, 99, json!(["x"])), "not_found"),
        (set(1, 5, json!([""])), "invalid_sdf"),
        (set(1, 5, json!(["line\nbreak"])), "invalid_sdf"),
        (set(1, 5, json!(["carriage\rreturn"])), "invalid_sdf"),
        (set(1, 5, json!(["$$$$reserved"])), "invalid_sdf"),
        (set(1, 5, json!(["é"])), "unsupported_content"),
        (set(1, 5, json!(["\0"])), "invalid_sdf"),
        (set(1, 5, json!(["a".repeat(201)])), "resource_limit"),
        (set(1, 5, json!(raw_over)), "resource_limit"),
        (set(1, 5, json!(escaped_over)), "resource_limit"),
    ] {
        let before_content = engine.content().clone();
        assert_eq!(
            engine
                .dispatch("sdf.field.set_value", params)
                .err()
                .ok_or("refused field edit")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &before_content);
        assert_eq!(fs::read(&input)?, field::source(false));
    }
    Ok(())
}

#[test]
fn empty_spaces_tabs_and_exact_json_boundary_remain_inspectable() -> TestResult {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let input = directory.path("values.sdf");
    fs::write(&input, field::source(false))?;
    let mut exact = vec!["a".repeat(200); 322];
    exact.push("a".repeat(166));
    assert_eq!(serde_json::to_vec(&exact)?.len(), 65536);
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    for (revision, lines, literal) in [
        (1, json!([]), Vec::new()),
        (2, json!([" ", "\t"]), b" \n\t\n".to_vec()),
        (
            3,
            json!(exact),
            exact
                .iter()
                .flat_map(|s| s.bytes().chain([b'\n']))
                .collect(),
        ),
    ] {
        engine.dispatch("sdf.field.set_value", set(revision, 5, lines.clone()))?;
        assert_eq!(
            engine.content().sdf()?.as_bytes(),
            [
                &field::source(false)[..383],
                literal.as_slice(),
                &field::source(false)[387..]
            ]
            .concat()
        );
        let fields = inspected(&mut engine, revision + 1)?;
        assert_eq!(fields[5]["value_lines"], lines);
        assert_eq!(fields[5]["omitted_reason"], Value::Null);
        assert_eq!(fields[5]["value_byte_length"], literal.len());
        assert!(fields[5]["value_byte_length"].as_u64().ok_or("raw bytes")? <= 65536);
    }
    Ok(())
}

#[test]
fn crlf_noop_retains_outer_and_source_identity_redo_and_close_approval() -> TestResult {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let source = std::str::from_utf8(field::source(false))?.replace('\n', "\r\n");
    let input = directory.path("noop-crlf.sdf");
    fs::write(&input, source.as_bytes())?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    engine.dispatch("sdf.field.set_value", set(1, 5, json!(["temporary"])))?;
    engine.dispatch("history.undo", json!({}))?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":3,"action":"request"}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let outer = std::ptr::from_ref(engine.content());
    let source_pointer = engine.content().sdf()?.as_bytes().as_ptr();
    let records_pointer = engine.content().sdf()?.records().as_ptr();
    let result = engine.dispatch("sdf.field.set_value", set(3, 5, json!(["two"])))?;
    assert!(!result.changed);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(result.revision, 3);
    assert_eq!(result.redo_depth, 1);
    assert_eq!(std::ptr::from_ref(engine.content()), outer);
    assert_eq!(engine.content().sdf()?.as_bytes().as_ptr(), source_pointer);
    assert_eq!(engine.content().sdf()?.records().as_ptr(), records_pointer);
    assert_eq!(engine.content().sdf()?.as_bytes(), source.as_bytes());
    assert!(
        engine.complete_close(),
        "a no-op must not invalidate a clean close approval"
    );
    assert_eq!(fs::read(input)?, source.as_bytes());
    Ok(())
}

#[test]
fn immutable_field_candidates_refuse_reverse_stale_cancelled_and_committed_completions()
-> TestResult {
    let ordered = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/sdf/ordered-fields-v2000.sdf"
    );
    let mixed = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"
    );
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":ordered}))?;
    let query = engine.prepare(Command::parse(
        "sdf.record.inspect",
        json!({"expected_revision":1,"record_id":1}),
    )?)?;
    let before = serde_json::to_value(engine.state())?;
    let first = engine.prepare(Command::parse(
        "sdf.field.set_value",
        set(1, 5, json!(["first candidate"])),
    )?)?;
    let second = engine.prepare(Command::parse(
        "sdf.field.set_value",
        set(1, 5, json!(["second candidate"])),
    )?)?;
    assert!(first.is_deferred() && second.is_deferred());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let first = first.complete()?;
    let second = second.complete()?;
    engine.finish(second)?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .finish(first)
            .err()
            .ok_or("reverse candidate")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let historical = serde_json::to_value(engine.finish(query.complete()?)?)?;
    assert_eq!(historical["sdf_record"]["input_revision"], 1);
    assert_eq!(
        historical["sdf_record"]["fields"][5]["value_lines"],
        json!(["two"])
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    for after_work in [false, true] {
        let work = engine.prepare(Command::parse(
            "sdf.field.set_value",
            set(2, 5, json!(["cancelled"])),
        )?)?;
        let cancel = bondline_commands::Cancellation::default();
        let error = if after_work {
            let complete = work.complete_with_cancel(&cancel)?;
            cancel.cancel();
            engine.finish(complete).err().ok_or("cancelled finish")?
        } else {
            cancel.cancel();
            work.complete_with_cancel(&cancel)
                .err()
                .ok_or("cancelled work")?
        };
        assert_eq!(error.info().code, "cancelled");
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    let old = engine
        .prepare(Command::parse(
            "sdf.field.set_value",
            set(2, 5, json!(["old generation"])),
        )?)?
        .complete()?;
    engine.dispatch(
        "document.open_sdf",
        json!({"path":mixed,"discard_unsaved":true}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .finish(old)
            .err()
            .ok_or("old generation")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let late = engine
        .prepare(Command::parse(
            "sdf.field.set_value",
            set(3, 2, json!(["late"])),
        )?)?
        .complete()?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":3,"action":"request"}),
    )?;
    assert!(engine.complete_close());
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .finish(late)
            .err()
            .ok_or("late after close")?
            .info()
            .code,
        "close_committed"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn a_captured_whole_save_survives_a_real_field_edit_and_keeps_its_lease_until_finish() -> TestResult
{
    let directory = sdf_support::SdfTestDirectory::new()?;
    let input = directory.path("captured-input.sdf");
    fs::write(&input, field::source(false))?;
    let output = directory.path("captured-output.sdf");
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let save = engine.prepare(Command::parse("document.save_sdf", json!({"path":output}))?)?;
    engine.dispatch(
        "sdf.field.set_value",
        set(1, 5, json!(["revised", "multiline"])),
    )?;
    let completed = save.complete()?;
    sdf_support::independently_read_sdf(&output, field::source(false), false)?;
    assert_eq!(
        engine
            .prepare(Command::parse(
                "document.save_sdf",
                json!({"path":output,"overwrite":true})
            )?)
            .err()
            .ok_or("published Save busy")?
            .info()
            .code,
        "busy"
    );
    let receipt = engine.finish(completed)?;
    assert_eq!(
        receipt
            .file_operation
            .ok_or("captured receipt")?
            .byte_length,
        690
    );
    assert!(receipt.file.dirty);
    assert!(receipt.source.ok_or("source")?.modified);
    assert_eq!(
        engine.content().sdf()?.as_bytes(),
        field::edited_bytes(false)
    );
    let saved = engine.dispatch("document.save_sdf", json!({"path":output,"overwrite":true}))?;
    assert_eq!(
        saved.file_operation.ok_or("current receipt")?.byte_length,
        704
    );
    assert!(!saved.file.dirty);
    assert!(saved.source.ok_or("original source")?.modified);
    field::assert_edited_file(&output, false)?;
    Ok(())
}

#[test]
fn close_save_receipt_remains_valid_for_noop_but_not_for_real_edit_undo_aba() -> TestResult {
    let directory = sdf_support::SdfTestDirectory::new()?;
    for noop in [true, false] {
        let input = directory.path(if noop {
            "noop-close.sdf"
        } else {
            "aba-close.sdf"
        });
        fs::write(&input, field::source(false))?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_sdf", json!({"path":input}))?;
        engine.dispatch(
            "sdf.field.set_value",
            set(1, 5, json!(["revised", "multiline"])),
        )?;
        engine.set_ui_file_host(true);
        engine.dispatch(
            "document.close",
            json!({"expected_revision":2,"action":"request"}),
        )?;
        let epoch = engine.state().close_epoch;
        engine.dispatch(
            "ui.file",
            json!({"expected_revision":2,"action":"save","close_epoch":epoch}),
        )?;
        let id = engine
            .state()
            .ui_file
            .request
            .ok_or("close Save request")?
            .id;
        assert!(matches!(
            engine.claim_ui_file(id)?,
            bondline_commands::UiFilePlan::SaveExisting {
                format: bondline_commands::FileFormat::Sdf,
                ..
            }
        ));
        let actual = Command::parse(
            "document.save_sdf",
            json!({"path":input.canonicalize()?,"overwrite":true}),
        )?;
        let prepared = engine.prepare_ui_file(id, actual)?;
        let result = engine.finish(prepared.complete()?)?;
        assert_eq!(
            result.file_operation.ok_or("receipt")?.ui_request_current,
            Some(true)
        );
        if noop {
            let before = serde_json::to_value(engine.state())?;
            assert!(
                !engine
                    .dispatch(
                        "sdf.field.set_value",
                        set(2, 5, json!(["revised", "multiline"]))
                    )?
                    .changed
            );
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert!(engine.finish_ui_file(id, false));
            engine.dispatch(
                "document.close",
                json!({"expected_revision":2,"action":"request"}),
            )?;
            assert!(engine.complete_close());
        } else {
            engine.dispatch("sdf.field.set_value", set(2, 5, json!(["two"])))?;
            engine.dispatch("history.undo", json!({}))?;
            assert!(!engine.state().file.dirty);
            assert_eq!(
                engine.content().sdf()?.as_bytes(),
                field::edited_bytes(false)
            );
            assert!(!engine.finish_ui_file(id, false));
            assert!(!engine.complete_close());
        }
        field::assert_edited_file(&input, false)?;
    }
    Ok(())
}

#[test]
fn field_edit_keeps_unprojectable_molecular_metadata_opaque_through_actual_save() -> TestResult {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let source = std::str::from_utf8(field::source(false))?.replacen(
        "M  END",
        "M  XYZ  1   1   7\nM  END",
        1,
    );
    let expected_source = source.replacen("\ntwo\n\n", "\nrevised\nmultiline\n\n", 1);
    let input = directory.path("opaque-field-input.sdf");
    let output = directory.path("opaque-field-saved.sdf");
    fs::write(&input, source.as_bytes())?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    engine.dispatch(
        "sdf.field.set_value",
        set(1, 5, json!(["revised", "multiline"])),
    )?;
    engine.dispatch("document.save_sdf", json!({"path":output}))?;
    assert_eq!(fs::read(&output)?, expected_source.as_bytes());
    assert_eq!(
        engine.content().sdf()?.as_bytes(),
        expected_source.as_bytes()
    );
    assert!(engine.state().source.ok_or("source")?.modified);
    assert!(!engine.state().file.dirty);
    let raw = sdf_support::describe_source("sdf_oracle.py", &fs::read(&output)?)?;
    assert_eq!(
        raw["records"][0]["fields"][5]["value_lines"],
        json!(["revised", "multiline"])
    );
    assert_eq!(raw["records"][0]["rdkit"]["smiles"], "CC");
    assert_eq!(raw["records"][1]["rdkit"]["smiles"], "CC");
    assert_eq!(fs::read(input)?, source.as_bytes());
    Ok(())
}

#[test]
fn record_identity_first_duplicate_and_dt_only_occurrences_are_forwarded_exactly() -> TestResult {
    let directory = sdf_support::SdfTestDirectory::new()?;
    let input = directory.path("other-occurrences.sdf");
    fs::write(&input, field::source(true))?;
    let mut expected_source = std::str::from_utf8(field::source(true))?.to_owned();
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    engine.dispatch("sdf.field.set_value",json!({"expected_revision":1,"record_id":2,"field_index":0,"value_lines":["second edited"]}))?;
    expected_source = expected_source.replacen("\nsecond record\n\n", "\nsecond edited\n\n", 1);
    assert_eq!(
        engine.content().sdf()?.as_bytes(),
        expected_source.as_bytes()
    );
    assert_eq!(expected_source.len(), 777);
    assert!(engine.state().source.ok_or("source")?.modified);
    assert_eq!(
        inspected(&mut engine, 2)?,
        expected::expected_fields(true, 1)
    );
    engine.dispatch(
        "sdf.field.set_value",
        json!({"expected_revision":2,"record_id":1,"field_index":1,"value_lines":["new first"]}),
    )?;
    expected_source = expected_source.replacen("\nfirst\n\n", "\nnew first\n\n", 1);
    assert_eq!(
        engine.content().sdf()?.as_bytes(),
        expected_source.as_bytes()
    );
    let fields = inspected(&mut engine, 3)?;
    assert_eq!(fields[1]["value_lines"], json!(["new first"]));
    assert_eq!(fields[2]["value_lines"], json!(["second"]));
    engine.dispatch(
        "sdf.field.set_value",
        json!({"expected_revision":3,"record_id":1,"field_index":4,"value_lines":["textual DT"]}),
    )?;
    expected_source = expected_source.replacen("\nraw\n\n", "\ntextual DT\n\n", 1);
    assert_eq!(
        engine.content().sdf()?.as_bytes(),
        expected_source.as_bytes()
    );
    let fields = inspected(&mut engine, 4)?;
    assert_eq!(fields[4]["name"], Value::Null);
    assert_eq!(fields[4]["dt_number"], "0012");
    assert_eq!(fields[4]["value_lines"], json!(["textual DT"]));
    let output = directory.path("other-occurrences-saved.sdf");
    engine.dispatch("document.save_sdf", json!({"path":output}))?;
    assert_eq!(fs::read(&output)?, expected_source.as_bytes());
    let second = expected_source
        .find("Original V2000 vector\n")
        .ok_or("literal second MOL start")?;
    let molecule = sdf_support::describe_source(
        "mol_oracle.py",
        expected_source
            .as_bytes()
            .get(second..second + 282)
            .ok_or("literal MOL span")?,
    )?;
    assert_eq!(molecule["smiles"], "CC");
    assert_eq!(fs::read(input)?, field::source(true));
    Ok(())
}
