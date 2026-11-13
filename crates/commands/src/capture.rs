use crate::files::{Completed, Completion, FilePlan};
use crate::{CommandResult, Error};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use bondline_model::Page;
use bondline_render::{
    Cancellation, MAX_CAPTURE_DIMENSION_PX, carbon_page_scene, cpu_scene_with_cancel,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RenderBackend {
    Cpu,
    Gpu,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderCapture {
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 1, max = MAX_CAPTURE_DIMENSION_PX))]
    pub width_px: u32,
    #[schemars(range(min = 1, max = MAX_CAPTURE_DIMENSION_PX))]
    pub height_px: u32,
    pub backend: RenderBackend,
}

#[derive(Clone, Debug, Serialize)]
pub struct CaptureAdapter {
    pub name: String,
    pub backend: String,
    pub device_type: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PngCapture {
    pub mime_type: &'static str,
    pub width_px: u32,
    pub height_px: u32,
    pub backend: RenderBackend,
    pub adapter: Option<CaptureAdapter>,
    pub png_base64: String,
}

/// Immutable deferred work, or the result of an already committed short command.
#[derive(Debug)]
pub struct Prepared {
    result: CommandResult,
    capture: Option<(RenderCapture, Page)>,
    file: Option<FilePlan>,
}

impl Prepared {
    pub(super) fn immediate(result: CommandResult) -> Self {
        Self {
            result,
            capture: None,
            file: None,
        }
    }

    pub(super) fn file(result: CommandResult, file: FilePlan) -> Self {
        Self {
            result,
            capture: None,
            file: Some(file),
        }
    }

    pub(super) fn capture(
        result: CommandResult,
        params: RenderCapture,
        page: Page,
    ) -> Result<Self, Error> {
        if [params.width_px, params.height_px]
            .iter()
            .any(|axis| !(1..=MAX_CAPTURE_DIMENSION_PX).contains(axis))
        {
            return Err(bondline_render::Error::InvalidDimensions.into());
        }
        Ok(Self {
            result,
            capture: Some((params, page)),
            file: None,
        })
    }

    pub(super) fn bind_ui_request(&mut self, id: u64) {
        if let Some(file) = &mut self.file {
            file.bind_ui_request(id);
        }
    }

    pub fn is_deferred(&self) -> bool {
        self.capture.is_some() || self.file.is_some()
    }

    pub fn complete(self) -> Result<Completed, Error> {
        self.complete_with_cancel(&Cancellation::default())
    }

    pub fn complete_with_cancel(mut self, cancel: &Cancellation) -> Result<Completed, Error> {
        if let Some(file) = self.file {
            return file.complete(cancel);
        }
        let deferred_query = self.capture.is_some();
        if let Some((params, page)) = self.capture {
            cancel.check()?;
            let scene = carbon_page_scene(params.width_px, params.height_px, &page)?;
            let (frame, adapter) = match params.backend {
                RenderBackend::Cpu => (cpu_scene_with_cancel(&scene, cancel)?, None),
                RenderBackend::Gpu => {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let (frame, info) = pollster::block_on(
                            bondline_render::gpu_scene_with_cancel(&scene, cancel),
                        )?;
                        (
                            frame,
                            Some(CaptureAdapter {
                                name: info.name,
                                backend: format!("{:?}", info.backend),
                                device_type: format!("{:?}", info.device_type),
                            }),
                        )
                    }
                    #[cfg(target_arch = "wasm32")]
                    return Err(Error::BackendUnavailable);
                }
            };
            cancel.check()?;
            let png = bondline_io::encode_png(&frame)?;
            cancel.check()?;
            self.result.capture = Some(PngCapture {
                mime_type: "image/png",
                width_px: frame.width,
                height_px: frame.height,
                backend: params.backend,
                adapter,
                png_base64: STANDARD.encode(png),
            });
        }
        Ok(Completed(Completion::Result {
            result: Box::new(self.result),
            cancel: deferred_query.then(|| cancel.clone()),
        }))
    }
}
