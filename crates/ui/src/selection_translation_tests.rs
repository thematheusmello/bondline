//! Independent actual-App acceptance of the original locked selection/translation slice.
use super::{Fixture, TestResult};
use eframe::egui;

#[test]
fn marquee_selection_is_a_visible_actionable_native_drawing_control() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let shown = fixture.warm();
    let before = fixture.state()?;
    let (id, rect) = fixture.node(
        "Marquee selection",
        &[
            egui::accesskit::Role::Button,
            egui::accesskit::Role::CheckBox,
        ],
    )?;
    assert!(
        !fixture
            .nodes
            .get(&id)
            .ok_or("selection control")?
            .is_disabled()
    );
    assert!(
        rect.is_positive()
            && egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1160.0, 800.0))
                .contains_rect(rect)
    );
    assert!(
        shown.shapes.iter().any(|paint| {
            if !paint.clip_rect.intersects(rect) {
                return false;
            }
            match &paint.shape {
                egui::Shape::LineSegment { points, stroke } => {
                    stroke.width > 0.0 && points.iter().all(|point| rect.contains(*point))
                }
                egui::Shape::Rect(shape) => {
                    shape.stroke.width > 0.0 && rect.contains_rect(shape.rect)
                }
                egui::Shape::Path(shape) => {
                    shape.stroke.width > 0.0
                        && !shape.points.is_empty()
                        && shape.points.iter().all(|point| rect.contains(*point))
                }
                _ => false,
            }
        }),
        "the accessible tool must have visible control/glyph paint within its actual bounds"
    );
    fixture.frame(vec![
        egui::Event::PointerMoved(rect.center()),
        egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    fixture.frame(vec![egui::Event::PointerButton {
        pos: rect.center(),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    let after = fixture.state()?;
    assert_eq!(after["ui_selection"]["active"], true);
    assert_eq!(after["ui_selection"]["page_id"], serde_json::Value::Null);
    assert_eq!(after["ui_selection"]["atom_ids"], serde_json::json!([]));
    for field in [
        "document",
        "revision",
        "undo_depth",
        "redo_depth",
        "source",
        "file",
    ] {
        assert_eq!(
            after[field], before[field],
            "tool activation authored {field}"
        );
    }
    Ok(())
}

// Additive independent cases; append after the qualified immutable initial declaration.
use super::{
    button, canvas, finish_actual_file_workflow, independently_read_actual_saved_graph, key,
    projected,
};
use egui::{Event, Key, Modifiers, Pos2, accesskit::Role};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as Process,
    time::{Duration, Instant},
};

fn selection_wait(
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
            return Err("selection App state did not become ready".into());
        }
        std::thread::yield_now();
    }
}
fn selection_choose(f: &mut Fixture, label: &str) -> TestResult {
    selection_wait(f, |f, _| {
        f.node(label, &[Role::Button, Role::CheckBox]).is_ok()
    })?;
    let (_, rect) = f.node(label, &[Role::Button, Role::CheckBox])?;
    f.frame(vec![
        Event::PointerMoved(rect.center()),
        button(rect.center(), true),
    ]);
    f.frame(vec![button(rect.center(), false)]);
    Ok(())
}
fn selection_key(code: Key, modifiers: Modifiers) -> Vec<Event> {
    vec![key(code, true, modifiers), key(code, false, modifiers)]
}
fn selection_shortcut(f: &mut Fixture, code: Key, shift: bool) -> egui::FullOutput {
    f.frame(selection_key(
        code,
        Modifiers {
            shift,
            ..Modifiers::COMMAND
        },
    ))
}
fn selection_ids(f: &Fixture) -> TestResult<Vec<u32>> {
    f.state()?["ui_selection"]["atom_ids"]
        .as_array()
        .ok_or("selection IDs")?
        .iter()
        .map(|id| {
            id.as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .ok_or_else(|| "positive selection ID".into())
        })
        .collect()
}
fn selection_expect(f: &Fixture, ids: &[u32]) -> TestResult {
    let state = f.state()?;
    assert_eq!(state["ui_selection"]["active"], true);
    assert_eq!(selection_ids(f)?, ids);
    assert_eq!(
        state["ui_selection"]["page_id"],
        if ids.is_empty() {
            Value::Null
        } else {
            json!(1)
        }
    );
    Ok(())
}
fn selection_authored_unchanged(f: &Fixture, before: &Value) -> TestResult {
    let after = f.state()?;
    for field in [
        "document",
        "revision",
        "undo_depth",
        "redo_depth",
        "source",
        "file",
        "close",
        "close_epoch",
    ] {
        assert_eq!(
            after[field], before[field],
            "selection input changed authored/close field {field}"
        );
    }
    Ok(())
}
fn selection_point(f: &mut Fixture, point: [f64; 2]) -> TestResult<Pos2> {
    let output = f.warm();
    let state = f.state()?;
    let size = &state["document"]["pages"][0]["size"];
    let view = f.view()?;
    let percent = view["zoom_percent"]
        .as_f64()
        .ok_or("explicit selection camera")?;
    let center = match (view["center_x_pt"].as_f64(), view["center_y_pt"].as_f64()) {
        (Some(x), Some(y)) => Some([x, y]),
        _ => None,
    };
    let p = projected(
        canvas(&output)?,
        [
            size["width_pt"].as_f64().ok_or("width")?,
            size["height_pt"].as_f64().ok_or("height")?,
        ],
        percent,
        center,
        point,
    );
    assert!(
        canvas(&output)?.contains(p),
        "declared chemical point must be on the actual canvas"
    );
    Ok(p)
}
fn selection_position(f: &Fixture, id: u32) -> TestResult<[f64; 2]> {
    let state = f.state()?;
    for fragment in state["document"]["pages"][0]["fragments"]
        .as_array()
        .ok_or("fragments")?
    {
        for atom in fragment["atoms"].as_array().ok_or("atoms")? {
            if atom["id"] == id {
                return Ok([
                    atom["position"]["x_pt"].as_f64().ok_or("atom x")?,
                    atom["position"]["y_pt"].as_f64().ok_or("atom y")?,
                ]);
            }
        }
    }
    Err("declared atom missing".into())
}
fn selection_quiet_clicks(f: &mut Fixture) -> TestResult {
    // Pinned SDK default double-click delay is 0.3s; >2 delays separates triple counts.
    // These are bounded synthetic egui frames, not a wall-clock/job wait or native timing claim.
    let start = f.context.input(|i| i.time);
    for _ in 0..64 {
        f.frame(vec![]);
        if f.context.input(|i| i.time) - start > 0.65 {
            return Ok(());
        }
    }
    Err("synthetic input time did not advance beyond the SDK multi-click window".into())
}
fn selection_pointer(pos: Pos2, pressed: bool, shift: bool) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers {
            shift,
            ..Modifiers::NONE
        },
    }
}
fn selection_click_at(f: &mut Fixture, point: [f64; 2], shift: bool) -> TestResult {
    selection_quiet_clicks(f)?;
    let p = selection_point(f, point)?;
    f.frame(vec![
        Event::PointerMoved(p),
        selection_pointer(p, true, shift),
    ]);
    f.frame(vec![selection_pointer(p, false, shift)]);
    Ok(())
}
fn selection_marquee(f: &mut Fixture, a: [f64; 2], b: [f64; 2], shift: bool) -> TestResult {
    selection_quiet_clicks(f)?;
    let start = selection_point(f, a)?;
    let end = selection_point(f, b)?;
    f.frame(vec![
        Event::PointerMoved(start),
        selection_pointer(start, true, shift),
    ]);
    f.frame(vec![Event::PointerMoved(end)]);
    f.frame(vec![selection_pointer(end, false, shift)]);
    Ok(())
}
fn selection_seed(f: &mut Fixture) -> TestResult<PathBuf> {
    for (a, b, y) in [(180, 260, 320), (380, 460, 520)] {
        f.command(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":a,"begin_y_pt":y,"end_x_pt":b,"end_y_pt":y}),
        )?;
    }
    f.command("atom.set_label",json!({"expected_revision":f.state()?["revision"],"page_id":1,"fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}))?;
    let path = f.directory.path("selection-original-化学.cdxml");
    f.command("document.save_cdxml", json!({"path":path}))?;
    f.command("document.open_cdxml", json!({"path":path}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    Ok(path)
}
fn selection_expected_positions(
    before: &Value,
    positions: &[(u32, f64, f64)],
) -> TestResult<Value> {
    let mut expected = before.clone();
    let mut count = 0;
    for fragment in expected["pages"][0]["fragments"]
        .as_array_mut()
        .ok_or("expected fragments")?
    {
        for atom in fragment["atoms"].as_array_mut().ok_or("expected atoms")? {
            for &(id, x, y) in positions {
                if atom["id"] == id {
                    atom["position"] = json!({"x_pt":x,"y_pt":y});
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, positions.len());
    Ok(expected)
}
fn selection_consume(path: &Path, document: &Value) -> TestResult {
    assert_eq!(
        independently_read_actual_saved_graph(path)?,
        document["pages"][0]["fragments"]
    );
    let output = Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?)
        .args(["-B","-c",r#"
from pathlib import Path
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
import sys
assert rdBase.rdkitVersion=='2026.09.2'
ms=Chem.MolsFromCDXML(Path(sys.argv[1]).read_text(encoding='utf-8'))
assert len(ms)==2
for m,z,h,smiles,formula in zip(ms,[[6,8],[6,6]],[[3,1],[3,3]],['CO','CC'],['CH4O','C2H6']):
 assert m.GetNumAtoms()==2 and m.GetNumBonds()==1
 assert [a.GetAtomicNum() for a in m.GetAtoms()]==z
 assert [a.GetTotalNumHs() for a in m.GetAtoms()]==h
 assert all(a.GetFormalCharge()==0 and a.GetIsotope()==0 and a.GetNumRadicalElectrons()==0 for a in m.GetAtoms())
 assert m.GetBondWithIdx(0).GetBondTypeAsDouble()==1
 assert Chem.MolToSmiles(m)==smiles and rdMolDescriptors.CalcMolFormula(m)==formula
"#]).arg(path).output()?;
    assert!(
        output.status.success(),
        "actual saved-file chemistry: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn exact_atom_bond_shift_marquee_and_select_all_sets_are_transient() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = selection_seed(&mut f)?;
    let raw = fs::read(&path)?;
    let before = f.state()?;
    selection_choose(&mut f, "Marquee selection")?;
    selection_click_at(&mut f, [180.0, 320.0], false)?;
    selection_expect(&f, &[1])?;
    selection_click_at(&mut f, [260.0, 320.0], true)?;
    selection_expect(&f, &[1, 2])?;
    selection_click_at(&mut f, [180.0, 320.0], true)?;
    selection_expect(&f, &[2])?;
    selection_click_at(&mut f, [220.0, 320.0], false)?;
    selection_expect(&f, &[1, 2])?;
    selection_click_at(&mut f, [320.0, 430.0], false)?;
    selection_expect(&f, &[])?;
    // A rectangle that crosses only the middle of a bond has no atom anchors.
    selection_marquee(&mut f, [210.0, 310.0], [230.0, 330.0], false)?;
    selection_expect(&f, &[])?;
    selection_marquee(&mut f, [165.0, 300.0], [205.0, 338.0], false)?;
    selection_expect(&f, &[1])?;
    selection_marquee(&mut f, [365.0, 500.0], [475.0, 540.0], true)?;
    selection_expect(&f, &[1, 3, 4])?;
    selection_marquee(&mut f, [475.0, 540.0], [365.0, 500.0], false)?;
    selection_expect(&f, &[3, 4])?;
    selection_shortcut(&mut f, Key::A, false);
    selection_expect(&f, &[1, 2, 3, 4])?;
    selection_choose(&mut f, "Edit")?;
    selection_choose(&mut f, "Deselect")?;
    selection_expect(&f, &[])?;
    selection_choose(&mut f, "Edit")?;
    selection_choose(&mut f, "Select All")?;
    selection_expect(&f, &[1, 2, 3, 4])?;
    f.frame(selection_key(Key::Escape, Modifiers::NONE));
    selection_expect(&f, &[])?;
    selection_authored_unchanged(&f, &before)?;
    assert_eq!(fs::read(&path)?, raw);
    Ok(())
}

#[test]
fn double_click_selects_actual_cycle_component_not_all_atoms_in_one_fragment() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.command("structure.paste_smiles",json!({"expected_revision":0,"page_id":1,"notation":"C1CCCCC1.CC","bond_length_pt":64,"origin_x_pt":96,"origin_y_pt":96}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    let before = f.state()?;
    assert_eq!(
        before["document"]["pages"][0]["fragments"]
            .as_array()
            .ok_or("one complete Fragment")?
            .len(),
        1
    );
    assert_eq!(
        before["document"]["pages"][0]["fragments"][0]["atoms"]
            .as_array()
            .ok_or("atoms")?
            .len(),
        8
    );
    assert_eq!(
        before["document"]["pages"][0]["fragments"][0]["bonds"]
            .as_array()
            .ok_or("bonds")?
            .len(),
        7
    );
    selection_choose(&mut f, "Marquee selection")?;
    for (id, expected) in [(1, vec![1, 2, 3, 4, 5, 6]), (7, vec![7, 8])] {
        selection_quiet_clicks(&mut f)?;
        let position = selection_position(&f, id)?;
        let point = selection_point(&mut f, position)?;
        for _ in 0..2 {
            f.frame(vec![
                Event::PointerMoved(point),
                selection_pointer(point, true, false),
            ]);
            f.frame(vec![selection_pointer(point, false, false)]);
        }
        selection_expect(&f, &expected)?;
        selection_authored_unchanged(&f, &before)?;
    }
    Ok(())
}

#[test]
fn selected_target_drag_and_point_nudges_are_single_commits_with_real_unicode_save_history()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = selection_seed(&mut f)?;
    let raw = fs::read(&path)?;
    let before = f.state()?;
    selection_choose(&mut f, "Marquee selection")?;
    selection_click_at(&mut f, [220.0, 320.0], false)?;
    selection_expect(&f, &[1, 2])?;
    let view = f.view()?;
    let epoch = f.state()?["ui_selection"]["epoch"]
        .as_u64()
        .ok_or("selection epoch")?;
    selection_quiet_clicks(&mut f)?;
    let start = selection_point(&mut f, [180.0, 320.0])?;
    let end = selection_point(&mut f, [204.0, 308.0])?;
    f.frame(vec![
        Event::PointerMoved(start),
        selection_pointer(start, true, false),
    ]);
    f.frame(vec![Event::PointerMoved(end)]);
    selection_authored_unchanged(&f, &before)?;
    assert_eq!(fs::read(&path)?, raw, "preview cannot publish");
    f.frame(vec![selection_pointer(end, false, false)]);
    let dragged =
        selection_expected_positions(&before["document"], &[(1, 204.0, 308.0), (2, 284.0, 308.0)])?;
    assert_eq!(f.state()?["document"], dragged);
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(
        f.state()?["revision"],
        before["revision"].as_u64().ok_or("revision")? + 1
    );
    selection_expect(&f, &[1, 2])?;
    assert_eq!(f.state()?["ui_selection"]["epoch"], epoch + 1);
    f.frame(selection_key(Key::ArrowRight, Modifiers::NONE));
    let right =
        selection_expected_positions(&before["document"], &[(1, 205.0, 308.0), (2, 285.0, 308.0)])?;
    assert_eq!(f.state()?["document"], right);
    assert_eq!(f.state()?["undo_depth"], 2);
    f.frame(selection_key(
        Key::ArrowDown,
        Modifiers {
            shift: true,
            ..Modifiers::NONE
        },
    ));
    let final_document =
        selection_expected_positions(&before["document"], &[(1, 205.0, 318.0), (2, 285.0, 318.0)])?;
    assert_eq!(f.state()?["document"], final_document);
    assert_eq!(f.state()?["undo_depth"], 3);
    assert_eq!(f.state()?["file"]["dirty"], true);
    assert_eq!(f.state()?["source"]["modified"], true);
    assert_eq!(f.view()?, view);
    assert_eq!(fs::read(&path)?, raw);
    let selected_epoch = f.state()?["ui_selection"]["epoch"].clone();
    selection_shortcut(&mut f, Key::S, false);
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(
        f.state()?["file"]["path"],
        fs::canonicalize(&path)?.to_str().ok_or("Unicode path")?
    );
    assert_eq!(f.state()?["file"]["dirty"], false);
    selection_expect(&f, &[1, 2])?;
    assert_eq!(f.state()?["ui_selection"]["epoch"], selected_epoch);
    selection_consume(&path, &final_document)?;
    selection_shortcut(&mut f, Key::Z, false);
    assert_eq!(f.state()?["document"], right);
    assert_eq!(f.state()?["undo_depth"], 2);
    assert_eq!(f.state()?["redo_depth"], 1);
    selection_expect(&f, &[])?;
    selection_shortcut(&mut f, Key::Z, true);
    assert_eq!(f.state()?["document"], final_document);
    assert_eq!(f.state()?["undo_depth"], 3);
    assert_eq!(f.state()?["redo_depth"], 0);
    selection_expect(&f, &[])?;
    assert_eq!(f.state()?["file"]["dirty"], false);
    selection_consume(&path, &final_document)?;
    f.command("document.open_cdxml", json!({"path":path}))?;
    f.warm();
    assert_eq!(f.state()?["document"], final_document);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    selection_expect(&f, &[])?;
    Ok(())
}

// Independent ordered-input, modal/context and complete-source guards.
#[test]
fn modal_text_and_frame_start_cancel_fence_selection_history_canvas_and_save() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = selection_seed(&mut f)?;
    let raw = fs::read(&path)?;
    selection_choose(&mut f, "Marquee selection")?;
    selection_click_at(&mut f, [220.0, 320.0], false)?;
    f.frame(selection_key(Key::ArrowRight, Modifiers::NONE));
    let before = f.state()?;
    assert_eq!(before["undo_depth"], 1);
    selection_expect(&f, &[1, 2])?;
    f.command("ui.atom_label",json!({"action":"open","expected_revision":before["revision"],"page_id":1,"fragment_id":1,"atom_id":2}))?;
    selection_wait(&mut f, |f, _| f.node("Element", &[Role::TextInput]).is_ok())?;
    let (element, _) = f.node("Element", &[Role::TextInput])?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: element,
            data: None,
        },
    )]);
    let mut text_events = selection_key(Key::A, Modifiers::COMMAND);
    text_events.push(Event::Text("Cl".into()));
    text_events.extend(selection_key(Key::ArrowRight, Modifiers::NONE));
    text_events.extend(selection_key(
        Key::ArrowDown,
        Modifiers {
            shift: true,
            ..Modifiers::NONE
        },
    ));
    f.frame(text_events);
    selection_authored_unchanged(&f, &before)?;
    selection_expect(&f, &[1, 2])?;
    assert_eq!(fs::read(&path)?, raw);
    let start = selection_point(&mut f, [181.0, 320.0])?;
    let end = selection_point(&mut f, [205.0, 308.0])?;
    let (cancel, _) = f.node("Cancel", &[Role::Button])?;
    let mut same_packet = vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: cancel,
            data: None,
        },
    )];
    same_packet.extend(selection_key(Key::ArrowRight, Modifiers::NONE));
    same_packet.extend(selection_key(Key::Z, Modifiers::COMMAND));
    same_packet.extend(selection_key(Key::S, Modifiers::COMMAND));
    same_packet.extend([
        Event::PointerMoved(start),
        selection_pointer(start, true, false),
        Event::PointerMoved(end),
        selection_pointer(end, false, false),
    ]);
    f.frame(same_packet);
    f.warm();
    assert!(f.node("Element", &[Role::TextInput]).is_err());
    selection_authored_unchanged(&f, &before)?;
    selection_expect(&f, &[1, 2])?;
    assert_eq!(f.state()?["ui_file"]["request"], Value::Null);
    assert!(!f.app.files.is_busy());
    assert_eq!(fs::read(&path)?, raw);
    // A later deliberate canvas action works; the cancelled modal never leaves a queued nudge.
    selection_click_at(&mut f, [221.0, 320.0], false)?;
    f.frame(selection_key(Key::ArrowRight, Modifiers::NONE));
    let expected =
        selection_expected_positions(&before["document"], &[(1, 182.0, 320.0), (2, 262.0, 320.0)])?;
    assert_eq!(f.state()?["document"], expected);
    assert_eq!(f.state()?["undo_depth"], 2);
    assert_eq!(fs::read(&path)?, raw);
    selection_shortcut(&mut f, Key::S, false);
    finish_actual_file_workflow(&mut f)?;
    selection_consume(&path, &expected)?;
    Ok(())
}

#[test]
fn selection_drag_cancels_on_escape_focus_view_tool_source_and_dpi_context_changes() -> TestResult {
    for change in ["escape", "focus", "view_aba", "tool_aba", "open_aba", "dpi"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = selection_seed(&mut f)?;
        let raw = fs::read(&path)?;
        selection_choose(&mut f, "Marquee selection")?;
        selection_click_at(&mut f, [220.0, 320.0], false)?;
        selection_expect(&f, &[1, 2])?;
        selection_quiet_clicks(&mut f)?;
        let start = selection_point(&mut f, [180.0, 320.0])?;
        let end = selection_point(&mut f, [204.0, 308.0])?;
        let preview = f.state()?;
        f.frame(vec![
            Event::PointerMoved(start),
            selection_pointer(start, true, false),
        ]);
        f.frame(vec![Event::PointerMoved(end)]);
        selection_authored_unchanged(&f, &preview)?;
        let mut release = Vec::new();
        match change {
            "escape" => release.extend(selection_key(Key::Escape, Modifiers::NONE)),
            "focus" => {
                f.frame(vec![Event::WindowFocused(false)]);
                release.push(Event::WindowFocused(true));
            }
            "view_aba" => {
                f.command("ui.view", json!({"action":"set_zoom","percent":125}))?;
                f.command("ui.view", json!({"action":"set_zoom","percent":100}))?;
            }
            "tool_aba" => {
                f.command("ui.bond_tool", json!({"order":"double"}))?;
                f.command("ui.bond_tool", json!({"order":"single"}))?;
                f.command(
                    "ui.selection",
                    json!({"action":"activate","expected_revision":f.state()?["revision"]}),
                )?;
            }
            "open_aba" => {
                f.command("document.open_cdxml", json!({"path":path}))?;
            }
            "dpi" => {
                f.dpi = 2.0;
            }
            _ => return Err("undeclared context".into()),
        }
        let after_context = f.state()?;
        release.extend([
            Event::PointerMoved(end),
            selection_pointer(end, false, false),
        ]);
        f.frame(release);
        f.warm();
        selection_authored_unchanged(&f, &after_context)?;
        assert_eq!(
            f.state()?["document"],
            preview["document"],
            "{change} cannot commit the old preview"
        );
        assert_eq!(fs::read(&path)?, raw);
        if change == "escape" {
            selection_expect(&f, &[1, 2])?;
        }
        // Every cancellation path admits a fresh selected-target operation afterward.
        selection_choose(&mut f, "Marquee selection")?;
        selection_click_at(&mut f, [220.0, 320.0], false)?;
        selection_expect(&f, &[1, 2])?;
        let fresh = f.state()?;
        f.frame(selection_key(Key::ArrowRight, Modifiers::NONE));
        let expected = selection_expected_positions(
            &preview["document"],
            &[(1, 181.0, 320.0), (2, 261.0, 320.0)],
        )?;
        assert_eq!(
            f.state()?["document"],
            expected,
            "fresh post-{change} input"
        );
        assert_eq!(
            f.state()?["revision"],
            fresh["revision"].as_u64().ok_or("revision")? + 1
        );
    }
    // Escape during an unfinished rectangle preserves the previous set; bare Escape clears it.
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    selection_seed(&mut f)?;
    selection_choose(&mut f, "Marquee selection")?;
    selection_click_at(&mut f, [180.0, 320.0], false)?;
    let before = f.state()?;
    let a = selection_point(&mut f, [365.0, 500.0])?;
    let b = selection_point(&mut f, [475.0, 540.0])?;
    f.frame(vec![
        Event::PointerMoved(a),
        selection_pointer(a, true, false),
        Event::PointerMoved(b),
    ]);
    let mut cancel = selection_key(Key::Escape, Modifiers::NONE);
    cancel.push(selection_pointer(b, false, false));
    f.frame(cancel);
    selection_expect(&f, &[1])?;
    selection_authored_unchanged(&f, &before)?;
    f.frame(selection_key(Key::Escape, Modifiers::NONE));
    selection_expect(&f, &[])?;
    selection_authored_unchanged(&f, &before)?;
    Ok(())
}

#[test]
fn ordered_nudge_save_and_history_use_the_latest_qualified_snapshot() -> TestResult {
    for move_first in [true, false] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = selection_seed(&mut f)?;
        let raw = fs::read(&path)?;
        let before = f.state()?;
        selection_choose(&mut f, "Marquee selection")?;
        selection_click_at(&mut f, [220.0, 320.0], false)?;
        let nudge = selection_key(Key::ArrowRight, Modifiers::NONE);
        let save = selection_key(Key::S, Modifiers::COMMAND);
        let events = if move_first {
            [nudge, save].concat()
        } else {
            [save, nudge].concat()
        };
        f.frame(events);
        finish_actual_file_workflow(&mut f)?;
        let expected = if move_first {
            selection_expected_positions(
                &before["document"],
                &[(1, 181.0, 320.0), (2, 261.0, 320.0)],
            )?
        } else {
            before["document"].clone()
        };
        assert_eq!(f.state()?["document"], expected);
        assert_eq!(
            f.state()?["revision"],
            before["revision"].as_u64().ok_or("revision")? + u64::from(move_first)
        );
        assert_eq!(f.state()?["undo_depth"], u64::from(move_first));
        assert_eq!(f.state()?["file"]["dirty"], false);
        selection_expect(&f, &[1, 2])?;
        selection_consume(&path, &expected)?;
        if !move_first {
            assert_eq!(
                fs::read(&path)?,
                raw,
                "Save before a later nudge cannot publish a hidden edit"
            );
        }
        let saved_bytes = fs::read(&path)?;
        let saved = f.state()?;
        f.frame(selection_key(Key::ArrowRight, Modifiers::NONE));
        let x = if move_first { 182.0 } else { 181.0 };
        let fresh = selection_expected_positions(
            &before["document"],
            &[(1, x, 320.0), (2, x + 80.0, 320.0)],
        )?;
        assert_eq!(f.state()?["document"], fresh);
        assert_eq!(f.state()?["undo_depth"], u64::from(move_first) + 1);
        assert_eq!(f.state()?["file"]["dirty"], true);
        assert_eq!(fs::read(&path)?, saved_bytes);
        // The earlier nudge is a real commit, then Undo reverses it at its raw index.
        let mut packet = selection_key(Key::ArrowDown, Modifiers::NONE);
        packet.extend(selection_key(Key::Z, Modifiers::COMMAND));
        let prior = f.state()?;
        f.frame(packet);
        assert_eq!(f.state()?["document"], fresh);
        assert_eq!(
            f.state()?["revision"],
            prior["revision"].as_u64().ok_or("revision")? + 2
        );
        assert_eq!(f.state()?["undo_depth"], prior["undo_depth"]);
        assert_eq!(f.state()?["redo_depth"], 1);
        selection_expect(&f, &[])?;
        assert_eq!(f.state()?["file"]["path"], saved["file"]["path"]);
    }
    Ok(())
}

#[test]
fn complete_foreign_and_collection_sources_never_gain_geometry_authority_from_selection()
-> TestResult {
    let cases: [(&str, &str, &[u8]); 4] = [
        (
            "document.open_mol",
            "selection-source.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol"),
        ),
        (
            "document.open_cdx",
            "selection-source.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx"),
        ),
        (
            "document.open_sdf",
            "selection-source.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"),
        ),
        (
            "document.open_smiles",
            "selection-source.smi",
            b"CC first\r\nC1.N.C1 \t\0\xffPRIVATE-META\r\nCC duplicate\tname",
        ),
    ];
    for (command, name, source) in cases {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = f.directory.path(name);
        fs::write(&path, source)?;
        let params = if command == "document.open_mol" {
            json!({"path":path,"origin_x_pt":220,"origin_y_pt":340})
        } else {
            json!({"path":path})
        };
        f.command(command, params)?;
        f.warm();
        let before = f.state()?;
        if command == "document.open_mol" {
            f.command("ui.view", json!({"action":"actual"}))?;
            selection_choose(&mut f, "Marquee selection")?;
            let first = selection_position(&f, 1)?;
            selection_click_at(&mut f, first, false)?;
            selection_expect(&f, &[1])?;
            selection_shortcut(&mut f, Key::A, false);
            selection_expect(&f, &[1, 2])?;
            f.frame(selection_key(Key::ArrowRight, Modifiers::NONE));
            selection_authored_unchanged(&f, &before)?;
            assert_eq!(fs::read(&path)?, source);
            continue;
        }
        let error = f
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .dispatch(
                "ui.selection",
                json!({"action":"activate","expected_revision":before["revision"]}),
            )
            .err()
            .ok_or("foreign source selection was authorized")?;
        assert_eq!(error.info().code, "unsupported_content");
        let controls = f.command("ui.controls", json!({}))?;
        for control in controls["controls"].as_array().ok_or("controls")? {
            if ["tool.selection", "edit.select_all"].contains(&control["id"].as_str().unwrap_or(""))
            {
                assert_eq!(
                    control["enabled"], false,
                    "{command} cannot expose an enabled geometry-selection action"
                );
            }
        }
        let mut keys = selection_key(Key::A, Modifiers::COMMAND);
        keys.extend(selection_key(Key::ArrowRight, Modifiers::NONE));
        f.frame(keys);
        selection_authored_unchanged(&f, &before)?;
        assert_eq!(fs::read(&path)?, source);
        assert!(selection_ids(&f).map_or(true, |ids| ids.is_empty()));
    }
    Ok(())
}

// Additive independent chronology checks; the original eight declarations stay unchanged.
fn selection_tool_activation(f: &Fixture, label: &str) -> TestResult<Event> {
    let (id, _) = f.node(label, &[Role::Button, Role::CheckBox])?;
    assert!(!f.nodes.get(&id).ok_or("current tool")?.is_disabled());
    Ok(Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: id,
            data: None,
        },
    ))
}

#[test]
fn selected_movement_and_later_bond_tool_activation_obey_raw_event_order() -> TestResult {
    for movement in ["nudge", "drag"] {
        for tool_first in [false, true] {
            let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
            let path = selection_seed(&mut f)?;
            let raw = fs::read(&path)?;
            let before = f.state()?;
            selection_choose(&mut f, "Marquee selection")?;
            selection_click_at(&mut f, [220.0, 320.0], false)?;
            selection_expect(&f, &[1, 2])?;
            let movement_events = match movement {
                "nudge" => selection_key(Key::ArrowRight, Modifiers::NONE),
                "drag" => {
                    selection_quiet_clicks(&mut f)?;
                    let a = selection_point(&mut f, [180.0, 320.0])?;
                    let b = selection_point(&mut f, [204.0, 308.0])?;
                    f.frame(vec![
                        Event::PointerMoved(a),
                        selection_pointer(a, true, false),
                    ]);
                    f.frame(vec![Event::PointerMoved(b)]);
                    selection_authored_unchanged(&f, &before)?;
                    vec![selection_pointer(b, false, false)]
                }
                _ => return Err("undeclared selected movement".into()),
            };
            let tool = selection_tool_activation(&f, "Double bond")?;
            let packet = if tool_first {
                [vec![tool], movement_events].concat()
            } else {
                [movement_events, vec![tool]].concat()
            };
            f.frame(packet);
            f.warm();
            let changed = if tool_first { 0_u64 } else { 1 };
            let expected = if tool_first {
                before["document"].clone()
            } else if movement == "nudge" {
                selection_expected_positions(
                    &before["document"],
                    &[(1, 181.0, 320.0), (2, 261.0, 320.0)],
                )?
            } else {
                selection_expected_positions(
                    &before["document"],
                    &[(1, 204.0, 308.0), (2, 284.0, 308.0)],
                )?
            };
            assert_eq!(
                f.state()?["document"],
                expected,
                "{movement}, tool_first={tool_first}"
            );
            assert_eq!(
                f.state()?["revision"],
                before["revision"].as_u64().ok_or("revision")? + changed
            );
            assert_eq!(f.state()?["undo_depth"], changed);
            assert_eq!(f.state()?["redo_depth"], 0);
            assert_eq!(f.state()?["file"]["dirty"], !tool_first);
            assert_eq!(f.state()?["source"]["modified"], !tool_first);
            assert_eq!(f.state()?["ui_selection"]["active"], false);
            assert_eq!(selection_ids(&f)?, Vec::<u32>::new());
            assert_eq!(f.state()?["ui_selection"]["page_id"], Value::Null);
            assert_eq!(
                serde_json::to_value(f.app.engine.lock().map_err(|_| "engine")?.bond_tool().order)?,
                "double"
            );
            assert!(
                f.app.error.is_none(),
                "{movement}, tool_first={tool_first}: {:?}",
                f.app.error
            );
            assert_eq!(
                fs::read(&path)?,
                raw,
                "tool changes/movement never publish a Save"
            );
        }
    }
    Ok(())
}

#[test]
fn earlier_native_authoring_then_marquee_activation_uses_the_revision_at_that_raw_boundary()
-> TestResult {
    for marquee_first in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = selection_seed(&mut f)?;
        let raw = fs::read(&path)?;
        selection_choose(&mut f, "Single bond")?;
        selection_quiet_clicks(&mut f)?;
        let a = selection_point(&mut f, [180.0, 620.0])?;
        let b = selection_point(&mut f, [260.0, 620.0])?;
        let before = f.state()?;
        f.frame(vec![
            Event::PointerMoved(a),
            selection_pointer(a, true, false),
        ]);
        f.frame(vec![Event::PointerMoved(b)]);
        selection_authored_unchanged(&f, &before)?;
        let marquee = selection_tool_activation(&f, "Marquee selection")?;
        let release = selection_pointer(b, false, false);
        f.frame(if marquee_first {
            vec![marquee, release]
        } else {
            vec![release, marquee]
        });
        f.warm();
        let mut expected = before["document"].clone();
        if !marquee_first {
            expected["pages"][0]["fragments"].as_array_mut().ok_or("fragments")?.push(json!({
                "id":3,"atoms":[
                    {"id":5,"element":6,"position":{"x_pt":180.0,"y_pt":620.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
                    {"id":6,"element":6,"position":{"x_pt":260.0,"y_pt":620.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],
                "bonds":[{"id":3,"begin":5,"end":6,"order":"single"}]
            }));
        }
        assert_eq!(
            f.state()?["document"],
            expected,
            "marquee_first={marquee_first}"
        );
        let changed = if marquee_first { 0_u64 } else { 1 };
        assert_eq!(
            f.state()?["revision"],
            before["revision"].as_u64().ok_or("revision")? + changed
        );
        assert_eq!(f.state()?["undo_depth"], changed);
        assert_eq!(f.state()?["redo_depth"], 0);
        assert_eq!(f.state()?["file"]["dirty"], !marquee_first);
        selection_expect(&f, &[])?;
        assert!(
            f.app.error.is_none(),
            "Marquee must resolve expected_revision after the earlier commit: {:?}",
            f.app.error
        );
        assert_eq!(fs::read(&path)?, raw);
    }
    Ok(())
}

// Independent pointer lifecycle checks, added after the unchanged ten-case declaration.
#[test]
fn a_selected_atom_drag_out_and_back_is_an_exact_zero_translation_noop_not_a_new_click()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = selection_seed(&mut f)?;
    let raw = fs::read(&path)?;
    selection_choose(&mut f, "Marquee selection")?;
    // The preceding target is a Bond, so no multi-click timing assumption is involved.
    selection_click_at(&mut f, [220.0, 320.0], false)?;
    selection_expect(&f, &[1, 2])?;
    let a = selection_point(&mut f, [180.0, 320.0])?;
    let b = selection_point(&mut f, [204.0, 308.0])?;
    assert!(
        a.distance(b) > 6.0,
        "the actual pointer path must cross the SDK click distance"
    );
    let before = f.state()?;
    let (stamp, model) = {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        (
            engine.capture_stamp(),
            engine.stored_document()?.model_snapshot(),
        )
    };
    f.frame(vec![
        Event::PointerMoved(a),
        selection_pointer(a, true, false),
    ]);
    f.frame(vec![Event::PointerMoved(b)]);
    assert_eq!(
        f.state()?,
        before,
        "the nonzero preview is not an authored edit"
    );
    f.frame(vec![Event::PointerMoved(a)]);
    f.frame(vec![selection_pointer(a, false, false)]);
    f.warm();
    selection_expect(&f, &[1, 2])?;
    assert_eq!(
        f.state()?,
        before,
        "a completed zero-displacement drag cannot collapse the original group or advance epochs/history"
    );
    {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        assert_eq!(engine.capture_stamp(), stamp);
        assert!(std::sync::Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    assert_eq!(fs::read(&path)?, raw);
    Ok(())
}

#[test]
fn leaving_the_canvas_cancels_selection_drag_even_when_pointer_returns_before_release() -> TestResult
{
    // PointerGone is a separately labelled control, not a substitute for leaving the canvas
    // while remaining inside the application with an ordinary PointerMoved event.
    for exit in ["outside_canvas", "pointer_gone"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = selection_seed(&mut f)?;
        let raw = fs::read(&path)?;
        selection_choose(&mut f, "Marquee selection")?;
        selection_click_at(&mut f, [220.0, 320.0], false)?;
        selection_expect(&f, &[1, 2])?;
        let a = selection_point(&mut f, [180.0, 320.0])?;
        let b = selection_point(&mut f, [204.0, 308.0])?;
        let area = canvas(&f.warm())?;
        let outside = egui::pos2(area.left() - 12.0, area.center().y);
        assert!(!area.contains(outside));
        assert!(
            egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(f.width, f.height)).contains(outside),
            "ordinary exit control stays inside the App; it must not imply PointerGone"
        );
        let before = f.state()?;
        let (stamp, model) = {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            (
                engine.capture_stamp(),
                engine.stored_document()?.model_snapshot(),
            )
        };
        f.frame(vec![
            Event::PointerMoved(a),
            selection_pointer(a, true, false),
        ]);
        f.frame(vec![Event::PointerMoved(b)]);
        assert_eq!(f.state()?, before);
        match exit {
            "outside_canvas" => {
                f.frame(vec![Event::PointerMoved(outside)]);
            }
            "pointer_gone" => {
                f.frame(vec![Event::PointerGone]);
            }
            _ => return Err("undeclared pointer exit".into()),
        }
        f.frame(vec![Event::PointerMoved(b)]);
        f.frame(vec![selection_pointer(b, false, false)]);
        f.warm();
        selection_expect(&f, &[1, 2])?;
        assert_eq!(
            f.state()?,
            before,
            "{exit}: the old drag cannot revive on re-entry/release"
        );
        {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            assert_eq!(engine.capture_stamp(), stamp);
            assert!(std::sync::Arc::ptr_eq(
                &model,
                &engine.stored_document()?.model_snapshot()
            ));
        }
        assert_eq!(fs::read(&path)?, raw);
    }
    Ok(())
}

#[path = "selection_translation_regressions.rs"]
mod regressions;
