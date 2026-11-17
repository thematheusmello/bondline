//! Independent chair request through existing JSON dispatch; no future Rust API.
use bondline_commands::Engine;
use serde_json::{Value, json};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn free_chair_one_is_one_saturated_six_carbon_edit_with_an_explicit_first_edge() -> TestResult {
    let mut engine = Engine::default();
    let original = engine.content().clone();
    let result = engine.dispatch(
        "ring.create",
        json!({"expected_revision":0,"page_id":1,"kind":"cyclohexane_chair_1",
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
    for (i, atom) in atoms.iter().enumerate() {
        assert_eq!(atom["id"], i + 1);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], 2);
        assert_eq!(atom["charge"], 0);
        assert_eq!(atom["isotope"], Value::Null);
        assert_eq!(atom["radical"], "none");
    }
    assert_eq!(atoms[0]["position"], json!({"x_pt":120.0,"y_pt":168.0}));
    assert_eq!(atoms[1]["position"], json!({"x_pt":120.0,"y_pt":120.0}));
    for (i, bond) in bonds.iter().enumerate() {
        assert_eq!(
            bond,
            &json!({"id":i+1,"begin":i+1,
        "end":if i==5 {1} else {i+2},"order":"single"})
        );
    }
    let authored = engine.content().clone();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &original);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &authored);
    Ok(())
}

#[test]
fn both_chairs_and_sides_match_independently_projected_tetrahedral_coordinates() -> TestResult {
    // Independent Python3D/plane/projection calculation, before the producer.
    let cases = [
        (
            "cyclohexane_chair_1",
            [
                [120.0, 168.0],
                [120.0, 120.0],
                [138.1379404550311, 147.93801646291257],
                [200.70458000987293, 174.73441003184456],
                [200.70458000987293, 222.7344100318445],
                [182.5666395548418, 194.79639356893196],
            ],
        ),
        (
            "cyclohexane_chair_2",
            [
                [120.0, 168.0],
                [120.0, 120.0],
                [182.5666395548418, 93.20360643106801],
                [200.7045800098729, 65.26558996815542],
                [200.70458000987293, 113.26558996815544],
                [138.13794045503107, 140.06198353708743],
            ],
        ),
    ];
    for (kind, expected) in cases {
        for side in ["positive", "negative"] {
            let mut engine = Engine::default();
            let state = serde_json::to_value(engine.dispatch(
                "ring.create",
                json!({
                    "expected_revision":0,"page_id":1,"kind":kind,
                    "hydrogen_policy":"rectify_neutral_carbon","placement":{
                        "mode":"free","begin_x_pt":120,"begin_y_pt":168,
                        "end_x_pt":120,"end_y_pt":120,"side":side
                    }
                }),
            )?)?;
            let atoms = state["document"]["pages"][0]["fragments"][0]["atoms"]
                .as_array()
                .ok_or("atoms")?;
            assert_eq!(atoms.len(), 6);
            for (atom, point) in atoms.iter().zip(expected) {
                let x = if side == "negative" {
                    240.0 - point[0]
                } else {
                    point[0]
                };
                assert!((atom["position"]["x_pt"].as_f64().ok_or("x")? - x).abs() < 1e-9);
                assert!((atom["position"]["y_pt"].as_f64().ok_or("y")? - point[1]).abs() < 1e-9);
                assert_eq!(atom["show_non_terminal_carbon_labels"], false);
                assert!(atom.get("show_terminal_carbon_labels").is_none());
            }
        }
    }
    Ok(())
}

#[test]
fn chair_reference_edge_supports_the_whole_projection_for_ordinary_attachment() -> TestResult {
    for kind in ["cyclohexane_chair_1", "cyclohexane_chair_2"] {
        let mut engine = Engine::default();
        let state = serde_json::to_value(engine.dispatch(
            "ring.create",
            json!({
                "expected_revision":0,"page_id":1,"kind":kind,
                "hydrogen_policy":"rectify_neutral_carbon","placement":{
                    "mode":"free","begin_x_pt":120,"begin_y_pt":120,
                    "end_x_pt":168,"end_y_pt":120,"side":"positive"
                }
            }),
        )?)?;
        let atoms = state["document"]["pages"][0]["fragments"][0]["atoms"]
            .as_array()
            .ok_or("atoms")?;
        for atom in atoms.iter().skip(2) {
            assert!(
                atom["position"]["y_pt"].as_f64().ok_or("y")? > 120.0,
                "{kind}: a concave reference edge defeats ordinary outward attachment: {atom}"
            );
        }
    }
    Ok(())
}

// Additive expectations from the locked orthographic projection, not a product builder.
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
const POSITION_TOLERANCE: f64 = 1e-9;

fn chair_params(engine: &Engine, kind: &str, placement: Value) -> Value {
    json!({"expected_revision":engine.state().revision,"page_id":1,"kind":kind,
        "hydrogen_policy":"rectify_neutral_carbon","placement":placement})
}
fn free(a: [f64; 2], b: [f64; 2], side: &str) -> Value {
    json!({"mode":"free","begin_x_pt":a[0],"begin_y_pt":a[1],
        "end_x_pt":b[0],"end_y_pt":b[1],"side":side})
}
fn normalized_chair(kind: &str) -> Check<[[f64; 2]; 6]> {
    // Same ideal chair and unit orthographic camera; outer reference edges use
    // One source indices[1,0,5,4,3,2], Two[3,2,1,0,5,4]. Eliminating
    // dot/cross normalization with D=(7+sqrt(6))/17 gives these closed radicals.
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
    match kind {
        "cyclohexane_chair_1" => Ok(one),
        "cyclohexane_chair_2" => Ok([
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0 - one[5][0], one[5][1]],
            [1.0 - one[4][0], one[4][1]],
            [1.0 - one[3][0], one[3][1]],
            [1.0 - one[2][0], one[2][1]],
        ]),
        _ => Err("chair kind".into()),
    }
}
fn chair_points(kind: &str, a: [f64; 2], b: [f64; 2], side: &str) -> Check<[[f64; 2]; 6]> {
    let e = [b[0] - a[0], b[1] - a[1]];
    let sign = if side == "positive" { 1.0 } else { -1.0 };
    let mut points = normalized_chair(kind)?.map(|[x, y]| {
        [
            a[0] + x * e[0] - sign * y * e[1],
            a[1] + x * e[1] + sign * y * e[0],
        ]
    });
    points[0] = a;
    points[1] = b;
    Ok(points)
}
fn area(points: &[[f64; 2]]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a[0] * b[1] - a[1] * b[0])
        .sum::<f64>()
        / 2.0
}
fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn graph(engine: &Engine) -> Check<Value> {
    Ok(serde_json::to_value(engine.document()?)?["pages"][0]["fragments"][0].clone())
}
fn literal(points: &[[f64; 2]], h: &[u16], edges: &[[u32; 2]], flags: &[Option<bool>]) -> Value {
    json!({"id":1,"atoms":points.iter().zip(h).zip(flags).enumerate().map(|(i,((p,h),flag))|{
        let mut a=json!({"id":i+1,"element":6,"isotope":null,"charge":0,"radical":"none",
            "explicit_hydrogens":h,"position":{"x_pt":p[0],"y_pt":p[1]}});
        if let Some(flag)=flag {a["show_non_terminal_carbon_labels"]=json!(flag);} a
    }).collect::<Vec<_>>(),"bonds":edges.iter().enumerate().map(|(i,[a,b])|json!({"id":i+1,"begin":a,"end":b,"order":"single"})).collect::<Vec<_>>()})
}
fn check_graph(actual: &Value, expected: &Value) -> TestResult {
    let mut a = actual.clone();
    let mut e = expected.clone();
    let aa = a["atoms"].as_array_mut().ok_or("actual atoms")?;
    let ee = e["atoms"].as_array_mut().ok_or("expected atoms")?;
    assert_eq!(aa.len(), ee.len());
    for (a, e) in aa.iter_mut().zip(ee) {
        for axis in ["x_pt", "y_pt"] {
            assert!(
                (a["position"][axis].as_f64().ok_or("actual coordinate")?
                    - e["position"][axis].as_f64().ok_or("expected coordinate")?)
                .abs()
                    <= POSITION_TOLERANCE
            );
        }
        a.as_object_mut().ok_or("actual atom")?.remove("position");
        e.as_object_mut().ok_or("expected atom")?.remove("position");
    }
    assert_eq!(a, e);
    Ok(())
}
fn seed(g: Value) -> Check<Engine> {
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(g["id"].as_u64().ok_or("fragment")?.try_into()?)
                .ok_or("fragment id")?,
            serde_json::from_value::<Vec<Atom>>(g["atoms"].clone())?,
            serde_json::from_value::<Vec<Bond>>(g["bonds"].clone())?,
        )?],
    )?;
    Ok(Engine::from_document(stored))
}
fn regular() -> Value {
    let r = 3.0_f64.sqrt();
    literal(
        &[
            [120.0, 168.0],
            [120.0, 120.0],
            [120.0 + 24.0 * r, 96.0],
            [120.0 + 48.0 * r, 120.0],
            [120.0 + 48.0 * r, 168.0],
            [120.0 + 24.0 * r, 192.0],
        ],
        &[2; 6],
        &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
        &[None; 6],
    )
}
fn reject(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let state = serde_json::to_value(engine.state())?;
    let whole = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let tool = engine.bond_tool();
    let view = engine.view();
    let file = engine.ui_file_context_stamp();
    let error = engine
        .dispatch("ring.create", params)
        .err()
        .ok_or("chair accepted invalid input")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert_eq!(engine.content(), &whole);
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.view(), view);
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
fn both_chairs_and_sides_follow_independent_orthographic_radicals_without_regularizing_edges()
-> TestResult {
    // The ideal 3D seed independently certifies its tetrahedral local geometry and chair plane.
    let h = 2.0_f64.sqrt() / 8.0;
    for s in [1.0, -1.0] {
        let p = [
            [1.0, 0.0, s * h],
            [0.5, 3.0_f64.sqrt() / 2.0, -s * h],
            [-0.5, 3.0_f64.sqrt() / 2.0, s * h],
            [-1.0, 0.0, -s * h],
            [-0.5, -3.0_f64.sqrt() / 2.0, s * h],
            [0.5, -3.0_f64.sqrt() / 2.0, -s * h],
        ];
        for i in 0..6 {
            let a = p[(i + 5) % 6];
            let b = p[i];
            let c = p[(i + 1) % 6];
            let u = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            let v = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
            let uu = u.iter().map(|v| v * v).sum::<f64>();
            let vv = v.iter().map(|v| v * v).sum::<f64>();
            assert!((uu - 9.0 / 8.0).abs() < 1e-14 && (vv - 9.0 / 8.0).abs() < 1e-14);
            assert!(
                (u.iter().zip(v).map(|(a, b)| a * b).sum::<f64>() / (9.0 / 8.0) + 1.0 / 3.0).abs()
                    < 1e-14
            );
        }
        let u = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let v = [p[3][0] - p[0][0], p[3][1] - p[0][1], p[3][2] - p[0][2]];
        let n = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let distance = |i: usize| {
            n.iter()
                .enumerate()
                .map(|(j, x)| x * (p[i][j] - p[0][j]))
                .sum::<f64>()
        };
        assert!(distance(4).abs() < 1e-14);
        assert!(distance(2) * distance(5) < 0.0);
    }
    for kind in ["cyclohexane_chair_1", "cyclohexane_chair_2"] {
        let t = normalized_chair(kind)?;
        assert!(area(&t) > 0.0);
        assert!(
            t.iter().skip(2).all(|p| p[1] > 0.0),
            "outer reference edge strictly supports every other vertex"
        );
        for i in 0..6 {
            for j in (i + 1)..6 {
                if j == i + 1 || (i == 0 && j == 5) {
                    continue;
                }
                let a = t[i];
                let b = t[(i + 1) % 6];
                let c = t[j];
                let d = t[(j + 1) % 6];
                assert!(
                    !(cross(a, b, c) * cross(a, b, d) < 0.0
                        && cross(c, d, a) * cross(c, d, b) < 0.0)
                );
            }
        }
        for side in ["positive", "negative"] {
            for (a, b) in [
                ([120.0, 168.0], [120.0, 120.0]),
                ([250.0, 300.0], [298.0, 300.0]),
                ([250.0, 300.0], [279.0, 347.0]),
            ] {
                let mut engine = Engine::default();
                let expected = chair_points(kind, a, b, side)?;
                let params = chair_params(&engine, kind, free(a, b, side));
                engine.dispatch("ring.create", params)?;
                check_graph(
                    &graph(&engine)?,
                    &literal(
                        &expected,
                        &[2; 6],
                        &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
                        &[Some(false); 6],
                    ),
                )?;
                assert_eq!(
                    graph(&engine)?["atoms"][0]["position"],
                    json!({"x_pt":a[0],"y_pt":a[1]})
                );
                assert_eq!(
                    graph(&engine)?["atoms"][1]["position"],
                    json!({"x_pt":b[0],"y_pt":b[1]})
                );
                let length = (b[0] - a[0]).hypot(b[1] - a[1]);
                assert!(
                    expected
                        .iter()
                        .zip(expected.iter().cycle().skip(1))
                        .take(6)
                        .any(
                            |(p, q)| ((p[0] - q[0]).hypot(p[1] - q[1]) / length - 1.0).abs() > 0.1
                        )
                );
                assert_eq!(area(&expected).is_sign_positive(), side == "positive");
                // Check actual authored output, not only the independently normalized table.
                for atom in graph(&engine)?["atoms"]
                    .as_array()
                    .ok_or("supporting vertices")?
                    .iter()
                    .skip(2)
                {
                    let p = [
                        atom["position"]["x_pt"].as_f64().ok_or("supporting x")?,
                        atom["position"]["y_pt"].as_f64().ok_or("supporting y")?,
                    ];
                    assert!(
                        cross(a, b, p) * (if side == "positive" { 1.0 } else { -1.0 }) > 0.0,
                        "every new vertex must lie strictly on the requested supporting-edge side"
                    );
                }
            }
        }
    }
    // Legacy regular and unsaturated requests keep their distinct unchanged geometry/data.
    let mut old = Engine::default();
    let mut p = chair_params(
        &old,
        "benzene",
        free([120.0, 168.0], [120.0, 120.0], "positive"),
    );
    p.as_object_mut().ok_or("params")?.remove("kind");
    p["size"] = json!(6);
    old.dispatch("ring.create", p)?;
    check_graph(&graph(&old)?, &regular())?;
    let mut benzene = Engine::default();
    let mut p = chair_params(
        &benzene,
        "benzene",
        free([120.0, 168.0], [120.0, 120.0], "positive"),
    );
    p["phase"] = json!("single_first");
    benzene.dispatch("ring.create", p)?;
    let g = graph(&benzene)?;
    assert_eq!(
        g["bonds"]
            .as_array()
            .ok_or("bonds")?
            .iter()
            .map(|b| b["order"].clone())
            .collect::<Vec<_>>(),
        json!(["single", "double", "single", "double", "single", "double"])
            .as_array()
            .ok_or("expected bonds")?
            .clone()
    );
    Ok(())
}

fn node<'a>(root: &'a Value, v: &'a Value) -> Check<&'a Value> {
    match v.get("$ref").and_then(Value::as_str) {
        Some(r) => root
            .pointer(r.strip_prefix('#').ok_or("nonlocal schema")?)
            .ok_or_else(|| "schema target".into()),
        None => Ok(v),
    }
}
#[test]
fn chair_objects_are_a_third_closed_nonphase_alternative_without_changing_named_kinds() -> TestResult
{
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 47);
    for command in ["ring.create", "ui.ring_tool"] {
        let entry = registry
            .iter()
            .find(|e| e.name == command)
            .ok_or("registry entry")?;
        let schema = serde_json::to_value(&entry.input_schema)?;
        assert_eq!(schema["type"], "object");
        let variants = schema
            .get("anyOf")
            .or_else(|| schema.get("oneOf"))
            .and_then(Value::as_array)
            .ok_or("alternatives")?;
        assert_eq!(variants.len(), 3);
        let mut seen = BTreeSet::new();
        for branch in variants {
            let branch = node(&schema, branch)?;
            assert_eq!(branch["type"], "object");
            assert_eq!(branch["additionalProperties"], false);
            let props = branch["properties"].as_object().ok_or("properties")?;
            let key = if props.contains_key("size") {
                "sized"
            } else {
                let values = &node(&schema, &props["kind"])?["enum"];
                if values == &json!(["benzene", "cyclopentadiene"]) {
                    "named"
                } else {
                    assert_eq!(
                        values,
                        &json!(["cyclohexane_chair_1", "cyclohexane_chair_2"])
                    );
                    "chair"
                }
            };
            assert!(seen.insert(key));
            if key == "chair" {
                let expected = if command == "ring.create" {
                    vec![
                        "expected_revision",
                        "page_id",
                        "kind",
                        "hydrogen_policy",
                        "placement",
                    ]
                } else {
                    vec!["kind"]
                };
                let want = expected.into_iter().collect::<BTreeSet<_>>();
                assert_eq!(
                    props.keys().map(String::as_str).collect::<BTreeSet<_>>(),
                    want
                );
                assert_eq!(
                    branch["required"]
                        .as_array()
                        .ok_or("required")?
                        .iter()
                        .map(|v| v.as_str().ok_or("name"))
                        .collect::<Result<BTreeSet<_>, _>>()?,
                    want
                );
            }
        }
        assert_eq!(seen, ["sized", "named", "chair"].into_iter().collect());
    }
    let engine = Engine::default();
    for kind in ["cyclohexane_chair_1", "cyclohexane_chair_2"] {
        let valid = chair_params(
            &engine,
            kind,
            free([120.0, 168.0], [120.0, 120.0], "positive"),
        );
        for (command, params) in [
            ("ring.create", valid.clone()),
            ("ui.ring_tool", json!({"kind":kind})),
        ] {
            let parsed = Command::parse(command, params.clone())?;
            assert_eq!(
                serde_json::to_value(parsed)?,
                json!({"command":command,"params":params})
            );
        }
        for (field, value) in [
            ("phase", Value::Null),
            ("phase", json!("double_first")),
            ("phase", json!("single_first")),
            ("size", json!(6)),
            ("kind", Value::Null),
            ("kind", json!({"cyclohexane_chair_1":null})),
            ("extra", json!(true)),
        ] {
            let mut p = valid.clone();
            p[field] = value;
            let mut e = Engine::default();
            reject(&mut e, p, "invalid_params")?;
        }
        for field in [
            "expected_revision",
            "page_id",
            "kind",
            "hydrogen_policy",
            "placement",
        ] {
            let mut p = valid.clone();
            p.as_object_mut().ok_or("params")?.remove(field);
            let mut e = Engine::default();
            reject(&mut e, p, "invalid_params")?;
        }
        for invalid in [
            json!({"kind":kind,"phase":"single_first"}),
            json!({"kind":kind,"size":6}),
            json!({"kind":kind,"extra":true}),
        ] {
            assert_eq!(
                Command::parse("ui.ring_tool", invalid)
                    .err()
                    .ok_or("tool extra accepted")?
                    .info()
                    .code,
                "invalid_params"
            );
        }
    }
    Ok(())
}

#[test]
fn chair_precision_occupancy_weighted_h_and_foreign_source_refusals_are_atomic() -> TestResult {
    for kind in ["cyclohexane_chair_1", "cyclohexane_chair_2"] {
        let mut e = Engine::default();
        let a = 65_536.0_f64;
        let b = f64::from_bits(a.to_bits() + 1);
        for (a, b) in [
            ([a, a], [b, a]),
            ([0.0, 0.0], [f64::from_bits(1), 0.0]),
            ([99_999.0, 99_999.0], [100_000.0, 99_999.0]),
            ([120.0, 120.0], [120.0, 120.0]),
        ] {
            let p = chair_params(&e, kind, free(a, b, "positive"));
            reject(&mut e, p, "invalid_params")?;
        }
        let p = chair_params(&e, kind, free([0.0, 0.0], [1e-200, 0.0], "positive"));
        e.dispatch("ring.create", p)?;
        let g = graph(&e)?;
        let t = normalized_chair(kind)?;
        for (a, t) in g["atoms"].as_array().ok_or("atoms")?.iter().zip(t) {
            assert!((a["position"]["x_pt"].as_f64().ok_or("x")? / 1e-200 - t[0]).abs() <= 1e-10);
            assert!((a["position"]["y_pt"].as_f64().ok_or("y")? / 1e-200 - t[1]).abs() <= 1e-10);
        }
        let p = chair_params(&e, kind, free([0.0, 0.0], [48.0, 0.0], "negative"));
        reject(&mut e, p, "invalid_params")?;
        for order in ["double", "triple"] {
            let mut pair = Engine::default();
            pair.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":120,"begin_y_pt":168,"end_x_pt":120,"end_y_pt":120,"order":order}))?;
            let p = chair_params(
                &pair,
                kind,
                json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"positive"}),
            );
            reject(&mut pair, p, "unsupported_content")?;
        }
        let mut native =
            Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
        let mut stale = chair_params(
            &native,
            kind,
            free([300.0, 168.0], [300.0, 120.0], "positive"),
        );
        stale["expected_revision"] = json!(1);
        reject(&mut native, stale, "stale_revision")?;
        let opaque = String::from_utf8(NATIVE.to_vec())?
            .replace("<CDXML ", "<CDXML Private=\"retain-all\" ");
        let mut foreign = Engine::from_document(StoredDocument::from_cdxml(
            opaque.as_bytes().into(),
            PageSize::A4,
        )?);
        let p = chair_params(
            &foreign,
            kind,
            free([300.0, 168.0], [300.0, 120.0], "positive"),
        );
        reject(&mut foreign, p, "unsupported_content")?;
        assert_eq!(
            foreign.stored_document()?.export_cdxml()?,
            opaque.as_bytes()
        );
    }
    Ok(())
}

#[test]
fn shared_atom_keeps_sparse_metadata_and_undo_branches_never_reuse_allocated_ids() -> TestResult {
    let mut input = literal(
        &[[72.0, 120.0], [120.0, 120.0]],
        &[3, 3],
        &[[1, 2]],
        &[Some(true), None],
    );
    input["id"] = json!(7);
    input["atoms"][0]["id"] = json!(10);
    input["atoms"][1]["id"] = json!(40);
    input["atoms"][0]["show_terminal_carbon_labels"] = json!(true);
    input["atoms"][1]["show_terminal_carbon_labels"] = json!(false);
    input["bonds"][0] = json!({"id":70,"begin":40,"end":10,"order":"single"});
    let mut e = seed(input.clone())?;
    let original = e.content().clone();
    let p = chair_params(
        &e,
        "cyclohexane_chair_1",
        json!({"mode":"atom","fragment_id":7,"atom_id":40,"end_x_pt":168,"end_y_pt":120,"side":"positive"}),
    );
    e.dispatch("ring.create", p)?;
    let edited = e.content().clone();
    let g = graph(&e)?;
    assert_eq!(g["id"], 7);
    assert_eq!(g["atoms"][0], input["atoms"][0]);
    let mut anchor = input["atoms"][1].clone();
    anchor["explicit_hydrogens"] = json!(1);
    assert_eq!(g["atoms"][1], anchor);
    assert_eq!(g["bonds"][0], input["bonds"][0]);
    assert_eq!(
        g["atoms"]
            .as_array()
            .ok_or("atoms")?
            .iter()
            .map(|a| a["id"].clone())
            .collect::<Vec<_>>(),
        vec![
            json!(10),
            json!(40),
            json!(41),
            json!(42),
            json!(43),
            json!(44),
            json!(45)
        ]
    );
    for a in g["atoms"].as_array().ok_or("atoms")?.iter().skip(2) {
        assert_eq!(a["explicit_hydrogens"], 2);
        assert_eq!(a["show_non_terminal_carbon_labels"], false);
        assert!(a.get("show_terminal_carbon_labels").is_none());
    }
    let perimeter = [40, 41, 42, 43, 44, 45];
    for (i, b) in g["bonds"]
        .as_array()
        .ok_or("bonds")?
        .iter()
        .skip(1)
        .enumerate()
    {
        assert_eq!(
            b,
            &json!({"id":71+i,"begin":perimeter[i],"end":perimeter[(i+1)%6],"order":"single"})
        );
    }
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.content(), &original);
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(e.content(), &edited);
    e.dispatch("history.undo", json!({}))?;
    let p = chair_params(
        &e,
        "cyclohexane_chair_2",
        free([300.0, 240.0], [348.0, 240.0], "negative"),
    );
    e.dispatch("ring.create", p)?;
    let v = serde_json::to_value(e.document()?)?;
    let f = &v["pages"][0]["fragments"][1];
    assert_eq!(f["id"], 8);
    assert_eq!(f["atoms"][0]["id"], 46);
    assert_eq!(f["atoms"][5]["id"], 51);
    assert_eq!(f["bonds"][0]["id"], 77);
    assert_eq!(f["bonds"][5]["id"], 82);
    assert_eq!(e.state().redo_depth, 0);
    Ok(())
}

fn consume(
    path: &Path,
    expected: &Value,
    smiles: &str,
    formula: &str,
    flags: &[Option<bool>],
) -> TestResult {
    let raw = fs::read(path)?;
    assert!(
        !std::str::from_utf8(&raw)?.contains("ShowTerminalCarbonLabels"),
        "no requested terminal flag in these literal inputs"
    );
    let args = serde_json::to_string(
        &json!({"graph":[expected],"smiles":smiles,"formula":formula,"flags":flags,
        "double_positions":vec![Option::<&str>::None;expected["bonds"].as_array().ok_or("bonds")?.len()]}),
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
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(&raw)?;
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
fn five_unicode_files_keep_native_source_and_match_unchanged_raw_decimal_and_rdkit_consumers()
-> TestResult {
    let dir = support::FileTestDirectory::new()?;
    for (case, kind, smiles, formula) in [
        ("free", "cyclohexane_chair_1", "C1CCCCC1", "C6H12"),
        ("atom", "cyclohexane_chair_1", "CC1CCCCC1", "C7H14"),
        (
            "spiro",
            "cyclohexane_chair_2",
            "C1CCC2(CC1)CCCCC2",
            "C11H20",
        ),
        ("edge", "cyclohexane_chair_2", "C1CCCCC1", "C6H12"),
        ("fused", "cyclohexane_chair_1", "C1CCC2CCCCC2C1", "C10H18"),
    ] {
        let mut e = if case == "atom" || case == "edge" {
            Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?)
        } else if case == "spiro" || case == "fused" {
            seed(regular())?
        } else {
            Engine::default()
        };
        let input = dir.path(&format!("{case}-前-λ.cdxml"));
        e.dispatch("document.save_cdxml", json!({"path":input}))?;
        let input_bytes = fs::read(&input)?;
        if case == "atom" || case == "edge" {
            assert_eq!(input_bytes, NATIVE);
        }
        let original = e.content().clone();
        let original_model = e.stored_document()?.model_snapshot();
        let original_ptr = e
            .stored_document()?
            .export_cdxml()
            .ok()
            .map(|raw| raw.as_ptr());
        let (placement, expected, flags) = if case == "free" {
            let ps = chair_points(kind, [120.0, 168.0], [120.0, 120.0], "positive")?;
            (
                free([120.0, 168.0], [120.0, 120.0], "positive"),
                literal(
                    &ps,
                    &[2; 6],
                    &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
                    &[Some(false); 6],
                ),
                vec![Some(false); 6],
            )
        } else if case == "atom" {
            let ps = chair_points(kind, [140.0, 200.0], [188.0, 200.0], "positive")?;
            let mut pts = vec![[100.0, 200.0], [140.0, 200.0]];
            pts.extend_from_slice(&ps[1..]);
            let flags = [vec![None; 2], vec![Some(false); 5]].concat();
            (
                json!({"mode":"atom","fragment_id":1,"atom_id":2,"end_x_pt":188,"end_y_pt":200,"side":"positive"}),
                literal(
                    &pts,
                    &[3, 1, 2, 2, 2, 2, 2],
                    &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7], [7, 2]],
                    &flags,
                ),
                flags,
            )
        } else if case == "edge" {
            let ps = chair_points(kind, [100.0, 200.0], [140.0, 200.0], "positive")?;
            let flags = [vec![None; 2], vec![Some(false); 4]].concat();
            (
                json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"positive"}),
                literal(
                    &ps,
                    &[2; 6],
                    &[[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
                    &flags,
                ),
                flags,
            )
        } else {
            let old = regular();
            let mut pts = old["atoms"]
                .as_array()
                .ok_or("old atoms")?
                .iter()
                .map(|a| {
                    Ok([
                        a["position"]["x_pt"].as_f64().ok_or("x")?,
                        a["position"]["y_pt"].as_f64().ok_or("y")?,
                    ])
                })
                .collect::<Check<Vec<_>>>()?;
            if case == "spiro" {
                let ps = chair_points(kind, [120.0, 168.0], [72.0, 168.0], "negative")?;
                pts.extend_from_slice(&ps[1..]);
                let flags = [vec![None; 6], vec![Some(false); 5]].concat();
                (
                    json!({"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":72,"end_y_pt":168,"side":"negative"}),
                    literal(
                        &pts,
                        &[0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
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
                        &flags,
                    ),
                    flags,
                )
            } else {
                let ps = chair_points(kind, [120.0, 168.0], [120.0, 120.0], "negative")?;
                pts.extend_from_slice(&ps[2..]);
                let flags = [vec![None; 6], vec![Some(false); 4]].concat();
                (
                    json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"negative"}),
                    literal(
                        &pts,
                        &[1, 1, 2, 2, 2, 2, 2, 2, 2, 2],
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
                        &flags,
                    ),
                    flags,
                )
            }
        };
        let p = chair_params(&e, kind, placement);
        let result = e.dispatch("ring.create", p)?;
        assert!(result.changed);
        assert_eq!(result.undo_depth, 1);
        assert!(result.file.dirty);
        assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
        check_graph(&graph(&e)?, &expected)?;
        let source = e.stored_document()?.source_info();
        if let Some(source) = source {
            assert_eq!(source.format, "cdxml");
            assert!(source.modified);
        }
        assert_eq!(fs::read(&input)?, input_bytes);
        let edited = e.content().clone();
        let output = dir.path(&format!("{case}-椅子-化学.cdxml"));
        e.dispatch("document.save_cdxml", json!({"path":output}))?;
        assert!(!e.state().file.dirty);
        consume(&output, &expected, smiles, formula, &flags)?;
        e.dispatch("history.undo", json!({}))?;
        assert_eq!(e.content(), &original);
        assert!(Arc::ptr_eq(
            &original_model,
            &e.stored_document()?.model_snapshot()
        ));
        if let Some(original_ptr) = original_ptr {
            assert_eq!(e.stored_document()?.export_cdxml()?.as_ptr(), original_ptr);
        }
        assert!(e.state().file.dirty);
        e.dispatch("history.redo", json!({}))?;
        assert_eq!(e.content(), &edited);
        assert!(!e.state().file.dirty);
        let saved = fs::read(&output)?;
        e.dispatch("document.open_cdxml", json!({"path":output}))?;
        check_graph(&graph(&e)?, &expected)?;
        assert!(e.stored_document()?.is_fully_mapped());
        assert!(!e.state().file.dirty);
        assert!(
            !e.stored_document()?
                .source_info()
                .ok_or("reopened source")?
                .modified
        );
        assert_eq!(fs::read(&output)?, saved);
        assert_eq!(fs::read(&input)?, input_bytes);
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    #[test]
    fn generated_chair_sequences_restore_complete_stored_snapshots(choices in prop::collection::vec((any::<bool>(),any::<bool>()),1..6)) {
        let mut e=Engine::default();let original=e.content().clone();let mut snapshots=Vec::new();
        for (i,(two,negative)) in choices.iter().enumerate() {
            let kind=if *two {"cyclohexane_chair_2"} else {"cyclohexane_chair_1"};
            let side=if *negative {"negative"} else {"positive"};let x=144.0+192.0*i as f64;
            let p=chair_params(&e,kind,free([x,240.0],[x+48.0,240.0],side));let result=e.dispatch("ring.create",p)?;
            prop_assert!(result.changed);prop_assert_eq!(result.undo_depth,i+1);snapshots.push(e.content().clone());
        }
        for expected in snapshots.iter().rev().skip(1) {e.dispatch("history.undo",json!({}))?;prop_assert_eq!(e.content(),expected);}
        e.dispatch("history.undo",json!({}))?;prop_assert_eq!(e.content(),&original);
        for expected in &snapshots {e.dispatch("history.redo",json!({}))?;prop_assert_eq!(e.content(),expected);}
    }
}
