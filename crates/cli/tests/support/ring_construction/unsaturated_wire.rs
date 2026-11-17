//! Independent real CLI, initialized stdio and authenticated socket named-ring composition.
//! Child of the unchanged ring_construction_wire.rs: reuse clients/handshake/timeouts.
use super::{
    Client, Engine, LoopbackServer, SharedEngine, TestResult, cli, compare_fragment, expected,
    initialize, rejected, socket, support,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    net::TcpStream,
    path::Path,
    process::Command as Process,
    sync::{Arc, Mutex},
    time::Duration,
};

fn named(revision: u64, kind: &str, placement: Value, phase: Option<&str>) -> Value {
    let mut value = json!({"expected_revision":revision,"page_id":1,"kind":kind,
        "hydrogen_policy":"rectify_neutral_carbon","placement":placement});
    if let Some(phase) = phase {
        value["phase"] = json!(phase);
    }
    value
}
fn free(revision: u64, kind: &str, phase: Option<&str>) -> Value {
    named(
        revision,
        kind,
        json!({"mode":"free","begin_x_pt":120,"begin_y_pt":168,
        "end_x_pt":120,"end_y_pt":120,"side":"positive"}),
        phase,
    )
}
fn create(params: Value) -> Value {
    json!({"command":"ring.create","params":params})
}
fn atom(id: usize, point: [f64; 2], h: u16, hidden: bool) -> Value {
    let mut value = json!({"id":id,"element":6,"position":{"x_pt":point[0],"y_pt":point[1]},
        "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":h});
    if hidden {
        value["show_non_terminal_carbon_labels"] = json!(false);
    }
    value
}
fn benzene() -> Value {
    // Reuse only the old independent analytic hexagon coordinates; never a product writer.
    let mut fragment = expected("free");
    for atom in fragment["atoms"].as_array_mut().into_iter().flatten() {
        atom["explicit_hydrogens"] = json!(1);
        atom["show_non_terminal_carbon_labels"] = json!(false);
    }
    fragment["bonds"] = json!([
        {"id":1,"begin":1,"end":2,"order":"double","double_position":"right"},
        {"id":2,"begin":2,"end":3,"order":"single"},
        {"id":3,"begin":3,"end":4,"order":"double","double_position":"right"},
        {"id":4,"begin":4,"end":5,"order":"single"},
        {"id":5,"begin":5,"end":6,"order":"double","double_position":"right"},
        {"id":6,"begin":6,"end":1,"order":"single"}]);
    fragment
}
fn cyclopentadiene_single_first() -> Value {
    // Closed circumcircle expectation, independent of the iterative product polygon builder.
    let radius = 24.0 / 36.0_f64.to_radians().sin();
    let center = [120.0 + 24.0 / 36.0_f64.to_radians().tan(), 144.0];
    let atoms = [1, 1, 1, 2, 1]
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            let angle = (144.0 + 72.0 * i as f64).to_radians();
            atom(
                i + 1,
                [
                    center[0] + radius * angle.cos(),
                    center[1] + radius * angle.sin(),
                ],
                h,
                true,
            )
        })
        .collect::<Vec<_>>();
    json!({"id":1,"atoms":atoms,"bonds":[
        {"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"double","double_position":"right"},
        {"id":3,"begin":3,"end":4,"order":"single"},
        {"id":4,"begin":4,"end":5,"order":"single"},
        {"id":5,"begin":5,"end":1,"order":"double","double_position":"right"}]})
}
fn toluene() -> Value {
    let mut atoms = vec![
        atom(1, [72.0, 168.0], 3, false),
        atom(2, [120.0, 168.0], 0, false),
    ];
    let s = 3.0_f64.sqrt();
    for (id, point) in [
        (3, [120.0, 120.0]),
        (4, [120.0 + 24.0 * s, 96.0]),
        (5, [120.0 + 48.0 * s, 120.0]),
        (6, [120.0 + 48.0 * s, 168.0]),
        (7, [120.0 + 24.0 * s, 192.0]),
    ] {
        atoms.push(atom(id, point, 1, true));
    }
    json!({"id":1,"atoms":atoms,"bonds":[
        {"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"double","double_position":"right"},
        {"id":3,"begin":3,"end":4,"order":"single"},
        {"id":4,"begin":4,"end":5,"order":"double","double_position":"right"},
        {"id":5,"begin":5,"end":6,"order":"single"},
        {"id":6,"begin":6,"end":7,"order":"double","double_position":"right"},
        {"id":7,"begin":7,"end":2,"order":"single"}]})
}
fn graph(value: &Value, wanted: &Value) -> TestResult {
    let fragments = value["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    compare_fragment(&fragments[0], wanted)
}
fn consume(path: &Path, wanted: &Value, notation: &str, formula: &str) -> TestResult {
    // New property-only check FIRST; old graph/chemistry consumers remain byte-exact.
    // No style is projected away until every actual flag/side/value/presence was checked.
    let checked = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .args(["-B","-c",r#"
import copy,json,subprocess,sys,xml.parsers.expat as expat
from pathlib import Path
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=Path(sys.argv[1]).read_bytes();assert 0<len(raw)<=1048576
expected=json.loads(sys.argv[2]);notation,formula=sys.argv[3:5]
objects=[]
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:objects.append((name,attrs))
p.Parse(raw,True)
ns=[a for n,a in objects if n=='n'];bs=[a for n,a in objects if n=='b']
assert [n for n,a in objects]==['CDXML','page','fragment']+['n']*len(expected['atoms'])+['b']*len(expected['bonds'])
assert len(ns)==len(expected['atoms']) and len(bs)==len(expected['bonds'])
for i,(a,e) in enumerate(zip(ns,expected['atoms'])):
 fields={'id','NodeType','Element','Charge','Radical','p','NumHydrogens'}
 if 'show_non_terminal_carbon_labels' in e:
  fields.add('ShowNonTerminalCarbonLabels')
  assert a['ShowNonTerminalCarbonLabels']==('yes' if e['show_non_terminal_carbon_labels'] else 'no')
 else: assert 'ShowNonTerminalCarbonLabels' not in a
 assert set(a)==fields and int(a['id'])==i+3
 assert a['NodeType']=='Element' and a['Element']=='6' and a['Charge']=='0' and a['Radical']=='None'
 assert int(a['NumHydrogens'])==e['explicit_hydrogens']
for i,(a,e) in enumerate(zip(bs,expected['bonds'])):
 fields={'id','B','E','Order'}
 if 'double_position' in e:
  fields.add('DoublePosition');assert a['DoublePosition']=={'left':'Left','right':'Right','center':'Center'}[e['double_position']]
 else: assert 'DoublePosition' not in a
 assert set(a)==fields and int(a['id'])==len(ns)+i+3
 assert [int(a['B'])-2,int(a['E'])-2]==[e['begin'],e['end']]
 assert a['Order']=={'single':'1','double':'2','triple':'3'}[e['order']]
# Explicit style checks are complete before the unchanged chemistry-only projection.
graph=json.loads(subprocess.run([sys.executable,'-B',sys.argv[5]],input=raw,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True).stdout)
assert len(graph)==1
options={'atoms':[[6,None,a['explicit_hydrogens']] for a in expected['atoms']],
         'bonds':[[b['begin'],b['end'],{'single':1,'double':2,'triple':3}[b['order']]] for b in expected['bonds']],
         'formula':formula}
verified=json.loads(subprocess.run([sys.executable,'-B',sys.argv[6],notation,json.dumps(options)],input=json.dumps(graph[0]).encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True).stdout)
ms=Chem.MolsFromCDXML(raw.decode('utf-8'),sanitize=True,removeHs=False);assert len(ms)==1
m=ms[0];reference=Chem.MolFromSmiles(notation);assert reference is not None
assert m.GetNumAtoms()==len(ns) and m.GetNumBonds()==len(bs)
assert [a.GetTotalNumHs(includeNeighbors=True) for a in m.GetAtoms()]==[a['explicit_hydrogens'] for a in expected['atoms']]
assert all(a.GetAtomicNum()==6 and a.GetFormalCharge()==a.GetIsotope()==a.GetNumRadicalElectrons()==0 for a in m.GetAtoms())
assert rdMolDescriptors.CalcMolFormula(m)==formula and Chem.MolToSmiles(m)==Chem.MolToSmiles(reference)
print(json.dumps({'graph':graph[0],'formula':formula,'chemistry':verified}))
"#]).arg(path).arg(wanted.to_string()).arg(notation).arg(formula)
        .arg(concat!(env!("CARGO_MANIFEST_DIR"),"/../../xtask/oracles/cdxml_graph.py"))
        .arg(concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/smiles_authoring_oracle.py"))
        .output()?;
    assert!(
        checked.status.success(),
        "actual file/style/unchanged consumers: {}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let result: Value = serde_json::from_slice(&checked.stdout)?;
    let mut chemical = wanted.clone();
    for atom in chemical["atoms"].as_array_mut().ok_or("expected atoms")? {
        atom.as_object_mut()
            .ok_or("expected atom")?
            .remove("show_non_terminal_carbon_labels");
    }
    for bond in chemical["bonds"].as_array_mut().ok_or("expected bonds")? {
        bond.as_object_mut()
            .ok_or("expected bond")?
            .remove("double_position");
    }
    compare_fragment(&result["graph"], &chemical)?;
    assert_eq!(result["formula"], formula);
    Ok(())
}

#[test]
fn cli_named_benzene_history_unicode_save_open_and_same_order_noop_are_real_commands() -> TestResult
{
    let directory = support::SdfTestDirectory::new()?;
    let path = directory.path("benzene-cli-化学-λ.cdxml");
    let created = cli(&[create(free(0, "benzene", None))])?;
    graph(&created, &benzene())?;
    assert_eq!(created["changed"], true);
    assert_eq!(created["revision"], 1);
    assert_eq!(created["undo_depth"], 1);
    assert_eq!(created["file"]["dirty"], true);
    assert!(created.get("conversion").is_none() && created.get("file_operation").is_none());
    let saved = cli(&[
        create(free(0, "benzene", None)),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":path}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    graph(&saved, &benzene())?;
    assert_eq!(saved["revision"], 3);
    assert_eq!(saved["undo_depth"], 1);
    assert_eq!(saved["redo_depth"], 0);
    assert_eq!(saved["file"]["dirty"], false);
    consume(&path, &benzene(), "c1ccccc1", "C6H6")?;
    let bytes = fs::read(&path)?;
    let reopened = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":path}}),
        json!({"command":"bond.set_order","params":{"expected_revision":1,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    graph(&reopened, &benzene())?;
    assert_eq!(reopened["revision"], 1);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["redo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["source"]["modified"], false);
    assert_eq!(fs::read(&path)?, bytes);
    Ok(())
}

#[test]
fn initialized_stdio_named_cp_single_first_closed_errors_history_and_saved_file_are_literal()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let path = directory.path("cyclopentadiene-stdio-保存.cdxml");
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(
        tools
            .iter()
            .find(|tool| tool["name"] == "ring.create")
            .ok_or("ring tool")?["annotations"]["readOnlyHint"],
        false
    );
    let created = client.call(
        3,
        "ring.create",
        free(0, "cyclopentadiene", Some("single_first")),
    )?;
    graph(&created, &cyclopentadiene_single_first())?;
    assert_eq!(created["revision"], 1);
    assert_eq!(created["undo_depth"], 1);
    assert!(created.get("file_operation").is_none());
    let stable = client.call(4, "document.inspect", json!({}))?;
    let mut invalid = named(
        1,
        "cyclopentadiene",
        json!({"mode":"free","begin_x_pt":360,"begin_y_pt":300,"end_x_pt":408,"end_y_pt":300,"side":"positive"}),
        None,
    );
    invalid["size"] = json!(5);
    rejected(&mut client, 5, invalid, "invalid_params")?;
    assert_eq!(client.call(6, "document.inspect", json!({}))?, stable);
    let stale = named(
        0,
        "benzene",
        json!({"mode":"free","begin_x_pt":360,"begin_y_pt":300,"end_x_pt":408,"end_y_pt":300,"side":"positive"}),
        None,
    );
    rejected(&mut client, 7, stale, "stale_revision")?;
    assert_eq!(client.call(8, "document.inspect", json!({}))?, stable);
    let saved = client.call(9, "document.save_cdxml", json!({"path":path}))?;
    assert_eq!(saved["file"]["dirty"], false);
    consume(&path, &cyclopentadiene_single_first(), "C1C=CCC=1", "C5H6")?;
    let bytes = fs::read(&path)?;
    let undo = client.call(10, "history.undo", json!({}))?;
    assert_eq!(undo["document"]["pages"][0]["fragments"], json!([]));
    assert_eq!(undo["file"]["dirty"], true);
    let redo = client.call(11, "history.redo", json!({}))?;
    graph(&redo, &cyclopentadiene_single_first())?;
    assert_eq!(redo["revision"], 3);
    assert_eq!(redo["undo_depth"], 1);
    assert_eq!(redo["redo_depth"], 0);
    assert_eq!(redo["file"]["dirty"], false);
    let opened = client.call(12, "document.open_cdxml", json!({"path":path}))?;
    graph(&opened, &cyclopentadiene_single_first())?;
    assert_eq!(opened["undo_depth"], 0);
    assert_eq!(opened["file"]["dirty"], false);
    assert_eq!(opened["source"]["modified"], false);
    assert_eq!(fs::read(&path)?, bytes);
    client.finish()
}

#[test]
fn authenticated_named_atom_attachment_retains_old_source_and_opaque_refusal_is_atomic()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("ethane-socket-原.cdxml");
    let output = directory.path("toluene-socket-保存.cdxml");
    let opaque = directory.path("opaque-named-ring-原.cdxml");
    let copy = directory.path("opaque-named-ring-保存.cdxml");
    let raw = String::from_utf8(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/fixtures/bondline-v0.cdxml"
        ))
        .to_vec(),
    )?
    .replace("<CDXML ", "<CDXML Private=\"named-ring-whole-source\" ");
    fs::write(&opaque, raw.as_bytes())?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let server = LoopbackServer::bind(0, Arc::clone(&engine), || {})?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let seeded = socket(
        &mut client,
        json!({"command":"bond.create","params":{"page_id":1,"begin_x_pt":72,"begin_y_pt":168,"end_x_pt":120,"end_y_pt":168}}),
    )?;
    assert!(seeded.get("error").is_none(), "{seeded}");
    let saved = socket(
        &mut client,
        json!({"command":"document.save_cdxml","params":{"path":input}}),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    let opened = socket(
        &mut client,
        json!({"command":"document.open_cdxml","params":{"path":input}}),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    let original = fs::read(&input)?;
    let old = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    let attached = socket(
        &mut client,
        create(named(
            2,
            "benzene",
            json!({"mode":"atom","fragment_id":1,"atom_id":2,"end_x_pt":120,"end_y_pt":120,"side":"positive"}),
            None,
        )),
    )?;
    assert!(attached.get("error").is_none(), "{attached}");
    let attached = &attached["result"];
    graph(attached, &toluene())?;
    assert_eq!(attached["revision"], 3);
    assert_eq!(attached["undo_depth"], 1);
    assert_eq!(attached["file"]["dirty"], true);
    assert_eq!(
        attached["source"],
        json!({"format":"cdxml","byte_length":original.len(),"modified":true})
    );
    assert_eq!(
        &attached["document"]["pages"][0]["fragments"][0]["bonds"][0],
        &old["document"]["pages"][0]["fragments"][0]["bonds"][0]
    );
    let saved = socket(
        &mut client,
        json!({"command":"document.save_cdxml","params":{"path":output}}),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    assert_eq!(saved["result"]["file"]["dirty"], false);
    consume(&output, &toluene(), "Cc1ccccc1", "C7H8")?;
    let undo = socket(&mut client, json!({"command":"history.undo","params":{}}))?;
    assert_eq!(undo["result"]["document"], old["document"]);
    assert_eq!(undo["result"]["source"]["modified"], false);
    let redo = socket(&mut client, json!({"command":"history.redo","params":{}}))?;
    graph(&redo["result"], &toluene())?;
    assert_eq!(redo["result"]["revision"], 5);
    assert_eq!(redo["result"]["file"]["dirty"], false);
    assert_eq!(fs::read(&input)?, original);
    let opened = socket(
        &mut client,
        json!({"command":"document.open_cdxml","params":{"path":output}}),
    )?;
    graph(&opened["result"], &toluene())?;
    assert_eq!(opened["result"]["undo_depth"], 0);
    assert_eq!(opened["result"]["source"]["modified"], false);

    let opened = socket(
        &mut client,
        json!({"command":"document.open_cdxml","params":{"path":opaque}}),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    let (whole, stamp) = {
        let e = engine.lock().map_err(|_| "engine")?;
        (e.content().clone(), e.capture_stamp())
    };
    let refused = socket(
        &mut client,
        create(free(
            before["revision"].as_u64().ok_or("revision")?,
            "cyclopentadiene",
            None,
        )),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?["result"],
        before
    );
    {
        let e = engine.lock().map_err(|_| "engine")?;
        assert_eq!(e.content(), &whole);
        assert!(e.capture_stamp_matches(&stamp));
    }
    let copied = socket(
        &mut client,
        json!({"command":"document.save_cdxml","params":{"path":copy}}),
    )?;
    assert!(copied.get("error").is_none(), "{copied}");
    assert_eq!(fs::read(&opaque)?, raw.as_bytes());
    assert_eq!(fs::read(&copy)?, raw.as_bytes());
    server.shutdown()?;
    Ok(())
}

#[path = "chair_wire.rs"]
mod chair_wire;

#[path = "sprout_wire.rs"]
mod sprout_wire;
