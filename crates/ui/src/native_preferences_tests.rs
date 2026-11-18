//! Narrow Preferences App regressions, supplementary to the independently frozen8 families.
//! Child of view_tests.rs; no future private form/field API or direct local draft setter.
use super::{Fixture, TestResult, key};
use eframe::egui;
use egui::{
    Event, Key, Modifiers,
    accesskit::{Action, ActionRequest, DefaultActionVerb, NodeId, Role, Toggled},
};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    fs,
    time::{Duration, Instant},
};

const CHECK: &str = "Sprout rings instead of spiro when clicking";
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
fn action(id: NodeId) -> Event {
    Event::AccessKitActionRequest(ActionRequest {
        action: Action::Default,
        target: id,
        data: None,
    })
}
fn current(f: &Fixture, name: &str, role: Role) -> TestResult<(NodeId, egui::Rect)> {
    let mut stack = vec![NodeId(egui::accesskit_root_id().value())];
    let mut seen = HashSet::new();
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(n) = f.nodes.get(&id)
        {
            stack.extend_from_slice(n.children());
        }
    }
    let found = f
        .nodes
        .iter()
        .filter_map(|(id, n)| {
            if !seen.contains(id) || n.role() != role {
                return None;
            }
            let named = n.name() == Some(name)
                || n.labelled_by().iter().any(|id| {
                    seen.contains(id) && f.nodes.get(id).is_some_and(|n| n.name() == Some(name))
                });
            if !named {
                return None;
            }
            let r = n.bounds()?;
            let r = egui::Rect::from_min_max(
                egui::pos2(r.x0 as f32, r.y0 as f32),
                egui::pos2(r.x1 as f32, r.y1 as f32),
            );
            r.is_positive().then_some((*id, r))
        })
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err(format!(
            "current {name}: expected one reachable role, found{}",
            found.len()
        )
        .into());
    }
    found
        .first()
        .copied()
        .ok_or_else(|| "current control missing".into())
}
fn enabled(f: &Fixture, name: &str, role: Role) -> bool {
    current(f, name, role).is_ok_and(|(id, _)| {
        f.nodes.get(&id).is_some_and(|n| {
            !n.is_disabled() && n.default_action_verb() == Some(DefaultActionVerb::Click)
        })
    })
}
fn press(f: &Fixture, name: &str, role: Role) -> TestResult<Event> {
    let (id, _) = current(f, name, role)?;
    let n = f.nodes.get(&id).ok_or("current node")?;
    assert!(!n.is_disabled(), "{name} is enabled");
    assert_eq!(n.default_action_verb(), Some(DefaultActionVerb::Click));
    Ok(action(id))
}
fn painted(shown: &egui::FullOutput, label: &str) -> bool {
    shown.shapes.iter().any(|s| {
        matches!(&s.shape,egui::Shape::Text(t)
        if t.galley.text()==label && s.clip_rect.contains_rect(s.shape.visual_bounding_rect()))
    })
}
fn wait(
    f: &mut Fixture,
    ready: impl Fn(&Fixture, &egui::FullOutput) -> bool,
) -> TestResult<egui::FullOutput> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let shown = f.frame(vec![]);
        if ready(f, &shown) {
            return Ok(shown);
        }
        if Instant::now() >= deadline {
            return Err("Preferences current-control predicate timed out".into());
        }
        std::thread::yield_now();
    }
}
fn open(f: &mut Fixture) -> TestResult<egui::FullOutput> {
    f.choose("File")?;
    wait(f, |f, _| enabled(f, "Preferences…", Role::Button))?;
    f.choose("Preferences…")?;
    wait(f, |f, shown| {
        painted(shown, "Preferences")
            && painted(shown, "Building/Display")
            && enabled(f, CHECK, Role::CheckBox)
            && ["Use Defaults", "OK", "Cancel"]
                .iter()
                .all(|name| enabled(f, name, Role::Button))
    })
}
fn close(f: &mut Fixture) -> TestResult {
    wait(f, |f, _| current(f, CHECK, Role::CheckBox).is_err())?;
    Ok(())
}
fn toggled(f: &Fixture, want: bool) -> TestResult {
    let (id, _) = current(f, CHECK, Role::CheckBox)?;
    assert_eq!(
        f.nodes.get(&id).ok_or("real checkbox")?.toggled(),
        Some(if want { Toggled::True } else { Toggled::False })
    );
    Ok(())
}
fn inspect(f: &Fixture) -> TestResult<Value> {
    Ok(f.command("ui.preferences", json!({"action":"inspect"}))?["preferences"].clone())
}
fn authored(f: &Fixture) -> TestResult<Value> {
    let mut s = f.state()?;
    s.as_object_mut().ok_or("state")?.remove("ui_preferences");
    Ok(s)
}

#[test]
fn every_checkbox_activation_has_current_same_frame_ax_parity_before_ok() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.command(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":380,"end_x_pt":300,"end_y_pt":380}),
    )?;
    f.command(
        "ui.selection",
        json!({"action":"set","expected_revision":1,"page_id":1,"atom_ids":[1]}),
    )?;
    let before = authored(&f)?;
    let mode = f.app.engine.lock().map_err(|_| "engine")?.effective_tool();
    let order = f.app.engine.lock().map_err(|_| "engine")?.bond_tool().order;
    open(&mut f)?;
    toggled(&f, false)?;
    assert_eq!(
        inspect(&f)?,
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    let (check, _) = current(&f, CHECK, Role::CheckBox)?;
    // Inspect EXACTLY the frame returned for the packet; no settling/warm frame repairs its AX.
    f.frame(vec![action(check), action(check)]);
    toggled(&f, false)?;
    assert_eq!(
        inspect(&f)?,
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    assert_eq!(authored(&f)?, before);
    let toggle = press(&f, CHECK, Role::CheckBox)?;
    f.frame(vec![toggle]);
    toggled(&f, true)?;
    let (check, _) = current(&f, CHECK, Role::CheckBox)?;
    f.frame(vec![action(check), action(check)]);
    toggled(&f, true)?;
    let defaults = press(&f, "Use Defaults", Role::Button)?;
    f.frame(vec![defaults]);
    toggled(&f, false)?;
    let toggle = press(&f, CHECK, Role::CheckBox)?;
    f.frame(vec![toggle]);
    toggled(&f, true)?;
    assert_eq!(authored(&f)?, before);
    assert_eq!(
        inspect(&f)?,
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    let ok = press(&f, "OK", Role::Button)?;
    f.frame(vec![ok]);
    close(&mut f)?;
    assert_eq!(
        inspect(&f)?,
        json!({"sprout_rings_when_clicking":true,"epoch":1})
    );
    assert_eq!(authored(&f)?, before);
    assert_eq!(
        f.app.engine.lock().map_err(|_| "engine")?.effective_tool(),
        mode
    );
    assert_eq!(
        f.app.engine.lock().map_err(|_| "engine")?.bond_tool().order,
        order
    );
    Ok(())
}

#[test]
fn a_stale_source_draft_stays_copyable_until_exact_remote_cancel_and_fences_retirement_packet()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = f.directory.path("preferences-source-λ.cdxml");
    fs::write(&path, NATIVE)?;
    f.command("document.open_cdxml", json!({"path":path}))?;
    let original = f.state()?;
    open(&mut f)?;
    let id = f.state()?["ui_preferences"]["id"]
        .as_u64()
        .ok_or("request id")?;
    let toggle = press(&f, CHECK, Role::CheckBox)?;
    f.frame(vec![toggle]);
    toggled(&f, true)?;
    // A real independent source edit makes the captured form stale; it is not an explicit Cancel.
    f.command(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":220,"begin_y_pt":300,"end_x_pt":268,"end_y_pt":300}),
    )?;
    let latest = authored(&f)?;
    let held = f.app.engine.lock().map_err(|_| "engine")?.content().clone();
    let revision = f.state()?["revision"].as_u64().ok_or("revision")?;
    f.frame(vec![]);
    assert_eq!(f.state()?["ui_preferences"]["id"], id);
    assert_eq!(f.state()?["ui_preferences"]["current"], false);
    toggled(&f, true)?;
    let (ok, _) = current(&f, "OK", Role::Button)?;
    assert!(f.nodes.get(&ok).ok_or("stale OK")?.is_disabled());
    assert!(
        enabled(&f, CHECK, Role::CheckBox)
            && enabled(&f, "Use Defaults", Role::Button)
            && enabled(&f, "Cancel", Role::Button)
    );
    let defaults = press(&f, "Use Defaults", Role::Button)?;
    let toggle = press(&f, CHECK, Role::CheckBox)?;
    f.frame(vec![defaults, toggle]);
    toggled(&f, true)?;
    f.frame(vec![action(ok)]);
    assert_eq!(authored(&f)?, latest);
    assert_eq!(
        inspect(&f)?,
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    let wrong = id.checked_add(1).ok_or("request id overflow")?;
    let failure = {
        let mut engine = f.app.engine.lock().map_err(|_| "engine")?;
        engine
            .dispatch(
                "ui.preferences",
                json!({"action":"cancel","expected_revision":revision,"request_id":wrong}),
            )
            .err()
            .ok_or("wrong Cancel accepted")?
    };
    assert_eq!(failure.info().code, "ui_stale_request");
    f.frame(vec![]);
    assert_eq!(f.state()?["ui_preferences"]["id"], id);
    toggled(&f, true)?;
    assert_eq!(authored(&f)?, latest);
    let canceled = f.command(
        "ui.preferences",
        json!({"action":"cancel","expected_revision":revision,"request_id":id}),
    )?;
    assert_eq!(canceled["changed"], false);
    assert!(f.state()?.get("ui_preferences").is_none());
    // The old shown form owns this dismissal frame even though the Engine request is gone.
    f.frame(vec![
        action(ok),
        key(Key::Z, true, Modifiers::COMMAND),
        key(Key::Z, false, Modifiers::COMMAND),
    ]);
    assert!(current(&f, CHECK, Role::CheckBox).is_err());
    assert_eq!(authored(&f)?, latest);
    assert_eq!(f.app.engine.lock().map_err(|_| "engine")?.content(), &held);
    assert_eq!(
        inspect(&f)?,
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    assert_eq!(fs::read(&path)?, NATIVE);
    // A NEW later packet can use ordinary Undo: retirement does not permanently swallow input.
    f.frame(vec![
        key(Key::Z, true, Modifiers::COMMAND),
        key(Key::Z, false, Modifiers::COMMAND),
    ]);
    let undone = f.state()?;
    assert_eq!(undone["document"], original["document"]);
    assert_eq!(undone["undo_depth"], 0);
    assert_eq!(undone["redo_depth"], 1);
    assert_eq!(undone["file"]["dirty"], false);
    assert_eq!(undone["source"]["modified"], false);
    assert_eq!(fs::read(&path)?, NATIVE);
    Ok(())
}

#[test]
fn small_viewport_keeps_defaults_ok_and_cancel_inside_the_visible_painted_form() -> TestResult {
    let mut f = Fixture::new(480.0, 300.0, 1.0)?;
    let before = authored(&f)?;
    let shown = open(&mut f)?;
    assert!(painted(&shown, "Applies to all documents in this session."));
    let screen = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(f.width, f.height));
    for name in ["Use Defaults", "OK", "Cancel"] {
        let (_, r) = current(&f, name, Role::Button)?;
        assert!(screen.contains_rect(r), "{name} must fit the viewport");
        assert!(enabled(&f, name, Role::Button));
        assert!(painted(&shown, name), "{name} text is not clipped");
    }
    let toggle = press(&f, CHECK, Role::CheckBox)?;
    f.frame(vec![toggle]);
    toggled(&f, true)?;
    let defaults = press(&f, "Use Defaults", Role::Button)?;
    f.frame(vec![defaults]);
    toggled(&f, false)?;
    let cancel = press(&f, "Cancel", Role::Button)?;
    f.frame(vec![cancel]);
    close(&mut f)?;
    assert_eq!(authored(&f)?, before);
    assert_eq!(
        inspect(&f)?,
        json!({"sprout_rings_when_clicking":false,"epoch":0})
    );
    assert!(f.state()?["ui_file"]["request"].is_null());
    assert!(!f.app.files.is_busy());
    Ok(())
}
