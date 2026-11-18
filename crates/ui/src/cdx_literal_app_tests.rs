//! Actual App file controls preserve typed CDX metadata without a chemical canvas grant.
use super::{Fixture, TestResult, finish_actual_file_workflow, painted, set, shortcut};
use eframe::egui::{Key, accesskit::Role};
use serde_json::json;
use std::fs;
#[path = "../../../xtask/oracles/cdx_literal_support.rs"]
#[allow(
    dead_code,
    reason = "frozen independent complete-model and actual-file expectations"
)]
mod f;

fn wait_file(app: &mut Fixture) -> TestResult {
    super::wait(app, |a, _| a.node("File name", &[Role::TextInput]).is_ok())?;
    Ok(())
}

#[test]
fn literal_cdx_open_known_save_unicode_save_as_and_reopen_use_the_actual_app() -> TestResult {
    let mut app = Fixture::new(1160., 800., 1.)?;
    let input = app.directory.path("literal-原-λ.cdx");
    let output = app.directory.path("literal-保存-λ.cdx");
    fs::write(&input, f::source("methylammonium_explicit"))?;
    app.command("ui.ring_tool", json!({"size":8}))?;
    app.warm();
    shortcut(&mut app, Key::O, false);
    wait_file(&mut app)?;
    set(&mut app, "File name", input.to_str().ok_or("input")?)?;
    app.choose("Open")?;
    if let Err(error) = finish_actual_file_workflow(&mut app) {
        return Err(format!("{error}; actual App error: {:?}", app.app.error).into());
    }
    let opened = app.state()?;
    assert_eq!(opened["document"], f::model("methylammonium_explicit")?);
    assert_eq!(opened["file"]["format"], "cdx");
    assert_eq!(opened["source"]["modified"], false);
    assert_eq!(opened["undo_depth"], 0);
    {
        let engine = app.app.engine.lock().map_err(|_| "engine")?;
        assert!(!engine.stored_document()?.is_fully_mapped());
        assert!(!engine.stored_document()?.has_display_projection());
    }
    shortcut(&mut app, Key::S, false);
    finish_actual_file_workflow(&mut app)?;
    assert_eq!(fs::read(&input)?, f::source("methylammonium_explicit"));
    assert_eq!(app.state()?["document"], opened["document"]);
    shortcut(&mut app, Key::S, true);
    wait_file(&mut app)?;
    assert!(painted(&app.warm(), "CDX (*.cdx)"));
    set(&mut app, "File name", output.to_str().ok_or("output")?)?;
    app.choose("Save")?;
    finish_actual_file_workflow(&mut app)?;
    assert_eq!(fs::read(&output)?, f::source("methylammonium_explicit"));
    assert_eq!(
        f::read(&fs::read(&output)?)?,
        f::facts("methylammonium_explicit")
    );
    shortcut(&mut app, Key::O, false);
    wait_file(&mut app)?;
    set(&mut app, "File name", output.to_str().ok_or("reopen")?)?;
    app.choose("Open")?;
    finish_actual_file_workflow(&mut app)?;
    let reopened = app.state()?;
    assert_eq!(reopened["document"], opened["document"]);
    assert_eq!(reopened["file"]["format"], "cdx");
    assert_eq!(reopened["file"]["dirty"], false);
    assert_eq!(reopened["source"]["modified"], false);
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["redo_depth"], 0);
    assert_eq!(fs::read(input)?, f::source("methylammonium_explicit"));
    Ok(())
}
