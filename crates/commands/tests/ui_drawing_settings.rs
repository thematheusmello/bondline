//! Existing-JSON presentation declaration; no new Rust API is needed for baseline RED.
use bondline_commands::Engine;
use serde_json::json;
use std::sync::Arc;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn open_requests_settings_without_authoring_and_blocks_file_presentation() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":144,"begin_y_pt":144,
            "end_x_pt":192,"end_y_pt":144}),
    )?;
    engine.set_ui_file_host(true);
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let model = engine.stored_document()?.model_snapshot();
    let stamp = engine.capture_stamp();
    let view = engine.view();
    let tool = (engine.bond_tool(), engine.effective_tool());
    let result = engine.dispatch(
        "ui.drawing_settings",
        json!({"action":"open","expected_revision":1}),
    )?;
    assert!(!result.changed);
    let actual = serde_json::to_value(result)?;
    for key in [
        "document",
        "source",
        "file",
        "revision",
        "undo_depth",
        "redo_depth",
        "close",
        "close_epoch",
        "ui_file",
    ] {
        assert_eq!(actual.get(key), before.get(key), "Open changed {key}");
    }
    assert!(actual.get("file_operation").is_none());
    assert!(actual.get("ui_file_operation").is_none());
    assert_eq!(engine.content(), &content);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.view(), view);
    assert_eq!((engine.bond_tool(), engine.effective_tool()), tool);
    let error = engine
        .dispatch("ui.file", json!({"expected_revision":1,"action":"open"}))
        .err()
        .ok_or("File presentation overlapped the settings form")?;
    assert_eq!(error.info().code, "ui_busy");
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    Ok(())
}

use bondline_commands::{Command, UiDrawingSettings};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::Value;
use std::{collections::BTreeSet, fs};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the unchanged isolated-directory helper"
)]
mod support;

const NAME: &str = "ui.drawing_settings";
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const OLD_NAMES: &[&str] = &[
    "document.inspect",
    "document.new",
    "document.close",
    "ui.file",
    "ui.atom_label",
    "ui.controls",
    "ui.view",
    "ui.bond_tool",
    "document.open_cdxml",
    "document.save_cdxml",
    "document.open_cdx",
    "document.save_cdx",
    "document.open_mol",
    "document.save_mol",
    "document.open_sdf",
    "document.save_sdf",
    "document.open_smiles",
    "document.import_smiles_records",
    "document.save_smiles",
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
    "atom.set_label",
    "history.undo",
    "history.redo",
    "render.capture",
    "ui.selection",
    "selection.translate",
    "ring.create",
    "ui.ring_tool",
    "chain.create",
    "chain.create_path",
    "document.set_drawing_settings",
];
fn native_host() -> Result<Engine, Box<dyn std::error::Error>> {
    let stored = StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let mut engine = Engine::from_document(stored);
    engine.set_ui_file_host(true);
    Ok(engine)
}
fn open(engine: &mut Engine) -> Result<u64, Box<dyn std::error::Error>> {
    engine.dispatch(
        NAME,
        json!({"action":"open","expected_revision":engine.state().revision}),
    )?;
    let request = engine
        .ui_drawing_settings_request()
        .ok_or("settings request")?;
    assert!(request.current && engine.ui_drawing_settings_request_current(request.id));
    Ok(request.id)
}
fn refuse(engine: &mut Engine, input: Value, code: &str) -> TestResult {
    let state = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let error = engine
        .dispatch(NAME, input)
        .err()
        .ok_or("request unexpectedly accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    Ok(())
}
fn schema_node<'a>(
    root: &'a Value,
    value: &'a Value,
) -> Result<&'a Value, Box<dyn std::error::Error>> {
    match value.get("$ref").and_then(Value::as_str) {
        Some(reference) => Ok(root
            .pointer(reference.strip_prefix('#').ok_or("local schema")?)
            .ok_or("schema target")?),
        None => Ok(value),
    }
}

#[test]
fn public_api_and_closed_schema_add_one_presentation_without_changing_the_old_name_order()
-> TestResult {
    let rows = bondline_commands::registry();
    assert_eq!(rows.len(), 47);
    assert_eq!(
        rows[..41].iter().map(|row| row.name).collect::<Vec<_>>(),
        OLD_NAMES
    );
    assert_eq!(rows[41].name, NAME);
    assert_eq!(rows[42].name, "ui.chain_tool");
    assert_eq!(rows[43].name, "ring.sprout");
    assert_eq!(rows[44].name, "ui.preferences");
    assert!(!rows[41].read_only);
    let schema = rows[41].input_schema.as_value();
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["unevaluatedProperties"], false);
    let variants = schema["oneOf"].as_array().ok_or("closed action branches")?;
    assert_eq!(variants.len(), 2);
    let mut actions = BTreeSet::new();
    for value in variants {
        let branch = schema_node(schema, value)?;
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let fields = branch["properties"].as_object().ok_or("fields")?;
        let action = fields["action"]["const"].as_str().ok_or("action")?;
        actions.insert(action);
        let expected: BTreeSet<_> = if action == "open" {
            ["action", "expected_revision"].into_iter().collect()
        } else {
            assert_eq!(action, "cancel");
            assert_eq!(fields["request_id"]["minimum"], 1);
            ["action", "expected_revision", "request_id"]
                .into_iter()
                .collect()
        };
        assert_eq!(
            fields.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            branch["required"]
                .as_array()
                .ok_or("required")?
                .iter()
                .map(|value| value.as_str().ok_or("required name"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            expected
        );
    }
    assert_eq!(actions, BTreeSet::from(["open", "cancel"]));
    assert_eq!(
        serde_json::to_value(UiDrawingSettings::Open {
            expected_revision: 7
        })?,
        json!({"action":"open","expected_revision":7})
    );
    assert_eq!(
        serde_json::to_value(UiDrawingSettings::Cancel {
            expected_revision: 7,
            request_id: 3
        })?,
        json!({"action":"cancel","expected_revision":7,"request_id":3})
    );

    let mut engine = native_host()?;
    for input in [
        json!({}),
        json!({"action":"open"}),
        json!({"expected_revision":0}),
        json!({"action":null,"expected_revision":0}),
        json!({"action":"open","expected_revision":null}),
        json!({"action":"open","expected_revision":-1}),
        json!({"action":"open","expected_revision":0.5}),
        json!({"action":"open","expected_revision":0,"request_id":1}),
        json!({"action":"open","expected_revision":0,"bond_length_pt":24.5}),
        json!({"action":"cancel","expected_revision":0}),
        json!({"action":"cancel","expected_revision":0,"request_id":1,"extra":true}),
        json!([0, "open"]),
    ] {
        refuse(&mut engine, input, "invalid_params")?;
    }
    refuse(
        &mut engine,
        json!({"action":"open","expected_revision":1}),
        "stale_revision",
    )?;
    let controls = serde_json::to_value(engine.controls())?;
    assert_eq!(controls.as_array().ok_or("controls")?.len(), 39);
    let control = controls
        .as_array()
        .ok_or("controls")?
        .iter()
        .find(|control| control["id"] == "file.document_settings")
        .ok_or("settings binding")?;
    assert_eq!(control["menu"], "File");
    assert_eq!(control["enabled"], true);
    assert_eq!(
        control["command"],
        json!({"command":NAME,"params":{"action":"open","expected_revision":0}})
    );
    Ok(())
}

#[test]
fn apply_reuses_the_atomic_pair_once_and_validated_noop_retires_without_clearing_redo() -> TestResult
{
    let mut engine = native_host()?;
    engine.dispatch(
        "document.set_drawing_settings",
        json!({"expected_revision":0,
        "bond_length_pt":24.5,"chain_angle_degrees":null}),
    )?;
    engine.dispatch(
        "document.set_drawing_settings",
        json!({"expected_revision":1,
        "bond_length_pt":48,"chain_angle_degrees":120.5}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    let before = serde_json::to_value(engine.state())?;
    let model = engine.stored_document()?.model_snapshot();
    let stamp = engine.capture_stamp();
    let id = open(&mut engine)?;
    assert_eq!(
        serde_json::to_value(engine.ui_drawing_settings_request().ok_or("request")?)?,
        json!({"id":id,"input_revision":3,"bond_length_pt":24.5,
            "chain_angle_degrees":null,"line_width_pt":null,"bond_spacing_percent":null,"current":true})
    );
    assert!(!engine.finish_ui_drawing_settings(id + 1));
    assert!(engine.ui_drawing_settings_request_current(id));
    refuse(
        &mut engine,
        json!({"action":"open","expected_revision":3}),
        "ui_busy",
    )?;
    let state = serde_json::to_value(engine.state())?;
    for (length, angle) in [
        (Some(f64::NAN), None),
        (Some(0.0), None),
        (None, Some(180.0)),
    ] {
        let error = engine
            .apply_ui_drawing_settings(id, length, angle)
            .err()
            .ok_or("invalid Apply")?;
        assert_eq!(error.info().code, "invalid_params");
        assert_eq!(serde_json::to_value(engine.state())?, state);
        assert!(engine.ui_drawing_settings_request_current(id));
    }
    let noop = engine.apply_ui_drawing_settings(id, Some(24.5), None)?;
    assert!(!noop.changed);
    assert_eq!((noop.revision, noop.undo_depth, noop.redo_depth), (3, 1, 1));
    assert!(noop.file_operation.is_none() && noop.ui_file_operation.is_none());
    assert_eq!(
        serde_json::to_value(noop)?["ui_drawing_settings_operation"],
        id
    );
    assert!(engine.ui_drawing_settings_request().is_none());
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(
        serde_json::to_value(engine.state())?["document"],
        before["document"]
    );

    let fresh = open(&mut engine)?;
    assert!(fresh > id);
    let result = engine.apply_ui_drawing_settings(fresh, None, Some(90.25))?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (4, 2, 0)
    );
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    assert!(engine.ui_drawing_settings_request().is_none());
    assert_eq!(engine.document()?.pages(), model.pages());
    assert_eq!(engine.document()?.drawing_settings().bond_length_pt(), None);
    assert_eq!(
        engine.document()?.drawing_settings().chain_angle_degrees(),
        Some(90.25)
    );
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(
        engine.document()?.drawing_settings().bond_length_pt(),
        Some(24.5)
    );
    assert_eq!(
        engine.document()?.drawing_settings().chain_angle_degrees(),
        None
    );
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(
        engine.document()?.drawing_settings().chain_angle_degrees(),
        Some(90.25)
    );
    Ok(())
}

#[test]
fn stale_view_tool_close_and_source_aba_stays_addressable_only_for_current_revision_cancel()
-> TestResult {
    for change in ["view", "tool", "close", "source"] {
        let mut engine = native_host()?;
        let id = open(&mut engine)?;
        match change {
            "view" => {
                engine.dispatch("ui.view", json!({"action":"actual"}))?;
                engine.dispatch("ui.view", json!({"action":"fit"}))?;
            }
            "tool" => {
                engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
                engine.dispatch("ui.bond_tool", json!({"order":"single"}))?;
            }
            "close" => {
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":0,"action":"request"}),
                )?;
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":0,"action":"cancel"}),
                )?;
            }
            "source" => {
                engine.dispatch(
                    "document.set_drawing_settings",
                    json!({"expected_revision":0,
                    "bond_length_pt":30,"chain_angle_degrees":120}),
                )?;
                engine.dispatch("history.undo", json!({}))?;
            }
            _ => return Err("case".into()),
        }
        let request = engine
            .ui_drawing_settings_request()
            .ok_or("stale request kept copyable")?;
        assert_eq!(request.id, id);
        assert_eq!(
            (request.bond_length_pt, request.chain_angle_degrees),
            (None, None)
        );
        assert!(!request.current && !engine.ui_drawing_settings_request_current(id));
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        let stamp = engine.capture_stamp();
        let error = engine
            .apply_ui_drawing_settings(id, Some(24.5), Some(120.5))
            .err()
            .ok_or("stale Apply")?;
        assert_eq!(error.info().code, "ui_stale_request");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &content);
        assert!(engine.capture_stamp_matches(&stamp));
        let revision = engine.state().revision;
        engine.dispatch(
            NAME,
            json!({"action":"cancel","expected_revision":revision,"request_id":id}),
        )?;
        assert!(engine.ui_drawing_settings_request().is_none());
        assert_eq!(engine.content(), &content);
    }
    let mut engine = native_host()?;
    let id = open(&mut engine)?;
    engine.set_ui_file_host(false);
    assert!(engine.ui_drawing_settings_request().is_none());
    assert!(!engine.finish_ui_drawing_settings(id));
    engine.set_ui_file_host(true);
    assert!(open(&mut engine)? > id);
    Ok(())
}

#[test]
fn host_source_and_other_form_guards_fail_before_request_id_allocation() -> TestResult {
    let mut headless = Engine::default();
    refuse(
        &mut headless,
        json!({"action":"open","expected_revision":0}),
        "ui_unavailable",
    )?;
    headless.set_ui_file_host(true);
    assert_eq!(open(&mut headless)?, 1);

    let mut engine = native_host()?;
    engine.dispatch("ui.file", json!({"expected_revision":0,"action":"open"}))?;
    refuse(
        &mut engine,
        json!({"action":"open","expected_revision":0}),
        "ui_busy",
    )?;
    let file = engine.state().ui_file.request.ok_or("file request")?.id;
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":0,"action":"cancel","request_id":file}),
    )?;
    assert_eq!(open(&mut engine)?, 1);
    assert!(engine.finish_ui_drawing_settings(1));
    engine.dispatch(
        "ui.atom_label",
        json!({"action":"open","expected_revision":0,
        "page_id":1,"fragment_id":1,"atom_id":1}),
    )?;
    refuse(
        &mut engine,
        json!({"action":"open","expected_revision":0}),
        "ui_busy",
    )?;
    let atom = engine.ui_atom_label_request().ok_or("atom request")?.id;
    assert!(engine.finish_ui_atom_label(atom));
    let id = open(&mut engine)?;
    let before = serde_json::to_value(engine.state())?;
    let error = engine
        .dispatch(
            "ui.atom_label",
            json!({"action":"open","expected_revision":0,
        "page_id":1,"fragment_id":1,"atom_id":1}),
        )
        .err()
        .ok_or("Atom/settings overlap")?;
    assert_eq!(error.info().code, "ui_busy");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.finish_ui_drawing_settings(id));

    let directory = support::FileTestDirectory::new()?;
    for (name, bytes, command, controls) in [
        (
            "old.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
            22,
        ),
        (
            "old.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
            22,
        ),
        (
            "old.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
            16,
        ),
        (
            "old.smi",
            b"CC opaque\0\xff\r\n".as_slice(),
            "document.open_smiles",
            12,
        ),
    ] {
        let path = directory.path(name);
        fs::write(&path, bytes)?;
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch(command, json!({"path":path}))?;
        let before = fs::read(&path)?;
        refuse(
            &mut engine,
            json!({"action":"open","expected_revision":1}),
            "unsupported_content",
        )?;
        assert_eq!(engine.controls().len(), controls);
        let preferences = engine
            .controls()
            .into_iter()
            .find(|row| row.id == "file.preferences")
            .ok_or("Preferences remains available for every source family")?;
        assert!(preferences.enabled);
        assert_eq!(preferences.disabled_reason, None);
        assert!(preferences.shortcut.is_none());
        assert_eq!(
            serde_json::to_value(preferences.command)?,
            json!({"command":"ui.preferences",
            "params":{"action":"open","expected_revision":1}})
        );
        assert_eq!(fs::read(&path)?, before);
        engine.dispatch("document.new", json!({"expected_revision":1}))?;
        assert_eq!(open(&mut engine)?, 1);
    }
    let foreign =
        String::from_utf8(NATIVE.to_vec())?.replace("<CDXML ", "<CDXML Private=\"retain\" ");
    let mut engine = Engine::from_document(StoredDocument::from_cdxml(
        foreign.as_bytes().into(),
        PageSize::A4,
    )?);
    engine.set_ui_file_host(true);
    refuse(
        &mut engine,
        json!({"action":"open","expected_revision":0}),
        "unsupported_content",
    )?;
    assert_eq!(
        engine.stored_document()?.export_cdxml()?,
        foreign.as_bytes()
    );
    Ok(())
}

#[test]
fn actual_save_association_aba_latches_stale_and_captured_save_cannot_clean_ui_applied_settings()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let a = directory.path("settings-原-λ.cdxml");
    let b = directory.path("settings-other-化学.cdxml");
    fs::write(&a, NATIVE)?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_cdxml", json!({"path":a}))?;
    let id = open(&mut engine)?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    engine.dispatch("document.save_cdxml", json!({"path":b}))?;
    engine.dispatch("document.save_cdxml", json!({"path":a,"overwrite":true}))?;
    assert_eq!(engine.state().revision, 1);
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(fs::read(&a)?, NATIVE);
    assert_eq!(fs::read(&b)?, NATIVE);
    assert_eq!(
        engine.state().file.path.as_deref(),
        fs::canonicalize(&a)?.to_str()
    );
    assert!(!engine.ui_drawing_settings_request_current(id));
    let before = serde_json::to_value(engine.state())?;
    let error = engine
        .apply_ui_drawing_settings(id, Some(24.5), Some(120.5))
        .err()
        .ok_or("file ABA Apply")?;
    assert_eq!(error.info().code, "ui_stale_request");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.finish_ui_drawing_settings(id));

    let fresh = open(&mut engine)?;
    let output = directory.path("captured-before-設定.cdxml");
    let unused = directory.path("unused.cdxml");
    let pending = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":output}),
    )?)?;
    let changed = engine.apply_ui_drawing_settings(fresh, Some(24.5), Some(120.5))?;
    assert!(changed.changed);
    assert!(engine.ui_drawing_settings_request().is_none());
    let error = engine
        .prepare(Command::parse(
            "document.save_cdxml",
            json!({"path":unused}),
        )?)
        .err()
        .ok_or("lost save lease")?;
    assert_eq!(error.info().code, "busy");
    let completed = pending.complete()?;
    assert_eq!(fs::read(&output)?, NATIVE);
    let result = engine.finish(completed)?;
    assert_eq!(result.revision, 2);
    assert!(result.file.dirty);
    let receipt = result.file_operation.ok_or("actual Save receipt")?;
    assert_eq!(receipt.input_revision, 1);
    assert!(receipt.ui_request_id.is_none() && receipt.ui_request_current.is_none());
    assert_eq!(
        engine.document()?.drawing_settings().bond_length_pt(),
        Some(24.5)
    );
    assert_eq!(
        engine.document()?.drawing_settings().chain_angle_degrees(),
        Some(120.5)
    );
    assert_eq!(fs::read(&a)?, NATIVE);
    assert!(!unused.exists());
    Ok(())
}
