//! Independent actual-frame acceptance for the original desktop shell; no native proof.
use super::*;
use egui::{Color32, Event, Modifiers, PointerButton, Pos2, Rect, Shape};
use serde_json::{Value, json};
use std::collections::HashMap;

#[path = "planned_tools_tests.rs"]
mod planned_tools;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const TOOL_LABELS: [&str; 12] = [
    "Single bond",
    "Double bond",
    "Triple bond",
    "Marquee selection",
    "Cyclopropane",
    "Cyclobutane",
    "Cyclopentane",
    "Cyclohexane",
    "Cycloheptane",
    "Cyclooctane",
    "Benzene",
    "Cyclopentadiene",
];

#[test]
fn original_logo_has_a_named_header_image_without_touching_document_or_tool() -> TestResult {
    let mut workspace = Workspace::new(1160.0, 800.0)?;
    let before = workspace.state()?;
    workspace.warm();
    let logo = workspace
        .nodes
        .values()
        .find(|node| {
            node.role() == egui::accesskit::Role::Image && node.name() == Some("Bondline logo")
        })
        .ok_or("missing original Bondline logo in the actual header")?;
    let bounds = logo.bounds().ok_or("logo bounds")?;
    assert_eq!((bounds.width(), bounds.height()), (28.0, 28.0));
    let edit = workspace.button("Edit")?;
    assert!(bounds.x1 < f64::from(edit.left()));
    assert_eq!(workspace.state()?, before);
    assert_eq!(workspace.engine.bond_tool().order, BondDrawOrder::Single);
    Ok(())
}

struct Workspace {
    engine: Engine,
    context: egui::Context,
    drawing: DrawingState,
    error: Option<String>,
    viewport: Rect,
    nodes: HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
}

impl Workspace {
    fn new(width: f32, height: f32) -> TestResult<Self> {
        let mut engine = Engine::default();
        engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":420,"height_pt":280}),
        )?;
        for (order, y) in [("single", 64), ("double", 132), ("triple", 200)] {
            engine.dispatch(
                "bond.create",
                json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":y,
                       "end_x_pt":120,"end_y_pt":y,"order":order}),
            )?;
        }
        let context = egui::Context::default();
        configure_context(&context);
        context.enable_accesskit();
        Ok(Self {
            engine,
            context,
            drawing: DrawingState::default(),
            error: None,
            viewport: Rect::from_min_size(Pos2::ZERO, egui::vec2(width, height)),
            nodes: HashMap::new(),
        })
    }

    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        let mut input = egui::RawInput {
            screen_rect: Some(self.viewport),
            focused: true,
            events,
            ..Default::default()
        };
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(1.0);
        let context = self.context.clone();
        let output = context.run(input, |ctx| {
            show_workspace(ctx, &mut self.engine, &mut self.error, &mut self.drawing);
        });
        if let Some(update) = &output.platform_output.accesskit_update {
            for (id, node) in &update.nodes {
                self.nodes.insert(*id, node.clone());
            }
        }
        output
    }

    fn warm(&mut self) -> egui::FullOutput {
        self.frame(vec![]);
        self.frame(vec![])
    }

    fn button(&self, label: &str) -> TestResult<Rect> {
        let node = self
            .nodes
            .values()
            .find(|node| node.role() == egui::accesskit::Role::Button && node.name() == Some(label))
            .ok_or_else(|| format!("accessible actual control {label} missing"))?;
        assert_eq!(node.role(), egui::accesskit::Role::Button);
        let bounds = node.bounds().ok_or("accessible control bounds missing")?;
        Ok(Rect::from_min_max(
            egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
            egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
        ))
    }

    fn state(&self) -> TestResult<Value> {
        Ok(serde_json::to_value(self.engine.state())?)
    }

    fn assert_three_pairs(&self) -> TestResult {
        let page = self.engine.document()?.pages().first().ok_or("page")?;
        assert_eq!(page.size().width_pt(), 420.0);
        assert_eq!(page.size().height_pt(), 280.0);
        assert_eq!(page.fragments().len(), 3);
        assert_eq!(
            page.fragments()
                .iter()
                .map(|f| f.atoms().len())
                .sum::<usize>(),
            6
        );
        assert_eq!(
            page.fragments()
                .iter()
                .map(|f| f.bonds().len())
                .sum::<usize>(),
            3
        );
        Ok(())
    }
}

fn paper(output: &egui::FullOutput) -> TestResult<(Rect, Rect, Rect)> {
    let (canvas, border) = output
        .shapes
        .iter()
        .find_map(|paint| match &paint.shape {
            Shape::Rect(rect) if rect.fill == Color32::from_gray(196) => {
                Some((paint.clip_rect, rect.rect))
            }
            _ => None,
        })
        .ok_or("actual chemical-paper border missing")?;
    let white = output
        .shapes
        .iter()
        .find_map(|paint| match &paint.shape {
            Shape::Rect(rect)
                if paint.clip_rect == canvas
                    && rect.fill == Color32::WHITE
                    && rect.rect.min == border.min + egui::vec2(1.0, 1.0)
                    && rect.rect.size() == border.size() - egui::vec2(2.0, 2.0) =>
            {
                Some(rect.rect)
            }
            _ => None,
        })
        .ok_or("literal one-unit white paper inset missing")?;
    assert!(canvas.contains_rect(border));
    let ratio = 420.0 / 280.0;
    assert!((border.width() - ratio * border.height()).abs() <= (1.0 + ratio) / 2.0);
    Ok((canvas, border, white))
}

fn assert_layout(workspace: &Workspace, output: &egui::FullOutput) -> TestResult<Rect> {
    let (canvas, _, _) = paper(output)?;
    assert!(workspace.viewport.contains_rect(canvas));
    assert!(
        (canvas.left() - 64.0).abs() <= 4.0,
        "the canvas must follow the agreed 64-unit left tool rail: {canvas:?}"
    );
    assert!(
        (workspace.viewport.right() - canvas.right() - 248.0).abs() <= 4.0,
        "the agreed right inspector must reserve space outside the canvas: {canvas:?}"
    );
    let mut tool_bounds = Vec::new();
    for label in TOOL_LABELS {
        let bounds = workspace.button(label)?;
        assert!(
            workspace.viewport.contains_rect(bounds),
            "{label}: {bounds:?}"
        );
        assert!(bounds.width() >= 28.0 && bounds.height() >= 22.0);
        assert!(
            bounds.right() <= canvas.left() && bounds.top() >= canvas.top(),
            "{label} must be in the left tool rail, not the header/canvas: {bounds:?}"
        );
        assert!(!bounds.intersects(canvas));
        tool_bounds.push(bounds);
    }
    for (index, bounds) in tool_bounds.iter().enumerate() {
        assert!(
            tool_bounds
                .iter()
                .skip(index + 1)
                .all(|other| !bounds.intersects(*other))
        );
    }
    let mut planned_count = 0;
    let mut catalogue_count = 0;
    let mut unavailable_text_count = 0;
    let mut rail_buttons = workspace
        .nodes
        .values()
        .filter(|node| node.role() == egui::accesskit::Role::Button)
        .filter_map(|node| {
            let b = node.bounds()?;
            if b.x1 > f64::from(canvas.left()) || b.y0 < f64::from(canvas.top()) {
                return None;
            }
            let name = node.name()?;
            if name == "All tools…" {
                assert!(!node.is_disabled());
                catalogue_count += 1;
                None
            } else if name == "Text" {
                assert!(node.is_disabled());
                assert_eq!(node.description(), Some("Unavailable for this document"));
                assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::False));
                unavailable_text_count += 1;
                None
            } else if node.description() == Some("Not implemented") {
                assert!(node.is_disabled());
                assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::False));
                planned_count += 1;
                None
            } else {
                Some(name.to_owned())
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(catalogue_count, 1);
    assert_eq!(planned_count, 123);
    assert_eq!(unavailable_text_count, 1);
    rail_buttons.sort();
    assert_eq!(
        rail_buttons,
        [
            "Acyclic chain",
            "Benzene",
            "Cyclobutane",
            "Cycloheptane",
            "Cyclohexane",
            "Cyclohexane chair 1",
            "Cyclohexane chair 2",
            "Cyclooctane",
            "Cyclopentadiene",
            "Cyclopentane",
            "Cyclopropane",
            "Double bond",
            "Marquee selection",
            "Single bond",
            "Snaking chain",
            "Triple bond"
        ]
    );

    let mut inspector_text = Vec::new();
    for paint in &output.shapes {
        if let Shape::Text(text) = &paint.shape {
            let bounds = text.galley.rect.translate(text.pos.to_vec2());
            if bounds.left() >= canvas.right() {
                assert!(workspace.viewport.contains_rect(bounds));
                assert!(paint.clip_rect.contains_rect(bounds));
                assert!(!bounds.intersects(canvas));
                inspector_text.push(text.galley.job.text.as_str());
            }
        }
    }
    assert!(
        inspector_text.contains(&"Document"),
        "a real right Document inspector is required"
    );
    let text = inspector_text.join("\n");
    for expected in ["6 atoms", "3 bonds", "3 fragments", "420", "280", "pt"] {
        assert!(
            text.contains(expected),
            "actual document metadata {expected:?} absent: {text}"
        );
    }
    assert!(workspace.error.is_none(), "{:?}", workspace.error);
    Ok(canvas)
}

fn chemical_paint(output: &egui::FullOutput, canvas: Rect) -> Vec<egui::epaint::ClippedShape> {
    output
        .shapes
        .iter()
        .filter(|paint| paint.clip_rect == canvas)
        .filter(|paint| match &paint.shape {
            Shape::Rect(rect) => {
                rect.fill == Color32::BLACK
                    || rect.fill == Color32::WHITE
                    || rect.fill == Color32::from_gray(196)
            }
            Shape::Path(path) => path.closed && path.fill == Color32::BLACK,
            _ => false,
        })
        .cloned()
        .collect()
}

fn click(point: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: point,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn workspace_has_left_registered_tools_right_actual_document_and_unchanged_white_paper()
-> TestResult {
    for [width, height] in [[1280.0, 900.0], [760.0, 520.0]] {
        let mut workspace = Workspace::new(width, height)?;
        workspace.assert_three_pairs()?;
        let before = workspace.state()?;
        let content = workspace.engine.content().clone();
        let output = workspace.warm();
        assert_layout(&workspace, &output)?;
        assert_eq!(
            workspace.state()?,
            before,
            "layout must not author document data"
        );
        assert_eq!(workspace.engine.content(), &content);
    }
    Ok(())
}

#[test]
fn selecting_actual_left_double_preserves_authored_snapshot_history_and_chemical_geometry()
-> TestResult {
    for [width, height] in [[1280.0, 900.0], [760.0, 520.0]] {
        let mut workspace = Workspace::new(width, height)?;
        let output = workspace.warm();
        let canvas = assert_layout(&workspace, &output)?;
        let before = workspace.state()?;
        let content = workspace.engine.content().clone();
        let ink = chemical_paint(&output, canvas);
        assert!(
            ink.len() > 8,
            "the actual page must contain molecular strokes and labels"
        );
        let point = workspace.button("Double bond")?.center();
        workspace.frame(vec![Event::PointerMoved(point), click(point, true)]);
        workspace.frame(vec![click(point, false)]);
        let after = workspace.frame(vec![]);
        let after_canvas = assert_layout(&workspace, &after)?;
        assert_eq!(after_canvas, canvas);
        assert_eq!(workspace.state()?, before);
        assert_eq!(workspace.engine.content(), &content);
        assert_eq!(chemical_paint(&after, after_canvas), ink);
        workspace.assert_three_pairs()?;
        for (id, expected) in [
            ("tool.bond.single", false),
            ("tool.bond.double", true),
            ("tool.bond.triple", false),
            ("tool.selection", false),
        ] {
            let control = workspace
                .engine
                .controls()
                .into_iter()
                .find(|control| control.id == id)
                .ok_or("registered tool binding")?;
            assert_eq!(control.selected, Some(expected));
        }
    }
    Ok(())
}
