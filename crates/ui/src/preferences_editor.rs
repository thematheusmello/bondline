//! A session-local draft; only the registered Apply operation changes preferences.
use bondline_commands::{Command, Engine, UiPreferences, UiPreferencesRequest};
use eframe::egui;

struct Form {
    request: UiPreferencesRequest,
    draft: bool,
    detached: bool,
    message: Option<String>,
    controls: [Option<egui::Id>; 4],
}

#[derive(Default)]
pub(super) struct PreferencesEditor {
    form: Option<Form>,
}

#[derive(Clone, Copy)]
enum Action {
    Toggle,
    Defaults,
    Apply,
    Cancel,
}

impl PreferencesEditor {
    pub fn present(&self) -> bool {
        self.form.is_some()
    }

    pub fn sync(&mut self, engine: &Engine) {
        if self.form.is_none()
            && let Some(request) = engine.ui_preferences_request()
        {
            self.form = Some(Form {
                draft: request.sprout_rings_when_clicking,
                detached: !request.current,
                request,
                message: None,
                controls: [None; 4],
            });
        }
    }

    fn end(&mut self, context: &egui::Context) {
        if let Some(form) = self.form.take() {
            context.memory_mut(|memory| {
                for id in form.controls.into_iter().flatten() {
                    memory.surrender_focus(id);
                }
            });
            context.request_discard("Preferences presentation retired");
            context.request_repaint();
        }
    }

    pub fn show(
        &mut self,
        context: &egui::Context,
        mut engine: Option<&mut Engine>,
        events: &[egui::Event],
        editable: bool,
    ) {
        let Some(form) = &mut self.form else {
            return;
        };
        if form.request.was_cancelled() {
            self.end(context);
            return;
        }
        let id = form.request.id;
        form.detached |= !engine
            .as_ref()
            .is_some_and(|engine| engine.ui_preferences_request_current(id));
        let mut shown = form.draft;
        let mut controls = Vec::with_capacity(4);
        egui::Window::new("Preferences")
            .id(egui::Id::new(("bondline-preferences", id)))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .order(egui::Order::Foreground).collapsible(false).resizable(false)
            .default_width(370.0).show(context, |ui| {
                ui.heading("Building/Display");
                ui.label("Applies to all documents in this session.");
                controls.push((Action::Toggle, ui.checkbox(&mut shown,
                    "Sprout rings instead of spiro when clicking")));
                if form.detached {
                    ui.colored_label(egui::Color32::DARK_RED,
                        "The session or view changed. Keep this draft or Cancel; it cannot be applied here.");
                }
                if let Some(message) = &form.message { ui.colored_label(egui::Color32::DARK_RED, message); }
                ui.horizontal(|ui| {
                    controls.push((Action::Defaults, ui.button("Use Defaults")));
                    controls.push((Action::Apply, ui.add_enabled(!form.detached && editable, egui::Button::new("OK"))));
                    controls.push((Action::Cancel, ui.button("Cancel")));
                });
            });
        for (slot, (_, response)) in form.controls.iter_mut().zip(&controls) {
            *slot = Some(response.id);
        }
        let mut ended = false;
        // SDK responses aggregate the packet; replay every activation in its original order.
        for event in events {
            let action = if matches!(event, egui::Event::Key {
                key: egui::Key::Escape, pressed: true, repeat: false, modifiers, .. } if modifiers.is_none())
            {
                Some(Action::Cancel)
            } else {
                controls.iter().find_map(|(action, response)| {
                    (response.enabled()
                        && response.clicked()
                        && crate::activation_event(response, event))
                    .then_some(*action)
                })
            };
            match action {
                Some(Action::Toggle) => form.draft = !form.draft,
                Some(Action::Defaults) => form.draft = false,
                Some(Action::Cancel) => {
                    if let Some(engine) = engine.as_deref_mut() {
                        let _ = engine.execute(Command::UiPreferences(UiPreferences::Cancel {
                            expected_revision: engine.state().revision,
                            request_id: id,
                        }));
                        engine.finish_ui_preferences(id);
                    }
                    ended = true;
                }
                Some(Action::Apply) if !form.detached && editable => {
                    if let Some(engine) = engine.as_deref_mut() {
                        match engine.execute(Command::UiPreferences(UiPreferences::Apply {
                            expected_revision: engine.state().revision,
                            request_id: id,
                            sprout_rings_when_clicking: form.draft,
                        })) {
                            Ok(_) => ended = true,
                            Err(error) => form.message = Some(error.to_string()),
                        }
                    }
                }
                _ => {}
            }
            if ended {
                break;
            }
        }
        if ended {
            self.end(context);
        } else if shown != form.draft {
            // Repaint the authoritative draft, including even-parity duplicate toggles.
            context.request_discard("Preferences draft differs from the SDK aggregate");
            context.request_repaint();
        }
    }
}
