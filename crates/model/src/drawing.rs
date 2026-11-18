use crate::{Error, MAX_POSITION_PT};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor, value::MapAccessDeserializer},
};

/// Known document-root creation preferences; absence never supplies a numeric default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct DrawingSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    bond_length_pt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain_angle_degrees: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDrawingSettings {
    bond_length_pt: Option<f64>,
    chain_angle_degrees: Option<f64>,
}

impl DrawingSettings {
    pub fn new(
        bond_length_pt: Option<f64>,
        chain_angle_degrees: Option<f64>,
    ) -> Result<Self, Error> {
        if bond_length_pt.is_some_and(|v| !v.is_finite() || v <= 0.0 || v > MAX_POSITION_PT)
            || chain_angle_degrees.is_some_and(|v| !(1.0..=179.0).contains(&v))
        {
            return Err(Error::InvalidDrawingSettings);
        }
        Ok(Self {
            bond_length_pt,
            chain_angle_degrees,
        })
    }

    pub const fn bond_length_pt(self) -> Option<f64> {
        self.bond_length_pt
    }
    pub const fn chain_angle_degrees(self) -> Option<f64> {
        self.chain_angle_degrees
    }
    pub const fn is_empty(&self) -> bool {
        self.bond_length_pt.is_none() && self.chain_angle_degrees.is_none()
    }
}

impl<'de> Deserialize<'de> for DrawingSettings {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SettingsVisitor;
        impl<'de> Visitor<'de> for SettingsVisitor {
            type Value = DrawingSettings;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a drawing settings object")
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                let raw = RawDrawingSettings::deserialize(MapAccessDeserializer::new(map))?;
                DrawingSettings::new(raw.bond_length_pt, raw.chain_angle_degrees)
                    .map_err(A::Error::custom)
            }
        }
        deserializer.deserialize_map(SettingsVisitor)
    }
}
