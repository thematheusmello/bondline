use crate::files::{Completed, Completion, FilePlan};
use crate::{CommandResult, Engine, Error, StoredContent, ViewState};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use bondline_model::PageId;
use bondline_render::{Cancellation, MAX_CAPTURE_DIMENSION_PX, cpu_scene_with_cancel};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Weak};

/// Opaque identity for a captured complete content snapshot; it does not keep source alive.
/// Close state is an independent host policy, not part of this identity.
///
/// ```compile_fail
/// let stamp = bondline_commands::Engine::default().capture_stamp();
/// let _ = serde_json::to_value(stamp);
/// ```
#[derive(Clone)]
pub struct CaptureStamp {
    owner: Weak<StoredContent>,
    revision: u64,
    generation: u64,
}

impl std::fmt::Debug for CaptureStamp {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("CaptureStamp")
            .field("revision", &self.revision)
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

impl PartialEq for CaptureStamp {
    fn eq(&self, other: &Self) -> bool {
        self.revision == other.revision
            && self.generation == other.generation
            && Weak::ptr_eq(&self.owner, &other.owner)
    }
}

impl Eq for CaptureStamp {}

impl Engine {
    pub fn capture_stamp(&self) -> CaptureStamp {
        CaptureStamp {
            owner: Arc::downgrade(&self.document),
            revision: self.revision,
            generation: self.generation,
        }
    }

    pub fn capture_stamp_matches(&self, stamp: &CaptureStamp) -> bool {
        stamp == &self.capture_stamp()
    }

    /// Shared current Drawing scene for native vector painting; it grants no edit authority.
    pub fn drawing_page_scene(
        &self,
        width: u32,
        height: u32,
        page_id: PageId,
        cancel: &Cancellation,
    ) -> Result<bondline_render::GraphScene, Error> {
        Ok(bondline_io::display::drawing_page_scene(
            width,
            height,
            self.stored_document()?,
            page_id,
            cancel,
        )?)
    }
}

fn capture_scene(
    content: &StoredContent,
    params: &RenderCapture,
    context: Option<&ViewCaptureContext>,
    cancel: &Cancellation,
) -> Result<
    (
        bondline_render::GraphScene,
        Option<bondline_model::PageSize>,
    ),
    Error,
> {
    match content {
        StoredContent::Sdf { .. } => {
            let preview = bondline_io::sdf::preview::prepare_sdf_preview(content.sdf()?, cancel)?;
            let graph = if let Some(context) = context {
                capture_view(params, context, preview.page().size())?
                    .graph_scene_with_display_hydrogens(
                        preview.page(),
                        preview.display_hydrogens(),
                        cancel,
                    )?
            } else {
                bondline_render::graph_page_scene_with_display_hydrogens(
                    params.width_px,
                    params.height_px,
                    preview.page(),
                    preview.display_hydrogens(),
                    cancel,
                )?
            };
            Ok((graph, Some(preview.page().size())))
        }
        _ => {
            let stored = content.drawing()?;
            let id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
            let graph = if let Some(context) = context {
                let page = stored
                    .model()
                    .pages()
                    .iter()
                    .find(|page| page.id() == id)
                    .ok_or(bondline_model::Error::MissingPage(id.get()))?;
                bondline_io::display::drawing_page_scene_with_view(
                    &capture_view(params, context, page.size())?,
                    stored,
                    id,
                    cancel,
                )?
            } else {
                bondline_io::display::drawing_page_scene(
                    params.width_px,
                    params.height_px,
                    stored,
                    id,
                    cancel,
                )?
            };
            Ok((graph, None))
        }
    }
}

fn capture_view(
    params: &RenderCapture,
    context: &ViewCaptureContext,
    size: bondline_model::PageSize,
) -> Result<bondline_render::PageView, Error> {
    let view = params
        .view
        .ok_or(Error::InvalidView("current-view dimensions are missing"))?;
    Ok(
        crate::view::page_view_for(context.state, view.logical_width, view.logical_height, size)?
            .at_pixel_size(params.width_px, params.height_px)?,
    )
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<CaptureView>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureView {
    #[schemars(range(min = 1, max = 100_000))]
    pub logical_width: u32,
    #[schemars(range(min = 1, max = 100_000))]
    pub logical_height: u32,
    pub expected_view_epoch: u64,
}

#[derive(Debug)]
pub(super) struct ViewCaptureContext {
    pub state: ViewState,
    pub stamp: CaptureStamp,
    pub close_epoch: u64,
}

impl Engine {
    pub(super) fn prepare_capture(&self, params: RenderCapture) -> Result<Prepared, Error> {
        let context = if let Some(view) = params.view {
            if !(1..=100_000).contains(&view.logical_width)
                || !(1..=100_000).contains(&view.logical_height)
            {
                return Err(Error::InvalidView(
                    "logical dimensions must be within 1..100000",
                ));
            }
            if view.expected_view_epoch != self.view().epoch {
                return Err(Error::StaleView);
            }
            Some(ViewCaptureContext {
                state: self.view(),
                stamp: self.capture_stamp(),
                close_epoch: self.close_policy.epoch,
            })
        } else {
            None
        };
        Prepared::capture(self.state(), params, Arc::clone(&self.document), context)
    }
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
    /// Actual main-label em from this scene, excluding subscripts; native presentation only.
    #[serde(skip)]
    pub label_em_px: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CaptureImageError {
    #[error("capture MIME type must be image/png")]
    InvalidMime,
    #[error("capture dimensions must be between 1 and 2048 pixels")]
    InvalidDimensions,
    #[error("capture image exceeds its dimension-specific byte budget")]
    SizeLimit,
    #[error("capture image is not canonical base64")]
    InvalidBase64,
    #[error("capture PNG signature or IHDR is invalid or incomplete")]
    InvalidHeader,
    #[error("capture PNG dimensions do not match its metadata")]
    DimensionsMismatch,
    #[error("capture PNG must use noninterlaced 8-bit RGBA encoding")]
    UnsupportedEncoding,
    #[error(
        "capture PNG must have one complete IDAT and IEND, without other chunks or trailing data"
    )]
    InvalidFraming,
}

impl CaptureImageError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::SizeLimit => "resource_limit",
            _ => "invalid_capture_image",
        }
    }
}

fn png_budget(width: u32, height: u32) -> Result<(usize, usize), CaptureImageError> {
    if [width, height]
        .iter()
        .any(|axis| !(1..=MAX_CAPTURE_DIMENSION_PX).contains(axis))
    {
        return Err(CaptureImageError::InvalidDimensions);
    }
    let width = usize::try_from(width).map_err(|_| CaptureImageError::SizeLimit)?;
    let height = usize::try_from(height).map_err(|_| CaptureImageError::SizeLimit)?;
    let raw = width
        .checked_mul(4)
        .and_then(|row| row.checked_add(1))
        .and_then(|row| row.checked_mul(height))
        .ok_or(CaptureImageError::SizeLimit)?;
    // Pinned Fast encoder falls back to stored blocks: 5 bytes/65535 +6 zlib +57 PNG.
    let blocks = raw
        .checked_add(65_534)
        .ok_or(CaptureImageError::SizeLimit)?
        / 65_535;
    let png = blocks
        .checked_mul(5)
        .and_then(|headers| raw.checked_add(headers))
        .and_then(|bytes| bytes.checked_add(63))
        .ok_or(CaptureImageError::SizeLimit)?;
    let base64 = png
        .checked_add(2)
        .map(|bytes| bytes / 3)
        .and_then(|groups| groups.checked_mul(4))
        .ok_or(CaptureImageError::SizeLimit)?;
    Ok((png, base64))
}

fn png_u32(bytes: &[u8], range: std::ops::Range<usize>) -> Result<u32, CaptureImageError> {
    Ok(u32::from_be_bytes(
        bytes
            .get(range)
            .and_then(|field| field.try_into().ok())
            .ok_or(CaptureImageError::InvalidHeader)?,
    ))
}

impl PngCapture {
    /// Decode only this pinned producer's bounded PNG envelope, not general PNG files.
    /// Signature/header/framing are preflighted; the real image decoder must still validate
    /// CRC, DEFLATE and scanline contents before using pixels.
    pub fn png_bytes(&self) -> Result<Vec<u8>, CaptureImageError> {
        if self.mime_type != "image/png" {
            return Err(CaptureImageError::InvalidMime);
        }
        let (png_limit, base64_limit) = png_budget(self.width_px, self.height_px)?;
        let input = self.png_base64.as_bytes();
        if input.len() > base64_limit {
            return Err(CaptureImageError::SizeLimit);
        }
        if !input.len().is_multiple_of(4) {
            return Err(CaptureImageError::InvalidBase64);
        }
        let padding = input
            .iter()
            .rev()
            .take(3)
            .take_while(|byte| **byte == b'=')
            .count();
        if padding > 2 {
            return Err(CaptureImageError::InvalidBase64);
        }
        let decoded_length = (input.len() / 4)
            .checked_mul(3)
            .and_then(|length| length.checked_sub(padding))
            .ok_or(CaptureImageError::InvalidBase64)?;
        if decoded_length > png_limit {
            return Err(CaptureImageError::SizeLimit);
        }
        let mut bytes = vec![0; decoded_length];
        let length = STANDARD
            .decode_slice(input, &mut bytes)
            .map_err(|_| CaptureImageError::InvalidBase64)?;
        if length != decoded_length {
            return Err(CaptureImageError::InvalidBase64);
        }
        if bytes.get(..8) != Some(b"\x89PNG\r\n\x1a\n")
            || bytes.get(8..16) != Some(b"\0\0\0\rIHDR")
            || bytes.get(..33).is_none()
        {
            return Err(CaptureImageError::InvalidHeader);
        }
        if png_u32(&bytes, 16..20)? != self.width_px || png_u32(&bytes, 20..24)? != self.height_px {
            return Err(CaptureImageError::DimensionsMismatch);
        }
        if bytes.get(24..29) != Some([8, 6, 0, 0, 0].as_slice()) {
            return Err(CaptureImageError::UnsupportedEncoding);
        }
        let data_length = usize::try_from(
            png_u32(&bytes, 33..37).map_err(|_| CaptureImageError::InvalidFraming)?,
        )
        .map_err(|_| CaptureImageError::InvalidFraming)?;
        let end = data_length
            .checked_add(45)
            .ok_or(CaptureImageError::InvalidFraming)?;
        let complete = end
            .checked_add(12)
            .ok_or(CaptureImageError::InvalidFraming)?;
        let end_chunk = bytes.get(end..complete);
        if data_length == 0
            || bytes.get(37..41) != Some(b"IDAT")
            || complete != bytes.len()
            || end_chunk.and_then(|chunk| chunk.get(..8)) != Some(b"\0\0\0\0IEND")
        {
            return Err(CaptureImageError::InvalidFraming);
        }
        Ok(bytes)
    }
}

/// Immutable deferred work, or the result of an already committed short command.
#[derive(Debug)]
pub struct Prepared {
    result: CommandResult,
    capture: Option<(
        RenderCapture,
        Arc<StoredContent>,
        Option<ViewCaptureContext>,
    )>,
    file: Option<FilePlan>,
    sdf_record: Option<crate::sdf_query::RecordPlan>,
    sdf_edit: Option<crate::sdf_edit::EditPlan>,
    smiles_record: Option<crate::smiles_query::RecordPlan>,
}

impl Prepared {
    pub(super) fn immediate(result: CommandResult) -> Self {
        Self {
            result,
            capture: None,
            file: None,
            sdf_record: None,
            sdf_edit: None,
            smiles_record: None,
        }
    }

    pub(super) fn file(result: CommandResult, file: FilePlan) -> Self {
        Self {
            result,
            capture: None,
            file: Some(file),
            sdf_record: None,
            sdf_edit: None,
            smiles_record: None,
        }
    }

    pub(super) fn capture(
        result: CommandResult,
        params: RenderCapture,
        content: Arc<StoredContent>,
        context: Option<ViewCaptureContext>,
    ) -> Result<Self, Error> {
        if [params.width_px, params.height_px]
            .iter()
            .any(|axis| !(1..=MAX_CAPTURE_DIMENSION_PX).contains(axis))
        {
            return Err(bondline_render::Error::InvalidDimensions.into());
        }
        Ok(Self {
            result,
            capture: Some((params, content, context)),
            file: None,
            sdf_record: None,
            sdf_edit: None,
            smiles_record: None,
        })
    }

    pub(super) fn sdf_record(result: CommandResult, plan: crate::sdf_query::RecordPlan) -> Self {
        Self {
            result,
            capture: None,
            file: None,
            sdf_record: Some(plan),
            sdf_edit: None,
            smiles_record: None,
        }
    }

    pub(super) fn sdf_edit(result: CommandResult, plan: crate::sdf_edit::EditPlan) -> Self {
        Self {
            result,
            capture: None,
            file: None,
            sdf_record: None,
            sdf_edit: Some(plan),
            smiles_record: None,
        }
    }

    pub(super) fn smiles_record(
        result: CommandResult,
        plan: crate::smiles_query::RecordPlan,
    ) -> Self {
        Self {
            result,
            capture: None,
            file: None,
            sdf_record: None,
            sdf_edit: None,
            smiles_record: Some(plan),
        }
    }

    pub(super) fn bind_ui_request(&mut self, id: u64) -> Result<(), Error> {
        self.file
            .as_mut()
            .ok_or(Error::InvalidUiFileParameters(
                "only file work can bind a native file request",
            ))?
            .bind_ui_request(id)
    }

    pub fn is_deferred(&self) -> bool {
        self.capture.is_some()
            || self.file.is_some()
            || self.sdf_record.is_some()
            || self.sdf_edit.is_some()
            || self.smiles_record.is_some()
    }

    pub fn complete(self) -> Result<Completed, Error> {
        self.complete_with_cancel(&Cancellation::default())
    }

    pub fn complete_with_cancel(mut self, cancel: &Cancellation) -> Result<Completed, Error> {
        if let Some(file) = self.file {
            return file.complete(cancel);
        }
        if let Some(plan) = self.sdf_edit {
            return Ok(Completed(Completion::SdfEdit(plan.complete(cancel)?)));
        }
        let deferred_query =
            self.capture.is_some() || self.sdf_record.is_some() || self.smiles_record.is_some();
        if let Some(plan) = self.sdf_record {
            self.result.sdf_record = Some(plan.complete(cancel)?);
        }
        if let Some(plan) = self.smiles_record {
            self.result.smiles_record = Some(plan.complete(cancel)?);
        }
        let mut view_completion = None;
        if let Some((params, content, context)) = self.capture {
            cancel.check()?;
            let (graph, extent) = capture_scene(&content, &params, context.as_ref(), cancel)?;
            let label_em_px = graph.label_em_px();
            let scene = graph.into_scene();
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
                label_em_px,
            });
            if let Some(context) = context {
                view_completion = Some((context, extent));
            }
        }
        if let Some((context, extent)) = view_completion {
            return Ok(Completed(Completion::ViewCapture {
                result: Box::new(self.result),
                context,
                extent,
                cancel: cancel.clone(),
            }));
        }
        Ok(Completed(Completion::Result {
            result: Box::new(self.result),
            cancel: deferred_query.then(|| cancel.clone()),
        }))
    }
}
