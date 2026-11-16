//! Post-first-draft actual App regression; child of view_tests.rs.
use super::{Fixture, TestResult, tap};
use eframe::egui;
use egui::{
    Event, Key,
    accesskit::{Action, ActionRequest, NodeId, Role},
};
use serde_json::json;
use std::{collections::HashSet, fs};

const SOURCE: &[u8] = include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

fn default(id: NodeId) -> Event {
    Event::AccessKitActionRequest(ActionRequest {
        action: Action::Default,
        target: id,
        data: None,
    })
}
fn enabled_button(f: &Fixture, name: &str) -> TestResult<NodeId> {
    let (id, rect) = f.node(name, &[Role::Button, Role::CheckBox])?;
    let node = f.nodes.get(&id).ok_or("current button")?;
    assert!(
        rect.is_positive() && !node.is_disabled(),
        "current enabled {name}"
    );
    assert_eq!(
        node.default_action_verb(),
        Some(egui::accesskit::DefaultActionVerb::Click)
    );
    Ok(id)
}
fn search_present(f: &Fixture) -> bool {
    let mut seen = HashSet::new();
    let mut stack = vec![NodeId(egui::accesskit_root_id().value())];
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = f.nodes.get(&id)
        {
            if node.role() == Role::TextInput && node.name() == Some("Search tools") {
                return true;
            }
            stack.extend_from_slice(node.children());
        }
    }
    false
}

#[test]
fn sdf_removal_foreground_hides_and_disables_catalogue_and_owns_its_escape() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let input = f.directory.path("catalogue-sdf-original-化学.sdf");
    fs::write(&input, SOURCE)?;
    f.command("document.open_sdf", json!({"path":input}))?;
    f.choose("Records")?;
    f.warm();
    let before = f.state()?;
    assert_eq!(before["collection"]["record_count"], 2);
    assert_eq!(before["collection"]["byte_length"], 777);
    assert_eq!(before["undo_depth"], 0);
    let original = f.app.engine.lock().map_err(|_| "engine")?.content().clone();
    let all = enabled_button(&f, "All tools…")?;
    f.frame(vec![default(all)]);
    f.warm();
    assert!(search_present(&f));
    // This is an existing actual record-row action, not a private editor-state setup.
    let remove = enabled_button(&f, "Remove record 1")?;
    f.frame(vec![default(remove)]);
    f.warm();
    enabled_button(&f, "Keep record")?;
    enabled_button(&f, "Confirm record removal")?;
    assert!(
        f.app.sdf_fields.presentation_present(),
        "actual SDF removal owns a foreground presentation"
    );
    let hidden = !search_present(&f);
    let rail_inert = match f.node("All tools…", &[Role::Button, Role::CheckBox]) {
        Ok((id, _)) => f.nodes.get(&id).is_some_and(|node| node.is_disabled()),
        Err(_) => true,
    };
    // Do not toggle the catalogue before this Escape: that could mask event theft.
    f.frame(tap(Key::Escape));
    f.warm();
    let dismissed = f.node("Keep record", &[Role::Button]).is_err()
        && f.node("Confirm record removal", &[Role::Button]).is_err();
    assert_eq!(f.state()?, before);
    let engine = f.app.engine.lock().map_err(|_| "engine")?;
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
    assert_eq!(fs::read(&input)?, SOURCE);
    assert!(
        hidden && rail_inert && dismissed,
        "SDF foreground priority: catalogue_hidden={hidden}, rail_inert={rail_inert}, removal_dismissed_by_escape={dismissed}"
    );
    Ok(())
}
