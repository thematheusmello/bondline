//! Independent future-API acceptance; authored after Root began the producer.
//! Missing APIs are compile qualification, not an existing-JSON behavior RED.
use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const TOOL: &str = "ui.chain_tool";
const COORDINATE_EPSILON_PT: f64 = 1e-10;

fn native_host() -> TestResult<Engine> {
    let stored = StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let mut engine = Engine::from_document(stored);
    engine.set_ui_file_host(true);
    engine.dispatch(TOOL, json!({"action":"select","kind":"acyclic"}))?;
    Ok(engine)
}
fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= COORDINATE_EPSILON_PT,
        "coordinate {actual} differs from independent {expected}"
    );
}
fn open_add(engine: &mut Engine, target: serde_json::Value) -> TestResult<u64> {
    let before = engine.content().clone();
    let stamp = engine.capture_stamp();
    let result = engine.dispatch(
        TOOL,
        json!({"action":"open_add",
        "expected_revision":engine.state().revision,"page_id":1,"target":target}),
    )?;
    assert!(!result.changed);
    assert_eq!(engine.content(), &before);
    assert!(engine.capture_stamp_matches(&stamp));
    let request = engine.ui_add_chain_request().ok_or("Add request")?;
    assert!(request.current && engine.ui_add_chain_request_current(request.id));
    assert!(!request.was_cancelled());
    Ok(request.id)
}

#[test]
fn add_latches_optional_preferences_and_one_complete_native_edit_with_exact_history() -> TestResult
{
    let mut engine = native_host()?;
    let original = engine.content().clone();
    let source = engine.capture_stamp();
    let preferences = engine.chain_preferences()?;
    assert_eq!(
        (preferences.bond_length_pt, preferences.angle_degrees),
        (48.0, 120.0)
    );
    assert!(preferences.fixed_lengths && preferences.fixed_angles);
    assert!(engine.document()?.drawing_settings().is_empty());
    assert!(engine.capture_stamp_matches(&source));
    engine.dispatch(
        TOOL,
        json!({"action":"constraints","fixed_lengths":false,"fixed_angles":false}),
    )?;
    engine.dispatch(
        "document.set_drawing_settings",
        json!({"expected_revision":0,
        "bond_length_pt":24.5,"chain_angle_degrees":null}),
    )?;
    let preferences = engine.chain_preferences()?;
    assert_eq!(
        (preferences.bond_length_pt, preferences.angle_degrees),
        (24.5, 120.0)
    );
    assert!(!preferences.fixed_lengths && !preferences.fixed_angles);
    assert_eq!(
        engine.document()?.drawing_settings().chain_angle_degrees(),
        None
    );
    let before_add = engine.content().clone();
    let old_fragment = Arc::clone(&engine.document()?.pages()[0].fragments()[0]);
    let page_size = engine.document()?.pages()[0].size();
    let target = json!({"mode":"free","x_pt":200.0,"y_pt":300.0});
    let id = open_add(&mut engine, target.clone())?;
    assert_eq!(
        serde_json::to_value(engine.ui_add_chain_request().ok_or("request")?)?,
        json!({"id":id,"input_revision":1,"page_id":1,"target":target,"count":6,
            "bond_length_pt":24.5,"angle_degrees":120.0,"current":true})
    );
    let unchanged = serde_json::to_value(engine.state())?;
    let candidates = engine.ui_add_chain_candidates(id, 6)?;
    assert_eq!(
        candidates.len(),
        1,
        "free numeric pose has one upper-first candidate"
    );
    assert_eq!(
        serde_json::to_value(&candidates)?,
        serde_json::to_value(engine.ui_add_chain_candidates(id, 6)?)?
    );
    let first = candidates.first().ok_or("numeric candidate")?;
    assert_eq!(
        (
            first.expected_revision,
            first.page_id,
            first.count,
            first.angle_degrees
        ),
        (1, 1, 6, 120.0)
    );
    let q = 24.5 * 3.0_f64.sqrt() / 2.0;
    let p = serde_json::to_value(&first.placement)?;
    assert_eq!(p["mode"], "free");
    assert_eq!(p["side"], "positive");
    near(p["begin_x_pt"].as_f64().ok_or("begin x")?, 200.0);
    near(p["begin_y_pt"].as_f64().ok_or("begin y")?, 300.0);
    near(p["end_x_pt"].as_f64().ok_or("end x")?, 200.0 + q);
    near(p["end_y_pt"].as_f64().ok_or("end y")?, 300.0 - 12.25);
    let preview = engine.preview_chain_create(first)?;
    assert_eq!(serde_json::to_value(engine.state())?, unchanged);
    let result = engine.apply_ui_add_chain(id, 6, 0)?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 2, 0)
    );
    assert!(
        result.file_operation.is_none()
            && result.ui_file_operation.is_none()
            && result.close_operation.is_none()
    );
    assert!(engine.ui_add_chain_request().is_none());
    assert_eq!(engine.document()?, preview.model());
    let page = &engine.document()?.pages()[0];
    assert_eq!(page.size(), page_size);
    assert_eq!(page.fragments().len(), 2);
    assert!(Arc::ptr_eq(&old_fragment, &page.fragments()[0]));
    let added = &page.fragments()[1];
    assert_eq!(added.id().get(), 2);
    assert_eq!((added.atoms().len(), added.bonds().len()), (6, 5));
    for (i, atom) in added.atoms().iter().enumerate() {
        assert_eq!(atom.id.get(), 3 + i as u32);
        assert_eq!(atom.element.get(), 6);
        assert!(atom.isotope.is_none());
        assert_eq!(atom.charge, 0);
        assert_eq!(serde_json::to_value(atom.radical)?, json!("none"));
        assert_eq!(
            atom.explicit_hydrogens,
            Some(if i == 0 || i == 5 { 3 } else { 2 })
        );
        assert_eq!(atom.show_terminal_carbon_labels, Some(false));
        assert_eq!(atom.show_non_terminal_carbon_labels, Some(false));
        let position = atom.position.ok_or("new authored position")?;
        near(position.x_pt(), 200.0 + i as f64 * q);
        near(
            position.y_pt(),
            300.0 - if i % 2 == 0 { 0.0 } else { 12.25 },
        );
    }
    for (i, bond) in added.bonds().iter().enumerate() {
        assert_eq!(
            serde_json::to_value(bond)?,
            json!({"id":2+i as u32,
            "begin":3+i as u32,"end":4+i as u32,"order":"single"})
        );
    }
    let after = engine.content().clone();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &before_add);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    engine.dispatch("history.redo", json!({}))?;
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &after);
    Ok(())
}

// Further post-producer acceptance. No initial declaration bytes are changed.
use bondline_commands::{ChainTarget, Command, Error};
use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId, Point2};
use std::{fs, sync::Mutex};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the unchanged isolated test-directory owner"
)]
mod support;

fn free() -> serde_json::Value {
    json!({"mode":"free","x_pt":200.0,"y_pt":400.0})
}
fn without_add(mut value: serde_json::Value) -> TestResult<serde_json::Value> {
    let object = value.as_object_mut().ok_or("state")?;
    object.remove("ui_add_chain");
    object.remove("ui_add_chain_operation");
    Ok(value)
}
fn api_error<T>(
    engine: &mut Engine,
    code: &str,
    operation: impl FnOnce(&mut Engine) -> Result<T, Error>,
) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let tool = (engine.bond_tool(), engine.effective_tool());
    let view = engine.view();
    let error = operation(engine)
        .err()
        .ok_or("API unexpectedly succeeded")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!((engine.bond_tool(), engine.effective_tool()), tool);
    assert_eq!(engine.view(), view);
    Ok(())
}
fn dispatch_error(
    engine: &mut Engine,
    name: &str,
    input: serde_json::Value,
    code: &str,
) -> TestResult {
    api_error(engine, code, |engine| engine.dispatch(name, input))
}
fn cancel_add(engine: &mut Engine, id: u64) -> TestResult {
    let before = without_add(serde_json::to_value(engine.state())?)?;
    let result = engine.dispatch(
        TOOL,
        json!({"action":"cancel_add",
        "expected_revision":engine.state().revision,"request_id":id}),
    )?;
    assert!(!result.changed);
    assert_eq!(without_add(serde_json::to_value(result)?)?, before);
    assert!(engine.ui_add_chain_request().is_none());
    Ok(())
}

#[test]
fn contexts_never_revive_after_any_form_ownership_or_view_tool_close_source_aba() -> TestResult {
    for form in ["add_cancel", "add_finish", "atom", "settings"] {
        let mut engine = native_host()?;
        let context = engine.chain_context(1)?;
        assert!(context.is_current(&engine));
        let before = serde_json::to_value(engine.state())?;
        let stamp = engine.capture_stamp();
        let tool = engine.bond_tool();
        let view = engine.view();
        match form {
            "add_cancel" | "add_finish" => {
                let id = open_add(&mut engine, free())?;
                let request = engine.ui_add_chain_request().ok_or("owned form")?;
                assert!(!context.is_current(&engine));
                assert!(!engine.ui_add_chain_candidates(id, 6)?.is_empty());
                if form == "add_cancel" {
                    cancel_add(&mut engine, id)?;
                    assert!(request.was_cancelled());
                } else {
                    assert!(engine.finish_ui_add_chain(id));
                    assert!(!request.was_cancelled());
                }
            }
            "atom" => {
                engine.dispatch(
                    "ui.atom_label",
                    json!({"action":"open","expected_revision":0,
                    "page_id":1,"fragment_id":1,"atom_id":1}),
                )?;
                assert!(!context.is_current(&engine));
                let id = engine.ui_atom_label_request().ok_or("atom owner")?.id;
                engine.dispatch(
                    "ui.atom_label",
                    json!({"action":"cancel","expected_revision":0,"request_id":id}),
                )?;
            }
            "settings" => {
                engine.dispatch(
                    "ui.drawing_settings",
                    json!({"action":"open","expected_revision":0}),
                )?;
                assert!(!context.is_current(&engine));
                let id = engine
                    .ui_drawing_settings_request()
                    .ok_or("settings owner")?
                    .id;
                engine.dispatch(
                    "ui.drawing_settings",
                    json!({"action":"cancel","expected_revision":0,"request_id":id}),
                )?;
            }
            _ => return Err("form case".into()),
        }
        assert!(
            !context.is_current(&engine),
            "{form} returned to idle but must remain latched stale"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(engine.capture_stamp_matches(&stamp));
        assert_eq!(engine.bond_tool(), tool);
        assert_eq!(engine.view(), view);
        assert!(engine.chain_context(1)?.is_current(&engine));
    }
    for change in ["view", "tool", "close", "source", "foreign"] {
        let mut engine = native_host()?;
        let idle = engine.chain_context(1)?;
        if change == "view" {
            engine.dispatch("ui.view", json!({"action":"actual"}))?;
            engine.dispatch("ui.view", json!({"action":"fit"}))?;
        } else if change == "tool" {
            engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
            engine.dispatch(TOOL, json!({"action":"select","kind":"acyclic"}))?;
        } else if change == "close" {
            engine.dispatch(
                "document.close",
                json!({"expected_revision":0,"action":"request"}),
            )?;
            engine.dispatch(
                "document.close",
                json!({"expected_revision":0,"action":"cancel"}),
            )?;
        } else if change == "source" {
            engine.dispatch(
                "document.set_drawing_settings",
                json!({"expected_revision":0,
                "bond_length_pt":30.0,"chain_angle_degrees":120.0}),
            )?;
            engine.dispatch("history.undo", json!({}))?;
        } else {
            engine.dispatch(
                "document.open_mol",
                json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),
                "/../io/tests/fixtures/mol/ethane-v2000.mol")}),
            )?;
        }
        assert!(!idle.is_current(&engine), "{change}");
    }
    // A stale form remains addressable and explicit Cancel works with latest revision,
    // including changed mode/source. Error assertions compare the LATEST content.
    for change in ["tool", "view", "close", "source", "foreign"] {
        let mut engine = native_host()?;
        let id = open_add(&mut engine, free())?;
        let request = engine.ui_add_chain_request().ok_or("form")?;
        let mut metadata = serde_json::to_value(&request)?;
        metadata["current"] = json!(false);
        match change {
            "tool" => {
                engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
            }
            "view" => {
                engine.dispatch("ui.view", json!({"action":"actual"}))?;
                engine.dispatch("ui.view", json!({"action":"fit"}))?;
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
                    "bond_length_pt":30.0,"chain_angle_degrees":120.0}),
                )?;
                engine.dispatch("history.undo", json!({}))?;
            }
            "foreign" => {
                engine.dispatch(
                    "document.open_mol",
                    json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),
                "/../io/tests/fixtures/mol/ethane-v2000.mol")}),
                )?;
            }
            _ => return Err("stale form case".into()),
        }
        assert_eq!(
            serde_json::to_value(engine.ui_add_chain_request().ok_or("copyable stale form")?)?,
            metadata
        );
        assert!(!engine.ui_add_chain_request_current(id));
        api_error(&mut engine, "ui_stale_request", |engine| {
            engine.ui_add_chain_candidates(id, 6)
        })?;
        api_error(&mut engine, "ui_stale_request", |engine| {
            engine.apply_ui_add_chain(id, 6, 0)
        })?;
        assert!(!request.was_cancelled());
        cancel_add(&mut engine, id)?;
        assert!(request.was_cancelled());
    }
    Ok(())
}

#[test]
fn strict_host_source_targets_counts_index_and_foreground_guards_preserve_the_request() -> TestResult
{
    let stored = StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?;
    let mut engine = Engine::from_document(stored);
    engine.dispatch(TOOL, json!({"action":"select","kind":"acyclic"}))?;
    api_error(&mut engine, "ui_unavailable", |engine| {
        engine.chain_context(1)
    })?;
    dispatch_error(
        &mut engine,
        TOOL,
        json!({"action":"open_add","expected_revision":1,"page_id":1,"target":free()}),
        "stale_revision",
    )?;
    dispatch_error(
        &mut engine,
        TOOL,
        json!({"action":"open_add","expected_revision":0,"page_id":1,"target":free()}),
        "ui_unavailable",
    )?;
    engine.set_ui_file_host(true);
    for target in [
        json!({"mode":"free","x_pt":100001.0,"y_pt":300.0}),
        json!({"mode":"free","x_pt":200.0,"y_pt":300.0,"atom_id":1}),
        json!({"mode":"atom","fragment_id":1,"atom_id":999}),
    ] {
        let code = if target["mode"] == "atom" {
            "not_found"
        } else {
            "invalid_params"
        };
        dispatch_error(
            &mut engine,
            TOOL,
            json!({"action":"open_add","expected_revision":0,"page_id":1,"target":target}),
            code,
        )?;
    }
    dispatch_error(
        &mut engine,
        TOOL,
        json!({"action":"open_add","expected_revision":0,"page_id":0,"target":free()}),
        "invalid_params",
    )?;
    dispatch_error(
        &mut engine,
        TOOL,
        json!({"action":"open_add","expected_revision":0,"page_id":1,"target":free(),"count":6}),
        "invalid_params",
    )?;
    let id = open_add(&mut engine, free())?;
    assert_eq!(id, 1);
    let first = engine.ui_add_chain_request().ok_or("first token")?;
    for count in [0, 1, 65, 255] {
        api_error(&mut engine, "invalid_params", |engine| {
            engine.ui_add_chain_candidates(id, count)
        })?;
        api_error(&mut engine, "invalid_params", |engine| {
            engine.apply_ui_add_chain(id, count, 0)
        })?;
    }
    api_error(&mut engine, "invalid_params", |engine| {
        engine.apply_ui_add_chain(id, 6, usize::MAX)
    })?;
    api_error(&mut engine, "ui_stale_request", |engine| {
        engine.ui_add_chain_candidates(id + 1, 6)
    })?;
    api_error(&mut engine, "ui_stale_request", |engine| {
        engine.apply_ui_add_chain(id + 1, 6, 0)
    })?;
    for (name, params) in [
        ("ui.file", json!({"action":"open","expected_revision":0})),
        (
            "ui.atom_label",
            json!({"action":"open","expected_revision":0,"page_id":1,"fragment_id":1,"atom_id":1}),
        ),
        (
            "ui.drawing_settings",
            json!({"action":"open","expected_revision":0}),
        ),
    ] {
        dispatch_error(&mut engine, name, params, "ui_busy")?;
    }
    assert!(!first.was_cancelled());
    engine.set_ui_file_host(false);
    assert!(engine.ui_add_chain_request().is_none());
    assert!(!first.was_cancelled());
    engine.set_ui_file_host(true);
    let next = open_add(&mut engine, free())?;
    assert!(next > id);
    let second = engine.ui_add_chain_request().ok_or("new token")?;
    dispatch_error(
        &mut engine,
        TOOL,
        json!({"action":"cancel_add","expected_revision":0,"request_id":id}),
        "ui_stale_request",
    )?;
    assert!(!second.was_cancelled());
    cancel_add(&mut engine, next)?;
    assert!(second.was_cancelled());
    assert!(!first.was_cancelled());
    engine.dispatch(
        "ui.drawing_settings",
        json!({"action":"open","expected_revision":0}),
    )?;
    let settings = engine.ui_drawing_settings_request().ok_or("settings")?.id;
    dispatch_error(
        &mut engine,
        TOOL,
        json!({"action":"open_add","expected_revision":0,"page_id":1,"target":free()}),
        "ui_busy",
    )?;
    engine.dispatch(
        "ui.drawing_settings",
        json!({"action":"cancel","expected_revision":0,"request_id":settings}),
    )?;

    let opaque =
        String::from_utf8(NATIVE.to_vec())?.replace("<CDXML ", "<CDXML Private=\"retain\" ");
    let mut foreign = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    api_error(&mut foreign, "unsupported_content", |engine| {
        engine.chain_preferences()
    })?;
    api_error(&mut foreign, "unsupported_content", |engine| {
        engine.chain_context(1)
    })?;
    dispatch_error(
        &mut foreign,
        TOOL,
        json!({"action":"select","kind":"acyclic"}),
        "unsupported_content",
    )?;
    assert_eq!(
        foreign.stored_document()?.export_cdxml()?,
        opaque.as_bytes()
    );
    let directory = support::FileTestDirectory::new()?;
    let raw = directory.path("complete-原.smi");
    let bytes = b"CC private\0\xff\r\n";
    fs::write(&raw, bytes)?;
    let mut foreign = native_host()?;
    foreign.dispatch("document.open_smiles", json!({"path":raw}))?;
    api_error(&mut foreign, "unsupported_content", |engine| {
        engine.chain_preferences()
    })?;
    api_error(&mut foreign, "unsupported_content", |engine| {
        engine.chain_context(1)
    })?;
    dispatch_error(
        &mut foreign,
        TOOL,
        json!({"action":"select","kind":"acyclic"}),
        "unsupported_content",
    )?;
    assert_eq!(fs::read(&raw)?, bytes);
    foreign.dispatch("document.import_smiles_records",json!({"expected_revision":1,"selection":{"mode":"all"},
        "bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,"page_width_pt":612,"page_height_pt":792}))?;
    api_error(&mut foreign, "unsupported_content", |engine| {
        engine.chain_preferences()
    })?;
    api_error(&mut foreign, "unsupported_content", |engine| {
        engine.chain_context(1)
    })?;

    let mut stored = StoredDocument::default();
    let atom = Atom::new(
        AtomId::new(7).ok_or("atom id")?,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(240.0, 240.0)?),
    );
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(9).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    let mut unknown = Engine::from_document(stored);
    unknown.set_ui_file_host(true);
    unknown.dispatch(TOOL, json!({"action":"select","kind":"acyclic"}))?;
    dispatch_error(
        &mut unknown,
        TOOL,
        json!({"action":"open_add","expected_revision":0,"page_id":1,
        "target":{"mode":"atom","fragment_id":9,"atom_id":7}}),
        "unsupported_content",
    )?;
    Ok(())
}

#[test]
fn post_add_repeat_and_named_save_aba_use_current_proofs_and_preserve_captured_save_truth()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let a = directory.path("named-原-λ.cdxml");
    let b = directory.path("named-other-化学.cdxml");
    let output = directory.path("captured-before-設定.cdxml");
    let unused = directory.path("must-not-publish.cdxml");
    fs::write(&a, NATIVE)?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_cdxml", json!({"path":a}))?;
    engine.dispatch(TOOL, json!({"action":"select","kind":"acyclic"}))?;
    let old = engine.chain_context(1)?;
    let owner = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    engine.dispatch("document.save_cdxml", json!({"path":b}))?;
    engine.dispatch("document.save_cdxml", json!({"path":a,"overwrite":true}))?;
    assert!(engine.capture_stamp_matches(&owner));
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.state().revision, 1);
    assert!(!engine.state().file.dirty);
    assert_eq!(
        engine.state().file.path.as_deref(),
        fs::canonicalize(&a)?.to_str()
    );
    assert!(!old.is_current(&engine));
    let target: ChainTarget = serde_json::from_value(free())?;
    api_error(&mut engine, "ui_stale_request", |engine| {
        engine.chain_repeat_candidates(&old, 1, &target, 6)
    })?;
    let shared = Mutex::new(engine);
    let (pending, post_add) = {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        let before = serde_json::to_value(engine.document()?)?;
        let id = open_add(
            &mut engine,
            json!({"mode":"atom","fragment_id":1,"atom_id":2}),
        )?;
        let candidates = engine.ui_add_chain_candidates(id, 6)?;
        assert!(!candidates.is_empty() && candidates.len() <= 4);
        let pending = engine.prepare(Command::parse(
            "document.save_cdxml",
            json!({"path":output}),
        )?)?;
        let result = engine.apply_ui_add_chain(id, 6, 0)?;
        assert!(result.changed);
        assert_eq!(
            (result.revision, result.undo_depth, result.redo_depth),
            (2, 1, 0)
        );
        assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
        let page = &engine.document()?.pages()[0];
        assert_eq!(page.fragments().len(), 1);
        let fragment = &page.fragments()[0];
        assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (7, 6));
        let actual = serde_json::to_value(engine.document()?)?;
        assert_eq!(
            actual["pages"][0]["fragments"][0]["atoms"][0],
            before["pages"][0]["fragments"][0]["atoms"][0]
        );
        let mut expected_anchor = before["pages"][0]["fragments"][0]["atoms"][1].clone();
        assert_eq!(expected_anchor["explicit_hydrogens"], 3);
        expected_anchor["explicit_hydrogens"] = json!(2);
        assert_eq!(
            actual["pages"][0]["fragments"][0]["atoms"][1],
            expected_anchor
        );
        assert_eq!(
            actual["pages"][0]["fragments"][0]["bonds"][0],
            before["pages"][0]["fragments"][0]["bonds"][0]
        );
        for (i, atom) in fragment.atoms().iter().skip(2).enumerate() {
            assert_eq!(atom.id.get(), 3 + i as u32);
            assert_eq!(atom.element.get(), 6);
            assert_eq!(atom.explicit_hydrogens, Some(if i == 4 { 3 } else { 2 }));
            assert_eq!(atom.show_terminal_carbon_labels, Some(false));
            assert_eq!(atom.show_non_terminal_carbon_labels, Some(false));
        }
        let proof = engine.chain_context(1)?;
        assert!(proof.is_current(&engine));
        // Commit result and POST proof are retained inside this same real mutex guard.
        (pending, proof)
    };
    let post_repeat = {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        let before = serde_json::to_value(engine.state())?;
        let candidates = engine.chain_repeat_candidates(&post_add, 1, &target, 6)?;
        assert_eq!(candidates.len(), 1);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        let first = candidates.first().ok_or("repeat")?;
        assert_eq!(
            (first.expected_revision, first.page_id, first.count),
            (2, 1, 6)
        );
        let preview = engine.preview_chain_create(first)?;
        let result = engine.execute(Command::ChainCreate(first.clone()))?;
        assert!(result.changed);
        assert_eq!(
            (result.revision, result.undo_depth, result.redo_depth),
            (3, 2, 0)
        );
        assert_eq!(engine.document()?, preview.model());
        assert!(!post_add.is_current(&engine));
        assert_eq!(
            engine.document()?.pages()[0]
                .fragments()
                .iter()
                .map(|f| f.atoms().len())
                .sum::<usize>(),
            13
        );
        let proof = engine.chain_context(1)?;
        assert!(proof.is_current(&engine));
        api_error(&mut engine, "busy", |engine| {
            engine.prepare(Command::parse(
                "document.save_cdxml",
                json!({"path":unused}),
            )?)
        })?;
        proof
    };
    let completed = pending.complete()?;
    assert_eq!(fs::read(&output)?, NATIVE);
    {
        let mut engine = shared.lock().map_err(|_| "engine")?;
        api_error(&mut engine, "busy", |engine| {
            engine.prepare(Command::parse(
                "document.save_cdxml",
                json!({"path":unused}),
            )?)
        })?;
        let result = engine.finish(completed)?;
        assert_eq!(
            (result.revision, result.undo_depth, result.redo_depth),
            (3, 2, 0)
        );
        assert!(result.file.dirty);
        assert!(result.ui_file_operation.is_none());
        let receipt = result.file_operation.ok_or("truthful captured Save")?;
        assert_eq!(receipt.input_revision, 1);
        assert!(receipt.ui_request_id.is_none() && receipt.ui_request_current.is_none());
        assert!(!post_repeat.is_current(&engine));
        api_error(&mut engine, "ui_stale_request", |engine| {
            engine.chain_repeat_candidates(&post_repeat, 1, &target, 6)
        })?;
        assert!(
            engine
                .state()
                .source
                .ok_or("original native source")?
                .modified
        );
    }
    assert_eq!(fs::read(&a)?, NATIVE);
    assert_eq!(fs::read(&b)?, NATIVE);
    assert!(!unused.exists());
    Ok(())
}

#[test]
fn equal_public_state_in_another_engine_cannot_accept_an_origin_chain_context() -> TestResult {
    let origin = native_host()?;
    let proof = origin.chain_context(1)?;
    let mut other = native_host()?;
    assert_eq!(
        serde_json::to_value(origin.state())?,
        serde_json::to_value(other.state())?
    );
    assert!(proof.is_current(&origin));
    assert!(!proof.is_current(&other));
    let target: ChainTarget = serde_json::from_value(free())?;
    api_error(&mut other, "ui_stale_request", |engine| {
        engine.chain_repeat_candidates(&proof, 1, &target, 6)
    })?;
    assert!(proof.is_current(&origin));
    Ok(())
}
