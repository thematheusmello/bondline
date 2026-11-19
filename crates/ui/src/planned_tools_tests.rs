//! Two actual-frame cases; child of workspace_presentation_tests.rs, no new harness/API.
use super::{TOOL_LABELS, TestResult, Workspace, chemical_paint, click, paper};
use eframe::egui;
use egui::{
    Color32, Event, Rect, Shape,
    accesskit::{Action, ActionRequest, NodeId, Role},
};
use std::collections::HashSet;

fn current(
    w: &Workspace,
    name: &str,
    roles: &[Role],
    accept: impl Fn(Rect) -> bool,
) -> TestResult<(NodeId, Rect)> {
    let mut seen = HashSet::new();
    let mut stack = vec![NodeId(egui::accesskit_root_id().value())];
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(n) = w.nodes.get(&id)
        {
            stack.extend_from_slice(n.children());
        }
    }
    let found = w
        .nodes
        .iter()
        .filter(|(id, n)| seen.contains(id) && roles.contains(&n.role()))
        .filter_map(|(id, n)| {
            let labelled = n.name() == Some(name)
                || n.labelled_by().iter().any(|label| {
                    seen.contains(label)
                        && w.nodes.get(label).is_some_and(|n| n.name() == Some(name))
                });
            let r = n.bounds()?;
            let rect = Rect::from_min_max(
                egui::pos2(r.x0 as f32, r.y0 as f32),
                egui::pos2(r.x1 as f32, r.y1 as f32),
            );
            (labelled && rect.is_positive() && accept(rect)).then_some((*id, rect))
        })
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err(format!(
            "expected one current {name} role in the declared region, found {}",
            found.len()
        )
        .into());
    }
    found
        .first()
        .copied()
        .ok_or_else(|| format!("current {name} missing").into())
}
fn active(w: &Workspace, name: &str) -> TestResult<(NodeId, Rect)> {
    let (id, r) = current(w, name, &[Role::Button, Role::CheckBox], |_| true)?;
    let node = w.nodes.get(&id).ok_or("button")?;
    assert!(!node.is_disabled());
    assert_eq!(
        node.default_action_verb(),
        Some(egui::accesskit::DefaultActionVerb::Click)
    );
    Ok((id, r))
}
fn disabled(
    w: &Workspace,
    name: &str,
    accept: impl Fn(Rect) -> bool,
) -> TestResult<(NodeId, Rect)> {
    let (id, r) = current(w, name, &[Role::Button, Role::CheckBox], accept)?;
    let n = w.nodes.get(&id).ok_or("planned node")?;
    assert!(n.is_disabled(), "planned {name} must be disabled");
    assert_ne!(n.toggled(), Some(egui::accesskit::Toggled::True));
    assert_eq!(n.description(), Some("Not implemented"));
    Ok((id, r))
}
fn default(id: NodeId) -> Event {
    Event::AccessKitActionRequest(ActionRequest {
        action: Action::Default,
        target: id,
        data: None,
    })
}
fn grey(c: Color32) -> bool {
    let channels = [c.r(), c.g(), c.b()];
    let low = channels.into_iter().min().unwrap_or(0);
    let high = channels.into_iter().max().unwrap_or(255);
    c.a() > 0 && low >= 64 && high <= 224 && high - low <= 24
}
fn has_grey_mark(output: &egui::FullOutput, rect: Rect) -> bool {
    let inner = rect.shrink(4.0);
    output.shapes.iter().any(|s| {
        if !s.clip_rect.contains_rect(s.shape.visual_bounding_rect())
            || !inner.contains_rect(s.shape.visual_bounding_rect())
        {
            return false;
        }
        match &s.shape {
            Shape::LineSegment { stroke, .. } => {
                stroke.width > 0.0
                    && matches!(&stroke.color,egui::epaint::ColorMode::Solid(c)if grey(*c))
            }
            Shape::Path(p) => {
                p.stroke.width > 0.0
                    && matches!(&p.stroke.color,egui::epaint::ColorMode::Solid(c)if grey(*c))
            }
            Shape::Circle(c) => c.stroke.width > 0.0 && grey(c.stroke.color),
            Shape::Rect(r) => r.stroke.width > 0.0 && grey(r.stroke.color),
            _ => false,
        }
    })
}
#[test]
fn delivered_twelve_stay_fixed_above_grey_lasso_and_disabled_pointer_ax_attempts_have_no_authority()
-> TestResult {
    for [width, height] in [[1280.0, 900.0], [760.0, 520.0]] {
        let mut w = Workspace::new(width, height)?;
        let shown = w.warm();
        let (canvas, _, _) = paper(&shown)?;
        active(&w, "All tools…")?;
        let mut boxes = Vec::new();
        for label in TOOL_LABELS {
            let (_, r) = active(&w, label)?;
            assert!(w.viewport.contains_rect(r) && r.right() <= canvas.left());
            assert!(r.top() >= canvas.top());
            assert!(r.width() >= 28.0 && r.height() >= 22.0);
            boxes.push(r);
        }
        for (i, r) in boxes.iter().enumerate() {
            assert!(boxes.iter().skip(i + 1).all(|other| !r.intersects(*other)));
        }
        for pair in boxes.windows(2) {
            assert!(
                pair[1].top() > pair[0].top()
                    || ((pair[1].top() - pair[0].top()).abs() <= 0.5
                        && pair[1].left() > pair[0].left()),
                "delivered source order"
            );
        }
        let (_, lasso) = disabled(&w, "Lasso", |r| r.right() <= canvas.left())?;
        assert!(
            lasso.top()
                >= boxes
                    .iter()
                    .map(Rect::bottom)
                    .fold(f32::NEG_INFINITY, f32::max)
        );
        assert!(w.viewport.contains_rect(lasso));
        assert!(
            has_grey_mark(&shown, lasso),
            "actual grey Lasso mark in its fresh AX bounds"
        );
        let before = w.state()?;
        let content = w.engine.content().clone();
        let mode = w.engine.effective_tool();
        let tool = w.engine.bond_tool();
        let controls = serde_json::to_value(w.engine.controls())?;
        let error = w.error.clone();
        let ink = chemical_paint(&shown, canvas);
        w.frame(vec![
            Event::PointerMoved(lasso.center()),
            click(lasso.center(), true),
        ]);
        w.frame(vec![click(lasso.center(), false)]);
        let (id, _) = disabled(&w, "Lasso", |r| r.right() <= canvas.left())?;
        w.frame(vec![default(id)]);
        let after = w.warm();
        assert_eq!(w.state()?, before);
        assert_eq!(w.engine.content(), &content);
        assert_eq!(w.engine.effective_tool(), mode);
        assert_eq!(w.engine.bond_tool(), tool);
        assert_eq!(serde_json::to_value(w.engine.controls())?, controls);
        assert_eq!(w.error, error);
        let (after_canvas, _, _) = paper(&after)?;
        assert_eq!(after_canvas, canvas);
        assert_eq!(chemical_paint(&after, after_canvas), ink);
        w.assert_three_pairs()?;
    }
    Ok(())
}

use egui::{Key, Modifiers};
fn key(key: Key, pressed: bool, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers,
    }
}
fn search(w: &mut Workspace, text: &str) -> TestResult {
    let (id, _) = current(w, "Search tools", &[Role::TextInput], |_| true)?;
    assert!(!w.nodes.get(&id).ok_or("search")?.is_disabled());
    w.frame(vec![Event::AccessKitActionRequest(ActionRequest {
        action: Action::Focus,
        target: id,
        data: None,
    })]);
    w.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Text(text.into()),
    ]);
    w.warm();
    let (id, _) = current(w, "Search tools", &[Role::TextInput], |_| true)?;
    assert_eq!(w.nodes.get(&id).and_then(|n| n.value()), Some(text));
    Ok(())
}

#[test]
fn modeless_catalogue_search_exposes_disabled_advanced_tools_and_text_escape_do_not_edit_the_canvas()
-> TestResult {
    let mut w = Workspace::new(1280.0, 900.0)?;
    let shown = w.warm();
    let (canvas, border, _) = paper(&shown)?;
    let scale = (border.width() / 420.0).min(border.height() / 280.0);
    let bond = border.min + egui::vec2(96.0 * scale, 64.0 * scale);
    w.frame(vec![Event::PointerMoved(bond)]);
    assert!(
        matches!(w.drawing.hover_target(),Some(bondline_render::HitTarget::Bond{fragment_id,bond_id})if fragment_id.get()==1&&bond_id.get()==1)
    );
    let before = w.state()?;
    let content = w.engine.content().clone();
    let mode = w.engine.effective_tool();
    let tool = w.engine.bond_tool();
    let error = w.error.clone();
    let (id, _) = active(&w, "All tools…")?;
    w.frame(vec![default(id)]);
    w.warm();
    let (search_id, _) = current(&w, "Search tools", &[Role::TextInput], |_| true)?;
    assert!(!w.nodes.get(&search_id).ok_or("search")?.is_disabled());
    w.frame(vec![Event::AccessKitActionRequest(ActionRequest {
        action: Action::Focus,
        target: search_id,
        data: None,
    })]);
    w.frame(vec![
        key(Key::Num2, true, Modifiers::NONE),
        Event::Text("2".into()),
        key(Key::Num2, false, Modifiers::NONE),
    ]);
    w.warm();
    let (id, _) = current(&w, "Search tools", &[Role::TextInput], |_| true)?;
    assert_eq!(w.nodes.get(&id).and_then(|n| n.value()), Some("2"));
    assert_eq!(w.state()?, before);
    assert_eq!(w.engine.effective_tool(), mode);
    assert_eq!(w.engine.bond_tool(), tool);
    assert_eq!(w.error, error);
    for name in [
        "Lasso",
        "Wedged bond",
        "Curved arrow",
        "DNA",
        "HELM sequence",
    ] {
        search(&mut w, name)?;
        let (_, r) = disabled(&w, name, |r| r.right() > canvas.left())?;
        assert!(w.viewport.contains_rect(r));
        // A disabled modeless-window row over the canvas must not dispatch or click through.
        w.frame(vec![
            Event::PointerMoved(r.center()),
            click(r.center(), true),
        ]);
        w.frame(vec![click(r.center(), false)]);
        let (id, _) = disabled(&w, name, |r| r.right() > canvas.left())?;
        w.frame(vec![default(id)]);
        w.warm();
        assert_eq!(w.state()?, before);
        assert_eq!(w.engine.content(), &content);
        assert_eq!(w.engine.effective_tool(), mode);
        assert_eq!(w.engine.bond_tool(), tool);
        assert_eq!(w.error, error);
    }
    search(&mut w, "Text")?;
    let (id, _) = current(&w, "Text", &[Role::Button, Role::CheckBox], |r| {
        r.right() > canvas.left()
    })?;
    let text = w.nodes.get(&id).ok_or("Text catalogue node")?;
    assert!(text.is_disabled());
    assert_eq!(text.description(), Some("Unavailable for this document"));
    assert!(w.engine.controls().iter().all(|c| c.id != "tool.caption"));
    w.frame(vec![default(id)]);
    w.warm();
    assert_eq!(w.state()?, before);
    assert_eq!(w.engine.content(), &content);
    assert_eq!(w.engine.effective_tool(), mode);
    assert_eq!(w.engine.bond_tool(), tool);
    assert_eq!(w.error, error);
    w.frame(vec![
        key(Key::Escape, true, Modifiers::NONE),
        key(Key::Escape, false, Modifiers::NONE),
    ]);
    let closed = w.warm();
    assert!(
        current(&w, "Search tools", &[Role::TextInput], |_| true).is_err(),
        "Escape must close the catalogue"
    );
    let (id, r) = active(&w, "Double bond")?;
    assert!(!w.nodes.get(&id).ok_or("Double")?.is_disabled());
    let ink = chemical_paint(&closed, paper(&closed)?.0);
    w.frame(vec![
        Event::PointerMoved(r.center()),
        click(r.center(), true),
    ]);
    w.frame(vec![click(r.center(), false)]);
    let selected = w.warm();
    assert_eq!(
        w.engine
            .controls()
            .into_iter()
            .find(|c| c.id == "tool.bond.double")
            .ok_or("original Double binding")?
            .selected,
        Some(true)
    );
    assert_eq!(w.state()?, before);
    assert_eq!(w.engine.content(), &content);
    assert_eq!(w.error, error);
    assert_eq!(chemical_paint(&selected, paper(&selected)?.0), ink);
    w.assert_three_pairs()?;
    Ok(())
}

#[test]
fn catalogue_escape_keeps_an_existing_persistent_selection_and_its_tool_epoch() -> TestResult {
    let mut w = Workspace::new(1160.0, 800.0)?;
    w.warm();
    w.engine.dispatch(
        "ui.selection",
        serde_json::json!({"action":"all",
        "expected_revision":w.engine.state().revision,"page_id":1}),
    )?;
    let before = w.state()?;
    let selection = w.engine.selection().clone();
    let mode = w.engine.effective_tool();
    assert!(!selection.atom_ids.is_empty());
    let (id, _) = active(&w, "All tools…")?;
    w.frame(vec![default(id)]);
    w.warm();
    search(&mut w, "Acyclic chain")?;
    w.frame(vec![
        key(Key::Escape, true, Modifiers::NONE),
        key(Key::Escape, false, Modifiers::NONE),
    ]);
    w.warm();
    assert!(current(&w, "Search tools", &[Role::TextInput], |_| true).is_err());
    assert_eq!(w.state()?, before);
    assert_eq!(w.engine.selection(), &selection);
    assert_eq!(w.engine.effective_tool(), mode);
    Ok(())
}

#[test]
fn catalogue_search_then_accessible_real_tool_selects_the_existing_binding_without_an_edit()
-> TestResult {
    let mut w = Workspace::new(1160.0, 800.0)?;
    let shown = w.warm();
    let (canvas, _, _) = paper(&shown)?;
    let before = w.state()?;
    let content = w.engine.content().clone();
    let tool = w.engine.bond_tool();
    let (id, _) = active(&w, "All tools…")?;
    w.frame(vec![default(id)]);
    w.warm();
    search(&mut w, "Double bond")?;
    let (id, _) = current(&w, "Double bond", &[Role::Button, Role::CheckBox], |r| {
        r.right() > canvas.left()
    })?;
    assert!(
        !w.nodes
            .get(&id)
            .ok_or("real catalogue binding")?
            .is_disabled()
    );
    w.frame(vec![default(id)]);
    w.warm();
    assert_eq!(
        w.engine.bond_tool().order,
        bondline_commands::BondDrawOrder::Double
    );
    assert_eq!(w.engine.bond_tool().epoch, tool.epoch + 1);
    assert_eq!(
        w.engine
            .controls()
            .into_iter()
            .find(|c| c.id == "tool.bond.double")
            .ok_or("Double")?
            .selected,
        Some(true)
    );
    assert!(current(&w, "Search tools", &[Role::TextInput], |_| true).is_err());
    assert_eq!(w.state()?, before);
    assert_eq!(w.engine.content(), &content);
    assert!(w.error.is_none());
    Ok(())
}
