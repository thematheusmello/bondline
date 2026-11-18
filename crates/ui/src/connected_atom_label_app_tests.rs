//! Adopt as a child of native_atom_label_tests.rs; reuse its actual App/SDK helpers.
use super::{
    Fixture, TestResult, canvas, click_alt, close_form, finish_actual_file_workflow, painted,
    projected, set, shortcut, value, wait,
};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton};
use serde_json::json;
use std::fs;
#[path = "../../../xtask/oracles/connected_label_support.rs"]
#[allow(dead_code, reason = "shared independent source/file vectors")]
mod connected;

fn start_connected(f: &mut Fixture, mode: &str) -> TestResult {
    match mode {
        "ring" => {
            f.command("ui.ring_tool", json!({"size":6}))?;
        }
        "chain" => {
            f.command("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))?;
        }
        "selection" => {
            f.command(
                "ui.selection",
                json!({"action":"activate","expected_revision":0}),
            )?;
        }
        "bond" => {}
        _ => return Err("test mode".into()),
    }
    let tool = f.app.engine.lock().map_err(|_| "engine")?.effective_tool();
    let input = f.directory.path("connected-app-original-λ.cdxml");
    fs::write(&input, connected::SOURCE)?;
    f.command("document.open_cdxml", json!({"path":input}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    assert_eq!(
        f.app.engine.lock().map_err(|_| "engine")?.effective_tool(),
        tool
    );
    assert_eq!(
        f.state()?["document"]["pages"][0]["fragments"][0]["atoms"][0]["authored_label"],
        connected::label(8, 0)
    );
    Ok(())
}

#[test]
fn connected_alt_targeting_preserves_retained_tools_and_marquee_only_changes_transient_targets()
-> TestResult {
    for mode in ["bond", "ring", "chain", "selection"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        start_connected(&mut f, mode)?;
        let before = f.state()?;
        let tool = f.app.engine.lock().map_err(|_| "engine")?.effective_tool();
        let shown = f.warm();
        let area = canvas(&shown)?;
        let from = projected(area, [512.0, 512.0], 100.0, None, [200.0, 220.0]);
        let to = projected(area, [512.0, 512.0], 100.0, None, [260.0, 220.0]);
        f.frame(vec![
            Event::PointerMoved(from),
            Event::PointerButton {
                pos: from,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]);
        f.frame(vec![Event::PointerMoved(to)]);
        f.frame(vec![Event::PointerButton {
            pos: to,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }]);
        f.warm();
        let mut expected = before.clone();
        if mode == "selection" {
            let epoch = before["ui_selection"]["epoch"].as_u64().ok_or("epoch")?;
            expected["ui_selection"] = json!({
                "active":true,"atom_ids":[3],"epoch":epoch+1,"page_id":1
            });
        }
        assert_eq!(
            f.state()?,
            expected,
            "{mode} changed unrelated source state"
        );
        click_alt(&mut f, connected::POINT)?;
        wait(&mut f, |f, o| {
            painted(o, "Atom label") && value(f, "Element").is_ok()
        })?;
        assert_eq!(value(&f, "Element")?, "O");
        assert_eq!(value(&f, "Attached hydrogens")?, "0");
        assert_eq!(
            f.app.engine.lock().map_err(|_| "engine")?.effective_tool(),
            tool
        );
        set(&mut f, "Element", "N")?;
        set(&mut f, "Attached hydrogens", "1")?;
        close_form(&mut f, "Apply")?;
        let changed = f.state()?;
        assert_eq!(changed["revision"], 2);
        assert_eq!(changed["undo_depth"], 1);
        assert_eq!(changed["file"]["dirty"], true);
        assert_eq!(
            changed["document"]["pages"][0]["fragments"][0]["atoms"][0]["authored_label"],
            connected::label(7, 1)
        );
        let encoded = f
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .to_cdxml()?
            .into_owned();
        assert_eq!(encoded, connected::edited().as_bytes());
        assert_eq!(connected::read(&encoded)?, connected::facts(7, 1));
        assert_eq!(
            f.app.engine.lock().map_err(|_| "engine")?.effective_tool(),
            tool
        );
        shortcut(&mut f, Key::Z, false);
        assert_eq!(f.state()?["document"], before["document"]);
        shortcut(&mut f, Key::Z, true);
        assert_eq!(f.state()?["document"], changed["document"]);
    }
    Ok(())
}

#[test]
fn connected_alt_press_uses_event_local_modifiers_before_a_later_report_and_plain_press_cannot_gain_it()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    start_connected(&mut f, "ring")?;
    let shown = f.warm();
    let point = projected(
        canvas(&shown)?,
        [512.0, 512.0],
        100.0,
        None,
        connected::POINT,
    );
    let alt = Modifiers {
        alt: true,
        ..Modifiers::NONE
    };
    let before = f.state()?;
    f.frame(vec![
        Event::PointerMoved(point),
        Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        Event::ModifiersChanged(alt),
        Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: alt,
        },
    ]);
    let shown = f.warm();
    assert!(!painted(&shown, "Atom label"));
    assert_eq!(f.state()?, before);
    f.frame(vec![
        Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: alt,
        },
        Event::ModifiersChanged(Modifiers::NONE),
        Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ]);
    wait(&mut f, |f, o| {
        painted(o, "Atom label") && value(f, "Element").is_ok()
    })?;
    close_form(&mut f, "Cancel")?;
    assert_eq!(f.state()?["document"], before["document"]);
    Ok(())
}

#[test]
fn connected_actual_atom_form_applies_command48_and_unicode_save_reopen_retains_editable_box_source()
-> TestResult {
    use egui::accesskit::Role;
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    start_connected(&mut f, "chain")?;
    click_alt(&mut f, connected::POINT)?;
    wait(&mut f, |f, o| {
        painted(o, "Atom label") && value(f, "Element").is_ok()
    })?;
    set(&mut f, "Element", "N")?;
    set(&mut f, "Attached hydrogens", "1")?;
    close_form(&mut f, "Apply")?;
    let changed = f.state()?["document"].clone();
    let output = f.directory.path("connected-native-保存-λ.cdxml");
    shortcut(&mut f, Key::S, true);
    wait(&mut f, |f, _| {
        f.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut f, "File name", output.to_str().ok_or("path")?)?;
    f.choose("Save")?;
    finish_actual_file_workflow(&mut f)?;
    assert!(!f.state()?["file"]["dirty"].as_bool().ok_or("dirty")?);
    let raw = fs::read(&output)?;
    assert_eq!(raw, connected::edited().as_bytes());
    assert_eq!(connected::read(&raw)?, connected::facts(7, 1));
    f.command("document.open_cdxml", json!({"path":output}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    assert_eq!(f.state()?["document"], changed);
    click_alt(&mut f, connected::POINT)?;
    wait(&mut f, |f, o| {
        painted(o, "Atom label") && value(f, "Element").is_ok()
    })?;
    assert_eq!(value(&f, "Element")?, "N");
    set(&mut f, "Element", "O")?;
    set(&mut f, "Attached hydrogens", "0")?;
    close_form(&mut f, "Apply")?;
    assert_eq!(
        f.state()?["document"]["pages"][0]["fragments"][0]["atoms"][0]["authored_label"],
        connected::label(8, 0)
    );
    Ok(())
}
