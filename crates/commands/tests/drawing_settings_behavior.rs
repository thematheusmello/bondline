//! Independent locked command acceptance; Root owns adoption and execution.
use bondline_commands::{Cancellation, CloseState, Command, Engine};
use bondline_io::StoredDocument;
use bondline_model::{DrawingSettings, PageId, PageSize, Point2};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    io::Write as _,
    process::{Command as Process, Stdio},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only unchanged isolated-directory/consumer helpers"
)]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const NAME: &str = "document.set_drawing_settings";
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

fn native() -> TestResult<Engine> {
    let stored = StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    Ok(Engine::from_document(stored))
}
fn params(engine: &Engine, length: Option<f64>, angle: Option<f64>) -> Value {
    json!({"expected_revision":engine.state().revision,
        "bond_length_pt":length,"chain_angle_degrees":angle})
}
fn select(engine: &mut Engine) -> TestResult {
    engine.dispatch(
        "ui.selection",
        json!({"action":"set",
        "expected_revision":engine.state().revision,"page_id":1,"atom_ids":[1,2]}),
    )?;
    Ok(())
}
fn refuse(engine: &mut Engine, input: Value, code: &str) -> TestResult {
    let state = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let model = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let owner = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let view = engine.view();
    let tool = engine.bond_tool();
    let effective = engine.effective_tool();
    let generation = engine.session_generation();
    let error = engine
        .dispatch(NAME, input)
        .err()
        .ok_or("invalid settings accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), owner);
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert_eq!(
        (engine.view(), engine.bond_tool(), engine.effective_tool()),
        (view, tool, effective)
    );
    assert_eq!(engine.session_generation(), generation);
    if let Some(model) = model {
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    Ok(())
}
fn shared_noop(engine: &mut Engine) -> TestResult {
    let value = engine.document()?.drawing_settings();
    let state = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let model = engine.stored_document()?.model_snapshot();
    let owner = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let view = engine.view();
    let tool = engine.bond_tool();
    let effective = engine.effective_tool();
    let generation = engine.session_generation();
    let result = engine.dispatch(
        NAME,
        params(engine, value.bond_length_pt(), value.chain_angle_degrees()),
    )?;
    assert!(!result.changed);
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert_eq!(engine.content(), &content);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.capture_stamp(), owner);
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert_eq!(
        (engine.view(), engine.bond_tool(), engine.effective_tool()),
        (view, tool, effective)
    );
    assert_eq!(engine.session_generation(), generation);
    Ok(())
}

#[test]
fn closed_required_nullable_pair_replaces_both_values_and_validates_atomically() -> TestResult {
    let rows = bondline_commands::registry();
    assert_eq!(rows.len(), 42);
    assert_eq!(rows.get(40).ok_or("legacy settings position")?.name, NAME);
    let row = rows
        .iter()
        .find(|row| row.name == NAME)
        .ok_or("settings registry")?;
    assert!(!row.read_only);
    let schema = row.input_schema.as_value();
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    let required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|value| value.as_str().ok_or("required name"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(
        required,
        BTreeSet::from(["expected_revision", "bond_length_pt", "chain_angle_degrees"])
    );
    assert_eq!(
        schema["properties"].as_object().ok_or("properties")?.len(),
        3
    );
    for key in ["bond_length_pt", "chain_angle_degrees"] {
        let types = schema["properties"][key]["type"]
            .as_array()
            .ok_or("nullable number types")?;
        assert_eq!(types.len(), 2);
        assert!(types.contains(&json!("number")) && types.contains(&json!("null")));
    }
    assert_eq!(
        schema["properties"]["bond_length_pt"]["exclusiveMinimum"].as_f64(),
        Some(0.0)
    );
    assert_eq!(
        schema["properties"]["bond_length_pt"]["maximum"].as_f64(),
        Some(100_000.0)
    );
    assert_eq!(
        schema["properties"]["chain_angle_degrees"]["minimum"].as_f64(),
        Some(1.0)
    );
    assert_eq!(
        schema["properties"]["chain_angle_degrees"]["maximum"].as_f64(),
        Some(179.0)
    );

    let mut engine = native()?;
    for (length, angle) in [
        (Some(24.5), Some(120.5)),
        (None, Some(90.25)),
        (Some(100_000.0), Some(1.0)),
        (Some(0.25), Some(179.0)),
        (None, None),
    ] {
        let mut expected = serde_json::to_value(engine.document()?)?;
        let mut fields = serde_json::Map::new();
        if let Some(value) = length {
            fields.insert("bond_length_pt".into(), json!(value));
        }
        if let Some(value) = angle {
            fields.insert("chain_angle_degrees".into(), json!(value));
        }
        if fields.is_empty() {
            expected
                .as_object_mut()
                .ok_or("document")?
                .remove("drawing_settings");
        } else {
            expected["drawing_settings"] = Value::Object(fields);
        }
        let revision = engine.state().revision;
        let result = engine.dispatch(NAME, params(&engine, length, angle))?;
        assert!(result.changed);
        assert_eq!(result.revision, revision + 1);
        assert_eq!(serde_json::to_value(engine.document()?)?, expected);
    }
    for key in ["expected_revision", "bond_length_pt", "chain_angle_degrees"] {
        let mut input = params(&engine, None, None);
        input.as_object_mut().ok_or("params")?.remove(key);
        refuse(&mut engine, input, "invalid_params")?;
    }
    for (key, value) in [
        ("expected_revision", json!(null)),
        ("expected_revision", json!(-1)),
        ("expected_revision", json!(0.5)),
        ("bond_length_pt", json!(true)),
        ("bond_length_pt", json!("24.5")),
        ("bond_length_pt", json!(0)),
        ("bond_length_pt", json!(-1)),
        ("bond_length_pt", json!(100_000.000_1)),
        ("chain_angle_degrees", json!(false)),
        ("chain_angle_degrees", json!("120")),
        ("chain_angle_degrees", json!(0.999)),
        ("chain_angle_degrees", json!(179.001)),
        ("page_id", json!(1)),
    ] {
        let mut input = params(&engine, Some(24.5), Some(120.5));
        input[key] = value;
        refuse(&mut engine, input, "invalid_params")?;
    }
    for input in [Value::Null, json!([engine.state().revision, 24.5, 120.5])] {
        refuse(&mut engine, input, "invalid_params")?;
    }
    let mut stale = params(&engine, Some(0.0), Some(180.0));
    stale["expected_revision"] = json!(0);
    refuse(&mut engine, stale, "stale_revision")?;
    Ok(())
}

#[test]
fn validated_noops_preserve_redo_selection_weak_contexts_close_and_file_intent() -> TestResult {
    let mut engine = native()?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.state().redo_depth, 1);
    engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
    engine.dispatch("ui.view", json!({"action":"actual"}))?;
    engine.dispatch(
        "ui.view",
        json!({"action":"pan","center_x_pt":120,"center_y_pt":200}),
    )?;
    engine.set_ui_file_host(true);
    select(&mut engine)?;
    let selection = engine.selection_context()?;
    shared_noop(&mut engine)?;
    assert!(selection.is_current(&engine));
    assert_eq!(engine.state().redo_depth, 1);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(
        engine.document()?.pages()[0].size(),
        PageSize::new(400.0, 600.0)?
    );

    for (dirty, action, expected) in [
        (false, "request", CloseState::Approved),
        (true, "request", CloseState::NeedsDecision),
        (true, "discard", CloseState::Approved),
    ] {
        let mut engine = native()?;
        if dirty {
            engine.dispatch(
                "page.set_size",
                json!({"page_id":1,"width_pt":400,"height_pt":600}),
            )?;
        }
        engine.dispatch(
            "document.close",
            json!({"expected_revision":engine.state().revision,"action":action}),
        )?;
        assert_eq!(engine.state().close, expected);
        shared_noop(&mut engine)?;
        assert_eq!(engine.state().close, expected);
    }
    let mut engine = native()?;
    engine.set_ui_file_host(true);
    engine.dispatch("ui.file", json!({"expected_revision":0,"action":"save"}))?;
    assert!(serde_json::to_value(engine.state())?["ui_file"]["request"].is_object());
    shared_noop(&mut engine)?;
    Ok(())
}

#[test]
fn one_real_edit_keeps_scene_pages_h_and_floors_but_invalidates_old_selection() -> TestResult {
    let mut engine = native()?;
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":200,"begin_y_pt":300,
        "end_x_pt":248,"end_y_pt":300}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.state().redo_depth, 1);
    select(&mut engine)?;
    let selection = engine.selection_context()?;
    let original = engine.stored_document()?.model_snapshot();
    let owner = engine.capture_stamp();
    let view = engine.view();
    let tool = engine.bond_tool();
    let generation = engine.session_generation();
    let scene = engine.drawing_view_scene(
        1024,
        1024,
        PageId::new(1).ok_or("page")?,
        &Cancellation::default(),
    )?;
    let revision = engine.state().revision;
    let result = engine.dispatch(NAME, params(&engine, Some(24.5), Some(120.5)))?;
    assert_eq!(
        (
            result.changed,
            result.revision,
            result.undo_depth,
            result.redo_depth
        ),
        (true, revision + 1, 1, 0)
    );
    assert!(result.file.dirty);
    assert!(result.source.ok_or("original source")?.modified);
    assert!(!selection.is_current(&engine));
    assert!(engine.selection().atom_ids.is_empty());
    assert!(!engine.capture_stamp_matches(&owner));
    assert_eq!(engine.document()?.pages(), original.pages());
    assert!(Arc::ptr_eq(
        &engine.document()?.pages()[0].fragments()[0],
        &original.pages()[0].fragments()[0]
    ));
    assert_eq!(
        engine.document()?.drawing_settings(),
        DrawingSettings::new(Some(24.5), Some(120.5))?
    );
    assert_eq!(
        engine
            .drawing_view_scene(
                1024,
                1024,
                PageId::new(1).ok_or("page")?,
                &Cancellation::default()
            )?
            .scene(),
        scene.scene()
    );
    assert_eq!(
        (
            engine.view(),
            engine.bond_tool(),
            engine.session_generation()
        ),
        (view, tool, generation)
    );
    let changed = engine.stored_document()?.model_snapshot();
    engine.dispatch("history.undo", json!({}))?;
    assert!(Arc::ptr_eq(
        &original,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    engine.dispatch("history.redo", json!({}))?;
    assert!(Arc::ptr_eq(
        &changed,
        &engine.stored_document()?.model_snapshot()
    ));
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":200,"begin_y_pt":300,
        "end_x_pt":248,"end_y_pt":300}),
    )?;
    let fragment = &engine.document()?.pages()[0].fragments()[1];
    assert_eq!(fragment.id().get(), 3);
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|atom| atom.id.get())
            .collect::<Vec<_>>(),
        vec![5, 6]
    );
    assert_eq!(fragment.bonds()[0].id.get(), 3);
    Ok(())
}

#[test]
fn unsupported_whole_sources_refuse_equal_values_before_domain_or_noop() -> TestResult {
    for source in [
        String::from_utf8(NATIVE.to_vec())?.replace("<CDXML ", "<CDXML Private=\"retain\" "),
        String::from_utf8(NATIVE.to_vec())?.replace("><page", ">\n<page"),
    ] {
        let stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(!stored.is_fully_mapped());
        let mut engine = Engine::from_document(stored);
        for (length, angle) in [
            (None, None),
            (Some(24.5), Some(120.5)),
            (Some(0.0), Some(180.0)),
        ] {
            let input = params(&engine, length, angle);
            refuse(&mut engine, input, "unsupported_content")?;
        }
        assert_eq!(engine.stored_document()?.export_cdxml()?, source.as_bytes());
    }
    let directory = support::FileTestDirectory::new()?;
    for (name, bytes, command) in [
        (
            "old-v2.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "old-v3.mol",
            include_bytes!("../../io/tests/fixtures/v3000/noncontiguous-cc.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "old.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
        ),
        (
            "old.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        (
            "old.smi",
            b"CC original\r\nO last\n".as_slice(),
            "document.open_smiles",
        ),
    ] {
        let path = directory.path(name);
        fs::write(&path, bytes)?;
        let mut engine = Engine::default();
        engine.dispatch(command, json!({"path":path}))?;
        for (length, angle) in [
            (None, None),
            (Some(24.5), Some(120.5)),
            (Some(0.0), Some(180.0)),
        ] {
            let input = params(&engine, length, angle);
            refuse(&mut engine, input, "unsupported_content")?;
        }
        let mut stale = params(&engine, None, None);
        stale["expected_revision"] = json!(0);
        refuse(&mut engine, stale, "stale_revision")?;
        assert_eq!(fs::read(&path)?, bytes);
        if command == "document.open_smiles" {
            engine.dispatch(
                "document.import_smiles_records",
                json!({
                "expected_revision":engine.state().revision,"selection":{"mode":"all"},
                "bond_length_pt":64,"origin_x_pt":96,"origin_y_pt":96,
                "page_width_pt":612,"page_height_pt":792}),
            )?;
            assert!(engine.stored_document()?.has_smiles_projection());
            for (length, angle) in [(None, None), (Some(24.5), Some(120.5))] {
                let input = params(&engine, length, angle);
                refuse(&mut engine, input, "unsupported_content")?;
            }
            assert_eq!(engine.stored_document()?.smiles_file()?.as_bytes(), bytes);
        }
    }
    let mut closed = native()?;
    closed.dispatch("document.close", json!({"expected_revision":0}))?;
    assert!(closed.complete_close());
    let input = json!({"expected_revision":99,"bond_length_pt":0,"chain_angle_degrees":180});
    refuse(&mut closed, input, "close_committed")?;
    Ok(())
}

fn reader(relative: &str, bytes: &[u8]) -> TestResult<Value> {
    let mut child = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .arg("-B")
        .arg(format!("{}/{relative}", env!("CARGO_MANIFEST_DIR")))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn unicode_save_keeps_original_current_and_saved_settings_distinct_through_history() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("original-設定-λ.cdxml");
    let output = directory.path("saved-設定-λ.cdxml");
    fs::write(&input, NATIVE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    let original = engine.stored_document()?.model_snapshot();
    engine.dispatch(NAME, params(&engine, Some(24.5), Some(120.5)))?;
    let changed = engine.stored_document()?.model_snapshot();
    assert!(engine.state().file.dirty);
    assert!(engine.state().source.ok_or("source")?.modified);
    let saved = engine.dispatch("document.save_cdxml", json!({"path":output}))?;
    assert!(!saved.file.dirty);
    assert_eq!(
        saved.file.path.as_deref(),
        fs::canonicalize(&output)?.to_str()
    );
    assert!(
        saved
            .source
            .ok_or("source remains original-relative")?
            .modified
    );
    let bytes = fs::read(&output)?;
    assert_eq!(
        reader("../io/tests/drawing_settings_root.py", &bytes)?,
        json!({
        "root":[["CreationProgram","Bondline 0.1.0"],["BondLength","24.5"],["ChainAngle","120.5"]],
        "decimal":{"BondLength":"24.5","ChainAngle":"120.5"}})
    );
    assert_eq!(
        reader("../../xtask/oracles/cdxml_graph.py", &bytes)?,
        json!([
        {"id":1,"atoms":[
            {"id":1,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
            {"id":2,"element":6,"position":{"x_pt":140.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]}])
    );
    support::independently_read_file(&output, 1)?;
    engine.dispatch("history.undo", json!({}))?;
    assert!(engine.state().file.dirty);
    assert!(!engine.state().source.ok_or("original source")?.modified);
    assert!(Arc::ptr_eq(
        &original,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    engine.dispatch("history.redo", json!({}))?;
    assert!(!engine.state().file.dirty);
    assert!(Arc::ptr_eq(
        &changed,
        &engine.stored_document()?.model_snapshot()
    ));
    engine.dispatch(NAME, params(&engine, None, None))?;
    assert!(
        engine.state().file.dirty,
        "current equals original, not the styled saved baseline"
    );
    assert!(!engine.state().source.ok_or("original source")?.modified);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    assert_eq!(fs::read(&input)?, NATIVE);
    assert_eq!(fs::read(&output)?, bytes);
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert!(!reopened.state().file.dirty);
    assert!(!reopened.state().source.ok_or("new original")?.modified);
    assert_eq!(
        reopened.document()?.drawing_settings(),
        DrawingSettings::new(Some(24.5), Some(120.5))?
    );
    assert_eq!(reopened.document()?.pages(), changed.pages());
    assert_eq!(reopened.stored_document()?.export_cdxml()?, bytes);
    Ok(())
}

#[test]
fn immediate_prepare_late_cancel_and_context_invalidation_keep_existing_policy_and_focus()
-> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,
        "end_x_pt":140,"end_y_pt":200}),
    )?;
    engine.set_ui_file_host(true);
    engine.dispatch("ui.file", json!({"expected_revision":1,"action":"save"}))?;
    let file = engine.ui_file_context_stamp();
    let generation = engine.session_generation();
    let view = engine.view();
    let prepared = engine.prepare(Command::parse(NAME, params(&engine, Some(30.0), None))?)?;
    assert_eq!(
        engine.state().revision,
        2,
        "the normal immediate lane commits during prepare"
    );
    assert!(!engine.ui_file_context_stamp_matches(&file));
    assert!(serde_json::to_value(engine.state())?["ui_file"]["request"].is_null());
    let model = engine.stored_document()?.model_snapshot();
    let cancel = Cancellation::default();
    cancel.cancel();
    let result = engine.finish(prepared.complete_with_cancel(&cancel)?)?;
    assert!(result.changed);
    assert_eq!((result.revision, result.undo_depth), (2, 2));
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(
        (engine.view(), engine.session_generation()),
        (view, generation)
    );
    engine.dispatch(
        "ui.view",
        json!({"action":"magnify","width_px":1000,"height_px":1000}),
    )?;
    assert_eq!(
        engine.view().center,
        Some(Point2::new(120.0, 200.0)?),
        "settings must retain the earlier authored midpoint as the next magnify focus"
    );

    let mut engine = native()?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert_eq!(engine.state().close, CloseState::Approved);
    engine.dispatch(NAME, params(&engine, Some(30.0), Some(90.0)))?;
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    assert!(
        !engine.complete_close(),
        "old approval cannot approve the changed revision"
    );
    assert_eq!(engine.state().revision, 1);
    Ok(())
}
