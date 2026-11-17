//! Independent native-sized App frames for complete-source SMILES record authoring.
use super::{Fixture, TestResult};
use eframe::egui;
use serde_json::json;
use std::fs;

const SOURCE: &[u8] = b"CC first\r\nC1.N.C1 \t\0\xffPRIVATE-META\r\nCC duplicate\tname";

#[test]
fn actual_file_menu_opens_a_smiles_record_import_form_without_replacing_source() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = fixture.directory.path("native-records-原.smi");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_smiles", json!({"path":path}))?;
    fixture.warm();
    let original = fixture.state()?;
    fixture.choose("File")?;
    // Actual missing File-menu role is the intended initial runtime RED.
    fixture.node("Import SMILES records…", &[egui::accesskit::Role::Button])?;
    fixture.choose("Import SMILES records…")?;
    let shown = fixture.warm();
    assert!(shown.shapes.iter().any(|paint| matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text() == "Import SMILES records"
            && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))));
    let mut after = fixture.state()?;
    assert_eq!(
        after["ui_file"]["request"]["action"],
        "import_smiles_records"
    );
    after["ui_file"] = original["ui_file"].clone();
    assert_eq!(after, original);
    assert_eq!(fs::read(path)?, SOURCE);
    Ok(())
}

// Additive actual-App expectations preserve the frozen1,423-byte menu declaration.
use super::{button, canvas, finish_actual_file_workflow, key, projected, tap};
use egui::{
    Event, Key, Modifiers,
    accesskit::{Role, Toggled},
};
use serde_json::Value;
use std::{
    collections::HashSet,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, UNIX_EPOCH},
};

const EDITED: &[u8] = b"CC first\r\n[CH2]=[CH2].[NH3] \t\0\xffPRIVATE-META\r\nCC duplicate\tname";
fn has_text(output: &egui::FullOutput, text: &str) -> bool {
    output.shapes.iter().any(|paint| matches!(&paint.shape,egui::Shape::Text(shape)
        if shape.galley.text()==text && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect())))
}
#[track_caller]
fn wait_for(
    f: &mut Fixture,
    ready: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = f.frame(vec![]);
        if ready(f, &output) {
            return Ok(output);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "record UI did not become ready at {}: order_busy={}, file_busy={}, executor_active={}, revision={:?}, error={:?}",
                std::panic::Location::caller(),
                f.app.drawing.order.is_busy(),
                f.app.files.is_busy(),
                f.app.executor.has_active_work(),
                f.state().ok().and_then(|state| state["revision"].as_u64()),
                f.app.error,
            )
            .into());
        }
        std::thread::yield_now();
    }
}
fn node_any(f: &Fixture, label: &str, roles: &[Role]) -> TestResult<egui::accesskit::NodeId> {
    let mut stack = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    let mut seen = HashSet::new();
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = f.nodes.get(&id)
        {
            let named = node.name() == Some(label)
                || node
                    .labelled_by()
                    .iter()
                    .any(|id| f.nodes.get(id).is_some_and(|n| n.name() == Some(label)));
            if roles.contains(&node.role()) && named {
                return Ok(id);
            }
            stack.extend_from_slice(node.children());
        }
    }
    Err(format!("reachable {label} node missing").into())
}
fn value(f: &Fixture, label: &str) -> TestResult<String> {
    let id = node_any(f, label, &[Role::TextInput, Role::MultilineTextInput])?;
    let mut stack = vec![id];
    let mut text = String::new();
    while let Some(id) = stack.pop() {
        let node = f.nodes.get(&id).ok_or("text subtree")?;
        if node.role() == Role::InlineTextBox {
            text.push_str(node.value().ok_or("inline text")?)
        }
        stack.extend(node.children().iter().rev().copied());
    }
    Ok(text)
}
fn set_field(f: &mut Fixture, label: &str, text: &str) -> TestResult {
    let (id, _) = f.node(label, &[Role::TextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::SetValue,
            target: id,
            data: Some(egui::accesskit::ActionData::Value(text.into())),
        },
    )]);
    wait_for(f, |f, _| value(f, label).is_ok_and(|v| v == text))?;
    assert_eq!(value(f, label)?, text);
    Ok(())
}
fn radio(f: &mut Fixture, label: &str) -> TestResult {
    f.warm();
    let (_, bounds) = f.node(label, &[Role::RadioButton])?;
    f.frame(vec![
        Event::PointerMoved(bounds.center()),
        button(bounds.center(), true),
    ]);
    f.frame(vec![button(bounds.center(), false)]);
    f.warm();
    let (id, _) = f.node(label, &[Role::RadioButton])?;
    assert_eq!(
        f.nodes.get(&id).ok_or("radio")?.toggled(),
        Some(Toggled::True)
    );
    Ok(())
}
fn shortcut(f: &mut Fixture, keycode: Key, shift: bool) -> egui::FullOutput {
    let m = Modifiers {
        shift,
        ..Modifiers::COMMAND
    };
    f.frame(vec![key(keycode, true, m), key(keycode, false, m)])
}
fn open_source(f: &mut Fixture, source: &[u8]) -> TestResult<PathBuf> {
    let path = f.directory.path("native-records-原.smi");
    fs::write(&path, source)?;
    f.command("document.open_smiles", json!({"path":path}))?;
    f.warm();
    Ok(path)
}
fn open_form(f: &mut Fixture) -> TestResult {
    f.choose("File")?;
    f.choose("Import SMILES records…")?;
    wait_for(f, |f, o| {
        has_text(o, "Import SMILES records") && value(f, "Import record number").is_ok()
    })?;
    Ok(())
}
fn geometry(f: &mut Fixture) -> TestResult {
    for (label, text) in [
        ("Bond length (pt)", "48"),
        ("Origin X (pt)", "72"),
        ("Origin Y (pt)", "72"),
        ("Page width (pt)", "595.2755905511812"),
        ("Page height (pt)", "841.8897637795276"),
    ] {
        set_field(f, label, text)?;
    }
    Ok(())
}
fn import_all(f: &mut Fixture) -> TestResult {
    open_form(f)?;
    radio(f, "All records")?;
    geometry(f)?;
    f.choose("Import")?;
    wait_for(f, |f, o| {
        f.state().is_ok_and(|s| s["document"].is_object())
            && !f.app.files.is_busy()
            && !f.app.executor.has_active_work()
            && !has_text(o, "Import SMILES records")
    })?;
    Ok(())
}
fn current_bytes(f: &Fixture) -> TestResult<Vec<u8>> {
    Ok(f.app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .content()
        .smiles_file()?
        .as_bytes()
        .to_vec())
}
fn preserve(f: &Fixture, old: &Value) -> TestResult {
    let mut current = f.state()?;
    current["ui_file"] = old["ui_file"].clone();
    assert_eq!(&current, old);
    Ok(())
}
fn pan(f: &mut Fixture) -> TestResult {
    f.frame(tap(Key::F5));
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    wait_for(f, |f, _| {
        ["Horizontal position", "Vertical position"]
            .iter()
            .all(|label| f.node(label, &[Role::Slider]).is_ok())
    })?;
    for label in ["Horizontal position", "Vertical position"] {
        let (id, _) = f.node(label, &[Role::Slider])?;
        f.frame(vec![Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::SetValue,
                target: id,
                data: Some(egui::accesskit::ActionData::NumericValue(200.0)),
            },
        )]);
    }
    assert_eq!(f.view()?["center_x_pt"], 200.0);
    assert_eq!(f.view()?["center_y_pt"], 200.0);
    Ok(())
}
fn bond_point(f: &Fixture, output: &egui::FullOutput, fragment: usize) -> TestResult<egui::Pos2> {
    let state = f.state()?;
    let p = &state["document"]["pages"][0];
    let g = &p["fragments"][fragment];
    let b = &g["bonds"][0];
    let atoms = g["atoms"].as_array().ok_or("atoms")?;
    let a = atoms
        .iter()
        .find(|a| a["id"] == b["begin"])
        .ok_or("begin")?;
    let z = atoms.iter().find(|a| a["id"] == b["end"]).ok_or("end")?;
    let midpoint = [
        (a["position"]["x_pt"].as_f64().ok_or("x")? + z["position"]["x_pt"].as_f64().ok_or("x")?)
            / 2.0,
        (a["position"]["y_pt"].as_f64().ok_or("y")? + z["position"]["y_pt"].as_f64().ok_or("y")?)
            / 2.0,
    ];
    let view = f.view()?;
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    Ok(projected(
        canvas(output)?,
        [
            p["size"]["width_pt"].as_f64().ok_or("width")?,
            p["size"]["height_pt"].as_f64().ok_or("height")?,
        ],
        view["zoom_percent"].as_f64().ok_or("actual zoom")?,
        center,
        midpoint,
    ))
}
fn assert_single_paint(f: &Fixture, o: &egui::FullOutput, count: usize) -> TestResult {
    let paths = o
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Path(p)
                if p.closed && p.points.len() == 4 && p.fill == egui::Color32::BLACK =>
            {
                Some((s, p))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        paths.len(),
        count,
        "actual bond quads, not only nonblank paper"
    );
    for at in 0..count {
        let p = bond_point(f, o, at)?;
        assert!(
            paths.iter().any(|(s, path)| s.clip_rect.contains(p)
                && egui::Rect::from_points(&path.points).contains(p)),
            "missing painted source bond {at}"
        );
    }
    Ok(())
}
fn hold_capture_slots(f: &Fixture) -> TestResult<[bondline_mcp::UiCaptureWork; 2]> {
    let stamp = f.app.engine.lock().map_err(|_| "engine")?.capture_stamp();
    let params = || {
        serde_json::from_value::<bondline_commands::RenderCapture>(
            json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
        )
    };
    Ok([
        f.app
            .executor
            .prepare_ui_capture(&stamp, params()?)
            .map_err(|e| e.message)?,
        f.app
            .executor
            .prepare_ui_capture(&stamp, params()?)
            .map_err(|e| e.message)?,
    ])
}
fn hold_raw_slots(f: &Fixture) -> TestResult<[bondline_mcp::UiSmilesWork; 2]> {
    let e = f.app.engine.lock().map_err(|_| "engine")?;
    let stamp = e.capture_stamp();
    let epoch = e.state().close_epoch;
    let revision = e.state().revision;
    drop(e);
    let p = || {
        serde_json::from_value::<bondline_commands::SmilesRecordInspect>(
            json!({"expected_revision":revision,"record_id":1}),
        )
    };
    Ok([
        f.app
            .executor
            .prepare_ui_smiles(&stamp, epoch, p()?)
            .map_err(|e| e.message)?,
        f.app
            .executor
            .prepare_ui_smiles(&stamp, epoch, p()?)
            .map_err(|e| e.message)?,
    ])
}
fn finish_order(f: &mut Fixture, revision: u64) -> TestResult {
    wait_for(f, |f, _| {
        f.state().is_ok_and(|s| s["revision"] == revision)
            && !f.app.files.is_busy()
            && !f.app.executor.has_active_work()
            // Executor completion precedes the owner's actual thread retirement.
            && !f.app.drawing.order.is_busy()
    })?;
    f.warm();
    Ok(())
}
fn choose_file(f: &mut Fixture, action: &str, path: &Path) -> TestResult {
    wait_for(f, |f, _| value(f, "File name").is_ok())?;
    set_field(
        f,
        "File name",
        path.file_name()
            .and_then(|s| s.to_str())
            .ok_or("UTF8 name")?,
    )?;
    f.choose(action)?;
    finish_actual_file_workflow(f)
}
fn consume(script: &str, args: &[String], bytes: &[u8]) -> TestResult<Value> {
    let mut child = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .arg("-B")
        .arg(format!(
            "{}/../io/tests/{script}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}
fn consume_saved(path: &Path, state: &Value) -> TestResult {
    let bytes = fs::read(path)?;
    assert_eq!(bytes, EDITED);
    let raw = consume("smiles_file_raw_oracle.py", &[], &bytes)?;
    assert_eq!(raw["byte_length"], 62);
    assert_eq!(raw["records"][1]["notation_range"], json!([10, 27]));
    assert_eq!(
        raw["records"][1]["application_data_hex"],
        "00ff505249564154452d4d455441"
    );
    assert_eq!(raw["records"][2]["line_ending_range"], json!([62, 62]));
    for span in [0..2, 10..27, 45..47] {
        assert_eq!(
            consume(
                "smiles_oracle.py",
                &[],
                bytes.get(span).ok_or("literal notation")?
            )?["accepted"],
            true
        );
    }
    let options = json!({"atoms":[[6,null,2],[6,null,2],[7,null,3]],"bonds":[[1,2,2]],"components":[[1,2],[3]],"formula":"C2H7N"});
    let checked = consume(
        "smiles_authoring_oracle.py",
        &["[CH2]=[CH2].[NH3]".into(), options.to_string()],
        &serde_json::to_vec(&state["document"]["pages"][0]["fragments"][1])?,
    )?;
    assert_eq!(checked["no_assigned_stereo"], true);
    assert_eq!(checked["no_radicals"], true);
    Ok(())
}
fn retire_import_with_old_surface_input(f: &mut Fixture, area: egui::Rect) -> TestResult {
    let point = projected(
        area,
        [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
        100.0,
        None,
        [96.0, 72.0],
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        // This packet belongs to the still-visible raw-file/import surface.
        // F5 makes its following target meaningful if it is wrongly replayed on
        // the new Drawing. A fresh equivalent key is proved effective below.
        let mut packet = tap(Key::F5);
        packet.push(Event::PointerMoved(point));
        packet.extend(tap(Key::Num2));
        let shown = f.frame(packet);
        if f.state()?["document"].is_object() && !has_text(&shown, "Import SMILES records") {
            break;
        }
        if Instant::now() >= deadline {
            return Err("real import did not retire to Drawing".into());
        }
        std::thread::yield_now();
    }
    wait_for(f, |f, _| {
        !f.app.files.is_busy() && !f.app.executor.has_active_work()
    })?;
    assert_eq!(f.state()?["revision"], 2);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(current_bytes(f)?, SOURCE);
    Ok(())
}

#[test]
fn all_records_paint_and_key_order_waits_for_a_worker_before_history_and_whole_source_save()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = open_source(&mut f, SOURCE)?;
    open_form(&mut f)?;
    let (id, _) = f.node("One record", &[Role::RadioButton])?;
    assert_eq!(
        f.nodes.get(&id).ok_or("one radio")?.toggled(),
        Some(Toggled::True)
    );
    assert_eq!(value(&f, "Import record number")?, "1");
    for (label, text) in [
        ("Bond length (pt)", "48"),
        ("Origin X (pt)", "72"),
        ("Origin Y (pt)", "72"),
        ("Page width (pt)", "595.2755905511812"),
        ("Page height (pt)", "841.8897637795276"),
    ] {
        assert_eq!(value(&f, label)?, text);
    }
    radio(&mut f, "All records")?;
    let id = node_any(&f, "Import record number", &[Role::TextInput])?;
    assert!(f.nodes.get(&id).ok_or("disabled record")?.is_disabled());
    let old_area = canvas(&f.warm())?;
    f.choose("Import")?;
    retire_import_with_old_surface_input(&mut f, old_area)?;
    let imported = f.state()?;
    assert_eq!(
        imported["smiles_file"]["projected_records"],
        json!([1, 2, 3])
    );
    assert_eq!(imported["undo_depth"], 0);
    assert_eq!(imported["file"]["dirty"], false);
    assert!(
        !f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .is_fully_mapped()
    );
    pan(&mut f)?;
    assert_eq!(f.state()?, imported);
    let shown = f.warm();
    assert!(has_text(&shown, "SMILES"));
    assert!(has_text(&shown, "3 of 3 SMILES records"));
    assert!(has_text(&shown, "7 atoms"));
    assert!(has_text(&shown, "3 bonds"));
    assert_single_paint(&f, &shown, 3)?;
    let point = bond_point(&f, &shown, 1)?;
    let old_time = UNIX_EPOCH + Duration::from_secs(946684800);
    fs::OpenOptions::new()
        .write(true)
        .open(&input)?
        .set_times(fs::FileTimes::new().set_modified(old_time))?;
    let slots = hold_capture_slots(&f)?;
    let before = f.state()?;
    let mut packet = vec![Event::PointerMoved(point)];
    packet.extend(tap(Key::Num2));
    packet.extend([
        key(Key::S, true, Modifiers::COMMAND),
        key(Key::S, false, Modifiers::COMMAND),
    ]);
    f.frame(packet);
    let busy = f.warm();
    assert_eq!(f.state()?, before);
    assert_eq!(current_bytes(&f)?, SOURCE);
    assert_eq!(fs::metadata(&input)?.modified()?, old_time);
    let message = f
        .app
        .error
        .as_deref()
        .ok_or("blocked Save needs visible busy feedback")?;
    assert!(has_text(&busy, message));
    drop(slots);
    finish_order(&mut f, 3)?;
    assert_eq!(current_bytes(&f)?, EDITED);
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(f.state()?["file"]["dirty"], true);
    let output = f.directory.path("native-edited-λ-化学.smi");
    shortcut(&mut f, Key::S, true);
    choose_file(&mut f, "Save", &output)?;
    assert_eq!(fs::read(&output)?, EDITED);
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(f.state()?["source"]["modified"], true);
    shortcut(&mut f, Key::Z, false);
    assert_eq!(current_bytes(&f)?, SOURCE);
    assert_eq!(f.state()?["file"]["dirty"], true);
    assert_eq!(f.state()?["source"]["modified"], false);
    shortcut(&mut f, Key::Z, true);
    assert_eq!(current_bytes(&f)?, EDITED);
    assert_eq!(f.state()?["file"]["dirty"], false);
    shortcut(&mut f, Key::O, false);
    choose_file(&mut f, "Open", &output)?;
    assert_eq!(f.state()?["document"], Value::Null);
    assert_eq!(
        f.state()?["smiles_file"],
        json!({"format":"smiles","byte_length":62,"record_count":3})
    );
    import_all(&mut f)?;
    let reimported = f.state()?;
    assert_eq!(reimported["revision"], 7);
    assert_eq!(reimported["undo_depth"], 0);
    assert_eq!(reimported["source"]["modified"], false);
    consume_saved(&output, &reimported)?;
    Ok(())
}

#[test]
fn one_record_double_tool_click_is_deferred_and_other_authoring_remains_blocked() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    open_source(&mut f, SOURCE)?;
    open_form(&mut f)?;
    set_field(&mut f, "Import record number", "2")?;
    geometry(&mut f)?;
    f.choose("Import")?;
    wait_for(&mut f, |f, o| {
        f.state().is_ok_and(|s| s["revision"] == 2)
            && !f.app.files.is_busy()
            && !f.app.executor.has_active_work()
            && !has_text(o, "Import SMILES records")
    })?;
    pan(&mut f)?;
    let shown = f.warm();
    assert!(has_text(&shown, "1 of 3 SMILES records"));
    assert_single_paint(&f, &shown, 1)?;
    assert_eq!(f.state()?["smiles_file"]["projected_records"], json!([2]));
    let before = f.state()?;
    let area = canvas(&shown)?;
    let free = projected(
        area,
        [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
        100.0,
        Some([200.0, 200.0]),
        [220.0, 300.0],
    );
    f.frame(vec![
        Event::PointerMoved(free),
        button(free, true),
        Event::PointerMoved(free + egui::vec2(48.0, 0.0)),
        button(free + egui::vec2(48.0, 0.0), false),
    ]);
    assert_eq!(f.state()?, before);
    let atom = &before["document"]["pages"][0]["fragments"][0]["atoms"][0];
    let anchor = projected(
        area,
        [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
        100.0,
        Some([200.0, 200.0]),
        [
            atom["position"]["x_pt"].as_f64().ok_or("x")?,
            atom["position"]["y_pt"].as_f64().ok_or("y")?,
        ],
    );
    f.frame(vec![
        Event::PointerMoved(anchor),
        button(anchor, true),
        Event::PointerMoved(anchor + egui::vec2(0.0, 48.0)),
        button(anchor + egui::vec2(0.0, 48.0), false),
    ]);
    assert_eq!(f.state()?, before);
    assert_eq!(current_bytes(&f)?, SOURCE);
    f.choose("Page")?;
    let rotate = node_any(&f, "Rotate page", &[Role::Button])?;
    assert!(f.nodes.get(&rotate).ok_or("page control")?.is_disabled());
    f.frame(tap(Key::Escape));
    f.choose("Double bond")?;
    let shown = f.warm();
    let point = bond_point(&f, &shown, 0)?;
    let slots = hold_capture_slots(&f)?;
    f.frame(vec![
        Event::PointerMoved(point),
        button(point, true),
        button(point, false),
    ]);
    f.warm();
    assert_eq!(f.state()?, before);
    assert_eq!(current_bytes(&f)?, SOURCE);
    drop(slots);
    finish_order(&mut f, 3)?;
    assert_eq!(current_bytes(&f)?, EDITED);
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(
        f.state()?["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fragments")?
            .len(),
        1
    );
    Ok(())
}

#[test]
fn unsupported_all_keeps_source_and_form_then_explicit_one_record_retry_succeeds() -> TestResult {
    const LATER: &[u8] = b"CC chosen\r\ns1cccc1 unsupported\t\0\xff";
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = open_source(&mut f, LATER)?;
    let before = f.state()?;
    open_form(&mut f)?;
    set_field(&mut f, "Import record number", "2")?;
    f.choose("Cancel")?;
    finish_actual_file_workflow(&mut f)?;
    preserve(&f, &before)?;
    open_form(&mut f)?;
    geometry(&mut f)?;
    radio(&mut f, "All records")?;
    f.choose("Import")?;
    wait_for(&mut f, |f, o| {
        !f.app.files.is_busy()
            && !f.app.executor.has_active_work()
            && has_text(o, "Import SMILES records")
    })?;
    preserve(&f, &before)?;
    assert_eq!(current_bytes(&f)?, LATER);
    assert_eq!(fs::read(&path)?, LATER);
    let (id, _) = f.node("All records", &[Role::RadioButton])?;
    assert_eq!(
        f.nodes.get(&id).ok_or("all")?.toggled(),
        Some(Toggled::True)
    );
    assert_eq!(value(&f, "Import record number")?, "1");
    assert_eq!(value(&f, "Bond length (pt)")?, "48");
    assert_eq!(value(&f, "Origin X (pt)")?, "72");
    assert_eq!(value(&f, "Origin Y (pt)")?, "72");
    radio(&mut f, "One record")?;
    set_field(&mut f, "Import record number", "1")?;
    f.choose("Import")?;
    wait_for(&mut f, |f, o| {
        f.state().is_ok_and(|s| s["revision"] == 2)
            && !f.app.files.is_busy()
            && !f.app.executor.has_active_work()
            && !has_text(o, "Import SMILES records")
    })?;
    assert_eq!(f.state()?["smiles_file"]["projected_records"], json!([1]));
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(current_bytes(&f)?, LATER);
    f.frame(tap(Key::F5));
    let shown = f.warm();
    assert!(has_text(&shown, "1 of 2 SMILES records"));
    assert_single_paint(&f, &shown, 1)?;
    assert_eq!(fs::read(path)?, LATER);
    Ok(())
}

#[test]
fn queued_import_cancel_source_and_close_aba_cannot_later_install_an_old_selection() -> TestResult {
    for change in ["cancel", "source", "close"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = open_source(&mut f, SOURCE)?;
        let original = f.state()?;
        let slots = hold_raw_slots(&f)?;
        open_form(&mut f)?;
        set_field(&mut f, "Import record number", "2")?;
        geometry(&mut f)?;
        f.choose("Import")?;
        f.warm();
        preserve(&f, &original)?;
        assert_eq!(current_bytes(&f)?, SOURCE);
        match change {
            "cancel" => {
                f.choose("Cancel")?;
            }
            "source" => {
                f.command("document.open_smiles", json!({"path":path}))?;
                f.warm();
            }
            "close" => {
                f.command(
                    "document.close",
                    json!({"expected_revision":1,"action":"request"}),
                )?;
                let shown = f.frame(vec![]);
                assert_ne!(f.state()?["close"], "committed");
                assert!(
                    !shown
                        .viewport_output
                        .values()
                        .flat_map(|v| v.commands.iter())
                        .any(|v| matches!(v, egui::ViewportCommand::Close))
                );
                f.command(
                    "document.close",
                    json!({"expected_revision":1,"action":"cancel"}),
                )?;
                f.warm();
            }
            _ => return Err("unknown original context vector".into()),
        }
        let current = f.state()?;
        drop(slots);
        wait_for(&mut f, |f, _| {
            !f.app.files.is_busy() && !f.app.executor.has_active_work()
        })?;
        f.warm();
        assert_eq!(f.state()?["document"], Value::Null);
        assert_eq!(current_bytes(&f)?, SOURCE);
        preserve(&f, &current)?;
        if change != "cancel" {
            assert_eq!(value(&f, "Import record number")?, "2");
            let id = node_any(&f, "Import", &[Role::Button])?;
            assert!(f.nodes.get(&id).ok_or("stale Import")?.is_disabled());
            let (id, _) = f.node("Import record number", &[Role::TextInput])?;
            f.frame(vec![Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Focus,
                    target: id,
                    data: None,
                },
            )]);
            assert_eq!(value(&f, "Import record number")?, "2");
            f.choose("Cancel")?;
        }
        preserve(&f, &current)?;
        assert_eq!(fs::read(path)?, SOURCE);
    }
    // A view ABA after an order intent was queued is also stale, despite equal
    // final camera values. The view changes here are explicit external commands.
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    open_source(&mut f, SOURCE)?;
    import_all(&mut f)?;
    pan(&mut f)?;
    let shown = f.warm();
    let target = bond_point(&f, &shown, 1)?;
    let old = f.state()?;
    let slots = hold_capture_slots(&f)?;
    let mut packet = vec![Event::PointerMoved(target)];
    packet.extend(tap(Key::Num2));
    f.frame(packet);
    assert_eq!(f.state()?, old);
    let view = f.view()?;
    f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
    f.command("ui.view", json!({"action":"set_zoom","percent":100}))?;
    assert_eq!(f.view()?["zoom_percent"], view["zoom_percent"]);
    assert!(
        f.view()?["epoch"].as_u64().ok_or("epoch")? > view["epoch"].as_u64().ok_or("old epoch")?
    );
    f.warm();
    drop(slots);
    wait_for(&mut f, |f, _| {
        !f.app.files.is_busy() && !f.app.executor.has_active_work()
    })?;
    f.warm();
    assert_eq!(f.state()?, old);
    assert_eq!(current_bytes(&f)?, SOURCE);
    Ok(())
}
// Independent ordered-input regression: a later radio must not relabel an earlier Import.
#[test]
fn import_release_and_radio_activation_use_the_selection_at_each_raw_event() -> TestResult {
    for radio_first in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = open_source(&mut f, SOURCE)?;
        open_form(&mut f)?;
        let (one, _) = f.node("One record", &[Role::RadioButton])?;
        assert_eq!(
            f.nodes.get(&one).ok_or("one radio")?.toggled(),
            Some(Toggled::True)
        );
        assert_eq!(value(&f, "Import record number")?, "1");
        let before = f.state()?;
        let (_, submit) = f.node("Import", &[Role::Button])?;
        f.frame(vec![
            Event::PointerMoved(submit.center()),
            button(submit.center(), true),
        ]);
        let (all, _) = f.node("All records", &[Role::RadioButton])?;
        let radio_node = f.nodes.get(&all).ok_or("all radio")?;
        assert_eq!(
            radio_node.default_action_verb(),
            Some(egui::accesskit::DefaultActionVerb::Click)
        );
        assert!(radio_node.supports_action(egui::accesskit::Action::Focus));
        assert!(!radio_node.is_disabled());
        let radio = Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: all,
            data: None,
        });
        let release = button(submit.center(), false);
        let packet = if radio_first {
            vec![radio, release]
        } else {
            vec![release, radio]
        };
        f.frame(packet);
        wait_for(&mut f, |f, shown| {
            f.state().is_ok_and(|state| state["document"].is_object())
                && !f.app.files.is_busy()
                && !f.app.executor.has_active_work()
                && !has_text(shown, "Import SMILES records")
        })?;
        let imported = f.state()?;
        let expected = if radio_first {
            json!([1, 2, 3])
        } else {
            json!([1])
        };
        assert_eq!(
            imported["smiles_file"]["projected_records"], expected,
            "Import must use the selection at its own activation, radio_first={radio_first}"
        );
        assert_eq!(
            imported["revision"],
            before["revision"].as_u64().ok_or("revision")? + 1
        );
        assert_eq!(imported["undo_depth"], 0);
        assert_eq!(imported["redo_depth"], 0);
        assert_eq!(imported["source"], before["source"]);
        assert_eq!(imported["file"], before["file"]);
        assert_eq!(imported["file"]["dirty"], false);
        assert!(imported.get("file_operation").is_none());
        assert_eq!(current_bytes(&f)?, SOURCE);
        assert_eq!(fs::read(input)?, SOURCE);
    }
    Ok(())
}
