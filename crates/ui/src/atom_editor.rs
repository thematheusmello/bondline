//! One exact-target draft; only the shared registry authors an atom label.
use crate::{file_workflow::smiles_paste::field_output, sdf_draft_text::SdfDraftText};
use bondline_commands::{Command, Engine, UiAtomLabel, UiAtomLabelRequest};
use bondline_model::Element;
use eframe::egui;

struct Form {
    request: UiAtomLabelRequest,
    element: SdfDraftText,
    hydrogens: SdfDraftText,
    detached: bool,
    message: Option<String>,
    buttons: [Option<egui::Id>; 2],
    fields: [Option<egui::text_edit::TextEditOutput>; 2],
    deferred_text: Vec<egui::Event>,
}

impl Form {
    fn values(&self) -> Result<(u8, u16), &'static str> {
        let element = Element::from_symbol(self.element.as_str())
            .ok_or("Enter an exact-case element symbol, such as C, N, O or Cl.")?;
        let text = self.hydrogens.as_str();
        let hydrogens = if !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()) {
            text.parse::<u16>().ok()
        } else {
            None
        }
        .ok_or("Attached hydrogens must be an explicit integer from 0 to 65535.")?;
        if element.get() == 1 && hydrogens != 0 {
            return Err("A physical H atom cannot carry attached hydrogens.");
        }
        Ok((element.get(), hydrogens))
    }
}

#[derive(Default)]
pub(super) struct AtomEditor {
    form: Option<Form>,
}

#[derive(Clone, Copy)]
enum Action {
    Apply,
    Cancel,
}

fn route_prefix(
    form: &Form,
    context: &egui::Context,
    events: &[egui::Event],
    focus: Option<egui::Id>,
) -> crate::pair_input::Routed {
    crate::pair_input::route_prefix(
        &form.fields,
        [2, 5],
        5,
        |index| field_id(form.request.id, index),
        context,
        events,
        focus,
    )
}

fn field_id(key: u64, index: usize) -> egui::Id {
    egui::Id::new(("atom-label-field", key, index))
}

impl AtomEditor {
    pub fn present(&self) -> bool {
        self.form.is_some()
    }

    pub fn sync(&mut self, engine: &Engine, error: &mut Option<String>) {
        if self.form.is_some() {
            return;
        }
        let Some(request) = engine.ui_atom_label_request() else {
            return;
        };
        let values = (|| {
            let symbol = Element::new(request.element)
                .and_then(Element::symbol)
                .ok_or("Unknown target element.")?;
            let element = SdfDraftText::with_byte_limit(symbol, 2).map_err(|e| e.to_string())?;
            let h = request
                .explicit_hydrogens
                .map_or_else(String::new, |h| h.to_string());
            let hydrogens = SdfDraftText::with_byte_limit(&h, 5).map_err(|e| e.to_string())?;
            Ok::<_, String>((element, hydrogens))
        })();
        match values {
            Ok((element, hydrogens)) => {
                self.form = Some(Form {
                    detached: !request.current,
                    request,
                    element,
                    hydrogens,
                    message: None,
                    buttons: [None, None],
                    fields: [None, None],
                    deferred_text: Vec::new(),
                })
            }
            Err(message) => *error = Some(message),
        }
    }

    fn end(&mut self, context: &egui::Context) {
        if let Some(form) = self.form.take() {
            for index in 0..2 {
                let id = field_id(form.request.id, index);
                context.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(id));
                context.memory_mut(|memory| memory.surrender_focus(id));
            }
            context.request_repaint();
        }
    }

    pub fn show(
        &mut self,
        context: &egui::Context,
        mut engine: Option<&mut Engine>,
        events: &[egui::Event],
    ) {
        let Some(form) = &mut self.form else {
            return;
        };
        let id = form.request.id;
        let original_focus = context.memory(|memory| memory.focused());
        if !engine
            .as_ref()
            .is_some_and(|engine| engine.ui_atom_label_request_current(id))
        {
            form.detached = true;
        }
        let mut action = None;
        for (at, event) in events.iter().enumerate() {
            if matches!(event,egui::Event::Key {key:egui::Key::Escape,pressed:true,repeat:false,modifiers,..} if modifiers.is_none())
            {
                action = Some((at, Action::Cancel));
                break;
            }
            for (index, button) in form.buttons.iter().enumerate() {
                if button
                    .and_then(|id| context.read_response(id))
                    .is_some_and(|response| {
                        response.enabled()
                            && response.clicked()
                            && crate::activation_event(&response, event)
                    })
                {
                    action = Some((
                        at,
                        if index == 0 {
                            Action::Apply
                        } else {
                            Action::Cancel
                        },
                    ));
                    break;
                }
            }
            if action.is_some() {
                break;
            }
        }
        // The SDK aggregates focus/pointer for the full packet; fields see only the action prefix.
        let mut field_events = std::mem::take(&mut form.deferred_text);
        field_events.extend_from_slice(
            action.map_or(events, |(at, _)| events.get(..=at).unwrap_or_default()),
        );
        let mut routed = action.map(|_| route_prefix(form, context, &field_events, original_focus));
        let saved_events =
            context.input_mut(|input| std::mem::replace(&mut input.events, field_events));
        let mut saved_pointer =
            action.map(|_| context.input_mut(|input| std::mem::take(&mut input.pointer)));
        let mut current_values = form.values();
        let mut input_rejected = false;
        egui::Window::new("Atom label")
            .id(egui::Id::new(("atom-label",id)))
            .anchor(egui::Align2::CENTER_CENTER,egui::Vec2::ZERO)
            .order(egui::Order::Foreground).collapsible(false).resizable(false)
            .default_width(370.0).show(context,|ui| {
                ui.label(format!("Page {} · Fragment {} · Atom {}",form.request.page_id,form.request.fragment_id,form.request.atom_id));
                ui.label("Set literal element and attached-H fields. Neighbors are unchanged.");
                if form.request.creation_style.is_some() {
                    ui.label("New printed label · Roboto Regular · 12 pt · Black");
                }
                for (index,(label,text)) in [("Element",&mut form.element),("Attached hydrogens",&mut form.hydrogens)].into_iter().enumerate() {
                    if let Some(routed)=&mut routed {
                        let queue=routed.fields.get_mut(index).map(std::mem::take).unwrap_or_default();
                        let focus=!queue.is_empty() || routed.focus==Some(index);
                        context.input_mut(|input|input.events=queue);
                        context.memory_mut(|memory| {
                            if focus {memory.request_focus(field_id(id,index));}
                            else {memory.surrender_focus(field_id(id,index));}
                        });
                    }
                    let (shown,failure)=field_output(ui,field_id(id,index),label,text,false,true);
                    if shown.response.enabled() && ui.is_visible() && !context.will_discard()
                        && let Some(slot)=form.fields.get_mut(index) {*slot=Some(shown);}
                    let failure=failure.or_else(||routed.as_ref().and_then(|routed|routed.errors.get(index)).cloned().flatten());
                    if let Some(message)=failure {form.message=Some(message);input_rejected=true;}
                }
                if let Some(routed)=&routed {
                    context.memory_mut(|memory| {
                        for index in 0..2 {memory.surrender_focus(field_id(id,index));}
                        if let Some(index)=routed.focus {memory.request_focus(field_id(id,index));}
                    });
                }
                if let Some(pointer)=saved_pointer.take() {context.input_mut(|input|input.pointer=pointer);}
                current_values=form.values();
                if form.detached {
                    ui.colored_label(egui::Color32::DARK_RED,"The document or view changed. Copy this draft or Cancel; it cannot be applied here.");
                }
                if let Err(message)=current_values {ui.colored_label(egui::Color32::DARK_RED,message);}
                if let Some(message)=&form.message {ui.colored_label(egui::Color32::DARK_RED,message);}
                ui.horizontal(|ui| {
                    let apply=ui.add_enabled(!form.detached && !input_rejected && current_values.is_ok(),egui::Button::new("Apply"));
                    let cancel=ui.button("Cancel");
                    form.buttons=[Some(apply.id),Some(cancel.id)];
                });
            });
        if let Some(pointer) = saved_pointer {
            context.input_mut(|input| input.pointer = pointer);
        }
        context.input_mut(|input| input.events = saved_events);
        match action {
            Some((_, Action::Cancel)) => {
                if let Some(engine) = engine.as_deref_mut() {
                    // Explicit local Cancel also retires a stale/remote-cancelled shutdown form.
                    let _ = engine.execute(Command::UiAtomLabel(UiAtomLabel::Cancel {
                        expected_revision: engine.state().revision,
                        request_id: id,
                    }));
                    engine.finish_ui_atom_label(id);
                }
                self.end(context);
            }
            Some((at, Action::Apply)) => {
                if !form.detached
                    && !input_rejected
                    && let (Some(engine), Ok((element, hydrogens))) = (engine, current_values)
                {
                    match engine.apply_ui_atom_label(id, element, hydrogens) {
                        Ok(_) => {
                            self.end(context);
                            return;
                        }
                        Err(error) => form.message = Some(error.to_string()),
                    }
                }
                // Match the field bridge's last replacement, bounded before retaining any payload.
                let suffix = events.get(at.saturating_add(1)..).unwrap_or_default();
                for (index, limit) in [(0, 2), (1, 5)] {
                    if let Some(event) = suffix.iter().rev().find(|event| {
                        matches!(event,
                        egui::Event::AccessKitActionRequest(request)
                        if request.action == egui::accesskit::Action::SetValue
                            && request.target.0 == field_id(id,index).value())
                    }) && let egui::Event::AccessKitActionRequest(request) = event
                        && let Some(egui::accesskit::ActionData::Value(value)) = &request.data
                    {
                        if value.len() <= limit {
                            form.deferred_text.push(event.clone());
                        } else {
                            form.message = Some(
                                crate::sdf_draft_text::DraftTextError::ByteLimit {
                                    attempted_bytes: value.len(),
                                    limit_bytes: limit,
                                }
                                .to_string(),
                            );
                        }
                    }
                }
                context.request_repaint();
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod deferred_atom_lifecycle_tests;
