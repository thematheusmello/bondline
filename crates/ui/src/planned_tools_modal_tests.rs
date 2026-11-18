//! Actual App modal priority; child of view_tests.rs, using its unchanged Fixture.
use super::{Fixture, TestResult, button, canvas, key, tap};
use eframe::egui;
use egui::{
    Event, Key, Modifiers,
    accesskit::{Action, ActionData, ActionRequest, NodeId, Role},
};
use serde_json::{Value, json};
use std::{collections::HashSet, fs};

const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

fn activate(id: NodeId) -> Event {
    Event::AccessKitActionRequest(ActionRequest {
        action: Action::Default,
        target: id,
        data: None,
    })
}
fn current_button(f: &Fixture, label: &str) -> TestResult<(NodeId, egui::Rect)> {
    let (id, rect) = f.node(label, &[Role::Button, Role::CheckBox])?;
    let node = f.nodes.get(&id).ok_or("current button")?;
    assert!(
        rect.is_positive() && !node.is_disabled(),
        "enabled current {label}"
    );
    assert_eq!(
        node.default_action_verb(),
        Some(egui::accesskit::DefaultActionVerb::Click)
    );
    Ok((id, rect))
}
fn set(f: &mut Fixture, label: &str, value: &str) -> TestResult {
    let (id, _) = f.node(label, &[Role::TextInput])?;
    assert!(!f.nodes.get(&id).ok_or("field")?.is_disabled());
    f.frame(vec![Event::AccessKitActionRequest(ActionRequest {
        action: Action::SetValue,
        target: id,
        data: Some(ActionData::Value(value.into())),
    })]);
    f.warm();
    let (id, _) = f.node(label, &[Role::TextInput])?;
    assert_eq!(f.nodes.get(&id).and_then(|node| node.value()), Some(value));
    Ok(())
}
fn search_is_reachable(f: &Fixture) -> bool {
    let mut seen = HashSet::new();
    let mut stack = vec![NodeId(egui::accesskit_root_id().value())];
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = f.nodes.get(&id)
        {
            let named = node.name() == Some("Search tools")
                || node.labelled_by().iter().any(|id| {
                    f.nodes
                        .get(id)
                        .is_some_and(|label| label.name() == Some("Search tools"))
                });
            if node.role() == Role::TextInput && named {
                return true;
            }
            stack.extend_from_slice(node.children());
        }
    }
    false
}
fn authored(f: &Fixture) -> TestResult<Value> {
    let mut state = f.state()?;
    let object = state.as_object_mut().ok_or("state object")?;
    object.remove("ui_file");
    object.remove("ui_atom_label");
    Ok(state)
}

#[test]
fn existing_file_and_atom_forms_hide_catalogue_own_escape_and_preserve_selection_source_and_drafts()
-> TestResult {
    for form in ["file", "atom"] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        let path = f.directory.path("catalogue-modal-original-λ.cdxml");
        fs::write(&path, NATIVE)?;
        f.command("document.open_cdxml", json!({"path":path}))?;
        // Existing authored history makes a leaked Undo observable.
        f.command(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":380,
            "end_x_pt":300,"end_y_pt":380}),
        )?;
        f.command("ui.view", json!({"action":"actual"}))?;
        let state = f.state()?;
        let fragment = state["document"]["pages"][0]["fragments"][0]
            .as_object()
            .ok_or("fragment")?;
        let fragment_id = fragment
            .get("id")
            .and_then(Value::as_u64)
            .ok_or("fragment ID")?;
        let atom_id = fragment
            .get("atoms")
            .and_then(Value::as_array)
            .and_then(|atoms| atoms.first())
            .and_then(|atom| atom["id"].as_u64())
            .ok_or("atom ID")?;
        f.command(
            "ui.selection",
            json!({"action":"set","expected_revision":state["revision"],
            "page_id":1,"atom_ids":[atom_id]}),
        )?;
        f.warm();
        let before = authored(&f)?;
        assert_eq!(before["undo_depth"], 1);
        assert_eq!(before["ui_selection"]["atom_ids"], json!([atom_id]));
        let content = f.app.engine.lock().map_err(|_| "engine")?.content().clone();
        let (tool, mode) = {
            let engine = f.app.engine.lock().map_err(|_| "engine")?;
            (engine.bond_tool(), engine.effective_tool())
        };
        let (all_tools, old_bounds) = current_button(&f, "All tools…")?;
        f.frame(vec![activate(all_tools)]);
        f.warm();
        let (search, _) = f.node("Search tools", &[Role::TextInput])?;
        assert!(!f.nodes.get(&search).ok_or("search")?.is_disabled());
        assert!(search_is_reachable(&f));
        if form == "file" {
            f.command(
                "ui.file",
                json!({"action":"open","expected_revision":before["revision"]}),
            )?;
        } else {
            f.command(
                "ui.atom_label",
                json!({"action":"open","expected_revision":before["revision"],
                "page_id":1,"fragment_id":fragment_id,"atom_id":atom_id}),
            )?;
        }
        f.warm();
        current_button(&f, "Cancel")?;
        assert!(
            !search_is_reachable(&f),
            "authoritative {form} form must hide the catalogue"
        );
        let (field, draft) = if form == "file" {
            ("File name", "keep-this-name.cdxml")
        } else {
            ("Element", "Cl")
        };
        set(&mut f, field, draft)?;
        assert_eq!(authored(&f)?, before);
        let error = f.app.error.clone();
        let (cancel, _) = current_button(&f, "Cancel")?;
        f.frame(vec![Event::AccessKitActionRequest(ActionRequest {
            action: Action::Focus,
            target: cancel,
            data: None,
        })]);
        let shown = f.warm();
        let surface = canvas(&shown)?;
        let start = egui::pos2(surface.left() + 24.0, surface.bottom() - 24.0);
        let end = start + egui::vec2(48.0, 0.0);
        // Old presentation activation and ordinary background commands have no authority.
        let mut packet = vec![
            activate(all_tools),
            Event::PointerMoved(old_bounds.center()),
            button(old_bounds.center(), true),
            button(old_bounds.center(), false),
            Event::PointerMoved(start),
            button(start, true),
            Event::PointerMoved(end),
            button(end, false),
        ];
        for code in [Key::Z, Key::S] {
            packet.extend([
                key(code, true, Modifiers::COMMAND),
                key(code, false, Modifiers::COMMAND),
            ]);
        }
        packet.extend(tap(Key::ArrowRight));
        f.frame(packet);
        f.warm();
        current_button(&f, "Cancel")?;
        assert!(!search_is_reachable(&f));
        let (id, _) = f.node(field, &[Role::TextInput])?;
        assert_eq!(f.nodes.get(&id).and_then(|node| node.value()), Some(draft));
        assert_eq!(authored(&f)?, before);
        assert_eq!(f.app.error, error);
        f.frame(tap(Key::Escape));
        f.warm();
        assert!(
            f.node(field, &[Role::TextInput]).is_err(),
            "Escape belongs to the active {form} form"
        );
        assert_eq!(f.state()?["ui_file"]["request"], Value::Null);
        assert_eq!(f.state()?["ui_atom_label"], Value::Null);
        assert_eq!(
            authored(&f)?,
            before,
            "Escape must not clear the existing selection"
        );
        assert!(!f.app.files.is_busy());
        let engine = f.app.engine.lock().map_err(|_| "engine")?;
        assert_eq!(engine.content(), &content);
        assert_eq!(engine.bond_tool(), tool);
        assert_eq!(engine.effective_tool(), mode);
        assert_eq!(fs::read(&path)?, NATIVE);
    }
    Ok(())
}
