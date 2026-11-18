use bondline_commands::Engine;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, PageSize, Point2,
};
use bondline_ui::{DrawingState, configure_context, show_workspace};
use eframe::egui::{self, Event};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn engine() -> TestResult<Engine> {
    let mut stored = Engine::default().stored_document()?.clone();
    let id = PageId::new(1).ok_or("page")?;
    stored
        .model_mut()
        .set_page_size(id, PageSize::new(60.0, 60.0)?)?;
    let atoms = [[10.5, 20.5], [30.5, 20.5], [30.5, 40.5]]
        .into_iter()
        .enumerate()
        .map(|(i, [x, y])| {
            Ok(Atom::new(
                AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(x, y)?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let bonds = [[1, 2], [2, 3]]
        .into_iter()
        .enumerate()
        .map(|(i, [a, b])| {
            Ok(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(u32::try_from(i + 1)?).ok_or("bond")?,
                begin: AtomId::new(a).ok_or("atom")?,
                end: AtomId::new(b).ok_or("atom")?,
                order: BondOrder::Single,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    stored.model_mut().append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(Engine::from_document(stored))
}

fn frame(
    context: &egui::Context,
    engine: &mut Engine,
    drawing: &mut DrawingState,
    events: Vec<Event>,
) -> egui::FullOutput {
    frame_at(
        context,
        engine,
        drawing,
        events,
        egui::vec2(1000.0, 900.0),
        true,
    )
}

fn frame_at(
    context: &egui::Context,
    engine: &mut Engine,
    drawing: &mut DrawingState,
    events: Vec<Event>,
    size: egui::Vec2,
    focused: bool,
) -> egui::FullOutput {
    frame_dpi(context, engine, drawing, events, size, focused, 1.0)
}

fn frame_dpi(
    context: &egui::Context,
    engine: &mut Engine,
    drawing: &mut DrawingState,
    events: Vec<Event>,
    size: egui::Vec2,
    focused: bool,
    dpi: f32,
) -> egui::FullOutput {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
        events,
        focused,
        ..Default::default()
    };
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .native_pixels_per_point = Some(dpi);
    frame_input(context, engine, drawing, input)
}

fn frame_input(
    context: &egui::Context,
    engine: &mut Engine,
    drawing: &mut DrawingState,
    input: egui::RawInput,
) -> egui::FullOutput {
    context.begin_pass(input);
    let mut error = None;
    show_workspace(context, engine, &mut error, drawing);
    let output = context.end_pass();
    assert!(error.is_none(), "{error:?}");
    output
}

fn paper(output: &egui::FullOutput) -> TestResult<egui::Rect> {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_gray(196) => {
                Some(rect.rect)
            }
            _ => None,
        })
        .ok_or_else(|| "paper border".into())
}

fn document_point(paper: egui::Rect, x: f32, y: f32) -> egui::Pos2 {
    paper.min + egui::vec2(paper.width() * x / 60.0, paper.height() * y / 60.0)
}

#[test]
fn supported_graph_has_a_real_transient_atom_hover_outline() -> TestResult {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = engine()?;
    let mut drawing = DrawingState::default();
    let initial = frame(&context, &mut engine, &mut drawing, vec![]);
    let paper = paper(&initial)?;
    let point = paper.min + egui::vec2(paper.width() * 10.5 / 60.0, paper.height() * 20.5 / 60.0);
    let before = serde_json::to_vec(&engine.state())?;
    let output = frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    assert!(output.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Circle(circle) if circle.center.distance(point)<0.01 && circle.stroke.color==egui::Color32::from_rgb(40,120,180))),"hover must paint an original outline around the selected atom, outside the shared capture scene");
    assert_eq!(
        serde_json::to_vec(&engine.state())?,
        before,
        "hover is not an authoring edit"
    );
    assert!(
        matches!(drawing.hover_target(),Some(bondline_render::HitTarget::Atom {atom_id,..}) if atom_id.get()==1)
    );
    let middle = document_point(paper, 20.5, 20.5);
    let output = frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(middle)],
    );
    assert!(
        matches!(drawing.hover_target(),Some(bondline_render::HitTarget::Bond {bond_id,..}) if bond_id.get()==1)
    );
    assert!(output.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Path(path) if path.closed && path.stroke.color==eframe::epaint::ColorMode::Solid(egui::Color32::from_rgb(40,120,180)))));
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(document_point(paper, 55.0, 5.0))],
    );
    assert_eq!(drawing.hover_target(), None);
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    Ok(())
}

#[test]
fn hover_invalidates_on_revision_viewport_focus_and_pointer_loss() -> TestResult {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = engine()?;
    let mut drawing = DrawingState::default();
    let output = frame(&context, &mut engine, &mut drawing, vec![]);
    let point = document_point(paper(&output)?, 10.5, 20.5);
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
        vec![Event::PointerGone],
    );
    assert_eq!(drawing.hover_target(), None);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    frame_at(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::WindowFocused(false)],
        egui::vec2(1000.0, 900.0),
        false,
    );
    assert_eq!(drawing.hover_target(), None);
    frame_at(
        &context,
        &mut engine,
        &mut drawing,
        vec![],
        egui::vec2(1000.0, 900.0),
        false,
    );
    assert_eq!(drawing.hover_target(), None);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    engine.dispatch(
        "page.set_size",
        serde_json::json!({"page_id":1,"width_pt":70,"height_pt":70}),
    )?;
    frame(&context, &mut engine, &mut drawing, vec![]);
    assert_eq!(drawing.hover_target(), None);
    engine.dispatch("history.undo", serde_json::json!({}))?;
    let output = frame(&context, &mut engine, &mut drawing, vec![]);
    let point = document_point(paper(&output)?, 10.5, 20.5);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    assert!(drawing.hover_target().is_some());
    frame_at(
        &context,
        &mut engine,
        &mut drawing,
        vec![],
        egui::vec2(1100.0, 900.0),
        true,
    );
    // A square page can retain its paper size on a horizontal resize; the actual viewport key still changes.
    assert_eq!(drawing.hover_target(), None);
    Ok(())
}

#[test]
fn two_device_pixels_per_logical_point_does_not_double_pointer_coordinates() -> TestResult {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = engine()?;
    let mut drawing = DrawingState::default();
    let output = frame_dpi(
        &context,
        &mut engine,
        &mut drawing,
        vec![],
        egui::vec2(1000.0, 900.0),
        true,
        2.0,
    );
    assert_eq!(context.pixels_per_point(), 2.0);
    let point = document_point(paper(&output)?, 30.5, 40.5);
    frame_dpi(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
        egui::vec2(1000.0, 900.0),
        true,
        2.0,
    );
    assert!(
        matches!(drawing.hover_target(),Some(bondline_render::HitTarget::Atom {atom_id,..}) if atom_id.get()==3)
    );
    Ok(())
}

#[test]
fn revision_invalidation_stays_clear_through_idle_frames_until_fresh_position() -> TestResult {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = engine()?;
    let mut drawing = DrawingState::default();
    let output = frame(&context, &mut engine, &mut drawing, vec![]);
    let point = document_point(paper(&output)?, 10.5, 20.5);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    assert!(drawing.hover_target().is_some());
    engine.dispatch(
        "page.set_size",
        serde_json::json!({"page_id":1,"width_pt":70,"height_pt":70}),
    )?;
    engine.dispatch("history.undo", serde_json::json!({}))?;
    let before = serde_json::to_vec(&engine.state())?;
    for idle in 0..3 {
        frame(&context, &mut engine, &mut drawing, vec![]);
        assert_eq!(drawing.hover_target(), None, "revision idle frame {idle}");
    }
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    assert!(
        matches!(drawing.hover_target(), Some(bondline_render::HitTarget::Atom { atom_id, .. }) if atom_id.get() == 1)
    );
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    Ok(())
}

fn escape(pressed: bool) -> Event {
    Event::Key {
        key: egui::Key::Escape,
        physical_key: Some(egui::Key::Escape),
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

#[test]
fn escape_invalidation_stays_clear_after_release_and_idle_frames() -> TestResult {
    for same_frame_release in [false, true] {
        let context = egui::Context::default();
        configure_context(&context);
        let mut engine = engine()?;
        let mut drawing = DrawingState::default();
        let output = frame(&context, &mut engine, &mut drawing, vec![]);
        let point = document_point(paper(&output)?, 10.5, 20.5);
        frame(
            &context,
            &mut engine,
            &mut drawing,
            vec![Event::PointerMoved(point)],
        );
        assert!(drawing.hover_target().is_some());
        let before = serde_json::to_vec(&engine.state())?;
        let events = if same_frame_release {
            vec![escape(true), escape(false)]
        } else {
            vec![escape(true)]
        };
        frame(&context, &mut engine, &mut drawing, events);
        assert_eq!(
            drawing.hover_target(),
            None,
            "Escape down+up={same_frame_release}"
        );
        if !same_frame_release {
            frame(&context, &mut engine, &mut drawing, vec![escape(false)]);
            assert_eq!(
                drawing.hover_target(),
                None,
                "Escape release is not a fresh position"
            );
        }
        for idle in 0..3 {
            frame(&context, &mut engine, &mut drawing, vec![]);
            assert_eq!(drawing.hover_target(), None, "Escape idle frame {idle}");
        }
        frame(
            &context,
            &mut engine,
            &mut drawing,
            vec![Event::PointerMoved(point)],
        );
        assert!(drawing.hover_target().is_some());
        assert_eq!(serde_json::to_vec(&engine.state())?, before);
    }
    Ok(())
}

#[test]
fn focus_return_without_positional_input_keeps_hover_clear() -> TestResult {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = engine()?;
    let mut drawing = DrawingState::default();
    let output = frame(&context, &mut engine, &mut drawing, vec![]);
    let point = document_point(paper(&output)?, 10.5, 20.5);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    assert!(drawing.hover_target().is_some());
    let before = serde_json::to_vec(&engine.state())?;
    frame_at(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::WindowFocused(false)],
        egui::vec2(1000.0, 900.0),
        false,
    );
    assert_eq!(drawing.hover_target(), None);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::WindowFocused(true)],
    );
    assert_eq!(
        drawing.hover_target(),
        None,
        "focus gain is not a fresh position"
    );
    for idle in 0..3 {
        frame(&context, &mut engine, &mut drawing, vec![]);
        assert_eq!(drawing.hover_target(), None, "focus idle frame {idle}");
    }
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    assert!(drawing.hover_target().is_some());
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    Ok(())
}

fn native_input(origin: egui::Pos2, dpi: f32, inner: bool) -> egui::RawInput {
    let size = egui::vec2(1000.0, 900.0);
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
        ..Default::default()
    };
    let info = input.viewports.entry(egui::ViewportId::ROOT).or_default();
    info.native_pixels_per_point = Some(dpi);
    let rect = Some(egui::Rect::from_min_size(origin, size));
    info.inner_rect = if inner { rect } else { None };
    info.outer_rect = rect;
    input
}

fn assert_context_invalidation(mut before: egui::RawInput, after: egui::RawInput) -> TestResult {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = engine()?;
    let mut drawing = DrawingState::default();
    let output = frame_input(&context, &mut engine, &mut drawing, before.clone());
    let point = document_point(paper(&output)?, 10.5, 20.5);
    before.events = vec![Event::PointerMoved(point)];
    frame_input(&context, &mut engine, &mut drawing, before);
    assert!(drawing.hover_target().is_some());
    let before = serde_json::to_vec(&engine.state())?;
    let mut output = None;
    for idle in 0..3 {
        output = Some(frame_input(
            &context,
            &mut engine,
            &mut drawing,
            after.clone(),
        ));
        assert_eq!(drawing.hover_target(), None, "context idle frame {idle}");
    }
    let point = document_point(paper(&output.ok_or("last output")?)?, 10.5, 20.5);
    let mut fresh = after;
    fresh.events = vec![Event::PointerMoved(point)];
    frame_input(&context, &mut engine, &mut drawing, fresh);
    assert!(
        matches!(drawing.hover_target(), Some(bondline_render::HitTarget::Atom { atom_id, .. }) if atom_id.get() == 1)
    );
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    Ok(())
}

#[test]
fn native_inner_origin_change_requires_a_fresh_position() -> TestResult {
    assert_context_invalidation(
        native_input(egui::pos2(100.0, 100.0), 1.0, true),
        native_input(egui::pos2(-1500.0, 180.0), 1.0, true),
    )
}

#[test]
fn native_outer_origin_fallback_requires_a_fresh_position() -> TestResult {
    assert_context_invalidation(
        native_input(egui::pos2(100.0, 100.0), 1.0, false),
        native_input(egui::pos2(-1500.0, 180.0), 1.0, false),
    )
}

#[test]
fn native_pixels_per_point_change_requires_a_fresh_position() -> TestResult {
    assert_context_invalidation(
        native_input(egui::pos2(100.0, 100.0), 1.0, true),
        native_input(egui::pos2(100.0, 100.0), 2.0, true),
    )
}

#[test]
fn canvas_available_origin_change_requires_a_fresh_position() -> TestResult {
    let before = native_input(egui::pos2(100.0, 100.0), 1.0, true);
    let mut after = before.clone();
    after.screen_rect = after
        .screen_rect
        .map(|rect| rect.translate(egui::vec2(0.25, 0.25)));
    assert_context_invalidation(before, after)
}

#[test]
fn positional_events_rearm_in_order_and_jitter_click_does_not_edit() -> TestResult {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = engine()?;
    let mut drawing = DrawingState::default();
    let output = frame(&context, &mut engine, &mut drawing, vec![]);
    let point = document_point(paper(&output)?, 10.5, 20.5);
    let before = serde_json::to_vec(&engine.state())?;
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point), Event::PointerGone],
    );
    assert_eq!(drawing.hover_target(), None);
    for _ in 0..3 {
        frame(&context, &mut engine, &mut drawing, vec![]);
        assert_eq!(drawing.hover_target(), None);
    }
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerGone, Event::PointerMoved(point)],
    );
    assert!(drawing.hover_target().is_some());
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![escape(true), escape(false)],
    );
    assert_eq!(drawing.hover_target(), None);
    let events = [true, false]
        .map(|pressed| Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        })
        .to_vec();
    frame(&context, &mut engine, &mut drawing, events);
    assert!(
        matches!(drawing.hover_target(), Some(bondline_render::HitTarget::Atom { atom_id, .. }) if atom_id.get() == 1)
    );
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    Ok(())
}

fn assert_blocked_position_does_not_rearm(
    block: Event,
    unblock: Event,
    button_input: bool,
) -> TestResult {
    let context = egui::Context::default();
    configure_context(&context);
    let mut engine = engine()?;
    let mut drawing = DrawingState::default();
    let output = frame(&context, &mut engine, &mut drawing, vec![]);
    let point = document_point(paper(&output)?, 10.5, 20.5);
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![Event::PointerMoved(point)],
    );
    assert!(drawing.hover_target().is_some());
    let before = serde_json::to_vec(&engine.state())?;
    let mut events = vec![block.clone()];
    if button_input {
        events.extend([true, false].map(|pressed| Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }));
    } else {
        events.push(Event::PointerMoved(point));
    }
    events.push(unblock.clone());
    frame(&context, &mut engine, &mut drawing, events);
    assert_eq!(
        drawing.hover_target(),
        None,
        "input while blocked must not rearm"
    );
    for idle in 0..3 {
        frame(&context, &mut engine, &mut drawing, vec![]);
        assert_eq!(
            drawing.hover_target(),
            None,
            "blocked input idle frame {idle}"
        );
    }
    frame(
        &context,
        &mut engine,
        &mut drawing,
        vec![block, unblock, Event::PointerMoved(point)],
    );
    assert!(
        matches!(drawing.hover_target(), Some(bondline_render::HitTarget::Atom { atom_id, .. }) if atom_id.get() == 1),
        "a position after unblock in the same frame must work"
    );
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    Ok(())
}

#[test]
fn escape_down_move_up_does_not_rearm_hover() -> TestResult {
    assert_blocked_position_does_not_rearm(escape(true), escape(false), false)
}

#[test]
fn focus_loss_move_gain_does_not_rearm_hover() -> TestResult {
    assert_blocked_position_does_not_rearm(
        Event::WindowFocused(false),
        Event::WindowFocused(true),
        false,
    )
}

#[test]
fn buttons_while_escape_or_focus_blocked_do_not_rearm_hover() -> TestResult {
    assert_blocked_position_does_not_rearm(escape(true), escape(false), true)?;
    assert_blocked_position_does_not_rearm(
        Event::WindowFocused(false),
        Event::WindowFocused(true),
        true,
    )
}

#[test]
fn positions_before_release_or_focus_gain_respect_prior_frame_state() -> TestResult {
    for escape_case in [true, false] {
        let context = egui::Context::default();
        configure_context(&context);
        let mut engine = engine()?;
        let mut drawing = DrawingState::default();
        let output = frame(&context, &mut engine, &mut drawing, vec![]);
        let point = document_point(paper(&output)?, 10.5, 20.5);
        let before = serde_json::to_vec(&engine.state())?;
        let block = if escape_case {
            escape(true)
        } else {
            Event::WindowFocused(false)
        };
        frame_at(
            &context,
            &mut engine,
            &mut drawing,
            vec![block.clone()],
            egui::vec2(1000.0, 900.0),
            escape_case,
        );
        let mut events = vec![Event::PointerMoved(point)];
        if escape_case {
            // egui marks this as a repeat of the previous frame's held Escape.
            events.extend([block, escape(false)]);
        } else {
            events.push(Event::WindowFocused(true));
        }
        frame(&context, &mut engine, &mut drawing, events);
        assert_eq!(
            drawing.hover_target(),
            None,
            "the earlier position was still blocked"
        );
        for _ in 0..3 {
            frame(&context, &mut engine, &mut drawing, vec![]);
            assert_eq!(drawing.hover_target(), None);
        }
        frame(
            &context,
            &mut engine,
            &mut drawing,
            vec![Event::PointerMoved(point)],
        );
        assert!(drawing.hover_target().is_some());
        assert_eq!(serde_json::to_vec(&engine.state())?, before);
    }
    Ok(())
}
