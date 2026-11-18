//! Independent actual-App checks for truthful Fit versus explicit numeric Zoom.
use super::{Fixture, TestResult, key};
use eframe::egui;
use egui::{Event, Key, Modifiers};
use serde_json::json;
use std::fs;

const SOURCE: &[u8] =
    include_bytes!("../../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");

fn fit_readout(fixture: &Fixture, output: &egui::FullOutput) -> TestResult {
    let (id, bounds) = fixture.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    assert!(
        output.shapes.iter().any(|paint| matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text() == "Fit"
            && paint.clip_rect.intersects(paint.shape.visual_bounding_rect())
            && bounds.intersects(paint.shape.visual_bounding_rect()))),
        "unknown SDF Fit must visibly report Fit, not the100% fallback seed"
    );
    let node = fixture.nodes.get(&id).ok_or("current Zoom node")?;
    assert!(
        node.numeric_value().is_none(),
        "Fit has no truthful numeric percentage to advertise"
    );
    let mut stack = vec![id];
    let mut reports_fit = false;
    while let Some(id) = stack.pop() {
        let node = fixture.nodes.get(&id).ok_or("current Zoom subtree")?;
        reports_fit |= node.value() == Some("Fit") || node.name() == Some("Fit");
        stack.extend_from_slice(node.children());
    }
    assert!(
        reports_fit,
        "the actual Zoom accessibility subtree must report Fit"
    );
    Ok(())
}

fn focus_zoom(fixture: &mut Fixture) -> TestResult {
    let (id, _) = fixture.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    fixture.frame(vec![Event::AccessKitActionRequest(
        egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        },
    )]);
    Ok(())
}

#[test]
fn unknown_fit_stays_truthful_on_idle_focus_cancel_and_explicit100_enter_commits() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let path = fixture.directory.path("unknown-fit.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    let before = fixture.state()?;
    let stamp = fixture
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .capture_stamp();
    let view = fixture.view()?;
    assert!(view["zoom_percent"].is_null());
    assert_eq!(view["epoch"], 0);
    let output = fixture.warm();
    assert!(
        fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .view_percent()
            .is_none()
    );
    fit_readout(&fixture, &output)?;
    for _ in 0..3 {
        let output = fixture.frame(vec![]);
        fit_readout(&fixture, &output)?;
    }
    focus_zoom(&mut fixture)?;
    assert_eq!(fixture.view()?, view);
    fixture.frame(vec![
        key(Key::Escape, true, Modifiers::NONE),
        key(Key::Escape, false, Modifiers::NONE),
    ]);
    let output = fixture.frame(vec![]);
    fit_readout(&fixture, &output)?;
    assert_eq!(fixture.view()?, view);
    assert_eq!(fixture.state()?, before);
    focus_zoom(&mut fixture)?;
    fixture.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Text("100".into()),
    ]);
    assert_eq!(fixture.view()?, view, "typing is not a camera commit");
    fixture.frame(vec![
        key(Key::Enter, true, Modifiers::NONE),
        key(Key::Enter, false, Modifiers::NONE),
    ]);
    assert_eq!(
        fixture.view()?["zoom_percent"],
        100.0,
        "explicit100 differs from unknown Fit even when the old numeric seed was100"
    );
    assert_eq!(fixture.view()?["epoch"], 1);
    assert_eq!(fixture.state()?, before);
    let engine = fixture.app.engine.lock().map_err(|_| "engine")?;
    assert!(engine.capture_stamp_matches(&stamp));
    assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
    assert_eq!(fs::read(&path)?, SOURCE);
    Ok(())
}

fn sdf_fixture(dpi: f32) -> TestResult<Fixture> {
    let mut fixture = Fixture::new(1160.0, 800.0, dpi)?;
    let path = fixture.directory.path("fit-control-source.sdf");
    fs::write(&path, SOURCE)?;
    fixture.command("document.open_sdf", json!({"path":path}))?;
    fixture.warm();
    Ok(fixture)
}

fn numeric_readout(fixture: &Fixture, output: &egui::FullOutput, expected: f64) -> TestResult {
    let (id, bounds) = fixture.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    let numeric = fixture
        .nodes
        .get(&id)
        .and_then(|node| node.numeric_value())
        .ok_or("truthful accessible numeric percentage")?;
    assert!((numeric - expected).abs() < 1e-8);
    assert!(
        output.shapes.iter().any(|paint| match &paint.shape {
            egui::Shape::Text(text) if bounds.intersects(paint.shape.visual_bounding_rect()) =>
                text.galley
                    .text()
                    .strip_suffix('%')
                    .and_then(|number| number.trim().parse::<f64>().ok())
                    .is_some_and(|number| (number - expected).abs() <= 0.011),
            _ => false,
        }),
        "visible numeric percentage agrees with actual view to its two-decimal presentation"
    );
    Ok(())
}

#[test]
fn explicit100_blur_is_a_commit_even_when_the_unknown_fit_seed_was100() -> TestResult {
    let mut fixture = sdf_fixture(1.0)?;
    let before = fixture.state()?;
    let view = fixture.view()?;
    focus_zoom(&mut fixture)?;
    fixture.frame(vec![
        key(Key::A, true, Modifiers::COMMAND),
        key(Key::A, false, Modifiers::COMMAND),
        Event::Text("100".into()),
    ]);
    assert_eq!(fixture.view()?, view);
    fixture.choose("View")?;
    assert_eq!(fixture.view()?["zoom_percent"], 100.0);
    assert_eq!(fixture.view()?["epoch"], 1);
    fixture.frame(vec![
        key(Key::Escape, true, Modifiers::NONE),
        key(Key::Escape, false, Modifiers::NONE),
    ]);
    let output = fixture.frame(vec![]);
    numeric_readout(&fixture, &output, 100.0)?;
    assert_eq!(fixture.state()?, before);
    assert_eq!(
        fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .sdf()?
            .as_bytes(),
        SOURCE
    );
    Ok(())
}

#[test]
fn sdf_numeric_ax100_then_prepared_fit_at2x_reports_fit_without_a_numeric_claim() -> TestResult {
    let mut fixture = sdf_fixture(2.0)?;
    let before = fixture.state()?;
    let output = fixture.frame(vec![]);
    fit_readout(&fixture, &output)?;
    let (zoom, _) = fixture.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    fixture.frame(vec![super::set_slider(zoom, 100.0)]);
    assert_eq!(fixture.view()?["zoom_percent"], 100.0);
    assert_eq!(fixture.view()?["epoch"], 1);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let output = fixture.frame(vec![]);
        if !fixture.app.preview.is_busy()
            && !fixture.app.executor.has_active_work()
            && !fixture.view()?["prepared_sdf_extent"].is_null()
        {
            numeric_readout(&fixture, &output, 100.0)?;
            break;
        }
        if std::time::Instant::now() >= deadline {
            return Err("Actual100% did not prepare real SDF extent".into());
        }
        std::thread::yield_now();
    }
    fixture.choose("View")?;
    fixture.choose("Fit to window")?;
    let output = fixture.warm();
    fit_readout(&fixture, &output)?;
    let view = fixture.view()?;
    assert!(view["zoom_percent"].is_null());
    assert!(
        !view["prepared_sdf_extent"].is_null(),
        "the known cache must not change Fit's presentation policy"
    );
    assert_eq!(view["epoch"], 2);
    assert_eq!(fixture.state()?, before);
    assert_eq!(
        fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .sdf()?
            .as_bytes(),
        SOURCE
    );
    Ok(())
}

#[test]
fn drawing_known_fit_and_explicit_percentage_remain_visible_and_accessible_numbers() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let output = fixture.warm();
    let before = fixture.state()?;
    let fitted = fixture
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .view_percent()
        .ok_or("actual known Drawing fit percentage")?;
    assert!(fitted > 1.0 && fitted < 99.0);
    numeric_readout(&fixture, &output, fitted)?;
    let (zoom, _) = fixture.node(
        "Zoom",
        &[
            egui::accesskit::Role::SpinButton,
            egui::accesskit::Role::TextInput,
        ],
    )?;
    fixture.frame(vec![super::set_slider(zoom, 150.0)]);
    let output = fixture.frame(vec![]);
    numeric_readout(&fixture, &output, 150.0)?;
    assert_eq!(fixture.view()?["zoom_percent"], 150.0);
    assert_eq!(fixture.state()?, before);
    Ok(())
}

fn disabled_zoom(fixture: &Fixture) -> TestResult<(egui::accesskit::NodeId, egui::Rect)> {
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![egui::accesskit::NodeId(egui::accesskit_root_id().value())];
    while let Some(id) = stack.pop() {
        if seen.insert(id)
            && let Some(node) = fixture.nodes.get(&id)
        {
            stack.extend_from_slice(node.children());
        }
    }
    fixture
        .nodes
        .iter()
        .find_map(|(id, node)| {
            let labelled = node.name() == Some("Zoom")
                || node.labelled_by().iter().any(|id| {
                    seen.contains(id)
                        && fixture
                            .nodes
                            .get(id)
                            .is_some_and(|label| label.name() == Some("Zoom"))
                });
            if !seen.contains(id)
                || !labelled
                || !matches!(
                    node.role(),
                    egui::accesskit::Role::SpinButton | egui::accesskit::Role::TextInput
                )
            {
                return None;
            }
            let rect = node.bounds()?;
            Some((
                *id,
                egui::Rect::from_min_max(
                    egui::pos2(rect.x0 as f32, rect.y0 as f32),
                    egui::pos2(rect.x1 as f32, rect.y1 as f32),
                ),
            ))
        })
        .ok_or_else(|| "reachable disabled Zoom field".into())
}

#[test]
fn file_picker_disables_input_without_turning_sdf_fit_into_a_false_numeric_percentage() -> TestResult
{
    let mut fixture = sdf_fixture(1.0)?;
    let before = fixture.state()?;
    let view = fixture.view()?;
    fixture.command("ui.file", json!({"expected_revision":1,"action":"open"}))?;
    fixture.warm();
    let output = fixture.frame(vec![]);
    let modal = fixture.state()?;
    assert_eq!(modal["ui_file"]["request"]["phase"], "accepted");
    let (id, bounds) = disabled_zoom(&fixture)?;
    let node = fixture.nodes.get(&id).ok_or("current disabled Zoom")?;
    assert!(
        node.is_disabled(),
        "the real accepted picker disables the existing input"
    );
    assert!(
        output.shapes.iter().any(|paint| matches!(&paint.shape,
        egui::Shape::Text(text) if text.galley.text() == "Fit"
            && bounds.intersects(paint.shape.visual_bounding_rect())
            && paint.clip_rect.intersects(paint.shape.visual_bounding_rect()))),
        "temporarily disabled SDF Fit must still visibly say Fit rather than100%"
    );
    assert!(
        node.numeric_value().is_none(),
        "disabled controls cannot invent a numeric Fit value"
    );
    let mut reports_fit = false;
    let mut stack = vec![id];
    while let Some(id) = stack.pop() {
        let node = fixture
            .nodes
            .get(&id)
            .ok_or("current disabled Zoom subtree")?;
        reports_fit |= node.value() == Some("Fit") || node.name() == Some("Fit");
        stack.extend_from_slice(node.children());
    }
    assert!(reports_fit);
    assert_eq!(fixture.view()?, view);
    let request = modal["ui_file"]["request"]["id"]
        .as_u64()
        .ok_or("current picker request")?;
    fixture.command(
        "ui.file",
        json!({"expected_revision":1,"action":"cancel","request_id":request}),
    )?;
    let output = fixture.frame(vec![]);
    fit_readout(&fixture, &output)?;
    let mut after = fixture.state()?;
    assert!(after["ui_file"]["request"].is_null());
    assert_eq!(after["ui_file"]["epoch"], 2);
    after["ui_file"] = before["ui_file"].clone();
    assert_eq!(
        after, before,
        "only the explicit picker intent changes its own epoch"
    );
    assert_eq!(fixture.view()?, view);
    assert_eq!(
        fixture
            .app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .content()
            .sdf()?
            .as_bytes(),
        SOURCE
    );
    Ok(())
}
