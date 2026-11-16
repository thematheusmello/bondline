//! Named ring command through existing JSON APIs; no future Rust type dependency.
use bondline_commands::Engine;
use serde_json::{Value, json};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn free_benzene_defaults_to_double_first_with_literal_h1_skeletal_flags_and_inward_strokes()
-> TestResult {
    let mut engine = Engine::default();
    let result = engine.dispatch(
        "ring.create",
        json!({"expected_revision":0,"page_id":1,
        "kind":"benzene","hydrogen_policy":"rectify_neutral_carbon","placement":{
        "mode":"free","begin_x_pt":120,"begin_y_pt":168,
        "end_x_pt":120,"end_y_pt":120,"side":"positive"}}),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (1, 1, 0)
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
    assert_eq!((atoms.len(), bonds.len()), (6, 6));
    let r = 3.0_f64.sqrt();
    let points = [
        [120.0, 168.0],
        [120.0, 120.0],
        [120.0 + 24.0 * r, 96.0],
        [120.0 + 48.0 * r, 120.0],
        [120.0 + 48.0 * r, 168.0],
        [120.0 + 24.0 * r, 192.0],
    ];
    for (i, (atom, p)) in atoms.iter().zip(points).enumerate() {
        assert_eq!(atom["id"], i + 1);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], 1);
        assert_eq!(atom["show_non_terminal_carbon_labels"], false);
        assert_eq!(atom["isotope"], Value::Null);
        assert_eq!(atom["charge"], 0);
        assert_eq!(atom["radical"], "none");
        for (axis, want) in ["x_pt", "y_pt"].into_iter().zip(p) {
            assert!((atom["position"][axis].as_f64().ok_or("position")? - want).abs() <= 1e-9);
        }
    }
    for (i, bond) in bonds.iter().enumerate() {
        assert_eq!(
            bond,
            &if i % 2 == 0 {
                json!({"id":i+1,"begin":i+1,
            "end":if i==5{1}else{i+2},"order":"double","double_position":"right"})
            } else {
                json!({"id":i+1,"begin":i+1,"end":if i==5{1}else{i+2},"order":"single"})
            }
        );
    }
    Ok(())
}

// Additive declaration; adopted only as a child of tests/ring_create.rs.
use super::{carbon_graph, original_hexagon, refuse, schema_node, support};
use bondline_io::StoredDocument;
use bondline_model::{Atom, Bond, Fragment, FragmentId, PageId, PageSize};
use std::{collections::BTreeSet, fs, sync::Arc};

fn params(engine: &Engine, kind: &str, phase: Option<&str>, placement: Value) -> Value {
    let mut p = json!({"expected_revision":engine.state().revision,"page_id":1,"kind":kind,
        "hydrogen_policy":"rectify_neutral_carbon","placement":placement});
    if let Some(phase) = phase {
        p["phase"] = json!(phase);
    }
    p
}
fn free(side: &str) -> Value {
    json!({"mode":"free","begin_x_pt":120,"begin_y_pt":168,
    "end_x_pt":120,"end_y_pt":120,"side":side})
}
fn graph(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.document()?)?["pages"][0]["fragments"][0].clone())
}
fn points(n: usize, side: &str) -> Vec<[f64; 2]> {
    let sign = if side == "positive" { 1.0 } else { -1.0 };
    if n == 6 {
        let r = 3.0_f64.sqrt();
        vec![
            [120.0, 168.0],
            [120.0, 120.0],
            [120.0 + sign * 24.0 * r, 96.0],
            [120.0 + sign * 48.0 * r, 120.0],
            [120.0 + sign * 48.0 * r, 168.0],
            [120.0 + sign * 24.0 * r, 192.0],
        ]
    } else {
        // Closed radical pentagon coordinates, independent of sequential edge rotation.
        let r = 5.0_f64.sqrt();
        let dx = 12.0 * (10.0 + 2.0 * r).sqrt();
        let far = 12.0 * ((10.0 + 2.0 * r).sqrt() + (10.0 - 2.0 * r).sqrt());
        vec![
            [120.0, 168.0],
            [120.0, 120.0],
            [120.0 + sign * dx, 120.0 - 12.0 * (r - 1.0)],
            [120.0 + sign * far, 144.0],
            [120.0 + sign * dx, 168.0 + 12.0 * (r - 1.0)],
        ]
    }
}
fn positions(actual: &Value, want: &[[f64; 2]]) -> TestResult {
    let atoms = actual["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), want.len());
    for (a, p) in atoms.iter().zip(want) {
        for (k, x) in ["x_pt", "y_pt"].into_iter().zip(p) {
            assert!((a["position"][k].as_f64().ok_or("point")? - x).abs() <= 1e-9);
        }
    }
    Ok(())
}
fn fixed(graph: &Value, h: &[u16], weights: &[u8]) -> TestResult {
    assert_eq!(graph["atoms"].as_array().ok_or("atoms")?.len(), h.len());
    assert_eq!(
        graph["bonds"].as_array().ok_or("bonds")?.len(),
        weights.len()
    );
    assert_eq!(
        graph["atoms"]
            .as_array()
            .ok_or("atoms")?
            .iter()
            .map(|a| a["explicit_hydrogens"].clone())
            .collect::<Vec<_>>(),
        h.iter().map(|h| json!(h)).collect::<Vec<_>>()
    );
    assert_eq!(
        graph["bonds"]
            .as_array()
            .ok_or("bonds")?
            .iter()
            .map(|b| b["order"].clone())
            .collect::<Vec<_>>(),
        weights
            .iter()
            .map(|w| json!(if *w == 2 { "double" } else { "single" }))
            .collect::<Vec<_>>()
    );
    Ok(())
}
fn literal_graph(
    h: &[u16],
    p: &[[f64; 2]],
    edges: &[[u32; 2]],
    weights: &[u8],
) -> TestResult<Value> {
    let mut result = carbon_graph(h, p, edges);
    assert_eq!(weights.len(), edges.len());
    for (b, w) in result["bonds"]
        .as_array_mut()
        .ok_or("bonds")?
        .iter_mut()
        .zip(weights)
    {
        b["order"] = json!(if *w == 2 { "double" } else { "single" });
    }
    Ok(result)
}
fn seed(fragment: u32, atoms: Value, bonds: Value) -> TestResult<Engine> {
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(fragment).ok_or("fragment")?,
            serde_json::from_value::<Vec<Atom>>(atoms)?,
            serde_json::from_value::<Vec<Bond>>(bonds)?,
        )?],
    )?;
    Ok(Engine::from_document(stored))
}
fn c(id: u32, x: f64, y: f64, h: u16, flag: Option<bool>) -> Value {
    let mut a = json!({"id":id,"element":6,"position":{"x_pt":x,"y_pt":y},
        "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h});
    if let Some(flag) = flag {
        a["show_non_terminal_carbon_labels"] = json!(flag);
    }
    a
}

#[test]
fn both_named_free_phases_and_sides_have_literal_h_and_metadata_without_altering_legacy_size()
-> TestResult {
    for (kind, phase, weights, h) in [
        ("benzene", None, vec![2, 1, 2, 1, 2, 1], vec![1; 6]),
        (
            "benzene",
            Some("double_first"),
            vec![2, 1, 2, 1, 2, 1],
            vec![1; 6],
        ),
        (
            "benzene",
            Some("single_first"),
            vec![1, 2, 1, 2, 1, 2],
            vec![1; 6],
        ),
        (
            "cyclopentadiene",
            None,
            vec![2, 1, 2, 1, 1],
            vec![1, 1, 1, 1, 2],
        ),
        (
            "cyclopentadiene",
            Some("double_first"),
            vec![2, 1, 2, 1, 1],
            vec![1, 1, 1, 1, 2],
        ),
        (
            "cyclopentadiene",
            Some("single_first"),
            vec![1, 2, 1, 1, 2],
            vec![1, 1, 1, 2, 1],
        ),
    ] {
        for side in ["positive", "negative"] {
            let mut engine = Engine::default();
            let before = engine.content().clone();
            let p = params(&engine, kind, phase, free(side));
            engine.dispatch("ring.create", p)?;
            let f = graph(&engine)?;
            fixed(&f, &h, &weights)?;
            positions(&f, &points(weights.len(), side))?;
            for a in f["atoms"].as_array().ok_or("atoms")? {
                assert_eq!(a["show_non_terminal_carbon_labels"], false);
            }
            for (b, w) in f["bonds"]
                .as_array()
                .ok_or("bonds")?
                .iter()
                .zip(weights.iter().copied())
            {
                if w == 2 {
                    assert_eq!(
                        b["double_position"],
                        if side == "positive" { "right" } else { "left" }
                    );
                } else {
                    assert!(b.get("double_position").is_none());
                }
            }
            let changed = engine.content().clone();
            engine.dispatch("history.undo", json!({}))?;
            assert_eq!(engine.content(), &before);
            engine.dispatch("history.redo", json!({}))?;
            assert_eq!(engine.content(), &changed);
        }
    }
    // Same command, exact legacy request: no flag, H2 and only Single order.
    let mut old = Engine::default();
    old.dispatch(
        "ring.create",
        json!({"expected_revision":0,"page_id":1,"size":6,
        "hydrogen_policy":"rectify_neutral_carbon","placement":free("positive")}),
    )?;
    let f = graph(&old)?;
    fixed(&f, &[2; 6], &[1; 6])?;
    positions(&f, &points(6, "positive"))?;
    for a in f["atoms"].as_array().ok_or("atoms")? {
        assert!(a.get("show_non_terminal_carbon_labels").is_none());
    }
    for b in f["bonds"].as_array().ok_or("bonds")? {
        assert!(b.get("double_position").is_none());
    }
    Ok(())
}

#[test]
fn reused_single_or_double_edge_selects_phase_and_preserves_all_four_shared_double_styles()
-> TestResult {
    for (order, h, style, weights) in [
        ("single", 3, None, vec![1, 2, 1, 2, 1, 2]),
        ("double", 2, None, vec![2, 1, 2, 1, 2, 1]),
        ("double", 2, Some("center"), vec![2, 1, 2, 1, 2, 1]),
        ("double", 2, Some("left"), vec![2, 1, 2, 1, 2, 1]),
        ("double", 2, Some("right"), vec![2, 1, 2, 1, 2, 1]),
    ] {
        let mut bond = json!({"id":41,"begin":29,"end":11,"order":order});
        if let Some(style) = style {
            bond["double_position"] = json!(style);
        }
        let mut engine = seed(
            17,
            json!([
                c(29, 120.0, 168.0, h, Some(true)),
                c(11, 120.0, 120.0, h, None)
            ]),
            json!([bond.clone()]),
        )?;
        let before = engine.content().clone();
        let old = graph(&engine)?;
        let p = params(
            &engine,
            "benzene",
            None,
            json!({"mode":"bond","fragment_id":17,"bond_id":41,"side":"positive"}),
        );
        engine.dispatch("ring.create", p)?;
        let f = graph(&engine)?;
        fixed(&f, &[1; 6], &weights)?;
        positions(&f, &points(6, "positive"))?;
        assert_eq!(f["id"], 17);
        assert_eq!(f["bonds"][0], bond);
        for i in 0..2 {
            let mut old_atom = old["atoms"][i].clone();
            old_atom["explicit_hydrogens"] = json!(1);
            assert_eq!(f["atoms"][i], old_atom);
        }
        assert_eq!(
            f["atoms"]
                .as_array()
                .ok_or("atoms")?
                .iter()
                .map(|a| a["id"].clone())
                .collect::<Vec<_>>(),
            vec![
                json!(29),
                json!(11),
                json!(30),
                json!(31),
                json!(32),
                json!(33)
            ]
        );
        let perimeter = [29, 11, 30, 31, 32, 33];
        for i in 1..6 {
            assert_eq!(f["bonds"][i]["id"], 41 + i);
            assert_eq!(f["bonds"][i]["begin"], perimeter[i]);
            assert_eq!(f["bonds"][i]["end"], perimeter[(i + 1) % 6]);
            if weights[i] == 2 {
                assert_eq!(f["bonds"][i]["double_position"], "right");
            } else {
                assert!(f["bonds"][i].get("double_position").is_none());
            }
        }
        for i in 2..6 {
            assert_eq!(f["atoms"][i]["show_non_terminal_carbon_labels"], false);
        }
        let after = engine.content().clone();
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.content(), &before);
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.content(), &after);
    }
    Ok(())
}

#[test]
fn cyclopentadiene_uses_asymmetric_endpoint_budgets_and_the_two_hydrogen_spiro_rotation()
-> TestResult {
    // X1--B2--E3--Y4; only the target's B/E direction changes in the last row.
    for (left, right, reverse, h, weights, new_h) in [
        (
            "single",
            "single",
            false,
            [3, 2, 2, 3],
            vec![1, 2, 1, 1, 2],
            vec![1, 2, 1],
        ),
        (
            "single",
            "double",
            false,
            [3, 2, 1, 2],
            vec![1, 1, 2, 1, 2],
            vec![1, 1, 1],
        ),
        (
            "double",
            "single",
            false,
            [2, 1, 2, 3],
            vec![1, 2, 1, 2, 1],
            vec![1, 1, 1],
        ),
        (
            "single",
            "double",
            true,
            [3, 2, 1, 2],
            vec![1, 2, 1, 2, 1],
            vec![1, 1, 1],
        ),
    ] {
        let target = if reverse {
            json!({"id":2,"begin":3,"end":2,"order":"single"})
        } else {
            json!({"id":2,"begin":2,"end":3,"order":"single"})
        };
        let mut engine = seed(
            1,
            json!([
                c(1, 72.0, 192.0, h[0], None),
                c(2, 120.0, 168.0, h[1], Some(true)),
                c(3, 120.0, 120.0, h[2], Some(false)),
                c(4, 72.0, 96.0, h[3], None)
            ]),
            json!([{"id":1,"begin":1,"end":2,"order":left},target,
                {"id":3,"begin":3,"end":4,"order":right}]),
        )?;
        let old = graph(&engine)?;
        let p = params(
            &engine,
            "cyclopentadiene",
            None,
            json!({"mode":"bond","fragment_id":1,"bond_id":2,"side":if reverse{"negative"}else{"positive"}}),
        );
        engine.dispatch("ring.create", p)?;
        let f = graph(&engine)?;
        assert_eq!(
            &f["bonds"].as_array().ok_or("bonds")?[..3],
            old["bonds"].as_array().ok_or("old bonds")?
        );
        let mut expected_h = vec![h[0], 0, 0, h[3]];
        expected_h.extend(&new_h);
        let mut expected_weights = vec![
            if left == "double" { 2 } else { 1 },
            1,
            if right == "double" { 2 } else { 1 },
        ];
        expected_weights.extend_from_slice(&weights[1..]);
        fixed(&f, &expected_h, &expected_weights)?;
        for i in 0..4 {
            let mut atom = old["atoms"][i].clone();
            if i == 1 || i == 2 {
                atom["explicit_hydrogens"] = json!(0);
            }
            assert_eq!(f["atoms"][i], atom);
        }
        let perimeter = if reverse {
            [3, 2, 5, 6, 7]
        } else {
            [2, 3, 5, 6, 7]
        };
        for i in 1..5 {
            assert_eq!(f["bonds"][i + 2]["begin"], perimeter[i]);
            assert_eq!(f["bonds"][i + 2]["end"], perimeter[(i + 1) % 5]);
        }
        assert_eq!(
            f["atoms"]
                .as_array()
                .ok_or("atoms")?
                .iter()
                .skip(4)
                .map(|a| a["show_non_terminal_carbon_labels"].clone())
                .collect::<Vec<_>>(),
            vec![json!(false); 3]
        );
    }
    let mut engine = original_hexagon()?;
    let placement = json!({"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":120,"end_y_pt":216,"side":"positive"});
    let impossible = params(
        &engine,
        "cyclopentadiene",
        Some("double_first"),
        placement.clone(),
    );
    refuse(&mut engine, impossible, "unsupported_content")?;
    let impossible = params(&engine, "benzene", None, placement.clone());
    refuse(&mut engine, impossible, "unsupported_content")?;
    let old = graph(&engine)?;
    let p = params(&engine, "cyclopentadiene", None, placement);
    engine.dispatch("ring.create", p)?;
    let f = graph(&engine)?;
    fixed(
        &f,
        &[0, 2, 2, 2, 2, 2, 1, 1, 1, 1],
        &[1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 1],
    )?;
    assert_eq!(
        &f["bonds"].as_array().ok_or("bonds")?[..6],
        old["bonds"].as_array().ok_or("old bonds")?
    );
    assert_eq!(
        f["atoms"][0]["show_non_terminal_carbon_labels"],
        Value::Null
    );
    Ok(())
}

#[test]
fn actual_unicode_files_match_independent_expat_decimal_rdkit_and_native_source_history()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let r = 3.0_f64.sqrt();
    let pent = 5.0_f64.sqrt();
    let dx = 12.0 * (10.0 + 2.0 * pent).sqrt();
    let far = 12.0 * ((10.0 + 2.0 * pent).sqrt() + (10.0 - 2.0 * pent).sqrt());
    for name in ["benzene", "cyclopentadiene", "toluene", "fused", "spiro"] {
        let mut engine = if name == "toluene" {
            Engine::from_document(StoredDocument::from_cdxml(
                super::NATIVE.into(),
                PageSize::A4,
            )?)
        } else if name == "fused" || name == "spiro" {
            original_hexagon()?
        } else {
            Engine::default()
        };
        let original = engine.content().clone();
        let original_model = engine.stored_document()?.model_snapshot();
        let old_path = directory.path("native-before-原.cdxml");
        if name == "toluene" {
            engine.dispatch("document.save_cdxml", json!({"path":old_path}))?;
            assert_eq!(fs::read(&old_path)?, super::NATIVE);
            assert!(Arc::ptr_eq(
                &original_model,
                &engine.stored_document()?.model_snapshot()
            ));
        }
        let (kind, placement, h, p, edges, w, smiles, formula) = match name {
            "benzene" => (
                "benzene",
                free("positive"),
                vec![1; 6],
                points(6, "positive"),
                vec![[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 1]],
                vec![2, 1, 2, 1, 2, 1],
                "c1ccccc1",
                "C6H6",
            ),
            "cyclopentadiene" => (
                "cyclopentadiene",
                free("positive"),
                vec![1, 1, 1, 1, 2],
                points(5, "positive"),
                vec![[1, 2], [2, 3], [3, 4], [4, 5], [5, 1]],
                vec![2, 1, 2, 1, 1],
                "C1=CC=CC1",
                "C5H6",
            ),
            "toluene" => (
                "benzene",
                json!({"mode":"atom","fragment_id":1,"atom_id":2,
                "end_x_pt":160,"end_y_pt":200.0-20.0*r,"side":"positive"}),
                vec![3, 0, 1, 1, 1, 1, 1],
                vec![
                    [100.0, 200.0],
                    [140.0, 200.0],
                    [160.0, 200.0 - 20.0 * r],
                    [200.0, 200.0 - 20.0 * r],
                    [220.0, 200.0],
                    [200.0, 200.0 + 20.0 * r],
                    [160.0, 200.0 + 20.0 * r],
                ],
                vec![[1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7], [7, 2]],
                vec![1, 2, 1, 2, 1, 2, 1],
                "Cc1ccccc1",
                "C7H8",
            ),
            "fused" => (
                "benzene",
                json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"negative"}),
                vec![0, 0, 2, 2, 2, 2, 1, 1, 1, 1],
                vec![
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
                vec![
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
                vec![1, 1, 1, 1, 1, 1, 2, 1, 2, 1, 2],
                "c1ccc2CCCCc2c1",
                "C10H12",
            ),
            _ => (
                "cyclopentadiene",
                json!({"mode":"atom","fragment_id":1,"atom_id":1,
                "end_x_pt":120,"end_y_pt":216,"side":"positive"}),
                vec![0, 2, 2, 2, 2, 2, 1, 1, 1, 1],
                vec![
                    [120.0, 168.0],
                    [120.0, 120.0],
                    [120.0 + 24.0 * r, 96.0],
                    [120.0 + 48.0 * r, 120.0],
                    [120.0 + 48.0 * r, 168.0],
                    [120.0 + 24.0 * r, 192.0],
                    [120.0, 216.0],
                    [120.0 - dx, 216.0 + 12.0 * (pent - 1.0)],
                    [120.0 - far, 192.0],
                    [120.0 - dx, 168.0 - 12.0 * (pent - 1.0)],
                ],
                vec![
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
                    [10, 1],
                ],
                vec![1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 1],
                "C1CCC2(CC1)C=CC=C2",
                "C10H14",
            ),
        };
        let request = params(&engine, kind, None, placement);
        let changed = engine.dispatch("ring.create", request)?;
        assert!(changed.changed && changed.file.dirty);
        assert_eq!(changed.undo_depth, 1);
        let current = engine.content().clone();
        let model = serde_json::to_value(engine.document()?)?;
        let expected = literal_graph(&h, &p, &edges, &w)?;
        let path = directory.path(&format!("{name}-環-化学-λ.cdxml"));
        let saved = engine.dispatch("document.save_cdxml", json!({"path":path}))?;
        assert!(!saved.file.dirty);
        assert!(saved.file_operation.is_some());
        let (flags, sides): (Vec<Option<bool>>, Vec<Option<&str>>) = match name {
            "benzene" => (
                vec![Some(false); 6],
                vec![
                    Some("right"),
                    None,
                    Some("right"),
                    None,
                    Some("right"),
                    None,
                ],
            ),
            "cyclopentadiene" => (
                vec![Some(false); 5],
                vec![Some("right"), None, Some("right"), None, None],
            ),
            "toluene" => (
                vec![
                    None,
                    None,
                    Some(false),
                    Some(false),
                    Some(false),
                    Some(false),
                    Some(false),
                ],
                vec![
                    None,
                    Some("right"),
                    None,
                    Some("right"),
                    None,
                    Some("right"),
                    None,
                ],
            ),
            "fused" => (
                vec![
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(false),
                    Some(false),
                    Some(false),
                    Some(false),
                ],
                vec![
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("left"),
                    None,
                    Some("left"),
                    None,
                    Some("left"),
                ],
            ),
            _ => (
                vec![
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(false),
                    Some(false),
                    Some(false),
                    Some(false),
                ],
                vec![
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("right"),
                    None,
                    Some("right"),
                    None,
                ],
            ),
        };
        consume_named_file(&path, &json!([expected]), smiles, formula, &flags, &sides)?;
        let exact_saved = fs::read(&path)?;
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.content(), &original);
        assert!(engine.state().file.dirty);
        if name == "toluene" {
            assert!(
                !engine
                    .stored_document()?
                    .source_info()
                    .ok_or("source")?
                    .modified
            );
            assert_eq!(engine.stored_document()?.export_cdxml()?, super::NATIVE);
            assert_eq!(fs::read(&old_path)?, super::NATIVE);
        }
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.content(), &current);
        assert!(!engine.state().file.dirty);
        assert_eq!(fs::read(&path)?, exact_saved);
        if name == "toluene" {
            assert!(
                engine
                    .stored_document()?
                    .source_info()
                    .ok_or("source")?
                    .modified
            );
            assert!(engine.stored_document()?.export_cdxml().is_err());
        }
        engine.dispatch("document.open_cdxml", json!({"path":path}))?;
        assert!(!engine.state().file.dirty);
        assert_eq!(engine.state().undo_depth, 0);
        assert_eq!(serde_json::to_value(engine.document()?)?, model);
        assert_eq!(engine.stored_document()?.export_cdxml()?, exact_saved);
    }
    Ok(())
}

#[test]
fn selector_union_schema_and_runtime_reject_every_mixed_null_or_unclosed_named_request_atomically()
-> TestResult {
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 38);
    let entry = registry
        .iter()
        .find(|c| c.name == "ring.create")
        .ok_or("ring registry")?;
    assert!(!entry.read_only);
    let schema = serde_json::to_value(&entry.input_schema)?;
    assert_eq!(schema["type"], "object");
    let variants = schema
        .get("oneOf")
        .or_else(|| schema.get("anyOf"))
        .and_then(Value::as_array)
        .ok_or("selector union")?;
    assert_eq!(variants.len(), 2);
    let mut seen = BTreeSet::new();
    for v in variants {
        let v = schema_node(&schema, v)?;
        assert_eq!(v["type"], "object");
        assert_eq!(v["additionalProperties"], false);
        let p = v["properties"].as_object().ok_or("properties")?;
        let required = v["required"]
            .as_array()
            .ok_or("required")?
            .iter()
            .map(|x| x.as_str().ok_or("field"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        let common = [
            "expected_revision",
            "page_id",
            "hydrogen_policy",
            "placement",
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        let mut expected = common.clone();
        let mut allowed = common;
        if p.contains_key("size") {
            assert!(seen.insert("legacy"));
            expected.insert("size");
            allowed.insert("size");
            assert!(!p.contains_key("kind") && !p.contains_key("phase"));
        } else {
            assert!(seen.insert("named"));
            expected.insert("kind");
            allowed.extend(["kind", "phase"]);
            let kind = schema_node(&schema, &p["kind"])?;
            assert_eq!(kind["enum"], json!(["benzene", "cyclopentadiene"]));
            let phase = schema_node(&schema, &p["phase"])?;
            assert_eq!(phase["type"], "string");
            assert_eq!(phase["enum"], json!(["double_first", "single_first"]));
            assert!(!required.contains("phase"));
        }
        assert_eq!(required, expected);
        assert_eq!(
            p.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            allowed
        );
    }
    let mut engine = Engine::default();
    let valid = params(&engine, "benzene", None, free("positive"));
    for (field, value) in [
        ("size", json!(6)),
        ("kind", json!("cyclohexatriene")),
        ("kind", Value::Null),
        ("phase", Value::Null),
        ("phase", json!("resonate")),
        ("phase", json!(0)),
        ("unexpected", json!(true)),
    ] {
        let mut p = valid.clone();
        p[field] = value;
        refuse(&mut engine, p, "invalid_params")?;
    }
    for field in [
        "kind",
        "expected_revision",
        "page_id",
        "hydrogen_policy",
        "placement",
    ] {
        let mut p = valid.clone();
        p.as_object_mut().ok_or("params")?.remove(field);
        refuse(&mut engine, p, "invalid_params")?;
    }
    let mut legacy = valid.clone();
    legacy.as_object_mut().ok_or("params")?.remove("kind");
    legacy["size"] = json!(6);
    legacy["phase"] = json!("double_first");
    refuse(&mut engine, legacy, "invalid_params")?;
    let mut p = valid.clone();
    p["placement"]["unexpected"] = json!(true);
    refuse(&mut engine, p, "invalid_params")?;
    let mut p = valid.clone();
    p["expected_revision"] = json!(9);
    refuse(&mut engine, p, "stale_revision")?;
    let mut p = valid.clone();
    p["page_id"] = json!(9);
    refuse(&mut engine, p, "not_found")?;
    let mut p = valid;
    p["placement"]["end_y_pt"] = json!(168);
    refuse(&mut engine, p, "invalid_params")?;
    Ok(())
}

#[test]
fn named_paths_retain_complete_source_refusals_close_precedence_and_identity_budgets() -> TestResult
{
    let opaque = String::from_utf8(super::NATIVE.to_vec())?
        .replace("<CDXML ", "<CDXML OriginalOpaque=\"preserve\" ");
    let mut engine = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    let p = params(&engine, "benzene", None, free("positive"));
    refuse(&mut engine, p, "unsupported_content")?;
    assert_eq!(engine.stored_document()?.export_cdxml()?, opaque.as_bytes());
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("whole-opaque.smiles");
    let source = b"CC first\r\nO\topaque\xff\0";
    fs::write(&path, source)?;
    let mut raw = Engine::default();
    raw.dispatch("document.open_smiles", json!({"path":path}))?;
    let p = params(&raw, "cyclopentadiene", None, free("positive"));
    refuse(&mut raw, p, "unsupported_content")?;
    assert_eq!(fs::read(&path)?, source);
    for (af, bf, ff) in [(u32::MAX, 41, 17), (29, u32::MAX, 17), (29, 41, u32::MAX)] {
        let mut exhausted = seed(
            ff,
            json!([c(af, 120.0, 168.0, 3, None), c(11, 120.0, 120.0, 3, None)]),
            json!([{"id":bf,"begin":af,"end":11,"order":"single"}]),
        )?;
        let mut place = free("positive");
        place["begin_x_pt"] = json!(300);
        place["end_x_pt"] = json!(300);
        let p = params(&exhausted, "benzene", None, place);
        refuse(&mut exhausted, p, "identity_exhausted")?;
    }
    let mut closed = Engine::default();
    closed.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"discard"}),
    )?;
    assert!(closed.complete_close());
    let mut p = params(&closed, "benzene", None, free("positive"));
    p["expected_revision"] = json!(99);
    refuse(&mut closed, p, "close_committed")?;
    Ok(())
}

// A separate consumer for the NEW unsaturated cases; the old parent's consumer is untouched.
use std::{
    io::Write,
    path::Path,
    process::{Command as Process, Stdio},
};
fn consume_named_file(
    path: &Path,
    expected: &Value,
    smiles: &str,
    formula: &str,
    flags: &[Option<bool>],
    double_positions: &[Option<&str>],
) -> TestResult {
    let raw = fs::read(path)?;
    super::compare_graph_positions(&super::external_graph(&raw)?, expected)?;
    let args = serde_json::to_string(&json!({"graph":expected,"smiles":smiles,"formula":formula,
        "flags":flags,"double_positions":double_positions}))?;
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
        .ok_or("named ring consumer stdin")?
        .write_all(&raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(actual["formula"], formula);
    assert_eq!(actual["literal_integer_orders"], true);
    assert_eq!(actual["literal_direction"], true);
    assert_eq!(actual["literal_flags_and_double_positions"], true);
    assert_eq!(actual["no_assigned_stereo"], true);
    Ok(())
}
