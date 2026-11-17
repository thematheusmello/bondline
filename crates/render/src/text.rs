//! Resolved plain captions; no system font, substitution, bidi or chemical interpretation.
use crate::{Cancellation, Error, PageViewport, Primitive, Rectangle, Scene};
use ab_glyph::{Font, GlyphId, OutlinedGlyph, point};
use bondline_model::Page;
use ttf_parser::{
    Face, Tag,
    gpos::{PairAdjustment, PositioningSubtable, ValueRecord},
};

const MAX_GLYPHS: usize = 4096;
const MAX_RUNS: usize = 4096;
const MAX_RASTER_PIXELS: u64 = 1_048_576;

pub(crate) fn append(
    page: &Page,
    viewport: &PageViewport,
    scene: &mut Scene,
    cancel: &Cancellation,
) -> Result<Option<[f32; 2]>, Error> {
    if page.texts().is_empty() {
        return Ok(None);
    }
    let mut raster_pixels = 0_u64;
    let mut outlined_glyphs = scene.primitives.len();
    // Validate the entire immutable page before allocating the first glyph bitmap.
    let range = walk(page, viewport, cancel, |glyph, _| {
        outlined_glyphs = outlined_glyphs
            .checked_add(1)
            .filter(|total| *total <= 4096)
            .ok_or(Error::SceneTooComplex)?;
        let bounds = bounds(&glyph, viewport)?;
        raster_pixels = raster_pixels
            .checked_add(u64::from(bounds.width) * u64::from(bounds.height))
            .filter(|total| *total <= MAX_RASTER_PIXELS)
            .ok_or(Error::SceneTooComplex)?;
        Ok(())
    })?;
    walk(page, viewport, cancel, |glyph, color| {
        let bounds = bounds(&glyph, viewport)?;
        let length = usize::try_from(u64::from(bounds.width) * u64::from(bounds.height))
            .map_err(|_| Error::SceneTooComplex)?;
        let mut bitmap = vec![false; length];
        let mut invalid = false;
        glyph.draw(|x, y, coverage| {
            if cancel.is_cancelled() {
                return;
            }
            if x >= bounds.width || y >= bounds.height || !coverage.is_finite() {
                invalid = true;
                return;
            }
            if let Some(pixel) =
                bitmap.get_mut((u64::from(y) * u64::from(bounds.width) + u64::from(x)) as usize)
            {
                *pixel = coverage >= 0.5;
            } else {
                invalid = true;
            }
        });
        cancel.check()?;
        if invalid {
            return Err(Error::InvalidFont);
        }
        let start = scene.primitives.len();
        for (row, scanline) in bitmap.chunks_exact(bounds.width as usize).enumerate() {
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
                if scene.primitives.len() >= 4096 {
                    return Err(Error::SceneTooComplex);
                }
                scene.primitives.push(Primitive::Rectangle(Rectangle {
                    x: bounds.x + begin as u32,
                    y: bounds.y + row as u32,
                    width: (column - begin) as u32,
                    height: 1,
                    color,
                }));
            }
        }
        if scene.primitives.len() == start {
            return Err(Error::LabelScale);
        }
        Ok(())
    })?;
    Ok(range)
}

fn walk(
    page: &Page,
    viewport: &PageViewport,
    cancel: &Cancellation,
    mut visit: impl FnMut(OutlinedGlyph, [u8; 4]) -> Result<(), Error>,
) -> Result<Option<[f32; 2]>, Error> {
    let mut runs = 0_usize;
    for text in page.texts() {
        cancel.check()?;
        runs = runs
            .checked_add(text.runs().len())
            .filter(|total| *total <= MAX_RUNS)
            .ok_or(Error::SceneTooComplex)?;
    }
    let font = crate::label::font()?;
    let face = Face::parse(font.font_data(), 0).map_err(|_| Error::InvalidFont)?;
    let units = f64::from(font.units_per_em().ok_or(Error::InvalidFont)?);
    let mut glyphs = 0_usize;
    let mut range: Option<[f32; 2]> = None;
    for text in page.texts() {
        let [mut cursor, baseline] = viewport.position_f64(text.position());
        for run in text.runs() {
            cancel.check()?;
            let style = run.style();
            if style.font_family() != "Roboto" || style.face() != 0 {
                return Err(Error::UnsupportedText(
                    "only exact Roboto regular face0 is supported",
                ));
            }
            let em = style.size_pt() * viewport.scale();
            if !em.is_finite() || !(8.0..=256.0).contains(&em) {
                return Err(Error::LabelScale);
            }
            range = Some(match range {
                Some([min, max]) => [min.min(em as f32), max.max(em as f32)],
                None => [em as f32; 2],
            });
            let factor = em / units;
            let [r, g, b] = style.rgb().map(|channel| (channel * 255.0).round() as u8);
            let mut previous = None;
            for character in run.text().chars() {
                cancel.check()?;
                glyphs = glyphs
                    .checked_add(1)
                    .filter(|total| *total <= MAX_GLYPHS)
                    .ok_or(Error::SceneTooComplex)?;
                if character.is_control()
                    || matches!(character,
                    '\u{0590}'..='\u{1cff}' | '\u{1dc0}'..='\u{1dff}' |
                    '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202e}' |
                    '\u{2060}'..='\u{206f}' | '\u{20d0}'..='\u{20ff}' |
                    '\u{fe00}'..='\u{fe0f}' | '\u{fe20}'..='\u{fe2f}' |
                    '\u{fb1d}'..='\u{fdff}' | '\u{fe70}'..='\u{feff}')
                {
                    return Err(Error::UnsupportedText(
                        "line controls or complex/bidirectional shaping are unsupported",
                    ));
                }
                let glyph = font.glyph_id(character);
                if glyph.0 == 0 {
                    return Err(Error::UnsupportedText(
                        "the requested character has no font glyph",
                    ));
                }
                if face
                    .tables()
                    .gdef
                    .and_then(|table| table.glyph_class(ttf_parser::GlyphId(glyph.0)))
                    == Some(ttf_parser::gdef::GlyphClass::Mark)
                {
                    return Err(Error::UnsupportedText(
                        "combining mark positioning is unsupported",
                    ));
                }
                if let Some(previous) = previous {
                    cursor += kerning(&face, previous, glyph)? * factor;
                }
                if !cursor.is_finite()
                    || !baseline.is_finite()
                    || cursor.abs() > 100_000.0
                    || baseline.abs() > 100_000.0
                {
                    return Err(Error::LabelPlacement);
                }
                if let Some(outline) = font.outline_glyph(glyph.with_scale_and_position(
                    (factor * f64::from(font.height_unscaled())) as f32,
                    point(cursor as f32, baseline as f32),
                )) {
                    visit(outline, [r, g, b, 255])?;
                }
                cursor += f64::from(font.h_advance_unscaled(glyph)) * factor;
                if !cursor.is_finite() || cursor.abs() > 100_000.0 {
                    return Err(Error::LabelPlacement);
                }
                previous = Some(glyph);
            }
        }
    }
    Ok(range)
}

fn bounds(glyph: &OutlinedGlyph, viewport: &PageViewport) -> Result<Rectangle, Error> {
    let bounds = glyph.px_bounds();
    let [left, top, right, bottom] = viewport.paper_bounds();
    if [bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y]
        .iter()
        .any(|v| !v.is_finite())
        || f64::from(bounds.min.x) < left + 1.0
        || f64::from(bounds.min.y) < top + 1.0
        || f64::from(bounds.max.x) > right - 1.0
        || f64::from(bounds.max.y) > bottom - 1.0
    {
        return Err(Error::LabelPlacement);
    }
    let width = bounds.max.x - bounds.min.x;
    let height = bounds.max.y - bounds.min.y;
    if width <= 0.0 || height <= 0.0 || width > 100_000.0 || height > 100_000.0 {
        return Err(Error::LabelScale);
    }
    Ok(Rectangle {
        x: bounds.min.x as u32,
        y: bounds.min.y as u32,
        width: width as u32,
        height: height as u32,
        color: [0; 4],
    })
}

// The pinned Roboto's DFLT/Latin/Greek/Cyrillic kern features share the same
// pair lookup (FontTools-qualified). No GSUB, contextual, mark or device shaping.
fn kerning(face: &Face<'_>, first: GlyphId, second: GlyphId) -> Result<f64, Error> {
    let table = face.tables().gpos.ok_or(Error::InvalidFont)?;
    let language = table
        .scripts
        .find(Tag::from_bytes(b"DFLT"))
        .and_then(|script| script.default_language)
        .ok_or(Error::InvalidFont)?;
    let (first, second) = (ttf_parser::GlyphId(first.0), ttf_parser::GlyphId(second.0));
    let mut advance = 0_i32;
    for index in language.feature_indices {
        let feature = table.features.get(index).ok_or(Error::InvalidFont)?;
        if feature.tag != Tag::from_bytes(b"kern") {
            continue;
        }
        for index in feature.lookup_indices {
            let lookup = table.lookups.get(index).ok_or(Error::InvalidFont)?;
            if lookup.flags.0 != 0 {
                return Err(Error::InvalidFont);
            }
            for subtable in lookup.subtables.into_iter::<PositioningSubtable<'_>>() {
                let PositioningSubtable::Pair(pair) = subtable else {
                    return Err(Error::InvalidFont);
                };
                let values = match pair {
                    PairAdjustment::Format1 { coverage, sets } => coverage
                        .get(first)
                        .and_then(|index| sets.get(index))
                        .and_then(|set| set.get(second)),
                    PairAdjustment::Format2 {
                        coverage,
                        classes,
                        matrix,
                    } => coverage
                        .get(first)
                        .and_then(|_| matrix.get((classes.0.get(first), classes.1.get(second)))),
                };
                if let Some((a, b)) = values {
                    if !plain_advance(a) || !plain_advance(b) || b.x_advance != 0 {
                        return Err(Error::InvalidFont);
                    }
                    advance = advance
                        .checked_add(i32::from(a.x_advance))
                        .ok_or(Error::InvalidFont)?;
                    break;
                }
            }
        }
    }
    Ok(f64::from(advance))
}

fn plain_advance(value: ValueRecord<'_>) -> bool {
    value.x_placement == 0
        && value.y_placement == 0
        && value.y_advance == 0
        && value.x_placement_device.is_none()
        && value.y_placement_device.is_none()
        && value.x_advance_device.is_none()
        && value.y_advance_device.is_none()
}
