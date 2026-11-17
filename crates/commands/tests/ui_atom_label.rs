//! Independent native atom-label presentation through the existing JSON dispatcher.
use bondline_commands::Engine;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn open_atom_label_request_captures_an_exact_existing_target_without_authoring() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72.0,
        "begin_y_pt":72.0,"end_x_pt":120.0,"end_y_pt":72.0}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.capture_stamp();
    // Baseline must reach actual dispatch and fail UnknownCommand, not a missing Rust API.
    let result = engine.dispatch(
        "ui.atom_label",
        json!({"action":"open",
        "expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":2}),
    )?;
    let mut value = serde_json::to_value(result)?;
    let id = value["ui_atom_label_operation"]
        .as_u64()
        .ok_or("request ID")?;
    assert!(id > 0);
    assert_eq!(
        value["ui_atom_label"],
        json!({"id":id,"input_revision":1,
        "page_id":1,"fragment_id":1,"atom_id":2,"element":6,
        "explicit_hydrogens":3,"current":true})
    );
    value
        .as_object_mut()
        .ok_or("state")?
        .remove("ui_atom_label");
    value
        .as_object_mut()
        .ok_or("state")?
        .remove("ui_atom_label_operation");
    assert_eq!(value, before);
    assert!(engine.capture_stamp_matches(&owner));
    Ok(())
}

// Additive acceptance keeps the frozen1,572-byte real-dispatch declaration exact.
use bondline_commands::{CloseState, Command};
use bondline_io::StoredDocument;
use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId, PageSize, Radical};
use serde_json::Value;
use std::{fs, num::NonZeroU16, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated test directory only")]
mod support;

const NAME: &str = "ui.atom_label";
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
fn pair() -> TestResult<Engine> {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72.0,
        "begin_y_pt":72.0,"end_x_pt":120.0,"end_y_pt":72.0}),
    )?;
    engine.set_ui_file_host(true);
    Ok(engine)
}
fn open_params(revision: u64) -> Value {
    json!({"action":"open","expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":2})
}
fn current_open(engine: &mut Engine) -> TestResult<(u64, Value)> {
    let result = engine.dispatch(NAME, open_params(engine.state().revision))?;
    let value = serde_json::to_value(result)?;
    let id = value["ui_atom_label_operation"]
        .as_u64()
        .ok_or("request ID")?;
    assert!(id > 0);
    assert_eq!(value["ui_atom_label"]["id"], id);
    assert_eq!(value["ui_atom_label"]["current"], true);
    assert_eq!(value["changed"], false);
    Ok((id, value))
}
fn without_request(mut value: Value) -> TestResult<Value> {
    let object = value.as_object_mut().ok_or("state")?;
    object.remove("ui_atom_label");
    object.remove("ui_atom_label_operation");
    Ok(value)
}
fn fail(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let owner = engine.capture_stamp();
    let file_context = engine.ui_file_context_stamp();
    let model = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let view = engine.view();
    let tool = engine.bond_tool();
    let error = engine
        .dispatch(NAME, params)
        .err()
        .ok_or("invalid presentation accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), owner);
    assert!(engine.ui_file_context_stamp_matches(&file_context));
    assert_eq!(engine.view(), view);
    assert_eq!(engine.bond_tool(), tool);
    if let Some(model) = model {
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    Ok(())
}
fn cancel(engine: &mut Engine, id: u64) -> TestResult {
    let before = without_request(serde_json::to_value(engine.state())?)?;
    let owner = engine.capture_stamp();
    let result = engine.dispatch(
        NAME,
        json!({"action":"cancel",
        "expected_revision":engine.state().revision,"request_id":id}),
    )?;
    let value = serde_json::to_value(result)?;
    assert_eq!(value["ui_atom_label_operation"], id);
    assert!(value.get("ui_atom_label").is_none());
    assert_eq!(without_request(value)?, before);
    assert!(engine.capture_stamp_matches(&owner));
    Ok(())
}
fn seeded(atom: Atom) -> TestResult<Engine> {
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(9).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    let mut engine = Engine::from_document(stored);
    engine.set_ui_file_host(true);
    Ok(engine)
}

#[test]
fn closed_tagged_schema_host_and_invalid_target_errors_are_non_authoring() -> TestResult {
    let registry = bondline_commands::registry();
    let names = registry
        .iter()
        .map(|v| v.name)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(registry.len(), 43);
    assert_eq!(names.len(), 43);
    for old in [
        "document.inspect",
        "document.new",
        "document.close",
        "ui.file",
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
    ] {
        assert!(names.contains(old), "old command {old}");
    }
    let entry = registry
        .iter()
        .find(|v| v.name == NAME)
        .ok_or("presentation command")?;
    assert!(!entry.read_only);
    let schema = serde_json::to_value(&entry.input_schema)?;
    assert_eq!(schema["type"], "object");
    let variants = schema
        .get("oneOf")
        .or_else(|| schema.get("anyOf"))
        .and_then(Value::as_array)
        .ok_or("tagged variants")?;
    assert_eq!(variants.len(), 2);
    let mut seen = std::collections::BTreeSet::new();
    for variant in variants {
        let action = &variant["properties"]["action"];
        let tag = action
            .get("const")
            .and_then(Value::as_str)
            .or_else(|| action.get("enum")?.as_array()?.first()?.as_str())
            .ok_or("action tag")?;
        assert!(seen.insert(tag));
        let fields = match tag {
            "open" => vec![
                "action",
                "expected_revision",
                "page_id",
                "fragment_id",
                "atom_id",
            ],
            "cancel" => vec!["action", "expected_revision", "request_id"],
            _ => return Err("unknown tagged action".into()),
        }
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
        let properties = variant["properties"]
            .as_object()
            .ok_or("variant properties")?;
        assert_eq!(
            properties
                .keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>(),
            fields
        );
        assert_eq!(
            variant["required"]
                .as_array()
                .ok_or("required")?
                .iter()
                .map(|v| v.as_str().ok_or("required field"))
                .collect::<Result<std::collections::BTreeSet<_>, _>>()?,
            fields
        );
        assert!(
            variant["additionalProperties"] == false
                || schema["additionalProperties"] == false
                || schema["unevaluatedProperties"] == false
        );
    }
    assert_eq!(seen, std::collections::BTreeSet::from(["open", "cancel"]));
    let mut engine = pair()?;
    engine.set_ui_file_host(false);
    fail(&mut engine, open_params(1), "ui_unavailable")?;
    fail(&mut engine, open_params(0), "stale_revision")?;
    engine.set_ui_file_host(true);
    for (key, value, code) in [
        ("page_id", json!(0), "invalid_params"),
        ("fragment_id", json!(0), "invalid_params"),
        ("atom_id", json!(0), "invalid_params"),
        ("page_id", json!(99), "not_found"),
        ("fragment_id", json!(99), "not_found"),
        ("atom_id", json!(99), "not_found"),
    ] {
        let mut params = open_params(1);
        params[key] = value;
        fail(&mut engine, params, code)?;
    }
    let mut stale = open_params(0);
    stale["atom_id"] = json!(0);
    fail(&mut engine, stale, "stale_revision")?;
    for params in [
        json!({"action":"open","expected_revision":1,"page_id":1,"fragment_id":1}),
        json!({"action":"open","expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":2,"request_id":1}),
        json!({"action":"cancel","expected_revision":1}),
        json!({"action":"cancel","expected_revision":1,"request_id":1,"atom_id":2}),
        json!({"action":"inspect","expected_revision":1}),
        json!({"expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":2}),
    ] {
        assert_eq!(
            Command::parse(NAME, params.clone())
                .err()
                .ok_or("closed parse")?
                .info()
                .code,
            "invalid_params"
        );
        fail(&mut engine, params, "invalid_params")?;
    }
    fail(
        &mut engine,
        json!({"action":"cancel","expected_revision":1,"request_id":99}),
        "ui_stale_request",
    )?;
    assert!(
        !engine
            .controls()
            .iter()
            .any(|control| control.command.name() == NAME),
        "no executable empty-target native control"
    );
    Ok(())
}

#[test]
fn exact_optional_h_and_target_metadata_are_preserved_and_foreign_sources_never_gain_authority()
-> TestResult {
    for h in [None, Some(0), Some(65535)] {
        let mut atom = Atom::new(
            AtomId::new(17).ok_or("atom")?,
            Element::new(8).ok_or("oxygen")?,
            None,
        );
        atom.explicit_hydrogens = h;
        let mut engine = seeded(atom)?;
        let before = serde_json::to_value(engine.state())?;
        let owner = engine.capture_stamp();
        let result = engine.dispatch(
            NAME,
            json!({"action":"open","expected_revision":0,
            "page_id":1,"fragment_id":9,"atom_id":17}),
        )?;
        let value = serde_json::to_value(result)?;
        assert_eq!(value["ui_atom_label"]["explicit_hydrogens"], json!(h));
        assert_eq!(value["ui_atom_label"]["element"], 8);
        assert_eq!(without_request(value.clone())?, before);
        assert!(engine.capture_stamp_matches(&owner));
        cancel(
            &mut engine,
            value["ui_atom_label"]["id"].as_u64().ok_or("request ID")?,
        )?;
    }
    for kind in ["charge", "isotope", "radical"] {
        let mut atom = Atom::new(
            AtomId::new(17).ok_or("atom")?,
            Element::new(8).ok_or("oxygen")?,
            None,
        );
        match kind {
            "charge" => atom.charge = -1,
            "isotope" => atom.isotope = NonZeroU16::new(18),
            _ => atom.radical = Radical::Doublet,
        }
        let mut engine = seeded(atom)?;
        fail(
            &mut engine,
            json!({"action":"open","expected_revision":0,
            "page_id":1,"fragment_id":9,"atom_id":17}),
            "unsupported_content",
        )?;
    }
    let mut native =
        Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    native.set_ui_file_host(true);
    assert!(native.stored_document()?.is_fully_mapped());
    let native_bytes = native.stored_document()?.export_cdxml()?.to_vec();
    let (id, _) = current_open(&mut native)?;
    assert_eq!(native.stored_document()?.export_cdxml()?, native_bytes);
    cancel(&mut native, id)?;
    let opaque = String::from_utf8(NATIVE.to_vec())?
        .replace("<CDXML ", "<CDXML Private=\"original-preserved\" ");
    let mut foreign = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    foreign.set_ui_file_host(true);
    fail(&mut foreign, open_params(0), "unsupported_content")?;
    assert_eq!(
        foreign.stored_document()?.export_cdxml()?,
        opaque.as_bytes()
    );
    let directory = support::FileTestDirectory::new()?;
    for (name, bytes, command) in [
        (
            "original.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "original.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
        ),
        (
            "original.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        (
            "original.smi",
            b"CC chosen\r\n[13CH4]\t\0\xffopaque".as_slice(),
            "document.open_smiles",
        ),
    ] {
        let path = directory.path(name);
        fs::write(&path, bytes)?;
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch(command, json!({"path":path}))?;
        fail(&mut engine, open_params(1), "unsupported_content")?;
        fail(&mut engine, open_params(0), "stale_revision")?;
        assert_eq!(fs::read(&path)?, bytes);
        if command == "document.open_smiles" {
            engine.dispatch(
                "document.import_smiles_records",
                json!({"expected_revision":1,
                "selection":{"mode":"record","record_id":1},"bond_length_pt":48,
                "origin_x_pt":72,"origin_y_pt":72,"page_width_pt":612,"page_height_pt":792}),
            )?;
            assert!(engine.stored_document()?.has_smiles_projection());
            fail(&mut engine, open_params(2), "unsupported_content")?;
            assert_eq!(engine.stored_document()?.smiles_file()?.as_bytes(), bytes);
        }
    }
    Ok(())
}

#[test]
fn source_revision_file_view_tool_and_close_aba_stale_but_never_retarget_the_request() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    for change in [
        "authored",
        "history",
        "source",
        "view",
        "tool",
        "file_association",
        "file_request",
        "close",
    ] {
        let mut engine = pair()?;
        let a = directory.path(&format!("{change}-A.cdxml"));
        let b = directory.path(&format!("{change}-B.cdxml"));
        engine.dispatch("document.save_cdxml", json!({"path":a}))?;
        let saved = fs::read(&a)?;
        let (id, opened) = current_open(&mut engine)?;
        let initial = opened["ui_atom_label"].clone();
        let owner = engine.capture_stamp();
        let initial_view = engine.view();
        let initial_tool = engine.bond_tool();
        match change {
            "authored" | "history" => {
                engine.dispatch(
                    "atom.set_label",
                    json!({"expected_revision":1,"page_id":1,
                    "fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}),
                )?;
                if change == "history" {
                    engine.dispatch("history.undo", json!({}))?;
                }
            }
            "source" => {
                engine.dispatch("document.open_cdxml", json!({"path":a}))?;
            }
            "view" => {
                engine.dispatch("ui.view", json!({"action":"set_zoom","percent":125}))?;
                engine.dispatch("ui.view", json!({"action":"fit"}))?;
                assert_eq!(engine.view().zoom_percent, initial_view.zoom_percent);
                assert_eq!(engine.view().center, initial_view.center);
                assert!(engine.view().epoch > initial_view.epoch);
            }
            "tool" => {
                engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
                engine.dispatch("ui.bond_tool", json!({"order":"single"}))?;
                assert_eq!(engine.bond_tool().order, initial_tool.order);
                assert!(engine.bond_tool().epoch > initial_tool.epoch);
            }
            "file_association" => {
                engine.dispatch("document.save_cdxml", json!({"path":b}))?;
                engine.dispatch("document.save_cdxml", json!({"path":a,"overwrite":true}))?;
                assert_eq!(
                    engine.state().file.path,
                    Some(fs::canonicalize(&a)?.to_string_lossy().into_owned())
                );
                assert_eq!(fs::read(&b)?, saved);
            }
            "file_request" => {
                let request =
                    engine.dispatch("ui.file", json!({"expected_revision":1,"action":"save"}))?;
                let ticket = request.ui_file_operation.ok_or("file request")?;
                engine.dispatch(
                    "ui.file",
                    json!({"expected_revision":1,"action":"cancel","request_id":ticket}),
                )?;
                assert!(engine.state().ui_file.request.is_none());
            }
            _ => {
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":1,"action":"request"}),
                )?;
                assert_eq!(engine.state().close, CloseState::Approved);
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":1,"action":"cancel"}),
                )?;
                assert_eq!(engine.state().close, CloseState::Open);
            }
        }
        let state = serde_json::to_value(engine.state())?;
        let mut expected = initial;
        expected["current"] = json!(false);
        assert_eq!(state["ui_atom_label"], expected, "context {change}");
        if !matches!(change, "authored" | "history" | "source") {
            assert!(engine.capture_stamp_matches(&owner));
        } else {
            assert!(!engine.capture_stamp_matches(&owner));
        }
        let revision = engine.state().revision;
        fail(&mut engine, open_params(revision), "ui_busy")?;
        fail(
            &mut engine,
            json!({"action":"cancel","expected_revision":revision,"request_id":id+1}),
            "ui_stale_request",
        )?;
        cancel(&mut engine, id)?;
        assert!(
            serde_json::to_value(engine.state())?
                .get("ui_atom_label")
                .is_none()
        );
        assert_eq!(fs::read(&a)?, saved);
    }
    Ok(())
}

#[test]
fn close_and_native_file_presentation_block_open_without_consuming_or_mutating_any_request()
-> TestResult {
    let mut engine = pair()?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    fail(&mut engine, open_params(1), "ui_busy")?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"discard"}),
    )?;
    assert_eq!(engine.state().close, CloseState::Approved);
    fail(&mut engine, open_params(1), "ui_busy")?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"cancel"}),
    )?;
    let file = engine.dispatch("ui.file", json!({"expected_revision":1,"action":"open"}))?;
    let ticket = file.ui_file_operation.ok_or("file request")?;
    fail(&mut engine, open_params(1), "ui_busy")?;
    engine.claim_ui_file(ticket)?;
    fail(&mut engine, open_params(1), "ui_busy")?;
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":1,"action":"cancel","request_id":ticket}),
    )?;
    let (id, _) = current_open(&mut engine)?;
    fail(&mut engine, open_params(1), "ui_busy")?;
    cancel(&mut engine, id)
}

#[test]
fn host_detachment_monotonic_ids_and_committed_close_retirement_never_acknowledge_authoring()
-> TestResult {
    let mut engine = pair()?;
    let owner = engine.capture_stamp();
    let authored = without_request(serde_json::to_value(engine.state())?)?;
    let (first, _) = current_open(&mut engine)?;
    let before = serde_json::to_value(engine.state())?;
    assert!(!engine.finish_ui_atom_label(first + 1));
    assert_eq!(serde_json::to_value(engine.state())?, before);
    engine.set_ui_file_host(false);
    assert!(
        serde_json::to_value(engine.state())?
            .get("ui_atom_label")
            .is_none()
    );
    assert!(!engine.state().ui_file.available);
    assert!(engine.capture_stamp_matches(&owner));
    engine.set_ui_file_host(true);
    assert!(
        serde_json::to_value(engine.state())?
            .get("ui_atom_label")
            .is_none()
    );
    let (second, _) = current_open(&mut engine)?;
    assert!(second > first);
    fail(
        &mut engine,
        json!({"action":"cancel","expected_revision":1,"request_id":first}),
        "ui_stale_request",
    )?;
    assert!(engine.finish_ui_atom_label(second));
    assert!(!engine.finish_ui_atom_label(second));
    assert_eq!(
        without_request(serde_json::to_value(engine.state())?)?,
        authored
    );
    let (third, _) = current_open(&mut engine)?;
    assert!(third > second);
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"request"}),
    )?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"discard"}),
    )?;
    assert!(engine.complete_close());
    assert_eq!(engine.state().close, CloseState::Committed);
    fail(
        &mut engine,
        json!({"action":"cancel","expected_revision":1,"request_id":third}),
        "close_committed",
    )?;
    fail(&mut engine, open_params(0), "close_committed")?;
    let before = without_request(serde_json::to_value(engine.state())?)?;
    assert!(!engine.finish_ui_atom_label(third + 1));
    assert!(engine.finish_ui_atom_label(third));
    assert_eq!(
        without_request(serde_json::to_value(engine.state())?)?,
        before
    );
    assert!(engine.capture_stamp_matches(&owner));
    assert_eq!(engine.state().close, CloseState::Committed);
    assert!(engine.state().ui_file.request.is_none());
    assert!(!engine.finish_ui_file(third, false));
    Ok(())
}
