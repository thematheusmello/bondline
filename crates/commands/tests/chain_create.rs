//! Independent chain.create through existing JSON dispatch; no future Rust API.
use bondline_commands::Engine;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn literal_three_carbon_chain_is_one_atomic_known_h_edit() -> TestResult {
    let mut engine = Engine::default();
    let result = engine.dispatch(
        "chain.create",
        json!({
        "expected_revision":0,"page_id":1,"count":3,"angle_degrees":120,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{
            "mode":"free","begin_x_pt":64,"begin_y_pt":112,
            "end_x_pt":112,"end_y_pt":112,"side":"positive"}}),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (1, 1, 0)
    );
    assert!(result.source.is_none() && result.file.path.is_none() && result.file.dirty);
    let value = serde_json::to_value(result)?;
    let fragments = value["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 3);
    for (i, h) in [3, 2, 3].into_iter().enumerate() {
        assert_eq!(atoms[i]["id"], i + 1);
        assert_eq!(atoms[i]["element"], 6);
        assert_eq!(atoms[i]["explicit_hydrogens"], h);
        assert_eq!(atoms[i]["show_terminal_carbon_labels"], false);
        assert_eq!(atoms[i]["show_non_terminal_carbon_labels"], false);
    }
    assert_eq!(
        fragments[0]["bonds"],
        json!([
        {"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"single"}])
    );
    assert!((atoms[2]["position"]["x_pt"].as_f64().ok_or("x")? - 136.0).abs() < 1e-10);
    assert!(
        (atoms[2]["position"]["y_pt"].as_f64().ok_or("y")? - (112.0 + 24.0 * 3.0_f64.sqrt())).abs()
            < 1e-10
    );
    Ok(())
}

// Additive geometry/precision declarations after the immutable first1,804 bytes.
use bondline_commands::{CloseState, Command};
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, Element, Fragment, FragmentId,
    PageId, PageSize, Point2, Radical,
};
use serde_json::Value;
use std::{fs, sync::Arc};

const CHAIN_NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
fn chain_wire(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.state())?)
}
fn chain_free(a: [f64; 2], b: [f64; 2], side: &str) -> Value {
    json!({"mode":"free","begin_x_pt":a[0],"begin_y_pt":a[1],
        "end_x_pt":b[0],"end_y_pt":b[1],"side":side})
}
fn chain_params(engine: &Engine, count: u8, angle: f64, placement: Value) -> Value {
    json!({"expected_revision":engine.state().revision,"page_id":1,"count":count,
        "angle_degrees":angle,"hydrogen_policy":"rectify_neutral_carbon","placement":placement})
}
fn chain_refuse(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = chain_wire(engine)?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let file = engine.ui_file_context_stamp();
    let tool = engine.bond_tool();
    let view = engine.view();
    let generation = engine.session_generation();
    let error = engine
        .dispatch("chain.create", params)
        .err()
        .ok_or("invalid chain accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(chain_wire(engine)?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.view(), view);
    assert_eq!(engine.session_generation(), generation);
    if let Some(model) = model {
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    Ok(())
}
fn chain_geometry(
    count: u8,
    angle: f64,
    side: &str,
    a: [f64; 2],
    b: [f64; 2],
) -> TestResult<Engine> {
    let mut engine = Engine::default();
    let params = chain_params(&engine, count, angle, chain_free(a, b, side));
    let before = engine.view();
    let result = engine.dispatch("chain.create", params)?;
    assert_eq!(
        (
            result.changed,
            result.revision,
            result.undo_depth,
            result.redo_depth
        ),
        (true, 1, 1, 0)
    );
    assert_eq!(
        engine.view(),
        before,
        "headless creation does not rescale or reframe the paper"
    );
    let fragments = engine.document()?.pages()[0].fragments();
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0].id().get(), 1);
    let atoms = fragments[0].atoms();
    let bonds = fragments[0].bonds();
    assert_eq!(atoms.len(), usize::from(count));
    assert_eq!(bonds.len(), usize::from(count) - 1);
    let points = atoms
        .iter()
        .map(|atom| {
            let p = atom.position.ok_or("position")?;
            Ok([p.x_pt(), p.y_pt()])
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(points[0], a);
    assert_eq!(points[1], b);
    let length = (b[0] - a[0]).hypot(b[1] - a[1]);
    for (i, atom) in atoms.iter().enumerate() {
        assert_eq!(atom.id.get(), u32::try_from(i + 1)?);
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.charge, 0);
        assert_eq!(atom.isotope, None);
        assert_eq!(atom.radical, Radical::None);
        let value = serde_json::to_value(atom)?;
        assert_eq!(value["show_terminal_carbon_labels"], false);
        assert_eq!(value["show_non_terminal_carbon_labels"], false);
        assert_eq!(
            atom.explicit_hydrogens,
            Some(if i == 0 || i + 1 == atoms.len() { 3 } else { 2 })
        );
    }
    assert_eq!(
        atoms
            .iter()
            .map(|a| u64::from(a.explicit_hydrogens.unwrap_or(0)))
            .sum::<u64>(),
        2 * u64::from(count) + 2,
        "literal alkane H, not consumer inference"
    );
    for (i, bond) in bonds.iter().enumerate() {
        assert_eq!(
            (bond.id.get(), bond.begin.get(), bond.end.get()),
            (
                u32::try_from(i + 1)?,
                u32::try_from(i + 1)?,
                u32::try_from(i + 2)?
            )
        );
        assert_eq!(bond.order, BondOrder::Single);
        assert_eq!(bond.double_position, DoublePosition::Unspecified);
    }
    for pair in points.windows(2) {
        let norm = (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]) / length;
        assert!(
            (norm - 1.0).abs() <= 1e-10,
            "actual relative length{norm}, n{count}, angle{angle}"
        );
    }
    for (i, triple) in points.windows(3).enumerate() {
        let u = [
            (triple[0][0] - triple[1][0]) / length,
            (triple[0][1] - triple[1][1]) / length,
        ];
        let v = [
            (triple[2][0] - triple[1][0]) / length,
            (triple[2][1] - triple[1][1]) / length,
        ];
        let cross = u[0] * v[1] - u[1] * v[0];
        let dot = u[0] * v[0] + u[1] * v[1];
        assert!((cross.abs().atan2(dot) - angle.to_radians()).abs() <= 1e-10);
        let turn = if (side == "positive") == (i % 2 == 0) {
            1.0
        } else {
            -1.0
        };
        assert!(
            -cross * turn > 0.0,
            "actual handedness must alternate from the first signed turn"
        );
    }
    Ok(engine)
}

#[test]
fn every_count_and_both_sides_keep_literal_geometry_with_custom_interior_angles() -> TestResult {
    for count in 2..=64 {
        for side in ["positive", "negative"] {
            chain_geometry(count, 120.0, side, [96.0, 96.0], [144.0, 96.0])?;
        }
    }
    for angle in [1.0, 45.0, 90.0, 137.5, 179.0] {
        for side in ["positive", "negative"] {
            for b in [
                [80.0, 160.0],
                [128.0, 112.0],
                [128.0 + 24.0 * 3.0_f64.sqrt(), 184.0],
            ] {
                chain_geometry(4, angle, side, [128.0, 160.0], b)?;
            }
            chain_geometry(64, angle, side, [96.0, 96.0], [144.0, 96.0])?;
        }
    }
    // Explicit points are bounded model coordinates, not a page-containment policy.
    let outside = chain_geometry(3, 90.0, "negative", [-1000.0, -1000.0], [-952.0, -1000.0])?;
    assert!(
        outside.document()?.pages()[0].fragments()[0].atoms()[0]
            .position
            .ok_or("position")?
            .x_pt()
            < 0.0
    );
    Ok(())
}

#[test]
fn tiny_well_conditioned_and_ulp_grid_geometry_are_distinct_from_rounded_distortion() -> TestResult
{
    for side in ["positive", "negative"] {
        chain_geometry(64, 120.0, side, [0.0, 0.0], [1e-200, 0.0])?;
    }
    let epsilon = f64::EPSILON;
    let grid = chain_geometry(3, 90.0, "positive", [1.0, 1.0], [1.0 + epsilon, 1.0])?;
    assert_eq!(
        grid.document()?.pages()[0].fragments()[0].atoms()[2].position,
        Some(Point2::new(1.0 + epsilon, 1.0 + epsilon)?)
    );
    for (a, b) in [
        ([1.0, 1.0], [1.0 + epsilon, 1.0]),
        ([0.0, 0.0], [f64::from_bits(1), 0.0]),
    ] {
        let mut engine = Engine::from_document(StoredDocument::from_cdxml(
            CHAIN_NATIVE.into(),
            PageSize::A4,
        )?);
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":300,"begin_y_pt":400,
            "end_x_pt":348,"end_y_pt":400}),
        )?;
        let redo = engine.document()?.clone();
        engine.dispatch("history.undo", json!({}))?;
        engine.dispatch(
            "ui.selection",
            json!({"action":"set","expected_revision":engine.state().revision,
            "page_id":1,"atom_ids":[1,2]}),
        )?;
        let selection = engine.selection_context()?;
        let params = chain_params(&engine, 3, 120.0, chain_free(a, b, "positive"));
        chain_refuse(&mut engine, params, "invalid_params")?;
        assert!(selection.is_current(&engine));
        assert_eq!(engine.stored_document()?.export_cdxml()?, CHAIN_NATIVE);
        assert_eq!(
            (engine.state().undo_depth, engine.state().redo_depth),
            (0, 1)
        );
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.document()?, &redo);
        engine.dispatch("history.undo", json!({}))?;
        assert!(!engine.state().file.dirty);
        let params = chain_params(
            &engine,
            2,
            120.0,
            chain_free([400.0, 500.0], [448.0, 500.0], "positive"),
        );
        engine.dispatch("chain.create", params)?;
        let fragment = engine.document()?.pages()[0]
            .fragments()
            .last()
            .ok_or("new fragment")?;
        assert_eq!(
            fragment.id().get(),
            3,
            "failed candidates spend no fragment ID"
        );
        assert_eq!(
            fragment
                .atoms()
                .iter()
                .map(|a| a.id.get())
                .collect::<Vec<_>>(),
            vec![5, 6]
        );
        assert_eq!(fragment.bonds()[0].id.get(), 3);
    }
    Ok(())
}

// Compact final additive families; actual file chemistry is independently wire-owned.
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the unchanged isolated test directory")]
mod chain_file_support;
fn chain_atom(id: u32, z: u8, p: [f64; 2], h: u16) -> TestResult<Atom> {
    let mut a = Atom::new(
        AtomId::new(id).ok_or("atom")?,
        Element::try_from(z)?,
        Some(Point2::new(p[0], p[1])?),
    );
    a.explicit_hydrogens = Some(h);
    Ok(a)
}
fn chain_bond(id: u32, a: u32, b: u32, order: BondOrder, side: DoublePosition) -> TestResult<Bond> {
    Ok(Bond {
        id: BondId::new(id).ok_or("bond")?,
        begin: AtomId::new(a).ok_or("begin")?,
        end: AtomId::new(b).ok_or("end")?,
        order,
        double_position: side,
    })
}
fn chain_model(atoms: Vec<Atom>, bonds: Vec<Bond>, unrelated: bool) -> TestResult<Engine> {
    let mut stored = StoredDocument::default();
    let page = PageId::new(1).ok_or("page")?;
    stored
        .model_mut()
        .set_page_size(page, PageSize::new(256.0, 256.0)?)?;
    stored.model_mut().append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    if unrelated {
        let mut other = chain_atom(900, 8, [224.0, 224.0], 0)?;
        other.charge = -1;
        other.isotope = std::num::NonZeroU16::new(18);
        other.radical = Radical::Doublet;
        stored.model_mut().append_fragments(
            page,
            vec![Fragment::new(
                FragmentId::new(99).ok_or("other")?,
                vec![other],
                vec![],
            )?],
        )?;
    }
    Ok(Engine::from_document(stored))
}
fn chain_anchor(fragment: u32, atom: u32, b: [f64; 2]) -> Value {
    json!({"mode":"atom","fragment_id":fragment,"atom_id":atom,"end_x_pt":b[0],"end_y_pt":b[1],"side":"positive"})
}

#[test]
fn anchors_are_target_local_preserving_old_arrays_styles_and_unrelated_owners() -> TestResult {
    for (order, h) in [
        (BondOrder::Single, 3),
        (BondOrder::Double, 2),
        (BondOrder::Triple, 1),
    ] {
        let mut target = chain_atom(29, 6, [128.0, 128.0], h)?;
        target.show_terminal_carbon_labels = Some(true);
        target.show_non_terminal_carbon_labels = Some(false);
        let old_bond = chain_bond(73, 5, 29, order, DoublePosition::Right)?;
        let mut engine = chain_model(
            vec![chain_atom(5, 6, [64.0, 128.0], h)?, target],
            vec![old_bond],
            true,
        )?;
        let old = engine.document()?.pages()[0].fragments()[0].clone();
        let other = engine.document()?.pages()[0].fragments()[1].clone();
        let mut wanted = old.atoms().to_vec();
        wanted[1].explicit_hydrogens = Some(h - 1);
        let params = chain_params(&engine, 3, 120.0, chain_anchor(17, 29, [192.0, 128.0]));
        let result = engine.dispatch("chain.create", params)?;
        assert_eq!(
            (result.changed, result.revision, result.undo_depth),
            (true, 1, 1)
        );
        let f = &engine.document()?.pages()[0].fragments()[0];
        assert_eq!(f.id(), old.id());
        assert_eq!(&f.atoms()[..2], wanted.as_slice());
        assert_eq!(&f.bonds()[..1], old.bonds());
        assert_eq!(
            f.atoms().iter().map(|a| a.id.get()).collect::<Vec<_>>(),
            vec![5, 29, 901, 902]
        );
        assert_eq!(
            f.atoms()
                .iter()
                .map(|a| a.explicit_hydrogens)
                .collect::<Vec<_>>(),
            vec![Some(h), Some(h - 1), Some(2), Some(3)]
        );
        assert_eq!(
            serde_json::to_value(&f.bonds()[1..])?,
            json!([
            {"id":74,"begin":29,"end":901,"order":"single"},
            {"id":75,"begin":901,"end":902,"order":"single"}])
        );
        for a in &f.atoms()[2..] {
            let v = serde_json::to_value(a)?;
            assert_eq!(v["show_terminal_carbon_labels"], false);
            assert_eq!(v["show_non_terminal_carbon_labels"], false);
        }
        assert!(Arc::ptr_eq(
            &other,
            &engine.document()?.pages()[0].fragments()[1]
        ));
    }
    // Nonterminal propane center: one added carbon gives literal isobutane C4H10.
    let mut engine = chain_model(
        vec![
            chain_atom(29, 6, [64.0, 128.0], 3)?,
            chain_atom(5, 6, [128.0, 128.0], 2)?,
            chain_atom(107, 6, [160.0, 128.0 + 32.0 * 3.0_f64.sqrt()], 3)?,
        ],
        vec![
            chain_bond(73, 5, 29, BondOrder::Single, DoublePosition::Unspecified)?,
            chain_bond(9, 107, 5, BondOrder::Single, DoublePosition::Unspecified)?,
        ],
        false,
    )?;
    let params = chain_params(&engine, 2, 120.0, chain_anchor(17, 5, [128.0, 64.0]));
    engine.dispatch("chain.create", params)?;
    let f = &engine.document()?.pages()[0].fragments()[0];
    assert_eq!(
        f.atoms()
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        vec![Some(3), Some(1), Some(3), Some(3)]
    );
    assert_eq!(f.bonds().last().ok_or("new bond")?.begin.get(), 5);
    assert_eq!(f.bonds().last().ok_or("new bond")?.end.get(), 108);
    Ok(())
}

#[test]
fn closed_requests_target_guards_and_occupied_centers_refuse_atomically() -> TestResult {
    let mut engine = Engine::default();
    let valid = chain_params(
        &engine,
        3,
        120.0,
        chain_free([64.0, 112.0], [112.0, 112.0], "positive"),
    );
    let info = bondline_commands::registry()
        .into_iter()
        .find(|r| r.name == "chain.create")
        .ok_or("registry")?;
    assert!(!info.read_only);
    let schema = serde_json::to_value(info.input_schema)?;
    assert_eq!(schema["additionalProperties"], false);
    let mut required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or("field"))
        .collect::<Result<Vec<_>, _>>()?;
    required.sort();
    assert_eq!(
        required,
        vec![
            "angle_degrees",
            "count",
            "expected_revision",
            "hydrogen_policy",
            "page_id",
            "placement"
        ]
    );
    assert_eq!(
        schema["properties"].as_object().ok_or("properties")?.len(),
        6
    );
    for field in [
        "expected_revision",
        "page_id",
        "count",
        "angle_degrees",
        "hydrogen_policy",
        "placement",
    ] {
        let mut p = valid.clone();
        p.as_object_mut().ok_or("params")?.remove(field);
        chain_refuse(&mut engine, p, "invalid_params")?;
    }
    for field in ["count", "angle_degrees"] {
        for value in [json!(0), json!(180), Value::Null, json!("3")] {
            let mut p = valid.clone();
            p[field] = value;
            chain_refuse(&mut engine, p, "invalid_params")?;
        }
    }
    for count in [1, 65, 256] {
        let mut p = valid.clone();
        p["count"] = json!(count);
        chain_refuse(&mut engine, p, "invalid_params")?;
    }
    for value in [
        Value::Null,
        json!({"positive":null}),
        json!(["positive"]),
        json!("left"),
    ] {
        let mut p = valid.clone();
        p["placement"]["side"] = value;
        chain_refuse(&mut engine, p, "invalid_params")?;
    }
    let mut p = valid.clone();
    p["hydrogen_policy"] = json!({"rectify_neutral_carbon":null});
    chain_refuse(&mut engine, p, "invalid_params")?;
    let mut p = valid.clone();
    p["placement"]["fragment_id"] = json!(17);
    chain_refuse(&mut engine, p, "invalid_params")?;
    let mut p = valid.clone();
    p["unused"] = json!(true);
    chain_refuse(&mut engine, p, "invalid_params")?;
    chain_refuse(&mut engine, Value::Array(vec![]), "invalid_params")?;
    let mut p = valid.clone();
    p["placement"]["end_x_pt"] = json!(64);
    chain_refuse(&mut engine, p, "invalid_params")?;
    let mut p = valid.clone();
    p["placement"]["begin_x_pt"] = json!(100001);
    chain_refuse(&mut engine, p, "invalid_params")?;
    let mut p = valid.clone();
    p["page_id"] = json!(0);
    chain_refuse(&mut engine, p, "invalid_params")?;
    let mut p = valid.clone();
    p["page_id"] = json!(2);
    chain_refuse(&mut engine, p, "not_found")?;
    let mut p = valid.clone();
    p["expected_revision"] = json!(1);
    p["count"] = json!(1);
    chain_refuse(&mut engine, p, "stale_revision")?;
    for change in [
        "unknown_h",
        "bad_h",
        "charge",
        "isotope",
        "radical",
        "position",
        "oxygen",
        "aromatic",
    ] {
        let mut target = chain_atom(29, 6, [128.0, 128.0], 3)?;
        let mut order = BondOrder::Single;
        match change {
            "unknown_h" => target.explicit_hydrogens = None,
            "bad_h" => target.explicit_hydrogens = Some(2),
            "charge" => target.charge = 1,
            "isotope" => target.isotope = std::num::NonZeroU16::new(13),
            "radical" => target.radical = Radical::Doublet,
            "position" => target.position = None,
            "oxygen" => target.element = Element::try_from(8)?,
            "aromatic" => order = BondOrder::Aromatic,
            _ => return Err("test fixture tag".into()),
        }
        let mut e = chain_model(
            vec![chain_atom(5, 6, [64.0, 128.0], 3)?, target],
            vec![chain_bond(73, 5, 29, order, DoublePosition::Unspecified)?],
            false,
        )?;
        let params = chain_params(&e, 2, 120.0, chain_anchor(17, 29, [192.0, 128.0]));
        chain_refuse(&mut e, params, "unsupported_content")?;
    }
    let mut e = chain_model(
        vec![
            chain_atom(5, 6, [64.0, 128.0], 3)?,
            chain_atom(29, 6, [128.0, 128.0], 3)?,
        ],
        vec![chain_bond(
            73,
            5,
            29,
            BondOrder::Single,
            DoublePosition::Unspecified,
        )?],
        false,
    )?;
    for (fragment, atom, code) in [
        (0, 29, "invalid_params"),
        (17, 0, "invalid_params"),
        (18, 29, "not_found"),
        (17, 99, "not_found"),
    ] {
        let params = chain_params(&e, 3, 120.0, chain_anchor(fragment, atom, [192.0, 128.0]));
        chain_refuse(&mut e, params, code)?;
    }
    // All new centers are checked, including the first free center and a later one.
    for placement in [
        chain_free([64.0, 128.0], [112.0, 128.0], "positive"),
        chain_anchor(17, 29, [64.0, 128.0]),
        chain_free([64.0, 80.0], [64.0, 104.0], "positive"),
    ] {
        let angle = if placement["mode"] == "free" && placement["begin_y_pt"] == json!(80.0) {
            90.0
        } else {
            120.0
        };
        let params = chain_params(&e, 4, angle, placement);
        // In the last fixture P3=(40,128), not occupied; use explicit90-degree
        // start above the existing point so P2=(64,128) is occupied instead.
        if params["placement"]["begin_y_pt"] == json!(80.0) {
            continue;
        }
        chain_refuse(&mut e, params, "invalid_params")?;
    }
    let params = chain_params(
        &e,
        3,
        90.0,
        chain_free([40.0, 104.0], [64.0, 104.0], "positive"),
    );
    chain_refuse(&mut e, params, "invalid_params")?;
    Ok(())
}

#[test]
fn native_backing_history_and_foreign_complete_sources_keep_their_authority() -> TestResult {
    let original = StoredDocument::from_cdxml(CHAIN_NATIVE.into(), PageSize::A4)?;
    assert!(original.is_fully_mapped());
    let original_model = original.model_snapshot();
    let mut engine = Engine::from_document(original.clone());
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[1,2]}),
    )?;
    let params = chain_params(&engine, 3, 120.0, chain_anchor(1, 1, [100.0, 152.0]));
    engine.dispatch("chain.create", params)?;
    assert!(engine.selection().atom_ids.is_empty());
    let changed: Document = engine.document()?.clone();
    let changed_owner = engine.stored_document()?.model_snapshot();
    assert_eq!(original.export_cdxml()?, CHAIN_NATIVE);
    assert_eq!(
        changed.pages()[0].fragments()[0].atoms()[0].explicit_hydrogens,
        Some(2)
    );
    assert_eq!(
        changed.pages()[0].fragments()[0].atoms()[1],
        original.model().pages()[0].fragments()[0].atoms()[1]
    );
    assert_eq!(
        &changed.pages()[0].fragments()[0].bonds()[..1],
        original.model().pages()[0].fragments()[0].bonds()
    );
    assert_ne!(engine.stored_document()?.to_cdxml()?.as_ref(), CHAIN_NATIVE);
    engine.dispatch("history.undo", json!({}))?;
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.stored_document()?.export_cdxml()?, CHAIN_NATIVE);
    assert!(Arc::ptr_eq(
        &original_model,
        &engine.stored_document()?.model_snapshot()
    ));
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.document()?, &changed);
    assert!(Arc::ptr_eq(
        &changed_owner,
        &engine.stored_document()?.model_snapshot()
    ));
    let directory = chain_file_support::FileTestDirectory::new()?;
    let opaque = String::from_utf8(CHAIN_NATIVE.to_vec())?
        .replace("<CDXML ", "<CDXML ChainOwner=\"preserved\" ");
    let mut e = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    let mut p = chain_params(
        &e,
        0,
        0.0,
        chain_free([64.0, 112.0], [112.0, 112.0], "positive"),
    );
    chain_refuse(&mut e, p.clone(), "unsupported_content")?;
    p["expected_revision"] = json!(1);
    chain_refuse(&mut e, p, "stale_revision")?;
    assert_eq!(e.stored_document()?.export_cdxml()?, opaque.as_bytes());
    for (name, bytes, command) in [
        (
            "source.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "source.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
        ),
        (
            "source.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        (
            "source.smiles",
            b"CC first\r\nO\topaque\xff\0".as_slice(),
            "document.open_smiles",
        ),
    ] {
        let path = directory.path(name);
        fs::write(&path, bytes)?;
        let mut e = Engine::default();
        e.dispatch(command, json!({"path":path}))?;
        let p = chain_params(
            &e,
            3,
            120.0,
            chain_free([300.0, 300.0], [348.0, 300.0], "positive"),
        );
        chain_refuse(&mut e, p, "unsupported_content")?;
        assert_eq!(fs::read(&path)?, bytes);
        if command == "document.open_smiles" {
            e.dispatch("document.import_smiles_records",json!({"expected_revision":e.state().revision,
                "selection":{"mode":"record","record_id":1},"bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,
                "page_width_pt":595.275590551,"page_height_pt":841.88976378}))?;
            let p = chain_params(&e, 3, 120.0, chain_anchor(1, 1, [200.0, 200.0]));
            chain_refuse(&mut e, p, "unsupported_content")?;
            assert_eq!(e.stored_document()?.smiles_file()?.as_bytes(), bytes);
        }
    }
    Ok(())
}

#[test]
fn immediate_commit_invalidates_pending_contexts_late_cancel_is_truthful_and_c2_stays_labeled()
-> TestResult {
    let mut engine = chain_model(
        vec![
            chain_atom(1, 6, [64.0, 112.0], 3)?,
            chain_atom(2, 6, [112.0, 112.0], 3)?,
        ],
        vec![chain_bond(
            1,
            1,
            2,
            BondOrder::Single,
            DoublePosition::Unspecified,
        )?],
        false,
    )?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert_eq!(
        engine.state().close,
        CloseState::Approved,
        "initial model is saved/clean"
    );
    let params = chain_params(
        &engine,
        2,
        120.0,
        chain_free([64.0, 176.0], [112.0, 176.0], "positive"),
    );
    let prepared = engine.prepare(Command::parse("chain.create", params)?)?;
    assert_eq!(
        engine.state().revision,
        1,
        "ordinary immediate prepare has already committed"
    );
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    assert!(
        !engine.complete_close(),
        "the old approval cannot close the new revision"
    );
    let owner = engine.stored_document()?.model_snapshot();
    let cancel = bondline_commands::Cancellation::default();
    cancel.cancel();
    let result = engine.finish(prepared.complete_with_cancel(&cancel)?)?;
    assert!(result.changed);
    assert_eq!(result.revision, 1);
    assert!(Arc::ptr_eq(
        &owner,
        &engine.stored_document()?.model_snapshot()
    ));
    engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"cancel"}),
    )?;
    engine.set_ui_file_host(true);
    let request = engine.dispatch("ui.file", json!({"expected_revision":1,"action":"save"}))?;
    assert!(request.ui_file_operation.is_some());
    let context = engine.ui_file_context_stamp();
    let params = chain_params(
        &engine,
        2,
        120.0,
        chain_free([160.0, 176.0], [208.0, 176.0], "negative"),
    );
    assert!(engine.dispatch("chain.create", params)?.changed);
    assert!(
        !engine.ui_file_context_stamp_matches(&context),
        "valid headless edit invalidates a pending file context"
    );
    let mut stored = StoredDocument::default();
    stored
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(256.0, 256.0)?)?;
    let mut c2 = Engine::from_document(stored);
    let params = chain_params(
        &c2,
        2,
        1.0,
        chain_free([64.0, 112.0], [112.0, 112.0], "positive"),
    );
    c2.dispatch("chain.create", params)?;
    let before = c2.content().clone();
    let graph = bondline_render::graph_page_scene(576, 576, &c2.document()?.pages()[0])?;
    assert!(
        graph.label_em_px().is_some(),
        "isolated C2 has mandatory CH3 labels despite both new false flags"
    );
    for point in [[160.0, 256.0], [256.0, 256.0]] {
        assert!(graph.scene().primitives.iter().any(|p|matches!(p,bondline_render::Primitive::Rectangle(r)
            if r.color==[0,0,0,255] && (r.x as f64)<point[0]+16.0 && ((r.x+r.width) as f64)>point[0]-16.0
            && (r.y as f64)<point[1]+16.0 && ((r.y+r.height) as f64)>point[1]-16.0)));
    }
    assert_eq!(c2.content(), &before);
    let mut closed = Engine::default();
    closed.dispatch("document.close", json!({"expected_revision":0}))?;
    assert!(closed.complete_close());
    let mut params = chain_params(
        &closed,
        0,
        0.0,
        chain_free([0.0, 0.0], [0.0, 0.0], "positive"),
    );
    params["expected_revision"] = json!(99);
    chain_refuse(&mut closed, params, "close_committed")?;
    Ok(())
}
