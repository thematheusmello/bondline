//! Independent real-transport authoring acceptance, separate from file conversion.

use serde_json::{Value, json};
use std::process::Command;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn executable_pastes_a_complete_editable_benzene_through_the_shared_registry() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("run")
        .arg("--cmd")
        .arg(
            json!({"command":"structure.paste_smiles","params":{
                "expected_revision":0,"page_id":1,"notation":"c1ccccc1",
                "bond_length_pt":64.0,"origin_x_pt":96.0,"origin_y_pt":96.0
            }})
            .to_string(),
        )
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    let fragments = result["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 6);
    assert!(
        atoms
            .iter()
            .all(|a| a["element"] == 6 && a["explicit_hydrogens"] == 1)
    );
    assert_eq!(fragments[0]["bonds"].as_array().ok_or("bonds")?.len(), 6);
    assert_eq!(
        (result["revision"].as_u64(), result["undo_depth"].as_u64()),
        (Some(1), Some(1))
    );
    assert_eq!(result["file"]["dirty"], true);
    assert!(result.get("conversion").is_none());
    assert!(result.get("file_operation").is_none());
    Ok(())
}

use super::{Client, cli, socket, support};
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use std::{
    fs,
    io::{BufReader, Write as _},
    net::TcpStream,
    path::Path,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};

const NAME: &str = "structure.paste_smiles";

fn paste(revision: u64, notation: &str) -> Value {
    json!({"expected_revision":revision,"page_id":1,"notation":notation,
        "bond_length_pt":64.0,"origin_x_pt":96.0,"origin_y_pt":96.0})
}

fn create() -> Value {
    json!({"page_id":1,"begin_x_pt":72.0,"begin_y_pt":480.0,"end_x_pt":120.0,"end_y_pt":480.0})
}

fn fragments(state: &Value) -> TestResult<&Vec<Value>> {
    Ok(state["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?)
}

fn assert_pasted(state: &Value, old: &Value, edited: bool) -> TestResult {
    let all = fragments(state)?;
    assert_eq!(all.len(), 2);
    assert_eq!(all.first().ok_or("old fragment")?, old);
    let pasted = all.get(1).ok_or("pasted fragment")?;
    assert_eq!(pasted["id"], 2);
    let atoms = pasted["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 6);
    for (i, atom) in atoms.iter().enumerate() {
        assert_eq!(atom["id"], u32::try_from(i + 3)?);
        assert_eq!(atom["element"], 6);
        assert_eq!(
            atom["explicit_hydrogens"],
            if edited && i < 2 { 2 } else { 1 }
        );
        assert!(atom["position"].is_object());
    }
    let edges = pasted["bonds"].as_array().ok_or("bonds")?;
    assert_eq!(edges.len(), 6);
    for (i, (edge, (begin, end, order))) in edges
        .iter()
        .zip([
            (3, 4, if edited { "single" } else { "double" }),
            (4, 5, "single"),
            (5, 6, "double"),
            (6, 7, "single"),
            (7, 8, "double"),
            (3, 8, "single"),
        ])
        .enumerate()
    {
        assert_eq!(
            (
                edge["id"].clone(),
                edge["begin"].clone(),
                edge["end"].clone(),
                edge["order"].clone()
            ),
            (json!(i + 2), json!(begin), json!(end), json!(order))
        );
    }
    assert!(state.get("conversion").is_none());
    Ok(())
}

fn same_state(actual: &Value, expected: &Value) {
    for key in [
        "document",
        "source",
        "file",
        "revision",
        "undo_depth",
        "redo_depth",
        "close",
        "close_epoch",
        "ui_file",
    ] {
        assert_eq!(actual[key], expected[key], "unchanged {key}");
    }
}

fn consume(path: &Path, state: &Value, edited: bool) -> TestResult {
    // Independent Expat compares actual saved XML with the inspected complete graph;
    // actual pinned RDKit checks both saved molecules. This does not reserialize our model.
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let mut child=Command::new(&python).args(["-B","-c",r#"
import json,sys,xml.parsers.expat
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=open(sys.argv[1],'rb').read(1048577)
assert 0<len(raw)<=1048576
expected=json.load(sys.stdin)['document']['pages'][0]['fragments']
fragments=[];stack=[];ids=set()
p=xml.parsers.expat.ParserCreate()
p.SetParamEntityParsing(xml.parsers.expat.XML_PARAM_ENTITY_PARSING_NEVER)
def start(name,a):
 if 'id' in a:
  assert a['id'] not in ids;ids.add(a['id'])
 if name=='fragment':
  assert stack[-1]=='page';fragments.append({'atoms':[],'bonds':[]})
 elif name=='n':
  assert stack[-1]=='fragment';fragments[-1]['atoms'].append(a)
 elif name=='b':
  assert stack[-1]=='fragment';fragments[-1]['bonds'].append(a)
 stack.append(name)
def end(name): assert stack.pop()==name
p.StartElementHandler=start;p.EndElementHandler=end;p.Parse(raw,True)
assert len(fragments)==len(expected)==2
for actual,model in zip(fragments,expected):
 assert len(actual['atoms'])==len(model['atoms'])
 assert len(actual['bonds'])==len(model['bonds'])
 source_ids={a['id']:i for i,a in enumerate(actual['atoms'])}
 model_ids={a['id']:i for i,a in enumerate(model['atoms'])}
 for a,m in zip(actual['atoms'],model['atoms']):
  assert int(a['Element'])==m['element']==6
  assert int(a['NumHydrogens'])==m['explicit_hydrogens']
  assert int(a['Charge'])==m['charge']==0
  assert a['Radical'].lower()==m['radical']=='none'
  assert 'Isotope' not in a and m['isotope'] is None
  position=list(map(float,a['p'].split()));assert len(position)==2
  assert all(abs(x-y)<1e-9 for x,y in zip(position,[m['position']['x_pt'],m['position']['y_pt']]))
 for a,m in zip(actual['bonds'],model['bonds']):
  assert [source_ids[a['B']],source_ids[a['E']]]==[model_ids[m['begin']],model_ids[m['end']]]
  assert int(a['Order'])=={'single':1,'double':2}[m['order']]
molecules=Chem.MolsFromCDXML(raw.decode('utf8'))
assert len(molecules)==2
assert [m.GetNumAtoms() for m in molecules]==[2,6]
assert [m.GetNumBonds() for m in molecules]==[1,6]
formula=['C2H6','C6H8' if sys.argv[2]=='edited' else 'C6H6']
assert [rdMolDescriptors.CalcMolFormula(m) for m in molecules]==formula
assert all(a.GetFormalCharge()==a.GetNumRadicalElectrons()==0 and a.GetChiralTag()==Chem.ChiralType.CHI_UNSPECIFIED for m in molecules for a in m.GetAtoms())
print(json.dumps(formula))
"#]).arg(path).arg(if edited {"edited"} else {"original"})
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child
        .stdin
        .take()
        .ok_or("XML consumer stdin")?
        .write_all(&serde_json::to_vec(state)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let formula = if edited { "C6H8" } else { "C6H6" };
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)?,
        json!(["C2H6", formula])
    );
    let mut atoms = vec![json!([6, null, 1]); 6];
    if edited {
        *atoms.get_mut(0).ok_or("H1")? = json!([6, null, 2]);
        *atoms.get_mut(1).ok_or("H2")? = json!([6, null, 2]);
    }
    let options = json!({"atoms":atoms,"bonds":[[1,2,if edited {1}else{2}],[2,3,1],[3,4,2],[4,5,1],[5,6,2],[1,6,1]],
        "formula":formula,"components":[[1,2,3,4,5,6]]});
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_authoring_oracle.py"
        ))
        .arg(if edited { "C1CC=CC=C1" } else { "c1ccccc1" })
        .arg(options.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("authoring consumer stdin")?
        .write_all(&serde_json::to_vec(
            fragments(state)?.get(1).ok_or("pasted fragment")?,
        )?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let verified: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(verified["formula"], formula);
    assert_eq!(verified["no_assigned_stereo"], true);
    assert_eq!(verified["no_radicals"], true);
    Ok(())
}

#[test]
fn real_cli_and_stdio_append_history_edit_and_unicode_persistence_share_one_command() -> TestResult
{
    let directory = support::SdfTestDirectory::new()?;
    let source = directory.path("seed-λ.cdxml");
    let cli_output = directory.path("cli-pasted-化学.cdxml");
    let seed = cli(&[
        json!({"command":"bond.create","params":create()}),
        json!({"command":"document.save_cdxml","params":{"path":source}}),
    ])?;
    let original = fs::read(&source)?;
    let old = fragments(&seed)?.first().ok_or("seed")?.clone();
    let result = cli(&[
        json!({"command":"bond.create","params":create()}),
        json!({"command":NAME,"params":paste(1,"c1ccccc1")}),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":cli_output}}),
        json!({"command":"document.open_cdxml","params":{"path":cli_output}}),
    ])?;
    assert_pasted(&result, &old, false)?;
    assert_eq!(result["file"]["dirty"], false);
    consume(&cli_output, &result, false)?;

    let mut client = Client::start()?;
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"paste-authoring","version":"1"}}}))?;
    assert_eq!(
        client.replies.recv_timeout(Duration::from_secs(10))??["id"],
        1
    );
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    assert_eq!(tools.len(), 42);
    let tool = tools
        .iter()
        .find(|t| t["name"] == NAME)
        .ok_or("paste tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    let mut required = tool["inputSchema"]["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or("field"))
        .collect::<Result<Vec<_>, _>>()?;
    required.sort_unstable();
    assert_eq!(
        required,
        [
            "bond_length_pt",
            "expected_revision",
            "notation",
            "origin_x_pt",
            "origin_y_pt",
            "page_id"
        ]
    );
    client.call(3, "document.open_cdxml", json!({"path":source}))?;
    let before = client.call(4, "document.inspect", json!({}))?;
    let pasted = client.call(5, NAME, paste(1, "c1ccccc1"))?;
    assert_pasted(&pasted, &old, false)?;
    assert_eq!(
        (pasted["revision"].as_u64(), pasted["undo_depth"].as_u64()),
        (Some(2), Some(1))
    );
    assert_eq!(pasted["file"]["path"], before["file"]["path"]);
    assert_eq!(pasted["source"]["byte_length"], original.len());
    assert_eq!(pasted["source"]["modified"], true);
    let undone = client.call(6, "history.undo", json!({}))?;
    assert_eq!(undone["document"], before["document"]);
    assert_eq!(undone["file"]["dirty"], false);
    let restored = client.call(7, "history.redo", json!({}))?;
    assert_eq!(restored["document"], pasted["document"]);
    for (id, args, code) in [
        (8, paste(1, "c1ccccc1"), "stale_revision"),
        (9, paste(4, "[NH4+]"), "unsupported_content"),
    ] {
        client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":NAME,"arguments":args}}))?;
        let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
        assert_eq!(reply["id"], id);
        assert_eq!(reply["result"]["isError"], true);
        assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    }
    same_state(&client.call(10, "document.inspect", json!({}))?, &restored);
    let edited = client.call(
        11,
        "bond.set_order",
        json!({"expected_revision":4,"page_id":1,"fragment_id":2,"bond_id":2,
        "order":"single","hydrogen_policy":"rectify_neutral_carbon"}),
    )?;
    assert_pasted(&edited, &old, true)?;
    let output = directory.path("stdio-diene-化学.cdxml");
    let saved = client.call(12, "document.save_cdxml", json!({"path":output}))?;
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["file_operation"]["input_revision"], 5);
    consume(&output, &saved, true)?;
    let reopened = client.call(13, "document.open_cdxml", json!({"path":output}))?;
    assert_pasted(&reopened, &old, true)?;
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(fs::read(&source)?, original);
    consume(&output, &reopened, true)?;
    client.finish()
}

#[test]
fn authenticated_paste_keeps_history_close_and_complete_foreign_source_guards() -> TestResult {
    let directory = support::SdfTestDirectory::new()?;
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
    let created = socket(
        &mut client,
        json!({"command":"bond.create","params":create()}),
    )?;
    assert!(created.get("error").is_none(), "{created}");
    let old = fragments(&created["result"])?.first().ok_or("old")?.clone();
    let pasted = socket(
        &mut client,
        json!({"command":NAME,"params":paste(1,"c1ccccc1")}),
    )?;
    assert!(pasted.get("error").is_none(), "{pasted}");
    assert_pasted(&pasted["result"], &old, false)?;
    assert_eq!(pasted["result"]["undo_depth"], 2);
    let requested = socket(
        &mut client,
        json!({"command":"document.close","params":{"expected_revision":2,"action":"request"}}),
    )?;
    assert_eq!(requested["result"]["close"], "needs_decision");
    let rejected = socket(
        &mut client,
        json!({"command":NAME,"params":paste(2,"c1ccccc1")}),
    )?;
    assert!(
        rejected.get("error").is_some(),
        "pending-close paste must refuse"
    );
    let after = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    same_state(&after["result"], &requested["result"]);
    socket(
        &mut client,
        json!({"command":"document.close","params":{"expected_revision":2,"action":"cancel"}}),
    )?;
    let undone = socket(&mut client, json!({"command":"history.undo","params":{}}))?;
    assert_eq!(fragments(&undone["result"])?, &vec![old.clone()]);
    let restored = socket(&mut client, json!({"command":"history.redo","params":{}}))?;
    assert_eq!(restored["result"]["document"], pasted["result"]["document"]);
    let output = directory.path("socket-paste-λ.cdxml");
    let saved = socket(
        &mut client,
        json!({"command":"document.save_cdxml","params":{"path":output}}),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    consume(&output, &saved["result"], false)?;
    let whole = b"CC PRIVATE_FIRST\r\nO\tPRIVATE_SECOND\xff\0\n";
    let source = directory.path("whole-λ.smi");
    let copy = directory.path("whole-copy-化学.smi");
    fs::write(&source, whole)?;
    let opened = socket(
        &mut client,
        json!({"command":"document.open_smiles","params":{"path":source}}),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    assert_eq!(before["smiles_file"]["record_count"], 2);
    assert!(!before.to_string().contains("PRIVATE_"));
    let revision = before["revision"].as_u64().ok_or("revision")?;
    let rejected = socket(
        &mut client,
        json!({"command":NAME,"params":paste(revision,"c1ccccc1")}),
    )?;
    assert_eq!(rejected["error"]["code"], "unsupported_content");
    let after = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    same_state(&after["result"], &before);
    let saved = socket(
        &mut client,
        json!({"command":"document.save_smiles","params":{"path":copy}}),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    assert_eq!(fs::read(&copy)?, whole);
    assert_eq!(fs::read(&source)?, whole);
    drop(client);
    server.shutdown()?;
    Ok(())
}
