use base64::{Engine as _, engine::general_purpose::STANDARD};
use bondline_commands::{Cancellation, Command, Engine};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, PageId,
    PageSize, Point2,
};
use bondline_render::{Primitive, Stroke, cpu_scene, graph_page_scene};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command as Process, Stdio},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated file directory")]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn python() -> std::ffi::OsString {
    std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    })
}

fn external(code: &str, payload: &Value) -> TestResult<Value> {
    let mut child = Process::new(python())
        .args(["-B", "-c", code])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(child.stdin.take().ok_or("oracle stdin")?, payload)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

// Independently authored fixed-column and V3000 records; no VAL/HCOUNT or writer use.
fn original_mol(version: u8, weight: u8) -> Vec<u8> {
    if version == 2 {
        format!("Bondline original multiple-bond vector\n  Bondline          2D\nIndependent C2 geometry, no declared valence\n  2  1  0  0  0  0            999 V2000\n    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    2.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n  1  2  {weight}  0  0  0  0\nM  END\n").into_bytes()
    } else {
        format!("Bondline original multiple-bond vector\n  Bondline          2D\nIndependent C2 geometry, nonsequential IDs\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS 2 1 0 0 0\nM  V30 BEGIN ATOM\nM  V30 17 C 0 0 0 0\nM  V30 41 C 2 0 0 0\nM  V30 END ATOM\nM  V30 BEGIN BOND\nM  V30 99 {weight} 17 41\nM  V30 END BOND\nM  V30 END CTAB\nM  END\n").into_bytes()
    }
}

fn order(weight: u8) -> &'static str {
    match weight {
        1 => "single",
        2 => "double",
        _ => "triple",
    }
}

fn pair_formula(weight: u8) -> (&'static str, &'static str) {
    match weight {
        1 => ("CC", "C2H6"),
        2 => ("C=C", "C2H4"),
        _ => ("C#C", "C2H2"),
    }
}

fn read_mol(bytes: &[u8]) -> TestResult<Value> {
    let mut child = Process::new(python())
        .args([
            "-B",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../io/tests/mol_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("MOL stdin")?.write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

fn assert_molecular_result(
    result: &Value,
    smiles: &str,
    formula: &str,
    hydrogens: &[u16],
    weights: &[u8],
) -> TestResult {
    assert_eq!(result["smiles"], smiles);
    assert_eq!(result["formula"], formula);
    let atoms = result["atoms"].as_array().ok_or("external atoms")?;
    assert_eq!(atoms.len(), hydrogens.len());
    for (atom, &h) in atoms.iter().zip(hydrogens) {
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["charge"], 0);
        assert_eq!(atom["isotope"], 0);
        assert_eq!(atom["radical_electrons"], 0);
        assert_eq!(atom["hydrogens"], h);
    }
    assert_eq!(result["bonds"], json!(weights.iter().enumerate().map(|(index,&weight)|json!({"begin":index,"end":index+1,"order":f64::from(weight)})).collect::<Vec<_>>()));
    Ok(())
}

fn request(backend: &str) -> Value {
    json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":backend})
}

fn saved_files(
    engine: &mut Engine,
    name: &str,
    h: &[u16],
    weights: &[u8],
    smiles: &str,
    formula: &str,
) -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let xml = directory.path(&format!("{name}-λ.cdxml"));
    let mol = directory.path(&format!("{name}-λ.mol"));
    let model = engine.document()?.clone();
    let before = engine.content().clone();
    let revision = engine.state().revision;
    engine.dispatch("document.save_cdxml", json!({"path":xml}))?;
    let xml_bytes = fs::read(&xml)?;
    let result = external(
        r#"
import base64,json,sys,xml.parsers.expat
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
p=json.load(sys.stdin);source=base64.b64decode(p['source'],validate=True)
nodes=[];bonds=[];ids=set()
def start(name,attributes):
 if 'id' in attributes:
  assert attributes['id'] not in ids;ids.add(attributes['id'])
 if name=='n':nodes.append(attributes)
 if name=='b':bonds.append(attributes)
parser=xml.parsers.expat.ParserCreate();parser.SetParamEntityParsing(xml.parsers.expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.StartElementHandler=start;parser.Parse(source,True)
assert len(nodes)==len(p['h']) and len(bonds)==len(p['weights'])
for i,(node,h) in enumerate(zip(nodes,p['h'])):
 assert node['Element']=='6' and node['Charge']=='0' and int(node['NumHydrogens'])==h
 assert [float(v) for v in node['p'].split()]==[72+48*i,72]
for i,(bond,weight) in enumerate(zip(bonds,p['weights'])):
 assert bond['B']==nodes[i]['id'] and bond['E']==nodes[i+1]['id'] and int(bond['Order'])==weight
molecules=Chem.MolsFromCDXML(source.decode('ascii'));assert len(molecules)==1
m=molecules[0]
print(json.dumps({'smiles':Chem.MolToSmiles(m),'formula':rdMolDescriptors.CalcMolFormula(m),
 'atoms':[{'element':a.GetAtomicNum(),'charge':a.GetFormalCharge(),'isotope':a.GetIsotope(),'radical_electrons':a.GetNumRadicalElectrons(),'hydrogens':a.GetTotalNumHs()} for a in m.GetAtoms()],
 'bonds':[{'begin':b.GetBeginAtomIdx(),'end':b.GetEndAtomIdx(),'order':b.GetBondTypeAsDouble()} for b in m.GetBonds()]}))
"#,
        &json!({"source":STANDARD.encode(&xml_bytes),"h":h,"weights":weights}),
    )?;
    assert_molecular_result(&result, smiles, formula, h, weights)?;
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_cdxml", json!({"path":xml}))?;
    assert_eq!(reopened.document()?, &model);
    assert!(reopened.stored_document()?.is_fully_mapped());
    engine.dispatch("document.save_mol", json!({"path":mol}))?;
    let bytes = fs::read(&mol)?;
    let result = read_mol(&bytes)?;
    assert_molecular_result(&result, smiles, formula, h, weights)?;
    for (index, atom) in result["atoms"]
        .as_array()
        .ok_or("external atoms")?
        .iter()
        .enumerate()
    {
        assert_eq!(atom["position"], json!([2.0 * index as f64, 0.0, 0.0]));
    }
    reopened.dispatch("document.open_mol", json!({"path":mol}))?;
    assert_eq!(reopened.document()?, &model);
    assert_eq!(reopened.stored_document()?.export_mol()?, bytes);
    assert_eq!(engine.content(), &before);
    assert_eq!(engine.state().revision, revision);
    assert!(!engine.state().file.dirty);
    Ok(())
}

// The old literal Single producer supplies only the protected font/paper/masks.
// Multiple-bond strokes are replaced with independent declared parallel strips.
fn reference_png(weight: u8, drawn: u8, h: u16) -> TestResult<String> {
    let mut document = Document::default();
    let mut atoms = Vec::new();
    for (id, x) in [(1, 72.0), (2, 120.0)] {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("atom")?,
            Element::new(6).ok_or("C")?,
            Some(Point2::new(x, 72.0)?),
        );
        atom.explicit_hydrogens = Some(h);
        atoms.push(atom);
    }
    document.append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(1).ok_or("bond")?,
                begin: AtomId::new(1).ok_or("begin")?,
                end: AtomId::new(2).ok_or("end")?,
                order: BondOrder::Single,
            }],
        )?],
    )?;
    let mut scene =
        graph_page_scene(1024, 1024, document.pages().first().ok_or("page")?)?.into_scene();
    assert_eq!(
        scene
            .primitives
            .iter()
            .filter(|item| matches!(item, Primitive::Stroke(_)))
            .count(),
        1
    );
    let index = scene
        .primitives
        .iter()
        .position(|item| matches!(item, Primitive::Stroke(_)))
        .ok_or("reference axis")?;
    let page = PageSize::A4;
    let w = (679.0 / page.width_pt()).min(960.0 / page.height_pt()) as f32;
    let x = |p: f64| (172.0 + p / page.width_pt() * 679.0) as f32;
    let y = (32.0 + 72.0 / page.height_pt() * 960.0) as f32;
    let offsets: &[f64] = match drawn {
        1 => &[0.0],
        2 => &[-1.5, 1.5],
        _ => &[-3.0, 0.0, 3.0],
    };
    let strokes = offsets
        .iter()
        .map(|factor| {
            Primitive::Stroke(Stroke {
                begin: [x(72.0), (f64::from(y) + factor * f64::from(w)) as f32],
                end: [x(120.0), (f64::from(y) + factor * f64::from(w)) as f32],
                width: w,
                color: [0, 0, 0, 255],
            })
        })
        .collect::<Vec<_>>();
    scene.primitives.splice(index..index + 1, strokes);
    assert!((1..=3).contains(&weight));
    Ok(STANDARD.encode(bondline_io::encode_png(&cpu_scene(&scene)?)?))
}

fn pixels(weight: u8, cpu: &str, gpu: &str) -> TestResult {
    let h = u16::from(4 - weight);
    let payload = json!({"order":weight,"cpu":cpu,"gpu":gpu,"reference":reference_png(weight,weight,h)?,"collapsed":reference_png(weight,1,h)?,"wrong_order":reference_png(weight,if weight==2 {3} else {2},h)?,"wrong_h":reference_png(weight,weight,0)?});
    let mut child = Process::new(python())
        .args([
            "-B",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/multiple_pixels.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(child.stdin.take().ok_or("pixel stdin")?, &payload)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

fn create(weight: Option<&str>) -> Value {
    let mut value =
        json!({"page_id":1,"begin_x_pt":72.0,"begin_y_pt":72.0,"end_x_pt":120.0,"end_y_pt":72.0});
    if let Some(weight) = weight {
        value["order"] = json!(weight);
    }
    value
}

fn extend(revision: u64, weight: &str) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"begin_atom_id":2,"end_x_pt":168.0,"end_y_pt":72.0,"hydrogen_policy":"rectify_terminal_carbon","order":weight})
}

fn open_original(path: &Path, version: u8, weight: u8) -> TestResult<Engine> {
    fs::write(path, original_mol(version, weight))?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_mol", json!({"path":path}))?;
    Ok(engine)
}

fn assert_failure(engine: &mut Engine, name: &str, params: Value, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.content().clone();
    assert_eq!(
        engine
            .dispatch(name, params)
            .err()
            .ok_or("accepted invalid operation")?
            .info()
            .code,
        code
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &owner);
    Ok(())
}

#[test]
fn defaults_malformed_orders_source_and_unmapped_style_guards_remain_atomic() -> TestResult {
    let mut default = Engine::default();
    default.dispatch("bond.create", create(None))?;
    let mut explicit = Engine::default();
    explicit.dispatch("bond.create", create(Some("single")))?;
    assert_eq!(default.content(), explicit.content());
    assert_eq!(
        default.stored_document()?.to_cdxml()?.as_ref(),
        explicit.stored_document()?.to_cdxml()?.as_ref()
    );
    assert_eq!(
        default
            .dispatch("render.capture", request("cpu"))?
            .capture
            .ok_or("Single capture")?
            .png_base64,
        reference_png(1, 1, 3)?
    );
    saved_files(
        &mut default,
        "unchanged-default-single",
        &[3, 3],
        &[1],
        "CC",
        "C2H6",
    )?;

    let mut engine = Engine::default();
    engine.dispatch("bond.create", create(None))?;
    engine.dispatch("history.undo", json!({}))?;
    for value in [
        Value::Null,
        json!("aromatic"),
        json!("Double"),
        json!(2),
        json!(true),
        json!({"order":"double"}),
    ] {
        let mut params = create(Some("double"));
        params["order"] = value.clone();
        assert_failure(&mut engine, "bond.create", params, "invalid_params")?;
        let mut params = extend(2, "double");
        params["order"] = value;
        assert_failure(&mut engine, "bond.extend", params, "invalid_params")?;
    }
    let mut missing = extend(2, "double");
    missing
        .as_object_mut()
        .ok_or("params")?
        .remove("hydrogen_policy");
    assert_failure(&mut engine, "bond.extend", missing, "invalid_params")?;
    let mut stale = extend(1, "triple");
    stale["begin_atom_id"] = json!(2);
    assert_failure(&mut engine, "bond.extend", stale, "stale_revision")?;
    assert_eq!(
        (
            engine.state().revision,
            engine.state().undo_depth,
            engine.state().redo_depth
        ),
        (2, 0, 1)
    );
    engine.dispatch("history.redo", json!({}))?;
    engine.dispatch("bond.extend", extend(3, "double"))?;
    let model = serde_json::to_value(engine.document()?)?;
    assert_eq!(model["pages"][0]["fragments"][0]["atoms"][2]["id"], 3);
    assert_eq!(model["pages"][0]["fragments"][0]["bonds"][1]["id"], 2);
    let mut existing_multiple = extend(4, "triple");
    existing_multiple["begin_atom_id"] = json!(1);
    existing_multiple["end_x_pt"] = json!(24.0);
    // A valid terminal's literal H budget is independent of a supported Double elsewhere.
    engine.dispatch("bond.extend", existing_multiple)?;
    let model = serde_json::to_value(engine.document()?)?;
    assert_eq!(
        model["pages"][0]["fragments"][0]["atoms"][0]["explicit_hydrogens"],
        0
    );
    assert_eq!(model["pages"][0]["fragments"][0]["atoms"][3]["id"], 4);
    assert_eq!(
        model["pages"][0]["fragments"][0]["bonds"][2]["order"],
        "triple"
    );

    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("declared-source.mol");
    let source = String::from_utf8(original_mol(3, 1))?
        .replace("17 C 0 0 0 0", "17 C 0 0 0 0 VAL=4")
        .replace("41 C 2 0 0 0", "41 C 2 0 0 0 VAL=4");
    fs::write(&input, source.as_bytes())?;
    let mut imported = Engine::default();
    imported.dispatch("document.open_mol", json!({"path":input}))?;
    assert!(imported.stored_document()?.is_fully_mapped());
    assert_eq!(
        imported.document()?.pages()[0].fragments()[0].atoms()[1].explicit_hydrogens,
        Some(3)
    );
    assert_failure(
        &mut imported,
        "bond.extend",
        extend(1, "triple"),
        "unsupported_content",
    )?;
    assert_eq!(imported.stored_document()?.export_mol()?, source.as_bytes());

    let mut authored = Engine::default();
    authored.dispatch("bond.create", create(Some("double")))?;
    let source = String::from_utf8(authored.stored_document()?.to_cdxml()?.into_owned())?
        .replace("<b ", "<b DoublePosition=\"Right\" BondSpacing=\"18\" ");
    let stored =
        bondline_io::StoredDocument::from_cdxml(source.clone().into_bytes().into(), PageSize::A4)?;
    assert!(!stored.is_fully_mapped());
    let mut guarded = Engine::from_document(stored);
    for backend in ["cpu", "gpu"] {
        assert_failure(
            &mut guarded,
            "render.capture",
            request(backend),
            "unsupported_content",
        )?;
    }
    assert_eq!(
        guarded.stored_document()?.to_cdxml()?.as_ref(),
        source.as_bytes()
    );
    Ok(())
}

#[test]
fn terminal_double_and_triple_extend_only_within_their_authored_hydrogen_budget() -> TestResult {
    for (existing, requested, h, weights, smiles, formula) in [
        (2, 1, vec![2, 1, 3], vec![2, 1], "C=CC", "C3H6"),
        (3, 1, vec![1, 0, 3], vec![3, 1], "C#CC", "C3H4"),
        (2, 2, vec![2, 0, 2], vec![2, 2], "C=C=C", "C3H4"),
    ] {
        let mut engine = Engine::default();
        engine.dispatch("bond.create", create(Some(order(existing))))?;
        let before = engine.content().clone();
        engine.dispatch("bond.extend", extend(1, order(requested)))?;
        let model = serde_json::to_value(engine.document()?)?;
        assert_eq!(
            model["pages"][0]["fragments"][0]["atoms"]
                .as_array()
                .ok_or("atoms")?
                .iter()
                .map(|atom| atom["explicit_hydrogens"].clone())
                .collect::<Vec<_>>(),
            h.iter().map(|value| json!(value)).collect::<Vec<_>>()
        );
        saved_files(
            &mut engine,
            &format!("terminal-{existing}-plus-{requested}"),
            &h,
            &weights,
            smiles,
            formula,
        )?;
        let after = engine.content().clone();
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.content(), &before);
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.content(), &after);
    }
    for (existing, requested) in [(2, 3), (3, 2), (3, 3)] {
        let mut engine = Engine::default();
        engine.dispatch("bond.create", create(Some(order(existing))))?;
        assert_failure(
            &mut engine,
            "bond.extend",
            extend(1, order(requested)),
            "unsupported_content",
        )?;
    }
    Ok(())
}

#[test]
fn multiple_capture_keeps_historical_source_and_cancellation_cannot_apply_a_result() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("old-triple.mol");
    let next = directory.path("new-double.mol");
    let mut engine = open_original(&input, 3, 3)?;
    let original = engine.document()?.clone();
    let work = engine.prepare(Command::parse("render.capture", request("cpu"))?)?;
    assert!(work.is_deferred());
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let edited = engine.content().clone();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.document()?, &original);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &edited);
    fs::remove_file(&input)?;
    fs::write(&next, original_mol(2, 2))?;
    engine.dispatch(
        "document.open_mol",
        json!({"path":next,"discard_unsaved":true}),
    )?;
    let revision = engine.state().revision;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":revision,"action":"request"}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.content().clone();
    let result = engine.finish(work.complete()?)?;
    assert_eq!(result.revision, 1);
    assert_eq!(result.document.as_deref(), Some(&original));
    assert_eq!(
        result
            .source
            .as_ref()
            .ok_or("historical source")?
            .byte_length,
        original_mol(3, 3).len()
    );
    assert!(!result.source.as_ref().ok_or("historical source")?.modified);
    let image = result.capture.ok_or("historical image")?.png_base64;
    pixels(3, &image, &image)?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &owner);
    for late in [false, true] {
        let cancel = Cancellation::default();
        let work = engine.prepare(Command::parse(
            "render.capture",
            request(if late { "cpu" } else { "gpu" }),
        )?)?;
        let error = if late {
            let completed = work.complete_with_cancel(&cancel)?;
            cancel.cancel();
            engine
                .finish(completed)
                .err()
                .ok_or("late cancel ignored")?
        } else {
            cancel.cancel();
            work.complete_with_cancel(&cancel)
                .err()
                .ok_or("early cancel ignored")?
        };
        assert_eq!(error.info().code, "cancelled");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &owner);
    }
    let mut missing = request("cpu");
    missing["page_id"] = json!(2);
    assert_failure(&mut engine, "render.capture", missing, "not_found")?;
    assert_eq!(fs::read(&next)?, original_mol(2, 2));
    Ok(())
}

#[test]
fn explicit_tool_selection_is_transient_and_does_not_override_headless_order_parameters()
-> TestResult {
    let mut engine = Engine::default();
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.content().clone();
    let result = serde_json::to_value(engine.dispatch("ui.bond_tool", json!({"order":"triple"}))?)?;
    assert_eq!(result["ui_bond_tool_operation"]["order"], "triple");
    assert_eq!(result["ui_bond_tool_operation"]["changed"], true);
    assert!(
        result["ui_bond_tool_operation"]["epoch"]
            .as_u64()
            .is_some_and(|epoch| epoch > 0)
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &owner);
    let inspect = serde_json::to_value(engine.dispatch("document.inspect", json!({}))?)?;
    assert_eq!(inspect, before);
    assert!(inspect.get("ui_bond_tool_operation").is_none());
    let controls = serde_json::to_value(engine.dispatch("ui.controls", json!({}))?)?;
    let controls = controls["controls"].as_array().ok_or("controls")?;
    for (name, selected) in [("single", false), ("double", false), ("triple", true)] {
        let id = format!("tool.bond.{name}");
        let control = controls
            .iter()
            .find(|control| control["id"] == id)
            .ok_or("bond tool control")?;
        assert_eq!(control["selected"], selected);
        assert_eq!(control["command"]["command"], "ui.bond_tool");
        assert_eq!(control["command"]["params"]["order"], name);
    }
    engine.dispatch("bond.create", create(None))?;
    assert_eq!(
        serde_json::to_value(engine.document()?)?["pages"][0]["fragments"][0]["bonds"][0]["order"],
        "single"
    );
    engine.dispatch("bond.extend", extend(1, "double"))?;
    let model = serde_json::to_value(engine.document()?)?;
    assert_eq!(
        model["pages"][0]["fragments"][0]["bonds"][1]["order"],
        "double"
    );
    saved_files(
        &mut engine,
        "explicit-order-with-triple-tool-selected",
        &[3, 1, 2],
        &[1, 2],
        "C=CC",
        "C3H6",
    )?;
    Ok(())
}

#[test]
fn baseline_multiple_mol_capture_displays_original_c2h4_and_c2h2() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for version in [2, 3] {
        for weight in [2, 3] {
            let source = original_mol(version, weight);
            let independent = read_mol(&source)?;
            let (smiles, formula) = pair_formula(weight);
            assert_molecular_result(
                &independent,
                smiles,
                formula,
                &[u16::from(4 - weight); 2],
                &[weight],
            )?;
            assert_eq!(independent["atoms"][0]["position"], json!([0.0, 0.0, 0.0]));
            assert_eq!(independent["atoms"][1]["position"], json!([2.0, 0.0, 0.0]));
            let reference = reference_png(weight, weight, u16::from(4 - weight))?;
            pixels(weight, &reference, &reference)?; // Independent oracle/control self-check, no GPU claim.
            let input = directory.path(&format!("original-v{version}-{}.mol", order(weight)));
            let output = directory.path(&format!("saved-v{version}-λ-{}.data", order(weight)));
            fs::write(&input, &source)?;
            let mut engine = Engine::default();
            engine.dispatch("document.open_mol", json!({"path":input}))?;
            assert!(engine.stored_document()?.is_fully_mapped());
            let doc = engine.document()?;
            let atoms = doc.pages()[0].fragments()[0].atoms();
            assert_eq!(
                atoms.iter().map(|atom| atom.position).collect::<Vec<_>>(),
                [
                    Some(Point2::new(72.0, 72.0)?),
                    Some(Point2::new(120.0, 72.0)?)
                ]
            );
            assert_eq!(
                atoms
                    .iter()
                    .map(|atom| atom.explicit_hydrogens)
                    .collect::<Vec<_>>(),
                [None, None]
            );
            assert_eq!(
                serde_json::to_value(engine.state())?["file"]["format"],
                if version == 2 {
                    "mol_v2000"
                } else {
                    "mol_v3000"
                }
            );
            let before = serde_json::to_value(engine.state())?;
            let owner = engine.content().clone();
            let cpu = engine.dispatch("render.capture", request("cpu"));
            assert!(
                cpu.is_ok(),
                "order{weight} actual MOL capture refused: {:?}",
                cpu.as_ref().err().map(|error| error.info().code)
            );
            let cpu = cpu?.capture.ok_or("CPU PNG")?;
            let gpu = engine
                .dispatch("render.capture", request("gpu"))?
                .capture
                .ok_or("GPU PNG")?;
            assert!(
                gpu.adapter
                    .as_ref()
                    .is_some_and(|adapter| !adapter.name.is_empty())
            );
            pixels(weight, &cpu.png_base64, &gpu.png_base64)?;
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert_eq!(engine.content(), &owner);
            engine.dispatch("document.save_mol", json!({"path":output}))?;
            assert_eq!(fs::read(&input)?, source);
            assert_eq!(fs::read(&output)?, source);
            assert_molecular_result(
                &read_mol(&fs::read(&output)?)?,
                smiles,
                formula,
                &[u16::from(4 - weight); 2],
                &[weight],
            )?;
            let mut reopened = Engine::default();
            reopened.dispatch("document.open_mol", json!({"path":output}))?;
            assert_eq!(reopened.document()?, engine.document()?);
            assert_eq!(reopened.stored_document()?.export_mol()?, source);
        }
    }
    Ok(())
}

#[test]
fn baseline_explicit_order_authors_c2_and_attached_c3_with_one_history_step() -> TestResult {
    for weight in [2, 3] {
        let mut engine = Engine::default();
        let initial = engine.content().clone();
        let result = engine.dispatch("bond.create", create(Some(order(weight))));
        assert!(
            result.is_ok(),
            "explicit bond.create.order rejected: {:?}",
            result.as_ref().err().map(|error| error.info().code)
        );
        let result = result?;
        assert!(result.changed);
        assert_eq!(result.revision, 1);
        let model = serde_json::to_value(engine.document()?)?;
        let fragment = &model["pages"][0]["fragments"][0];
        assert_eq!(fragment["bonds"][0]["order"], order(weight));
        assert_eq!(fragment["atoms"][0]["explicit_hydrogens"], 4 - weight);
        assert_eq!(fragment["atoms"][1]["explicit_hydrogens"], 4 - weight);
        let (smiles, formula) = pair_formula(weight);
        saved_files(
            &mut engine,
            &format!("created-{}", order(weight)),
            &[u16::from(4 - weight); 2],
            &[weight],
            smiles,
            formula,
        )?;
        let created = engine.content().clone();
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.content(), &initial);
        assert!(engine.state().file.dirty);
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(engine.content(), &created);
        assert!(!engine.state().file.dirty);
        let mut attached = Engine::default();
        attached.dispatch("bond.create", create(None))?;
        let old = attached.content().clone();
        let result = attached.dispatch("bond.extend", extend(1, order(weight)))?;
        assert_eq!(
            (result.revision, result.undo_depth, result.redo_depth),
            (2, 2, 0)
        );
        let model = serde_json::to_value(attached.document()?)?;
        let fragment = &model["pages"][0]["fragments"][0];
        assert_eq!(fragment["id"], 1);
        assert_eq!(
            fragment["atoms"]
                .as_array()
                .ok_or("atoms")?
                .iter()
                .map(|atom| atom["id"].clone())
                .collect::<Vec<_>>(),
            vec![json!(1), json!(2), json!(3)]
        );
        assert_eq!(
            fragment["bonds"],
            json!([{"id":1,"begin":1,"end":2,"order":"single"},{"id":2,"begin":2,"end":3,"order":order(weight)}])
        );
        assert_eq!(fragment["atoms"][0]["explicit_hydrogens"], 3);
        assert_eq!(fragment["atoms"][1]["explicit_hydrogens"], 3 - weight);
        assert_eq!(fragment["atoms"][2]["explicit_hydrogens"], 4 - weight);
        let (smiles, formula) = if weight == 2 {
            ("C=CC", "C3H6")
        } else {
            ("C#CC", "C3H4")
        };
        saved_files(
            &mut attached,
            &format!("attached-{}", order(weight)),
            &[3, u16::from(3 - weight), u16::from(4 - weight)],
            &[1, weight],
            smiles,
            formula,
        )?;
        let extended = attached.content().clone();
        attached.dispatch("history.undo", json!({}))?;
        assert_eq!(attached.content(), &old);
        assert!(attached.state().file.dirty);
        attached.dispatch("history.redo", json!({}))?;
        assert_eq!(attached.content(), &extended);
        assert!(!attached.state().file.dirty);

        // Failed branch attempts never permit reuse of IDs committed before Undo.
        engine.dispatch("history.undo", json!({}))?;
        engine.dispatch(
            "bond.create",
            create(Some(if weight == 2 { "triple" } else { "double" })),
        )?;
        let branch = serde_json::to_value(engine.document()?)?;
        let f = &branch["pages"][0]["fragments"][0];
        assert_eq!(f["id"], 2);
        assert_eq!(f["atoms"][0]["id"], 3);
        assert_eq!(f["atoms"][1]["id"], 4);
        assert_eq!(f["bonds"][0]["id"], 2);
    }
    Ok(())
}
