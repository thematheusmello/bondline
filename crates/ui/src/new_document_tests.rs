//! Independent File > New acceptance through the actual App.
use super::{
    Fixture, TestResult, button, canvas, finish_actual_file_workflow,
    independently_read_actual_saved_graph, key, pair, projected, tap,
};
use eframe::egui;
use egui::{Event, Key, Modifiers};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    fs,
    io::Write as _,
    path::Path,
    process::{Command as Process, Stdio},
    time::{Duration, Instant},
};

#[path = "../../../xtask/oracles/sdf_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse unchanged independent molecular consumers without altering old tests"
)]
mod sdf_support;

const SDF: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
const SMILES: &[u8] = b"CC first\r\n\n \xffignored\n[13CH3][OH] \0\xffPRIVATE\nO last";

#[test]
fn actual_file_new_replaces_a_clean_sdf_with_a_clean_drawing() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = fixture.directory.path("new-from-化学.sdf");
    fs::write(&path, SDF)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    assert_eq!(fixture.state()?["collection"]["record_count"], 2);
    fixture.choose("File")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("New Document", &[egui::accesskit::Role::Button])
            .is_ok_and(|(id, _)| {
                fixture
                    .nodes
                    .get(&id)
                    .is_some_and(|node| !node.is_disabled())
            })
    })?;
    // This baseline must fail at the actual discoverable File control, without a
    // future Rust API or a direct replacement command standing in for the App.
    let (id, _) = fixture.node("New Document", &[egui::accesskit::Role::Button])?;
    assert!(!fixture.nodes.get(&id).ok_or("New control")?.is_disabled());
    fixture.choose("New Document")?;
    let fresh = fixture.state()?;
    assert!(fresh["document"].is_object());
    assert!(fresh["collection"].is_null());
    assert!(fresh["smiles_file"].is_null());
    assert!(fresh["source"].is_null());
    assert!(fresh["file"]["path"].is_null());
    assert_eq!(fresh["file"]["format"], "cdxml");
    assert_eq!(fresh["file"]["dirty"], false);
    assert_eq!(fresh["undo_depth"], 0);
    assert_eq!(fresh["redo_depth"], 0);
    assert_eq!(fs::read(&path)?, SDF);
    Ok(())
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
            return Err(format!(
                "actual New workflow did not finish: {:?}",
                fixture.app.error
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn text(output: &egui::FullOutput, expected: &str) -> bool {
    output.shapes.iter().any(|paint| {
        matches!(&paint.shape, egui::Shape::Text(label)
            if label.galley.text() == expected
            && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
    })
}

fn current_button_exists(fixture: &Fixture, name: &str) -> bool {
    let mut stack = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    let mut seen = HashSet::new();
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = fixture.nodes.get(&id)
        {
            if node.role() == egui::accesskit::Role::Button && node.name() == Some(name) {
                return true;
            }
            stack.extend_from_slice(node.children());
        }
    }
    false
}

fn current_bytes(fixture: &Fixture) -> TestResult<Vec<u8>> {
    Ok(fixture
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .content()
        .sdf()?
        .as_bytes()
        .to_vec())
}

fn text_value(fixture: &Fixture, name: &str) -> TestResult<String> {
    let (id, _) = fixture.node(
        name,
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
            value.push_str(node.value().ok_or("literal inline text")?);
        }
        stack.extend(node.children().iter().rev().copied());
    }
    Ok(value)
}

fn filename(fixture: &mut Fixture, value: &str) -> TestResult {
    let (id, _) = fixture.node("File name", &[egui::accesskit::Role::TextInput])?;
    fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::SetValue,
            target: id,
            data: Some(egui::accesskit::ActionData::Value(value.into())),
        },
    )]);
    Ok(())
}

fn shortcut(fixture: &mut Fixture, code: Key) {
    fixture.frame(vec![
        key(code, true, Modifiers::COMMAND),
        key(code, false, Modifiers::COMMAND),
    ]);
}

fn menu_new(fixture: &mut Fixture) -> TestResult {
    fixture.choose("File")?;
    fixture.choose("New Document")
}

fn is_fresh(fixture: &Fixture) -> bool {
    fixture.state().is_ok_and(|state| {
        state["document"]["pages"]
            .as_array()
            .is_some_and(|pages| pages.len() == 1)
            && state["document"]["pages"][0]["fragments"] == json!([])
            && state["source"].is_null()
            && state["file"]["path"].is_null()
            && state["file"]["dirty"] == false
            && state["undo_depth"] == 0
            && state["redo_depth"] == 0
            && state["ui_file"]["request"].is_null()
    })
}

fn assert_fresh(fixture: &Fixture) -> TestResult {
    assert!(is_fresh(fixture));
    let state = fixture.state()?;
    assert!(state["collection"].is_null());
    assert!(state["smiles_file"].is_null());
    assert_eq!(state["file"]["format"], "cdxml");
    assert_eq!(
        state["document"]["pages"][0]["size"],
        json!({
            "width_pt":210.0 * 72.0 / 25.4,"height_pt":297.0 * 72.0 / 25.4
        })
    );
    let view = fixture.view()?;
    assert!(view["zoom_percent"].is_null());
    assert!(view["center_x_pt"].is_null());
    assert!(view["center_y_pt"].is_null());
    Ok(())
}

fn assert_authored_unchanged(fixture: &Fixture, before: &Value) -> TestResult {
    let mut after = fixture.state()?;
    // A real ui.file cancellation changes its transient request/epoch. All other
    // authored/source/file/history/close data must remain exactly as captured.
    after["ui_file"] = before["ui_file"].clone();
    assert_eq!(&after, before);
    Ok(())
}

fn draw_carbon(fixture: &mut Fixture) -> TestResult {
    fixture.choose("Single bond")?;
    let output = fixture.frame(tap(Key::F5));
    let area = canvas(&output)?;
    let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    let a = projected(area, page, 100.0, None, [220.0, 380.0]);
    let b = projected(area, page, 100.0, None, [300.0, 380.0]);
    fixture.frame(vec![
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
    ]);
    let state = fixture.state()?;
    let fragments = state["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("authored fragments")?;
    assert_eq!(fragments.len(), 1);
    let atoms = fragments[0]["atoms"].as_array().ok_or("authored atoms")?;
    assert_eq!(atoms.len(), 2);
    for (atom, x) in atoms.iter().zip([220.0, 300.0]) {
        assert_eq!(atom["element"], 6);
        assert_eq!(atom["explicit_hydrogens"], 3);
        assert_eq!(atom["position"], json!({"x_pt":x,"y_pt":380.0}));
    }
    assert_eq!(
        fragments[0]["bonds"]
            .as_array()
            .ok_or("authored bonds")?
            .len(),
        1
    );
    assert_eq!(fragments[0]["bonds"][0]["order"], "single");
    assert_eq!(state["file"]["dirty"], true);
    Ok(())
}

fn independent_carbon(path: &Path) -> TestResult {
    assert_eq!(
        independently_read_actual_saved_graph(path)?,
        json!([pair(1, "single", 3, 380.0)])
    );
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let mut child = Process::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/native_bond_rdkit.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("actual saved-file oracle input")?
        .write_all(&fs::read(path)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "actual saved CDXML chemistry: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn save_new_drawing(fixture: &mut Fixture, name: &str) -> TestResult<std::path::PathBuf> {
    let output = fixture.directory.path(name);
    shortcut(fixture, Key::S);
    wait_for(fixture, |fixture, _| {
        fixture
            .node("File name", &[egui::accesskit::Role::TextInput])
            .is_ok()
    })?;
    filename(fixture, name)?;
    fixture.choose("Save")?;
    finish_actual_file_workflow(fixture)?;
    assert_eq!(
        fixture.state()?["file"]["path"],
        fs::canonicalize(&output)?.to_str().ok_or("Unicode path")?
    );
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    independent_carbon(&output)?;
    Ok(output)
}

#[test]
fn smiles_to_drawing_dirty_cancel_discard_save_and_new_carbon_unicode_persistence() -> TestResult {
    for decision in ["Cancel", "Discard Changes", "Save"] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let source = fixture.directory.path("complete-化学.smiles");
        fs::write(&source, SMILES)?;
        fixture.command("document.open_smiles", json!({"path":source}))?;
        let previous = fixture.state()?;
        let previous_view_epoch = fixture.view()?["epoch"].as_u64().ok_or("view epoch")?;
        let controls = fixture.command("ui.controls", json!({}))?;
        let new = controls["controls"]
            .as_array()
            .ok_or("central controls")?
            .iter()
            .find(|row| row["id"] == "file.new")
            .ok_or("central New binding")?;
        assert_eq!(new["label"], "New Document");
        assert_eq!(new["menu"], "File");
        assert_eq!(
            new["shortcut"],
            json!({"key":"n","primary":true,"shift":false})
        );
        assert_eq!(new["command"]["command"], "ui.file");
        assert_eq!(new["command"]["params"]["action"], "new");
        assert_eq!(new["enabled"], true);
        menu_new(&mut fixture)?;
        wait_for(&mut fixture, |fixture, _| is_fresh(fixture))?;
        assert_fresh(&fixture)?;
        assert_eq!(
            fixture.state()?["revision"].as_u64(),
            previous["revision"].as_u64().map(|revision| revision + 1)
        );
        assert_eq!(fixture.view()?["epoch"], previous_view_epoch + 1);
        assert!(
            fixture
                .node("Inspect record", &[egui::accesskit::Role::Button])
                .is_err()
        );
        draw_carbon(&mut fixture)?;
        let before = fixture.state()?;
        let before_view = fixture.view()?;
        shortcut(&mut fixture, Key::N);
        wait_for(&mut fixture, |fixture, output| {
            text(output, "New document")
                && fixture
                    .node("Discard Changes", &[egui::accesskit::Role::Button])
                    .is_ok()
        })?;
        assert_authored_unchanged(&fixture, &before)?;
        fixture.choose(decision)?;
        if decision == "Cancel" {
            assert_authored_unchanged(&fixture, &before)?;
            assert_eq!(fixture.view()?, before_view);
            assert!(fixture.state()?["ui_file"]["request"].is_null());
            assert!(!text(&fixture.frame(vec![]), "New document"));
        } else {
            if decision == "Save" {
                wait_for(&mut fixture, |fixture, _| {
                    fixture
                        .node("File name", &[egui::accesskit::Role::TextInput])
                        .is_ok()
                })?;
                filename(&mut fixture, "before-new-β-化学.cdxml")?;
                fixture.choose("Save")?;
            }
            wait_for(&mut fixture, |fixture, _| {
                is_fresh(fixture)
                    && !fixture.app.files.is_busy()
                    && !fixture.app.executor.has_active_work()
            })?;
            assert_fresh(&fixture)?;
            if decision == "Save" {
                let retained = fixture.directory.path("before-new-β-化学.cdxml");
                independent_carbon(&retained)?;
                let retained_bytes = fs::read(&retained)?;
                draw_carbon(&mut fixture)?;
                let output = save_new_drawing(&mut fixture, "after-new-γ-化学.cdxml")?;
                let saved_graph = fixture.state()?["document"].clone();
                shortcut(&mut fixture, Key::O);
                wait_for(&mut fixture, |fixture, _| {
                    fixture
                        .node("File name", &[egui::accesskit::Role::TextInput])
                        .is_ok()
                })?;
                filename(&mut fixture, "after-new-γ-化学.cdxml")?;
                fixture.choose("Open")?;
                finish_actual_file_workflow(&mut fixture)?;
                assert_eq!(fixture.state()?["document"], saved_graph);
                assert_eq!(fixture.state()?["source"]["modified"], false);
                assert_eq!(fixture.state()?["file"]["dirty"], false);
                assert_eq!(fixture.state()?["undo_depth"], 0);
                independent_carbon(&output)?;
                assert_eq!(fs::read(retained)?, retained_bytes);
            }
        }
        assert_eq!(fs::read(source)?, SMILES);
        assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    }
    Ok(())
}

#[test]
fn changed_sdf_draft_resolves_before_a_fresh_new_intent_and_applied_data_is_saved() -> TestResult {
    let applied = [
        SDF.get(..757).ok_or("value prefix")?,
        b"reviewed\nline two\n",
        SDF.get(771..).ok_or("value suffix")?,
    ]
    .concat();
    assert_eq!(applied.len(), 781);
    for decision in ["Cancel", "Discard", "Apply"] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = fixture.directory.path("draft-before-new-化学.sdf");
        fs::write(&path, SDF)?;
        fixture.command("document.open_sdf", json!({"path":path}))?;
        fixture.choose("Records")?;
        fixture.choose("Inspect record 2 fields")?;
        wait_for(&mut fixture, |fixture, _| {
            fixture
                .node("Field 1: ORDINARY", &[egui::accesskit::Role::Button])
                .is_ok()
        })?;
        fixture.choose("Field 1: ORDINARY")?;
        wait_for(&mut fixture, |fixture, _| {
            text_value(fixture, "Record 2 field 1 value")
                .is_ok_and(|value| value == "second record")
        })?;
        fixture.choose("Edit record 2 field 1")?;
        let label = "Record 2 field 1 draft";
        wait_for(&mut fixture, |fixture, _| {
            text_value(fixture, label).is_ok_and(|value| value == "second record")
        })?;
        let (id, _) = fixture.node(
            label,
            &[
                egui::accesskit::Role::TextInput,
                egui::accesskit::Role::MultilineTextInput,
            ],
        )?;
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
            Event::Text("reviewed\nline two".into()),
        ]);
        assert_eq!(text_value(&fixture, label)?, "reviewed\nline two");
        let before = fixture.state()?;
        menu_new(&mut fixture)?;
        let output = wait_for(&mut fixture, |fixture, output| {
            text(output, "Unapplied field value")
                && fixture
                    .node("Apply", &[egui::accesskit::Role::Button])
                    .is_ok()
        })?;
        assert!(!text(&output, "New document"));
        assert_eq!(current_bytes(&fixture)?, SDF);
        assert_eq!(fs::read(&path)?, SDF);
        fixture.choose(decision)?;
        if decision == "Cancel" {
            assert_eq!(text_value(&fixture, label)?, "reviewed\nline two");
            assert_authored_unchanged(&fixture, &before)?;
            assert_eq!(current_bytes(&fixture)?, SDF);
            assert!(!text(&fixture.frame(vec![]), "New document"));
        } else {
            if decision == "Apply" {
                wait_for(&mut fixture, |fixture, output| {
                    current_bytes(fixture).is_ok_and(|bytes| bytes == applied)
                        && text(output, "New document")
                })?;
                assert_eq!(fixture.state()?["undo_depth"], 1);
                assert_eq!(fixture.state()?["file"]["dirty"], true);
                assert_eq!(fs::read(&path)?, SDF, "Apply is not an implicit Save");
                fixture.choose("Save")?;
            }
            wait_for(&mut fixture, |fixture, _| {
                is_fresh(fixture)
                    && !fixture.app.files.is_busy()
                    && !fixture.app.sdf_fields.is_busy()
                    && !fixture.app.executor.has_active_work()
            })?;
            assert_fresh(&fixture)?;
            assert!(text_value(&fixture, label).is_err());
            if decision == "Apply" {
                sdf_support::independently_read_sdf(&path, &applied, true)?;
            }
        }
        if decision != "Apply" {
            assert_eq!(fs::read(&path)?, SDF);
        }
    }
    Ok(())
}

fn dirty_sdf(fixture: &mut Fixture, name: &str) -> TestResult<std::path::PathBuf> {
    let path = fixture.directory.path(name);
    fs::write(&path, SDF)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let revision = fixture.state()?["revision"]
        .as_u64()
        .ok_or("current revision")?;
    fixture.command("sdf.field.set_value",json!({"expected_revision":revision,"record_id":2,"field_index":0,"value_lines":["reviewed","line two"]}))?;
    assert_eq!(fixture.state()?["collection"]["byte_length"], 781);
    assert_eq!(fixture.state()?["file"]["dirty"], true);
    Ok(path)
}

#[test]
fn ordered_undo_then_new_and_stale_new_request_do_not_retarget_replacement_sources() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = dirty_sdf(&mut fixture, "ordered-new-化学.sdf")?;
    fixture.warm();
    let before = fixture.state()?;
    fixture.frame(vec![
        key(Key::Z, true, Modifiers::COMMAND),
        key(Key::Z, false, Modifiers::COMMAND),
        key(Key::N, true, Modifiers::COMMAND),
        key(Key::N, false, Modifiers::COMMAND),
    ]);
    wait_for(&mut fixture, |fixture, _| is_fresh(fixture))?;
    assert_fresh(&fixture)?;
    assert_eq!(
        fixture.state()?["revision"].as_u64(),
        before["revision"].as_u64().map(|revision| revision + 2)
    );
    assert!(
        !text(
            &fixture.frame(vec![]),
            "Save your changes before creating a new document?"
        ),
        "Undo made the source clean before New in the same packet"
    );
    assert!(!current_button_exists(&fixture, "Discard Changes"));
    assert_eq!(fs::read(&path)?, SDF);

    for same_source in [false, true] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = dirty_sdf(&mut fixture, "stale-new-化学.sdf")?;
        shortcut(&mut fixture, Key::N);
        wait_for(&mut fixture, |fixture, output| {
            text(output, "New document")
                && fixture
                    .node("Discard Changes", &[egui::accesskit::Role::Button])
                    .is_ok()
        })?;
        let other = fixture.directory.path("replacement-化学.smiles");
        fs::write(&other, SMILES)?;
        if same_source {
            fixture.command(
                "document.open_sdf",
                json!({"path":path,"discard_unsaved":true}),
            )?;
        } else {
            fixture.command(
                "document.open_smiles",
                json!({"path":other,"discard_unsaved":true}),
            )?;
        }
        let replacement = fixture.state()?;
        let output = wait_for(&mut fixture, |fixture, output| {
            !text(output, "New document")
                && fixture
                    .state()
                    .is_ok_and(|state| state["ui_file"]["request"].is_null())
        })?;
        assert!(!text(&output, "New document"));
        assert_authored_unchanged(&fixture, &replacement)?;
        assert!(fixture.state()?["document"].is_null());
        if same_source {
            assert_eq!(current_bytes(&fixture)?, SDF);
            assert_eq!(fixture.state()?["collection"]["record_count"], 2);
        } else {
            assert_eq!(fixture.state()?["smiles_file"]["record_count"], 3);
            assert_eq!(
                fixture
                    .app
                    .engine
                    .lock()
                    .map_err(|_| "engine")?
                    .content()
                    .smiles_file()?
                    .as_bytes(),
                SMILES
            );
        }
        assert!(
            fixture
                .node("Discard Changes", &[egui::accesskit::Role::Button])
                .is_err()
        );
        assert_eq!(fs::read(path)?, SDF);
        assert_eq!(fs::read(other)?, SMILES);
    }
    Ok(())
}

#[test]
fn poll_start_save_to_new_rejects_pointer_input_from_the_previous_collection_surface() -> TestResult
{
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = dirty_sdf(&mut fixture, "poll-new-化学.sdf")?;
    let expected = [
        SDF.get(..757).ok_or("literal value prefix")?,
        b"reviewed\nline two\n",
        SDF.get(771..).ok_or("literal value suffix")?,
    ]
    .concat();
    assert_eq!(expected.len(), 781);
    let revision = fixture.state()?["revision"]
        .as_u64()
        .ok_or("dirty revision")?;
    shortcut(&mut fixture, Key::N);
    wait_for(&mut fixture, |fixture, output| {
        text(output, "New document")
            && fixture
                .node("Save", &[egui::accesskit::Role::Button])
                .is_ok()
    })?;
    let (save, _) = fixture.node("Save", &[egui::accesskit::Role::Button])?;
    // One real activation frame: choose() would warm through worker completion.
    let mut output = fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: save,
            data: None,
        },
    )]);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut collection_packets = 0;
    loop {
        let before = fixture.state()?;
        if before["document"].is_object() {
            break;
        }
        assert!(before["document"].is_null());
        assert_eq!(before["collection"]["record_count"], 2);
        // At sampling these coherent press/drag/release events belong to the old
        // read-only collection, even if poll() creates a Drawing in this frame.
        let a = egui::pos2(400.0, 300.0);
        let b = egui::pos2(448.0, 300.0);
        output = fixture.frame(vec![
            Event::PointerMoved(a),
            button(a, true),
            Event::PointerMoved(b),
            button(b, false),
        ]);
        collection_packets += 1;
        if fixture.state()?["document"].is_object() {
            break;
        }
        if Instant::now() >= deadline {
            return Err("actual Save-to-New transition did not finish within ten seconds".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    // Stop at the first Drawing-kind frame, including an incorrectly edited one.
    assert!(
        collection_packets > 0,
        "test must sample the old collection surface"
    );
    let fresh = fixture.state()?;
    assert_eq!(fresh["document"]["pages"][0]["fragments"], json!([]));
    assert_eq!(fresh["revision"], revision + 1);
    assert_eq!(fresh["undo_depth"], 0);
    assert_eq!(fresh["redo_depth"], 0);
    assert_eq!(fresh["file"]["dirty"], false);
    assert!(fresh["file"]["path"].is_null());
    assert!(fresh["source"].is_null());
    sdf_support::independently_read_sdf(&path, &expected, true)?;

    // A fresh next-frame Actual/gesture packet must still author the known geometry.
    let area = canvas(&output)?;
    let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    let a = projected(area, page, 100.0, None, [220.0, 380.0]);
    let b = projected(area, page, 100.0, None, [300.0, 380.0]);
    let mut events = tap(Key::F5);
    events.extend([
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
    ]);
    fixture.frame(events);
    assert_eq!(
        fixture.state()?["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fresh fragment")?
            .len(),
        1
    );
    assert_eq!(fixture.state()?["revision"], revision + 2);
    assert_eq!(fixture.state()?["undo_depth"], 1);
    save_new_drawing(&mut fixture, "fresh-after-poll-化学.cdxml")?;
    assert_eq!(fs::read(&path)?, expected);
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}

#[test]
fn between_frame_shared_new_rejects_pointer_input_from_the_last_presented_sdf_surface() -> TestResult
{
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = fixture.directory.path("shared-new-化学.sdf");
    fs::write(&path, SDF)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let shown = fixture.warm();
    assert!(text(&shown, "SDF collection: 2 records, 777 bytes"));
    assert!(text(&shown, "Read-only chemical preview"));
    assert_eq!(fixture.state()?["revision"], 1);
    assert!(fixture.state()?["document"].is_null());

    // Publish through the actual registry under its shared mutex BETWEEN frames.
    // No Fixture helper or frame warm may present the new Drawing before the drag.
    {
        let mut engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        let result = engine.dispatch("document.new", json!({"expected_revision":1}))?;
        assert!(result.changed);
        assert_eq!(result.revision, 2);
    }
    let a = egui::pos2(400.0, 300.0);
    let b = egui::pos2(448.0, 300.0);
    let output = fixture.frame(vec![
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
    ]);
    let fresh = fixture.state()?;
    assert_eq!(fresh["document"]["pages"][0]["fragments"], json!([]));
    assert_eq!(fresh["revision"], 2);
    assert_eq!(fresh["undo_depth"], 0);
    assert_eq!(fresh["redo_depth"], 0);
    assert_eq!(fresh["file"]["dirty"], false);
    assert!(fresh["source"].is_null());
    assert!(fresh["file"]["path"].is_null());
    sdf_support::independently_read_sdf(&path, SDF, true)?;

    let area = canvas(&output)?;
    let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    let a = projected(area, page, 100.0, None, [220.0, 380.0]);
    let b = projected(area, page, 100.0, None, [300.0, 380.0]);
    let mut events = tap(Key::F5);
    events.extend([
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
    ]);
    fixture.frame(events);
    assert_eq!(fixture.state()?["revision"], 3);
    assert_eq!(fixture.state()?["undo_depth"], 1);
    save_new_drawing(&mut fixture, "fresh-after-shared-new-化学.cdxml")?;
    assert_eq!(fs::read(path)?, SDF);
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}
