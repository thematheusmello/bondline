//! Independent ring authoring through existing JSON dispatch, before product code.
use bondline_commands::Engine;
use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn literal_six_ring_is_one_source_free_history_edit_with_fixed_h_and_directed_closure() -> TestResult
{
    let mut engine = Engine::default();
    let result = engine.dispatch(
        "ring.create",
        json!({"expected_revision":0,"page_id":1,"size":6,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{
            "mode":"free","begin_x_pt":120,"begin_y_pt":168,
            "end_x_pt":120,"end_y_pt":120,"side":"positive"}}),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (1, 1, 0)
    );
    assert!(result.source.is_none());
    assert!(result.file.path.is_none() && result.file.dirty);
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    let value = serde_json::to_value(result)?;
    let fragments = value["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0]["id"], 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    let bonds = fragments[0]["bonds"].as_array().ok_or("bonds")?;
    assert_eq!((atoms.len(), bonds.len()), (6, 6));
    let r = 3.0_f64.sqrt();
    let expected = [
        (120.0, 168.0),
        (120.0, 120.0),
        (120.0 + 24.0 * r, 96.0),
        (120.0 + 48.0 * r, 120.0),
        (120.0 + 48.0 * r, 168.0),
        (120.0 + 24.0 * r, 192.0),
    ];
    for (i, (atom, (x, y))) in atoms.iter().zip(expected).enumerate() {
        assert_eq!(atom["id"], i + 1);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], 2);
        assert_eq!(atom["isotope"], Value::Null);
        assert_eq!(atom["charge"], 0);
        assert_eq!(atom["radical"], "none");
        assert!((atom["position"]["x_pt"].as_f64().ok_or("x")? - x).abs() <= 1e-9);
        assert!((atom["position"]["y_pt"].as_f64().ok_or("y")? - y).abs() <= 1e-9);
    }
    for (i, bond) in bonds.iter().enumerate() {
        assert_eq!(
            bond,
            &json!({"id":i+1,"begin":i+1,
            "end":if i==5 {1} else {i+2},"order":"single"})
        );
    }
    Ok(())
}

// First additive stage: original model inputs make each attachment the first new dispatch.
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, Point2,
};

fn original_hexagon() -> TestResult<Engine> {
    let r = 3.0_f64.sqrt();
    let points = [
        (120.0, 168.0),
        (120.0, 120.0),
        (120.0 + 24.0 * r, 96.0),
        (120.0 + 48.0 * r, 120.0),
        (120.0 + 48.0 * r, 168.0),
        (120.0 + 24.0 * r, 192.0),
    ];
    let atoms = points
        .into_iter()
        .enumerate()
        .map(|(i, (x, y))| {
            let mut a = Atom::new(
                AtomId::new((i + 1) as u32).ok_or("id")?,
                Element::new(6).ok_or("C")?,
                Some(Point2::new(x, y)?),
            );
            a.explicit_hydrogens = Some(2);
            Ok(a)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = (1..=6_u32)
        .map(|i| {
            Ok(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(i).ok_or("bond")?,
                begin: AtomId::new(i).ok_or("begin")?,
                end: AtomId::new(if i == 6 { 1 } else { i + 1 }).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(Engine::from_document(stored))
}

fn ring_params(engine: &Engine, size: u8, placement: Value) -> Value {
    json!({"expected_revision":engine.state().revision,"page_id":1,"size":size,
        "hydrogen_policy":"rectify_neutral_carbon","placement":placement})
}

fn assert_carbon_fragment(
    fragment: &Value,
    h: &[u16],
    points: &[[f64; 2]],
    edges: &[[u32; 2]],
) -> TestResult {
    let atoms = fragment["atoms"].as_array().ok_or("atoms")?;
    let bonds = fragment["bonds"].as_array().ok_or("bonds")?;
    assert_eq!(atoms.len(), h.len());
    assert_eq!(atoms.len(), points.len());
    assert_eq!(bonds.len(), edges.len());
    for (i, ((atom, &h), p)) in atoms.iter().zip(h).zip(points).enumerate() {
        assert_eq!(atom["id"], i + 1);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], h);
        assert_eq!(atom["isotope"], Value::Null);
        assert_eq!(atom["charge"], 0);
        assert_eq!(atom["radical"], "none");
        for (axis, want) in ["x_pt", "y_pt"].into_iter().zip(p) {
            assert!((atom["position"][axis].as_f64().ok_or("coordinate")? - want).abs() <= 1e-9);
        }
    }
    for (i, (bond, [a, b])) in bonds.iter().zip(edges).enumerate() {
        assert_eq!(bond, &json!({"id":i+1,"begin":a,"end":b,"order":"single"}));
    }
    Ok(())
}

#[test]
fn shared_single_edge_adds_literal_decalin_with_preserved_edge_and_anchors() -> TestResult {
    let mut engine = original_hexagon()?;
    let original = engine.content().clone();
    let old = serde_json::to_value(engine.document()?)?;
    let params = ring_params(
        &engine,
        6,
        json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"negative"}),
    );
    let result = engine.dispatch("ring.create", params)?;
    assert!(result.changed);
    assert_eq!((result.revision, result.undo_depth), (1, 1));
    let doc = serde_json::to_value(engine.document()?)?;
    let f = &doc["pages"][0]["fragments"][0];
    assert_eq!(
        doc["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        1
    );
    assert_eq!(
        &f["bonds"].as_array().ok_or("bonds")?[..6],
        old["pages"][0]["fragments"][0]["bonds"]
            .as_array()
            .ok_or("old bonds")?
    );
    let r = 3.0_f64.sqrt();
    let points = [
        [120.0, 168.0],
        [120.0, 120.0],
        [120.0 + 24.0 * r, 96.0],
        [120.0 + 48.0 * r, 120.0],
        [120.0 + 48.0 * r, 168.0],
        [120.0 + 24.0 * r, 192.0],
        [120.0 - 24.0 * r, 96.0],
        [120.0 - 48.0 * r, 120.0],
        [120.0 - 48.0 * r, 168.0],
        [120.0 - 24.0 * r, 192.0],
    ];
    assert_carbon_fragment(
        f,
        &[1, 1, 2, 2, 2, 2, 2, 2, 2, 2],
        &points,
        &[
            [1, 2],
            [2, 3],
            [3, 4],
            [4, 5],
            [5, 6],
            [6, 1],
            [2, 7],
            [7, 8],
            [8, 9],
            [9, 10],
            [10, 1],
        ],
    )?;
    for i in 0..6 {
        assert_eq!(
            f["atoms"][i]["position"],
            old["pages"][0]["fragments"][0]["atoms"][i]["position"]
        );
    }
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, doc);
    Ok(())
}

#[test]
fn shared_atom_adds_literal_spiro_ring_and_consumes_exactly_two_hydrogens() -> TestResult {
    let mut engine = original_hexagon()?;
    let old = serde_json::to_value(engine.document()?)?;
    let params = ring_params(
        &engine,
        6,
        json!({"mode":"atom","fragment_id":1,"atom_id":1,
        "end_x_pt":120,"end_y_pt":216,"side":"positive"}),
    );
    let result = engine.dispatch("ring.create", params)?;
    assert!(result.changed);
    assert_eq!((result.revision, result.undo_depth), (1, 1));
    let doc = serde_json::to_value(engine.document()?)?;
    let f = &doc["pages"][0]["fragments"][0];
    let r = 3.0_f64.sqrt();
    let points = [
        [120.0, 168.0],
        [120.0, 120.0],
        [120.0 + 24.0 * r, 96.0],
        [120.0 + 48.0 * r, 120.0],
        [120.0 + 48.0 * r, 168.0],
        [120.0 + 24.0 * r, 192.0],
        [120.0, 216.0],
        [120.0 - 24.0 * r, 240.0],
        [120.0 - 48.0 * r, 216.0],
        [120.0 - 48.0 * r, 168.0],
        [120.0 - 24.0 * r, 144.0],
    ];
    assert_carbon_fragment(
        f,
        &[0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        &points,
        &[
            [1, 2],
            [2, 3],
            [3, 4],
            [4, 5],
            [5, 6],
            [6, 1],
            [1, 7],
            [7, 8],
            [8, 9],
            [9, 10],
            [10, 11],
            [11, 1],
        ],
    )?;
    assert_eq!(
        &f["bonds"].as_array().ok_or("bonds")?[..6],
        old["pages"][0]["fragments"][0]["bonds"]
            .as_array()
            .ok_or("old bonds")?
    );
    for i in 0..6 {
        assert_eq!(
            f["atoms"][i]["position"],
            old["pages"][0]["fragments"][0]["atoms"][i]["position"]
        );
    }
    Ok(())
}

#[test]
fn terminal_atom_attachment_keeps_the_original_ethane_bond_and_literal_methyl_group() -> TestResult
{
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,
        "end_x_pt":120,"end_y_pt":72}),
    )?;
    let r = 3.0_f64.sqrt();
    let params = ring_params(
        &engine,
        6,
        json!({"mode":"atom","fragment_id":1,"atom_id":2,
        "end_x_pt":144.0,"end_y_pt":72.0-24.0*r,"side":"positive"}),
    );
    engine.dispatch("ring.create", params)?;
    let doc = serde_json::to_value(engine.document()?)?;
    assert_carbon_fragment(
        &doc["pages"][0]["fragments"][0],
        &[3, 1, 2, 2, 2, 2, 2],
        &[
            [72.0, 72.0],
            [120.0, 72.0],
            [144.0, 72.0 - 24.0 * r],
            [192.0, 72.0 - 24.0 * r],
            [216.0, 72.0],
            [192.0, 72.0 + 24.0 * r],
            [144.0, 72.0 + 24.0 * r],
        ],
        &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7], [7, 2]],
    )?;
    assert_eq!((engine.state().revision, engine.state().undo_depth), (2, 2));
    Ok(())
}

// Final additive independent checks. The initial and first-placement stage stay byte-exact.
use bondline_commands::{CloseState, Command};
use bondline_model::{PageSize, Radical};
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    num::NonZeroU16,
    path::Path,
    process::{Command as Process, Stdio},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the existing isolated directory and file consumer"
)]
mod support;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const SMILES_SOURCE: &[u8] = b"CC first\r\nO\topaque\xff\0";

fn wire(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.state())?)
}
fn free_json(a: [f64; 2], b: [f64; 2], side: &str) -> Value {
    json!({"mode":"free","begin_x_pt":a[0],"begin_y_pt":a[1],
        "end_x_pt":b[0],"end_y_pt":b[1],"side":side})
}
fn refuse(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = wire(engine)?;
    let owner = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let tool = engine.bond_tool();
    let view = engine.view();
    let generation = engine.session_generation();
    let file = engine.ui_file_context_stamp();
    let error = engine
        .dispatch("ring.create", params)
        .err()
        .ok_or("invalid ring accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(wire(engine)?, before);
    assert_eq!(engine.content(), &owner);
    assert_eq!(engine.capture_stamp(), stamp);
    assert_eq!(engine.bond_tool(), tool);
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
fn all_small_sizes_and_both_sides_follow_independent_circle_and_angle_geometry() -> TestResult {
    // Circumcenter/radius construction is independent of a sequential edge-turn implementation.
    for n in 3..=8_u8 {
        for side in ["positive", "negative"] {
            for b in [
                [288.0, 240.0],
                [240.0, 192.0],
                [264.0, 240.0 + 24.0 * 3.0_f64.sqrt()],
            ] {
                let a = [240.0, 240.0];
                let mut engine = Engine::default();
                let params = ring_params(&engine, n, free_json(a, b, side));
                let before = engine.content().clone();
                engine.dispatch("ring.create", params)?;
                let doc = serde_json::to_value(engine.document()?)?;
                let f = &doc["pages"][0]["fragments"][0];
                let atoms = f["atoms"].as_array().ok_or("atoms")?;
                let bonds = f["bonds"].as_array().ok_or("bonds")?;
                assert_eq!((atoms.len(), bonds.len()), (usize::from(n), usize::from(n)));
                assert_eq!(atoms[0]["position"], json!({"x_pt":a[0],"y_pt":a[1]}));
                assert_eq!(atoms[1]["position"], json!({"x_pt":b[0],"y_pt":b[1]}));
                let sign = if side == "positive" { 1.0 } else { -1.0 };
                let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                let length = dx.hypot(dy);
                let alpha = std::f64::consts::PI / f64::from(n);
                let apothem = length / (2.0 * alpha.tan());
                let radius = length / (2.0 * alpha.sin());
                let center = [
                    (a[0] + b[0]) / 2.0 - sign * dy / length * apothem,
                    (a[1] + b[1]) / 2.0 + sign * dx / length * apothem,
                ];
                let phase = (a[1] - center[1]).atan2(a[0] - center[0]);
                let points = atoms
                    .iter()
                    .map(|atom| {
                        Ok([
                            atom["position"]["x_pt"].as_f64().ok_or("x")?,
                            atom["position"]["y_pt"].as_f64().ok_or("y")?,
                        ])
                    })
                    .collect::<TestResult<Vec<_>>>()?;
                for (i, p) in points.iter().enumerate() {
                    let theta = phase + sign * std::f64::consts::TAU * i as f64 / f64::from(n);
                    let expected = [
                        center[0] + radius * theta.cos(),
                        center[1] + radius * theta.sin(),
                    ];
                    assert!(
                        (p[0] - expected[0]).abs() <= 1e-9 && (p[1] - expected[1]).abs() <= 1e-9
                    );
                    let next = points[(i + 1) % points.len()];
                    let previous = points[(i + points.len() - 1) % points.len()];
                    assert!(((next[0] - p[0]).hypot(next[1] - p[1]) - length).abs() <= 1e-9);
                    let u = [previous[0] - p[0], previous[1] - p[1]];
                    let v = [next[0] - p[0], next[1] - p[1]];
                    let angle = ((u[0] * v[0] + u[1] * v[1])
                        / (u[0].hypot(u[1]) * v[0].hypot(v[1])))
                    .acos();
                    assert!((angle - (f64::from(n) - 2.0) * alpha).abs() <= 1e-10);
                    assert_eq!(atoms[i]["explicit_hydrogens"], 2);
                    assert_eq!(atoms[i]["id"], i + 1);
                    assert_eq!(
                        bonds[i],
                        json!({"id":i+1,"begin":i+1,
                    "end":if i+1==points.len(){1}else{i+2},"order":"single"})
                    );
                    if i >= 2 {
                        assert!(sign * (dx * (p[1] - a[1]) - dy * (p[0] - a[0])) > 0.0);
                    }
                }
                engine.dispatch("history.undo", json!({}))?;
                assert_eq!(engine.content(), &before);
            }
        }
    }
    Ok(())
}

fn schema_node<'a>(root: &'a Value, node: &'a Value) -> TestResult<&'a Value> {
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        let pointer = reference
            .strip_prefix('#')
            .ok_or("local schema reference")?;
        root.pointer(pointer)
            .ok_or_else(|| "schema reference target".into())
    } else {
        Ok(node)
    }
}

#[test]
fn one_closed_registry_addition_and_invalid_inputs_are_atomic() -> TestResult {
    // All36 existing names are retained as a literal declaration, then exactly one is added.
    const OLD_36: [&str; 36] = [
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
    assert_eq!(registry.len(), 39);
    let mut expected = OLD_36.into_iter().collect::<BTreeSet<_>>();
    assert!(expected.insert("ring.create"));
    assert!(expected.insert("ui.ring_tool"));
    assert!(expected.insert("chain.create"));
    assert_eq!(
        registry.iter().map(|r| r.name).collect::<BTreeSet<_>>(),
        expected
    );
    let entry = registry
        .iter()
        .find(|r| r.name == "ring.create")
        .ok_or("ring registry")?;
    assert!(!entry.read_only);
    let root = serde_json::to_value(&entry.input_schema)?;
    // Root remains an object union; putting additionalProperties:false here would
    // reject every branch field because this wrapper defines no properties.
    assert_eq!(root["type"], "object");
    assert!(root.get("additionalProperties").is_none());
    let variants = root["anyOf"].as_array().ok_or("ring selector variants")?;
    assert_eq!(variants.len(), 2);
    let mut sized = None;
    let mut named = None;
    for raw in variants {
        let branch = schema_node(&root, raw)?;
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let fields = branch["properties"].as_object().ok_or("selector fields")?;
        let required = branch["required"]
            .as_array()
            .ok_or("selector required")?
            .iter()
            .map(|field| field.as_str().ok_or("selector field"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if fields.contains_key("size") {
            assert!(sized.replace(branch).is_none(), "duplicate sized branch");
            assert!(required.contains("size"));
            assert!(!fields.contains_key("kind") && !fields.contains_key("phase"));
            assert!(!required.contains("kind"));
        } else {
            assert!(named.replace(branch).is_none(), "duplicate named branch");
            assert!(fields.contains_key("kind") && required.contains("kind"));
            assert!(!fields.contains_key("size") && !required.contains("size"));
        }
    }
    // Closedness plus opposite required/absent selectors makes the branches disjoint.
    named.ok_or("missing named branch")?;
    let schema = sized.ok_or("missing exact legacy sized branch")?;
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    let required = [
        "expected_revision",
        "page_id",
        "size",
        "hydrogen_policy",
        "placement",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        schema["properties"]
            .as_object()
            .ok_or("root fields")?
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        required
    );
    assert_eq!(
        schema["required"]
            .as_array()
            .ok_or("root required")?
            .iter()
            .map(|v| v.as_str().ok_or("field"))
            .collect::<Result<BTreeSet<_>, _>>()?,
        required
    );
    assert_eq!(schema["properties"]["size"]["minimum"].as_f64(), Some(3.0));
    assert_eq!(schema["properties"]["size"]["maximum"].as_f64(), Some(8.0));
    assert_eq!(
        schema["properties"]["page_id"]["minimum"].as_f64(),
        Some(1.0)
    );
    let placement = schema_node(&root, &schema["properties"]["placement"])?;
    let variants = placement
        .get("oneOf")
        .or_else(|| placement.get("anyOf"))
        .and_then(Value::as_array)
        .ok_or("placement variants")?;
    assert_eq!(variants.len(), 3);
    let mut modes = BTreeSet::new();
    for raw in variants {
        let v = schema_node(&root, raw)?;
        let tag = v["properties"]["mode"]
            .get("const")
            .and_then(Value::as_str)
            .or_else(|| {
                v["properties"]["mode"]
                    .get("enum")?
                    .as_array()?
                    .first()?
                    .as_str()
            })
            .ok_or("mode")?;
        assert!(modes.insert(tag));
        let names = match tag {
            "free" => vec![
                "mode",
                "begin_x_pt",
                "begin_y_pt",
                "end_x_pt",
                "end_y_pt",
                "side",
            ],
            "atom" => vec![
                "mode",
                "fragment_id",
                "atom_id",
                "end_x_pt",
                "end_y_pt",
                "side",
            ],
            "bond" => vec!["mode", "fragment_id", "bond_id", "side"],
            _ => return Err("extra mode".into()),
        }
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(v["additionalProperties"], false);
        assert_eq!(
            v["properties"]
                .as_object()
                .ok_or("variant properties")?
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            names
        );
        assert_eq!(
            v["required"]
                .as_array()
                .ok_or("variant required")?
                .iter()
                .map(|s| s.as_str().ok_or("field"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            names
        );
        let side = schema_node(&root, &v["properties"]["side"])?;
        assert_eq!(side["enum"], json!(["positive", "negative"]));
    }
    assert_eq!(
        modes,
        ["free", "atom", "bond"]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    let mut engine = original_hexagon()?;
    let valid = ring_params(
        &engine,
        6,
        free_json([300.0, 168.0], [300.0, 120.0], "positive"),
    );
    for (key, value, code) in [
        ("expected_revision", json!(1), "stale_revision"),
        ("expected_revision", json!(-1), "invalid_params"),
        ("page_id", json!(0), "invalid_params"),
        ("page_id", json!(9), "not_found"),
        ("page_id", json!(1.5), "invalid_params"),
        ("page_id", json!(u64::MAX), "invalid_params"),
        ("size", json!(2), "invalid_params"),
        ("size", json!(9), "invalid_params"),
        ("size", json!(256), "invalid_params"),
        ("size", json!(6.5), "invalid_params"),
        ("size", Value::Null, "invalid_params"),
        ("hydrogen_policy", json!("infer"), "invalid_params"),
        ("hydrogen_policy", Value::Null, "invalid_params"),
        ("placement", Value::Null, "invalid_params"),
        ("surplus", json!(true), "invalid_params"),
    ] {
        let mut p = valid.clone();
        p[key] = value;
        refuse(&mut engine, p, code)?;
    }
    for key in [
        "expected_revision",
        "page_id",
        "size",
        "hydrogen_policy",
        "placement",
    ] {
        let mut p = valid.clone();
        p.as_object_mut().ok_or("params")?.remove(key);
        refuse(&mut engine, p, "invalid_params")?;
    }
    for placement in [
        free_json([300.0, 168.0], [300.0, 120.0], "positive"),
        json!({"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":120,"end_y_pt":216,"side":"positive"}),
        json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"negative"}),
    ] {
        for key in placement.as_object().ok_or("placement")?.keys() {
            let mut v = placement.clone();
            v.as_object_mut().ok_or("placement")?.remove(key);
            let p = ring_params(&engine, 6, v);
            refuse(&mut engine, p, "invalid_params")?;
        }
        let mut v = placement.clone();
        v["extra"] = json!(0);
        let p = ring_params(&engine, 6, v);
        refuse(&mut engine, p, "invalid_params")?;
        for side in [json!("left"), json!(0), Value::Null] {
            let mut v = placement.clone();
            v["side"] = side;
            let p = ring_params(&engine, 6, v);
            refuse(&mut engine, p, "invalid_params")?;
        }
    }
    for (v, code) in [
        (
            json!({"mode":"bond","fragment_id":0,"bond_id":1,"side":"negative"}),
            "invalid_params",
        ),
        (
            json!({"mode":"bond","fragment_id":9,"bond_id":1,"side":"negative"}),
            "not_found",
        ),
        (
            json!({"mode":"bond","fragment_id":1,"bond_id":0,"side":"negative"}),
            "invalid_params",
        ),
        (
            json!({"mode":"bond","fragment_id":1,"bond_id":99,"side":"negative"}),
            "not_found",
        ),
        (
            json!({"mode":"atom","fragment_id":1,"atom_id":0,"end_x_pt":120,"end_y_pt":216,"side":"positive"}),
            "invalid_params",
        ),
        (
            json!({"mode":"atom","fragment_id":1,"atom_id":99,"end_x_pt":120,"end_y_pt":216,"side":"positive"}),
            "not_found",
        ),
        (
            free_json([300.0, 168.0], [300.0, 168.0], "positive"),
            "invalid_params",
        ),
        (
            free_json([100000.0, 0.0], [100000.0, 48.0], "negative"),
            "invalid_params",
        ),
        (
            free_json([f64::MAX, 0.0], [300.0, 120.0], "positive"),
            "invalid_params",
        ),
        (
            free_json([120.0, 168.0], [168.0, 168.0], "positive"),
            "invalid_params",
        ),
        (
            json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"positive"}),
            "invalid_params",
        ),
        (
            json!({"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":120,"end_y_pt":120,"side":"negative"}),
            "invalid_params",
        ),
    ] {
        let p = ring_params(&engine, 6, v);
        refuse(&mut engine, p, code)?;
    }
    engine.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"discard"}),
    )?;
    assert!(engine.complete_close());
    assert_eq!(engine.state().close, CloseState::Committed);
    let mut p = valid;
    p["expected_revision"] = json!(99);
    refuse(&mut engine, p, "close_committed")?;
    Ok(())
}

#[test]
fn attachments_preserve_sparse_arrays_directions_metadata_and_unrelated_arc() -> TestResult {
    let mut target = Atom::new(
        AtomId::new(29).ok_or("target")?,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(200.0, 200.0)?),
    );
    target.explicit_hydrogens = Some(3);
    let mut neighbor = Atom::new(
        AtomId::new(11).ok_or("neighbor")?,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(248.0, 200.0)?),
    );
    neighbor.explicit_hydrogens = Some(3);
    let mut unrelated = Atom::new(
        AtomId::new(91).ok_or("unrelated")?,
        Element::new(118).ok_or("Og")?,
        Some(Point2::new(500.0, 500.0)?),
    );
    unrelated.isotope = NonZeroU16::new(300);
    unrelated.charge = -2;
    unrelated.radical = Radical::Triplet;
    unrelated.explicit_hydrogens = Some(19);
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![
            Fragment::new(
                FragmentId::new(17).ok_or("fragment")?,
                vec![target, neighbor],
                vec![Bond {
                    double_position: bondline_model::DoublePosition::Unspecified,
                    id: BondId::new(41).ok_or("bond")?,
                    begin: AtomId::new(11).ok_or("begin")?,
                    end: AtomId::new(29).ok_or("end")?,
                    order: BondOrder::Single,
                }],
            )?,
            Fragment::new(
                FragmentId::new(7).ok_or("fragment")?,
                vec![unrelated],
                vec![],
            )?,
        ],
    )?;
    let mut engine = Engine::from_document(stored);
    let old = engine.content().clone();
    let untouched = Arc::clone(&engine.document()?.pages()[0].fragments()[1]);
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[29,91]}),
    )?;
    let selected = wire(&engine)?["ui_selection"].clone();
    let mut expected = serde_json::to_value(engine.document()?)?;
    expected["pages"][0]["fragments"][0]["atoms"][0]["explicit_hydrogens"] = json!(1);
    let r = 3.0_f64.sqrt();
    let new_points = [
        [200.0, 248.0],
        [200.0 - 24.0 * r, 272.0],
        [200.0 - 48.0 * r, 248.0],
        [200.0 - 48.0 * r, 200.0],
        [200.0 - 24.0 * r, 176.0],
    ];
    let atoms = expected["pages"][0]["fragments"][0]["atoms"]
        .as_array_mut()
        .ok_or("atoms")?;
    for (i, [x, y]) in new_points.into_iter().enumerate() {
        atoms.push(json!({"id":92+i,"element":6,
        "position":{"x_pt":x,"y_pt":y},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2}));
    }
    let bonds = expected["pages"][0]["fragments"][0]["bonds"]
        .as_array_mut()
        .ok_or("bonds")?;
    for (i, [a, b]) in [[29, 92], [92, 93], [93, 94], [94, 95], [95, 96], [96, 29]]
        .into_iter()
        .enumerate()
    {
        bonds.push(json!({"id":42+i,"begin":a,"end":b,"order":"single"}));
    }
    let p = ring_params(
        &engine,
        6,
        json!({"mode":"atom","fragment_id":17,"atom_id":29,
        "end_x_pt":200,"end_y_pt":248,"side":"positive"}),
    );
    engine.dispatch("ring.create", p)?;
    let actual = serde_json::to_value(engine.document()?)?;
    compare_graph_positions(
        &actual["pages"][0]["fragments"],
        &expected["pages"][0]["fragments"],
    )?;
    assert!(Arc::ptr_eq(
        &untouched,
        &engine.document()?.pages()[0].fragments()[1]
    ));
    assert_eq!(
        wire(&engine)?["ui_selection"],
        json!({"active":true,
        "epoch":selected["epoch"].as_u64().ok_or("epoch")?+1,"page_id":null,"atom_ids":[]})
    );
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &old);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, actual);
    // An attachment does not consume fragment18. New atom/bond floors include the attachment.
    let p = ring_params(
        &engine,
        3,
        free_json([600.0, 400.0], [648.0, 400.0], "positive"),
    );
    engine.dispatch("ring.create", p)?;
    let f = &engine.document()?.pages()[0].fragments()[2];
    assert_eq!(f.id().get(), 18);
    assert_eq!(
        f.atoms().iter().map(|a| a.id.get()).collect::<Vec<_>>(),
        [97, 98, 99]
    );
    assert_eq!(
        f.bonds().iter().map(|b| b.id.get()).collect::<Vec<_>>(),
        [48, 49, 50]
    );
    Ok(())
}

fn custom_pair(change: &str) -> TestResult<Engine> {
    let mut a = Atom::new(
        AtomId::new(1).ok_or("id")?,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(72.0, 72.0)?),
    );
    a.explicit_hydrogens = Some(3);
    let mut b = Atom::new(
        AtomId::new(2).ok_or("id")?,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(120.0, 72.0)?),
    );
    b.explicit_hydrogens = Some(3);
    let mut order = BondOrder::Single;
    match change {
        "element" => b.element = Element::new(8).ok_or("O")?,
        "charge" => b.charge = 1,
        "isotope" => b.isotope = NonZeroU16::new(13),
        "radical" => b.radical = Radical::Doublet,
        "position" => b.position = None,
        "h-none" => b.explicit_hydrogens = None,
        "h-zero" => b.explicit_hydrogens = Some(0),
        "h-two" => b.explicit_hydrogens = Some(2),
        "h-four" => b.explicit_hydrogens = Some(4),
        "triple" => {
            order = BondOrder::Triple;
            a.explicit_hydrogens = Some(1);
            b.explicit_hydrogens = Some(1);
        }
        "double" => {
            order = BondOrder::Double;
            a.explicit_hydrogens = Some(2);
            b.explicit_hydrogens = Some(2);
        }
        "aromatic" => order = BondOrder::Aromatic,
        _ => return Err("unknown test change".into()),
    }
    let f = Fragment::new(
        FragmentId::new(1).ok_or("fragment")?,
        vec![a, b],
        vec![Bond {
            double_position: bondline_model::DoublePosition::Unspecified,
            id: BondId::new(1).ok_or("bond")?,
            begin: AtomId::new(1).ok_or("a")?,
            end: AtomId::new(2).ok_or("b")?,
            order,
        }],
    )?;
    let mut stored = StoredDocument::default();
    stored
        .model_mut()
        .append_fragments(PageId::new(1).ok_or("page")?, vec![f])?;
    Ok(Engine::from_document(stored))
}

#[test]
fn literal_h_metadata_order_and_whole_model_limits_refuse_without_partial_edits() -> TestResult {
    for change in [
        "element", "charge", "isotope", "radical", "position", "h-none", "h-zero", "h-two",
        "h-four", "triple", "aromatic",
    ] {
        let mut engine = custom_pair(change)?;
        let p = ring_params(
            &engine,
            6,
            json!({"mode":"atom","fragment_id":1,"atom_id":2,
            "end_x_pt":144,"end_y_pt":72.0-24.0*3.0_f64.sqrt(),"side":"positive"}),
        );
        refuse(&mut engine, p, "unsupported_content")?;
    }
    for change in [
        "element", "charge", "isotope", "radical", "position", "h-none", "h-two", "double",
        "triple", "aromatic",
    ] {
        let mut engine = custom_pair(change)?;
        let p = ring_params(
            &engine,
            6,
            json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"positive"}),
        );
        refuse(&mut engine, p, "unsupported_content")?;
    }
    let atoms = (1..=100_000_u32)
        .map(|id| {
            Ok(Atom::new(
                AtomId::new(id).ok_or("id")?,
                Element::new(6).ok_or("C")?,
                None,
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![],
        )?],
    )?;
    let mut engine = Engine::from_document(stored);
    let p = ring_params(
        &engine,
        3,
        free_json([120.0, 168.0], [120.0, 120.0], "positive"),
    );
    refuse(&mut engine, p, "invalid_params")?;
    Ok(())
}

#[test]
fn every_incomplete_or_foreign_owner_remains_complete_and_has_no_ring_authority() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let opaque = String::from_utf8(NATIVE.to_vec())?
        .replace("<CDXML ", "<CDXML RingOwnerTest=\"retained\" ");
    let mut engine = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    let p = ring_params(
        &engine,
        6,
        free_json([300.0, 168.0], [300.0, 120.0], "positive"),
    );
    refuse(&mut engine, p, "unsupported_content")?;
    assert_eq!(engine.stored_document()?.export_cdxml()?, opaque.as_bytes());
    for (name, bytes, command) in [
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
        ("whole.smiles", SMILES_SOURCE, "document.open_smiles"),
    ] {
        let path = directory.path(name);
        fs::write(&path, bytes)?;
        let mut engine = Engine::default();
        engine.dispatch(command, json!({"path":path}))?;
        for placement in [
            free_json([300.0, 168.0], [300.0, 120.0], "positive"),
            json!({"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":300,"end_y_pt":120,"side":"positive"}),
            json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"negative"}),
        ] {
            let mut p = ring_params(&engine, 6, placement.clone());
            p["expected_revision"] = json!(0);
            refuse(&mut engine, p, "stale_revision")?;
            let p = ring_params(&engine, 6, placement);
            refuse(&mut engine, p, "unsupported_content")?;
        }
        assert_eq!(fs::read(&path)?, bytes);
        if command == "document.open_smiles" {
            engine.dispatch("document.import_smiles_records",json!({"expected_revision":engine.state().revision,
                "selection":{"mode":"record","record_id":1},"bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,
                "page_width_pt":595.275590551,"page_height_pt":841.88976378}))?;
            assert!(engine.stored_document()?.has_smiles_projection());
            let p = ring_params(
                &engine,
                6,
                free_json([300.0, 168.0], [300.0, 120.0], "positive"),
            );
            refuse(&mut engine, p, "unsupported_content")?;
            assert_eq!(engine.stored_document()?.smiles_file()?.as_bytes(), bytes);
        }
    }
    Ok(())
}

fn compare_graph_positions(actual: &Value, expected: &Value) -> TestResult {
    let mut a = actual.clone();
    let mut e = expected.clone();
    let aa = a.as_array_mut().ok_or("actual fragments")?;
    let ee = e.as_array_mut().ok_or("expected fragments")?;
    assert_eq!(aa.len(), ee.len());
    for (a, e) in aa.iter_mut().zip(ee) {
        let aa = a["atoms"].as_array_mut().ok_or("actual atoms")?;
        let ee = e["atoms"].as_array_mut().ok_or("expected atoms")?;
        assert_eq!(aa.len(), ee.len());
        for (a, e) in aa.iter_mut().zip(ee) {
            if !e["position"].is_null() {
                for axis in ["x_pt", "y_pt"] {
                    assert!(
                        (a["position"][axis].as_f64().ok_or("actual coordinate")?
                            - e["position"][axis].as_f64().ok_or("expected coordinate")?)
                        .abs()
                            <= 1e-9
                    );
                }
            } else {
                assert!(a["position"].is_null());
            }
            a.as_object_mut().ok_or("atom")?.remove("position");
            e.as_object_mut().ok_or("atom")?.remove("position");
        }
    }
    assert_eq!(a, e);
    Ok(())
}

fn external_graph(raw: &[u8]) -> TestResult<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON required")?;
    let mut child = Process::new(python)
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

fn consume_file(path: &Path, expected: &Value, smiles: &[&str], formula: &[&str]) -> TestResult {
    let raw = fs::read(path)?;
    compare_graph_positions(&external_graph(&raw)?, expected)?;
    let script = r#"
import json,sys,xml.parsers.expat as expat
from decimal import Decimal
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=sys.stdin.buffer.read(); expected=json.loads(sys.argv[1]); points=[]
p=expat.ParserCreate(); p.StartElementHandler=lambda name,a: points.append(tuple(map(Decimal,a['p'].split()))) if name=='n' else None
p.Parse(raw,True)
want=[a['position'] for f in expected['graph'] for a in f['atoms']]
assert len(points)==len(want)
for actual,target in zip(points,want):
 for v,k in zip(actual,['x_pt','y_pt']): assert abs(v-Decimal(str(target[k])))<=Decimal('0.000000001')
ms=Chem.MolsFromCDXML(raw.decode('ascii')); assert len(ms)==len(expected['smiles'])
assert [Chem.MolToSmiles(m) for m in ms]==[Chem.MolToSmiles(Chem.MolFromSmiles(s)) for s in expected['smiles']]
assert [rdMolDescriptors.CalcMolFormula(m) for m in ms]==expected['formula']
assert all(a.GetFormalCharge()==0 and a.GetIsotope()==0 and a.GetNumRadicalElectrons()==0 and a.GetChiralTag()==Chem.ChiralType.CHI_UNSPECIFIED for m in ms for a in m.GetAtoms())
assert all(b.GetBondTypeAsDouble() in (1.0,2.0) and b.GetStereo()==Chem.BondStereo.STEREONONE for m in ms for b in m.GetBonds())
"#;
    let args = serde_json::to_string(&json!({"graph":expected,"smiles":smiles,"formula":formula}))?;
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON required")?;
    let mut child = Process::new(python)
        .args(["-B", "-c", script, &args])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("RDKit stdin")?.write_all(&raw)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

fn carbon_graph(h: &[u16], points: &[[f64; 2]], edges: &[[u32; 2]]) -> Value {
    json!({"id":1,"atoms":h.iter().zip(points).enumerate().map(|(i,(&h,[x,y]))|json!({"id":i+1,"element":6,
        "position":{"x_pt":x,"y_pt":y},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h})).collect::<Vec<_>>(),
        "bonds":edges.iter().enumerate().map(|(i,[a,b])|json!({"id":i+1,"begin":a,"end":b,"order":"single"})).collect::<Vec<_>>()})
}

#[test]
fn actual_chemical_files_history_native_source_and_historical_save_lease_are_independent()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let r = 3.0_f64.sqrt();
    // Actual attachment files have literal graph/formula expectations, not own-roundtrip-only proof.
    for kind in ["fused", "spiro", "terminal", "double-terminal"] {
        let mut engine = if kind == "fused" || kind == "spiro" {
            original_hexagon()?
        } else {
            let mut e = Engine::default();
            e.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,
                "end_x_pt":120,"end_y_pt":72,"order":if kind=="double-terminal"{"double"}else{"single"}}))?;
            e
        };
        let (placement, expected, smiles, formula) = match kind {
            "fused" => (
                json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"negative"}),
                carbon_graph(
                    &[1, 1, 2, 2, 2, 2, 2, 2, 2, 2],
                    &[
                        [120.0, 168.0],
                        [120.0, 120.0],
                        [120.0 + 24.0 * r, 96.0],
                        [120.0 + 48.0 * r, 120.0],
                        [120.0 + 48.0 * r, 168.0],
                        [120.0 + 24.0 * r, 192.0],
                        [120.0 - 24.0 * r, 96.0],
                        [120.0 - 48.0 * r, 120.0],
                        [120.0 - 48.0 * r, 168.0],
                        [120.0 - 24.0 * r, 192.0],
                    ],
                    &[
                        [1, 2],
                        [2, 3],
                        [3, 4],
                        [4, 5],
                        [5, 6],
                        [6, 1],
                        [2, 7],
                        [7, 8],
                        [8, 9],
                        [9, 10],
                        [10, 1],
                    ],
                ),
                "C1CCC2CCCCC2C1",
                "C10H18",
            ),
            "spiro" => (
                json!({"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":120,"end_y_pt":216,"side":"positive"}),
                carbon_graph(
                    &[0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
                    &[
                        [120.0, 168.0],
                        [120.0, 120.0],
                        [120.0 + 24.0 * r, 96.0],
                        [120.0 + 48.0 * r, 120.0],
                        [120.0 + 48.0 * r, 168.0],
                        [120.0 + 24.0 * r, 192.0],
                        [120.0, 216.0],
                        [120.0 - 24.0 * r, 240.0],
                        [120.0 - 48.0 * r, 216.0],
                        [120.0 - 48.0 * r, 168.0],
                        [120.0 - 24.0 * r, 144.0],
                    ],
                    &[
                        [1, 2],
                        [2, 3],
                        [3, 4],
                        [4, 5],
                        [5, 6],
                        [6, 1],
                        [1, 7],
                        [7, 8],
                        [8, 9],
                        [9, 10],
                        [10, 11],
                        [11, 1],
                    ],
                ),
                "C1CCC2(CC1)CCCCC2",
                "C11H20",
            ),
            _ => {
                let mut graph = carbon_graph(
                    if kind == "double-terminal" {
                        &[2, 0, 2, 2, 2, 2, 2]
                    } else {
                        &[3, 1, 2, 2, 2, 2, 2]
                    },
                    &[
                        [72.0, 72.0],
                        [120.0, 72.0],
                        [144.0, 72.0 - 24.0 * r],
                        [192.0, 72.0 - 24.0 * r],
                        [216.0, 72.0],
                        [192.0, 72.0 + 24.0 * r],
                        [144.0, 72.0 + 24.0 * r],
                    ],
                    &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7], [7, 2]],
                );
                if kind == "double-terminal" {
                    graph["bonds"][0]["order"] = json!("double");
                }
                (
                    json!({"mode":"atom","fragment_id":1,"atom_id":2,"end_x_pt":144,"end_y_pt":72.0-24.0*r,"side":"positive"}),
                    graph,
                    if kind == "double-terminal" {
                        "C=C1CCCCC1"
                    } else {
                        "CC1CCCCC1"
                    },
                    if kind == "double-terminal" {
                        "C7H12"
                    } else {
                        "C7H14"
                    },
                )
            }
        };
        let p = ring_params(&engine, 6, placement);
        engine.dispatch("ring.create", p)?;
        let path = directory.path(&format!("{kind}-環-λ.cdxml"));
        engine.dispatch("document.save_cdxml", json!({"path":path}))?;
        consume_file(&path, &json!([expected]), &[smiles], &[formula])?;
    }
    // Native complete backing and the same real save reservation through publication/finish.
    let mut engine =
        Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    let original = engine.content().clone();
    let original_ptr = engine.stored_document()?.export_cdxml()?.as_ptr();
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[1,2]}),
    )?;
    let selected = wire(&engine)?["ui_selection"].clone();
    let old_path = directory.path("before-ring-原.cdxml");
    let path = directory.path("after-ring-環.cdxml");
    let pending = engine.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":old_path}),
    )?)?;
    let p = ring_params(
        &engine,
        6,
        free_json([300.0, 168.0], [300.0, 120.0], "positive"),
    );
    engine.dispatch("ring.create", p)?;
    assert_eq!(
        wire(&engine)?["ui_selection"],
        json!({"active":true,"epoch":selected["epoch"].as_u64().ok_or("epoch")?+1,"page_id":null,"atom_ids":[]})
    );
    let current = serde_json::to_value(engine.document()?)?;
    let completed = pending.complete()?;
    assert_eq!(fs::read(&old_path)?, NATIVE);
    let before = wire(&engine)?;
    let error = engine
        .prepare(Command::parse("document.save_cdxml", json!({"path":path}))?)
        .err()
        .ok_or("published save lost lease")?;
    assert_eq!(error.info().code, "busy");
    assert_eq!(wire(&engine)?, before);
    assert!(!path.exists());
    let receipt = engine.finish(completed)?;
    let file = receipt.file_operation.ok_or("historical Save receipt")?;
    assert_eq!(file.input_revision, 0);
    assert!(file.ui_request_id.is_none() && file.ui_request_current.is_none());
    assert_eq!(serde_json::to_value(engine.document()?)?, current);
    assert!(engine.state().file.dirty);
    assert!(
        engine
            .stored_document()?
            .source_info()
            .ok_or("source")?
            .modified
    );
    assert!(engine.stored_document()?.export_cdxml().is_err());
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    assert!(!engine.state().file.dirty);
    let mut old_graph = carbon_graph(&[3, 3], &[[100.0, 200.0], [140.0, 200.0]], &[[1, 2]]);
    let mut new_graph = carbon_graph(
        &[2, 2, 2, 2, 2, 2],
        &[
            [300.0, 168.0],
            [300.0, 120.0],
            [300.0 + 24.0 * r, 96.0],
            [300.0 + 48.0 * r, 120.0],
            [300.0 + 48.0 * r, 168.0],
            [300.0 + 24.0 * r, 192.0],
        ],
        &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
    );
    old_graph["id"] = json!(1);
    new_graph["id"] = json!(2);
    for a in new_graph["atoms"].as_array_mut().ok_or("atoms")? {
        a["id"] = json!(a["id"].as_u64().ok_or("id")? + 2);
    }
    for b in new_graph["bonds"].as_array_mut().ok_or("bonds")? {
        b["id"] = json!(b["id"].as_u64().ok_or("id")? + 1);
        b["begin"] = json!(b["begin"].as_u64().ok_or("begin")? + 2);
        b["end"] = json!(b["end"].as_u64().ok_or("end")? + 2);
    }
    consume_file(
        &path,
        &json!([old_graph, new_graph]),
        &["CC", "C1CCCCC1"],
        &["C2H6", "C6H12"],
    )?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    assert_eq!(
        engine.stored_document()?.export_cdxml()?.as_ptr(),
        original_ptr
    );
    assert!(engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, current);
    assert!(!engine.state().file.dirty);
    let saved_bytes = fs::read(&path)?;
    engine.dispatch("document.open_cdxml", json!({"path":path}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, current);
    assert!(engine.stored_document()?.is_fully_mapped());
    assert!(!engine.state().file.dirty);
    assert!(
        !engine
            .stored_document()?
            .source_info()
            .ok_or("source")?
            .modified
    );
    assert_eq!(fs::read(&path)?, saved_bytes);
    assert_eq!(fs::read(old_path)?, NATIVE);
    // Branching after Undo never reuses IDs spent by a successful abandoned ring.
    let mut branch = Engine::default();
    let p = ring_params(
        &branch,
        6,
        free_json([120.0, 168.0], [120.0, 120.0], "positive"),
    );
    let historical = branch
        .dispatch("ring.create", p)?
        .document
        .ok_or("historical model")?;
    branch.dispatch("history.undo", json!({}))?;
    let p = ring_params(
        &branch,
        3,
        free_json([120.0, 168.0], [120.0, 120.0], "negative"),
    );
    branch.dispatch("ring.create", p)?;
    let f = &branch.document()?.pages()[0].fragments()[0];
    assert_eq!(f.id().get(), 2);
    assert_eq!(
        f.atoms().iter().map(|a| a.id.get()).collect::<Vec<_>>(),
        [7, 8, 9]
    );
    assert_eq!(
        f.bonds().iter().map(|b| b.id.get()).collect::<Vec<_>>(),
        [7, 8, 9]
    );
    assert_eq!(branch.state().redo_depth, 0);
    assert_eq!(historical.pages()[0].fragments()[0].atoms().len(), 6);
    Ok(())
}

#[path = "ring_create/unsaturated_rings.rs"]
mod unsaturated;

#[path = "ring_create/request_json.rs"]
mod request_json;
