//! Original bond-loop mark; coordinates match assets/brand/bondline-mark.svg.
use super::{ACCENT, egui};

const PATHS: &[&[[f32; 2]]] = &[
    &[[16.0, 12.0], [16.0, 52.0]],
    &[
        [16.0, 12.0],
        [36.0, 12.0],
        [48.0, 22.0],
        [36.0, 32.0],
        [16.0, 32.0],
    ],
    &[
        [16.0, 32.0],
        [36.0, 32.0],
        [48.0, 42.0],
        [36.0, 52.0],
        [16.0, 52.0],
    ],
];

pub(super) fn header(ui: &mut egui::Ui) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
    let project = |p: [f32; 2]| rect.min + egui::vec2(p[0], p[1]) * (28.0 / 64.0);
    let stroke = egui::Stroke::new(6.0 * 28.0 / 64.0, ACCENT);
    let mut joins = Vec::new();
    for path in PATHS {
        for edge in path.windows(2) {
            if let [a, b] = edge {
                ui.painter()
                    .line_segment([project(*a), project(*b)], stroke);
            }
        }
        for point in *path {
            if !joins.contains(point) {
                joins.push(*point);
            }
        }
    }
    for point in joins {
        ui.painter()
            .circle_filled(project(point), stroke.width / 2.0, ACCENT);
    }
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Other, true, "Bondline logo"));
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Image)
    });
}

pub(super) fn icon() -> Result<egui::IconData, bondline_render::Error> {
    let mut frame = bondline_render::cpu_solid(128, 128, [0; 4])?;
    for (index, pixel) in frame.rgba.chunks_exact_mut(4).enumerate() {
        let x = (index % 128) as f32;
        let y = (index / 128) as f32;
        let mut covered = 0_u16;
        let mut white = 0_u16;
        // Four samples keep the original vector's rounded tile, joins and caps legible.
        for sy in [0.25, 0.75] {
            for sx in [0.25, 0.75] {
                let p = [(x + sx) * 0.5, (y + sy) * 0.5];
                let dx = p[0] - p[0].clamp(14.0, 50.0);
                let dy = p[1] - p[1].clamp(14.0, 50.0);
                if dx * dx + dy * dy > 14.0 * 14.0 {
                    continue;
                }
                covered += 1;
                let ink = PATHS.iter().flat_map(|path| path.windows(2)).any(|edge| {
                    let a = edge[0];
                    let b = edge[1];
                    let dx = b[0] - a[0];
                    let dy = b[1] - a[1];
                    let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy))
                        .clamp(0.0, 1.0);
                    let x = p[0] - a[0] - t * dx;
                    let y = p[1] - a[1] - t * dy;
                    x * x + y * y <= 9.0
                });
                white += u16::from(ink);
            }
        }
        if covered != 0 {
            for (channel, teal) in pixel.iter_mut().take(3).zip([21_u16, 117, 118]) {
                *channel = ((white * 255 + (covered - white) * teal) / covered) as u8;
            }
            if let Some(alpha) = pixel.get_mut(3) {
                *alpha = (covered * 255 / 4) as u8;
            }
        }
    }
    Ok(egui::IconData {
        rgba: frame.rgba,
        width: frame.width,
        height: frame.height,
    })
}
