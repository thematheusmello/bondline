//! Ring sprout through existing JSON APIs; intended baseline is UnknownCommand.
use bondline_commands::Engine;
use serde_json::{Value, json};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn an_explicit_single_bridge_adds_a_disjoint_saturated_ring_in_one_history_edit() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":144,
        "end_x_pt":120,"end_y_pt":144,"order":"single"}),
    )?;
    let before = engine.content().clone();
    let old = serde_json::to_value(engine.document()?)?["pages"][0]["fragments"][0].clone();
    let result = engine.dispatch(
        "ring.sprout",
        json!({"expected_revision":1,"page_id":1,
        "fragment_id":1,"atom_id":2,"hydrogen_policy":"rectify_neutral_carbon",
        "ring":{"kind":"polygon","size":6},"ring_begin_x_pt":168,"ring_begin_y_pt":144,
        "ring_end_x_pt":168,"ring_end_y_pt":96,"side":"positive"}),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 2, 0)
    );
    assert!(result.source.is_none() && result.file.path.is_none() && result.file.dirty);
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    let value = serde_json::to_value(result)?;
    let fragments = value["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0]["id"], 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    let bonds = fragments[0]["bonds"].as_array().ok_or("bonds")?;
    assert_eq!((atoms.len(), bonds.len()), (8, 8));
    assert_eq!(atoms[0], old["atoms"][0]);
    let mut root = old["atoms"][1].clone();
    root["explicit_hydrogens"] = json!(2);
    assert_eq!(atoms[1], root);
    assert_eq!(bonds[0], old["bonds"][0]);
    for (i, atom) in atoms.iter().enumerate().skip(2) {
        assert_eq!(atom["id"], i + 1);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], if i == 2 { 1 } else { 2 });
        assert_eq!(atom["show_non_terminal_carbon_labels"], false);
        assert!(atom.get("show_terminal_carbon_labels").is_none());
        assert_eq!(atom["charge"], 0);
        assert_eq!(atom["isotope"], Value::Null);
        assert_eq!(atom["radical"], "none");
    }
    assert_eq!(atoms[2]["position"], json!({"x_pt":168.0,"y_pt":144.0}));
    assert_eq!(atoms[3]["position"], json!({"x_pt":168.0,"y_pt":96.0}));
    for (i, [begin, end]) in [[2, 3], [3, 4], [4, 5], [5, 6], [6, 7], [7, 8], [8, 3]]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            bonds[i + 1],
            json!({"id":i+2,"begin":begin,"end":end,"order":"single"})
        );
    }
    let after = engine.content().clone();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &before);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &after);
    Ok(())
}

// Expanded independent headless declaration; UI/application preferences are a later gate.
use bondline_commands::Command;
use bondline_io::StoredDocument;
use bondline_model::{Atom, Bond, Fragment, FragmentId, PageId, PageSize};
use proptest::prelude::*;
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::Path,
    process::{Command as Process, Stdio},
    sync::Arc,
};
type Check<T> = Result<T, Box<dyn std::error::Error>>;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the existing isolated temporary directory"
)]
mod support;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const POSITION_TOL: f64 = 1e-9;

fn pair() -> Check<Engine> {
    let mut e = Engine::default();
    e.dispatch(
        "bond.create",
        json!({"page_id":1,
        "begin_x_pt":72,"begin_y_pt":144,"end_x_pt":120,"end_y_pt":144,"order":"single"}),
    )?;
    Ok(e)
}
fn params(e: &Engine, ring: Value, a: [f64; 2], b: [f64; 2], side: &str) -> Value {
    json!({"expected_revision":e.state().revision,"page_id":1,"fragment_id":1,"atom_id":2,
        "hydrogen_policy":"rectify_neutral_carbon","ring":ring,"ring_begin_x_pt":a[0],
        "ring_begin_y_pt":a[1],"ring_end_x_pt":b[0],"ring_end_y_pt":b[1],"side":side})
}
fn g(e: &Engine) -> Check<Value> {
    Ok(serde_json::to_value(e.document()?)?["pages"][0]["fragments"][0].clone())
}
fn carbon(id: u32, p: [f64; 2], h: u16) -> Value {
    json!({"id":id,"element":6,"isotope":null,"charge":0,"radical":"none",
        "position":{"x_pt":p[0],"y_pt":p[1]},"explicit_hydrogens":h})
}
fn ethane() -> Value {
    json!({"id":1,"atoms":[carbon(1,[72.0,144.0],3),carbon(2,[120.0,144.0],3)],
    "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]})
}
fn seed(graphs: &[Value]) -> Check<Engine> {
    let mut stored = StoredDocument::default();
    let mut fragments = Vec::new();
    for f in graphs {
        fragments.push(Fragment::new(
            FragmentId::new(f["id"].as_u64().ok_or("fragment")?.try_into()?)
                .ok_or("fragment id")?,
            serde_json::from_value::<Vec<Atom>>(f["atoms"].clone())?,
            serde_json::from_value::<Vec<Bond>>(f["bonds"].clone())?,
        )?);
    }
    stored
        .model_mut()
        .append_fragments(PageId::new(1).ok_or("page")?, fragments)?;
    Ok(Engine::from_document(stored))
}
fn polygon(n: usize, a: [f64; 2], b: [f64; 2], side: &str) -> Vec<[f64; 2]> {
    // Circumcenter construction, independent of sequential turning of perimeter edges.
    let e = [b[0] - a[0], b[1] - a[1]];
    let sign = if side == "positive" { 1.0 } else { -1.0 };
    let alpha = std::f64::consts::PI / n as f64;
    let k = sign / (2.0 * alpha.tan());
    let center = [
        (a[0] + b[0]) / 2.0 - k * e[1],
        (a[1] + b[1]) / 2.0 + k * e[0],
    ];
    let v = [a[0] - center[0], a[1] - center[1]];
    let mut p = (0..n)
        .map(|i| {
            let t = sign * std::f64::consts::TAU * i as f64 / n as f64;
            [
                center[0] + t.cos() * v[0] - t.sin() * v[1],
                center[1] + t.sin() * v[0] + t.cos() * v[1],
            ]
        })
        .collect::<Vec<_>>();
    p[0] = a;
    p[1] = b;
    p
}
fn chair(two: bool, a: [f64; 2], b: [f64; 2], side: &str) -> Vec<[f64; 2]> {
    // Independently reduced final outer-edge orthographic coordinates from the chair lock.
    let r = 6.0_f64.sqrt();
    let d = 2.0 * (7.0 + r);
    let far = 17.0_f64.sqrt() * (3.0 * 2.0_f64.sqrt() + 2.0 * 3.0_f64.sqrt()) / d;
    let one = [
        [0.0, 0.0],
        [1.0, 0.0],
        [(3.0 + 2.0 * r) / d, 51.0_f64.sqrt() / d],
        [(-10.0 + 3.0 * r) / d, far],
        [(-24.0 + r) / d, far],
        [
            (-13.0 + r) / d,
            17.0_f64.sqrt() * (3.0 * 2.0_f64.sqrt() + 3.0_f64.sqrt()) / d,
        ],
    ];
    let t = if two {
        [
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0 - one[5][0], one[5][1]],
            [1.0 - one[4][0], one[4][1]],
            [1.0 - one[3][0], one[3][1]],
            [1.0 - one[2][0], one[2][1]],
        ]
    } else {
        one
    };
    let e = [b[0] - a[0], b[1] - a[1]];
    let sign = if side == "positive" { 1.0 } else { -1.0 };
    let mut p = t
        .map(|[x, y]| {
            [
                a[0] + x * e[0] - sign * y * e[1],
                a[1] + x * e[1] + sign * y * e[0],
            ]
        })
        .to_vec();
    p[0] = a;
    p[1] = b;
    p
}
fn points(ring: &Value, a: [f64; 2], b: [f64; 2], side: &str) -> Check<Vec<[f64; 2]>> {
    Ok(match ring["kind"].as_str().ok_or("kind")? {
        "polygon" => polygon(ring["size"].as_u64().ok_or("size")?.try_into()?, a, b, side),
        "benzene" => polygon(6, a, b, side),
        "cyclopentadiene" => polygon(5, a, b, side),
        "cyclohexane_chair_1" => chair(false, a, b, side),
        "cyclohexane_chair_2" => chair(true, a, b, side),
        _ => return Err("ring kind".into()),
    })
}
#[allow(
    clippy::too_many_arguments,
    reason = "independent graph expectations keep geometry, chemistry and identity floors explicit"
)]
fn expected(
    mut old: Value,
    root_index: usize,
    root_h: u16,
    p: &[[f64; 2]],
    h: &[u16],
    weights: &[u16],
    side: &str,
    atom_floor: u32,
    bond_floor: u32,
) -> Check<Value> {
    assert_eq!(p.len(), h.len());
    assert_eq!(p.len(), weights.len());
    let root = old["atoms"][root_index]["id"].clone();
    old["atoms"][root_index]["explicit_hydrogens"] = json!(root_h);
    let atoms = old["atoms"].as_array_mut().ok_or("atoms")?;
    for (i, (p, h)) in p.iter().zip(h).enumerate() {
        let mut a = carbon(atom_floor + u32::try_from(i)? + 1, *p, *h);
        a["show_non_terminal_carbon_labels"] = json!(false);
        atoms.push(a);
    }
    let bonds = old["bonds"].as_array_mut().ok_or("bonds")?;
    bonds.push(json!({"id":bond_floor+1,"begin":root,"end":atom_floor+1,"order":"single"}));
    for (i, w) in weights.iter().enumerate() {
        let mut b = json!({"id":bond_floor+2+u32::try_from(i)?,"begin":atom_floor+1+u32::try_from(i)?,
            "end":atom_floor+1+u32::try_from((i+1)%p.len())?,"order":if *w==2 {"double"}else{"single"}});
        if *w == 2 {
            b["double_position"] = json!(if side == "positive" { "right" } else { "left" });
        }
        bonds.push(b);
    }
    Ok(old)
}
fn graph_eq(actual: &Value, want: &Value) -> TestResult {
    let mut a = actual.clone();
    let mut w = want.clone();
    let aa = a["atoms"].as_array_mut().ok_or("actual atoms")?;
    let ww = w["atoms"].as_array_mut().ok_or("expected atoms")?;
    assert_eq!(aa.len(), ww.len());
    for (a, w) in aa.iter_mut().zip(ww) {
        for axis in ["x_pt", "y_pt"] {
            assert!(
                (a["position"][axis].as_f64().ok_or("actual point")?
                    - w["position"][axis].as_f64().ok_or("expected point")?)
                .abs()
                    <= POSITION_TOL
            );
        }
        a.as_object_mut().ok_or("actual atom")?.remove("position");
        w.as_object_mut().ok_or("expected atom")?.remove("position");
    }
    assert_eq!(a, w);
    Ok(())
}
fn reject(e: &mut Engine, p: Value, code: &str) -> TestResult {
    let state = serde_json::to_value(e.state())?;
    let before = e.content().clone();
    let stamp = e.capture_stamp();
    let tool = e.effective_tool();
    let model = e.stored_document().ok().map(StoredDocument::model_snapshot);
    let file = e.ui_file_context_stamp();
    let view = e.view();
    let error = e
        .dispatch("ring.sprout", p)
        .err()
        .ok_or("invalid sprout accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert_eq!(e.content(), &before);
    assert!(e.capture_stamp_matches(&stamp));
    assert_eq!(e.effective_tool(), tool);
    assert_eq!(e.view(), view);
    assert!(e.ui_file_context_stamp_matches(&file));
    if let Some(model) = model {
        assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    }
    Ok(())
}

#[test]
fn every_ring_family_has_independent_cycle_geometry_h_and_perimeter_phase_after_its_single_bridge()
-> TestResult {
    let mut cases = Vec::new();
    for n in 3..=8 {
        let mut h = vec![2; n];
        h[0] = 1;
        cases.push((json!({"kind":"polygon","size":n}), h, vec![1; n]));
    }
    cases.extend([
        (
            json!({"kind":"benzene"}),
            vec![0, 1, 1, 1, 1, 1],
            vec![2, 1, 2, 1, 2, 1],
        ),
        (
            json!({"kind":"benzene","phase":"double_first"}),
            vec![0, 1, 1, 1, 1, 1],
            vec![2, 1, 2, 1, 2, 1],
        ),
        (
            json!({"kind":"benzene","phase":"single_first"}),
            vec![0, 1, 1, 1, 1, 1],
            vec![1, 2, 1, 2, 1, 2],
        ),
        (
            json!({"kind":"cyclopentadiene"}),
            vec![0, 1, 1, 1, 2],
            vec![2, 1, 2, 1, 1],
        ),
        (
            json!({"kind":"cyclopentadiene","phase":"double_first"}),
            vec![0, 1, 1, 1, 2],
            vec![2, 1, 2, 1, 1],
        ),
        (
            json!({"kind":"cyclopentadiene","phase":"single_first"}),
            vec![0, 1, 1, 2, 1],
            vec![1, 2, 1, 1, 2],
        ),
        (
            json!({"kind":"cyclohexane_chair_1"}),
            vec![1, 2, 2, 2, 2, 2],
            vec![1; 6],
        ),
        (
            json!({"kind":"cyclohexane_chair_2"}),
            vec![1, 2, 2, 2, 2, 2],
            vec![1; 6],
        ),
    ]);
    for (ring, h, weights) in cases {
        for side in ["positive", "negative"] {
            let mut e = pair()?;
            let before = e.content().clone();
            let tool = e.effective_tool();
            let x = if ring == json!({"kind":"polygon","size":4}) && side == "negative" {
                216.0
            } else {
                168.0
            };
            let a = [x, 144.0];
            let b = [x, 96.0];
            let p = points(&ring, a, b, side)?;
            let want = expected(ethane(), 1, 2, &p, &h, &weights, side, 2, 1)?;
            let request = params(&e, ring.clone(), a, b, side);
            let result = e
                .dispatch("ring.sprout", request)
                .map_err(|error| format!("{ring} / {side}: {error}"))?;
            assert!(result.changed);
            assert_eq!(
                (result.revision, result.undo_depth, result.redo_depth),
                (2, 2, 0)
            );
            assert_eq!(e.effective_tool(), tool);
            graph_eq(&g(&e)?, &want)?;
            let value = g(&e)?;
            assert_eq!(
                value["atoms"][2]["position"],
                json!({"x_pt":a[0],"y_pt":a[1]})
            );
            assert_eq!(
                value["atoms"][3]["position"],
                json!({"x_pt":b[0],"y_pt":b[1]})
            );
            let after = e.content().clone();
            e.dispatch("history.undo", json!({}))?;
            assert_eq!(e.content(), &before);
            e.dispatch("history.redo", json!({}))?;
            assert_eq!(e.content(), &after);
        }
    }
    Ok(())
}

fn schema_node<'a>(root: &'a Value, v: &'a Value) -> Check<&'a Value> {
    match v.get("$ref").and_then(Value::as_str) {
        Some(r) => root
            .pointer(r.strip_prefix('#').ok_or("nonlocal schema")?)
            .ok_or_else(|| "schema target".into()),
        None => Ok(v),
    }
}
fn required(v: &Value) -> Check<BTreeSet<&str>> {
    v["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or_else(|| "required field".into()))
        .collect()
}
#[test]
fn one_closed_sprout_schema_and_runtime_selectors_add_only_the_authoring_command() -> TestResult {
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 48);
    let names = registry.iter().map(|e| e.name).collect::<Vec<_>>();
    assert_eq!(&names[..43], OLD_COMMANDS.as_slice());
    assert_eq!(names[43], "ring.sprout");
    assert_eq!(names[44], "ui.preferences");
    let entry = registry
        .iter()
        .find(|e| e.name == "ring.sprout")
        .ok_or("registry entry")?;
    assert!(!entry.read_only);
    let root = serde_json::to_value(&entry.input_schema)?;
    assert_eq!(root["type"], "object");
    assert_eq!(root["additionalProperties"], false);
    let fields = [
        "expected_revision",
        "page_id",
        "fragment_id",
        "atom_id",
        "hydrogen_policy",
        "ring",
        "ring_begin_x_pt",
        "ring_begin_y_pt",
        "ring_end_x_pt",
        "ring_end_y_pt",
        "side",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(required(&root)?, fields);
    assert_eq!(
        root["properties"]
            .as_object()
            .ok_or("properties")?
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        fields
    );
    let ring = schema_node(&root, &root["properties"]["ring"])?;
    let variants = ring
        .get("oneOf")
        .or_else(|| ring.get("anyOf"))
        .and_then(Value::as_array)
        .ok_or("ring variants")?;
    assert_eq!(variants.len(), 5);
    let mut kinds = BTreeSet::new();
    for branch in variants {
        let branch = schema_node(&root, branch)?;
        assert_eq!(branch["additionalProperties"], false);
        let p = branch["properties"].as_object().ok_or("ring properties")?;
        let tag = schema_node(&root, &p["kind"])?;
        let kind = tag
            .get("const")
            .and_then(Value::as_str)
            .or_else(|| tag["enum"].get(0).and_then(Value::as_str))
            .ok_or("kind")?;
        assert!(kinds.insert(kind));
        match kind {
            "polygon" => {
                assert_eq!(required(branch)?, ["kind", "size"].into_iter().collect());
                assert_eq!(p.len(), 2);
                assert_eq!(p["size"]["minimum"].as_f64(), Some(3.0));
                assert_eq!(p["size"]["maximum"].as_f64(), Some(8.0));
            }
            "benzene" | "cyclopentadiene" => {
                assert_eq!(required(branch)?, ["kind"].into_iter().collect());
                assert_eq!(p.len(), 2);
                let phase = schema_node(&root, &p["phase"])?;
                assert_eq!(phase["type"], "string");
                assert_eq!(phase["enum"], json!(["double_first", "single_first"]));
            }
            "cyclohexane_chair_1" | "cyclohexane_chair_2" => {
                assert_eq!(required(branch)?, ["kind"].into_iter().collect());
                assert_eq!(p.len(), 1);
            }
            _ => return Err("unexpected ring kind".into()),
        }
    }
    assert_eq!(
        kinds,
        [
            "polygon",
            "benzene",
            "cyclopentadiene",
            "cyclohexane_chair_1",
            "cyclohexane_chair_2"
        ]
        .into_iter()
        .collect()
    );
    let mut e = pair()?;
    let valid = params(
        &e,
        json!({"kind":"polygon","size":6}),
        [168.0, 144.0],
        [168.0, 96.0],
        "positive",
    );
    for field in fields {
        let mut p = valid.clone();
        p.as_object_mut().ok_or("params")?.remove(field);
        reject(&mut e, p, "invalid_params")?;
    }
    for (field, v) in [
        ("extra", json!(true)),
        ("side", Value::Null),
        ("side", json!({"positive":null})),
        ("hydrogen_policy", json!({"rectify_neutral_carbon":null})),
        ("atom_id", json!(0)),
        ("page_id", json!(1.5)),
        ("fragment_id", json!(u64::MAX)),
        ("ring_end_x_pt", Value::Null),
    ] {
        let mut p = valid.clone();
        p[field] = v;
        reject(&mut e, p, "invalid_params")?;
    }
    for invalid in [
        json!(["polygon", 6]),
        json!(["benzene"]),
        json!(["cyclopentadiene", "single_first"]),
        json!(["cyclohexane_chair_1"]),
        json!(["cyclohexane_chair_2"]),
        json!({"kind":"polygon"}),
        json!({"kind":"polygon","size":2}),
        json!({"kind":"polygon","size":9}),
        json!({"kind":"polygon","size":6,"phase":"double_first"}),
        json!({"kind":"benzene","phase":null}),
        json!({"kind":"benzene","phase":{"double_first":null}}),
        json!({"kind":"cyclopentadiene","size":5}),
        json!({"kind":"cyclohexane_chair_1","phase":"single_first"}),
        json!({"kind":"cyclohexane_chair_2","size":6}),
        json!({"kind":"cyclohexane_chair_1","unexpected":true}),
        json!({"kind":"cyclohexane_chair_2","unexpected":true}),
        json!({"kind":null}),
        json!({"polygon":{"size":6}}),
    ] {
        let mut p = valid.clone();
        p["ring"] = invalid;
        reject(&mut e, p, "invalid_params")?;
    }
    assert_eq!(
        e.controls().len(),
        39,
        "headless authoring adds no native binding"
    );
    Ok(())
}

#[test]
fn weighted_h_and_every_existing_double_style_survive_and_sparse_ids_keep_one_target_fragment()
-> TestResult {
    for style in [None, Some("center"), Some("left"), Some("right")] {
        let mut old = ethane();
        old["atoms"][0]["explicit_hydrogens"] = json!(2);
        old["atoms"][1]["explicit_hydrogens"] = json!(2);
        old["atoms"][0]["show_terminal_carbon_labels"] = json!(true);
        old["atoms"][1]["show_non_terminal_carbon_labels"] = json!(true);
        old["bonds"][0]["order"] = json!("double");
        if let Some(style) = style {
            old["bonds"][0]["double_position"] = json!(style);
        }
        let mut e = seed(std::slice::from_ref(&old))?;
        let ring = json!({"kind":"benzene","phase":"single_first"});
        let p = points(&ring, [168.0, 144.0], [168.0, 96.0], "negative")?;
        let want = expected(
            old,
            1,
            1,
            &p,
            &[0, 1, 1, 1, 1, 1],
            &[1, 2, 1, 2, 1, 2],
            "negative",
            2,
            1,
        )?;
        let request = params(&e, ring, [168.0, 144.0], [168.0, 96.0], "negative");
        e.dispatch("ring.sprout", request)?;
        graph_eq(&g(&e)?, &want)?;
    }
    let mut triple = ethane();
    triple["atoms"][0]["explicit_hydrogens"] = json!(1);
    triple["atoms"][1]["explicit_hydrogens"] = json!(1);
    triple["bonds"][0]["order"] = json!("triple");
    let mut e = seed(std::slice::from_ref(&triple))?;
    let p = params(
        &e,
        json!({"kind":"benzene"}),
        [168.0, 144.0],
        [168.0, 96.0],
        "positive",
    );
    e.dispatch("ring.sprout", p)?;
    assert_eq!(g(&e)?["atoms"][1]["explicit_hydrogens"], 0);
    assert_eq!(g(&e)?["atoms"][2]["explicit_hydrogens"], 0);
    let mut target = ethane();
    target["id"] = json!(17);
    target["atoms"][0]["id"] = json!(11);
    target["atoms"][1]["id"] = json!(29);
    target["atoms"][0]["show_terminal_carbon_labels"] = json!(true);
    target["atoms"][1]["show_terminal_carbon_labels"] = json!(false);
    target["bonds"][0] = json!({"id":41,"begin":29,"end":11,"order":"single"});
    let unrelated = json!({"id":7,"atoms":[{"id":91,"element":8,"isotope":18,"charge":-1,"radical":"triplet","position":null,"explicit_hydrogens":19}],"bonds":[]});
    let mut e = seed(&[target.clone(), unrelated.clone()])?;
    let before = e.content().clone();
    let arc = Arc::clone(&e.document()?.pages()[0].fragments()[1]);
    let ring = json!({"kind":"polygon","size":6});
    let ps = points(&ring, [168.0, 144.0], [168.0, 96.0], "positive")?;
    let want = expected(
        target.clone(),
        1,
        2,
        &ps,
        &[1, 2, 2, 2, 2, 2],
        &[1; 6],
        "positive",
        91,
        41,
    )?;
    let mut p = params(&e, ring, [168.0, 144.0], [168.0, 96.0], "positive");
    p["fragment_id"] = json!(17);
    p["atom_id"] = json!(29);
    e.dispatch("ring.sprout", p)?;
    graph_eq(&g(&e)?, &want)?;
    assert!(Arc::ptr_eq(&arc, &e.document()?.pages()[0].fragments()[1]));
    assert_eq!(
        serde_json::to_value(e.document()?)?["pages"][0]["fragments"][1],
        unrelated
    );
    let after = e.content().clone();
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.content(), &before);
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(e.content(), &after);
    e.dispatch("history.undo", json!({}))?;
    let mut p = params(
        &e,
        json!({"kind":"polygon","size":3}),
        [260.0, 300.0],
        [308.0, 300.0],
        "negative",
    );
    p["fragment_id"] = json!(17);
    p["atom_id"] = json!(29);
    e.dispatch("ring.sprout", p)?;
    let v = g(&e)?;
    assert_eq!(v["id"], 17);
    assert_eq!(v["atoms"][2]["id"], 98);
    assert_eq!(v["bonds"][1]["id"], 49);
    assert_eq!(e.state().redo_depth, 0);
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":400,"begin_y_pt":450,"end_x_pt":448,"end_y_pt":450}),
    )?;
    assert_eq!(
        e.document()?.pages()[0].fragments()[2].id().get(),
        18,
        "sprout allocates no intermediate Fragment"
    );
    Ok(())
}

#[test]
fn spiro_remains_a_distinct_legacy_operation_and_failures_keep_source_selection_redo_and_floors()
-> TestResult {
    let r = 3.0_f64.sqrt();
    let ps = [
        [120.0, 168.0],
        [120.0, 120.0],
        [120.0 + 24.0 * r, 96.0],
        [120.0 + 48.0 * r, 120.0],
        [120.0 + 48.0 * r, 168.0],
        [120.0 + 24.0 * r, 192.0],
    ];
    let old = json!({"id":1,"atoms":ps.iter().enumerate().map(|(i,p)|carbon(i as u32+1,*p,2)).collect::<Vec<_>>(),
        "bonds":(0..6).map(|i|json!({"id":i+1,"begin":i+1,"end":if i==5 {1}else{i+2},"order":"single"})).collect::<Vec<_>>()});
    let mut sprout = seed(std::slice::from_ref(&old))?;
    let mut p = params(
        &sprout,
        json!({"kind":"polygon","size":6}),
        [72.0, 216.0],
        [24.0, 216.0],
        "negative",
    );
    p["atom_id"] = json!(1);
    sprout.dispatch("ring.sprout", p)?;
    assert_eq!(g(&sprout)?["atoms"].as_array().ok_or("atoms")?.len(), 12);
    assert_eq!(g(&sprout)?["atoms"][0]["explicit_hydrogens"], 1);
    assert_eq!(g(&sprout)?["atoms"][6]["explicit_hydrogens"], 1);
    let mut spiro = seed(std::slice::from_ref(&old))?;
    spiro.dispatch("ring.create",json!({"expected_revision":0,"page_id":1,"size":6,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":72,"end_y_pt":168,"side":"negative"}}))?;
    assert_eq!(g(&spiro)?["atoms"].as_array().ok_or("atoms")?.len(), 11);
    assert_eq!(g(&spiro)?["atoms"][0]["explicit_hydrogens"], 0);
    let mut e = Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":300,"begin_y_pt":400,"end_x_pt":348,"end_y_pt":400}),
    )?;
    e.dispatch("history.undo", json!({}))?;
    e.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":2,"page_id":1,"atom_ids":[1,2]}),
    )?;
    let proof = e.selection_context()?;
    let base = params(
        &e,
        json!({"kind":"polygon","size":6}),
        [188.0, 200.0],
        [188.0, 152.0],
        "positive",
    );
    for (field, value, code) in [
        ("expected_revision", json!(0), "stale_revision"),
        ("atom_id", json!(999), "not_found"),
        ("fragment_id", json!(999), "not_found"),
        ("ring_begin_x_pt", json!(140.0), "invalid_params"),
        ("ring_end_x_pt", json!(100001.0), "invalid_params"),
    ] {
        let mut p = base.clone();
        p[field] = value;
        if field == "ring_begin_x_pt" {
            p["ring_begin_y_pt"] = json!(200.0);
        }
        reject(&mut e, p, code)?;
        assert!(proof.is_current(&e));
    }
    for (a, b, ring) in [
        (
            [188.0, 200.0],
            [188.0, 200.0],
            json!({"kind":"polygon","size":6}),
        ),
        (
            [100.0, 200.0],
            [100.0, 152.0],
            json!({"kind":"polygon","size":6}),
        ),
        (
            [65_536.0, 65_536.0],
            [f64::from_bits(65_536.0_f64.to_bits() + 1), 65_536.0],
            json!({"kind":"cyclohexane_chair_1"}),
        ),
    ] {
        let p = params(&e, ring, a, b, "positive");
        reject(&mut e, p, "invalid_params")?;
        assert!(proof.is_current(&e));
    }
    assert_eq!(e.stored_document()?.export_cdxml()?, NATIVE);
    assert!(!e.state().file.dirty);
    assert_eq!(e.state().redo_depth, 1);
    e.dispatch("ring.sprout", base)?;
    assert_eq!(g(&e)?["atoms"][2]["id"], 5);
    assert_eq!(g(&e)?["bonds"][1]["id"], 3);
    let isolated = json!({"id":1,"atoms":[carbon(1,[0.0,0.0],4)],"bonds":[]});
    let mut tiny = seed(&[isolated])?;
    let mut p = params(
        &tiny,
        json!({"kind":"polygon","size":6}),
        [1e-200, 0.0],
        [48.0, 0.0],
        "positive",
    );
    p["atom_id"] = json!(1);
    tiny.dispatch("ring.sprout", p)?;
    assert_eq!(
        g(&tiny)?["atoms"][1]["position"]["x_pt"].as_f64(),
        Some(1e-200)
    );
    assert_eq!(
        g(&tiny)?["atoms"][0]["explicit_hydrogens"],
        3,
        "no minimum mathematical connector length"
    );
    let mut full = ethane();
    full["atoms"][1]["explicit_hydrogens"] = json!(0);
    let mut bad = seed(&[full])?;
    let p = params(
        &bad,
        json!({"kind":"polygon","size":6}),
        [168.0, 144.0],
        [168.0, 96.0],
        "positive",
    );
    reject(&mut bad, p, "unsupported_content")?;
    let mut exhausted = ethane();
    exhausted["atoms"][1]["id"] = json!(u32::MAX);
    exhausted["bonds"][0]["end"] = json!(u32::MAX);
    let mut e = seed(&[exhausted])?;
    let mut p = params(
        &e,
        json!({"kind":"polygon","size":6}),
        [168.0, 144.0],
        [168.0, 96.0],
        "positive",
    );
    p["atom_id"] = json!(u32::MAX);
    reject(&mut e, p, "identity_exhausted")?;
    let mut collision = pair()?;
    let p = params(
        &collision,
        json!({"kind":"polygon","size":4}),
        [168.0, 144.0],
        [168.0, 96.0],
        "negative",
    );
    reject(&mut collision, p.clone(), "invalid_params")?;
    assert!(matches!(
        collision.dispatch("ring.sprout", p),
        Err(bondline_commands::Error::OccupiedEndpoint)
    ));
    let p = params(
        &collision,
        json!({"kind":"polygon","size":4}),
        [216.0, 144.0],
        [216.0, 96.0],
        "negative",
    );
    collision.dispatch("ring.sprout", p)?;
    assert_eq!(g(&collision)?["atoms"][2]["id"], 3);
    assert_eq!(g(&collision)?["bonds"][1]["id"], 2);
    let mut closed = pair()?;
    closed.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"discard"}),
    )?;
    assert!(closed.complete_close());
    let p = params(
        &closed,
        json!({"kind":"polygon","size":6}),
        [168.0, 144.0],
        [168.0, 96.0],
        "positive",
    );
    reject(&mut closed, p, "close_committed")?;
    Ok(())
}

fn consume(
    path: &Path,
    want: &Value,
    smiles: &str,
    formula: &str,
    flags: &[Option<bool>],
    styles: &[Option<&str>],
) -> TestResult {
    let raw = fs::read(path)?;
    assert!(!std::str::from_utf8(&raw)?.contains("ShowTerminalCarbonLabels"));
    let args = serde_json::to_string(
        &json!({"graph":[want],"smiles":smiles,"formula":formula,"flags":flags,"double_positions":styles}),
    )?;
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/ring_create/named_ring_oracle.py"
            ),
            &args,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(&raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["formula"], formula);
    assert_eq!(result["literal_integer_orders"], true);
    assert_eq!(result["literal_direction"], true);
    assert_eq!(result["literal_flags_and_double_positions"], true);
    assert_eq!(result["no_assigned_stereo"], true);
    Ok(())
}
#[test]
fn unicode_native_files_preserve_complete_source_and_match_unchanged_raw_decimal_and_rdkit_consumers()
-> TestResult {
    let dir = support::FileTestDirectory::new()?;
    for (ring, h, weights, smiles, formula) in [
        (
            json!({"kind":"polygon","size":6}),
            vec![1, 2, 2, 2, 2, 2],
            vec![1; 6],
            "CCC1CCCCC1",
            "C8H16",
        ),
        (
            json!({"kind":"benzene"}),
            vec![0, 1, 1, 1, 1, 1],
            vec![2, 1, 2, 1, 2, 1],
            "CCc1ccccc1",
            "C8H10",
        ),
        (
            json!({"kind":"cyclopentadiene"}),
            vec![0, 1, 1, 1, 2],
            vec![2, 1, 2, 1, 1],
            "CCC1=CC=CC1",
            "C7H10",
        ),
        (
            json!({"kind":"cyclohexane_chair_1"}),
            vec![1, 2, 2, 2, 2, 2],
            vec![1; 6],
            "CCC1CCCCC1",
            "C8H16",
        ),
        (
            json!({"kind":"cyclohexane_chair_2"}),
            vec![1, 2, 2, 2, 2, 2],
            vec![1; 6],
            "CCC1CCCCC1",
            "C8H16",
        ),
    ] {
        let name = ring["kind"].as_str().ok_or("kind")?;
        let input = dir.path(&format!("{name}-元-λ.cdxml"));
        fs::write(&input, NATIVE)?;
        let mut e = Engine::default();
        e.dispatch("document.open_cdxml", json!({"path":input}))?;
        let before = e.content().clone();
        let original_ptr = e.stored_document()?.export_cdxml()?.as_ptr();
        let old = json!({"id":1,"atoms":[carbon(1,[100.0,200.0],3),carbon(2,[140.0,200.0],3)],"bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]});
        let ps = points(&ring, [188.0, 200.0], [188.0, 152.0], "positive")?;
        let want = expected(old, 1, 2, &ps, &h, &weights, "positive", 2, 1)?;
        let request = params(&e, ring.clone(), [188.0, 200.0], [188.0, 152.0], "positive");
        let result = e.dispatch("ring.sprout", request)?;
        assert_eq!((result.revision, result.undo_depth), (2, 1));
        assert!(result.changed && result.file.dirty);
        assert!(result.source.ok_or("source")?.modified);
        assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
        graph_eq(&g(&e)?, &want)?;
        assert!(e.stored_document()?.export_cdxml().is_err());
        let after = e.content().clone();
        let output = dir.path(&format!("{name}-芽-化学.cdxml"));
        e.dispatch("document.save_cdxml", json!({"path":output}))?;
        assert!(!e.state().file.dirty);
        let flags = [vec![None; 2], vec![Some(false); h.len()]].concat();
        let mut styles = vec![None; 2];
        styles.extend(
            weights
                .iter()
                .map(|w| if *w == 2 { Some("right") } else { None }),
        );
        consume(&output, &want, smiles, formula, &flags, &styles)?;
        assert_eq!(fs::read(&input)?, NATIVE);
        e.dispatch("history.undo", json!({}))?;
        assert_eq!(e.content(), &before);
        assert_eq!(e.stored_document()?.export_cdxml()?.as_ptr(), original_ptr);
        assert!(e.state().file.dirty);
        e.dispatch("history.redo", json!({}))?;
        assert_eq!(e.content(), &after);
        assert!(!e.state().file.dirty);
        let bytes = fs::read(&output)?;
        e.dispatch("document.open_cdxml", json!({"path":output}))?;
        graph_eq(&g(&e)?, &want)?;
        assert!(e.stored_document()?.is_fully_mapped());
        assert!(!e.state().file.dirty);
        assert!(
            !e.stored_document()?
                .source_info()
                .ok_or("reopen source")?
                .modified
        );
        assert_eq!(fs::read(&output)?, bytes);
        assert_eq!(fs::read(&input)?, NATIVE);
    }
    for (name, bytes, open) in [
        (
            "opaque.cdxml",
            String::from_utf8(NATIVE.to_vec())?
                .replace("<CDXML ", "<CDXML Private=\"keep\" ")
                .into_bytes(),
            "document.open_cdxml",
        ),
        (
            "retained.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").to_vec(),
            "document.open_mol",
        ),
        (
            "retained.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").to_vec(),
            "document.open_sdf",
        ),
        (
            "retained.smi",
            b"CC\tPRIVATE\xff\0\r\nO next".to_vec(),
            "document.open_smiles",
        ),
    ] {
        let input = dir.path(name);
        fs::write(&input, &bytes)?;
        let mut e = Engine::default();
        e.dispatch(open, json!({"path":input}))?;
        let p = params(
            &e,
            json!({"kind":"polygon","size":6}),
            [300.0, 200.0],
            [300.0, 152.0],
            "positive",
        );
        reject(&mut e, p, "unsupported_content")?;
        assert_eq!(fs::read(&input)?, bytes);
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    #[test]
    fn generated_sprout_sequences_restore_complete_snapshots_and_exact_root_h(choices in prop::collection::vec((any::<bool>(),any::<bool>()),1..4)) {
        let mut e=pair().map_err(|e|TestCaseError::fail(e.to_string()))?;let original=e.content().clone();let mut snapshots=Vec::new();
        for (i,(benzene,negative)) in choices.iter().enumerate() {
            let ring=if *benzene {json!({"kind":"benzene"})}else{json!({"kind":"polygon","size":6})};
            let x=240.0+192.0*i as f64;let side=if *negative {"negative"}else{"positive"};
            let p=params(&e,ring,[x,240.0],[x+48.0,240.0],side);e.dispatch("ring.sprout",p)?;
            let actual=g(&e).map_err(|e|TestCaseError::fail(e.to_string()))?;prop_assert_eq!(&actual["atoms"][1]["explicit_hydrogens"],&json!(2-i));
            prop_assert_eq!(actual["atoms"].as_array().ok_or_else(||TestCaseError::fail("atoms"))?.len(),2+6*(i+1));snapshots.push(e.content().clone());
        }
        for expected in snapshots.iter().rev().skip(1) {e.dispatch("history.undo",json!({}))?;prop_assert_eq!(e.content(),expected);}
        e.dispatch("history.undo",json!({}))?;prop_assert_eq!(e.content(),&original);
        for expected in &snapshots {e.dispatch("history.redo",json!({}))?;prop_assert_eq!(e.content(),expected);}
    }
}

const OLD_COMMANDS: [&str; 43] = [
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
    "ui.drawing_settings",
    "ui.chain_tool",
];

#[test]
fn actual_sprout_cpu_gpu_pngs_match_an_independent_literal_page_with_bridge_and_cycle_ink()
-> TestResult {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use bondline_model::BondId;
    use bondline_render::{HitTarget, cpu_scene, graph_page_scene};
    let ring = json!({"kind":"polygon","size":6});
    let ps = polygon(6, [168.0, 144.0], [168.0, 96.0], "positive");
    let want = expected(
        ethane(),
        1,
        2,
        &ps,
        &[1, 2, 2, 2, 2, 2],
        &[1; 6],
        "positive",
        2,
        1,
    )?;
    let mut reference = seed(std::slice::from_ref(&want))?;
    reference.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":288,"height_pt":288}),
    )?;
    let document = reference.document()?;
    let page = &document.pages()[0];
    let graph = graph_page_scene(640, 640, page)?;
    let frame = cpu_scene(graph.scene())?;
    let pixel = |x: u32, y: u32| -> Check<&[u8]> {
        let at = usize::try_from((y * 640 + x) * 4)?;
        frame
            .rgba
            .get(at..at + 4)
            .ok_or_else(|| "pixel bounds".into())
    };
    //288pt paper at640px gives exactly2px/pt with32px margins and2px Single width.
    assert_eq!(
        pixel(320, 320)?,
        [0, 0, 0, 255],
        "bridge midpoint must have actual ink"
    );
    assert_eq!(
        graph.hit_test([320.5, 320.5], 0.0)?,
        Some(HitTarget::Bond {
            fragment_id: FragmentId::new(1).ok_or("fragment")?,
            bond_id: BondId::new(2).ok_or("bridge")?
        })
    );
    for y in 252..292 {
        for x in 366..370 {
            assert_eq!(
                pixel(x, y)?,
                if x == 367 || x == 368 {
                    &[0, 0, 0, 255]
                } else {
                    &[255, 255, 255, 255]
                },
                "new first perimeter strip{x},{y}"
            );
        }
    }
    assert_eq!(
        pixel(451, 271)?,
        [255, 255, 255, 255],
        "ring interior stays empty"
    );
    assert_eq!(graph.hit_test([451.5, 271.5], 0.0)?, None);
    let mut actual = pair()?;
    actual.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":288,"height_pt":288}),
    )?;
    let request = params(&actual, ring, [168.0, 144.0], [168.0, 96.0], "positive");
    actual.dispatch("ring.sprout", request)?;
    graph_eq(&g(&actual)?, &want)?;
    let state = serde_json::to_value(actual.state())?;
    let whole = actual.content().clone();
    let stamp = actual.capture_stamp();
    let view = actual.view();
    let mut pngs = Vec::new();
    for backend in ["cpu", "gpu"] {
        let result = actual.dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":640,"height_px":640,"backend":backend}),
        )?;
        assert!(!result.changed);
        assert!(result.file_operation.is_none());
        let image = result.capture.ok_or("actual registered PNG")?;
        assert_eq!((image.width_px, image.height_px), (640, 640));
        assert_eq!(image.mime_type, "image/png");
        if backend == "gpu" {
            let adapter = image.adapter.as_ref().ok_or("actual GPU adapter")?;
            assert!(
                !adapter.name.is_empty()
                    && !adapter.backend.is_empty()
                    && !adapter.device_type.is_empty()
            );
        } else {
            assert!(image.adapter.is_none());
        }
        pngs.push(image.png_base64);
        assert_eq!(serde_json::to_value(actual.state())?, state);
        assert_eq!(actual.content(), &whole);
        assert!(actual.capture_stamp_matches(&stamp));
        assert_eq!(actual.view(), view);
    }
    assert_eq!(pngs[0], pngs[1]);
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .args([
            "-B",
            "-c",
            r#"
import base64,io,json,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
p=json.load(sys.stdin); expected=base64.b64decode(p['rgba'],validate=True)
assert len(expected)==640*640*4 and len(p['pngs'])==2
for encoded in p['pngs']:
 raw=base64.b64decode(encoded,validate=True)
 with Image.open(io.BytesIO(raw)) as im: im.verify()
 with Image.open(io.BytesIO(raw)) as im:
  assert im.format=='PNG' and im.mode=='RGBA' and im.size==(640,640)
  assert im.tobytes()==expected
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(
        child.stdin.take().ok_or("Pillow stdin")?,
        &json!({"pngs":pngs,"rgba":STANDARD.encode(&frame.rgba)}),
    )?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[path = "ring_sprout/api.rs"]
mod api;
