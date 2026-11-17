use crate::Error;
use bondline_io::StoredDocument;
use bondline_model::DrawingSettings;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Full pair replacement: both keys must be present; null clears a preference.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("required" = ["expected_revision", "bond_length_pt", "chain_angle_degrees"]))]
pub struct DrawingSettingsSet {
    pub expected_revision: u64,
    #[serde(
        deserialize_with = "required_nullable",
        serialize_with = "finite_nullable"
    )]
    #[schemars(extend("exclusiveMinimum" = 0.0, "maximum" = bondline_model::MAX_POSITION_PT))]
    pub bond_length_pt: Option<f64>,
    #[serde(
        deserialize_with = "required_nullable",
        serialize_with = "finite_nullable"
    )]
    #[schemars(range(min = 1, max = 179))]
    pub chain_angle_degrees: Option<f64>,
}

fn required_nullable<'de, D: Deserializer<'de>>(value: D) -> Result<Option<f64>, D::Error> {
    Option::<f64>::deserialize(value)
}

fn finite_nullable<S: Serializer>(value: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error> {
    if value.is_some_and(|value| !value.is_finite()) {
        return Err(serde::ser::Error::custom(
            "drawing settings must be finite numbers or explicit null",
        ));
    }
    value.serialize(serializer)
}

pub(super) fn require_source(document: &StoredDocument) -> Result<(), Error> {
    if !document.is_fully_mapped()
        || document
            .source_info()
            .is_some_and(|source| source.format != "cdxml")
    {
        return Err(Error::UnsupportedDrawingSettingsSource);
    }
    Ok(())
}

pub(super) fn set(
    document: &StoredDocument,
    params: DrawingSettingsSet,
) -> Result<StoredDocument, Error> {
    require_source(document)?;
    let settings = DrawingSettings::new(params.bond_length_pt, params.chain_angle_degrees)?;
    if document.model().drawing_settings() == settings {
        return Ok(document.clone());
    }
    let mut candidate = document.clone();
    candidate.model_mut().set_drawing_settings(settings);
    Ok(candidate)
}
