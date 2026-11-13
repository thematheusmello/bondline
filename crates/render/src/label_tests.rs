use ab_glyph::Font;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn terminal_font_metrics_match_independent_fonttools_4611_vectors()
-> Result<(), Box<dyn std::error::Error>> {
    // From the already SHA-pinned Apache Roboto with external fontTools 4.61.1.
    let font = super::label::font()?;
    assert_eq!(font.units_per_em(), Some(2048.0));
    assert_eq!(font.ascent_unscaled(), 2146.0);
    assert_eq!(font.descent_unscaled(), -555.0);
    for (character, id, advance, [xmin, ymin, xmax, ymax]) in [
        ('C', 40, 1333, [119, -20, 1240, 1476]),
        ('H', 45, 1460, [169, 0, 1288, 1456]),
        ('3', 24, 1150, [94, -20, 1017, 1476]),
    ] {
        let glyph = font.glyph_id(character);
        assert_eq!(glyph.0, id);
        assert_eq!(font.h_advance_unscaled(glyph), advance as f32);
        let bounds = font.outline(glyph).ok_or("outline")?.bounds;
        assert_eq!(
            [bounds.min.x, bounds.max.y, bounds.max.x, bounds.min.y],
            [xmin, ymin, xmax, ymax].map(|value| value as f32)
        );
    }
    Ok(())
}

#[test]
fn terminal_labels_keep_the_carbon_at_the_bond_endpoint_and_bound_raster_work()
-> Result<(), Box<dyn std::error::Error>> {
    let left = super::label::terminal([60.0, 60.0], [100.0, 60.0], 1.0, 120, 120)?;
    let right = super::label::terminal([60.0, 60.0], [20.0, 60.0], 1.0, 120, 120)?;
    assert!(left.mask.x < right.mask.x);
    assert_eq!(left.mask.y, right.mask.y);
    for label in [left, right] {
        assert!(!label.pixels.is_empty());
        assert_eq!(label.mask.color, [255; 4]);
        for pixel in label.pixels {
            assert_eq!(pixel.color, [0, 0, 0, 255]);
            assert_eq!(pixel.height, 1);
            assert!(
                pixel.x >= label.mask.x && pixel.x + pixel.width <= label.mask.x + label.mask.width
            );
            assert!(
                pixel.y >= label.mask.y
                    && pixel.y + pixel.height <= label.mask.y + label.mask.height
            );
        }
    }
    for scale in [0.0, f32::NAN, 0.01, 1000.0] {
        assert!(super::label::terminal([60.0, 60.0], [100.0, 60.0], scale, 120, 120).is_err());
    }
    assert!(super::label::terminal([0.0, 0.0], [10.0, 10.0], 1.0, 120, 120).is_err());
    Ok(())
}

#[test]
fn terminal_raster_labels_agree_with_independent_external_freetype()
-> Result<(), Box<dyn std::error::Error>> {
    let interpreter = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    for toward in [[300.0, 200.0], [100.0, 200.0]] {
        let label = super::label::terminal([200.0, 200.0], toward, 8.0, 400, 400)?;
        let boxes: Vec<_> = label
            .pixels
            .iter()
            .map(|r| [r.x, r.y, r.width, r.height])
            .collect();
        let mut child = Command::new(&interpreter)
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/label_oracle.py"
            ))
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../assets/fonts/Roboto-Regular.ttf"
            ))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut input = child.stdin.take().ok_or("oracle stdin")?;
        serde_json::to_writer(
            &mut input,
            &serde_json::json!({"toward_right":toward[0]>200.0,"boxes":boxes}),
        )?;
        input.flush()?;
        drop(input);
        let output = child.wait_with_output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}
