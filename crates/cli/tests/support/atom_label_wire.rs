//! Independent existing-atom label authoring over the real adapter host helpers.
use super::*;

const NAME: &str = "atom.set_label";

#[test]
fn cli_atom_label_replaces_only_the_existing_element_and_literal_h() -> TestResult {
    let result = cli(&[
        json!({"command":"bond.create","params":{"page_id":1,"begin_x_pt":72.0,
            "begin_y_pt":72.0,"end_x_pt":120.0,"end_y_pt":72.0}}),
        json!({"command":NAME,"params":{"expected_revision":1,"page_id":1,
            "fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}}),
    ])?;
    assert_eq!(result["revision"], 2);
    assert_eq!(result["undo_depth"], 2);
    assert_eq!(result["file"]["dirty"], true);
    assert_eq!(
        result["document"]["pages"][0]["fragments"],
        json!([{"id":1,
        "atoms":[
            {"id":1,"element":6,"position":{"x_pt":72.0,"y_pt":72.0},"isotope":null,
             "charge":0,"radical":"none","explicit_hydrogens":3},
            {"id":2,"element":8,"position":{"x_pt":120.0,"y_pt":72.0},"isotope":null,
             "charge":0,"radical":"none","explicit_hydrogens":1}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]}])
    );
    Ok(())
}

// Additive declarations preserve the frozen1,189-byte actualCLI dispatch case.
type AnyResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const OLD_NAMES: [&str; 32] = [
    "document.inspect",
    "document.new",
    "document.close",
    "ui.file",
    "ui.controls",
    "ui.view",
    "ui.bond_tool",
    "document.open_cdxml",
    "document.save_cdxml",
    "document.open_cdx",
    "document.save_cdx",
    "document.open_mol",
    "document.save_mol",
    "document.open_sdf",
    "document.save_sdf",
    "document.open_smiles",
    "document.import_smiles_records",
    "document.save_smiles",
    "file.convert_smiles_to_cdxml",
    "file.export_smiles_fragment",
    "sdf.record.inspect",
    "smiles.record.inspect",
    "structure.paste_smiles",
    "sdf.field.set_value",
    "sdf.record.remove",
    "page.set_size",
    "bond.create",
    "bond.extend",
    "bond.set_order",
    "history.undo",
    "history.redo",
    "render.capture",
];
fn create() -> Value {
    json!({"page_id":1,"begin_x_pt":72.0,"begin_y_pt":72.0,
        "end_x_pt":120.0,"end_y_pt":72.0})
}
fn label(revision: u64, element: u8, h: u16) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":2,
        "element":element,"explicit_hydrogens":h})
}
fn pair(element: u8, h: u16) -> Value {
    json!({"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":72.0,"y_pt":72.0},"isotope":null,
         "charge":0,"radical":"none","explicit_hydrogens":3},
        {"id":2,"element":element,"position":{"x_pt":120.0,"y_pt":72.0},"isotope":null,
         "charge":0,"radical":"none","explicit_hydrogens":h}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]})
}
fn graph(state: &Value, element: u8, h: u16) {
    assert_eq!(
        state["document"]["pages"][0]["fragments"],
        json!([pair(element, h)])
    );
    assert!(state.get("collection").is_none());
    assert!(state.get("smiles_file").is_none());
}
fn names(values: &[Value]) -> TestResult {
    let actual = values
        .iter()
        .map(|v| v["name"].as_str().ok_or("command name"))
        .collect::<Result<Vec<_>, _>>()?;
    let unique = actual
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let expected = OLD_NAMES
        .into_iter()
        .chain([NAME, "ui.atom_label", "ui.selection", "selection.translate"])
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(values.len(), 36);
    assert_eq!(unique.len(), 36);
    assert_eq!(unique, expected);
    Ok(())
}
fn schema(value: &Value) -> TestResult {
    assert_eq!(value["additionalProperties"], false);
    let required = value["required"]
        .as_array()
        .ok_or("required fields")?
        .iter()
        .map(|v| v.as_str().ok_or("required name"))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
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
    assert_eq!(required, fields);
    let properties = value["properties"].as_object().ok_or("properties")?;
    assert_eq!(
        properties
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        fields
    );
    for field in fields {
        assert_eq!(properties[field]["type"], "integer");
    }
    for field in ["page_id", "fragment_id", "atom_id", "element"] {
        assert_eq!(properties[field]["minimum"].as_f64(), Some(1.0));
    }
    assert_eq!(properties["element"]["maximum"].as_f64(), Some(118.0));
    assert_eq!(
        properties["explicit_hydrogens"]["minimum"].as_f64(),
        Some(0.0)
    );
    assert_eq!(
        properties["explicit_hydrogens"]["maximum"].as_f64(),
        Some(65535.0)
    );
    Ok(())
}
fn consume_saved(
    path: &std::path::Path,
    element: u8,
    h: u16,
    formula: &str,
    smiles: &str,
) -> TestResult {
    let raw = fs::read(path)?;
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    // The existing Expat/Decimal consumer reads actual output, compared with literal IDs/positions/H.
    let mut child = Command::new(&python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/cdxml_graph.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("Expat stdin")?.write_all(&raw)?;
    let parsed = child.wait_with_output()?;
    assert!(
        parsed.status.success(),
        "{}",
        String::from_utf8_lossy(&parsed.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&parsed.stdout)?,
        json!([pair(element, h)])
    );
    // Original independent actual-file chemistry check; no Bondline re-encoding or old oracle change.
    let checked = Command::new(&python).args(["-B","-c",r#"
import json,sys
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=open(sys.argv[1],'rb').read(1048577)
assert 0<len(raw)<=1048576
molecules=Chem.MolsFromCDXML(raw.decode('utf8'))
assert len(molecules)==1
m=molecules[0]
assert m.GetNumAtoms()==2 and m.GetNumBonds()==1
assert [a.GetAtomicNum() for a in m.GetAtoms()]==[6,int(sys.argv[2])]
assert [a.GetTotalNumHs(includeNeighbors=True) for a in m.GetAtoms()]==[3,int(sys.argv[3])]
assert [(b.GetBeginAtomIdx(),b.GetEndAtomIdx(),str(b.GetBondType())) for b in m.GetBonds()]==[(0,1,'SINGLE')]
assert all(a.GetFormalCharge()==0 and a.GetNumRadicalElectrons()==0 and a.GetIsotope()==0 and not a.GetIsAromatic() and a.GetChiralTag()==Chem.ChiralType.CHI_UNSPECIFIED for a in m.GetAtoms())
assert all(b.GetStereo()==Chem.BondStereo.STEREONONE for b in m.GetBonds())
formula=rdMolDescriptors.CalcMolFormula(m)
smiles=Chem.MolToSmiles(m)
assert formula==sys.argv[4] and smiles==sys.argv[5]
print(json.dumps({'formula':formula,'smiles':smiles,'atoms':2,'bonds':1}))
"#]).arg(path).arg(element.to_string()).arg(h.to_string()).arg(formula).arg(smiles)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&checked.stdout)?,
        json!({"formula":formula,"smiles":smiles,"atoms":2,"bonds":1})
    );
    Ok(())
}

#[test]
fn cli_lists_only_one_new_closed_command_and_preserves_native_source_through_label_history()
-> TestResult {
    let listed = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(listed.status.success());
    let registry: Value = serde_json::from_slice(&listed.stdout)?;
    let entries = registry.as_array().ok_or("registry")?;
    names(entries)?;
    let command = entries
        .iter()
        .find(|v| v["name"] == NAME)
        .ok_or("atom label command")?;
    assert_eq!(command["read_only"], false);
    schema(&command["input_schema"])?;
    let dir = support::SdfTestDirectory::new()?;
    let methanol = dir.path("atom-cli-methanol-λ-化学.cdxml");
    let amine = dir.path("atom-cli-methylamine-λ-化学.cdxml");
    let state = cli(&[
        json!({"command":"bond.create","params":create()}),
        json!({"command":NAME,"params":label(1,8,1)}),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":methanol}}),
        json!({"command":"document.open_cdxml","params":{"path":methanol}}),
        json!({"command":NAME,"params":label(5,7,2)}),
        json!({"command":"document.save_cdxml","params":{"path":amine}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    graph(&state, 7, 2);
    assert_eq!(state["revision"], 6);
    assert_eq!(state["undo_depth"], 1);
    assert_eq!(state["redo_depth"], 0);
    assert_eq!(state["file"]["dirty"], false);
    assert_eq!(state["file"]["format"], "cdxml");
    assert_eq!(state["file"]["path"], json!(fs::canonicalize(&amine)?));
    assert_eq!(
        state["source"],
        json!({"format":"cdxml","byte_length":fs::metadata(&methanol)?.len(),"modified":true})
    );
    assert!(state.get("file_operation").is_none());
    consume_saved(&methanol, 8, 1, "CH4O", "CO")?;
    consume_saved(&amine, 7, 2, "CH5N", "CN")
}

fn initialize(client: &mut Client) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"atom-label-wire","version":"1"}}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], 1);
    assert!(reply.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
}
fn rejected(client: &mut Client, id: u64, args: Value, code: &str) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":NAME,"arguments":args}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], id);
    assert_eq!(reply["result"]["isError"], true);
    assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    Ok(())
}

#[test]
fn initialized_stdio_exposes_the_typed_label_and_keeps_noop_error_and_saved_baselines_exact()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    let methanol = dir.path("atom-stdio-λ-甲醇.cdxml");
    let amine = dir.path("atom-stdio-λ-甲胺.cdxml");
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    names(tools)?;
    let tool = tools
        .iter()
        .find(|v| v["name"] == NAME)
        .ok_or("atom label tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    schema(&tool["inputSchema"])?;
    client.call(3, "bond.create", create())?;
    let before = client.call(4, "document.inspect", json!({}))?;
    graph(&before, 6, 3);
    let mut missing = label(1, 8, 1);
    missing
        .as_object_mut()
        .ok_or("params")?
        .remove("explicit_hydrogens");
    let mut extra = label(1, 8, 1);
    extra["label"] = json!("OH");
    let mut bad_h = label(1, 8, 1);
    bad_h["explicit_hydrogens"] = json!(65536);
    let mut zero = label(1, 8, 1);
    zero["atom_id"] = json!(0);
    let mut absent = label(1, 8, 1);
    absent["atom_id"] = json!(99);
    for (index, (args, code)) in [
        (missing, "invalid_params"),
        (extra, "invalid_params"),
        (label(1, 119, 1), "invalid_params"),
        (bad_h, "invalid_params"),
        (zero, "invalid_params"),
        (absent, "not_found"),
        (label(1, 1, 1), "invalid_params"),
        (label(0, 8, 1), "stale_revision"),
    ]
    .into_iter()
    .enumerate()
    {
        rejected(&mut client, 10 + index as u64 * 2, args, code)?;
        assert_eq!(
            client.call(11 + index as u64 * 2, "document.inspect", json!({}))?,
            before
        );
    }
    let edited = client.call(30, NAME, label(1, 8, 1))?;
    assert_eq!(edited["revision"], 2);
    assert_eq!(edited["undo_depth"], 2);
    graph(&edited, 8, 1);
    let before_noop = client.call(31, "document.inspect", json!({}))?;
    assert_eq!(client.call(32, NAME, label(2, 8, 1))?, before_noop);
    let saved = client.call(33, "document.save_cdxml", json!({"path":methanol}))?;
    assert_eq!(saved["file"]["dirty"], false);
    let undone = client.call(34, "history.undo", json!({}))?;
    graph(&undone, 6, 3);
    assert_eq!(undone["revision"], 3);
    assert_eq!(undone["file"]["dirty"], true);
    assert_eq!(undone["redo_depth"], 1);
    let redone = client.call(35, "history.redo", json!({}))?;
    graph(&redone, 8, 1);
    assert_eq!(redone["revision"], 4);
    assert_eq!(redone["file"]["dirty"], false);
    let opened = client.call(36, "document.open_cdxml", json!({"path":methanol}))?;
    graph(&opened, 8, 1);
    assert_eq!(opened["revision"], 5);
    assert_eq!(opened["undo_depth"], 0);
    assert_eq!(opened["source"]["modified"], false);
    let methanol_bytes = fs::read(&methanol)?;
    let edited = client.call(37, NAME, label(5, 7, 2))?;
    graph(&edited, 7, 2);
    assert_eq!(edited["revision"], 6);
    assert_eq!(edited["undo_depth"], 1);
    assert_eq!(
        edited["source"],
        json!({"format":"cdxml","byte_length":methanol_bytes.len(),"modified":true})
    );
    let saved = client.call(38, "document.save_cdxml", json!({"path":amine}))?;
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["source"]["modified"], true);
    assert_eq!(fs::read(&methanol)?, methanol_bytes);
    consume_saved(&methanol, 8, 1, "CH4O", "CO")?;
    consume_saved(&amine, 7, 2, "CH5N", "CN")?;
    client.finish()
}

fn call_socket(client: &mut BufReader<TcpStream>, name: &str, params: Value) -> AnyResult<Value> {
    let reply = socket(client, json!({"command":name,"params":params}))?;
    assert!(reply.get("error").is_none(), "{reply}");
    Ok(reply["result"].clone())
}

#[test]
fn authenticated_label_edits_never_grant_whole_source_imports_new_authority_or_repaint_on_refusal()
-> TestResult {
    const SMILES: &[u8] = b"CC named\r\n[13CH4]\t\0\xffopaque-final";
    let dir = support::SdfTestDirectory::new()?;
    let native = dir.path("atom-socket-λ.cdxml");
    let sdf = dir.path("atom-原.sdf");
    let sdf_copy = dir.path("atom-保存.sdf");
    let smiles = dir.path("atom-原.smi");
    let smiles_copy = dir.path("atom-保存.smi");
    fs::write(&sdf, SOURCE)?;
    fs::write(&smiles, SMILES)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let changes = Arc::new(AtomicUsize::new(0));
    let notified = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, engine, move || {
        notified.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    // Send the credential only on this trusted connection, never into a diagnostic value.
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    call_socket(&mut client, "bond.create", create())?;
    let edited = call_socket(&mut client, NAME, label(1, 8, 1))?;
    graph(&edited, 8, 1);
    assert_eq!(edited["revision"], 2);
    let before = call_socket(&mut client, "document.inspect", json!({}))?;
    let count = changes.load(Ordering::SeqCst);
    let failure = socket(&mut client, json!({"command":NAME,"params":label(1,8,1)}))?;
    assert_eq!(failure["error"]["code"], "stale_revision");
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        before
    );
    assert_eq!(call_socket(&mut client, NAME, label(2, 8, 1))?, before);
    assert_eq!(changes.load(Ordering::SeqCst), count);
    call_socket(&mut client, "document.save_cdxml", json!({"path":native}))?;
    let native_bytes = fs::read(&native)?;
    consume_saved(&native, 8, 1, "CH4O", "CO")?;
    let opened = call_socket(&mut client, "document.open_sdf", json!({"path":sdf}))?;
    assert_eq!(opened["revision"], 3);
    let before = call_socket(&mut client, "document.inspect", json!({}))?;
    let count = changes.load(Ordering::SeqCst);
    let failure = socket(&mut client, json!({"command":NAME,"params":label(3,8,1)}))?;
    assert_eq!(failure["error"]["code"], "unsupported_content");
    assert_eq!(
        call_socket(&mut client, "document.inspect", json!({}))?,
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), count);
    call_socket(&mut client, "document.save_sdf", json!({"path":sdf_copy}))?;
    assert_eq!(fs::read(&sdf)?, SOURCE);
    assert_eq!(fs::read(&sdf_copy)?, SOURCE);
    support::independently_read_sdf(&sdf_copy, SOURCE, true)?;
    let raw = call_socket(&mut client, "document.open_smiles", json!({"path":smiles}))?;
    assert_eq!(raw["revision"], 4);
    let imported = call_socket(
        &mut client,
        "document.import_smiles_records",
        json!({
        "expected_revision":4,"selection":{"mode":"record","record_id":1},
        "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
        "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4}),
    )?;
    assert_eq!(imported["revision"], 5);
    assert_eq!(imported["smiles_file"]["projected_records"], json!([1]));
    let before = call_socket(&mut client, "document.inspect", json!({}))?;
    let count = changes.load(Ordering::SeqCst);
    // Both an apparent no-op and an element change must refuse the synchronized-H/order-only backing.
    for params in [label(5, 6, 3), label(5, 8, 1)] {
        let failure = socket(&mut client, json!({"command":NAME,"params":params}))?;
        assert_eq!(failure["error"]["code"], "unsupported_content");
        assert_eq!(
            call_socket(&mut client, "document.inspect", json!({}))?,
            before
        );
    }
    assert_eq!(changes.load(Ordering::SeqCst), count);
    call_socket(
        &mut client,
        "document.save_smiles",
        json!({"path":smiles_copy}),
    )?;
    assert_eq!(fs::read(&smiles)?, SMILES);
    assert_eq!(fs::read(&smiles_copy)?, SMILES);
    assert_eq!(fs::read(&native)?, native_bytes);
    server.shutdown()?;
    Ok(())
}
