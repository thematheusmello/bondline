use base64::{Engine as _, engine::general_purpose::STANDARD};
use bondline_commands::{Command, Engine};
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, PageSize, Point2,
    Radical,
};
use proptest::prelude::*;
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    num::NonZeroU16,
    path::Path,
    process::{Command as Process, Stdio},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the original isolated directory")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const ORDERS: [(&str, BondOrder, u16); 3] = [
    ("single", BondOrder::Single, 3),
    ("double", BondOrder::Double, 2),
    ("triple", BondOrder::Triple, 1),
];

fn params(engine: &Engine, fragment: u32, bond: u32, order: &str) -> Value {
    json!({"expected_revision":engine.state().revision,"page_id":1,"fragment_id":fragment,
        "bond_id":bond,"order":order,"hydrogen_policy":"rectify_neutral_carbon"})
}

fn carbon(id: u32, x: f64, y: f64, h: Option<u16>) -> TestResult<Atom> {
    let mut a = Atom::new(
        AtomId::new(id).ok_or("atom")?,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(x, y)?),
    );
    a.explicit_hydrogens = h;
    Ok(a)
}

fn bond(id: u32, begin: u32, end: u32, order: BondOrder) -> TestResult<Bond> {
    Ok(Bond {
        id: BondId::new(id).ok_or("bond")?,
        begin: AtomId::new(begin).ok_or("begin")?,
        end: AtomId::new(end).ok_or("end")?,
        order,
    })
}

fn seed(fragment: u32, atoms: Vec<Atom>, bonds: Vec<Bond>) -> TestResult<Engine> {
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(fragment).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(Engine::from_document(stored))
}

fn pair(order: BondOrder, h: u16) -> TestResult<Engine> {
    seed(
        17,
        vec![
            carbon(11, 72.0, 144.0, Some(h))?,
            carbon(29, 120.0, 144.0, Some(h))?,
        ],
        vec![bond(41, 29, 11, order)?],
    )
}

fn created(order: &str) -> TestResult<Engine> {
    let mut e = Engine::default();
    e.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":144,"end_x_pt":120,"end_y_pt":144,"order":order}))?;
    Ok(e)
}

fn chain() -> TestResult<Engine> {
    seed(
        1,
        vec![
            carbon(1, 72.0, 144.0, Some(3))?,
            carbon(2, 120.0, 144.0, Some(2))?,
            carbon(3, 168.0, 144.0, Some(2))?,
            carbon(4, 216.0, 144.0, Some(3))?,
        ],
        vec![
            bond(1, 1, 2, BondOrder::Single)?,
            bond(2, 3, 2, BondOrder::Single)?,
            bond(3, 3, 4, BondOrder::Single)?,
        ],
    )
}

fn fail(engine: &mut Engine, p: Value, code: &str) -> TestResult {
    let state = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let error = engine
        .dispatch("bond.set_order", p)
        .err()
        .ok_or("invalid edit accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(engine.content(), &content);
    assert_eq!(serde_json::to_value(engine.state())?, state);
    assert_eq!(engine.capture_stamp(), stamp);
    Ok(())
}

fn hydrogens(engine: &Engine) -> TestResult<Vec<Option<u16>>> {
    Ok(engine.document()?.pages()[0].fragments()[0]
        .atoms()
        .iter()
        .map(|a| a.explicit_hydrogens)
        .collect())
}

#[test]
fn single_to_double_is_one_exact_existing_bond_edit() -> TestResult {
    let mut engine = Engine::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":148.0,"end_y_pt":200.0}),
    )?;
    let result = engine.dispatch(
        "bond.set_order",
        json!({"expected_revision":1,"page_id":1,"fragment_id":1,"bond_id":1,
            "order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
    )?;
    assert!(result.changed);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (2, 2, 0)
    );
    assert_eq!(
        serde_json::to_value(result.document)?["pages"][0]["fragments"],
        json!([{"id":1,"atoms":[
            {"id":1,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2},
            {"id":2,"element":6,"position":{"x_pt":148.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2}],
            "bonds":[{"id":1,"begin":1,"end":2,"order":"double"}]}]),
    );
    Ok(())
}

#[test]
fn all_nine_pair_conversions_preserve_nonsequential_ids_and_reversed_direction() -> TestResult {
    for (old, old_order, old_h) in ORDERS {
        for (new, _, new_h) in ORDERS {
            let mut e = pair(old_order, old_h)?;
            let original = e.stored_document()?.clone();
            let result = e.dispatch("bond.set_order", params(&e, 17, 41, new))?;
            assert_eq!(result.changed, old != new);
            assert_eq!(
                (result.revision, result.undo_depth, result.redo_depth),
                if old == new { (0, 0, 0) } else { (1, 1, 0) }
            );
            assert_eq!(
                serde_json::to_value(e.document()?)?["pages"][0]["fragments"],
                json!([{"id":17,"atoms":[
                {"id":11,"element":6,"position":{"x_pt":72.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":new_h},
                {"id":29,"element":6,"position":{"x_pt":120.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":new_h}],
                "bonds":[{"id":41,"begin":29,"end":11,"order":new}]}])
            );
            if old != new {
                let edited = e.stored_document()?.clone();
                e.dispatch("history.undo", json!({}))?;
                assert_eq!(e.stored_document()?, &original);
                e.dispatch("history.redo", json!({}))?;
                assert_eq!(e.stored_document()?, &edited);
            }
        }
    }
    Ok(())
}

#[test]
fn internal_chain_and_cycle_orders_use_all_incident_weights_without_changing_other_bonds()
-> TestResult {
    for (name, _, h) in ORDERS {
        let mut e = chain()?;
        let before = serde_json::to_value(e.document()?)?;
        let mut expected = before;
        expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(h - 1);
        expected["pages"][0]["fragments"][0]["atoms"][2]["explicit_hydrogens"] = json!(h - 1);
        expected["pages"][0]["fragments"][0]["bonds"][1]["order"] = json!(name);
        e.dispatch("bond.set_order", params(&e, 1, 2, name))?;
        assert_eq!(serde_json::to_value(e.document()?)?, expected);
    }
    for (name, expected_h) in [("double", [1, 1, 2, 2]), ("triple", [0, 0, 2, 2])] {
        let mut e = seed(
            1,
            vec![
                carbon(1, 72.0, 72.0, Some(2))?,
                carbon(2, 120.0, 72.0, Some(2))?,
                carbon(3, 120.0, 120.0, Some(2))?,
                carbon(4, 72.0, 120.0, Some(2))?,
            ],
            vec![
                bond(1, 1, 2, BondOrder::Single)?,
                bond(2, 2, 3, BondOrder::Single)?,
                bond(3, 3, 4, BondOrder::Single)?,
                bond(4, 4, 1, BondOrder::Single)?,
            ],
        )?;
        let mut expected = serde_json::to_value(e.document()?)?;
        for (atom, h) in expected["pages"][0]["fragments"][0]["atoms"]
            .as_array_mut()
            .ok_or("atoms")?
            .iter_mut()
            .zip(expected_h)
        {
            atom["explicit_hydrogens"] = json!(h);
        }
        expected["pages"][0]["fragments"][0]["bonds"][0]["order"] = json!(name);
        e.dispatch("bond.set_order", params(&e, 1, 1, name))?;
        assert_eq!(serde_json::to_value(e.document()?)?, expected);
    }
    Ok(())
}

#[test]
fn missing_positions_and_unaffected_hetero_metadata_are_preserved_as_graph_data() -> TestResult {
    let mut first = carbon(1, 72.0, 72.0, Some(3))?;
    first.position = None;
    let mut second = carbon(2, 120.0, 72.0, Some(2))?;
    second.position = None;
    let mut oxygen = carbon(3, 168.0, 72.0, None)?;
    oxygen.element = Element::new(8).ok_or("O")?;
    oxygen.position = None;
    oxygen.isotope = NonZeroU16::new(18);
    oxygen.charge = -1;
    oxygen.radical = Radical::Doublet;
    let oxygen_before = oxygen.clone();
    let mut e = seed(
        1,
        vec![first, second, oxygen],
        vec![
            bond(1, 1, 2, BondOrder::Single)?,
            bond(2, 2, 3, BondOrder::Single)?,
        ],
    )?;
    let mut separate = carbon(91, 500.0, 500.0, None)?;
    separate.element = Element::new(118).ok_or("Og")?;
    separate.radical = Radical::Triplet;
    let untouched = Fragment::new(
        FragmentId::new(7).ok_or("fragment")?,
        vec![separate],
        vec![],
    )?;
    let mut stored = e.stored_document()?.clone();
    stored
        .model_mut()
        .append_fragments(PageId::new(1).ok_or("page")?, vec![untouched])?;
    e = Engine::from_document(stored);
    let untouched_arc = Arc::clone(&e.document()?.pages()[0].fragments()[1]);
    e.dispatch("bond.set_order", params(&e, 1, 1, "double"))?;
    assert_eq!(hydrogens(&e)?, vec![Some(2), Some(1), None]);
    assert_eq!(
        e.document()?.pages()[0].fragments()[0].atoms()[2],
        oxygen_before
    );
    assert!(
        e.document()?.pages()[0].fragments()[0]
            .atoms()
            .iter()
            .all(|a| a.position.is_none())
    );
    assert!(Arc::ptr_eq(
        &untouched_arc,
        &e.document()?.pages()[0].fragments()[1]
    ));
    Ok(())
}

#[test]
fn validated_same_order_is_a_shared_noop_preserving_redo_and_saved_state() -> TestResult {
    let mut e = pair(BondOrder::Double, 2)?;
    e.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    e.dispatch("history.undo", json!({}))?;
    let before = serde_json::to_value(e.state())?;
    let model = e.stored_document()?.model_snapshot();
    let stamp = e.capture_stamp();
    let result = e.dispatch("bond.set_order", params(&e, 17, 41, "double"))?;
    assert!(!result.changed);
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert_eq!(e.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    assert_eq!(e.state().redo_depth, 1);
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(
        e.document()?.pages()[0].size(),
        PageSize::new(612.0, 792.0)?
    );
    Ok(())
}

#[test]
fn parameter_object_policy_target_ids_and_revision_are_closed_and_atomic() -> TestResult {
    let mut e = pair(BondOrder::Single, 3)?;
    e.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":400,"height_pt":600}),
    )?;
    e.dispatch("history.undo", json!({}))?;
    for (key, value, code) in [
        ("expected_revision", json!(0), "stale_revision"),
        ("expected_revision", json!(u64::MAX), "stale_revision"),
        ("page_id", json!(0), "invalid_params"),
        ("fragment_id", json!(0), "invalid_params"),
        ("bond_id", json!(0), "invalid_params"),
        ("page_id", json!(9), "not_found"),
        ("fragment_id", json!(9), "not_found"),
        ("bond_id", json!(9), "not_found"),
        ("order", json!("aromatic"), "invalid_params"),
        ("order", json!("DOUBLE"), "invalid_params"),
        ("order", Value::Null, "invalid_params"),
        (
            "hydrogen_policy",
            json!("rectify_terminal_carbon"),
            "invalid_params",
        ),
        ("hydrogen_policy", Value::Null, "invalid_params"),
        ("begin_atom_id", json!(11), "invalid_params"),
        ("isotope", json!(13), "invalid_params"),
    ] {
        let mut p = params(&e, 17, 41, "double");
        p[key] = value;
        fail(&mut e, p, code)?;
    }
    for key in [
        "expected_revision",
        "page_id",
        "fragment_id",
        "bond_id",
        "order",
        "hydrogen_policy",
    ] {
        let mut p = params(&e, 17, 41, "double");
        p.as_object_mut().ok_or("params")?.remove(key);
        fail(&mut e, p, "invalid_params")?;
    }
    for (key, value) in [
        ("expected_revision", json!(-1)),
        ("page_id", json!(1.5)),
        ("fragment_id", json!(u64::MAX)),
        ("bond_id", json!("41")),
        ("expected_revision", Value::Null),
    ] {
        let mut p = params(&e, 17, 41, "double");
        p[key] = value;
        fail(&mut e, p, "invalid_params")?;
    }
    assert_eq!(e.state().redo_depth, 1);
    Ok(())
}

#[test]
fn both_endpoint_chemistries_and_old_literal_h_are_validated_even_for_same_order() -> TestResult {
    for affected in 0..2 {
        for invalid in 0..10 {
            let mut atoms = vec![
                carbon(11, 72.0, 144.0, Some(3))?,
                carbon(29, 120.0, 144.0, Some(3))?,
            ];
            let a = atoms.get_mut(affected).ok_or("affected")?;
            match invalid {
                0 => a.element = Element::new(8).ok_or("O")?,
                1 => a.isotope = NonZeroU16::new(13),
                2 => a.charge = 1,
                3 => a.radical = Radical::Singlet,
                4 => a.radical = Radical::Doublet,
                5 => a.radical = Radical::Triplet,
                6 => a.explicit_hydrogens = None,
                7 => a.explicit_hydrogens = Some(0),
                8 => a.explicit_hydrogens = Some(4),
                _ => a.explicit_hydrogens = Some(u16::MAX),
            }
            let mut e = seed(17, atoms, vec![bond(41, 29, 11, BondOrder::Single)?])?;
            for requested in ["single", "double"] {
                let p = params(&e, 17, 41, requested);
                fail(&mut e, p, "unsupported_content")?;
            }
        }
    }
    Ok(())
}

#[test]
fn unsupported_incident_orders_and_valence_exhaustion_refuse_without_partial_endpoint_edits()
-> TestResult {
    for unsupported in [
        BondOrder::Quadruple,
        BondOrder::Quintuple,
        BondOrder::Sextuple,
        BondOrder::Half,
        BondOrder::Aromatic,
        BondOrder::TwoAndHalf,
        BondOrder::ThreeAndHalf,
        BondOrder::FourAndHalf,
        BondOrder::FiveAndHalf,
        BondOrder::Dative,
        BondOrder::Ionic,
        BondOrder::Hydrogen,
        BondOrder::ThreeCenter,
    ] {
        let mut e = seed(
            1,
            vec![
                carbon(1, 72.0, 72.0, Some(3))?,
                carbon(2, 120.0, 72.0, Some(2))?,
                carbon(3, 168.0, 72.0, Some(3))?,
            ],
            vec![
                bond(1, 1, 2, BondOrder::Single)?,
                bond(2, 2, 3, unsupported)?,
            ],
        )?;
        for requested in ["single", "double"] {
            let p = params(&e, 1, 1, requested);
            fail(&mut e, p, "unsupported_content")?;
        }
    }
    let mut e = seed(
        1,
        vec![
            carbon(1, 72.0, 72.0, Some(3))?,
            carbon(2, 120.0, 72.0, Some(1))?,
            carbon(3, 168.0, 48.0, Some(3))?,
            carbon(4, 168.0, 96.0, Some(3))?,
        ],
        vec![
            bond(1, 1, 2, BondOrder::Single)?,
            bond(2, 2, 3, BondOrder::Single)?,
            bond(3, 2, 4, BondOrder::Single)?,
        ],
    )?;
    let p = params(&e, 1, 1, "triple");
    fail(&mut e, p, "unsupported_content")?;
    e.dispatch("bond.set_order", params(&e, 1, 1, "double"))?;
    assert_eq!(hydrogens(&e)?, vec![Some(2), Some(0), Some(3), Some(3)]);
    let mut impossible = seed(
        1,
        vec![
            carbon(1, 72.0, 72.0, Some(1))?,
            carbon(2, 120.0, 72.0, Some(0))?,
            carbon(3, 168.0, 72.0, Some(1))?,
        ],
        vec![
            bond(1, 1, 2, BondOrder::Triple)?,
            bond(2, 2, 3, BondOrder::Triple)?,
        ],
    )?;
    let p = params(&impossible, 1, 1, "single");
    fail(&mut impossible, p, "unsupported_content")?;
    Ok(())
}

#[test]
fn source_guards_and_wrong_collection_kind_preserve_the_complete_owner() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let extended = String::from_utf8(NATIVE.to_vec())?
        .replace("<CDXML ", "<CDXML Private=\"unmapped-retained\" ");
    let mut foreign = Engine::from_document(StoredDocument::from_cdxml(
        extended.as_bytes().into(),
        PageSize::A4,
    )?);
    assert!(!foreign.stored_document()?.is_fully_mapped());
    let p = params(&foreign, 1, 1, "double");
    fail(&mut foreign, p, "unsupported_content")?;
    assert_eq!(
        foreign.stored_document()?.export_cdxml()?,
        extended.as_bytes()
    );
    let cdx = include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx");
    let mut binary = Engine::from_document(StoredDocument::from_cdx(
        cdx.as_slice().into(),
        PageSize::A4,
    )?);
    let p = params(&binary, 1, 1, "double");
    fail(&mut binary, p, "unsupported_content")?;
    assert_eq!(binary.stored_document()?.export_cdx()?, cdx);
    let mut native = created("single")?;
    let mol = directory.path("authority-化学.mol");
    native.dispatch("document.save_mol", json!({"path":mol}))?;
    assert!(native.state().source.is_none());
    native.dispatch("bond.set_order", params(&native, 1, 1, "double"))?;
    let mut imported = Engine::default();
    imported.dispatch("document.open_mol", json!({"path":mol}))?;
    let bytes = fs::read(&mol)?;
    let p = params(&imported, 1, 1, "double");
    fail(&mut imported, p, "unsupported_content")?;
    assert_eq!(imported.stored_document()?.export_mol()?, bytes);
    for (extension, source, command) in [
        (
            "sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        ("smi", b"CC\n".as_slice(), "document.open_smiles"),
    ] {
        let path = directory.path(&format!("collection.{extension}"));
        fs::write(&path, source)?;
        let mut e = Engine::default();
        e.dispatch(command, json!({"path":path}))?;
        let p = params(&e, 1, 1, "double");
        fail(&mut e, p, "unsupported_content")?;
        assert_eq!(fs::read(&path)?, source);
    }
    Ok(())
}

fn python() -> std::ffi::OsString {
    std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    })
}

fn consume_xml(path: &Path, h: u16, weight: u8) -> TestResult {
    let code = r#"
import base64,json,sys,xml.parsers.expat
from decimal import Decimal
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
p=json.load(sys.stdin);raw=base64.b64decode(p['source'],validate=True);nodes=[];bonds=[];ids=set()
def start(name,a):
 if 'id' in a: assert a['id'] not in ids;ids.add(a['id'])
 if name=='n': nodes.append(a)
 if name=='b': bonds.append(a)
parser=xml.parsers.expat.ParserCreate();parser.SetParamEntityParsing(xml.parsers.expat.XML_PARAM_ENTITY_PARSING_NEVER);parser.StartElementHandler=start;parser.Parse(raw,True)
assert len(nodes)==2 and len(bonds)==1
for node,point in zip(nodes,[[72,144],[120,144]]):
 assert node['Element']=='6' and node['Charge']=='0' and node['Radical']=='None'
 assert int(node['NumHydrogens'])==p['h'] and [Decimal(x) for x in node['p'].split()]==list(map(Decimal,point))
assert bonds[0]['B']==nodes[0]['id'] and bonds[0]['E']==nodes[1]['id'] and int(bonds[0]['Order'])==p['weight']
ms=Chem.MolsFromCDXML(raw.decode('ascii'));assert len(ms)==1;m=ms[0]
expected={1:('CC','C2H6'),2:('C=C','C2H4'),3:('C#C','C2H2')}[p['weight']]
assert (Chem.MolToSmiles(m),rdMolDescriptors.CalcMolFormula(m))==expected
assert [a.GetTotalNumHs() for a in m.GetAtoms()]==[p['h'],p['h']]
assert [b.GetBondTypeAsDouble() for b in m.GetBonds()]==[p['weight']]
print('{}')
"#;
    let mut child = Process::new(python())
        .args(["-B", "-c", code])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(
        child.stdin.take().ok_or("stdin")?,
        &json!({"source":STANDARD.encode(fs::read(path)?),"h":h,"weight":weight}),
    )?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn consume_mol(path: &Path, h: u16, weight: u8) -> TestResult {
    let mut child = Process::new(python())
        .args([
            "-B",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../io/tests/mol_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("stdin")?
        .write_all(&fs::read(path)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    let (smiles, formula) = match weight {
        1 => ("CC", "C2H6"),
        2 => ("C=C", "C2H4"),
        _ => ("C#C", "C2H2"),
    };
    assert_eq!(result["smiles"], smiles);
    assert_eq!(result["formula"], formula);
    assert_eq!(result["atoms"].as_array().ok_or("atoms")?.len(), 2);
    for (a, point) in result["atoms"]
        .as_array()
        .ok_or("atoms")?
        .iter()
        .zip([[0.0, -3.0, 0.0], [2.0, -3.0, 0.0]])
    {
        assert_eq!(a["element"], 6);
        assert_eq!(a["charge"], 0);
        assert_eq!(a["isotope"], 0);
        assert_eq!(a["radical_electrons"], 0);
        assert_eq!(a["hydrogens"], h);
        assert_eq!(a["position"], json!(point));
    }
    assert_eq!(
        result["bonds"],
        json!([{"begin":0,"end":1,"order":f64::from(weight)}])
    );
    Ok(())
}

#[test]
fn all_nine_native_reopen_edits_save_real_cdxml_and_both_mol_versions_for_independent_consumers()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for (old, _, _) in ORDERS {
        for (new, _, h) in ORDERS {
            let mut original = created(old)?;
            let input = directory.path(&format!("original-{old}-{new}-λ.cdxml"));
            original.dispatch("document.save_cdxml", json!({"path":input}))?;
            let raw = fs::read(&input)?;
            let mut e = Engine::default();
            e.dispatch("document.open_cdxml", json!({"path":input}))?;
            assert!(e.stored_document()?.is_fully_mapped());
            let before = e.stored_document()?.clone();
            e.dispatch("bond.set_order", params(&e, 1, 1, new))?;
            assert_eq!(fs::read(&input)?, raw);
            assert_eq!(e.state().source.ok_or("source")?.modified, old != new);
            let after = e.stored_document()?.clone();
            let model = e.document()?.clone();
            let xml = directory.path(&format!("edited-{old}-{new}-化学.cdxml"));
            e.dispatch("document.save_cdxml", json!({"path":xml}))?;
            if old == new {
                assert_eq!(fs::read(&xml)?, raw);
            }
            consume_xml(&xml, h, (4 - h) as u8)?;
            let mut reopened = Engine::default();
            reopened.dispatch("document.open_cdxml", json!({"path":xml}))?;
            assert_eq!(reopened.document()?, &model);
            assert!(reopened.stored_document()?.is_fully_mapped());
            for version in ["v2000", "v3000"] {
                let path = directory.path(&format!("edited-{old}-{new}-{version}-化学.mol"));
                e.dispatch("document.save_mol", json!({"path":path,"version":version}))?;
                consume_mol(&path, h, (4 - h) as u8)?;
                let mut reopened = Engine::default();
                reopened.dispatch("document.open_mol", json!({"path":path}))?;
                assert_eq!(reopened.document()?, &model);
                assert_eq!(reopened.stored_document()?.export_mol()?, fs::read(&path)?);
            }
            if old != new {
                e.dispatch("history.undo", json!({}))?;
                assert_eq!(e.stored_document()?, &before);
                assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), raw);
                e.dispatch("history.redo", json!({}))?;
                assert_eq!(e.stored_document()?, &after);
            }
        }
    }
    Ok(())
}

#[test]
fn original_native_v0_source_and_pending_save_inputs_survive_edits_and_history() -> TestResult {
    let mut old = Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    let baseline = old.stored_document()?.clone();
    old.dispatch("bond.set_order", params(&old, 1, 1, "triple"))?;
    assert_eq!(hydrogens(&old)?, vec![Some(1), Some(1)]);
    old.dispatch("history.undo", json!({}))?;
    assert_eq!(old.stored_document()?, &baseline);
    assert_eq!(old.stored_document()?.to_cdxml()?.as_ref(), NATIVE);
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("captured-single.cdxml");
    let newer = directory.path("current-double.cdxml");
    let mut e = created("single")?;
    let work = e.prepare(Command::parse("document.save_cdxml", json!({"path":path}))?)?;
    e.dispatch("bond.set_order", params(&e, 1, 1, "double"))?;
    let completed = work.complete()?;
    assert_eq!(
        e.prepare(Command::parse(
            "document.save_cdxml",
            json!({"path":newer})
        )?)
        .err()
        .ok_or("save permit released")?
        .info()
        .code,
        "busy"
    );
    assert!(!newer.try_exists()?);
    let receipt = e.finish(completed)?;
    assert_eq!(receipt.file_operation.ok_or("receipt")?.input_revision, 1);
    assert!(receipt.file.dirty);
    consume_xml(&path, 3, 1)?;
    e.dispatch("document.save_cdxml", json!({"path":newer}))?;
    consume_xml(&newer, 2, 2)?;
    let stale = e.prepare(Command::parse("document.open_cdxml", json!({"path":path}))?)?;
    e.dispatch("bond.set_order", params(&e, 1, 1, "triple"))?;
    let before = serde_json::to_value(e.state())?;
    let owner = e.content().clone();
    assert_eq!(
        e.finish(stale.complete()?)
            .err()
            .ok_or("stale open accepted")?
            .info()
            .code,
        "stale_revision"
    );
    assert_eq!(serde_json::to_value(e.state())?, before);
    assert_eq!(e.content(), &owner);
    Ok(())
}

#[test]
fn editing_allocates_no_ids_and_discovery_requires_every_policy_field() -> TestResult {
    let mut e = created("single")?;
    for order in ["double", "triple", "single"] {
        e.dispatch("bond.set_order", params(&e, 1, 1, order))?;
    }
    e.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":240,"begin_y_pt":240,"end_x_pt":288,"end_y_pt":240}),
    )?;
    let second = &e.document()?.pages()[0].fragments()[1];
    assert_eq!(second.id().get(), 2);
    assert_eq!(
        second
            .atoms()
            .iter()
            .map(|a| a.id.get())
            .collect::<Vec<_>>(),
        [3, 4]
    );
    assert_eq!(second.bonds()[0].id.get(), 2);
    let registry = bondline_commands::registry();
    let entry = registry
        .iter()
        .find(|c| c.name == "bond.set_order")
        .ok_or("command missing")?;
    assert!(!entry.read_only);
    let schema = serde_json::to_value(&entry.input_schema)?;
    assert_eq!(schema["additionalProperties"], false);
    let mut required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or("required name"))
        .collect::<Result<Vec<_>, _>>()?;
    required.sort_unstable();
    assert_eq!(
        required,
        [
            "bond_id",
            "expected_revision",
            "fragment_id",
            "hydrogen_policy",
            "order",
            "page_id"
        ]
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases:128,rng_seed:proptest::test_runner::RngSeed::Fixed(0xb0_0d_0d_e2),failure_persistence:None,..ProptestConfig::default() })]
    #[test]
    fn generated_internal_order_edits_undo_and_redo_complete_snapshots(orders in proptest::collection::vec(0usize..3,1..24)) {
        let mut e=chain().map_err(|error|TestCaseError::fail(error.to_string()))?;let original=e.stored_document()?.clone();let mut snapshots=Vec::new();
        for index in orders {
            let (name,_,h)=ORDERS[index];let before=e.stored_document()?.clone();
            let result=e.dispatch("bond.set_order",params(&e,1,2,name))?;
            prop_assert_eq!(hydrogens(&e).map_err(|error|TestCaseError::fail(error.to_string()))?,vec![Some(3),Some(h-1),Some(h-1),Some(3)]);
            prop_assert_eq!(e.document()?.pages()[0].fragments()[0].bonds()[0].order,BondOrder::Single);
            prop_assert_eq!(e.document()?.pages()[0].fragments()[0].bonds()[2].order,BondOrder::Single);
            if result.changed {
                let after=e.stored_document()?.clone();e.dispatch("history.undo",json!({}))?;prop_assert_eq!(e.stored_document()?,&before);e.dispatch("history.redo",json!({}))?;prop_assert_eq!(e.stored_document()?,&after);snapshots.push(after);
            }
        }
        for expected in snapshots.iter().rev().skip(1) {e.dispatch("history.undo",json!({}))?;prop_assert_eq!(e.stored_document()?,expected);}
        if !snapshots.is_empty() {e.dispatch("history.undo",json!({}))?;}
        prop_assert_eq!(e.stored_document()?,&original);
        for expected in &snapshots {e.dispatch("history.redo",json!({}))?;prop_assert_eq!(e.stored_document()?,expected);}
    }
}
