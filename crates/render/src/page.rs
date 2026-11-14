use crate::hits::{AtomHit, BondHit, intersects_label, visible_polygon};
use crate::{
    Cancellation, Error, GraphScene, HitTarget, MAX_HIT_REGIONS, Primitive, Rectangle, Scene,
    Stroke, page_scene,
};
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

/// Compatibility entry point; all carbon topology uses the same graph producer.
pub fn carbon_page_scene(width: u32, height: u32, page: &Page) -> Result<Scene, Error> {
    Ok(graph_page_scene(width, height, page)?.into_scene())
}

/// Neutral, positioned elements and Single bonds. Unknown hydrogens are never inferred.
pub fn graph_page_scene(width: u32, height: u32, page: &Page) -> Result<GraphScene, Error> {
    graph_page_scene_with_cancel(width, height, page, &Cancellation::default())
}

pub fn graph_page_scene_with_cancel(
    width: u32,
    height: u32,
    page: &Page,
    cancel: &Cancellation,
) -> Result<GraphScene, Error> {
    cancel.check()?;
    let viewport = PageViewport::new(width, height, page.size())?;
    let mut scene = page_scene(width, height, page.size())?;
    let mut hit_count = 0_usize;
    let mut count = scene.primitives.len();
    for fragment in page.fragments() {
        cancel.check()?;
        hit_count = hit_count
            .saturating_add(fragment.atoms().len())
            .saturating_add(fragment.bonds().len());
        count = count.saturating_add(fragment.bonds().len());
        if hit_count > MAX_HIT_REGIONS || count > 4096 {
            return Err(Error::SceneTooComplex);
        }
    }
    let mut labels = Vec::new();
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    for fragment in page.fragments() {
        cancel.check()?;
        let mut positions = BTreeMap::new();
        let mut neighbours = BTreeMap::<_, (usize, i32)>::new();
        for atom in fragment.atoms() {
            cancel.check()?;
            if atom.charge != 0 || atom.isotope.is_some() || atom.radical != Radical::None {
                return Err(Error::UnsupportedChemicalContent);
            }
            let position = viewport
                .canvas_position(atom.position.ok_or(Error::UnsupportedChemicalContent)?)?;
            positions.insert(atom.id, position);
        }
        for bond in fragment.bonds() {
            cancel.check()?;
            if bond.order != BondOrder::Single {
                return Err(Error::UnsupportedChemicalContent);
            }
            let begin = *positions
                .get(&bond.begin)
                .ok_or(Error::UnsupportedChemicalContent)?;
            let end = *positions
                .get(&bond.end)
                .ok_or(Error::UnsupportedChemicalContent)?;
            let stroke = Stroke {
                begin,
                end,
                width: viewport.stroke_width(),
                color: [0, 0, 0, 255],
            };
            stroke.validate()?;
            for (id, dx) in [
                (bond.begin, f64::from(end[0]) - f64::from(begin[0])),
                (bond.end, f64::from(begin[0]) - f64::from(end[0])),
            ] {
                let entry = neighbours.entry(id).or_default();
                // Original display policy: place H away from the more occupied horizontal side.
                // Counting, rather than summing floats, keeps ties independent of storage order.
                entry.0 += 1;
                entry.1 += if dx > 0.0 {
                    1
                } else if dx < 0.0 {
                    -1
                } else {
                    0
                };
            }
            let polygon = visible_polygon(&stroke, width, height);
            bonds.push(BondHit {
                target: HitTarget::Bond {
                    fragment_id: fragment.id(),
                    bond_id: bond.id,
                },
                endpoints: [bond.begin, bond.end],
                stroke: stroke.clone(),
                polygon,
            });
            scene.primitives.push(Primitive::Stroke(stroke));
        }
        for atom in fragment.atoms() {
            cancel.check()?;
            let position = *positions
                .get(&atom.id)
                .ok_or(Error::UnsupportedChemicalContent)?;
            let neighbours = neighbours.get(&atom.id).copied().unwrap_or_default();
            let label = if atom.element.get() != 6 {
                Some(crate::label::element(
                    position,
                    atom.element,
                    neighbours.0 != 0 && neighbours.1 >= 0,
                    atom.explicit_hydrogens.unwrap_or(0),
                    viewport.stroke_width(),
                    [width, height],
                    cancel,
                )?)
            } else {
                match atom.explicit_hydrogens {
                    Some(hydrogens) => Some(crate::label::carbon(
                        position,
                        neighbours.0 != 0 && neighbours.1 >= 0,
                        hydrogens,
                        viewport.stroke_width(),
                        [width, height],
                        cancel,
                    )?),
                    None if neighbours.0 != 0 => None,
                    None => return Err(Error::UnsupportedChemicalContent),
                }
            };
            let mask = if let Some(label) = label {
                count = count.saturating_add(1).saturating_add(label.pixels.len());
                if count > 4096 {
                    return Err(Error::SceneTooComplex);
                }
                let mask = label.mask.clone();
                labels.push((fragment.id(), atom.id, label));
                Some(mask)
            } else {
                None
            };
            atoms.push(AtomHit {
                target: HitTarget::Atom {
                    fragment_id: fragment.id(),
                    atom_id: atom.id,
                },
                position,
                label: mask,
            });
        }
    }
    for (index, (fragment, atom, label)) in labels.iter().enumerate() {
        cancel.check()?;
        if labels.iter().skip(index+1).any(|(_,_,other)| {
            label.mask.x<other.mask.x+other.mask.width && other.mask.x<label.mask.x+label.mask.width
            && label.mask.y<other.mask.y+other.mask.height && other.mask.y<label.mask.y+label.mask.height
        }) || bonds.iter().any(|bond| {
            let incident=matches!(bond.target,HitTarget::Bond {fragment_id,..} if fragment_id==*fragment) && bond.endpoints.contains(atom);
            !incident && intersects_label(bond,&label.mask)
        }) { return Err(Error::LabelOverlap); }
        scene
            .primitives
            .push(Primitive::Rectangle(label.mask.clone()));
    }
    for (_, _, label) in labels {
        cancel.check()?;
        scene
            .primitives
            .extend(label.pixels.into_iter().map(Primitive::Rectangle));
    }
    cancel.check()?;
    Ok(GraphScene {
        scene,
        atoms,
        bonds,
    })
}
