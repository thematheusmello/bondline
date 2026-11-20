//! Child of existing chain_creation/settings_wire.rs; no new transport implementation.
use super::*;
#[path = "bond_spacing_wire.rs"]
mod relative_spacing;
const COMBINED: &str = "document.set_document_settings";
const MIXED: &[u8] = include_bytes!("line-width-mixed.cdxml");
const OLD45: &str = include_str!("document-settings-old45.json");
fn set3(rev: u64, length: Option<f64>, angle: Option<f64>, width: Option<f64>) -> Value {
    json!({"expected_revision":rev,"bond_length_pt":length,
        "chain_angle_degrees":angle,"line_width_pt":width})
}
fn chemical(h: u8) -> Value {
    json!({"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,
         "charge":0,"radical":"none","explicit_hydrogens":h},
        {"id":2,"element":6,"position":{"x_pt":140.0,"y_pt":200.0},"isotope":null,
         "charge":0,"radical":"none","explicit_hydrogens":h}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":if h==2{"double"}else{"single"}}]})
}
fn model(v: &Value, h: u8, settings: (Option<f64>, Option<f64>, f64)) -> TestResult {
    graph(v, &chemical(h))?;
    assert_eq!(
        v["document"]["pages"][0]["size"],
        json!({"width_pt":612.0,"height_pt":792.0})
    );
    assert_eq!(
        v["document"]["drawing_style"],
        json!({"line_width_pt":settings.2})
    );
    let mut pair = json!({});
    if let Some(value) = settings.0 {
        pair["bond_length_pt"] = json!(value);
    }
    if let Some(value) = settings.1 {
        pair["chain_angle_degrees"] = json!(value);
    }
    if settings.0.is_none() && settings.1.is_none() {
        assert!(v["document"].get("drawing_settings").is_none());
    } else {
        assert_eq!(v["document"]["drawing_settings"], pair);
    }
    Ok(())
}
fn new_schema(s: &Value) -> TestResult {
    assert_eq!(s["type"], "object");
    assert_eq!(s["additionalProperties"], false);
    let keys = BTreeSet::from([
        "expected_revision",
        "bond_length_pt",
        "chain_angle_degrees",
        "line_width_pt",
    ]);
    let required = s["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or("required key"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(required, keys);
    let mut properties = keys.clone();
    properties.insert("bond_spacing_percent");
    assert_eq!(
        s["properties"]
            .as_object()
            .ok_or("properties")?
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        properties
    );
    for name in [
        "bond_length_pt",
        "chain_angle_degrees",
        "line_width_pt",
        "bond_spacing_percent",
    ] {
        let types = s["properties"][name]["type"].as_array().ok_or("nullable")?;
        assert_eq!(types.len(), 2);
        assert!(types.contains(&json!("number")) && types.contains(&json!("null")));
    }
    for name in ["bond_length_pt", "line_width_pt"] {
        assert_eq!(
            s["properties"][name]["exclusiveMinimum"].as_f64(),
            Some(0.0)
        );
        assert_eq!(s["properties"][name]["maximum"].as_f64(), Some(100_000.0));
    }
    assert_eq!(
        s["properties"]["chain_angle_degrees"]["minimum"].as_f64(),
        Some(1.0)
    );
    assert_eq!(
        s["properties"]["chain_angle_degrees"]["maximum"].as_f64(),
        Some(179.0)
    );
    assert_eq!(
        s["properties"]["bond_spacing_percent"]["minimum"].as_f64(),
        Some(1.0)
    );
    assert_eq!(
        s["properties"]["bond_spacing_percent"]["maximum"].as_f64(),
        Some(100.0)
    );
    Ok(())
}
fn metadata(rows: &[Value], mcp: bool) -> TestResult {
    let old: Vec<Value> = serde_json::from_str(OLD45)?;
    assert_eq!((old.len(), rows.len()), (45, 53));
    for (actual, expected) in rows.iter().zip(&old) {
        if mcp {
            assert_eq!(actual["name"], expected["name"]);
            assert_eq!(actual["description"], expected["description"]);
            assert_eq!(actual["annotations"]["readOnlyHint"], expected["read_only"]);
            assert_eq!(
                actual["inputSchema"], expected["input_schema"],
                "old schema:{}",
                expected["name"]
            );
        } else {
            assert_eq!(
                actual, expected,
                "all old metadata/schema records stay exact"
            );
        }
    }
    let last = rows.get(45).ok_or("settings row46")?;
    assert_eq!(last["name"], COMBINED);
    assert_eq!(
        if mcp {
            &last["annotations"]["readOnlyHint"]
        } else {
            &last["read_only"]
        },
        &json!(false)
    );
    new_schema(&last[if mcp { "inputSchema" } else { "input_schema" }])
}
fn literal(width: &str, double: bool, with_pair: bool) -> TestResult<Vec<u8>> {
    // Only these independently specified value changes/additions are allowed.
    let mut s =
        std::str::from_utf8(MIXED)?.replace("LineWidth='.500'", &format!("LineWidth='{width}'"));
    if double {
        s = s
            .replace("NumHydrogens='3'", "NumHydrogens='2'")
            .replace("NumHydrogens=\"3\"", "NumHydrogens=\"2\"")
            .replace("Order=\"1\"", "Order=\"2\"");
    }
    if with_pair {
        s = s.replace(
            "CreationProgram=\"Bondline 0.1.0\">",
            "CreationProgram=\"Bondline 0.1.0\" BondLength=\"24.5\" ChainAngle=\"120.5\">",
        );
    }
    Ok(s.into_bytes())
}
fn root_facts(raw: &[u8]) -> TestResult<Value> {
    let mut child=Process::new(python()).args(["-X","utf8","-B","-c",r#"
import json,sys,xml.parsers.expat as expat
from decimal import Decimal
raw=sys.stdin.buffer.read(1048577)
assert 0<len(raw)<=1048576
objects=[]
p=expat.ParserCreate(namespace_separator='|');p.ordered_attributes=True
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda n,a:objects.append((n,list(zip(a[::2],a[1::2]))))
p.Parse(raw,True)
assert [n for n,_ in objects]==['CDXML','page','fragment','n','n','b']
root=objects[0][1];attrs=dict(root)
assert set(attrs)<=set(['CreationProgram','BondLength','ChainAngle','LineWidth'])
assert attrs['CreationProgram']=='Bondline 0.1.0'
numbers={}
for k in ['BondLength','ChainAngle','LineWidth']:
 if k not in attrs: continue
 value=Decimal(attrs[k]);assert value.is_finite()
 assert (Decimal(1)<=value<=Decimal(179)) if k=='ChainAngle' else (Decimal(0)<value<=Decimal(100000))
 numbers[k]=str(value)
print(json.dumps({'root':root,'decimal':numbers},separators=(',',':')))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child
        .stdin
        .take()
        .ok_or("Root reader stdin")?
        .write_all(raw)?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(format!("Root reader: {}", String::from_utf8_lossy(&out.stderr)).into());
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}
fn checked_file(path: &Path, expected: &[u8], h: u8, full: bool) -> TestResult {
    let raw = fs::read(path)?;
    assert_eq!(raw, expected, "complete source bytes BEFORE consumers");
    let width = if raw == MIXED { ".500" } else { "2" };
    let mut attrs = vec![
        json!(["LineWidth", width]),
        json!(["CreationProgram", "Bondline 0.1.0"]),
    ];
    let mut numbers = json!({"LineWidth":if width==".500"{"0.500"}else{"2"}});
    if full {
        attrs.extend([
            json!(["BondLength", "24.5"]),
            json!(["ChainAngle", "120.5"]),
        ]);
        numbers["BondLength"] = json!("24.5");
        numbers["ChainAngle"] = json!("120.5");
    }
    assert_eq!(root_facts(&raw)?, json!({"root":attrs,"decimal":numbers}));
    // Reuse unchanged raw both-flags/directed-order/Decimal then named-ring/RDKit helpers.
    consume(
        path,
        &chemical(h),
        if h == 2 { "C=C" } else { "CC" },
        if h == 2 { "C2H4" } else { "C2H6" },
    )
}
fn qualify(dir: &support::SdfTestDirectory) -> TestResult {
    let manual = dir.path("manual-original-λ.cdxml");
    fs::write(&manual, MIXED)?;
    checked_file(&manual, MIXED, 3, false)?;
    let changed = literal("2", true, true)?;
    fs::write(&manual, &changed)?;
    checked_file(&manual, &changed, 2, true)?;
    let bad = std::str::from_utf8(MIXED)?.replace(".500", "NaN");
    assert!(root_facts(bad.as_bytes()).is_err());
    let other = std::str::from_utf8(MIXED)?.replace(".500", ".750");
    assert_eq!(
        root_facts(other.as_bytes())?["decimal"]["LineWidth"],
        "0.750"
    );
    Ok(())
}
fn denied(c: &mut Client, id: u64, name: &str, p: Value, code: &str) -> TestResult {
    c.send(
        json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":p}}),
    )?;
    let reply = c.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(reply["id"], id);
    assert_eq!(reply["result"]["isError"], true);
    assert_eq!(reply["result"]["structuredContent"]["error"]["code"], code);
    Ok(())
}
#[test]
fn actual_cli_keeps_old45_schemas_and_source_lexemes_through_style_and_chemical_history()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    qualify(&dir)?;
    let list = Process::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let rows: Vec<Value> = serde_json::from_slice(&list.stdout)?;
    metadata(&rows, false)?;
    let input = dir.path("cli-style-original-λ.cdxml");
    let copy = dir.path("cli-exact-λ.cdxml");
    let output = dir.path("cli-style-edited-化学.cdxml");
    fs::write(&input, MIXED)?;
    cli(&[
        request("document.open_cdxml", json!({"path":input})),
        request("document.save_cdxml", json!({"path":copy})),
    ])?;
    checked_file(&copy, MIXED, 3, false)?;
    let result = cli(&[
        request("document.open_cdxml", json!({"path":input})),
        request(COMBINED, set3(1, None, None, Some(2.0))),
        request(SET, set(2, Value::Null, Value::Null)),
        request(
            "bond.set_order",
            json!({"expected_revision":2,"page_id":1,"fragment_id":1,"bond_id":1,
            "order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
        ),
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request("document.save_cdxml", json!({"path":output})),
        request("document.open_cdxml", json!({"path":output})),
    ])?;
    model(&result, 2, (None, None, 2.0))?;
    assert_eq!(
        (
            result["revision"].clone(),
            result["undo_depth"].clone(),
            result["redo_depth"].clone()
        ),
        (json!(6), json!(0), json!(0))
    );
    assert_eq!(result["file"]["dirty"], false);
    assert_eq!(result["source"]["modified"], false);
    checked_file(&output, &literal("2", true, false)?, 2, false)?;
    assert_eq!(fs::read(&input)?, MIXED);
    Ok(())
}
#[test]
fn initialized_stdio_has_atomic_required_triple_full_history_and_opaque_refusal() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    qualify(&dir)?;
    let input = dir.path("stdio-style-original-λ.cdxml");
    let output = dir.path("stdio-style-保存.cdxml");
    let opaque = dir.path("stdio-opaque.cdxml");
    let opaque_copy = dir.path("stdio-opaque-copy.cdxml");
    fs::write(&input, MIXED)?;
    let raw = std::str::from_utf8(MIXED)?.replacen("<CDXML ", "<CDXML Private='retain' ", 1);
    fs::write(&opaque, &raw)?;
    let mut c = Client::start()?;
    initialize(&mut c)?;
    c.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let list = c.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(list["id"], 2);
    metadata(list["result"]["tools"].as_array().ok_or("tools")?, true)?;
    c.call(3, "document.open_cdxml", json!({"path":input}))?;
    let edit = c.call(4, COMBINED, set3(1, Some(24.5), Some(120.5), Some(2.0)))?;
    model(&edit, 3, (Some(24.5), Some(120.5), 2.0))?;
    assert_eq!(
        (edit["revision"].clone(), edit["undo_depth"].clone()),
        (json!(2), json!(1))
    );
    assert!(edit.get("file_operation").is_none() && edit.get("ui_file_operation").is_none());
    let before = c.call(5, "document.inspect", json!({}))?;
    assert_eq!(c.call(6, SET, set(2, json!(24.5), json!(120.5)))?, before);
    let mut missing = set3(2, None, None, None);
    missing
        .as_object_mut()
        .ok_or("params")?
        .remove("line_width_pt");
    denied(&mut c, 7, COMBINED, missing, "invalid_params")?;
    denied(
        &mut c,
        8,
        COMBINED,
        set3(1, None, None, Some(0.0)),
        "stale_revision",
    )?;
    let mut pair = set(2, json!(24.5), json!(120.5));
    pair["line_width_pt"] = json!(2);
    denied(&mut c, 9, SET, pair, "invalid_params")?;
    assert_eq!(c.call(10, "document.inspect", json!({}))?, before);
    let clear = c.call(11, COMBINED, set3(2, None, None, None))?;
    assert!(
        clear["document"].get("drawing_style").is_none()
            && clear["document"].get("drawing_settings").is_none()
    );
    model(
        &c.call(12, "history.undo", json!({}))?,
        3,
        (Some(24.5), Some(120.5), 2.0),
    )?;
    let original = c.call(13, "history.undo", json!({}))?;
    model(&original, 3, (None, None, 0.5))?;
    assert_eq!(original["file"]["dirty"], false);
    assert_eq!(original["source"]["modified"], false);
    c.call(14, "history.redo", json!({}))?;
    c.call(15, "document.save_cdxml", json!({"path":output}))?;
    checked_file(&output, &literal("2", false, true)?, 3, true)?;
    let reopened = c.call(16, "document.open_cdxml", json!({"path":output}))?;
    model(&reopened, 3, (Some(24.5), Some(120.5), 2.0))?;
    assert_eq!(reopened["revision"], 7);
    assert_eq!(reopened["undo_depth"], 0);
    c.call(17, "document.open_cdxml", json!({"path":opaque}))?;
    let before = c.call(18, "document.inspect", json!({}))?;
    denied(
        &mut c,
        19,
        COMBINED,
        set3(8, None, None, Some(0.0)),
        "unsupported_content",
    )?;
    assert_eq!(c.call(20, "document.inspect", json!({}))?, before);
    c.call(21, "document.save_cdxml", json!({"path":opaque_copy}))?;
    assert_eq!(fs::read(&opaque_copy)?, raw.as_bytes());
    assert_eq!(fs::read(&input)?, MIXED);
    c.finish()
}
#[test]
fn authenticated_loopback_publishes_only_real_style_edits_and_retains_failed_ink_snapshot()
-> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    qualify(&dir)?;
    let input = dir.path("socket-style-original-λ.cdxml");
    let output = dir.path("socket-style-化学.cdxml");
    fs::write(&input, MIXED)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let changed = Arc::clone(&callbacks);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        changed.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut client = BufReader::new(stream);
    assert_eq!(
        socket(&mut client, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    socket(
        &mut client,
        request("document.open_cdxml", json!({"path":input})),
    )?;
    socket(&mut client, request("document.inspect", json!({})))?;
    let n = callbacks.load(Ordering::SeqCst);
    let edit = socket(
        &mut client,
        request(COMBINED, set3(1, None, None, Some(2.0))),
    )?;
    assert!(edit.get("error").is_none(), "{edit}");
    model(&edit["result"], 3, (None, None, 2.0))?;
    let stable = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    assert_eq!(callbacks.load(Ordering::SeqCst), n + 1);
    let noop = socket(
        &mut client,
        request(COMBINED, set3(2, None, None, Some(2.0))),
    )?;
    assert_eq!(noop["result"], stable);
    let error = socket(
        &mut client,
        request(COMBINED, set3(1, None, None, Some(0.0))),
    )?;
    assert_eq!(error["error"]["code"], "stale_revision");
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        stable
    );
    assert_eq!(callbacks.load(Ordering::SeqCst), n + 1);
    let bond = socket(
        &mut client,
        request(
            "bond.set_order",
            json!({"expected_revision":2,"page_id":1,
        "fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
        ),
    )?;
    assert!(bond.get("error").is_none(), "{bond}");
    let before = socket(&mut client, request("document.inspect", json!({})))?["result"].clone();
    let count = callbacks.load(Ordering::SeqCst);
    let (whole, stamp) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.content().clone(), guard.capture_stamp())
    };
    let refused = socket(
        &mut client,
        request(COMBINED, set3(3, None, None, Some(3.0))),
    )?;
    assert_eq!(refused["error"]["code"], "unsupported_scale");
    assert_eq!(
        socket(&mut client, request("document.inspect", json!({})))?["result"],
        before
    );
    assert_eq!(callbacks.load(Ordering::SeqCst), count);
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &whole);
        assert!(guard.capture_stamp_matches(&stamp));
    }
    let saved = socket(
        &mut client,
        request("document.save_cdxml", json!({"path":output})),
    )?;
    assert!(saved.get("error").is_none(), "{saved}");
    checked_file(&output, &literal("2", true, false)?, 2, false)?;
    let open = socket(
        &mut client,
        request("document.open_cdxml", json!({"path":output})),
    )?;
    model(&open["result"], 2, (None, None, 2.0))?;
    assert_eq!(open["result"]["file"]["dirty"], false);
    assert_eq!(fs::read(&input)?, MIXED);
    server.shutdown()?;
    Ok(())
}
