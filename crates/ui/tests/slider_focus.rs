//! Exercise the pinned SDK directly: no Bondline engine, window or native input.
use eframe::egui::{self, Context, Event, Id, Key, Modifiers, RawInput};

struct Frame {
    slider: Id,
    during: Option<Id>,
    after: Option<Id>,
    value: f64,
    changed: bool,
}

fn frame(context: &Context, value: &mut f64, events: Vec<Event>, time: &mut f64) -> Frame {
    *time += 1.0 / 60.0;
    let mut slider = Id::NULL;
    let mut during = None;
    let mut changed = false;
    let _ = context.run(
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 300.0),
            )),
            time: Some(*time),
            focused: true,
            events,
            ..Default::default()
        },
        |context| {
            egui::CentralPanel::default().show(context, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().slider_width = 300.0;
                    let response = ui.add(egui::Slider::new(value, 0.0..=600.0).show_value(false));
                    slider = response.id;
                    changed = response.changed();
                    let _ = ui.button("Other focus target");
                    during = context.memory(|memory| memory.focused());
                });
            });
        },
    );
    assert_ne!(slider, Id::NULL);
    Frame {
        slider,
        during,
        after: context.memory(|memory| memory.focused()),
        value: *value,
        changed,
    }
}

fn tap(key: Key) -> Vec<Event> {
    [true, false]
        .into_iter()
        .map(|pressed| Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        })
        .collect()
}

fn check_slider_focus(settle: bool) {
    let context = Context::default();
    context.enable_accesskit();
    let mut fonts = egui::FontDefinitions::empty();
    fonts.font_data.insert(
        "focus-probe".into(),
        egui::FontData::from_static(include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts.families.insert(family, vec!["focus-probe".into()]);
    }
    context.set_fonts(fonts);
    let mut value = 250.0;
    let mut time = 0.0;
    let initial = frame(&context, &mut value, vec![], &mut time);
    let focused = frame(
        &context,
        &mut value,
        vec![Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Focus,
                target: egui::accesskit::NodeId(initial.slider.value()),
                data: None,
            },
        )],
        &mut time,
    );
    assert_eq!(focused.after, Some(initial.slider));
    if settle {
        let idle = frame(&context, &mut value, vec![], &mut time);
        assert_eq!(idle.after, Some(initial.slider));
    }
    let right = frame(&context, &mut value, tap(Key::ArrowRight), &mut time);
    let left = frame(&context, &mut value, tap(Key::ArrowLeft), &mut time);
    assert_eq!(right.slider, initial.slider);
    assert_eq!(right.during, Some(initial.slider));
    assert!(right.changed && right.value > focused.value);
    assert_eq!(
        right.after,
        Some(initial.slider),
        "the first arrow must not schedule a second focus-navigation action"
    );
    assert_eq!(left.after, Some(initial.slider));
    assert!(
        left.changed && left.value < right.value,
        "the opposite arrow must still reach the same focused slider"
    );
}

#[test]
fn newly_focused_slider_keeps_both_arrow_keys_without_an_idle_frame() {
    check_slider_focus(false);
}

#[test]
fn settled_slider_focus_still_keeps_both_arrow_keys() {
    check_slider_focus(true);
}
