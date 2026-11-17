//! Ordered input routing shared by bounded native forms.
use eframe::egui;

pub(super) struct Routed {
    pub fields: [Vec<egui::Event>; 2],
    pub errors: [Option<String>; 2],
    pub focus: Option<usize>,
}

fn key_event(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}

pub(super) fn route_prefix(
    fields: &[Option<egui::text_edit::TextEditOutput>; 2],
    limits: [usize; 2],
    caret_limit: usize,
    field_id: impl Fn(usize) -> egui::Id,
    context: &egui::Context,
    events: &[egui::Event],
    focus: Option<egui::Id>,
) -> Routed {
    use egui::accesskit::{Action, ActionData, ActionRequest, NodeId, TextPosition, TextSelection};
    let index_of = |node: u64| (0..2).find(|index| field_id(*index).value() == node);
    let mut routed = Routed {
        fields: [Vec::new(), Vec::new()],
        errors: [None, None],
        focus: focus.and_then(|id| index_of(id.value())),
    };
    let mut dragging = fields.iter().position(|shown| {
        shown
            .as_ref()
            .is_some_and(|shown| shown.response.is_pointer_button_down_on())
    });
    for event in events {
        let mut caret = None;
        match event {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers,
            } => {
                let target = fields.iter().enumerate().find_map(|(index, shown)| {
                    let shown = shown.as_ref()?;
                    (shown.response.enabled()
                        && shown.response.interact_rect.contains(*pos)
                        && context.layer_id_at(*pos) == Some(shown.response.layer_id))
                    .then_some(index)
                });
                routed.focus = target;
                dragging = target;
                caret = target.map(|index| (index, *pos, modifiers.shift));
            }
            egui::Event::PointerMoved(pos) => caret = dragging.map(|index| (index, *pos, true)),
            egui::Event::PointerButton {
                button: egui::PointerButton::Primary,
                pressed: false,
                ..
            } => dragging = None,
            egui::Event::AccessKitActionRequest(request) => {
                if let Some(index) = index_of(request.target.0) {
                    if request.action == Action::Focus {
                        routed.focus = Some(index);
                        continue;
                    }
                    if request.action == Action::SetValue {
                        if let Some(ActionData::Value(value)) = &request.data {
                            let Some(limit) = limits.get(index).copied() else {
                                continue;
                            };
                            if value.len() > limit {
                                if let Some(queue) = routed.fields.get_mut(index) {
                                    queue.clear();
                                }
                                if let Some(error) = routed.errors.get_mut(index) {
                                    *error = Some(
                                        crate::sdf_draft_text::DraftTextError::ByteLimit {
                                            attempted_bytes: value.len(),
                                            limit_bytes: limit,
                                        }
                                        .to_string(),
                                    );
                                }
                            } else if routed.errors.get(index).is_some_and(Option::is_none)
                                && let Some(queue) = routed.fields.get_mut(index)
                            {
                                queue.push(key_event(egui::Key::A, egui::Modifiers::COMMAND));
                                queue.push(if value.is_empty() {
                                    key_event(egui::Key::Backspace, egui::Modifiers::NONE)
                                } else {
                                    egui::Event::Paste(value.to_string())
                                });
                            }
                        }
                        continue;
                    }
                    if let Some(queue) = routed.fields.get_mut(index) {
                        queue.push(event.clone());
                    }
                }
                continue;
            }
            _ => {
                if let Some(index) = routed.focus
                    && routed.errors.get(index).is_some_and(Option::is_none)
                    && let Some(queue) = routed.fields.get_mut(index)
                {
                    queue.push(event.clone());
                }
            }
        }
        if let Some((index, pos, extend)) = caret
            && routed.errors.get(index).is_some_and(Option::is_none)
            && let Some(shown) = fields.get(index).and_then(Option::as_ref)
            && let Some(queue) = routed.fields.get_mut(index)
        {
            let cursor = shown.galley.cursor_from_pos(pos - shown.galley_pos);
            if extend {
                // SDK movement preserves the then-current anchor after preceding edits.
                queue.push(key_event(
                    egui::Key::Home,
                    egui::Modifiers {
                        shift: true,
                        ctrl: true,
                        ..egui::Modifiers::NONE
                    },
                ));
                for _ in 0..cursor.ccursor.index.min(caret_limit) {
                    queue.push(key_event(egui::Key::ArrowRight, egui::Modifiers::SHIFT));
                }
            } else {
                let position = TextPosition {
                    node: NodeId(field_id(index).with(cursor.rcursor.row).value()),
                    character_index: cursor.rcursor.column,
                };
                queue.push(egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::SetTextSelection,
                    target: NodeId(field_id(index).value()),
                    data: Some(ActionData::SetTextSelection(TextSelection {
                        anchor: position,
                        focus: position,
                    })),
                }));
            }
        }
    }
    routed
}
