//! Independent executable selection acceptance through the existing JSON adapter.
use serde_json::{Value, json};
use std::process::Command;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn cli_selection_is_explicit_sorted_and_does_not_author_the_existing_graph() -> TestResult {
    let seed = vec![
        json!({"command":"bond.create","params":{"page_id":1,
            "begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}}),
        json!({"command":"bond.create","params":{"page_id":1,
            "begin_x_pt":220,"begin_y_pt":72,"end_x_pt":268,"end_y_pt":72}}),
        json!({"command":"atom.set_label","params":{"expected_revision":2,
            "page_id":1,"fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}}),
    ];
    let run = |requests: &[Value]| -> TestResult<Value> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_bondline-cli"));
        command.arg("run");
        for request in requests {
            command.arg("--cmd").arg(request.to_string());
        }
        let output = command.output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(serde_json::from_slice(&output.stdout)?)
    };
    let before = run(&seed)?;
    assert_eq!(before["revision"], 3);
    assert_eq!(before["undo_depth"], 3);
    assert!(before.get("ui_selection").is_none());
    let mut requests = seed;
    requests.push(json!({"command":"ui.selection","params":{
        "action":"set","expected_revision":3,"page_id":1,"atom_ids":[2,1]}}));
    // On the unmodified registry this reaches the real CLI UnknownCommand failure.
    let selected = run(&requests)?;
    assert_eq!(
        selected["ui_selection"],
        json!({
        "active":true,"epoch":1,"page_id":1,"atom_ids":[1,2]})
    );
    assert_eq!(
        selected["ui_selection_operation"],
        json!({"epoch":1,"changed":true})
    );
    assert_eq!(selected["changed"], false);
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
        assert_eq!(selected[key], before[key], "selection authored {key}");
    }
    assert!(selected.get("file_operation").is_none());
    Ok(())
}

// Additive declaration preserves the complete frozen 2,204-byte initial test.
use super::{Client, cli, socket, support};
use bondline_commands::Engine;
use bondline_mcp::{CommandExecutor, SharedEngine, control::LoopbackServer};
use std::{
    collections::BTreeSet,
    fs,
    io::{BufReader, Write as _},
    net::TcpStream,
    path::Path,
    process::Stdio,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

const SELECT: &str = "ui.selection";
const MOVE: &str = "selection.translate";
const OLD_NAMES: [&str; 34] = [
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
    "atom.set_label",
    "ui.atom_label",
];

fn seed() -> Vec<Value> {
    vec![
        json!({"command":"bond.create","params":{"page_id":1,
            "begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}}),
        json!({"command":"bond.create","params":{"page_id":1,
            "begin_x_pt":220,"begin_y_pt":72,"end_x_pt":268,"end_y_pt":72}}),
        json!({"command":"atom.set_label","params":{"expected_revision":2,
            "page_id":1,"fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}}),
    ]
}
fn set(revision: u64) -> Value {
    json!({"action":"set","expected_revision":revision,"page_id":1,"atom_ids":[2,1]})
}
fn translate(revision: u64, epoch: u64, x: f64, y: f64) -> Value {
    json!({"expected_revision":revision,"expected_selection_epoch":epoch,"delta_x_pt":x,"delta_y_pt":y})
}
fn expected_graph(moved: bool) -> Value {
    let (x1, x2, y) = if moved {
        (96.0, 144.0, 60.0)
    } else {
        (72.0, 120.0, 72.0)
    };
    json!([
        {"id":1,"atoms":[
            {"id":1,"element":6,"position":{"x_pt":x1,"y_pt":y},"isotope":null,
             "charge":0,"radical":"none","explicit_hydrogens":3},
            {"id":2,"element":8,"position":{"x_pt":x2,"y_pt":y},"isotope":null,
             "charge":0,"radical":"none","explicit_hydrogens":1}],
         "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]},
        {"id":2,"atoms":[
            {"id":3,"element":6,"position":{"x_pt":220.0,"y_pt":72.0},"isotope":null,
             "charge":0,"radical":"none","explicit_hydrogens":3},
            {"id":4,"element":6,"position":{"x_pt":268.0,"y_pt":72.0},"isotope":null,
             "charge":0,"radical":"none","explicit_hydrogens":3}],
         "bonds":[{"id":2,"begin":3,"end":4,"order":"single"}]}
    ])
}
fn graph(state: &Value, moved: bool) {
    assert_eq!(
        state["document"]["pages"][0]["fragments"],
        expected_graph(moved)
    );
}
fn same_authored(actual: &Value, expected: &Value) {
    for name in [
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
        assert_eq!(actual[name], expected[name], "unchanged {name}");
    }
}
fn operation(value: &Value, epoch: u64, changed: bool) {
    assert_eq!(value["changed"], false);
    assert_eq!(
        value["ui_selection_operation"],
        json!({"epoch":epoch,"changed":changed})
    );
    assert!(value.get("file_operation").is_none());
    assert!(value.get("close_operation").is_none());
}
fn names(entries: &[Value]) -> TestResult {
    let actual = entries
        .iter()
        .map(|entry| entry["name"].as_str().ok_or("command name"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(entries.len(), 48);
    assert_eq!(
        actual,
        OLD_NAMES
            .into_iter()
            .chain([
                SELECT,
                MOVE,
                "ring.create",
                "ui.ring_tool",
                "chain.create",
                "chain.create_path",
                "document.set_drawing_settings",
                "ui.drawing_settings",
                "ui.chain_tool",
                "ring.sprout",
                "ui.preferences",
                "document.set_document_settings",
                "text.set_run",
                "atom.set_authored_label",
            ])
            .collect::<BTreeSet<_>>()
    );
    Ok(())
}
fn closed_schema(schema: &Value, selection: bool) -> TestResult {
    assert_eq!(schema["type"], "object");
    if selection {
        let branches = schema["oneOf"].as_array().ok_or("selection branches")?;
        assert_eq!(branches.len(), 5);
        let mut seen = BTreeSet::new();
        for branch in branches {
            assert_eq!(branch["type"], "object");
            assert_eq!(branch["additionalProperties"], false);
            let fields = branch["properties"]
                .as_object()
                .ok_or("selection properties")?;
            let action = fields["action"]["const"].as_str().ok_or("selection tag")?;
            assert!(seen.insert(action));
            let expected: BTreeSet<_> = match action {
                "activate" | "clear" | "inspect" => {
                    ["action", "expected_revision"].into_iter().collect()
                }
                "all" => ["action", "expected_revision", "page_id"]
                    .into_iter()
                    .collect(),
                "set" => ["action", "expected_revision", "page_id", "atom_ids"]
                    .into_iter()
                    .collect(),
                _ => return Err("unexpected selection action".into()),
            };
            assert_eq!(
                fields.keys().map(String::as_str).collect::<BTreeSet<_>>(),
                expected
            );
            assert_eq!(
                branch["required"]
                    .as_array()
                    .ok_or("required fields")?
                    .iter()
                    .map(|v| v.as_str().ok_or("required name"))
                    .collect::<Result<BTreeSet<_>, _>>()?,
                expected
            );
        }
    } else {
        assert_eq!(schema["additionalProperties"], false);
        let expected = [
            "expected_revision",
            "expected_selection_epoch",
            "delta_x_pt",
            "delta_y_pt",
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(
            schema["properties"]
                .as_object()
                .ok_or("translate properties")?
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            schema["required"]
                .as_array()
                .ok_or("translate required")?
                .iter()
                .map(|v| v.as_str().ok_or("required name"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            expected
        );
    }
    Ok(())
}

fn consume_saved(path: &Path, moved: bool) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
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
    child
        .stdin
        .take()
        .ok_or("Expat stdin")?
        .write_all(&fs::read(path)?)?;
    let parsed = child.wait_with_output()?;
    assert!(
        parsed.status.success(),
        "{}",
        String::from_utf8_lossy(&parsed.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&parsed.stdout)?,
        expected_graph(moved)
    );
    // Same pinned RDKit CDXML reader/settings as existing actual-file tests; literal chemistry.
    let checked = Command::new(python).args(["-B","-c",r#"
import json,sys
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=open(sys.argv[1],'rb').read(1048577)
assert 0<len(raw)<=1048576
mols=Chem.MolsFromCDXML(raw.decode('utf8'))
assert len(mols)==2
assert [m.GetNumAtoms() for m in mols]==[2,2]
assert [m.GetNumBonds() for m in mols]==[1,1]
assert [[a.GetAtomicNum() for a in m.GetAtoms()] for m in mols]==[[6,8],[6,6]]
assert [[a.GetTotalNumHs(includeNeighbors=True) for a in m.GetAtoms()] for m in mols]==[[3,1],[3,3]]
assert [[(b.GetBeginAtomIdx(),b.GetEndAtomIdx(),str(b.GetBondType())) for b in m.GetBonds()] for m in mols]==[[(0,1,'SINGLE')],[(0,1,'SINGLE')]]
assert all(a.GetFormalCharge()==a.GetNumRadicalElectrons()==a.GetIsotope()==0 and a.GetChiralTag()==Chem.ChiralType.CHI_UNSPECIFIED for m in mols for a in m.GetAtoms())
assert all(b.GetStereo()==Chem.BondStereo.STEREONONE for m in mols for b in m.GetBonds())
assert [rdMolDescriptors.CalcMolFormula(m) for m in mols]==['CH4O','C2H6']
assert [Chem.MolToSmiles(m) for m in mols]==['CO','CC']
print(json.dumps({'formulas':['CH4O','C2H6'],'atoms':4,'bonds':2}))
"#]).arg(path).stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&checked.stdout)?,
        json!({"formulas":["CH4O","C2H6"],"atoms":4,"bonds":2})
    );
    Ok(())
}

#[test]
fn cli_moves_native_targets_once_and_saves_exact_geometry_through_unicode_history() -> TestResult {
    let listed = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(listed.status.success());
    let entries: Value = serde_json::from_slice(&listed.stdout)?;
    let entries = entries.as_array().ok_or("CLI registry")?;
    names(entries)?;
    for (name, selection) in [(SELECT, true), (MOVE, false)] {
        let entry = entries
            .iter()
            .find(|entry| entry["name"] == name)
            .ok_or("new command")?;
        assert_eq!(entry["read_only"], false);
        closed_schema(&entry["input_schema"], selection)?;
    }
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("selection-original-λ.cdxml");
    let output = directory.path("selection-moved-化学.cdxml");
    let mut initial = seed();
    initial.push(json!({"command":"document.save_cdxml","params":{"path":input}}));
    cli(&initial)?;
    consume_saved(&input, false)?;
    let original_bytes = fs::read(&input)?;
    let moved = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":input}}),
        json!({"command":SELECT,"params":set(1)}),
        json!({"command":MOVE,"params":translate(1,1,24.0,-12.0)}),
    ])?;
    graph(&moved, true);
    assert_eq!(moved["revision"], 2);
    assert_eq!(moved["undo_depth"], 1);
    assert_eq!(
        moved["ui_selection"],
        json!({"active":true,"epoch":2,"page_id":1,"atom_ids":[1,2]})
    );
    assert_eq!(moved["file"]["dirty"], true);
    assert_eq!(
        moved["source"],
        json!({"format":"cdxml","byte_length":original_bytes.len(),"modified":true})
    );
    let saved = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":input}}),
        json!({"command":SELECT,"params":set(1)}),
        json!({"command":MOVE,"params":translate(1,1,24.0,-12.0)}),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":output}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    graph(&saved, true);
    assert_eq!(saved["revision"], 4);
    assert_eq!(saved["undo_depth"], 1);
    assert_eq!(saved["redo_depth"], 0);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["file"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(saved["source"], moved["source"]);
    assert_eq!(fs::read(&input)?, original_bytes);
    consume_saved(&output, true)?;
    let reopened = cli(&[json!({"command":"document.open_cdxml","params":{"path":output}})])?;
    graph(&reopened, true);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["source"]["modified"], false);
    Ok(())
}

fn initialize(client: &mut Client) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"selection-acceptance","version":"1"}}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], 1);
    assert!(reply.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
}
fn rejected(client: &mut Client, id: u64, name: &str, params: Value, code: &str) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":params}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], id);
    assert_eq!(reply["result"]["isError"], true);
    assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    Ok(())
}

#[test]
fn initialized_stdio_keeps_selection_errors_noops_history_and_whole_readonly_source_exact()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let output = directory.path("selection-stdio-保存.cdxml");
    let input_sdf = directory.path("selection-readonly.sdf");
    let copy_sdf = directory.path("selection-preserved.sdf");
    const SDF: &[u8] = include_bytes!("../../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
    fs::write(&input_sdf, SDF)?;
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    names(tools)?;
    for (name, selection) in [(SELECT, true), (MOVE, false)] {
        let tool = tools
            .iter()
            .find(|tool| tool["name"] == name)
            .ok_or("selection tool")?;
        assert_eq!(tool["annotations"]["readOnlyHint"], false);
        closed_schema(&tool["inputSchema"], selection)?;
    }
    for (id, request) in (3..).zip(seed()) {
        client.call(
            id,
            request["command"].as_str().ok_or("seed command")?,
            request["params"].clone(),
        )?;
    }
    let before = client.call(6, "document.inspect", json!({}))?;
    graph(&before, false);
    let selected = client.call(7, SELECT, set(3))?;
    operation(&selected, 1, true);
    same_authored(&selected, &before);
    let stable = client.call(8, "document.inspect", json!({}))?;
    for (index, params) in [
        json!({"action":"activate","expected_revision":3,"atom_ids":[]}),
        json!({"action":"inspect","expected_revision":3,"unexpected":true}),
        json!({"action":"clear","expected_revision":3,"page_id":1}),
        json!({"action":"all","expected_revision":3,"atom_ids":[1]}),
        json!({"action":"set","expected_revision":3,"page_id":1,"atom_ids":[1,1]}),
        json!({"action":"set","expected_revision":3,"page_id":1,"atom_ids":[0]}),
    ]
    .into_iter()
    .enumerate()
    {
        rejected(
            &mut client,
            20 + index as u64 * 2,
            SELECT,
            params,
            "invalid_params",
        )?;
        assert_eq!(
            client.call(21 + index as u64 * 2, "document.inspect", json!({}))?,
            stable
        );
    }
    rejected(
        &mut client,
        40,
        SELECT,
        json!({"action":"set","expected_revision":3,"page_id":1,"atom_ids":[99]}),
        "not_found",
    )?;
    rejected(
        &mut client,
        41,
        MOVE,
        translate(2, 1, 24.0, -12.0),
        "stale_revision",
    )?;
    rejected(
        &mut client,
        42,
        MOVE,
        translate(3, 0, 24.0, -12.0),
        "stale_selection",
    )?;
    let mut extra = translate(3, 1, 24.0, -12.0);
    extra["snap"] = json!(true);
    rejected(&mut client, 43, MOVE, extra, "invalid_params")?;
    assert_eq!(client.call(44, "document.inspect", json!({}))?, stable);
    let noop = client.call(45, MOVE, translate(3, 1, 0.0, 0.0))?;
    same_authored(&noop, &stable);
    assert_eq!(noop["changed"], false);
    assert_eq!(noop["ui_selection"], stable["ui_selection"]);
    let inspected = client.call(
        46,
        SELECT,
        json!({"action":"inspect","expected_revision":3}),
    )?;
    operation(&inspected, 1, false);
    same_authored(&inspected, &stable);
    let moved = client.call(47, MOVE, translate(3, 1, 24.0, -12.0))?;
    graph(&moved, true);
    assert_eq!(moved["revision"], 4);
    assert_eq!(moved["undo_depth"], 4);
    assert_eq!(
        moved["document"]["pages"][0]["size"],
        before["document"]["pages"][0]["size"]
    );
    let undo = client.call(48, "history.undo", json!({}))?;
    graph(&undo, false);
    assert_eq!(undo["revision"], 5);
    assert_eq!(undo["undo_depth"], 3);
    assert_eq!(undo["redo_depth"], 1);
    assert_eq!(undo["ui_selection"]["atom_ids"], json!([]));
    assert_eq!(undo["ui_selection"]["page_id"], Value::Null);
    let reselected = client.call(80, SELECT, set(5))?;
    let epoch = reselected["ui_selection"]["epoch"]
        .as_u64()
        .ok_or("undo selection epoch")?;
    let noop = client.call(81, MOVE, translate(5, epoch, 0.0, 0.0))?;
    same_authored(&noop, &undo);
    assert_eq!(noop["redo_depth"], 1);
    assert_eq!(noop["ui_selection"], reselected["ui_selection"]);
    assert_eq!(noop["changed"], false);
    let redo = client.call(49, "history.redo", json!({}))?;
    graph(&redo, true);
    assert_eq!(redo["revision"], 6);
    assert_eq!(redo["undo_depth"], 4);
    assert_eq!(redo["redo_depth"], 0);
    let all = client.call(
        50,
        SELECT,
        json!({"action":"all","expected_revision":6,"page_id":1}),
    )?;
    assert_eq!(all["ui_selection"]["atom_ids"], json!([1, 2, 3, 4]));
    same_authored(&all, &redo);
    let clear = client.call(51, SELECT, json!({"action":"clear","expected_revision":6}))?;
    assert_eq!(clear["ui_selection"]["active"], true);
    assert_eq!(clear["ui_selection"]["page_id"], Value::Null);
    assert_eq!(clear["ui_selection"]["atom_ids"], json!([]));
    same_authored(&clear, &redo);
    let epoch = clear["ui_selection"]["epoch"]
        .as_u64()
        .ok_or("empty epoch")?;
    let active = client.call(
        52,
        SELECT,
        json!({"action":"activate","expected_revision":6}),
    )?;
    operation(&active, epoch, false);
    let saved = client.call(53, "document.save_cdxml", json!({"path":output}))?;
    assert_eq!(saved["file"]["dirty"], false);
    consume_saved(&output, true)?;
    let opened = client.call(54, "document.open_sdf", json!({"path":input_sdf}))?;
    assert_eq!(opened["document"], Value::Null);
    let revision = opened["revision"].as_u64().ok_or("SDF revision")?;
    let before = client.call(55, "document.inspect", json!({}))?;
    rejected(
        &mut client,
        56,
        SELECT,
        set(revision),
        "unsupported_content",
    )?;
    assert_eq!(client.call(57, "document.inspect", json!({}))?, before);
    let clear = client.call(
        58,
        SELECT,
        json!({"action":"clear","expected_revision":revision}),
    )?;
    same_authored(&clear, &before);
    let inspected = client.call(
        59,
        SELECT,
        json!({"action":"inspect","expected_revision":revision}),
    )?;
    same_authored(&inspected, &before);
    client.call(60, "document.save_sdf", json!({"path":copy_sdf}))?;
    assert_eq!(fs::read(&input_sdf)?, SDF);
    assert_eq!(fs::read(&copy_sdf)?, SDF);
    support::independently_read_sdf(&copy_sdf, SDF, true)?;
    client.finish()
}

#[test]
fn authenticated_selection_repaints_after_published_replies_but_never_for_inspect_noop_or_error()
-> TestResult {
    let mut session = Engine::default();
    for request in seed() {
        session.dispatch(
            request["command"].as_str().ok_or("seed command")?,
            request["params"].clone(),
        )?;
    }
    let before = serde_json::to_value(session.state())?;
    let content = session.content().clone();
    let owner = session.capture_stamp();
    let engine: SharedEngine = Arc::new(Mutex::new(session));
    let executor = CommandExecutor::new(Arc::clone(&engine));
    let observer = executor.clone();
    let (notify, noticed) = mpsc::channel();
    let (acknowledge, acknowledgements) = mpsc::channel();
    let acknowledgements = Mutex::new(acknowledgements);
    let (retire, retired) = mpsc::channel();
    let server = LoopbackServer::bind_with_executor(0, executor, move || {
        let _ = notify.send(observer.has_active_work());
        let acknowledged = acknowledgements
            .lock()
            .ok()
            .is_some_and(|receiver| receiver.recv_timeout(Duration::from_secs(10)).is_ok());
        let _ = retire.send(acknowledged);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    assert!(noticed.try_recv().is_err());
    let selected = socket(&mut client, json!({"command":SELECT,"params":set(3)}))?;
    assert!(selected.get("error").is_none(), "{selected}");
    operation(&selected["result"], 1, true);
    same_authored(&selected["result"], &before);
    // The complete response was readable while its callback still awaited this ACK.
    assert!(
        !noticed.recv_timeout(Duration::from_secs(10))?,
        "reply activity must retire before repaint"
    );
    acknowledge.send(())?;
    assert!(retired.recv_timeout(Duration::from_secs(10))?);
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &content);
        assert!(guard.capture_stamp_matches(&owner));
    }
    let moved = socket(
        &mut client,
        json!({"command":MOVE,"params":translate(3,1,24.0,-12.0)}),
    )?;
    assert!(moved.get("error").is_none(), "{moved}");
    graph(&moved["result"], true);
    assert_eq!(moved["result"]["revision"], 4);
    assert_eq!(moved["result"]["undo_depth"], 4);
    assert_eq!(moved["result"]["ui_selection"]["epoch"], 2);
    assert!(!noticed.recv_timeout(Duration::from_secs(10))?);
    acknowledge.send(())?;
    assert!(retired.recv_timeout(Duration::from_secs(10))?);
    let stable = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    for (name, params) in [
        (SELECT, json!({"action":"inspect","expected_revision":4})),
        (SELECT, set(4)),
        (MOVE, translate(4, 2, 0.0, 0.0)),
    ] {
        let result = socket(&mut client, json!({"command":name,"params":params}))?;
        assert!(result.get("error").is_none(), "{result}");
        same_authored(&result["result"], &stable);
        if name == SELECT {
            operation(&result["result"], 2, false);
        }
    }
    for (params, code) in [
        (translate(3, 2, 24.0, 0.0), "stale_revision"),
        (translate(4, 1, 24.0, 0.0), "stale_selection"),
    ] {
        let result = socket(&mut client, json!({"command":MOVE,"params":params}))?;
        assert_eq!(result["error"]["code"], code);
    }
    let final_state = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    assert_eq!(final_state["result"], stable);
    drop(client);
    server.shutdown()?;
    // Retirement makes absence decisive; a false callback cannot hide behind a later task.
    assert!(noticed.try_recv().is_err());
    assert!(retired.try_recv().is_err());
    Ok(())
}
