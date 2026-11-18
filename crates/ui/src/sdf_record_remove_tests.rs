//! Independent record-removal acceptance through the actual App and file workflow.
use super::{Fixture, TestResult, button, finish_actual_file_workflow, key};
use eframe::egui;
use egui::{Event, Key, Modifiers};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    fs,
    path::Path,
    time::{Duration, Instant},
};

#[path = "../../../xtask/oracles/sdf_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse unchanged independent raw/per-MOL consumers without altering old tests"
)]
mod sdf_support;

const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

#[test]
fn actual_app_exposes_enabled_removal_for_each_record_of_a_complete_sdf() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = fixture.directory.path("record-removal-化学.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let original = fixture.state()?;
    assert_eq!(original["collection"]["record_count"], 2);
    assert_eq!(original["collection"]["byte_length"], 777);
    fixture.choose("Records")?;
    // Runtime baseline: the current real App has no Remove control. No future Rust
    // API or direct mutation stands in for this control and its accessible role.
    let (id, _) = fixture.node("Remove record 1", &[egui::accesskit::Role::Button])?;
    assert!(!fixture.nodes.get(&id).ok_or("Remove button")?.is_disabled());
    assert_eq!(fixture.state()?, original);
    assert_eq!(fs::read(&path)?, SOURCE);
    Ok(())
}

fn wait_for(
    fixture: &mut Fixture,
    condition: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = fixture.frame(vec![]);
        if condition(fixture, &output) {
            return Ok(output);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "real record-removal workflow did not finish: {:?}",
                fixture.app.error
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
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

fn current_control<'a>(fixture: &'a Fixture, name: &str) -> TestResult<&'a egui::accesskit::Node> {
    let mut stack = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    let mut seen = HashSet::new();
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = fixture.nodes.get(&id)
        {
            if node.name() == Some(name) && node.role() == egui::accesskit::Role::Button {
                return Ok(node);
            }
            stack.extend_from_slice(node.children());
        }
    }
    Err(format!("current {name} button missing").into())
}

fn text_value(fixture: &Fixture, label: &str) -> TestResult<String> {
    let (id, _) = fixture.node(
        label,
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    let mut text = String::new();
    let mut stack = vec![id];
    while let Some(id) = stack.pop() {
        let node = fixture.nodes.get(&id).ok_or("current text node")?;
        if node.role() == egui::accesskit::Role::InlineTextBox {
            text.push_str(node.value().ok_or("literal inline text")?);
        }
        stack.extend(node.children().iter().rev().copied());
    }
    Ok(text)
}

fn set_text(fixture: &mut Fixture, label: &str, text: &str) -> TestResult {
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
        Event::Text(text.to_owned()),
    ]);
    assert_eq!(text_value(fixture, label)?, text);
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

fn shows_text(output: &egui::FullOutput, expected: &str) -> bool {
    output.shapes.iter().any(|paint| {
        matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text() == expected
            && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
    })
}

fn settled(fixture: &Fixture) -> bool {
    !fixture.app.sdf_fields.is_busy()
        && !fixture.app.files.is_busy()
        && !fixture.app.executor.has_active_work()
}

fn history(fixture: &mut Fixture, redo: bool) {
    let modifiers = Modifiers {
        shift: redo,
        ..Modifiers::COMMAND
    };
    fixture.frame(vec![
        key(Key::Z, true, modifiers),
        key(Key::Z, false, modifiers),
    ]);
}

fn assert_actual_survivor_file(path: &Path, expected: &[u8]) -> TestResult {
    let bytes = fs::read(path)?;
    assert_eq!(
        bytes, expected,
        "actual FileWorkflow must save the literal survivor"
    );
    assert_eq!(bytes.len(), 315);
    let raw = sdf_support::describe_source("sdf_oracle.py", &bytes)?;
    assert_eq!(raw["records"].as_array().ok_or("raw records")?.len(), 1);
    assert_eq!(raw["records"][0]["range"], json!([0, 315]));
    assert_eq!(raw["records"][0]["mol_range"], json!([0, 282]));
    assert_eq!(raw["records"][0]["delimiter_range"], json!([310, 315]));
    assert_eq!(raw["records"][0]["fields"][0]["name"], "ORDINARY");
    assert_eq!(
        raw["records"][0]["fields"][0]["value_lines"],
        json!(["second record"])
    );
    let molecule = sdf_support::describe_source(
        "mol_oracle.py",
        bytes.get(..282).ok_or("literal retained MOL")?,
    )?;
    assert_eq!(molecule["smiles"], "CC");
    assert_eq!(molecule["formula"], "C2H6");
    assert_eq!(molecule["bonds"], json!([{"begin":0,"end":1,"order":1.0}]));
    Ok(())
}

#[test]
fn cancel_then_confirm_first_record_preserves_literal_history_and_unicode_save_reopen() -> TestResult
{
    let expected = SOURCE.get(462..).ok_or("literal second-record source")?;
    assert_eq!(expected.len(), 315);
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = fixture.directory.path("original-化学.sdf");
    fs::write(&input, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":input}))?;
    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Field 3: DUP", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Field 3: DUP")?;
    wait_for(&mut fixture, |fixture, _| {
        text_value(fixture, "Record 1 field 3 value").is_ok_and(|text| text == "second")
    })?;
    let original = fixture.state()?;
    fixture.choose("Remove record 1")?;
    let confirmation = wait_for(&mut fixture, |fixture, output| {
        current_control(fixture, "Confirm record removal").is_ok()
            && shows_text(output, "Remove SDF record")
    })?;
    assert!(shows_text(&confirmation, "Remove SDF record"));
    assert_eq!(fixture.state()?, original);
    assert_eq!(current_bytes(&fixture)?, SOURCE);
    fixture.choose("Keep record")?;
    assert_eq!(
        fixture.state()?,
        original,
        "Cancel is a complete authoring no-op"
    );
    assert_eq!(current_bytes(&fixture)?, SOURCE);
    assert_eq!(fs::read(&input)?, SOURCE);
    assert!(current_control(&fixture, "Confirm record removal").is_err());

    fixture.choose("Remove record 1")?;
    wait_for(&mut fixture, |fixture, _| {
        current_control(fixture, "Confirm record removal").is_ok()
    })?;
    fixture.choose("Confirm record removal")?;
    wait_for(&mut fixture, |fixture, _| {
        current_bytes(fixture).is_ok_and(|bytes| bytes == expected) && settled(fixture)
    })?;
    let removed = fixture.state()?;
    assert_eq!(removed["document"], Value::Null);
    assert_eq!(removed["collection"]["record_count"], 1);
    assert_eq!(removed["collection"]["byte_length"], 315);
    assert_eq!(removed["source"]["byte_length"], 777);
    assert_eq!(removed["source"]["modified"], true);
    assert_eq!(removed["file"]["dirty"], true);
    assert_eq!(removed["undo_depth"], 1);
    assert_eq!(removed["redo_depth"], 0);
    assert!(
        fixture
            .node("Field 3: DUP", &[egui::accesskit::Role::Button])
            .is_err()
    );
    assert!(text_value(&fixture, "Record 1 field 3 value").is_err());
    assert!(current_control(&fixture, "Remove record 1")?.is_disabled());
    assert!(current_control(&fixture, "Remove record 2").is_err());
    assert_eq!(fs::read(&input)?, SOURCE, "removal cannot silently Save");

    history(&mut fixture, false);
    assert_eq!(current_bytes(&fixture)?, SOURCE);
    assert_eq!(fixture.state()?["undo_depth"], 0);
    assert_eq!(fixture.state()?["redo_depth"], 1);
    assert_eq!(fixture.state()?["source"]["modified"], false);
    history(&mut fixture, true);
    assert_eq!(current_bytes(&fixture)?, expected);
    assert_eq!(fixture.state()?["undo_depth"], 1);
    assert_eq!(fixture.state()?["redo_depth"], 0);

    let output = fixture.directory.path("retained-β-化学.sdf");
    let save_as = Modifiers {
        shift: true,
        ..Modifiers::COMMAND
    };
    fixture.frame(vec![
        key(Key::S, true, save_as),
        key(Key::S, false, save_as),
    ]);
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("File name", &[egui::accesskit::Role::TextInput])
            .is_ok()
    })?;
    filename(&mut fixture, "retained-β-化学.sdf")?;
    fixture.choose("Save")?;
    finish_actual_file_workflow(&mut fixture)?;
    assert_actual_survivor_file(&output, expected)?;
    assert_eq!(fs::read(&input)?, SOURCE);
    let saved = fixture.state()?;
    assert_eq!(saved["file"]["format"], "sdf");
    assert_eq!(
        saved["file"]["path"],
        fs::canonicalize(&output)?.to_str().ok_or("Unicode path")?
    );
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["undo_depth"], 1);
    fixture.frame(vec![
        key(Key::O, true, Modifiers::COMMAND),
        key(Key::O, false, Modifiers::COMMAND),
    ]);
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("File name", &[egui::accesskit::Role::TextInput])
            .is_ok()
    })?;
    filename(&mut fixture, "retained-β-化学.sdf")?;
    fixture.choose("Open")?;
    finish_actual_file_workflow(&mut fixture)?;
    assert_eq!(current_bytes(&fixture)?, expected);
    let reopened = fixture.state()?;
    assert_eq!(reopened["collection"]["record_count"], 1);
    assert_eq!(reopened["collection"]["records"][0]["id"], 1);
    assert_eq!(reopened["source"]["byte_length"], 315);
    assert_eq!(reopened["source"]["modified"], false);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["redo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    fixture.warm();
    assert!(current_control(&fixture, "Remove record 1")?.is_disabled());
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}

#[test]
fn survivor_draft_cancel_discard_apply_and_invalid_apply_resolve_before_fresh_confirmation()
-> TestResult {
    let edited_whole = [
        SOURCE.get(..757).ok_or("value prefix")?,
        b"reviewed\nline two\n",
        SOURCE.get(771..).ok_or("value suffix")?,
    ]
    .concat();
    let edited_survivor = [
        SOURCE.get(462..757).ok_or("survivor prefix")?,
        b"reviewed\nline two\n",
        SOURCE.get(771..).ok_or("survivor suffix")?,
    ]
    .concat();
    assert_eq!(edited_whole.len(), 781);
    assert_eq!(edited_survivor.len(), 319);
    for decision in ["Cancel", "Discard", "Apply", "invalid Apply"] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = fixture.directory.path("survivor-draft-化学.sdf");
        fs::write(&path, SOURCE)?;
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
            text_value(fixture, "Record 2 field 1 value").is_ok_and(|text| text == "second record")
        })?;
        fixture.choose("Edit record 2 field 1")?;
        let label = "Record 2 field 1 draft";
        wait_for(&mut fixture, |fixture, _| {
            text_value(fixture, label).is_ok_and(|text| text == "second record")
        })?;
        let value = if decision == "invalid Apply" {
            "a".repeat(201)
        } else {
            "reviewed\nline two".to_owned()
        };
        set_text(&mut fixture, label, &value)?;
        let before = fixture.state()?;
        assert_eq!(current_bytes(&fixture)?, SOURCE);
        fixture.choose("Remove record 1")?;
        wait_for(&mut fixture, |fixture, output| {
            current_control(fixture, "Cancel").is_ok()
                && shows_text(output, "Unapplied field value")
        })?;
        assert_eq!(fixture.state()?, before);
        assert!(current_control(&fixture, "Confirm record removal").is_err());
        fixture.choose(if decision == "invalid Apply" {
            "Apply"
        } else {
            decision
        })?;
        if decision == "Cancel" {
            assert_eq!(text_value(&fixture, label)?, value);
            assert_eq!(fixture.state()?, before);
            assert_eq!(current_bytes(&fixture)?, SOURCE);
            assert!(current_control(&fixture, "Confirm record removal").is_err());
        } else if decision == "invalid Apply" {
            let output = wait_for(&mut fixture, |fixture, _| {
                settled(fixture) && text_value(fixture, label).is_ok_and(|text| text == value)
            })?;
            assert!(output.shapes.iter().any(|paint| matches!(&paint.shape, egui::Shape::Text(text) if text.galley.text().contains("200"))));
            assert_eq!(fixture.state()?, before);
            assert_eq!(current_bytes(&fixture)?, SOURCE);
            assert!(current_control(&fixture, "Confirm record removal").is_err());
        } else {
            wait_for(&mut fixture, |fixture, output| {
                current_control(fixture, "Confirm record removal").is_ok()
                    && shows_text(output, "Remove SDF record")
            })?;
            let applied = decision == "Apply";
            assert_eq!(
                current_bytes(&fixture)?,
                if applied {
                    edited_whole.as_slice()
                } else {
                    SOURCE
                }
            );
            assert_eq!(fixture.state()?["undo_depth"], if applied { 1 } else { 0 });
            assert!(text_value(&fixture, label).is_err());
            fixture.choose("Confirm record removal")?;
            let expected = if applied {
                edited_survivor.as_slice()
            } else {
                SOURCE.get(462..).ok_or("literal survivor")?
            };
            wait_for(&mut fixture, |fixture, _| {
                current_bytes(fixture).is_ok_and(|bytes| bytes == expected) && settled(fixture)
            })?;
            assert_eq!(fixture.state()?["collection"]["record_count"], 1);
            assert_eq!(fixture.state()?["undo_depth"], if applied { 2 } else { 1 });
            assert_eq!(fixture.state()?["redo_depth"], 0);
            history(&mut fixture, false);
            assert_eq!(
                current_bytes(&fixture)?,
                if applied {
                    edited_whole.as_slice()
                } else {
                    SOURCE
                }
            );
            if applied {
                assert_eq!(fixture.state()?["undo_depth"], 1);
                history(&mut fixture, false);
                assert_eq!(current_bytes(&fixture)?, SOURCE);
            }
            assert_eq!(fixture.state()?["undo_depth"], 0);
        }
        assert_eq!(
            fs::read(&path)?,
            SOURCE,
            "draft resolution and removal never silently Save"
        );
    }
    Ok(())
}

#[test]
fn confirm_release_then_save_in_one_packet_writes_the_removed_source_through_file_workflow()
-> TestResult {
    let expected = SOURCE.get(462..).ok_or("literal survivor")?;
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = fixture.directory.path("ordered-remove-save-化学.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    fixture.choose("Records")?;
    fixture.choose("Remove record 1")?;
    wait_for(&mut fixture, |fixture, _| {
        current_control(fixture, "Confirm record removal").is_ok()
    })?;
    fixture.warm();
    let (_, bounds) = fixture.node("Confirm record removal", &[egui::accesskit::Role::Button])?;
    fixture.frame(vec![
        Event::PointerMoved(bounds.center()),
        button(bounds.center(), true),
    ]);
    fixture.frame(vec![
        button(bounds.center(), false),
        key(Key::S, true, Modifiers::COMMAND),
        key(Key::S, false, Modifiers::COMMAND),
    ]);
    wait_for(&mut fixture, |fixture, _| {
        current_bytes(fixture).is_ok_and(|bytes| bytes == expected)
            && fs::read(&path).is_ok_and(|bytes| bytes == expected)
            && fixture.state().is_ok_and(|state| {
                state["file"]["dirty"] == false && state["ui_file"]["request"].is_null()
            })
            && settled(fixture)
    })?;
    assert_actual_survivor_file(&path, expected)?;
    let state = fixture.state()?;
    assert_eq!(state["revision"], 2);
    assert_eq!(state["undo_depth"], 1);
    assert_eq!(state["redo_depth"], 0);
    assert_eq!(state["collection"]["record_count"], 1);
    assert_eq!(state["source"]["byte_length"], 777);
    assert_eq!(state["source"]["modified"], true);
    assert_eq!(state["file"]["format"], "sdf");
    assert_eq!(
        state["file"]["path"],
        fs::canonicalize(&path)?.to_str().ok_or("canonical path")?
    );
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}
