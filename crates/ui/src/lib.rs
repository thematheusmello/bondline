//! Native GPU workspace. Authoring controls dispatch through the shared command registry.

mod file_workflow;
pub mod files;
#[cfg(test)]
mod mol_display_tests;
mod preview;

use bondline_commands::{
    BondCreate, BondDrawOrder, BondExtend, CloseState, Command, ControlInfo, ControlKey,
    ControlShortcut, Engine, ExtendHydrogenPolicy,
};
use bondline_mcp::{CommandExecutor, SharedEngine, control::LoopbackServer};
use bondline_model::{PageId, Point2};
use eframe::egui;
use file_workflow::{FileIntentAction, FileWorkflow};
use preview::PreviewWorkflow;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

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
    context.set_visuals(egui::Visuals::light());
}

#[derive(Default)]
pub struct DrawingState {
    gesture: Option<BondGesture>,
    hover: Option<bondline_render::HitTarget>,
    hover_context: Option<HoverContext>,
    hover_fresh: bool,
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
    }
}

#[derive(Clone, Copy, PartialEq)]
struct HoverContext {
    revision: u64,
    page: PageId,
    width: u32,
    height: u32,
    canvas_origin: egui::Pos2,
    host_origin: Option<egui::Pos2>,
    pixels_per_point: f32,
}

struct BondGesture {
    begin: Point2,
    end: Point2,
    press: egui::Pos2,
    context: HoverContext,
    target: Option<bondline_render::HitTarget>,
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

fn paint_hover(
    painter: &egui::Painter,
    origin: egui::Pos2,
    page: &bondline_model::Page,
    viewport: &bondline_render::PageViewport,
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
            let bond = fragment
                .bonds()
                .iter()
                .find(|b| b.id == bond_id)
                .ok_or(Error::UnsupportedChemicalContent)?;
            let outline = bondline_render::Stroke {
                begin: position(bond.begin)?,
                end: position(bond.end)?,
                width: viewport.stroke_width() + 4.0,
                color: [0; 4],
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
    };
    egui::KeyboardShortcut::new(modifiers, key)
}

fn control_shortcut(
    context: &egui::Context,
    controls: &[ControlInfo],
    menu: &str,
) -> Option<Command> {
    // Pinned egui ignores extra Shift: consume specific shortcuts even when disabled.
    for shifted in [true, false] {
        for control in controls.iter().filter(|control| control.menu == menu) {
            if let Some(shortcut) = control
                .shortcut
                .filter(|shortcut| shortcut.shift == shifted)
                && context
                    .input_mut(|input| input.consume_shortcut(&control_shortcut_key(shortcut)))
            {
                return control.enabled.then(|| control.command.clone());
            }
        }
    }
    None
}

fn control_menu(ui: &mut egui::Ui, controls: &[ControlInfo], menu: &str) -> Option<Command> {
    let mut selected = None;
    ui.menu_button(menu, |ui| {
        for control in controls.iter().filter(|control| control.menu == menu) {
            let mut button = egui::Button::new(control.label);
            if let Some(shortcut) = control.shortcut {
                button =
                    button.shortcut_text(ui.ctx().format_shortcut(&control_shortcut_key(shortcut)));
            }
            if ui.add_enabled(control.enabled, button).clicked() {
                selected = Some(control.command.clone());
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

fn show_workspace_with_preview(
    context: &egui::Context,
    engine: &mut Engine,
    error: &mut Option<String>,
    drawing: &mut DrawingState,
    mut files: Option<&mut FileWorkflow>,
    active_work: bool,
    mut preview: Option<&mut PreviewWorkflow>,
) -> Option<FileIntentAction> {
    let state = engine.state();
    let mut selected = None;
    let mut file_action = None;
    egui::TopBottomPanel::top("title").show(context, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("bondline").strong().size(22.0));
            ui.separator();
            if let Some(files) = files.as_deref_mut() {
                file_action = files.menu(ui, engine, error);
            }
            let current = engine.state();
            let blocked = current.close != CloseState::Open
                || files
                    .as_ref()
                    .is_some_and(|files| files.blocks_authoring(&current));
            ui.add_enabled_ui(!blocked, |ui| {
                let controls = engine.controls();
                let edit = control_menu(ui, &controls, "Edit");
                let page = control_menu(ui, &controls, "Page");
                selected = edit.or(page);
            });
            ui.separator();
            let name = state
                .file
                .path
                .as_deref()
                .and_then(|path| std::path::Path::new(path).file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("Untitled");
            ui.label(format!(
                "{name}{}",
                if state.file.dirty { " *" } else { "" }
            ));
        });
    });
    let current = engine.state();
    let blocked = current.close != CloseState::Open
        || files
            .as_ref()
            .is_some_and(|files| files.blocks_authoring(&current));
    if blocked {
        drawing.gesture = None;
    } else {
        selected = selected.or_else(|| control_shortcut(context, &engine.controls(), "Edit"));
    }
    if let Some(command) = selected {
        *error = engine.execute(command).err().map(|error| error.to_string());
    }
    if drawing
        .gesture
        .as_ref()
        .is_some_and(|gesture| gesture.context.revision != engine.state().revision)
    {
        drawing.gesture = None;
    }
    egui::TopBottomPanel::bottom("status").show(context, |ui| {
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
            if engine.document().is_ok() {
                ui.label("Drag empty paper for C-C; extend terminal CH3 to CH2; Esc cancels");
            }
            if let Some(message) = error {
                ui.colored_label(egui::Color32::DARK_RED, message.as_str());
            }
            if let Some(files) = files.as_deref_mut() {
                files.job_status(ui);
            }
        });
    });
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
                return;
            }
            if let Some(collection) = engine.content().collection() {
                drawing.gesture = None;
                drawing.invalidate_hover();
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
                                        ui.end_row();
                                    }
                                });
                            });
                    });
                    let rect = ui.available_rect_before_wrap();
                    preview.show(ui, engine, rect);
                    return;
                }
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
            let viewport = match bondline_render::PageViewport::new(width, height, page.size()) {
                Ok(viewport) => viewport,
                Err(error) => {
                    drawing.gesture = None;
                    drawing.invalidate_hover();
                    ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                    return;
                }
            };
            let response = ui.interact(
                available,
                ui.make_persistent_id("drawing-canvas"),
                egui::Sense::drag(),
            );
            let local = |position: egui::Pos2| {
                let position = position - available.min;
                [position.x, position.y]
            };
            let mut graph = engine.drawing_page_scene(
                width,
                height,
                page.id(),
                &bondline_render::Cancellation::default(),
            );
            if graph.is_err() {
                drawing.gesture = None;
            }
            let mut interaction_attempted = false;
            let (mut key, events, mut escape_held, mut focused) = context.input(|input| {
                let info = input.viewport();
                let key = HoverContext {
                    revision: engine.state().revision,
                    page: page.id(),
                    width,
                    height,
                    canvas_origin: available.min,
                    host_origin: info.inner_rect.or(info.outer_rect).map(|r| r.min),
                    pixels_per_point: input.pixels_per_point,
                };
                (
                    key,
                    input.events.clone(),
                    input.key_down(egui::Key::Escape),
                    input.focused,
                )
            });
            if drawing.hover_context != Some(key) {
                drawing.hover_fresh = false;
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
            // A native frame may contain the entire drag; press_origin is cleared on release.
            for event in &events {
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
                        }
                        continue;
                    }
                    egui::Event::WindowFocused(value) => {
                        focused = *value;
                        if !focused {
                            drawing.gesture = None;
                            drawing.hover_fresh = false;
                        }
                        continue;
                    }
                    egui::Event::PointerGone => {
                        drawing.gesture = None;
                        drawing.hover_fresh = false;
                        continue;
                    }
                    _ => {}
                }
                if blocked || escape_held || !focused {
                    continue;
                }
                if matches!(
                    event,
                    egui::Event::PointerMoved(_) | egui::Event::PointerButton { .. }
                ) {
                    drawing.hover_fresh = true;
                }
                match event {
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
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
                        ..
                    } => {
                        let Some(gesture) = drawing.gesture.take() else {
                            continue;
                        };
                        if gesture.context != key
                            || !response.interact_rect.contains(*pos)
                            || !viewport.contains(local(*pos))
                            || context.layer_id_at(*pos) != Some(response.layer_id)
                            || gesture.press.distance(*pos) <= 3.0
                        {
                            continue;
                        }
                        let Ok(current_graph) = &graph else {
                            continue;
                        };
                        let Ok(end) = viewport.document_position(local(*pos)) else {
                            continue;
                        };
                        interaction_attempted = true;
                        let command = match current_graph.hit_test(local(*pos), 5.0) {
                            Ok(None) => match gesture.target {
                                None => Ok(Command::BondCreate(BondCreate {
                                    order: BondDrawOrder::Single,
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
                                    order: BondDrawOrder::Single,
                                    expected_revision: gesture.context.revision,
                                    page_id: gesture.context.page.get(),
                                    fragment_id: fragment_id.get(),
                                    begin_atom_id: atom_id.get(),
                                    end_x_pt: end.x_pt(),
                                    end_y_pt: end.y_pt(),
                                    hydrogen_policy: ExtendHydrogenPolicy::RectifyTerminalCarbon,
                                })),
                                Some(bondline_render::HitTarget::Bond { .. }) => {
                                    Err(bondline_commands::Error::UnsupportedBondExtension)
                                }
                            },
                            Ok(Some(_)) => Err(bondline_commands::Error::OccupiedEndpoint),
                            Err(error) => Err(error.into()),
                        };
                        drawing.invalidate_hover();
                        *error = command
                            .and_then(|command| engine.execute(command))
                            .err()
                            .map(|error| error.to_string());
                        // Later same-frame presses pick the graph that actually committed.
                        let Ok(stored) = engine.stored_document() else {
                            drawing.gesture = None;
                            drawing.invalidate_hover();
                            return;
                        };
                        document = stored.model_snapshot();
                        key.revision = engine.state().revision;
                        drawing.hover_context = Some(key);
                        graph = engine.drawing_page_scene(
                            width,
                            height,
                            key.page,
                            &bondline_render::Cancellation::default(),
                        );
                        context.request_repaint();
                    }
                    _ => {}
                }
            }
            let pointer = context.input(|input| {
                if !input.pointer.primary_down() {
                    drawing.gesture = None;
                }
                if blocked || !input.focused || input.key_down(egui::Key::Escape) {
                    drawing.gesture = None;
                    drawing.hover_fresh = false;
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
                            paint_hover(&painter, available.min, page, &viewport, target)
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
                        && let Err(error) = paint_stroke(
                            &painter,
                            available.min,
                            &bondline_render::Stroke {
                                begin,
                                end,
                                width: viewport.stroke_width(),
                                color: [70, 105, 155, 255],
                            },
                        )
                    {
                        ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                    }
                }
                Err(error) => {
                    drawing.invalidate_hover();
                    ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                }
            }
        });
    if let Some(files) = files {
        file_action = files
            .dialogs(
                context,
                engine,
                error,
                active_work || preview.as_ref().is_some_and(|preview| preview.is_busy()),
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
                if native_close && engine.state().close != CloseState::Committed {
                    context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    self.files.request_close(&mut engine, &mut self.error);
                }
                if smoke_close {
                    self.files.request_close(&mut engine, &mut self.error);
                }
                action = show_workspace_with_preview(
                    context,
                    &mut engine,
                    &mut self.error,
                    &mut self.drawing,
                    Some(&mut self.files),
                    self.executor.has_active_work() || self.preview.is_busy(),
                    Some(&mut self.preview),
                );
                self.preview.sync(&engine, context);
                if self.files.complete_close_if_idle(
                    &mut engine,
                    self.executor.has_active_work() || self.preview.is_busy(),
                    action.is_some(),
                ) {
                    context.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            Err(_) => {
                self.preview.deactivate(context);
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
