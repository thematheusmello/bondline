//! Hidden-node authoring uses actual App events, the shared form and real file workflow.
use super::{
    Fixture, TestResult, click_alt, close_form, finish_actual_file_workflow, painted, set, shortcut,
};
use eframe::egui::{Key, accesskit::Role};
use serde_json::json;
use std::fs;
#[path = "../../../xtask/oracles/created_label_support.rs"]
#[allow(
    dead_code,
    reason = "frozen full-model/file expectations and external consumer"
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

#[test]
fn hidden_alt_form_declares_creation_style_and_one_apply_preserves_flags_and_history() -> TestResult
{
    let mut app = Fixture::new(1160., 800., 1.)?;
    let input = app.directory.path("creation-input-原-λ.cdxml");
    fs::write(&input, f::source("base"))?;
    app.command("ui.ring_tool", json!({"size":6}))?;
    app.warm();
    shortcut(&mut app, Key::O, false);
    super::wait(&mut app, |a, _| {
        a.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut app, "File name", input.to_str().ok_or("input")?)?;
    app.choose("Open")?;
    finish_actual_file_workflow(&mut app)?;
    app.command("ui.view", json!({"action":"actual"}))?;
    let before = app.state()?;
    click_alt(&mut app, [240., 160.])?;
    assert!(painted(&app.warm(), "Atom label"));
    assert!(painted(
        &app.warm(),
        "New printed label · Roboto Regular · 12 pt · Black"
    ));
    assert_eq!(app.state()?["document"], before["document"]);
    close_form(&mut app, "Cancel")?;
    assert_eq!(app.state()?, before);
    click_alt(&mut app, [240., 160.])?;
    close_form(&mut app, "Apply")?;
    let after = app.state()?;
    exact(&app, &before["document"], "created_3_black")?;
    assert_eq!(after["undo_depth"], 1);
    assert_eq!(after["redo_depth"], 0);
    assert_eq!(
        after["revision"],
        before["revision"].as_u64().ok_or("revision")? + 1
    );
    let old = &before["document"]["pages"][0]["fragments"][0]["atoms"][2];
    let new = &after["document"]["pages"][0]["fragments"][0]["atoms"][2];
    let mut expected = old.clone();
    expected["authored_label"] = new["authored_label"].clone();
    assert_eq!(*new, expected);
    assert_eq!(new["authored_label"]["runs"][0]["text"], "CH2");
    shortcut(&mut app, Key::Z, false);
    assert_eq!(app.state()?["document"], before["document"]);
    shortcut(&mut app, Key::Z, true);
    assert_eq!(app.state()?["document"], after["document"]);
    for position in [[168., 208.], [312., 208.]] {
        click_alt(&mut app, position)?;
        assert!(painted(
            &app.warm(),
            "New printed label · Roboto Regular · 12 pt · Black"
        ));
        close_form(&mut app, "Apply")?;
    }
    exact(&app, &before["document"], "created_all_black")?;
    assert_eq!(app.state()?["undo_depth"], 3);
    let output = app.directory.path("created-all-保存-λ.cdxml");
    shortcut(&mut app, Key::S, true);
    super::wait(&mut app, |a, _| {
        a.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut app, "File name", output.to_str().ok_or("output")?)?;
    app.choose("Save")?;
    finish_actual_file_workflow(&mut app)?;
    assert_eq!(
        fs::read(&output)?,
        f::source("created_all_black").as_bytes()
    );
    assert_eq!(f::read(&fs::read(&output)?)?, f::facts("created_all_black"));
    shortcut(&mut app, Key::O, false);
    super::wait(&mut app, |a, _| {
        a.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut app, "File name", output.to_str().ok_or("reopen")?)?;
    app.choose("Open")?;
    finish_actual_file_workflow(&mut app)?;
    exact(&app, &before["document"], "created_all_black")?;
    assert_eq!(app.state()?["undo_depth"], 0);
    assert_eq!(app.state()?["redo_depth"], 0);
    assert_eq!(app.state()?["file"]["dirty"], false);
    click_alt(&mut app, [240., 160.])?;
    assert!(!painted(
        &app.warm(),
        "New printed label · Roboto Regular · 12 pt · Black"
    ));
    set(&mut app, "Element", "N")?;
    set(&mut app, "Attached hydrogens", "1")?;
    close_form(&mut app, "Apply")?;
    assert_eq!(app.state()?["undo_depth"], 1);
    shortcut(&mut app, Key::Z, false);
    exact(&app, &before["document"], "created_all_black")?;
    assert_eq!(fs::read(&input)?, f::source("base").as_bytes());
    Ok(())
}
