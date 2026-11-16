use bondline_commands::Engine;
use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn literal_atom_label_is_a_named_atomic_authoring_command() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,
        "begin_x_pt":72.0,"begin_y_pt":144.0,"end_x_pt":120.0,"end_y_pt":144.0}),
    )?;
    let revision = engine.state().revision;
    let result = engine.dispatch(
        "atom.set_label",
        json!({
            "expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":2,
            "element":8,"explicit_hydrogens":1
        }),
    )?;
    assert!(result.changed);
    assert_eq!(result.revision, revision + 1);
    let state: Value = serde_json::to_value(result)?;
    let atoms = &state["document"]["pages"][0]["fragments"][0]["atoms"];
    assert_eq!(atoms[0]["element"], 6);
    assert_eq!(atoms[0]["explicit_hydrogens"], 3);
    assert_eq!(atoms[1]["element"], 8);
    assert_eq!(atoms[1]["explicit_hydrogens"], 1);
    Ok(())
}

use bondline_commands::{CloseState, Command};
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, PageSize, Point2,
    Radical,
};
use proptest::prelude::*;
use std::{
    fs,
    io::Write,
    num::NonZeroU16,
    path::Path,
    process::{Command as Process, Stdio},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod support;

const NAME: &str = "atom.set_label";
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

fn label(engine: &Engine, fragment: u32, atom: u32, element: u8, h: u16) -> Value {
    json!({"expected_revision":engine.state().revision,"page_id":1,
        "fragment_id":fragment,"atom_id":atom,"element":element,"explicit_hydrogens":h})
}

fn carbon(id: u32, position: Option<Point2>, h: Option<u16>) -> TestResult<Atom> {
    let mut atom = Atom::new(
        AtomId::new(id).ok_or("atom identity")?,
        Element::new(6).ok_or("carbon")?,
        position,
    );
    atom.explicit_hydrogens = h;
    Ok(atom)
}

fn seeded(atoms: Vec<Atom>, order: BondOrder) -> TestResult<Engine> {
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            atoms,
            vec![Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(41).ok_or("bond")?,
                begin: AtomId::new(29).ok_or("begin")?,
                end: AtomId::new(11).ok_or("end")?,
                order,
            }],
        )?],
    )?;
    Ok(Engine::from_document(stored))
}

fn sparse_pair() -> TestResult<Engine> {
    seeded(
        vec![
            carbon(11, Some(Point2::new(72.0, 144.0)?), Some(3))?,
            carbon(29, Some(Point2::new(120.0, 144.0)?), Some(3))?,
        ],
        BondOrder::Single,
    )
}

fn fail(engine: &mut Engine, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let model = engine
        .stored_document()
        .ok()
        .map(StoredDocument::model_snapshot);
    let error = engine
        .dispatch(NAME, params)
        .err()
        .ok_or("invalid label accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
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
fn only_two_literal_fields_change_with_sparse_ids_missing_positions_and_unrelated_metadata()
-> TestResult {
    let target = carbon(29, None, None)?;
    let mut neighbour = carbon(11, None, None)?;
    neighbour.element = Element::new(8).ok_or("oxygen")?;
    neighbour.isotope = NonZeroU16::new(18);
    neighbour.charge = -1;
    neighbour.radical = Radical::Doublet;
    let mut engine = seeded(vec![neighbour, target], BondOrder::Dative)?;
    let mut stored = engine.stored_document()?.clone();
    let mut unrelated = carbon(91, Some(Point2::new(500.0, 500.0)?), Some(0))?;
    unrelated.element = Element::new(118).ok_or("Og")?;
    unrelated.radical = Radical::Triplet;
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(7).ok_or("unrelated")?,
            vec![unrelated],
            vec![],
        )?],
    )?;
    engine = Engine::from_document(stored);
    let original = engine.stored_document()?.clone();
    let unselected = Arc::clone(&engine.document()?.pages()[0].fragments()[1]);
    let mut expected = serde_json::to_value(engine.document()?)?;
    expected["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(7);
    expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(2);
    let result = engine.dispatch(NAME, label(&engine, 17, 29, 7, 2))?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (1, 1, 0)
    );
    assert_eq!(serde_json::to_value(engine.document()?)?, expected);
    assert!(Arc::ptr_eq(
        &unselected,
        &engine.document()?.pages()[0].fragments()[1]
    ));
    let edited = engine.stored_document()?.clone();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document()?, &original);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document()?, &edited);
    Ok(())
}

#[test]
fn closed_six_scalar_fields_bounds_target_errors_and_revision_priority_are_atomic() -> TestResult {
    let registry = bondline_commands::registry();
    let entry = registry
        .iter()
        .find(|entry| entry.name == NAME)
        .ok_or("label command missing")?;
    assert!(!entry.read_only);
    let schema = serde_json::to_value(&entry.input_schema)?;
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["properties"].as_object().ok_or("properties")?.len(),
        6
    );
    let mut required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|value| value.as_str().ok_or("required name"))
        .collect::<Result<Vec<_>, _>>()?;
    required.sort_unstable();
    assert_eq!(
        required,
        [
            "atom_id",
            "element",
            "expected_revision",
            "explicit_hydrogens",
            "fragment_id",
            "page_id"
        ]
    );
    assert_eq!(
        schema["properties"]["element"]["minimum"].as_f64(),
        Some(1.0)
    );
    assert_eq!(
        schema["properties"]["element"]["maximum"].as_f64(),
        Some(118.0)
    );
    assert_eq!(
        schema["properties"]["explicit_hydrogens"]["minimum"].as_f64(),
        Some(0.0)
    );
    assert_eq!(
        schema["properties"]["explicit_hydrogens"]["maximum"].as_f64(),
        Some(65535.0)
    );

    let mut engine = sparse_pair()?;
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    let valid = label(&engine, 17, 29, 8, 1);
    assert_eq!(Command::parse(NAME, valid.clone())?.name(), NAME);
    for (key, value, code) in [
        ("expected_revision", json!(0), "stale_revision"),
        ("expected_revision", json!(u64::MAX), "stale_revision"),
        ("page_id", json!(0), "invalid_params"),
        ("fragment_id", json!(0), "invalid_params"),
        ("atom_id", json!(0), "invalid_params"),
        ("page_id", json!(9), "not_found"),
        ("fragment_id", json!(9), "not_found"),
        ("atom_id", json!(9), "not_found"),
        ("element", json!(0), "invalid_params"),
        ("element", json!(119), "invalid_params"),
        ("element", json!(255), "invalid_params"),
        ("element", json!(256), "invalid_params"),
        ("element", json!(-1), "invalid_params"),
        ("element", json!(8.5), "invalid_params"),
        ("element", json!("O"), "invalid_params"),
        ("element", Value::Null, "invalid_params"),
        ("explicit_hydrogens", json!(65536), "invalid_params"),
        ("explicit_hydrogens", json!(-1), "invalid_params"),
        ("explicit_hydrogens", json!(1.5), "invalid_params"),
        ("explicit_hydrogens", json!("1"), "invalid_params"),
        ("explicit_hydrogens", Value::Null, "invalid_params"),
        ("expected_revision", json!(-1), "invalid_params"),
        ("page_id", json!(u64::MAX), "invalid_params"),
        ("fragment_id", json!(1.5), "invalid_params"),
        ("atom_id", json!("29"), "invalid_params"),
        ("isotope", json!(13), "invalid_params"),
        ("hydrogen_policy", json!("automatic"), "invalid_params"),
    ] {
        let mut params = valid.clone();
        params[key] = value;
        fail(&mut engine, params, code)?;
    }
    for key in [
        "expected_revision",
        "page_id",
        "fragment_id",
        "atom_id",
        "element",
        "explicit_hydrogens",
    ] {
        let mut params = valid.clone();
        params.as_object_mut().ok_or("params")?.remove(key);
        fail(&mut engine, params, "invalid_params")?;
    }
    let mut stale = valid.clone();
    stale["expected_revision"] = json!(0);
    stale["page_id"] = json!(0);
    stale["element"] = json!(0);
    fail(&mut engine, stale, "stale_revision")?;
    let mut physical_h = valid;
    physical_h["element"] = json!(1);
    fail(&mut engine, physical_h, "invalid_params")?;
    assert_eq!(engine.state().redo_depth, 1);
    Ok(())
}

#[test]
fn unsupported_target_metadata_refuses_even_an_otherwise_identical_label() -> TestResult {
    for variant in 0..6 {
        let first = carbon(11, None, Some(3))?;
        let mut target = carbon(29, None, Some(3))?;
        match variant {
            0 => target.charge = 1,
            1 => target.charge = -1,
            2 => target.isotope = NonZeroU16::new(13),
            3 => target.radical = Radical::Singlet,
            4 => target.radical = Radical::Doublet,
            _ => target.radical = Radical::Triplet,
        }
        let mut engine = seeded(vec![first, target], BondOrder::Single)?;
        for (element, h) in [(6, 3), (8, 1)] {
            let params = label(&engine, 17, 29, element, h);
            fail(&mut engine, params, "unsupported_content")?;
        }
    }
    Ok(())
}

#[test]
fn every_foreign_backing_and_collection_refuses_without_losing_original_bytes() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let opaque = String::from_utf8(NATIVE.to_vec())?
        .replace("<CDXML ", "<CDXML Private=\"retained-original\" ");
    let mut engine = Engine::from_document(StoredDocument::from_cdxml(
        opaque.as_bytes().into(),
        PageSize::A4,
    )?);
    assert!(!engine.stored_document()?.is_fully_mapped());
    for (element, h) in [(6, 3), (8, 1)] {
        let params = label(&engine, 1, 2, element, h);
        fail(&mut engine, params, "unsupported_content")?;
    }
    assert_eq!(engine.stored_document()?.export_cdxml()?, opaque.as_bytes());
    for (name, bytes, command) in [
        (
            "original-v2.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "original-v3.mol",
            include_bytes!("../../io/tests/fixtures/v3000/noncontiguous-cc.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "original.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
        ),
        (
            "original.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        (
            "original.smi",
            b"CC original\n".as_slice(),
            "document.open_smiles",
        ),
    ] {
        let path = directory.path(name);
        fs::write(&path, bytes)?;
        let mut engine = Engine::default();
        engine.dispatch(command, json!({"path":path}))?;
        for (element, h) in [(6, 3), (8, 1)] {
            let params = label(&engine, 1, 2, element, h);
            fail(&mut engine, params, "unsupported_content")?;
        }
        let mut stale = label(&engine, 1, 2, 8, 1);
        stale["expected_revision"] = json!(0);
        fail(&mut engine, stale, "stale_revision")?;
        let mut invalid_target = label(&engine, 0, 0, 8, 1);
        invalid_target["page_id"] = json!(0);
        fail(&mut engine, invalid_target, "unsupported_content")?;
        assert_eq!(fs::read(&path)?, bytes);
        if command == "document.open_smiles" {
            engine.dispatch(
                "document.import_smiles_records",
                json!({
                    "expected_revision":engine.state().revision,"selection":{"mode":"all"},
                    "bond_length_pt":64,"origin_x_pt":96,"origin_y_pt":96,
                    "page_width_pt":612,"page_height_pt":792
                }),
            )?;
            assert!(engine.stored_document()?.has_smiles_projection());
            assert!(!engine.stored_document()?.is_fully_mapped());
            for (element, h) in [(6, 3), (8, 1)] {
                let params = label(&engine, 1, 2, element, h);
                fail(&mut engine, params, "unsupported_content")?;
            }
            assert_eq!(engine.stored_document()?.smiles_file()?.as_bytes(), bytes);
            assert_eq!(fs::read(&path)?, bytes);
        }
    }
    Ok(())
}

#[test]
fn true_noop_shares_native_source_model_redo_and_close_but_changed_edit_commits_once() -> TestResult
{
    let mut engine =
        Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    assert!(engine.stored_document()?.is_fully_mapped());
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"request"}),
    )?;
    assert_eq!(engine.state().close, CloseState::Approved);
    let state = serde_json::to_value(engine.state())?;
    let stamp = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let source = engine.stored_document()?.export_cdxml()?.as_ptr();
    let original = engine.stored_document()?.clone();
    let noop = engine.dispatch(NAME, label(&engine, 1, 2, 6, 3))?;
    assert!(!noop.changed);
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.stored_document()?.export_cdxml()?.as_ptr(), source);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    assert_eq!(engine.state().redo_depth, 1);
    let old_revision = engine.state().revision;
    let result = engine.dispatch(NAME, label(&engine, 1, 2, 8, 1))?;
    assert_eq!(
        (
            result.changed,
            result.revision,
            result.undo_depth,
            result.redo_depth
        ),
        (true, old_revision + 1, 1, 0)
    );
    assert_eq!(result.close, CloseState::NeedsDecision);
    assert!(result.file.dirty);
    assert_eq!(serde_json::to_value(&result.source)?["modified"], true);
    let edited = engine.stored_document()?.clone();
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"cancel"}),
    )?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.stored_document()?, &original);
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.stored_document()?, &edited);

    let mut committed = sparse_pair()?;
    committed.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert!(committed.complete_close());
    for (element, h) in [(6, 3), (8, 1)] {
        let params = label(&committed, 17, 29, element, h);
        fail(&mut committed, params, "close_committed")?;
    }
    Ok(())
}

#[test]
fn all_118_element_codes_and_uint16_literal_h_are_format_data_without_valence_inference()
-> TestResult {
    let mut engine = sparse_pair()?;
    for element in 1..=118_u8 {
        let before = serde_json::to_value(engine.document()?)?;
        let mut expected = before;
        expected["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(element);
        expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(0);
        engine.dispatch(NAME, label(&engine, 17, 29, element, 0))?;
        assert_eq!(serde_json::to_value(engine.document()?)?, expected);
    }
    engine.dispatch(NAME, label(&engine, 17, 29, 14, u16::MAX))?;
    let atom = &engine.document()?.pages()[0].fragments()[0].atoms()[1];
    assert_eq!(
        (atom.element.get(), atom.explicit_hydrogens),
        (14, Some(u16::MAX))
    );
    // This intentionally non-saturated literal is not passed to a chemistry oracle.
    let raw = engine.stored_document()?.to_cdxml()?.into_owned();
    let projected = expat(&raw)?;
    assert_eq!(projected[0]["atoms"][1]["element"], 14);
    assert_eq!(projected[0]["atoms"][1]["explicit_hydrogens"], 65535);
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,
        "begin_x_pt":300,"begin_y_pt":300,"end_x_pt":348,"end_y_pt":300}),
    )?;
    let new = &engine.document()?.pages()[0].fragments()[1];
    assert_eq!(new.id().get(), 18);
    assert_eq!(
        new.atoms()
            .iter()
            .map(|atom| atom.id.get())
            .collect::<Vec<_>>(),
        [30, 31]
    );
    assert_eq!(new.bonds()[0].id.get(), 42);
    Ok(())
}

fn python() -> TestResult<std::ffi::OsString> {
    std::env::var_os("BONDLINE_PYTHON")
        .ok_or_else(|| "BONDLINE_PYTHON is required for independent file consumers".into())
}

fn expat(bytes: &[u8]) -> TestResult<Value> {
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
    child.stdin.take().ok_or("Expat stdin")?.write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn consume_saved(
    path: &Path,
    element: u8,
    h: u16,
    carbon_h: u16,
    order: u8,
    canonical: &str,
    formula: &str,
) -> TestResult {
    let raw = fs::read(path)?;
    let graph = expat(&raw)?;
    let kind = if order == 1 { "single" } else { "double" };
    assert_eq!(
        graph,
        json!([{"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":72.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":carbon_h},
        {"id":2,"element":element,"position":{"x_pt":120.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":kind}]}])
    );
    let code = r#"
import json,sys
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
molecules=Chem.MolsFromCDXML(sys.stdin.buffer.read().decode('ascii'))
assert len(molecules)==1
m=molecules[0]
print(json.dumps({'atoms':[[a.GetAtomicNum(),a.GetTotalNumHs(),a.GetFormalCharge(),a.GetIsotope(),a.GetNumRadicalElectrons()] for a in m.GetAtoms()],
 'bonds':[[b.GetBeginAtomIdx(),b.GetEndAtomIdx(),b.GetBondTypeAsDouble()] for b in m.GetBonds()],
 'canonical':Chem.MolToSmiles(m),'formula':rdMolDescriptors.CalcMolFormula(m)},sort_keys=True))
"#;
    let mut child = Process::new(python()?)
        .args(["-B", "-c", code])
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
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({"atoms":[[6,carbon_h,0,0,0],[element,h,0,0,0]],
        "bonds":[[0,1,f64::from(order)]],"canonical":canonical,"formula":formula})
    );
    Ok(())
}

#[test]
fn actual_unicode_cdxml_saves_reopen_as_literal_oh_nh2_cl_and_carbonyl_graphs() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for backed in [false, true] {
        for (element, h, carbon_h, order, canonical, formula) in [
            (8, 1, 3, 1, "CO", "CH4O"),
            (7, 2, 3, 1, "CN", "CH5N"),
            (17, 0, 3, 1, "CCl", "CH3Cl"),
            (8, 0, 2, 2, "C=O", "CH2O"),
        ] {
            let mut engine = Engine::default();
            engine.dispatch(
                "bond.create",
                json!({"page_id":1,"order":if order == 1 {"single"} else {"double"},
                "begin_x_pt":72,"begin_y_pt":144,"end_x_pt":120,"end_y_pt":144}),
            )?;
            let input = directory.path(&format!("original-{backed}-{canonical}-原.cdxml"));
            let original_bytes = engine.stored_document()?.to_cdxml()?.into_owned();
            if backed {
                fs::write(&input, &original_bytes)?;
                engine = Engine::default();
                engine.dispatch("document.open_cdxml", json!({"path":input}))?;
                assert!(engine.stored_document()?.is_fully_mapped());
            }
            let original = engine.stored_document()?.clone();
            let old = engine.state();
            let result = engine.dispatch(NAME, label(&engine, 1, 2, element, h))?;
            assert_eq!(
                (
                    result.changed,
                    result.revision,
                    result.undo_depth,
                    result.redo_depth
                ),
                (true, old.revision + 1, old.undo_depth + 1, 0)
            );
            assert_eq!(result.source.is_some(), backed);
            let edited = engine.stored_document()?.clone();
            engine.dispatch("history.undo", json!({}))?;
            assert_eq!(engine.stored_document()?, &original);
            engine.dispatch("history.redo", json!({}))?;
            assert_eq!(engine.stored_document()?, &edited);
            let output = directory.path(&format!("label-{backed}-{canonical}-化学-λ.cdxml"));
            engine.dispatch("document.save_cdxml", json!({"path":output}))?;
            assert!(!engine.state().file.dirty);
            assert_eq!(
                engine.state().file.path.as_deref(),
                fs::canonicalize(&output)?.to_str()
            );
            consume_saved(&output, element, h, carbon_h, order, canonical, formula)?;
            let model = serde_json::to_value(engine.document()?)?;
            let saved = fs::read(&output)?;
            let mut reopened = Engine::default();
            reopened.dispatch("document.open_cdxml", json!({"path":output}))?;
            assert!(!reopened.state().file.dirty);
            assert!(reopened.stored_document()?.is_fully_mapped());
            assert_eq!(serde_json::to_value(reopened.document()?)?, model);
            let second = directory.path(&format!("reopened-{backed}-{canonical}-再.cdxml"));
            reopened.dispatch("document.save_cdxml", json!({"path":second}))?;
            assert_eq!(fs::read(&second)?, saved);
            if backed {
                assert_eq!(fs::read(&input)?, original_bytes);
                engine.dispatch("history.undo", json!({}))?;
                assert_eq!(engine.stored_document()?, &original);
                assert_eq!(engine.stored_document()?.export_cdxml()?, original_bytes);
                assert!(engine.state().file.dirty);
                engine.dispatch("history.redo", json!({}))?;
                assert_eq!(engine.stored_document()?, &edited);
                assert!(!engine.state().file.dirty);
            }
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases:64,rng_seed:proptest::test_runner::RngSeed::Fixed(0xa7_0a_1a_be),failure_persistence:None,..ProptestConfig::default() })]
    #[test]
    fn literal_label_sequences_restore_complete_snapshots_without_neighbor_correction(
        labels in proptest::collection::vec(0usize..6, 1..24)
    ) {
        let mut engine = sparse_pair().map_err(|error| TestCaseError::fail(error.to_string()))?;
        let original = engine.stored_document()?.clone();
        let mut expected = serde_json::to_value(engine.document()?)?;
        let mut snapshots = Vec::new();
        let literals = [(6,3),(8,1),(7,2),(17,0),(14,0),(6,65535)];
        for index in labels {
            let (element,h) = literals[index];
            let before = engine.stored_document()?.clone();
            let same = expected["pages"][0]["fragments"][0]["atoms"][1]["element"] == json!(element)
                && expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] == json!(h);
            expected["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(element);
            expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(h);
            let result = engine.dispatch(NAME, label(&engine,17,29,element,h))?;
            prop_assert_eq!(result.changed, !same);
            prop_assert_eq!(serde_json::to_value(engine.document()?)?, expected.clone());
            if result.changed {
                let after = engine.stored_document()?.clone();
                engine.dispatch("history.undo",json!({}))?;
                prop_assert_eq!(engine.stored_document()?, &before);
                engine.dispatch("history.redo",json!({}))?;
                prop_assert_eq!(engine.stored_document()?, &after);
                snapshots.push(after);
            }
        }
        for expected in snapshots.iter().rev().skip(1) {
            engine.dispatch("history.undo",json!({}))?;
            prop_assert_eq!(engine.stored_document()?, expected);
        }
        if !snapshots.is_empty() {engine.dispatch("history.undo",json!({}))?;}
        prop_assert_eq!(engine.stored_document()?, &original);
        for expected in &snapshots {
            engine.dispatch("history.redo",json!({}))?;
            prop_assert_eq!(engine.stored_document()?, expected);
        }
    }
}
