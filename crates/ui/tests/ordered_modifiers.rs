//! Synthetic public-SDK transport checks; no Engine, native window or chain consumer.
use eframe::egui::{self, Context, Event, EventFilter, Modifiers, PointerButton, RawInput};

fn input(modifiers: Modifiers, events: Vec<Event>) -> RawInput {
    RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(640.0, 480.0),
        )),
        modifiers,
        events,
        focused: true,
        ..Default::default()
    }
}
fn primary(pos: egui::Pos2, pressed: bool, modifiers: Modifiers) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers,
    }
}

#[test]
fn append_take_begin_pass_and_filters_keep_exact_order_beside_the_latest_snapshot() {
    let a = egui::pos2(20.0, 30.0);
    let b = egui::pos2(68.0, 30.0);
    let c = egui::pos2(92.0, 54.0);
    let control = Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    };
    let all = Modifiers {
        alt: true,
        ctrl: true,
        shift: true,
        mac_cmd: true,
        command: true,
    };
    let first = vec![
        Event::PointerMoved(a),
        primary(a, true, Modifiers::NONE),
        Event::ModifiersChanged(control),
        Event::PointerMoved(b),
    ];
    let last = vec![
        Event::ModifiersChanged(all),
        Event::PointerMoved(c),
        Event::ModifiersChanged(Modifiers::NONE),
        primary(c, false, Modifiers::NONE),
    ];
    let mut expected = first.clone();
    expected.extend(last.clone());
    let mut buffered = input(control, first);
    buffered.append(input(Modifiers::NONE, last));
    assert_eq!(
        buffered.modifiers,
        Modifiers::NONE,
        "append keeps the latest global snapshot"
    );
    assert_eq!(buffered.events, expected);
    let drained = buffered.take();
    assert_eq!(drained.events, expected);
    assert!(buffered.events.is_empty());
    assert_eq!(buffered.modifiers, Modifiers::NONE);
    assert!(
        buffered.take().events.is_empty(),
        "take must not replay a transition"
    );

    let context = Context::default();
    context.begin_pass(drained);
    context.input(|state| {
        assert_eq!(state.modifiers, Modifiers::NONE);
        assert_eq!(state.raw.modifiers, Modifiers::NONE);
        assert_eq!(state.events, expected);
        assert_eq!(state.raw.events, expected);
        assert_eq!(state.filtered_events(&EventFilter::default()), expected);
        assert_eq!(
            state.filtered_events(&EventFilter {
                tab: true,
                horizontal_arrows: true,
                vertical_arrows: true,
                escape: true,
            }),
            expected
        );
        assert!(state.keys_down.is_empty());
        assert!(
            !state
                .events
                .iter()
                .any(|event| matches!(event, Event::Key { .. })),
            "a modifier transition must not fabricate a keyboard command"
        );
    });
    let _ = context.end_pass();
}

#[test]
fn a_press_snapshot_and_later_transitions_survive_frames_and_focus_loss_without_replay() {
    let a = egui::pos2(40.0, 70.0);
    let b = egui::pos2(88.0, 70.0);
    let c = egui::pos2(136.0, 94.0);
    let control = Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    };
    let shift = Modifiers {
        shift: true,
        ..Modifiers::NONE
    };
    let context = Context::default();
    let expected = vec![
        Event::PointerMoved(a),
        primary(a, true, control),
        Event::ModifiersChanged(shift),
        Event::PointerMoved(b),
    ];
    let mut buffered = input(shift, expected.clone());
    context.begin_pass(buffered.take());
    context.input(|state| {
        assert_eq!(state.modifiers, shift);
        assert_eq!(state.events, expected);
        assert!(
            matches!(&state.events[1], Event::PointerButton { pressed: true, modifiers, .. }
            if *modifiers == control),
            "the later snapshot cannot rewrite the press seed"
        );
        assert!(state.pointer.primary_down());
        assert!(state.keys_down.is_empty());
    });
    let _ = context.end_pass();

    for _ in 0..2 {
        context.begin_pass(buffered.take());
        context.input(|state| {
            assert_eq!(state.modifiers, shift);
            assert!(state.events.is_empty() && state.raw.events.is_empty());
            assert!(state.filtered_events(&EventFilter::default()).is_empty());
            assert!(state.pointer.primary_down());
            assert!(!state.pointer.any_pressed() && !state.pointer.any_released());
            assert!(state.keys_down.is_empty());
        });
        let _ = context.end_pass();
    }

    let cancelled_surface = vec![
        Event::PointerMoved(c),
        Event::WindowFocused(false),
        Event::ModifiersChanged(Modifiers::NONE),
        Event::PointerGone,
    ];
    let mut unfocused = input(Modifiers::NONE, cancelled_surface.clone());
    unfocused.focused = false;
    buffered.append(unfocused);
    context.begin_pass(buffered.take());
    context.input(|state| {
        assert!(!state.focused);
        assert_eq!(state.modifiers, Modifiers::NONE);
        assert_eq!(state.events, cancelled_surface);
        assert_eq!(
            state.filtered_events(&EventFilter::default()),
            cancelled_surface
        );
        assert!(state.keys_down.is_empty());
    });
    let _ = context.end_pass();
    context.begin_pass(buffered.take());
    context.input(|state| {
        assert!(!state.focused);
        assert!(state.events.is_empty() && state.raw.events.is_empty());
    });
    let _ = context.end_pass();
}

#[test]
fn a_first_frame_global_snapshot_alone_does_not_invent_prior_transitions_or_keys() {
    let context = Context::default();
    let mac_command = Modifiers {
        mac_cmd: true,
        command: true,
        ..Modifiers::NONE
    };
    context.begin_pass(input(mac_command, vec![]));
    context.input(|state| {
        assert_eq!(state.modifiers, mac_command);
        assert_eq!(state.raw.modifiers, mac_command);
        assert!(state.events.is_empty() && state.raw.events.is_empty());
        assert!(state.filtered_events(&EventFilter::default()).is_empty());
        assert!(state.keys_down.is_empty());
    });
    let _ = context.end_pass();
}
