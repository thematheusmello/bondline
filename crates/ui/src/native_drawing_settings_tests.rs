//! Independent actual-App File menu baseline; no new Rust API.
#[path = "document_settings_ui_tests.rs"]
mod document_style;

use super::{Fixture, TestResult};
use eframe::egui;
use serde_json::json;

#[test]
fn actual_file_menu_opens_document_drawing_settings_without_authoring_defaults() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.command(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":380,
        "end_x_pt":300,"end_y_pt":380}),
    )?;
    let before = f.state()?;
    f.choose("File")?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        f.frame(vec![]);
        if let Ok((id, _)) = f.node("Document Settings…", &[egui::accesskit::Role::Button])
            && f.nodes.get(&id).is_some_and(|node| !node.is_disabled())
        {
            break;
        }
        if std::time::Instant::now() >= deadline {
            return Err("current actionable Document Settings menu control missing".into());
        }
        std::thread::yield_now();
    }
    f.choose("Document Settings…")?;
    f.warm();
    f.node("Bond length (pt)", &[egui::accesskit::Role::TextInput])?;
    f.node("Chain angle (degrees)", &[egui::accesskit::Role::TextInput])?;
    let mut after = f.state()?;
    assert_eq!(after["ui_drawing_settings"]["input_revision"], 1);
    assert_eq!(
        after["ui_drawing_settings"]["bond_length_pt"],
        serde_json::Value::Null
    );
    assert_eq!(
        after["ui_drawing_settings"]["chain_angle_degrees"],
        serde_json::Value::Null
    );
    assert_eq!(after["ui_drawing_settings"]["current"], true);
    after
        .as_object_mut()
        .ok_or("state")?
        .remove("ui_drawing_settings");
    assert_eq!(
        after, before,
        "opening preferences cannot materialize model defaults or history"
    );
    Ok(())
}

// Additive acceptance; the independent1,690-byte menu declaration stays byte-exact.
use super::{button, canvas, finish_actual_file_workflow, key, projected};
use egui::{
    Event, Key, Modifiers,
    accesskit::{Action, ActionData, ActionRequest, NodeId, Role},
};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command as Process, Stdio},
    sync::Arc,
    time::{Duration, Instant, UNIX_EPOCH},
};

#[track_caller]
fn settings_wait(
    f: &mut Fixture,
    ready: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let caller = std::panic::Location::caller();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let shown = f.frame(vec![]);
        if ready(f, &shown) {
            return Ok(shown);
        }
        if Instant::now() >= deadline {
            return Err(format!("settings wait at {caller}: {:?}", f.app.error).into());
        }
        std::thread::yield_now();
    }
}
fn settings_painted(shown: &egui::FullOutput, text: &str) -> bool {
    shown.shapes.iter().any(|paint| matches!(&paint.shape,egui::Shape::Text(t)
        if t.galley.text()==text && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect())))
}
fn settings_node(f: &Fixture, name: &str, role: Role) -> Option<NodeId> {
    let mut pending = vec![NodeId(egui::accesskit_root_id().value())];
    let mut seen = HashSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(node) = f.nodes.get(&id) else {
            continue;
        };
        let named = node.name() == Some(name)
            || node
                .labelled_by()
                .iter()
                .any(|id| f.nodes.get(id).is_some_and(|n| n.name() == Some(name)));
        if named && node.role() == role {
            return Some(id);
        }
        pending.extend_from_slice(node.children());
    }
    None
}
fn settings_value(f: &Fixture, name: &str) -> TestResult<String> {
    let id = settings_node(f, name, Role::TextInput).ok_or("reachable settings field")?;
    let mut pending = vec![id];
    let mut text = String::new();
    while let Some(id) = pending.pop() {
        let node = f.nodes.get(&id).ok_or("text subtree")?;
        if node.role() == Role::InlineTextBox {
            text.push_str(node.value().ok_or("painted text")?);
        }
        pending.extend(node.children().iter().rev().copied());
    }
    Ok(text)
}
fn settings_action(id: NodeId, action: Action, data: Option<ActionData>) -> Event {
    Event::AccessKitActionRequest(ActionRequest {
        action,
        target: id,
        data,
    })
}
fn settings_set(f: &mut Fixture, name: &str, value: &str) -> TestResult {
    let (id, _) = f.node(name, &[Role::TextInput])?;
    f.frame(vec![settings_action(
        id,
        Action::SetValue,
        Some(ActionData::Value(value.into())),
    )]);
    assert_eq!(f.nodes.get(&id).and_then(|node| node.value()), Some(value));
    settings_wait(f, |f, _| settings_value(f, name).is_ok_and(|v| v == value))?;
    assert_eq!(settings_value(f, name)?, value);
    Ok(())
}
fn settings_focus_all(f: &mut Fixture, name: &str) -> TestResult {
    let (id, _) = f.node(name, &[Role::TextInput])?;
    f.frame(vec![settings_action(id, Action::Focus, None)]);
    f.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
    ]);
    Ok(())
}
fn settings_apply_enabled(f: &Fixture) -> TestResult<bool> {
    let id = settings_node(f, "Apply", Role::Button).ok_or("reachable Apply")?;
    Ok(!f.nodes.get(&id).ok_or("Apply")?.is_disabled())
}
fn settings_state(f: &Fixture) -> TestResult<Value> {
    let mut state = f.state()?;
    state
        .as_object_mut()
        .ok_or("state")?
        .remove("ui_drawing_settings");
    Ok(state)
}
fn settings_open(f: &mut Fixture) -> TestResult {
    f.command(
        "ui.drawing_settings",
        json!({"action":"open","expected_revision":f.state()?["revision"]}),
    )?;
    settings_wait(f, |f, shown| {
        settings_painted(shown, "Document Settings")
            && settings_painted(shown, "Drawing")
            && f.node("Bond length (pt)", &[Role::TextInput]).is_ok()
            && f.node("Chain angle (degrees)", &[Role::TextInput]).is_ok()
    })?;
    assert_eq!(f.state()?["ui_drawing_settings"]["current"], true);
    Ok(())
}
fn settings_closed(f: &mut Fixture) -> TestResult {
    settings_wait(f, |f, _| {
        f.state()
            .is_ok_and(|s| s.get("ui_drawing_settings").is_none())
            && settings_node(f, "Bond length (pt)", Role::TextInput).is_none()
    })?;
    Ok(())
}
fn settings_shortcut(f: &mut Fixture, code: Key, shift: bool) {
    let modifiers = Modifiers {
        shift,
        ..Modifiers::COMMAND
    };
    f.frame(vec![
        key(code, true, modifiers),
        key(code, false, modifiers),
    ]);
}
fn settings_seed(f: &mut Fixture) -> TestResult<PathBuf> {
    f.command(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":380,
        "end_x_pt":300,"end_y_pt":380}),
    )?;
    let input = f.directory.path("source-設定-λ.cdxml");
    f.command("document.save_cdxml", json!({"path":input}))?;
    f.command("document.open_cdxml", json!({"path":input}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    Ok(input)
}
fn settings_document(before: &Value, length: Option<f64>, angle: Option<f64>) -> TestResult<Value> {
    let mut expected = before.clone();
    let mut values = serde_json::Map::new();
    if let Some(v) = length {
        values.insert("bond_length_pt".into(), json!(v));
    }
    if let Some(v) = angle {
        values.insert("chain_angle_degrees".into(), json!(v));
    }
    if values.is_empty() {
        expected
            .as_object_mut()
            .ok_or("document")?
            .remove("drawing_settings");
    } else {
        expected["drawing_settings"] = Value::Object(values);
    }
    Ok(expected)
}
fn settings_reader(relative: &str, bytes: &[u8]) -> TestResult<Value> {
    let mut child = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .arg("-B")
        .arg(format!("{}/{relative}", env!("CARGO_MANIFEST_DIR")))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}
fn settings_consume(path: &Path, length: Option<&str>, angle: Option<&str>) -> TestResult {
    let bytes = fs::read(path)?;
    let mut root = vec![json!(["CreationProgram", "Bondline 0.1.0"])];
    let mut decimal = serde_json::Map::new();
    for (key, value) in [("BondLength", length), ("ChainAngle", angle)] {
        if let Some(value) = value {
            root.push(json!([key, value]));
            decimal.insert(key.into(), json!(value));
        }
    }
    assert_eq!(
        settings_reader("../io/tests/drawing_settings_root.py", &bytes)?,
        json!({"root":root,"decimal":decimal})
    );
    let expected_graph = json!([
        {"id":1,"atoms":[
            {"id":1,"element":6,"position":{"x_pt":220.0,"y_pt":380.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
            {"id":2,"element":6,"position":{"x_pt":300.0,"y_pt":380.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]}]);
    assert_eq!(
        settings_reader("../../xtask/oracles/cdxml_graph.py", &bytes)?,
        expected_graph
    );
    let expected = json!({"graph":expected_graph,"smiles":"CC","formula":"C2H6",
        "flags":[null,null],"double_positions":[null]})
    .to_string();
    let mut child = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../commands/tests/ring_create/named_ring_oracle.py"
            ),
            &expected,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("chemical consumer stdin")?
        .write_all(&bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn actual_fields_apply_clear_history_and_unicode_native_files_preserve_geometry() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = settings_seed(&mut f)?;
    let original_bytes = fs::read(&input)?;
    let original = settings_state(&f)?;
    let view = f.app.engine.lock().map_err(|_| "engine")?.view();
    let scene = f
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .drawing_view_scene(
            1024,
            1024,
            bondline_model::PageId::new(1).ok_or("page")?,
            &bondline_render::Cancellation::default(),
        )?;
    settings_open(&mut f)?;
    assert_eq!(settings_value(&f, "Bond length (pt)")?, "");
    assert_eq!(settings_value(&f, "Chain angle (degrees)")?, "");
    settings_set(&mut f, "Bond length (pt)", "24.5")?;
    settings_set(&mut f, "Chain angle (degrees)", "120.5")?;
    assert_eq!(settings_state(&f)?, original);
    f.choose("Apply")?;
    settings_closed(&mut f)?;
    let edited = settings_state(&f)?;
    assert_eq!(
        edited["document"],
        settings_document(&original["document"], Some(24.5), Some(120.5))?
    );
    assert_eq!(
        edited["revision"],
        original["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(
        (edited["undo_depth"].clone(), edited["redo_depth"].clone()),
        (json!(1), json!(0))
    );
    assert_eq!(edited["file"]["dirty"], true);
    assert_eq!(edited["source"]["modified"], true);
    assert_eq!(f.app.engine.lock().map_err(|_| "engine")?.view(), view);
    assert_eq!(
        f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .drawing_view_scene(
                1024,
                1024,
                bondline_model::PageId::new(1).ok_or("page")?,
                &bondline_render::Cancellation::default()
            )?
            .scene(),
        scene.scene()
    );
    assert_eq!(fs::read(&input)?, original_bytes);

    let output = f.directory.path("saved-設定-λ.cdxml");
    settings_shortcut(&mut f, Key::S, true);
    settings_wait(&mut f, |f, _| {
        f.node("File name", &[Role::TextInput]).is_ok()
    })?;
    settings_set(
        &mut f,
        "File name",
        output
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("UTF8 name")?,
    )?;
    f.choose("Save")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(
        f.state()?["file"]["path"],
        json!(fs::canonicalize(&output)?)
    );
    assert_eq!(f.state()?["file"]["dirty"], false);
    settings_consume(&output, Some("24.5"), Some("120.5"))?;
    settings_shortcut(&mut f, Key::Z, false);
    assert_eq!(f.state()?["document"], original["document"]);
    assert_eq!(f.state()?["file"]["dirty"], true);
    assert_eq!(f.state()?["source"]["modified"], false);
    settings_shortcut(&mut f, Key::Z, true);
    assert_eq!(f.state()?["document"], edited["document"]);
    assert_eq!(f.state()?["file"]["dirty"], false);

    settings_open(&mut f)?;
    assert_eq!(settings_value(&f, "Bond length (pt)")?, "24.5");
    settings_set(&mut f, "Bond length (pt)", "")?;
    settings_set(&mut f, "Chain angle (degrees)", "137.5")?;
    f.choose("Apply")?;
    settings_closed(&mut f)?;
    assert_eq!(
        f.state()?["document"],
        settings_document(&original["document"], None, Some(137.5))?
    );
    settings_shortcut(&mut f, Key::S, false);
    finish_actual_file_workflow(&mut f)?;
    settings_consume(&output, None, Some("137.5"))?;
    settings_open(&mut f)?;
    settings_set(&mut f, "Bond length (pt)", "   ")?;
    settings_set(&mut f, "Chain angle (degrees)", "")?;
    f.choose("Apply")?;
    settings_closed(&mut f)?;
    assert_eq!(f.state()?["document"], original["document"]);
    assert_eq!(f.state()?["source"]["modified"], false);
    assert_eq!(f.state()?["file"]["dirty"], true);
    settings_shortcut(&mut f, Key::S, false);
    finish_actual_file_workflow(&mut f)?;
    settings_consume(&output, None, None)?;
    assert_eq!(fs::read(&output)?, original_bytes);
    assert_eq!(fs::read(&input)?, original_bytes);
    settings_shortcut(&mut f, Key::O, false);
    settings_wait(&mut f, |f, _| {
        f.node("File name", &[Role::TextInput]).is_ok()
    })?;
    settings_set(
        &mut f,
        "File name",
        output
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("UTF8 name")?,
    )?;
    f.choose("Open")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(f.state()?["document"], original["document"]);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(f.state()?["source"]["modified"], false);
    Ok(())
}

fn settings_close_packet(f: &mut Fixture) -> egui::FullOutput {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(f.width, f.height),
        )),
        focused: true,
        ..Default::default()
    };
    let viewport = input.viewports.entry(egui::ViewportId::ROOT).or_default();
    viewport.native_pixels_per_point = Some(f.dpi);
    viewport.events.push(egui::ViewportEvent::Close);
    let context = f.context.clone();
    let output = context.run(input, |context| f.app.update_workspace(context));
    if let Some(update) = &output.platform_output.accesskit_update {
        f.nodes.extend(update.nodes.iter().cloned());
    }
    output
}

#[test]
fn invalid_and_oversize_edits_noop_cancel_and_native_close_do_not_publish_or_fall_through()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = settings_seed(&mut f)?;
    f.command(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    f.command("history.undo", json!({}))?;
    f.command("ui.bond_tool", json!({"order":"double"}))?;
    f.command("ui.selection",json!({"action":"set","expected_revision":f.state()?["revision"],"page_id":1,"atom_ids":[1,2]}))?;
    let original = settings_state(&f)?;
    let raw = fs::read(&input)?;
    let model = f
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .stored_document()?
        .model_snapshot();
    let stamp = f.app.engine.lock().map_err(|_| "engine")?.capture_stamp();
    settings_open(&mut f)?;
    for (name, text) in [
        ("Bond length (pt)", "0"),
        ("Bond length (pt)", "-1"),
        ("Bond length (pt)", "100001"),
        ("Bond length (pt)", "NaN"),
        ("Bond length (pt)", "inf"),
        ("Chain angle (degrees)", "0"),
        ("Chain angle (degrees)", "180"),
        ("Chain angle (degrees)", "-inf"),
    ] {
        settings_set(&mut f, "Bond length (pt)", "")?;
        settings_set(&mut f, "Chain angle (degrees)", "")?;
        settings_set(&mut f, name, text)?;
        assert!(!settings_apply_enabled(&f)?);
        let apply = settings_node(&f, "Apply", Role::Button).ok_or("Apply")?;
        f.frame(vec![settings_action(apply, Action::Default, None)]);
        assert_eq!(settings_state(&f)?, original);
        assert_eq!(settings_value(&f, name)?, text);
    }
    settings_set(&mut f, "Bond length (pt)", "")?;
    settings_set(&mut f, "Chain angle (degrees)", "")?;
    f.choose("Apply")?;
    settings_closed(&mut f)?;
    assert_eq!(settings_state(&f)?, original);
    assert!(Arc::ptr_eq(
        &model,
        &f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .model_snapshot()
    ));
    assert!(
        f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp_matches(&stamp)
    );
    assert_eq!(f.state()?["redo_depth"], 1);

    for paste in [false, true] {
        settings_open(&mut f)?;
        let maximum = format!("{}24.5", "0".repeat(380));
        let oversized = format!("{}24.5", "0".repeat(381));
        assert_eq!((maximum.len(), oversized.len()), (384, 385));
        settings_set(&mut f, "Bond length (pt)", &maximum)?;
        settings_set(&mut f, "Chain angle (degrees)", "120.5")?;
        assert!(settings_apply_enabled(&f)?);
        if paste {
            settings_focus_all(&mut f, "Bond length (pt)")?;
        }
        let field = f.node("Bond length (pt)", &[Role::TextInput])?.0;
        let apply = f.node("Apply", &[Role::Button])?.0;
        let edit = if paste {
            Event::Paste(oversized)
        } else {
            settings_action(
                field,
                Action::SetValue,
                Some(ActionData::Value(oversized.into())),
            )
        };
        f.frame(vec![edit, settings_action(apply, Action::Default, None)]);
        assert_eq!(
            settings_state(&f)?,
            original,
            "rejected prefix cannot Apply the old valid buffer"
        );
        assert_eq!(settings_value(&f, "Bond length (pt)")?, maximum);
        assert_eq!(settings_value(&f, "Chain angle (degrees)")?, "120.5");
        f.choose("Cancel")?;
        settings_closed(&mut f)?;
    }

    settings_open(&mut f)?;
    settings_set(&mut f, "Bond length (pt)", "30")?;
    let old_time = UNIX_EPOCH + Duration::from_secs(946684800);
    fs::OpenOptions::new()
        .write(true)
        .open(&input)?
        .set_times(fs::FileTimes::new().set_modified(old_time))?;
    let shown = settings_close_packet(&mut f);
    assert!(
        shown
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .is_some_and(|v| v
                .commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::CancelClose)))
    );
    assert_eq!(settings_state(&f)?, original);
    assert_eq!(settings_value(&f, "Bond length (pt)")?, "30");
    let before_cancel = f.warm();
    let area = canvas(&before_cancel)?;
    let size = &original["document"]["pages"][0]["size"];
    let a = projected(
        area,
        [
            size["width_pt"].as_f64().ok_or("width")?,
            size["height_pt"].as_f64().ok_or("height")?,
        ],
        100.0,
        None,
        [100.0, 200.0],
    );
    let b = a + egui::vec2(48.0, 0.0);
    let cancel = f.node("Cancel", &[Role::Button])?.0;
    f.frame(vec![
        settings_action(cancel, Action::Default, None),
        key(Key::Z, true, Modifiers::COMMAND),
        key(Key::Z, false, Modifiers::COMMAND),
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
        key(Key::S, true, Modifiers::COMMAND),
        key(Key::S, false, Modifiers::COMMAND),
    ]);
    settings_closed(&mut f)?;
    assert_eq!(settings_state(&f)?, original);
    assert_eq!(fs::metadata(&input)?.modified()?, old_time);
    assert_eq!(fs::read(&input)?, raw);
    assert!(!f.app.files.is_busy() && !f.app.executor.has_active_work());
    Ok(())
}

#[test]
fn text_and_accessibility_edits_apply_at_their_raw_prefix_and_dismissal_fences_the_suffix()
-> TestResult {
    for (typed, early) in [(false, true), (false, false), (true, true), (true, false)] {
        let mut f = Fixture::new(1160.0, 800.0, if typed { 2.0 } else { 1.0 })?;
        let input = settings_seed(&mut f)?;
        let raw = fs::read(&input)?;
        let original = settings_state(&f)?;
        settings_open(&mut f)?;
        settings_set(&mut f, "Bond length (pt)", "24.5")?;
        settings_set(&mut f, "Chain angle (degrees)", "120.5")?;
        if typed {
            settings_focus_all(&mut f, "Bond length (pt)")?;
        }
        let field = f.node("Bond length (pt)", &[Role::TextInput])?.0;
        let (apply, rect) = f.node("Apply", &[Role::Button])?;
        let edit = if typed {
            Event::Text("48".into())
        } else {
            settings_action(
                field,
                Action::SetValue,
                Some(ActionData::Value("48".into())),
            )
        };
        let activation = if typed {
            vec![
                Event::PointerMoved(rect.center()),
                button(rect.center(), true),
                button(rect.center(), false),
            ]
        } else {
            vec![settings_action(apply, Action::Default, None)]
        };
        let mut packet = Vec::new();
        if early {
            packet.push(edit.clone());
        }
        packet.extend(activation);
        if !early {
            packet.push(edit);
        }
        let angle = f.node("Chain angle (degrees)", &[Role::TextInput])?.0;
        packet.push(settings_action(
            angle,
            Action::SetValue,
            Some(ActionData::Value("90".into())),
        ));
        packet.extend([
            key(Key::Z, true, Modifiers::COMMAND),
            key(Key::Z, false, Modifiers::COMMAND),
            key(Key::S, true, Modifiers::COMMAND),
            key(Key::S, false, Modifiers::COMMAND),
        ]);
        f.frame(packet);
        settings_closed(&mut f)?;
        let current = settings_state(&f)?;
        assert_eq!(
            current["document"],
            settings_document(
                &original["document"],
                Some(if early { 48.0 } else { 24.5 }),
                Some(120.5)
            )?,
            "Apply must use its exact earlier draft, typed={typed},early={early}"
        );
        assert_eq!(
            current["revision"],
            original["revision"].as_u64().ok_or("revision")? + 1
        );
        assert_eq!(current["undo_depth"], 1);
        assert_eq!(current["redo_depth"], 0);
        assert_eq!(current["file"]["dirty"], true);
        assert_eq!(current["ui_file"], original["ui_file"]);
        assert_eq!(
            fs::read(&input)?,
            raw,
            "the later Save cannot fall through form dismissal"
        );
        assert!(!f.app.files.is_busy() && !f.app.executor.has_active_work());
    }
    Ok(())
}

#[test]
fn remote_requests_stale_across_context_aba_keep_drafts_and_cannot_stack_collection_work()
-> TestResult {
    for change in ["view", "tool", "file", "close", "source", "sdf", "smiles"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = settings_seed(&mut f)?;
        let sdf = f.directory.path("other-原.sdf");
        let smi = f.directory.path("other-原.smi");
        const SDF: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
        const SMI: &[u8] = b"CC first\nO second\n";
        fs::write(&sdf, SDF)?;
        fs::write(&smi, SMI)?;
        if change == "sdf" {
            f.command("document.open_sdf", json!({"path":sdf}))?;
            f.warm();
            f.choose("Records")?;
            f.warm();
            f.node("Inspect record 1 fields", &[Role::Button])?;
            f.node("Remove record 1", &[Role::Button])?;
            f.command("document.open_cdxml", json!({"path":input}))?;
            f.command("ui.view", json!({"action":"actual"}))?;
            f.warm();
        }
        settings_open(&mut f)?;
        settings_set(&mut f, "Bond length (pt)", "24.5")?;
        settings_set(&mut f, "Chain angle (degrees)", "120.5")?;
        let request = f.state()?["ui_drawing_settings"]["id"]
            .as_u64()
            .ok_or("request id")?;
        match change {
            "view" => {
                f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
                f.command("ui.view", json!({"action":"actual"}))?;
            }
            "tool" => {
                f.command("ui.bond_tool", json!({"order":"double"}))?;
                f.command("ui.bond_tool", json!({"order":"single"}))?;
            }
            "file" => {
                let other = f.directory.path("other-save.cdxml");
                f.command("document.save_cdxml", json!({"path":other}))?;
                f.command(
                    "document.save_cdxml",
                    json!({"path":input,"overwrite":true}),
                )?;
            }
            "close" => {
                let revision = f.state()?["revision"].clone();
                f.command(
                    "document.close",
                    json!({"expected_revision":revision,"action":"request"}),
                )?;
                f.command(
                    "document.close",
                    json!({"expected_revision":revision,"action":"cancel"}),
                )?;
            }
            "source" => {
                f.command(
                    "document.open_cdxml",
                    json!({"path":input,"discard_unsaved":true}),
                )?;
            }
            "sdf" => {
                f.command(
                    "document.open_sdf",
                    json!({"path":sdf,"discard_unsaved":true}),
                )?;
            }
            "smiles" => {
                f.command(
                    "document.open_smiles",
                    json!({"path":smi,"discard_unsaved":true}),
                )?;
            }
            _ => return Err("declared context case".into()),
        }
        let current = settings_state(&f)?;
        settings_wait(&mut f, |f, _| {
            settings_value(f, "Bond length (pt)").is_ok_and(|v| v == "24.5")
        })?;
        assert_eq!(settings_value(&f, "Chain angle (degrees)")?, "120.5");
        assert!(
            !settings_apply_enabled(&f)?,
            "{change} must detach the old request"
        );
        assert_eq!(f.state()?["ui_drawing_settings"]["current"], false);
        let apply = settings_node(&f, "Apply", Role::Button).ok_or("retained Apply")?;
        f.frame(vec![settings_action(apply, Action::Default, None)]);
        assert_eq!(settings_state(&f)?, current);
        if change == "sdf" {
            for name in ["Inspect record 1 fields", "Remove record 1"] {
                if let Some(id) = settings_node(&f, name, Role::Button) {
                    f.frame(vec![settings_action(id, Action::Default, None)]);
                }
                assert!(!f.app.sdf_fields.presentation_present() && !f.app.sdf_fields.is_busy());
            }
            assert_eq!(fs::read(&sdf)?, SDF);
        }
        if change == "smiles" {
            if let Some(id) = settings_node(&f, "Record number", Role::SpinButton) {
                f.frame(vec![settings_action(
                    id,
                    Action::SetValue,
                    Some(ActionData::NumericValue(2.0)),
                )]);
            }
            if let Some(id) = settings_node(&f, "Record number", Role::SpinButton) {
                assert_eq!(
                    f.nodes.get(&id).ok_or("ordinal")?.numeric_value(),
                    Some(1.0),
                    "a retained settings form must fence background record navigation"
                );
            }
            if let Some(id) = settings_node(&f, "Inspect record", Role::Button) {
                f.frame(vec![settings_action(id, Action::Default, None)]);
            }
            assert!(!f.app.smiles_records.is_busy());
            assert_eq!(fs::read(&smi)?, SMI);
        }
        if let Some(id) = settings_node(&f, "All tools…", Role::Button) {
            f.frame(vec![settings_action(id, Action::Default, None)]);
        }
        let shown = f.warm();
        assert!(!settings_painted(&shown, "All tools"));
        assert_eq!(settings_state(&f)?, current);
        // Exercise the actual closed cancel JSON with its current revision and exact old ID.
        f.command("ui.drawing_settings",json!({"action":"cancel","expected_revision":f.state()?["revision"],"request_id":request}))?;
        settings_closed(&mut f)?;
        let shown = f.warm();
        assert_eq!(settings_state(&f)?, current);
        assert!(!f.app.sdf_fields.presentation_present() && !f.app.sdf_fields.is_busy());
        assert!(!f.app.smiles_records.is_busy());
        assert!(!settings_painted(&shown, "Record 1 fields"));
        assert!(settings_node(&f, "Record 1 notation", Role::MultilineTextInput).is_none());
        if matches!(change, "sdf" | "smiles") {
            f.choose("File")?;
            settings_wait(&mut f, |f, _| {
                settings_node(f, "Document Settings…", Role::Button).is_some()
            })?;
            let id = settings_node(&f, "Document Settings…", Role::Button)
                .ok_or("disabled settings control")?;
            assert!(
                f.nodes
                    .get(&id)
                    .ok_or("unsupported settings control")?
                    .is_disabled()
            );
            f.frame(vec![
                key(Key::Escape, true, Modifiers::NONE),
                key(Key::Escape, false, Modifiers::NONE),
            ]);
            assert_eq!(settings_state(&f)?, current);
        }
    }
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    settings_seed(&mut f)?;
    let original = settings_state(&f)?;
    settings_open(&mut f)?;
    settings_set(&mut f, "Bond length (pt)", "24.5")?;
    settings_set(&mut f, "Chain angle (degrees)", "120.5")?;
    f.dpi = 2.0;
    let shown = settings_wait(&mut f, |f, shown| {
        shown.pixels_per_point == 2.0 && f.node("Apply", &[Role::Button]).is_ok()
    })?;
    assert_eq!(shown.pixels_per_point, 2.0);
    assert_eq!(
        f.state()?["ui_drawing_settings"]["current"],
        true,
        "DPI alone must not detach document preferences"
    );
    assert_eq!(settings_value(&f, "Bond length (pt)")?, "24.5");
    f.choose("Apply")?;
    settings_closed(&mut f)?;
    assert_eq!(
        f.state()?["document"],
        settings_document(&original["document"], Some(24.5), Some(120.5))?
    );
    Ok(())
}
