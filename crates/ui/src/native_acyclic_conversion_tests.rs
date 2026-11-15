use super::{Fixture, TestResult, open_form};
use eframe::egui::accesskit::Role;

#[test]
fn native_acyclic_profile_has_an_explicit_accessible_choice() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    open_form(&mut fixture)?;
    let (id, _) = fixture.node("Acyclic neutral structures", &[Role::RadioButton])?;
    let choice = fixture.nodes.get(&id).ok_or("acyclic profile control")?;
    assert!(!choice.is_disabled());
    assert_ne!(choice.is_selected(), Some(true));
    Ok(())
}

use super::{
    assert_preserved, button, canvas, close_result, dirty_native_drawing,
    finish_actual_file_workflow, has_text, projected, set_field, shortcut, tap, wait_for,
};
use eframe::egui::{self, Event, Key, accesskit::Toggled};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn profile_selected(fixture: &Fixture, caption: &str) -> TestResult<bool> {
    let (id, _) = fixture.node(caption, &[Role::RadioButton])?;
    Ok(fixture.nodes.get(&id).ok_or("profile node")?.toggled() == Some(Toggled::True))
}

fn choose_profile(fixture: &mut Fixture, caption: &str) -> TestResult {
    fixture.warm();
    let (_, bounds) = fixture.node(caption, &[Role::RadioButton])?;
    fixture.frame(vec![
        Event::PointerMoved(bounds.center()),
        button(bounds.center(), true),
    ]);
    fixture.frame(vec![button(bounds.center(), false)]);
    fixture.warm();
    assert!(profile_selected(fixture, caption)?);
    Ok(())
}

fn independent_output(path: &Path) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_acyclic_oracle.py"
        ))
        .arg("CC(C)O")
        .arg(
            json!({"atoms":[[6,3],[6,1],[6,3],[8,1]],
            "bonds":[[1,2,1],[2,3,1],[2,4,1]],"formula":"C3H8O"})
            .to_string(),
        )
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)?,
        json!({"atoms":4,"bonds":3,"formula":"C3H8O","stereo_none":true,"no_assigned_stereo":true,"bond_stereo":["STEREONONE","STEREONONE","STEREONONE"],"physical_h":0})
    );
    Ok(())
}

#[test]
fn acyclic_choice_publishes_a_branch_then_actual_open_edit_undo_save_reopens() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let current = dirty_native_drawing(&mut fixture)?;
    let current_bytes = fs::read(&current)?;
    let before = fixture.state()?;
    let (owner, stamp) = {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        (engine.content().clone(), engine.capture_stamp())
    };
    let input = fixture.directory.path("branched-λ.smi");
    let output = fixture.directory.path("branched-λ.cdxml");
    fs::write(&input, b"CC(C)O\n")?;
    open_form(&mut fixture)?;
    assert!(profile_selected(&fixture, "Carbon chains")?);
    choose_profile(&mut fixture, "Acyclic neutral structures")?;
    choose_profile(&mut fixture, "Carbon chains")?;
    choose_profile(&mut fixture, "Acyclic neutral structures")?;
    set_field(
        &mut fixture,
        "Input file",
        input.to_str().ok_or("input path")?,
    )?;
    set_field(
        &mut fixture,
        "Output file",
        output.to_str().ok_or("output path")?,
    )?;
    fixture.choose("Convert")?;
    let result = wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion complete")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    assert!(has_text(&result, "4 atoms, 3 bonds"));
    independent_output(&output)?;
    let converted = fs::read(&output)?;
    assert_preserved(&fixture, &before)?;
    {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        assert!(engine.content() == &owner);
        assert!(engine.capture_stamp_matches(&stamp));
    }
    assert_eq!(fs::read(&input)?, b"CC(C)O\n");
    assert_eq!(fs::read(&current)?, current_bytes);
    close_result(&mut fixture)?;

    shortcut(&mut fixture, Key::O, false);
    wait_for(&mut fixture, |fixture, _| {
        super::field(fixture, "File name").is_ok()
    })?;
    set_field(&mut fixture, "File name", "branched-λ.cdxml")?;
    fixture.choose("Open")?;
    wait_for(&mut fixture, |fixture, _| {
        super::enabled(fixture, "Discard and Open")
    })?;
    fixture.choose("Discard and Open")?;
    finish_actual_file_workflow(&mut fixture)?;
    let opened = fixture.state()?;
    assert_eq!(opened["file"]["dirty"], false);
    let atoms = opened["document"]["pages"][0]["fragments"][0]["atoms"]
        .as_array()
        .ok_or("opened atoms")?;
    assert_eq!(atoms.len(), 4);
    assert_eq!(
        atoms
            .iter()
            .map(|atom| atom["explicit_hydrogens"].as_u64())
            .collect::<Vec<_>>(),
        vec![Some(3), Some(1), Some(3), Some(1)]
    );
    fixture.choose("Double bond")?;
    let shown = fixture.frame(tap(Key::F5));
    let a = &atoms.first().ok_or("first carbon")?["position"];
    let b = &atoms.get(1).ok_or("branch carbon")?["position"];
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
    fixture.frame(vec![
        Event::PointerMoved(hit),
        button(hit, true),
        button(hit, false),
    ]);
    assert_eq!(
        fixture.state()?["document"]["pages"][0]["fragments"][0]["bonds"][0]["order"],
        "double"
    );
    assert_eq!(fixture.state()?["file"]["dirty"], true);
    shortcut(&mut fixture, Key::Z, false);
    assert_eq!(fixture.state()?["document"], opened["document"]);
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    shortcut(&mut fixture, Key::S, false);
    finish_actual_file_workflow(&mut fixture)?;
    assert_eq!(fs::read(&output)?, converted);
    independent_output(&output)?;
    shortcut(&mut fixture, Key::O, false);
    wait_for(&mut fixture, |fixture, _| {
        super::field(fixture, "File name").is_ok()
    })?;
    set_field(&mut fixture, "File name", "branched-λ.cdxml")?;
    fixture.choose("Open")?;
    finish_actual_file_workflow(&mut fixture)?;
    assert_eq!(fixture.state()?["document"], opened["document"]);
    assert_eq!(fs::read(&input)?, b"CC(C)O\n");
    assert_eq!(fs::read(&current)?, current_bytes);
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}

#[test]
fn a_later_profile_action_cannot_relabel_an_already_submitted_conversion() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    dirty_native_drawing(&mut fixture)?;
    let before = fixture.state()?;
    let input = fixture.directory.path("ordered.smi");
    let output = fixture.directory.path("ordered.cdxml");
    fs::write(&input, b"CC(C)O\n")?;
    open_form(&mut fixture)?;
    set_field(&mut fixture, "Input file", input.to_str().ok_or("input")?)?;
    set_field(
        &mut fixture,
        "Output file",
        output.to_str().ok_or("output")?,
    )?;
    fixture.warm();
    let (convert, _) = fixture.node("Convert", &[Role::Button])?;
    let (profile, _) = fixture.node("Acyclic neutral structures", &[Role::RadioButton])?;
    let action = |target| {
        Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target,
            data: None,
        })
    };
    fixture.frame(vec![action(convert), action(profile)]);
    wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion failed")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    assert!(
        !output.exists(),
        "the already-submitted default profile must still refuse the branch"
    );
    assert!(profile_selected(&fixture, "Carbon chains")?);
    assert_preserved(&fixture, &before)?;
    choose_profile(&mut fixture, "Acyclic neutral structures")?;
    fixture.choose("Convert")?;
    wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion complete")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    independent_output(&output)?;
    assert_preserved(&fixture, &before)?;
    assert_eq!(fs::read(&input)?, b"CC(C)O\n");
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    close_result(&mut fixture)?;
    Ok(())
}
