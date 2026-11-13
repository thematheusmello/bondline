use crate::{Error, Rectangle};

/// Closed, constant-width butt stroke in canvas units, without blending or antialiasing.
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub begin: [f32; 2],
    pub end: [f32; 2],
    pub width: f32,
    pub color: [u8; 4],
}

impl Stroke {
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
