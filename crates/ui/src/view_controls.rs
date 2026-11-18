use super::*;

pub(super) enum Action {
    Control(&'static str),
    Zoom(f64),
    PanX(f64),
    PanY(f64),
}

impl Action {
    pub(super) fn command(&self, engine: &Engine) -> Option<Command> {
        if let Self::Control(id) = *self {
            return control_command(engine, id);
        }
        if let Self::Zoom(percent) = *self {
            return Some(Command::UiView(bondline_commands::UiView::SetZoom {
                percent,
            }));
        }
        let size = engine.view_page_size().ok()?;
        let [mut x, mut y] = center(engine, size);
        let action = match *self {
            Self::Control(_) | Self::Zoom(_) => return None,
            Self::PanX(value) => {
                x = value;
                bondline_commands::UiView::Pan {
                    center_x_pt: x,
                    center_y_pt: y,
                }
            }
            Self::PanY(value) => {
                y = value;
                bondline_commands::UiView::Pan {
                    center_x_pt: x,
                    center_y_pt: y,
                }
            }
        };
        Some(Command::UiView(action))
    }
}

pub(super) fn center(engine: &Engine, size: bondline_model::PageSize) -> [f64; 2] {
    let center = engine.view().center;
    [
        center
            .map_or(size.width_pt() / 2.0, Point2::x_pt)
            .clamp(0.0, size.width_pt()),
        center
            .map_or(size.height_pt() / 2.0, Point2::y_pt)
            .clamp(0.0, size.height_pt()),
    ]
}

pub(super) fn wheel(
    engine: &Engine,
    size: bondline_model::PageSize,
    canvas: [u32; 2],
    unit: egui::MouseWheelUnit,
    delta: egui::Vec2,
) -> Option<Command> {
    if !delta.is_finite() || delta == egui::Vec2::ZERO {
        return None;
    }
    let scale = engine.view().zoom_percent? / 100.0;
    let [x, y] = center(engine, size);
    let factor = match unit {
        egui::MouseWheelUnit::Point => [1.0, 1.0],
        egui::MouseWheelUnit::Line => [24.0, 24.0],
        egui::MouseWheelUnit::Page => [f64::from(canvas[0]) * 0.8, f64::from(canvas[1]) * 0.8],
    };
    Some(Command::UiView(bondline_commands::UiView::Pan {
        center_x_pt: (x - f64::from(delta.x) * factor[0] / scale).clamp(0.0, size.width_pt()),
        center_y_pt: (y - f64::from(delta.y) * factor[1] / scale).clamp(0.0, size.height_pt()),
    }))
}

/// Numeric edits commit through the registry at their input event, never during painting.
pub(super) fn zoom(
    ui: &mut egui::Ui,
    engine: &Engine,
    events: &[egui::Event],
) -> Vec<(usize, Action)> {
    let enabled = engine
        .controls()
        .iter()
        .any(|row| row.id == "view.actual" && row.enabled);
    let known_percent = engine.view_percent();
    let fit = engine.view().zoom_percent.is_none()
        && match engine.content() {
            bondline_commands::StoredContent::Sdf { .. } => true,
            bondline_commands::StoredContent::Drawing(_) => known_percent.is_none(),
            bondline_commands::StoredContent::SmilesFile(_) => false,
        };
    let mut percent = known_percent.filter(|_| !fit).unwrap_or(100.0);
    let mut submitted = None;
    let mut actions = Vec::new();
    ui.push_id("view-zoom", |ui| {
        ui.add_enabled_ui(enabled, |ui| {
            let label = ui.label("Zoom");
            // Explicit100 must commit from Fit even when it equals the editor's seed.
            let mut editor = egui::DragValue::from_get_set(|value| {
                if let Some(value) = value {
                    percent = value;
                    submitted = Some(value);
                }
                percent
            })
            .range(1.0..=999.0)
            .clamp_existing_to_range(false)
            .speed(1.0)
            .max_decimals(2)
            .suffix(if fit { "" } else { "%" })
            .update_while_editing(false);
            if fit {
                editor = editor.custom_formatter(|_, _| "Fit".to_owned());
                ui.style_mut().explanation_tooltips = false;
            }
            let response = ui.add(editor).labelled_by(label.id);
            if fit {
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.clear_numeric_value();
                });
            }
            if enabled {
                actions.extend(
                    numeric_requests(&response, events, 1.0, 999.0)
                        .into_iter()
                        .map(|(index, value)| (index, Action::Zoom(value))),
                );
            }
            if enabled
                && actions.is_empty()
                && let Some(value) = submitted
                && let Some(index) = change_event(&response, events)
            {
                actions.push((index, Action::Zoom(value)));
            }
        });
    });
    actions
}

fn numeric_requests(
    response: &egui::Response,
    events: &[egui::Event],
    minimum: f64,
    maximum: f64,
) -> Vec<(usize, f64)> {
    events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| {
            if let egui::Event::AccessKitActionRequest(request) = event
                && request.target.0 == response.id.value()
                && request.action == egui::accesskit::Action::SetValue
                && let Some(egui::accesskit::ActionData::NumericValue(value)) = request.data
            {
                Some((
                    index,
                    if value.is_finite() {
                        value.clamp(minimum, maximum)
                    } else {
                        value
                    },
                ))
            } else {
                None
            }
        })
        .collect()
}

fn change_event(response: &egui::Response, events: &[egui::Event]) -> Option<usize> {
    events.iter().enumerate().rev().find_map(|(index, event)| {
        let applies = match event {
            egui::Event::PointerMoved(_) => response.dragged() || response.drag_stopped(),
            egui::Event::PointerButton {
                button: egui::PointerButton::Primary,
                pressed: true,
                ..
            } => response.lost_focus(),
            egui::Event::Key {
                key:
                    egui::Key::Enter
                    | egui::Key::Tab
                    | egui::Key::ArrowUp
                    | egui::Key::ArrowDown
                    | egui::Key::ArrowLeft
                    | egui::Key::ArrowRight,
                pressed: true,
                ..
            } => true,
            egui::Event::AccessKitActionRequest(request) => {
                request.target.0 == response.id.value()
                    && matches!(
                        request.action,
                        egui::accesskit::Action::SetValue
                            | egui::accesskit::Action::Increment
                            | egui::accesskit::Action::Decrement
                    )
            }
            _ => false,
        };
        applies.then_some(index)
    })
}

/// Overlay position controls leave the drawing viewport unchanged, including Fit geometry.
pub(super) fn pan(
    ui: &egui::Ui,
    rect: egui::Rect,
    engine: &Engine,
    size: bondline_model::PageSize,
    events: &[egui::Event],
    blocked: bool,
) -> Vec<(usize, Action)> {
    let mut actions = Vec::new();
    if engine.view().zoom_percent.is_none() || rect.width() < 32.0 || rect.height() < 32.0 {
        return actions;
    }
    let [mut x, mut y] = center(engine, size);
    for vertical in [false, true] {
        let bounds = if vertical {
            egui::Rect::from_min_max(
                egui::pos2(rect.right() - 14.0, rect.top()),
                egui::pos2(rect.right(), rect.bottom() - 14.0),
            )
        } else {
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.bottom() - 14.0),
                egui::pos2(rect.right() - 14.0, rect.bottom()),
            )
        };
        let name = if vertical {
            "Vertical position"
        } else {
            "Horizontal position"
        };
        egui::Area::new(ui.make_persistent_id(name))
            .order(egui::Order::Foreground)
            .fixed_pos(bounds.min)
            .default_size(bounds.size())
            .movable(false)
            .constrain(false)
            .sense(egui::Sense::hover())
            .show(ui.ctx(), |ui| {
                ui.set_min_size(bounds.size());
                ui.set_max_size(bounds.size());
                ui.set_clip_rect(bounds);
                ui.spacing_mut().slider_width = if vertical {
                    bounds.height()
                } else {
                    bounds.width()
                };
                ui.add_enabled_ui(!blocked, |ui| {
                    let slider = if vertical {
                        egui::Slider::new(&mut y, size.height_pt()..=0.0)
                            .vertical()
                            .show_value(false)
                    } else {
                        egui::Slider::new(&mut x, 0.0..=size.width_pt()).show_value(false)
                    };
                    let response = ui.add_sized(bounds.size(), slider);
                    response.widget_info(|| {
                        egui::WidgetInfo::slider(!blocked, if vertical { y } else { x }, name)
                    });
                    if vertical {
                        ui.ctx().accesskit_node_builder(response.id, |node| {
                            node.set_min_numeric_value(0.0);
                            node.set_max_numeric_value(size.height_pt());
                            if y < size.height_pt() {
                                node.add_action(egui::accesskit::Action::Increment);
                            }
                            if y > 0.0 {
                                node.add_action(egui::accesskit::Action::Decrement);
                            }
                        });
                    }
                    let maximum = if vertical {
                        size.height_pt()
                    } else {
                        size.width_pt()
                    };
                    let numeric = if blocked {
                        Vec::new()
                    } else {
                        numeric_requests(&response, events, 0.0, maximum)
                    };
                    let numeric_action = !numeric.is_empty();
                    actions.extend(numeric.into_iter().map(|(index, value)| {
                        (
                            index,
                            if vertical {
                                Action::PanY(value)
                            } else {
                                Action::PanX(value)
                            },
                        )
                    }));
                    if !numeric_action
                        && response.changed()
                        && let Some(index) = change_event(&response, events)
                    {
                        actions.push((
                            index,
                            if vertical {
                                Action::PanY(y)
                            } else {
                                Action::PanX(x)
                            },
                        ));
                    }
                });
            });
    }
    actions.sort_by_key(|(index, _)| *index);
    actions
}
