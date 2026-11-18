use bondline_model::{DrawingSettings, Error};

/// Unsupported values remain in complete source backing, never in an invented default.
pub(crate) fn project(length: Option<f64>, angle: Option<f64>) -> Result<DrawingSettings, Error> {
    DrawingSettings::new(
        length.filter(|value| DrawingSettings::new(Some(*value), None).is_ok()),
        angle.filter(|value| DrawingSettings::new(None, Some(*value)).is_ok()),
    )
}
