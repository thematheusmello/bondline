//! Literal no-p atom text whose explicit ink box certifies placement and numeric agreement.
use crate::{Cancellation, Error, PageViewport, Rectangle, label::Label};
use ab_glyph::{Font, GlyphId, point};
use bondline_model::{Atom, AtomLabel, Point2, Radical, TextJustification, TextRun, TextStyle};
use std::sync::Arc;
use ttf_parser::{Face, OutlineBuilder};

struct Layout {
    body: String,
    glyphs: Vec<(GlyphId, f64)>,
    ink: [f64; 4],
    element: [f64; 4],
    factor: f64,
}

fn layout(atom: &Atom) -> Result<Layout, Error> {
    let label = atom
        .authored_label
        .as_ref()
        .ok_or(Error::UnsupportedChemicalContent)?;
    if atom.charge != 0
        || atom.isotope.is_some()
        || atom.radical != Radical::None
        || label.position().is_some()
        || label.justification() != Some(TextJustification::Left)
        || label.label_justification() != Some(TextJustification::Left)
        || label.label_alignment().is_some()
        || label.runs().len() != 1
    {
        return Err(Error::UnsupportedChemicalContent);
    }
    let run = label
        .runs()
        .first()
        .ok_or(Error::UnsupportedChemicalContent)?;
    if run.style().font_family() != "Roboto" || run.style().face() != 0 {
        return Err(Error::UnsupportedChemicalContent);
    }
    let hydrogens = atom
        .explicit_hydrogens
        .ok_or(Error::UnsupportedChemicalContent)?;
    if atom.element.get() == 1 && hydrogens != 0 {
        return Err(Error::UnsupportedChemicalContent);
    }
    let symbol = atom
        .element
        .symbol()
        .ok_or(Error::UnsupportedChemicalContent)?;
    let mut body = symbol.to_owned();
    if hydrogens != 0 {
        body.push('H');
        if hydrogens != 1 {
            body.push_str(&hydrogens.to_string());
        }
    }
    let font = crate::label::font()?;
    let face = Face::parse(font.font_data(), 0).map_err(|_| Error::InvalidFont)?;
    let factor = run.style().size_pt() / f64::from(face.units_per_em());
    let mut glyphs = Vec::new();
    let (mut ink, mut element) = (None, None);
    let mut cursor = 0.0;
    let mut previous = None;
    for (index, character) in body.chars().enumerate() {
        let glyph = font.glyph_id(character);
        if glyph.0 == 0 {
            return Err(Error::InvalidFont);
        }
        if let Some(previous) = previous {
            cursor += crate::text::kerning(&face, previous, glyph)?;
        }
        let mut pen = CurveBounds::default();
        face.outline_glyph(ttf_parser::GlyphId(glyph.0), &mut pen)
            .ok_or(Error::InvalidFont)?;
        let [left, bottom, right, top] = pen.finish()?;
        let bounds = [left + cursor, bottom, right + cursor, top];
        union(&mut ink, bounds);
        if index < symbol.len() {
            union(&mut element, bounds);
        }
        glyphs.push((glyph, cursor));
        cursor += f64::from(font.h_advance_unscaled(glyph));
        previous = Some(glyph);
    }
    Ok(Layout {
        body,
        glyphs,
        ink: ink.ok_or(Error::InvalidFont)?,
        element: element.ok_or(Error::InvalidFont)?,
        factor,
    })
}

fn generated_box(atom: &Atom, layout: &Layout) -> Result<[f64; 4], Error> {
    let position = atom.position.ok_or(Error::UnsupportedChemicalContent)?;
    let baseline = [
        position.x_pt() - (layout.element[0] + layout.element[2]) * layout.factor / 2.0,
        position.y_pt() + (layout.element[1] + layout.element[3]) * layout.factor / 2.0,
    ];
    Ok([
        baseline[0] + layout.ink[0] * layout.factor,
        baseline[1] - layout.ink[3] * layout.factor,
        baseline[0] + layout.ink[2] * layout.factor,
        baseline[1] - layout.ink[1] * layout.factor,
    ])
}

/// Create a positioned literal label with a deliberately supplied supported style.
/// The input must have no authored label. No defaults, H or editing authority are inferred.
pub fn create_atom_label(atom: &Atom, style: TextStyle) -> Result<AtomLabel, Error> {
    if atom.authored_label.is_some() {
        return Err(Error::UnsupportedChemicalContent);
    }
    let symbol = atom
        .element
        .symbol()
        .ok_or(Error::UnsupportedChemicalContent)?;
    let seed = AtomLabel::new(
        None,
        Some(TextJustification::Left),
        Some(TextJustification::Left),
        None,
        vec![
            TextRun::new(symbol.to_owned(), style)
                .map_err(|_| Error::UnsupportedChemicalContent)?,
        ],
    )
    .map_err(|_| Error::UnsupportedChemicalContent)?;
    // This private typed seed never enters a document. The shared generator supplies
    // the complete canonical body and exact curve/GPOS box, retaining all atom data.
    let mut candidate = atom.clone();
    candidate.authored_label = Some(Arc::new(seed));
    position_atom_label(&candidate)
}

/// Generate literal body/ink bounds around the unchanged complete element-token anchor.
/// This does not infer valence, validate a source, or authorize document editing.
pub fn position_atom_label(atom: &Atom) -> Result<AtomLabel, Error> {
    let layout = layout(atom)?;
    let old = atom
        .authored_label
        .as_ref()
        .ok_or(Error::UnsupportedChemicalContent)?;
    let style = old
        .runs()
        .first()
        .ok_or(Error::UnsupportedChemicalContent)?
        .style()
        .clone();
    let run =
        TextRun::new(layout.body.clone(), style).map_err(|_| Error::UnsupportedChemicalContent)?;
    AtomLabel::new(
        None,
        old.justification(),
        old.label_justification(),
        None,
        vec![run],
    )
    .map_err(|_| Error::UnsupportedChemicalContent)?
    .with_bounding_box(Some(generated_box(atom, &layout)?))
    .map_err(|_| Error::LabelPlacement)
}

fn checked_layout(atom: &Atom) -> Result<(Layout, [f64; 4]), Error> {
    let label = atom
        .authored_label
        .as_ref()
        .ok_or(Error::UnsupportedChemicalContent)?;
    let raw = label
        .bounding_box()
        .ok_or(Error::UnsupportedChemicalContent)?;
    if raw[0] >= raw[2] || raw[1] >= raw[3] {
        return Err(Error::UnsupportedChemicalContent);
    }
    let layout = layout(atom)?;
    if label
        .runs()
        .first()
        .is_none_or(|run| run.text() != layout.body)
    {
        return Err(Error::UnsupportedChemicalContent);
    }
    let expected = generated_box(atom, &layout)?;
    if raw.iter().zip(expected).any(|(a, b)| {
        (a - b).abs() > 1.0 / 65536.0 + 128.0 * f64::EPSILON * a.abs().max(b.abs()).max(1.0)
    }) {
        return Err(Error::UnsupportedChemicalContent);
    }
    Ok((layout, raw))
}

/// Check the authored body and raw box; a generated replacement cannot prove the input.
pub fn validate_atom_label(atom: &Atom) -> Result<(), Error> {
    checked_layout(atom).map(|_| ())
}

pub(crate) fn render(
    atom: &Atom,
    viewport: &PageViewport,
    budget: &mut crate::text::RasterBudget,
    cancel: &Cancellation,
) -> Result<Label, Error> {
    cancel.check()?;
    let (layout, raw) = checked_layout(atom)?;
    let label = atom
        .authored_label
        .as_ref()
        .ok_or(Error::UnsupportedChemicalContent)?;
    let style = label
        .runs()
        .first()
        .ok_or(Error::UnsupportedChemicalContent)?
        .style();
    let em = style.size_pt() * viewport.scale();
    if !em.is_finite() || !(8.0..=256.0).contains(&em) {
        return Err(Error::LabelScale);
    }
    let baseline = Point2::new(
        raw[0] - layout.ink[0] * layout.factor,
        raw[1] + layout.ink[3] * layout.factor,
    )
    .map_err(|_| Error::LabelPlacement)?;
    let [x, y] = viewport.position_f64(baseline);
    if [x, y].iter().any(|v| !v.is_finite() || v.abs() > 100_000.0) {
        return Err(Error::LabelPlacement);
    }
    let font = crate::label::font()?;
    let factor = em / f64::from(font.units_per_em().ok_or(Error::InvalidFont)?);
    budget.reserve_runs(1)?;
    budget.reserve_characters(layout.body.chars().count())?;
    let glyphs = layout
        .glyphs
        .iter()
        .map(|(glyph, offset)| {
            font.outline_glyph(glyph.with_scale_and_position(
                (factor * f64::from(font.height_unscaled())) as f32,
                point((x + offset * factor) as f32, y as f32),
            ))
            .ok_or(Error::InvalidFont)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for glyph in &glyphs {
        cancel.check()?;
        budget.reserve(glyph, viewport)?;
    }
    let [r, g, b] = style.rgb().map(|v| (v * 255.0).round() as u8);
    let placed_glyphs = glyphs.iter().map(|glyph| glyph.glyph().clone()).collect();
    let mut pixels = Vec::new();
    let mut bounds = None;
    for glyph in glyphs {
        cancel.check()?;
        let rect = crate::text::bounds(&glyph, viewport)?;
        union(
            &mut bounds,
            [
                f64::from(rect.x),
                f64::from(rect.y),
                f64::from(rect.x + rect.width),
                f64::from(rect.y + rect.height),
            ],
        );
        pixels.extend(crate::text::raster(
            &glyph,
            viewport,
            [r, g, b, 255],
            cancel,
        )?);
        if pixels.len() > 4096 {
            return Err(Error::SceneTooComplex);
        }
    }
    let [left, top, right, bottom] = bounds.ok_or(Error::InvalidFont)?;
    let mask = Rectangle {
        x: left as u32 - 1,
        y: top as u32 - 1,
        width: (right - left) as u32 + 2,
        height: (bottom - top) as u32 + 2,
        color: [255; 4],
    };
    Ok(Label::flat(em as f32, mask, pixels, placed_glyphs))
}

fn union(target: &mut Option<[f64; 4]>, other: [f64; 4]) {
    *target = Some(match *target {
        Some(old) => [
            old[0].min(other[0]),
            old[1].min(other[1]),
            old[2].max(other[2]),
            old[3].max(other[3]),
        ],
        None => other,
    });
}

#[derive(Default)]
struct CurveBounds {
    bounds: Option<[f64; 4]>,
    current: [f64; 2],
    start: [f64; 2],
    invalid: bool,
}

impl CurveBounds {
    fn add(&mut self, point: [f64; 2]) {
        self.invalid |= point.iter().any(|v| !v.is_finite());
        union(&mut self.bounds, [point[0], point[1], point[0], point[1]]);
    }
    fn finish(self) -> Result<[f64; 4], Error> {
        let bounds = self.bounds.ok_or(Error::InvalidFont)?;
        if self.invalid || bounds[0] >= bounds[2] || bounds[1] >= bounds[3] {
            return Err(Error::InvalidFont);
        }
        Ok(bounds)
    }
}

impl OutlineBuilder for CurveBounds {
    fn move_to(&mut self, x: f32, y: f32) {
        self.current = [f64::from(x), f64::from(y)];
        self.start = self.current;
        self.add(self.current);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.current = [f64::from(x), f64::from(y)];
        self.add(self.current);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let from = self.current;
        let control = [f64::from(x1), f64::from(y1)];
        let to = [f64::from(x), f64::from(y)];
        self.add(to);
        for axis in 0..2 {
            let denominator = from[axis] - 2.0 * control[axis] + to[axis];
            if denominator != 0.0 {
                let t = (from[axis] - control[axis]) / denominator;
                if t > 0.0 && t < 1.0 {
                    self.add(std::array::from_fn(|i| {
                        (1.0 - t).powi(2) * from[i]
                            + 2.0 * (1.0 - t) * t * control[i]
                            + t * t * to[i]
                    }));
                }
            }
        }
        self.current = to;
    }
    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {
        // shortcut: pinned Roboto needs quadratics; add exact cubic extrema before CFF support.
        self.invalid = true;
    }
    fn close(&mut self) {
        self.current = self.start;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quadratic_bounds_use_curve_extrema_instead_of_loose_control_points() -> Result<(), Error> {
        let mut pen = CurveBounds::default();
        pen.move_to(0.0, 0.0);
        pen.quad_to(4.0, 10.0, 8.0, 0.0);
        pen.close();
        assert_eq!(pen.finish()?, [0.0, 0.0, 8.0, 5.0]);
        Ok(())
    }
}
