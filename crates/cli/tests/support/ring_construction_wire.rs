//! Independent real executable authoring, not a converter or future typed API.
use serde_json::{Value, json};
use std::process::Command;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn cli_creates_one_literal_six_carbon_ring_through_the_shared_registry() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("run")
        .arg("--cmd")
        .arg(
            json!({"command":"ring.create","params":{
                "expected_revision":0,"page_id":1,"size":6,
                "hydrogen_policy":"rectify_neutral_carbon",
                "placement":{"mode":"free","begin_x_pt":120,"begin_y_pt":168,
                    "end_x_pt":120,"end_y_pt":120,"side":"positive"}
            }})
            .to_string(),
        )
        .output()?;
    // The unchanged registry must reach a real CLI UnknownCommand failure here.
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
    assert_eq!(fragments[0]["id"], 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    assert_eq!(atoms.len(), 6);
    for (index, atom) in atoms.iter().enumerate() {
        assert_eq!(atom["id"], index + 1);
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], 2);
        assert_eq!(atom["isotope"], Value::Null);
        assert_eq!(atom["charge"], 0);
        assert_eq!(atom["radical"], "none");
    }
    let bonds = fragments[0]["bonds"].as_array().ok_or("bonds")?;
    assert_eq!(bonds.len(), 6);
    for (index, bond) in bonds.iter().enumerate() {
        assert_eq!(
            bond,
            &json!({"id":index+1,"begin":index+1,"end":(index+1)%6+1,"order":"single"})
        );
    }
    assert_eq!(result["revision"], 1);
    assert_eq!(result["undo_depth"], 1);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(result["changed"], true);
    assert_eq!(result["file"]["dirty"], true);
    assert!(result.get("conversion").is_none());
    assert!(result.get("file_operation").is_none());
    Ok(())
}

// Additive declarations preserve the exact first 2,095-byte executable case.
use super::{Client, cli, socket, support};
use bondline_commands::Engine;
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use std::{
    collections::BTreeSet,
    fs,
    io::{BufReader, Write as _},
    net::TcpStream,
    path::Path,
    process::Stdio,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

const NAME: &str = "ring.create";
const OLD_NAMES: [&str; 36] = [
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
    "ui.selection",
    "selection.translate",
];

fn params(revision: u64, placement: Value) -> Value {
    json!({"expected_revision":revision,"page_id":1,"size":6,
        "hydrogen_policy":"rectify_neutral_carbon","placement":placement})
}
fn free(revision: u64) -> Value {
    params(
        revision,
        json!({"mode":"free","begin_x_pt":120,"begin_y_pt":168,
        "end_x_pt":120,"end_y_pt":120,"side":"positive"}),
    )
}
fn fused(revision: u64) -> Value {
    params(
        revision,
        json!({"mode":"bond","fragment_id":1,"bond_id":1,"side":"negative"}),
    )
}
fn spiro(revision: u64) -> Value {
    params(
        revision,
        json!({"mode":"atom","fragment_id":1,"atom_id":1,
        "end_x_pt":120.0-24.0*3.0_f64.sqrt(),"end_y_pt":144,"side":"negative"}),
    )
}
fn request(params: Value) -> Value {
    json!({"command":NAME,"params":params})
}

fn expected(kind: &str) -> Value {
    let s = 3.0_f64.sqrt();
    let mut points = vec![
        [120.0, 168.0],
        [120.0, 120.0],
        [120.0 + 24.0 * s, 96.0],
        [120.0 + 48.0 * s, 120.0],
        [120.0 + 48.0 * s, 168.0],
        [120.0 + 24.0 * s, 192.0],
    ];
    let mut h = vec![2; 6];
    let mut bonds = vec![(1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 1)];
    match kind {
        "free" => {}
        "fused" => {
            points.extend([
                [120.0 - 24.0 * s, 96.0],
                [120.0 - 48.0 * s, 120.0],
                [120.0 - 48.0 * s, 168.0],
                [120.0 - 24.0 * s, 192.0],
            ]);
            h[0] = 1;
            h[1] = 1;
            h.extend([2; 4]);
            bonds.extend([(2, 7), (7, 8), (8, 9), (9, 10), (10, 1)]);
        }
        "spiro" => {
            // The two ring-centre bisectors at atom1 are analytically opposed.
            points.extend([
                [120.0 - 24.0 * s, 144.0],
                [120.0 - 48.0 * s, 168.0],
                [120.0 - 48.0 * s, 216.0],
                [120.0 - 24.0 * s, 240.0],
                [120.0, 216.0],
            ]);
            h[0] = 0;
            h.extend([2; 5]);
            bonds.extend([(1, 7), (7, 8), (8, 9), (9, 10), (10, 11), (11, 1)]);
        }
        _ => return Value::Null,
    }
    json!({"id":1,"atoms":points.into_iter().zip(h).enumerate().map(|(i,(p,h))|json!({
        "id":i+1,"element":6,"position":{"x_pt":p[0],"y_pt":p[1]},"isotope":null,
        "charge":0,"radical":"none","explicit_hydrogens":h})).collect::<Vec<_>>(),
        "bonds":bonds.into_iter().enumerate().map(|(i,(a,b))|json!({
            "id":i+1,"begin":a,"end":b,"order":"single"})).collect::<Vec<_>>()})
}
fn compare_fragment(actual: &Value, wanted: &Value) -> TestResult {
    assert_eq!(actual["id"], wanted["id"]);
    assert_eq!(actual["bonds"], wanted["bonds"]);
    let atoms = actual["atoms"].as_array().ok_or("actual atoms")?;
    let expected = wanted["atoms"].as_array().ok_or("expected atoms")?;
    assert_eq!(atoms.len(), expected.len());
    for (a, b) in atoms.iter().zip(expected) {
        let mut a = a.clone();
        let mut b = b.clone();
        let point = a
            .as_object_mut()
            .ok_or("atom")?
            .remove("position")
            .ok_or("position")?;
        let target = b
            .as_object_mut()
            .ok_or("atom")?
            .remove("position")
            .ok_or("position")?;
        assert_eq!(a, b, "all non-coordinate fields are exact");
        for axis in ["x_pt", "y_pt"] {
            assert!(
                (point[axis].as_f64().ok_or("point")? - target[axis].as_f64().ok_or("point")?)
                    .abs()
                    <= 1e-9
            );
        }
    }
    Ok(())
}
fn graph(state: &Value, kind: &str) -> TestResult {
    let fragments = state["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    compare_fragment(&fragments[0], &expected(kind))?;
    let mut cycles = vec![vec![1, 2, 3, 4, 5, 6]];
    if kind == "fused" {
        cycles.push(vec![1, 2, 7, 8, 9, 10]);
    }
    if kind == "spiro" {
        cycles.push(vec![1, 7, 8, 9, 10, 11]);
    }
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    let point = |id: u64| -> TestResult<[f64; 2]> {
        let a = atoms.iter().find(|a| a["id"] == id).ok_or("cycle atom")?;
        Ok([
            a["position"]["x_pt"].as_f64().ok_or("x")?,
            a["position"]["y_pt"].as_f64().ok_or("y")?,
        ])
    };
    for cycle in cycles {
        for i in 0..6 {
            let a = point(cycle[(i + 5) % 6])?;
            let b = point(cycle[i])?;
            let c = point(cycle[(i + 1) % 6])?;
            let u = [a[0] - b[0], a[1] - b[1]];
            let v = [c[0] - b[0], c[1] - b[1]];
            let left = u[0].hypot(u[1]);
            let right = v[0].hypot(v[1]);
            assert!((left - 48.0).abs() <= 1e-9 && (right - 48.0).abs() <= 1e-9);
            let angle = ((u[0] * v[0] + u[1] * v[1]) / (left * right))
                .clamp(-1.0, 1.0)
                .acos();
            assert!((angle - 2.0 * std::f64::consts::PI / 3.0).abs() <= 1e-10);
        }
    }
    Ok(())
}
fn shared_prefix(before: &Value, after: &Value, kind: &str) -> TestResult {
    let old = &before["document"]["pages"][0]["fragments"][0];
    let new = &after["document"]["pages"][0]["fragments"][0];
    let mut atoms = old["atoms"].as_array().ok_or("old atoms")?.clone();
    atoms[0]["explicit_hydrogens"] = json!(if kind == "spiro" { 0 } else { 1 });
    if kind == "fused" {
        atoms[1]["explicit_hydrogens"] = json!(1);
    }
    assert_eq!(
        new["atoms"]
            .as_array()
            .ok_or("new atoms")?
            .get(..6)
            .ok_or("old atom prefix")?,
        atoms.as_slice()
    );
    assert_eq!(
        new["bonds"]
            .as_array()
            .ok_or("new bonds")?
            .get(..6)
            .ok_or("old bond prefix")?,
        old["bonds"].as_array().ok_or("old bonds")?.as_slice()
    );
    assert_eq!(
        after["document"]["pages"][0]["size"],
        before["document"]["pages"][0]["size"]
    );
    Ok(())
}
fn names(entries: &[Value]) -> TestResult {
    let names = entries
        .iter()
        .map(|entry| entry["name"].as_str().ok_or("name"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(entries.len(), 54);
    assert_eq!(
        names,
        OLD_NAMES
            .into_iter()
            .chain([
                NAME,
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
                "atom.create_authored_label",
                "ui.caption",
                "text.create",
                "analysis.formula",
                "ui.analysis",
            ])
            .collect::<BTreeSet<_>>()
    );
    Ok(())
}
fn schema_node<'a>(root: &'a Value, node: &'a Value) -> TestResult<&'a Value> {
    match node.get("$ref").and_then(Value::as_str) {
        Some(reference) => Ok(root
            .pointer(
                reference
                    .strip_prefix('#')
                    .ok_or("nonlocal schema reference")?,
            )
            .ok_or("schema reference")?),
        None => Ok(node),
    }
}
fn schema(schema: &Value) -> TestResult {
    let root = schema;
    // Root remains an object union; putting additionalProperties:false here would
    // reject every branch field because this wrapper defines no properties.
    assert_eq!(root["type"], "object");
    assert!(root.get("additionalProperties").is_none());
    let variants = root["anyOf"].as_array().ok_or("ring selector variants")?;
    assert_eq!(variants.len(), 3);
    let mut sized = None;
    let mut named = None;
    let mut chair = None;
    for raw in variants {
        let branch = schema_node(root, raw)?;
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let fields = branch["properties"].as_object().ok_or("selector fields")?;
        let required = branch["required"]
            .as_array()
            .ok_or("selector required")?
            .iter()
            .map(|field| field.as_str().ok_or("selector field"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if fields.contains_key("size") {
            assert!(sized.replace(branch).is_none(), "duplicate sized branch");
            assert!(required.contains("size"));
            assert!(!fields.contains_key("kind") && !fields.contains_key("phase"));
            assert!(!required.contains("kind"));
        } else if schema_node(root, fields.get("kind").ok_or("kind selector")?)?["enum"]
            == json!(["cyclohexane_chair_1", "cyclohexane_chair_2"])
        {
            assert!(chair.replace(branch).is_none(), "duplicate chair branch");
            let expected = [
                "expected_revision",
                "page_id",
                "kind",
                "hydrogen_policy",
                "placement",
            ]
            .into_iter()
            .collect::<BTreeSet<_>>();
            assert_eq!(required, expected);
            assert_eq!(
                fields.keys().map(String::as_str).collect::<BTreeSet<_>>(),
                expected
            );
            assert_eq!(schema_node(root, &fields["kind"])?["type"], "string");
        } else {
            assert!(named.replace(branch).is_none(), "duplicate named branch");
            assert!(fields.contains_key("kind") && required.contains("kind"));
            assert!(!fields.contains_key("size") && !required.contains("size"));
        }
    }
    // Closedness plus opposite required/absent selectors makes the branches disjoint.
    named.ok_or("missing named branch")?;
    chair.ok_or("missing chair branch")?;
    let schema = sized.ok_or("missing exact legacy sized branch")?;
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    let fields = schema["properties"].as_object().ok_or("ring properties")?;
    let expected = [
        "expected_revision",
        "page_id",
        "size",
        "hydrogen_policy",
        "placement",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        fields.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(
        schema["required"]
            .as_array()
            .ok_or("required")?
            .iter()
            .map(|v| v.as_str().ok_or("name"))
            .collect::<Result<BTreeSet<_>, _>>()?,
        expected
    );
    assert_eq!(fields["size"]["minimum"].as_f64(), Some(3.0));
    assert_eq!(fields["size"]["maximum"].as_f64(), Some(8.0));
    assert_eq!(
        schema_node(root, &fields["hydrogen_policy"])?["enum"],
        json!(["rectify_neutral_carbon"])
    );
    let placement = schema_node(root, &fields["placement"])?;
    let branches = placement["oneOf"].as_array().ok_or("placements")?;
    assert_eq!(branches.len(), 3);
    let mut modes = BTreeSet::new();
    for branch in branches {
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let p = branch["properties"]
            .as_object()
            .ok_or("placement properties")?;
        let mode = p["mode"]["const"].as_str().ok_or("mode")?;
        assert!(modes.insert(mode));
        let expected = match mode {
            "free" => [
                "mode",
                "begin_x_pt",
                "begin_y_pt",
                "end_x_pt",
                "end_y_pt",
                "side",
            ]
            .into_iter()
            .collect::<BTreeSet<_>>(),
            "atom" => [
                "mode",
                "fragment_id",
                "atom_id",
                "end_x_pt",
                "end_y_pt",
                "side",
            ]
            .into_iter()
            .collect(),
            "bond" => ["mode", "fragment_id", "bond_id", "side"]
                .into_iter()
                .collect(),
            _ => return Err("unexpected placement".into()),
        };
        assert_eq!(
            p.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            branch["required"]
                .as_array()
                .ok_or("placement required")?
                .iter()
                .map(|v| v.as_str().ok_or("name"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            expected
        );
        let side = schema_node(root, &p["side"])?["enum"]
            .as_array()
            .ok_or("sides")?;
        assert_eq!(
            side.iter()
                .map(|v| v.as_str().ok_or("side"))
                .collect::<Result<BTreeSet<_>, _>>()?,
            ["positive", "negative"]
                .into_iter()
                .collect::<BTreeSet<_>>()
        );
    }
    Ok(())
}

fn consume(path: &Path, kind: &str) -> TestResult {
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
    let out = child.wait_with_output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let fragments: Value = serde_json::from_slice(&out.stdout)?;
    assert_eq!(fragments.as_array().ok_or("saved fragments")?.len(), 1);
    compare_fragment(&fragments[0], &expected(kind))?;
    let (atoms, bonds, formula, h) = match kind {
        "free" => (6, 6, "C6H12", vec![2; 6]),
        "fused" => (10, 11, "C10H18", vec![1, 1, 2, 2, 2, 2, 2, 2, 2, 2]),
        "spiro" => (11, 12, "C11H20", vec![0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]),
        _ => return Err("unknown consumer vector".into()),
    };
    // Same pinned actual-CDXML RDKit reader/settings; graph and H literals are independent.
    let out=Command::new(python).args(["-B","-c",r#"
import json,sys
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=open(sys.argv[1],'rb').read(1048577);assert 0<len(raw)<=1048576
mols=Chem.MolsFromCDXML(raw.decode('utf8'));assert len(mols)==1
m=mols[0];assert m.GetNumAtoms()==int(sys.argv[2]) and m.GetNumBonds()==int(sys.argv[3])
assert all(a.GetAtomicNum()==6 and a.GetFormalCharge()==a.GetNumRadicalElectrons()==a.GetIsotope()==0 and a.GetChiralTag()==Chem.ChiralType.CHI_UNSPECIFIED for a in m.GetAtoms())
assert all(b.GetBondType()==Chem.BondType.SINGLE and b.GetStereo()==Chem.BondStereo.STEREONONE for b in m.GetBonds())
assert [a.GetTotalNumHs(includeNeighbors=True) for a in m.GetAtoms()]==json.loads(sys.argv[5])
formula=rdMolDescriptors.CalcMolFormula(m);assert formula==sys.argv[4]
print(json.dumps({'formula':formula,'atoms':m.GetNumAtoms(),'bonds':m.GetNumBonds()}))
"#]).arg(path).arg(atoms.to_string()).arg(bonds.to_string()).arg(formula).arg(serde_json::to_string(&h)?)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout)?,
        json!({"formula":formula,"atoms":atoms,"bonds":bonds})
    );
    Ok(())
}

#[test]
fn cli_spiro_native_history_unicode_save_and_identity_floor_branch_are_real_authoring() -> TestResult
{
    let out = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(out.status.success());
    let entries: Value = serde_json::from_slice(&out.stdout)?;
    let entries = entries.as_array().ok_or("commands")?;
    names(entries)?;
    let entry = entries
        .iter()
        .find(|e| e["name"] == NAME)
        .ok_or("ring command")?;
    assert_eq!(entry["read_only"], false);
    schema(&entry["input_schema"])?;
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("ring-cli-original-λ.cdxml");
    let output = directory.path("ring-spiro-化学.cdxml");
    let original = cli(&[
        request(free(0)),
        json!({"command":"document.save_cdxml","params":{"path":input}}),
    ])?;
    graph(&original, "free")?;
    consume(&input, "free")?;
    let source = fs::read(&input)?;
    let created = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":input}}),
        json!({"command":"ui.selection","params":{"action":"set","expected_revision":1,"page_id":1,"atom_ids":[2,1]}}),
        request(spiro(1)),
    ])?;
    graph(&created, "spiro")?;
    shared_prefix(&original, &created, "spiro")?;
    assert_eq!(created["revision"], 2);
    assert_eq!(created["undo_depth"], 1);
    assert_eq!(created["ui_selection"]["atom_ids"], json!([]));
    assert_eq!(created["ui_selection"]["active"], true);
    let saved = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":input}}),
        request(spiro(1)),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":output}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    graph(&saved, "spiro")?;
    assert_eq!(saved["revision"], 4);
    assert_eq!(saved["undo_depth"], 1);
    assert_eq!(saved["redo_depth"], 0);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["file"]["path"], json!(fs::canonicalize(&output)?));
    assert_eq!(
        saved["source"],
        json!({"format":"cdxml","byte_length":source.len(),"modified":true})
    );
    consume(&output, "spiro")?;
    assert_eq!(fs::read(&input)?, source);
    let reopened = cli(&[json!({"command":"document.open_cdxml","params":{"path":output}})])?;
    graph(&reopened, "spiro")?;
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["source"]["modified"], false);
    let mut triangle = free(3);
    triangle["size"] = json!(3);
    triangle["placement"] = json!({"mode":"free",
        "begin_x_pt":360,"begin_y_pt":300,"end_x_pt":408,"end_y_pt":300,"side":"positive"});
    let branch = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":input}}),
        request(spiro(1)),
        json!({"command":"history.undo","params":{}}),
        request(triangle),
    ])?;
    let new = &branch["document"]["pages"][0]["fragments"][1];
    assert_eq!(new["id"], 2);
    assert_eq!(
        new["atoms"]
            .as_array()
            .ok_or("new atoms")?
            .iter()
            .map(|a| a["id"].clone())
            .collect::<Vec<_>>(),
        vec![json!(12), json!(13), json!(14)]
    );
    assert_eq!(
        new["bonds"],
        json!([{"id":13,"begin":12,"end":13,"order":"single"},
        {"id":14,"begin":13,"end":14,"order":"single"},{"id":15,"begin":14,"end":12,"order":"single"}])
    );
    assert_eq!(branch["revision"], 4);
    assert_eq!(branch["redo_depth"], 0);
    Ok(())
}

fn initialize(client: &mut Client) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25",
        "capabilities":{},"clientInfo":{"name":"ring-construction","version":"1"}}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], 1);
    assert!(reply.get("error").is_none());
    client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
}
fn rejected(client: &mut Client, id: u64, params: Value, code: &str) -> TestResult {
    client.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":NAME,"arguments":params}}))?;
    let reply = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], id);
    assert_eq!(reply["result"]["isError"], true);
    assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    Ok(())
}
#[test]
fn initialized_stdio_fuses_only_the_declared_edge_and_preserves_selection_on_refusal() -> TestResult
{
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("ring-stdio-original-λ.cdxml");
    let output = directory.path("ring-fused-保存.cdxml");
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().ok_or("tools")?;
    names(tools)?;
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == NAME)
        .ok_or("ring tool")?;
    assert_eq!(tool["annotations"]["readOnlyHint"], false);
    schema(&tool["inputSchema"])?;
    client.call(3, NAME, free(0))?;
    client.call(4, "document.save_cdxml", json!({"path":input}))?;
    client.call(5, "document.open_cdxml", json!({"path":input}))?;
    let source = fs::read(&input)?;
    let before = client.call(6, "document.inspect", json!({}))?;
    graph(&before, "free")?;
    client.call(
        7,
        "ui.selection",
        json!({"action":"set","expected_revision":2,"page_id":1,"atom_ids":[2,1]}),
    )?;
    let stable = client.call(8, "document.inspect", json!({}))?;
    let mut missing = free(2);
    missing
        .as_object_mut()
        .ok_or("params")?
        .remove("hydrogen_policy");
    let mut extra = fused(2);
    extra["placement"]["end_x_pt"] = json!(0);
    let mut side = free(2);
    side["placement"]["side"] = json!("left");
    let absent = params(
        2,
        json!({"mode":"atom","fragment_id":1,"atom_id":999,"end_x_pt":72,"end_y_pt":168,"side":"negative"}),
    );
    for (index, (p, code)) in [
        (missing, "invalid_params"),
        (extra, "invalid_params"),
        (side, "invalid_params"),
        (absent, "not_found"),
        (fused(1), "stale_revision"),
    ]
    .into_iter()
    .enumerate()
    {
        rejected(&mut client, 10 + index as u64 * 2, p, code)?;
        assert_eq!(
            client.call(11 + index as u64 * 2, "document.inspect", json!({}))?,
            stable
        );
    }
    let created = client.call(30, NAME, fused(2))?;
    graph(&created, "fused")?;
    shared_prefix(&before, &created, "fused")?;
    assert_eq!(created["revision"], 3);
    assert_eq!(created["undo_depth"], 1);
    assert_eq!(created["ui_selection"]["atom_ids"], json!([]));
    assert_eq!(created["ui_selection"]["active"], true);
    let saved = client.call(31, "document.save_cdxml", json!({"path":output}))?;
    assert_eq!(saved["file"]["dirty"], false);
    consume(&output, "fused")?;
    let undo = client.call(32, "history.undo", json!({}))?;
    graph(&undo, "free")?;
    assert_eq!(undo["revision"], 4);
    assert_eq!(undo["undo_depth"], 0);
    assert_eq!(undo["redo_depth"], 1);
    assert_eq!(undo["file"]["dirty"], true);
    assert_eq!(undo["source"]["modified"], false);
    let redo = client.call(33, "history.redo", json!({}))?;
    graph(&redo, "fused")?;
    assert_eq!(redo["revision"], 5);
    assert_eq!(redo["undo_depth"], 1);
    assert_eq!(redo["file"]["dirty"], false);
    assert_eq!(redo["source"]["modified"], true);
    assert_eq!(fs::read(&input)?, source);
    let reopened = client.call(34, "document.open_cdxml", json!({"path":output}))?;
    graph(&reopened, "fused")?;
    assert_eq!(reopened["revision"], 6);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["source"]["modified"], false);
    client.finish()
}

#[test]
fn authenticated_ring_refusals_keep_whole_owner_selection_and_opaque_source_authority() -> TestResult
{
    let directory = support::SdfTestDirectory::new()?;
    let opaque = directory.path("ring-opaque-原.cdxml");
    let copy = directory.path("ring-opaque-保存.cdxml");
    let raw =
        String::from_utf8(include_bytes!("../../../io/tests/fixtures/bondline-v0.cdxml").to_vec())?
            .replace("<CDXML ", "<CDXML Private=\"ring-preserve\" ");
    fs::write(&opaque, raw.as_bytes())?;
    let mut session = Engine::default();
    session.dispatch(NAME, free(0))?;
    session.dispatch(NAME, fused(1))?;
    session.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":2,"page_id":1,"atom_ids":[2,1]}),
    )?;
    let content = session.content().clone();
    let owner = session.capture_stamp();
    let stable = serde_json::to_value(session.state())?;
    graph(&stable, "fused")?;
    let engine: SharedEngine = Arc::new(Mutex::new(session));
    let changes = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&changes);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        count.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    // Bridgeheads have consistent H1, but cannot spend two H for an atom attachment.
    let insufficient = params(
        2,
        json!({"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":72,"end_y_pt":168,"side":"negative"}),
    );
    for (p, code) in [
        (insufficient, "unsupported_content"),
        (free(1), "stale_revision"),
        (free(2), "invalid_params"),
    ] {
        let result = socket(&mut client, request(p))?;
        assert_eq!(result["error"]["code"], code);
        let now = socket(
            &mut client,
            json!({"command":"document.inspect","params":{}}),
        )?;
        assert_eq!(now["result"], stable);
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &content);
        assert!(guard.capture_stamp_matches(&owner));
    }
    assert_eq!(changes.load(Ordering::SeqCst), 0);
    let mut triangle = free(2);
    triangle["size"] = json!(3);
    triangle["placement"] = json!({"mode":"free",
        "begin_x_pt":360,"begin_y_pt":300,"end_x_pt":408,"end_y_pt":300,"side":"positive"});
    let created = socket(&mut client, request(triangle))?;
    assert!(created.get("error").is_none(), "{created}");
    let new = &created["result"]["document"]["pages"][0]["fragments"][1];
    assert_eq!(new["id"], 2);
    assert_eq!(
        new["atoms"]
            .as_array()
            .ok_or("new atoms")?
            .iter()
            .map(|a| a["id"].clone())
            .collect::<Vec<_>>(),
        vec![json!(11), json!(12), json!(13)]
    );
    assert_eq!(
        new["bonds"],
        json!([{"id":12,"begin":11,"end":12,"order":"single"},
        {"id":13,"begin":12,"end":13,"order":"single"},{"id":14,"begin":13,"end":11,"order":"single"}])
    );
    assert_eq!(
        created["result"]["document"]["pages"][0]["fragments"][0],
        stable["document"]["pages"][0]["fragments"][0]
    );
    assert_eq!(created["result"]["revision"], 3);
    assert_eq!(created["result"]["ui_selection"]["atom_ids"], json!([]));
    socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    assert_eq!(changes.load(Ordering::SeqCst), 1);
    let opened = socket(
        &mut client,
        json!({"command":"document.open_cdxml","params":{"path":opaque,"discard_unsaved":true}}),
    )?;
    assert!(opened.get("error").is_none(), "{opened}");
    let before = socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?["result"]
        .clone();
    assert!(
        !engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .is_fully_mapped()
    );
    let revision = before["revision"].as_u64().ok_or("revision")?;
    let result = socket(&mut client, request(free(revision)))?;
    assert_eq!(result["error"]["code"], "unsupported_content");
    assert_eq!(
        socket(
            &mut client,
            json!({"command":"document.inspect","params":{}})
        )?["result"],
        before
    );
    assert_eq!(changes.load(Ordering::SeqCst), 2);
    let saved = socket(
        &mut client,
        json!({"command":"document.save_cdxml","params":{"path":copy}}),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    assert_eq!(fs::read(&opaque)?, raw.as_bytes());
    assert_eq!(fs::read(&copy)?, raw.as_bytes());
    socket(
        &mut client,
        json!({"command":"document.inspect","params":{}}),
    )?;
    assert_eq!(changes.load(Ordering::SeqCst), 3);
    server.shutdown()?;
    Ok(())
}

#[path = "ring_construction/unsaturated_wire.rs"]
mod unsaturated_wire;
