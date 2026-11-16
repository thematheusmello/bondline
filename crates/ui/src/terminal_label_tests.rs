//! Actual App hidden-terminal authoring; child of native_atom_label_tests.rs.
use super::{
    Fixture, TestResult, canvas, click_alt, close_form, finish_actual_file_workflow, painted,
    projected, set, shortcut, state_without_form, value, wait,
};
use eframe::egui;
use egui::{Event, Key, accesskit::Role};
use serde_json::json;
use std::fs;
#[path = "../../../xtask/oracles/terminal_label_test_support.rs"]
#[allow(
    dead_code,
    reason = "share original literal bytes/model and unchanged existing chemical consumer"
)]
mod literal;

fn current_button(f: &Fixture, name: &str) -> bool {
    f.node(name, &[Role::Button]).is_ok_and(|(id, rect)| {
        rect.is_positive()
            && f.nodes.get(&id).is_some_and(|node| {
                !node.is_disabled()
                    && node.default_action_verb() == Some(egui::accesskit::DefaultActionVerb::Click)
            })
    })
}
fn current_field(f: &Fixture, name: &str) -> bool {
    f.node(name, &[Role::TextInput]).is_ok_and(|(id, rect)| {
        rect.is_positive() && f.nodes.get(&id).is_some_and(|node| !node.is_disabled())
    })
}
fn endpoint_paint(f: &Fixture, shown: &egui::FullOutput, visible: bool) -> TestResult {
    let area = canvas(shown)?;
    let at = projected(area, [420.0, 280.0], 100.0, None, [144.0, 144.0]);
    let region = egui::Rect::from_center_size(at, egui::vec2(48.0, 40.0));
    assert!(area.contains_rect(region));
    let labels = shown
        .shapes
        .iter()
        .filter(|paint| {
            matches!(&paint.shape,egui::Shape::Rect(r)
        if (r.fill==egui::Color32::BLACK||r.fill==egui::Color32::WHITE)
            &&region.contains_rect(r.rect)&&paint.clip_rect.contains_rect(r.rect))
        })
        .count();
    if visible {
        assert!(
            labels > 0,
            "restored endpoint must paint its full literal CH3 glyph/mask"
        );
    } else {
        assert_eq!(
            labels, 0,
            "hidden endpoint must retain neither glyph nor white mask"
        );
    }
    let strokes = shown
        .shapes
        .iter()
        .filter_map(|paint| match &paint.shape {
            egui::Shape::Path(p)
                if p.closed
                    && p.fill == egui::Color32::BLACK
                    && p.points.len() == 4
                    && p.points.iter().all(|p| area.contains(*p)) =>
            {
                Some(p)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(strokes.len(), 2);
    for p in [[176.0, 144.0], [224.0, 120.0]] {
        let p = projected(area, [420.0, 280.0], 100.0, None, p);
        assert!(
            strokes
                .iter()
                .any(|s| egui::Rect::from_points(&s.points).contains(p))
        );
    }
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}

#[test]
fn hidden_terminal_alt_target_cancel_same_fields_apply_history_and_unicode_file_keep_both_properties_and_chemistry()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    let input = f.directory.path("terminal-app-original-炭-λ.cdxml");
    fs::write(&input, literal::HIDDEN)?;
    f.command("document.open_cdxml", json!({"path":input}))?;
    assert!(
        f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .is_fully_mapped()
    );
    f.command("ui.view", json!({"action":"actual"}))?;
    let shown = f.warm();
    let before = state_without_form(&f)?;
    assert_eq!(
        before["document"]["pages"][0]["fragments"],
        json!([literal::fragment(&literal::ALL_FALSE)])
    );
    endpoint_paint(&f, &shown, false)?;
    let anchor = projected(canvas(&shown)?, [420.0, 280.0], 100.0, None, [144.0, 144.0]);
    f.frame(vec![Event::PointerMoved(anchor)]);
    assert!(
        matches!(f.app.drawing.hover_target(),Some(bondline_render::HitTarget::Atom{fragment_id,atom_id})
        if fragment_id.get()==1&&atom_id.get()==1)
    );
    click_alt(&mut f, [144.0, 144.0])?;
    wait(&mut f, |f, out| {
        painted(out, "Atom label")
            && current_field(f, "Element")
            && current_field(f, "Attached hydrogens")
            && current_button(f, "Cancel")
            && current_button(f, "Apply")
    })?;
    assert_eq!(value(&f, "Element")?, "C");
    assert_eq!(value(&f, "Attached hydrogens")?, "3");
    assert_eq!(state_without_form(&f)?, before);
    close_form(&mut f, "Cancel")?;
    assert_eq!(state_without_form(&f)?, before);
    assert_eq!(fs::read(&input)?, literal::HIDDEN);
    click_alt(&mut f, [144.0, 144.0])?;
    wait(&mut f, |f, out| {
        painted(out, "Atom label") && current_button(f, "Apply") && current_button(f, "Cancel")
    })?;
    assert_eq!(value(&f, "Element")?, "C");
    assert_eq!(value(&f, "Attached hydrogens")?, "3");
    close_form(&mut f, "Apply")?;
    let mut restored = literal::ALL_FALSE;
    restored[0] = (Some(true), Some(true));
    let after = state_without_form(&f)?;
    assert_eq!(
        after["document"]["pages"][0]["fragments"],
        json!([literal::fragment(&restored)])
    );
    assert_eq!(
        after["revision"],
        before["revision"].as_u64().ok_or("revision")? + 1
    );
    assert_eq!(after["undo_depth"], 1);
    assert_eq!(after["redo_depth"], 0);
    assert_eq!(after["source"]["modified"], true);
    assert_eq!(after["file"]["dirty"], true);
    let shown = f.warm();
    endpoint_paint(&f, &shown, true)?;
    click_alt(&mut f, [144.0, 144.0])?;
    wait(&mut f, |f, out| {
        painted(out, "Atom label") && current_button(f, "Apply")
    })?;
    close_form(&mut f, "Apply")?;
    assert_eq!(
        state_without_form(&f)?,
        after,
        "repeated true fields are an exact validated no-op"
    );
    shortcut(&mut f, Key::Z, false);
    assert_eq!(f.state()?["document"], before["document"]);
    assert_eq!(f.state()?["file"]["dirty"], false);
    let shown = f.warm();
    endpoint_paint(&f, &shown, false)?;
    shortcut(&mut f, Key::Z, true);
    assert_eq!(f.state()?["document"], after["document"]);
    assert_eq!(f.state()?["undo_depth"], 1);
    let output = f.directory.path("terminal-app-restored-化学-λ.cdxml");
    shortcut(&mut f, Key::S, true);
    wait(&mut f, |f, _| {
        current_field(f, "File name") && current_button(f, "Save")
    })?;
    set(
        &mut f,
        "File name",
        output.to_str().ok_or("Unicode output")?,
    )?;
    f.choose("Save")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(f.state()?["file"]["dirty"], false);
    literal::consume(&output, &restored)?;
    assert_eq!(fs::read(&input)?, literal::HIDDEN);
    shortcut(&mut f, Key::O, false);
    wait(&mut f, |f, _| {
        current_field(f, "File name") && current_button(f, "Open")
    })?;
    set(
        &mut f,
        "File name",
        output.to_str().ok_or("Unicode output")?,
    )?;
    f.choose("Open")?;
    finish_actual_file_workflow(&mut f)?;
    assert_eq!(f.state()?["document"], after["document"]);
    assert_eq!(f.state()?["undo_depth"], 0);
    assert_eq!(f.state()?["file"]["dirty"], false);
    assert_eq!(f.state()?["source"]["modified"], false);
    assert_eq!(
        f.state()?["file"]["path"],
        json!(fs::canonicalize(&output)?)
    );
    literal::consume(&output, &restored)?;
    Ok(())
}
