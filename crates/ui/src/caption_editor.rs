//! A painted-run or blank-paper click and one bounded caption draft.
use crate::{
    HoverContext, file_workflow::smiles_paste::field_output, sdf_draft_text::SdfDraftText,
};
use bondline_commands::{
    CaptionContext, Command, Engine, MAX_CAPTION_DRAFT_BYTES, UiCaption, UiCaptionRequest,
    UiCaptionTarget,
};
use bondline_model::Point2;
use bondline_render::{CaptionRunTarget, GraphScene, PageView};
use eframe::egui;

const CLICK_SLOP: f32 = 3.0;

#[derive(Clone, Copy)]
pub(super) enum Target {
    Run(CaptionRunTarget),
    Paper(Point2),
}

pub(super) fn canvas_target(
    engine: &Engine,
    graph: &GraphScene,
    view: &PageView,
    point: [f32; 2],
    density: f32,
) -> Option<Target> {
    if !view.contains(point) {
        return None;
    }
    if let Some(hit) = graph.caption_hit_test(point).ok()? {
        return Some(Target::Run(hit));
    }
    if !engine.caption_creation_available()
        || graph.hit_test(point, 5.0 * density).ok()?.is_some()
        || graph.caption_envelope_contains(point).ok()?
    {
        return None;
    }
    view.document_position(point).ok().map(Target::Paper)
}

struct Press {
    position: egui::Pos2,
    target: Target,
    surface: HoverContext,
    layer: egui::LayerId,
    context: CaptionContext,
}

struct Form {
    request: UiCaptionRequest,
    text: SdfDraftText,
    field: [Option<egui::text_edit::TextEditOutput>; 1],
    buttons: [Option<egui::Id>; 2],
    deferred: Option<egui::Event>,
    detached: bool,
    message: Option<String>,
}

#[derive(Clone, Copy)]
enum Action {
    Apply,
    Cancel,
}

#[derive(Default)]
pub(super) struct CaptionEditor {
    press: Option<Press>,
    form: Option<Form>,
}

fn field_id(id: u64) -> egui::Id {
    egui::Id::new(("caption-run", id))
}

impl CaptionEditor {
    pub fn cancel_press(&mut self) {
        self.press = None;
    }

    pub fn sync_pointer(&mut self, engine: &Engine, surface: HoverContext, layer: egui::LayerId) {
        if self.press.as_ref().is_some_and(|press| {
            press.surface != surface || press.layer != layer || !press.context.is_current(engine)
        }) {
            self.cancel_press();
        }
    }

    pub fn pointer(
        &mut self,
        engine: &Engine,
        event: &egui::Event,
        surface: HoverContext,
        layer: egui::LayerId,
        target: Option<Target>,
    ) -> Option<Command> {
        let same = |press: &Press, pos: egui::Pos2| {
            let same_target = match (press.target, target) {
                (Target::Run(before), Some(Target::Run(now))) => before == now,
                (Target::Paper(_), Some(Target::Paper(_))) => true,
                _ => false,
            };
            press.surface == surface
                && press.layer == layer
                && same_target
                && press.position.distance(pos) <= CLICK_SLOP
                && press.context.is_current(engine)
        };
        match event {
            egui::Event::PointerMoved(pos) => {
                if self.press.as_ref().is_some_and(|press| !same(press, *pos)) {
                    self.cancel_press();
                }
            }
            egui::Event::ModifiersChanged(modifiers) if !modifiers.is_none() => self.cancel_press(),
            egui::Event::PointerButton {
                button: egui::PointerButton::Primary,
                pressed: true,
                pos,
                modifiers,
            } => {
                self.press = target.filter(|_| modifiers.is_none()).and_then(|target| {
                    Some(Press {
                        position: *pos,
                        target,
                        surface,
                        layer,
                        context: engine.caption_context().ok()?,
                    })
                });
            }
            egui::Event::PointerButton {
                button: egui::PointerButton::Primary,
                pressed: false,
                pos,
                modifiers,
            } => {
                let press = self.press.take()?;
                if modifiers.is_none() && same(&press, *pos) {
                    let expected_revision = surface.revision;
                    let page_id = surface.page.get();
                    let open = match press.target {
                        Target::Run(hit) => UiCaption::Open {
                            expected_revision,
                            page_id,
                            text_id: hit.text_id.get(),
                            run_index: hit.run_index,
                        },
                        Target::Paper(point) => UiCaption::OpenNew {
                            expected_revision,
                            page_id,
                            x_pt: point.x_pt(),
                            y_pt: point.y_pt(),
                        },
                    };
                    return Some(Command::UiCaption(open));
                }
            }
            egui::Event::PointerButton { .. } => self.cancel_press(),
            _ => {}
        }
        None
    }

    pub fn present(&self) -> bool {
        self.form.is_some()
    }

    pub fn sync(&mut self, engine: &Engine, context: &egui::Context, error: &mut Option<String>) {
        if self
            .form
            .as_ref()
            .is_some_and(|f| f.request.was_cancelled())
        {
            self.end(context);
        }
        if self.form.is_some() {
            return;
        }
        let Some(request) = engine.ui_caption_request() else {
            return;
        };
        match SdfDraftText::with_byte_limit(&request.text, MAX_CAPTION_DRAFT_BYTES) {
            Ok(text) => {
                self.form = Some(Form {
                    detached: !request.current,
                    request,
                    text,
                    field: [None],
                    buttons: [None; 2],
                    deferred: None,
                    message: None,
                });
            }
            Err(failure) => *error = Some(failure.to_string()),
        }
    }

    fn end(&mut self, context: &egui::Context) {
        if let Some(form) = self.form.take() {
            let id = field_id(form.request.id);
            context.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(id));
            context.memory_mut(|memory| memory.surrender_focus(id));
            context.request_repaint();
        }
    }

    pub fn show(
        &mut self,
        context: &egui::Context,
        engine: Option<&mut Engine>,
        events: &[egui::Event],
    ) {
        let Some(form) = &mut self.form else {
            return;
        };
        let request = form.request.id;
        let id = field_id(request);
        form.detached |= !engine
            .as_ref()
            .is_some_and(|engine| engine.ui_caption_request_current(request));
        if form.field[0].is_none() {
            context.memory_mut(|memory| memory.request_focus(id));
        }
        let action = events.iter().enumerate().find_map(|(at, event)| {
            if matches!(event, egui::Event::Key {
                key: egui::Key::Escape, pressed: true, repeat: false, modifiers, ..
            } if modifiers.is_none())
            {
                return Some((at, Action::Cancel));
            }
            form.buttons
                .iter()
                .zip([Action::Apply, Action::Cancel])
                .find_map(|(button, action)| {
                    let shown = context.read_response((*button)?)?;
                    (shown.enabled() && shown.clicked() && crate::activation_event(&shown, event))
                        .then_some((at, action))
                })
        });
        // Retain the event-local text preceding Apply even when egui's final pointer focus changed.
        let mut queued: Vec<_> = form.deferred.take().into_iter().collect();
        let mut input = (action.is_some() || !queued.is_empty()).then(|| {
            queued.extend_from_slice(action.map_or(events, |(at, _)| &events[..=at]));
            let mut error = None;
            if action.is_some() {
                let focus = context.memory(|memory| memory.focused());
                let mut routed = crate::pair_input::route_prefix(
                    &form.field,
                    [MAX_CAPTION_DRAFT_BYTES],
                    MAX_CAPTION_DRAFT_BYTES,
                    |_| id,
                    context,
                    &queued,
                    focus,
                );
                queued = std::mem::take(&mut routed.fields[0]);
                if routed.focus.is_some() || !queued.is_empty() {
                    context.memory_mut(|memory| memory.request_focus(id));
                }
                error = routed.errors[0].take();
            }
            let saved = context.input_mut(|input| {
                (
                    std::mem::replace(&mut input.events, queued),
                    action.map(|_| std::mem::take(&mut input.pointer)),
                )
            });
            (saved, error)
        });
        let mut input_rejected = false;
        let is_new = matches!(form.request.target, UiCaptionTarget::New { .. });
        egui::Window::new(if is_new {
            "New caption"
        } else {
            "Edit caption"
        })
        .id(egui::Id::new(("edit-caption", request)))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .order(egui::Order::Foreground)
        .collapsible(false)
        .resizable(false)
        .default_width(390.0)
        .show(context, |ui| {
            if is_new {
                ui.weak("Roboto Regular · 12 pt · Black");
            }
            if let UiCaptionTarget::Existing {
                run_index,
                run_count,
                ..
            } = form.request.target
                && run_count > 1
            {
                ui.weak(format!("Run {} of {}", run_index + 1, run_count));
            }
            let (shown, failure) =
                field_output(ui, id, "Caption text", &mut form.text, false, true);
            if ui.is_visible() && !context.will_discard() {
                form.field = [Some(shown)];
            }
            let failure = failure.or_else(|| input.as_mut().and_then(|(_, error)| error.take()));
            if let Some(message) = failure {
                form.message = Some(message);
                input_rejected = true;
            }
            if let Some(((events, pointer), _)) = input.take() {
                context.input_mut(|input| {
                    input.events = events;
                    if let Some(pointer) = pointer {
                        input.pointer = pointer;
                    }
                });
            }
            if form.detached {
                ui.colored_label(
                    egui::Color32::DARK_RED,
                    "The document or view changed. Copy your draft or click Cancel.",
                );
            }
            if let Some(message) = &form.message {
                ui.colored_label(egui::Color32::DARK_RED, message);
            }
            ui.horizontal(|ui| {
                let apply = ui.add_enabled(
                    !form.detached && !input_rejected,
                    egui::Button::new("Apply"),
                );
                let cancel = ui.button("Cancel");
                form.buttons = [Some(apply.id), Some(cancel.id)];
            });
        });
        if let Some(((events, pointer), _)) = input {
            context.input_mut(|input| {
                input.events = events;
                if let Some(pointer) = pointer {
                    input.pointer = pointer;
                }
            });
        }
        match action {
            Some((_, Action::Cancel)) => {
                if let Some(engine) = engine {
                    let _ = engine.execute(Command::UiCaption(UiCaption::Cancel {
                        expected_revision: engine.state().revision,
                        request_id: request,
                    }));
                    engine.finish_ui_caption(request);
                }
                self.end(context);
            }
            Some((at, Action::Apply)) => {
                if !form.detached
                    && !input_rejected
                    && let Some(engine) = engine
                {
                    match engine.execute(Command::UiCaption(UiCaption::Apply {
                        expected_revision: form.request.input_revision,
                        request_id: request,
                        text: form.text.as_str().to_owned(),
                    })) {
                        Ok(_) => {
                            self.end(context);
                            return;
                        }
                        Err(error) => form.message = Some(error.to_string()),
                    }
                }
                if let Some((event, value)) = events[at + 1..].iter().rev().find_map(|event| {
                    use egui::accesskit::{Action, ActionData};
                    if let egui::Event::AccessKitActionRequest(request) = event
                        && request.target.0 == id.value()
                        && request.action == Action::SetValue
                        && let Some(ActionData::Value(value)) = &request.data
                    {
                        Some((event, value))
                    } else {
                        None
                    }
                }) {
                    if value.len() <= MAX_CAPTION_DRAFT_BYTES {
                        form.deferred = Some(event.clone());
                    } else {
                        form.message = Some(
                            crate::sdf_draft_text::DraftTextError::ByteLimit {
                                attempted_bytes: value.len(),
                                limit_bytes: MAX_CAPTION_DRAFT_BYTES,
                            }
                            .to_string(),
                        );
                    }
                }
                context.request_repaint();
            }
            _ => {}
        }
    }
}
