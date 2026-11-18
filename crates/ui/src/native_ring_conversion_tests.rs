use super::{Fixture, TestResult, open_form};
use eframe::egui::accesskit::Role;

#[test]
fn native_simple_ring_profile_has_an_explicit_accessible_choice() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    open_form(&mut fixture)?;
    let (id, _) = fixture.node("Neutral ring structures", &[Role::RadioButton])?;
    let choice = fixture.nodes.get(&id).ok_or("ring profile control")?;
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

fn selected(fixture: &Fixture, label: &str) -> TestResult<bool> {
    let (id, _) = fixture.node(label, &[Role::RadioButton])?;
    Ok(fixture.nodes.get(&id).ok_or("profile")?.toggled() == Some(Toggled::True))
}

fn choose(fixture: &mut Fixture, label: &str) -> TestResult {
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

fn consume(path: &Path, edited: bool) -> TestResult {
    let (notation, atoms, formula, order) = if edited {
        (
            "C=C1CCCCC1",
            json!([[6, 2], [6, 0], [6, 2], [6, 2], [6, 2], [6, 2], [6, 2]]),
            "C7H12",
            2,
        )
    } else {
        (
            "CC1CCCCC1",
            json!([[6, 3], [6, 1], [6, 2], [6, 2], [6, 2], [6, 2], [6, 2]]),
            "C7H14",
            1,
        )
    };
    let output = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../io/tests/smiles_ring_oracle.py"
        ))
        .arg(notation)
        .arg(
            json!({"atoms":atoms,"formula":formula,
            "bonds":[[1,2,order],[2,3,1],[3,4,1],[4,5,1],[5,6,1],[6,7,1],[2,7,1]]})
            .to_string(),
        )
        .stdin(fs::File::open(path)?)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["formula"], formula);
    assert_eq!(value["atoms"], 7);
    assert_eq!(value["bonds"], 7);
    assert_eq!(value["no_assigned_stereo"], true);
    Ok(())
}

#[test]
fn ring_profile_converts_without_replacement_then_explicit_open_edits_and_saves() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let active = dirty_native_drawing(&mut fixture)?;
    let active_bytes = fs::read(&active)?;
    let before = fixture.state()?;
    let (content, stamp) = {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        (engine.content().clone(), engine.capture_stamp())
    };
    let input = fixture.directory.path("ring.smi");
    let output = fixture.directory.path("methyl-λ.cdxml");
    fs::write(&input, b"CC1CCCCC1\n")?;
    open_form(&mut fixture)?;
    assert!(selected(&fixture, "Carbon chains")?);
    choose(&mut fixture, "Neutral ring structures")?;
    set_field(&mut fixture, "Input file", input.to_str().ok_or("input")?)?;
    set_field(
        &mut fixture,
        "Output file",
        output.to_str().ok_or("output")?,
    )?;
    fixture.choose("Convert")?;
    let result = wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion complete")
            && has_text(shown, "7 atoms, 7 bonds")
            && has_text(shown, "Layout: Neutral ring structures")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    assert!(has_text(&result, "7 atoms, 7 bonds"));
    assert!(has_text(&result, "Layout: Neutral ring structures"));
    consume(&output, false)?;
    let single = fs::read(&output)?;
    assert_preserved(&fixture, &before)?;
    {
        let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
        assert_eq!(engine.content(), &content);
        assert!(engine.capture_stamp_matches(&stamp));
    }
    close_result(&mut fixture)?;
    shortcut(&mut fixture, Key::O, false);
    wait_for(&mut fixture, |fixture, _| {
        super::field(fixture, "File name").is_ok()
    })?;
    set_field(&mut fixture, "File name", "methyl-λ.cdxml")?;
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
        .ok_or("atoms")?;
    assert_eq!(atoms.len(), 7);
    assert_eq!(
        atoms
            .iter()
            .map(|atom| atom["explicit_hydrogens"].as_u64())
            .collect::<Vec<_>>(),
        vec![
            Some(3),
            Some(1),
            Some(2),
            Some(2),
            Some(2),
            Some(2),
            Some(2)
        ]
    );
    fixture.choose("Double bond")?;
    let shown = fixture.frame(tap(Key::F5));
    let a = &atoms.first().ok_or("methyl")?["position"];
    let b = &atoms.get(1).ok_or("ring carbon")?["position"];
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
    let double = fixture.state()?["document"].clone();
    shortcut(&mut fixture, Key::Z, false);
    assert_eq!(fixture.state()?["document"], opened["document"]);
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    shortcut(&mut fixture, Key::Z, true);
    assert_eq!(fixture.state()?["document"], double);
    assert_eq!(fixture.state()?["file"]["dirty"], true);
    shortcut(&mut fixture, Key::S, true);
    wait_for(&mut fixture, |fixture, _| {
        super::field(fixture, "File name").is_ok()
    })?;
    set_field(&mut fixture, "File name", "edited-ring-λ.cdxml")?;
    fixture.choose("Save")?;
    finish_actual_file_workflow(&mut fixture)?;
    let saved = fixture.directory.path("edited-ring-λ.cdxml");
    consume(&saved, true)?;
    assert_eq!(fixture.state()?["file"]["dirty"], false);
    shortcut(&mut fixture, Key::O, false);
    wait_for(&mut fixture, |fixture, _| {
        super::field(fixture, "File name").is_ok()
    })?;
    set_field(&mut fixture, "File name", "edited-ring-λ.cdxml")?;
    fixture.choose("Open")?;
    finish_actual_file_workflow(&mut fixture)?;
    assert_eq!(fixture.state()?["document"], double);
    assert_eq!(fs::read(&output)?, single);
    assert_eq!(fs::read(&input)?, b"CC1CCCCC1\n");
    assert_eq!(fs::read(&active)?, active_bytes);
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    Ok(())
}

#[test]
fn a_later_ring_choice_cannot_relabel_an_earlier_acyclic_convert() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    dirty_native_drawing(&mut fixture)?;
    let before = fixture.state()?;
    let input = fixture.directory.path("ordered-ring.smi");
    let output = fixture.directory.path("ordered-ring.cdxml");
    fs::write(&input, b"CC1CCCCC1\n")?;
    open_form(&mut fixture)?;
    choose(&mut fixture, "Acyclic neutral structures")?;
    set_field(&mut fixture, "Input file", input.to_str().ok_or("input")?)?;
    set_field(
        &mut fixture,
        "Output file",
        output.to_str().ok_or("output")?,
    )?;
    fixture.warm();
    let (convert, _) = fixture.node("Convert", &[Role::Button])?;
    let (ring, _) = fixture.node("Neutral ring structures", &[Role::RadioButton])?;
    let action = |target| {
        Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target,
            data: None,
        })
    };
    fixture.frame(vec![action(convert), action(ring)]);
    wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion failed")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    assert!(!output.try_exists()?);
    assert!(selected(&fixture, "Acyclic neutral structures")?);
    assert_preserved(&fixture, &before)?;
    choose(&mut fixture, "Neutral ring structures")?;
    fixture.choose("Convert")?;
    wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion complete")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    consume(&output, false)?;
    assert_preserved(&fixture, &before)?;
    assert_eq!(fs::read(&input)?, b"CC1CCCCC1\n");
    assert!(fixture.app.error.is_none(), "{:?}", fixture.app.error);
    close_result(&mut fixture)?;
    Ok(())
}
