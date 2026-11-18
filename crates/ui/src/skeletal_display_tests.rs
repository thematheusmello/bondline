//! Independent actual-App acceptance for persisted skeletal-carbon label restoration.
//! Adopt as a child of native_atom_label_tests.rs to reuse its unchanged real App helpers.
use super::{Fixture, TestResult, close_form, painted, state_without_form, value, wait};
use serde_json::json;
use std::fs;

// Original literal canonical native XML: no product writer generates an expected input.
// Atom2 is the bent degree2 CH2 center. Terminal CH3 atoms retain the legacy profile.
const HIDDEN_BENT: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="420" Height="280"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="144 144" NumHydrogens="3"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="208 144" NumHydrogens="2" ShowNonTerminalCarbonLabels="no"/><n id="5" NodeType="Element" Element="6" Charge="0" Radical="None" p="240 96" NumHydrogens="3"/><b id="6" B="3" E="4" Order="1"/><b id="7" B="4" E="5" Order="1"/></fragment></page></CDXML>"#;

#[test]
fn unchanged_fields_in_the_actual_atom_editor_restore_the_persisted_false_label_once() -> TestResult
{
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("skeletal-original-炭-λ.cdxml");
    fs::write(&input, HIDDEN_BENT)?;
    f.command("document.open_cdxml", json!({"path":input}))?;
    assert!(
        f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .is_fully_mapped()
    );
    let before = state_without_form(&f)?;
    assert_eq!(
        before["document"]["pages"][0]["fragments"][0]["atoms"][1]["show_non_terminal_carbon_labels"],
        false
    );
    assert_eq!(before["undo_depth"], 0);
    assert_eq!(before["redo_depth"], 0);
    assert_eq!(before["file"]["dirty"], false);
    assert_eq!(before["source"]["modified"], false);
    // Existing exact-target native presentation, not atom.set_label or a new parameter.
    f.command("ui.atom_label", json!({"action":"open","expected_revision":before["revision"],"page_id":1,"fragment_id":1,"atom_id":2}))?;
    wait(&mut f, |f, shown| {
        painted(shown, "Atom label") && value(f, "Element").is_ok()
    })?;
    assert_eq!(value(&f, "Element")?, "C");
    assert_eq!(value(&f, "Attached hydrogens")?, "2");
    assert_eq!(state_without_form(&f)?, before);
    close_form(&mut f, "Apply")?;
    let after = f.state()?;
    let mut expected = before["document"].clone();
    expected["pages"][0]["fragments"][0]["atoms"][1]["show_non_terminal_carbon_labels"] =
        json!(true);
    assert_eq!(
        after["document"], expected,
        "unchanged literal fields must restore false to true through the real form"
    );
    assert_eq!(
        after["revision"],
        before["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(after["undo_depth"], 1);
    assert_eq!(after["redo_depth"], 0);
    assert_eq!(after["file"]["dirty"], true);
    assert_eq!(after["source"]["modified"], true);
    assert!(after["ui_atom_label"].is_null());
    assert_eq!(fs::read(&input)?, HIDDEN_BENT);
    Ok(())
}

// Additive actual hidden-anchor/frame and source-stale cases. The first3,027B stay exact.
use super::{
    apply_enabled, canvas, click_alt, finish_actual_file_workflow,
    independently_read_actual_saved_graph, projected, set, shortcut,
};
use eframe::egui;
use egui::{Event, Key, accesskit::Role};
use std::{collections::HashSet, sync::Arc};

const VISIBLE_BENT: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="420" Height="280"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="144 144" NumHydrogens="3"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="208 144" NumHydrogens="2" ShowNonTerminalCarbonLabels="yes"/><n id="5" NodeType="Element" Element="6" Charge="0" Radical="None" p="240 96" NumHydrogens="3"/><b id="6" B="3" E="4" Order="1"/><b id="7" B="4" E="5" Order="1"/></fragment></page></CDXML>"#;

fn hidden_seed(f: &mut Fixture) -> TestResult<std::path::PathBuf> {
    let input = f.directory.path("hidden-bent-original-炭-λ.cdxml");
    fs::write(&input, HIDDEN_BENT)?;
    f.command("document.open_cdxml", json!({"path":input}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    Ok(input)
}

// Inspect only actual App paint shapes. No direct renderer, generated success or image loader.
fn assert_bent_frame(f: &Fixture, shown: &egui::FullOutput, hidden: bool) -> TestResult {
    let area = canvas(shown)?;
    let anchor = projected(area, [420.0, 280.0], 100.0, None, [208.0, 144.0]);
    let region = egui::Rect::from_center_size(anchor, egui::vec2(48.0, 40.0));
    assert!(area.contains_rect(region));
    let atom_rectangles = shown
        .shapes
        .iter()
        .filter(|paint| {
            matches!(&paint.shape, egui::Shape::Rect(shape)
            if (shape.fill == egui::Color32::BLACK || shape.fill == egui::Color32::WHITE)
                && region.contains_rect(shape.rect)
                && paint.clip_rect.contains_rect(shape.rect))
        })
        .count();
    if hidden {
        assert_eq!(
            atom_rectangles, 0,
            "the hidden center has neither glyph rectangles nor a white label mask"
        );
    } else {
        assert!(
            atom_rectangles > 0,
            "restored CH2 must produce actual glyph/mask rectangles"
        );
    }
    let strokes = shown
        .shapes
        .iter()
        .filter_map(|paint| match &paint.shape {
            egui::Shape::Path(shape)
                if shape.closed
                    && shape.fill == egui::Color32::BLACK
                    && shape.points.len() == 4
                    && shape.points.iter().all(|point| area.contains(*point)) =>
            {
                Some(shape)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        strokes.len(),
        2,
        "the actual canvas must paint both original Single bond rectangles"
    );
    for midpoint in [[176.0, 144.0], [224.0, 120.0]] {
        let midpoint = projected(area, [420.0, 280.0], 100.0, None, midpoint);
        assert!(
            strokes
                .iter()
                .any(|stroke| egui::Rect::from_points(&stroke.points).contains(midpoint)),
            "actual stroke missing at literal original bond midpoint {midpoint:?}"
        );
    }
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}

fn assert_actual_saved_propane(path: &std::path::Path) -> TestResult {
    assert_eq!(fs::read(path)?, VISIBLE_BENT);
    let literal_graph = json!([{"id":1,"atoms":[
        {"id":1,"element":6,"position":{"x_pt":144.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
        {"id":2,"element":6,"position":{"x_pt":208.0,"y_pt":144.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2},
        {"id":3,"element":6,"position":{"x_pt":240.0,"y_pt":96.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],
        "bonds":[{"id":1,"begin":1,"end":2,"order":"single"},{"id":2,"begin":2,"end":3,"order":"single"}]}]);
    // Unchanged Expat graph consumer has its original chemistry-only schema. Exact bytes
    // above independently require the standard yes flag as well as every literal field.
    assert_eq!(independently_read_actual_saved_graph(path)?, literal_graph);
    let result = std::process::Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .args(["-B", "-c", r#"
from pathlib import Path
from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors
import sys
assert rdBase.rdkitVersion == '2026.09.2'
ms = Chem.MolsFromCDXML(Path(sys.argv[1]).read_text(encoding='utf-8'), sanitize=True, removeHs=False)
assert len(ms) == 1
m = ms[0]
assert [[a.GetAtomicNum(), a.GetTotalNumHs(), a.GetFormalCharge(), a.GetIsotope(), a.GetNumRadicalElectrons()] for a in m.GetAtoms()] == [[6,3,0,0,0],[6,2,0,0,0],[6,3,0,0,0]]
assert [(b.GetBeginAtomIdx(), b.GetEndAtomIdx(), b.GetBondTypeAsDouble()) for b in m.GetBonds()] == [(0,1,1.0),(1,2,1.0)]
assert Chem.MolToSmiles(m) == 'CCC'
assert rdMolDescriptors.CalcMolFormula(m) == 'C3H8'
"#]).arg(path).output()?;
    assert!(
        result.status.success(),
        "independent actual-file chemistry: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn hidden_bent_anchor_alt_target_cancel_restore_history_and_unicode_save_use_actual_app_frames()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    let input = hidden_seed(&mut f)?;
    let original = state_without_form(&f)?;
    let shown = f.warm();
    assert_bent_frame(&f, &shown, true)?;
    let anchor = projected(canvas(&shown)?, [420.0, 280.0], 100.0, None, [208.0, 144.0]);
    f.frame(vec![Event::PointerMoved(anchor)]);
    assert!(
        matches!(f.app.drawing.hover_target(), Some(bondline_render::HitTarget::Atom { fragment_id, atom_id })
        if fragment_id.get() == 1 && atom_id.get() == 2),
        "a hidden center retains the actual stable atom pick target"
    );
    assert_eq!(state_without_form(&f)?, original);

    click_alt(&mut f, [208.0, 144.0])?;
    wait(&mut f, |f, shown| {
        painted(shown, "Atom label") && value(f, "Element").is_ok()
    })?;
    assert_eq!(value(&f, "Element")?, "C");
    assert_eq!(value(&f, "Attached hydrogens")?, "2");
    assert!(apply_enabled(&f)?);
    close_form(&mut f, "Cancel")?;
    assert_eq!(state_without_form(&f)?, original);
    assert_eq!(fs::read(&input)?, HIDDEN_BENT);
    let shown = f.warm();
    assert_bent_frame(&f, &shown, true)?;

    click_alt(&mut f, [208.0, 144.0])?;
    wait(&mut f, |f, shown| {
        painted(shown, "Atom label") && value(f, "Element").is_ok()
    })?;
    close_form(&mut f, "Apply")?;
    let mut restored = original["document"].clone();
    restored["pages"][0]["fragments"][0]["atoms"][1]["show_non_terminal_carbon_labels"] =
        json!(true);
    assert_eq!(f.state()?["document"], restored);
    assert_eq!(
        f.state()?["revision"],
        original["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(f.state()?["redo_depth"], 0);
    assert_eq!(f.state()?["source"]["modified"], true);
    assert_eq!(f.state()?["file"]["dirty"], true);
    let shown = f.warm();
    assert_bent_frame(&f, &shown, false)?;
    shortcut(&mut f, Key::Z, false);
    assert_eq!(f.state()?["document"], original["document"]);
    assert_eq!(f.state()?["file"]["dirty"], false);
    let shown = f.warm();
    assert_bent_frame(&f, &shown, true)?;
    shortcut(&mut f, Key::Z, true);
    assert_eq!(f.state()?["document"], restored);
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(f.state()?["redo_depth"], 0);

    let output = f.directory.path("restored-skeletal-化学-λ.cdxml");
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
    assert_eq!(f.state()?["document"], restored);
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(f.state()?["source"]["modified"], true);
    assert_actual_saved_propane(&output)?;
    assert_eq!(fs::read(&input)?, HIDDEN_BENT);
    shortcut(&mut f, Key::O, false);
    wait(&mut f, |f, _| {
        f.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut f, "File name", output.to_str().ok_or("output")?)?;
    f.choose("Open")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(f.state()?["document"], restored);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["redo_depth"], 0);
    assert_eq!(f.state()?["source"]["modified"], false);
    assert_eq!(f.state()?["file"]["dirty"], false);
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    let no_op = state_without_form(&f)?;
    let owner = f
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .stored_document()?
        .model_snapshot();
    click_alt(&mut f, [208.0, 144.0])?;
    wait(&mut f, |f, shown| {
        painted(shown, "Atom label") && value(f, "Element").is_ok()
    })?;
    close_form(&mut f, "Apply")?;
    assert_eq!(
        state_without_form(&f)?,
        no_op,
        "already-visible same-count Apply remains a real no-op"
    );
    assert!(Arc::ptr_eq(
        &owner,
        &f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .model_snapshot()
    ));
    assert_actual_saved_propane(&output)?;
    Ok(())
}

fn disabled_current_apply(f: &Fixture) -> TestResult<egui::accesskit::NodeId> {
    let mut stack = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    let mut seen = HashSet::new();
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = f.nodes.get(&id)
        {
            if node.role() == Role::Button && node.name() == Some("Apply") {
                assert!(
                    node.is_disabled(),
                    "the stale target cannot expose an enabled Apply"
                );
                return Ok(id);
            }
            stack.extend_from_slice(node.children());
        }
    }
    Err("current disabled Apply button missing".into())
}

#[test]
fn a_same_bytes_source_reopen_detaches_the_hidden_anchor_form_without_restoring_an_old_target()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = hidden_seed(&mut f)?;
    click_alt(&mut f, [208.0, 144.0])?;
    wait(&mut f, |f, shown| {
        painted(shown, "Atom label") && value(f, "Element").is_ok()
    })?;
    let old_revision = f.state()?["revision"].as_u64().ok_or("revision")?;
    f.command("document.open_cdxml", json!({"path":input}))?;
    let current = state_without_form(&f)?;
    assert_eq!(current["revision"], old_revision + 1);
    let owner = f
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .stored_document()?
        .model_snapshot();
    let shown = f.warm();
    assert!(painted(&shown, "Atom label"));
    assert_eq!(value(&f, "Element")?, "C");
    assert_eq!(value(&f, "Attached hydrogens")?, "2");
    assert!(!apply_enabled(&f)?);
    assert_eq!(f.state()?["ui_atom_label"]["current"], false);
    let disabled = disabled_current_apply(&f)?;
    f.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Default,
            target: disabled,
            data: None,
        },
    )]);
    assert_eq!(state_without_form(&f)?, current);
    assert!(Arc::ptr_eq(
        &owner,
        &f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .model_snapshot()
    ));
    close_form(&mut f, "Cancel")?;
    assert_eq!(state_without_form(&f)?, current);
    assert_eq!(
        current["document"]["pages"][0]["fragments"][0]["atoms"][1]["show_non_terminal_carbon_labels"],
        false
    );
    assert_eq!(current["undo_depth"], 0);
    assert_eq!(current["redo_depth"], 0);
    assert_eq!(current["file"]["dirty"], false);
    assert_eq!(current["source"]["modified"], false);
    assert_eq!(fs::read(&input)?, HIDDEN_BENT);
    // A new real gesture can target the replacement only after explicit stale-form Cancel.
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    click_alt(&mut f, [208.0, 144.0])?;
    wait(&mut f, |f, shown| {
        painted(shown, "Atom label") && value(f, "Element").is_ok()
    })?;
    close_form(&mut f, "Apply")?;
    let mut expected = current["document"].clone();
    expected["pages"][0]["fragments"][0]["atoms"][1]["show_non_terminal_carbon_labels"] =
        json!(true);
    assert_eq!(f.state()?["document"], expected);
    assert_eq!(
        f.state()?["revision"],
        current["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(f.state()?["undo_depth"], 1);
    assert_eq!(fs::read(&input)?, HIDDEN_BENT);
    Ok(())
}

// Additive all-glyphless native App control; first15,290B and all earlier assertions stay exact.
const HIDDEN_SQUARE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0"><page id="1" DrawingSpace="Poster" Width="420" Height="280"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="160 96" NumHydrogens="2" ShowNonTerminalCarbonLabels="no"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="224 96" NumHydrogens="2" ShowNonTerminalCarbonLabels="no"/><n id="5" NodeType="Element" Element="6" Charge="0" Radical="None" p="224 160" NumHydrogens="2" ShowNonTerminalCarbonLabels="no"/><n id="6" NodeType="Element" Element="6" Charge="0" Radical="None" p="160 160" NumHydrogens="2" ShowNonTerminalCarbonLabels="no"/><b id="7" B="3" E="4" Order="1"/><b id="8" B="4" E="5" Order="1"/><b id="9" B="5" E="6" Order="1"/><b id="10" B="6" E="3" Order="1"/></fragment></page></CDXML>"#;

#[test]
fn entirely_glyphless_square_retains_four_real_bonds_and_corner_picking_at_actual_dpi2()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    let input = f.directory.path("all-hidden-square-炭-λ.cdxml");
    fs::write(&input, HIDDEN_SQUARE)?;
    f.command("document.open_cdxml", json!({"path":input}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    let before = state_without_form(&f)?;
    let (owner, stamp) = {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        assert!(engine.stored_document()?.is_fully_mapped());
        (
            engine.stored_document()?.model_snapshot(),
            engine.capture_stamp(),
        )
    };
    assert_eq!(
        before["document"]["pages"][0]["fragments"][0]["atoms"],
        json!([
        {"id":1,"element":6,"position":{"x_pt":160.0,"y_pt":96.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2,"show_non_terminal_carbon_labels":false},
        {"id":2,"element":6,"position":{"x_pt":224.0,"y_pt":96.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2,"show_non_terminal_carbon_labels":false},
        {"id":3,"element":6,"position":{"x_pt":224.0,"y_pt":160.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2,"show_non_terminal_carbon_labels":false},
        {"id":4,"element":6,"position":{"x_pt":160.0,"y_pt":160.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":2,"show_non_terminal_carbon_labels":false}])
    );
    assert_eq!(
        before["document"]["pages"][0]["fragments"][0]["bonds"],
        json!([
        {"id":1,"begin":1,"end":2,"order":"single"},
        {"id":2,"begin":2,"end":3,"order":"single"},
        {"id":3,"begin":3,"end":4,"order":"single"},
        {"id":4,"begin":4,"end":1,"order":"single"}])
    );
    let shown = f.warm();
    assert_eq!(f.view()?["zoom_percent"], 100.0);
    let area = canvas(&shown)?;
    let paths = shown
        .shapes
        .iter()
        .filter_map(|paint| match &paint.shape {
            egui::Shape::Path(shape)
                if shape.closed
                    && shape.fill == egui::Color32::BLACK
                    && shape.points.len() == 4
                    && shape.points.iter().all(|point| area.contains(*point)) =>
            {
                Some(shape)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        paths.len(),
        4,
        "glyphless is a rendered square, not an error/blank canvas"
    );
    for midpoint in [
        [192.0, 96.0],
        [224.0, 128.0],
        [192.0, 160.0],
        [160.0, 128.0],
    ] {
        let point = projected(area, [420.0, 280.0], 100.0, None, midpoint);
        assert!(
            paths
                .iter()
                .any(|path| egui::Rect::from_points(&path.points).contains(point)),
            "actual square edge missing at literal midpoint {midpoint:?}"
        );
    }
    for corner in [[160.0, 96.0], [224.0, 96.0], [224.0, 160.0], [160.0, 160.0]] {
        let point = projected(area, [420.0, 280.0], 100.0, None, corner);
        let region = egui::Rect::from_center_size(point, egui::vec2(48.0, 40.0));
        assert!(area.contains_rect(region));
        assert!(!shown.shapes.iter().any(|paint| matches!(&paint.shape,
            egui::Shape::Rect(shape) if (shape.fill == egui::Color32::BLACK || shape.fill == egui::Color32::WHITE)
                && region.contains_rect(shape.rect) && paint.clip_rect.contains_rect(shape.rect))),
            "an all-hidden corner has no chemical glyph or white label/supplement mask");
    }
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    let first = projected(area, [420.0, 280.0], 100.0, None, [160.0, 96.0]);
    f.frame(vec![Event::PointerMoved(first)]);
    assert!(
        matches!(f.app.drawing.hover_target(), Some(bondline_render::HitTarget::Atom { fragment_id, atom_id })
        if fragment_id.get() == 1 && atom_id.get() == 1),
        "glyphless square still has its stable corner atom target"
    );
    assert_eq!(state_without_form(&f)?, before);
    let engine = f.app.engine.lock().map_err(|_| "engine")?;
    assert!(engine.capture_stamp_matches(&stamp));
    assert!(Arc::ptr_eq(
        &owner,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(
        engine.stored_document()?.to_cdxml()?.as_ref(),
        HIDDEN_SQUARE
    );
    assert_eq!(fs::read(&input)?, HIDDEN_SQUARE);
    Ok(())
}
