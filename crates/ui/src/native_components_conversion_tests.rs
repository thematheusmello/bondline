use super::{Fixture, TestResult, open_form};
use eframe::egui::accesskit::Role;

#[test]
fn native_components_profile_has_an_explicit_accessible_choice() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    open_form(&mut fixture)?;
    let (id, _) = fixture.node("Neutral components", &[Role::RadioButton])?;
    let choice = fixture.nodes.get(&id).ok_or("components profile control")?;
    assert!(!choice.is_disabled());
    assert_ne!(choice.is_selected(), Some(true));
    Ok(())
}

use super::{
    assert_preserved, button, canvas, close_result, dirty_native_drawing,
    finish_actual_file_workflow, has_text, paths, projected, set_field, shortcut, tap, wait_for,
};
use eframe::egui::{self, Event, Key, accesskit::Toggled};
use std::fs;

#[path = "../../../xtask/oracles/components_test_support.rs"]
mod component_reference;
use component_reference::consume;

const INPUT: &[u8] = b"CC.O.C1CCCCC1\r\n";

fn selected(fixture: &Fixture, label: &str) -> TestResult<bool> {
    let (id, _) = fixture.node(label, &[Role::RadioButton])?;
    Ok(fixture.nodes.get(&id).ok_or("profile")?.toggled() == Some(Toggled::True))
}

fn choose_profile(fixture: &mut Fixture, label: &str) -> TestResult {
    fixture.warm();
    let (_, bounds) = fixture.node(label, &[Role::RadioButton])?;
    fixture.frame(vec![
        Event::PointerMoved(bounds.center()),
        button(bounds.center(), true),
    ]);
    fixture.frame(vec![button(bounds.center(), false)]);
    fixture.warm();
    assert!(selected(fixture, label)?);
    Ok(())
}

#[test]
fn complete_components_convert_then_explicit_open_edit_history_and_unicode_reopen() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let active = dirty_native_drawing(&mut fixture)?;
    let original_active_bytes = fs::read(&active)?;
    let before = fixture.state()?;
    let (content, stamp) = {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        (engine.content().clone(), engine.capture_stamp())
    };
    let input = fixture.directory.path("components.smi");
    let output = fixture.directory.path("components-λ.cdxml");
    fs::write(&input, INPUT)?;
    open_form(&mut fixture)?;
    assert!(selected(&fixture, "Carbon chains")?);
    choose_profile(&mut fixture, "Neutral components")?;
    paths(&mut fixture, &input, &output)?;
    fixture.choose("Convert")?;
    let shown = wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion complete")
            && has_text(shown, "9 atoms, 7 bonds")
            && has_text(shown, "Layout: Neutral components")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    assert!(has_text(&shown, "9 atoms, 7 bonds"));
    consume(&output, false)?;
    let single = fs::read(&output)?;
    assert_preserved(&fixture, &before)?;
    {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        assert_eq!(engine.content(), &content);
        assert!(engine.capture_stamp_matches(&stamp));
    }
    assert_eq!(fs::read(&active)?, original_active_bytes);
    close_result(&mut fixture)?;
    shortcut(&mut fixture, Key::O, false);
    wait_for(&mut fixture, |fixture, _| {
        super::field(fixture, "File name").is_ok()
    })?;
    set_field(&mut fixture, "File name", output.to_str().ok_or("output")?)?;
    fixture.choose("Open")?;
    wait_for(&mut fixture, |fixture, _| {
        super::enabled(fixture, "Discard and Open")
    })?;
    fixture.choose("Discard and Open")?;
    finish_actual_file_workflow(&mut fixture)?;
    let opened = fixture.state()?;
    assert_eq!(opened["file"]["dirty"], false);
    let fragment = &opened["document"]["pages"][0]["fragments"][0];
    let atoms = fragment["atoms"].as_array().ok_or("atoms")?;
    let bonds = fragment["bonds"].as_array().ok_or("bonds")?;
    assert_eq!(atoms.len(), 9);
    assert_eq!(bonds.len(), 7);
    assert_eq!(
        atoms
            .iter()
            .map(|a| a["explicit_hydrogens"].as_u64())
            .collect::<Vec<_>>(),
        vec![
            Some(3),
            Some(3),
            Some(2),
            Some(2),
            Some(2),
            Some(2),
            Some(2),
            Some(2),
            Some(2)
        ]
    );
    fixture.choose("Double bond")?;
    let shown = fixture.frame(tap(Key::F5));
    let a = &atoms.first().ok_or("first carbon")?["position"];
    let b = &atoms.get(1).ok_or("second carbon")?["position"];
    let midpoint = [
        (a["x_pt"].as_f64().ok_or("a.x")? + b["x_pt"].as_f64().ok_or("b.x")?) / 2.0,
        (a["y_pt"].as_f64().ok_or("a.y")? + b["y_pt"].as_f64().ok_or("b.y")?) / 2.0,
    ];
    let hit = projected(
        canvas(&shown)?,
        [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
        100.0,
        None,
        midpoint,
    );
    fixture.frame(vec![Event::PointerMoved(hit), button(hit, true)]);
    fixture.frame(vec![button(hit, false)]);
    fixture.warm();
    let edited = fixture.state()?;
    assert_eq!(edited["file"]["dirty"], true);
    assert_eq!(edited["undo_depth"], 1);
    let edited_fragment = &edited["document"]["pages"][0]["fragments"][0];
    assert_eq!(edited_fragment["bonds"][0]["order"], "double");
    assert_eq!(edited_fragment["atoms"][0]["explicit_hydrogens"], 2);
    assert_eq!(edited_fragment["atoms"][1]["explicit_hydrogens"], 2);
    assert_eq!(
        &edited_fragment["atoms"].as_array().ok_or("edited atoms")?[2..],
        &atoms[2..]
    );
    assert_eq!(
        &edited_fragment["bonds"].as_array().ok_or("edited bonds")?[1..],
        &bonds[1..]
    );
    shortcut(&mut fixture, Key::Z, false);
    fixture.warm();
    assert_eq!(fixture.state()?["document"], opened["document"]);
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    shortcut(&mut fixture, Key::Z, true);
    fixture.warm();
    assert_eq!(fixture.state()?["document"], edited["document"]);
    shortcut(&mut fixture, Key::S, true);
    wait_for(&mut fixture, |fixture, _| {
        super::field(fixture, "File name").is_ok()
    })?;
    let saved = fixture.directory.path("edited-components-化学.cdxml");
    set_field(
        &mut fixture,
        "File name",
        saved.to_str().ok_or("saved path")?,
    )?;
    fixture.choose("Save")?;
    finish_actual_file_workflow(&mut fixture)?;
    consume(&saved, true)?;
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    shortcut(&mut fixture, Key::O, false);
    wait_for(&mut fixture, |fixture, _| {
        super::field(fixture, "File name").is_ok()
    })?;
    set_field(
        &mut fixture,
        "File name",
        saved.to_str().ok_or("saved path")?,
    )?;
    fixture.choose("Open")?;
    finish_actual_file_workflow(&mut fixture)?;
    assert_eq!(fixture.state()?["document"], edited["document"]);
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    assert_eq!(fs::read(&input)?, INPUT);
    assert_eq!(fs::read(&output)?, single);
    assert_eq!(fs::read(&active)?, original_active_bytes);
    Ok(())
}

#[test]
fn fourth_choice_and_convert_are_captured_in_raw_action_order() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    dirty_native_drawing(&mut fixture)?;
    let before = fixture.state()?;
    let input = fixture.directory.path("ordered-components.smi");
    let output = fixture.directory.path("ordered-components.cdxml");
    fs::write(&input, INPUT)?;
    open_form(&mut fixture)?;
    choose_profile(&mut fixture, "Acyclic neutral structures")?;
    paths(&mut fixture, &input, &output)?;
    fixture.warm();
    let action = |target| {
        Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target,
            data: None,
        })
    };
    let (convert, _) = fixture.node("Convert", &[Role::Button])?;
    let (components, _) = fixture.node("Neutral components", &[Role::RadioButton])?;
    fixture.frame(vec![action(convert), action(components)]);
    wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion failed")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    assert!(!output.try_exists()?);
    assert!(selected(&fixture, "Acyclic neutral structures")?);
    assert_preserved(&fixture, &before)?;
    fixture.warm();
    let (convert, _) = fixture.node("Convert", &[Role::Button])?;
    let (components, _) = fixture.node("Neutral components", &[Role::RadioButton])?;
    fixture.frame(vec![action(components), action(convert)]);
    wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion complete")
            && has_text(shown, "9 atoms, 7 bonds")
            && has_text(shown, "Layout: Neutral components")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    assert!(selected(&fixture, "Neutral components")?);
    consume(&output, false)?;
    assert_preserved(&fixture, &before)?;
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    assert_eq!(fs::read(&input)?, INPUT);
    close_result(&mut fixture)?;
    Ok(())
}
