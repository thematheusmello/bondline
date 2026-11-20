//! A captured drawing revision, published separately from its editable source.
use crate::{ConversionOutput, Engine, Error, files::FilePlan};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    num::{NonZeroU16, NonZeroU32},
    sync::Arc,
};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportPng {
    pub expected_revision: u64,
    pub page_id: NonZeroU32,
    pub dpi: NonZeroU16,
    pub transparent: bool,
    pub output_path: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PngExportReceipt {
    pub input_revision: u64,
    pub session_generation: u64,
    pub page_id: NonZeroU32,
    pub dpi: NonZeroU16,
    pub transparent: bool,
    pub output: ConversionOutput,
}

impl Engine {
    pub(super) fn png_export_plan(&self, params: ExportPng) -> Result<FilePlan, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        self.document.drawing()?;
        Ok(FilePlan::ExportPng {
            params,
            document: Arc::clone(&self.document),
            save_slot: self.reserve_save()?,
            generation: self.generation,
        })
    }
}
