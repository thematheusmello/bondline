//! Explicit cycle geometry for a new ring joined to an existing atom by one Single.
use super::{ChairKind, Pattern, RingKind, RingPhase, RingSide};
use crate::OrderHydrogenPolicy;
use schemars::JsonSchema;
use serde::de::value::StringDeserializer;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum SproutRing {
    Polygon {
        #[schemars(range(min = 3, max = 8))]
        size: u8,
    },
    Benzene {
        #[serde(
            default,
            deserialize_with = "super::request::present_phase",
            skip_serializing_if = "Option::is_none"
        )]
        #[schemars(with = "RingPhase")]
        phase: Option<RingPhase>,
    },
    Cyclopentadiene {
        #[serde(
            default,
            deserialize_with = "super::request::present_phase",
            skip_serializing_if = "Option::is_none"
        )]
        #[schemars(with = "RingPhase")]
        phase: Option<RingPhase>,
    },
    #[serde(rename = "cyclohexane_chair_1")]
    CyclohexaneChair1 {},
    #[serde(rename = "cyclohexane_chair_2")]
    CyclohexaneChair2 {},
}

impl SproutRing {
    pub const fn size(&self) -> u8 {
        match self {
            Self::Polygon { size } => *size,
            Self::Cyclopentadiene { .. } => 5,
            _ => 6,
        }
    }

    pub(super) const fn pattern(&self) -> Pattern {
        match self {
            Self::Polygon { .. } => Pattern::Regular,
            Self::Benzene { phase } => Pattern::Named(RingKind::Benzene, *phase),
            Self::Cyclopentadiene { phase } => Pattern::Named(RingKind::Cyclopentadiene, *phase),
            Self::CyclohexaneChair1 {} => Pattern::Chair(ChairKind::One),
            Self::CyclohexaneChair2 {} => Pattern::Chair(ChairKind::Two),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RingSprout {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 1))]
    pub fragment_id: u32,
    #[schemars(range(min = 1))]
    pub atom_id: u32,
    #[serde(deserialize_with = "string_enum")]
    #[schemars(with = "OrderHydrogenPolicy")]
    pub hydrogen_policy: OrderHydrogenPolicy,
    #[serde(deserialize_with = "ring_object")]
    pub ring: SproutRing,
    #[schemars(range(min = -100000, max = 100000))]
    pub ring_begin_x_pt: f64,
    #[schemars(range(min = -100000, max = 100000))]
    pub ring_begin_y_pt: f64,
    #[schemars(range(min = -100000, max = 100000))]
    pub ring_end_x_pt: f64,
    #[schemars(range(min = -100000, max = 100000))]
    pub ring_end_y_pt: f64,
    #[serde(deserialize_with = "string_enum")]
    #[schemars(with = "RingSide")]
    pub side: RingSide,
}

fn ring_object<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SproutRing, D::Error> {
    // Internally tagged enum decoding otherwise accepts positional arrays.
    let object = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;
    serde_json::from_value(serde_json::Value::Object(object)).map_err(serde::de::Error::custom)
}

// Enum-map aliases are outside the published string-only JSON boundary.
fn string_enum<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let value = String::deserialize(deserializer)?;
    T::deserialize(StringDeserializer::<D::Error>::new(value))
}
