use crate::{Error, MAX_POSITION_PT};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor, value::MapAccessDeserializer},
};

/// Authored root appearance. Absence is distinct from each explicit property.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct DrawingStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    line_width_pt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bond_spacing_percent: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStyle {
    line_width_pt: Option<f64>,
    bond_spacing_percent: Option<f64>,
}

impl DrawingStyle {
    pub fn new(line_width_pt: Option<f64>) -> Result<Self, Error> {
        if line_width_pt.is_some_and(|v| !v.is_finite() || v <= 0.0 || v > MAX_POSITION_PT) {
            return Err(Error::InvalidDrawingStyle);
        }
        Ok(Self {
            line_width_pt,
            bond_spacing_percent: None,
        })
    }

    pub const fn line_width_pt(self) -> Option<f64> {
        self.line_width_pt
    }
    /// Update width without discarding independently authored spacing.
    pub fn with_line_width(mut self, line_width_pt: Option<f64>) -> Result<Self, Error> {
        self.line_width_pt = Self::new(line_width_pt)?.line_width_pt;
        Ok(self)
    }
    /// Explicit relative spacing; absence supplies no style-sheet default.
    pub const fn bond_spacing_percent(self) -> Option<f64> {
        self.bond_spacing_percent
    }
    /// Initial nonnegative INT16/10 interpretation policy, not general XML admission.
    pub fn with_bond_spacing_percent(mut self, value: Option<f64>) -> Result<Self, Error> {
        if value.is_some_and(|v| !v.is_finite() || !(0.0..=3276.7).contains(&v)) {
            return Err(Error::InvalidDrawingStyle);
        }
        self.bond_spacing_percent = value;
        Ok(self)
    }
    pub const fn is_empty(&self) -> bool {
        self.line_width_pt.is_none() && self.bond_spacing_percent.is_none()
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
                DrawingStyle::new(raw.line_width_pt)
                    .and_then(|style| style.with_bond_spacing_percent(raw.bond_spacing_percent))
                    .map_err(A::Error::custom)
            }
        }
        deserializer.deserialize_map(StyleVisitor)
    }
}
