//! Independent existing-JSON actual-adapter acceptance; Root owns all execution.
use super::{TestResult, cli};
use serde_json::json;

#[test]
fn cli_sprout_adds_a_distinct_six_ring_and_one_connector_in_one_authored_step() -> TestResult {
    let result = cli(&[
        json!({"command":"ring.create","params":{"expected_revision":0,"page_id":1,"size":6,
            "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free",
            "begin_x_pt":120,"begin_y_pt":168,"end_x_pt":120,"end_y_pt":120,"side":"positive"}}}),
        json!({"command":"ring.sprout","params":{"expected_revision":1,"page_id":1,
            "fragment_id":1,"atom_id":1,"hydrogen_policy":"rectify_neutral_carbon",
            "ring":{"kind":"polygon","size":6},"ring_begin_x_pt":360,"ring_begin_y_pt":240,
            "ring_end_x_pt":408,"ring_end_y_pt":240,"side":"positive"}}),
    ])?;
    assert_eq!(result["revision"], 2);
    assert_eq!(result["undo_depth"], 2);
    assert_eq!(result["redo_depth"], 0);
    assert_eq!(result["changed"], true);
    let fragments = result["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?;
    assert_eq!(fragments.len(), 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("atoms")?;
    let bonds = fragments[0]["bonds"].as_array().ok_or("bonds")?;
    assert_eq!(atoms.len(), 12);
    assert_eq!(bonds.len(), 13);
    assert_eq!(atoms[0]["explicit_hydrogens"], 1);
    assert_eq!(atoms[6]["explicit_hydrogens"], 1);
    assert_eq!(bonds[6], json!({"id":7,"begin":1,"end":7,"order":"single"}));
    assert_eq!(
        bonds[12],
        json!({"id":13,"begin":12,"end":7,"order":"single"})
    );
    assert!(result.get("file_operation").is_none());
    Ok(())
}

// Additive declarations retain the exact1,700-byte first executable JSON case.
use super::super::{OLD_NAMES, schema_node};
use super::{
    Client, Engine, LoopbackServer, SharedEngine, atom, consume, graph, initialize, socket, support,
};
use bondline_mcp::CommandExecutor;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    io::BufReader,
    net::TcpStream,
    process::Command,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

const NAME: &str = "ring.sprout";
struct Vector {
    label: &'static str,
    ring: Value,
    weights: &'static [u16],
    notation: &'static str,
    formula: &'static str,
    negative: bool,
}
fn vectors() -> [Vector; 12] {
    [
        Vector {
            label: "triangle",
            ring: json!({"kind":"polygon","size":3}),
            weights: &[1, 1, 1],
            notation: "CCC1CC1",
            formula: "C5H10",
            negative: false,
        },
        Vector {
            label: "square",
            ring: json!({"kind":"polygon","size":4}),
            weights: &[1, 1, 1, 1],
            notation: "CCC1CCC1",
            formula: "C6H12",
            negative: false,
        },
        Vector {
            label: "pentagon",
            ring: json!({"kind":"polygon","size":5}),
            weights: &[1, 1, 1, 1, 1],
            notation: "CCC1CCCC1",
            formula: "C7H14",
            negative: false,
        },
        Vector {
            label: "hexagon",
            ring: json!({"kind":"polygon","size":6}),
            weights: &[1, 1, 1, 1, 1, 1],
            notation: "CCC1CCCCC1",
            formula: "C8H16",
            negative: false,
        },
        Vector {
            label: "heptagon",
            ring: json!({"kind":"polygon","size":7}),
            weights: &[1, 1, 1, 1, 1, 1, 1],
            notation: "CCC1CCCCCC1",
            formula: "C9H18",
            negative: false,
        },
        Vector {
            label: "octagon",
            ring: json!({"kind":"polygon","size":8}),
            weights: &[1, 1, 1, 1, 1, 1, 1, 1],
            notation: "CCC1CCCCCCC1",
            formula: "C10H20",
            negative: false,
        },
        Vector {
            label: "benzene-default",
            ring: json!({"kind":"benzene"}),
            weights: &[2, 1, 2, 1, 2, 1],
            notation: "CCc1ccccc1",
            formula: "C8H10",
            negative: false,
        },
        Vector {
            label: "benzene-single-negative",
            ring: json!({"kind":"benzene","phase":"single_first"}),
            weights: &[1, 2, 1, 2, 1, 2],
            notation: "CCc1ccccc1",
            formula: "C8H10",
            negative: true,
        },
        Vector {
            label: "cp-double",
            ring: json!({"kind":"cyclopentadiene","phase":"double_first"}),
            weights: &[2, 1, 2, 1, 1],
            notation: "CCC1=CC=CC1",
            formula: "C7H10",
            negative: false,
        },
        Vector {
            label: "cp-single",
            ring: json!({"kind":"cyclopentadiene","phase":"single_first"}),
            weights: &[1, 2, 1, 1, 2],
            notation: "CCC1C=CCC=1",
            formula: "C7H10",
            negative: false,
        },
        Vector {
            label: "chair-one",
            ring: json!({"kind":"cyclohexane_chair_1"}),
            weights: &[1, 1, 1, 1, 1, 1],
            notation: "CCC1CCCCC1",
            formula: "C8H16",
            negative: false,
        },
        Vector {
            label: "chair-two-negative",
            ring: json!({"kind":"cyclohexane_chair_2"}),
            weights: &[1, 1, 1, 1, 1, 1],
            notation: "CCC1CCCCC1",
            formula: "C8H16",
            negative: true,
        },
    ]
}
fn sprout(
    revision: u64,
    atom_id: u32,
    ring: Value,
    begin: [f64; 2],
    length: f64,
    negative: bool,
) -> Value {
    json!({"expected_revision":revision,"page_id":1,"fragment_id":1,"atom_id":atom_id,
        "hydrogen_policy":"rectify_neutral_carbon","ring":ring,
        "ring_begin_x_pt":begin[0],"ring_begin_y_pt":begin[1],
        "ring_end_x_pt":begin[0]+length,"ring_end_y_pt":begin[1],
        "side":if negative {"negative"} else {"positive"}})
}
fn seed_ethane() -> Value {
    json!({"command":"bond.create","params":{"page_id":1,"begin_x_pt":72,"begin_y_pt":168,
        "end_x_pt":120,"end_y_pt":168}})
}
fn seed_hexagon() -> Value {
    json!({"command":"ring.create","params":{"expected_revision":0,"page_id":1,"size":6,
        "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free",
        "begin_x_pt":120,"begin_y_pt":168,"end_x_pt":120,"end_y_pt":120,"side":"positive"}}})
}
fn points(vector: &Vector, begin: [f64; 2], length: f64) -> Vec<[f64; 2]> {
    let kind = vector.ring["kind"].as_str();
    let normalized = if matches!(kind, Some("cyclohexane_chair_1" | "cyclohexane_chair_2")) {
        // Independently closed orthographic tuples; preserve the supporting end-bond policy.
        let h = 2.0_f64.sqrt() / 8.0;
        let q = 3.0_f64.sqrt() / 8.0;
        let g = 4.0 / 17.0_f64.sqrt();
        let two = kind == Some("cyclohexane_chair_2");
        let raw = if two {
            [
                [-1.0, g * h],
                [-0.5, -g * (h + q)],
                [0.5, g * (h - q)],
                [1.0, -g * h],
                [0.5, g * (h + q)],
                [-0.5, g * (q - h)],
            ]
        } else {
            [
                [0.5, -g * (h + q)],
                [1.0, g * h],
                [0.5, g * (q - h)],
                [-0.5, g * (q + h)],
                [-1.0, -g * h],
                [-0.5, g * (h - q)],
            ]
        };
        let e = [0.5, (if two { -1.0 } else { 1.0 }) * g * (2.0 * h + q)];
        let d = (7.0 + 6.0_f64.sqrt()) / 17.0;
        raw.into_iter()
            .map(|p| {
                let w = [p[0] - raw[0][0], p[1] - raw[0][1]];
                [
                    (w[0] * e[0] + w[1] * e[1]) / d,
                    (e[0] * w[1] - e[1] * w[0]) / d,
                ]
            })
            .collect::<Vec<_>>()
    } else {
        // Closed circumcircle, not the product's iterative exterior-angle recurrence.
        let half = std::f64::consts::PI / vector.weights.len() as f64;
        let radius = 0.5 / half.sin();
        let center = [0.5, 0.5 / half.tan()];
        (0..vector.weights.len())
            .map(|i| {
                let angle = -std::f64::consts::FRAC_PI_2 - half + 2.0 * half * i as f64;
                [
                    center[0] + radius * angle.cos(),
                    center[1] + radius * angle.sin(),
                ]
            })
            .collect()
    };
    normalized
        .into_iter()
        .map(|p| {
            [
                begin[0] + length * p[0],
                begin[1] + (if vector.negative { -1.0 } else { 1.0 }) * length * p[1],
            ]
        })
        .collect()
}
fn append_ring(
    mut old: Value,
    vector: &Vector,
    target: usize,
    begin: [f64; 2],
    length: f64,
) -> TestResult<Value> {
    let old_atoms = old["atoms"].as_array().ok_or("seed atoms")?.len();
    let old_bonds = old["bonds"].as_array().ok_or("seed bonds")?.len();
    let h = old["atoms"][target - 1]["explicit_hydrogens"]
        .as_u64()
        .ok_or("old H")?;
    old["atoms"][target - 1]["explicit_hydrogens"] =
        json!(h.checked_sub(1).ok_or("target budget")?);
    let positions = points(vector, begin, length);
    let n = vector.weights.len();
    for (i, p) in positions.into_iter().enumerate() {
        let incident = vector.weights[(i + n - 1) % n] + vector.weights[i] + u16::from(i == 0);
        let h = 4_u16.checked_sub(incident).ok_or("literal ring H")?;
        old["atoms"]
            .as_array_mut()
            .ok_or("atoms")?
            .push(atom(old_atoms + i + 1, p, h, true));
    }
    old["bonds"].as_array_mut().ok_or("bonds")?.push(json!({
        "id":old_bonds+1,"begin":target,"end":old_atoms+1,"order":"single"}));
    for (i, w) in vector.weights.iter().copied().enumerate() {
        let mut b = json!({"id":old_bonds+i+2,"begin":old_atoms+i+1,
            "end":old_atoms+(i+1)%n+1,"order":if w==2 {"double"} else {"single"}});
        if w == 2 {
            b["double_position"] = json!(if vector.negative { "left" } else { "right" });
        }
        old["bonds"].as_array_mut().ok_or("bonds")?.push(b);
    }
    Ok(old)
}
fn expected_ethane(vector: &Vector) -> TestResult<Value> {
    append_ring(
        json!({"id":1,"atoms":[atom(1,[72.0,168.0],3,false),atom(2,[120.0,168.0],3,false)],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]}),
        vector,
        2,
        [240.0, 240.0],
        64.0,
    )
}
fn scalar_fields() -> BTreeSet<&'static str> {
    [
        "expected_revision",
        "page_id",
        "fragment_id",
        "atom_id",
        "hydrogen_policy",
        "ring",
        "ring_begin_x_pt",
        "ring_begin_y_pt",
        "ring_end_x_pt",
        "ring_end_y_pt",
        "side",
    ]
    .into_iter()
    .collect()
}
fn names(value: &Value) -> TestResult<BTreeSet<&str>> {
    Ok(value
        .as_array()
        .ok_or("string list")?
        .iter()
        .map(|v| v.as_str().ok_or("name"))
        .collect::<Result<_, _>>()?)
}
fn discovery(entries: &[Value], key: &str) -> TestResult {
    assert_eq!(entries.len(), 49);
    assert_eq!(
        entries.get(43).ok_or("preserved sprout index")?["name"],
        NAME
    );
    assert_eq!(
        entries.get(44).ok_or("appended preferences index")?["name"],
        "ui.preferences"
    );
    assert_eq!(
        entries
            .iter()
            .map(|r| r["name"].as_str().ok_or("tool name"))
            .collect::<Result<BTreeSet<_>, _>>()?,
        OLD_NAMES
            .into_iter()
            .chain([
                "ring.create",
                "ui.ring_tool",
                "chain.create",
                "chain.create_path",
                "document.set_drawing_settings",
                "ui.drawing_settings",
                "ui.chain_tool",
                NAME,
                "ui.preferences",
                "document.set_document_settings",
                "text.set_run",
                "atom.set_authored_label",
                "atom.create_authored_label",
            ])
            .collect::<BTreeSet<_>>()
    );
    let entry = entries
        .iter()
        .find(|r| r["name"] == NAME)
        .ok_or("sprout metadata")?;
    if key == "input_schema" {
        assert_eq!(entry["read_only"], false);
    } else {
        assert_eq!(entry["annotations"]["readOnlyHint"], false);
    }
    let root = &entry[key];
    assert_eq!(root["type"], "object");
    assert_eq!(root["additionalProperties"], false);
    let fields = root["properties"].as_object().ok_or("sprout properties")?;
    assert_eq!(
        fields.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        scalar_fields()
    );
    assert_eq!(names(&root["required"])?, scalar_fields());
    for name in ["page_id", "fragment_id", "atom_id"] {
        assert_eq!(fields[name]["minimum"].as_f64(), Some(1.0));
    }
    for name in [
        "ring_begin_x_pt",
        "ring_begin_y_pt",
        "ring_end_x_pt",
        "ring_end_y_pt",
    ] {
        assert_eq!(fields[name]["type"], "number");
        assert_eq!(fields[name]["minimum"].as_f64(), Some(-100000.0));
        assert_eq!(fields[name]["maximum"].as_f64(), Some(100000.0));
    }
    assert_eq!(
        schema_node(root, &fields["hydrogen_policy"])?["enum"],
        json!(["rectify_neutral_carbon"])
    );
    assert_eq!(
        schema_node(root, &fields["side"])?["enum"],
        json!(["positive", "negative"])
    );
    let ring = schema_node(root, &fields["ring"])?;
    assert_eq!(ring["type"], "object");
    assert_eq!(ring["unevaluatedProperties"], false);
    let variants = ring["oneOf"].as_array().ok_or("five ring alternatives")?;
    assert_eq!(variants.len(), 5);
    let mut seen = BTreeSet::new();
    for raw in variants {
        let branch = schema_node(root, raw)?;
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let p = branch["properties"].as_object().ok_or("ring properties")?;
        let kind = p["kind"]["const"].as_str().ok_or("ring kind")?;
        assert!(seen.insert(kind));
        let mut required = BTreeSet::from(["kind"]);
        let mut allowed = required.clone();
        match kind {
            "polygon" => {
                required.insert("size");
                allowed.insert("size");
                assert_eq!(p["size"]["minimum"].as_f64(), Some(3.0));
                assert_eq!(p["size"]["maximum"].as_f64(), Some(8.0));
            }
            "benzene" | "cyclopentadiene" => {
                allowed.insert("phase");
                let phase = schema_node(root, &p["phase"])?;
                assert_eq!(phase["type"], "string");
                assert_eq!(phase["enum"], json!(["double_first", "single_first"]));
                assert!(p["phase"].get("default").is_none());
            }
            "cyclohexane_chair_1" | "cyclohexane_chair_2" => {}
            _ => return Err("unexpected ring alternative".into()),
        }
        assert_eq!(names(&branch["required"])?, required);
        assert_eq!(
            p.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            allowed
        );
    }
    assert_eq!(
        seen,
        BTreeSet::from([
            "polygon",
            "benzene",
            "cyclopentadiene",
            "cyclohexane_chair_1",
            "cyclohexane_chair_2"
        ])
    );
    Ok(())
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
fn cli_sprout_native_unicode_history_and_reopen_preserve_the_complete_old_source() -> TestResult {
    let listed = Command::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    discovery(
        serde_json::from_slice::<Value>(&listed.stdout)?
            .as_array()
            .ok_or("commands")?,
        "input_schema",
    )?;
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("sprout-cli-旧環-λ.cdxml");
    let output = directory.path("sprout-cli-新環-保存.cdxml");
    let copy = directory.path("sprout-cli-再保存.cdxml");
    let seed = cli(&[
        seed_hexagon(),
        json!({"command":"document.save_cdxml","params":{"path":input}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    let old = super::super::expected("free");
    graph(&seed, &old)?;
    consume(&input, &old, "C1CCCCC1", "C6H12")?;
    let original = fs::read(&input)?;
    let vector = Vector {
        label: "hexagon",
        ring: json!({"kind":"polygon","size":6}),
        weights: &[1, 1, 1, 1, 1, 1],
        notation: "C1(CCCCC1)C2CCCCC2",
        formula: "C12H22",
        negative: false,
    };
    let wanted = append_ring(old.clone(), &vector, 1, [360.0, 240.0], 48.0)?;
    let saved = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":input}}),
        json!({"command":NAME,"params":sprout(1,1,vector.ring.clone(),[360.0,240.0],48.0,false)}),
        json!({"command":"history.undo","params":{}}),
        json!({"command":"history.redo","params":{}}),
        json!({"command":"document.save_cdxml","params":{"path":output}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    graph(&saved, &wanted)?;
    assert_eq!(saved["revision"], 4);
    assert_eq!(saved["undo_depth"], 1);
    assert_eq!(saved["redo_depth"], 0);
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(
        saved["source"],
        json!({"format":"cdxml","byte_length":original.len(),"modified":true})
    );
    assert_eq!(fs::read(&input)?, original);
    consume(&output, &wanted, vector.notation, vector.formula)?;
    let bytes = fs::read(&output)?;
    let opened = cli(&[
        json!({"command":"document.open_cdxml","params":{"path":output}}),
        json!({"command":"bond.set_order","params":{"expected_revision":1,"page_id":1,"fragment_id":1,
            "bond_id":1,"order":"single","hydrogen_policy":"rectify_neutral_carbon"}}),
        json!({"command":"document.save_cdxml","params":{"path":copy}}),
        json!({"command":"document.inspect","params":{}}),
    ])?;
    graph(&opened, &wanted)?;
    assert_eq!(opened["revision"], 1);
    assert_eq!(opened["undo_depth"], 0);
    assert_eq!(opened["redo_depth"], 0);
    assert_eq!(opened["file"]["dirty"], false);
    assert_eq!(opened["source"]["modified"], false);
    assert_eq!(fs::read(&output)?, bytes);
    assert_eq!(fs::read(&copy)?, bytes);
    consume(&copy, &wanted, vector.notation, vector.formula)?;
    Ok(())
}

#[test]
fn initialized_stdio_sprout_all_ring_families_and_phases_are_closed_literal_file_workflows()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let mut client = Client::start()?;
    initialize(&mut client)?;
    client.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let listed = client.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(listed["id"], 2);
    discovery(
        listed["result"]["tools"].as_array().ok_or("tools")?,
        "inputSchema",
    )?;
    for (i, vector) in vectors().into_iter().enumerate() {
        let index = u64::try_from(i)?;
        let id = 10 + 5 * index;
        let revision = 3 * index;
        let empty = client.call(
            id,
            "document.new",
            json!({"expected_revision":revision,"discard_unsaved":true}),
        )?;
        assert_eq!(empty["revision"], revision + 1);
        assert_eq!(empty["undo_depth"], 0);
        let seed = client.call(id + 1, "bond.create", seed_ethane()["params"].clone())?;
        assert_eq!(seed["revision"], revision + 2);
        let created = client.call(
            id + 2,
            NAME,
            sprout(
                revision + 2,
                2,
                vector.ring.clone(),
                [240.0, 240.0],
                64.0,
                vector.negative,
            ),
        )?;
        let wanted = expected_ethane(&vector)?;
        graph(&created, &wanted)?;
        assert_eq!(created["revision"], revision + 3);
        assert_eq!(created["undo_depth"], 2);
        assert_eq!(created["redo_depth"], 0);
        assert_eq!(created["changed"], true);
        assert_eq!(created["source"], Value::Null);
        assert!(created.get("file_operation").is_none());
        assert!(created.get("ui_ring_tool_operation").is_none());
        let path = directory.path(&format!("sprout-stdio-{}-化学.cdxml", vector.label));
        let saved = client.call(id + 3, "document.save_cdxml", json!({"path":path}))?;
        assert_eq!(saved["file"]["dirty"], false);
        assert_eq!(saved["file_operation"]["input_revision"], revision + 3);
        consume(&path, &wanted, vector.notation, vector.formula)?;
        let stable = client.call(id + 4, "document.inspect", json!({}))?;
        graph(&stable, &wanted)?;
        assert_eq!(stable["revision"], revision + 3);
        assert_eq!(stable["undo_depth"], 2);
        assert!(stable.get("file_operation").is_none());
    }
    let stable = client.call(80, "document.inspect", json!({}))?;
    assert_eq!(stable["revision"], 36);
    let valid = sprout(
        36,
        2,
        json!({"kind":"polygon","size":6}),
        [420.0, 420.0],
        48.0,
        false,
    );
    for (i, params) in [
        {
            let mut v = valid.clone();
            v["size"] = json!(6);
            v
        },
        {
            let mut v = valid.clone();
            v["ring"] = json!({"kind":"polygon","size":6,"phase":"single_first"});
            v
        },
        {
            let mut v = valid.clone();
            v["ring"] = json!({"kind":"benzene","size":6});
            v
        },
        {
            let mut v = valid.clone();
            v["ring"] = json!({"kind":"benzene","phase":null});
            v
        },
        {
            let mut v = valid.clone();
            v["ring"] = json!({"kind":"cyclohexane_chair_1","phase":"double_first"});
            v
        },
        {
            let mut v = valid.clone();
            v["ring"] = json!({"kind":null});
            v
        },
        {
            let mut v = valid.clone();
            v["ring"] = json!({"kind":"unknown_ring"});
            v
        },
        {
            let mut v = valid.clone();
            v.as_object_mut().ok_or("params")?.remove("ring_end_y_pt");
            v
        },
    ]
    .into_iter()
    .enumerate()
    {
        let id = 90 + 2 * u64::try_from(i)?;
        rejected(&mut client, id, params, "invalid_params")?;
        assert_eq!(client.call(id + 1, "document.inspect", json!({}))?, stable);
    }
    let mut stale = valid;
    stale["expected_revision"] = json!(0);
    rejected(&mut client, 110, stale, "stale_revision")?;
    assert_eq!(client.call(111, "document.inspect", json!({}))?, stable);
    let queried = client.call(112, "ui.controls", json!({}))?;
    assert_eq!(queried["controls"].as_array().ok_or("controls")?.len(), 39);
    client.finish()
}

#[test]
fn authenticated_sprout_publishes_after_reply_and_preserves_old_owners_on_source_refusal()
-> TestResult {
    let directory = support::SdfTestDirectory::new()?;
    let input = directory.path("sprout-socket-原.cdxml");
    let output = directory.path("sprout-socket-新-保存.cdxml");
    let opaque = directory.path("sprout-opaque-原.cdxml");
    let copy = directory.path("sprout-opaque-保存.cdxml");
    let raw = String::from_utf8(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/fixtures/bondline-v0.cdxml"
        ))
        .to_vec(),
    )?
    .replace("<CDXML ", "<CDXML Private=\"sprout-retained-source\" ");
    fs::write(&opaque, raw.as_bytes())?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let executor = CommandExecutor::new(Arc::clone(&engine));
    let observer = executor.clone();
    let (notify, noticed) = mpsc::channel();
    let (ack, acks) = mpsc::channel();
    let acks = Mutex::new(acks);
    let (retire, retired) = mpsc::channel();
    let server = LoopbackServer::bind_with_executor(0, executor, move || {
        let _ = notify.send(observer.has_active_work());
        let acknowledged = acks
            .lock()
            .ok()
            .is_some_and(|r| r.recv_timeout(Duration::from_secs(10)).is_ok());
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
    let call =
        |client: &mut BufReader<TcpStream>, name: &str, params: Value| -> TestResult<Value> {
            let response = socket(client, json!({"command":name,"params":params}))?;
            assert!(response.get("error").is_none(), "{response}");
            Ok(response["result"].clone())
        };
    let publication = || -> TestResult {
        assert!(
            !noticed.recv_timeout(Duration::from_secs(10))?,
            "reply activity must be released before callback"
        );
        ack.send(())?;
        assert!(retired.recv_timeout(Duration::from_secs(10))?);
        Ok(())
    };
    call(&mut client, "bond.create", seed_ethane()["params"].clone())?;
    publication()?;
    call(&mut client, "document.save_cdxml", json!({"path":input}))?;
    publication()?;
    call(&mut client, "document.open_cdxml", json!({"path":input}))?;
    publication()?;
    let original = fs::read(&input)?;
    let before = call(&mut client, "document.inspect", json!({}))?;
    assert_eq!(before["revision"], 2);
    assert_eq!(before["undo_depth"], 0);
    assert_eq!(before["file"]["dirty"], false);
    let vector = Vector {
        label: "benzene-negative",
        ring: json!({"kind":"benzene","phase":"single_first"}),
        weights: &[1, 2, 1, 2, 1, 2],
        notation: "CCc1ccccc1",
        formula: "C8H10",
        negative: true,
    };
    let wanted = expected_ethane(&vector)?;
    let created = call(
        &mut client,
        NAME,
        sprout(2, 2, vector.ring.clone(), [240.0, 240.0], 64.0, true),
    )?;
    graph(&created, &wanted)?;
    assert_eq!(created["revision"], 3);
    assert_eq!(created["undo_depth"], 1);
    assert_eq!(created["redo_depth"], 0);
    assert_eq!(created["changed"], true);
    assert_eq!(
        created["source"],
        json!({"format":"cdxml","byte_length":original.len(),"modified":true})
    );
    assert!(created.get("file_operation").is_none());
    publication()?;
    let stable = call(&mut client, "document.inspect", json!({}))?;
    let (content, stamp, model) = {
        let e = engine.lock().map_err(|_| "engine")?;
        (
            e.content().clone(),
            e.capture_stamp(),
            e.stored_document()?.model_snapshot(),
        )
    };
    let stale = socket(
        &mut client,
        json!({"command":NAME,
        "params":sprout(2,2,vector.ring.clone(),[440.0,440.0],48.0,true)}),
    )?;
    assert_eq!(stale["error"]["code"], "stale_revision");
    assert_eq!(call(&mut client, "document.inspect", json!({}))?, stable);
    {
        let e = engine.lock().map_err(|_| "engine")?;
        assert_eq!(e.content(), &content);
        assert!(e.capture_stamp_matches(&stamp));
        assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    }
    assert!(
        noticed.try_recv().is_err(),
        "refused work cannot publish a repaint"
    );
    let saved = call(&mut client, "document.save_cdxml", json!({"path":output}))?;
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["file_operation"]["input_revision"], 3);
    publication()?;
    consume(&output, &wanted, vector.notation, vector.formula)?;
    let undo = call(&mut client, "history.undo", json!({}))?;
    assert_eq!(undo["document"], before["document"]);
    assert_eq!(undo["source"]["modified"], false);
    publication()?;
    let redo = call(&mut client, "history.redo", json!({}))?;
    graph(&redo, &wanted)?;
    assert_eq!(redo["revision"], 5);
    assert_eq!(redo["file"]["dirty"], false);
    publication()?;
    assert_eq!(fs::read(&input)?, original);
    call(&mut client, "document.open_cdxml", json!({"path":opaque}))?;
    publication()?;
    let stable = call(&mut client, "document.inspect", json!({}))?;
    assert_eq!(stable["revision"], 6);
    let (whole, stamp) = {
        let e = engine.lock().map_err(|_| "engine")?;
        (e.content().clone(), e.capture_stamp())
    };
    let refused = socket(
        &mut client,
        json!({"command":NAME,
        "params":sprout(6,2,json!({"kind":"polygon","size":6}),[440.0,440.0],48.0,false)}),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_content");
    assert_eq!(call(&mut client, "document.inspect", json!({}))?, stable);
    {
        let e = engine.lock().map_err(|_| "engine")?;
        assert_eq!(e.content(), &whole);
        assert!(e.capture_stamp_matches(&stamp));
    }
    assert!(noticed.try_recv().is_err());
    call(&mut client, "document.save_cdxml", json!({"path":copy}))?;
    publication()?;
    assert_eq!(fs::read(&opaque)?, raw.as_bytes());
    assert_eq!(fs::read(&copy)?, raw.as_bytes());
    assert_eq!(fs::read(&input)?, original);
    drop(client);
    server.shutdown()?;
    Ok(())
}
