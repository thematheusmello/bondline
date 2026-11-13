use crate::{Error, Rectangle};
use ab_glyph::{Font, FontRef, point};

pub(crate) struct Label {
    pub mask: Rectangle,
    pub pixels: Vec<Rectangle>,
}

pub(crate) fn font() -> Result<FontRef<'static>, Error> {
    FontRef::try_from_slice(include_bytes!("../../../assets/fonts/Roboto-Regular.ttf"))
        .map_err(|_| Error::InvalidFont)
}

/// Authored terminal CH3 only. No valence inference or general chemical text shaping.
pub(crate) fn terminal(
    position: [f32; 2],
    toward: [f32; 2],
    scale: f32,
    width: u32,
    height: u32,
) -> Result<Label, Error> {
    let em = 12.0 * scale;
    if !em.is_finite() || !(8.0..=256.0).contains(&em) {
        return Err(Error::LabelScale);
    }
    if position
        .iter()
        .chain(&toward)
        .any(|value| !value.is_finite() || value.abs() > 100_000.0)
    {
        return Err(Error::LabelPlacement);
    }
    let font = font()?;
    let units = font.units_per_em().ok_or(Error::InvalidFont)?;
    let factor = em / units;
    let c = font.glyph_id('C');
    let h = font.glyph_id('H');
    let three = font.glyph_id('3');
    let bounds = font.outline(c).ok_or(Error::InvalidFont)?.bounds;
    let origin = position[0] - (bounds.min.x + bounds.max.x) * factor / 2.0;
    let baseline = position[1] + (bounds.min.y + bounds.max.y) * factor / 2.0;
    let sub_factor = factor * 2.0 / 3.0;
    let (h_origin, three_origin) = if toward[0] >= position[0] {
        let three_origin = origin - font.h_advance_unscaled(three) * sub_factor;
        (
            three_origin - font.h_advance_unscaled(h) * factor,
            three_origin,
        )
    } else {
        let h_origin = origin + font.h_advance_unscaled(c) * factor;
        (h_origin, h_origin + font.h_advance_unscaled(h) * factor)
    };
    let mut pixels = Vec::new();
    let mut left = u32::MAX;
    let mut top = u32::MAX;
    let mut right = 0;
    let mut bottom = 0;
    for (glyph, x, y, factor) in [
        (c, origin, baseline, factor),
        (h, h_origin, baseline, factor),
        (three, three_origin, baseline + em / 4.0, sub_factor),
    ] {
        let outlined = font
            .outline_glyph(
                glyph.with_scale_and_position(factor * font.height_unscaled(), point(x, y)),
            )
            .ok_or(Error::InvalidFont)?;
        let bounds = outlined.px_bounds();
        if bounds.min.x < 1.0
            || bounds.min.y < 1.0
            || bounds.max.x >= width as f32
            || bounds.max.y >= height as f32
        {
            return Err(Error::LabelPlacement);
        }
        let x = bounds.min.x as u32;
        let y = bounds.min.y as u32;
        let w = (bounds.max.x - bounds.min.x) as u32;
        let h = (bounds.max.y - bounds.min.y) as u32;
        if w == 0 || h == 0 || w > 512 || h > 512 {
            return Err(Error::LabelScale);
        }
        left = left.min(x);
        top = top.min(y);
        right = right.max(x + w);
        bottom = bottom.max(y + h);
        // shortcut: thresholded spans reuse rectangles; smooth text needs a dedicated primitive.
        let mut bitmap = vec![false; w as usize * h as usize];
        let mut invalid = false;
        outlined.draw(|column, row, coverage| {
            if column >= w || row >= h || !coverage.is_finite() {
                invalid = true;
                return;
            }
            if let Some(pixel) = bitmap.get_mut((row * w + column) as usize) {
                *pixel = coverage >= 0.5;
            } else {
                invalid = true;
            }
        });
        if invalid {
            return Err(Error::InvalidFont);
        }
        let start = pixels.len();
        for (row, scanline) in bitmap.chunks_exact(w as usize).enumerate() {
            let mut column = 0;
            while column < scanline.len() {
                if scanline.get(column) != Some(&true) {
                    column += 1;
                    continue;
                }
                let begin = column;
                while scanline.get(column) == Some(&true) {
                    column += 1;
                }
                pixels.push(Rectangle {
                    x: x + begin as u32,
                    y: y + row as u32,
                    width: (column - begin) as u32,
                    height: 1,
                    color: [0, 0, 0, 255],
                });
                if pixels.len() > 4096 {
                    return Err(Error::SceneTooComplex);
                }
            }
        }
        if pixels.len() == start {
            return Err(Error::LabelScale);
        }
    }
    Ok(Label {
        mask: Rectangle {
            x: left - 1,
            y: top - 1,
            width: right - left + 2,
            height: bottom - top + 2,
            color: [255; 4],
        },
        pixels,
    })
}
