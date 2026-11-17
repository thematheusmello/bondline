use crate::stroke::{PixelRegion, first_lattice_true, visible_pixel};
use crate::{Cancellation, Error, Rectangle, Stroke};
use bondline_model::DoublePosition;

pub(crate) fn strokes_with_unit(
    axis: &Stroke,
    side: DoublePosition,
    begin_neighbours: &[[f32; 2]],
    end_neighbours: &[[f32; 2]],
    cancel: &Cancellation,
    unit: f32,
) -> Result<Vec<Stroke>, Error> {
    cancel.check()?;
    axis.validate()?;
    if !unit.is_finite() || unit <= 0.0 || unit > 100_000.0 {
        return Err(Error::InvalidStroke);
    }
    let sign = match side {
        DoublePosition::Right => 1.0,
        DoublePosition::Left => -1.0,
        _ => return Err(Error::UnsupportedChemicalContent),
    };
    let begin = axis.begin.map(f64::from);
    let end = axis.end.map(f64::from);
    let delta = [end[0] - begin[0], end[1] - begin[1]];
    let length = delta[0].hypot(delta[1]);
    let spacing = 3.0 * f64::from(unit);
    if f64::from(axis.width) >= spacing {
        return Err(Error::BondScale);
    }
    if 3.0 * spacing > length {
        return Err(Error::UnsupportedChemicalContent);
    }
    let tangent = delta.map(|value| value / length);
    let normal = [-tangent[1] * sign, tangent[0] * sign];
    let a = trim(
        axis,
        (axis.begin, tangent),
        normal,
        spacing,
        begin_neighbours,
        cancel,
    )?;
    let b = trim(
        axis,
        (axis.end, tangent.map(|v| -v)),
        normal,
        spacing,
        end_neighbours,
        cancel,
    )?;
    if !a.is_finite() || !b.is_finite() || a + b >= length {
        return Err(Error::UnsupportedChemicalContent);
    }
    let second = Stroke {
        begin: [
            (begin[0] + spacing * normal[0] + a * tangent[0]) as f32,
            (begin[1] + spacing * normal[1] + a * tangent[1]) as f32,
        ],
        end: [
            (end[0] + spacing * normal[0] - b * tangent[0]) as f32,
            (end[1] + spacing * normal[1] - b * tangent[1]) as f32,
        ],
        width: axis.width,
        color: axis.color,
    };
    second.validate()?;
    Ok(vec![axis.clone(), second])
}

fn trim(
    axis: &Stroke,
    (origin, inward): ([f32; 2], [f64; 2]),
    normal: [f64; 2],
    spacing: f64,
    neighbours: &[[f32; 2]],
    cancel: &Cancellation,
) -> Result<f64, Error> {
    let mut result = 0.0_f64;
    for point in neighbours {
        cancel.check()?;
        Stroke {
            begin: origin,
            end: *point,
            ..axis.clone()
        }
        .validate()?;
        let delta = [
            f64::from(point[0]) - f64::from(origin[0]),
            f64::from(point[1]) - f64::from(origin[1]),
        ];
        let length = delta[0].hypot(delta[1]);
        let ray = delta.map(|v| v / length);
        let c = normal[0] * ray[0] + normal[1] * ray[1];
        let d = inward[0] * ray[0] + inward[1] * ray[1];
        if c == 0.0 && d > 0.0 {
            return Err(Error::UnsupportedChemicalContent);
        }
        if c > 0.0 {
            // Equivalent bisector forms keep a near-straight continuation well conditioned.
            let distance = if d >= 0.0 {
                spacing * (1.0 + d) / c
            } else {
                spacing * c / (1.0 - d)
            };
            result = result.max(distance);
        }
    }
    Ok(result)
}

/// Validation only: the true space between two finite butt strokes, never painted.
pub(crate) struct Gap {
    edges: [Edge; 2],
    radius: f64,
}

struct Edge {
    begin: [f64; 2],
    end: [f64; 2],
    delta: [f64; 2],
    length2: f64,
    limit2: f64,
    sign: f64,
}

impl Edge {
    fn new(stroke: &Stroke, sign: f64) -> Self {
        let begin = stroke.begin.map(f64::from);
        let end = stroke.end.map(f64::from);
        let delta = [end[0] - begin[0], end[1] - begin[1]];
        let length2 = delta[0] * delta[0] + delta[1] * delta[1];
        Self {
            begin,
            end,
            delta,
            length2,
            limit2: (f64::from(stroke.width) / 2.0).powi(2) * length2,
            sign,
        }
    }

    fn terms(&self, x: f64, y: f64) -> (f64, f64) {
        let [px, py] = [x - self.begin[0], y - self.begin[1]];
        (
            px * self.delta[0] + py * self.delta[1],
            self.sign * (self.delta[0] * py - self.delta[1] * px),
        )
    }

    fn outside(&self, cross: f64) -> bool {
        cross > 0.0 && cross * cross > self.limit2
    }

    fn contains(&self, x: f64, y: f64) -> bool {
        let (along, cross) = self.terms(x, y);
        (0.0..=self.length2).contains(&along) && self.outside(cross)
    }
}

impl Gap {
    pub(crate) fn new(
        axis: &Stroke,
        secondary: &Stroke,
        side: DoublePosition,
    ) -> Result<Self, Error> {
        axis.validate()?;
        secondary.validate()?;
        let sign = match side {
            DoublePosition::Right => 1.0,
            DoublePosition::Left => -1.0,
            _ => return Err(Error::UnsupportedChemicalContent),
        };
        let edges = [Edge::new(axis, sign), Edge::new(secondary, -sign)];
        if edges[0].delta[0] * edges[1].delta[0] + edges[0].delta[1] * edges[1].delta[1] <= 0.0 {
            return Err(Error::UnsupportedChemicalContent);
        }
        Ok(Self {
            edges,
            radius: f64::from(axis.width.max(secondary.width)) / 2.0,
        })
    }

    pub(crate) fn has_visible_pixel(
        &self,
        dimensions: [u32; 2],
        masks: &[&Rectangle],
        other: Option<&Stroke>,
        excluded: &[&Stroke],
        cancel: &Cancellation,
    ) -> Result<bool, Error> {
        visible_pixel(
            PixelRegion::Gap(self),
            dimensions,
            masks,
            other,
            excluded,
            cancel,
        )
    }

    pub(crate) fn horizontal(&self) -> bool {
        self.edges[0].delta[0].abs() >= self.edges[0].delta[1].abs()
    }

    pub(crate) fn covers(&self, x: f64, y: f64) -> bool {
        self.edges.iter().all(|edge| edge.contains(x, y))
    }

    pub(crate) fn bounds(&self, width: u32, height: u32) -> Option<Rectangle> {
        let horizontal = self.edges.iter().all(|edge| edge.delta[1] == 0.0);
        let vertical = self.edges.iter().all(|edge| edge.delta[0] == 0.0);
        if !horizontal && !vertical {
            // shortcut: use viewport bounds for oblique gaps; tighten cap bounds if profiling warrants it.
            return (width > 0 && height > 0).then_some(Rectangle {
                x: 0,
                y: 0,
                width,
                height,
                color: [255; 4],
            });
        }
        let points = [
            self.edges[0].begin,
            self.edges[0].end,
            self.edges[1].begin,
            self.edges[1].end,
        ];
        let minimum = |axis| points.iter().fold(f64::INFINITY, |v, p| v.min(p[axis]));
        let maximum = |axis| points.iter().fold(f64::NEG_INFINITY, |v, p| v.max(p[axis]));
        let left = (minimum(0) - self.radius)
            .floor()
            .clamp(0.0, f64::from(width)) as u32;
        let top = (minimum(1) - self.radius)
            .floor()
            .clamp(0.0, f64::from(height)) as u32;
        let right = (maximum(0) + self.radius)
            .ceil()
            .clamp(0.0, f64::from(width)) as u32;
        let bottom = (maximum(1) + self.radius)
            .ceil()
            .clamp(0.0, f64::from(height)) as u32;
        (left < right && top < bottom).then_some(Rectangle {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
            color: [255; 4],
        })
    }

    pub(crate) fn pixel_range(
        &self,
        horizontal: bool,
        major: u32,
        range: &mut std::ops::Range<u32>,
    ) -> bool {
        for edge in &self.edges {
            let terms = |minor| {
                if horizontal {
                    edge.terms(f64::from(major) + 0.5, f64::from(minor) + 0.5)
                } else {
                    edge.terms(f64::from(minor) + 0.5, f64::from(major) + 0.5)
                }
            };
            let along_increasing = if horizontal {
                edge.delta[1] >= 0.0
            } else {
                edge.delta[0] >= 0.0
            };
            range.start = first_lattice_true(range.clone(), |i| {
                if along_increasing {
                    terms(i).0 >= 0.0
                } else {
                    terms(i).0 <= edge.length2
                }
            });
            range.end = first_lattice_true(range.clone(), |i| {
                if along_increasing {
                    terms(i).0 > edge.length2
                } else {
                    terms(i).0 < 0.0
                }
            });
            if range.start >= range.end {
                return false;
            }
            let cross_increasing = if horizontal {
                edge.sign * edge.delta[0] >= 0.0
            } else {
                -edge.sign * edge.delta[1] >= 0.0
            };
            if cross_increasing {
                range.start = first_lattice_true(range.clone(), |i| edge.outside(terms(i).1));
            } else {
                range.end = first_lattice_true(range.clone(), |i| !edge.outside(terms(i).1));
            }
            if range.start >= range.end {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod gap_tests;
