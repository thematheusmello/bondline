use crate::{Cancellation, Error, Rectangle};
use ab_glyph::{Font, FontRef, point};
use bondline_model::Element;

pub(crate) struct Label {
    pub em_px: f32,
    pub mask: Rectangle,
    pub pixels: Vec<Rectangle>,
    pub symbol_mask: Rectangle,
    pub hydrogen_mask: Option<Rectangle>,
    stacked: bool,
}

impl Label {
    pub fn masks(&self) -> impl Iterator<Item = &Rectangle> {
        std::iter::once(if self.stacked {
            &self.symbol_mask
        } else {
            &self.mask
        })
        .chain(self.hydrogen_mask.as_ref().filter(|_| self.stacked))
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Layout {
    Horizontal(bool),
    Above,
    Below,
}

pub(crate) fn font() -> Result<FontRef<'static>, Error> {
    FontRef::try_from_slice(include_bytes!("../../../assets/fonts/Roboto-Regular.ttf"))
        .map_err(|_| Error::InvalidFont)
}

#[cfg(test)]
pub(crate) fn terminal(
    position: [f32; 2],
    toward: [f32; 2],
    scale: f32,
    width: u32,
    height: u32,
) -> Result<Label, Error> {
    if toward
        .iter()
        .any(|value| !value.is_finite() || value.abs() > 100_000.0)
    {
        return Err(Error::LabelPlacement);
    }
    carbon(
        position,
        toward[0] >= position[0],
        3,
        scale,
        [width, height],
        &Cancellation::default(),
    )
}

/// Literal authored C/H count; no valence inference or general chemical text shaping.
pub(crate) fn carbon(
    position: [f32; 2],
    hydrogen_left: bool,
    hydrogens: u16,
    scale: f32,
    [width, height]: [u32; 2],
    cancel: &Cancellation,
) -> Result<Label, Error> {
    text(
        position,
        "C",
        Layout::Horizontal(hydrogen_left),
        hydrogens,
        scale,
        [width, height],
        cancel,
    )
}

pub(crate) fn element(
    position: [f32; 2],
    element: Element,
    hydrogen_left: bool,
    hydrogens: u16,
    scale: f32,
    viewport: [u32; 2],
    cancel: &Cancellation,
) -> Result<Label, Error> {
    if element.get() == 1 && hydrogens != 0 {
        return Err(Error::UnsupportedChemicalContent);
    }
    text(
        position,
        element.symbol().ok_or(Error::UnsupportedChemicalContent)?,
        Layout::Horizontal(hydrogen_left),
        hydrogens,
        scale,
        viewport,
        cancel,
    )
}

pub(crate) fn resolved_element(
    position: [f32; 2],
    element: Element,
    layout: Layout,
    hydrogens: u16,
    scale: f32,
    viewport: [u32; 2],
    cancel: &Cancellation,
) -> Result<Label, Error> {
    if element.get() == 1 && hydrogens != 0 {
        return Err(Error::UnsupportedChemicalContent);
    }
    text(
        position,
        element.symbol().ok_or(Error::UnsupportedChemicalContent)?,
        layout,
        hydrogens,
        scale,
        viewport,
        cancel,
    )
}

fn text(
    position: [f32; 2],
    symbol: &str,
    layout: Layout,
    hydrogens: u16,
    scale: f32,
    [width, height]: [u32; 2],
    cancel: &Cancellation,
) -> Result<Label, Error> {
    cancel.check()?;
    let hydrogen_left = matches!(layout, Layout::Horizontal(true));
    let stacked = !matches!(layout, Layout::Horizontal(_));
    let em = 12.0 * scale;
    if !em.is_finite() || !(8.0..=256.0).contains(&em) {
        return Err(Error::LabelScale);
    }
    if position
        .iter()
        .any(|value| !value.is_finite() || value.abs() > 100_000.0)
    {
        return Err(Error::LabelPlacement);
    }
    let font = font()?;
    let units = font.units_per_em().ok_or(Error::InvalidFont)?;
    let factor = em / units;
    let h = font.glyph_id('H');
    let mut letters = Vec::new();
    let mut advance = 0.0;
    let mut anchor = None;
    for character in symbol.chars() {
        let glyph = font.glyph_id(character);
        if anchor.is_none() || hydrogen_left {
            anchor = Some((glyph, advance));
        }
        letters.push((glyph, advance));
        advance += font.h_advance_unscaled(glyph);
    }
    let (glyph, prefix) = anchor.ok_or(Error::InvalidFont)?;
    let bounds = font.outline(glyph).ok_or(Error::InvalidFont)?.bounds;
    let origin = position[0] - prefix * factor - (bounds.min.x + bounds.max.x) * factor / 2.0;
    let baseline = position[1] + (bounds.min.y + bounds.max.y) * factor / 2.0;
    let sub_factor = factor * 2.0 / 3.0;
    let symbol_count = letters.len();
    let mut glyphs = letters
        .into_iter()
        .map(|(glyph, offset)| (glyph, origin + offset * factor, baseline, factor))
        .collect::<Vec<_>>();
    if hydrogens != 0 {
        let digits = if hydrogens == 1 {
            String::new()
        } else {
            hydrogens.to_string()
        };
        let mut subs = Vec::new();
        let h_origin;
        let h_baseline;
        if stacked {
            let bounds = font.outline(h).ok_or(Error::InvalidFont)?.bounds;
            h_origin = position[0] - (bounds.min.x + bounds.max.x) * factor / 2.0;
            h_baseline = position[1]
                + if matches!(layout, Layout::Above) {
                    -em
                } else {
                    em
                }
                + (bounds.min.y + bounds.max.y) * factor / 2.0;
            let mut cursor = h_origin + font.h_advance_unscaled(h) * factor;
            for digit in digits.chars() {
                let glyph = font.glyph_id(digit);
                subs.push((glyph, cursor, h_baseline + em / 4.0, sub_factor));
                cursor += font.h_advance_unscaled(glyph) * sub_factor;
            }
        } else if hydrogen_left {
            h_baseline = baseline;
            let mut cursor = origin;
            for digit in digits.chars().rev() {
                let glyph = font.glyph_id(digit);
                cursor -= font.h_advance_unscaled(glyph) * sub_factor;
                subs.push((glyph, cursor, baseline + em / 4.0, sub_factor));
            }
            subs.reverse();
            h_origin = cursor - font.h_advance_unscaled(h) * factor;
        } else {
            h_baseline = baseline;
            h_origin = origin + advance * factor;
            let mut cursor = h_origin + font.h_advance_unscaled(h) * factor;
            for digit in digits.chars() {
                let glyph = font.glyph_id(digit);
                subs.push((glyph, cursor, baseline + em / 4.0, sub_factor));
                cursor += font.h_advance_unscaled(glyph) * sub_factor;
            }
        }
        glyphs.push((h, h_origin, h_baseline, factor));
        glyphs.extend(subs);
    }
    let mut pixels = Vec::new();
    let mut left = u32::MAX;
    let mut top = u32::MAX;
    let mut right = 0;
    let mut bottom = 0;
    let mut symbol_bounds = [u32::MAX, u32::MAX, 0, 0];
    let mut hydrogen_bounds = [u32::MAX, u32::MAX, 0, 0];
    for (index, (glyph, x, y, factor)) in glyphs.into_iter().enumerate() {
        cancel.check()?;
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
        let group = if index < symbol_count {
            &mut symbol_bounds
        } else {
            &mut hydrogen_bounds
        };
        group[0] = group[0].min(x);
        group[1] = group[1].min(y);
        group[2] = group[2].max(x + w);
        group[3] = group[3].max(y + h);
        // shortcut: thresholded spans reuse rectangles; smooth text needs a dedicated primitive.
        let mut bitmap = vec![false; w as usize * h as usize];
        let mut invalid = false;
        outlined.draw(|column, row, coverage| {
            if cancel.is_cancelled() {
                return;
            }
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
        cancel.check()?;
        if invalid {
            return Err(Error::InvalidFont);
        }
        let start = pixels.len();
        for (row, scanline) in bitmap.chunks_exact(w as usize).enumerate() {
            cancel.check()?;
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
        em_px: em,
        mask: Rectangle {
            x: left - 1,
            y: top - 1,
            width: right - left + 2,
            height: bottom - top + 2,
            color: [255; 4],
        },
        pixels,
        symbol_mask: mask(symbol_bounds),
        hydrogen_mask: (hydrogens != 0).then(|| mask(hydrogen_bounds)),
        stacked,
    })
}

fn mask([left, top, right, bottom]: [u32; 4]) -> Rectangle {
    Rectangle {
        x: left - 1,
        y: top - 1,
        width: right - left + 2,
        height: bottom - top + 2,
        color: [255; 4],
    }
}
