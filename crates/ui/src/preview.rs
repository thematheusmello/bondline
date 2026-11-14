//! Read-only registered SDF preview and bounded transient native state.

use bondline_commands::{
    Cancellation, CaptureStamp, CloseState, Engine, PngCapture, RenderBackend, RenderCapture,
    StoredContent,
};
use bondline_mcp::CommandExecutor;
use eframe::egui;
use std::{
    sync::mpsc::{self, Receiver},
    thread::JoinHandle,
    time::Duration,
};

#[cfg(test)]
mod scheduling_tests;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreviewKey {
    stamp: CaptureStamp,
    close_epoch: u64,
    rect_bits: [u32; 4],
    pixels: [u32; 2],
    dpi_bits: u32,
}

impl PreviewKey {
    fn new(
        engine: &Engine,
        rect: egui::Rect,
        dpi: f32,
        max_texture_side: usize,
    ) -> Result<Self, String> {
        let size = rect.size();
        if !rect.min.x.is_finite()
            || !rect.min.y.is_finite()
            || [size.x, size.y]
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.0 || *v > 100_000.0)
            || !dpi.is_finite()
            || dpi <= 0.0
            || max_texture_side == 0
        {
            return Err("The preview needs a valid display area and scale.".into());
        }
        let cap = max_texture_side.min(bondline_render::MAX_CAPTURE_DIMENSION_PX as usize) as u32;
        let scale = f64::from(dpi)
            .min(f64::from(cap) / f64::from(size.x))
            .min(f64::from(cap) / f64::from(size.y));
        let pixels = [size.x, size.y]
            .map(|v| (f64::from(v) * scale).round().clamp(1.0, f64::from(cap)) as u32);
        Ok(Self {
            stamp: engine.capture_stamp(),
            close_epoch: engine.state().close_epoch,
            rect_bits: [
                rect.min.x.to_bits(),
                rect.min.y.to_bits(),
                size.x.to_bits(),
                size.y.to_bits(),
            ],
            pixels,
            dpi_bits: dpi.to_bits(),
        })
    }

    fn rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(
                f32::from_bits(self.rect_bits[0]),
                f32::from_bits(self.rect_bits[1]),
            ),
            egui::vec2(
                f32::from_bits(self.rect_bits[2]),
                f32::from_bits(self.rect_bits[3]),
            ),
        )
    }

    fn current(&self, engine: &Engine) -> bool {
        let state = engine.state();
        matches!(engine.content(), StoredContent::Sdf { .. })
            && state.close == CloseState::Open
            && state.close_epoch == self.close_epoch
            && engine.capture_stamp_matches(&self.stamp)
    }

    fn labels_readable(&self, em: Option<f32>) -> bool {
        let size = self.rect().size();
        let ratio = (size.x / self.pixels[0] as f32).min(size.y / self.pixels[1] as f32);
        em.is_some_and(|em| em.is_finite() && (8.0..=256.0).contains(&(em * ratio)))
    }

    fn request(&self) -> RenderCapture {
        RenderCapture {
            page_id: 1,
            width_px: self.pixels[0],
            height_px: self.pixels[1],
            backend: RenderBackend::Cpu,
        }
    }
}

struct PreviewJob {
    key: PreviewKey,
    cancel: Cancellation,
    result: Receiver<Result<egui::ColorImage, String>>,
    worker: JoinHandle<()>,
}

struct ReadyPreview {
    key: PreviewKey,
    texture: egui::TextureHandle,
}

#[derive(Default)]
pub(crate) struct PreviewWorkflow {
    desired: Option<PreviewKey>,
    pending: Option<PreviewKey>,
    job: Option<PreviewJob>,
    ready: Option<ReadyPreview>,
    error: Option<(PreviewKey, String)>,
}

impl PreviewWorkflow {
    pub(crate) fn is_busy(&self) -> bool {
        self.job.is_some() || self.pending.is_some()
    }

    fn clear(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
        self.desired = None;
        self.pending = None;
        self.ready = None;
        self.error = None;
    }

    pub(crate) fn sync(&mut self, engine: &Engine, context: &egui::Context) {
        if self
            .desired
            .as_ref()
            .is_some_and(|key| !key.current(engine))
        {
            self.clear();
        }
        // A finished current job is installed only after this frame's actual view key is known.
        if self
            .job
            .as_ref()
            .is_some_and(|job| self.desired.as_ref() != Some(&job.key))
        {
            self.retire(context, None);
        }
    }

    pub(crate) fn deactivate(&mut self, context: &egui::Context) {
        self.clear();
        self.retire(context, None);
    }

    fn retire(&mut self, context: &egui::Context, engine: Option<&Engine>) {
        if self
            .job
            .as_ref()
            .is_some_and(|job| !job.worker.is_finished())
        {
            context.request_repaint_after(Duration::from_millis(40));
            return;
        }
        let Some(job) = self.job.take() else {
            return;
        };
        let PreviewJob {
            key,
            cancel,
            result,
            worker,
        } = job;
        let result = match worker.join() {
            Ok(()) => result
                .try_recv()
                .unwrap_or_else(|_| Err("The preview worker stopped without an image.".into())),
            Err(_) => Err("The preview worker did not complete.".into()),
        };
        if cancel.is_cancelled()
            || self.desired.as_ref() != Some(&key)
            || !engine.is_some_and(|engine| key.current(engine))
        {
            return;
        }
        match result {
            Ok(image)
                if image.size == key.pixels.map(|v| v as usize)
                    && image
                        .size
                        .iter()
                        .all(|size| *size <= context.input(|i| i.max_texture_side)) =>
            {
                self.ready = Some(ReadyPreview {
                    key,
                    texture: context.load_texture(
                        "sdf-preview",
                        image,
                        egui::TextureOptions::NEAREST,
                    ),
                });
            }
            Ok(_) => {
                self.error = Some((
                    key,
                    "The preview image dimensions do not match this view.".into(),
                ))
            }
            Err(message) => self.error = Some((key, message)),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, engine: &Engine, rect: egui::Rect) {
        self.sync(engine, ui.ctx());
        ui.allocate_rect(rect, egui::Sense::hover());
        if engine.state().close != CloseState::Open {
            self.deactivate(ui.ctx());
            return;
        }
        let key = match PreviewKey::new(
            engine,
            rect,
            ui.ctx().pixels_per_point(),
            ui.input(|i| i.max_texture_side),
        ) {
            Ok(key) => key,
            Err(message) => {
                self.deactivate(ui.ctx());
                ui.put(rect, egui::Label::new(message));
                return;
            }
        };
        if self.desired.as_ref() != Some(&key) {
            self.clear();
            self.desired = Some(key.clone());
        }
        self.retire(ui.ctx(), Some(engine));
        if let Some(ready) = &self.ready
            && ready.key == key
        {
            ui.painter_at(rect).image(
                ready.texture.id(),
                rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            return;
        }
        let mut overlay = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("sdf-preview-status")
                .max_rect(rect.shrink(8.0))
                .layout(egui::Layout::top_down(egui::Align::Center)),
        );
        overlay.set_clip_rect(rect.intersect(ui.clip_rect()));
        if let Some((error_key, message)) = &self.error
            && error_key == &key
        {
            overlay.colored_label(
                egui::Color32::DARK_RED,
                format!("Preview unavailable: {message}"),
            );
            if overlay
                .add_enabled(!self.is_busy(), egui::Button::new("Retry preview"))
                .clicked()
            {
                self.error = None;
                self.pending = Some(key);
            }
        } else {
            if self.job.is_none() {
                self.pending = Some(key.clone());
            }
            overlay.spinner();
            overlay.label("Preparing chemical preview…");
            if overlay.button("Cancel preview").clicked() {
                if let Some(job) = &self.job {
                    job.cancel.cancel();
                }
                self.pending = None;
                self.error = Some((key, "Preview cancelled.".into()));
            }
        }
        if self.is_busy() {
            ui.ctx().request_repaint_after(Duration::from_millis(40));
        }
    }

    /// Called after the frame's Engine guard is released; pending intent already blocks close.
    pub(crate) fn start_pending(&mut self, executor: &CommandExecutor, context: &egui::Context) {
        let Some(key) = self.pending.take() else {
            return;
        };
        if self.job.is_some() || self.desired.as_ref() != Some(&key) {
            return;
        }
        let work = match executor.prepare_ui_capture(&key.stamp, key.request()) {
            Ok(work) => work,
            Err(error) => {
                if error.code == "busy" {
                    context.request_repaint_after(Duration::from_millis(40));
                    return;
                }
                if !matches!(
                    error.code,
                    "stale_revision" | "ui_stale_request" | "close_committed"
                ) {
                    self.error = Some((key, error.message));
                }
                context.request_repaint();
                return;
            }
        };
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let worker_key = key.clone();
        let worker_context = context.clone();
        let (sender, result) = mpsc::sync_channel(1);
        match std::thread::Builder::new()
            .name("bondline-preview".into())
            .spawn(move || {
                let output = work
                    .complete_blocking(&worker_cancel)
                    .map_err(|error| error.message)
                    .and_then(|result| {
                        result
                            .capture
                            .ok_or_else(|| "The preview command returned no image.".to_owned())
                    })
                    .and_then(|capture| decode(capture, &worker_key, &worker_cancel));
                let _ = sender.send(output);
                worker_context.request_repaint();
            }) {
            Ok(worker) => {
                self.job = Some(PreviewJob {
                    key,
                    cancel,
                    result,
                    worker,
                })
            }
            Err(_) => {
                self.error = Some((key, "The preview worker could not start. Try again.".into()));
                context.request_repaint();
            }
        }
    }
}

fn decode(
    capture: PngCapture,
    key: &PreviewKey,
    cancel: &Cancellation,
) -> Result<egui::ColorImage, String> {
    cancel.check().map_err(|e| e.to_string())?;
    if [capture.width_px, capture.height_px] != key.pixels {
        return Err("The preview image dimensions do not match this view.".into());
    }
    if !key.labels_readable(capture.label_em_px) {
        return Err("Enlarge the view to read every chemical label.".into());
    }
    let bytes = capture.png_bytes().map_err(|e| e.to_string())?;
    cancel.check().map_err(|e| e.to_string())?;
    let image = eframe::icon_data::from_png_bytes(&bytes)
        .map_err(|_| "The preview image could not be decoded.".to_owned())?;
    cancel.check().map_err(|e| e.to_string())?;
    if [image.width, image.height] != key.pixels
        || image.rgba.len() != key.pixels[0] as usize * key.pixels[1] as usize * 4
    {
        return Err("The preview image dimensions do not match this view.".into());
    }
    let image =
        egui::ColorImage::from_rgba_unmultiplied(key.pixels.map(|v| v as usize), &image.rgba);
    cancel.check().map_err(|e| e.to_string())?;
    Ok(image)
}

impl Drop for PreviewWorkflow {
    fn drop(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
    }
}
