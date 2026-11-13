use crate::{Error, Primitive, Rectangle, Scene, Stroke, page_scene};
use bondline_model::{BondOrder, Page, PageSize, Point2, Radical};
use std::collections::BTreeMap;

/// The same rounded fit used for the paper and authoring pointer conversion.
pub struct PageViewport {
    paper: Rectangle,
    page: PageSize,
}

impl PageViewport {
    pub fn new(width: u32, height: u32, page: PageSize) -> Result<Self, Error> {
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
        // shortcut: extreme aspect ratios occupy at least one display unit; print export needs authored dimensions.
        let paper_width = (normalized_width * scale)
            .round()
            .clamp(1.0, f64::from(available_width)) as u32;
        let paper_height = (normalized_height * scale)
            .round()
            .clamp(1.0, f64::from(available_height)) as u32;
        Ok(Self {
            paper: Rectangle {
                x: (width - paper_width) / 2,
                y: (height - paper_height) / 2,
                width: paper_width,
                height: paper_height,
                color: [196, 196, 196, 255],
            },
            page,
        })
    }

    pub fn paper(&self) -> &Rectangle {
        &self.paper
    }

    pub fn contains(&self, [x, y]: [f32; 2]) -> bool {
        x > self.paper.x as f32
            && x < (self.paper.x + self.paper.width) as f32
            && y > self.paper.y as f32
            && y < (self.paper.y + self.paper.height) as f32
    }

    pub fn canvas_position(&self, point: Point2) -> Result<[f32; 2], Error> {
        let position = [
            f64::from(self.paper.x)
                + point.x_pt() / self.page.width_pt() * f64::from(self.paper.width),
            f64::from(self.paper.y)
                + point.y_pt() / self.page.height_pt() * f64::from(self.paper.height),
        ]
        .map(|value| value as f32);
        if position
            .iter()
            .any(|value| !value.is_finite() || value.abs() > 100_000.0)
        {
            return Err(Error::InvalidStroke);
        }
        Ok(position)
    }

    pub fn document_position(&self, [x, y]: [f32; 2]) -> Result<Point2, bondline_model::Error> {
        Point2::new(
            (f64::from(x) - f64::from(self.paper.x)) / f64::from(self.paper.width)
                * self.page.width_pt(),
            (f64::from(y) - f64::from(self.paper.y)) / f64::from(self.paper.height)
                * self.page.height_pt(),
        )
    }

    pub fn stroke_width(&self) -> f32 {
        // One authored point is an original initial style, not an imported journal preset.
        (f64::from(self.paper.width) / self.page.width_pt())
            .min(f64::from(self.paper.height) / self.page.height_pt()) as f32
    }
}

/// Initial two-carbon fragment preview. Refuse unsupported chemistry instead of hiding it.
pub fn carbon_page_scene(width: u32, height: u32, page: &Page) -> Result<Scene, Error> {
    // shortcut: only independent C-C/three-H fragments; extend with general label/style mapping.
    let viewport = PageViewport::new(width, height, page.size())?;
    let mut scene = page_scene(width, height, page.size())?;
    let count = page
        .fragments()
        .iter()
        .fold(scene.primitives.len(), |count, fragment| {
            count.saturating_add(fragment.bonds().len())
        });
    if count > 4096 {
        return Err(Error::SceneTooComplex);
    }
    let mut labels = Vec::new();
    let mut count = count;
    for fragment in page.fragments() {
        if fragment.atoms().is_empty() && fragment.bonds().is_empty() {
            continue;
        }
        if fragment.atoms().len() != 2 || fragment.bonds().len() != 1 {
            return Err(Error::UnsupportedChemicalContent);
        }
        let mut positions = BTreeMap::new();
        for atom in fragment.atoms() {
            if atom.element.get() != 6
                || atom.charge != 0
                || atom.isotope.is_some()
                || atom.radical != Radical::None
                || atom.explicit_hydrogens != Some(3)
            {
                return Err(Error::UnsupportedChemicalContent);
            }
            let position = atom.position.ok_or(Error::UnsupportedChemicalContent)?;
            positions.insert(atom.id, position);
        }
        for bond in fragment.bonds() {
            if bond.order != BondOrder::Single {
                return Err(Error::UnsupportedChemicalContent);
            }
            let begin = positions
                .get(&bond.begin)
                .ok_or(Error::UnsupportedChemicalContent)?;
            let end = positions
                .get(&bond.end)
                .ok_or(Error::UnsupportedChemicalContent)?;
            let stroke = Stroke {
                begin: viewport.canvas_position(*begin)?,
                end: viewport.canvas_position(*end)?,
                width: viewport.stroke_width(),
                color: [0, 0, 0, 255],
            };
            stroke.validate()?;
            for (position, toward) in [(stroke.begin, stroke.end), (stroke.end, stroke.begin)] {
                let label = crate::label::terminal(
                    position,
                    toward,
                    viewport.stroke_width(),
                    width,
                    height,
                )?;
                count = count.saturating_add(1).saturating_add(label.pixels.len());
                if count > 4096 {
                    return Err(Error::SceneTooComplex);
                }
                labels.push(label);
            }
            scene.primitives.push(Primitive::Stroke(stroke));
        }
    }
    for (index, label) in labels.iter().enumerate() {
        if labels.iter().skip(index + 1).any(|other| {
            label.mask.x < other.mask.x + other.mask.width
                && other.mask.x < label.mask.x + label.mask.width
                && label.mask.y < other.mask.y + other.mask.height
                && other.mask.y < label.mask.y + label.mask.height
        }) {
            return Err(Error::LabelOverlap);
        }
        scene
            .primitives
            .push(Primitive::Rectangle(label.mask.clone()));
    }
    for label in labels {
        scene
            .primitives
            .extend(label.pixels.into_iter().map(Primitive::Rectangle));
    }
    Ok(scene)
}
