use crate::Error;
use serde::{Deserialize, Serialize};

pub const MAX_POSITION_PT: f64 = 100_000.0;

/// Physical points: x increases right, y down. Missing positions are represented separately.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawPoint2")]
pub struct Point2 {
    x_pt: f64,
    y_pt: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPoint2 {
    x_pt: f64,
    y_pt: f64,
}

impl TryFrom<RawPoint2> for Point2 {
    type Error = Error;
    fn try_from(raw: RawPoint2) -> Result<Self, Error> {
        Self::new(raw.x_pt, raw.y_pt)
    }
}

impl Point2 {
    pub fn new(x_pt: f64, y_pt: f64) -> Result<Self, Error> {
        // shortcut: coordinates are bounded to ±100000 pt; revisit for larger authored spaces.
        if [x_pt, y_pt]
            .iter()
            .any(|value| !value.is_finite() || value.abs() > MAX_POSITION_PT)
        {
            return Err(Error::InvalidPosition);
        }
        Ok(Self { x_pt, y_pt })
    }

    pub const fn x_pt(self) -> f64 {
        self.x_pt
    }
    pub const fn y_pt(self) -> f64 {
        self.y_pt
    }
}
