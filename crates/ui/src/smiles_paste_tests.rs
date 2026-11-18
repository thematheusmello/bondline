//! Independent editable SMILES paste through actual App frames.

use super::{Fixture, TestResult};
use eframe::egui;
use std::time::{Duration, Instant};

fn painted(output: &egui::FullOutput, text: &str) -> bool {
    output.shapes.iter().any(|paint| {
        matches!(&paint.shape, egui::Shape::Text(shape)
            if shape.galley.text() == text
                && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
    })
}

fn wait_for(
    fixture: &mut Fixture,
    ready: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = fixture.frame(vec![]);
        if ready(fixture, &output) {
            return Ok(output);
        }
        if Instant::now() >= deadline {
            return Err("current actionable SMILES paste form/work did not become ready".into());
        }
        std::thread::yield_now();
    }
}

#[test]
fn actual_edit_paste_special_menu_opens_a_visible_smiles_draft_without_authoring() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    fixture.warm();
    let original = fixture.state()?;
    fixture.choose("Edit")?;
    // The intended baseline RED is a missing actual submenu role, not a future
    // Rust API, direct presentation dispatch or empty authoring command.
    fixture.node("Paste Special", &[egui::accesskit::Role::Button])?;
    fixture.choose("Paste Special")?;
    fixture.node("SMILES…", &[egui::accesskit::Role::Button])?;
    fixture.choose("SMILES…")?;
    let output = wait_for(&mut fixture, |fixture, output| {
        painted(output, "Paste SMILES")
            && fixture
                .node(
                    "SMILES",
                    &[
                        egui::accesskit::Role::TextInput,
                        egui::accesskit::Role::MultilineTextInput,
                    ],
                )
                .is_ok()
            && fixture
                .node("Cancel", &[egui::accesskit::Role::Button])
                .is_ok()
    })?;
    assert!(painted(&output, "Apply"));
    fixture.node("SMILES", &[egui::accesskit::Role::MultilineTextInput])?;
    for label in ["Bond length (pt)", "Origin X (pt)", "Origin Y (pt)"] {
        fixture.node(label, &[egui::accesskit::Role::TextInput])?;
    }
    let mut after = fixture.state()?;
    assert_eq!(after["ui_file"]["request"]["action"], "paste_smiles");
    after["ui_file"] = original["ui_file"].clone();
    assert_eq!(after, original, "opening a notation draft must not author");
    Ok(())
}

// Additive cases preserve the frozen 2,604-byte missing-control declaration above.
use super::{
    button, canvas, finish_actual_file_workflow, independently_read_actual_saved_graph, key,
    projected, tap,
};
use egui::{Event, Key, Modifiers};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command as Process, Stdio},
    sync::Arc,
    time::UNIX_EPOCH,
};

fn text_value(fixture: &Fixture, label: &str) -> TestResult<String> {
    let (id, _) = fixture.node(
        label,
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    let mut stack = vec![id];
    let mut value = String::new();
    while let Some(id) = stack.pop() {
        let node = fixture.nodes.get(&id).ok_or("current text node")?;
        if node.role() == egui::accesskit::Role::InlineTextBox {
            value.push_str(node.value().ok_or("painted inline value")?);
        }
        stack.extend(node.children().iter().rev().copied());
    }
    Ok(value)
}

fn button_enabled(fixture: &Fixture, label: &str) -> TestResult<bool> {
    // Disabled controls need not support Focus, so do not relax Fixture::node.
    let mut stack = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    let mut seen = HashSet::new();
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = fixture.nodes.get(&id)
        {
            if node.role() == egui::accesskit::Role::Button && node.name() == Some(label) {
                return Ok(!node.is_disabled());
            }
            stack.extend_from_slice(node.children());
        }
    }
    Err(format!("reachable {label} button missing").into())
}

fn shortcut(fixture: &mut Fixture, code: Key, shift: bool) -> egui::FullOutput {
    let modifiers = Modifiers {
        shift,
        ..Modifiers::COMMAND
    };
    fixture.frame(vec![
        key(code, true, modifiers),
        key(code, false, modifiers),
    ])
}

fn open_form(fixture: &mut Fixture) -> TestResult {
    fixture.choose("Edit")?;
    fixture.choose("Paste Special")?;
    fixture.choose("SMILES…")?;
    wait_for(fixture, |fixture, output| {
        painted(output, "Paste SMILES") && text_value(fixture, "SMILES").is_ok()
    })?;
    Ok(())
}

fn paste_text(fixture: &mut Fixture, text: &str, expected: &str) -> TestResult {
    let (id, _) = fixture.node("SMILES", &[egui::accesskit::Role::MultilineTextInput])?;
    fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    fixture.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Paste(text.into()),
    ]);
    wait_for(fixture, |fixture, _| {
        text_value(fixture, "SMILES").is_ok_and(|value| value == expected)
    })?;
    assert_eq!(text_value(fixture, "SMILES")?, expected);
    Ok(())
}

fn set_field(fixture: &mut Fixture, label: &str, value: &str) -> TestResult {
    let (id, _) = fixture.node(label, &[egui::accesskit::Role::TextInput])?;
    fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::SetValue,
            target: id,
            data: Some(egui::accesskit::ActionData::Value(value.into())),
        },
    )]);
    wait_for(fixture, |fixture, _| {
        text_value(fixture, label).is_ok_and(|painted| painted == value)
    })?;
    assert_eq!(text_value(fixture, label)?, value);
    Ok(())
}

fn geometry(fixture: &mut Fixture) -> TestResult {
    for (label, value) in [
        ("Bond length (pt)", "64"),
        ("Origin X (pt)", "96"),
        ("Origin Y (pt)", "96"),
    ] {
        set_field(fixture, label, value)?;
    }
    Ok(())
}

fn native_pair(fixture: &mut Fixture) -> TestResult<PathBuf> {
    fixture.command(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":72.0,
        "begin_y_pt":480.0,"end_x_pt":120.0,"end_y_pt":480.0}),
    )?;
    let path = fixture.directory.path("current-原.cdxml");
    fixture.command("document.save_cdxml", json!({"path":path}))?;
    fixture.command("document.open_cdxml", json!({"path":path}))?;
    fixture.warm();
    fixture.frame(tap(Key::F5));
    assert_eq!(fixture.view()?["zoom_percent"], 100.0);
    Ok(path)
}

fn preserved(fixture: &Fixture, original: &Value) -> TestResult {
    let mut after = fixture.state()?;
    after["ui_file"] = original["ui_file"].clone();
    assert_eq!(&after, original);
    Ok(())
}

fn consume(fragment: &bondline_model::Fragment, edited: bool) -> TestResult {
    let mut atoms = vec![json!([6, null, 1]); 6];
    let mut bonds = json!([
        [1, 2, 2],
        [2, 3, 1],
        [3, 4, 2],
        [4, 5, 1],
        [5, 6, 2],
        [1, 6, 1]
    ]);
    if edited {
        atoms[0] = json!([6, null, 2]);
        atoms[1] = json!([6, null, 2]);
        bonds[0][2] = json!(1);
    }
    let mut child = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_authoring_oracle.py"
        ))
        .arg(if edited { "C1CC=CC=C1" } else { "c1ccccc1" })
        .arg(
            json!({"atoms":atoms,"bonds":bonds,
            "formula":if edited {"C6H8"} else {"C6H6"},"components":[[1,2,3,4,5,6]]})
            .to_string(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(&serde_json::to_vec(fragment)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["no_assigned_stereo"], true);
    assert_eq!(result["no_radicals"], true);
    Ok(())
}

fn inspect_pasted(fixture: &Fixture, edited: bool) -> TestResult {
    let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
    let fragments = engine.document()?.pages()[0].fragments();
    assert_eq!(fragments.len(), 2);
    assert_eq!(fragments[0].atoms()[0].explicit_hydrogens, Some(3));
    assert_eq!(fragments[0].atoms()[1].explicit_hydrogens, Some(3));
    let pasted = &fragments[1];
    assert_eq!(pasted.id().get(), 2);
    assert_eq!(
        pasted
            .atoms()
            .iter()
            .map(|atom| atom.id.get())
            .collect::<Vec<_>>(),
        (3..=8).collect::<Vec<_>>()
    );
    assert_eq!(
        pasted
            .bonds()
            .iter()
            .map(|bond| bond.id.get())
            .collect::<Vec<_>>(),
        (2..=7).collect::<Vec<_>>()
    );
    assert!(
        (pasted
            .atoms()
            .iter()
            .filter_map(|atom| atom.position)
            .map(|p| p.x_pt())
            .fold(f64::INFINITY, f64::min)
            - 96.0)
            .abs()
            < 1e-9
    );
    assert!(
        (pasted
            .atoms()
            .iter()
            .filter_map(|atom| atom.position)
            .map(|p| p.y_pt())
            .fold(f64::INFINITY, f64::min)
            - 96.0)
            .abs()
            < 1e-9
    );
    for bond in pasted.bonds() {
        let a = pasted
            .atoms()
            .iter()
            .find(|atom| atom.id == bond.begin)
            .and_then(|atom| atom.position)
            .ok_or("begin position")?;
        let b = pasted
            .atoms()
            .iter()
            .find(|atom| atom.id == bond.end)
            .and_then(|atom| atom.position)
            .ok_or("end position")?;
        assert!(
            ((a.x_pt() - b.x_pt()).powi(2) + (a.y_pt() - b.y_pt()).powi(2) - 64.0_f64.powi(2))
                .abs()
                < 1e-8
        );
    }
    consume(pasted, edited)
}

fn choose_file(fixture: &mut Fixture, action: &str, path: &Path) -> TestResult {
    wait_for(fixture, |fixture, _| {
        text_value(fixture, "File name").is_ok()
    })?;
    set_field(
        fixture,
        "File name",
        path.file_name()
            .and_then(|name| name.to_str())
            .ok_or("UTF8 filename")?,
    )?;
    fixture.choose(action)?;
    finish_actual_file_workflow(fixture)
}

#[test]
fn actual_pasted_benzene_is_one_edit_then_native_hotspot_edit_unicode_save_and_reopen() -> TestResult
{
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let original_path = native_pair(&mut fixture)?;
    let original_bytes = fs::read(&original_path)?;
    let before = fixture.state()?;
    let (owner, old_fragment) = {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        (
            engine.content().clone(),
            Arc::clone(&engine.document()?.pages()[0].fragments()[0]),
        )
    };
    open_form(&mut fixture)?;
    paste_text(&mut fixture, "c1ccccc1", "c1ccccc1")?;
    geometry(&mut fixture)?;
    preserved(&fixture, &before)?;
    fixture.choose("Apply")?;
    wait_for(&mut fixture, |fixture, output| {
        fixture.state().is_ok_and(|state| {
            state["revision"].as_u64() == before["revision"].as_u64().map(|r| r + 1)
        }) && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
            && !painted(output, "Paste SMILES")
    })?;
    let after = fixture.state()?;
    assert_eq!(after["undo_depth"], 1);
    assert_eq!(after["redo_depth"], 0);
    assert_eq!(after["file"]["dirty"], true);
    assert_eq!(after["source"]["modified"], true);
    assert_eq!(after["file_operation"], Value::Null);
    assert_eq!(after["ui_file"]["request"], Value::Null);
    assert_eq!(after["file"]["path"], before["file"]["path"]);
    assert_eq!(fs::read(&original_path)?, original_bytes);
    inspect_pasted(&fixture, false)?;
    let whole = fixture
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .content()
        .clone();
    {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        assert!(Arc::ptr_eq(
            &old_fragment,
            &engine.document()?.pages()[0].fragments()[0]
        ));
    }
    shortcut(&mut fixture, Key::Z, false);
    assert_eq!(
        fixture.app.engine.lock().map_err(|_| "engine")?.content(),
        &owner
    );
    assert_eq!(fixture.state()?["undo_depth"], 0);
    assert_eq!(fixture.state()?["redo_depth"], 1);
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    shortcut(&mut fixture, Key::Z, true);
    assert_eq!(
        fixture.app.engine.lock().map_err(|_| "engine")?.content(),
        &whole
    );
    inspect_pasted(&fixture, false)?;
    let output = fixture.warm();
    let midpoint = {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        let atoms = engine.document()?.pages()[0].fragments()[1].atoms();
        let a = atoms[0].position.ok_or("first position")?;
        let b = atoms[1].position.ok_or("second position")?;
        [(a.x_pt() + b.x_pt()) / 2.0, (a.y_pt() + b.y_pt()) / 2.0]
    };
    let point = projected(
        canvas(&output)?,
        [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
        100.0,
        None,
        midpoint,
    );
    let mut events = vec![Event::PointerMoved(point)];
    events.extend(tap(Key::Num1));
    fixture.frame(events);
    inspect_pasted(&fixture, true)?;
    assert_eq!(fixture.state()?["undo_depth"], 2);
    let edited = fixture.state()?["document"].clone();
    let target = fixture.directory.path("pasted-λ-化学.cdxml");
    shortcut(&mut fixture, Key::S, true);
    choose_file(&mut fixture, "Save", &target)?;
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    assert_eq!(fixture.state()?["source"]["modified"], true);
    assert_eq!(
        independently_read_actual_saved_graph(&target)?,
        edited["pages"][0]["fragments"]
    );
    shortcut(&mut fixture, Key::O, false);
    choose_file(&mut fixture, "Open", &target)?;
    assert_eq!(fixture.state()?["document"], edited);
    assert_eq!(fixture.state()?["undo_depth"], 0);
    assert_eq!(fixture.state()?["redo_depth"], 0);
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    assert_eq!(fixture.state()?["source"]["modified"], false);
    inspect_pasted(&fixture, true)?;
    assert_eq!(fs::read(original_path)?, original_bytes);
    Ok(())
}

#[test]
fn invalid_empty_and_oversized_drafts_do_not_publish_save_or_canvas_edits() -> TestResult {
    for source in [
        String::new(),
        "c1cccc1".into(),
        "CC PRIVATE-PROBE-SMILES".into(),
        "C".repeat(65_537),
    ] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = native_pair(&mut fixture)?;
        let bytes = fs::read(&path)?;
        let old_time = UNIX_EPOCH + Duration::from_secs(946_684_800);
        fs::OpenOptions::new()
            .write(true)
            .open(&path)?
            .set_times(fs::FileTimes::new().set_modified(old_time))?;
        let before = fixture.state()?;
        let area = canvas(&fixture.warm())?;
        open_form(&mut fixture)?;
        let expected = if source.len() > 65_536 {
            ""
        } else {
            source.as_str()
        };
        paste_text(&mut fixture, &source, expected)?;
        geometry(&mut fixture)?;
        let a = projected(
            area,
            [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
            100.0,
            None,
            [200.0, 700.0],
        );
        let b = a + egui::vec2(48.0, 0.0);
        fixture.frame(vec![
            Event::PointerMoved(a),
            button(a, true),
            Event::PointerMoved(b),
            button(b, false),
        ]);
        shortcut(&mut fixture, Key::S, false);
        preserved(&fixture, &before)?;
        assert_eq!(
            fs::metadata(&path)?.modified()?,
            old_time,
            "modal Save must not publish unchanged old bytes"
        );
        if button_enabled(&fixture, "Apply")? {
            fixture.choose("Apply")?;
        }
        wait_for(&mut fixture, |fixture, output| {
            !fixture.app.files.is_busy()
                && !fixture.app.executor.has_active_work()
                && painted(output, "Paste SMILES")
        })?;
        preserved(&fixture, &before)?;
        assert_eq!(text_value(&fixture, "SMILES")?, expected);
        assert!(
            !fixture
                .state()?
                .to_string()
                .contains("PRIVATE-PROBE-SMILES")
        );
        assert_eq!(fs::read(&path)?, bytes);
        assert_eq!(fs::metadata(&path)?.modified()?, old_time);
        fixture.choose("Cancel")?;
        finish_actual_file_workflow(&mut fixture)?;
        preserved(&fixture, &before)?;
    }
    Ok(())
}

#[test]
fn source_close_and_cancelled_intent_view_aba_keep_a_detached_copyable_draft() -> TestResult {
    for change in ["source", "close", "view"] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = native_pair(&mut fixture)?;
        open_form(&mut fixture)?;
        paste_text(&mut fixture, "c1ccccc1", "c1ccccc1")?;
        geometry(&mut fixture)?;
        let old_view = fixture.view()?;
        let revision = fixture.state()?["revision"].as_u64().ok_or("revision")?;
        match change {
            "source" => {
                fixture.command("document.open_cdxml", json!({"path":path}))?;
            }
            "close" => {
                fixture.command(
                    "document.close",
                    json!({"expected_revision":revision,"action":"request"}),
                )?;
                fixture.command(
                    "document.close",
                    json!({"expected_revision":revision,"action":"cancel"}),
                )?;
            }
            "view" => {
                // Accepted modal intents already reject remote view changes. Cancel
                // that intent first, then exercise a same-value camera ABA before paint.
                assert!(
                    fixture
                        .command("ui.view", json!({"action":"set_zoom","percent":125}))
                        .is_err()
                );
                let id = fixture.state()?["ui_file"]["request"]["id"]
                    .as_u64()
                    .ok_or("request")?;
                fixture.command(
                    "ui.file",
                    json!({"expected_revision":revision,"action":"cancel","request_id":id}),
                )?;
                fixture.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
                fixture.command("ui.view", json!({"action":"set_zoom","percent":100}))?;
                assert_eq!(fixture.view()?["zoom_percent"], old_view["zoom_percent"]);
                assert!(fixture.view()?["epoch"].as_u64() > old_view["epoch"].as_u64());
            }
            _ => return Err("unknown original stale-context vector".into()),
        }
        let current = fixture.state()?;
        wait_for(&mut fixture, |fixture, output| {
            painted(output, "Paste SMILES")
                && text_value(fixture, "SMILES").is_ok_and(|value| value == "c1ccccc1")
        })?;
        assert!(!button_enabled(&fixture, "Apply")?);
        let (id, _) = fixture.node("SMILES", &[egui::accesskit::Role::MultilineTextInput])?;
        fixture.frame(vec![Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Focus,
                target: id,
                data: None,
            },
        )]);
        assert_eq!(text_value(&fixture, "SMILES")?, "c1ccccc1");
        preserved(&fixture, &current)?;
        fixture.choose("Cancel")?;
        preserved(&fixture, &current)?;
        assert_eq!(fixture.state()?["close"], "open");
        assert_eq!(
            fixture
                .app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .stored_document()?
                .to_cdxml()?
                .as_ref(),
            fs::read(path)?
        );
    }
    Ok(())
}

#[test]
fn full_registered_budget_preserves_paste_intent_then_cancel_disarms_pending_apply() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    native_pair(&mut fixture)?;
    let before = fixture.state()?;
    let stamp = fixture
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .capture_stamp();
    let capture = || {
        serde_json::from_value::<bondline_commands::RenderCapture>(
            json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
        )
    };
    let first = fixture
        .app
        .executor
        .prepare_ui_capture(&stamp, capture()?)
        .map_err(|error| error.message)?;
    let second = fixture
        .app
        .executor
        .prepare_ui_capture(&stamp, capture()?)
        .map_err(|error| error.message)?;
    open_form(&mut fixture)?;
    paste_text(&mut fixture, "c1ccccc1", "c1ccccc1")?;
    geometry(&mut fixture)?;
    let accepted = fixture.state()?["ui_file"]["request"].clone();
    fixture.choose("Apply")?;
    fixture.warm();
    preserved(&fixture, &before)?;
    assert_eq!(
        fixture.state()?["ui_file"]["request"],
        accepted,
        "full budget cannot consume the accepted intent"
    );
    fixture.choose("Cancel")?;
    drop(first);
    drop(second);
    finish_actual_file_workflow(&mut fixture)?;
    fixture.warm();
    preserved(&fixture, &before)?;
    assert_eq!(fixture.state()?["ui_file"]["request"], Value::Null);
    assert!(!fixture.app.files.is_busy());
    assert!(!fixture.app.executor.has_active_work());
    Ok(())
}
// Independent suffix: preserve the existing five-case source as an exact prefix.
#[test]
fn postcommit_save_association_aba_keeps_the_committed_paste_draft_detached() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path_a = native_pair(&mut fixture)?;
    let path_b = fixture.directory.path("postcommit-B-化学.cdxml");
    let original_bytes = fs::read(&path_a)?;
    let before = fixture.state()?;
    let revision = before["revision"].as_u64().ok_or("original revision")?;
    open_form(&mut fixture)?;
    paste_text(&mut fixture, "c1ccccc1", "c1ccccc1")?;
    geometry(&mut fixture)?;
    fixture.choose("Apply")?;

    // No further App frame may consume/dismiss the result before this real ABA.
    // Read the actual Engine commit; do not synthesize Finished/Applied or a result.
    let deadline = Instant::now() + Duration::from_secs(10);
    let (pasted, stamp, after_saves, bytes) = loop {
        if Instant::now() >= deadline {
            return Err("real paste did not commit before the bounded postcommit ABA".into());
        }
        let mut engine = match fixture.app.engine.try_lock() {
            Ok(engine) => engine,
            Err(std::sync::TryLockError::WouldBlock) => {
                std::thread::yield_now();
                continue;
            }
            Err(std::sync::TryLockError::Poisoned(_)) => return Err("engine poisoned".into()),
        };
        let state = engine.state();
        if state.revision != revision + 1 {
            drop(engine);
            std::thread::yield_now();
            continue;
        }
        assert_eq!(state.undo_depth, 1);
        assert!(state.file.dirty);
        assert!(state.file_operation.is_none());
        let pasted = engine.content().clone();
        let stamp = engine.capture_stamp();
        let model = engine.stored_document()?.model_snapshot();
        let graph = serde_json::to_value(engine.document()?)?;
        assert_eq!(
            graph["pages"][0]["fragments"]
                .as_array()
                .ok_or("fragments")?
                .len(),
            2
        );
        let bytes = engine.stored_document()?.to_cdxml()?.into_owned();
        let file_context = engine.ui_file_context_stamp();

        // Hold the same mutex for both actual saves. The worker's second lock
        // can observe A only, never an intermediate B association.
        let saved_b = engine.dispatch("document.save_cdxml", json!({"path":path_b}))?;
        assert!(saved_b.file_operation.is_some());
        let saved_a = engine.dispatch(
            "document.save_cdxml",
            json!({"path":path_a,"overwrite":true}),
        )?;
        assert!(saved_a.file_operation.is_some());
        assert_eq!(engine.state().revision, revision + 1);
        assert_eq!(engine.content(), &pasted);
        assert!(engine.capture_stamp_matches(&stamp));
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
        assert_eq!(serde_json::to_value(engine.document()?)?, graph);
        assert!(!engine.ui_file_context_stamp_matches(&file_context));
        let after = serde_json::to_value(engine.state())?;
        assert_eq!(after["file"]["path"], before["file"]["path"]);
        assert_eq!(after["file"]["format"], before["file"]["format"]);
        assert_eq!(
            after["ui_file"]["epoch"],
            before["ui_file"]["epoch"]
                .as_u64()
                .ok_or("original file epoch")?
                + 1
        );
        assert_eq!(after["file"]["dirty"], false);
        assert_eq!(after["source"]["modified"], true);
        assert_eq!(after["undo_depth"], 1);
        assert_eq!(after["redo_depth"], 0);
        break (pasted, stamp, after, bytes);
    };
    assert_ne!(bytes, original_bytes);
    assert_eq!(fs::read(&path_a)?, bytes);
    assert_eq!(fs::read(&path_b)?, bytes);
    let fragments = &after_saves["document"]["pages"][0]["fragments"];
    assert_eq!(independently_read_actual_saved_graph(&path_a)?, *fragments);
    assert_eq!(independently_read_actual_saved_graph(&path_b)?, *fragments);
    inspect_pasted(&fixture, false)?;

    // Registered completion releases its permit before the host thread sends
    // and retires. Do not confuse an inactive executor with that retirement.
    let deadline = Instant::now() + Duration::from_secs(10);
    while fixture.app.executor.has_active_work() {
        if Instant::now() >= deadline {
            return Err("registered paste completion did not release its worker permit".into());
        }
        std::thread::yield_now();
    }
    wait_for(&mut fixture, |fixture, _| {
        !fixture.app.files.is_busy() && !fixture.app.executor.has_active_work()
    })?;

    // Old code instead dismisses the draft as 'current': its source/revision,
    // path/format and visible file epoch all match despite the invalidating ABA.
    assert_eq!(text_value(&fixture, "SMILES")?, "c1ccccc1");
    assert!(!button_enabled(&fixture, "Apply")?);
    assert_eq!(fixture.state()?, after_saves);
    {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        assert_eq!(engine.content(), &pasted);
        assert!(engine.capture_stamp_matches(&stamp));
    }
    let (id, _) = fixture.node("SMILES", &[egui::accesskit::Role::MultilineTextInput])?;
    fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    let copied = fixture.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Copy,
    ]);
    assert_eq!(copied.platform_output.copied_text, "c1ccccc1");
    fixture.choose("Cancel")?;
    finish_actual_file_workflow(&mut fixture)?;
    assert_eq!(fixture.state()?, after_saves);
    assert_eq!(fs::read(&path_a)?, bytes);
    assert_eq!(fs::read(&path_b)?, bytes);
    Ok(())
}
