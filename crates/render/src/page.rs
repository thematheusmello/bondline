use crate::hits::{AtomHit, BondHit, intersects_label, visible_polygon};
use crate::{
    Cancellation, Error, GraphScene, HitTarget, MAX_HIT_REGIONS, Primitive, Rectangle, Scene,
    Stroke, page_scene,
};
use bondline_model::{AtomId, BondOrder, Page, PageSize, Point2, Radical};
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
    graph_scene(width, height, page, None, cancel)
}

/// Display-only resolved counts: exact atom coverage, with authored values authoritative.
/// No valence is chosen here, and the Page (including unknown authored H) stays untouched.
/// Unlike the literal compatibility entry, all ink must fit the physical paper interior.
pub fn graph_page_scene_with_display_hydrogens(
    width: u32,
    height: u32,
    page: &Page,
    counts: &BTreeMap<AtomId, u16>,
    cancel: &Cancellation,
) -> Result<GraphScene, Error> {
    cancel.check()?;
    let mut count = 0_usize;
    for atom in page
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.atoms())
    {
        cancel.check()?;
        count = count.saturating_add(1);
        let value = counts
            .get(&atom.id)
            .ok_or(Error::UnsupportedChemicalContent)?;
        if atom
            .explicit_hydrogens
            .is_some_and(|authored| authored != *value)
        {
            return Err(Error::UnsupportedChemicalContent);
        }
    }
    if counts.len() != count {
        return Err(Error::UnsupportedChemicalContent);
    }
    graph_scene(width, height, page, Some(counts), cancel)
}

fn graph_scene(
    width: u32,
    height: u32,
    page: &Page,
    counts: Option<&BTreeMap<AtomId, u16>>,
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
            let label = if let Some(counts) = counts {
                let hydrogens = *counts
                    .get(&atom.id)
                    .ok_or(Error::UnsupportedChemicalContent)?;
                let preferred_left = neighbours.0 != 0 && neighbours.1 >= 0;
                let incident = bonds.iter().filter(|bond| {
                    matches!(bond.target, HitTarget::Bond { fragment_id, .. } if fragment_id == fragment.id())
                        && bond.endpoints.contains(&atom.id)
                }).collect::<Vec<_>>();
                Some(resolved_label(
                    position,
                    atom.element,
                    preferred_left,
                    hydrogens,
                    &incident,
                    &viewport,
                    [width, height],
                    cancel,
                )?)
            } else if atom.element.get() != 6 {
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
            let mut supplement = None;
            let mask = if let Some(label) = label {
                let masks = label.masks().collect::<Vec<_>>();
                count = count
                    .saturating_add(masks.len())
                    .saturating_add(label.pixels.len());
                if count > 4096 {
                    return Err(Error::SceneTooComplex);
                }
                let mask = masks
                    .first()
                    .ok_or(Error::InvalidGeometry)?
                    .to_owned()
                    .clone();
                supplement = masks.get(1).map(|mask| (*mask).clone());
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
                supplement,
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
            .extend(label.masks().cloned().map(Primitive::Rectangle));
    }
    if counts.is_some() {
        for bond in &bonds {
            cancel.check()?;
            let mut covered = labels
                .iter()
                .flat_map(|(_, _, label)| label.masks())
                .filter_map(|mask| segment_interval(&bond.stroke, mask))
                .collect::<Vec<_>>();
            covered.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut end = 0.0_f64;
            for (start, stop) in covered {
                if start > end {
                    break;
                }
                end = end.max(stop);
            }
            if end >= 1.0 {
                return Err(Error::LabelOverlap);
            }
        }
    }
    for (_, _, label) in labels {
        cancel.check()?;
        scene
            .primitives
            .extend(label.pixels.into_iter().map(Primitive::Rectangle));
    }
    cancel.check()?;
    if counts.is_some() {
        validate_paper_ink(
            &scene,
            &viewport,
            page_scene(width, height, page.size())?.primitives.len(),
            cancel,
        )?;
    }
    Ok(GraphScene {
        scene,
        atoms,
        bonds,
    })
}

#[allow(clippy::too_many_arguments)]
fn resolved_label(
    position: [f32; 2],
    element: bondline_model::Element,
    preferred_left: bool,
    hydrogens: u16,
    incident: &[&BondHit],
    viewport: &PageViewport,
    dimensions: [u32; 2],
    cancel: &Cancellation,
) -> Result<crate::label::Label, Error> {
    use crate::label::Layout;
    if element.get() == 1 && hydrogens != 0 {
        return Err(Error::UnsupportedChemicalContent);
    }
    let mut last_error = Error::LabelOverlap;
    for layout in [
        Layout::Horizontal(preferred_left),
        Layout::Horizontal(!preferred_left),
        Layout::Above,
        Layout::Below,
    ] {
        cancel.check()?;
        let label = match crate::label::resolved_element(
            position,
            element,
            layout,
            hydrogens,
            viewport.stroke_width(),
            dimensions,
            cancel,
        ) {
            Ok(label) => label,
            Err(Error::LabelPlacement) => {
                last_error = Error::LabelPlacement;
                continue;
            }
            Err(error) => return Err(error),
        };
        if !label
            .masks()
            .chain(&label.pixels)
            .all(|rect| rect_inside_paper(rect, viewport))
        {
            last_error = Error::LabelPlacement;
            continue;
        }
        let clear = incident.iter().all(|bond| {
            // Keep the literal entry's horizontal padding when its H is clear.
            // Stacked masks are separate, so the gap between lines cannot hide ink.
            !label
                .hydrogen_mask
                .as_ref()
                .is_some_and(|mask| intersects_label(bond, mask))
        });
        if clear {
            return Ok(label);
        }
    }
    Err(last_error)
}

/// Center-line clipping against a label's closed padding rectangle.
fn segment_interval(stroke: &Stroke, mask: &Rectangle) -> Option<(f64, f64)> {
    let mut entry = 0.0_f64;
    let mut exit = 1.0_f64;
    for (a, b, lower, upper) in [
        (stroke.begin[0], stroke.end[0], mask.x, mask.x + mask.width),
        (stroke.begin[1], stroke.end[1], mask.y, mask.y + mask.height),
    ] {
        let (a, delta, lower, upper) = (
            f64::from(a),
            f64::from(b) - f64::from(a),
            f64::from(lower),
            f64::from(upper),
        );
        if delta == 0.0 {
            if a < lower || a > upper {
                return None;
            }
        } else {
            let (first, last) = ((lower - a) / delta, (upper - a) / delta);
            entry = entry.max(first.min(last));
            exit = exit.min(first.max(last));
            if entry > exit {
                return None;
            }
        }
    }
    Some((entry, exit))
}

fn rect_inside_paper(rect: &Rectangle, viewport: &PageViewport) -> bool {
    point_inside_paper([f64::from(rect.x), f64::from(rect.y)], viewport)
        && point_inside_paper(
            [
                f64::from(rect.x) + f64::from(rect.width),
                f64::from(rect.y) + f64::from(rect.height),
            ],
            viewport,
        )
}

fn point_inside_paper([x, y]: [f64; 2], viewport: &PageViewport) -> bool {
    let paper = viewport.paper();
    x >= f64::from(paper.x) + 1.0
        && x <= f64::from(paper.x) + f64::from(paper.width) - 1.0
        && y >= f64::from(paper.y) + 1.0
        && y <= f64::from(paper.y) + f64::from(paper.height) - 1.0
}

fn validate_paper_ink(
    scene: &Scene,
    viewport: &PageViewport,
    prefix: usize,
    cancel: &Cancellation,
) -> Result<(), Error> {
    for primitive in scene.primitives.iter().skip(prefix) {
        cancel.check()?;
        let contained = match primitive {
            Primitive::Rectangle(rect) => rect_inside_paper(rect, viewport),
            Primitive::Stroke(stroke) => stroke
                .corners()?
                .iter()
                .all(|point| point_inside_paper(point.map(f64::from), viewport)),
        };
        if !contained {
            return Err(Error::LabelPlacement);
        }
    }
    Ok(())
}

#[cfg(test)]
mod display_bounds_tests {
    use super::*;

    #[test]
    fn paper_check_uses_actual_glyph_rectangles_and_butt_cap_corners() -> Result<(), Error> {
        let viewport = PageViewport::new(
            576,
            576,
            PageSize::new(256.0, 256.0).map_err(|_| Error::InvalidGeometry)?,
        )?;
        let mut scene = page_scene(576, 576, viewport.page)?;
        let prefix = scene.primitives.len();
        let cancel = Cancellation::default();
        // Both centers lie inside the paper; the upper corners of this width2
        // stroke extend past its interior y=33 boundary.
        scene.primitives.push(Primitive::Stroke(Stroke {
            begin: [64.0, 33.5],
            end: [128.0, 33.5],
            width: 2.0,
            color: [0, 0, 0, 255],
        }));
        assert!(matches!(
            validate_paper_ink(&scene, &viewport, prefix, &cancel),
            Err(Error::LabelPlacement)
        ));
        scene.primitives.pop();
        scene.primitives.push(Primitive::Stroke(Stroke {
            begin: [33.0, 34.0],
            end: [128.0, 34.0],
            width: 2.0,
            color: [0, 0, 0, 255],
        }));
        validate_paper_ink(&scene, &viewport, prefix, &cancel)?;
        scene.primitives.push(Primitive::Rectangle(Rectangle {
            x: 32,
            y: 128,
            width: 1,
            height: 1,
            color: [0, 0, 0, 255],
        }));
        assert!(matches!(
            validate_paper_ink(&scene, &viewport, prefix, &cancel),
            Err(Error::LabelPlacement)
        ));
        scene.primitives.pop();
        scene.primitives.push(Primitive::Rectangle(Rectangle {
            x: 33,
            y: 128,
            width: 1,
            height: 1,
            color: [0, 0, 0, 255],
        }));
        validate_paper_ink(&scene, &viewport, prefix, &cancel)?;
        cancel.cancel();
        assert!(matches!(
            validate_paper_ink(&scene, &viewport, prefix, &cancel),
            Err(Error::Cancelled)
        ));
        Ok(())
    }
}
