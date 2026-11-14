use crate::{Error, Rectangle, Scene, Stroke};
use bondline_model::{AtomId, BondId, FragmentId};

pub const MAX_HIT_REGIONS: usize = 4096;

/// Application identities, never primitive indices or source-format identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HitTarget {
    Atom {
        fragment_id: FragmentId,
        atom_id: AtomId,
    },
    Bond {
        fragment_id: FragmentId,
        bond_id: BondId,
    },
}

pub(crate) struct AtomHit {
    pub target: HitTarget,
    pub position: [f32; 2],
    pub label: Option<Rectangle>,
    pub supplement: Option<Rectangle>,
}

pub(crate) struct BondHit {
    pub target: HitTarget,
    pub endpoints: [AtomId; 2],
    pub stroke: Stroke,
    pub polygon: Vec<[f64; 2]>,
}

/// Paint and picking share an immutable, bounded projection of the same Page.
pub struct GraphScene {
    pub(crate) label_em_px: Option<f32>,
    pub(crate) scene: Scene,
    pub(crate) atoms: Vec<AtomHit>,
    pub(crate) bonds: Vec<BondHit>,
}

impl GraphScene {
    /// Smallest main atom-label em in this actual scene, excluding subscript scaling.
    pub fn label_em_px(&self) -> Option<f32> {
        self.label_em_px
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }
    pub fn into_scene(self) -> Scene {
        self.scene
    }

    /// Five logical canvas units is the initial native policy. The caller chooses
    /// tolerance in the same units as this scene, not document points or device DPI.
    pub fn hit_test(&self, point: [f32; 2], tolerance: f32) -> Result<Option<HitTarget>, Error> {
        if point.iter().any(|v| !v.is_finite())
            || !tolerance.is_finite()
            || !(0.0..=100_000.0).contains(&tolerance)
        {
            return Err(Error::InvalidHitTest);
        }
        let [x, y] = point;
        if x < 0.0 || y < 0.0 || x >= self.scene.width as f32 || y >= self.scene.height as f32 {
            return Ok(None);
        }
        let p = point.map(f64::from);
        let radius2 = f64::from(tolerance).powi(2);
        let mut best: Option<(f64, f64, HitTarget)> = None;
        for atom in &self.atoms {
            let anchor = distance2(p, atom.position.map(f64::from));
            let visible = atom.position[0] >= 0.0
                && atom.position[1] >= 0.0
                && atom.position[0] < self.scene.width as f32
                && atom.position[1] < self.scene.height as f32;
            if !visible && atom.label.is_none() && atom.supplement.is_none() {
                continue;
            }
            let distance = atom
                .label
                .iter()
                .chain(&atom.supplement)
                .fold(anchor, |distance, r| distance.min(rect_distance2(p, r)));
            consider(&mut best, (distance, anchor, atom.target), radius2);
        }
        if let Some((_, _, target)) = best {
            return Ok(Some(target));
        }
        for bond in &self.bonds {
            if let Some(distance) = polygon_distance2(p, &bond.polygon) {
                consider(
                    &mut best,
                    (
                        distance,
                        segment_distance2(
                            p,
                            bond.stroke.begin.map(f64::from),
                            bond.stroke.end.map(f64::from),
                        ),
                        bond.target,
                    ),
                    radius2,
                );
            }
        }
        Ok(best.map(|(_, _, target)| target))
    }
}

fn consider(
    best: &mut Option<(f64, f64, HitTarget)>,
    candidate: (f64, f64, HitTarget),
    radius2: f64,
) {
    if candidate.0 <= radius2 && best.as_ref().is_none_or(|old| candidate < *old) {
        *best = Some(candidate);
    }
}

fn distance2(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}
fn segment_distance2(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let length2 = d[0] * d[0] + d[1] * d[1];
    if length2 == 0.0 {
        return distance2(p, a);
    }
    let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / length2).clamp(0.0, 1.0);
    distance2(p, [a[0] + t * d[0], a[1] + t * d[1]])
}
fn rect_distance2(p: [f64; 2], r: &Rectangle) -> f64 {
    let x = f64::from(r.x);
    let y = f64::from(r.y);
    distance2(
        p,
        [
            p[0].clamp(x, x + f64::from(r.width)),
            p[1].clamp(y, y + f64::from(r.height)),
        ],
    )
}

pub(crate) fn visible_polygon(stroke: &Stroke, width: u32, height: u32) -> Vec<[f64; 2]> {
    let [ax, ay] = stroke.begin.map(f64::from);
    let [bx, by] = stroke.end.map(f64::from);
    let dx = bx - ax;
    let dy = by - ay;
    let length = dx.hypot(dy);
    let nx = -dy / length * f64::from(stroke.width) / 2.0;
    let ny = dx / length * f64::from(stroke.width) / 2.0;
    let polygon = clip_polygon(
        &[
            [ax - nx, ay - ny],
            [bx - nx, by - ny],
            [bx + nx, by + ny],
            [ax + nx, ay + ny],
        ],
        [0.0, 0.0, f64::from(width), f64::from(height)],
    );
    if area2(&polygon) == 0.0 {
        Vec::new()
    } else {
        polygon
    }
}

pub(crate) fn intersects_label(bond: &BondHit, label: &Rectangle) -> bool {
    area2(&clip_polygon(
        &bond.polygon,
        [
            f64::from(label.x),
            f64::from(label.y),
            f64::from(label.x + label.width),
            f64::from(label.y + label.height),
        ],
    )) != 0.0
}

// Convex clipping needs at most eight vertices for a rectangle against a viewport.
fn clip_polygon(points: &[[f64; 2]], bounds: [f64; 4]) -> Vec<[f64; 2]> {
    let mut output = points.to_vec();
    for (horizontal, bound, greater) in [
        (true, bounds[0], true),
        (true, bounds[2], false),
        (false, bounds[1], true),
        (false, bounds[3], false),
    ] {
        let input = std::mem::take(&mut output);
        let Some(mut previous) = input.last().copied() else {
            break;
        };
        let coordinate = |p: [f64; 2]| if horizontal { p[0] } else { p[1] };
        let inside = |p: [f64; 2]| {
            if greater {
                coordinate(p) >= bound
            } else {
                coordinate(p) <= bound
            }
        };
        for current in input.iter().copied() {
            if inside(previous) != inside(current) {
                let t =
                    (bound - coordinate(previous)) / (coordinate(current) - coordinate(previous));
                let mut intersection = [
                    previous[0] + t * (current[0] - previous[0]),
                    previous[1] + t * (current[1] - previous[1]),
                ];
                if horizontal {
                    intersection[0] = bound;
                } else {
                    intersection[1] = bound;
                }
                output.push(intersection);
            }
            if inside(current) {
                output.push(current);
            }
            previous = current;
        }
    }
    output
}

fn area2(polygon: &[[f64; 2]]) -> f64 {
    let Some(origin) = polygon.first() else {
        return 0.0;
    };
    let Some(mut previous) = polygon.last().copied() else {
        return 0.0;
    };
    let mut area = 0.0;
    for current in polygon.iter().copied() {
        area += (previous[0] - origin[0]) * (current[1] - origin[1])
            - (current[0] - origin[0]) * (previous[1] - origin[1]);
        previous = current;
    }
    area
}

fn polygon_distance2(p: [f64; 2], polygon: &[[f64; 2]]) -> Option<f64> {
    let mut previous = *polygon.last()?;
    let mut positive = false;
    let mut negative = false;
    let mut distance = f64::INFINITY;
    for current in polygon.iter().copied() {
        let cross = (current[0] - previous[0]) * (p[1] - previous[1])
            - (current[1] - previous[1]) * (p[0] - previous[0]);
        positive |= cross > 0.0;
        negative |= cross < 0.0;
        distance = distance.min(segment_distance2(p, previous, current));
        previous = current;
    }
    Some(if area2(polygon) != 0.0 && !(positive && negative) {
        0.0
    } else {
        distance
    })
}
