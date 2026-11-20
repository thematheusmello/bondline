//! Interchange adapters; model and standalone format parsers remain free of native I/O.

pub mod cdx;
pub mod cdxml;
pub mod display;
mod document;
mod drawing_settings;
pub mod mol;
pub mod rxn;
pub mod sdf;
pub mod smiles;
pub use document::{DocumentError, SourceInfo, StoredDocument};
#[cfg(not(target_arch = "wasm32"))]
pub mod files;

use bondline_render::Frame;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Frame(#[from] bondline_render::Error),
    #[error(transparent)]
    Display(#[from] display::DisplayError),
    #[error(transparent)]
    Png(#[from] png::EncodingError),
}

/// Encode a bounded, tightly packed straight-RGBA8 frame as PNG. No filesystem writes.
pub fn encode_png(frame: &Frame) -> Result<Vec<u8>, Error> {
    frame.validate()?;
    png_bytes(frame, None, None)
}

/// Export the complete source-qualified page at physical resolution in pixels per inch.
pub fn encode_page_png(
    stored: &StoredDocument,
    page: bondline_model::PageId,
    dpi: u16,
    transparent: bool,
    cancel: &bondline_render::Cancellation,
) -> Result<Vec<u8>, Error> {
    let background = if transparent { [0; 4] } else { [255; 4] };
    let graph = display::publication_scene(stored, page, dpi, background, cancel)?;
    let frame = bondline_render::cpu_publication(graph.scene(), cancel)?;
    // One inch is exactly 0.0254 metres. PNG stores an integral pixel density.
    let pixels_per_metre = (u32::from(dpi) * 10_000 + 127) / 254;
    let bytes = png_bytes(
        &frame,
        Some(png::PixelDimensions {
            xppu: pixels_per_metre,
            yppu: pixels_per_metre,
            unit: png::Unit::Meter,
        }),
        Some(cancel),
    )?;
    cancel.check()?;
    Ok(bytes)
}

fn png_bytes(
    frame: &Frame,
    dimensions: Option<png::PixelDimensions>,
    cancel: Option<&bondline_render::Cancellation>,
) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, frame.width, frame.height);
        encoder.set_pixel_dims(dimensions);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder.write_header()?;
        if let Some(cancel) = cancel {
            use std::io::Write;
            let mut stream = writer.stream_writer()?;
            for chunk in frame.rgba.chunks(256 * 1024) {
                cancel.check()?;
                stream.write_all(chunk).map_err(png::EncodingError::from)?;
            }
            stream.finish()?;
        } else {
            writer.write_image_data(&frame.rgba)?;
        }
        writer.finish()?;
    }
    Ok(bytes)
}
