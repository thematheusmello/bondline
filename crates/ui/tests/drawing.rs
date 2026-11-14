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

fn escape(pressed: bool) -> Event {
    Event::Key {
        key: Key::Escape,
        physical_key: Some(Key::Escape),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn assert_blocked_press_is_not_a_gesture(
    block: Event,
    unblock: Event,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    frame(&context, &mut engine, &mut drawing, vec![]);
    let begin = Pos2::new(420.0, 300.0);
    let end = Pos2::new(500.0, 350.0);
    let document = engine.stored_document()?.clone();
    let before = serde_json::to_vec(&engine.state())?;
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![
            block.clone(),
            button(begin, true),
            Event::PointerMoved(end),
            unblock.clone(),
            button(end, false),
        ],
    );
    assert_eq!(
        engine.stored_document()?,
        &document,
        "a press while blocked cannot start a bond"
    );
    assert_eq!(
        serde_json::to_vec(&engine.state())?,
        before,
        "rejected input must preserve revision/history/file state"
    );
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![
            block,
            unblock,
            Event::PointerMoved(begin),
            button(begin, true),
            Event::PointerMoved(end),
            button(end, false),
        ],
    );
    assert_eq!(
        engine.state().revision,
        1,
        "a complete drag after unblock in the same frame must work"
    );
    assert_eq!(engine.state().undo_depth, 1);
    assert_eq!(
        engine
            .document()?
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
fn batched_escape_held_press_does_not_create_a_bond() -> Result<(), Box<dyn std::error::Error>> {
    assert_blocked_press_is_not_a_gesture(escape(true), escape(false))
}

#[test]
fn batched_unfocused_press_does_not_create_a_bond() -> Result<(), Box<dyn std::error::Error>> {
    assert_blocked_press_is_not_a_gesture(Event::WindowFocused(false), Event::WindowFocused(true))
}

#[test]
fn a_held_escape_repeat_does_not_enable_an_earlier_press() -> Result<(), Box<dyn std::error::Error>>
{
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    frame(&context, &mut engine, &mut drawing, vec![]);
    frame(&context, &mut engine, &mut drawing, vec![escape(true)]);
    let before = serde_json::to_vec(&engine.state())?;
    let document = engine.stored_document()?.clone();
    let begin = Pos2::new(420.0, 300.0);
    let end = Pos2::new(500.0, 350.0);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![
            button(begin, true),
            Event::PointerMoved(end),
            escape(true),
            escape(false),
            button(end, false),
        ],
    );
    assert_eq!(engine.stored_document()?, &document);
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
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
    assert_eq!(engine.state().revision, 1);
    assert_eq!(engine.state().undo_depth, 1);
    Ok(())
}

#[test]
fn completed_batched_drag_survives_a_later_move_outside_the_canvas()
-> Result<(), Box<dyn std::error::Error>> {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    let mut drawing = DrawingState::default();
    frame(&context, &mut engine, &mut drawing, vec![]);
    let original = engine.stored_document()?.clone();
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
            Event::PointerMoved(Pos2::new(20.0, 10.0)),
        ],
    );
    assert_eq!(
        engine.state().revision,
        1,
        "the valid press and release must not depend on a later pointer position"
    );
    assert_eq!(engine.state().undo_depth, 1);
    assert_eq!(
        engine
            .document()?
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .len(),
        1
    );
    let created = engine.stored_document()?.clone();
    engine.dispatch("history.undo", serde_json::json!({}))?;
    assert_eq!(engine.stored_document()?, &original);
    engine.dispatch("history.redo", serde_json::json!({}))?;
    assert_eq!(engine.stored_document()?, &created);
    Ok(())
}

#[test]
fn an_initial_press_covered_by_a_foreground_area_never_starts_a_bond()
-> Result<(), Box<dyn std::error::Error>> {
    for end in [Pos2::new(460.0, 320.0), Pos2::new(500.0, 350.0)] {
        let context = egui::Context::default();
        configure_context(&context);
        let mut engine = Engine::default();
        let mut drawing = DrawingState::default();
        let before = serde_json::to_vec(&engine.state())?;
        let original = engine.stored_document()?.clone();
        let begin = Pos2::new(420.0, 300.0);
        let drag = vec![
            Event::PointerMoved(begin),
            button(begin, true),
            Event::PointerMoved(end),
            button(end, false),
        ];
        // Establish the foreground area's size/order before delivering input.
        for (index, events) in [vec![], vec![], drag].into_iter().enumerate() {
            context.begin_pass(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(1000.0, 700.0),
                )),
                events,
                ..Default::default()
            });
            let overlay = egui::Area::new(egui::Id::new("foreground-input-test"))
                .order(egui::Order::Foreground)
                .fixed_pos(Pos2::new(400.0, 280.0))
                .show(&context, |ui| {
                    ui.set_min_size(egui::vec2(80.0, 80.0));
                    ui.label("Foreground");
                });
            assert!(overlay.response.rect.contains(begin));
            if index == 2 {
                assert_eq!(context.layer_id_at(begin), Some(overlay.response.layer_id));
            }
            let mut error = None;
            show_workspace(&context, &mut engine, &mut error, &mut drawing);
            let _ = context.end_pass();
            assert!(error.is_none(), "{error:?}");
        }
        assert_eq!(
            engine.stored_document()?,
            &original,
            "a foreground-covered press must stay rejected even when the release moves outside that area"
        );
        assert_eq!(serde_json::to_vec(&engine.state())?, before);
    }
    Ok(())
}

#[test]
fn reopened_native_profile_is_visible_and_editable_in_the_real_workspace_renderer()
-> Result<(), Box<dyn std::error::Error>> {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml",serde_json::json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/bondline-v0.cdxml")}))?;
    assert!(
        engine.stored_document()?.is_fully_mapped(),
        "pinned native profile must remain fully mapped on file import"
    );
    let mut drawing = DrawingState::default();
    // Isolate source completeness from the renderer's separately tested minimum glyph scale.
    context.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1000.0, 900.0),
        )),
        ..Default::default()
    });
    show_workspace(&context, &mut engine, &mut None, &mut drawing);
    let output = context.end_pass();
    let messages = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.text()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,egui::Shape::Rect(rect) if rect.fill == egui::Color32::WHITE)),"a complete native source profile must paint its editable page: {messages:?}");
    let before = engine
        .document()?
        .pages()
        .first()
        .ok_or("page")?
        .fragments()
        .len();
    let begin = Pos2::new(420.0, 500.0);
    let end = Pos2::new(500.0, 500.0);
    context.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1000.0, 900.0),
        )),
        events: vec![
            Event::PointerMoved(begin),
            button(begin, true),
            Event::PointerMoved(end),
            button(end, false),
        ],
        ..Default::default()
    });
    show_workspace(&context, &mut engine, &mut None, &mut drawing);
    let _ = context.end_pass();
    assert_eq!(
        engine
            .document()?
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .len(),
        before + 1
    );
    Ok(())
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
            .document()?
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .len(),
        1
    );
    let original = engine.stored_document()?.clone();
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
    assert_eq!(engine.stored_document()?, &original);
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
            .document()?
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
            .document()?
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
