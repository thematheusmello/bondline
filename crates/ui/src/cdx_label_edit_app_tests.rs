//! Actual App frames, native shortcuts, form widgets and isolated file jobs.
use super::{
    Fixture, TestResult, canvas, click_alt, close_form, finish_actual_file_workflow, painted,
    projected, set, shortcut, state_without_form, value, wait,
};
use eframe::egui::{Event, Key, Modifiers, PointerButton, accesskit::Role};
use serde_json::{Value, json};
use std::fs;

const RAW: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-label-preview/water.cdx");
const EDITED: &[u8] = include_bytes!("../../io/tests/fixtures/cdx-label-edit/water_to_nh.cdx");
const WITNESSES: &str = include_str!("../../io/tests/fixtures/cdx-label-edit/cases.json");
const ANCHOR: [f64; 2] = [256.0, 218.25];

#[test]
fn cdx_option_click_uses_the_actual_form_history_and_unicode_file_controls() -> TestResult {
    let witnesses: Value = serde_json::from_str(WITNESSES)?;
    let case = &witnesses["cases"]["water_to_nh"];
    let mut app = Fixture::new(1160.0, 800.0, 2.0)?;
    let input = app.directory.path("water-原-λ.cdx");
    let output = app.directory.path("water-保存-β.cdx");
    fs::write(&input, RAW)?;
    app.command("ui.ring_tool", json!({"size":8}))?;
    app.warm();
    shortcut(&mut app, Key::O, false);
    wait(&mut app, |app, _| {
        app.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut app, "File name", input.to_str().ok_or("input path")?)?;
    app.choose("Open")?;
    finish_actual_file_workflow(&mut app)?;
    app.command("ui.view", json!({"action":"actual"}))?;
    let shown = app.warm();
    let opened = state_without_form(&app)?;
    assert_eq!(opened["document"], case["before_model"]);
    assert_eq!(opened["file"]["dirty"], false);

    let paper = canvas(&shown)?;
    let pos = projected(paper, [512.0, 512.0], 100.0, None, ANCHOR);
    assert!(paper.contains(pos));
    for modifiers in [
        Modifiers::NONE,
        Modifiers {
            alt: true,
            shift: true,
            ..Modifiers::NONE
        },
    ] {
        app.frame(vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers,
            },
        ]);
        app.frame(vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers,
        }]);
        app.warm();
        assert!(app.state()?["ui_atom_label"].is_null());
        assert_eq!(state_without_form(&app)?, opened);
    }

    for action in ["Cancel", "Apply"] {
        click_alt(&mut app, ANCHOR)?;
        wait(&mut app, |app, shown| {
            painted(shown, "Atom label") && value(app, "Element").is_ok()
        })?;
        assert_eq!(value(&app, "Element")?, "O");
        assert_eq!(value(&app, "Attached hydrogens")?, "0");
        assert_eq!(state_without_form(&app)?, opened);
        set(&mut app, "Element", "N")?;
        set(&mut app, "Attached hydrogens", "1")?;
        close_form(&mut app, action)?;
    }
    let changed = app.state()?;
    assert_eq!(changed["document"], case["after_model"]);
    assert_eq!(changed["undo_depth"], 1);
    assert_eq!(changed["file"]["dirty"], true);
    assert_eq!(fs::read(&input)?, RAW);

    shortcut(&mut app, Key::Z, false);
    let undone = app.state()?;
    assert_eq!(undone["document"], case["before_model"]);
    assert_eq!(undone["file"]["dirty"], false);
    shortcut(&mut app, Key::Z, true);
    let redone = app.state()?;
    assert_eq!(redone["document"], case["after_model"]);
    assert_eq!(redone["file"]["dirty"], true);

    shortcut(&mut app, Key::S, false);
    finish_actual_file_workflow(&mut app)?;
    assert_eq!(fs::read(&input)?, EDITED);
    assert_eq!(app.state()?["file"]["dirty"], false);
    shortcut(&mut app, Key::S, true);
    wait(&mut app, |app, _| {
        app.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut app, "File name", output.to_str().ok_or("output path")?)?;
    app.choose("Save")?;
    finish_actual_file_workflow(&mut app)?;
    assert_eq!(fs::read(&output)?, EDITED);

    shortcut(&mut app, Key::O, false);
    wait(&mut app, |app, _| {
        app.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut app, "File name", output.to_str().ok_or("reopen path")?)?;
    app.choose("Open")?;
    finish_actual_file_workflow(&mut app)?;
    let reopened = app.state()?;
    assert_eq!(reopened["document"], case["after_model"]);
    assert_eq!(
        reopened["source"],
        json!({"format":"cdx","byte_length":481,"modified":false})
    );
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    Ok(())
}
