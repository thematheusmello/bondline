//! Native GPU workspace. Authoring controls dispatch through the shared command registry.

use bondline_commands::{Command, Engine, NoParams, PageSetSize};
use bondline_mcp::{SharedEngine, control::LoopbackServer};
use eframe::egui;
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

/// Paint document state and route every authored edit through the engine.
pub fn show_workspace(context: &egui::Context, engine: &mut Engine, error: &mut Option<String>) {
    let undo = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
    let redo = egui::KeyboardShortcut::new(
        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        egui::Key::Z,
    );
    let state = engine.state();
    let mut selected = context.input_mut(|input| {
        if input.consume_shortcut(&redo) {
            (state.redo_depth > 0).then_some(Command::Redo(NoParams {}))
        } else if input.consume_shortcut(&undo) {
            (state.undo_depth > 0).then_some(Command::Undo(NoParams {}))
        } else {
            None
        }
    });
    let page = engine
        .document()
        .pages()
        .first()
        .map(|page| (page.id().get(), page.size()));
    egui::TopBottomPanel::top("title").show(context, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("bondline").strong().size(22.0));
            ui.separator();
            ui.label("Untitled");
            ui.separator();
            ui.menu_button("Edit", |ui| {
                let modifier = if cfg!(target_os = "macos") {
                    "Cmd"
                } else {
                    "Ctrl"
                };
                if ui
                    .add_enabled(
                        state.undo_depth > 0,
                        egui::Button::new("Undo").shortcut_text(format!("{modifier}+Z")),
                    )
                    .clicked()
                {
                    selected = Some(Command::Undo(NoParams {}));
                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        state.redo_depth > 0,
                        egui::Button::new("Redo").shortcut_text(format!("{modifier}+Shift+Z")),
                    )
                    .clicked()
                {
                    selected = Some(Command::Redo(NoParams {}));
                    ui.close_menu();
                }
            });
            ui.menu_button("Page", |ui| {
                if let Some((page_id, size)) = page {
                    for (label, width_pt, height_pt) in [
                        ("A4 portrait", 210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4),
                        ("Letter portrait", 612.0, 792.0),
                        ("Rotate page", size.height_pt(), size.width_pt()),
                    ] {
                        if ui.button(label).clicked() {
                            selected = Some(Command::PageSetSize(PageSetSize {
                                page_id,
                                width_pt,
                                height_pt,
                            }));
                            ui.close_menu();
                        }
                    }
                }
            });
        });
    });
    if let Some(command) = selected {
        *error = engine.execute(command).err().map(|error| error.to_string());
    }
    egui::TopBottomPanel::bottom("status").show(context, |ui| {
        ui.horizontal(|ui| {
            if let Some(page) = engine.document().pages().first() {
                ui.label(format!("Page {}", page.id().get()));
                ui.separator();
                ui.label(format!(
                    "{:.1} x {:.1} pt",
                    page.size().width_pt(),
                    page.size().height_pt()
                ));
            }
            if let Some(message) = error {
                ui.colored_label(egui::Color32::DARK_RED, message.as_str());
            }
        });
    });
    egui::CentralPanel::default()
        .frame(egui::Frame::default().inner_margin(0.0))
        .show(context, |ui| {
            let available = ui.available_rect_before_wrap();
            let Some(page) = engine.document().pages().first() else {
                return;
            };
            match bondline_render::page_scene(
                available.width().floor().max(1.0) as u32,
                available.height().floor().max(1.0) as u32,
                page.size(),
            ) {
                Ok(scene) => {
                    let painter = ui.painter_at(available);
                    let [r, g, b, a] = scene.background;
                    painter.rect_filled(
                        available,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(r, g, b, a),
                    );
                    for rectangle in scene.rectangles {
                        let [r, g, b, a] = rectangle.color;
                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                available.min + egui::vec2(rectangle.x as f32, rectangle.y as f32),
                                egui::vec2(rectangle.width as f32, rectangle.height as f32),
                            ),
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(r, g, b, a),
                        );
                    }
                }
                Err(error) => {
                    ui.colored_label(egui::Color32::DARK_RED, error.to_string());
                }
            }
        });
}

/// Native application state for startup and bounded smoke verification.
pub struct BondlineApp {
    smoke_frames: Option<u8>,
    engine: SharedEngine,
    control: Option<LoopbackServer>,
    error: Option<String>,
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
        let engine = Arc::new(Mutex::new(Engine::default()));
        let control = control_port
            .map(|port| {
                let context = cc.egui_ctx.clone();
                LoopbackServer::bind(port, Arc::clone(&engine), move || context.request_repaint())
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
            control,
            error: None,
        })
    }
}

impl eframe::App for BondlineApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(control) = &self.control {
            egui::TopBottomPanel::bottom("local-control").show(context, |ui| {
                if control.is_running() {
                    ui.label(format!("Local control: {}", control.address()));
                } else {
                    ui.colored_label(egui::Color32::DARK_RED, "Local control stopped");
                }
            });
        }
        match self.engine.lock() {
            Ok(mut engine) => show_workspace(context, &mut engine, &mut self.error),
            Err(_) => {
                egui::CentralPanel::default().show(context, |ui| {
                    ui.colored_label(egui::Color32::DARK_RED, "Document engine unavailable");
                });
            }
        }
        if let Some(frames) = &mut self.smoke_frames {
            *frames = frames.saturating_add(1);
            if *frames >= 3 {
                context.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                context.request_repaint();
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
            super::show_workspace(&context, &mut engine, &mut error);
            let frame = context.end_pass();
            let page = engine.document().pages().first().ok_or("missing page")?;
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
