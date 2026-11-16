//! Independent design coordinates, not a chemistry golden or native OS claim.
use super::*;

#[test]
fn original_bond_loop_icon_uses_teal_white_and_transparent_rounded_corners()
-> Result<(), Box<dyn std::error::Error>> {
    let icon = original_icon()?;
    assert_eq!((icon.width, icon.height), (128, 128));
    assert_eq!(icon.rgba.len(), 128 * 128 * 4);
    let pixel = |x: usize, y: usize| -> Result<&[u8], Box<dyn std::error::Error>> {
        icon.rgba
            .get((y * 128 + x) * 4..(y * 128 + x + 1) * 4)
            .ok_or_else(|| "icon pixel".into())
    };
    for (x, y) in [(0, 0), (127, 0), (0, 127), (127, 127)] {
        assert_eq!(pixel(x, y)?, [0, 0, 0, 0]);
    }
    for (x, y) in [(64, 10), (64, 44), (112, 64)] {
        assert_eq!(pixel(x, y)?, [21, 117, 118, 255]);
    }
    for (x, y) in [(32, 40), (32, 64), (64, 64), (64, 104)] {
        assert_eq!(pixel(x, y)?, [255; 4]);
    }
    assert!(icon.rgba.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 255));
    Ok(())
}

#[test]
fn header_mark_keeps_the_vector_masters_round_caps_and_joins()
-> Result<(), Box<dyn std::error::Error>> {
    let context = egui::Context::default();
    context.enable_accesskit();
    let output = context.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 64.0),
            )),
            ..Default::default()
        },
        |context| {
            egui::CentralPanel::default().show(context, branding::header);
        },
    );
    let update = output
        .platform_output
        .accesskit_update
        .as_ref()
        .ok_or("AX output")?;
    let bounds = update
        .nodes
        .iter()
        .find_map(|(_, n)| {
            (n.name() == Some("Bondline logo"))
                .then(|| n.bounds())
                .flatten()
        })
        .ok_or("logo bounds")?;
    for [x, y] in [
        [16.0, 12.0],
        [16.0, 52.0],
        [36.0, 12.0],
        [48.0, 22.0],
        [36.0, 32.0],
        [16.0, 32.0],
        [48.0, 42.0],
        [36.0, 52.0],
    ] {
        let center = egui::pos2(
            bounds.x0 as f32 + x * 28.0 / 64.0,
            bounds.y0 as f32 + y * 28.0 / 64.0,
        );
        assert!(
            output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::Shape::Circle(circle) if circle.center.distance(center) < 0.001
                && (circle.radius - 1.3125).abs() < 0.001 && circle.fill == ACCENT)),
            "the original SVG requires a round cap/join at {x},{y}"
        );
    }
    Ok(())
}
