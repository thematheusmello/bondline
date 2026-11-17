//! Independent selection/translation acceptance through existing JSON APIs.
use bondline_commands::{Command, Engine};
use serde_json::{Value, json};
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn selected_methanol_pair_is_transient_and_leaves_the_other_ethane_untouched() -> TestResult {
    let mut engine = Engine::default();
    for (begin, end) in [(72, 120), (220, 268)] {
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,
            "begin_x_pt":begin,"begin_y_pt":72,"end_x_pt":end,"end_y_pt":72}),
        )?;
    }
    engine.dispatch(
        "atom.set_label",
        json!({"expected_revision":2,"page_id":1,
        "fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    assert_eq!(before["revision"], 3);
    assert_eq!(
        before["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        2
    );
    assert!(before.get("ui_selection").is_none());
    let result = engine.dispatch(
        "ui.selection",
        json!({"action":"set",
        "expected_revision":3,"page_id":1,"atom_ids":[2,1]}),
    )?;
    assert!(!result.changed);
    let mut received = serde_json::to_value(result)?;
    let selected = json!({"active":true,"epoch":1,"page_id":1,"atom_ids":[1,2]});
    assert_eq!(received["ui_selection"], selected);
    assert_eq!(
        received["ui_selection_operation"],
        json!({"epoch":1,"changed":true})
    );
    assert_eq!(
        serde_json::to_value(engine.state())?["ui_selection"],
        selected
    );
    let object = received.as_object_mut().ok_or("state object")?;
    object.remove("ui_selection");
    object.remove("ui_selection_operation");
    assert_eq!(
        received, before,
        "selection must not change any authored or file state"
    );
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    Ok(())
}

#[test]
fn exact_translation_is_a_named_closed_command_independent_of_selection_setup() -> TestResult {
    let command = Command::parse(
        "selection.translate",
        json!({"expected_revision":0,
        "expected_selection_epoch":0,"delta_x_pt":24.0,"delta_y_pt":-12.0}),
    )?;
    assert_eq!(command.name(), "selection.translate");
    Ok(())
}

// Additive independent cases; the original two existing-API declarations stay exact.
use bondline_commands::CloseState;
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, PageSize, Point2,
    Radical,
};
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    num::NonZeroU16,
    path::Path,
    process::{Command as Process, Stdio},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated test directory")]
mod support;

const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const RETAINED_SMILES: &[u8] = b"CC first\r\nO\tprivate\xff\0\n";

fn wire(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.state())?)
}

fn author_state(engine: &Engine) -> TestResult<Value> {
    let mut result = wire(engine)?;
    result
        .as_object_mut()
        .ok_or("state object")?
        .remove("ui_selection");
    Ok(result)
}

fn selected(engine: &Engine) -> TestResult<Value> {
    Ok(wire(engine)?
        .get("ui_selection")
        .cloned()
        .ok_or("selection state")?)
}

fn epoch(engine: &Engine) -> TestResult<u64> {
    Ok(selected(engine)?["epoch"]
        .as_u64()
        .ok_or("selection epoch")?)
}

fn action(engine: &Engine, name: &str) -> Value {
    json!({"action":name,"expected_revision":engine.state().revision})
}

fn set_params(engine: &Engine, ids: &[u32]) -> Value {
    json!({"action":"set","expected_revision":engine.state().revision,"page_id":1,
        "atom_ids":ids})
}

fn select(engine: &mut Engine, ids: &[u32]) -> TestResult<Value> {
    let params = set_params(engine, ids);
    Ok(serde_json::to_value(
        engine.dispatch("ui.selection", params)?,
    )?)
}

fn motion(engine: &Engine, x: f64, y: f64) -> TestResult<Value> {
    Ok(json!({"expected_revision":engine.state().revision,
        "expected_selection_epoch":epoch(engine)?,"delta_x_pt":x,"delta_y_pt":y}))
}

fn move_selected(engine: &mut Engine, x: f64, y: f64) -> TestResult<Value> {
    let params = motion(engine, x, y)?;
    Ok(serde_json::to_value(
        engine.dispatch("selection.translate", params)?,
    )?)
}

fn co_and_cc() -> TestResult<Engine> {
    let mut engine = Engine::default();
    for (begin, end) in [(72, 120), (220, 268)] {
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":begin,
            "begin_y_pt":72,"end_x_pt":end,"end_y_pt":72}),
        )?;
    }
    engine.dispatch(
        "atom.set_label",
        json!({"expected_revision":2,"page_id":1,
        "fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}),
    )?;
    Ok(engine)
}

fn failed(engine: &mut Engine, name: &str, params: Value, code: &str) -> TestResult {
    let before = wire(engine)?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let generation = engine.session_generation();
    let view = engine.view();
    let tool = engine.bond_tool();
    let model = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let error = engine
        .dispatch(name, params)
        .err()
        .ok_or("invalid command accepted")?;
    assert_eq!(error.info().code, code, "{name}: {error}");
    assert_eq!(wire(engine)?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert_eq!(engine.session_generation(), generation);
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

fn named_selection(engine: &mut Engine, name: &str) -> TestResult<Value> {
    let params = action(engine, name);
    Ok(serde_json::to_value(
        engine.dispatch("ui.selection", params)?,
    )?)
}

#[test]
fn schemas_add_only_two_closed_commands_and_invalid_wire_inputs_are_atomic() -> TestResult {
    let registry = bondline_commands::registry();
    let names = registry
        .iter()
        .map(|entry| entry.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(registry.len(), 42);
    assert_eq!(names.len(), 42);
    assert_eq!(
        names,
        [
            "atom.set_label",
            "bond.create",
            "bond.extend",
            "bond.set_order",
            "chain.create",
            "chain.create_path",
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
            "document.set_drawing_settings",
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
            "ui.ring_tool",
            "ui.controls",
            "ui.drawing_settings",
            "ui.file",
            "ui.selection",
            "ui.view",
        ]
        .into_iter()
        .collect::<BTreeSet<_>>()
    );
    for name in ["ui.selection", "selection.translate"] {
        assert!(
            !registry
                .iter()
                .find(|entry| entry.name == name)
                .ok_or("new command")?
                .read_only
        );
    }
    let schema = serde_json::to_value(
        &registry
            .iter()
            .find(|entry| entry.name == "ui.selection")
            .ok_or("selection schema")?
            .input_schema,
    )?;
    assert_eq!(schema["type"], "object");
    let variants = schema
        .get("oneOf")
        .or_else(|| schema.get("anyOf"))
        .and_then(Value::as_array)
        .ok_or("tagged variants")?;
    assert_eq!(variants.len(), 5);
    let mut tags = BTreeSet::new();
    for variant in variants {
        let tag = variant["properties"]["action"]
            .get("const")
            .and_then(Value::as_str)
            .or_else(|| {
                variant["properties"]["action"]
                    .get("enum")?
                    .as_array()?
                    .first()?
                    .as_str()
            })
            .ok_or("action tag")?;
        assert!(tags.insert(tag));
        let fields = match tag {
            "set" => vec!["action", "expected_revision", "page_id", "atom_ids"],
            "all" => vec!["action", "expected_revision", "page_id"],
            "activate" | "clear" | "inspect" => vec!["action", "expected_revision"],
            _ => return Err("undeclared selection action".into()),
        }
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(
            variant["properties"]
                .as_object()
                .ok_or("properties")?
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            fields
        );
        assert_eq!(
            variant["required"]
                .as_array()
                .ok_or("required")?
                .iter()
                .map(|value| value.as_str().ok_or("field name"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            fields
        );
        assert_eq!(variant["additionalProperties"], false);
        if tag == "set" || tag == "all" {
            assert_eq!(
                variant["properties"]["page_id"]["minimum"].as_f64(),
                Some(1.0)
            );
        }
    }
    assert_eq!(
        tags,
        ["activate", "set", "all", "clear", "inspect"]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    let translate = serde_json::to_value(
        &registry
            .iter()
            .find(|entry| entry.name == "selection.translate")
            .ok_or("translate schema")?
            .input_schema,
    )?;
    assert_eq!(translate["additionalProperties"], false);
    let fields = [
        "expected_revision",
        "expected_selection_epoch",
        "delta_x_pt",
        "delta_y_pt",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        translate["properties"]
            .as_object()
            .ok_or("translate properties")?
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        fields
    );
    assert_eq!(
        translate["required"]
            .as_array()
            .ok_or("translate required")?
            .iter()
            .map(|value| value.as_str().ok_or("translate name"))
            .collect::<Result<BTreeSet<_>, _>>()?,
        fields
    );
    let mut engine = co_and_cc()?;
    select(&mut engine, &[1, 2])?;
    let valid = set_params(&engine, &[1, 2]);
    for (field, value, code) in [
        ("expected_revision", json!(0), "stale_revision"),
        ("expected_revision", json!(-1), "invalid_params"),
        ("expected_revision", Value::Null, "invalid_params"),
        ("page_id", json!(0), "invalid_params"),
        ("page_id", json!(9), "not_found"),
        ("page_id", json!(1.5), "invalid_params"),
        ("page_id", json!(u64::MAX), "invalid_params"),
        ("atom_ids", json!([0]), "invalid_params"),
        ("atom_ids", json!([1, 1]), "invalid_params"),
        ("atom_ids", json!([999]), "not_found"),
        ("atom_ids", json!([1, "2"]), "invalid_params"),
        ("atom_ids", Value::Null, "invalid_params"),
        ("action", json!("toggle"), "invalid_params"),
        ("action", Value::Null, "invalid_params"),
        ("fragment_id", json!(1), "invalid_params"),
    ] {
        let mut params = valid.clone();
        params[field] = value;
        failed(&mut engine, "ui.selection", params, code)?;
    }
    for key in ["action", "expected_revision", "page_id", "atom_ids"] {
        let mut params = valid.clone();
        params.as_object_mut().ok_or("params")?.remove(key);
        failed(&mut engine, "ui.selection", params, "invalid_params")?;
    }
    let mut too_many = valid;
    too_many["atom_ids"] = json!((1..=100_001_u32).collect::<Vec<_>>());
    failed(&mut engine, "ui.selection", too_many, "invalid_params")?;
    for tag in ["activate", "clear", "inspect", "all"] {
        let mut params = action(&engine, tag);
        if tag == "all" {
            params["page_id"] = json!(1);
        }
        params["surplus"] = json!(true);
        failed(&mut engine, "ui.selection", params, "invalid_params")?;
    }
    let valid = motion(&engine, 24.0, -12.0)?;
    for (field, value, code) in [
        ("expected_revision", json!(0), "stale_revision"),
        ("expected_selection_epoch", json!(0), "stale_selection"),
        (
            "expected_selection_epoch",
            json!(u64::MAX),
            "stale_selection",
        ),
        ("expected_selection_epoch", Value::Null, "invalid_params"),
        ("delta_x_pt", json!(f64::MAX), "invalid_params"),
        ("delta_y_pt", json!(-f64::MAX), "invalid_params"),
        ("delta_x_pt", Value::Null, "invalid_params"),
        ("delta_y_pt", json!("12"), "invalid_params"),
        ("page_id", json!(1), "invalid_params"),
    ] {
        let mut params = valid.clone();
        params[field] = value;
        failed(&mut engine, "selection.translate", params, code)?;
    }
    for key in [
        "expected_revision",
        "expected_selection_epoch",
        "delta_x_pt",
        "delta_y_pt",
    ] {
        let mut params = valid.clone();
        params.as_object_mut().ok_or("params")?.remove(key);
        failed(&mut engine, "selection.translate", params, "invalid_params")?;
    }
    let mut stale = motion(&engine, 24.0, -12.0)?;
    stale["expected_revision"] = json!(0);
    stale["expected_selection_epoch"] = json!(0);
    failed(&mut engine, "selection.translate", stale, "stale_revision")?;
    Ok(())
}

#[test]
fn transient_selection_mode_epoch_controls_and_close_file_guards_are_explicit() -> TestResult {
    let mut engine = co_and_cc()?;
    let authored = author_state(&engine)?;
    let initial_tool = engine.bond_tool();
    let activated = named_selection(&mut engine, "activate")?;
    assert!(!activated["changed"].as_bool().ok_or("changed")?);
    assert_eq!(
        selected(&engine)?,
        json!({"active":true,"epoch":1,"page_id":null,"atom_ids":[]})
    );
    assert_eq!(engine.bond_tool().epoch, initial_tool.epoch + 1);
    let first = select(&mut engine, &[4, 1, 2])?;
    assert_eq!(
        first["ui_selection_operation"],
        json!({"epoch":2,"changed":true})
    );
    let tool = engine.bond_tool();
    let before = wire(&engine)?;
    assert_eq!(
        select(&mut engine, &[2, 4, 1])?["ui_selection_operation"],
        json!({"epoch":2,"changed":false})
    );
    assert_eq!(wire(&engine)?, before);
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(
        named_selection(&mut engine, "inspect")?["ui_selection"],
        selected(&engine)?
    );
    assert_eq!(author_state(&engine)?, authored);
    let revision = engine.state().revision;
    engine.dispatch(
        "ui.selection",
        json!({"action":"all","expected_revision":revision,"page_id":1}),
    )?;
    assert_eq!(
        selected(&engine)?,
        json!({"active":true,"epoch":3,"page_id":1,"atom_ids":[1,2,3,4]})
    );
    let controls = engine.controls();
    for (id, label, tag) in [
        ("tool.selection", "Marquee selection", "activate"),
        ("edit.select_all", "Select All", "all"),
        ("edit.deselect", "Deselect", "clear"),
    ] {
        let control = controls
            .iter()
            .find(|control| control.id == id)
            .ok_or("selection control")?;
        assert_eq!(control.label, label);
        assert_eq!(control.command.name(), "ui.selection");
        let params = serde_json::to_value(&control.command)?;
        assert_eq!(params["params"]["action"], tag);
        assert_eq!(params["params"]["expected_revision"], revision);
    }
    assert_eq!(
        controls
            .iter()
            .find(|control| control.id == "tool.selection")
            .ok_or("selection tool")?
            .selected,
        Some(true)
    );
    for id in ["tool.bond.single", "tool.bond.double", "tool.bond.triple"] {
        assert_eq!(
            controls
                .iter()
                .find(|control| control.id == id)
                .ok_or("bond control")?
                .selected,
            Some(false)
        );
    }
    named_selection(&mut engine, "clear")?;
    assert_eq!(
        selected(&engine)?,
        json!({"active":true,"epoch":4,"page_id":null,"atom_ids":[]})
    );
    let before = wire(&engine)?;
    assert_eq!(
        named_selection(&mut engine, "clear")?["ui_selection_operation"],
        json!({"epoch":4,"changed":false})
    );
    assert_eq!(wire(&engine)?, before);
    engine.dispatch("ui.bond_tool", json!({"order":"single"}))?;
    assert_eq!(
        selected(&engine)?,
        json!({"active":false,"epoch":5,"page_id":null,"atom_ids":[]})
    );
    assert_eq!(engine.bond_tool().order, initial_tool.order);
    assert_eq!(engine.bond_tool().epoch, initial_tool.epoch + 2);
    assert_eq!(author_state(&engine)?, authored);
    select(&mut engine, &[1, 2])?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":revision,"action":"request"}),
    )?;
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    for tag in ["activate", "clear"] {
        let params = action(&engine, tag);
        failed(&mut engine, "ui.selection", params, "ui_busy")?;
    }
    let queried = named_selection(&mut engine, "inspect")?;
    assert_eq!(queried["ui_selection"], selected(&engine)?);
    let params = motion(&engine, 1.0, 0.0)?;
    assert!(engine.dispatch("selection.translate", params)?.changed);
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"cancel"}),
    )?;
    engine.set_ui_file_host(true);
    let request = engine.dispatch(
        "ui.file",
        json!({"expected_revision":engine.state().revision,"action":"save"}),
    )?;
    let request_id = request.ui_file_operation.ok_or("file request")?;
    let params = action(&engine, "clear");
    failed(&mut engine, "ui.selection", params, "ui_busy")?;
    named_selection(&mut engine, "inspect")?;
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":engine.state().revision,
        "action":"cancel","request_id":request_id}),
    )?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"discard"}),
    )?;
    assert!(engine.complete_close());
    for tag in ["activate", "clear", "inspect"] {
        let params = action(&engine, tag);
        failed(&mut engine, "ui.selection", params, "close_committed")?;
    }
    let params = motion(&engine, 0.0, 0.0)?;
    failed(
        &mut engine,
        "selection.translate",
        params,
        "close_committed",
    )?;
    Ok(())
}

fn atom(id: u32, x: f64, y: f64) -> TestResult<Atom> {
    Ok(Atom::new(
        AtomId::new(id).ok_or("atom id")?,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(x, y)?),
    ))
}

#[test]
fn sparse_multifragment_motion_changes_only_selected_positions_and_keeps_every_metadata_field()
-> TestResult {
    let mut first = atom(29, 0.125, -10.75)?;
    first.isotope = NonZeroU16::new(13);
    first.charge = -1;
    first.radical = Radical::Doublet;
    let mut second = atom(11, 48.125, -10.75)?;
    second.element = Element::new(8).ok_or("O")?;
    second.explicit_hydrogens = Some(0);
    let mut unrelated = atom(91, 500.0, 500.0)?;
    unrelated.element = Element::new(118).ok_or("Og")?;
    unrelated.radical = Radical::Triplet;
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![
            Fragment::new(
                FragmentId::new(17).ok_or("fragment")?,
                vec![first, second],
                vec![Bond {
                    double_position: bondline_model::DoublePosition::Unspecified,
                    id: BondId::new(41).ok_or("bond")?,
                    begin: AtomId::new(11).ok_or("begin")?,
                    end: AtomId::new(29).ok_or("end")?,
                    order: BondOrder::Dative,
                }],
            )?,
            Fragment::new(
                FragmentId::new(7).ok_or("fragment")?,
                vec![unrelated],
                vec![],
            )?,
            Fragment::new(
                FragmentId::new(31).ok_or("fragment")?,
                vec![atom(401, 220.0, 72.0)?, atom(407, 268.0, 72.0)?],
                vec![Bond {
                    double_position: bondline_model::DoublePosition::Unspecified,
                    id: BondId::new(655).ok_or("bond")?,
                    begin: AtomId::new(407).ok_or("begin")?,
                    end: AtomId::new(401).ok_or("end")?,
                    order: BondOrder::Aromatic,
                }],
            )?,
        ],
    )?;
    let mut engine = Engine::from_document(stored);
    let original = engine.content().clone();
    let untouched = Arc::clone(&engine.document()?.pages()[0].fragments()[1]);
    let mut expected = serde_json::to_value(engine.document()?)?;
    expected["pages"][0]["fragments"][0]["atoms"][0]["position"] =
        json!({"x_pt":24.125,"y_pt":-22.75});
    expected["pages"][0]["fragments"][2]["atoms"][1]["position"] =
        json!({"x_pt":292.0,"y_pt":60.0});
    select(&mut engine, &[407, 29])?;
    let old_epoch = epoch(&engine)?;
    let moved = move_selected(&mut engine, 24.0, -12.0)?;
    assert_eq!(moved["document"], expected);
    assert_eq!(moved["revision"], 1);
    assert_eq!(moved["undo_depth"], 1);
    assert_eq!(moved["redo_depth"], 0);
    assert_eq!(
        selected(&engine)?,
        json!({"active":true,"epoch":old_epoch+1,
        "page_id":1,"atom_ids":[29,407]})
    );
    assert!(Arc::ptr_eq(
        &untouched,
        &engine.document()?.pages()[0].fragments()[1]
    ));
    assert!(moved.get("file_operation").is_none());
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert_eq!(selected(&engine)?["atom_ids"], json!([]));
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, expected);
    let mut absent = atom(1, 72.0, 72.0)?;
    absent.position = None;
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![absent],
            vec![],
        )?],
    )?;
    let mut engine = Engine::from_document(stored);
    select(&mut engine, &[1])?;
    for (x, y) in [(0.0, 0.0), (1.0, 1.0)] {
        let params = motion(&engine, x, y)?;
        failed(
            &mut engine,
            "selection.translate",
            params,
            "unsupported_content",
        )?;
    }
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![atom(1, -100_000.0, -100_000.0)?],
            vec![],
        )?],
    )?;
    let mut engine = Engine::from_document(stored);
    select(&mut engine, &[1])?;
    move_selected(&mut engine, 200_000.0, 200_000.0)?;
    assert_eq!(
        engine.document()?.pages()[0].fragments()[0].atoms()[0].position,
        Some(Point2::new(100_000.0, 100_000.0)?)
    );
    let params = motion(&engine, 0.001, 0.0)?;
    failed(&mut engine, "selection.translate", params, "invalid_params")?;
    Ok(())
}

fn python() -> TestResult<std::ffi::OsString> {
    std::env::var_os("BONDLINE_PYTHON").ok_or_else(|| "BONDLINE_PYTHON is required".into())
}

fn expat(raw: &[u8]) -> TestResult<Value> {
    let mut child = Process::new(python()?)
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../xtask/oracles/cdxml_graph.py"
            ),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("Expat stdin")?.write_all(raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn consume_moved_co_cc(path: &Path) -> TestResult {
    let raw = fs::read(path)?;
    let graph = expat(&raw)?;
    assert_eq!(
        graph,
        json!([
            {"id":1,"atoms":[
                {"id":1,"element":6,"position":{"x_pt":96.0,"y_pt":60.0},"isotope":null,
                    "charge":0,"radical":"none","explicit_hydrogens":3},
                {"id":2,"element":8,"position":{"x_pt":144.0,"y_pt":60.0},"isotope":null,
                    "charge":0,"radical":"none","explicit_hydrogens":1}],
                "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]},
            {"id":2,"atoms":[
                {"id":3,"element":6,"position":{"x_pt":220.0,"y_pt":72.0},"isotope":null,
                    "charge":0,"radical":"none","explicit_hydrogens":3},
                {"id":4,"element":6,"position":{"x_pt":268.0,"y_pt":72.0},"isotope":null,
                    "charge":0,"radical":"none","explicit_hydrogens":3}],
                "bonds":[{"id":2,"begin":3,"end":4,"order":"single"}]}
        ])
    );
    let script = r#"
import sys,xml.parsers.expat as expat
from decimal import Decimal
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=sys.stdin.buffer.read(); positions=[]
p=expat.ParserCreate()
p.StartElementHandler=lambda name,a: positions.append(tuple(map(Decimal,a['p'].split()))) if name=='n' else None
p.Parse(raw,True)
assert positions==[(Decimal('96'),Decimal('60')),(Decimal('144'),Decimal('60')),(Decimal('220'),Decimal('72')),(Decimal('268'),Decimal('72'))]
ms=Chem.MolsFromCDXML(raw.decode('ascii')); assert len(ms)==2
assert [[(a.GetAtomicNum(),a.GetTotalNumHs(),a.GetFormalCharge(),a.GetIsotope(),a.GetNumRadicalElectrons()) for a in m.GetAtoms()] for m in ms]==[[(6,3,0,0,0),(8,1,0,0,0)],[(6,3,0,0,0),(6,3,0,0,0)]]
assert [[(b.GetBeginAtomIdx(),b.GetEndAtomIdx(),b.GetBondTypeAsDouble()) for b in m.GetBonds()] for m in ms]==[[(0,1,1.0)],[(0,1,1.0)]]
assert [Chem.MolToSmiles(m) for m in ms]==['CO','CC']
assert [rdMolDescriptors.CalcMolFormula(m) for m in ms]==['CH4O','C2H6']
"#;
    let mut child = Process::new(python()?)
        .args(["-B", "-c", script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("RDKit stdin")?.write_all(&raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn literal_geometry_one_history_step_unicode_save_reopen_and_validated_native_noops() -> TestResult
{
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("translated-化学-λ.cdxml");
    let mut engine = co_and_cc()?;
    let original = engine.content().clone();
    let unchanged = Arc::clone(&engine.document()?.pages()[0].fragments()[1]);
    let mut expected = serde_json::to_value(engine.document()?)?;
    expected["pages"][0]["fragments"][0]["atoms"][0]["position"] = json!({"x_pt":96.0,"y_pt":60.0});
    expected["pages"][0]["fragments"][0]["atoms"][1]["position"] =
        json!({"x_pt":144.0,"y_pt":60.0});
    select(&mut engine, &[2, 1])?;
    let selected_epoch = epoch(&engine)?;
    let result = move_selected(&mut engine, 24.0, -12.0)?;
    assert_eq!(result["document"], expected);
    assert_eq!(
        (
            result["revision"].as_u64(),
            result["undo_depth"].as_u64(),
            result["redo_depth"].as_u64()
        ),
        (Some(4), Some(4), Some(0))
    );
    assert_eq!(
        selected(&engine)?,
        json!({"active":true,"epoch":selected_epoch+1,
        "page_id":1,"atom_ids":[1,2]})
    );
    assert!(Arc::ptr_eq(
        &unchanged,
        &engine.document()?.pages()[0].fragments()[1]
    ));
    assert!(!path.exists());
    let moved_owner = engine.capture_stamp();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert_eq!(selected(&engine)?["atom_ids"], json!([]));
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, expected);
    assert_ne!(
        engine.capture_stamp(),
        moved_owner,
        "history cannot revive old selection authority"
    );
    select(&mut engine, &[1, 2])?;
    let before_save = selected(&engine)?;
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    assert_eq!(
        selected(&engine)?,
        before_save,
        "Save does not clear selection"
    );
    assert!(!engine.state().file.dirty);
    consume_moved_co_cc(&path)?;
    let saved = fs::read(&path)?;
    engine.dispatch("document.open_cdxml", json!({"path":path}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, expected);
    assert!(engine.stored_document()?.is_fully_mapped());
    assert_eq!(selected(&engine)?["atom_ids"], json!([]));
    assert_eq!(fs::read(&path)?, saved);

    let mut native =
        Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    native.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    native.dispatch("history.undo", json!({}))?;
    select(&mut native, &[1, 2])?;
    native.dispatch(
        "document.close",
        json!({"expected_revision":native.state().revision,"action":"request"}),
    )?;
    assert_eq!(native.state().close, CloseState::Approved);
    let before = wire(&native)?;
    let stamp = native.capture_stamp();
    let file = native.ui_file_context_stamp();
    let model = native.stored_document()?.model_snapshot();
    let source = native.stored_document()?.export_cdxml()?.as_ptr();
    for (x, y) in [(0.0, 0.0), (1e-30, -1e-30)] {
        assert_eq!(move_selected(&mut native, x, y)?["changed"], false);
        assert_eq!(wire(&native)?, before);
        assert_eq!(native.capture_stamp(), stamp);
        assert!(native.ui_file_context_stamp_matches(&file));
        assert!(Arc::ptr_eq(
            &model,
            &native.stored_document()?.model_snapshot()
        ));
        assert_eq!(native.stored_document()?.export_cdxml()?.as_ptr(), source);
        assert_eq!(native.stored_document()?.export_cdxml()?, NATIVE);
    }
    Ok(())
}

#[test]
fn complete_foreign_sources_refuse_even_selection_activation_without_erasing_data() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let opaque = String::from_utf8(NATIVE.to_vec())?.replace(
        "<CDXML ",
        "<CDXML PrivateSelectionTest=\"original-complete-source\" ",
    );
    let mut engine = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    assert!(!engine.stored_document()?.is_fully_mapped());
    let params = action(&engine, "activate");
    failed(&mut engine, "ui.selection", params, "unsupported_content")?;
    assert_eq!(engine.stored_document()?.export_cdxml()?, opaque.as_bytes());
    for (name, source, open) in [
        (
            "whole-v2.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "whole-v3.mol",
            include_bytes!("../../io/tests/fixtures/v3000/noncontiguous-cc.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "whole.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
        ),
        (
            "whole.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        ("whole.smiles", RETAINED_SMILES, "document.open_smiles"),
    ] {
        let path = directory.path(name);
        fs::write(&path, source)?;
        let mut engine = co_and_cc()?;
        select(&mut engine, &[1, 2])?;
        let old_epoch = epoch(&engine)?;
        engine.dispatch(open, json!({"path":path,"discard_unsaved":true}))?;
        assert_eq!(
            selected(&engine)?,
            json!({"active":true,"epoch":old_epoch+1,
            "page_id":null,"atom_ids":[]})
        );
        for params in [
            action(&engine, "activate"),
            set_params(&engine, &[1]),
            json!({"action":"all","expected_revision":engine.state().revision,"page_id":1}),
        ] {
            failed(&mut engine, "ui.selection", params, "unsupported_content")?;
        }
        let before = wire(&engine)?;
        assert_eq!(
            named_selection(&mut engine, "inspect")?["ui_selection"],
            selected(&engine)?
        );
        assert_eq!(
            named_selection(&mut engine, "clear")?["ui_selection_operation"]["changed"],
            false
        );
        assert_eq!(wire(&engine)?, before);
        let mut params = action(&engine, "activate");
        params["expected_revision"] = json!(0);
        failed(&mut engine, "ui.selection", params, "stale_revision")?;
        assert_eq!(fs::read(&path)?, source);
        if open == "document.open_smiles" {
            engine.dispatch(
                "document.import_smiles_records",
                json!({"expected_revision":engine.state().revision,
                "selection":{"mode":"all"},"bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,
                "page_width_pt":612,"page_height_pt":792}),
            )?;
            assert!(engine.stored_document()?.has_smiles_projection());
            for params in [
                action(&engine, "activate"),
                set_params(&engine, &[1]),
                json!({"action":"all","expected_revision":engine.state().revision,"page_id":1}),
            ] {
                failed(&mut engine, "ui.selection", params, "unsupported_content")?;
            }
            assert_eq!(engine.stored_document()?.smiles_file()?.as_bytes(), source);
            assert_eq!(fs::read(&path)?, source);
        }
    }
    Ok(())
}

#[test]
fn only_real_authored_changes_clear_targets_and_old_ids_or_epochs_never_revive() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("replacement-原.cdxml");
    fs::write(&path, NATIVE)?;
    for edit in [
        "label", "bond", "create", "page", "undo", "redo", "new", "open",
    ] {
        let mut engine = co_and_cc()?;
        if edit == "redo" {
            engine.dispatch(
                "page.set_size",
                json!({"page_id":1,"width_pt":400,"height_pt":600}),
            )?;
            engine.dispatch("history.undo", json!({}))?;
        }
        select(&mut engine, &[1, 2])?;
        let old_epoch = epoch(&engine)?;
        let tool = engine.bond_tool();
        match edit {
            "label" => {
                engine.dispatch(
                    "atom.set_label",
                    json!({"expected_revision":engine.state().revision,
                "page_id":1,"fragment_id":1,"atom_id":2,"element":7,"explicit_hydrogens":2}),
                )?;
            }
            "bond" => {
                engine.dispatch("bond.set_order",json!({"expected_revision":engine.state().revision,
                "page_id":1,"fragment_id":2,"bond_id":2,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}))?;
            }
            "create" => {
                engine.dispatch(
                    "bond.create",
                    json!({"page_id":1,"begin_x_pt":350,"begin_y_pt":72,
                "end_x_pt":398,"end_y_pt":72}),
                )?;
            }
            "page" | "redo" => {
                if edit == "page" {
                    engine.dispatch(
                        "page.set_size",
                        json!({"page_id":1,"width_pt":400,"height_pt":600}),
                    )?;
                } else {
                    engine.dispatch("history.redo", json!({}))?;
                }
            }
            "undo" => {
                engine.dispatch("history.undo", json!({}))?;
            }
            "new" => {
                engine.dispatch(
                    "document.new",
                    json!({"expected_revision":engine.state().revision,"discard_unsaved":true}),
                )?;
            }
            _ => {
                engine.dispatch(
                    "document.open_cdxml",
                    json!({"path":path,"discard_unsaved":true}),
                )?;
            }
        }
        assert_eq!(
            selected(&engine)?,
            json!({"active":true,"epoch":old_epoch+1,"page_id":null,"atom_ids":[]})
        );
        assert_eq!(
            engine.bond_tool(),
            tool,
            "authored replacement does not choose a different tool"
        );
        let params = json!({"expected_revision":engine.state().revision,"expected_selection_epoch":old_epoch,
            "delta_x_pt":1,"delta_y_pt":0});
        failed(
            &mut engine,
            "selection.translate",
            params,
            "stale_selection",
        )?;
    }
    let mut engine = co_and_cc()?;
    select(&mut engine, &[1, 2])?;
    let first_epoch = epoch(&engine)?;
    let author = author_state(&engine)?;
    engine.set_view_canvas_size(848, 728)?;
    engine.dispatch("ui.view", json!({"action":"actual"}))?;
    engine.dispatch(
        "ui.view",
        json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
    )?;
    assert_eq!(epoch(&engine)?, first_epoch);
    assert_eq!(selected(&engine)?["atom_ids"], json!([1, 2]));
    assert_eq!(author_state(&engine)?, author);
    let before = wire(&engine)?;
    assert!(
        !engine
            .dispatch(
                "atom.set_label",
                json!({"expected_revision":engine.state().revision,"page_id":1,
        "fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1})
            )?
            .changed
    );
    assert_eq!(wire(&engine)?, before);
    select(&mut engine, &[3, 4])?;
    select(&mut engine, &[2, 1])?;
    assert_eq!(selected(&engine)?["atom_ids"], json!([1, 2]));
    assert_eq!(epoch(&engine)?, first_epoch + 2);
    let params = json!({"expected_revision":engine.state().revision,"expected_selection_epoch":first_epoch,
        "delta_x_pt":1,"delta_y_pt":0});
    failed(
        &mut engine,
        "selection.translate",
        params,
        "stale_selection",
    )?;
    Ok(())
}

#[test]
fn captured_save_retains_its_lease_and_historical_snapshot_without_clearing_current_selection()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let captured = directory.path("before-move.cdxml");
    let next = directory.path("after-move.cdxml");
    let mut engine = co_and_cc()?;
    select(&mut engine, &[1, 2])?;
    let old_document = serde_json::to_value(engine.document()?)?;
    let captured_revision = engine.state().revision;
    let save = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":captured}),
    )?)?;
    move_selected(&mut engine, 24.0, -12.0)?;
    let current_document = serde_json::to_value(engine.document()?)?;
    let selection = selected(&engine)?;
    let completed = save.complete()?;
    assert_eq!(
        expat(&fs::read(&captured)?)?,
        old_document["pages"][0]["fragments"]
    );
    let before = wire(&engine)?;
    let error = engine
        .prepare(Command::parse("document.save_cdxml", json!({"path":next}))?)
        .err()
        .ok_or("published Save lost its lease before finish")?;
    assert_eq!(error.info().code, "busy");
    assert_eq!(wire(&engine)?, before);
    assert!(!next.exists());
    let receipt = engine.finish(completed)?;
    let operation = receipt.file_operation.ok_or("actual Save receipt")?;
    assert_eq!(operation.input_revision, captured_revision);
    assert!(operation.ui_request_id.is_none() && operation.ui_request_current.is_none());
    assert_eq!(operation.byte_length, fs::read(&captured)?.len());
    assert_eq!(serde_json::to_value(engine.document()?)?, current_document);
    assert_eq!(selected(&engine)?, selection);
    assert!(
        engine.state().file.dirty,
        "captured old baseline cannot clean the moved graph"
    );
    assert_eq!(
        engine.state().file.path.as_deref(),
        fs::canonicalize(&captured)?.to_str()
    );
    engine.dispatch("document.save_cdxml", json!({"path":next}))?;
    assert!(!engine.state().file.dirty);
    assert_eq!(selected(&engine)?, selection);
    consume_moved_co_cc(&next)?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, old_document);
    assert!(engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, current_document);
    assert!(!engine.state().file.dirty);
    Ok(())
}
