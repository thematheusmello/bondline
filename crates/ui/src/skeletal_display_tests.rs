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

#[test]
fn skeletal_cdx_opens_the_real_poster_and_preserves_its_raw_hydrogen_fields() -> TestResult {
    let choose_file = |f: &mut Fixture, path: &std::path::Path, action: &str| -> TestResult {
        wait(f, |f, _| f.node("File name", &[Role::TextInput]).is_ok())?;
        set(f, "File name", path.to_str().ok_or("Unicode path")?)?;
        f.choose(action)?;
        finish_actual_file_workflow(f)
    };
    for (source, hydrogens, text_selected) in [
        ("propane", json!([3, 2, 3]), false),
        ("missing_h", json!([3, null, 3]), true),
        ("all_h_missing", json!([null, null, null]), false),
        ("dormant_true", json!([3, 2, 3]), true),
    ] {
        let raw = fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../io/tests/fixtures/cdx-skeletal")
                .join(format!("{source}.cdx")),
        )?;
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = f.directory.path("skeletal-原.cdx");
        let output = f.directory.path("skeletal-保存-β.cdx");
        fs::write(&path, &raw)?;
        f.warm();
        if text_selected {
            f.choose("Text")?;
        } else {
            f.command("ui.ring_tool", json!({"size":8}))?;
        }
        shortcut(&mut f, Key::O, false);
        choose_file(&mut f, &path, "Open")?;
        f.command("ui.view", json!({"action":"actual"}))?;
        let shown = f.warm();
        let opened = f.state()?;
        let page = &opened["document"]["pages"][0];
        assert_eq!(page["size"], json!({"width_pt":512.0,"height_pt":512.0}));
        let atoms = page["fragments"][0]["atoms"].as_array().ok_or("atoms")?;
        let actual_h = atoms
            .iter()
            .map(|a| a["explicit_hydrogens"].clone())
            .collect::<Vec<_>>();
        assert_eq!(json!(actual_h), hydrogens);
        {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            let controls = engine.controls();
            for id in [
                "tool.bond.single",
                "page.rotate",
                "page.a4",
                "page.letter",
                "file.document_settings",
            ] {
                let control = controls.iter().find(|c| c.id == id).ok_or("control")?;
                assert!(!control.enabled, "{id}");
                assert_eq!(control.disabled_reason, Some("unsupported_content"), "{id}");
            }
            assert!(!controls.iter().any(|c| c.enabled && c.id == "tool.caption"));
        }
        assert!(
            f.app
                .engine
                .lock()
                .map_err(|_| "engine")?
                .controls()
                .iter()
                .any(|c| c.id == "tool.selection" && c.enabled)
        );
        assert!(painted(&shown, "Skeletal CDX preview"));
        let paper = canvas(&shown)?;
        let paths = shown
            .shapes
            .iter()
            .filter_map(|paint| match &paint.shape {
                egui::Shape::Path(path)
                    if path.closed
                        && path.fill == egui::Color32::BLACK
                        && path.points.len() == 4
                        && path.points.iter().all(|p| paper.contains(*p)) =>
                {
                    Some(path)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 2);
        for midpoint in [[192., 216.], [256., 216.]] {
            let point = projected(paper, [512., 512.], 100., None, midpoint);
            assert!(
                paths
                    .iter()
                    .any(|p| egui::Rect::from_points(&p.points).contains(point))
            );
        }
        for (start, end) in [
            ([160., 240.], [160., 240.]),
            ([192., 216.], [208., 232.]),
            ([300., 300.], [330., 330.]),
        ] {
            let start = projected(paper, [512., 512.], 100., None, start);
            let end = projected(paper, [512., 512.], 100., None, end);
            f.frame(vec![Event::PointerMoved(start), super::button(start, true)]);
            f.frame(vec![Event::PointerMoved(end), super::button(end, false)]);
        }
        click_alt(&mut f, [224., 192.])?;
        assert_eq!(f.state()?, opened);
        assert!(f.app.error.is_none(), "{:?}", f.app.error);
        shortcut(&mut f, Key::S, true);
        choose_file(&mut f, &output, "Save")?;
        assert_eq!(fs::read(&output)?, raw);
        assert_eq!(fs::read(&path)?, raw);
        shortcut(&mut f, Key::O, false);
        choose_file(&mut f, &output, "Open")?;
        let reopened = f.state()?;
        assert_eq!(reopened["document"], opened["document"]);
        assert_eq!(reopened["file"]["dirty"], false);
        assert_eq!(reopened["undo_depth"], 0);
    }
    Ok(())
}

#[test]
fn skeletal_selection_drag_previews_the_moved_ink_and_commits_one_exact_edit() -> TestResult {
    let raw = include_bytes!("../../io/tests/fixtures/cdx-skeletal/all_h_missing.cdx");
    let after =
        include_bytes!("../../io/tests/fixtures/cdx-skeletal-move/all_h_missing_center.cdx");
    let mut f = Fixture::new(1160., 800., 1.)?;
    let input = f.directory.path("selected-原.cdx");
    let output = f.directory.path("moved-保存.cdx");
    fs::write(&input, raw)?;
    f.command("document.open_cdx", json!({"path":input}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    f.warm();
    f.choose("Marquee selection")?;
    let paper = canvas(&f.warm())?;
    let to_screen = |p| projected(paper, [512., 512.], 100., None, p);
    let center = to_screen([224., 192.]);
    let shifted = to_screen([232.25, 186.875]);
    f.frame(vec![
        Event::PointerMoved(center),
        super::button(center, true),
    ]);
    f.frame(vec![super::button(center, false)]);
    assert_eq!(f.state()?["ui_selection"]["atom_ids"], json!([2]));
    let original = f.state()?;
    f.frame(vec![super::button(center, true)]);
    let preview = f.frame(vec![Event::PointerMoved(shifted)]);
    assert_eq!(f.state()?, original);
    // The real uncommitted frame must use the missing-H skeletal source facts.
    let centroids = preview
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Path(p)
                if p.closed
                    && p.fill == egui::Color32::BLACK
                    && p.points.len() == 4
                    && p.points.iter().all(|p| paper.contains(*p)) =>
            {
                Some(egui::Rect::from_points(&p.points).center())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(centroids.len(), 2);
    for midpoint in [[196.125, 213.4375], [260.125, 213.4375]] {
        assert!(
            centroids
                .iter()
                .any(|p| p.distance(to_screen(midpoint)) < 0.25)
        );
    }
    f.frame(vec![super::button(shifted, false)]);
    let bytes = |f: &Fixture| -> TestResult<Vec<u8>> {
        Ok(f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .to_cdx()?
            .into_owned())
    };
    assert_eq!(bytes(&f)?, after);
    assert_eq!(f.state()?["undo_depth"], 1);
    f.frame(vec![
        super::key(Key::ArrowRight, true, egui::Modifiers::NONE),
        super::key(Key::ArrowRight, false, egui::Modifiers::NONE),
    ]);
    assert_eq!(
        f.state()?["document"]["pages"][0]["fragments"][0]["atoms"][1]["position"]["x_pt"],
        233.25
    );
    shortcut(&mut f, Key::Z, false);
    assert_eq!(bytes(&f)?, after);
    shortcut(&mut f, Key::Z, false);
    assert_eq!(bytes(&f)?, raw);
    shortcut(&mut f, Key::Z, true);
    shortcut(&mut f, Key::S, true);
    wait(&mut f, |f, _| {
        f.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut f, "File name", output.to_str().ok_or("path")?)?;
    f.choose("Save")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(fs::read(&input)?, raw);
    assert_eq!(fs::read(&output)?, after);
    f.command("document.open_cdx", json!({"path":output}))?;
    assert_eq!(bytes(&f)?, after);
    assert_eq!(f.state()?["file"]["dirty"], false);
    Ok(())
}

#[test]
fn mixed_skeletal_binary_shows_printed_paper_without_a_canvas_author() -> TestResult {
    let raw = include_bytes!("../../io/tests/fixtures/cdx-mixed-skeleton/ethanol.cdx");
    let mut f = Fixture::new(1160., 800., 1.)?;
    let path = f.directory.path("éthanol-原.cdx");
    let saved = f.directory.path("éthanol-保存-β.cdx");
    fs::write(&path, raw)?;
    f.warm();
    f.choose("Marquee selection")?;
    f.command("document.open_cdx", json!({"path":path}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    let shown = f.warm();
    assert!(painted(&shown, "Skeletal CDX with printed labels"));
    let area = canvas(&shown)?;
    let opened = f.state()?;
    {
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        assert!(
            !engine
                .controls()
                .iter()
                .any(|c| c.id == "tool.selection" && c.enabled)
        );
        for id in [
            "tool.bond.single",
            "page.rotate",
            "page.a4",
            "page.letter",
            "file.document_settings",
        ] {
            assert!(
                !engine
                    .controls()
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or("control")?
                    .enabled,
                "{id}"
            );
        }
        assert!(!engine.caption_creation_available() && !engine.selection_available());
    }
    for paper in [[56., 144.], [128., 192.], [205., 144.], [310., 290.]] {
        let start = projected(area, [512., 512.], 100., None, paper);
        let end = start + egui::vec2(8., 12.);
        f.frame(vec![Event::PointerMoved(start), super::button(start, true)]);
        f.frame(vec![Event::PointerMoved(end), super::button(end, false)]);
        click_alt(&mut f, paper)?;
    }
    assert_eq!(f.state()?, opened);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    shortcut(&mut f, Key::S, true);
    wait(&mut f, |f, _| {
        f.node("File name", &[Role::TextInput]).is_ok()
    })?;
    set(&mut f, "File name", saved.to_str().ok_or("path")?)?;
    f.choose("Save")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(fs::read(&saved)?, raw);
    f.command("document.open_cdx", json!({"path":saved}))?;
    assert_eq!(f.state()?["document"], opened["document"]);
    Ok(())
}
