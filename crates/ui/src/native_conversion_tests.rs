//! Independent restricted SMILES conversion through actual App frames.
use super::{Fixture, TestResult};
use eframe::egui;
use std::time::{Duration, Instant};

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
            return Err("current actionable SMILES conversion control/form missing".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn actual_file_menu_offers_restricted_smiles_conversion_form_without_replacing_drawing()
-> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let original = fixture.state()?;
    fixture.choose("File")?;
    // Missing actual role is the baseline RED; a future Rust API or direct
    // conversion command cannot stand in for the App's discoverable control.
    fixture.node("Convert SMILES file…", &[egui::accesskit::Role::Button])?;
    wait_for(&mut fixture, |fixture, _| {
        fixture
            .node("Convert SMILES file…", &[egui::accesskit::Role::Button])
            .is_ok_and(|(id, _)| {
                fixture
                    .nodes
                    .get(&id)
                    .is_some_and(|node| !node.is_disabled())
            })
    })?;
    fixture.choose("Convert SMILES file…")?;
    wait_for(&mut fixture, |_, output| {
        output.shapes.iter().any(|paint| {
            matches!(&paint.shape, egui::Shape::Text(text)
                if text.galley.text() == "Convert SMILES file"
                && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
        })
    })?;
    let mut after = fixture.state()?;
    after["ui_file"] = original["ui_file"].clone();
    assert_eq!(
        after, original,
        "opening a conversion form preserves the drawing"
    );
    Ok(())
}

// Additive acceptance follows the frozen 2,067-byte missing-control runtime RED.
use super::{button, canvas, finish_actual_file_workflow, key, projected, tap};
use egui::{Event, Key, Modifiers};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[path = "../../cli/tests/support/conversion.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse unchanged external conversion consumers"
)]
mod conversion_support;
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse unchanged literal custom-geometry consumer"
)]
mod files_support;

fn has_text(output: &egui::FullOutput, needle: &str) -> bool {
    output.shapes.iter().any(|paint| {
        matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text().contains(needle))
    })
}

fn enabled(fixture: &Fixture, label: &str) -> bool {
    fixture
        .node(label, &[egui::accesskit::Role::Button])
        .is_ok_and(|(id, _)| {
            fixture
                .nodes
                .get(&id)
                .is_some_and(|node| !node.is_disabled())
        })
}

fn activate(fixture: &mut Fixture, label: &str) -> TestResult<egui::FullOutput> {
    let (id, _) = fixture.node(label, &[egui::accesskit::Role::Button])?;
    Ok(fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: id,
            data: None,
        },
    )]))
}

fn field(fixture: &Fixture, label: &str) -> TestResult<String> {
    let (id, _) = fixture.node(label, &[egui::accesskit::Role::TextInput])?;
    let mut stack = vec![id];
    let mut text = String::new();
    while let Some(id) = stack.pop() {
        let node = fixture.nodes.get(&id).ok_or("current text node")?;
        if node.role() == egui::accesskit::Role::InlineTextBox {
            text.push_str(node.value().ok_or("inline text value")?);
        }
        stack.extend(node.children().iter().rev().copied());
    }
    Ok(text)
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
    assert_eq!(
        fixture.nodes.get(&id).and_then(|node| node.value()),
        Some(value),
        "the accessible parent value must update in the setter frame"
    );
    wait_for(fixture, |fixture, _| {
        field(fixture, label).is_ok_and(|painted| painted == value)
    })?;
    assert_eq!(field(fixture, label)?, value);
    Ok(())
}

fn open_form(fixture: &mut Fixture) -> TestResult<egui::FullOutput> {
    fixture.warm();
    activate(fixture, "File")?;
    wait_for(fixture, |fixture, _| {
        enabled(fixture, "Convert SMILES file…")
    })?;
    activate(fixture, "Convert SMILES file…")?;
    wait_for(fixture, |fixture, output| {
        has_text(output, "Convert SMILES file")
            && enabled(fixture, "Convert")
            && field(fixture, "Input file").is_ok()
    })
}

fn shortcut(fixture: &mut Fixture, keycode: Key, shift: bool) {
    let modifiers = Modifiers {
        shift,
        ..Modifiers::COMMAND
    };
    fixture.frame(vec![
        key(keycode, true, modifiers),
        key(keycode, false, modifiers),
    ]);
}

fn assert_preserved(fixture: &Fixture, before: &Value) -> TestResult {
    let mut after = fixture.state()?;
    // Real presentation intents/cancellation have their own transient request epoch.
    after["ui_file"] = before["ui_file"].clone();
    assert_eq!(&after, before);
    Ok(())
}

fn dirty_native_drawing(fixture: &mut Fixture) -> TestResult<PathBuf> {
    fixture.command(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":380,
        "end_x_pt":300,"end_y_pt":380,"order":"single"}),
    )?;
    let path = fixture.directory.path("current-化学.cdxml");
    fixture.command("document.save_cdxml", json!({"path":path}))?;
    fixture.command("document.open_cdxml", json!({"path":path}))?;
    let revision = fixture.state()?["revision"]
        .as_u64()
        .ok_or("current revision")?;
    fixture.command(
        "bond.set_order",
        json!({"expected_revision":revision,"page_id":1,
        "fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
    )?;
    assert_eq!(fixture.state()?["file"]["dirty"], true);
    assert_eq!(fixture.state()?["source"]["modified"], true);
    assert_eq!(fixture.state()?["undo_depth"], 1);
    Ok(path)
}

fn paths(fixture: &mut Fixture, input: &Path, output: &Path) -> TestResult {
    set_field(
        fixture,
        "Input file",
        input.to_str().ok_or("UTF8 input path")?,
    )?;
    set_field(
        fixture,
        "Output file",
        output.to_str().ok_or("UTF8 output path")?,
    )
}

fn destination(path: &Path) -> TestResult<PathBuf> {
    Ok(fs::canonicalize(path.parent().ok_or("output parent")?)?
        .join(path.file_name().ok_or("output filename")?))
}

fn fill_geometry(fixture: &mut Fixture, custom: bool) -> TestResult {
    for (label, value) in if custom {
        [
            ("Bond length (pt)", "40"),
            ("Origin X (pt)", "100"),
            ("Origin Y (pt)", "200"),
            ("Page width (pt)", "612"),
            ("Page height (pt)", "792"),
        ]
    } else {
        [
            ("Bond length (pt)", "48"),
            ("Origin X (pt)", "72"),
            ("Origin Y (pt)", "72"),
            ("Page width (pt)", "595.2755905511812"),
            ("Page height (pt)", "841.8897637795276"),
        ]
    } {
        set_field(fixture, label, value)?;
    }
    Ok(())
}

fn actual_output(path: &Path, custom: bool) -> TestResult {
    if custom {
        files_support::independently_read_file(path, 1)?;
    } else {
        conversion_support::independent(path, 2)?;
    }
    Ok(())
}

fn close_result(fixture: &mut Fixture) -> TestResult {
    fixture.choose("Close conversion")?;
    wait_for(fixture, |fixture, output| {
        !has_text(output, "Convert SMILES file")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    Ok(())
}

#[test]
fn unicode_browse_conversion_preserves_dirty_source_then_explicit_open_edit_undo_save_reopens()
-> TestResult {
    for custom in [false, true] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let current = dirty_native_drawing(&mut fixture)?;
        let current_bytes = fs::read(&current)?;
        let before = fixture.state()?;
        let (owner, stamp) = {
            let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
            (engine.content().clone(), engine.capture_stamp())
        };
        let input = fixture.directory.path("input-α-化学.smi");
        let output = fixture.directory.path("converted-β-化学.cdxml");
        fs::write(&input, b"CC\n")?;
        let shown = open_form(&mut fixture)?;
        assert!(has_text(
            &shown,
            "Supports one connected chain of 1–12 neutral carbon atoms with single bonds."
        ));
        assert!(has_text(
            &shown,
            "The current document stays open. The output must be a new CDXML file; existing files are never overwritten."
        ));
        for (label, value) in [
            ("Bond length (pt)", 48.0),
            ("Origin X (pt)", 72.0),
            ("Origin Y (pt)", 72.0),
            ("Page width (pt)", 210.0 * 72.0 / 25.4),
            ("Page height (pt)", 297.0 * 72.0 / 25.4),
        ] {
            assert_eq!(field(&fixture, label)?.parse::<f64>()?, value);
        }
        fixture.choose("Browse input")?;
        wait_for(&mut fixture, |fixture, output| {
            has_text(output, "Choose SMILES input") && field(fixture, "File name").is_ok()
        })?;
        set_field(&mut fixture, "File name", "input-α-化学.smi")?;
        fixture.choose("Open")?;
        wait_for(&mut fixture, |fixture, _| {
            field(fixture, "Input file")
                .is_ok_and(|value| fs::canonicalize(value).ok() == fs::canonicalize(&input).ok())
        })?;
        assert_preserved(&fixture, &before)?;
        assert!(
            fixture
                .node("Discard and Open", &[egui::accesskit::Role::Button])
                .is_err()
        );
        fixture.choose("Browse output")?;
        wait_for(&mut fixture, |fixture, output| {
            has_text(output, "Choose new CDXML output") && field(fixture, "File name").is_ok()
        })?;
        set_field(&mut fixture, "File name", "converted-β-化学.cdxml")?;
        fixture.choose("Save")?;
        wait_for(&mut fixture, |fixture, _| {
            field(fixture, "Output file")
                .is_ok_and(|value| destination(Path::new(&value)).ok() == destination(&output).ok())
        })?;
        fill_geometry(&mut fixture, custom)?;
        assert_preserved(&fixture, &before)?;
        fixture.choose("Convert")?;
        let result = wait_for(&mut fixture, |fixture, shown| {
            has_text(shown, "Conversion complete")
                && output.exists()
                && !fixture.app.executor.has_active_work()
        })?;
        assert!(has_text(&result, "2 atoms, 1 bond"));
        assert!(has_text(
            &result,
            fs::canonicalize(&output)?.to_str().ok_or("output path")?
        ));
        assert_preserved(&fixture, &before)?;
        {
            let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
            assert_eq!(engine.content(), &owner);
            assert!(engine.capture_stamp_matches(&stamp));
        }
        assert_eq!(fs::read(&input)?, b"CC\n");
        assert_eq!(fs::read(&current)?, current_bytes);
        actual_output(&output, custom)?;
        close_result(&mut fixture)?;
        shortcut(&mut fixture, Key::Z, false);
        assert_eq!(
            fixture.state()?["file"]["dirty"],
            false,
            "conversion retained the exact saved baseline"
        );
        shortcut(&mut fixture, Key::Z, true);
        assert_eq!(fixture.state()?["document"], before["document"]);
        assert_eq!(fixture.state()?["file"]["dirty"], true);

        // Ordinary Open owns an explicit dirty-discard decision; conversion never replaces it.
        shortcut(&mut fixture, Key::O, false);
        wait_for(&mut fixture, |fixture, _| {
            field(fixture, "File name").is_ok()
        })?;
        set_field(&mut fixture, "File name", "converted-β-化学.cdxml")?;
        fixture.choose("Open")?;
        wait_for(&mut fixture, |fixture, _| {
            enabled(fixture, "Discard and Open")
        })?;
        assert_eq!(fixture.state()?["document"], before["document"]);
        fixture.choose("Discard and Open")?;
        finish_actual_file_workflow(&mut fixture)?;
        let opened = fixture.state()?;
        assert_eq!(
            opened["file"]["path"],
            fs::canonicalize(&output)?
                .to_str()
                .ok_or("canonical path")?
        );
        assert_eq!(opened["undo_depth"], 0);
        assert_eq!(opened["file"]["dirty"], false);
        assert_eq!(
            opened["document"]["pages"][0]["size"],
            if custom {
                json!({"width_pt":612.0,"height_pt":792.0})
            } else {
                json!({"width_pt":210.0*72.0/25.4,"height_pt":297.0*72.0/25.4})
            }
        );
        fixture.choose("Double bond")?;
        let shown = fixture.frame(tap(Key::F5));
        let page = if custom {
            [612.0, 792.0]
        } else {
            [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4]
        };
        let midpoint = if custom { [120.0, 200.0] } else { [96.0, 72.0] };
        let point = projected(canvas(&shown)?, page, 100.0, None, midpoint);
        fixture.frame(vec![
            Event::PointerMoved(point),
            button(point, true),
            button(point, false),
        ]);
        assert_eq!(
            fixture.state()?["document"]["pages"][0]["fragments"][0]["bonds"][0]["order"],
            "double"
        );
        assert_eq!(fixture.state()?["undo_depth"], 1);
        shortcut(&mut fixture, Key::Z, false);
        assert_eq!(fixture.state()?["document"], opened["document"]);
        assert_eq!(fixture.state()?["file"]["dirty"], false);
        shortcut(&mut fixture, Key::S, false);
        finish_actual_file_workflow(&mut fixture)?;
        actual_output(&output, custom)?;
        shortcut(&mut fixture, Key::O, false);
        wait_for(&mut fixture, |fixture, _| {
            field(fixture, "File name").is_ok()
        })?;
        set_field(&mut fixture, "File name", "converted-β-化学.cdxml")?;
        fixture.choose("Open")?;
        finish_actual_file_workflow(&mut fixture)?;
        assert_eq!(fixture.state()?["document"], opened["document"]);
        assert_eq!(fs::read(&current)?, current_bytes);
        assert_eq!(fs::read(&input)?, b"CC\n");
        assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    }
    Ok(())
}

#[test]
fn unsupported_existing_destination_and_nonfinite_geometry_refuse_then_explicit_retry_works()
-> TestResult {
    let entry = bondline_commands::registry()
        .into_iter()
        .find(|entry| entry.name == "file.convert_smiles_to_cdxml")
        .ok_or("conversion registry")?;
    assert_eq!(entry.input_schema.as_value()["additionalProperties"], false);
    assert_eq!(
        entry.input_schema.as_value()["required"]
            .as_array()
            .ok_or("required params")?
            .len(),
        8
    );
    assert_eq!(bondline_commands::registry().len(), 38);
    for case in ["unsupported", "existing", "nonfinite"] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let current = dirty_native_drawing(&mut fixture)?;
        let current_bytes = fs::read(&current)?;
        let before = fixture.state()?;
        let input = fixture.directory.path("refusal-化学.smi");
        let output = fixture.directory.path("refusal-化学.cdxml");
        let input_bytes = if case == "unsupported" {
            b"CC=O\n".as_slice()
        } else {
            b"CC\n".as_slice()
        };
        fs::write(&input, input_bytes)?;
        if case == "existing" {
            fs::write(&output, b"existing destination stays exact\n")?;
        }
        open_form(&mut fixture)?;
        paths(&mut fixture, &input, &output)?;
        if case == "nonfinite" {
            set_field(&mut fixture, "Bond length (pt)", "NaN")?;
        }
        fixture.choose("Convert")?;
        wait_for(&mut fixture, |fixture, shown| {
            has_text(shown, "Conversion failed")
                && enabled(fixture, "Convert")
                && !fixture.app.executor.has_active_work()
        })?;
        assert_preserved(&fixture, &before)?;
        assert_eq!(fs::read(&current)?, current_bytes);
        assert_eq!(fs::read(&input)?, input_bytes);
        if case == "existing" {
            assert_eq!(fs::read(&output)?, b"existing destination stays exact\n");
        } else {
            assert!(!output.exists());
        }
        assert!(
            fixture
                .node("Replace File", &[egui::accesskit::Role::Button])
                .is_err()
        );
        // Deliberate corrections for an explicit retry, never an automatic retry.
        fs::write(&input, b"CC\n")?;
        let retry = fixture.directory.path("retry-new-化学.cdxml");
        paths(&mut fixture, &input, &retry)?;
        set_field(&mut fixture, "Bond length (pt)", "48")?;
        fixture.choose("Convert")?;
        wait_for(&mut fixture, |fixture, shown| {
            has_text(shown, "Conversion complete")
                && retry.exists()
                && !fixture.app.executor.has_active_work()
        })?;
        actual_output(&retry, false)?;
        assert_preserved(&fixture, &before)?;
        close_result(&mut fixture)?;
        if case == "existing" {
            assert_eq!(fs::read(&output)?, b"existing destination stays exact\n");
        } else {
            assert!(!output.exists());
        }
    }
    Ok(())
}

fn modal_packet(fixture: &mut Fixture, label: &str) -> TestResult<egui::FullOutput> {
    let (id, _) = fixture.node(label, &[egui::accesskit::Role::Button])?;
    let a = egui::pos2(500.0, 480.0);
    let b = egui::pos2(548.0, 480.0);
    Ok(fixture.frame(vec![
        Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: id,
            data: None,
        }),
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
        key(Key::Num3, true, Modifiers::NONE),
        key(Key::Num3, false, Modifiers::NONE),
    ]))
}

#[test]
fn modal_numeric_typing_convert_cancel_and_close_fence_underlying_authoring_packets() -> TestResult
{
    for action in ["Cancel conversion", "Close conversion"] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let current = dirty_native_drawing(&mut fixture)?;
        let current_bytes = fs::read(&current)?;
        let shown = fixture.frame(tap(Key::F5));
        let midpoint = projected(
            canvas(&shown)?,
            [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
            100.0,
            None,
            [260.0, 380.0],
        );
        fixture.frame(vec![Event::PointerMoved(midpoint)]);
        assert!(
            fixture.app.drawing.hover.is_some(),
            "begin with a genuinely hovered bond"
        );
        let before = fixture.state()?;
        let tool_before = fixture.app.engine.lock().map_err(|_| "engine")?.bond_tool();
        let input = fixture.directory.path("modal-化学.smi");
        let output = fixture.directory.path("modal-new-化学.cdxml");
        fs::write(&input, b"CC\n")?;
        open_form(&mut fixture)?;
        paths(&mut fixture, &input, &output)?;
        let (id, _) = fixture.node("Bond length (pt)", &[egui::accesskit::Role::TextInput])?;
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
            key(Key::Num1, true, Modifiers::NONE),
            Event::Text("1".into()),
            key(Key::Num1, false, Modifiers::NONE),
            key(Key::Num2, true, Modifiers::NONE),
            Event::Text("2".into()),
            key(Key::Num2, false, Modifiers::NONE),
            key(Key::Num3, true, Modifiers::NONE),
            Event::Text("3".into()),
            key(Key::Num3, false, Modifiers::NONE),
        ]);
        assert_eq!(field(&fixture, "Bond length (pt)")?, "123");
        assert_preserved(&fixture, &before)?;
        assert_eq!(
            fixture.app.engine.lock().map_err(|_| "engine")?.bond_tool(),
            tool_before
        );
        assert!(!output.exists());
        if action == "Close conversion" {
            set_field(&mut fixture, "Bond length (pt)", "48")?;
            modal_packet(&mut fixture, "Convert")?;
            assert_preserved(&fixture, &before)?;
            wait_for(&mut fixture, |fixture, shown| {
                has_text(shown, "Conversion complete")
                    && output.exists()
                    && !fixture.app.executor.has_active_work()
            })?;
            actual_output(&output, false)?;
        }
        modal_packet(&mut fixture, action)?;
        assert_preserved(&fixture, &before)?;
        wait_for(&mut fixture, |fixture, shown| {
            !has_text(shown, "Convert SMILES file")
                && !fixture.app.files.is_busy()
                && !fixture.app.executor.has_active_work()
        })?;
        assert_preserved(&fixture, &before)?;
        assert_eq!(
            fixture.app.engine.lock().map_err(|_| "engine")?.bond_tool(),
            tool_before
        );
        if action == "Cancel conversion" {
            assert!(!output.exists());
        }
        assert_eq!(fs::read(&input)?, b"CC\n");
        assert_eq!(fs::read(&current)?, current_bytes);
        // The very next fresh frame may author again; the modal must not stay latched.
        let a = egui::pos2(500.0, 480.0);
        let b = egui::pos2(548.0, 480.0);
        fixture.frame(vec![
            Event::PointerMoved(a),
            button(a, true),
            Event::PointerMoved(b),
            button(b, false),
        ]);
        assert_eq!(
            fixture.state()?["document"]["pages"][0]["fragments"]
                .as_array()
                .ok_or("fresh fragments")?
                .len(),
            2
        );
        let fresh = fixture.state()?;
        let added = fresh["document"]["pages"][0]["fragments"]
            .get(1)
            .ok_or("fresh added fragment")?;
        assert_eq!(added["bonds"][0]["order"], "single");
        let atoms = added["atoms"].as_array().ok_or("fresh added atoms")?;
        assert_eq!(atoms.len(), 2);
        assert!(atoms.iter().all(|atom| atom["explicit_hydrogens"] == 3));
    }
    Ok(())
}

#[test]
fn cancel_before_submit_and_stale_source_or_close_never_execute_the_old_conversion() -> TestResult {
    for action in ["cancel", "source", "close"] {
        let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
        let current = dirty_native_drawing(&mut fixture)?;
        let current_bytes = fs::read(&current)?;
        let before = fixture.state()?;
        let input = fixture.directory.path("unstarted-化学.smi");
        let output = fixture.directory.path("unstarted-new-化学.cdxml");
        fs::write(&input, b"CC\n")?;
        open_form(&mut fixture)?;
        paths(&mut fixture, &input, &output)?;
        let (convert, _) = fixture.node("Convert", &[egui::accesskit::Role::Button])?;
        if action == "cancel" {
            fixture.choose("Cancel conversion")?;
            assert_preserved(&fixture, &before)?;
        } else {
            let revision = fixture.state()?["revision"]
                .as_u64()
                .ok_or("current revision")?;
            let replacement = {
                let mut engine = fixture.app.engine.lock().map_err(|_| "engine")?;
                if action == "source" {
                    engine.dispatch(
                        "document.new",
                        json!({"expected_revision":revision,"discard_unsaved":true}),
                    )?;
                } else {
                    engine.dispatch(
                        "document.close",
                        json!({"expected_revision":revision,"action":"request"}),
                    )?;
                }
                serde_json::to_value(engine.state())?
            };
            fixture.frame(vec![Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Default,
                    target: convert,
                    data: None,
                },
            )]);
            wait_for(&mut fixture, |fixture, shown| {
                !has_text(shown, "Convert SMILES file") && !fixture.app.executor.has_active_work()
            })?;
            for key in [
                "document",
                "source",
                "file",
                "revision",
                "undo_depth",
                "redo_depth",
            ] {
                assert_eq!(fixture.state()?[key], replacement[key]);
            }
            if action == "close" {
                assert_ne!(fixture.state()?["close"], "open");
            } else {
                assert!(fixture.state()?["source"].is_null());
                assert_eq!(fixture.state()?["undo_depth"], 0);
            }
        }
        assert!(
            !output.exists(),
            "an unstarted/stale/closed request cannot publish"
        );
        assert_eq!(fs::read(&input)?, b"CC\n");
        assert_eq!(fs::read(&current)?, current_bytes);
    }
    Ok(())
}

#[test]
fn external_cancel_keeps_old_modal_input_from_authoring_or_publishing_on_the_first_frame()
-> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    fixture.warm();
    let before = fixture.state()?;
    let (generation, stamp) = {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        (engine.session_generation(), engine.capture_stamp())
    };
    let input = fixture.directory.path("external-cancel-化学.smi");
    let output = fixture.directory.path("external-cancel-new-化学.cdxml");
    fs::write(&input, b"CC\n")?;
    open_form(&mut fixture)?;
    paths(&mut fixture, &input, &output)?;
    let shown = fixture.warm();
    assert!(has_text(&shown, "Convert SMILES file"));
    let (convert, _) = fixture.node("Convert", &[egui::accesskit::Role::Button])?;
    let pending = fixture.state()?;
    let request = pending["ui_file"]["request"]["id"]
        .as_u64()
        .ok_or("accepted conversion request")?;
    let revision = pending["revision"].as_u64().ok_or("current revision")?;
    {
        // External shared-registry cancellation happens after the modal was painted.
        let mut engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        engine.dispatch(
            "ui.file",
            json!({"expected_revision":revision,"action":"cancel","request_id":request}),
        )?;
        assert_eq!(engine.session_generation(), generation);
        assert!(engine.capture_stamp_matches(&stamp));
    }
    let a = egui::pos2(400.0, 300.0);
    let b = egui::pos2(448.0, 300.0);
    let shown = fixture.frame(vec![
        Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: convert,
            data: None,
        }),
        Event::PointerMoved(a),
        button(a, true),
        Event::PointerMoved(b),
        button(b, false),
    ]);
    assert_preserved(&fixture, &before)?;
    assert!(fixture.state()?["ui_file"]["request"].is_null());
    assert!(!output.exists());
    assert_eq!(fs::read(&input)?, b"CC\n");
    assert!(!has_text(&shown, "Convert SMILES file"));
    assert!(!fixture.app.executor.has_active_work());
    {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        assert_eq!(engine.session_generation(), generation);
        assert!(engine.capture_stamp_matches(&stamp));
    }

    // New input on the now-presented Drawing must work on the very next frame.
    let area = canvas(&shown)?;
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
    let state = fixture.state()?;
    assert_eq!(
        state["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("fresh fragments")?
            .len(),
        1
    );
    let fragment = &state["document"]["pages"][0]["fragments"][0];
    assert_eq!(fragment["bonds"][0]["order"], "single");
    assert_eq!(
        fragment["atoms"][0]["position"],
        json!({"x_pt":220.0,"y_pt":380.0})
    );
    assert_eq!(
        fragment["atoms"][1]["position"],
        json!({"x_pt":300.0,"y_pt":380.0})
    );
    assert!(
        fragment["atoms"]
            .as_array()
            .ok_or("fresh atoms")?
            .iter()
            .all(|atom| atom["explicit_hydrogens"] == 3)
    );
    assert_eq!(state["undo_depth"], 1);
    assert!(!output.exists());
    Ok(())
}

#[path = "native_conversion_recovery_tests.rs"]
mod recovery;

#[path = "native_acyclic_conversion_tests.rs"]
mod acyclic_profile;

#[path = "native_ring_conversion_tests.rs"]
mod ring_profile;

#[path = "native_components_conversion_tests.rs"]
mod components_profile;
