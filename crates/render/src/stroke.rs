use crate::{Cancellation, Error, Rectangle};
use bondline_model::BondOrder;

/// Original centered_multiple_v1 geometry only: s=3w, and s<=one-third axis.
/// Graph construction separately validates viewport pixels, labels and congestion.
pub fn centered_bond_strokes(axis: &Stroke, order: BondOrder) -> Result<Vec<Stroke>, Error> {
    axis.centered_components(match order {
        BondOrder::Single => 1,
        BondOrder::Double => 2,
        BondOrder::Triple => 3,
        _ => return Err(Error::UnsupportedChemicalContent),
    })
}

/// Original 3-unit separation, with ink width independent of the geometric unit.
pub fn centered_bond_strokes_with_unit(
    axis: &Stroke,
    order: BondOrder,
    unit: f32,
) -> Result<Vec<Stroke>, Error> {
    axis.centered_components_with_unit(
        match order {
            BondOrder::Single => 1,
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            _ => return Err(Error::UnsupportedChemicalContent),
        },
        unit,
    )
}

pub(crate) fn centered_bond_strokes_with_spacing(
    axis: &Stroke,
    order: BondOrder,
    unit: f32,
    spacing: f64,
) -> Result<Vec<Stroke>, Error> {
    axis.centered_components_with_spacing(
        match order {
            BondOrder::Single => 1,
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            _ => return Err(Error::UnsupportedChemicalContent),
        },
        unit,
        Some(spacing),
    )
}

/// Closed, constant-width butt stroke in canvas units, without blending or antialiasing.
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub begin: [f32; 2],
    pub end: [f32; 2],
    pub width: f32,
    pub color: [u8; 4],
}

pub(crate) enum PixelRegion<'a> {
    Stroke(&'a Stroke),
    Gap(&'a crate::inset::Gap),
}

impl PixelRegion<'_> {
    fn bounds(&self, width: u32, height: u32) -> Option<Rectangle> {
        match self {
            Self::Stroke(stroke) => stroke.bounds(width, height),
            Self::Gap(gap) => gap.bounds(width, height),
        }
    }
    fn horizontal(&self) -> bool {
        match self {
            Self::Stroke(stroke) => {
                (stroke.end[0] - stroke.begin[0]).abs() >= (stroke.end[1] - stroke.begin[1]).abs()
            }
            Self::Gap(gap) => gap.horizontal(),
        }
    }
    fn pixel_range(&self, horizontal: bool, major: u32, range: &mut std::ops::Range<u32>) -> bool {
        match self {
            Self::Stroke(stroke) => stroke.pixel_range(horizontal, major, range),
            Self::Gap(gap) => gap.pixel_range(horizontal, major, range),
        }
    }
    fn covers(&self, x: f64, y: f64) -> bool {
        match self {
            Self::Stroke(stroke) => stroke.covers(x, y),
            Self::Gap(gap) => gap.covers(x, y),
        }
    }
}

// Both finite convex regions use the same bounded lattice, mask and excluded-ink search.
pub(crate) fn visible_pixel(
    region: PixelRegion<'_>,
    dimensions: [u32; 2],
    masks: &[&Rectangle],
    other: Option<&Stroke>,
    excluded: &[&Stroke],
    cancel: &Cancellation,
) -> Result<bool, Error> {
    cancel.check()?;
    let [width, height] = dimensions;
    let Some(mut bounds) = region.bounds(width, height) else {
        return Ok(false);
    };
    if let Some(other) = other {
        let Some(second) = other.bounds(width, height) else {
            return Ok(false);
        };
        let right = bounds
            .x
            .saturating_add(bounds.width)
            .min(second.x.saturating_add(second.width));
        let bottom = bounds
            .y
            .saturating_add(bounds.height)
            .min(second.y.saturating_add(second.height));
        bounds.x = bounds.x.max(second.x);
        bounds.y = bounds.y.max(second.y);
        if bounds.x >= right || bounds.y >= bottom {
            return Ok(false);
        }
        bounds.width = right - bounds.x;
        bounds.height = bottom - bounds.y;
    }
    let horizontal = region.horizontal();
    let (major_start, major_end, minor_start, minor_end) = if horizontal {
        (
            bounds.x,
            bounds.x + bounds.width,
            bounds.y,
            bounds.y + bounds.height,
        )
    } else {
        (
            bounds.y,
            bounds.y + bounds.height,
            bounds.x,
            bounds.x + bounds.width,
        )
    };
    for major in major_start..major_end {
        cancel.check()?;
        let mut range = minor_start..minor_end;
        if !region.pixel_range(horizontal, major, &mut range)
            || other.is_some_and(|stroke| !stroke.pixel_range(horizontal, major, &mut range))
        {
            continue;
        }
        let mut minor = range.start;
        let end = range.end - 1;
        for _ in 0..2 {
            loop {
                if minor > end || minor >= minor_end {
                    break;
                }
                let next = masks
                    .iter()
                    .filter_map(|mask| {
                        let (a, b, c, d) = if horizontal {
                            (
                                mask.x,
                                mask.x.saturating_add(mask.width),
                                mask.y,
                                mask.y.saturating_add(mask.height),
                            )
                        } else {
                            (
                                mask.y,
                                mask.y.saturating_add(mask.height),
                                mask.x,
                                mask.x.saturating_add(mask.width),
                            )
                        };
                        (major >= a && major < b && minor >= c && minor < d).then_some(d)
                    })
                    .max();
                if let Some(next) = next {
                    minor = next;
                    continue;
                }
                let point = |minor| {
                    if horizontal {
                        (f64::from(major) + 0.5, f64::from(minor) + 0.5)
                    } else {
                        (f64::from(minor) + 0.5, f64::from(major) + 0.5)
                    }
                };
                if let Some(blocker) = excluded.iter().find(|stroke| {
                    let (x, y) = point(minor);
                    stroke.covers(x, y)
                }) {
                    // A validated butt rectangle intersects one row in a contiguous
                    // interval. Start from known covered ink and jump just past it.
                    let (mut low, mut high) = (minor, minor_end);
                    while low + 1 < high {
                        cancel.check()?;
                        let mid = low + (high - low) / 2;
                        let (x, y) = point(mid);
                        if blocker.covers(x, y) {
                            low = mid;
                        } else {
                            high = mid;
                        }
                    }
                    minor = low + 1;
                    continue;
                }
                break;
            }
            if minor > end || minor >= minor_end {
                break;
            }
            let (x, y) = if horizontal {
                (major, minor)
            } else {
                (minor, major)
            };
            let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            if region.covers(px, py) && other.is_none_or(|stroke| stroke.covers(px, py)) {
                return Ok(true);
            }
            minor = minor.saturating_add(1);
        }
    }
    Ok(false)
}

impl Stroke {
    /// Search actual pixel centers without a frame allocation. Masks use the painter's
    /// integer half-open pixels; a second stroke optionally constrains an intersection.
    pub(crate) fn has_visible_pixel(
        &self,
        dimensions: [u32; 2],
        masks: &[&Rectangle],
        other: Option<&Self>,
        excluded: &[&Self],
        cancel: &Cancellation,
    ) -> Result<bool, Error> {
        visible_pixel(
            PixelRegion::Stroke(self),
            dimensions,
            masks,
            other,
            excluded,
            cancel,
        )
    }

    fn pixel_range(&self, horizontal: bool, major: u32, range: &mut std::ops::Range<u32>) -> bool {
        let [ax, ay] = self.begin.map(f64::from);
        let [bx, by] = self.end.map(f64::from);
        let (dx, dy) = (bx - ax, by - ay);
        let length2 = dx * dx + dy * dy;
        let cross_limit = (f64::from(self.width) / 2.0).powi(2) * length2;
        let terms = |minor: u32| {
            let (x, y) = if horizontal {
                (f64::from(major) + 0.5, f64::from(minor) + 0.5)
            } else {
                (f64::from(minor) + 0.5, f64::from(major) + 0.5)
            };
            let (px, py) = (x - ax, y - ay);
            (px * dx + py * dy, px * dy - py * dx)
        };
        // The four constraints are monotone on one integer row. Use exactly the
        // frozen covers arithmetic, including rounded equality at finite caps.
        let along_increasing = if horizontal { dy >= 0.0 } else { dx >= 0.0 };
        range.start = first_lattice_true(range.clone(), |i| {
            if along_increasing {
                terms(i).0 >= 0.0
            } else {
                terms(i).0 <= length2
            }
        });
        range.end = first_lattice_true(range.clone(), |i| {
            if along_increasing {
                terms(i).0 > length2
            } else {
                terms(i).0 < 0.0
            }
        });
        if range.start >= range.end {
            return false;
        }
        let cross_increasing = if horizontal { dx <= 0.0 } else { dy >= 0.0 };
        let low = |i| {
            let c = terms(i).1;
            c < 0.0 && c * c > cross_limit
        };
        let high = |i| {
            let c = terms(i).1;
            c > 0.0 && c * c > cross_limit
        };
        range.start = first_lattice_true(range.clone(), |i| {
            if cross_increasing { !low(i) } else { !high(i) }
        });
        range.end = first_lattice_true(range.clone(), |i| {
            if cross_increasing { high(i) } else { low(i) }
        });
        range.start < range.end
    }

    pub(crate) fn centered_components(&self, count: usize) -> Result<Vec<Self>, Error> {
        self.centered_components_with_unit(count, self.width)
    }

    fn centered_components_with_unit(&self, count: usize, unit: f32) -> Result<Vec<Self>, Error> {
        self.centered_components_with_spacing(count, unit, None)
    }

    fn centered_components_with_spacing(
        &self,
        count: usize,
        unit: f32,
        spacing: Option<f64>,
    ) -> Result<Vec<Self>, Error> {
        self.validate()?;
        if !unit.is_finite() || unit <= 0.0 || unit > 100_000.0 {
            return Err(Error::InvalidStroke);
        }
        let offsets: &[f64] = match (count, spacing.is_some()) {
            (1, _) => return Ok(vec![self.clone()]),
            (2, false) => &[-1.5, 1.5],
            (3, false) => &[-3.0, 0.0, 3.0],
            (2, true) => &[-0.5, 0.5],
            (3, true) => &[-1.0, 0.0, 1.0],
            _ => return Err(Error::UnsupportedChemicalContent),
        };
        let [ax, ay] = self.begin.map(f64::from);
        let [bx, by] = self.end.map(f64::from);
        let (dx, dy) = (bx - ax, by - ay);
        let length = dx.hypot(dy);
        let separation = spacing.unwrap_or(3.0 * f64::from(unit));
        if (spacing.is_some()
            && (!separation.is_finite() || separation <= 0.0 || separation > 100_000.0))
            || f64::from(self.width) >= separation
        {
            return Err(Error::BondScale);
        }
        // Original centered profile: adjacent spacing3w must not exceed one-third axis.
        if spacing.is_none() && 9.0 * f64::from(unit) > length {
            return Err(Error::UnsupportedChemicalContent);
        }
        offsets
            .iter()
            .map(|factor| {
                let distance = factor * spacing.unwrap_or(f64::from(unit));
                let shift = [-dy / length * distance, dx / length * distance];
                let stroke = Self {
                    begin: [(ax + shift[0]) as f32, (ay + shift[1]) as f32],
                    end: [(bx + shift[0]) as f32, (by + shift[1]) as f32],
                    width: self.width,
                    color: self.color,
                };
                stroke.validate()?;
                Ok(stroke)
            })
            .collect()
    }

    pub(crate) fn validate(&self) -> Result<(), Error> {
        let [ax, ay] = self.begin;
        let [bx, by] = self.end;
        if self
            .begin
            .iter()
            .chain(&self.end)
            .any(|value| !value.is_finite() || value.abs() > 100_000.0)
            || !self.width.is_finite()
            || self.width <= 0.0
            || self.width > 100_000.0
            || !((bx - ax).powi(2) + (by - ay).powi(2)).is_normal()
        {
            return Err(Error::InvalidStroke);
        }
        Ok(())
    }

    pub(crate) fn bounds(&self, width: u32, height: u32) -> Option<Rectangle> {
        let [ax, ay] = self.begin.map(f64::from);
        let [bx, by] = self.end.map(f64::from);
        let radius = f64::from(self.width) / 2.0;
        let left = (ax.min(bx) - radius).floor().clamp(0.0, f64::from(width)) as u32;
        let top = (ay.min(by) - radius).floor().clamp(0.0, f64::from(height)) as u32;
        let right = (ax.max(bx) + radius).ceil().clamp(0.0, f64::from(width)) as u32;
        let bottom = (ay.max(by) + radius).ceil().clamp(0.0, f64::from(height)) as u32;
        (left < right && top < bottom).then_some(Rectangle {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
            color: self.color,
        })
    }

    pub(crate) fn covers(&self, x: f64, y: f64) -> bool {
        let [ax, ay] = self.begin.map(f64::from);
        let [bx, by] = self.end.map(f64::from);
        let (dx, dy) = (bx - ax, by - ay);
        let (px, py) = (x - ax, y - ay);
        let length2 = dx * dx + dy * dy;
        let along = px * dx + py * dy;
        let cross = px * dy - py * dx;
        (0.0..=length2).contains(&along)
            && cross * cross <= (f64::from(self.width) / 2.0).powi(2) * length2
    }

    /// Butt-cap outline for native vector painting; coordinates remain in canvas units.
    pub fn corners(&self) -> Result<[[f32; 2]; 4], Error> {
        self.validate()?;
        let [ax, ay] = self.begin.map(f64::from);
        let [bx, by] = self.end.map(f64::from);
        let (dx, dy) = (bx - ax, by - ay);
        let length = dx.hypot(dy);
        let nx = -dy / length * f64::from(self.width) / 2.0;
        let ny = dx / length * f64::from(self.width) / 2.0;
        Ok([
            [ax - nx, ay - ny],
            [bx - nx, by - ny],
            [bx + nx, by + ny],
            [ax + nx, ay + ny],
        ]
        .map(|point| point.map(|value| value as f32)))
    }
}

pub(crate) fn first_lattice_true(
    range: std::ops::Range<u32>,
    mut predicate: impl FnMut(u32) -> bool,
) -> u32 {
    let (mut low, mut high) = (range.start, range.end);
    while low < high {
        let middle = low + (high - low) / 2;
        if predicate(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

#[cfg(test)]
mod multiple_tests {
    use super::*;

    #[test]
    fn declared_48_64_axis_has_literal_normal_offsets_and_reversal() -> Result<(), Error> {
        let axis = Stroke {
            begin: [20.0, 20.0],
            end: [68.0, 84.0],
            width: 2.0,
            color: [1, 2, 3, 4],
        };
        let cases = [
            (
                2,
                vec![([22.4, 18.2], [70.4, 82.2]), ([17.6, 21.8], [65.6, 85.8])],
            ),
            (
                3,
                vec![
                    ([24.8, 16.4], [72.8, 80.4]),
                    ([20.0, 20.0], [68.0, 84.0]),
                    ([15.2, 23.6], [63.2, 87.6]),
                ],
            ),
        ];
        for (count, expected) in cases {
            let components = axis.centered_components(count)?;
            for (actual, (begin, end)) in components.iter().zip(expected) {
                for (a, b) in actual
                    .begin
                    .iter()
                    .chain(&actual.end)
                    .zip(begin.iter().chain(&end))
                {
                    assert!((f64::from(*a) - *b).abs() <= 1e-5);
                }
                assert_eq!(actual.width, 2.0);
                assert_eq!(actual.color, [1, 2, 3, 4]);
            }
            let reverse = Stroke {
                begin: axis.end,
                end: axis.begin,
                ..axis.clone()
            }
            .centered_components(count)?;
            for (a, b) in reverse.iter().zip(components.iter().rev()) {
                assert_eq!(a.begin, b.end);
                assert_eq!(a.end, b.begin);
            }
        }
        assert_eq!(axis.centered_components(1)?, vec![axis.clone()]);
        assert!(matches!(
            Stroke {
                end: axis.begin,
                ..axis
            }
            .centered_components(2),
            Err(Error::InvalidStroke)
        ));
        Ok(())
    }

    #[test]
    fn bounded_interval_search_matches_exhaustive_frozen_covers_with_masks_and_intersections()
    -> Result<(), Error> {
        let cancel = Cancellation::default();
        let first = Rectangle {
            x: 8,
            y: 9,
            width: 17,
            height: 20,
            color: [255; 4],
        };
        let second = Rectangle {
            x: 20,
            y: 25,
            width: 15,
            height: 15,
            color: [255; 4],
        };
        let crossing = Stroke {
            begin: [22.5, 1.0],
            end: [22.5, 60.0],
            width: 0.95,
            color: [0; 4],
        };
        for width in [0.2, 0.5, 0.75, 0.95, 1.0, 2.0, 7.0, 20.0] {
            for (dx, dy) in [
                (1.0, 0.0),
                (0.0, 1.0),
                (3.0, 4.0),
                (4.0, 3.0),
                (-3.0, 4.0),
                (-4.0, -3.0),
                (0.0001, 1.0),
            ] {
                for phase in [0.0, 0.125, 0.5, 0.95] {
                    let stroke = Stroke {
                        begin: [15.0 + phase, 14.0 + phase],
                        end: [15.0 + phase + dx * 7.0, 14.0 + phase + dy * 7.0],
                        width,
                        color: [0; 4],
                    };
                    stroke.validate()?;
                    for masks in [vec![], vec![&first], vec![&first, &second]] {
                        for other in [None, Some(&crossing)] {
                            let expected = (0..64).any(|y| {
                                (0..64).any(|x| {
                                    let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
                                    stroke.covers(px, py)
                                        && other.is_none_or(|s| s.covers(px, py))
                                        && !masks.iter().any(|m| {
                                            x >= m.x
                                                && x < m.x + m.width
                                                && y >= m.y
                                                && y < m.y + m.height
                                        })
                                })
                            });
                            let actual =
                                stroke.has_visible_pixel([64, 64], &masks, other, &[], &cancel)?;
                            assert_eq!(
                                actual,
                                expected,
                                "width{width} direction{dx}/{dy} phase{phase} masks{} intersect{}",
                                masks.len(),
                                other.is_some()
                            );
                        }
                    }
                }
            }
        }
        cancel.cancel();
        assert!(matches!(
            crossing.has_visible_pixel([64, 64], &[], None, &[], &cancel),
            Err(Error::Cancelled)
        ));
        Ok(())
    }

    #[test]
    fn actual_gap_and_anchor_exclusions_match_exhaustive_frozen_covers() -> Result<(), Error> {
        let cancel = Cancellation::default();
        let gap = Stroke {
            begin: [10.0, 22.0],
            end: [30.0, 22.0],
            width: 8.0,
            color: [255; 4],
        };
        let blocker = Stroke {
            begin: [8.0, 19.0],
            end: [32.0, 19.0],
            width: 4.0,
            color: [0; 4],
        };
        let expected = (0..48).any(|y| {
            (0..48).any(|x| {
                gap.covers(f64::from(x) + 0.5, f64::from(y) + 0.5)
                    && !blocker.covers(f64::from(x) + 0.5, f64::from(y) + 0.5)
            })
        });
        let actual = gap.has_visible_pixel([48, 48], &[], None, &[&blocker], &cancel)?;
        assert_eq!(
            actual, expected,
            "first two gap candidates are blocked; later white pixels must still be found"
        );
        for width in [0.2, 0.5, 0.75, 0.95, 2.0, 8.0, 20.0] {
            for phase in [0.0, 0.125, 0.5, 0.95] {
                for (dx, dy) in [(30.0, 0.0), (0.0, 30.0), (24.0, 32.0), (-24.0, 32.0)] {
                    let stroke = Stroke {
                        begin: [15.0 + phase, 14.0 + phase],
                        end: [15.0 + phase + dx, 14.0 + phase + dy],
                        width,
                        color: [0; 4],
                    };
                    let anchor = if phase == 0.0 {
                        Rectangle {
                            x: 14,
                            y: 13,
                            width: 2,
                            height: 2,
                            color: [255; 4],
                        }
                    } else {
                        Rectangle {
                            x: 15,
                            y: 14,
                            width: 1,
                            height: 1,
                            color: [255; 4],
                        }
                    };
                    let horizontal = Stroke {
                        begin: [0.0, 14.0 + phase],
                        end: [63.0, 14.0 + phase],
                        width: 4.0,
                        color: [0; 4],
                    };
                    let vertical = Stroke {
                        begin: [15.0 + phase, 0.0],
                        end: [15.0 + phase, 63.0],
                        width: 0.95,
                        color: [0; 4],
                    };
                    for anchored in [false, true] {
                        let masks = if anchored { vec![&anchor] } else { vec![] };
                        for excluded in [
                            vec![&horizontal],
                            vec![&vertical],
                            vec![&horizontal, &vertical],
                        ] {
                            for other in [None, Some(&blocker)] {
                                let expected = (0..64).any(|y| {
                                    (0..64).any(|x| {
                                        let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
                                        stroke.covers(px, py)
                                            && other.is_none_or(|s| s.covers(px, py))
                                            && !excluded.iter().any(|s| s.covers(px, py))
                                            && (!anchored
                                                || (px - f64::from(stroke.begin[0])).abs() > 0.5
                                                || (py - f64::from(stroke.begin[1])).abs() > 0.5)
                                    })
                                });
                                let actual = stroke.has_visible_pixel(
                                    [64, 64],
                                    &masks,
                                    other,
                                    &excluded,
                                    &cancel,
                                )?;
                                assert_eq!(
                                    actual,
                                    expected,
                                    "width{width} phase{phase} delta{dx}/{dy} anchored{anchored} excluded{} intersection{}",
                                    excluded.len(),
                                    other.is_some()
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn near_horizontal_cap_rounding_uses_the_frozen_pixel_predicate() -> Result<(), Error> {
        let stroke = Stroke {
            begin: [0.5, 0.0],
            end: [63.5, 1e-14],
            width: 2.0,
            color: [0; 4],
        };
        stroke.validate()?;
        let mask = Rectangle {
            x: 0,
            y: 0,
            width: 63,
            height: 64,
            color: [255; 4],
        };
        assert!(stroke.covers(63.5, 0.5));
        let actual =
            stroke.has_visible_pixel([64, 64], &[&mask], None, &[], &Cancellation::default())?;
        assert!(
            actual,
            "the only unmasked column has a covered cap pixel despite rounded along-product equality"
        );
        Ok(())
    }
}
