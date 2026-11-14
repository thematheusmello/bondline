use ab_glyph::Font;
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn authored_count_oracle(
    label: &super::label::Label,
    left: bool,
    expected: u16,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let interpreter = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(interpreter)
        .arg("-B")
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
    let boxes = label
        .pixels
        .iter()
        .map(|r| [r.x, r.y, r.width, r.height])
        .collect::<Vec<_>>();
    serde_json::to_writer(
        child.stdin.take().ok_or("stdin")?,
        &serde_json::json!({"boxes":boxes,"toward_right":left,"hydrogens":expected,"position":[400,200],"width":800,"height":400}),
    )?;
    Ok(child.wait_with_output()?)
}

#[test]
fn literal_authored_hydrogen_counts_use_independent_freetype_without_inference()
-> Result<(), Box<dyn std::error::Error>> {
    for left in [false, true] {
        for hydrogens in [0, 1, 2, 3, 12, u16::MAX] {
            let label = super::label::carbon(
                [400.0, 200.0],
                left,
                hydrogens,
                8.0,
                [800, 400],
                &crate::Cancellation::default(),
            )?;
            let output = authored_count_oracle(&label, left, hydrogens)?;
            assert!(
                output.status.success(),
                "H{hydrogens}/{left}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    let label = super::label::carbon(
        [400.0, 200.0],
        false,
        0,
        8.0,
        [800, 400],
        &crate::Cancellation::default(),
    )?;
    assert!(
        !authored_count_oracle(&label, false, 3)?.status.success(),
        "oracle accepted an invented CH3 from authored zero"
    );
    let label = super::label::carbon(
        [400.0, 200.0],
        true,
        12,
        8.0,
        [800, 400],
        &crate::Cancellation::default(),
    )?;
    assert!(
        !authored_count_oracle(&label, true, 21)?.status.success(),
        "oracle accepted a reversed decimal count"
    );
    Ok(())
}

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
fn ascii_digit_advances_match_independent_opentype_hmtx_vectors()
-> Result<(), Box<dyn std::error::Error>> {
    // Independent stdlib cmap4/hmtx inspection of the pinned TTF, not renderer output.
    let font = super::label::font()?;
    for (index, digit) in ('0'..='9').enumerate() {
        let glyph = font.glyph_id(digit);
        assert_eq!(glyph.0, 21 + u16::try_from(index)?);
        assert_eq!(font.h_advance_unscaled(glyph), 1150.0);
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

fn element_oracle(
    label: &super::label::Label,
    symbol: &str,
    left: bool,
    hydrogens: u16,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let interpreter = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(interpreter)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/element_label_oracle.py"
        ))
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/fonts/Roboto-Regular.ttf"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let boxes = label
        .pixels
        .iter()
        .map(|r| [r.x, r.y, r.width, r.height])
        .collect::<Vec<_>>();
    serde_json::to_writer(
        child.stdin.take().ok_or("stdin")?,
        &serde_json::json!({
            "boxes":boxes,"symbol":symbol,"hydrogen_left":left,"hydrogens":hydrogens
        }),
    )?;
    Ok(child.wait_with_output()?)
}

#[test]
fn neutral_element_glyphs_and_authored_h_face_independent_freetype_masks()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    let mut minimum = 1.0_f64;
    let mut minimum_glyph = 1.0_f64;
    for (number, symbol) in [
        (7, "N"),
        (8, "O"),
        (9, "F"),
        (17, "Cl"),
        (35, "Br"),
        (14, "Si"),
        (2, "He"),
        (118, "Og"),
    ] {
        for left in [false, true] {
            for hydrogens in [0, 1, 2, 12, u16::MAX] {
                let label = super::label::element(
                    [400.0, 200.0],
                    bondline_model::Element::try_from(number)?,
                    left,
                    hydrogens,
                    8.0,
                    [800, 400],
                    &crate::Cancellation::default(),
                )?;
                let output = element_oracle(&label, symbol, left, hydrogens)?;
                if !output.status.success() {
                    failures.push(format!(
                        "{symbol}/{left}/H{hydrogens}: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                } else {
                    let data: serde_json::Value = serde_json::from_slice(&output.stdout)?;
                    minimum = minimum.min(
                        data["freetype_ink_iou"]
                            .as_f64()
                            .ok_or("whole-label score")?,
                    );
                    minimum_glyph =
                        minimum_glyph.min(data["minimum_glyph_iou"].as_f64().ok_or("glyph score")?);
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    println!(
        "80 whole-label comparisons: minimumIoU={minimum}; exploratory minimumglyphIoU={minimum_glyph}"
    );
    let label = super::label::element(
        [400.0, 200.0],
        bondline_model::Element::try_from(7)?,
        false,
        2,
        8.0,
        [800, 400],
        &crate::Cancellation::default(),
    )?;
    assert!(
        !element_oracle(&label, "O", false, 2)?.status.success(),
        "wrong element accepted"
    );
    assert!(
        !element_oracle(&label, "N", true, 2)?.status.success(),
        "wrong H placement accepted"
    );
    let positive = element_oracle(&label, "N", false, 2)?;
    assert!(positive.status.success());
    let positive: serde_json::Value = serde_json::from_slice(&positive.stdout)?;
    assert!(
        positive["minimum_glyph_iou"]
            .as_f64()
            .ok_or("positive glyph ROI")?
            >= 0.90
    );
    let wrong = element_oracle(&label, "N", false, 3)?;
    if wrong.status.success() {
        let wrong: serde_json::Value = serde_json::from_slice(&wrong.stdout)?;
        assert!(
            wrong["minimum_glyph_iou"]
                .as_f64()
                .ok_or("wrong subscript ROI")?
                < 0.90,
            "invented H accepted"
        );
        println!(
            "N/H2 correctROI={}, H3 counterfactualROI={}",
            positive["minimum_glyph_iou"], wrong["minimum_glyph_iou"]
        );
    }
    let label = super::label::element(
        [400.0, 200.0],
        bondline_model::Element::try_from(17)?,
        false,
        0,
        8.0,
        [800, 400],
        &crate::Cancellation::default(),
    )?;
    assert!(
        !element_oracle(&label, "CL", false, 0)?.status.success(),
        "wrong symbol case accepted"
    );
    let label = super::label::element(
        [400.0, 200.0],
        bondline_model::Element::try_from(35)?,
        true,
        0,
        8.0,
        [800, 400],
        &crate::Cancellation::default(),
    )?;
    assert!(
        !element_oracle(&label, "Br", false, 0)?.status.success(),
        "wrong chemical anchor accepted"
    );
    Ok(())
}

#[test]
fn neutral_label_metrics_match_independent_opentype_vectors()
-> Result<(), Box<dyn std::error::Error>> {
    let font = super::label::font()?;
    for (character, id, advance, [xmin, ymin, xmax, ymax]) in [
        ('N', 51, 1460, [169, 0, 1288, 1456]),
        ('O', 52, 1408, [118, -20, 1289, 1476]),
        ('F', 43, 1132, [169, 0, 1071, 1456]),
        ('l', 81, 497, [156, 0, 341, 1536]),
        ('B', 39, 1275, [169, 0, 1160, 1456]),
        ('r', 87, 693, [140, 0, 663, 1102]),
        ('S', 56, 1215, [80, -20, 1138, 1476]),
        ('i', 78, 497, [141, 0, 360, 1476]),
        ('e', 74, 1085, [93, -20, 1011, 1102]),
        ('g', 76, 1149, [96, -426, 1010, 1102]),
        ('L', 49, 1102, [169, 0, 1052, 1456]),
    ] {
        let glyph = font.glyph_id(character);
        assert_eq!(glyph.0, id);
        assert_eq!(font.h_advance_unscaled(glyph), advance as f32);
        let bounds = font.outline(glyph).ok_or("outline")?.bounds;
        assert_eq!(
            [bounds.min.x, bounds.max.y, bounds.max.x, bounds.min.y],
            [xmin, ymin, xmax, ymax].map(|v| v as f32)
        );
    }
    Ok(())
}
