//! Original unit-edge projection of an ideal tetrahedral chair; no stereo is assigned.
use super::RingSide;
use crate::Error;
use bondline_model::Point2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ChairKind {
    #[serde(rename = "cyclohexane_chair_1")]
    One,
    #[serde(rename = "cyclohexane_chair_2")]
    Two,
}

impl ChairKind {
    /// Unit first-edge coordinates from the independently specified orthographic view.
    pub fn reference_vertices(self) -> [[f64; 2]; 6] {
        let sign = match self {
            Self::One => 1.0,
            Self::Two => -1.0,
        };
        let h = std::f64::consts::SQRT_2 / 8.0;
        let q = 3.0_f64.sqrt() / 8.0;
        let g = 4.0 / 17.0_f64.sqrt();
        let [a, b, c, d, e, f] = [
            [-0.5, g * (sign * h - q)],
            [0.5, g * (-sign * h - q)],
            [1.0, g * sign * h],
            [0.5, g * (-sign * h + q)],
            [-0.5, g * (sign * h + q)],
            [-1.0, -g * sign * h],
        ];
        let points = match self {
            Self::One => [b, c, d, e, f, a],
            Self::Two => [f, a, b, c, d, e],
        };
        let [[origin_x, origin_y], [end_x, end_y], _, _, _, _] = points;
        let edge_x = end_x - origin_x;
        let edge_y = end_y - origin_y;
        let denominator = (7.0 + 6.0_f64.sqrt()) / 17.0;
        let [_, _, c, d, e, f] = points.map(|[x, y]| {
            let x = x - origin_x;
            let y = y - origin_y;
            [
                (edge_x * x + edge_y * y) / denominator,
                (edge_x * y - edge_y * x) / denominator,
            ]
        });
        [[0.0, 0.0], [1.0, 0.0], c, d, e, f]
    }
}

pub(super) fn points(
    kind: ChairKind,
    begin: Point2,
    end: Point2,
    side: RingSide,
) -> Result<Vec<Point2>, Error> {
    let dx = end.x_pt() - begin.x_pt();
    let dy = end.y_pt() - begin.y_pt();
    let length = dx.hypot(dy);
    let precision =
        || Error::InvalidRing("chair geometry loses its projection at this coordinate precision");
    if !length.is_finite() || length == 0.0 {
        return Err(precision());
    }
    let sign = match side {
        RingSide::Positive => 1.0,
        RingSide::Negative => -1.0,
    };
    let reference = kind.reference_vertices();
    let mut points = vec![begin, end];
    for [u, v] in reference.iter().skip(2) {
        let v = sign * v;
        let point = Point2::new(
            begin.x_pt() + u * dx - v * dy,
            begin.y_pt() + u * dy + v * dx,
        )?;
        if points.contains(&point) {
            return Err(precision());
        }
        points.push(point);
    }
    let axis = (dx / length, dy / length);
    for (point, [u, v]) in points.iter().zip(reference) {
        let x = (point.x_pt() - begin.x_pt()) / length;
        let y = (point.y_pt() - begin.y_pt()) / length;
        let actual_u = x * axis.0 + y * axis.1;
        let actual_v = -x * axis.1 + y * axis.0;
        if !actual_u.is_finite()
            || !actual_v.is_finite()
            || (actual_u - u).abs() > 1e-10
            || (actual_v - sign * v).abs() > 1e-10
        {
            return Err(precision());
        }
    }
    Ok(points)
}
