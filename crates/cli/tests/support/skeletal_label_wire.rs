//! Existing atom.set_label transaction through real CLI, initialized stdio and authenticated loopback.
use super::*;
use std::{path::Path, process::Command as Process};

fn sk_label(revision: u64) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":1,"element":6,"explicit_hydrogens":2})
}
fn sk_fragments(flag: bool) -> Value {
    json!([{"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":120.0,"y_pt":120.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2,"show_non_terminal_carbon_labels":flag},
        {"id":2,"element":6,"position":{"x_pt":72.0,"y_pt":168.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
        {"id":3,"element":6,"position":{"x_pt":168.0,"y_pt":168.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],
        "bonds":[{"id":1,"begin":2,"end":1,"order":"single"},{"id":2,"begin":1,"end":3,"order":"single"}]}])
}
fn sk_graph(state: &Value, flag: bool) {
    assert_eq!(
        state["document"]["pages"][0]["size"],
        json!({"width_pt":612.0,"height_pt":792.0})
    );
    assert_eq!(
        state["document"]["pages"][0]["fragments"],
        sk_fragments(flag)
    );
    assert!(state.get("collection").is_none() && state.get("smiles_file").is_none());
}
fn sk_schema(schema: &Value) -> TestResult {
    let fields = [
        "expected_revision",
        "page_id",
        "fragment_id",
        "atom_id",
        "element",
        "explicit_hydrogens",
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["properties"]
            .as_object()
            .ok_or("properties")?
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        fields
    );
    assert_eq!(
        schema["required"]
            .as_array()
            .ok_or("required")?
            .iter()
            .map(|v| v.as_str().ok_or("field"))
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?,
        fields
    );
    for name in fields {
        assert_eq!(schema["properties"][name]["type"], "integer");
    }
    assert_eq!(
        schema["properties"]["element"]["minimum"].as_f64(),
        Some(1.0)
    );
    assert_eq!(
        schema["properties"]["element"]["maximum"].as_f64(),
        Some(118.0)
    );
    assert_eq!(
        schema["properties"]["explicit_hydrogens"]["maximum"].as_f64(),
        Some(65535.0)
    );
    Ok(())
}

#[test]
fn cli_same_count_restoration_is_one_edit_with_exact_flags_history_unicode_source_and_saved_baseline()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("skeletal-cli-original-λ-化学.cdxml");
    let output = directory.path("skeletal-cli-restored-λ-化学.cdxml");
    fs::write(&input, SK_V)?;
    let listed = Process::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(listed.status.success());
    let registry: Value = serde_json::from_slice(&listed.stdout)?;
    let entries = registry.as_array().ok_or("registry")?;
    assert_eq!(entries.len(), 43);
    assert_eq!(
        entries
            .iter()
            .map(|v| v["name"].as_str().ok_or("name"))
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?
            .len(),
        43
    );
    let entry = entries
        .iter()
        .find(|v| v["name"] == "atom.set_label")
        .ok_or("atom label entry")?;
    assert_eq!(entry["read_only"], false);
    sk_schema(&entry["input_schema"])?;
    let opened = json!({"command":"document.open_cdxml","params":{"path":input}});
    let labeled = json!({"command":"atom.set_label","params":sk_label(1)});
    let changed = cli(&[opened.clone(), labeled.clone()])?;
    sk_graph(&changed, true);
    assert_eq!(changed["revision"], 2);
    assert_eq!(changed["undo_depth"], 1);
    assert_eq!(changed["redo_depth"], 0);
    assert_eq!(changed["file"]["dirty"], true);
    assert_eq!(
        changed["source"],
        json!({"format":"cdxml","byte_length":SK_V.len(),"modified":true})
    );
    let saved = cli(&[
        opened,
        labeled,
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":output}}),
        json!({"command":"document.open_cdxml","params":{"path":output}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    sk_graph(&saved, true);
    assert_eq!(saved["revision"], 5);
    assert_eq!(saved["undo_depth"], 0);
    assert_eq!(saved["redo_depth"], 0);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["file"]["format"], "cdxml");
    assert_eq!(saved["file"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(saved["source"]["modified"], false);
    assert_eq!(fs::read(&input)?, SK_V);
    sk_consume_file(&input, false)?;
    sk_consume_file(&output, true)?;
    Ok(())
}
fn sk_initialize(client: &mut Client) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"skeletal-label-test","version":"1"}}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], 1);
    assert!(reply.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
}
fn sk_stdio_reject(client: &mut Client, id: u64, params: Value, code: &str) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"atom.set_label","arguments":params}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], id);
    assert_eq!(reply["result"]["isError"], true);
    assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    Ok(())
}
fn sk_capture_same_state(captured: &Value, before: &Value) -> TestResult {
    assert_eq!(captured["changed"], false);
    assert_eq!(captured["capture"]["backend"], "cpu");
    assert_eq!(captured["capture"]["adapter"], Value::Null);
    assert_eq!(captured["capture"]["width_px"], 1024);
    assert_eq!(captured["capture"]["height_px"], 1024);
    assert!(
        !captured["capture"]["png_base64"]
            .as_str()
            .ok_or("actual PNG")?
            .is_empty()
    );
    let mut state = captured.clone();
    state
        .as_object_mut()
        .ok_or("capture object")?
        .remove("capture");
    assert_eq!(&state, before);
    Ok(())
}
#[test]
fn initialized_stdio_restores_then_preserves_validated_noop_stale_history_capture_and_actual_saved_data()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("skeletal-stdio-original-λ.cdxml");
    let output = directory.path("skeletal-stdio-restored-λ.cdxml");
    fs::write(&input, SK_V)?;
    let mut client = Client::start()?;
    sk_initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 43);
    let entry = tools
        .iter()
        .find(|v| v["name"] == "atom.set_label")
        .ok_or("label tool")?;
    assert_eq!(entry["annotations"]["readOnlyHint"], false);
    sk_schema(&entry["inputSchema"])?;
    let opened = client.call(3, "document.open_cdxml", json!({"path":input}))?;
    sk_graph(&opened, false);
    let changed = client.call(4, "atom.set_label", sk_label(1))?;
    assert_eq!(changed["changed"], true);
    assert_eq!(changed["revision"], 2);
    assert_eq!(changed["undo_depth"], 1);
    sk_graph(&changed, true);
    let before = client.call(5, "document.inspect", json!({}))?;
    assert_eq!(client.call(6, "atom.set_label", sk_label(2))?, before);
    sk_stdio_reject(&mut client, 7, sk_label(1), "stale_revision")?;
    let mut extra = sk_label(2);
    extra["show_non_terminal_carbon_labels"] = json!(false);
    sk_stdio_reject(&mut client, 8, extra, "invalid_params")?;
    assert_eq!(client.call(9, "document.inspect", json!({}))?, before);
    let undone = client.call(10, "history.undo", json!({}))?;
    sk_graph(&undone, false);
    assert_eq!(undone["revision"], 3);
    assert_eq!(undone["redo_depth"], 1);
    assert_eq!(undone["file"]["dirty"], false);
    let redone = client.call(11, "history.redo", json!({}))?;
    sk_graph(&redone, true);
    assert_eq!(redone["revision"], 4);
    let saved = client.call(12, "document.save_cdxml", json!({"path":output}))?;
    assert_eq!(saved["file"]["dirty"], false);
    let reopened = client.call(13, "document.open_cdxml", json!({"path":output}))?;
    sk_graph(&reopened, true);
    assert_eq!(reopened["revision"], 5);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["source"]["modified"], false);
    let before = client.call(14, "document.inspect", json!({}))?;
    let captured = client.call(
        15,
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
    )?;
    sk_capture_same_state(&captured, &before)?;
    assert_eq!(client.call(16, "document.inspect", json!({}))?, before);
    assert_eq!(fs::read(&input)?, SK_V);
    sk_consume_file(&input, false)?;
    sk_consume_file(&output, true)?;
    client.finish()
}
fn sk_socket(
    client: &mut BufReader<TcpStream>,
    name: &str,
    params: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let reply = socket(client, json!({"command":name,"params":params}))?;
    assert!(reply.get("error").is_none(), "{reply}");
    Ok(reply["result"].clone())
}
#[test]
fn authenticated_loopback_restores_once_without_new_authority_or_noop_error_capture_repaints()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("skeletal-socket-original-λ.cdxml");
    let output = directory.path("skeletal-socket-restored-λ.cdxml");
    let opaque = directory.path("skeletal-socket-opaque-λ.cdxml");
    fs::write(&input, SK_V)?;
    let opaque_bytes = String::from_utf8(SK_V.to_vec())?
        .replacen("<CDXML ", "<CDXML Future=\"retain\" ", 1)
        .into_bytes();
    fs::write(&opaque, &opaque_bytes)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        observed.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    // Credential stays solely on this trusted connection; no token enters a diagnostic value.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    sk_socket(&mut client, "document.open_cdxml", json!({"path":input}))?;
    let count = changes.load(Ordering::SeqCst);
    let changed = sk_socket(&mut client, "atom.set_label", sk_label(1))?;
    sk_graph(&changed, true);
    assert_eq!(changed["changed"], true);
    assert_eq!(changed["revision"], 2);
    assert_eq!(changed["undo_depth"], 1);
    assert_eq!(changes.load(Ordering::SeqCst), count + 1);
    let before = sk_socket(&mut client, "document.inspect", json!({}))?;
    let count = changes.load(Ordering::SeqCst);
    assert_eq!(
        sk_socket(&mut client, "atom.set_label", sk_label(2))?,
        before
    );
    let error = socket(
        &mut client,
        json!({"command":"atom.set_label","params":sk_label(1)}),
    )?;
    assert_eq!(error["error"]["code"], "stale_revision");
    let captured = sk_socket(
        &mut client,
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
    )?;
    sk_capture_same_state(&captured, &before)?;
    assert_eq!(
        sk_socket(&mut client, "document.inspect", json!({}))?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), count);
    sk_socket(&mut client, "document.save_cdxml", json!({"path":output}))?;
    let reopened = sk_socket(&mut client, "document.open_cdxml", json!({"path":output}))?;
    sk_graph(&reopened, true);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    sk_consume_file(&input, false)?;
    sk_consume_file(&output, true)?;
    let opened = sk_socket(&mut client, "document.open_cdxml", json!({"path":opaque}))?;
    sk_graph(&opened, false);
    let before = sk_socket(&mut client, "document.inspect", json!({}))?;
    let count = changes.load(Ordering::SeqCst);
    let error = socket(
        &mut client,
        json!({"command":"atom.set_label","params":sk_label(opened["revision"].as_u64().ok_or("revision")?)}),
    )?;
    assert_eq!(error["error"]["code"], "unsupported_content");
    assert_eq!(
        sk_socket(&mut client, "document.inspect", json!({}))?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), count);
    assert_eq!(fs::read(&input)?, SK_V);
    assert_eq!(fs::read(&opaque)?, opaque_bytes);
    server.shutdown()?;
    Ok(())
}

const SK_V: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="612" Height="792"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="120 120" NumHydrogens="2" ShowNonTerminalCarbonLabels="no"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="72 168" NumHydrogens="3"/><n id="5" NodeType="Element" Element="6" Charge="0" Radical="None" p="168 168" NumHydrogens="3"/><b id="6" B="4" E="3" Order="1"/><b id="7" B="3" E="5" Order="1"/></fragment></page></CDXML>"#;

fn sk_consume_file(path: &Path, flag: bool) -> TestResult {
    let output=Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON")?).args(["-B","-c",r#"
import decimal,json,subprocess,sys,xml.parsers.expat as expat
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=open(sys.argv[1],'rb').read(1048577)
assert 0<len(raw)<=1048576
flag='yes' if sys.argv[2]=='true' else 'no'
records=[]
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:records.append((name,attrs))
p.Parse(raw,True)
assert [n for n,_ in records]==['CDXML','page','fragment','n','n','n','b','b']
nodes=[a for n,a in records if n=='n'];bonds=[a for n,a in records if n=='b']
expected=[{'id':str(i),'NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':p,'NumHydrogens':h} for i,p,h in [(3,'120 120','2'),(4,'72 168','3'),(5,'168 168','3')]]
expected[0]['ShowNonTerminalCarbonLabels']=flag
assert nodes==expected,(nodes,expected)
assert bonds==[{'id':'6','B':'4','E':'3','Order':'1'},{'id':'7','B':'3','E':'5','Order':'1'}]
for a in nodes: assert all(decimal.Decimal(v).is_finite() for v in a['p'].split())
external=subprocess.run([sys.executable,'-B',sys.argv[3]],input=raw,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True)
assert json.loads(external.stdout)==[{'id':1,'atoms':[{'id':i,'element':6,'position':{'x_pt':x,'y_pt':y},'isotope':None,'charge':0,'radical':'none','explicit_hydrogens':h} for i,x,y,h in [(1,120.0,120.0,2),(2,72.0,168.0,3),(3,168.0,168.0,3)]],'bonds':[{'id':1,'begin':2,'end':1,'order':'single'},{'id':2,'begin':1,'end':3,'order':'single'}]}]
mols=Chem.MolsFromCDXML(raw.decode('utf8'),sanitize=True,removeHs=False)
assert len(mols)==1
m=mols[0]
assert m.GetNumAtoms()==3 and m.GetNumBonds()==2
assert [a.GetAtomicNum() for a in m.GetAtoms()]==[6,6,6]
assert [a.GetTotalNumHs(includeNeighbors=True) for a in m.GetAtoms()]==[2,3,3]
assert [(b.GetBeginAtomIdx(),b.GetEndAtomIdx(),str(b.GetBondType())) for b in m.GetBonds()]==[(1,0,'SINGLE'),(0,2,'SINGLE')]
assert all(a.GetFormalCharge()==0 and a.GetIsotope()==0 and a.GetNumRadicalElectrons()==0 and a.GetChiralTag()==Chem.ChiralType.CHI_UNSPECIFIED for a in m.GetAtoms())
assert all(b.GetStereo()==Chem.BondStereo.STEREONONE for b in m.GetBonds())
assert rdMolDescriptors.CalcMolFormula(m)=='C3H8' and Chem.MolToSmiles(m)=='CCC'
"#]).arg(path).arg(flag.to_string()).arg(concat!(env!("CARGO_MANIFEST_DIR"),"/../../xtask/oracles/cdxml_graph.py")).stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
    assert!(
        output.status.success(),
        "actual-file flag/Expat/Decimal/unchanged graph/RDKit checks: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
