use crate::{Error, Frame, byte_count, cpu_solid};
use bondline_model::PageSize;

/// Integer, half-open rectangle in canvas pixels. Later rectangles replace earlier pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub color: [u8; 4],
}

/// Shared canvas input. No antialiasing, blending or chemical primitives are claimed yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub background: [u8; 4],
    pub rectangles: Vec<Rectangle>,
}

impl Scene {
    pub(crate) fn validate(&self) -> Result<usize, Error> {
        let count = byte_count(self.width, self.height)?;
        if self.rectangles.len() > 4096 {
            return Err(Error::SceneTooComplex);
        }
        for rectangle in &self.rectangles {
            if rectangle.width == 0
                || rectangle.height == 0
                || rectangle.x.checked_add(rectangle.width).is_none()
                || rectangle.y.checked_add(rectangle.height).is_none()
            {
                return Err(Error::InvalidGeometry);
            }
        }
        Ok(count)
    }
}

/// CPU reference: clip each rectangle to the frame and replace straight RGBA8 pixels.
pub fn cpu_scene(scene: &Scene) -> Result<Frame, Error> {
    scene.validate()?;
    let mut frame = cpu_solid(scene.width, scene.height, scene.background)?;
    for rectangle in &scene.rectangles {
        let right = (rectangle.x + rectangle.width).min(scene.width);
        let bottom = (rectangle.y + rectangle.height).min(scene.height);
        let left = rectangle.x.min(scene.width);
        for y in rectangle.y.min(scene.height)..bottom {
            let start = ((y * scene.width + left) * 4) as usize;
            let end = ((y * scene.width + right) * 4) as usize;
            for pixel in frame
                .rgba
                .get_mut(start..end)
                .ok_or(Error::InvalidReadback)?
                .chunks_exact_mut(4)
            {
                pixel.copy_from_slice(&rectangle.color);
            }
        }
    }
    Ok(frame)
}

/// Fit the physical page into a viewport with a bounded margin and an original paper style.
/// Geometry can serve larger native viewports; pixel readbacks remain limited to 2048 per axis.
pub fn page_scene(width: u32, height: u32, page: PageSize) -> Result<Scene, Error> {
    if !(1..=100_000).contains(&width) || !(1..=100_000).contains(&height) {
        return Err(Error::InvalidViewport);
    }
    let margin = 32.min(width / 8).min(height / 8);
    let available_width = width - margin * 2;
    let available_height = height - margin * 2;
    let maximum = page.width_pt().max(page.height_pt());
    let normalized_width = page.width_pt() / maximum;
    let normalized_height = page.height_pt() / maximum;
    let scale = (f64::from(available_width) / normalized_width)
        .min(f64::from(available_height) / normalized_height);
    // shortcut: extreme page aspect ratios occupy at least one display pixel; exports use authored dimensions.
    let paper_width = (normalized_width * scale)
        .round()
        .clamp(1.0, f64::from(available_width)) as u32;
    let paper_height = (normalized_height * scale)
        .round()
        .clamp(1.0, f64::from(available_height)) as u32;
    let x = (width - paper_width) / 2;
    let y = (height - paper_height) / 2;
    let mut rectangles = vec![Rectangle {
        x: x + 3,
        y: y + 4,
        width: paper_width,
        height: paper_height,
        color: [210, 210, 210, 255],
    }];
    if paper_width >= 3 && paper_height >= 3 {
        rectangles.push(Rectangle {
            x,
            y,
            width: paper_width,
            height: paper_height,
            color: [196, 196, 196, 255],
        });
        rectangles.push(Rectangle {
            x: x + 1,
            y: y + 1,
            width: paper_width - 2,
            height: paper_height - 2,
            color: [255; 4],
        });
    } else {
        rectangles.push(Rectangle {
            x,
            y,
            width: paper_width,
            height: paper_height,
            color: [255; 4],
        });
    }
    Ok(Scene {
        width,
        height,
        background: [232, 232, 232, 255],
        rectangles,
    })
}
