use bondline_commands::Engine;
use bondline_ui::{DrawingState, configure_context, show_workspace};
use eframe::egui::{self, Event, PointerButton, Pos2};
use serde_json::json;

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the existing isolated file test directory"
)]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] =
    b"\n \xffUI_SECRET_METADATA_73\r\nCC \xff\0UI_SECRET_METADATA_73\nC#N second\r\n";

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

fn paper(output: &egui::FullOutput) -> Option<egui::Rect> {
    output.shapes.iter().find_map(|shape| match &shape.shape {
        egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_gray(196) => Some(rect.rect),
        _ => None,
    })
}

fn labels(output: &egui::FullOutput) -> String {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.text()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn smiles_open_clears_held_gesture_hover_and_paper_without_disclosing_record_bytes() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("collection.smiles");
    std::fs::write(&input, SOURCE)?;
    let context = egui::Context::default();
    configure_context(&context);
    context.enable_accesskit();
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    engine.dispatch(
        "bond.create",
        json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}),
    )?;
    let output = frame(&context, &mut engine, &mut drawing, vec![]);
    let rect = paper(&output).ok_or("original drawing paper")?;
    let size = engine.document()?.pages()[0].size();
    let point = rect.min
        + egui::vec2(
            rect.width() * 140.0 / size.width_pt() as f32,
            rect.height() * 200.0 / size.height_pt() as f32,
        );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    assert!(drawing.hover_target().is_some());
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    engine.dispatch(
        "document.open_smiles",
        json!({"path":input,"discard_unsaved":true}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(before["document"], json!(null));
    assert_eq!(before["smiles_file"]["record_count"], 2);
    let end = point + egui::vec2(50.0, 50.0);
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
    assert!(drawing.hover_target().is_none());
    assert!(paper(&output).is_none());
    let text = labels(&output);
    assert!(text.contains("SMILES file: 2 records"));
    assert!(text.contains(&format!("{} bytes", SOURCE.len())));
    assert!(text.contains("Chemical preview is unavailable"));
    assert!(!text.contains("UI_SECRET_METADATA_73"));
    assert!(!text.contains("Drag empty paper"));
    let tree = format!(
        "{:?}",
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("accessible summary")?
    );
    assert!(tree.contains("SMILES file: 2 records"));
    assert!(tree.contains("Chemical preview is unavailable"));
    assert!(!tree.contains("UI_SECRET_METADATA_73"));
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(std::fs::read(&input)?, SOURCE);
    engine.dispatch("document.open_cdxml",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/bondline-v0.cdxml")}))?;
    let before = serde_json::to_value(engine.state())?;
    for _ in 0..3 {
        let output = frame(&context, &mut engine, &mut drawing, vec![]);
        assert!(drawing.hover_target().is_none());
        assert!(paper(&output).is_some());
        assert!(!labels(&output).contains("SMILES file:"));
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

#[test]
fn zero_record_smiles_is_a_source_summary_without_a_synthetic_drawing() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("empty.smi");
    let source = b"\n \xffUI_SECRET_METADATA_73\r\n";
    std::fs::write(&input, source)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    let context = egui::Context::default();
    configure_context(&context);
    let output = frame(&context, &mut engine, &mut DrawingState::default(), vec![]);
    assert!(paper(&output).is_none());
    let text = labels(&output);
    assert!(text.contains("SMILES file: 0 records"));
    assert!(text.contains("Chemical preview is unavailable"));
    assert!(!text.contains("UI_SECRET_METADATA_73"));
    assert_eq!(before["document"], json!(null));
    assert_eq!(before["smiles_file"]["record_count"], 0);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}
