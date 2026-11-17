//! Independent transient ring-tool declaration using only baseline public APIs.
use bondline_commands::Engine;
use serde_json::{Value, json};
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn choosing_six_ring_is_a_headless_transient_operation_with_a_real_control() -> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let old_tool = engine.bond_tool();
    let result = engine.dispatch("ui.ring_tool", json!({"size":6}))?;
    assert!(!result.changed);
    let mut received = serde_json::to_value(result)?;
    assert_eq!(
        received["ui_ring_tool_operation"],
        json!({"size":6,"epoch":old_tool.epoch+1,"changed":true})
    );
    received
        .as_object_mut()
        .ok_or("result object")?
        .remove("ui_ring_tool_operation");
    assert_eq!(received, before);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.bond_tool().order, old_tool.order);
    assert_eq!(engine.bond_tool().epoch, old_tool.epoch + 1);
    let controls = engine.controls();
    let control = controls
        .iter()
        .find(|c| c.id == "tool.ring.6")
        .ok_or("Cyclohexane control")?;
    assert_eq!(control.label, "Cyclohexane");
    assert_eq!(control.menu, "Tools");
    assert!(control.enabled && control.shortcut.is_none());
    assert_eq!(control.selected, Some(true));
    assert_eq!(control.command.name(), "ui.ring_tool");
    assert_eq!(
        serde_json::to_value(&control.command)?,
        json!({"command":"ui.ring_tool","params":{"size":6}})
    );
    assert!(before.get("ui_ring_tool_operation").is_none());
    assert!(received.get("file_operation").is_none());
    let _: Option<&Value> = received.get("ui_ring_tool_operation");
    Ok(())
}

// Additive JSON-only modes/schema/source checks; no future Rust API can mask runtime RED.
use bondline_commands::Command;
use bondline_io::StoredDocument;
use std::{collections::BTreeSet, fs};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated file directory")]
mod support;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const RAW_SMILES: &[u8] = b"CC first\r\nO\topaque\xff\0";

fn authored(engine: &Engine) -> TestResult<Value> {
    let s = serde_json::to_value(engine.state())?;
    Ok(
        json!({"revision":s["revision"],"document":s["document"],"collection":s["collection"],
        "smiles_file":s["smiles_file"],"source":s["source"],"file":s["file"],
        "undo_depth":s["undo_depth"],"redo_depth":s["redo_depth"]}),
    )
}
fn assert_active_control(engine: &Engine, id: &str) -> TestResult {
    let active = engine
        .controls()
        .into_iter()
        .filter(|c| c.menu == "Tools" && c.selected == Some(true))
        .map(|c| c.id)
        .collect::<Vec<_>>();
    assert_eq!(active, vec![id]);
    Ok(())
}
fn ring_select(engine: &mut Engine, size: u8, epoch: u64, changed: bool) -> TestResult {
    let before = authored(engine)?;
    let owner = engine.capture_stamp();
    let tool = engine.bond_tool();
    let view = engine.view();
    let generation = engine.session_generation();
    let file = engine.ui_file_context_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let result = serde_json::to_value(engine.dispatch("ui.ring_tool", json!({"size":size}))?)?;
    assert_eq!(result["changed"], false);
    assert_eq!(
        result["ui_ring_tool_operation"],
        json!({"size":size,"epoch":epoch,"changed":changed})
    );
    assert_eq!(engine.bond_tool().epoch, epoch);
    assert_eq!(engine.bond_tool().order, tool.order);
    assert_eq!(authored(engine)?, before);
    assert_eq!(engine.capture_stamp(), owner);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert_eq!(engine.view(), view);
    assert_eq!(engine.session_generation(), generation);
    assert_active_control(engine, &format!("tool.ring.{size}"))?;
    assert!(
        serde_json::to_value(engine.state())?
            .get("ui_ring_tool_operation")
            .is_none()
    );
    Ok(())
}
fn refused_tool(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let owner = engine.capture_stamp();
    let model = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let tool = engine.bond_tool();
    let selection = engine.selection().clone();
    let view = engine.view();
    let file = engine.ui_file_context_stamp();
    let generation = engine.session_generation();
    let error = engine
        .dispatch("ui.ring_tool", params)
        .err()
        .ok_or("ring tool refusal accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), owner);
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.selection(), &selection);
    assert_eq!(engine.view(), view);
    assert_eq!(engine.session_generation(), generation);
    assert!(engine.ui_file_context_stamp_matches(&file));
    if let Some(model) = model {
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    Ok(())
}

#[test]
fn one_authoritative_mode_keeps_remembered_bond_order_and_derived_selection_consistent()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,
        "begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}),
    )?;
    let before = authored(&engine)?;
    assert_eq!(engine.bond_tool().epoch, 0);
    ring_select(&mut engine, 6, 1, true)?;
    ring_select(&mut engine, 6, 1, false)?;
    ring_select(&mut engine, 7, 2, true)?;
    let r = engine.dispatch("ui.bond_tool", json!({"order":"single"}))?;
    assert_eq!(
        serde_json::to_value(r)?["ui_bond_tool_operation"],
        json!({"order":"single","epoch":3,"changed":true})
    );
    assert_active_control(&engine, "tool.bond.single")?;
    engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
    assert_eq!(engine.bond_tool().epoch, 4);
    ring_select(&mut engine, 5, 5, true)?;
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[2,1]}),
    )?;
    assert_eq!(engine.bond_tool().epoch, 6);
    assert_active_control(&engine, "tool.selection")?;
    assert_eq!(
        serde_json::to_value(engine.selection())?,
        json!({"active":true,"epoch":1,"page_id":1,"atom_ids":[1,2]})
    );
    ring_select(&mut engine, 5, 7, true)?;
    assert_eq!(
        serde_json::to_value(engine.selection())?,
        json!({"active":false,"epoch":2,"page_id":null,"atom_ids":[]})
    );
    ring_select(&mut engine, 5, 7, false)?;
    engine.dispatch(
        "ui.selection",
        json!({"action":"activate","expected_revision":1}),
    )?;
    assert_eq!(engine.bond_tool().epoch, 8);
    assert_active_control(&engine, "tool.selection")?;
    assert!(engine.selection().active && engine.selection().atom_ids.is_empty());
    ring_select(&mut engine, 8, 9, true)?;
    let state = serde_json::to_value(engine.state())?;
    let clear = engine.dispatch(
        "ui.selection",
        json!({"action":"clear","expected_revision":1}),
    )?;
    assert_eq!(
        serde_json::to_value(clear)?["ui_selection_operation"]["changed"],
        false
    );
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert_active_control(&engine, "tool.ring.8")?;
    engine.dispatch(
        "ui.selection",
        json!({"action":"all","expected_revision":1,"page_id":1}),
    )?;
    assert_eq!(engine.bond_tool().epoch, 10);
    assert_active_control(&engine, "tool.selection")?;
    assert_eq!(engine.selection().atom_ids, [1, 2]);
    engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
    assert_eq!(engine.bond_tool().epoch, 11);
    assert_active_control(&engine, "tool.bond.double")?;
    assert_eq!(
        serde_json::to_value(engine.selection())?,
        json!({"active":false,"epoch":6,"page_id":null,"atom_ids":[]})
    );
    ring_select(&mut engine, 3, 12, true)?;
    assert_eq!(authored(&engine)?, before);
    let tool = engine.bond_tool();
    engine.dispatch(
        "ring.create",
        json!({"expected_revision":1,"page_id":1,"size":6,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free",
        "begin_x_pt":300,"begin_y_pt":168,"end_x_pt":300,"end_y_pt":120,"side":"positive"}}),
    )?;
    for command in ["history.undo", "history.redo"] {
        engine.dispatch(command, json!({}))?;
        assert_eq!(engine.bond_tool(), tool);
        assert_active_control(&engine, "tool.ring.3")?;
    }
    let path = directory.path("mode-retained-環.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    engine.dispatch("document.open_cdxml", json!({"path":path}))?;
    assert_eq!(engine.bond_tool(), tool);
    assert_active_control(&engine, "tool.ring.3")?;
    engine.dispatch(
        "document.new",
        json!({"expected_revision":engine.state().revision}),
    )?;
    assert_eq!(engine.bond_tool(), tool);
    ring_select(&mut engine, 3, 12, false)?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"request"}),
    )?;
    refused_tool(&mut engine, json!({"size":3}), "ui_busy")?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"cancel"}),
    )?;
    engine.set_ui_file_host(true);
    let request = engine.dispatch(
        "ui.file",
        json!({"expected_revision":engine.state().revision,"action":"save"}),
    )?;
    let id = request.ui_file_operation.ok_or("file request")?;
    refused_tool(&mut engine, json!({"size":3}), "ui_busy")?;
    engine.claim_ui_file(id)?;
    refused_tool(&mut engine, json!({"size":3}), "ui_busy")?;
    let path = directory.path("unpublished-tool-control.cdxml");
    let prepared = engine.prepare_ui_file(
        id,
        Command::parse("document.save_cdxml", json!({"path":path}))?,
    )?;
    ring_select(&mut engine, 3, 12, false)?;
    ring_select(&mut engine, 4, 13, true)?;
    drop(prepared);
    assert!(!path.exists());
    assert!(!engine.finish_ui_file(id, false));
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"discard"}),
    )?;
    assert!(engine.complete_close());
    refused_tool(&mut engine, json!({"size":4}), "close_committed")?;
    Ok(())
}

#[test]
fn ring_tool_schema_and_six_bindings_add_one_closed_command_without_placeholder_authoring()
-> TestResult {
    const OLD_37: [&str; 37] = [
        "atom.set_label",
        "bond.create",
        "bond.extend",
        "bond.set_order",
        "document.close",
        "document.import_smiles_records",
        "document.inspect",
        "document.new",
        "document.open_cdx",
        "document.open_cdxml",
        "document.open_mol",
        "document.open_sdf",
        "document.open_smiles",
        "document.save_cdx",
        "document.save_cdxml",
        "document.save_mol",
        "document.save_sdf",
        "document.save_smiles",
        "file.convert_smiles_to_cdxml",
        "file.export_smiles_fragment",
        "history.redo",
        "history.undo",
        "page.set_size",
        "render.capture",
        "ring.create",
        "sdf.field.set_value",
        "sdf.record.inspect",
        "sdf.record.remove",
        "selection.translate",
        "smiles.record.inspect",
        "structure.paste_smiles",
        "ui.atom_label",
        "ui.bond_tool",
        "ui.controls",
        "ui.file",
        "ui.selection",
        "ui.view",
    ];
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 42);
    let mut expected = OLD_37.into_iter().collect::<BTreeSet<_>>();
    assert!(expected.insert("ui.ring_tool"));
    assert!(expected.insert("chain.create"));
    assert!(expected.insert("chain.create_path"));
    assert!(expected.insert("document.set_drawing_settings"));
    assert!(expected.insert("ui.drawing_settings"));
    assert_eq!(
        registry.iter().map(|r| r.name).collect::<BTreeSet<_>>(),
        expected
    );
    let row = registry
        .iter()
        .find(|r| r.name == "ui.ring_tool")
        .ok_or("ring tool")?;
    assert!(!row.read_only);
    let union = serde_json::to_value(&row.input_schema)?;
    let schema = legacy_ui_ring_tool_schema(&union)?;
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"], json!(["size"]));
    assert_eq!(
        schema["properties"]
            .as_object()
            .ok_or("fields")?
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["size"]
    );
    assert_eq!(schema["properties"]["size"]["minimum"].as_f64(), Some(3.0));
    assert_eq!(schema["properties"]["size"]["maximum"].as_f64(), Some(8.0));
    let mut engine = Engine::default();
    assert_eq!(engine.controls().len(), 32);
    for (size, label) in [
        (3, "Cyclopropane"),
        (4, "Cyclobutane"),
        (5, "Cyclopentane"),
        (6, "Cyclohexane"),
        (7, "Cycloheptane"),
        (8, "Cyclooctane"),
    ] {
        let before = serde_json::to_value(engine.state())?;
        let id = format!("tool.ring.{size}");
        let controls = engine.controls();
        let c = controls.iter().find(|c| c.id == id).ok_or("ring control")?;
        assert_eq!(c.label, label);
        assert_eq!(c.menu, "Tools");
        assert!(c.shortcut.is_none());
        assert!(c.enabled);
        assert_eq!(c.selected, Some(false));
        assert_eq!(
            serde_json::to_value(&c.command)?,
            json!({"command":"ui.ring_tool","params":{"size":size}})
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
    }
    engine.dispatch("ui.ring_tool", json!({"size":6}))?;
    for params in [
        json!({}),
        json!({"size":null}),
        json!({"size":2}),
        json!({"size":9}),
        json!({"size":256}),
        json!({"size":6.0}),
        json!({"size":-1}),
        json!({"size":"6"}),
        json!({"size":6,"expected_revision":0}),
        json!({"size":6,"extra":true}),
    ] {
        refused_tool(&mut engine, params, "invalid_params")?;
    }
    Ok(())
}

#[test]
fn source_eligibility_precedes_same_mode_noop_and_complete_owners_stay_preserved() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let opaque = String::from_utf8(NATIVE.to_vec())?
        .replace("<CDXML ", "<CDXML NativeRingTest=\"preserved\" ");
    let opaque_path = directory.path("opaque.cdxml");
    fs::write(&opaque_path, opaque.as_bytes())?;
    for (name, bytes, open, count) in [
        (
            "full-v2.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
            None,
        ),
        (
            "full-v3.mol",
            include_bytes!("../../io/tests/fixtures/v3000/noncontiguous-cc.mol").as_slice(),
            "document.open_mol",
            None,
        ),
        (
            "full.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
            None,
        ),
        (
            "full.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
            Some(15),
        ),
        ("full.smiles", RAW_SMILES, "document.open_smiles", Some(11)),
        (
            "opaque.cdxml",
            opaque.as_bytes(),
            "document.open_cdxml",
            None,
        ),
    ] {
        let path = directory.path(name);
        fs::write(&path, bytes)?;
        let mut engine = Engine::default();
        engine.dispatch("ui.ring_tool", json!({"size":6}))?;
        let epoch = engine.bond_tool().epoch;
        engine.dispatch(open, json!({"path":path}))?;
        assert_eq!(engine.bond_tool().epoch, epoch);
        assert!(
            !engine
                .controls()
                .iter()
                .any(|c| c.id.starts_with("tool.ring."))
        );
        if let Some(count) = count {
            assert_eq!(engine.controls().len(), count);
            let settings = engine
                .controls()
                .into_iter()
                .find(|row| row.id == "file.document_settings")
                .ok_or("Document Settings binding")?;
            assert!(!settings.enabled);
            assert_eq!(settings.disabled_reason, Some("ui_unavailable"));
        }
        refused_tool(&mut engine, json!({"size":6}), "unsupported_content")?;
        refused_tool(&mut engine, json!({"size":7}), "unsupported_content")?;
        assert_eq!(fs::read(&path)?, bytes);
        if open == "document.open_smiles" {
            engine.dispatch("document.import_smiles_records",json!({"expected_revision":engine.state().revision,
                "selection":{"mode":"record","record_id":1},"bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,
                "page_width_pt":612,"page_height_pt":792}))?;
            assert!(engine.stored_document()?.has_smiles_projection());
            assert_eq!(engine.controls().len(), 21);
            assert!(
                !engine
                    .controls()
                    .iter()
                    .any(|c| c.id.starts_with("tool.ring."))
            );
            refused_tool(&mut engine, json!({"size":6}), "unsupported_content")?;
            assert_eq!(engine.stored_document()?.smiles_file()?.as_bytes(), bytes);
            engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
            assert_active_control(&engine, "tool.bond.double")?;
        }
    }
    Ok(())
}

// Test-only positive schema extension: returns only the preserved legacy object.
// All original size assertions following this call remain unchanged and meaningful.
fn legacy_ui_ring_tool_schema(root: &serde_json::Value) -> TestResult<serde_json::Value> {
    use serde_json::{Value, json};
    assert_eq!(root["type"], "object");
    let branches = root
        .get("anyOf")
        .or_else(|| root.get("oneOf"))
        .and_then(Value::as_array)
        .ok_or("two closed ring-tool alternatives")?;
    assert_eq!(branches.len(), 2);
    let mut legacy = None;
    let mut named = false;
    for branch in branches {
        let branch = if let Some(reference) = branch.get("$ref").and_then(Value::as_str) {
            root.pointer(reference.strip_prefix('#').ok_or("nonlocal schema")?)
                .ok_or("missing branch")?
        } else {
            branch
        };
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let fields = branch["properties"].as_object().ok_or("fields")?;
        if fields.contains_key("size") {
            assert!(legacy.is_none());
            assert_eq!(
                fields.keys().map(String::as_str).collect::<Vec<_>>(),
                vec!["size"]
            );
            legacy = Some(branch.clone());
        } else {
            assert!(!named);
            named = true;
            assert_eq!(
                fields.keys().map(String::as_str).collect::<Vec<_>>(),
                vec!["kind"]
            );
            assert_eq!(branch["required"], json!(["kind"]));
            let kind = &fields["kind"];
            let kind = if let Some(reference) = kind.get("$ref").and_then(Value::as_str) {
                root.pointer(reference.strip_prefix('#').ok_or("nonlocal kind")?)
                    .ok_or("missing kind")?
            } else {
                kind
            };
            assert_eq!(kind["type"], "string");
            assert_eq!(kind["enum"], json!(["benzene", "cyclopentadiene"]));
        }
    }
    assert!(named);
    legacy.ok_or_else(|| "missing original size object".into())
}
