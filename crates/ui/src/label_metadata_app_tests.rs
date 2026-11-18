//! Unused defaults do not change the real form's explicit authoring style or file bytes.
use super::{
    Fixture, TestResult, canvas, click_alt, close_form, finish_actual_file_workflow, painted,
    projected, set, shortcut,
};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, accesskit::Role};
use serde_json::json;
use std::fs;
#[path = "../../../xtask/oracles/label_metadata_support.rs"]
#[allow(
    dead_code,
    reason = "frozen independent source, model and actual-file facts"
)]
mod f;

fn exact(app: &Fixture, before: &serde_json::Value, name: &str) -> TestResult {
    let raw = app
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .stored_document()?
        .to_cdxml()?
        .into_owned();
    assert_eq!(raw, f::source(name).as_bytes());
    assert_eq!(f::read(&raw)?, f::facts(name));
    assert_eq!(app.state()?["document"], f::model_for_case(before, name)?);
    Ok(())
}
fn wait_file(app: &mut Fixture) -> TestResult {
    super::wait(app, |a, _| a.node("File name", &[Role::TextInput]).is_ok())?;
    Ok(())
}

#[test]
fn default_metadata_file_uses_explicit_alt_edit_create_drag_history_save_and_reopen() -> TestResult
{
    let mut app = Fixture::new(1160., 800., 1.)?;
    app.command("ui.ring_tool", json!({"size":6}))?;
    let input = app.directory.path("label-defaults-原-λ.cdxml");
    fs::write(&input, f::source("mixed_a"))?;
    app.warm();
    shortcut(&mut app, Key::O, false);
    wait_file(&mut app)?;
    set(&mut app, "File name", input.to_str().ok_or("input")?)?;
    app.choose("Open")?;
    finish_actual_file_workflow(&mut app)?;
    app.command("ui.view", json!({"action":"actual"}))?;
    let before = app.state()?;
    click_alt(&mut app, [240., 160.])?;
    assert!(painted(
        &app.warm(),
        "New printed label · Roboto Regular · 12 pt · Black"
    ));
    close_form(&mut app, "Cancel")?;
    assert_eq!(app.state()?, before);
    click_alt(&mut app, [240., 160.])?;
    close_form(&mut app, "Apply")?;
    exact(&app, &before["document"], "created_3")?;
    click_alt(&mut app, [96., 160.])?;
    assert!(painted(&app.warm(), "Atom label"));
    assert!(!painted(
        &app.warm(),
        "New printed label · Roboto Regular · 12 pt · Black"
    ));
    set(&mut app, "Element", "N")?;
    set(&mut app, "Attached hydrogens", "2")?;
    close_form(&mut app, "Apply")?;
    let edited = app.state()?;
    app.choose("Marquee selection")?;
    app.warm();
    shortcut(&mut app, Key::A, false);
    assert_eq!(
        app.state()?["ui_selection"]["atom_ids"],
        json!([1, 2, 3, 4, 5])
    );
    let shown = app.warm();
    let start = projected(canvas(&shown)?, [512., 512.], 100., None, [240., 160.]);
    let end = start + egui::vec2(24., -12.);
    let pointer = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    app.frame(vec![Event::PointerMoved(start), pointer(start, true)]);
    app.frame(vec![Event::PointerMoved(end)]);
    assert_eq!(app.state()?["document"], edited["document"]);
    app.frame(vec![pointer(end, false)]);
    app.warm();
    exact(&app, &before["document"], "compound")?;
    assert_eq!(app.state()?["undo_depth"], 3);
    shortcut(&mut app, Key::Z, false);
    assert_eq!(app.state()?["document"], edited["document"]);
    shortcut(&mut app, Key::Z, true);
    exact(&app, &before["document"], "compound")?;
    let output = app.directory.path("label-defaults-保存-λ.cdxml");
    shortcut(&mut app, Key::S, true);
    wait_file(&mut app)?;
    set(&mut app, "File name", output.to_str().ok_or("output")?)?;
    app.choose("Save")?;
    finish_actual_file_workflow(&mut app)?;
    assert_eq!(fs::read(&output)?, f::source("compound").as_bytes());
    assert_eq!(f::read(&fs::read(&output)?)?, f::facts("compound"));
    shortcut(&mut app, Key::O, false);
    wait_file(&mut app)?;
    set(&mut app, "File name", output.to_str().ok_or("reopen")?)?;
    app.choose("Open")?;
    finish_actual_file_workflow(&mut app)?;
    exact(&app, &before["document"], "compound")?;
    assert_eq!(app.state()?["undo_depth"], 0);
    assert_eq!(app.state()?["file"]["dirty"], false);
    click_alt(&mut app, [264., 148.])?;
    assert!(painted(&app.warm(), "Atom label"));
    close_form(&mut app, "Cancel")?;
    assert_eq!(fs::read(input)?, f::source("mixed_a").as_bytes());
    Ok(())
}
