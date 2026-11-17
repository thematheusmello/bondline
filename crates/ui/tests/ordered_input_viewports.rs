//! Synthetic SDK checks for repeated observations, viewports and discarded passes.
use eframe::egui::{
    self, Context, Event, EventFilter, Modifiers, RawInput, ViewportId, ViewportInfo,
};
use std::num::NonZeroUsize;

fn queued(
    viewport: ViewportId,
    child: ViewportId,
    modifiers: Modifiers,
    events: Vec<Event>,
) -> RawInput {
    let mut raw = RawInput {
        viewport_id: viewport,
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(640.0, 480.0),
        )),
        modifiers,
        events,
        focused: true,
        ..Default::default()
    };
    raw.viewports.insert(
        child,
        ViewportInfo {
            parent: Some(ViewportId::ROOT),
            ..Default::default()
        },
    );
    raw
}
fn retain_child(context: &Context, child: ViewportId) {
    // Only declare a deferred viewport in this bare Context's output. No backend runs it.
    context.show_viewport_deferred(child, egui::ViewportBuilder::default(), |_, _| {});
}

#[test]
fn repeated_same_state_observations_are_not_deduplicated_or_recast_as_keys() {
    let context = Context::default();
    let child = ViewportId::from_hash_of("unused-duplicate-control-child");
    let control = Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    };
    let expected = vec![
        Event::ModifiersChanged(control),
        Event::ModifiersChanged(control),
        Event::PointerMoved(egui::pos2(52.0, 67.0)),
        Event::ModifiersChanged(control),
    ];
    let mut raw = queued(ViewportId::ROOT, child, control, vec![expected[0].clone()]);
    raw.append(queued(
        ViewportId::ROOT,
        child,
        control,
        expected[1..].to_vec(),
    ));
    assert_eq!(raw.events, expected);
    context.begin_pass(raw.take());
    context.input(|state| {
        assert_eq!(state.modifiers, control);
        assert_eq!(state.events, expected);
        assert_eq!(state.raw.events, expected);
        assert_eq!(state.filtered_events(&EventFilter::default()), expected);
        assert!(state.keys_down.is_empty());
        assert!(
            !state
                .events
                .iter()
                .any(|event| matches!(event, Event::Key { .. }))
        );
    });
    let _ = context.end_pass();
    assert!(raw.events.is_empty());
}

#[test]
fn independent_viewport_queues_drain_and_update_only_their_own_context_input() {
    let root = ViewportId::ROOT;
    let child = ViewportId::from_hash_of("independent-modifier-child");
    assert_ne!(root, child);
    let control = Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    };
    let alt = Modifiers {
        alt: true,
        ..Modifiers::NONE
    };
    let root_events = vec![
        Event::ModifiersChanged(control),
        Event::PointerMoved(egui::pos2(23.0, 31.0)),
    ];
    let child_events = vec![
        Event::ModifiersChanged(alt),
        Event::PointerMoved(egui::pos2(83.0, 97.0)),
    ];
    let mut root_queue = queued(root, child, control, root_events.clone());
    let mut child_queue = queued(child, child, alt, child_events.clone());
    let context = Context::default();
    context.set_embed_viewports(false);

    context.begin_pass(root_queue.take());
    retain_child(&context, child);
    assert_eq!(
        child_queue.events, child_events,
        "taking root cannot drain the child queue"
    );
    context.input_for(root, |state| {
        assert_eq!(state.raw.viewport_id, root);
        assert_eq!(state.modifiers, control);
        assert_eq!(state.events, root_events);
    });
    context.input_for(child, |state| assert!(state.events.is_empty()));
    let _ = context.end_pass();

    context.begin_pass(child_queue.take());
    assert_eq!(context.viewport_id(), child);
    context.input(|state| assert_eq!(state.events, child_events));
    context.input_for(child, |state| {
        assert_eq!(state.raw.viewport_id, child);
        assert_eq!(state.modifiers, alt);
        assert_eq!(state.filtered_events(&EventFilter::default()), child_events);
    });
    context.input_for(root, |state| {
        assert_eq!(state.modifiers, control);
        assert_eq!(state.events, root_events);
    });
    let _ = context.end_pass();

    let later_root = vec![Event::ModifiersChanged(Modifiers::NONE)];
    root_queue.append(queued(root, child, Modifiers::NONE, later_root.clone()));
    assert!(child_queue.events.is_empty());
    context.begin_pass(root_queue.take());
    retain_child(&context, child);
    context.input_for(root, |state| {
        assert_eq!(state.modifiers, Modifiers::NONE);
        assert_eq!(state.events, later_root);
    });
    // input_for reads that viewport's latest pass; it must not substitute root's new input.
    context.input_for(child, |state| {
        assert_eq!(state.modifiers, alt);
        assert_eq!(state.events, child_events);
    });
    let _ = context.end_pass();

    context.begin_pass(child_queue.take());
    context.input_for(child, |state| {
        assert_eq!(state.modifiers, alt);
        assert!(state.events.is_empty() && state.raw.events.is_empty());
        assert!(state.keys_down.is_empty());
    });
    context.input_for(root, |state| {
        assert_eq!(state.modifiers, Modifiers::NONE);
        assert_eq!(state.events, later_root);
    });
    let _ = context.end_pass();
}

#[test]
fn an_explicit_discard_pass_drains_observations_without_replaying_them()
-> Result<(), Box<dyn std::error::Error>> {
    let context = Context::default();
    let child = ViewportId::from_hash_of("unused-discard-control-child");
    let control = Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    };
    let expected = vec![
        Event::ModifiersChanged(control),
        Event::ModifiersChanged(control),
    ];
    let raw = queued(ViewportId::ROOT, child, control, expected.clone());
    let passes = NonZeroUsize::new(2).ok_or("two passes")?;
    context.options_mut(|options| options.max_passes = passes);
    let mut seen = Vec::new();
    let output = context.run(raw, |context| {
        seen.push(context.input(|state| {
            assert_eq!(state.modifiers, control);
            assert_eq!(state.raw.events, state.events);
            assert!(state.keys_down.is_empty());
            state.events.clone()
        }));
        if seen.len() == 1 {
            context.request_discard("independent one-shot input-drain control");
        }
    });
    assert_eq!(output.platform_output.num_completed_passes, 2);
    assert_eq!(seen, vec![expected, vec![]]);
    Ok(())
}
