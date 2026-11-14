use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, PageSize, Point2,
    Radical,
};
use serde_json::{Value, json};
use std::{num::NonZeroU16, sync::Arc};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn create() -> Value {
    json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0})
}

fn extend(revision: u64) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"begin_atom_id":2,
        "end_x_pt":180.0,"end_y_pt":240.0,"hydrogen_policy":"rectify_terminal_carbon"})
}

fn atom(id: u32, x: f64, y: f64) -> TestResult<Atom> {
    let mut atom = Atom::new(
        AtomId::new(id).ok_or("atom")?,
        Element::try_from(6)?,
        Some(Point2::new(x, y)?),
    );
    atom.explicit_hydrogens = Some(3);
    Ok(atom)
}

fn pair(fragment: u32, first: u32, second: u32, bond: u32) -> TestResult<Fragment> {
    Ok(Fragment::new(
        FragmentId::new(fragment).ok_or("fragment")?,
        vec![atom(first, 100.0, 200.0)?, atom(second, 140.0, 200.0)?],
        vec![Bond {
            id: BondId::new(bond).ok_or("bond")?,
            begin: AtomId::new(first).ok_or("begin")?,
            end: AtomId::new(second).ok_or("end")?,
            order: BondOrder::Single,
        }],
    )?)
}

fn seeded(fragments: Vec<Fragment>) -> TestResult<Engine> {
    let mut stored = StoredDocument::default();
    stored
        .model_mut()
        .append_fragments(PageId::new(1).ok_or("page")?, fragments)?;
    Ok(Engine::from_document(stored))
}

fn assert_failure(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let document = engine.stored_document().clone();
    let before = serde_json::to_value(engine.state())?;
    let error = engine
        .dispatch("bond.extend", params)
        .err()
        .ok_or("request unexpectedly succeeded")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(engine.stored_document(), &document);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

#[test]
fn explicit_terminal_rectification_is_one_exact_propane_history_step() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch("bond.create", create())?;
    let ethane = engine.stored_document().clone();
    let result = engine.dispatch("bond.extend", extend(1))?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 2, 0)
    );
    assert_eq!(
        serde_json::to_value(&result.document)?["pages"][0]["fragments"],
        json!([
            {"id":1,"atoms":[
                {"id":1,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
                {"id":2,"element":6,"position":{"x_pt":140.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2},
                {"id":3,"element":6,"position":{"x_pt":180.0,"y_pt":240.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],
             "bonds":[{"id":1,"begin":1,"end":2,"order":"single"},{"id":2,"begin":2,"end":3,"order":"single"}]}
        ])
    );
    let propane = engine.stored_document().clone();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document(), &ethane);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document(), &propane);
    assert_eq!(
        serde_json::to_value(result.document)?["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"],
        2
    );
    Ok(())
}

#[test]
fn target_ids_policy_revision_and_free_geometry_are_closed_and_fail_atomically() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch("bond.create", create())?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400.0,"height_pt":600.0}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    for (key, value, code) in [
        ("expected_revision", json!(1), "stale_revision"),
        ("page_id", json!(0), "invalid_params"),
        ("page_id", json!(2), "not_found"),
        ("fragment_id", json!(0), "invalid_params"),
        ("fragment_id", json!(2), "not_found"),
        ("begin_atom_id", json!(0), "invalid_params"),
        ("begin_atom_id", json!(99), "not_found"),
        ("end_x_pt", json!(100001.0), "invalid_params"),
        ("end_x_pt", json!(-100001.0), "invalid_params"),
        ("end_y_pt", Value::Null, "invalid_params"),
        ("hydrogen_policy", json!("infer"), "invalid_params"),
        ("begin_x_pt", json!(140.0), "invalid_params"),
    ] {
        let mut params = extend(3);
        params[key] = value;
        assert_failure(&mut engine, params, code)?;
    }
    for missing in [
        "expected_revision",
        "page_id",
        "fragment_id",
        "begin_atom_id",
        "end_x_pt",
        "end_y_pt",
        "hydrogen_policy",
    ] {
        let mut params = extend(3);
        params.as_object_mut().ok_or("params")?.remove(missing);
        assert_failure(&mut engine, params, "invalid_params")?;
    }
    for (x, y) in [(140.0, 200.0), (100.0, 200.0)] {
        let mut params = extend(3);
        params["end_x_pt"] = json!(x);
        params["end_y_pt"] = json!(y);
        assert_failure(&mut engine, params, "invalid_params")?;
    }
    assert_eq!(engine.state().redo_depth, 1);
    let valid = engine.dispatch("bond.extend", extend(3))?;
    assert_eq!(valid.redo_depth, 0);
    assert_eq!(
        serde_json::to_value(valid.document)?["pages"][0]["fragments"][0]["atoms"][2]["id"],
        3,
        "rejected attempts cannot consume identities"
    );
    Ok(())
}

#[test]
fn explicit_h_overrides_unknown_counts_metadata_and_nonterminal_starts_are_not_reinferred()
-> TestResult {
    for hydrogens in [None, Some(0), Some(1), Some(2), Some(4), Some(u16::MAX)] {
        let source = pair(1, 1, 2, 1)?;
        let mut atoms = source.atoms().to_vec();
        atoms.get_mut(1).ok_or("target")?.explicit_hydrogens = hydrogens;
        let mut engine = seeded(vec![Fragment::new(
            source.id(),
            atoms,
            source.bonds().to_vec(),
        )?])?;
        assert_failure(&mut engine, extend(0), "unsupported_content")?;
    }
    for change in 0..7 {
        let source = pair(1, 1, 2, 1)?;
        let mut atoms = source.atoms().to_vec();
        let mut bonds = source.bonds().to_vec();
        match change {
            0 => atoms.get_mut(1).ok_or("atom")?.charge = 1,
            1 => atoms.get_mut(1).ok_or("atom")?.isotope = NonZeroU16::new(13),
            2 => atoms.get_mut(1).ok_or("atom")?.radical = Radical::Doublet,
            3 => atoms.get_mut(1).ok_or("atom")?.position = None,
            4 => atoms.get_mut(0).ok_or("atom")?.element = Element::try_from(8)?,
            5 => bonds.get_mut(0).ok_or("bond")?.order = BondOrder::Double,
            _ => bonds.clear(),
        }
        let mut engine = seeded(vec![Fragment::new(source.id(), atoms, bonds)?])?;
        assert_failure(&mut engine, extend(0), "unsupported_content")?;
    }
    let mut engine = Engine::default();
    engine.dispatch("bond.create", create())?;
    engine.dispatch("bond.extend", extend(1))?;
    assert_failure(&mut engine, extend(2), "unsupported_content")?;
    Ok(())
}

#[test]
fn old_fragment_identity_is_preserved_and_only_atom_bond_domains_allocate() -> TestResult {
    let mut engine = seeded(vec![pair(u32::MAX, 1, 2, 1)?])?;
    let mut params = extend(0);
    params["fragment_id"] = json!(u32::MAX);
    let result = engine.dispatch("bond.extend", params)?;
    let fragment = &serde_json::to_value(result.document)?["pages"][0]["fragments"][0];
    assert_eq!(fragment["id"], u32::MAX);
    assert_eq!(fragment["atoms"][2]["id"], 3);
    assert_eq!(fragment["bonds"][1]["id"], 2);
    for (last, bond) in [(u32::MAX, 1), (2, u32::MAX)] {
        let mut engine = seeded(vec![pair(1, 1, last, bond)?])?;
        let mut params = extend(0);
        params["begin_atom_id"] = json!(last);
        assert_failure(&mut engine, params, "identity_exhausted")?;
    }
    Ok(())
}

#[test]
fn document_wide_maxima_and_history_floors_do_not_reuse_removed_ids() -> TestResult {
    let other = pair(90, 500, 900, 800)?;
    let mut other_atoms = other.atoms().to_vec();
    other_atoms.first_mut().ok_or("other first")?.position = Some(Point2::new(300.0, 400.0)?);
    other_atoms.get_mut(1).ok_or("other second")?.position = Some(Point2::new(340.0, 400.0)?);
    let other = Fragment::new(other.id(), other_atoms, other.bonds().to_vec())?;
    let mut engine = seeded(vec![pair(1, 1, 2, 1)?, other])?;
    let mut wrong = extend(0);
    wrong["fragment_id"] = json!(90);
    assert_failure(&mut engine, wrong, "not_found")?;
    let mut wrong = extend(0);
    wrong["begin_atom_id"] = json!(900);
    assert_failure(&mut engine, wrong, "not_found")?;
    let mut occupied = extend(0);
    occupied["end_x_pt"] = json!(340.0);
    occupied["end_y_pt"] = json!(400.0);
    assert_failure(&mut engine, occupied, "invalid_params")?;
    let before = engine.stored_document().clone();
    engine.dispatch("bond.extend", extend(0))?;
    let after = engine.stored_document().clone();
    let model = serde_json::to_value(engine.document())?;
    assert_eq!(model["pages"][0]["fragments"][0]["atoms"][2]["id"], 901);
    assert_eq!(model["pages"][0]["fragments"][0]["bonds"][1]["id"], 801);
    assert!(Arc::ptr_eq(
        before.model().pages()[0]
            .fragments()
            .get(1)
            .ok_or("before unrelated")?,
        after.model().pages()[0]
            .fragments()
            .get(1)
            .ok_or("after unrelated")?
    ));
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document(), &before);
    let branch = engine.dispatch("bond.extend", extend(2))?;
    assert_eq!(
        serde_json::to_value(branch.document)?["pages"][0]["fragments"][0]["atoms"][2]["id"],
        902
    );
    assert_eq!(
        serde_json::to_value(engine.document())?["pages"][0]["fragments"][0]["bonds"][1]["id"],
        802
    );
    assert_eq!(engine.state().redo_depth, 0);
    let created=engine.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":200.0,"begin_y_pt":200.0,"end_x_pt":240.0,"end_y_pt":200.0}))?;
    let fragments = &serde_json::to_value(created.document)?["pages"][0]["fragments"];
    assert_eq!(fragments[2]["id"], 91);
    assert_eq!(fragments[2]["atoms"][0]["id"], 903);
    assert_eq!(fragments[2]["bonds"][0]["id"], 803);
    Ok(())
}

#[test]
fn native_profile_authority_is_not_granted_by_drawability_or_creator_name() -> TestResult {
    const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
    let mut engine =
        Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    assert!(engine.stored_document().is_fully_mapped());
    let before = engine.stored_document().clone();
    engine.dispatch("bond.extend", extend(0))?;
    assert!(engine.state().source.ok_or("source")?.modified);
    let saved = engine.stored_document().to_cdxml()?.into_owned();
    assert_eq!(
        StoredDocument::from_cdxml(saved.into(), PageSize::A4)?.model(),
        engine.document()
    );
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document(), &before);
    assert_eq!(engine.stored_document().to_cdxml()?.as_ref(), NATIVE);
    let source = std::str::from_utf8(NATIVE)?.replace("<CDXML ", "<CDXML Future=\"opaque\" ");
    let stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
    let mut engine = Engine::from_document(stored);
    assert_failure(&mut engine, extend(0), "unsupported_content")?;
    assert_eq!(
        engine.stored_document().to_cdxml()?.as_ref(),
        source.as_bytes()
    );
    for source in [
        include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
        include_bytes!("../../io/tests/fixtures/v3000/noncontiguous-cc.mol").as_slice(),
    ] {
        let frame = bondline_io::mol::MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
        let mut engine = Engine::from_document(StoredDocument::from_mol(
            source.into(),
            PageSize::A4,
            frame,
        )?);
        assert_failure(&mut engine, extend(0), "unsupported_content")?;
        assert_eq!(engine.stored_document().to_mol(frame)?.as_ref(), source);
    }
    let source = include_bytes!("../../cdx/tests/fixtures/original-ethane-header-document.cdx");
    let mut engine = Engine::from_document(StoredDocument::from_cdx(
        source.as_slice().into(),
        PageSize::A4,
    )?);
    assert_failure(&mut engine, extend(0), "unsupported_content")?;
    assert_eq!(engine.stored_document().to_cdx()?.as_ref(), source);
    Ok(())
}

#[test]
fn a_graph_at_the_atom_budget_refuses_extension_without_losing_its_snapshot() -> TestResult {
    let original = pair(1, 1, 2, 1)?;
    let mut atoms = original.atoms().to_vec();
    for id in 3..=bondline_model::MAX_GRAPH_ITEMS as u32 {
        atoms.push(atom(id, 0.0, 0.0)?);
    }
    let mut engine = seeded(vec![Fragment::new(
        original.id(),
        atoms,
        original.bonds().to_vec(),
    )?])?;
    assert_failure(&mut engine, extend(0), "invalid_params")?;
    Ok(())
}

#[test]
fn unrelated_authored_hydrogens_are_not_recalculated() -> TestResult {
    let original = pair(1, 1, 2, 1)?;
    let mut atoms = original.atoms().to_vec();
    atoms.first_mut().ok_or("other atom")?.explicit_hydrogens = Some(1);
    let mut engine = seeded(vec![Fragment::new(
        original.id(),
        atoms,
        original.bonds().to_vec(),
    )?])?;
    let result = engine.dispatch("bond.extend", extend(0))?;
    assert_eq!(
        serde_json::to_value(result.document)?["pages"][0]["fragments"][0]["atoms"][0]["explicit_hydrogens"],
        1
    );
    Ok(())
}

#[test]
fn extension_invalidates_close_approval_and_undo_cannot_resurrect_it() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch("bond.create", create())?;
    let before = engine.stored_document().clone();
    let approved = engine.dispatch(
        "document.close",
        json!({"expected_revision":1,"action":"discard"}),
    )?;
    assert_eq!(approved.close, bondline_commands::CloseState::Approved);
    engine.dispatch("bond.extend", extend(1))?;
    assert_eq!(
        engine.state().close,
        bondline_commands::CloseState::NeedsDecision
    );
    assert!(!engine.complete_close());
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document(), &before);
    assert!(!engine.complete_close());
    assert_eq!(
        engine
            .dispatch(
                "document.close",
                json!({"expected_revision":1,"action":"discard"})
            )
            .err()
            .ok_or("old close accepted")?
            .info()
            .code,
        "stale_revision"
    );
    engine.dispatch(
        "document.close",
        json!({"expected_revision":3,"action":"cancel"}),
    )?;
    assert_eq!(engine.state().close, bondline_commands::CloseState::Open);
    Ok(())
}

#[test]
fn registry_exposes_the_required_closed_extension_policy_and_target_contract() -> TestResult {
    let commands = bondline_commands::registry();
    let info = commands
        .iter()
        .find(|info| info.name == "bond.extend")
        .ok_or("extension schema")?;
    assert!(!info.read_only);
    let schema = info.input_schema.as_value();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["required"],
        json!([
            "expected_revision",
            "page_id",
            "fragment_id",
            "begin_atom_id",
            "end_x_pt",
            "end_y_pt",
            "hydrogen_policy"
        ])
    );
    assert_eq!(
        schema["$defs"]["ExtendHydrogenPolicy"]["enum"],
        json!(["rectify_terminal_carbon"])
    );
    assert!(
        schema["properties"]["hydrogen_policy"]
            .get("default")
            .is_none()
    );
    for key in ["page_id", "fragment_id", "begin_atom_id"] {
        assert_eq!(schema["properties"][key]["minimum"], 1);
    }
    for key in ["end_x_pt", "end_y_pt"] {
        assert_eq!(schema["properties"][key]["minimum"], -100000);
        assert_eq!(schema["properties"][key]["maximum"], 100000);
    }
    Ok(())
}
