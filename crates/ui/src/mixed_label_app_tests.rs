//! Frozen foreign-file workflows through real App controls, existing APIs only.
use super::{
    Fixture, TestResult, canvas, click_alt, close_form, finish_actual_file_workflow, painted,
    projected, set, shortcut, value,
};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};
#[path = "../../../xtask/oracles/mixed_label_support.rs"]
#[allow(
    dead_code,
    reason = "shared frozen source/model/actual-file declarations"
)]
mod f;

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn open(app: &mut Fixture, name: &str) -> TestResult<PathBuf> {
    app.command("ui.ring_tool", json!({"size":6}))?;
    let input = app.directory.path(&format!("mixed-{name}-原-λ.cdxml"));
    fs::write(&input, f::source(name))?;
    app.warm();
    shortcut(app, Key::O, false);
    wait_file(app)?;
    set(app, "File name", input.to_str().ok_or("Unicode input")?)?;
    app.choose("Open")?;
    finish_actual_file_workflow(app)?;
    app.command("ui.view", json!({"action":"actual"}))?;
    app.warm();
    assert_eq!(app.state()?["file"]["dirty"], false);
    Ok(input)
}
fn wait_file(app: &mut Fixture) -> TestResult {
    super::wait(app, |a, _| a.node("File name", &[Role::TextInput]).is_ok())?;
    Ok(())
}
fn point(app: &mut Fixture, p: [f64; 2]) -> TestResult<Pos2> {
    let shown = app.warm();
    let view = app.view()?;
    assert_eq!(view["zoom_percent"], 100.0);
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    Ok(projected(canvas(&shown)?, [512., 512.], 100., center, p))
}
fn marquee(app: &mut Fixture) -> TestResult {
    app.warm();
    let (id, r) = app.node("Marquee selection", &[Role::Button, Role::CheckBox])?;
    assert!(
        !app.nodes.get(&id).ok_or("marquee")?.is_disabled(),
        "Mixed source must enable existing selection"
    );
    app.frame(vec![
        Event::PointerMoved(r.center()),
        pointer(r.center(), true),
    ]);
    app.frame(vec![pointer(r.center(), false)]);
    assert_eq!(app.state()?["ui_selection"]["active"], true);
    Ok(())
}
fn encoded(app: &Fixture) -> TestResult<Vec<u8>> {
    Ok(app
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .stored_document()?
        .to_cdxml()?
        .into_owned())
}
fn exact(app: &Fixture, name: &str) -> TestResult {
    let raw = encoded(app)?;
    assert_eq!(raw, f::source(name).as_bytes());
    assert_eq!(f::read(&raw)?, f::facts(name));
    Ok(())
}

#[test]
fn mixed_printed_label_edit_and_all_atom_drag_history_unicode_save_reopen_are_real_app_workflows()
-> TestResult {
    let mut app = Fixture::new(1160., 800., 1.)?;
    let input = open(&mut app, "amino_alcohol")?;
    let before = app.state()?;
    click_alt(&mut app, [96., 160.])?;
    assert!(
        painted(&app.warm(), "Atom label"),
        "Mixed printed labels must reach the existing exact-target form before retained Ring input"
    );
    assert_eq!(value(&app, "Element")?, "O");
    set(&mut app, "Element", "N")?;
    set(&mut app, "Attached hydrogens", "2")?;
    close_form(&mut app, "Apply")?;
    exact(&app, "amino_diamine")?;
    let edited = app.state()?;
    assert_eq!(
        edited["document"],
        f::model_for_case(&before["document"], "amino_diamine")?
    );
    assert_eq!(edited["revision"], 2);
    assert_eq!(edited["undo_depth"], 1);
    marquee(&mut app)?;
    shortcut(&mut app, Key::A, false);
    assert_eq!(
        app.state()?["ui_selection"]["atom_ids"],
        json!([1, 2, 3, 4, 5])
    );
    let start = point(&mut app, [240., 160.])?;
    let end = start + egui::vec2(24., -12.);
    app.frame(vec![Event::PointerMoved(start), pointer(start, true)]);
    app.frame(vec![Event::PointerMoved(end)]);
    assert_eq!(
        app.state()?["document"],
        edited["document"],
        "preview must not author"
    );
    app.frame(vec![pointer(end, false)]);
    app.warm();
    exact(&app, "edited_all_p24_m12")?;
    let moved = app.state()?;
    assert_eq!(
        moved["document"],
        f::model_for_case(&before["document"], "edited_all_p24_m12")?
    );
    assert_eq!(moved["revision"], 3);
    assert_eq!(moved["undo_depth"], 2);
    shortcut(&mut app, Key::Z, false);
    exact(&app, "amino_diamine")?;
    shortcut(&mut app, Key::Z, false);
    exact(&app, "amino_alcohol")?;
    assert_eq!(app.state()?["file"]["dirty"], false);
    shortcut(&mut app, Key::Z, true);
    shortcut(&mut app, Key::Z, true);
    exact(&app, "edited_all_p24_m12")?;
    let output = app.directory.path("mixed-final-保存-λ.cdxml");
    shortcut(&mut app, Key::S, true);
    wait_file(&mut app)?;
    set(
        &mut app,
        "File name",
        output.to_str().ok_or("Unicode output")?,
    )?;
    app.choose("Save")?;
    finish_actual_file_workflow(&mut app)?;
    assert_eq!(
        fs::read(&output)?,
        f::source("edited_all_p24_m12").as_bytes()
    );
    assert_eq!(
        f::read(&fs::read(&output)?)?,
        f::facts("edited_all_p24_m12")
    );
    shortcut(&mut app, Key::O, false);
    wait_file(&mut app)?;
    set(&mut app, "File name", output.to_str().ok_or("reopen")?)?;
    app.choose("Open")?;
    finish_actual_file_workflow(&mut app)?;
    exact(&app, "edited_all_p24_m12")?;
    let reopened = app.state()?;
    assert_eq!(reopened["document"], moved["document"]);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["redo_depth"], 0);
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(fs::read(input)?, f::source("amino_alcohol").as_bytes());
    Ok(())
}

#[test]
fn mixed_hidden_creation_draft_cancel_and_neighbor_c2_apply_keep_font_guards() -> TestResult {
    let mut app = Fixture::new(1160., 800., 1.)?;
    open(&mut app, "methanol")?;
    marquee(&mut app)?;
    let before = app.state()?;
    click_alt(&mut app, [128., 192.])?;
    assert!(
        painted(&app.warm(), "Atom label"),
        "Command49 now permits an explicit hidden-carbon creation draft"
    );
    assert_eq!(app.state()?["document"], before["document"]);
    close_form(&mut app, "Cancel")?;
    assert_eq!(app.state()?, before);
    exact(&app, "methanol")?;
    click_alt(&mut app, [208., 144.])?;
    assert!(painted(&app.warm(), "Atom label"));
    set(&mut app, "Element", "C")?;
    set(&mut app, "Attached hydrogens", "3")?;
    let opened: Value = app.state()?;
    app.choose("Apply")?;
    assert!(
        painted(&app.warm(), "Atom label"),
        "unproved mandatory C2 ink must not close and commit"
    );
    assert_eq!(app.state()?, opened);
    exact(&app, "methanol")?;
    close_form(&mut app, "Cancel")?;
    assert_eq!(app.state()?, before);
    Ok(())
}
