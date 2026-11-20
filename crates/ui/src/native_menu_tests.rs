//! Native accessibility requests arrive even when another application has keyboard focus.
use super::*;
use egui::accesskit::{Action, ActionRequest, NodeId, Role};

fn ax(target: NodeId, action: Action) -> Event {
    Event::AccessKitActionRequest(ActionRequest {
        action,
        target,
        data: None,
    })
}

fn press(f: &mut Fixture, label: &str, focused: bool) -> TestResult {
    let (target, _) = f.node(label, &[Role::Button])?;
    f.frame_with_focus(vec![ax(target, Action::Default)], focused);
    f.frame_with_focus(Vec::new(), focused);
    Ok(())
}

#[test]
fn ax_menu_default_runs_only_the_target_on_background_drawing_and_collection() -> TestResult {
    for collection in [false, true] {
        let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
        if collection {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");
            f.command("document.open_sdf", json!({"path":path}))?;
        }
        let paper = canvas(&f.warm())?.center();
        let before = f.state()?;
        let camera = f.view()?;
        press(&mut f, "View", false)?;
        let (target, _) = f.node("Show Analysis Window", &[Role::Button])?;
        let mut events = vec![ax(target, Action::Default)];
        events.extend(tap(Key::F7));
        events.extend([
            Event::PointerMoved(paper),
            button(paper, true),
            button(paper, false),
        ]);
        f.frame_with_focus(events, false);
        let mut expected = before;
        expected["analysis_window_open"] = json!(true);
        assert_eq!(
            f.state()?,
            expected,
            "background menu action, collection={collection}"
        );
        assert_eq!(f.view()?, camera, "physical shortcut/press when unfocused");

        // A default sent to the retired popup item has no owner in this frame.
        f.frame_with_focus(vec![ax(target, Action::Default)], false);
        assert_eq!(f.state()?, expected);
    }
    Ok(())
}

#[test]
fn ax_menu_keeps_focused_pointer_and_keyboard_activation_off_the_paper() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let paper = canvas(&f.warm())?;
    let initial = f.state()?;
    f.choose("View")?;
    let (_, item) = f.node("Show Analysis Window", &[Role::Button])?;
    assert!(
        paper.contains(item.center()),
        "the popup must cover the drawing surface"
    );
    f.choose("Show Analysis Window")?;
    let mut shown = initial.clone();
    shown["analysis_window_open"] = json!(true);
    assert_eq!(f.state()?, shown, "menu press/release must not add a bond");

    f.choose("View")?;
    let (item, _) = f.node("Hide Analysis Window", &[Role::Button])?;
    f.frame(vec![ax(item, Action::Focus)]);
    f.frame(tap(Key::Enter));
    assert_eq!(
        f.state()?,
        initial,
        "focused Enter must execute the current menu item once"
    );
    Ok(())
}

#[test]
fn ax_menu_opens_nested_constraints_and_respects_the_current_tool() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    f.command("ui.chain_tool", json!({"action":"select","kind":"acyclic"}))?;
    f.warm();
    press(&mut f, "Object", false)?;
    press(&mut f, "Chain constraints", false)?;
    let (lengths, _) = f.node("Fixed Lengths", &[Role::Button])?;
    let previous = f.command("ui.controls", json!({}))?;
    let binding = |controls: &Value| -> TestResult<(bool, bool)> {
        let control = controls["controls"]
            .as_array()
            .ok_or("controls")?
            .iter()
            .find(|c| c["id"] == "chain.fixed_lengths")
            .ok_or("Fixed Lengths binding")?;
        Ok((
            control["enabled"].as_bool().ok_or("enabled")?,
            control["selected"].as_bool().ok_or("selected")?,
        ))
    };
    let (enabled, checked) = binding(&previous)?;
    assert!(enabled);
    let before = f.state()?;
    f.frame_with_focus(vec![ax(lengths, Action::Default)], false);
    let after = f.state()?;
    assert_eq!(
        binding(&f.command("ui.controls", json!({}))?)?,
        (true, !checked)
    );
    for key in ["revision", "document", "undo_depth", "redo_depth"] {
        assert_eq!(after[key], before[key]);
    }

    f.command("ui.bond_tool", json!({"order":"single"}))?;
    f.warm();
    assert_eq!(
        binding(&f.command("ui.controls", json!({}))?)?,
        (false, !checked)
    );
    press(&mut f, "Object", false)?;
    press(&mut f, "Chain constraints", false)?;
    assert!(
        f.nodes
            .get(&lengths)
            .ok_or("rendered disabled Fixed Lengths")?
            .is_disabled()
    );
    let disabled = f.state()?;
    f.frame_with_focus(vec![ax(lengths, Action::Default)], false);
    assert_eq!(f.state()?, disabled);
    assert_eq!(
        binding(&f.command("ui.controls", json!({}))?)?,
        (false, !checked)
    );
    Ok(())
}
