use super::{Fixture, TestResult, canvas, projected};
use eframe::egui;
use serde_json::json;

#[test]
fn fresh_alt_click_on_the_existing_atom_opens_the_atom_label_form() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    fixture.command(
        "bond.create",
        json!({"page_id":1,
        "begin_x_pt":220,"begin_y_pt":380,"end_x_pt":300,"end_y_pt":380}),
    )?;
    fixture.command("ui.view", json!({"action":"actual"}))?;
    let shown = fixture.warm();
    let point = projected(
        canvas(&shown)?,
        [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
        100.0,
        None,
        [300.0, 380.0],
    );
    let before = fixture.state()?;
    let modifiers = egui::Modifiers {
        alt: true,
        ..egui::Modifiers::NONE
    };
    fixture.frame(vec![
        egui::Event::PointerMoved(point),
        egui::Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers,
        },
    ]);
    fixture.frame(vec![egui::Event::PointerButton {
        pos: point,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers,
    }]);
    let opened = fixture.warm();
    assert!(
        opened.shapes.iter().any(|paint| matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text()=="Atom label"
            && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))),
        "fresh Alt-click on atom2 must open the actual Atom label form"
    );
    fixture.node("Element", &[egui::accesskit::Role::TextInput])?;
    fixture.node("Attached hydrogens", &[egui::accesskit::Role::TextInput])?;
    let after = fixture.state()?;
    for key in [
        "document",
        "revision",
        "undo_depth",
        "redo_depth",
        "file",
        "source",
    ] {
        assert_eq!(after[key], before[key], "opening the form changed {key}");
    }
    Ok(())
}

// Additive cases preserve the first1,698-byte actual Alt-click declaration.
use super::{button, finish_actual_file_workflow, independently_read_actual_saved_graph, key};
use egui::{Event, Key, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::{Command as Process, Stdio},
    time::{Duration, Instant},
};

fn painted(output: &egui::FullOutput, text: &str) -> bool {
    output.shapes.iter().any(|paint| {
        matches!(&paint.shape,
        egui::Shape::Text(shape) if shape.galley.text()==text
            && paint.clip_rect.contains_rect(paint.shape.visual_bounding_rect()))
    })
}

fn wait(
    f: &mut Fixture,
    ready: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = f.frame(vec![]);
        if ready(f, &output) {
            return Ok(output);
        }
        if Instant::now() >= deadline {
            return Err("atom-label App state did not become ready".into());
        }
        std::thread::yield_now();
    }
}

fn value(f: &Fixture, name: &str) -> TestResult<String> {
    let (id, _) = f.node(name, &[Role::TextInput])?;
    let mut stack = vec![id];
    let mut text = String::new();
    while let Some(id) = stack.pop() {
        let node = f.nodes.get(&id).ok_or("text node")?;
        if node.role() == Role::InlineTextBox {
            text.push_str(node.value().ok_or("inline text")?);
        }
        stack.extend(node.children().iter().rev().copied());
    }
    Ok(text)
}

fn apply_enabled(f: &Fixture) -> TestResult<bool> {
    let mut seen = HashSet::new();
    let mut stack = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = f.nodes.get(&id)
        {
            if node.role() == Role::Button && node.name() == Some("Apply") {
                return Ok(!node.is_disabled());
            }
            stack.extend_from_slice(node.children());
        }
    }
    Err("reachable Apply button missing".into())
}

fn set(f: &mut Fixture, name: &str, text: &str) -> TestResult {
    let (id, _) = f.node(name, &[Role::TextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::SetValue,
            target: id,
            data: Some(egui::accesskit::ActionData::Value(text.into())),
        },
    )]);
    assert_eq!(f.nodes.get(&id).and_then(|node| node.value()), Some(text));
    wait(f, |f, _| {
        value(f, name).is_ok_and(|painted| painted == text)
    })?;
    assert_eq!(value(f, name)?, text);
    Ok(())
}

fn focus(f: &mut Fixture, name: &str) -> TestResult {
    let (id, _) = f.node(name, &[Role::TextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    f.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
    ]);
    Ok(())
}

fn select_and_reject_paste(f: &mut Fixture, name: &str, text: &str, old: &str) -> TestResult {
    focus(f, name)?;
    f.frame(vec![Event::Paste(text.into())]);
    let (id, _) = f.node(name, &[Role::TextInput])?;
    assert_eq!(f.nodes.get(&id).and_then(|node| node.value()), Some(old));
    wait(f, |f, _| value(f, name).is_ok_and(|value| value == old))?;
    assert_eq!(
        value(f, name)?,
        old,
        "oversize selected paste must reject the entire replacement"
    );
    Ok(())
}

fn state_without_form(f: &Fixture) -> TestResult<Value> {
    let mut state = f.state()?;
    state
        .as_object_mut()
        .ok_or("state")?
        .remove("ui_atom_label");
    Ok(state)
}

fn shortcut(f: &mut Fixture, code: Key, shift: bool) -> egui::FullOutput {
    let modifiers = Modifiers {
        shift,
        ..Modifiers::COMMAND
    };
    f.frame(vec![
        key(code, true, modifiers),
        key(code, false, modifiers),
    ])
}

fn seed(f: &mut Fixture) -> TestResult<PathBuf> {
    f.command(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":380,
        "end_x_pt":300,"end_y_pt":380}),
    )?;
    let input = f.directory.path("original-carbon-原.cdxml");
    f.command("document.save_cdxml", json!({"path":input}))?;
    f.command("document.open_cdxml", json!({"path":input}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    Ok(input)
}

fn click_alt(f: &mut Fixture, point: [f64; 2]) -> TestResult {
    let shown = f.warm();
    let state = f.state()?;
    let size = &state["document"]["pages"][0]["size"];
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    let point = projected(
        canvas(&shown)?,
        [
            size["width_pt"].as_f64().ok_or("width")?,
            size["height_pt"].as_f64().ok_or("height")?,
        ],
        100.0,
        None,
        point,
    );
    let modifiers = Modifiers {
        alt: true,
        ..Modifiers::NONE
    };
    f.frame(vec![
        Event::PointerMoved(point),
        Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed: true,
            modifiers,
        },
    ]);
    f.frame(vec![Event::PointerButton {
        pos: point,
        button: PointerButton::Primary,
        pressed: false,
        modifiers,
    }]);
    Ok(())
}

fn open_atom(f: &mut Fixture) -> TestResult {
    click_alt(f, [300.0, 380.0])?;
    wait(f, |f, o| {
        painted(o, "Atom label") && value(f, "Element").is_ok()
    })?;
    f.node("Cancel", &[Role::Button])?;
    Ok(())
}

fn close_form(f: &mut Fixture, action: &str) -> TestResult {
    f.choose(action)?;
    wait(f, |f, o| {
        !painted(o, "Atom label") && f.node("Element", &[Role::TextInput]).is_err()
    })?;
    Ok(())
}

fn consume(
    path: &Path,
    expected: &Value,
    element: u8,
    h: u16,
    canonical: &str,
    formula: &str,
) -> TestResult {
    assert_eq!(
        independently_read_actual_saved_graph(path)?,
        expected["pages"][0]["fragments"]
    );
    let output=Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external Python")?)
        .args(["-B","-c",r#"
from pathlib import Path
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
import sys
assert rdBase.rdkitVersion=='2026.09.2'
ms=Chem.MolsFromCDXML(Path(sys.argv[1]).read_text(encoding='utf-8'))
assert len(ms)==1
m=ms[0]
assert [[a.GetAtomicNum(),a.GetTotalNumHs(),a.GetFormalCharge(),a.GetIsotope(),a.GetNumRadicalElectrons()] for a in m.GetAtoms()]==[[6,3,0,0,0],[int(sys.argv[2]),int(sys.argv[3]),0,0,0]]
assert [(b.GetBeginAtomIdx(),b.GetEndAtomIdx(),b.GetBondTypeAsDouble()) for b in m.GetBonds()]==[(0,1,1.0)]
assert Chem.MolToSmiles(m)==sys.argv[4] and rdMolDescriptors.CalcMolFormula(m)==sys.argv[5]
"#]).arg(path).arg(element.to_string()).arg(h.to_string()).arg(canonical).arg(formula)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn literal_oh_then_nh2_apply_history_and_unicode_file_save_reopen_are_actual_app_workflows()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = seed(&mut f)?;
    let original_bytes = fs::read(&input)?;
    let original = state_without_form(&f)?;
    open_atom(&mut f)?;
    assert_eq!(value(&f, "Element")?, "C");
    assert_eq!(value(&f, "Attached hydrogens")?, "3");
    set(&mut f, "Element", "O")?;
    set(&mut f, "Attached hydrogens", "1")?;
    assert_eq!(state_without_form(&f)?, original);
    close_form(&mut f, "Cancel")?;
    assert_eq!(state_without_form(&f)?, original);
    open_atom(&mut f)?;
    set(&mut f, "Element", "O")?;
    set(&mut f, "Attached hydrogens", "1")?;
    close_form(&mut f, "Apply")?;
    let mut methanol = original["document"].clone();
    methanol["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(8);
    methanol["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(1);
    assert_eq!(f.state()?["document"], methanol);
    assert_eq!(
        f.state()?["revision"],
        original["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(f.state()?["file"]["dirty"], true);
    shortcut(&mut f, Key::Z, false);
    assert_eq!(f.state()?["document"], original["document"]);
    assert_eq!(f.state()?["file"]["dirty"], false);
    shortcut(&mut f, Key::Z, true);
    assert_eq!(f.state()?["document"], methanol);
    let output = f.directory.path("native-label-化学-λ.cdxml");
    shortcut(&mut f, Key::S, true);
    wait(&mut f, |f, _| {
        f.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(
        &mut f,
        "File name",
        output.to_str().ok_or("Unicode output")?,
    )?;
    f.choose("Save")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(
        f.state()?["file"]["path"],
        json!(
            fs::canonicalize(&output)?
                .to_str()
                .ok_or("canonical output")?
        )
    );
    consume(&output, &methanol, 8, 1, "CO", "CH4O")?;
    shortcut(&mut f, Key::O, false);
    wait(&mut f, |f, _| {
        f.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut f, "File name", output.to_str().ok_or("output")?)?;
    f.choose("Open")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(f.state()?["document"], methanol);
    assert_eq!(f.state()?["undo_depth"], 0);
    open_atom(&mut f)?;
    assert_eq!(value(&f, "Element")?, "O");
    assert_eq!(value(&f, "Attached hydrogens")?, "1");
    set(&mut f, "Element", "N")?;
    set(&mut f, "Attached hydrogens", "2")?;
    close_form(&mut f, "Apply")?;
    let mut methylamine = methanol.clone();
    methylamine["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(7);
    methylamine["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(2);
    assert_eq!(f.state()?["document"], methylamine);
    assert_eq!(f.state()?["undo_depth"], 1);
    shortcut(&mut f, Key::S, false);
    finish_actual_file_workflow(&mut f)?;
    consume(&output, &methylamine, 7, 2, "CN", "CH5N")?;
    shortcut(&mut f, Key::Z, false);
    assert_eq!(f.state()?["document"], methanol);
    assert_eq!(f.state()?["file"]["dirty"], true);
    shortcut(&mut f, Key::Z, true);
    assert_eq!(f.state()?["document"], methylamine);
    assert_eq!(f.state()?["file"]["dirty"], false);
    shortcut(&mut f, Key::O, false);
    wait(&mut f, |f, _| {
        f.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut f, "File name", output.to_str().ok_or("output")?)?;
    f.choose("Open")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(f.state()?["document"], methylamine);
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(fs::read(&input)?, original_bytes);
    Ok(())
}

#[test]
fn exact_atom_target_bounded_selected_paste_and_invalid_literal_drafts_never_author() -> TestResult
{
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    seed(&mut f)?;
    let before = state_without_form(&f)?;
    for point in [[260.0, 380.0], [420.0, 500.0]] {
        click_alt(&mut f, point)?;
        let shown = f.warm();
        assert!(!painted(&shown, "Atom label"));
        assert!(f.node("Element", &[Role::TextInput]).is_err());
        assert_eq!(
            state_without_form(&f)?,
            before,
            "a bond/blank target cannot fall back to an atom"
        );
    }
    open_atom(&mut f)?;
    select_and_reject_paste(&mut f, "Element", "ClX", "C")?;
    select_and_reject_paste(&mut f, "Element", "汉", "C")?;
    select_and_reject_paste(&mut f, "Attached hydrogens", "123456", "3")?;
    for text in ["o", "D", "Me", "*", "C+", ""] {
        set(&mut f, "Element", text)?;
        assert!(
            !apply_enabled(&f)?,
            "unsupported symbol {text:?} enabled Apply"
        );
        assert_eq!(state_without_form(&f)?, before);
    }
    set(&mut f, "Element", "C")?;
    for text in ["-1", "1.5", "1e1", "65536", ""] {
        set(&mut f, "Attached hydrogens", text)?;
        assert!(!apply_enabled(&f)?, "invalid H {text:?} enabled Apply");
        assert_eq!(state_without_form(&f)?, before);
    }
    set(&mut f, "Element", "H")?;
    set(&mut f, "Attached hydrogens", "1")?;
    assert!(!apply_enabled(&f)?);
    set(&mut f, "Element", "Si")?;
    set(&mut f, "Attached hydrogens", "65535")?;
    assert!(
        apply_enabled(&f)?,
        "literal UINT16 range must not imply a valence restriction"
    );
    assert_eq!(state_without_form(&f)?, before);
    close_form(&mut f, "Cancel")?;
    assert_eq!(state_without_form(&f)?, before);

    for kind in ["mol", "smiles"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        seed(&mut f)?;
        let path = f.directory.path(&format!("guarded.{kind}"));
        if kind == "mol" {
            f.command("document.save_mol", json!({"path":path}))?;
            f.command("document.open_mol", json!({"path":path}))?;
        } else {
            fs::write(&path, b"CC retained-name\n")?;
            f.command("document.open_smiles", json!({"path":path}))?;
            f.command(
                "document.import_smiles_records",
                json!({"expected_revision":f.state()?["revision"],
                "selection":{"mode":"all"},"bond_length_pt":64,"origin_x_pt":220,"origin_y_pt":380,
                "page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4}),
            )?;
        }
        f.command("ui.view", json!({"action":"actual"}))?;
        let before = state_without_form(&f)?;
        let original = fs::read(&path)?;
        let atom = &before["document"]["pages"][0]["fragments"][0]["atoms"][1]["position"];
        click_alt(
            &mut f,
            [
                atom["x_pt"].as_f64().ok_or("x")?,
                atom["y_pt"].as_f64().ok_or("y")?,
            ],
        )?;
        assert!(!painted(&f.warm(), "Atom label"));
        assert!(f.node("Element", &[Role::TextInput]).is_err());
        assert_eq!(state_without_form(&f)?, before);
        assert_eq!(fs::read(&path)?, original);
    }
    Ok(())
}

fn native_close_frame(f: &mut Fixture) -> egui::FullOutput {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(f.width, f.height),
        )),
        focused: true,
        ..Default::default()
    };
    let viewport = input.viewports.entry(egui::ViewportId::ROOT).or_default();
    viewport.native_pixels_per_point = Some(f.dpi);
    viewport.events.push(egui::ViewportEvent::Close);
    let context = f.context.clone();
    let output = context.run(input, |context| f.app.update_workspace(context));
    if let Some(update) = &output.platform_output.accesskit_update {
        f.nodes.extend(update.nodes.iter().cloned());
    }
    output
}

#[test]
fn remote_context_aba_detaches_copyable_drafts_and_close_waits_for_explicit_cancel() -> TestResult {
    for change in [
        "revision",
        "view",
        "tool",
        "file",
        "source",
        "close",
        "committed",
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = seed(&mut f)?;
        let original = fs::read(&input)?;
        // The explicit presentation command must use the same target-qualified App form.
        f.command(
            "ui.atom_label",
            json!({"action":"open","expected_revision":f.state()?["revision"],
            "page_id":1,"fragment_id":1,"atom_id":2}),
        )?;
        wait(&mut f, |f, o| {
            painted(o, "Atom label") && value(f, "Element").is_ok()
        })?;
        set(&mut f, "Element", "Cl")?;
        set(&mut f, "Attached hydrogens", "0")?;
        if change == "close" {
            let before = state_without_form(&f)?;
            let closed = native_close_frame(&mut f);
            let commands = &closed
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .ok_or("viewport")?
                .commands;
            assert!(
                commands
                    .iter()
                    .any(|command| matches!(command, egui::ViewportCommand::CancelClose))
            );
            assert!(
                !commands
                    .iter()
                    .any(|command| matches!(command, egui::ViewportCommand::Close))
            );
            assert_eq!(
                state_without_form(&f)?,
                before,
                "native Close must not discard or advance the unresolved draft"
            );
        }
        match change {
            "revision" => {
                f.command("atom.set_label",json!({"expected_revision":f.state()?["revision"],"page_id":1,"fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}))?;
                f.command("history.undo", json!({}))?;
            }
            "view" => {
                f.command("ui.view", json!({"action":"set_zoom","percent":150}))?;
                f.command("ui.view", json!({"action":"actual"}))?;
            }
            "tool" => {
                f.command("ui.bond_tool", json!({"order":"double"}))?;
                f.command("ui.bond_tool", json!({"order":"single"}))?;
            }
            "file" => {
                let other = f.directory.path("association-B-λ.cdxml");
                let mut engine = f.app.engine.lock().map_err(|_| "engine")?;
                engine.dispatch("document.save_cdxml", json!({"path":other}))?;
                engine.dispatch(
                    "document.save_cdxml",
                    json!({"path":input,"overwrite":true}),
                )?;
            }
            "source" => {
                f.command("document.open_cdxml", json!({"path":input}))?;
            }
            "close" => {
                f.command(
                    "document.close",
                    json!({"expected_revision":f.state()?["revision"],"action":"request"}),
                )?;
                f.command(
                    "document.close",
                    json!({"expected_revision":f.state()?["revision"],"action":"cancel"}),
                )?;
            }
            _ => {
                f.command(
                    "document.close",
                    json!({"expected_revision":f.state()?["revision"],"action":"discard"}),
                )?;
                assert!(f.app.engine.lock().map_err(|_| "engine")?.complete_close());
            }
        }
        let current = state_without_form(&f)?;
        let shown = f.warm();
        assert!(painted(&shown, "Atom label"));
        assert_eq!(value(&f, "Element")?, "Cl");
        assert_eq!(value(&f, "Attached hydrogens")?, "0");
        assert!(!apply_enabled(&f)?, "{change} must detach the old target");
        if change == "committed" {
            assert!(
                !shown
                    .viewport_output
                    .get(&egui::ViewportId::ROOT)
                    .ok_or("viewport")?
                    .commands
                    .iter()
                    .any(|command| matches!(command, egui::ViewportCommand::Close))
            );
        }
        focus(&mut f, "Element")?;
        let copied = f.frame(vec![Event::Copy]);
        assert_eq!(
            copied.platform_output.copied_text, "Cl",
            "stale draft must remain selectable/copyable"
        );
        assert_eq!(state_without_form(&f)?, current);
        close_form(&mut f, "Cancel")?;
        assert_eq!(state_without_form(&f)?, current);
        assert_eq!(fs::read(&input)?, original);
    }
    Ok(())
}

#[test]
fn apply_uses_its_event_time_draft_and_dismissal_fences_later_canvas_history_and_save() -> TestResult
{
    for (action, input_first, typed) in [
        ("Apply", false, false),
        ("Apply", true, false),
        ("Apply", false, true),
        ("Apply", true, true),
        ("Cancel", false, false),
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = seed(&mut f)?;
        let original_bytes = fs::read(&input)?;
        let original = state_without_form(&f)?;
        let camera = f.view()?;
        let tool = f.app.engine.lock().map_err(|_| "engine")?.bond_tool();
        let (double_tool, _) = f.node("Double bond", &[Role::Button])?;
        let area = canvas(&f.warm())?;
        open_atom(&mut f)?;
        set(&mut f, "Element", "Cl")?;
        set(&mut f, "Attached hydrogens", "0")?;
        let (field, _) = f.node("Element", &[Role::TextInput])?;
        if typed {
            focus(&mut f, "Element")?;
        }
        let (apply, _) = f.node(action, &[Role::Button])?;
        let activate = Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: apply,
            data: None,
        });
        let edit = if typed {
            Event::Text("F".into())
        } else {
            Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::SetValue,
                target: field,
                data: Some(egui::accesskit::ActionData::Value("F".into())),
            })
        };
        let mut events = if input_first {
            vec![edit, activate]
        } else {
            vec![activate, edit]
        };
        let a = projected(
            area,
            [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
            100.0,
            None,
            [220.0, 500.0],
        );
        let b = projected(
            area,
            [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4],
            100.0,
            None,
            [300.0, 500.0],
        );
        events.extend([
            Event::PointerMoved(a),
            button(a, true),
            Event::PointerMoved(b),
            button(b, false),
            key(Key::Z, true, Modifiers::COMMAND),
            key(Key::Z, false, Modifiers::COMMAND),
            key(Key::S, true, Modifiers::COMMAND),
            key(Key::S, false, Modifiers::COMMAND),
            key(Key::F7, true, Modifiers::NONE),
            key(Key::F7, false, Modifiers::NONE),
            Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Default,
                target: double_tool,
                data: None,
            }),
        ]);
        f.frame(events);
        wait(&mut f, |f, o| {
            !painted(o, "Atom label") && f.node("Element", &[Role::TextInput]).is_err()
        })?;
        assert!(
            f.state()?["ui_file"]["request"].is_null(),
            "same-packet Save cannot become a held continuation"
        );
        assert_eq!(
            fs::read(&input)?,
            original_bytes,
            "same-packet Save must not publish"
        );
        assert_eq!(f.view()?, camera, "same-packet View input must stay fenced");
        assert_eq!(
            f.app.engine.lock().map_err(|_| "engine")?.bond_tool(),
            tool,
            "same-packet tool input must stay fenced"
        );
        let mut expected = original["document"].clone();
        if action == "Apply" {
            expected["pages"][0]["fragments"][0]["atoms"][1]["element"] =
                json!(if input_first { 9 } else { 17 });
            expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(0);
            assert_eq!(
                f.state()?["revision"],
                original["revision"].as_u64().ok_or("revision")? + 1
            );
            assert_eq!(f.state()?["undo_depth"], 1);
            assert_eq!(f.state()?["redo_depth"], 0);
            assert_eq!(
                f.state()?["document"],
                expected,
                "later draft input cannot change an earlier Apply"
            );
            assert_eq!(f.state()?["file"]["dirty"], true);
            shortcut(&mut f, Key::S, false);
            finish_actual_file_workflow(&mut f)?;
            assert_eq!(
                independently_read_actual_saved_graph(&input)?,
                expected["pages"][0]["fragments"]
            );
            assert_eq!(f.state()?["file"]["dirty"], false);
        } else {
            assert_eq!(state_without_form(&f)?, original);
            f.frame(vec![
                Event::PointerMoved(a),
                button(a, true),
                Event::PointerMoved(b),
                button(b, false),
            ]);
            assert_eq!(
                f.state()?["document"]["pages"][0]["fragments"]
                    .as_array()
                    .ok_or("fragments")?
                    .len(),
                2,
                "a fresh later gesture must author after Cancel releases the modal"
            );
        }
    }
    Ok(())
}
// Independent extra case: full-packet pointer focus loss must preserve raw text order.
#[test]
fn ordinary_text_and_physical_apply_use_the_draft_at_their_own_event_order() -> TestResult {
    for text_first in [true, false] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = seed(&mut f)?;
        let original_bytes = fs::read(&input)?;
        let original = state_without_form(&f)?;
        open_atom(&mut f)?;
        set(&mut f, "Element", "Cl")?;
        set(&mut f, "Attached hydrogens", "0")?;
        focus(&mut f, "Element")?;
        assert_eq!(value(&f, "Element")?, "Cl");
        assert_eq!(value(&f, "Attached hydrogens")?, "0");
        assert!(apply_enabled(&f)?);
        let (_, apply) = f.node("Apply", &[Role::Button])?;
        let press = button(apply.center(), true);
        let release = button(apply.center(), false);
        let events = if text_first {
            vec![
                Event::Text("F".into()),
                Event::PointerMoved(apply.center()),
                press,
                release,
            ]
        } else {
            vec![
                Event::PointerMoved(apply.center()),
                press,
                release,
                Event::Text("F".into()),
            ]
        };
        f.frame(events);
        wait(&mut f, |f, shown| {
            !painted(shown, "Atom label") && f.node("Element", &[Role::TextInput]).is_err()
        })?;
        let element = if text_first { 9 } else { 17 };
        let mut expected = original["document"].clone();
        expected["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(element);
        expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(0);
        let result = f.state()?;
        assert_eq!(
            result["document"], expected,
            "physical Apply must use its event-time draft, text_first={text_first}"
        );
        let revision = original["revision"].as_u64().ok_or("revision")? + 1;
        assert_eq!(result["revision"], revision);
        assert_eq!(result["undo_depth"], 1);
        assert_eq!(result["redo_depth"], 0);
        assert_eq!(result["file"]["path"], original["file"]["path"]);
        assert_eq!(result["file"]["dirty"], true);
        assert_eq!(result["ui_file"], original["ui_file"]);
        assert_eq!(
            fs::read(&input)?,
            original_bytes,
            "Apply alone must not publish a file"
        );
        shortcut(&mut f, Key::S, false);
        finish_actual_file_workflow(&mut f)?;
        assert_eq!(f.state()?["revision"], revision);
        assert_eq!(f.state()?["undo_depth"], 1);
        assert_eq!(f.state()?["file"]["dirty"], false);
        let (canonical, formula) = if text_first {
            ("CF", "CH3F")
        } else {
            ("CCl", "CH3Cl")
        };
        consume(&input, &expected, element, 0, canonical, formula)?;
    }
    Ok(())
}
// Append-only independent case using the frozen actual-App helpers; no internal input manipulation.
#[test]
fn earlier_cross_field_click_keeps_its_text_edit_before_a_later_pointer_apply() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = seed(&mut f)?;
    let original_bytes = fs::read(&input)?;
    let original = state_without_form(&f)?;
    let camera = f.view()?;
    let tool = f.app.engine.lock().map_err(|_| "engine")?.bond_tool();
    open_atom(&mut f)?;
    set(&mut f, "Element", "Cl")?;
    set(&mut f, "Attached hydrogens", "1")?;
    focus(&mut f, "Element")?;
    assert_eq!(value(&f, "Element")?, "Cl");
    assert_eq!(value(&f, "Attached hydrogens")?, "1");
    assert!(apply_enabled(&f)?);
    assert_eq!(state_without_form(&f)?, original);
    let (_, hydrogens) = f.node("Attached hydrogens", &[Role::TextInput])?;
    let (_, apply) = f.node("Apply", &[Role::Button])?;
    f.frame(vec![
        Event::PointerMoved(hydrogens.center()),
        button(hydrogens.center(), true),
        button(hydrogens.center(), false),
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Text("0".into()),
        Event::PointerMoved(apply.center()),
        button(apply.center(), true),
        button(apply.center(), false),
    ]);
    let mut expected = original["document"].clone();
    expected["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(17);
    expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(0);
    let applied = f.state()?;
    assert_eq!(
        applied["document"], expected,
        "the earlier click and text must edit H, not the previously focused Element"
    );
    assert_eq!(
        applied["revision"],
        original["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(applied["undo_depth"], 1);
    assert_eq!(applied["redo_depth"], 0);
    assert_eq!(applied["file"]["dirty"], true);
    assert_eq!(applied["source"]["modified"], true);
    assert!(applied["ui_file"]["request"].is_null());
    assert_eq!(
        fs::read(&input)?,
        original_bytes,
        "Apply must not publish old or new source"
    );
    wait(&mut f, |f, output| {
        !painted(output, "Atom label") && f.node("Element", &[Role::TextInput]).is_err()
    })?;
    assert!(f.state()?.get("ui_atom_label").is_none());
    assert_eq!(f.view()?, camera);
    assert_eq!(f.app.engine.lock().map_err(|_| "engine")?.bond_tool(), tool);
    shortcut(&mut f, Key::S, false);
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(f.state()?["document"], expected);
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(f.state()?["source"]["modified"], true);
    consume(&input, &expected, 17, 0, "CCl", "CH3Cl")?;
    Ok(())
}
// Independent native-surface serialization control after real failed registered work.
#[test]
fn a_retained_failed_paste_draft_cannot_be_covered_by_a_remote_atom_label_form() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = seed(&mut f)?;
    let original_bytes = fs::read(&input)?;
    let original = state_without_form(&f)?;
    let (owner, stamp, model) = {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        (
            engine.content().clone(),
            engine.capture_stamp(),
            engine.stored_document()?.model_snapshot(),
        )
    };
    let paste_value = |f: &Fixture| -> TestResult<String> {
        let (id, _) = f.node("SMILES", &[Role::MultilineTextInput])?;
        let mut stack = vec![id];
        let mut text = String::new();
        while let Some(id) = stack.pop() {
            let node = f.nodes.get(&id).ok_or("paste text node")?;
            if node.role() == Role::InlineTextBox {
                text.push_str(node.value().ok_or("paste inline text")?);
            }
            stack.extend(node.children().iter().rev().copied());
        }
        Ok(text)
    };
    f.choose("Edit")?;
    f.choose("Paste Special")?;
    f.choose("SMILES…")?;
    wait(&mut f, |f, shown| {
        painted(shown, "Paste SMILES") && paste_value(f).is_ok()
    })?;
    assert_eq!(paste_value(&f)?, "");
    let (notation, _) = f.node("SMILES", &[Role::MultilineTextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: notation,
            data: None,
        },
    )]);
    f.frame(vec![Event::Paste("[NH4+]".into())]);
    wait(&mut f, |f, _| {
        paste_value(f).is_ok_and(|text| text == "[NH4+]")
    })?;
    for (name, text) in [
        ("Bond length (pt)", "64"),
        ("Origin X (pt)", "96"),
        ("Origin Y (pt)", "96"),
    ] {
        set(&mut f, name, text)?;
    }
    assert_eq!(f.state()?["ui_file"]["request"]["action"], "paste_smiles");
    assert_eq!(f.state()?["ui_file"]["request"]["phase"], "accepted");
    f.choose("Apply")?;
    let failed = wait(&mut f, |f, shown| {
        !f.app.files.is_busy()
            && !f.app.executor.has_active_work()
            && f.state()
                .is_ok_and(|state| state["ui_file"]["request"].is_null())
            && painted(shown, "Paste SMILES")
            && paste_value(f).is_ok_and(|text| text == "[NH4+]")
            && shown.shapes.iter().any(|paint| {
                matches!(&paint.shape,egui::Shape::Text(text)
                if text.galley.text().to_ascii_lowercase().contains("charg"))
            })
    })?;
    let error_text = failed
        .shapes
        .iter()
        .find_map(|paint| match &paint.shape {
            egui::Shape::Text(text)
                if text.galley.text().to_ascii_lowercase().contains("charg") =>
            {
                Some(text.galley.text().to_owned())
            }
            _ => None,
        })
        .ok_or("actual charged-paste error")?;
    let failed_state = state_without_form(&f)?;
    let mut expected = original;
    expected["ui_file"] = failed_state["ui_file"].clone();
    assert_eq!(
        failed_state, expected,
        "failed real work must preserve complete authoring state"
    );
    assert!(
        failed_state["ui_file"]["request"].is_null(),
        "this test requires consumed accepted intent, not a fake pending request"
    );
    let atom_accepted = {
        let mut engine = f.app.engine.lock().map_err(|_| "engine")?;
        let revision = engine.state().revision;
        match engine.dispatch(
            "ui.atom_label",
            json!({"action":"open","expected_revision":revision,
            "page_id":1,"fragment_id":1,"atom_id":2}),
        ) {
            Ok(_) => true,
            Err(error) => {
                assert_eq!(
                    error.info().code,
                    "ui_busy",
                    "only truthful native-surface Busy may decline this otherwise valid request"
                );
                false
            }
        }
    };
    let shown = f.warm();
    assert!(
        !painted(&shown, "Atom label"),
        "an earlier retained Paste SMILES form owns the native surface even after its intent was consumed"
    );
    assert!(f.node("Element", &[Role::TextInput]).is_err());
    assert!(painted(&shown, "Paste SMILES"));
    assert_eq!(paste_value(&f)?, "[NH4+]");
    assert!(
        painted(&shown, &error_text),
        "the earlier failure must remain visible"
    );
    assert_eq!(state_without_form(&f)?, failed_state);
    assert_eq!(fs::read(&input)?, original_bytes);
    {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        assert_eq!(engine.content(), &owner);
        assert!(engine.capture_stamp_matches(&stamp));
        assert!(std::sync::Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    // There is exactly one visible Cancel here, belonging to the older paste form.
    f.choose("Cancel")?;
    wait(&mut f, |f, shown| {
        !painted(shown, "Paste SMILES")
            && f.node("SMILES", &[Role::MultilineTextInput]).is_err()
            && !f.app.files.is_busy()
            && !f.app.executor.has_active_work()
    })?;
    if !atom_accepted {
        f.command(
            "ui.atom_label",
            json!({"action":"open","expected_revision":f.state()?["revision"],
            "page_id":1,"fragment_id":1,"atom_id":2}),
        )?;
    }
    wait(&mut f, |f, shown| {
        painted(shown, "Atom label") && value(f, "Element").is_ok()
    })?;
    assert_eq!(value(&f, "Element")?, "C");
    assert_eq!(value(&f, "Attached hydrogens")?, "3");
    assert!(f.node("SMILES", &[Role::MultilineTextInput]).is_err());
    assert!(!f.app.files.is_busy());
    assert!(!f.app.executor.has_active_work());
    assert_eq!(state_without_form(&f)?, failed_state);
    close_form(&mut f, "Cancel")?;
    assert_eq!(state_without_form(&f)?, failed_state);
    assert_eq!(fs::read(&input)?, original_bytes);
    Ok(())
}
// Grounded single-line selection paths, without galley or cursor-state access.
#[test]
fn shift_click_and_drag_selection_before_physical_apply_replace_the_whole_element() -> TestResult {
    for shift_click in [true, false] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = seed(&mut f)?;
        let original_bytes = fs::read(&input)?;
        let original = state_without_form(&f)?;
        open_atom(&mut f)?;
        set(&mut f, "Element", "Cl")?;
        set(&mut f, "Attached hydrogens", "0")?;
        focus(&mut f, "Element")?;
        // Public End collapses the existing SelectAll to the known end of two characters.
        f.frame(vec![
            key(Key::End, true, Modifiers::NONE),
            key(Key::End, false, Modifiers::NONE),
        ]);
        f.frame(vec![]);
        assert_eq!(value(&f, "Element")?, "Cl");
        assert_eq!(value(&f, "Attached hydrogens")?, "0");
        assert_eq!(state_without_form(&f)?, original);
        let (_, field) = f.node("Element", &[Role::TextInput])?;
        let (_, apply) = f.node("Apply", &[Role::Button])?;
        assert!(field.width() > 8.0 && field.height() > 4.0);
        let left = egui::pos2(field.left() + 2.0, field.center().y);
        let right = egui::pos2(field.right() - 2.0, field.center().y);
        let mut events = if shift_click {
            let modifiers = Modifiers {
                shift: true,
                ..Modifiers::NONE
            };
            vec![
                Event::PointerMoved(left),
                Event::PointerButton {
                    pos: left,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers,
                },
                Event::PointerButton {
                    pos: left,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers,
                },
            ]
        } else {
            vec![
                Event::PointerMoved(left),
                button(left, true),
                Event::PointerMoved(right),
                button(right, false),
            ]
        };
        events.extend([
            Event::Text("F".into()),
            Event::PointerMoved(apply.center()),
            button(apply.center(), true),
            button(apply.center(), false),
        ]);
        f.frame(events);
        let mut expected = original["document"].clone();
        expected["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(9);
        expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(0);
        let result = f.state()?;
        assert_eq!(
            result["document"], expected,
            "selection before text and Apply must replace Cl with F, shift_click={shift_click}"
        );
        let revision = original["revision"].as_u64().ok_or("revision")? + 1;
        assert_eq!(result["revision"], revision);
        assert_eq!(result["undo_depth"], 1);
        assert_eq!(result["redo_depth"], 0);
        assert_eq!(result["file"]["path"], original["file"]["path"]);
        assert_eq!(result["file"]["dirty"], true);
        assert_eq!(result["ui_file"], original["ui_file"]);
        assert_eq!(
            fs::read(&input)?,
            original_bytes,
            "selection and Apply cannot publish a file"
        );
        wait(&mut f, |f, shown| {
            !painted(shown, "Atom label") && f.node("Element", &[Role::TextInput]).is_err()
        })?;
        shortcut(&mut f, Key::S, false);
        finish_actual_file_workflow(&mut f)?;
        assert_eq!(f.state()?["revision"], revision);
        assert_eq!(f.state()?["undo_depth"], 1);
        assert_eq!(f.state()?["file"]["dirty"], false);
        consume(&input, &expected, 9, 0, "CF", "CH3F")?;
    }
    Ok(())
}
// Append-only independent actual-App replacement-order and rejected-entry cases.
#[test]
fn atom_field_replacements_keep_prefix_order_and_rejection_never_applies_old_valid_text()
-> TestResult {
    for case in [
        "interleaved replacement",
        "empty replacement",
        "oversized replacement",
    ] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let input = seed(&mut f)?;
        let original_bytes = fs::read(&input)?;
        let original = state_without_form(&f)?;
        let camera = f.view()?;
        let tool = f.app.engine.lock().map_err(|_| "engine")?.bond_tool();
        open_atom(&mut f)?;
        set(&mut f, "Element", "Cl")?;
        set(&mut f, "Attached hydrogens", "0")?;
        focus(&mut f, "Element")?;
        assert_eq!(value(&f, "Element")?, "Cl");
        assert_eq!(value(&f, "Attached hydrogens")?, "0");
        assert!(
            apply_enabled(&f)?,
            "the previously shown Apply must be enabled"
        );
        assert_eq!(state_without_form(&f)?, original);
        let (field, _) = f.node("Element", &[Role::TextInput])?;
        let (_, apply) = f.node("Apply", &[Role::Button])?;
        let replacement = match case {
            "interleaved replacement" => "C",
            "empty replacement" => "",
            _ => "ClX", // Three UTF-8 bytes exceed this field's exact two-byte limit.
        };
        let replace = Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::SetValue,
            target: field,
            data: Some(egui::accesskit::ActionData::Value(replacement.into())),
        });
        let mut events = match case {
            "interleaved replacement" => {
                vec![Event::Text("N".into()), replace, Event::Text("l".into())]
            }
            "empty replacement" => vec![replace, Event::Text("F".into())],
            _ => vec![replace],
        };
        events.extend([
            Event::PointerMoved(apply.center()),
            button(apply.center(), true),
            button(apply.center(), false),
        ]);
        f.frame(events);

        if case == "oversized replacement" {
            assert_eq!(
                state_without_form(&f)?,
                original,
                "rejected entry before Apply must not commit the old valid Cl0 fields"
            );
            assert_eq!(fs::read(&input)?, original_bytes);
            let shown = wait(&mut f, |f, output| {
                painted(output, "Atom label")
                    && value(f, "Element").is_ok_and(|text| text == "Cl")
                    && value(f, "Attached hydrogens").is_ok_and(|text| text == "0")
            })?;
            assert!(painted(&shown, "Atom label"));
            assert_eq!(value(&f, "Element")?, "Cl");
            assert_eq!(value(&f, "Attached hydrogens")?, "0");
            assert_eq!(state_without_form(&f)?, original);
            assert_eq!(f.view()?, camera);
            assert_eq!(f.app.engine.lock().map_err(|_| "engine")?.bond_tool(), tool);
            assert_eq!(fs::read(&input)?, original_bytes);
            close_form(&mut f, "Cancel")?;
            assert_eq!(state_without_form(&f)?, original);
            assert_eq!(fs::read(&input)?, original_bytes);
            continue;
        }

        let (element, canonical, formula) = match case {
            "interleaved replacement" => (17, "CCl", "CH3Cl"),
            _ => (9, "CF", "CH3F"),
        };
        let mut expected = original["document"].clone();
        expected["pages"][0]["fragments"][0]["atoms"][1]["element"] = json!(element);
        expected["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"] = json!(0);
        let revision = original["revision"].as_u64().ok_or("revision")? + 1;
        let applied = f.state()?;
        assert_eq!(
            applied["document"], expected,
            "Apply must use the ordered replacement/text prefix: {case}"
        );
        assert_eq!(applied["revision"], revision);
        assert_eq!(applied["undo_depth"], 1);
        assert_eq!(applied["redo_depth"], 0);
        assert_eq!(applied["file"]["path"], original["file"]["path"]);
        assert_eq!(applied["file"]["dirty"], true);
        assert_eq!(applied["source"]["modified"], true);
        assert!(applied["ui_file"]["request"].is_null());
        assert_eq!(
            fs::read(&input)?,
            original_bytes,
            "Apply must not publish any file"
        );
        wait(&mut f, |f, output| {
            !painted(output, "Atom label") && f.node("Element", &[Role::TextInput]).is_err()
        })?;
        assert_eq!(f.view()?, camera);
        assert_eq!(f.app.engine.lock().map_err(|_| "engine")?.bond_tool(), tool);
        shortcut(&mut f, Key::S, false);
        finish_actual_file_workflow(&mut f)?;
        assert_eq!(f.state()?["document"], expected);
        assert_eq!(f.state()?["revision"], revision);
        assert_eq!(f.state()?["undo_depth"], 1);
        assert_eq!(f.state()?["redo_depth"], 0);
        assert_eq!(f.state()?["file"]["dirty"], false);
        assert_eq!(f.state()?["source"]["modified"], true);
        consume(&input, &expected, element, 0, canonical, formula)?;
    }
    Ok(())
}

// Independent modal-owner checks through existing collection callbacks after remote Open.
fn modal_review_control(f: &Fixture, name: &str, role: Role) -> Option<egui::accesskit::NodeId> {
    let mut seen = HashSet::new();
    let mut stack = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = f.nodes.get(&id)
        {
            let named = node.name() == Some(name)
                || node.labelled_by().iter().any(|label| {
                    f.nodes
                        .get(label)
                        .is_some_and(|node| node.name() == Some(name))
                });
            if node.role() == role && named {
                return Some(id);
            }
            stack.extend_from_slice(node.children());
        }
    }
    None
}

fn modal_review_default(id: egui::accesskit::NodeId) -> Event {
    Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
        action: egui::accesskit::Action::Default,
        target: id,
        data: None,
    })
}

#[test]
fn stale_atom_modal_cannot_queue_sdf_inspection_or_a_hidden_record_removal() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let drawing = seed(&mut f)?;
    let source = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");
    assert_eq!(source.len(), 777);
    let sdf = f.directory.path("modal-collection-原.sdf");
    fs::write(&sdf, source)?;
    // Establish the real Records expansion before the atom owns the surface.
    f.command("document.open_sdf", json!({"path":sdf}))?;
    f.warm();
    f.choose("Records")?;
    f.warm();
    f.node("Remove record 1", &[Role::Button])?;
    f.node("Inspect record 1 fields", &[Role::Button])?;
    f.command("document.open_cdxml", json!({"path":drawing}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    open_atom(&mut f)?;
    set(&mut f, "Element", "Cl")?;
    set(&mut f, "Attached hydrogens", "0")?;
    f.command("document.open_sdf", json!({"path":sdf}))?;
    let current = state_without_form(&f)?;
    let shown = f.warm();
    assert!(painted(&shown, "Atom label"));
    assert_eq!(value(&f, "Element")?, "Cl");
    assert!(!apply_enabled(&f)?);
    assert!(!f.app.sdf_fields.presentation_present());
    assert!(!f.app.sdf_fields.is_busy());
    // Suppressed/disabled controls are valid; a still-rendered control must reject activation.
    for name in ["Remove record 1", "Inspect record 1 fields"] {
        if let Some(id) = modal_review_control(&f, name, Role::Button) {
            f.frame(vec![modal_review_default(id)]);
        }
        assert!(
            !f.app.sdf_fields.presentation_present(),
            "{name} cannot create a hidden SDF presentation under the atom modal"
        );
        assert!(
            !f.app.sdf_fields.is_busy(),
            "{name} cannot queue a background field request under the atom modal"
        );
        assert_eq!(state_without_form(&f)?, current);
        assert_eq!(fs::read(&sdf)?, source);
    }
    assert_eq!(value(&f, "Element")?, "Cl");
    assert_eq!(value(&f, "Attached hydrogens")?, "0");
    close_form(&mut f, "Cancel")?;
    let after = f.warm();
    assert!(
        !painted(&after, "Remove SDF record"),
        "no hidden confirmation may appear after atom Cancel"
    );
    assert!(!painted(&after, "SDF field value"));
    assert!(!f.app.sdf_fields.presentation_present());
    assert!(!f.app.sdf_fields.is_busy());
    assert!(
        f.node("Record 1 field 1 value", &[Role::MultilineTextInput])
            .is_err(),
        "no background inspection may silently disclose a result after Cancel"
    );
    assert_eq!(state_without_form(&f)?, current);
    assert_eq!(fs::read(&sdf)?, source);
    // A new deliberate inspection remains usable once the owner has been dismissed.
    if modal_review_control(&f, "Inspect record 1 fields", Role::Button).is_none() {
        f.choose("Records")?;
    }
    f.choose("Inspect record 1 fields")?;
    wait(&mut f, |f, shown| {
        !f.app.sdf_fields.is_busy() && painted(shown, "Record 1 fields")
    })?;
    assert_eq!(state_without_form(&f)?, current);
    assert_eq!(fs::read(&sdf)?, source);
    Ok(())
}

#[test]
fn stale_atom_modal_blocks_raw_smiles_ordinal_and_inspection_until_explicit_cancel() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    seed(&mut f)?;
    open_atom(&mut f)?;
    set(&mut f, "Element", "Cl")?;
    set(&mut f, "Attached hydrogens", "0")?;
    let original = b"CC first\r\nC1.N.C1 \t\0\xffPRIVATE-META\r\nCC duplicate\tname";
    let input = f.directory.path("modal-records-原.smi");
    fs::write(&input, original)?;
    f.command("document.open_smiles", json!({"path":input}))?;
    let current = state_without_form(&f)?;
    let shown = f.warm();
    assert!(painted(&shown, "Atom label"));
    assert_eq!(value(&f, "Element")?, "Cl");
    assert!(!apply_enabled(&f)?);
    assert!(!f.app.smiles_records.is_busy());
    if let Some(id) = modal_review_control(&f, "Record number", Role::SpinButton) {
        f.frame(vec![Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::SetValue,
                target: id,
                data: Some(egui::accesskit::ActionData::NumericValue(2.0)),
            },
        )]);
    }
    f.warm();
    if let Some(id) = modal_review_control(&f, "Record number", Role::SpinButton) {
        assert_eq!(
            f.nodes.get(&id).ok_or("ordinal")?.numeric_value(),
            Some(1.0),
            "manual accessibility setters cannot navigate behind the atom modal"
        );
    }
    if let Some(id) = modal_review_control(&f, "Inspect record", Role::Button) {
        f.frame(vec![modal_review_default(id)]);
    }
    assert!(
        !f.app.smiles_records.is_busy(),
        "Inspect cannot start/queue hidden record work"
    );
    assert_eq!(state_without_form(&f)?, current);
    assert_eq!(fs::read(&input)?, original);
    assert_eq!(value(&f, "Element")?, "Cl");
    assert_eq!(value(&f, "Attached hydrogens")?, "0");
    close_form(&mut f, "Cancel")?;
    f.warm();
    assert!(!f.app.smiles_records.is_busy());
    assert!(
        f.node("Record 1 notation", &[Role::MultilineTextInput])
            .is_err()
    );
    assert!(
        f.node("Record 2 notation", &[Role::MultilineTextInput])
            .is_err()
    );
    let ordinal =
        modal_review_control(&f, "Record number", Role::SpinButton).ok_or("fresh ordinal")?;
    assert_eq!(
        f.nodes.get(&ordinal).ok_or("ordinal")?.numeric_value(),
        Some(1.0)
    );
    assert_eq!(state_without_form(&f)?, current);
    f.choose("Inspect record")?;
    wait(&mut f, |f, _| {
        !f.app.smiles_records.is_busy()
            && f.node("Record 1 notation", &[Role::MultilineTextInput])
                .is_ok()
    })?;
    assert_eq!(state_without_form(&f)?, current);
    assert_eq!(fs::read(&input)?, original);
    Ok(())
}

#[path = "skeletal_display_tests.rs"]
mod skeletal_display;

#[path = "terminal_label_tests.rs"]
mod terminal_label;

#[path = "boxed_atom_label_app_tests.rs"]
mod boxed_atom_label_app_tests;

#[path = "connected_atom_label_app_tests.rs"]
mod connected_atom_label_app_tests;

#[path = "dependent_label_app_tests.rs"]
mod dependent_label_app_tests;

#[path = "mixed_label_app_tests.rs"]
mod mixed_label_app_tests;
