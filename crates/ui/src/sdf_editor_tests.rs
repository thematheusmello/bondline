//! Independent existing-occurrence editing through the actual App and file worker.
use super::{Fixture, TestResult, finish_actual_file_workflow, key};
use eframe::egui;
use egui::{Event, Key, Modifiers};
use serde_json::{Value, json};
use std::{
    fs,
    time::{Duration, Instant},
};

#[path = "../../../xtask/oracles/sdf_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse unchanged independent per-MOL consumers without altering old tests"
)]
mod sdf_support;

const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

fn wait_for(fixture: &mut Fixture, condition: impl Fn(&Fixture) -> bool) -> TestResult {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        fixture.frame(vec![]);
        if condition(fixture) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "real SDF field workflow did not finish: {:?}",
                fixture.app.error
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
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

#[test]
fn explicit_second_duplicate_edit_apply_file_save_undo_redo_preserves_the_complete_source()
-> TestResult {
    assert_eq!(SOURCE.len(), 777);
    let expected = [
        SOURCE.get(..378).ok_or("literal prefix")?,
        b"reviewed\nline two\n",
        SOURCE.get(385..).ok_or("literal suffix")?,
    ]
    .concat();
    assert_eq!(expected.len(), 788);
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = fixture.directory.path("field-editor-化学.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let original = fixture.state()?;
    assert_eq!(original["document"], Value::Null);
    assert_eq!(original["collection"]["record_count"], 2);
    assert_eq!(original["source"]["byte_length"], 777);
    assert_eq!(original["undo_depth"], 0);
    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    wait_for(&mut fixture, |fixture| {
        fixture
            .node("Field 3: DUP", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Field 3: DUP")?;
    wait_for(&mut fixture, |fixture| {
        text_value(fixture, "Record 1 field 3 value").is_ok_and(|text| text == "second")
    })?;
    assert_eq!(fixture.state()?, original);

    // The baseline must fail here: no future editor API or direct setter is used.
    fixture.choose("Edit record 1 field 3")?;
    let label = "Record 1 field 3 draft";
    wait_for(&mut fixture, |fixture| {
        text_value(fixture, label).is_ok_and(|text| text == "second")
    })?;
    let (id, _) = fixture.node(
        label,
        &[
            egui::accesskit::Role::TextInput,
            egui::accesskit::Role::MultilineTextInput,
        ],
    )?;
    assert!(
        !fixture
            .nodes
            .get(&id)
            .ok_or("editable draft node")?
            .is_read_only()
    );
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
    assert_eq!(
        fixture.state()?,
        original,
        "a local draft is not authored history"
    );
    assert_eq!(current_bytes(&fixture)?, SOURCE);
    assert_eq!(fs::read(&path)?, SOURCE);
    fixture.choose("Apply field value")?;
    wait_for(&mut fixture, |fixture| {
        fixture.state().is_ok_and(|state| state["revision"] == 2)
            && !fixture.app.sdf_fields.is_busy()
    })?;
    let applied = fixture.state()?;
    assert_eq!(applied["document"], Value::Null);
    assert_eq!(applied["collection"]["record_count"], 2);
    assert_eq!(applied["collection"]["byte_length"], 788);
    assert_eq!(applied["source"]["byte_length"], 777);
    assert_eq!(applied["source"]["modified"], true);
    assert_eq!(applied["file"]["dirty"], true);
    assert_eq!(applied["undo_depth"], 1);
    assert_eq!(applied["redo_depth"], 0);
    assert_eq!(current_bytes(&fixture)?, expected);
    assert_eq!(fs::read(&path)?, SOURCE, "Apply does not silently save");

    fixture.frame(vec![
        key(Key::S, true, Modifiers::COMMAND),
        key(Key::S, false, Modifiers::COMMAND),
    ]);
    finish_actual_file_workflow(&mut fixture)?;
    let saved = fixture.state()?;
    assert_eq!(saved["file"]["dirty"], false);
    assert_eq!(saved["source"]["modified"], true);
    assert_eq!(saved["source"]["byte_length"], 777);
    let bytes = fs::read(&path)?;
    assert_eq!(
        bytes, expected,
        "the actual FileWorkflow saves the whole literal candidate"
    );
    assert_eq!(bytes.get(..378), SOURCE.get(..378));
    assert_eq!(bytes.get(396..), SOURCE.get(385..));
    assert_eq!(
        bytes.get(473..),
        SOURCE.get(462..),
        "the complete other record is unchanged"
    );
    let first =
        sdf_support::describe_source("v3000_oracle.py", bytes.get(..344).ok_or("V3000 MOL")?)?;
    let second =
        sdf_support::describe_source("mol_oracle.py", bytes.get(473..755).ok_or("V2000 MOL")?)?;
    assert_eq!(first["source_bytes"], 344);
    assert_eq!(first["atoms"].as_array().ok_or("V3000 atoms")?.len(), 2);
    assert_eq!(first["bonds"].as_array().ok_or("V3000 bonds")?.len(), 1);
    assert_eq!(first["atoms"][0]["element"], 6);
    assert_eq!(first["atoms"][1]["element"], 6);
    assert_eq!(second["smiles"], "CC");
    assert_eq!(second["formula"], "C2H6");

    fixture.frame(vec![
        key(Key::Z, true, Modifiers::COMMAND),
        key(Key::Z, false, Modifiers::COMMAND),
    ]);
    let undone = fixture.state()?;
    assert_eq!(undone["revision"], 3);
    assert_eq!(undone["undo_depth"], 0);
    assert_eq!(undone["redo_depth"], 1);
    assert_eq!(undone["source"]["modified"], false);
    assert_eq!(undone["file"]["dirty"], true);
    assert_eq!(current_bytes(&fixture)?, SOURCE);
    let redo = Modifiers {
        shift: true,
        ..Modifiers::COMMAND
    };
    fixture.frame(vec![key(Key::Z, true, redo), key(Key::Z, false, redo)]);
    let redone = fixture.state()?;
    assert_eq!(redone["revision"], 4);
    assert_eq!(redone["undo_depth"], 1);
    assert_eq!(redone["redo_depth"], 0);
    assert_eq!(redone["source"]["modified"], true);
    assert_eq!(redone["file"]["dirty"], false);
    assert_eq!(current_bytes(&fixture)?, expected);
    assert_eq!(fs::read(&path)?, expected);
    fixture.command("document.open_sdf", json!({"path":path}))?;
    assert_eq!(current_bytes(&fixture)?, expected);
    assert_eq!(fixture.state()?["source"]["modified"], false);
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}

fn begin_changed_draft(fixture: &mut Fixture) -> TestResult<std::path::PathBuf> {
    let path = fixture.directory.path("pending-field-化学.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    wait_for(fixture, |fixture| {
        fixture
            .node("Field 3: DUP", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Field 3: DUP")?;
    wait_for(fixture, |fixture| {
        text_value(fixture, "Record 1 field 3 value").is_ok_and(|text| text == "second")
    })?;
    fixture.choose("Edit record 1 field 3")?;
    let label = "Record 1 field 3 draft";
    wait_for(fixture, |fixture| {
        text_value(fixture, label).is_ok_and(|text| text == "second")
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
    assert_eq!(text_value(fixture, label)?, "reviewed\nline two");
    Ok(path)
}

fn shows_text(output: &egui::FullOutput, expected: &str) -> bool {
    output.shapes.iter().any(|paint| {
        matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text() == expected
            && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
    })
}

#[test]
fn changed_draft_record_navigation_requires_explicit_apply_discard_or_cancel() -> TestResult {
    let expected = [
        SOURCE.get(..378).ok_or("literal prefix")?,
        b"reviewed\nline two\n",
        SOURCE.get(385..).ok_or("literal suffix")?,
    ]
    .concat();
    for decision in ["Cancel", "Discard", "Apply"] {
        let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
        let path = begin_changed_draft(&mut fixture)?;
        let before = fixture.state()?;
        assert_eq!(current_bytes(&fixture)?, SOURCE);
        fixture.choose("Inspect record 2 fields")?;
        wait_for(&mut fixture, |fixture| {
            fixture
                .node("Cancel", &[egui::accesskit::Role::Button])
                .is_ok()
        })?;
        assert!(shows_text(&fixture.frame(vec![]), "Unapplied field value"));
        assert_eq!(fixture.state()?, before);
        assert_eq!(current_bytes(&fixture)?, SOURCE);
        fixture.choose(decision)?;
        if decision == "Cancel" {
            wait_for(&mut fixture, |fixture| {
                text_value(fixture, "Record 1 field 3 draft")
                    .is_ok_and(|text| text == "reviewed\nline two")
            })?;
            assert_eq!(fixture.state()?, before);
            assert_eq!(current_bytes(&fixture)?, SOURCE);
            assert!(
                fixture
                    .node("Field 1: ORDINARY", &[egui::accesskit::Role::Button])
                    .is_err()
            );
        } else {
            wait_for(&mut fixture, |fixture| {
                fixture
                    .node("Field 1: ORDINARY", &[egui::accesskit::Role::Button])
                    .is_ok()
                    && !fixture.app.sdf_fields.is_busy()
            })?;
            assert!(shows_text(&fixture.frame(vec![]), "Record 2 fields"));
            assert!(
                fixture
                    .node(
                        "Record 1 field 3 draft",
                        &[
                            egui::accesskit::Role::TextInput,
                            egui::accesskit::Role::MultilineTextInput,
                        ]
                    )
                    .is_err()
            );
            let state = fixture.state()?;
            if decision == "Discard" {
                assert_eq!(state, before);
                assert_eq!(current_bytes(&fixture)?, SOURCE);
            } else {
                assert_eq!(state["revision"], 2);
                assert_eq!(state["undo_depth"], 1);
                assert_eq!(state["redo_depth"], 0);
                assert_eq!(state["file"]["dirty"], true);
                assert_eq!(state["source"]["modified"], true);
                assert_eq!(current_bytes(&fixture)?, expected);
            }
        }
        assert_eq!(fs::read(&path)?, SOURCE, "navigation never silently saves");
    }
    Ok(())
}

#[test]
fn apply_release_then_save_in_one_input_packet_publishes_the_applied_whole_source() -> TestResult {
    let expected = [
        SOURCE.get(..378).ok_or("literal prefix")?,
        b"reviewed\nline two\n",
        SOURCE.get(385..).ok_or("literal suffix")?,
    ]
    .concat();
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = begin_changed_draft(&mut fixture)?;
    fixture.warm();
    let (_, bounds) = fixture.node("Apply field value", &[egui::accesskit::Role::Button])?;
    let button = |pressed| Event::PointerButton {
        pos: bounds.center(),
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    fixture.frame(vec![Event::PointerMoved(bounds.center()), button(true)]);
    fixture.frame(vec![
        button(false),
        key(Key::S, true, Modifiers::COMMAND),
        key(Key::S, false, Modifiers::COMMAND),
    ]);
    wait_for(&mut fixture, |fixture| {
        fixture
            .state()
            .is_ok_and(|state| state["revision"] == 2 && state["file"]["dirty"] == false)
            && fs::read(&path).is_ok_and(|bytes| bytes == expected)
            && !fixture.app.sdf_fields.is_busy()
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    let state = fixture.state()?;
    assert_eq!(state["undo_depth"], 1);
    assert_eq!(state["redo_depth"], 0);
    assert_eq!(state["source"]["byte_length"], 777);
    assert_eq!(state["source"]["modified"], true);
    assert_eq!(state["collection"]["byte_length"], 788);
    assert_eq!(current_bytes(&fixture)?, expected);
    assert_eq!(fs::read(&path)?, expected);
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}

fn named_current_node<'a>(
    fixture: &'a Fixture,
    label: &str,
    role: egui::accesskit::Role,
) -> TestResult<&'a egui::accesskit::Node> {
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = fixture.nodes.get(&id)
        {
            stack.extend_from_slice(node.children());
        }
    }
    fixture
        .nodes
        .iter()
        .find_map(|(id, node)| {
            let named = node.name() == Some(label)
                || node.labelled_by().iter().any(|id| {
                    seen.contains(id)
                        && fixture
                            .nodes
                            .get(id)
                            .is_some_and(|labelled| labelled.name() == Some(label))
                });
            (seen.contains(id) && node.role() == role && named).then_some(node)
        })
        .ok_or_else(|| format!("current {role:?} {label} missing").into())
}

fn click_output(fixture: &mut Fixture, label: &str) -> TestResult<egui::FullOutput> {
    fixture.warm();
    let (_, bounds) = fixture.node(label, &[egui::accesskit::Role::Button])?;
    fixture.frame(vec![
        Event::PointerMoved(bounds.center()),
        super::button(bounds.center(), true),
    ]);
    Ok(fixture.frame(vec![super::button(bounds.center(), false)]))
}

#[test]
fn external_open_including_identical_bytes_detaches_the_draft_and_discard_keeps_replacement()
-> TestResult {
    for identical in [false, true] {
        let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
        let prior_path = begin_changed_draft(&mut fixture)?;
        let prior_stamp = fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .capture_stamp();
        let replacement: &[u8] = if identical {
            SOURCE
        } else {
            include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf")
        };
        let path = fixture.directory.path("replacement-化学.sdf");
        fs::write(&path, replacement)?;
        fixture.command("document.open_sdf", json!({"path":path}))?;
        let state = fixture.state()?;
        assert_ne!(
            fixture
                .app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .capture_stamp(),
            prior_stamp
        );
        fixture.frame(vec![]);
        assert_eq!(
            text_value(&fixture, "Record 1 field 3 draft")?,
            "reviewed\nline two"
        );
        assert!(
            named_current_node(&fixture, "Apply field value", egui::accesskit::Role::Button)?
                .is_disabled()
        );
        let output = click_output(&mut fixture, "Copy draft")?;
        assert_eq!(output.platform_output.copied_text, "reviewed\nline two");
        assert_eq!(fixture.state()?, state);
        assert_eq!(current_bytes(&fixture)?, replacement);
        fixture.choose("Inspect record 2 fields")?;
        wait_for(&mut fixture, |fixture| {
            fixture
                .node("Discard", &[egui::accesskit::Role::Button])
                .is_ok()
        })?;
        assert!(
            named_current_node(&fixture, "Apply", egui::accesskit::Role::Button)?.is_disabled()
        );
        fixture.choose("Discard")?;
        wait_for(&mut fixture, |fixture| {
            !fixture.app.sdf_fields.is_busy()
                && fixture
                    .node(
                        "Record 1 field 3 draft",
                        &[
                            egui::accesskit::Role::TextInput,
                            egui::accesskit::Role::MultilineTextInput,
                        ],
                    )
                    .is_err()
        })?;
        assert_eq!(fixture.state()?, state);
        assert_eq!(current_bytes(&fixture)?, replacement);
        assert_eq!(fs::read(&path)?, replacement);
        assert_eq!(fs::read(&prior_path)?, SOURCE);
        assert!(shows_text(&fixture.frame(vec![]), "Record 2 fields"));
    }
    Ok(())
}

fn request_close(fixture: &mut Fixture, remote: bool) -> TestResult<egui::FullOutput> {
    if remote {
        let revision = fixture.state()?["revision"].as_u64().ok_or("revision")?;
        fixture.command("document.close", json!({"expected_revision":revision}))?;
        Ok(fixture.frame(vec![]))
    } else {
        Ok(fixture.frame(vec![
            key(Key::W, true, Modifiers::COMMAND),
            key(Key::W, false, Modifiers::COMMAND),
        ]))
    }
}

fn emitted_close(output: &egui::FullOutput) -> bool {
    output.viewport_output.values().any(|viewport| {
        viewport
            .commands
            .iter()
            .any(|command| matches!(command, egui::ViewportCommand::Close))
    })
}

#[test]
fn actual_app_and_remote_close_preserve_unapplied_text_until_explicit_decision() -> TestResult {
    for remote in [false, true] {
        let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
        let path = begin_changed_draft(&mut fixture)?;
        wait_for(&mut fixture, |fixture| {
            !fixture.app.preview.is_busy()
                && !fixture.app.sdf_fields.is_busy()
                && !fixture.app.executor.has_active_work()
        })?;
        assert_eq!(
            fixture.state()?["file"]["dirty"],
            false,
            "Engine is clean; only the UI draft differs"
        );
        let output = request_close(&mut fixture, remote)?;
        assert!(!emitted_close(&output));
        assert_ne!(fixture.state()?["close"], "committed");
        wait_for(&mut fixture, |fixture| {
            fixture
                .node("Cancel", &[egui::accesskit::Role::Button])
                .is_ok()
        })?;
        assert!(shows_text(&fixture.frame(vec![]), "Unapplied field value"));
        fixture.choose("Cancel")?;
        assert_eq!(fixture.state()?["close"], "open");
        assert_eq!(
            text_value(&fixture, "Record 1 field 3 draft")?,
            "reviewed\nline two"
        );
        assert_eq!(current_bytes(&fixture)?, SOURCE);
        let output = request_close(&mut fixture, remote)?;
        assert!(!emitted_close(&output));
        wait_for(&mut fixture, |fixture| {
            fixture
                .node("Discard", &[egui::accesskit::Role::Button])
                .is_ok()
        })?;
        fixture.choose("Discard")?;
        wait_for(&mut fixture, |fixture| {
            fixture
                .state()
                .is_ok_and(|state| state["close"] == "committed")
        })?;
        assert_eq!(fixture.state()?["revision"], 1);
        assert_eq!(fixture.state()?["undo_depth"], 0);
        assert_eq!(current_bytes(&fixture)?, SOURCE);
        assert_eq!(fs::read(&path)?, SOURCE);
    }

    // Apply must issue a fresh shared close after it creates a dirty authoring revision.
    let expected = [
        SOURCE.get(..378).ok_or("prefix")?,
        b"reviewed\nline two\n",
        SOURCE.get(385..).ok_or("suffix")?,
    ]
    .concat();
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = begin_changed_draft(&mut fixture)?;
    wait_for(&mut fixture, |fixture| {
        !fixture.app.preview.is_busy() && !fixture.app.executor.has_active_work()
    })?;
    let output = request_close(&mut fixture, true)?;
    assert!(!emitted_close(&output));
    wait_for(&mut fixture, |fixture| {
        fixture
            .node("Apply", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Apply")?;
    wait_for(&mut fixture, |fixture| {
        fixture
            .state()
            .is_ok_and(|state| state["revision"] == 2 && state["close"] == "needs_decision")
            && fixture
                .node("Save", &[egui::accesskit::Role::Button])
                .is_ok()
    })?;
    assert_eq!(current_bytes(&fixture)?, expected);
    assert_eq!(fs::read(&path)?, SOURCE);
    assert_eq!(fixture.state()?["file"]["dirty"], true);
    fixture.choose("Save")?;
    finish_actual_file_workflow(&mut fixture)?;
    wait_for(&mut fixture, |fixture| {
        fixture
            .state()
            .is_ok_and(|state| state["close"] == "committed")
    })?;
    assert_eq!(fs::read(&path)?, expected);
    assert_eq!(current_bytes(&fixture)?, expected);
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    assert_eq!(fixture.state()?["source"]["modified"], true);
    Ok(())
}

#[test]
fn same_value_apply_preserves_crlf_source_state_and_shared_snapshot_allocations() -> TestResult {
    let source = SOURCE
        .iter()
        .flat_map(|byte| {
            if *byte == b'\n' {
                vec![b'\r', b'\n']
            } else {
                vec![*byte]
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(source.len(), 823);
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = fixture.directory.path("same-value-crlf.sdf");
    fs::write(&path, &source)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let before = fixture.state()?;
    let (owner, stamp) = {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        (engine.content().clone(), engine.capture_stamp())
    };
    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    wait_for(&mut fixture, |fixture| {
        fixture
            .node("Field 3: DUP", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Field 3: DUP")?;
    wait_for(&mut fixture, |fixture| {
        text_value(fixture, "Record 1 field 3 value").is_ok_and(|text| text == "second")
    })?;
    fixture.choose("Edit record 1 field 3")?;
    wait_for(&mut fixture, |fixture| {
        text_value(fixture, "Record 1 field 3 draft").is_ok_and(|text| text == "second")
    })?;
    fixture.choose("Apply field value")?;
    wait_for(&mut fixture, |fixture| {
        !fixture.app.sdf_fields.is_busy()
            && fixture
                .node(
                    "Record 1 field 3 draft",
                    &[
                        egui::accesskit::Role::TextInput,
                        egui::accesskit::Role::MultilineTextInput,
                    ],
                )
                .is_err()
    })?;
    assert_eq!(fixture.state()?, before);
    let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
    assert_eq!(engine.content(), &owner);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(std::ptr::eq(
        engine.content().sdf()?.as_bytes(),
        owner.sdf()?.as_bytes()
    ));
    assert!(std::ptr::eq(
        engine.content().sdf()?.records(),
        owner.sdf()?.records()
    ));
    assert_eq!(engine.content().sdf()?.as_bytes(), source);
    assert_eq!(fs::read(&path)?, source);
    Ok(())
}

#[test]
fn invalid_line_apply_then_save_keeps_the_draft_and_never_publishes_old_content() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = begin_changed_draft(&mut fixture)?;
    let label = "Record 1 field 3 draft";
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
    let invalid = "a".repeat(201);
    fixture.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Text(invalid.clone()),
    ]);
    assert_eq!(text_value(&fixture, label)?, invalid);
    let before = fixture.state()?;
    let file = fs::OpenOptions::new().write(true).open(&path)?;
    file.set_times(
        fs::FileTimes::new()
            .set_modified(std::time::UNIX_EPOCH + Duration::from_secs(1_000_000_000)),
    )?;
    let modified = fs::metadata(&path)?.modified()?;
    fixture.warm();
    let (_, bounds) = fixture.node("Apply field value", &[egui::accesskit::Role::Button])?;
    fixture.frame(vec![
        Event::PointerMoved(bounds.center()),
        super::button(bounds.center(), true),
    ]);
    fixture.frame(vec![
        super::button(bounds.center(), false),
        key(Key::S, true, Modifiers::COMMAND),
        key(Key::S, false, Modifiers::COMMAND),
    ]);
    wait_for(&mut fixture, |fixture| {
        !fixture.app.sdf_fields.is_busy()
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    let output = fixture.frame(vec![]);
    assert!(output.shapes.iter().any(|paint| matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text().contains("No waiting Save/Open/close action was started"))));
    assert_eq!(text_value(&fixture, label)?, invalid);
    assert_eq!(fixture.state()?, before);
    assert_eq!(current_bytes(&fixture)?, SOURCE);
    assert_eq!(fs::read(&path)?, SOURCE);
    assert_eq!(
        fs::metadata(&path)?.modified()?,
        modified,
        "even byte-identical old-source publication is forbidden"
    );
    Ok(())
}

#[test]
fn apply_before_previous_fields_refreshes_then_installs_the_requested_prior_page() -> TestResult {
    let original = include_bytes!("../../io/tests/fixtures/sdf/ordered-fields-v2000.sdf");
    let mut source = original
        .get(..282)
        .ok_or("unchanged original MOL")?
        .to_vec();
    for field in 1..=65 {
        source.extend_from_slice(format!("> <F{field:02}>\nold{field:02}\n\n").as_bytes());
    }
    source.extend_from_slice(b"$$$$\n");
    assert_eq!(source.len(), 1262);
    assert_eq!(source.get(1250..1256), Some(b"old65\n".as_slice()));
    let expected = [
        source.get(..1250).ok_or("literal prefix")?,
        b"updated65\n",
        source.get(1256..).ok_or("literal suffix")?,
    ]
    .concat();
    assert_eq!(expected.len(), 1266);
    let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
    let path = fixture.directory.path("apply-previous-fields.sdf");
    fs::write(&path, &source)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    fixture.choose("Records")?;
    fixture.choose("Inspect record 1 fields")?;
    wait_for(&mut fixture, |fixture| {
        fixture
            .node("Field 1: F01", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Next fields")?;
    wait_for(&mut fixture, |fixture| {
        fixture
            .node("Field 65: F65", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Field 65: F65")?;
    wait_for(&mut fixture, |fixture| {
        text_value(fixture, "Record 1 field 65 value").is_ok_and(|text| text == "old65")
    })?;
    fixture.choose("Edit record 1 field 65")?;
    let label = "Record 1 field 65 draft";
    wait_for(&mut fixture, |fixture| {
        text_value(fixture, label).is_ok_and(|text| text == "old65")
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
        Event::Text("updated65".into()),
    ]);
    assert_eq!(text_value(&fixture, label)?, "updated65");
    fixture.choose("Previous fields")?;
    wait_for(&mut fixture, |fixture| {
        fixture
            .node("Apply", &[egui::accesskit::Role::Button])
            .is_ok()
    })?;
    fixture.choose("Apply")?;
    wait_for(&mut fixture, |fixture| {
        fixture.state().is_ok_and(|state| state["revision"] == 2)
            && !fixture.app.sdf_fields.is_busy()
            && fixture
                .node("Field 1: F01", &[egui::accesskit::Role::Button])
                .is_ok()
            && fixture
                .node("Field 65: F65", &[egui::accesskit::Role::Button])
                .is_err()
    })?;
    assert_eq!(current_bytes(&fixture)?, expected);
    assert_eq!(fixture.state()?["undo_depth"], 1);
    assert_eq!(fixture.state()?["file"]["dirty"], true);
    assert_eq!(fs::read(&path)?, source);
    assert!(
        fixture
            .node("Field 1: F01", &[egui::accesskit::Role::Button])
            .is_ok(),
        "Apply then Previous must install the requested first field page after refresh retirement"
    );
    assert!(
        fixture
            .node("Field 65: F65", &[egui::accesskit::Role::Button])
            .is_err()
    );
    Ok(())
}

#[test]
fn external_requested_save_waits_for_draft_decision_and_preserves_or_retickets_authority()
-> TestResult {
    let expected = [
        SOURCE.get(..378).ok_or("literal prefix")?,
        b"reviewed\nline two\n",
        SOURCE.get(385..).ok_or("literal suffix")?,
    ]
    .concat();
    for decision in ["Cancel", "Discard", "Apply"] {
        let mut fixture = Fixture::new(1160.0, 800.0, 2.0)?;
        let path = begin_changed_draft(&mut fixture)?;
        wait_for(&mut fixture, |fixture| {
            !fixture.app.sdf_fields.is_busy()
                && !fixture.app.preview.is_busy()
                && !fixture.app.executor.has_active_work()
        })?;
        let before = fixture.state()?;
        assert_eq!(before["revision"], 1);
        assert_eq!(before["ui_file"]["epoch"], 0);
        let file = fs::OpenOptions::new().write(true).open(&path)?;
        file.set_times(
            fs::FileTimes::new()
                .set_modified(std::time::UNIX_EPOCH + Duration::from_secs(1_000_000_000)),
        )?;
        let old_modified = fs::metadata(&path)?.modified()?;
        let requested =
            fixture.command("ui.file", json!({"expected_revision":1,"action":"save"}))?;
        assert_eq!(requested["ui_file"]["epoch"], 1);
        assert_eq!(requested["ui_file"]["request"]["id"], 1);
        assert_eq!(requested["ui_file"]["request"]["phase"], "requested");
        assert_eq!(requested["ui_file"]["request"]["input_revision"], 1);
        wait_for(&mut fixture, |fixture| {
            fixture
                .node("Cancel", &[egui::accesskit::Role::Button])
                .is_ok()
        })?;
        assert!(shows_text(&fixture.frame(vec![]), "Unapplied field value"));
        let held = fixture.state()?;
        assert_eq!(
            held["ui_file"], requested["ui_file"],
            "no host adoption before a draft decision"
        );
        assert_eq!(held["revision"], 1);
        assert_eq!(held["undo_depth"], 0);
        assert_eq!(held["file"]["dirty"], false);
        assert_eq!(current_bytes(&fixture)?, SOURCE);
        assert_eq!(fs::read(&path)?, SOURCE);
        assert_eq!(
            fs::metadata(&path)?.modified()?,
            old_modified,
            "no early publication even of identical old bytes"
        );
        assert!(!fixture.app.files.is_busy());
        assert!(!fixture.app.executor.has_active_work());
        fixture.choose(decision)?;
        if decision == "Cancel" {
            let cancelled = fixture.state()?;
            assert!(cancelled["ui_file"]["request"].is_null());
            assert_eq!(
                cancelled["ui_file"]["epoch"], 2,
                "real ui.file Cancel advances epoch; fake completion does not"
            );
            assert_eq!(cancelled["revision"], 1);
            assert_eq!(cancelled["undo_depth"], 0);
            assert_eq!(cancelled["redo_depth"], 0);
            assert_eq!(cancelled["file"]["dirty"], false);
            assert_eq!(cancelled["source"]["modified"], false);
            assert_eq!(
                text_value(&fixture, "Record 1 field 3 draft")?,
                "reviewed\nline two"
            );
            assert_eq!(current_bytes(&fixture)?, SOURCE);
            assert_eq!(fs::read(&path)?, SOURCE);
            assert_eq!(fs::metadata(&path)?.modified()?, old_modified);
            assert!(!fixture.app.files.is_busy());
            continue;
        }
        let ticket = if decision == "Apply" { 2 } else { 1 };
        let running_ticket = |fixture: &Fixture| {
            fixture.state().is_ok_and(|state| {
                state["ui_file"]["epoch"] == ticket
                    && state["ui_file"]["request"]["id"] == ticket
                    && state["ui_file"]["request"]["phase"] == "running"
            })
        };
        if !running_ticket(&fixture) {
            wait_for(&mut fixture, running_ticket)?;
        }
        let running = fixture.state()?;
        assert!(
            fixture.app.files.is_busy(),
            "the observed ticket owns actual FileWorkflow work"
        );
        assert_eq!(running["ui_file"]["request"]["action"], "save");
        assert_eq!(running["ui_file"]["request"]["input_revision"], ticket);
        assert_eq!(running["revision"], ticket);
        finish_actual_file_workflow(&mut fixture)?;
        wait_for(&mut fixture, |fixture| {
            !fixture.app.sdf_fields.is_busy()
                && !fixture.app.preview.is_busy()
                && !fixture.app.executor.has_active_work()
        })?;
        let saved = fixture.state()?;
        assert!(saved["ui_file"]["request"].is_null());
        assert_eq!(saved["ui_file"]["epoch"], ticket);
        assert_eq!(saved["revision"], ticket);
        assert_eq!(saved["file"]["dirty"], false);
        assert_eq!(saved["redo_depth"], 0);
        assert_eq!(saved["source"]["byte_length"], 777);
        if decision == "Apply" {
            assert_eq!(saved["undo_depth"], 1);
            assert_eq!(saved["source"]["modified"], true);
            assert_eq!(saved["collection"]["byte_length"], 788);
            assert_eq!(current_bytes(&fixture)?, expected);
            assert_eq!(fs::read(&path)?, expected);
        } else {
            assert_eq!(saved["undo_depth"], 0);
            assert_eq!(saved["source"]["modified"], false);
            assert_eq!(saved["collection"]["byte_length"], 777);
            assert_eq!(current_bytes(&fixture)?, SOURCE);
            assert_eq!(fs::read(&path)?, SOURCE);
        }
        assert!(
            fixture.app.error.is_none(),
            "decision={decision}: {:?}",
            fixture.app.error
        );
    }
    Ok(())
}
