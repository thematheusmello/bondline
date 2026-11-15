//! Native GPU workspace. Authoring controls dispatch through the shared command registry.

#[cfg(test)]
mod bond_history_tests;
#[cfg(test)]
mod bond_order_tests;
mod file_workflow;
pub mod files;
#[cfg(test)]
mod mol_display_tests;
#[cfg(test)]
mod multiple_bond_tests;
mod preview;
mod sdf_draft_text;
mod sdf_editor;
mod sdf_fields;
mod smiles_records;
mod view_controls;
#[cfg(test)]
mod view_tests;
#[cfg(test)]
mod visual_capture_tests;
#[cfg(test)]
mod workspace_presentation_tests;

use bondline_commands::{
    BondCreate, BondDrawOrder, BondExtend, BondSetOrder, BondToolState, CloseState, Command,
    ControlInfo, ControlKey, ControlShortcut, Engine, ExtendHydrogenPolicy, OrderHydrogenPolicy,
};
use bondline_mcp::{CommandExecutor, SharedEngine, control::LoopbackServer};
use bondline_model::{PageId, Point2};
use eframe::egui;
use file_workflow::{FileIntentAction, FileWorkflow};
use preview::PreviewWorkflow;
use sdf_fields::SdfFieldsWorkflow;
use smiles_records::SmilesRecordsWorkflow;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

const ACCENT: egui::Color32 = egui::Color32::from_rgb(21, 117, 118);

/// Native startup configuration: use the audited wgpu backend on every desktop platform.
pub fn native_options() -> Result<eframe::NativeOptions, bondline_render::Error> {
    Ok(eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1160.0, 800.0])
            .with_min_inner_size([760.0, 520.0])
            .with_icon(original_icon()?),
        ..Default::default()
    })
}

fn original_icon() -> Result<egui::IconData, bondline_render::Error> {
    let mut frame = bondline_render::cpu_solid(32, 32, [23, 81, 115, 255])?;
    for (index, pixel) in frame.rgba.chunks_exact_mut(4).enumerate() {
        let x = (index % 32) as i32;
        let y = (index / 32) as i32;
        let bond = (6..=25).contains(&x) && ((y - 10).abs() <= 1 || (y - 21).abs() <= 1)
            || (9..=22).contains(&y) && ((x - 6).abs() <= 1 || (x - 25).abs() <= 1);
        if bond {
            pixel.copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    Ok(egui::IconData {
        rgba: frame.rgba,
        width: frame.width,
        height: frame.height,
    })
}

/// Install only the independently sourced Apache-2.0 UI font.
pub fn configure_context(context: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "bondline-ui".to_owned(),
        egui::FontData::from_static(include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .insert(family, vec!["bondline-ui".to_owned()]);
    }
    context.set_fonts(fonts);
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = egui::Color32::from_rgb(246, 248, 250);
    visuals.window_fill = egui::Color32::from_rgb(252, 253, 254);
    visuals.override_text_color = Some(egui::Color32::from_rgb(37, 48, 61));
    visuals.selection.bg_fill = egui::Color32::from_rgb(222, 241, 240);
    visuals.selection.stroke = egui::Stroke::new(1.5, ACCENT);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(216, 223, 230));
    context.set_visuals(visuals);
    context.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        for text_style in [egui::TextStyle::Body, egui::TextStyle::Button] {
            style
                .text_styles
                .insert(text_style, egui::FontId::proportional(13.0));
        }
    });
}

#[derive(Default)]
pub struct DrawingState {
    gesture: Option<BondGesture>,
    hover: Option<bondline_render::HitTarget>,
    hover_context: Option<HoverContext>,
    hover_fresh: bool,
    key_hotspot: Option<BondHotspot>,
    navigation_position: Option<egui::Pos2>,
}

impl DrawingState {
    /// Transient result from the current viewport; never part of authored history.
    pub fn hover_target(&self) -> Option<bondline_render::HitTarget> {
        self.hover
    }

    fn invalidate_hover(&mut self) {
        self.hover = None;
        self.hover_context = None;
        self.hover_fresh = false;
        self.key_hotspot = None;
        self.navigation_position = None;
    }
}

#[derive(Clone, Copy)]
struct BondHotspot {
    position: egui::Pos2,
    target: Option<bondline_render::HitTarget>,
}

#[derive(Clone, Copy, PartialEq)]
struct HoverContext {
    revision: u64,
    tool: BondToolState,
    page: PageId,
    width: u32,
    height: u32,
    canvas_origin: egui::Pos2,
    host_origin: Option<egui::Pos2>,
    pixels_per_point: f32,
    view_epoch: u64,
}

struct BondGesture {
    begin: Point2,
    end: Point2,
    press: egui::Pos2,
    modifiers: egui::Modifiers,
    context: HoverContext,
    target: Option<bondline_render::HitTarget>,
}

fn clicked_bond_order(
    existing: bondline_model::BondOrder,
    tool: BondDrawOrder,
) -> Result<BondDrawOrder, bondline_commands::Error> {
    use bondline_model::BondOrder;
    match (existing, tool) {
        (BondOrder::Single, BondDrawOrder::Triple) => Ok(BondDrawOrder::Triple),
        (BondOrder::Single, _) => Ok(BondDrawOrder::Double),
        (BondOrder::Double, BondDrawOrder::Single | BondDrawOrder::Double) => {
            Err(bondline_commands::Error::UnsupportedBondConversion(
                "Double-bond alignment is unavailable. Hover over the bond and press 1, 2 or 3 to set its order.",
            ))
        }
        _ => Err(bondline_commands::Error::UnsupportedBondConversion(
            "Use 1, 2 or 3 over this bond to change its order.",
        )),
    }
}

fn paint_stroke(
    painter: &egui::Painter,
    origin: egui::Pos2,
    stroke: &bondline_render::Stroke,
) -> Result<(), bondline_render::Error> {
    let [r, g, b, a] = stroke.color;
    painter.add(egui::Shape::convex_polygon(
        stroke
            .corners()?
            .into_iter()
            .map(|[x, y]| origin + egui::vec2(x, y))
            .collect(),
        egui::Color32::from_rgba_unmultiplied(r, g, b, a),
        egui::Stroke::NONE,
    ));
    Ok(())
}

fn bond_tool_buttons(
    ui: &mut egui::Ui,
    controls: &[ControlInfo],
    events: &[egui::Event],
) -> Vec<(usize, Command)> {
    let mut commands = Vec::new();
    let tools: Vec<_> = controls
        .iter()
        .filter(|control| control.menu == "Tools")
        .collect();
    for row in tools.chunks(2) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(2.0, 6.0);
            for control in row {
                let Command::UiBondToolSelect(params) = &control.command else {
                    continue;
                };
                ui.push_id(control.id, |ui| {
                    let selected = control.selected.unwrap_or(false);
                    let response = ui
                        .add_enabled(
                            control.enabled,
                            egui::Button::new("")
                                .min_size(egui::vec2(28.0, 28.0))
                                .selected(selected),
                        )
                        .on_hover_text(control.label);
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::Button,
                            response.enabled(),
                            selected,
                            control.label,
                        )
                    });
                    let offsets: &[f32] = match params.order {
                        BondDrawOrder::Single => &[0.0],
                        BondDrawOrder::Double => &[-2.0, 2.0],
                        BondDrawOrder::Triple => &[-4.0, 0.0, 4.0],
                    };
                    let color = ui.style().interact(&response).fg_stroke.color;
                    for offset in offsets {
                        let y = response.rect.center().y + offset;
                        ui.painter().line_segment(
                            [
                                egui::pos2(response.rect.left() + 6.0, y),
                                egui::pos2(response.rect.right() - 6.0, y),
                            ],
                            egui::Stroke::new(1.5, color),
                        );
                    }
                    if response.enabled() {
                        for (index, event) in events.iter().enumerate() {
                            if activation_event(&response, event) {
                                commands.push((index, control.command.clone()));
                            }
                        }
                    }
                });
            }
        });
    }
    commands.sort_by_key(|(index, _)| *index);
    commands
}

fn activation_event(response: &egui::Response, event: &egui::Event) -> bool {
    match event {
        egui::Event::AccessKitActionRequest(request) => {
            request.action == egui::accesskit::Action::Default
                && request.target.0 == response.id.value()
        }
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            ..
        } => {
            response.clicked_by(egui::PointerButton::Primary)
                && response.interact_rect.contains(*pos)
        }
        egui::Event::Key {
            key: egui::Key::Space | egui::Key::Enter,
            pressed: true,
            ..
        } => response.has_focus() && response.clicked(),
        _ => false,
    }
}

fn document_inspector(ui: &mut egui::Ui, engine: &Engine, error: Option<&str>) {
    let state = engine.state();
    ui.heading("Document");
    if let Some(message) = error {
        ui.colored_label(egui::Color32::DARK_RED, message);
    }
    ui.add_space(8.0);
    ui.weak("FILE");
    let name = state
        .file
        .path
        .as_deref()
        .and_then(|path| std::path::Path::new(path).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled");
    ui.label(egui::RichText::new(name).strong());
    ui.weak(if state.file.dirty {
        "Unsaved changes"
    } else if state.file.path.is_some() {
        "Saved"
    } else {
        "New document"
    });
    if let Some(source) = &state.source {
        ui.label(source.format.to_uppercase());
    }
    ui.add_space(8.0);
    ui.separator();
    if let Ok(document) = engine.document() {
        if let Some(page) = document.pages().first() {
            ui.weak("PAGE");
            ui.label(format!(
                "{:.1} × {:.1} pt",
                page.size().width_pt(),
                page.size().height_pt()
            ));
            ui.add_space(8.0);
        }
        if engine
            .stored_document()
            .is_ok_and(|stored| stored.is_fully_mapped())
        {
            let (atoms, bonds, fragments) = document
                .pages()
                .iter()
                .flat_map(|page| page.fragments())
                .fold(
                    (0_usize, 0_usize, 0_usize),
                    |(atoms, bonds, fragments), fragment| {
                        (
                            atoms.saturating_add(fragment.atoms().len()),
                            bonds.saturating_add(fragment.bonds().len()),
                            fragments.saturating_add(1),
                        )
                    },
                );
            ui.weak("DRAWN CONTENT");
            ui.label(format!("{atoms} atoms"));
            ui.label(format!("{bonds} bonds"));
            ui.label(format!("{fragments} fragments"));
        } else {
            ui.label("Original source preserved");
        }
        ui.add_space(8.0);
        ui.separator();
        ui.weak("ACTIVE TOOL");
        let tool = match engine.bond_tool().order {
            BondDrawOrder::Single => "Single bond",
            BondDrawOrder::Double => "Double bond",
            BondDrawOrder::Triple => "Triple bond",
        };
        ui.label(
            egui::RichText::new(format!("{tool} selected"))
                .color(ACCENT)
                .strong(),
        );
        ui.label(match engine.bond_tool().order {
            BondDrawOrder::Single => {
                "Drag paper for C-C; extend terminal CH3 to CH2 / CH2 to CH / CH to C; Esc cancels"
            }
            BondDrawOrder::Double => {
                "Drag paper for C=C; extend terminal CH3 to CH / CH2 to C; Esc cancels"
            }
            BondDrawOrder::Triple => "Drag paper for C≡C; extend terminal CH3 to C; Esc cancels",
        });
    } else if let Some(collection) = engine.content().collection() {
        ui.label(format!("{} records", collection.record_count));
        ui.weak("Read-only chemical preview");
    } else if let Some(summary) = engine.content().smiles_summary() {
        ui.label(format!("{} records", summary.record_count));
        ui.weak("Source file preserved");
    }
}

fn paint_hover(
    painter: &egui::Painter,
    origin: egui::Pos2,
    page: &bondline_model::Page,
    viewport: &bondline_render::PageView,
    graph: &bondline_render::GraphScene,
    target: bondline_render::HitTarget,
) -> Result<(), bondline_render::Error> {
    use bondline_render::{Error, HitTarget};
    let fragment_id = match target {
        HitTarget::Atom { fragment_id, .. } | HitTarget::Bond { fragment_id, .. } => fragment_id,
    };
    let fragment = page
        .fragments()
        .iter()
        .find(|f| f.id() == fragment_id)
        .ok_or(Error::UnsupportedChemicalContent)?;
    let position = |id| {
        fragment
            .atoms()
            .iter()
            .find(|a| a.id == id)
            .and_then(|a| a.position)
            .ok_or(Error::UnsupportedChemicalContent)
            .and_then(|p| viewport.canvas_position(p))
    };
    let color = egui::Color32::from_rgb(40, 120, 180);
    match target {
        HitTarget::Atom { atom_id, .. } => {
            let [x, y] = position(atom_id)?;
            painter.circle_stroke(
                origin + egui::vec2(x, y),
                6.0,
                egui::Stroke::new(1.0, color),
            );
        }
        HitTarget::Bond { bond_id, .. } => {
            let mut components = graph.bond_strokes(fragment_id, bond_id);
            let first = components.next().ok_or(Error::UnsupportedChemicalContent)?;
            for stroke in std::iter::once(first).chain(components) {
                let outline = bondline_render::Stroke {
                    width: stroke.width + 4.0,
                    ..stroke.clone()
                }
                .corners()?;
                painter.add(egui::Shape::closed_line(
                    outline
                        .into_iter()
                        .map(|[x, y]| origin + egui::vec2(x, y))
                        .collect(),
                    egui::Stroke::new(1.0, color),
                ));
            }
        }
    }
    Ok(())
}

/// Paint document state and route every authored edit through the engine.
pub fn show_workspace(
    context: &egui::Context,
    engine: &mut Engine,
    error: &mut Option<String>,
    drawing: &mut DrawingState,
) {
    let _ = show_workspace_with_files(context, engine, error, drawing, None, false);
}

fn control_shortcut_key(shortcut: ControlShortcut) -> egui::KeyboardShortcut {
    let modifiers = egui::Modifiers {
        command: shortcut.primary,
        shift: shortcut.shift,
        ..egui::Modifiers::NONE
    };
    let key = match shortcut.key {
        ControlKey::O => egui::Key::O,
        ControlKey::S => egui::Key::S,
        ControlKey::W => egui::Key::W,
        ControlKey::Z => egui::Key::Z,
        ControlKey::Y => egui::Key::Y,
        ControlKey::F5 => egui::Key::F5,
        ControlKey::F6 => egui::Key::F6,
        ControlKey::F7 => egui::Key::F7,
        ControlKey::F8 => egui::Key::F8,
    };
    egui::KeyboardShortcut::new(modifiers, key)
}

fn text_edit_focused(context: &egui::Context) -> bool {
    context
        .memory(|memory| memory.focused())
        .is_some_and(|id| egui::TextEdit::load_state(context, id).is_some())
}

fn workspace_shortcut(engine: &Engine, event: &egui::Event, text_focused: bool) -> Option<Command> {
    let egui::Event::Key {
        key,
        pressed: true,
        modifiers,
        ..
    } = event
    else {
        return None;
    };
    engine.controls().into_iter().find_map(|control| {
        let shortcut = control.shortcut.map(control_shortcut_key)?;
        ((control.menu == "File" || (!text_focused && matches!(control.menu, "Edit" | "View")))
            && control.enabled
            && shortcut.logical_key == *key
            && modifiers.matches_exact(shortcut.modifiers))
        .then_some(control.command)
    })
}

fn pending_ui_blocks_authoring(state: &bondline_commands::CommandResult) -> bool {
    state.close != CloseState::Open
        || state
            .ui_file
            .request
            .as_ref()
            .is_some_and(|request| request.phase != bondline_commands::UiFilePhase::Running)
}

struct ShortcutRouting<'a> {
    actions: &'a [(usize, view_controls::Action)],
    fields: Option<&'a mut SdfFieldsWorkflow>,
    canvas: Option<(egui::Rect, egui::LayerId)>,
}

fn replay_edit_shortcuts(
    context: &egui::Context,
    engine: &mut Engine,
    error: &mut Option<String>,
    drawing: &mut DrawingState,
    events: &[egui::Event],
    blocked: bool,
    routing: ShortcutRouting<'_>,
) {
    let ShortcutRouting {
        actions,
        mut fields,
        canvas,
    } = routing;
    if blocked && fields.is_none() {
        return;
    }
    let text_focused = text_edit_focused(context);
    if blocked || text_focused || !context.input(|input| input.focused) {
        drawing.navigation_position = None;
    }
    let mut focused = context.input(|input| input.focused);
    for event in events.iter().rev() {
        if let egui::Event::WindowFocused(value) = event {
            focused = !value;
        }
    }
    for (index, event) in events.iter().enumerate() {
        if let Some(fields) = fields.as_deref_mut() {
            fields.replay_editor(index, engine, context);
        }
        if blocked || pending_ui_blocks_authoring(&engine.state()) {
            continue;
        }
        if let egui::Event::WindowFocused(value) = event {
            focused = *value;
            if !focused {
                drawing.navigation_position = None;
            }
        }
        if canvas.is_some()
            && focused
            && !text_focused
            && let egui::Event::PointerMoved(pos) | egui::Event::PointerButton { pos, .. } = event
        {
            drawing.navigation_position = Some(*pos);
        }
        let command = actions
            .iter()
            .find(|(at, action)| {
                *at == index && (!text_focused
                    || matches!(action, view_controls::Action::Zoom(_))
                    || matches!(action, view_controls::Action::Control(id) if id.starts_with("file.")))
            })
            .and_then(|(_, action)| action.command(engine))
            .or_else(|| workspace_shortcut(engine, event, text_focused))
            .or_else(|| {
                let (rect, layer) = canvas?;
                if text_focused || !matches!(engine.content(), bondline_commands::StoredContent::Sdf { .. })
                    || context.input(|input| input.key_down(egui::Key::Escape)) {
                    return None;
                }
                let egui::Event::MouseWheel { unit, delta, modifiers } = event else { return None; };
                if !modifiers.is_none() { return None; }
                let pos = drawing.navigation_position?;
                let size = engine.view_page_size().ok()?;
                let width = rect.width().floor().max(1.0) as u32;
                let height = rect.height().floor().max(1.0) as u32;
                let viewport = engine.page_view(width, height, PageId::new(1)?).ok()?;
                let local = pos - rect.min;
                if !rect.contains(pos) || context.layer_id_at(pos) != Some(layer)
                    || !viewport.contains([local.x, local.y]) {
                    drawing.navigation_position = None;
                    return None;
                }
                view_controls::wheel(engine, size, [width, height], *unit, *delta)
            });
        if focused && let Some(command) = command {
            let explicit = actions.iter().find_map(|(at, action)| {
                if *at == index
                    && let view_controls::Action::Control(id) = action
                {
                    Some(*id)
                } else {
                    None
                }
            });
            if fields.as_deref_mut().is_some_and(|fields| {
                fields.intercept_editor_command(&command, engine, context, explicit)
            }) {
                continue;
            }
            let view_command = matches!(&command, Command::UiView(_));
            let previous_view = engine.view();
            *error = engine.execute(command).err().map(|error| error.to_string());
            if view_command && engine.view() == previous_view {
                continue;
            }
            let position = view_command
                .then_some(drawing.navigation_position)
                .flatten();
            drawing.gesture = None;
            drawing.invalidate_hover();
            drawing.navigation_position = position;
            context.request_repaint();
        }
    }
}

fn control_command(engine: &Engine, id: &str) -> Option<Command> {
    engine
        .controls()
        .into_iter()
        .find(|control| control.id == id && control.enabled)
        .map(|control| control.command)
}

fn control_menu_action(
    ui: &mut egui::Ui,
    controls: &[ControlInfo],
    menu: &str,
    events: &[egui::Event],
) -> Option<(usize, &'static str)> {
    let mut selected = None;
    ui.menu_button(menu, |ui| {
        for control in controls.iter().filter(|control| control.menu == menu) {
            let mut button = egui::Button::new(control.label);
            if let Some(shortcut) = control.shortcut {
                button =
                    button.shortcut_text(ui.ctx().format_shortcut(&control_shortcut_key(shortcut)));
            }
            let response = ui.add_enabled(control.enabled, button);
            if response.clicked() {
                selected = events
                    .iter()
                    .position(|event| activation_event(&response, event))
                    .map(|index| (index, control.id));
                ui.close_menu();
            }
        }
    });
    selected
}

fn show_workspace_with_files(
    context: &egui::Context,
    engine: &mut Engine,
    error: &mut Option<String>,
    drawing: &mut DrawingState,
    files: Option<&mut FileWorkflow>,
    active_work: bool,
) -> Option<FileIntentAction> {
    show_workspace_with_preview(context, engine, error, drawing, files, active_work, None)
}

struct WorkspaceWorkflows<'a> {
    preview: &'a mut PreviewWorkflow,
    fields: &'a mut SdfFieldsWorkflow,
    smiles: &'a mut SmilesRecordsWorkflow,
}

fn show_workspace_with_preview(
    context: &egui::Context,
    engine: &mut Engine,
    error: &mut Option<String>,
    drawing: &mut DrawingState,
    mut files: Option<&mut FileWorkflow>,
    active_work: bool,
    workflows: Option<WorkspaceWorkflows<'_>>,
) -> Option<FileIntentAction> {
    let (mut preview, mut sdf_fields, mut smiles_records) = match workflows {
        Some(workflows) => (
            Some(workflows.preview),
            Some(workflows.fields),
            Some(workflows.smiles),
        ),
        None => (None, None, None),
    };
    if let Some(fields) = sdf_fields.as_deref_mut() {
        fields.begin_ui();
    }
    let events = context.input(|input| input.events.clone());
    if [
        context.screen_rect().width(),
        context.screen_rect().height(),
    ]
    .iter()
    .any(|dimension| !dimension.is_finite() || !(1.0..=100_000.0).contains(dimension))
    {
        drawing.gesture = None;
        drawing.invalidate_hover();
        let current = engine.state();
        let blocked = current.close != CloseState::Open
            || files
                .as_ref()
                .is_some_and(|files| files.blocks_authoring(&current));
        replay_edit_shortcuts(
            context,
            engine,
            error,
            drawing,
            &events,
            blocked,
            ShortcutRouting {
                actions: &[],
                fields: sdf_fields.as_deref_mut(),
                canvas: None,
            },
        );
        egui::CentralPanel::default().show(context, |ui| {
            ui.colored_label(
                egui::Color32::DARK_RED,
                bondline_render::Error::InvalidViewport.to_string(),
            );
        });
        return None;
    }
    let state = engine.state();
    let initial_tool = engine.bond_tool();
    let mut tool_actions = Vec::new();
    let mut view_actions = Vec::new();
    let mut selected = None;
    let mut file_action = None;
    egui::TopBottomPanel::top("title")
        .exact_height(48.0)
        .frame(egui::Frame::side_top_panel(&context.style()).inner_margin(8.0))
        .show(context, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("bondline")
                        .strong()
                        .size(20.0)
                        .color(ACCENT),
                );
                ui.separator();
                let file = files
                    .as_deref_mut()
                    .and_then(|files| files.menu(ui, engine, &events));
                let current = engine.state();
                let blocked = current.close != CloseState::Open
                    || files
                        .as_ref()
                        .is_some_and(|files| files.blocks_authoring(&current));
                ui.add_enabled_ui(!blocked, |ui| {
                    let controls = engine.controls();
                    let edit = control_menu_action(ui, &controls, "Edit", &events);
                    let page = control_menu_action(ui, &controls, "Page", &events);
                    let view = control_menu_action(ui, &controls, "View", &events);
                    selected = file.or(edit).or(page).or(view);
                });
                ui.separator();
                let name = state
                    .file
                    .path
                    .as_deref()
                    .and_then(|path| std::path::Path::new(path).file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or("Untitled");
                let name_width = (ui.available_width() - 150.0).clamp(24.0, 360.0);
                ui.add_sized(
                    [name_width, 24.0],
                    egui::Label::new(format!(
                        "{name}{}",
                        if state.file.dirty { " *" } else { "" }
                    ))
                    .truncate(),
                )
                .on_hover_text(name);
                view_actions.extend(view_controls::zoom(ui, engine, &events));
            });
        });
    let current = engine.state();
    let blocked = current.close != CloseState::Open
        || files
            .as_ref()
            .is_some_and(|files| files.blocks_authoring(&current));
    egui::SidePanel::left("drawing-tools")
        .exact_width(64.0)
        .resizable(false)
        .frame(egui::Frame::side_top_panel(&context.style()).inner_margin(3.0))
        .show(context, |ui| {
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Tools").small().strong());
            ui.separator();
            ui.add_enabled_ui(!blocked, |ui| {
                tool_actions = bond_tool_buttons(ui, &engine.controls(), &events);
            });
        });
    if blocked {
        drawing.gesture = None;
    }
    // Replay each transient state at its event; a later activation cannot relabel a press.
    let mut tool_states = Vec::with_capacity(tool_actions.len());
    for (index, command) in tool_actions {
        match engine.execute(command) {
            Ok(_) => {
                tool_states.push((index, engine.bond_tool()));
                context.request_repaint();
            }
            Err(failure) => *error = Some(failure.to_string()),
        }
    }
    if drawing.gesture.as_ref().is_some_and(|gesture| {
        gesture.context.revision != engine.state().revision || gesture.context.tool != initial_tool
    }) {
        drawing.gesture = None;
    }
    // Tool guidance and diagnostics stay in the inspector so the paper never shifts.
    egui::TopBottomPanel::bottom("status")
        .exact_height(24.0)
        .show(context, |ui| {
            ui.horizontal(|ui| {
                if let Ok(document) = engine.document()
                    && let Some(page) = document.pages().first()
                {
                    ui.label(format!("Page {}", page.id().get()));
                    ui.separator();
                    ui.label(format!(
                        "{:.1} x {:.1} pt",
                        page.size().width_pt(),
                        page.size().height_pt()
                    ));
                }
                if let Some(files) = files.as_deref_mut() {
                    files.job_status(ui);
                }
            });
        });
    egui::SidePanel::right("document-properties")
        .exact_width(248.0)
        .resizable(false)
        .frame(egui::Frame::side_top_panel(&context.style()).inner_margin(14.0))
        .show(context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                document_inspector(ui, engine, error.as_deref());
                if let Some(fields) = sdf_fields.as_deref_mut() {
                    fields.show(ui, engine);
                }
            });
        });
    if let Some(fields) = sdf_fields.as_deref_mut() {
        fields.show_editor(context, Some(engine), &events);
    }
    let mut history_handled = false;
    egui::CentralPanel::default()
        .frame(
            egui::Frame::default()
                .inner_margin(0.0)
                .fill(egui::Color32::from_gray(232)),
        )
        .show(context, |ui| {
            let available = ui.available_rect_before_wrap();
            drawing.hover = None;
            if let Some(summary) = engine.content().smiles_summary() {
                drawing.gesture = None;
                drawing.invalidate_hover();
                ui.label(format!(
                    "SMILES file: {} records, {} bytes",
                    summary.record_count, summary.byte_length
                ));
                ui.label("Chemical preview is unavailable for SMILES files.");
                if let Some(records) = smiles_records.as_deref_mut() {
                    records.show(ui, engine);
                }
                return;
            }
            if let Some(collection) = engine.content().collection() {
                drawing.gesture = None;
                drawing.hover = None;
                drawing.hover_fresh = false;
                drawing.key_hotspot = None;
                ui.label(format!(
                    "SDF collection: {} records, {} bytes",
                    collection.record_count, collection.byte_length
                ));
                if let Some(preview) = preview.as_deref_mut() {
                    ui.label("Read-only chemical preview");
                    egui::CollapsingHeader::new("Records").show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(160.0)
                            .show(ui, |ui| {
                                egui::Grid::new("sdf-record-summary").show(ui, |ui| {
                                    for record in &collection.records {
                                        ui.label(format!("Record {}", record.id));
                                        ui.label(record.mol_version);
                                        ui.label(format!("{} fields", record.field_count));
                                        ui.label(format!("{} bytes", record.byte_length));
                                        if let Some(fields) = sdf_fields.as_deref_mut()
                                            && ui
                                                .add_enabled(
                                                    state.close == CloseState::Open,
                                                    egui::Button::new(format!(
                                                        "Inspect record {} fields",
                                                        record.id
                                                    ))
                                                    .small(),
                                                )
                                                .clicked()
                                        {
                                            fields.inspect(engine, context, record.id);
                                            context.request_repaint();
                                        }
                                        if let Some(fields) = sdf_fields.as_deref_mut()
                                            && ui
                                                .add_enabled(
                                                    state.close == CloseState::Open
                                                        && state.ui_file.request.is_none()
                                                        && collection.record_count > 1
                                                        && !fields.is_busy(),
                                                    egui::Button::new(format!(
                                                        "Remove record {}",
                                                        record.id
                                                    ))
                                                    .small(),
                                                )
                                                .clicked()
                                        {
                                            fields.request_remove(engine, context, record.id);
                                        }
                                        ui.end_row();
                                    }
                                });
                            });
                    });
                    let rect = ui.available_rect_before_wrap();
                    let [width, height] =
                        [rect.width(), rect.height()].map(|axis| axis.floor().max(1.0) as u32);
                    let response = ui.interact(
                        rect,
                        ui.make_persistent_id("sdf-navigation"),
                        egui::Sense::hover(),
                    );
                    let Some(page_id) = PageId::new(1) else {
                        return;
                    };
                    let mut key = context.input(|input| HoverContext {
                        revision: engine.state().revision,
                        tool: engine.bond_tool(),
                        page: page_id,
                        width,
                        height,
                        canvas_origin: rect.min,
                        host_origin: input
                            .viewport()
                            .inner_rect
                            .or(input.viewport().outer_rect)
                            .map(|rect| rect.min),
                        pixels_per_point: input.pixels_per_point,
                        view_epoch: engine.view().epoch,
                    });
                    if drawing.hover_context != Some(key) {
                        drawing.invalidate_hover();
                    }
                    if let Err(failure) = engine.set_view_canvas_size(width, height) {
                        ui.colored_label(egui::Color32::DARK_RED, failure.to_string());
                        return;
                    }
                    if let Ok(size) = engine.view_page_size() {
                        view_actions
                            .extend(view_controls::pan(ui, rect, engine, size, &events, blocked));
                    }
                    // Resolve this frame's camera before scheduling or painting its image.
                    if let Some((index, id)) = selected {
                        view_actions.push((index, view_controls::Action::Control(id)));
                    }
                    replay_edit_shortcuts(
                        context,
                        engine,
                        error,
                        drawing,
                        &events,
                        blocked,
                        ShortcutRouting {
                            actions: &view_actions,
                            fields: sdf_fields.as_deref_mut(),
                            canvas: Some((rect, response.layer_id)),
                        },
                    );
                    history_handled = true;
                    if engine.content().collection().is_none() {
                        context.request_repaint();
                        return;
                    }
                    key.revision = engine.state().revision;
                    key.view_epoch = engine.view().epoch;
                    key.tool = engine.bond_tool();
                    drawing.hover_context = Some(key);
                    preview.show(ui, engine, rect);
                    return;
                }
                drawing.invalidate_hover();
                ui.label("Chemical preview is unavailable for SDF collections.");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("sdf-record-summary").show(ui, |ui| {
                        ui.label("Record");
                        ui.label("MOL version");
                        ui.label("Fields");
                        ui.label("Bytes");
                        ui.end_row();
                        for record in collection.records {
                            ui.label(record.id.to_string());
                            ui.label(record.mol_version);
                            ui.label(record.field_count.to_string());
                            ui.label(record.byte_length.to_string());
                            ui.end_row();
                        }
                    });
                });
                return;
            }
            let stored = match engine.stored_document() {
                Ok(stored) => stored,
                Err(error) => {
                    drawing.gesture = None;
                    drawing.invalidate_hover();
                    ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                    return;
                }
            };
            if !stored.is_fully_mapped() {
                drawing.gesture = None;
                drawing.invalidate_hover();
                ui.colored_label(
                    egui::Color32::DARK_RED,
                    bondline_commands::Error::UnsupportedDocumentPreview.to_string(),
                );
                return;
            }
            let Some(mut document) = engine.state().document else {
                drawing.gesture = None;
                drawing.invalidate_hover();
                return;
            };
            let Some(page) = document.pages().first() else {
                drawing.gesture = None;
                drawing.invalidate_hover();
                return;
            };
            let width = available.width().floor().max(1.0) as u32;
            let height = available.height().floor().max(1.0) as u32;
            let previous_percent = engine.view_percent();
            if let Err(failure) = engine.set_view_canvas_size(width, height) {
                drawing.gesture = None;
                drawing.invalidate_hover();
                ui.colored_label(egui::Color32::DARK_RED, failure.to_string());
                return;
            }
            if engine.view_percent() != previous_percent {
                context.request_repaint();
            }
            let mut viewport = match engine.page_view(width, height, page.id()) {
                Ok(viewport) => viewport,
                Err(error) => {
                    drawing.gesture = None;
                    drawing.invalidate_hover();
                    ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                    return;
                }
            };
            view_actions.extend(view_controls::pan(
                ui,
                available,
                engine,
                page.size(),
                &events,
                blocked,
            ));
            let response = ui.interact(
                available,
                ui.make_persistent_id("drawing-canvas"),
                egui::Sense::drag(),
            );
            let local = |position: egui::Pos2| {
                let position = position - available.min;
                [position.x, position.y]
            };
            let mut graph = engine.drawing_view_scene(
                width,
                height,
                page.id(),
                &bondline_render::Cancellation::default(),
            );
            if graph.is_err() {
                drawing.gesture = None;
            }
            let mut interaction_attempted = false;
            let (mut key, mut escape_held, mut focused) = context.input(|input| {
                let info = input.viewport();
                let key = HoverContext {
                    revision: engine.state().revision,
                    tool: initial_tool,
                    page: page.id(),
                    width,
                    height,
                    canvas_origin: available.min,
                    host_origin: info.inner_rect.or(info.outer_rect).map(|r| r.min),
                    pixels_per_point: input.pixels_per_point,
                    view_epoch: engine.view().epoch,
                };
                (key, input.key_down(egui::Key::Escape), input.focused)
            });
            if drawing.hover_context != Some(key) {
                drawing.hover_fresh = false;
                drawing.key_hotspot = None;
                drawing.navigation_position = None;
                drawing.hover_context = Some(key);
            }
            if drawing
                .gesture
                .as_ref()
                .is_some_and(|gesture| gesture.context != key)
            {
                drawing.gesture = None;
            }
            // InputState is final; recover eligibility before this frame's transitions.
            for event in events.iter().rev() {
                match event {
                    egui::Event::Key {
                        key: egui::Key::Escape,
                        pressed,
                        repeat,
                        ..
                    } => escape_held = !pressed || *repeat,
                    egui::Event::WindowFocused(value) => focused = !value,
                    _ => {}
                }
            }
            let text_focused = text_edit_focused(context);
            if blocked || escape_held || !focused || text_focused {
                drawing.key_hotspot = None;
                drawing.navigation_position = None;
            }
            let order_change = |page_id, fragment_id, bond_id, order, expected_revision| {
                Command::BondSetOrder(BondSetOrder {
                    expected_revision,
                    page_id,
                    fragment_id,
                    bond_id,
                    order,
                    hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
                })
            };
            // A native frame may contain the entire drag; press_origin is cleared on release.
            let mut tool_states = tool_states.into_iter().peekable();
            history_handled = true;
            for (index, event) in events.iter().enumerate() {
                if let Some(fields) = sdf_fields.as_deref_mut() {
                    fields.replay_editor(index, engine, context);
                }
                let blocked = blocked || pending_ui_blocks_authoring(&engine.state());
                while tool_states.peek().is_some_and(|(at, _)| *at == index) {
                    if let Some((_, tool)) = tool_states.next()
                        && key.tool != tool
                    {
                        key.tool = tool;
                        drawing.gesture = None;
                        drawing.invalidate_hover();
                        drawing.hover_context = Some(key);
                    }
                }
                match event {
                    egui::Event::Key {
                        key: egui::Key::Escape,
                        pressed,
                        ..
                    } => {
                        escape_held = *pressed;
                        if escape_held {
                            drawing.gesture = None;
                            drawing.hover_fresh = false;
                            drawing.key_hotspot = None;
                            drawing.navigation_position = None;
                        }
                        continue;
                    }
                    egui::Event::WindowFocused(value) => {
                        focused = *value;
                        if !focused {
                            drawing.gesture = None;
                            drawing.hover_fresh = false;
                            drawing.key_hotspot = None;
                            drawing.navigation_position = None;
                        }
                        continue;
                    }
                    egui::Event::PointerGone => {
                        drawing.gesture = None;
                        drawing.hover_fresh = false;
                        drawing.key_hotspot = None;
                        drawing.navigation_position = None;
                        continue;
                    }
                    _ => {}
                }
                // Resolve availability now: an earlier event may have populated Undo or Redo.
                let history_command = if !blocked && focused {
                    view_actions
                        .iter()
                        .find(|(at, _)| *at == index)
                        .and_then(|(_, action)| action.command(engine))
                        .or_else(|| {
                            selected
                                .filter(|(at, _)| *at == index)
                                .and_then(|(_, id)| control_command(engine, id))
                        })
                        .or_else(|| workspace_shortcut(engine, event, text_focused))
                } else {
                    None
                };
                if (blocked || escape_held || !focused) && history_command.is_none() {
                    continue;
                }
                let mut command = history_command.map(Ok);
                if command
                    .as_ref()
                    .is_some_and(|command| !matches!(command, Ok(Command::UiView(_))))
                {
                    drawing.gesture = None;
                }
                if command.is_none() {
                    if let egui::Event::PointerMoved(pos) | egui::Event::PointerButton { pos, .. } =
                        event
                    {
                        drawing.hover_fresh = true;
                        drawing.key_hotspot = (!text_focused).then_some(BondHotspot {
                            position: *pos,
                            target: None,
                        });
                        drawing.navigation_position = (!text_focused).then_some(*pos);
                    }
                    match event {
                        egui::Event::MouseWheel {
                            unit,
                            delta,
                            modifiers,
                        } if modifiers.is_none()
                            && !text_focused
                            && delta.is_finite()
                            && *delta != egui::Vec2::ZERO
                            && engine.view().zoom_percent.is_some() =>
                        {
                            let Some(pos) = drawing.navigation_position else {
                                continue;
                            };
                            if !response.interact_rect.contains(pos)
                                || !viewport.contains(local(pos))
                                || context.layer_id_at(pos) != Some(response.layer_id)
                            {
                                drawing.navigation_position = None;
                                continue;
                            }
                            let Some(page) =
                                document.pages().iter().find(|page| page.id() == key.page)
                            else {
                                continue;
                            };
                            command = view_controls::wheel(
                                engine,
                                page.size(),
                                [width, height],
                                *unit,
                                *delta,
                            )
                            .map(Ok);
                        }
                        egui::Event::Key {
                            key: pressed_key,
                            pressed: true,
                            repeat: false,
                            modifiers,
                            ..
                        } if modifiers.is_none()
                            && matches!(
                                pressed_key,
                                egui::Key::Num1 | egui::Key::Num2 | egui::Key::Num3
                            ) =>
                        {
                            let Some(mut hotspot) = drawing.key_hotspot else {
                                continue;
                            };
                            let pos = hotspot.position;
                            if text_focused
                                || drawing.gesture.is_some()
                                || !response.interact_rect.contains(pos)
                                || !viewport.contains(local(pos))
                                || context.layer_id_at(pos) != Some(response.layer_id)
                            {
                                drawing.key_hotspot = None;
                                continue;
                            }
                            let Ok(current_graph) = &graph else {
                                drawing.key_hotspot = None;
                                continue;
                            };
                            let target = hotspot
                                .target
                                .or_else(|| current_graph.hit_test(local(pos), 5.0).ok().flatten());
                            let Some(bondline_render::HitTarget::Bond {
                                fragment_id,
                                bond_id,
                            }) = target
                            else {
                                drawing.key_hotspot = None;
                                continue;
                            };
                            // Our order edit can move ink away from a stationary logical hotspot.
                            hotspot.target = target;
                            drawing.key_hotspot = Some(hotspot);
                            let order = match pressed_key {
                                egui::Key::Num1 => BondDrawOrder::Single,
                                egui::Key::Num2 => BondDrawOrder::Double,
                                _ => BondDrawOrder::Triple,
                            };
                            command = Some(Ok(order_change(
                                key.page.get(),
                                fragment_id.get(),
                                bond_id.get(),
                                order,
                                key.revision,
                            )));
                        }
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers,
                            ..
                        } => {
                            drawing.gesture = None;
                            if !response.interact_rect.contains(*pos)
                                || !viewport.contains(local(*pos))
                                || context.layer_id_at(*pos) != Some(response.layer_id)
                            {
                                continue;
                            }
                            let Ok(current_graph) = &graph else {
                                continue;
                            };
                            let target = match current_graph.hit_test(local(*pos), 5.0) {
                                Ok(target) => target,
                                Err(_) => continue,
                            };
                            if let Ok(begin) = viewport.document_position(local(*pos)) {
                                let begin = if let Some(bondline_render::HitTarget::Atom {
                                    fragment_id,
                                    atom_id,
                                }) = target
                                {
                                    let Some(anchor) = document
                                        .pages()
                                        .iter()
                                        .find(|page| page.id() == key.page)
                                        .and_then(|page| {
                                            page.fragments()
                                                .iter()
                                                .find(|fragment| fragment.id() == fragment_id)
                                        })
                                        .and_then(|fragment| {
                                            fragment.atoms().iter().find(|atom| atom.id == atom_id)
                                        })
                                        .and_then(|atom| atom.position)
                                    else {
                                        continue;
                                    };
                                    anchor
                                } else {
                                    begin
                                };
                                drawing.gesture = Some(BondGesture {
                                    begin,
                                    end: begin,
                                    press: *pos,
                                    modifiers: *modifiers,
                                    context: key,
                                    target,
                                });
                            }
                        }
                        egui::Event::PointerMoved(pos) => {
                            if let Some(gesture) = &mut drawing.gesture
                                && let Ok(end) = viewport.document_position(local(*pos))
                            {
                                gesture.end = end;
                            }
                        }
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers,
                            ..
                        } => {
                            let Some(gesture) = drawing.gesture.take() else {
                                continue;
                            };
                            if gesture.context != key
                                || !response.interact_rect.contains(*pos)
                                || !viewport.contains(local(*pos))
                                || context.layer_id_at(*pos) != Some(response.layer_id)
                            {
                                continue;
                            }
                            let Ok(current_graph) = &graph else {
                                continue;
                            };
                            let Ok(end) = viewport.document_position(local(*pos)) else {
                                continue;
                            };
                            let release_target = current_graph.hit_test(local(*pos), 5.0);
                            if gesture.press.distance(*pos) <= 3.0 {
                                if !gesture.modifiers.is_none() || !modifiers.is_none() {
                                    continue;
                                }
                                let Some(bondline_render::HitTarget::Bond {
                                    fragment_id,
                                    bond_id,
                                }) = gesture.target
                                else {
                                    continue;
                                };
                                if !matches!(release_target, Ok(target) if target == gesture.target)
                                {
                                    continue;
                                }
                                let Some(existing) = document
                                    .pages()
                                    .iter()
                                    .find(|page| page.id() == key.page)
                                    .and_then(|page| {
                                        page.fragments()
                                            .iter()
                                            .find(|fragment| fragment.id() == fragment_id)
                                    })
                                    .and_then(|fragment| {
                                        fragment.bonds().iter().find(|bond| bond.id == bond_id)
                                    })
                                    .map(|bond| bond.order)
                                else {
                                    continue;
                                };
                                command = Some(
                                    clicked_bond_order(existing, gesture.context.tool.order).map(
                                        |order| {
                                            order_change(
                                                key.page.get(),
                                                fragment_id.get(),
                                                bond_id.get(),
                                                order,
                                                key.revision,
                                            )
                                        },
                                    ),
                                );
                            } else {
                                command = Some(match release_target {
                                    Ok(None) => match gesture.target {
                                        None => Ok(Command::BondCreate(BondCreate {
                                            order: gesture.context.tool.order,
                                            page_id: gesture.context.page.get(),
                                            begin_x_pt: gesture.begin.x_pt(),
                                            begin_y_pt: gesture.begin.y_pt(),
                                            end_x_pt: end.x_pt(),
                                            end_y_pt: end.y_pt(),
                                        })),
                                        Some(bondline_render::HitTarget::Atom {
                                            fragment_id,
                                            atom_id,
                                        }) => Ok(Command::BondExtend(BondExtend {
                                            order: gesture.context.tool.order,
                                            expected_revision: gesture.context.revision,
                                            page_id: gesture.context.page.get(),
                                            fragment_id: fragment_id.get(),
                                            begin_atom_id: atom_id.get(),
                                            end_x_pt: end.x_pt(),
                                            end_y_pt: end.y_pt(),
                                            hydrogen_policy:
                                                ExtendHydrogenPolicy::RectifyTerminalCarbon,
                                        })),
                                        Some(bondline_render::HitTarget::Bond { .. }) => {
                                            Err(bondline_commands::Error::UnsupportedBondExtension)
                                        }
                                    },
                                    Ok(Some(bondline_render::HitTarget::Atom {
                                        fragment_id,
                                        atom_id: end_id,
                                    })) if gesture.context.tool.order == BondDrawOrder::Single
                                        && gesture.modifiers.is_none()
                                        && modifiers.is_none() =>
                                    {
                                        let existing = match gesture.target {
                                            Some(bondline_render::HitTarget::Atom {
                                                fragment_id: start_fragment,
                                                atom_id: begin_id,
                                            }) if start_fragment == fragment_id => document
                                                .pages()
                                                .iter()
                                                .find(|page| page.id() == key.page)
                                                .and_then(|page| {
                                                    page.fragments().iter().find(|fragment| {
                                                        fragment.id() == fragment_id
                                                    })
                                                })
                                                .and_then(|fragment| {
                                                    fragment.bonds().iter().find_map(|bond| {
                                                        if !((bond.begin == begin_id
                                                            && bond.end == end_id)
                                                            || (bond.begin == end_id
                                                                && bond.end == begin_id))
                                                        {
                                                            return None;
                                                        }
                                                        let order = match bond.order {
                                                            bondline_model::BondOrder::Single => {
                                                                BondDrawOrder::Double
                                                            }
                                                            bondline_model::BondOrder::Double => {
                                                                BondDrawOrder::Triple
                                                            }
                                                            _ => return None,
                                                        };
                                                        Some((bond.id, order))
                                                    })
                                                }),
                                            _ => None,
                                        };
                                        match existing {
                                            Some((bond_id, order)) => Ok(order_change(
                                                key.page.get(),
                                                fragment_id.get(),
                                                bond_id.get(),
                                                order,
                                                key.revision,
                                            )),
                                            None => Err(bondline_commands::Error::OccupiedEndpoint),
                                        }
                                    }
                                    Ok(Some(_)) => Err(bondline_commands::Error::OccupiedEndpoint),
                                    Err(error) => Err(error.into()),
                                });
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(command) = command {
                    let explicit = selected.filter(|(at, _)| *at == index).map(|(_, id)| id);
                    if let Ok(command) = &command
                        && sdf_fields.as_deref_mut().is_some_and(|fields| {
                            fields.intercept_editor_command(command, engine, context, explicit)
                        })
                    {
                        continue;
                    }
                    let view_command = matches!(&command, Ok(Command::UiView(_)));
                    let previous_view = engine.view();
                    let key_hotspot = command
                        .as_ref()
                        .ok()
                        .filter(|command| matches!(command, Command::BondSetOrder(_)))
                        .and(drawing.key_hotspot);
                    *error = command
                        .and_then(|command| engine.execute(command))
                        .err()
                        .map(|failure| failure.to_string());
                    if view_command && engine.view() == previous_view {
                        continue;
                    }
                    interaction_attempted = true;
                    let navigation_position = view_command
                        .then_some(drawing.navigation_position)
                        .flatten();
                    if view_command {
                        drawing.gesture = None;
                    }
                    drawing.invalidate_hover();
                    drawing.navigation_position = navigation_position;
                    // Later same-frame presses pick the graph that actually committed.
                    let Ok(stored) = engine.stored_document() else {
                        drawing.gesture = None;
                        drawing.invalidate_hover();
                        return;
                    };
                    document = stored.model_snapshot();
                    let Some(page) = document.pages().first() else {
                        drawing.gesture = None;
                        return;
                    };
                    viewport = match engine.page_view(width, height, page.id()) {
                        Ok(viewport) => viewport,
                        Err(failure) => {
                            drawing.gesture = None;
                            *error = Some(failure.to_string());
                            return;
                        }
                    };
                    key.page = page.id();
                    key.revision = engine.state().revision;
                    key.view_epoch = engine.view().epoch;
                    drawing.hover_context = Some(key);
                    graph = engine.drawing_view_scene(
                        width,
                        height,
                        key.page,
                        &bondline_render::Cancellation::default(),
                    );
                    if graph.is_ok() {
                        drawing.key_hotspot = key_hotspot;
                    }
                    context.request_repaint();
                }
            }
            let pointer = context.input(|input| {
                if !input.pointer.primary_down() {
                    drawing.gesture = None;
                }
                if blocked || !input.focused || input.key_down(egui::Key::Escape) {
                    drawing.gesture = None;
                    drawing.hover_fresh = false;
                    drawing.key_hotspot = None;
                    drawing.navigation_position = None;
                }
                if drawing.gesture.is_some() || interaction_attempted {
                    drawing.hover_fresh = false;
                }
                // Cached hover_pos alone must never revive invalidated hover.
                if drawing.hover_fresh {
                    input
                        .pointer
                        .hover_pos()
                        .filter(|p| available.contains(*p) && response.contains_pointer())
                } else {
                    None
                }
            });
            match graph {
                Ok(graph) => {
                    let Some(page) = document.pages().iter().find(|page| page.id() == key.page)
                    else {
                        drawing.invalidate_hover();
                        return;
                    };
                    if let Some(pointer) = pointer {
                        drawing.hover = graph.hit_test(local(pointer), 5.0).ok().flatten();
                    }
                    let scene = graph.scene();
                    let painter = ui.painter_at(available);
                    let [r, g, b, a] = scene.background;
                    painter.rect_filled(
                        available,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(r, g, b, a),
                    );
                    for primitive in &scene.primitives {
                        match primitive {
                            bondline_render::Primitive::Rectangle(rectangle) => {
                                let [r, g, b, a] = rectangle.color;
                                painter.rect_filled(
                                    egui::Rect::from_min_size(
                                        available.min
                                            + egui::vec2(rectangle.x as f32, rectangle.y as f32),
                                        egui::vec2(rectangle.width as f32, rectangle.height as f32),
                                    ),
                                    0.0,
                                    egui::Color32::from_rgba_unmultiplied(r, g, b, a),
                                );
                            }
                            bondline_render::Primitive::Stroke(stroke) => {
                                if let Err(error) = paint_stroke(&painter, available.min, stroke) {
                                    ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                                }
                            }
                        }
                    }
                    if let Some(target) = drawing.hover
                        && let Err(error) =
                            paint_hover(&painter, available.min, page, &viewport, &graph, target)
                    {
                        drawing.invalidate_hover();
                        ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                    }
                    if let Some(gesture) = &drawing.gesture
                        && gesture.begin != gesture.end
                        && let (Ok(begin), Ok(end)) = (
                            viewport.canvas_position(gesture.begin),
                            viewport.canvas_position(gesture.end),
                        )
                    {
                        let order = match gesture.context.tool.order {
                            BondDrawOrder::Single => bondline_model::BondOrder::Single,
                            BondDrawOrder::Double => bondline_model::BondOrder::Double,
                            BondDrawOrder::Triple => bondline_model::BondOrder::Triple,
                        };
                        let result = bondline_render::centered_bond_strokes(
                            &bondline_render::Stroke {
                                begin,
                                end,
                                width: viewport.stroke_width(),
                                color: [70, 105, 155, 255],
                            },
                            order,
                        )
                        .and_then(|strokes| {
                            for stroke in &strokes {
                                paint_stroke(&painter, available.min, stroke)?;
                            }
                            Ok(())
                        });
                        if let Err(error) = result {
                            ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                        }
                    }
                }
                Err(error) => {
                    drawing.invalidate_hover();
                    ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                }
            }
            // The inspector was painted before input; show this frame's diagnostic now.
            if interaction_attempted && let Some(message) = error {
                ui.colored_label(egui::Color32::DARK_RED, message.as_str());
            }
        });
    if !history_handled {
        if let Some((index, id)) = selected {
            view_actions.push((index, view_controls::Action::Control(id)));
        }
        replay_edit_shortcuts(
            context,
            engine,
            error,
            drawing,
            &events,
            blocked,
            ShortcutRouting {
                actions: &view_actions,
                fields: sdf_fields.as_deref_mut(),
                canvas: None,
            },
        );
    }
    if let Some(files) = files {
        if let Some(fields) = sdf_fields.as_deref_mut() {
            fields.settle_editor(engine, context);
            if fields.holds_files() {
                return None;
            }
        }
        file_action = files
            .dialogs(
                context,
                engine,
                error,
                active_work
                    || preview.as_ref().is_some_and(|preview| preview.is_busy())
                    || sdf_fields.as_ref().is_some_and(|fields| fields.is_busy())
                    || smiles_records
                        .as_ref()
                        .is_some_and(|records| records.is_busy()),
            )
            .or(file_action);
    }
    file_action
}

/// Native application state for startup and bounded smoke verification.
pub struct BondlineApp {
    smoke_frames: Option<u8>,
    engine: SharedEngine,
    executor: CommandExecutor,
    control: Option<LoopbackServer>,
    error: Option<String>,
    drawing: DrawingState,
    files: FileWorkflow,
    preview: PreviewWorkflow,
    sdf_fields: SdfFieldsWorkflow,
    smiles_records: SmilesRecordsWorkflow,
}

impl BondlineApp {
    /// Initialize the real wgpu context and font; smoke mode closes after three painted frames.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        smoke: bool,
        control_port: Option<u16>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let state = cc
            .wgpu_render_state
            .as_ref()
            .ok_or_else(|| std::io::Error::other("wgpu renderer did not initialize"))?;
        let info = state.adapter.get_info();
        let mut document_engine = Engine::default();
        document_engine.set_ui_file_host(true);
        let engine = Arc::new(Mutex::new(document_engine));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let control = control_port
            .map(|port| {
                let context = cc.egui_ctx.clone();
                LoopbackServer::bind_with_executor(port, executor.clone(), move || {
                    context.request_repaint()
                })
            })
            .transpose()?;
        if smoke || control.is_some() {
            let mut output = std::io::stdout().lock();
            serde_json::to_writer(
                &mut output,
                &serde_json::json!({"app":"bondline", "backend":format!("{:?}", info.backend), "adapter":info.name, "device_type":format!("{:?}", info.device_type), "control_address":control.as_ref().map(|server| server.address().to_string()), "control_token":control.as_ref().map(LoopbackServer::token)}),
            )?;
            output.write_all(b"\n")?;
        }
        configure_context(&cc.egui_ctx);
        Ok(Self {
            smoke_frames: smoke.then_some(0),
            engine,
            executor,
            control,
            error: None,
            drawing: DrawingState::default(),
            files: FileWorkflow::new(std::env::home_dir().map_or_else(std::env::current_dir, Ok)?),
            preview: PreviewWorkflow::default(),
            sdf_fields: SdfFieldsWorkflow::default(),
            smiles_records: SmilesRecordsWorkflow::default(),
        })
    }
}

impl BondlineApp {
    fn update_workspace(&mut self, context: &egui::Context) {
        if let Some(control) = &self.control {
            egui::TopBottomPanel::bottom("local-control").show(context, |ui| {
                if control.is_running() {
                    ui.label(format!("Local control: {}", control.address()));
                } else {
                    ui.colored_label(egui::Color32::DARK_RED, "Local control stopped");
                }
            });
        }
        let native_close = context.input(|input| input.viewport().close_requested());
        let smoke_close = if let Some(frames) = &mut self.smoke_frames {
            *frames = frames.saturating_add(1);
            if *frames < 3 {
                context.request_repaint();
            }
            *frames == 3
        } else {
            false
        };
        let mut action = None;
        match self.engine.lock() {
            Ok(mut engine) => {
                self.files.poll(&mut engine, &mut self.error);
                self.preview.sync(&engine, context);
                self.sdf_fields.sync(&engine, context);
                self.smiles_records.sync(&engine, context);
                self.sdf_fields.settle_editor(&mut engine, context);
                if native_close && engine.state().close != CloseState::Committed {
                    context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    if !self.sdf_fields.native_close(&engine, context) {
                        self.files.request_close(&mut engine, &mut self.error);
                    }
                }
                if smoke_close && !self.sdf_fields.native_close(&engine, context) {
                    self.files.request_close(&mut engine, &mut self.error);
                }
                action = show_workspace_with_preview(
                    context,
                    &mut engine,
                    &mut self.error,
                    &mut self.drawing,
                    Some(&mut self.files),
                    self.executor.has_active_work()
                        || self.preview.is_busy()
                        || self.sdf_fields.is_busy()
                        || self.smiles_records.is_busy(),
                    Some(WorkspaceWorkflows {
                        preview: &mut self.preview,
                        fields: &mut self.sdf_fields,
                        smiles: &mut self.smiles_records,
                    }),
                );
                self.preview.sync(&engine, context);
                self.sdf_fields.sync(&engine, context);
                self.smiles_records.sync(&engine, context);
                self.sdf_fields.settle_editor(&mut engine, context);
                if self.files.complete_close_if_idle(
                    &mut engine,
                    self.executor.has_active_work()
                        || self.preview.is_busy()
                        || self.sdf_fields.blocks_close()
                        || self.smiles_records.is_busy(),
                    action.is_some(),
                ) {
                    context.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            Err(_) => {
                self.preview.deactivate(context);
                self.sdf_fields.deactivate(context);
                self.smiles_records.deactivate(context);
                self.sdf_fields.show_editor(
                    context,
                    None,
                    &context.input(|input| input.events.clone()),
                );
                if native_close {
                    context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
                egui::CentralPanel::default().show(context, |ui| {
                    ui.colored_label(egui::Color32::DARK_RED, "Document engine unavailable");
                });
            }
        }
        if let Some(action) = action {
            self.error = self.files.start(action, &self.executor, context).err();
            if self.error.is_some()
                && let Ok(mut engine) = self.engine.lock()
            {
                self.files
                    .keep_open_after_file_failure(&mut engine, &mut self.error);
            }
        }
        self.preview.start_pending(&self.executor, context);
        self.sdf_fields.start_pending(&self.executor, context);
        self.smiles_records.start_pending(&self.executor, context);
    }
}

impl eframe::App for BondlineApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_workspace(context);
    }
}

impl Drop for BondlineApp {
    fn drop(&mut self) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.set_ui_file_host(false);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn metadata_shortcuts_prioritize_redo_and_do_not_fall_back_to_undo_when_redo_is_disabled()
    -> Result<(), Box<dyn std::error::Error>> {
        use eframe::egui;
        let context = egui::Context::default();
        super::configure_context(&context);
        let mut engine = bondline_commands::Engine::default();
        for width in [640, 641] {
            engine.dispatch(
                "page.set_size",
                serde_json::json!({"page_id":1,"width_pt":width,"height_pt":480}),
            )?;
        }
        engine.dispatch("history.undo", serde_json::json!({}))?;
        for expected_revision in [4, 4] {
            context.begin_pass(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 700.0),
                )),
                events: vec![egui::Event::Key {
                    key: egui::Key::Z,
                    physical_key: Some(egui::Key::Z),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                }],
                ..Default::default()
            });
            super::show_workspace(
                &context,
                &mut engine,
                &mut None,
                &mut super::DrawingState::default(),
            );
            let _ = context.end_pass();
            assert_eq!(engine.state().revision, expected_revision);
            assert_eq!(
                engine
                    .document()?
                    .pages()
                    .first()
                    .ok_or("page")?
                    .size()
                    .width_pt(),
                641.0
            );
        }
        Ok(())
    }

    #[test]
    fn delivered_page_controls_retain_independent_physical_point_expectations()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = bondline_commands::Engine::default();
        for (id, width, height) in [
            ("page.letter", 612.0, 792.0),
            ("page.rotate", 792.0, 612.0),
            ("page.a4", 595.2755905511812, 841.8897637795277),
        ] {
            let control = engine
                .controls()
                .into_iter()
                .find(|row| row.id == id)
                .ok_or("page control")?;
            assert!(control.enabled);
            engine.execute(control.command)?;
            let page = engine.document()?.pages().first().ok_or("page")?;
            assert!((page.size().width_pt() - width).abs() < 1e-10);
            assert!((page.size().height_pt() - height).abs() < 1e-10);
        }
        Ok(())
    }

    #[test]
    fn native_app_requires_wgpu() -> Result<(), Box<dyn std::error::Error>> {
        assert!(matches!(
            super::native_options()?.renderer,
            eframe::Renderer::Wgpu
        ));
        Ok(())
    }

    #[test]
    fn workspace_paints_a_white_page_using_the_bundled_font() {
        let context = eframe::egui::Context::default();
        super::configure_context(&context);
        context.begin_pass(eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                eframe::egui::vec2(1000.0, 700.0),
            )),
            ..Default::default()
        });
        super::show_workspace(
            &context,
            &mut bondline_commands::Engine::default(),
            &mut None,
            &mut super::DrawingState::default(),
        );
        let frame = context.end_pass();
        assert!(frame.shapes.iter().any(|shape| matches!(&shape.shape, eframe::egui::Shape::Rect(rect) if rect.fill == eframe::egui::Color32::WHITE)));
    }

    #[test]
    fn native_shortcuts_use_shared_history_and_page_dimensions()
    -> Result<(), Box<dyn std::error::Error>> {
        use eframe::egui::{self, Event, Key, Modifiers};
        let context = egui::Context::default();
        super::configure_context(&context);
        let mut engine = bondline_commands::Engine::default();
        engine.dispatch(
            "page.set_size",
            serde_json::json!({"page_id":1,"width_pt":612,"height_pt":792}),
        )?;
        let mut error = None;
        for redo in [false, true] {
            let modifiers = Modifiers {
                shift: redo,
                ..Modifiers::COMMAND
            };
            context.begin_pass(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 700.0),
                )),
                modifiers,
                events: vec![
                    Event::Key {
                        key: Key::Z,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers,
                    },
                    Event::Key {
                        key: Key::Z,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers,
                    },
                ],
                ..Default::default()
            });
            super::show_workspace(
                &context,
                &mut engine,
                &mut error,
                &mut super::DrawingState::default(),
            );
            let frame = context.end_pass();
            let page = engine.document()?.pages().first().ok_or("missing page")?;
            if redo {
                assert_eq!(page.size().width_pt(), 612.0);
            } else {
                assert!((page.size().width_pt() - 595.2755905511812).abs() < 1e-10);
            }
            let rect = frame
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Rect(rect) if rect.fill == egui::Color32::WHITE => Some(rect.rect),
                    _ => None,
                })
                .ok_or("page was not painted")?;
            let border = frame
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_gray(196) => {
                        Some(rect.rect)
                    }
                    _ => None,
                })
                .ok_or("shared scene border was not painted")?;
            assert_eq!(rect.min, border.min + egui::vec2(1.0, 1.0));
            assert_eq!(rect.size(), border.size() - egui::vec2(2.0, 2.0));
            let ratio = page.size().width_pt() / page.size().height_pt();
            // Each outer dimension rounds to a pixel: |w - ratio*h| <= (1 + ratio)/2.
            assert!(
                (f64::from(border.width()) - ratio * f64::from(border.height())).abs()
                    <= (1.0 + ratio) / 2.0
            );
            assert!(error.is_none());
        }
        Ok(())
    }
}
