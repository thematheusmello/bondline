use bondline_commands::Engine;
use bondline_ui::{DrawingState, configure_context, show_workspace};
use eframe::egui::{self, Event, PointerButton, Pos2};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn frame(
    context: &egui::Context,
    engine: &mut Engine,
    drawing: &mut DrawingState,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1000.0, 900.0),
        )),
        events,
        ..Default::default()
    });
    show_workspace(context, engine, &mut None, drawing);
    context.end_pass()
}

#[test]
fn sdf_open_clears_old_held_gestures_hits_and_paper_then_reports_the_complete_collection()
-> TestResult {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}),
    )?;
    let output = frame(&context, &mut engine, &mut drawing, vec![]);
    let paper = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_gray(196) => {
                Some(rect.rect)
            }
            _ => None,
        })
        .ok_or("drawing paper")?;
    let size = engine.document()?.pages()[0].size();
    let begin = paper.min
        + egui::vec2(
            paper.width() * 140.0 / size.width_pt() as f32,
            paper.height() * 200.0 / size.height_pt() as f32,
        );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(begin)],
    );
    assert!(drawing.hover_target().is_some());
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerButton {
            pos: begin,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    engine.dispatch("document.open_sdf",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf"),"discard_unsaved":true}))?;
    let before = serde_json::to_value(engine.state())?;
    let end = begin + egui::vec2(50.0, 50.0);
    let output = frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![
            Event::PointerMoved(end),
            Event::PointerButton {
                pos: end,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(before["document"], json!(null));
    assert_eq!(before["collection"]["record_count"], 2);
    assert!(drawing.hover_target().is_none());
    assert!(!output.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Rect(rect) if rect.fill==egui::Color32::from_gray(196))));
    let labels = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.text()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(labels.contains("SDF collection"));
    assert!(labels.contains("2 records"));
    assert!(labels.contains("Chemical preview is unavailable"));
    assert!(!labels.contains("Drag empty paper"));
    for value in ["second record", "<DUP>", "carbon\nisotope"] {
        assert!(!labels.contains(value));
    }
    engine.dispatch("document.open_cdxml",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/bondline-v0.cdxml")}))?;
    let before = serde_json::to_value(engine.state())?;
    for _ in 0..3 {
        let output = frame(&context, &mut engine, &mut drawing, vec![]);
        assert!(drawing.hover_target().is_none());
        assert!(output.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Rect(rect) if rect.fill==egui::Color32::from_gray(196))));
    }
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerButton {
            pos: end,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}
