//! Independent post-producer receiving; child of document_settings_wire.rs.
//! Source-only declaration: Root owns adoption, compilation and all execution.
use super::*;

fn spaced(revision: u64, pair: bool, width: f64, spacing: Value) -> Value {
    let mut p = set3(
        revision,
        if pair { Some(40.0) } else { None },
        if pair { Some(120.0) } else { None },
        Some(width),
    );
    p["bond_spacing_percent"] = spacing;
    p
}
fn original_spacing() -> TestResult<Vec<u8>> {
    Ok(std::str::from_utf8(MIXED)?
        .replacen("<CDXML ", "<CDXML BondSpacing='18.00' ", 1)
        .into_bytes())
}
fn spacing_literal(spacing: Option<&str>, width: &str, pair: bool, h: u8) -> TestResult<Vec<u8>> {
    let mut raw = String::from_utf8(original_spacing()?)?;
    raw = match spacing {
        Some(s) => raw.replacen("BondSpacing='18.00'", &format!("BondSpacing='{s}'"), 1),
        None => raw.replacen("BondSpacing='18.00'", "", 1),
    };
    raw = raw.replacen("LineWidth='.500'", &format!("LineWidth='{width}'"), 1);
    if pair {
        raw = raw.replacen(
            "CreationProgram=\"Bondline 0.1.0\">",
            "CreationProgram=\"Bondline 0.1.0\" BondLength=\"40\" ChainAngle=\"120\">",
            1,
        );
    }
    if h == 2 {
        raw = raw
            .replace("NumHydrogens='3'", "NumHydrogens='2'")
            .replace("NumHydrogens=\"3\"", "NumHydrogens=\"2\"")
            .replace("Order=\"1\"", "Order=\"2\"");
    }
    Ok(raw.into_bytes())
}
fn spacing_state(v: &Value, h: u8, width: f64, spacing: Option<f64>, pair: bool) -> TestResult {
    graph(v, &chemical(h))?;
    assert_eq!(
        v["document"]["pages"][0]["size"],
        json!({"width_pt":612.0,"height_pt":792.0})
    );
    let mut style = json!({"line_width_pt":width});
    if let Some(s) = spacing {
        style["bond_spacing_percent"] = json!(s);
    }
    assert_eq!(v["document"]["drawing_style"], style);
    if pair {
        assert_eq!(
            v["document"]["drawing_settings"],
            json!({"bond_length_pt":40.0,"chain_angle_degrees":120.0})
        );
    } else {
        assert!(v["document"].get("drawing_settings").is_none());
    }
    Ok(())
}
fn spacing_roots(raw: &[u8]) -> TestResult<Value> {
    let mut child = Process::new(python())
        .args([
            "-X",
            "utf8",
            "-B",
            "-c",
            r#"
import json,sys,xml.parsers.expat as expat
from decimal import Decimal
raw=sys.stdin.buffer.read(1048577);assert 0<len(raw)<=1048576
rows=[]
p=expat.ParserCreate(namespace_separator='|');p.ordered_attributes=True
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda n,a:rows.append((n,list(zip(a[::2],a[1::2]))))
p.Parse(raw,True)
assert [n for n,_ in rows]==['CDXML','page','fragment','n','n','b']
root=rows[0][1];attrs=dict(root)
assert set(attrs)<=set(['CreationProgram','BondLength','ChainAngle','LineWidth','BondSpacing'])
assert attrs['CreationProgram']=='Bondline 0.1.0'
numbers={}
for name in ['BondLength','ChainAngle','LineWidth','BondSpacing']:
 if name not in attrs:continue
 number=Decimal(attrs[name]);assert number.is_finite()
 if name=='BondSpacing':assert Decimal(1)<=number<=Decimal(100)
 elif name=='ChainAngle':assert Decimal(1)<=number<=Decimal(179)
 else:assert Decimal(0)<number<=Decimal(100000)
 numbers[name]=str(number)
print(json.dumps({'root':root,'decimal':numbers},separators=(',',':')))
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("spacing reader stdin")?
        .write_all(raw)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "spacing reader: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn checked_spacing(
    path: &Path,
    expected: &[u8],
    spacing: Option<&str>,
    width: &str,
    pair: bool,
    h: u8,
) -> TestResult {
    let raw = fs::read(path)?;
    assert_eq!(raw, expected, "whole original/current source bytes first");
    let mut attrs = Vec::new();
    let mut decimal = json!({"LineWidth":if width==".500"{"0.500"}else{width}});
    if let Some(s) = spacing {
        attrs.push(json!(["BondSpacing", s]));
        decimal["BondSpacing"] = json!(s);
    }
    attrs.push(json!(["LineWidth", width]));
    attrs.push(json!(["CreationProgram", "Bondline 0.1.0"]));
    if pair {
        attrs.extend([json!(["BondLength", "40"]), json!(["ChainAngle", "120"])]);
        decimal["BondLength"] = json!("40");
        decimal["ChainAngle"] = json!("120");
    }
    assert_eq!(
        spacing_roots(&raw)?,
        json!({"root":attrs,"decimal":decimal}),
        "spacing before chemistry"
    );
    consume(
        path,
        &chemical(h),
        if h == 2 { "C=C" } else { "CC" },
        if h == 2 { "C2H4" } else { "C2H6" },
    )
}
fn qualify_spacing(dir: &support::SdfTestDirectory) -> TestResult {
    let manual = dir.path("manual-spacing-λ.cdxml");
    fs::write(&manual, original_spacing()?)?;
    checked_spacing(
        &manual,
        &original_spacing()?,
        Some("18.00"),
        ".500",
        false,
        3,
    )?;
    let raw = spacing_literal(Some("24"), "2", true, 2)?;
    fs::write(&manual, &raw)?;
    checked_spacing(&manual, &raw, Some("24"), "2", true, 2)?;
    for bad in ["NaN", "0", "101"] {
        let raw = std::str::from_utf8(&original_spacing()?)?.replacen("18.00", bad, 1);
        assert!(spacing_roots(raw.as_bytes()).is_err(), "{bad}");
    }
    Ok(())
}

#[test]
fn actual_cli_spacing_preserve_set_clear_history_and_unicode_source_roundtrip() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    qualify_spacing(&dir)?;
    let list = Process::new(env!("CARGO_BIN_EXE_bondline-cli"))
        .arg("commands")
        .output()?;
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    metadata(&serde_json::from_slice::<Vec<Value>>(&list.stdout)?, false)?;
    let input = dir.path("cli-spacing-original-λ.cdxml");
    let omitted = dir.path("cli-spacing-preserved-化学.cdxml");
    let numeric = dir.path("cli-spacing-set-λ.cdxml");
    let cleared = dir.path("cli-spacing-clear-λ.cdxml");
    let output = dir.path("cli-spacing-double-保存.cdxml");
    let source = original_spacing()?;
    fs::write(&input, &source)?;
    let state = cli(&[
        request("document.open_cdxml", json!({"path":input})),
        request(COMBINED, set3(1, Some(40.0), Some(120.0), Some(2.0))),
        request("document.save_cdxml", json!({"path":omitted})),
        request(COMBINED, spaced(2, true, 2.0, json!(24))),
        request("document.save_cdxml", json!({"path":numeric})),
        request(COMBINED, spaced(3, true, 2.0, Value::Null)),
        request("document.save_cdxml", json!({"path":cleared})),
        request("history.undo", json!({})),
        request("history.redo", json!({})),
        request(COMBINED, spaced(6, true, 2.0, json!(18))),
        request(
            "bond.set_order",
            json!({"expected_revision":7,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
        ),
        request("document.save_cdxml", json!({"path":output})),
        request("document.open_cdxml", json!({"path":output})),
    ])?;
    spacing_state(&state, 2, 2.0, Some(18.0), true)?;
    assert_eq!(
        (
            state["revision"].clone(),
            state["undo_depth"].clone(),
            state["redo_depth"].clone()
        ),
        (json!(9), json!(0), json!(0))
    );
    assert_eq!(state["file"]["dirty"], false);
    assert_eq!(state["source"]["modified"], false);
    checked_spacing(
        &omitted,
        &spacing_literal(Some("18.00"), "2", true, 3)?,
        Some("18.00"),
        "2",
        true,
        3,
    )?;
    checked_spacing(
        &numeric,
        &spacing_literal(Some("24"), "2", true, 3)?,
        Some("24"),
        "2",
        true,
        3,
    )?;
    checked_spacing(
        &cleared,
        &spacing_literal(None, "2", true, 3)?,
        None,
        "2",
        true,
        3,
    )?;
    // Returning to the original numeric value restores its original lexical spelling.
    checked_spacing(
        &output,
        &spacing_literal(Some("18.00"), "2", true, 2)?,
        Some("18.00"),
        "2",
        true,
        2,
    )?;
    assert_eq!(fs::read(&input)?, source);
    Ok(())
}

#[test]
fn initialized_stdio_spacing_tristate_noops_errors_and_opaque_source_are_exact() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    qualify_spacing(&dir)?;
    let input = dir.path("stdio-spacing-original-λ.cdxml");
    let output = dir.path("stdio-spacing-保存.cdxml");
    let opaque = dir.path("stdio-spacing-opaque.cdxml");
    let copy = dir.path("stdio-spacing-opaque-copy.cdxml");
    let source = original_spacing()?;
    fs::write(&input, &source)?;
    let foreign = std::str::from_utf8(&source)?.replacen("<CDXML ", "<CDXML Private='retain' ", 1);
    fs::write(&opaque, &foreign)?;
    let mut c = Client::start()?;
    initialize(&mut c)?;
    c.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))?;
    let rows = c.replies.recv_timeout(Duration::from_secs(10))??;
    assert_eq!(rows["id"], 2);
    metadata(rows["result"]["tools"].as_array().ok_or("tools")?, true)?;
    c.call(3, "document.open_cdxml", json!({"path":input}))?;
    let original = c.call(4, "document.inspect", json!({}))?;
    assert_eq!(
        c.call(5, COMBINED, set3(1, None, None, Some(0.5)))?,
        original
    );
    let one = c.call(6, COMBINED, spaced(1, false, 0.5, json!(1)))?;
    spacing_state(&one, 3, 0.5, Some(1.0), false)?;
    assert_eq!(one["revision"], 2);
    assert_eq!(one["undo_depth"], 1);
    let stable = c.call(7, "document.inspect", json!({}))?;
    for (index, bad) in [
        json!(0),
        json!(100.001),
        json!(true),
        json!("18"),
        json!([]),
        json!({}),
    ]
    .into_iter()
    .enumerate()
    {
        denied(
            &mut c,
            100 + index as u64,
            COMBINED,
            spaced(2, false, 0.5, bad),
            "invalid_params",
        )?;
    }
    let mut missing = spaced(2, false, 0.5, json!(18));
    missing
        .as_object_mut()
        .ok_or("params")?
        .remove("line_width_pt");
    denied(&mut c, 110, COMBINED, missing, "invalid_params")?;
    denied(
        &mut c,
        111,
        COMBINED,
        spaced(1, false, 0.5, json!(0)),
        "stale_revision",
    )?;
    assert_eq!(c.call(112, "document.inspect", json!({}))?, stable);
    let hundred = c.call(113, COMBINED, spaced(2, false, 0.5, json!(100)))?;
    spacing_state(&hundred, 3, 0.5, Some(100.0), false)?;
    let before = c.call(114, "document.inspect", json!({}))?;
    assert_eq!(
        c.call(115, COMBINED, set3(3, None, None, Some(0.5)))?,
        before
    );
    let clear = c.call(116, COMBINED, spaced(3, false, 0.5, Value::Null))?;
    spacing_state(&clear, 3, 0.5, None, false)?;
    assert_eq!(clear["revision"], 4);
    let undo = c.call(117, "history.undo", json!({}))?;
    spacing_state(&undo, 3, 0.5, Some(100.0), false)?;
    assert_eq!(
        (undo["undo_depth"].clone(), undo["redo_depth"].clone()),
        (json!(2), json!(1))
    );
    let twice = c.call(118, "history.undo", json!({}))?;
    spacing_state(&twice, 3, 0.5, Some(1.0), false)?;
    c.call(119, "history.redo", json!({}))?;
    c.call(120, "document.save_cdxml", json!({"path":output}))?;
    checked_spacing(
        &output,
        &spacing_literal(Some("100"), ".500", false, 3)?,
        Some("100"),
        ".500",
        false,
        3,
    )?;
    let reopened = c.call(121, "document.open_cdxml", json!({"path":output}))?;
    spacing_state(&reopened, 3, 0.5, Some(100.0), false)?;
    assert_eq!(reopened["revision"], 8);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["source"]["modified"], false);
    c.call(122, "document.open_cdxml", json!({"path":opaque}))?;
    let before = c.call(123, "document.inspect", json!({}))?;
    denied(
        &mut c,
        124,
        COMBINED,
        spaced(9, false, 0.5, json!(18)),
        "unsupported_content",
    )?;
    assert_eq!(c.call(125, "document.inspect", json!({}))?, before);
    c.call(126, "document.save_cdxml", json!({"path":copy}))?;
    assert_eq!(fs::read(&copy)?, foreign.as_bytes());
    assert_eq!(fs::read(&input)?, source);
    c.finish()
}

#[test]
fn authenticated_loopback_spacing_bounds_noops_callbacks_history_and_saved_bytes() -> TestResult {
    let dir = support::SdfTestDirectory::new()?;
    qualify_spacing(&dir)?;
    let input = dir.path("socket-spacing-original-λ.cdxml");
    let output = dir.path("socket-spacing-clear-化学.cdxml");
    let source = original_spacing()?;
    fs::write(&input, &source)?;
    let engine: SharedEngine = Arc::new(Mutex::new(Engine::default()));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let changed = Arc::clone(&callbacks);
    let server = LoopbackServer::bind(0, Arc::clone(&engine), move || {
        changed.fetch_add(1, Ordering::SeqCst);
    })?;
    let stream = TcpStream::connect_timeout(&server.address(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut c = BufReader::new(stream);
    assert_eq!(
        socket(&mut c, json!({"token":server.token()}))?,
        json!({"authenticated":true})
    );
    let open = socket(
        &mut c,
        request("document.open_cdxml", json!({"path":input})),
    )?;
    assert!(open.get("error").is_none(), "{open}");
    let baseline = callbacks.load(Ordering::SeqCst);
    for (revision, value) in [(1, 1), (2, 100)] {
        let edit = socket(
            &mut c,
            request(COMBINED, spaced(revision, false, 2.0, json!(value))),
        )?;
        assert!(edit.get("error").is_none(), "{edit}");
        spacing_state(&edit["result"], 3, 2.0, Some(f64::from(value)), false)?;
        assert_eq!(edit["result"]["revision"], revision + 1);
    }
    let before = socket(&mut c, request("document.inspect", json!({})))?["result"].clone();
    let count = callbacks.load(Ordering::SeqCst);
    assert_eq!(count, baseline + 2);
    let (whole, stamp) = {
        let guard = engine.lock().map_err(|_| "engine")?;
        (guard.content().clone(), guard.capture_stamp())
    };
    let noop = socket(&mut c, request(COMBINED, set3(3, None, None, Some(2.0))))?;
    assert_eq!(noop["result"], before);
    for (revision, value, code) in [(3, 0, "invalid_params"), (2, 18, "stale_revision")] {
        let error = socket(
            &mut c,
            request(COMBINED, spaced(revision, false, 2.0, json!(value))),
        )?;
        assert_eq!(error["error"]["code"], code);
    }
    assert_eq!(
        socket(&mut c, request("document.inspect", json!({})))?["result"],
        before
    );
    assert_eq!(callbacks.load(Ordering::SeqCst), count);
    {
        let guard = engine.lock().map_err(|_| "engine")?;
        assert_eq!(guard.content(), &whole);
        assert!(guard.capture_stamp_matches(&stamp));
    }
    let clear = socket(
        &mut c,
        request(COMBINED, spaced(3, false, 2.0, Value::Null)),
    )?;
    assert!(clear.get("error").is_none(), "{clear}");
    spacing_state(&clear["result"], 3, 2.0, None, false)?;
    let save = socket(
        &mut c,
        request("document.save_cdxml", json!({"path":output})),
    )?;
    assert!(save.get("error").is_none(), "{save}");
    checked_spacing(
        &output,
        &spacing_literal(None, "2", false, 3)?,
        None,
        "2",
        false,
        3,
    )?;
    let undo = socket(&mut c, request("history.undo", json!({})))?;
    spacing_state(&undo["result"], 3, 2.0, Some(100.0), false)?;
    let redo = socket(&mut c, request("history.redo", json!({})))?;
    spacing_state(&redo["result"], 3, 2.0, None, false)?;
    assert_eq!(redo["result"]["file"]["dirty"], false);
    let reopen = socket(
        &mut c,
        request("document.open_cdxml", json!({"path":output})),
    )?;
    spacing_state(&reopen["result"], 3, 2.0, None, false)?;
    assert_eq!(reopen["result"]["revision"], 7);
    assert_eq!(reopen["result"]["undo_depth"], 0);
    assert_eq!(reopen["result"]["source"]["modified"], false);
    assert_eq!(fs::read(&input)?, source);
    server.shutdown()?;
    Ok(())
}
