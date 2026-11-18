use super::*;
use crate::{file_workflow::smiles_paste::field_output, sdf_draft_text::SdfDraftText};
use bondline_commands::UiAddChainRequest;

struct Form {
    request: UiAddChainRequest,
    count: SdfDraftText,
    detached: bool,
    message: Option<String>,
    fields: [Option<egui::text_edit::TextEditOutput>; 2],
    buttons: [Option<egui::Id>; 2],
    deferred: Vec<egui::Event>,
    canvas: Option<(HoverContext, egui::LayerId)>,
}
impl Form {
    fn count(&self) -> Result<u8, String> {
        let text = self.count.as_str().trim();
        let invalid =
            || "Enter a carbon count from 2 through 64, including an existing anchor.".to_owned();
        if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        let value = text.parse::<u8>().map_err(|_| invalid())?;
        if !(2..=64).contains(&value) {
            return Err(invalid());
        }
        Ok(value)
    }
}
#[derive(Default)]
pub(super) struct AddEditor {
    form: Option<Form>,
}
#[derive(Clone, Copy)]
enum Action {
    Add,
    Cancel,
}
fn field_id(id: u64, index: usize) -> egui::Id {
    egui::Id::new(("chain-count-field", id, index))
}

impl AddEditor {
    pub(super) fn present(&self) -> bool {
        self.form.is_some()
    }
    pub(super) fn lose_canvas(&mut self) {
        if let Some(form) = &mut self.form
            && form.canvas.is_some()
        {
            form.detached = true;
        }
    }
    pub(super) fn sync(&mut self, engine: &Engine, error: &mut Option<String>) {
        if self.form.is_some() {
            return;
        }
        let Some(request) = engine.ui_add_chain_request() else {
            return;
        };
        match SdfDraftText::with_byte_limit(&request.count.to_string(), 384) {
            Ok(count) => {
                self.form = Some(Form {
                    detached: !request.current,
                    request,
                    count,
                    message: None,
                    fields: [None, None],
                    buttons: [None, None],
                    deferred: Vec::new(),
                    canvas: None,
                })
            }
            Err(failure) => *error = Some(failure.to_string()),
        }
    }
    pub(super) fn sync_canvas(&mut self, hover: HoverContext, layer: egui::LayerId) {
        if let Some(form) = &mut self.form
            && form.canvas.is_some_and(|old| old != (hover, layer))
        {
            form.detached = true;
        }
    }
    pub(super) fn preview(
        &self,
        engine: &Engine,
        view: &PageView,
        density: f32,
    ) -> Option<Result<(Arc<Document>, GraphScene), Error>> {
        let form = self.form.as_ref()?;
        if form.detached
            || form.request.was_cancelled()
            || !engine.ui_add_chain_request_current(form.request.id)
        {
            return None;
        }
        let count = form.count().ok()?;
        Some(
            engine
                .ui_add_chain_candidates(form.request.id, count)
                .and_then(|params| numeric_candidate(engine, &params, view, density))
                .map(|(_, c)| (c.document, c.graph)),
        )
    }
    fn end(&mut self, context: &egui::Context) {
        if let Some(form) = self.form.take() {
            let id = field_id(form.request.id, 0);
            context.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(id));
            context.memory_mut(|memory| memory.surrender_focus(id));
            context.request_repaint();
        }
    }
    pub(super) fn show(
        &mut self,
        context: &egui::Context,
        mut engine: Option<&mut Engine>,
        events: &[egui::Event],
        editable: bool,
        canvas: Option<&Canvas>,
    ) -> Option<Repeat> {
        let form = self.form.as_mut()?;
        if form.request.was_cancelled() {
            self.end(context);
            return None;
        }
        let id = form.request.id;
        if !engine
            .as_deref()
            .is_some_and(|e| e.ui_add_chain_request_current(id))
        {
            form.detached = true;
        }
        match (form.canvas, canvas) {
            (Some(old), Some(current)) if old != (current.hover, current.layer) => {
                form.detached = true
            }
            (Some(_), None) => form.detached = true,
            (None, Some(current)) => form.canvas = Some((current.hover, current.layer)),
            _ => {}
        }
        let original_focus = context.memory(|m| m.focused());
        let mut action = None;
        for (at, event) in events.iter().enumerate() {
            if matches!(event,egui::Event::Key{key:egui::Key::Escape,pressed:true,repeat:false,modifiers,..} if modifiers.is_none())
            {
                action = Some((at, Action::Cancel));
                break;
            }
            for (index, button) in form.buttons.iter().enumerate() {
                if button
                    .and_then(|id| context.read_response(id))
                    .is_some_and(|r| {
                        r.enabled() && r.clicked() && crate::activation_event(&r, event)
                    })
                {
                    action = Some((
                        at,
                        if index == 0 {
                            Action::Add
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
        let mut field_events = std::mem::take(&mut form.deferred);
        field_events.extend_from_slice(
            action.map_or(events, |(at, _)| events.get(..=at).unwrap_or_default()),
        );
        let mut routed = action.map(|_| {
            crate::pair_input::route_prefix(
                &form.fields,
                [384, 0],
                384,
                |index| field_id(id, index),
                context,
                &field_events,
                original_focus,
            )
        });
        let saved_events =
            context.input_mut(|input| std::mem::replace(&mut input.events, field_events));
        let mut saved_pointer =
            action.map(|_| context.input_mut(|input| std::mem::take(&mut input.pointer)));
        let mut values = form.count();
        let mut rejected = false;
        egui::Window::new("Add Chain").id(egui::Id::new(("add-chain",id)))
            .anchor(egui::Align2::CENTER_CENTER,egui::Vec2::ZERO).order(egui::Order::Foreground)
            .collapsible(false).resizable(false).default_width(370.0).show(context,|ui|{
                ui.label("The carbon count includes an existing anchor atom.");
                ui.weak(format!("Preferred bond: {} pt; interior angle: {} degrees.",form.request.bond_length_pt,form.request.angle_degrees));
                if let Some(routed)=&mut routed {
                    let queue=routed.fields.get_mut(0).map(std::mem::take).unwrap_or_default();
                    let focus=!queue.is_empty()||routed.focus==Some(0);
                    context.input_mut(|input|input.events=queue);
                    context.memory_mut(|memory|if focus {memory.request_focus(field_id(id,0));}else{memory.surrender_focus(field_id(id,0));});
                }
                let (shown,failure)=field_output(ui,field_id(id,0),"Carbon count (including anchor)",&mut form.count,false,true);
                if shown.response.enabled()&&ui.is_visible()&&!context.will_discard(){form.fields[0]=Some(shown);}
                let failure=failure.or_else(||routed.as_ref().and_then(|r|r.errors.first()).cloned().flatten());
                if let Some(message)=failure {form.message=Some(message);rejected=true;}
                if let Some(routed)=&routed {context.memory_mut(|memory|if routed.focus==Some(0){memory.request_focus(field_id(id,0));}else{memory.surrender_focus(field_id(id,0));});}
                if let Some(pointer)=saved_pointer.take(){context.input_mut(|input|input.pointer=pointer);}
                values=form.count();
                let ready=engine.as_deref().zip(canvas).zip(values.as_ref().ok()).map(|((engine,canvas),count)|{
                    engine.ui_add_chain_candidates(id,*count).and_then(|params|numeric_candidate(engine,&params,&canvas.view,canvas.hover.pixels_per_point))
                });
                if form.detached {ui.colored_label(egui::Color32::DARK_RED,"The document or canvas changed. Copy this count or Cancel; it cannot be added here.");}
                if let Err(message)=&values {ui.colored_label(egui::Color32::DARK_RED,message);}
                if let Some(Err(error))=&ready {ui.colored_label(egui::Color32::DARK_RED,error.to_string());}
                if let Some(message)=&form.message {ui.colored_label(egui::Color32::DARK_RED,message);}
                ui.horizontal(|ui|{
                    let add=ui.add_enabled(!form.detached&&!rejected&&editable&&ready.as_ref().is_some_and(Result::is_ok),egui::Button::new("Add"));
                    let cancel=ui.button("Cancel");form.buttons=[Some(add.id),Some(cancel.id)];
                });
            });
        if let Some(pointer) = saved_pointer {
            context.input_mut(|input| input.pointer = pointer);
        }
        context.input_mut(|input| input.events = saved_events);
        match action {
            Some((_, Action::Cancel)) => {
                if let Some(engine) = engine.as_deref_mut() {
                    let _ = engine.execute(Command::UiChainTool(UiChainTool::CancelAdd {
                        expected_revision: engine.state().revision,
                        request_id: id,
                    }));
                    engine.finish_ui_add_chain(id);
                }
                self.end(context);
            }
            Some((at, Action::Add)) => {
                if !form.detached
                    && !rejected
                    && editable
                    && let (Some(engine), Some(canvas), Ok(count)) = (engine, canvas, values)
                {
                    let result = engine
                        .ui_add_chain_candidates(id, count)
                        .and_then(|params| {
                            numeric_candidate(
                                engine,
                                &params,
                                &canvas.view,
                                canvas.hover.pixels_per_point,
                            )
                        })
                        .and_then(|(index, _)| engine.apply_ui_add_chain(id, count, index));
                    match result {
                        Ok(_) => {
                            let repeat = make_repeat(engine, count, canvas.hover, canvas.layer);
                            self.end(context);
                            return repeat;
                        }
                        Err(error) => form.message = Some(error.to_string()),
                    }
                }
                // Retain only the last bounded explicit replacement after a failed Add.
                if let Some(event)=events.get(at.saturating_add(1)..).unwrap_or_default().iter().rev().find(|event|matches!(event,
                    egui::Event::AccessKitActionRequest(request) if request.action==egui::accesskit::Action::SetValue&&request.target.0==field_id(id,0).value()))
                    && let egui::Event::AccessKitActionRequest(request)=event
                    && let Some(egui::accesskit::ActionData::Value(value))=&request.data {
                        if value.len()<=384 {form.deferred.push(event.clone());}
                        else {form.message=Some(crate::sdf_draft_text::DraftTextError::ByteLimit{attempted_bytes:value.len(),limit_bytes:384}.to_string());}
                    }
                context.request_repaint();
            }
            None => {}
        }
        None
    }
}
