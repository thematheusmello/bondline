//! Interchange adapters; model and standalone format parsers remain free of native I/O.

pub mod cdxml;
mod document;
pub mod mol;
pub mod rxn;
pub mod sdf;
pub use document::{DocumentError, SourceInfo, StoredDocument};
#[cfg(not(target_arch = "wasm32"))]
pub mod files;

use bondline_render::Frame;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Frame(#[from] bondline_render::Error),
    #[error(transparent)]
    Png(#[from] png::EncodingError),
}

/// Encode a bounded, tightly packed straight-RGBA8 frame as PNG. No filesystem writes.
pub fn encode_png(frame: &Frame) -> Result<Vec<u8>, Error> {
    frame.validate()?;
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, frame.width, frame.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&frame.rgba)?;
        writer.finish()?;
    }
    Ok(bytes)
}
