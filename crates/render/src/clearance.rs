//! Original bounded Euclidean glyph-ink clearance for finite butt bond components.
use crate::{Cancellation, Error, Stroke, label::Label};
use ab_glyph::{Font, Glyph};
use ttf_parser::{Face, OutlineBuilder};

const CUT_TOLERANCE: f64 = 1e-4;
const MAX_WORK: usize = 131_072;
const MAX_DEPTH: u8 = 64;

#[derive(Default)]
pub(crate) struct Budget {
    work: usize,
}

impl Budget {
    fn reserve(&mut self, cancel: &Cancellation) -> Result<(), Error> {
        cancel.check()?;
        if self.work == MAX_WORK {
            return Err(Error::SceneTooComplex);
        }
        self.work += 1;
        Ok(())
    }
}

struct Maximum<'a> {
    margin: f64,
    halfwidth: f64,
    lower: f64,
    upper: f64,
    limit: f64,
    budget: &'a mut Budget,
    cancel: &'a Cancellation,
}

impl Maximum<'_> {
    fn value(&self, [a, c]: [f64; 2]) -> Option<f64> {
        let d = (c.abs() - self.halfwidth).max(0.0);
        (d <= self.margin).then(|| a + ((self.margin - d) * (self.margin + d)).max(0.0).sqrt())
    }

    fn sample(&mut self, p: [f64; 2]) -> Result<(), Error> {
        if let Some(value) = self.value(p) {
            self.lower = self.lower.max(value - arithmetic_pad(&[p], self.margin));
            if self.lower >= self.limit {
                return Err(Error::LabelOverlap);
            }
        }
        Ok(())
    }

    fn line(&mut self, a: [f64; 2], b: [f64; 2]) -> Result<(), Error> {
        self.budget.reserve(self.cancel)?;
        let delta = [b[0] - a[0], b[1] - a[1]];
        let extent = self.halfwidth + self.margin;
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        if delta[1] == 0.0 {
            if a[1].abs() > extent {
                return Ok(());
            }
        } else {
            let p = (-extent - a[1]) / delta[1];
            let q = (extent - a[1]) / delta[1];
            lo = lo.max(p.min(q));
            hi = hi.min(p.max(q));
            if lo > hi {
                return Ok(());
            }
        }
        let mut candidates = vec![lo, hi];
        if delta[1] != 0.0 {
            for sign in [-1.0, 1.0] {
                candidates.push((sign * self.halfwidth - a[1]) / delta[1]);
                if delta[0] * sign * delta[1] > 0.0 {
                    let reach = self.margin * delta[0].abs() / delta[0].hypot(delta[1]);
                    candidates.push((sign * (self.halfwidth + reach) - a[1]) / delta[1]);
                }
            }
        }
        for t in candidates {
            if (lo..=hi).contains(&t) {
                // Analytically clipped closed endpoints can round just outside the band.
                self.sample([
                    a[0] + delta[0] * t,
                    (a[1] + delta[1] * t).clamp(-extent, extent),
                ])?;
            }
        }
        self.upper = self
            .upper
            .max(self.lower + arithmetic_pad(&[a, b], self.margin));
        Ok(())
    }

    fn quadratic(&mut self, points: [[f64; 2]; 3]) -> Result<(), Error> {
        let mut stack = vec![(points, 0_u8)];
        while let Some((p, depth)) = stack.pop() {
            self.budget.reserve(self.cancel)?;
            let pad = arithmetic_pad(&p, self.margin);
            let min_c = p.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
            let max_c = p.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
            let distance = (min_c - self.halfwidth)
                .max(-max_c - self.halfwidth)
                .max(0.0);
            if distance > self.margin + pad {
                continue;
            }
            let distance = (distance - pad).max(0.0).min(self.margin);
            let bound = p.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max)
                + ((self.margin - distance) * (self.margin + distance)).sqrt()
                + pad;
            let left = midpoint(p[0], p[1]);
            let right = midpoint(p[1], p[2]);
            let middle = midpoint(left, right);
            for point in [p[0], middle, p[2]] {
                self.sample(point)?;
            }
            if bound <= self.lower + CUT_TOLERANCE {
                self.upper = self.upper.max(bound);
            } else {
                if depth == MAX_DEPTH {
                    return Err(Error::SceneTooComplex);
                }
                stack.push(([middle, right, p[2]], depth + 1));
                stack.push(([p[0], left, middle], depth + 1));
            }
        }
        Ok(())
    }
}

fn midpoint(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]
}

fn arithmetic_pad(points: &[[f64; 2]], margin: f64) -> f64 {
    64.0 * f64::EPSILON
        * points
            .iter()
            .flatten()
            .map(|v| v.abs())
            .fold(margin.max(1.0), f64::max)
}

struct Pen<'a, 'b> {
    maximum: &'a mut Maximum<'b>,
    glyph: &'a Glyph,
    factor: [f64; 2],
    origin: [f64; 2],
    tangent: [f64; 2],
    normal: [f64; 2],
    offset: f64,
    current: [f64; 2],
    start: [f64; 2],
    error: Option<Error>,
}

impl Pen<'_, '_> {
    fn point(&mut self, x: f32, y: f32) -> [f64; 2] {
        let dx = f64::from(self.glyph.position.x) + f64::from(x) * self.factor[0] - self.origin[0];
        let dy = f64::from(self.glyph.position.y) - f64::from(y) * self.factor[1] - self.origin[1];
        let p = [
            dx * self.tangent[0] + dy * self.tangent[1],
            dx * self.normal[0] + dy * self.normal[1] - self.offset,
        ];
        if p.iter().any(|v| !v.is_finite()) {
            self.error = Some(Error::InvalidFont);
        }
        p
    }
}

impl OutlineBuilder for Pen<'_, '_> {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.error.is_none() {
            self.current = self.point(x, y);
            self.start = self.current;
        }
    }
    fn line_to(&mut self, x: f32, y: f32) {
        if self.error.is_none() {
            let to = self.point(x, y);
            if self.error.is_none() {
                self.error = self.maximum.line(self.current, to).err();
                self.current = to;
            }
        }
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        if self.error.is_none() {
            let control = self.point(x1, y1);
            let to = self.point(x, y);
            if self.error.is_none() {
                self.error = self.maximum.quadratic([self.current, control, to]).err();
                self.current = to;
            }
        }
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.error = Some(Error::InvalidFont);
    }
    fn close(&mut self) {
        if self.error.is_none() && self.current != self.start {
            self.error = self.maximum.line(self.current, self.start).err();
            self.current = self.start;
        }
    }
}

fn cut(
    label: &Label,
    origin: [f32; 2],
    tangent: [f64; 2],
    components: &[Stroke],
    end: usize,
    maximum: &mut Maximum<'_>,
) -> Result<f64, Error> {
    let font = crate::label::font()?;
    let face = Face::parse(font.font_data(), 0).map_err(|_| Error::InvalidFont)?;
    let normal = [-tangent[1], tangent[0]];
    for component in components {
        let point = if end == 0 {
            component.begin
        } else {
            component.end
        };
        let offset = (f64::from(point[0]) - f64::from(origin[0])) * normal[0]
            + (f64::from(point[1]) - f64::from(origin[1])) * normal[1];
        maximum.halfwidth = f64::from(component.width) / 2.0;
        for glyph in &label.glyphs {
            maximum.cancel.check()?;
            let factor = [
                f64::from(glyph.scale.x / font.height_unscaled()),
                f64::from(glyph.scale.y / font.height_unscaled()),
            ];
            let mut pen = Pen {
                maximum,
                glyph,
                factor,
                origin: origin.map(f64::from),
                tangent,
                normal,
                offset,
                current: [0.0; 2],
                start: [0.0; 2],
                error: None,
            };
            face.outline_glyph(ttf_parser::GlyphId(glyph.id.0), &mut pen)
                .ok_or(Error::InvalidFont)?;
            if let Some(error) = pen.error {
                return Err(error);
            }
        }
    }
    Ok(maximum.lower.max(maximum.upper))
}

pub(crate) fn shorten(
    axis: &Stroke,
    components: &mut [Stroke],
    labels: [Option<&Label>; 2],
    margin: f64,
    budget: &mut Budget,
    cancel: &Cancellation,
) -> Result<(), Error> {
    cancel.check()?;
    if !margin.is_finite() || margin < 0.0 {
        return Err(Error::BondScale);
    }
    let delta = [
        f64::from(axis.end[0]) - f64::from(axis.begin[0]),
        f64::from(axis.end[1]) - f64::from(axis.begin[1]),
    ];
    let length = delta[0].hypot(delta[1]);
    let tangent = delta.map(|v| v / length);
    let rounding = components
        .iter()
        .flat_map(|s| s.begin.iter().chain(&s.end))
        .map(|v| f64::from(v.abs()))
        .fold(1.0, f64::max)
        * 4.0
        * f64::from(f32::EPSILON);
    let mut cuts = [0.0; 2];
    for (end, label) in labels.into_iter().enumerate() {
        if let Some(label) = label {
            let origin = if end == 0 { axis.begin } else { axis.end };
            let inward = if end == 0 {
                tangent
            } else {
                tangent.map(|v| -v)
            };
            let mut maximum = Maximum {
                margin,
                halfwidth: 0.0,
                lower: 0.0,
                upper: 0.0,
                limit: length,
                budget,
                cancel,
            };
            let value = cut(label, origin, inward, components, end, &mut maximum)?;
            cuts[end] = if value > 0.0 { value + rounding } else { 0.0 };
        }
    }
    if cuts[0] + cuts[1] >= length {
        return Err(Error::LabelOverlap);
    }
    for component in components {
        cancel.check()?;
        let begin = component.begin.map(f64::from);
        let end = component.end.map(f64::from);
        let start = (begin[0] - f64::from(axis.begin[0])) * tangent[0]
            + (begin[1] - f64::from(axis.begin[1])) * tangent[1];
        let stop = (end[0] - f64::from(axis.begin[0])) * tangent[0]
            + (end[1] - f64::from(axis.begin[1])) * tangent[1];
        let advance = (cuts[0] - start).max(0.0);
        let retreat = (cuts[1] - (length - stop)).max(0.0);
        if advance + retreat >= stop - start {
            return Err(Error::LabelOverlap);
        }
        component.begin = [
            (begin[0] + advance * tangent[0]) as f32,
            (begin[1] + advance * tangent[1]) as f32,
        ];
        component.end = [
            (end[0] - retreat * tangent[0]) as f32,
            (end[1] - retreat * tangent[1]) as f32,
        ];
        component.validate()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rectangle;
    use ab_glyph::point;
    use serde_json::Value;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn analytic(
        points: [[f64; 2]; 3],
        margin: f64,
        halfwidth: f64,
        line: bool,
    ) -> Result<[f64; 2], Error> {
        let mut budget = Budget::default();
        let cancel = Cancellation::default();
        let mut maximum = Maximum {
            margin,
            halfwidth,
            lower: 0.0,
            upper: 0.0,
            limit: 100.0,
            budget: &mut budget,
            cancel: &cancel,
        };
        if line {
            maximum.line(points[0], points[2])?;
        } else {
            maximum.quadratic(points)?;
        }
        Ok([maximum.lower, maximum.upper.max(maximum.lower)])
    }
    #[test]
    fn analytic_strip_circle_and_quadratic_extrema_are_enclosed_without_sampling_the_answer()
    -> TestResult {
        for (points, margin, width, line, expected) in [
            ([[2., 0.], [3.5, 0.], [5., 0.]], 2., 0.5, true, 7.),
            ([[0., 0.], [4., 0.], [0., 0.]], 0., 0.5, false, 2.),
            (
                [[0., 0.5], [0.5, 1.], [1., 1.5]],
                1.,
                0.5,
                true,
                2_f64.sqrt(),
            ),
            ([[0., 4.], [0., 0.], [0., 4.]], 1., 1., false, 0.),
            ([[0., 4.], [3., 4.], [6., 4.]], 1., 0.5, true, 0.),
            ([[2., 0.5], [3.5, 0.5], [5., 0.5]], 0., 0.5, true, 5.),
        ] {
            let [lo, hi] = analytic(points, margin, width, line)?;
            assert!(
                lo <= expected && expected <= hi,
                "{lo} <= {expected} <= {hi}"
            );
            assert!(hi - lo <= CUT_TOLERANCE, "bracket {lo}..{hi}");
        }
        Ok(())
    }
    #[test]
    fn work_exhaustion_and_cancellation_refuse_instead_of_publishing_a_partial_cut() {
        let cancel = Cancellation::default();
        let mut budget = Budget { work: MAX_WORK };
        let mut maximum = Maximum {
            margin: 1.0,
            halfwidth: 0.5,
            lower: 0.0,
            upper: 0.0,
            limit: 100.0,
            budget: &mut budget,
            cancel: &cancel,
        };
        assert!(matches!(
            maximum.quadratic([[0., 0.], [4., 2.], [0., 0.]]),
            Err(Error::SceneTooComplex)
        ));
        cancel.cancel();
        assert!(matches!(
            maximum.line([0., 0.], [1., 0.]),
            Err(Error::Cancelled)
        ));
    }
    fn number(v: &Value, key: &str) -> Result<f64, Box<dyn std::error::Error>> {
        v[key]
            .as_f64()
            .ok_or_else(|| format!("missing {key}").into())
    }
    #[test]
    fn actual_glyph_transforms_and_common_rail_cuts_match_independent_fonttools_intervals()
    -> TestResult {
        let all: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/connected-label-clearance.json"
        ))?;
        assert_eq!(all["schema_version"], 1);
        assert_eq!(
            all["font_sha256"],
            "56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d"
        );
        let font = crate::label::font()?;
        let cases = all["cases"].as_array().ok_or("cases")?;
        assert_eq!(cases.len(), 128);
        for v in cases {
            let scale = number(v, "canvas_scale")?;
            let anchor: [f64; 2] = serde_json::from_value(v["anchor_pt"].clone())?;
            let origin: [f64; 2] = serde_json::from_value(v["canvas_origin"].clone())?;
            let direction: [f64; 2] = serde_json::from_value(v["direction"].clone())?;
            let normal = [-direction[1], direction[0]];
            let begin = [origin[0] + anchor[0] * scale, origin[1] + anchor[1] * scale];
            let axis = Stroke {
                begin: begin.map(|x| x as f32),
                end: [
                    (begin[0] + direction[0] * 80.0 * scale) as f32,
                    (begin[1] + direction[1] * 80.0 * scale) as f32,
                ],
                width: (number(v, "line_width_pt")? * scale) as f32,
                color: [0, 0, 0, 255],
            };
            let mut components = v["rail_offsets_pt"]
                .as_array()
                .ok_or("offsets")?
                .iter()
                .map(|o| {
                    let offset = o.as_f64().ok_or("offset")? * scale;
                    Ok(Stroke {
                        begin: [
                            (f64::from(axis.begin[0]) + normal[0] * offset) as f32,
                            (f64::from(axis.begin[1]) + normal[1] * offset) as f32,
                        ],
                        end: [
                            (f64::from(axis.end[0]) + normal[0] * offset) as f32,
                            (f64::from(axis.end[1]) + normal[1] * offset) as f32,
                        ],
                        ..axis.clone()
                    })
                })
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
            let mut glyphs = Vec::new();
            for g in v["glyphs"].as_array().ok_or("glyphs")? {
                let character = g["character"]
                    .as_str()
                    .ok_or("character")?
                    .chars()
                    .next()
                    .ok_or("character")?;
                let p: [f64; 2] = serde_json::from_value(g["origin_pt"].clone())?;
                let em = number(g, "size_pt")? * scale;
                glyphs.push(font.glyph_id(character).with_scale_and_position(
                    (em / f64::from(font.units_per_em().ok_or("units")?)
                        * f64::from(font.height_unscaled())) as f32,
                    point(
                        (origin[0] + p[0] * scale) as f32,
                        (origin[1] + p[1] * scale) as f32,
                    ),
                ));
            }
            let label = Label::flat(
                12.0,
                Rectangle {
                    x: 1,
                    y: 1,
                    width: 1,
                    height: 1,
                    color: [255; 4],
                },
                vec![],
                glyphs,
            );
            shorten(
                &axis,
                &mut components,
                [Some(&label), None],
                number(v, "margin_pt")? * scale,
                &mut Budget::default(),
                &Cancellation::default(),
            )?;
            let expected: [f64; 2] =
                serde_json::from_value(v["common_cut_canvas_interval"].clone())?;
            let rounding = 8.0
                * f64::from(f32::EPSILON)
                * axis
                    .begin
                    .iter()
                    .chain(&axis.end)
                    .map(|x| f64::from(x.abs()))
                    .fold(1.0, f64::max);
            let mut actuals = Vec::new();
            for component in &components {
                let along = (f64::from(component.begin[0]) - f64::from(axis.begin[0]))
                    * direction[0]
                    + (f64::from(component.begin[1]) - f64::from(axis.begin[1])) * direction[1];
                assert!(
                    along + rounding >= expected[0]
                        && along <= expected[1] + CUT_TOLERANCE + rounding,
                    "{} got {along} expected {expected:?} + rounding {rounding}",
                    v["id"]
                );
                actuals.push(along);
                assert_eq!(component.width, axis.width);
                assert_eq!(component.color, axis.color);
            }
            let min = actuals.iter().copied().fold(f64::INFINITY, f64::min);
            let max = actuals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            assert!(
                max - min <= rounding,
                "{} staggered rails: {actuals:?}",
                v["id"]
            );
        }
        Ok(())
    }
}
