//! Native GPU workspace. Chemical editing is added through the command engine in later slices.

use eframe::egui;

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

/// Paint the original blank document workspace used to validate native startup.
pub fn show_workspace(context: &egui::Context) {
    egui::TopBottomPanel::top("title").show(context, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("bondline").strong().size(22.0));
            ui.separator();
            ui.label("Untitled");
        });
    });
    egui::TopBottomPanel::bottom("status").show(context, |ui| {
        ui.horizontal(|ui| {
            ui.label("Page 1");
            ui.separator();
            ui.label("A4");
        });
    });
    egui::CentralPanel::default()
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(232))
                .inner_margin(32.0),
        )
        .show(context, |ui| {
            let available = ui.available_rect_before_wrap();
            let width = available
                .width()
                .min(available.height() / std::f32::consts::SQRT_2)
                .max(1.0);
            let paper = egui::Rect::from_center_size(
                available.center(),
                egui::vec2(width, width * std::f32::consts::SQRT_2),
            );
            ui.painter().rect_filled(
                paper.translate(egui::vec2(3.0, 4.0)),
                1.0,
                egui::Color32::from_gray(210),
            );
            ui.painter().rect_filled(paper, 1.0, egui::Color32::WHITE);
            ui.painter().rect_stroke(
                paper,
                1.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(196)),
            );
        });
}

/// Native application state for startup and bounded smoke verification.
pub struct BondlineApp {
    smoke_frames: Option<u8>,
}

impl BondlineApp {
    /// Initialize the real wgpu context and font; smoke mode closes after three painted frames.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        smoke: bool,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let state = cc
            .wgpu_render_state
            .as_ref()
            .ok_or_else(|| std::io::Error::other("wgpu renderer did not initialize"))?;
        let info = state.adapter.get_info();
        if smoke {
            serde_json::to_writer(
                std::io::stdout(),
                &serde_json::json!({"app":"bondline", "backend":format!("{:?}", info.backend), "adapter":info.name, "device_type":format!("{:?}", info.device_type)}),
            )?;
        }
        configure_context(&cc.egui_ctx);
        Ok(Self {
            smoke_frames: smoke.then_some(0),
        })
    }
}

impl eframe::App for BondlineApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        show_workspace(context);
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
        super::show_workspace(&context);
        let frame = context.end_pass();
        assert!(frame.shapes.iter().any(|shape| matches!(&shape.shape, eframe::egui::Shape::Rect(rect) if rect.fill == eframe::egui::Color32::WHITE)));
    }
}
