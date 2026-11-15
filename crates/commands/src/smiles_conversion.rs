use crate::{Engine, Error};
use bondline_model::{PageSize, Point2};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SmilesConversionProfile {
    #[default]
    NeutralCarbonPathV1,
    NeutralAcyclicV1,
    NeutralSimpleRingV1,
    NeutralComponentsV1,
}

impl SmilesConversionProfile {
    pub const fn name(self) -> &'static str {
        match self {
            Self::NeutralCarbonPathV1 => "neutral_carbon_path_v1",
            Self::NeutralAcyclicV1 => "neutral_acyclic_v1",
            Self::NeutralSimpleRingV1 => "neutral_simple_ring_v1",
            Self::NeutralComponentsV1 => "neutral_components_v1",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::NeutralCarbonPathV1 => "Carbon chains",
            Self::NeutralAcyclicV1 => "Acyclic neutral structures",
            Self::NeutralSimpleRingV1 => "Neutral ring structures",
            Self::NeutralComponentsV1 => "Neutral components",
        }
    }

    fn is_legacy(&self) -> bool {
        *self == Self::NeutralCarbonPathV1
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConvertSmilesToCdxml {
    pub expected_revision: u64,
    pub input_path: String,
    pub output_path: String,
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub bond_length_pt: f64,
    #[schemars(range(min = -bondline_model::MAX_POSITION_PT, max = bondline_model::MAX_POSITION_PT))]
    pub origin_x_pt: f64,
    #[schemars(range(min = -bondline_model::MAX_POSITION_PT, max = bondline_model::MAX_POSITION_PT))]
    pub origin_y_pt: f64,
    #[schemars(extend("exclusiveMinimum" = 0.0, "maximum" = bondline_model::MAX_PAGE_DIMENSION_PT))]
    pub page_width_pt: f64,
    #[schemars(extend("exclusiveMinimum" = 0.0, "maximum" = bondline_model::MAX_PAGE_DIMENSION_PT))]
    pub page_height_pt: f64,
    #[serde(default, skip_serializing_if = "SmilesConversionProfile::is_legacy")]
    pub profile: SmilesConversionProfile,
}

#[derive(Clone, Debug, Serialize)]
pub struct ConversionOutput {
    pub path: String,
    pub byte_length: usize,
    pub directory_synced: bool,
}

/// Historical conversion identity and publication metadata, never a saved-session baseline.
#[derive(Clone, Debug, Serialize)]
pub struct ConversionReceipt {
    pub profile: &'static str,
    pub input_revision: u64,
    pub session_generation: u64,
    pub input_path: String,
    pub input_byte_length: usize,
    pub atom_count: usize,
    pub bond_count: usize,
    pub output: ConversionOutput,
}

impl ConvertSmilesToCdxml {
    pub(super) fn geometry(&self) -> Result<(Point2, PageSize), Error> {
        if !self.bond_length_pt.is_finite() || self.bond_length_pt <= 0.0 {
            return Err(Error::InvalidSmilesConversion(
                "bond length must be finite and positive",
            ));
        }
        Ok((
            Point2::new(self.origin_x_pt, self.origin_y_pt)?,
            PageSize::new(self.page_width_pt, self.page_height_pt)?,
        ))
    }
}

impl Engine {
    pub(super) fn smiles_conversion_plan(
        &self,
        params: ConvertSmilesToCdxml,
    ) -> Result<crate::files::FilePlan, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        let (origin, page) = params.geometry()?;
        Ok(crate::files::FilePlan::ConvertSmiles {
            params,
            origin,
            page,
            save_slot: self.reserve_save()?,
            generation: self.generation,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ConversionReceipt {
    pub(super) fn from_file(
        receipt: bondline_io::files::SmilesConversionReceipt,
        input_revision: u64,
        session_generation: u64,
        profile: SmilesConversionProfile,
    ) -> Result<Self, Error> {
        let path = |path: std::path::PathBuf| {
            path.into_os_string()
                .into_string()
                .map_err(|_| Error::UnrepresentableFilePath { published: true })
        };
        Ok(Self {
            profile: profile.name(),
            input_revision,
            session_generation,
            input_path: path(receipt.input_path)?,
            input_byte_length: receipt.input_byte_length,
            atom_count: receipt.atom_count,
            bond_count: receipt.bond_count,
            output: ConversionOutput {
                path: path(receipt.output.path)?,
                byte_length: receipt.output.byte_length,
                directory_synced: receipt.output.directory_synced,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Prepared};
    use serde_json::json;

    #[test]
    fn conversion_and_nonfile_plans_explicitly_refuse_native_request_binding()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::default();
        let before = serde_json::to_value(engine.state())?;
        let mut conversion = engine.prepare(Command::parse(
            "file.convert_smiles_to_cdxml",
            json!({
                "expected_revision":0,"input_path":"not-read.smi","output_path":"not-created.cdxml",
                "bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,
                "page_width_pt":595.0,"page_height_pt":842.0,
            }),
        )?)?;
        assert_eq!(
            conversion
                .bind_ui_request(1)
                .err()
                .ok_or("conversion bound")?
                .info()
                .code,
            "invalid_params"
        );
        assert_eq!(
            engine
                .reserve_save()
                .err()
                .ok_or("reservation lost")?
                .info()
                .code,
            "busy"
        );
        drop(conversion);
        drop(engine.reserve_save()?);
        let mut immediate = Prepared::immediate(engine.state());
        assert_eq!(
            immediate
                .bind_ui_request(1)
                .err()
                .ok_or("nonfile bound")?
                .info()
                .code,
            "invalid_params"
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        Ok(())
    }
}
