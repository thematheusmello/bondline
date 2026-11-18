//! Native SDK events preserve directed bond metadata through existing file controls.
use super::{Fixture, TestResult, finish_actual_file_workflow, painted, set, shortcut};
use eframe::egui::{Key, accesskit::Role};
use serde_json::json;
use std::fs;
#[path = "../../../xtask/oracles/cdx_bond_support.rs"]
#[allow(
    dead_code,
    reason = "independent frozen complete-model/file expectations"
)]
mod f;

fn wait_file(app: &mut Fixture) -> TestResult {
    super::wait(app, |a, _| a.node("File name", &[Role::TextInput]).is_ok())?;
    Ok(())
}

#[test]
fn directed_double_cdx_open_known_save_unicode_save_as_and_reopen_use_actual_controls() -> TestResult
{
    let mut app = Fixture::new(1160., 800., 1.)?;
    let input = app.directory.path("directed-原-λ.cdx");
    let output = app.directory.path("directed-保存-λ.cdx");
    fs::write(&input, f::source("manual_right"))?;
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
    assert_eq!(opened["document"], f::model("manual_right")?);
    assert_eq!(opened["source"]["modified"], false);
    {
        let engine = app.app.engine.lock().map_err(|_| "engine")?;
        assert!(!engine.stored_document()?.is_fully_mapped());
        assert!(!engine.stored_document()?.has_display_projection());
    }
    let save_epoch = opened["ui_file"]["epoch"].as_u64().ok_or("file epoch")?;
    shortcut(&mut app, Key::S, false);
    finish_actual_file_workflow(&mut app)?;
    assert!(
        app.state()?["ui_file"]["epoch"]
            .as_u64()
            .ok_or("save epoch")?
            > save_epoch
    );
    assert!(app.app.error.is_none(), "known Save completed successfully");
    assert_eq!(fs::read(&input)?, f::source("manual_right"));
    shortcut(&mut app, Key::S, true);
    wait_file(&mut app)?;
    assert!(painted(&app.warm(), "CDX (*.cdx)"));
    set(&mut app, "File name", output.to_str().ok_or("output")?)?;
    app.choose("Save")?;
    finish_actual_file_workflow(&mut app)?;
    let actual = fs::read(&output)?;
    assert_eq!(actual, f::source("manual_right"));
    assert_eq!(f::read(&actual)?, f::facts("manual_right"));
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
    assert_eq!(
        (
            reopened["undo_depth"].as_u64(),
            reopened["redo_depth"].as_u64()
        ),
        (Some(0), Some(0))
    );
    assert_eq!(fs::read(input)?, f::source("manual_right"));
    Ok(())
}
