//! Roadmap presentation. Only an existing engine control can grant an executable action.
use super::{ControlInfo, activation_event, egui, tool_icons};

#[path = "tool_catalogue_data.rs"]
mod data;
pub(super) use data::ENTRIES;

#[derive(Clone, Copy)]
pub(super) enum Icon {
    Arc,
    Arrow,
    Biology,
    Bond,
    Bracket,
    Chain,
    Chair,
    CharacterMap,
    Chromatography,
    Curve,
    Dagger,
    Electron,
    Eraser,
    Fragmentation,
    Helm,
    Label,
    Lasso,
    Mapping,
    Marquee,
    Name,
    Orbital,
    PeriodicTable,
    Perspective,
    Query,
    Ring,
    Shape,
    Symbol,
    Table,
    Template,
    Text,
}

pub(super) struct Entry {
    pub id: &'static str,
    pub source_id: &'static str,
    pub label: &'static str,
    pub group: &'static str,
    pub binding_id: Option<&'static str>,
    pub icon: Icon,
    pub variant: &'static str,
}

const GROUPS: &[&str] = &[
    "Selection",
    "Bonds",
    "Chains",
    "Rings",
    "Reactions",
    "Shapes",
    "Brackets",
    "Curves",
    "Labels and data",
    "Symbols",
    "Orbitals",
    "Queries",
    "Biology",
    "HELM",
];

impl Entry {
    fn matches(&self, query: &str) -> bool {
        [self.label, self.group, self.source_id]
            .iter()
            .any(|s| s.to_lowercase().contains(query))
    }

    fn control<'a>(&self, controls: &'a [ControlInfo]) -> Option<&'a ControlInfo> {
        let id = self.binding_id?;
        controls.iter().find(|c| c.id == id && c.menu == "Tools")
    }
}

#[derive(Default)]
pub(super) struct Catalogue {
    open: bool,
    query: String,
}

fn search_id() -> egui::Id {
    egui::Id::new("bondline-tool-catalogue-search")
}

impl Catalogue {
    pub(super) fn prepare_input(
        &mut self,
        context: &egui::Context,
        events: &mut Vec<egui::Event>,
        allowed: bool,
    ) {
        if !allowed {
            self.open = false;
            return;
        }
        if self.open
            && events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key {
                        key: egui::Key::Escape,
                        pressed: true,
                        ..
                    }
                )
            })
        {
            self.open = false;
            context.memory_mut(|memory| memory.surrender_focus(search_id()));
            context.input_mut(|input| {
                input.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
            });
            // The catalogue owns this Escape; it must not clear an authored selection.
            events.retain(|e| {
                !matches!(
                    e,
                    egui::Event::Key {
                        key: egui::Key::Escape,
                        ..
                    }
                )
            });
        }
    }

    pub(super) fn rail(&mut self, ui: &mut egui::Ui, allowed: bool) {
        ui.separator();
        let all = ui.add_enabled(
            allowed,
            egui::Button::new("All…").min_size(egui::vec2(56.0, 26.0)),
        );
        all.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, all.enabled(), "All tools…")
        });
        if all.clicked() {
            self.open = !self.open;
        }
        all.on_hover_text("All tools — available and planned");
        ui.separator();
        ui.scope(|ui| {
            ui.spacing_mut().scroll.floating = true;
            ui.spacing_mut().scroll.floating_allocated_width = 0.0;
            ui.spacing_mut().scroll.bar_width = 5.0;
            egui::ScrollArea::vertical()
                .id_salt("planned-tool-rail")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for group in GROUPS {
                        let planned: Vec<_> = ENTRIES
                            .iter()
                            .filter(|e| e.group == *group && e.binding_id.is_none())
                            .collect();
                        if planned.is_empty() {
                            continue;
                        }
                        let heading: &str = if *group == "Labels and data" {
                            "Labels"
                        } else {
                            group
                        };
                        ui.label(egui::RichText::new(heading).size(9.0).weak());
                        for row in planned.chunks(2) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(2.0, 6.0);
                                for entry in row {
                                    unavailable_button(ui, entry);
                                }
                            });
                        }
                        ui.add_space(5.0);
                    }
                });
        });
    }

    pub(super) fn show(
        &mut self,
        context: &egui::Context,
        controls: &[ControlInfo],
        events: &[egui::Event],
    ) -> Vec<(usize, &'static str)> {
        if !self.open {
            return Vec::new();
        }
        let mut open = self.open;
        let mut actions = Vec::new();
        egui::Window::new("All tools")
            .id(egui::Id::new("bondline-tool-catalogue"))
            .open(&mut open)
            .collapsible(false)
            .default_pos([84.0, 72.0])
            .default_width(470.0)
            .min_width(300.0)
            .max_height((context.screen_rect().height() - 96.0).max(120.0))
            .show(context, |ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .id(search_id())
                        .hint_text("Search tools")
                        .char_limit(128)
                        .desired_width(f32::INFINITY),
                );
                context.accesskit_node_builder(response.id, |node| node.set_name("Search tools"));
                let available = ENTRIES
                    .iter()
                    .filter_map(|e| e.control(controls))
                    .filter(|c| c.enabled)
                    .count();
                let planned = ENTRIES.iter().filter(|e| e.binding_id.is_none()).count();
                ui.label(
                    egui::RichText::new(format!("{available} available here · {planned} planned"))
                        .small()
                        .weak(),
                );
                ui.separator();
                let query = self.query.trim().to_lowercase();
                let mut scroll = egui::ScrollArea::vertical()
                    .id_salt("all-tool-entries")
                    .max_height((context.screen_rect().height() - 210.0).max(80.0));
                if response.changed() {
                    scroll = scroll.vertical_scroll_offset(0.0);
                }
                scroll.show(ui, |ui| {
                    let mut found = false;
                    for group in GROUPS {
                        let entries: Vec<_> = ENTRIES
                            .iter()
                            .filter(|e| e.group == *group && e.matches(&query))
                            .collect();
                        if entries.is_empty() {
                            continue;
                        }
                        found = true;
                        ui.label(egui::RichText::new(*group).small().strong());
                        for entry in entries {
                            ui.push_id(entry.id, |ui| {
                                let control = entry.control(controls);
                                let response = entry_button(ui, entry, control, true);
                                if let Some(control) = control.filter(|_| response.enabled()) {
                                    for (index, event) in events.iter().enumerate() {
                                        if activation_event(&response, event) {
                                            actions.push((index, control.id));
                                        }
                                    }
                                }
                            });
                        }
                        ui.add_space(8.0);
                    }
                    if !found {
                        ui.label("No tools match this search.");
                    }
                });
                ui.separator();
                ui.label(
                    egui::RichText::new(
                        "Planned tools stay grey until implemented. Esc closes this catalogue.",
                    )
                    .small()
                    .weak(),
                );
            });
        self.open = open && actions.is_empty();
        actions
    }
}

pub(super) fn unavailable_button(ui: &mut egui::Ui, entry: &Entry) {
    ui.push_id(entry.id, |ui| {
        entry_button(ui, entry, None, false);
    });
}

fn entry_button(
    ui: &mut egui::Ui,
    entry: &Entry,
    control: Option<&ControlInfo>,
    labeled: bool,
) -> egui::Response {
    let enabled = control.is_some_and(|c| c.enabled);
    let selected = control.and_then(|c| c.selected).unwrap_or(false);
    let width = if labeled { ui.available_width() } else { 28.0 };
    let response = ui.add_enabled(
        enabled,
        egui::Button::new("")
            .min_size(egui::vec2(width, 28.0))
            .selected(selected),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            response.enabled(),
            selected,
            entry.label,
        )
    });
    let reason = if entry.binding_id.is_none() {
        "Not implemented"
    } else if !enabled {
        "Unavailable for this document"
    } else {
        "Available"
    };
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_description(reason));
    let color = if response.enabled() {
        ui.style().interact(&response).fg_stroke.color
    } else {
        egui::Color32::from_gray(152)
    };
    let icon = egui::Rect::from_min_size(
        egui::pos2(response.rect.left(), response.rect.center().y - 14.0),
        egui::vec2(28.0, 28.0),
    );
    tool_icons::paint(ui.painter(), icon, entry, color);
    if labeled {
        ui.painter().text(
            egui::pos2(response.rect.left() + 36.0, response.rect.center().y),
            egui::Align2::LEFT_CENTER,
            entry.label,
            egui::FontId::proportional(13.0),
            color,
        );
    }
    let tooltip = format!("{}\n{reason}\n{}", entry.label, entry.source_id);
    response
        .on_hover_text(&tooltip)
        .on_disabled_hover_text(tooltip)
}
