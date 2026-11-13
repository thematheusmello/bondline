use bondline_commands::Engine;
use bondline_ui::{DrawingState, configure_context, show_workspace};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2};

fn frame(
    context: &egui::Context,
    engine: &mut Engine,
    drawing: &mut DrawingState,
    events: Vec<Event>,
) {
    context.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1000.0, 700.0),
        )),
        events,
        ..Default::default()
    });
    let mut error = None;
    show_workspace(context, engine, &mut error, drawing);
    let _output = context.end_pass();
    assert!(error.is_none(), "{error:?}");
}
fn button(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn a_drag_commits_one_bond_but_escape_and_stale_revision_cancel_without_edits()
-> Result<(), Box<dyn std::error::Error>> {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    frame(&context, &mut engine, &mut drawing, vec![]);
    let begin = Pos2::new(420.0, 300.0);
    let end = Pos2::new(500.0, 350.0);
    for events in [
        vec![Event::PointerMoved(begin), button(begin, true)],
        vec![Event::PointerMoved(end)],
    ] {
        frame(&context, &mut engine, &mut drawing, events);
        assert_eq!(engine.state().revision, 0);
    }
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![button(end, false)],
    );
    assert_eq!(engine.state().revision, 1);
    assert_eq!(engine.state().undo_depth, 1);
    assert_eq!(
        engine
            .document()
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .len(),
        1
    );
    let original = engine.stored_document().clone();
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(begin), button(begin, true)],
    );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(end)],
    );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::Key {
            key: Key::Escape,
            physical_key: Some(Key::Escape),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![button(end, false)],
    );
    assert_eq!(engine.stored_document(), &original);
    assert_eq!(engine.state().revision, 1);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::Key {
            key: Key::Escape,
            physical_key: Some(Key::Escape),
            pressed: false,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(begin), button(begin, true)],
    );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(end)],
    );
    engine.dispatch(
        "page.set_size",
        serde_json::json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![button(end, false)],
    );
    assert_eq!(engine.state().revision, 2);
    assert_eq!(
        engine
            .document()
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .len(),
        1
    );
    Ok(())
}

#[test]
fn unsupported_drawing_errors_stay_visible_inside_the_canvas()
-> Result<(), Box<dyn std::error::Error>> {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    for _ in 0..2 {
        engine.dispatch("bond.create",serde_json::json!({"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":200,"end_y_pt":200}))?;
    }
    context.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1000.0, 700.0),
        )),
        ..Default::default()
    });
    show_workspace(&context, &mut engine, &mut None, &mut drawing);
    let output = context.end_pass();
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,egui::Shape::Text(text)
        if text.galley.text().contains("chemical label") && shape.clip_rect.intersects(shape.shape.visual_bounding_rect()))),"render error was clipped out of the canvas");
    Ok(())
}

#[test]
fn batched_native_pointer_events_commit_a_complete_drag() -> Result<(), Box<dyn std::error::Error>>
{
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    frame(&context, &mut engine, &mut drawing, vec![]);
    let begin = Pos2::new(420.0, 300.0);
    let end = Pos2::new(500.0, 350.0);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![
            Event::PointerMoved(begin),
            button(begin, true),
            Event::PointerMoved(end),
            button(end, false),
        ],
    );
    assert_eq!(
        engine.state().revision,
        1,
        "a complete native drag was lost between frames"
    );
    assert_eq!(engine.state().undo_depth, 1);
    Ok(())
}

#[test]
fn multiple_batched_drags_keep_separate_history_steps() -> Result<(), Box<dyn std::error::Error>> {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    frame(&context, &mut engine, &mut drawing, vec![]);
    let events = [
        (Pos2::new(420.0, 300.0), Pos2::new(500.0, 350.0)),
        (Pos2::new(420.0, 450.0), Pos2::new(500.0, 500.0)),
    ]
    .into_iter()
    .flat_map(|(a, b)| {
        [
            Event::PointerMoved(a),
            button(a, true),
            Event::PointerMoved(b),
            button(b, false),
        ]
    })
    .collect();
    frame(&context, &mut engine, &mut drawing, events);
    assert_eq!(engine.state().revision, 2);
    assert_eq!(engine.state().undo_depth, 2);
    assert_eq!(
        engine
            .document()
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .len(),
        2
    );
    Ok(())
}

#[test]
fn focus_loss_outside_release_and_click_jitter_do_not_create_bonds()
-> Result<(), Box<dyn std::error::Error>> {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    frame(&context, &mut engine, &mut drawing, vec![]);
    let a = Pos2::new(420.0, 300.0);
    let b = Pos2::new(500.0, 350.0);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(a), button(a, true)],
    );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(b), Event::WindowFocused(false)],
    );
    frame(&context, &mut engine, &mut drawing, vec![button(b, false)]);
    assert_eq!(engine.state().revision, 0);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![
            Event::WindowFocused(true),
            Event::PointerMoved(a),
            button(a, true),
        ],
    );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![
            Event::PointerMoved(Pos2::new(5.0, 100.0)),
            button(Pos2::new(5.0, 100.0), false),
        ],
    );
    assert_eq!(engine.state().revision, 0);
    let nearby = a + egui::vec2(1.0, 1.0);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![
            Event::PointerMoved(a),
            button(a, true),
            Event::PointerMoved(nearby),
            button(nearby, false),
        ],
    );
    assert_eq!(engine.state().revision, 0);
    assert_eq!(engine.state().undo_depth, 0);
    Ok(())
}
