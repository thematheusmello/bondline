use crate::Error;
use bondline_io::StoredDocument;
use bondline_model::{DrawingSettings, DrawingStyle};
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

/// Atomic settings replacement; omitted spacing preserves it, null clears it.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("required" = ["expected_revision", "bond_length_pt", "chain_angle_degrees", "line_width_pt"]))]
pub struct DocumentSettingsSet {
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
    #[serde(
        deserialize_with = "required_nullable",
        serialize_with = "finite_nullable"
    )]
    #[schemars(extend("exclusiveMinimum" = 0.0, "maximum" = bondline_model::MAX_POSITION_PT))]
    pub line_width_pt: Option<f64>,
    #[serde(
        default,
        deserialize_with = "present_nullable",
        serialize_with = "finite_update",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(range(min = 1, max = 100))]
    pub bond_spacing_percent: Option<Option<f64>>,
}

fn required_nullable<'de, D: Deserializer<'de>>(value: D) -> Result<Option<f64>, D::Error> {
    Option::<f64>::deserialize(value)
}

fn present_nullable<'de, D: Deserializer<'de>>(value: D) -> Result<Option<Option<f64>>, D::Error> {
    required_nullable(value).map(Some)
}

fn finite_update<S: Serializer>(
    value: &Option<Option<f64>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        Some(value) => finite_nullable(value, serializer),
        None => serializer.serialize_none(),
    }
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
    candidate(document, settings, document.model().drawing_style())
}

pub(super) fn set_document(
    document: &StoredDocument,
    params: DocumentSettingsSet,
) -> Result<StoredDocument, Error> {
    require_source(document)?;
    let mut style = document
        .model()
        .drawing_style()
        .with_line_width(params.line_width_pt)?;
    if let Some(spacing) = params.bond_spacing_percent {
        if spacing.is_some_and(|value| !(1.0..=100.0).contains(&value)) {
            return Err(Error::InvalidBondSpacing);
        }
        style = style.with_bond_spacing_percent(spacing)?;
    }
    let settings = DrawingSettings::new(params.bond_length_pt, params.chain_angle_degrees)?;
    candidate(document, settings, style)
}

fn candidate(
    document: &StoredDocument,
    settings: DrawingSettings,
    style: DrawingStyle,
) -> Result<StoredDocument, Error> {
    let old_settings = document.model().drawing_settings();
    let old_style = document.model().drawing_style();
    if settings == old_settings && style == old_style {
        return Ok(document.clone());
    }
    let mut candidate = document.clone();
    candidate.model_mut().set_drawing_settings(settings);
    candidate.model_mut().set_drawing_style(style);
    if style != old_style
        || (style.bond_spacing_percent().is_some()
            && settings.bond_length_pt() != old_settings.bond_length_pt())
    {
        for page in candidate.model().pages() {
            bondline_io::smiles::validate_authoring_page_with_settings(
                page,
                style,
                settings,
                &bondline_render::Cancellation::default(),
            )
            .map_err(|error| match error {
                bondline_io::smiles::LayoutError::Render(error) => Error::Render(error),
                bondline_io::smiles::LayoutError::Geometry(reason) => {
                    Error::UnsupportedDrawingStyle(reason)
                }
                other => Error::SmilesLayout(other),
            })?;
        }
    }
    Ok(candidate)
}
