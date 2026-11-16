//! Additive wire request; the existing public sized-ring parameters stay unchanged.
use super::{RingCreate, RingPlacement};
use crate::OrderHydrogenPolicy;
use schemars::JsonSchema;
use serde::de::value::StringDeserializer;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RingKind {
    Benzene,
    Cyclopentadiene,
}

impl RingKind {
    pub const fn size(self) -> u8 {
        match self {
            Self::Benzene => 6,
            Self::Cyclopentadiene => 5,
        }
    }

    pub const fn base_weights(self) -> &'static [u16] {
        match self {
            Self::Benzene => &[2, 1, 2, 1, 2, 1],
            Self::Cyclopentadiene => &[2, 1, 2, 1, 1],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RingPhase {
    DoubleFirst,
    SingleFirst,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NamedRingCreate {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[serde(deserialize_with = "present_kind")]
    #[schemars(with = "RingKind")]
    pub kind: RingKind,
    pub hydrogen_policy: OrderHydrogenPolicy,
    pub placement: RingPlacement,
    #[serde(
        default,
        deserialize_with = "present_phase",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "RingPhase")]
    pub phase: Option<RingPhase>,
}

// Restrict new named fields to their documented string representation only.
pub(crate) fn present_kind<'de, D>(deserializer: D) -> Result<RingKind, D::Error>
where
    D: Deserializer<'de>,
{
    let kind = String::deserialize(deserializer)?;
    RingKind::deserialize(StringDeserializer::<D::Error>::new(kind))
}

// A missing field defaults to None; a present null must not become contextual phase.
fn present_phase<'de, D>(deserializer: D) -> Result<Option<RingPhase>, D::Error>
where
    D: Deserializer<'de>,
{
    let phase = String::deserialize(deserializer)?;
    RingPhase::deserialize(StringDeserializer::<D::Error>::new(phase)).map(Some)
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("type" = "object"))]
pub enum RingCreateRequest {
    Sized(RingCreate),
    Named(NamedRingCreate),
}

impl RingCreateRequest {
    pub const fn expected_revision(&self) -> u64 {
        match self {
            Self::Sized(params) => params.expected_revision,
            Self::Named(params) => params.expected_revision,
        }
    }
}

impl From<RingCreate> for RingCreateRequest {
    fn from(params: RingCreate) -> Self {
        Self::Sized(params)
    }
}

impl From<NamedRingCreate> for RingCreateRequest {
    fn from(params: NamedRingCreate) -> Self {
        Self::Named(params)
    }
}
