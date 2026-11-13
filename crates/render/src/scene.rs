use crate::{Cancellation, Error, Frame, PageViewport, Stroke, byte_count, cpu_solid};
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

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Rectangle(Rectangle),
    Stroke(Stroke),
}

impl Primitive {
    pub(crate) fn bounds(&self, width: u32, height: u32) -> Option<Rectangle> {
        match self {
            Self::Stroke(stroke) => stroke.bounds(width, height),
            Self::Rectangle(rectangle) => {
                let right = (rectangle.x + rectangle.width).min(width);
                let bottom = (rectangle.y + rectangle.height).min(height);
                (rectangle.x < right && rectangle.y < bottom).then(|| Rectangle {
                    width: right - rectangle.x,
                    height: bottom - rectangle.y,
                    ..rectangle.clone()
                })
            }
        }
    }
}

/// Ordered source-replacement primitives. No blending or antialiasing in readbacks.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub background: [u8; 4],
    pub primitives: Vec<Primitive>,
}

impl Scene {
    pub(crate) fn validate(&self) -> Result<usize, Error> {
        let count = byte_count(self.width, self.height)?;
        if self.primitives.len() > 4096 {
            return Err(Error::SceneTooComplex);
        }
        for primitive in &self.primitives {
            match primitive {
                Primitive::Stroke(stroke) => stroke.validate()?,
                Primitive::Rectangle(rectangle) => {
                    if rectangle.width == 0
                        || rectangle.height == 0
                        || rectangle.x.checked_add(rectangle.width).is_none()
                        || rectangle.y.checked_add(rectangle.height).is_none()
                    {
                        return Err(Error::InvalidGeometry);
                    }
                }
            }
        }
        Ok(count)
    }
}

/// CPU reference: clip primitives and replace covered pixel centers with straight RGBA8.
pub fn cpu_scene(scene: &Scene) -> Result<Frame, Error> {
    cpu_scene_with_cancel(scene, &Cancellation::default())
}

pub fn cpu_scene_with_cancel(scene: &Scene, cancel: &Cancellation) -> Result<Frame, Error> {
    scene.validate()?;
    cancel.check()?;
    let mut frame = cpu_solid(scene.width, scene.height, scene.background)?;
    for primitive in &scene.primitives {
        let Some(bounds) = primitive.bounds(scene.width, scene.height) else {
            continue;
        };
        let right = bounds.x + bounds.width;
        let bottom = bounds.y + bounds.height;
        for y in bounds.y..bottom {
            cancel.check()?;
            let start = ((y * scene.width + bounds.x) * 4) as usize;
            let end = ((y * scene.width + right) * 4) as usize;
            for (offset, pixel) in frame
                .rgba
                .get_mut(start..end)
                .ok_or(Error::InvalidReadback)?
                .chunks_exact_mut(4)
                .enumerate()
            {
                if let Primitive::Stroke(stroke) = primitive
                    && !stroke.covers(
                        f64::from(bounds.x) + offset as f64 + 0.5,
                        f64::from(y) + 0.5,
                    )
                {
                    continue;
                }
                pixel.copy_from_slice(&bounds.color);
            }
        }
    }
    Ok(frame)
}

/// Fit the physical page into a viewport with a bounded margin and an original paper style.
/// Geometry can serve larger native viewports; pixel readbacks remain limited to 2048 per axis.
pub fn page_scene(width: u32, height: u32, page: PageSize) -> Result<Scene, Error> {
    let viewport = PageViewport::new(width, height, page)?;
    let Rectangle {
        x,
        y,
        width: paper_width,
        height: paper_height,
        ..
    } = *viewport.paper();
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
        primitives: rectangles.into_iter().map(Primitive::Rectangle).collect(),
    })
}
