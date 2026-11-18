//! Disposable chain geometry. Authority, input chronology and scene guards live above it.
use bondline_commands::{ChainPreferences, Error};
use bondline_model::{MAX_POSITION_PT, Point2};

const MAX_POINTS: usize = 64;
const TIE: f64 = 1e-12;
const PRECISION: f64 = 1e-10;
type Vector = (f64, f64);

pub(super) struct Acyclic {
    begin: Point2,
    preferences: ChainPreferences,
    phase: f64,
    previous_raw_axis: Option<Vector>,
    points: Vec<Point2>,
}

impl Acyclic {
    pub(super) fn new(begin: Point2, preferences: ChainPreferences) -> Self {
        Self {
            begin,
            preferences,
            phase: -1.0,
            previous_raw_axis: None,
            points: vec![begin],
        }
    }

    pub(super) fn sample(&mut self, end: Point2) -> Result<Vec<Point2>, Error> {
        validate_preferences(self.preferences)?;
        let displacement = difference(self.begin, end);
        let distance = displacement.0.hypot(displacement.1);
        if distance == 0.0 {
            let points = vec![self.begin];
            self.points = points.clone();
            return Ok(points);
        }
        let raw = unit(displacement)?;
        let axis = axis(raw, self.preferences.fixed_angles);
        let extent = dot(displacement, axis);
        if !extent.is_finite() || extent <= 0.0 {
            return Err(invalid("chain extent must be finite and positive"));
        }
        let (a, b) = (self.preferences.angle_degrees.to_radians() / 2.0).sin_cos();
        // Divide separately: multiplying a tiny preferred length by a first can underflow.
        let count_ratio = (extent / self.preferences.bond_length_pt) / a;
        if !count_ratio.is_finite() {
            return Err(invalid("chain count exceeds the supported finite range"));
        }
        let edges = round_half_up(count_ratio).max(1.0);
        if !(1.0..=(MAX_POINTS - 1) as f64).contains(&edges) {
            return Err(invalid(
                "a chain cannot exceed 64 vertices including its anchor",
            ));
        }
        // Finite and bounded before the cast or any count-sized allocation.
        let edges = edges as usize;
        let mut phase = self.phase;
        if self.preferences.fixed_lengths && self.points.len() == edges + 1 {
            let cross = if self.preferences.fixed_angles {
                cross(axis, raw)
            } else {
                self.previous_raw_axis
                    .map_or(0.0, |previous| cross(previous, raw))
            };
            if cross > TIE {
                phase = 1.0;
            } else if cross < -TIE {
                phase = -1.0;
            }
        }
        let (points, length) = if self.preferences.fixed_lengths {
            let normal = (-axis.1, axis.0);
            let length = self.preferences.bond_length_pt;
            let e = (
                length * (a * axis.0 + phase * b * normal.0),
                length * (a * axis.1 + phase * b * normal.1),
            );
            let r = (
                length * (a * axis.0 - phase * b * normal.0),
                length * (a * axis.1 - phase * b * normal.1),
            );
            let mut points = Vec::with_capacity(edges + 1);
            points.push(self.begin);
            for i in 1..=edges {
                let e_count = i.div_ceil(2) as f64;
                let r_count = (i / 2) as f64;
                points.push(offset(
                    self.begin,
                    (e_count * e.0 + r_count * r.0, e_count * e.1 + r_count * r.1),
                )?);
            }
            (points, length)
        } else {
            let terminal = (
                edges as f64 * a,
                if edges.is_multiple_of(2) {
                    0.0
                } else {
                    phase * b
                },
            );
            let terminal_length = terminal.0.hypot(terminal.1);
            let terminal_axis = unit(terminal)?;
            let cosine = dot(terminal_axis, axis);
            let sine = cross(terminal_axis, axis);
            let length = extent / terminal_length;
            if !length.is_finite() || length <= 0.0 {
                return Err(invalid("chain endpoint normalization loses finite length"));
            }
            let mut points = Vec::with_capacity(edges + 1);
            points.push(self.begin);
            for i in 1..=edges {
                if i == edges {
                    // Preserve the actual unsnapped endpoint rather than round-tripping it.
                    points.push(if self.preferences.fixed_angles {
                        offset(self.begin, (extent * axis.0, extent * axis.1))?
                    } else {
                        end
                    });
                } else {
                    let point = (
                        i as f64 * a,
                        if i.is_multiple_of(2) { 0.0 } else { phase * b },
                    );
                    points.push(offset(
                        self.begin,
                        (
                            length * (cosine * point.0 - sine * point.1),
                            length * (sine * point.0 + cosine * point.1),
                        ),
                    )?);
                }
            }
            (points, length)
        };
        certify_fixed(&points, length, self.preferences.angle_degrees, phase)?;
        // Publish all state only after the complete sample succeeds.
        self.phase = phase;
        self.previous_raw_axis = Some(raw);
        self.points = points.clone();
        Ok(points)
    }

    /// The controller validates the chord and resamples its current endpoint afterward.
    pub(super) fn reverse(&mut self) {
        self.phase = -self.phase;
    }

    /// Last successful sample; an Err must remain visibly invalid in the controller.
    pub(super) fn points(&self) -> &[Point2] {
        &self.points
    }
}

pub(super) struct Snaking {
    preferences: ChainPreferences,
    accepted: Vec<Point2>,
    previous_pointer: Point2,
}

impl Snaking {
    pub(super) fn new(begin: Point2, preferences: ChainPreferences) -> Self {
        Self {
            preferences,
            accepted: vec![begin],
            previous_pointer: begin,
        }
    }

    pub(super) fn from_prefix(
        points: Vec<Point2>,
        preferences: ChainPreferences,
        previous_pointer: Point2,
    ) -> Result<Self, Error> {
        validate_preferences(preferences)?;
        validate_path(&points)?;
        Ok(Self {
            preferences,
            accepted: points,
            previous_pointer,
        })
    }

    pub(super) fn sample(&mut self, end: Point2) -> Result<Vec<Point2>, Error> {
        validate_preferences(self.preferences)?;
        let length = self.preferences.bond_length_pt;
        let mut accepted = self.accepted.clone();
        let movement = difference(self.previous_pointer, end);
        // A converted prefix may have different actual edge lengths. Never rescale it.
        // Each pop decreases a <=64-point vector; perform all pops before any append.
        while let Some(penultimate) = accepted.iter().rev().nth(1).copied() {
            let terminal = accepted
                .last()
                .copied()
                .ok_or(invalid("a chain needs an anchor"))?;
            let incoming = unit(difference(penultimate, terminal))?;
            let delta = difference(penultimate, end);
            if dot(movement, incoming) >= 0.0 || delta.0.hypot(delta.1) > length / 2.0 {
                break;
            }
            let _ = accepted.pop();
        }
        loop {
            let Some((heading, projection)) = heading(&accepted, end, self.preferences)? else {
                break;
            };
            if projection < length {
                break;
            }
            if accepted.len() >= MAX_POINTS {
                return Err(invalid(
                    "a chain cannot exceed 64 vertices including its anchor",
                ));
            }
            let terminal = accepted
                .last()
                .copied()
                .ok_or(invalid("a chain needs an anchor"))?;
            let next = step(terminal, heading, length)?;
            if accepted.contains(&next) {
                return Err(invalid(
                    "chain sampling produced a duplicate or collapsed vertex",
                ));
            }
            accepted.push(next);
        }
        let mut points = accepted.clone();
        if !self.preferences.fixed_lengths
            && let Some((heading, projection)) = heading(&accepted, end, self.preferences)?
            && projection > 0.0
            && projection < length
        {
            if points.len() >= MAX_POINTS {
                return Err(invalid(
                    "a disposable tip cannot become a 65th chain vertex",
                ));
            }
            let terminal = points
                .last()
                .copied()
                .ok_or(invalid("a chain needs an anchor"))?;
            points.push(step(terminal, heading, projection)?);
        }
        validate_path(&points)?;
        // The disposable tip never joins the retained prefix merely because it turned.
        self.accepted = accepted;
        self.previous_pointer = end;
        Ok(points)
    }
}

fn heading(
    points: &[Point2],
    end: Point2,
    preferences: ChainPreferences,
) -> Result<Option<(Vector, f64)>, Error> {
    let terminal = points
        .last()
        .copied()
        .ok_or(invalid("a chain needs an anchor"))?;
    let toward = difference(terminal, end);
    if toward.0 == 0.0 && toward.1 == 0.0 {
        return Ok(None);
    }
    let Some(previous) = points.iter().rev().nth(1).copied() else {
        // Snaking follows the raw/snapped pointer direction, not Acyclic's center axis.
        let heading = axis(unit(toward)?, preferences.fixed_angles);
        return projected(heading, toward);
    };
    let incoming = unit(difference(previous, terminal))?;
    let (sine, cosine) = (180.0 - preferences.angle_degrees).to_radians().sin_cos();
    let positive = (
        cosine * incoming.0 - sine * incoming.1,
        sine * incoming.0 + cosine * incoming.1,
    );
    let negative = (
        cosine * incoming.0 + sine * incoming.1,
        -sine * incoming.0 + cosine * incoming.1,
    );
    let positive_dot = dot(positive, toward);
    let negative_dot = dot(negative, toward);
    if !positive_dot.is_finite() || !negative_dot.is_finite() {
        return Err(invalid("chain projection must be finite"));
    }
    let choose_positive = if positive_dot == negative_dot {
        match points.iter().rev().nth(2).copied() {
            Some(earlier) => cross(unit(difference(earlier, previous))?, incoming) <= 0.0,
            None => true,
        }
    } else {
        positive_dot > negative_dot
    };
    projected(if choose_positive { positive } else { negative }, toward)
}

fn projected(heading: Vector, toward: Vector) -> Result<Option<(Vector, f64)>, Error> {
    let projection = dot(heading, toward);
    if !projection.is_finite() {
        Err(invalid("chain projection must be finite"))
    } else if projection > 0.0 {
        Ok(Some((heading, projection)))
    } else {
        Ok(None)
    }
}

fn validate_preferences(preferences: ChainPreferences) -> Result<(), Error> {
    if !preferences.bond_length_pt.is_finite()
        || preferences.bond_length_pt <= 0.0
        || preferences.bond_length_pt > MAX_POSITION_PT
        || !preferences.angle_degrees.is_finite()
        || !(1.0..=179.0).contains(&preferences.angle_degrees)
    {
        return Err(invalid(
            "chain preferences need finite supported length and angle",
        ));
    }
    Ok(())
}

fn validate_path(points: &[Point2]) -> Result<(), Error> {
    if points.is_empty() || points.len() > MAX_POINTS {
        return Err(invalid("a sampled chain needs 1 through 64 vertices"));
    }
    for (i, point) in points.iter().enumerate() {
        if points.iter().take(i).any(|previous| previous == point) {
            return Err(invalid("chain vertices must be distinct"));
        }
    }
    for pair in points.windows(2) {
        let [begin, end] = pair else {
            return Err(invalid("chain edge validation lost its pair"));
        };
        unit(difference(*begin, *end))?;
    }
    Ok(())
}

fn certify_fixed(points: &[Point2], length: f64, angle: f64, phase: f64) -> Result<(), Error> {
    validate_path(points)?;
    if !length.is_finite() || length <= 0.0 {
        return Err(invalid("chain geometry loses its finite length"));
    }
    for pair in points.windows(2) {
        let [begin, end] = pair else {
            return Err(invalid("chain edge validation lost its pair"));
        };
        let delta = difference(*begin, *end);
        let norm = (delta.0 / length).hypot(delta.1 / length);
        if !norm.is_finite() || (norm - 1.0).abs() > PRECISION {
            return Err(invalid(
                "chain geometry loses fidelity at this coordinate precision",
            ));
        }
    }
    for (i, triple) in points.windows(3).enumerate() {
        let [previous, point, next] = triple else {
            return Err(invalid("chain angle validation lost its triple"));
        };
        let before = difference(*point, *previous);
        let after = difference(*point, *next);
        let before = (before.0 / length, before.1 / length);
        let after = (after.0 / length, after.1 / length);
        let determinant = cross(before, after);
        let actual = determinant.abs().atan2(dot(before, after));
        let turn = if i.is_multiple_of(2) { phase } else { -phase };
        if !actual.is_finite()
            || (actual - angle.to_radians()).abs() > PRECISION
            || determinant * turn <= 0.0
        {
            return Err(invalid(
                "chain geometry loses its angle or side at this precision",
            ));
        }
    }
    Ok(())
}

fn axis(raw: Vector, fixed_angles: bool) -> Vector {
    if !fixed_angles {
        return raw;
    }
    let step = std::f64::consts::PI / 12.0;
    let angle = round_half_up(raw.1.atan2(raw.0) / step) * step;
    let (sine, cosine) = angle.sin_cos();
    (cosine, sine)
}

fn round_half_up(value: f64) -> f64 {
    let floor = value.floor();
    if (value - floor - 0.5).abs() <= TIE {
        floor + 1.0
    } else {
        (value + 0.5).floor()
    }
}

fn difference(begin: Point2, end: Point2) -> Vector {
    (end.x_pt() - begin.x_pt(), end.y_pt() - begin.y_pt())
}

fn offset(begin: Point2, delta: Vector) -> Result<Point2, Error> {
    Ok(Point2::new(begin.x_pt() + delta.0, begin.y_pt() + delta.1)?)
}

fn step(begin: Point2, heading: Vector, length: f64) -> Result<Point2, Error> {
    let point = offset(begin, (length * heading.0, length * heading.1))?;
    let actual = difference(begin, point);
    let error = (actual.0 / length - heading.0).hypot(actual.1 / length - heading.1);
    if !error.is_finite() || error > PRECISION {
        return Err(invalid(
            "chain step loses fidelity at this coordinate precision",
        ));
    }
    Ok(point)
}

fn unit(vector: Vector) -> Result<Vector, Error> {
    if !vector.0.is_finite() || !vector.1.is_finite() {
        return Err(invalid(
            "chain edges need finite nonzero represented length",
        ));
    }
    let scale = vector.0.abs().max(vector.1.abs());
    if !scale.is_finite() || scale == 0.0 {
        return Err(invalid(
            "chain edges need finite nonzero represented length",
        ));
    }
    // Scaling first also normalizes representable subnormal vectors without a false
    // unit length caused by rounding hypot(vector) to one subnormal quantum.
    let scaled = (vector.0 / scale, vector.1 / scale);
    let length = scaled.0.hypot(scaled.1);
    Ok((scaled.0 / length, scaled.1 / length))
}

fn dot(left: Vector, right: Vector) -> f64 {
    left.0 * right.0 + left.1 * right.1
}

fn cross(left: Vector, right: Vector) -> f64 {
    left.0 * right.1 - left.1 * right.0
}

fn invalid(message: &'static str) -> Error {
    Error::InvalidChain(message)
}

#[cfg(test)]
#[path = "chain_math_tests.rs"]
mod tests;
