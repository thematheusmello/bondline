use crate::{Error, MAX_POSITION_PT};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor, value::MapAccessDeserializer},
};

/// Authored root appearance. Absence is distinct from an explicit width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct DrawingStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    line_width_pt: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStyle {
    line_width_pt: Option<f64>,
}

impl DrawingStyle {
    pub fn new(line_width_pt: Option<f64>) -> Result<Self, Error> {
        if line_width_pt.is_some_and(|v| !v.is_finite() || v <= 0.0 || v > MAX_POSITION_PT) {
            return Err(Error::InvalidDrawingStyle);
        }
        Ok(Self { line_width_pt })
    }

    pub const fn line_width_pt(self) -> Option<f64> {
        self.line_width_pt
    }
    pub const fn is_empty(&self) -> bool {
        self.line_width_pt.is_none()
    }
}

impl<'de> Deserialize<'de> for DrawingStyle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StyleVisitor;
        impl<'de> Visitor<'de> for StyleVisitor {
            type Value = DrawingStyle;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a drawing style object")
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                let raw = RawStyle::deserialize(MapAccessDeserializer::new(map))?;
                DrawingStyle::new(raw.line_width_pt).map_err(A::Error::custom)
            }
        }
        deserializer.deserialize_map(StyleVisitor)
    }
}
