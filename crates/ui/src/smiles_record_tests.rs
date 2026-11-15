//! Independent read-only record workflows through actual BondlineApp frames.
use super::{Fixture, TestResult};
use eframe::egui;
use serde_json::{Value, json};
use std::fs;

const SOURCE: &[u8] = b"CC first\r\n\n \xffignored\n[13CH3][OH] \0\xffPRIVATE\nO last";

#[test]
fn opening_complete_smiles_offers_explicit_inspection_without_automatic_notation_disclosure()
-> TestResult {
    assert_eq!(SOURCE.len(), 49);
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = fixture.directory.path("records-化学.smiles");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_smiles", json!({"path":path}))?;
    let before = fixture.state()?;
    assert_eq!(before["document"], Value::Null);
    assert_eq!(before["smiles_file"]["record_count"], 3);
    let output = fixture.warm();
    assert!(!output.shapes.iter().any(|paint| {
        matches!(&paint.shape, egui::Shape::Text(text)
            if text.galley.text().contains("[13CH3][OH]")
                || text.galley.text().contains("PRIVATE"))
    }));
    fixture.node("Inspect record", &[egui::accesskit::Role::Button])?;
    assert_eq!(fixture.state()?, before);
    assert_eq!(fs::read(path)?, SOURCE);
    Ok(())
}

// Additive acceptance follows the frozen missing-control runtime RED.
use super::{finish_actual_file_workflow, key};
use egui::{Event, Key, Modifiers};
use std::{
    collections::HashSet,
    io::Write,
    process::{Command as Process, Stdio},
    time::{Duration, Instant},
};

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
                "actual record workflow did not settle: {:?}",
                fixture.app.error
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn text(output: &egui::FullOutput, expected: &str) -> bool {
    output.shapes.iter().any(|paint| {
        matches!(&paint.shape,
        egui::Shape::Text(label) if label.galley.text() == expected
            && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
    })
}

fn reachable(fixture: &Fixture) -> HashSet<egui::accesskit::NodeId> {
    let mut pending = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    let mut seen = HashSet::new();
    while let Some(id) = pending.pop() {
        if seen.insert(id)
            && let Some(node) = fixture.nodes.get(&id)
        {
            pending.extend_from_slice(node.children());
        }
    }
    seen
}

fn control<'a>(fixture: &'a Fixture, name: &str) -> TestResult<&'a egui::accesskit::Node> {
    let current = reachable(fixture);
    fixture
        .nodes
        .iter()
        .find_map(|(id, node)| {
            (current.contains(id)
                && node.role() == egui::accesskit::Role::Button
                && node.name() == Some(name))
            .then_some(node)
        })
        .ok_or_else(|| format!("current {name} button missing").into())
}

fn notation(fixture: &Fixture, record: u32) -> TestResult<String> {
    let (id, _) = fixture.node(
        &format!("Record {record} notation"),
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    let mut value = String::new();
    let mut pending = vec![id];
    while let Some(id) = pending.pop() {
        let node = fixture.nodes.get(&id).ok_or("notation node")?;
        if node.role() == egui::accesskit::Role::InlineTextBox {
            value.push_str(node.value().ok_or("literal inline notation")?);
        }
        pending.extend(node.children().iter().rev().copied());
    }
    Ok(value)
}

fn assert_private(fixture: &Fixture, output: &egui::FullOutput) {
    for paint in &output.shapes {
        if let egui::Shape::Text(label) = &paint.shape {
            let value = label.galley.text();
            assert!(
                !value.contains("PRIVATE") && !value.contains('\0') && !value.contains('\u{fffd}')
            );
        }
    }
    for id in reachable(fixture) {
        if let Some(node) = fixture.nodes.get(&id) {
            for value in [node.name(), node.value(), node.description()]
                .into_iter()
                .flatten()
            {
                assert!(
                    !value.contains("PRIVATE")
                        && !value.contains('\0')
                        && !value.contains('\u{fffd}')
                );
            }
        }
    }
    let copied = &output.platform_output.copied_text;
    assert!(!copied.contains("PRIVATE") && !copied.contains('\0') && !copied.contains('\u{fffd}'));
}

fn assert_not_disclosed(fixture: &Fixture, output: &egui::FullOutput, old: &str) {
    assert!(!output.shapes.iter().any(|paint| matches!(&paint.shape,
        egui::Shape::Text(label) if label.galley.text().contains(old))));
    for id in reachable(fixture) {
        if let Some(node) = fixture.nodes.get(&id) {
            assert!(!node.value().is_some_and(|value| value.contains(old)));
        }
    }
    assert_private(fixture, output);
}

fn set_record(fixture: &mut Fixture, ordinal: u32) -> TestResult {
    fixture.warm();
    let (id, _) = fixture.node("Record number", &[egui::accesskit::Role::SpinButton])?;
    fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::SetValue,
            target: id,
            data: Some(egui::accesskit::ActionData::NumericValue(f64::from(
                ordinal,
            ))),
        },
    )]);
    Ok(())
}

fn inspect(fixture: &mut Fixture, ordinal: u32, expected: &str) -> TestResult<egui::FullOutput> {
    set_record(fixture, ordinal)?;
    fixture.choose("Inspect record")?;
    wait_for(fixture, |fixture, _| {
        notation(fixture, ordinal).is_ok_and(|value| value == expected)
    })
}

fn open(fixture: &Fixture, name: &str, source: &[u8]) -> TestResult<std::path::PathBuf> {
    let path = fixture.directory.path(name);
    fs::write(&path, source)?;
    fixture.command("document.open_smiles", json!({"path":path}))?;
    Ok(path)
}

fn oracle(name: &str, bytes: &[u8]) -> TestResult<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned consumer interpreter")?;
    let mut child = Process::new(python)
        .arg("-B")
        .arg(format!("{}/../io/tests/{name}", env!("CARGO_MANIFEST_DIR")))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

fn assert_selected_consumer(bytes: &[u8]) -> TestResult {
    let graph = oracle("smiles_oracle.py", bytes)?;
    assert_eq!(graph["rdkit"], "2026.09.2");
    assert_eq!(graph["accepted"], true);
    let atoms = graph["atoms"].as_array().ok_or("external atoms")?;
    assert_eq!(atoms.len(), 2);
    assert_eq!(atoms[0]["element"], 6);
    assert_eq!(atoms[0]["isotope"], 13);
    assert_eq!(atoms[0]["explicit_h"], 3);
    assert_eq!(atoms[1]["element"], 8);
    assert_eq!(atoms[1]["explicit_h"], 1);
    assert_eq!(graph["bonds"], json!([[0, 1, "SINGLE"]]));
    Ok(())
}

fn assert_no_text_undo(context: &egui::Context, id: egui::Id) {
    if let Some(state) = egui::TextEdit::load_state(context, id) {
        let empty = (
            egui::text::CCursorRange::one(egui::text::CCursor::new(0)),
            String::new(),
        );
        assert!(!state.undoer().has_undo(&empty));
        assert!(!state.undoer().is_in_flux());
    }
}

#[test]
fn nonfirst_literal_record_is_readonly_copyable_private_and_has_no_retained_text_undo() -> TestResult
{
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = open(&fixture, "nonfirst.smiles", SOURCE)?;
    let before = fixture.state()?;
    let stamp = fixture
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .capture_stamp();
    let output = inspect(&mut fixture, 2, "[13CH3][OH]")?;
    for label in [
        "Record 2, line 4",
        "2 atoms, 1 bond",
        "Notation bytes: 21..32",
        "Record bytes: 21..43",
        "Application data: 9 bytes",
    ] {
        assert!(text(&output, label), "literal selected metadata {label}");
    }
    assert_private(&fixture, &output);
    let (id, _) = fixture.node(
        "Record 2 notation",
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    let node = fixture.nodes.get(&id).ok_or("current notation input")?;
    assert!(node.is_read_only());
    assert!(!node.supports_action(egui::accesskit::Action::SetValue));
    fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    let focused = fixture
        .context
        .memory(|memory| memory.focused())
        .ok_or("actual focused notation")?;
    let copied = fixture.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Copy,
    ]);
    assert_eq!(copied.platform_output.copied_text, "[13CH3][OH]");
    assert_private(&fixture, &copied);
    assert_selected_consumer(copied.platform_output.copied_text.as_bytes())?;
    let wrong = oracle("smiles_oracle.py", b"[CH3][OH]")?;
    assert_eq!(
        wrong["atoms"][0]["isotope"], 0,
        "consumer distinguishes stripped isotope"
    );
    fixture.frame(vec![
        Event::Text("replace".into()),
        Event::Paste("erase".into()),
        Event::Cut,
        key(Key::Z, true, Modifiers::COMMAND),
        key(Key::Z, false, Modifiers::COMMAND),
    ]);
    assert_eq!(notation(&fixture, 2)?, "[13CH3][OH]");
    assert_no_text_undo(&fixture.context, focused);
    fixture.choose("Close record")?;
    let closed = fixture.warm();
    assert_not_disclosed(&fixture, &closed, "[13CH3][OH]");
    assert_ne!(
        fixture.context.memory(|memory| memory.focused()),
        Some(focused)
    );
    assert_no_text_undo(&fixture.context, focused);
    assert_eq!(fixture.state()?, before);
    let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
    assert_eq!(engine.capture_stamp(), stamp);
    assert_eq!(engine.content().smiles_file()?.as_bytes(), SOURCE);
    assert!(engine.state().smiles_record.is_none());
    assert!(!format!("{:?}", engine.state()).contains("PRIVATE"));
    assert_eq!(fs::read(path)?, SOURCE);
    Ok(())
}

#[test]
fn actual_navigation_enforces_limits_and_source_aba_or_empty_file_clears_old_disclosure()
-> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let original = open(&fixture, "navigation.smiles", SOURCE)?;
    inspect(&mut fixture, 1, "CC")?;
    assert!(control(&fixture, "Previous record")?.is_disabled());
    assert!(!control(&fixture, "Next record")?.is_disabled());
    fixture.choose("Next record")?;
    wait_for(&mut fixture, |f, _| {
        notation(f, 2).is_ok_and(|v| v == "[13CH3][OH]")
    })?;
    fixture.choose("Next record")?;
    wait_for(&mut fixture, |f, _| notation(f, 3).is_ok_and(|v| v == "O"))?;
    assert!(control(&fixture, "Next record")?.is_disabled());
    fixture.choose("Previous record")?;
    wait_for(&mut fixture, |f, _| {
        notation(f, 2).is_ok_and(|v| v == "[13CH3][OH]")
    })?;
    set_record(&mut fixture, 4)?;
    fixture.choose("Inspect record")?;
    let invalid = wait_for(&mut fixture, |f, _| control(f, "Retry inspection").is_ok())?;
    assert_not_disclosed(&fixture, &invalid, "[13CH3][OH]");
    assert!(
        notation(&fixture, 3).is_err(),
        "out-of-range entry cannot silently inspect last record"
    );
    inspect(&mut fixture, 2, "[13CH3][OH]")?;
    let (id, _) = fixture.node(
        "Record 2 notation",
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
    let focused = fixture
        .context
        .memory(|memory| memory.focused())
        .ok_or("source focus")?;
    let alternate = open(&fixture, "other.smiles", b"N alternate\n")?;
    let changed = fixture.warm();
    assert_not_disclosed(&fixture, &changed, "[13CH3][OH]");
    assert_ne!(
        fixture.context.memory(|memory| memory.focused()),
        Some(focused)
    );
    assert_no_text_undo(&fixture.context, focused);
    fixture.command("document.open_smiles", json!({"path":original}))?;
    let aba = fixture.warm();
    assert_not_disclosed(&fixture, &aba, "[13CH3][OH]");
    inspect(&mut fixture, 2, "[13CH3][OH]")?;
    fixture.command("document.open_smiles", json!({"path":original}))?;
    let identical_open = fixture.warm();
    assert_not_disclosed(&fixture, &identical_open, "[13CH3][OH]");
    let empty = open(&fixture, "empty.smiles", b"\n")?;
    let output = fixture.warm();
    assert_eq!(fixture.state()?["smiles_file"]["record_count"], 0);
    assert!(control(&fixture, "Inspect record")?.is_disabled());
    assert!(control(&fixture, "Previous record")?.is_disabled());
    assert!(control(&fixture, "Next record")?.is_disabled());
    assert_not_disclosed(&fixture, &output, "[13CH3][OH]");
    assert_eq!(fs::read(original)?, SOURCE);
    assert_eq!(fs::read(alternate)?, b"N alternate\n");
    assert_eq!(fs::read(empty)?, b"\n");
    Ok(())
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

#[test]
fn actual_unicode_save_as_known_save_and_reopen_preserve_complete_opaque_file() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    open(&fixture, "source.smiles", SOURCE)?;
    inspect(&mut fixture, 2, "[13CH3][OH]")?;
    fixture.choose("Close record")?;
    let output = fixture.directory.path("copied-化学.smiles");
    fs::write(&output, b"existing target stays until explicit overwrite\n")?;
    let save_as = Modifiers {
        shift: true,
        ..Modifiers::COMMAND
    };
    fixture.frame(vec![
        key(Key::S, true, save_as),
        key(Key::S, false, save_as),
    ]);
    wait_for(&mut fixture, |f, _| {
        f.node("File name", &[egui::accesskit::Role::TextInput])
            .is_ok()
    })?;
    filename(&mut fixture, "copied-化学.smiles")?;
    fixture.choose("Save")?;
    assert_eq!(
        fs::read(&output)?,
        b"existing target stays until explicit overwrite\n"
    );
    wait_for(&mut fixture, |fixture, _| {
        control(fixture, "Replace File").is_ok()
    })?;
    assert_eq!(
        fs::read(&output)?,
        b"existing target stays until explicit overwrite\n"
    );
    fixture.choose("Replace File")?;
    finish_actual_file_workflow(&mut fixture)?;
    assert_eq!(fs::read(&output)?, SOURCE);
    let saved = fixture.state()?;
    assert_eq!(
        saved["file"]["path"],
        fs::canonicalize(&output)?.to_str().ok_or("unicode path")?
    );
    assert_eq!(saved["file"]["format"], "smiles");
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["undo_depth"], 0);
    let raw = oracle("smiles_file_raw_oracle.py", &fs::read(&output)?)?;
    assert_eq!(raw["byte_length"], 49);
    assert_eq!(raw["physical_lines"], 5);
    let records = raw["records"].as_array().ok_or("raw records")?;
    assert_eq!(records.len(), 3);
    assert_eq!(records[1]["full_range"], json!([21, 43]));
    assert_eq!(records[1]["notation_range"], json!([21, 32]));
    assert_eq!(records[1]["application_data_hex"], "00ff50524956415445");
    fixture.frame(vec![
        key(Key::S, true, Modifiers::COMMAND),
        key(Key::S, false, Modifiers::COMMAND),
    ]);
    finish_actual_file_workflow(&mut fixture)?;
    assert_eq!(fs::read(&output)?, SOURCE);
    fixture.frame(vec![
        key(Key::O, true, Modifiers::COMMAND),
        key(Key::O, false, Modifiers::COMMAND),
    ]);
    wait_for(&mut fixture, |f, _| {
        f.node("File name", &[egui::accesskit::Role::TextInput])
            .is_ok()
    })?;
    filename(&mut fixture, "copied-化学.smiles")?;
    fixture.choose("Open")?;
    finish_actual_file_workflow(&mut fixture)?;
    let reopened = fixture.warm();
    assert_not_disclosed(&fixture, &reopened, "[13CH3][OH]");
    let shown = inspect(&mut fixture, 2, "[13CH3][OH]")?;
    assert_private(&fixture, &shown);
    assert_selected_consumer(notation(&fixture, 2)?.as_bytes())?;
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
        SOURCE
    );
    assert_eq!(fs::read(output)?, SOURCE);
    Ok(())
}

#[test]
fn before_inspect_selected_notation_is_absent_from_paint_and_all_current_accessible_strings()
-> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    open(&fixture, "not-automatically-disclosed.smiles", SOURCE)?;
    let before = fixture.state()?;
    let output = fixture.warm();
    assert_not_disclosed(&fixture, &output, "[13CH3][OH]");
    for id in reachable(&fixture) {
        if let Some(node) = fixture.nodes.get(&id) {
            for value in [node.name(), node.value(), node.description()]
                .into_iter()
                .flatten()
            {
                assert!(
                    !value.contains("[13CH3][OH]"),
                    "opening a source is not an explicit record disclosure"
                );
            }
        }
    }
    assert_eq!(fixture.state()?, before);
    Ok(())
}
