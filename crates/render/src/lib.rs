//! GUI-free CPU reference and native GPU readback foundation.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub const MAX_CAPTURE_DIMENSION_PX: u32 = 2048;

/// Cooperative cancellation shared between an adapter and its rendering worker.
#[derive(Clone, Debug, Default)]
pub struct Cancellation(Arc<AtomicBool>);

impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
    pub fn check(&self) -> Result<(), Error> {
        if self.is_cancelled() {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}

/// An owned, tightly packed RGBA8 frame.
#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Frame {
    /// Validate caller-owned data before any interchange encoder consumes it.
    pub fn validate(&self) -> Result<(), Error> {
        if self.rgba.len() != byte_count(self.width, self.height)? {
            return Err(Error::InvalidFrame);
        }
        Ok(())
    }
}

/// Failures are explicit; a missing GPU never silently becomes a CPU result.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("render cancelled")]
    Cancelled,
    #[error("RGBA byte count does not match frame dimensions")]
    InvalidFrame,
    #[error("capture dimensions must be between 1 and 2048 pixels")]
    InvalidDimensions,
    #[error("viewport dimensions must be between 1 and 100000 pixels")]
    InvalidViewport,
    #[error(
        "view needs finite zoom within 1..999 percent, a center inside the page, and matching page/source geometry"
    )]
    InvalidView,
    #[error("rectangle dimensions must be nonzero and their edges must not overflow")]
    InvalidGeometry,
    #[error(
        "stroke coordinates must be finite within ±100000, width in (0,100000], and squared length normal in f32"
    )]
    InvalidStroke,
    #[error(
        "the preview requires supported neutral elements, coordinates and Single or centered Double/Triple bonds; literal isolated carbon needs authored H and physical H cannot carry attached H"
    )]
    UnsupportedChemicalContent,
    #[error("unsupported caption text: {0}")]
    UnsupportedText(&'static str),
    #[error("the bundled label font could not be decoded")]
    InvalidFont,
    #[error(
        "chemical labels need an em size between 8 and 256 canvas units; enlarge the view or capture"
    )]
    LabelScale,
    #[error(
        "multiple bond components and gaps need distinct visible pixels; enlarge the view or separate the endpoints"
    )]
    BondScale,
    #[error("a chemical label would extend beyond the viewport")]
    LabelPlacement,
    #[error(
        "chemical labels overlap other labels or unrelated bonds; undo the edit or separate them"
    )]
    LabelOverlap,
    #[error("hit coordinates must be finite and tolerance finite within 0..100000 canvas units")]
    InvalidHitTest,
    #[error("scene exceeds the 4096-primitive limit")]
    SceneTooComplex,
    #[error("GPU validation failed: {0}")]
    Validation(String),
    #[error("no native GPU adapter is available")]
    NoAdapter,
    #[error(transparent)]
    Device(#[from] wgpu::RequestDeviceError),
    #[error(transparent)]
    Readback(#[from] wgpu::BufferAsyncError),
    #[error(transparent)]
    Timeout(#[from] std::sync::mpsc::RecvTimeoutError),
    #[error("GPU readback layout did not match its descriptor")]
    InvalidReadback,
}

fn byte_count(width: u32, height: u32) -> Result<usize, Error> {
    // shortcut: captures are bounded to 2048 per axis; use tiled rendering for print-resolution export.
    if !(1..=MAX_CAPTURE_DIMENSION_PX).contains(&width)
        || !(1..=MAX_CAPTURE_DIMENSION_PX).contains(&height)
    {
        return Err(Error::InvalidDimensions);
    }
    usize::try_from(u64::from(width) * u64::from(height) * 4).map_err(|_| Error::InvalidDimensions)
}

/// CPU reference for an empty RGBA8 canvas.
pub fn cpu_solid(width: u32, height: u32, color: [u8; 4]) -> Result<Frame, Error> {
    let count = byte_count(width, height)?;
    Ok(Frame {
        width,
        height,
        rgba: color.repeat(count / 4),
    })
}

mod authored_label;
mod clearance;
#[cfg(test)]
mod display_label_tests;
pub use authored_label::{position_atom_label, validate_atom_label};
mod hits;
mod inset;
mod label;
#[cfg(test)]
mod label_tests;
mod page;
mod scene;
mod skeletal;
mod stroke;
mod text;
mod view;
pub use hits::{GraphScene, HitTarget, MAX_HIT_REGIONS};
pub use page::{
    PageViewport, carbon_page_scene, graph_page_scene, graph_page_scene_with_cancel,
    graph_page_scene_with_display_hydrogens,
};
pub use scene::{Primitive, Rectangle, Scene, cpu_scene, cpu_scene_with_cancel, page_scene};
pub use stroke::{Stroke, centered_bond_strokes, centered_bond_strokes_with_unit};
pub use view::PageView;

#[cfg(not(target_arch = "wasm32"))]
mod gpu;
#[cfg(not(target_arch = "wasm32"))]
pub use gpu::{gpu_scene, gpu_scene_with_cancel, gpu_solid};

#[cfg(test)]
mod tests {
    #[test]
    fn cpu_clear_matches_independent_pixel_reference() -> Result<(), Box<dyn std::error::Error>> {
        let frame = super::cpu_solid(2, 1, [12, 34, 56, 255])?;
        assert_eq!(frame.rgba, [12, 34, 56, 255, 12, 34, 56, 255]);
        assert!(super::cpu_solid(0, 1, [0; 4]).is_err());
        assert!(super::cpu_solid(u32::MAX, u32::MAX, [0; 4]).is_err());
        Ok(())
    }
}
