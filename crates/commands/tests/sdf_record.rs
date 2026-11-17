use bondline_commands::{Command, Engine, registry};
use serde_json::{Value, json};
use std::fs;

#[path = "../../../xtask/oracles/sdf_record_test_support.rs"]
mod expected;
#[path = "../../../xtask/oracles/sdf_test_support.rs"]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORDERED: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
const MIXED: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

#[test]
fn sdf_record_query_schema_is_read_only_closed_and_keeps_all_eighteen_names() -> TestResult {
    let names = registry()
        .into_iter()
        .map(|row| row.name)
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
            "document.open_smiles",
            "document.save_sdf",
            "document.save_smiles",
            "document.set_drawing_settings",
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
            "ui.chain_tool",
        ]
        .into_iter()
        .collect()
    );
    let schema = registry()
        .into_iter()
        .find(|row| row.name == "sdf.record.inspect")
        .ok_or("missing query schema")?;
    assert!(schema.read_only);
    let schema = schema.input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["required"],
        json!(["expected_revision", "record_id"])
    );
    assert_eq!(schema["properties"]["record_id"]["minimum"], 1);
    assert_eq!(schema["properties"]["field_offset"]["default"], 0);
    assert_eq!(schema["properties"]["field_limit"]["default"], 64);
    assert_eq!(schema["properties"]["field_limit"]["minimum"], 1);
    assert_eq!(schema["properties"]["field_limit"]["maximum"], 64);
    for params in [
        json!({}),
        json!({"expected_revision":1}),
        json!({"record_id":1}),
        json!({"expected_revision":1,"record_id":1,"include_preview":false}),
        json!({"expected_revision":1,"record_id":1,"field_offset":-1}),
        json!({"expected_revision":1,"record_id":1,"field_limit":256}),
    ] {
        assert_eq!(
            Command::parse("sdf.record.inspect", params)
                .err()
                .ok_or("invalid schema")?
                .info()
                .code,
            "invalid_params"
        );
    }
    Ok(())
}

#[test]
fn original_two_record_queries_disclose_ordered_fields_only_on_explicit_request() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    for (source, mixed) in [(ORDERED, false), (MIXED, true)] {
        let path = directory.path(if mixed { "mixed.sdf" } else { "ordered.sdf" });
        fs::write(&path, source)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_sdf", json!({"path":path}))?;
        let before = serde_json::to_value(engine.state())?;
        assert_eq!(before["collection"], support::expected_summary(mixed));
        assert!(before.get("sdf_record").is_none());
        for record_id in [1, 2] {
            let result = engine.dispatch(
                "sdf.record.inspect",
                json!({"expected_revision":1,"record_id":record_id}),
            )?;
            let wire = serde_json::to_value(&result)?;
            let page = &wire["sdf_record"];
            let fields = expected::expected_fields(mixed, record_id);
            assert_eq!(page["input_revision"], 1);
            assert_eq!(page["session_generation"], 1);
            assert_eq!(page["record_id"], record_id);
            assert_eq!(
                page["mol_version"],
                if mixed && record_id == 1 {
                    "v3000"
                } else {
                    "v2000"
                }
            );
            assert_eq!(
                page["field_count"],
                fields.as_array().ok_or("fields")?.len()
            );
            assert_eq!(page["field_offset"], 0);
            assert_eq!(page["field_limit"], 64);
            assert_eq!(page["fields"], fields);
            assert_eq!(page["next_field_offset"], Value::Null);
            assert_eq!(page["page_limited"], false);
            expected::assert_json_budget(page, None)?;
            assert!(!wire["changed"].as_bool().ok_or("changed")?);
            assert_eq!(wire["document"], Value::Null);
            assert_eq!(serde_json::to_value(engine.state())?, before);
            let debug = format!("{result:?}");
            for text in ["numbered", "> <EMPTY>", "alpha", "second record", "M  END"] {
                assert!(!debug.contains(text));
            }
        }
        support::independently_read_sdf(&path, source, mixed)?;
        assert_eq!(fs::read(&path)?, source);
    }
    Ok(())
}

#[test]
fn query_runtime_guards_pagination_zero_fields_and_end_offsets_are_side_effect_free() -> TestResult
{
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../io/tests/fixtures/sdf/ordered-fields-v2000.sdf"
    );
    let mut engine = Engine::default();
    assert_eq!(
        engine
            .dispatch(
                "sdf.record.inspect",
                json!({"expected_revision":0,"record_id":1})
            )
            .err()
            .ok_or("wrong kind")?
            .info()
            .code,
        "unsupported_content"
    );
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    for (params, code) in [
        (
            json!({"expected_revision":0,"record_id":1}),
            "stale_revision",
        ),
        (
            json!({"expected_revision":1,"record_id":0}),
            "invalid_params",
        ),
        (json!({"expected_revision":1,"record_id":3}), "not_found"),
        (
            json!({"expected_revision":1,"record_id":1,"field_offset":8}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":1,"record_id":1,"field_limit":0}),
            "invalid_params",
        ),
        (
            json!({"expected_revision":1,"record_id":1,"field_limit":65}),
            "invalid_params",
        ),
    ] {
        assert_eq!(
            engine
                .dispatch("sdf.record.inspect", params)
                .err()
                .ok_or("invalid runtime params")?
                .info()
                .code,
            code
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    for (record_id, offset, limit, indices, next) in [
        (1, 0, 2, vec![0, 1], Some(2)),
        (1, 2, 3, vec![2, 3, 4], Some(5)),
        (1, 5, 64, vec![5, 6], None),
        (1, 7, 64, vec![], None),
        (2, 0, 64, vec![], None),
    ] {
        let result=serde_json::to_value(engine.dispatch("sdf.record.inspect",json!({"expected_revision":1,"record_id":record_id,"field_offset":offset,"field_limit":limit}))?)?;
        let page = &result["sdf_record"];
        assert_eq!(
            page["fields"]
                .as_array()
                .ok_or("fields")?
                .iter()
                .map(|field| field["index"].as_u64().ok_or("index"))
                .collect::<Result<Vec<_>, _>>()?,
            indices
        );
        assert_eq!(page["next_field_offset"], json!(next));
        assert_eq!(page["page_limited"], false);
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    assert!(engine.complete_close());
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(
        engine
            .dispatch(
                "sdf.record.inspect",
                json!({"expected_revision":1,"record_id":1})
            )
            .err()
            .ok_or("committed close")?
            .info()
            .code,
        "close_committed"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn explicit_values_distinguish_empty_spaces_tabs_crlf_and_exact_json_and_raw_limits() -> TestResult
{
    let directory = support::SdfTestDirectory::new()?;
    let mut exact = vec!["a".repeat(200); 322];
    exact.push("a".repeat(166));
    assert_eq!(serde_json::to_vec(&exact)?.len(), 65536);
    let mut json_over = exact.clone();
    json_over.last_mut().ok_or("last line")?.push('b');
    let mut raw_over = vec!["a".repeat(200); 326];
    raw_over.push("a".repeat(10));
    let mut source = ORDERED
        .get(..282)
        .ok_or("original independent MOL span")?
        .to_vec();
    for (name, lines) in [
        ("ExactJson", exact.clone()),
        ("JsonOver", json_over.clone()),
        ("RawOver", raw_over.clone()),
        ("Empty", vec![]),
        ("SpacesTab", vec![" \t ".to_owned()]),
        ("EscapedOver", vec!["\t\"\\".repeat(60); 181]),
    ] {
        source.extend_from_slice(format!("> <{name}>\r\n").as_bytes());
        for line in lines {
            source.extend_from_slice(line.as_bytes());
            source.push(b'\n');
        }
        source.extend_from_slice(b"\r\n");
    }
    source.extend_from_slice(b"$$$$\r\n");
    let path = directory.path("literal-value-bounds.sdf");
    fs::write(&path, &source)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    let before = serde_json::to_value(engine.state())?;
    let result = serde_json::to_value(engine.dispatch(
        "sdf.record.inspect",
        json!({"expected_revision":1,"record_id":1}),
    )?)?;
    let fields = result["sdf_record"]["fields"].as_array().ok_or("fields")?;
    assert_eq!(fields.len(), 6);
    assert_eq!(fields[0]["value_lines"], json!(exact));
    assert_eq!(fields[0]["value_byte_length"], 64889);
    assert_eq!(fields[0]["omitted_reason"], Value::Null);
    assert_eq!(fields[1]["value_lines"], Value::Null);
    assert_eq!(fields[1]["value_byte_length"], 64890);
    assert_eq!(fields[1]["omitted_reason"], "value_json_limit");
    assert_eq!(fields[2]["value_lines"], Value::Null);
    assert_eq!(fields[2]["value_byte_length"], 65537);
    assert_eq!(fields[2]["omitted_reason"], "value_byte_limit");
    assert_eq!(fields[3]["value_lines"], json!([]));
    assert_eq!(fields[3]["value_byte_length"], 0);
    assert_eq!(fields[4]["value_lines"], json!([" \t "]));
    assert_eq!(fields[4]["value_byte_length"], 4);
    assert_eq!(fields[4]["header"], "> <SpacesTab>");
    assert_eq!(fields[5]["value_lines"], Value::Null);
    assert_eq!(fields[5]["value_byte_length"], 32761);
    assert_eq!(fields[5]["omitted_reason"], "value_json_limit");
    expected::assert_json_budget(&result["sdf_record"], None)?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(fs::read(&path)?, source);
    let molecule = support::describe_source("mol_oracle.py", &source[..282])?;
    assert_eq!(molecule["smiles"], "CC");
    Ok(())
}

#[test]
fn page_budget_counts_escaped_headers_and_line_vectors_and_makes_finite_ordered_progress()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let line = "\t\"\\".repeat(60);
    let lines = vec![line; 160];
    assert_eq!(serde_json::to_vec(&lines)?.len(), 58081);
    let mut source = ORDERED.get(..282).ok_or("independent MOL span")?.to_vec();
    let mut fields = Vec::new();
    for index in 0..64 {
        let name = format!("F{index}");
        let header = format!("> ({})<{name}>DT00009", "\t\"\\".repeat(1000));
        source.extend_from_slice(header.as_bytes());
        source.push(b'\n');
        for line in &lines {
            source.extend_from_slice(line.as_bytes());
            source.push(b'\n');
        }
        source.push(b'\n');
        fields.push(json!({"index":index,"header":header,"name":name,"dt_number":"00009","value_byte_length":28960,"value_line_count":160,"value_lines":lines,"omitted_reason":null}));
    }
    source.extend_from_slice(b"$$$$\n");
    let path = directory.path("escaped-page.sdf");
    fs::write(&path, &source)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    let before = serde_json::to_value(engine.state())?;
    let mut offset = 0;
    let mut pages = 0;
    while offset < 64 {
        let result = serde_json::to_value(engine.dispatch(
            "sdf.record.inspect",
            json!({"expected_revision":1,"record_id":1,"field_offset":offset}),
        )?)?;
        let page = &result["sdf_record"];
        let actual = page["fields"].as_array().ok_or("page fields")?;
        assert!(!actual.is_empty());
        let end = offset + actual.len();
        assert_eq!(
            actual,
            fields.get(offset..end).ok_or("literal expected page")?
        );
        expected::assert_json_budget(page, fields.get(end).cloned())?;
        if end < 64 {
            assert_eq!(page["next_field_offset"], end);
            assert_eq!(page["page_limited"], true);
        } else {
            assert_eq!(page["next_field_offset"], Value::Null);
            assert_eq!(page["page_limited"], false);
        }
        offset = end;
        pages += 1;
        assert!(pages <= 64);
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    assert!(pages > 1);
    assert_eq!(fs::read(&path)?, source);
    Ok(())
}

#[test]
fn immutable_queries_return_captured_identity_after_new_sessions_and_honor_cancellation()
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
    let query = || {
        Command::parse(
            "sdf.record.inspect",
            json!({"expected_revision":1,"record_id":1}),
        )
    };
    let first = engine.prepare(query()?)?;
    assert!(first.is_deferred());
    let second = engine.prepare(query()?)?;
    engine.dispatch("document.open_sdf", json!({"path":mixed}))?;
    let before = serde_json::to_value(engine.state())?;
    for work in [second, first] {
        let captured = serde_json::to_value(engine.finish(work.complete()?)?)?;
        assert_eq!(captured["revision"], 1);
        assert_eq!(captured["sdf_record"]["input_revision"], 1);
        assert_eq!(captured["sdf_record"]["session_generation"], 1);
        assert_eq!(
            captured["sdf_record"]["fields"],
            expected::expected_fields(false, 1)
        );
        assert_eq!(captured["collection"], support::expected_summary(false));
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    let query = || {
        Command::parse(
            "sdf.record.inspect",
            json!({"expected_revision":2,"record_id":1}),
        )
    };
    let cancel = bondline_commands::Cancellation::default();
    cancel.cancel();
    assert_eq!(
        engine
            .prepare(query()?)?
            .complete_with_cancel(&cancel)
            .err()
            .ok_or("cancelled query")?
            .info()
            .code,
        "cancelled"
    );
    let cancel = bondline_commands::Cancellation::default();
    let completed = engine.prepare(query()?)?.complete_with_cancel(&cancel)?;
    cancel.cancel();
    assert_eq!(
        engine
            .finish(completed)
            .err()
            .ok_or("cancelled query apply")?
            .info()
            .code,
        "cancelled"
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn crlf_values_keep_logical_empty_lines_and_exact_retained_byte_counts() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let source = std::str::from_utf8(ORDERED)?.replace('\n', "\r\n");
    let path = directory.path("literal-crlf.sdf");
    fs::write(&path, source.as_bytes())?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    let before = serde_json::to_value(engine.state())?;
    let result = serde_json::to_value(engine.dispatch(
        "sdf.record.inspect",
        json!({"expected_revision":1,"record_id":1}),
    )?)?;
    let mut fields = expected::expected_fields(false, 1);
    for field in fields.as_array_mut().ok_or("fields")? {
        field["value_byte_length"] = json!(
            field["value_byte_length"].as_u64().ok_or("bytes")?
                + field["value_line_count"].as_u64().ok_or("lines")?
        );
    }
    assert_eq!(result["sdf_record"]["fields"], fields);
    expected::assert_json_budget(&result["sdf_record"], None)?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    let independent = support::describe_source("sdf_oracle.py", &fs::read(path)?)?;
    assert_eq!(independent["records"][0]["fields"][3]["dt_number"], "0009");
    assert_eq!(
        independent["records"][0]["fields"][1]["value_lines"],
        json!([])
    );
    assert_eq!(independent["records"][0]["rdkit"]["smiles"], "CC");
    Ok(())
}

#[test]
fn query_retains_opaque_mol_content_without_projecting_or_disclosing_molecular_bytes() -> TestResult
{
    let directory = support::SdfTestDirectory::new()?;
    let source = std::str::from_utf8(ORDERED)?.replacen("M  END", "M  XYZ  1   1   7\nM  END", 1);
    let path = directory.path("opaque-query.sdf");
    fs::write(&path, source.as_bytes())?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":path}))?;
    let before = serde_json::to_value(engine.state())?;
    let result = serde_json::to_value(engine.dispatch(
        "sdf.record.inspect",
        json!({"expected_revision":1,"record_id":1}),
    )?)?;
    assert_eq!(
        result["sdf_record"]["fields"],
        expected::expected_fields(false, 1)
    );
    assert!(!result.to_string().contains("M  XYZ"));
    assert!(!result.to_string().contains("M  END"));
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(fs::read(&path)?, source.as_bytes());
    let independent = support::describe_source("sdf_oracle.py", &fs::read(path)?)?;
    assert_eq!(independent["records"][0]["rdkit"]["smiles"], "CC");
    assert_eq!(independent["records"][1]["rdkit"]["smiles"], "CC");
    Ok(())
}
