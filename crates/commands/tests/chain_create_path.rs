//! Independent literal path through the existing JSON API; no future Rust names.
use bondline_commands::Engine;
use serde_json::json;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn five_literal_mixed_turn_vertices_create_exact_c5h12_in_one_edit() -> TestResult {
    let mut engine = Engine::default();
    let result = engine.dispatch(
        "chain.create_path",
        json!({
        "expected_revision":0,"page_id":1,"hydrogen_policy":"rectify_neutral_carbon",
        "placement":{"mode":"free","points":[{"x_pt":144,"y_pt":144},
            {"x_pt":192,"y_pt":144},{"x_pt":192,"y_pt":192},
            {"x_pt":240,"y_pt":192},{"x_pt":240,"y_pt":144}]}}),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (1, 1, 0)
    );
    assert!(result.source.is_none() && result.file.path.is_none() && result.file.dirty);
    assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
    let state = serde_json::to_value(result)?;
    let fragments = state["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0]["id"], 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 5);
    for (i, (p, h)) in [
        [144.0, 144.0],
        [192.0, 144.0],
        [192.0, 192.0],
        [240.0, 192.0],
        [240.0, 144.0],
    ]
    .into_iter()
    .zip([3, 2, 2, 2, 3])
    .enumerate()
    {
        assert_eq!(
            atoms[i],
            json!({"id":i+1,"element":6,"position":{"x_pt":p[0],"y_pt":p[1]},
            "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h,
            "show_terminal_carbon_labels":false,"show_non_terminal_carbon_labels":false})
        );
    }
    assert_eq!(
        fragments[0]["bonds"],
        json!([
        {"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"single"},
        {"id":3,"begin":3,"end":4,"order":"single"},
        {"id":4,"begin":4,"end":5,"order":"single"}])
    );
    assert!(state.get("conversion").is_none());
    Ok(())
}

// Additive independent declarations; the original1,994-byte JSON case is exact.
use bondline_commands::{CloseState, Command};
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, DoublePosition, Element, Fragment, FragmentId, PageId,
    PageSize, Point2, Radical,
};
use serde_json::Value;
use std::{fs, sync::Arc};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated test directory")]
mod support;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

fn points(values: &[[f64; 2]]) -> Value {
    json!(
        values
            .iter()
            .map(|p| json!({"x_pt":p[0],"y_pt":p[1]}))
            .collect::<Vec<_>>()
    )
}
fn free(engine: &Engine, p: &[[f64; 2]]) -> Value {
    json!({"expected_revision":engine.state().revision,"page_id":1,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free","points":points(p)}})
}
fn attached(engine: &Engine, fragment: u32, atom: u32, p: &[[f64; 2]]) -> Value {
    json!({"expected_revision":engine.state().revision,"page_id":1,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"atom",
            "fragment_id":fragment,"atom_id":atom,"new_points":points(p)}})
}
fn state(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.state())?)
}
fn refuse(engine: &mut Engine, p: Value, code: &str) -> TestResult {
    let before = state(engine)?;
    let whole = engine.content().clone();
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
        .dispatch("chain.create_path", p)
        .err()
        .ok_or("invalid path accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(state(engine)?, before);
    assert_eq!(engine.content(), &whole);
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.view(), view);
    assert_eq!(engine.session_generation(), generation);
    if let Some(old) = model {
        assert!(Arc::ptr_eq(
            &old,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    Ok(())
}
fn atom(id: u32, p: [f64; 2], h: u16) -> TestResult<Atom> {
    let mut a = Atom::new(
        AtomId::new(id).ok_or("atom")?,
        Element::try_from(6)?,
        Some(Point2::new(p[0], p[1])?),
    );
    a.explicit_hydrogens = Some(h);
    Ok(a)
}
fn bond(id: u32, begin: u32, end: u32, order: BondOrder, side: DoublePosition) -> TestResult<Bond> {
    Ok(Bond {
        id: BondId::new(id).ok_or("bond")?,
        begin: AtomId::new(begin).ok_or("begin")?,
        end: AtomId::new(end).ok_or("end")?,
        order,
        double_position: side,
    })
}
fn model(fragment: u32, atoms: Vec<Atom>, bonds: Vec<Bond>, other: bool) -> TestResult<Engine> {
    let mut stored = StoredDocument::default();
    let page = PageId::new(1).ok_or("page")?;
    stored.model_mut().append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(fragment).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    if other {
        let mut a = atom(900, [500.0, 500.0], 19)?;
        a.element = Element::try_from(118)?;
        a.isotope = std::num::NonZeroU16::new(300);
        a.charge = -2;
        a.radical = Radical::Triplet;
        stored.model_mut().append_fragments(
            page,
            vec![Fragment::new(
                FragmentId::new(7).ok_or("other")?,
                vec![a],
                vec![],
            )?],
        )?;
    }
    Ok(Engine::from_document(stored))
}
fn seed() -> TestResult<Engine> {
    model(
        17,
        vec![atom(29, [128.0, 128.0], 3)?, atom(11, [64.0, 128.0], 3)?],
        vec![bond(
            41,
            11,
            29,
            BondOrder::Single,
            DoublePosition::Unspecified,
        )?],
        false,
    )
}

#[test]
fn unequal_collinear_crossing_tiny_ulp_and_outside_paper_paths_keep_each_literal_coordinate()
-> TestResult {
    let eps = f64::EPSILON;
    let mut paths = vec![
        vec![
            [-1000.0, -1000.0],
            [-999.0, -1000.0],
            [-997.0, -1000.0],
            [-997.0, -995.0],
        ],
        vec![
            [144.0, 144.0],
            [240.0, 240.0],
            [144.0, 240.0],
            [240.0, 144.0],
        ],
        vec![
            [0.0, 0.0],
            [1e-200, 0.0],
            [1e-200, 2e-200],
            [4e-200, 2e-200],
        ],
        vec![
            [1.0, 1.0],
            [1.0 + eps, 1.0],
            [1.0 + eps, 1.0 + eps],
            [1.0 + 2.0 * eps, 1.0 + eps],
        ],
        vec![
            [-100000.0, -100000.0],
            [100000.0, -100000.0],
            [100000.0, 100000.0],
        ],
        vec![[0.0, 0.0], [f64::from_bits(1), 0.0]],
    ];
    paths.push((0..64).map(|i| [f64::from(i), f64::from(i % 3)]).collect());
    for p in paths {
        let mut engine = Engine::default();
        let view = engine.view();
        let page = engine.document()?.pages()[0].size();
        let request = free(&engine, &p);
        let result = engine.dispatch("chain.create_path", request)?;
        assert_eq!(
            (
                result.changed,
                result.revision,
                result.undo_depth,
                result.redo_depth
            ),
            (true, 1, 1, 0)
        );
        assert_eq!(engine.view(), view);
        assert_eq!(engine.document()?.pages()[0].size(), page);
        let fragments = engine.document()?.pages()[0].fragments();
        assert_eq!(fragments.len(), 1);
        let f = &fragments[0];
        let total = p.len();
        assert_eq!(f.atoms().len(), total);
        assert_eq!(f.bonds().len(), total - 1);
        for (i, (a, p)) in f.atoms().iter().zip(&p).enumerate() {
            let at = a.position.ok_or("position")?;
            assert_eq!(at.x_pt().to_bits(), p[0].to_bits());
            assert_eq!(at.y_pt().to_bits(), p[1].to_bits());
            assert_eq!(a.id.get(), u32::try_from(i + 1)?);
            assert_eq!(a.element.get(), 6);
            assert_eq!(
                a.explicit_hydrogens,
                Some(if i == 0 || i + 1 == total { 3 } else { 2 })
            );
            assert_eq!(a.isotope, None);
            assert_eq!(a.charge, 0);
            assert_eq!(a.radical, Radical::None);
            let v = serde_json::to_value(a)?;
            assert_eq!(v["show_terminal_carbon_labels"], false);
            assert_eq!(v["show_non_terminal_carbon_labels"], false);
        }
        for (i, b) in f.bonds().iter().enumerate() {
            assert_eq!(
                (b.id.get(), b.begin.get(), b.end.get()),
                (
                    u32::try_from(i + 1)?,
                    u32::try_from(i + 1)?,
                    u32::try_from(i + 2)?
                )
            );
            assert_eq!(b.order, BondOrder::Single);
            assert_eq!(b.double_position, DoublePosition::Unspecified);
        }
    }
    Ok(())
}

#[test]
fn sparse_weighted_attachments_preserve_every_old_field_owner_history_and_identity_floor()
-> TestResult {
    for (order, h) in [
        (BondOrder::Single, 3),
        (BondOrder::Double, 2),
        (BondOrder::Triple, 1),
    ] {
        let mut target = atom(29, [128.0, 128.0], h)?;
        target.show_terminal_carbon_labels = Some(true);
        target.show_non_terminal_carbon_labels = Some(false);
        let side = if order == BondOrder::Double {
            DoublePosition::Left
        } else {
            DoublePosition::Unspecified
        };
        let mut engine = model(
            17,
            vec![target, atom(11, [64.0, 128.0], h)?],
            vec![bond(41, 11, 29, order, side)?],
            true,
        )?;
        let whole = engine.content().clone();
        let old = Arc::clone(&engine.document()?.pages()[0].fragments()[0]);
        let other = Arc::clone(&engine.document()?.pages()[0].fragments()[1]);
        engine.dispatch(
            "ui.selection",
            json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[29,900]}),
        )?;
        let request = attached(
            &engine,
            17,
            29,
            &[[192.0, 128.0], [192.0, 176.0], [216.0, 176.0]],
        );
        let result = engine.dispatch("chain.create_path", request)?;
        assert_eq!(
            (result.changed, result.revision, result.undo_depth),
            (true, 1, 1)
        );
        let f = &engine.document()?.pages()[0].fragments()[0];
        let mut wanted = old.atoms().to_vec();
        wanted[0].explicit_hydrogens = Some(h - 1);
        assert_eq!(&f.atoms()[..2], wanted.as_slice());
        assert_eq!(&f.bonds()[..1], old.bonds());
        assert_eq!(f.id(), old.id());
        assert_eq!(
            f.atoms().iter().map(|a| a.id.get()).collect::<Vec<_>>(),
            [29, 11, 901, 902, 903]
        );
        assert_eq!(
            f.atoms()
                .iter()
                .map(|a| a.explicit_hydrogens)
                .collect::<Vec<_>>(),
            [Some(h - 1), Some(h), Some(2), Some(2), Some(3)]
        );
        assert_eq!(
            serde_json::to_value(&f.bonds()[1..])?,
            json!([
            {"id":42,"begin":29,"end":901,"order":"single"},
            {"id":43,"begin":901,"end":902,"order":"single"},
            {"id":44,"begin":902,"end":903,"order":"single"}])
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
        assert!(engine.selection().atom_ids.is_empty());
        let changed = engine.content().clone();
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.content(), &whole);
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.content(), &changed);
        engine.dispatch("history.undo", json!({}))?;
        let invalid = free(&engine, &[[400.0, 400.0]]);
        refuse(&mut engine, invalid, "invalid_params")?;
        let request = free(&engine, &[[400.0, 400.0], [448.0, 400.0]]);
        engine.dispatch("chain.create_path", request)?;
        let f = &engine.document()?.pages()[0].fragments()[2];
        assert_eq!(f.id().get(), 18);
        assert_eq!(
            f.atoms().iter().map(|a| a.id.get()).collect::<Vec<_>>(),
            [904, 905]
        );
        assert_eq!(f.bonds()[0].id.get(), 45);
        assert_eq!(engine.state().redo_depth, 0);
    }
    // Nonterminal propane: one new point spends one of the center's two H.
    let mut engine = model(
        17,
        vec![
            atom(29, [64.0, 128.0], 3)?,
            atom(5, [128.0, 128.0], 2)?,
            atom(107, [160.0, 176.0], 3)?,
        ],
        vec![
            bond(73, 5, 29, BondOrder::Single, DoublePosition::Unspecified)?,
            bond(9, 107, 5, BondOrder::Single, DoublePosition::Unspecified)?,
        ],
        false,
    )?;
    let request = attached(&engine, 17, 5, &[[128.0, 64.0]]);
    engine.dispatch("chain.create_path", request)?;
    let f = &engine.document()?.pages()[0].fragments()[0];
    assert_eq!(
        f.atoms()
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(3), Some(1), Some(3), Some(3)]
    );
    assert_eq!(
        (
            f.bonds()[2].id.get(),
            f.bonds()[2].begin.get(),
            f.bonds()[2].end.get()
        ),
        (74, 5, 108)
    );
    Ok(())
}

#[test]
fn closed_point_objects_bounds_duplicates_occupancy_and_h_proofs_are_atomic() -> TestResult {
    let mut engine = seed()?;
    let valid = free(&engine, &[[200.0, 200.0], [248.0, 200.0]]);
    let mut bad = vec![Value::Null, json!([])];
    for field in [
        "expected_revision",
        "page_id",
        "hydrogen_policy",
        "placement",
    ] {
        let mut p = valid.clone();
        p.as_object_mut().ok_or("params")?.remove(field);
        bad.push(p);
    }
    for (field, value) in [
        ("count", json!(2)),
        ("angle_degrees", json!(120)),
        ("side", json!("positive")),
        ("unused", json!(true)),
    ] {
        let mut p = valid.clone();
        p[field] = value;
        bad.push(p);
    }
    for policy in [
        Value::Null,
        json!({"rectify_neutral_carbon":null}),
        json!(["rectify_neutral_carbon"]),
    ] {
        let mut p = valid.clone();
        p["hydrogen_policy"] = policy;
        bad.push(p);
    }
    for value in [
        json!([]),
        json!([{"x_pt":1,"y_pt":2}]),
        points(&(0..65).map(|i| [f64::from(i), 0.0]).collect::<Vec<_>>()),
    ] {
        let mut p = valid.clone();
        p["placement"]["points"] = value;
        bad.push(p);
    }
    for point in [
        json!([200, 200]),
        Value::Null,
        json!({"x_pt":200}),
        json!({"x_pt":200,"y_pt":200,"z_pt":0}),
        json!({"x_pt":"200","y_pt":200}),
        json!({"x_pt":null,"y_pt":200}),
        json!({"x_pt":100001,"y_pt":200}),
    ] {
        let mut p = valid.clone();
        p["placement"]["points"][0] = point;
        bad.push(p);
    }
    for (key, value) in [
        ("new_points", points(&[[300.0, 300.0]])),
        ("fragment_id", json!(17)),
        ("side", json!("positive")),
    ] {
        let mut p = valid.clone();
        p["placement"][key] = value;
        bad.push(p);
    }
    let mut p = valid.clone();
    p["placement"]["mode"] = json!("bond");
    bad.push(p);
    for n in [0, 64] {
        let p = attached(
            &engine,
            17,
            29,
            &(0..n)
                .map(|i| [200.0 + f64::from(i), 200.0])
                .collect::<Vec<_>>(),
        );
        bad.push(p);
    }
    let mut p = attached(&engine, 17, 29, &[[200.0, 200.0]]);
    p["placement"]["points"] = points(&[[200.0, 200.0]]);
    bad.push(p);
    for p in bad {
        refuse(&mut engine, p, "invalid_params")?;
    }
    for p in [
        free(&engine, &[[200.0, 200.0], [200.0, 200.0]]),
        free(&engine, &[[200.0, 200.0], [248.0, 200.0], [200.0, 200.0]]),
        attached(&engine, 17, 29, &[[128.0, 128.0]]),
        attached(&engine, 17, 29, &[[200.0, 200.0], [200.0, 200.0]]),
        free(&engine, &[[128.0, 128.0], [248.0, 200.0]]),
        free(&engine, &[[200.0, 200.0], [64.0, 128.0]]),
        attached(&engine, 17, 29, &[[64.0, 128.0]]),
    ] {
        refuse(&mut engine, p, "invalid_params")?;
    }
    for (f, a, code) in [
        (0, 29, "invalid_params"),
        (17, 0, "invalid_params"),
        (18, 29, "not_found"),
        (17, 999, "not_found"),
    ] {
        let p = attached(&engine, f, a, &[[200.0, 200.0]]);
        refuse(&mut engine, p, code)?;
    }
    let mut stale = free(&engine, &[]);
    stale["expected_revision"] = json!(1);
    refuse(&mut engine, stale, "stale_revision")?;
    let mut missing = valid.clone();
    missing["page_id"] = json!(2);
    refuse(&mut engine, missing, "not_found")?;
    let mut zero = valid;
    zero["page_id"] = json!(0);
    refuse(&mut engine, zero, "invalid_params")?;
    for change in ["unknown_h", "inconsistent_h", "position", "aromatic"] {
        let mut a = atom(29, [128.0, 128.0], 3)?;
        let mut order = BondOrder::Single;
        match change {
            "unknown_h" => a.explicit_hydrogens = None,
            "inconsistent_h" => a.explicit_hydrogens = Some(2),
            "position" => a.position = None,
            "aromatic" => order = BondOrder::Aromatic,
            _ => return Err("fixture tag".into()),
        }
        let mut e = model(
            17,
            vec![a, atom(11, [64.0, 128.0], 3)?],
            vec![bond(41, 29, 11, order, DoublePosition::Unspecified)?],
            false,
        )?;
        let p = attached(&e, 17, 29, &[[192.0, 128.0]]);
        refuse(&mut e, p, "unsupported_content")?;
    }
    let mut atoms = vec![atom(29, [128.0, 128.0], 0)?];
    let mut bonds = Vec::new();
    for (i, p) in [[80.0, 128.0], [176.0, 128.0], [128.0, 80.0], [128.0, 176.0]]
        .into_iter()
        .enumerate()
    {
        atoms.push(atom(i as u32 + 1, p, 3)?);
        bonds.push(bond(
            i as u32 + 1,
            29,
            i as u32 + 1,
            BondOrder::Single,
            DoublePosition::Unspecified,
        )?);
    }
    let mut full = model(17, atoms, bonds, false)?;
    let p = attached(&full, 17, 29, &[[200.0, 200.0]]);
    refuse(&mut full, p, "unsupported_content")?;
    Ok(())
}

#[test]
fn complete_native_and_foreign_sources_identity_limits_and_late_cancellation_keep_authority()
-> TestResult {
    let original = StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?;
    assert!(original.is_fully_mapped());
    let mut engine = Engine::from_document(original.clone());
    let before = engine.content().clone();
    let p = attached(&engine, 1, 1, &[[100.0, 152.0], [124.0, 152.0]]);
    engine.dispatch("chain.create_path", p)?;
    let changed = engine.content().clone();
    assert_eq!(original.export_cdxml()?, NATIVE);
    assert!(engine.state().source.ok_or("source")?.modified);
    assert!(engine.state().file.dirty);
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.content(), &before);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    assert!(!engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &changed);
    let raw = std::str::from_utf8(NATIVE)?.replace("<CDXML ", "<CDXML PathOpaque=\"retained\" ");
    let mut opaque = Engine::from_document(StoredDocument::from_cdxml(
        raw.as_bytes().into(),
        PageSize::A4,
    )?);
    let p = free(&opaque, &[]);
    refuse(&mut opaque, p, "unsupported_content")?;
    assert_eq!(opaque.stored_document()?.export_cdxml()?, raw.as_bytes());
    let dir = support::FileTestDirectory::new()?;
    for (name, bytes, command) in [
        (
            "all.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "all.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
        ),
        (
            "all.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        (
            "all.smi",
            b"CC first\r\nO\topaque\xff\0".as_slice(),
            "document.open_smiles",
        ),
    ] {
        let path = dir.path(name);
        fs::write(&path, bytes)?;
        let mut e = Engine::default();
        e.dispatch(command, json!({"path":path}))?;
        let p = free(&e, &[[300.0, 300.0], [348.0, 300.0]]);
        refuse(&mut e, p, "unsupported_content")?;
        assert_eq!(fs::read(&path)?, bytes);
        if command == "document.open_smiles" {
            e.dispatch("document.import_smiles_records",json!({"expected_revision":e.state().revision,"selection":{"mode":"record","record_id":1},
                "bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,"page_width_pt":595.275590551,"page_height_pt":841.88976378}))?;
            let p = free(&e, &[[300.0, 300.0], [348.0, 300.0]]);
            refuse(&mut e, p, "unsupported_content")?;
            assert_eq!(e.stored_document()?.smiles_file()?.as_bytes(), bytes);
        }
    }
    for (fragment, atoms, bonds) in [
        (17, vec![atom(u32::MAX, [100.0, 100.0], 4)?], vec![]),
        (u32::MAX, vec![atom(1, [100.0, 100.0], 4)?], vec![]),
        (
            17,
            vec![atom(1, [100.0, 100.0], 3)?, atom(2, [148.0, 100.0], 3)?],
            vec![bond(
                u32::MAX,
                1,
                2,
                BondOrder::Single,
                DoublePosition::Unspecified,
            )?],
        ),
    ] {
        let mut e = model(fragment, atoms, bonds, false)?;
        let p = free(&e, &[[400.0, 400.0], [448.0, 400.0]]);
        refuse(&mut e, p, "identity_exhausted")?;
    }
    let mut e = Engine::default();
    e.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert_eq!(e.state().close, CloseState::Approved);
    let p = free(&e, &[[64.0, 176.0], [112.0, 176.0]]);
    let prepared = e.prepare(Command::parse("chain.create_path", p)?)?;
    assert_eq!(e.state().revision, 1);
    assert_eq!(e.state().close, CloseState::NeedsDecision);
    assert!(!e.complete_close());
    let cancel = bondline_commands::Cancellation::default();
    cancel.cancel();
    let result = e.finish(prepared.complete_with_cancel(&cancel)?)?;
    assert!(result.changed);
    assert_eq!(result.revision, 1);
    let mut closed = Engine::default();
    closed.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert!(closed.complete_close());
    let p = free(&closed, &[]);
    refuse(&mut closed, p, "close_committed")?;
    Ok(())
}
