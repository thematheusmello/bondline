use crate::hits::{AtomHit, BondHit, intersects_label, visible_polygon};
#[cfg(test)]
use crate::page_scene;
use crate::{
    Cancellation, Error, GraphScene, HitTarget, MAX_HIT_REGIONS, Primitive, Rectangle, Scene,
    Stroke,
};
use bondline_model::{
    AtomId, BondOrder, DoublePosition, DrawingSettings, DrawingStyle, Page, PageSize, Point2,
    Radical,
};
use std::collections::BTreeMap;

/// The same rounded fit used for the paper and authoring pointer conversion.
#[derive(Clone)]
pub struct PageViewport {
    paper: Rectangle,
    page: PageSize,
    exact_scale: Option<f64>,
    raster: Option<RasterTransform>,
    style: DrawingStyle,
    settings: DrawingSettings,
}

#[derive(Clone)]
struct RasterTransform {
    origin: [f64; 2],
    axes: [f64; 2],
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
            exact_scale: None,
            raster: None,
            style: DrawingStyle::default(),
            settings: DrawingSettings::default(),
        })
    }

    pub(crate) fn for_zoom(page: PageSize, scale: f64) -> Result<(Self, [u32; 2]), Error> {
        let extents = [page.width_pt() * scale, page.height_pt() * scale];
        if !scale.is_finite()
            || scale <= 0.0
            || extents
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0 || value.ceil() > 100_000.0)
        {
            return Err(Error::InvalidViewport);
        }
        let [width, height] = extents.map(|value| value.ceil() as u32);
        // This solves the legacy margin=min(32,source_width/8,source_height/8).
        let margin = 32.min(width / 6).min(height / 6);
        let source = [width + margin * 2, height + margin * 2];
        if source.iter().any(|value| !(1..=100_000).contains(value)) {
            return Err(Error::InvalidViewport);
        }
        Ok((
            Self {
                paper: Rectangle {
                    x: margin,
                    y: margin,
                    width,
                    height,
                    color: [196, 196, 196, 255],
                },
                page,
                exact_scale: Some(scale),
                raster: None,
                style: DrawingStyle::default(),
                settings: DrawingSettings::default(),
            },
            source,
        ))
    }

    /// Fresh physical geometry, preserving the logical Fit's rounded axes or exact Zoom.
    /// Fractional camera phase enters glyph generation; later projection stays integral.
    pub(crate) fn at_density(&self, density: f64, phase: [f64; 2]) -> Result<Self, Error> {
        let [left, top, right, bottom] = self.paper_bounds();
        let origin = [left * density + phase[0], top * density + phase[1]];
        let size = [(right - left) * density, (bottom - top) * density];
        let ends = [origin[0] + size[0], origin[1] + size[1]];
        if !density.is_finite()
            || density <= 0.0
            || origin.iter().any(|v| !v.is_finite() || *v < 0.0)
            || size.iter().any(|v| !v.is_finite() || *v <= 0.0)
            || ends.iter().any(|v| !v.is_finite() || v.ceil() > 100_000.0)
        {
            return Err(Error::InvalidViewport);
        }
        let [x, y] = origin.map(|value| value.floor() as u32);
        let [right, bottom] = ends.map(|value| value.ceil() as u32);
        Ok(Self {
            paper: Rectangle {
                x,
                y,
                width: right.checked_sub(x).ok_or(Error::InvalidGeometry)?,
                height: bottom.checked_sub(y).ok_or(Error::InvalidGeometry)?,
                color: self.paper.color,
            },
            page: self.page,
            exact_scale: None,
            raster: Some(RasterTransform {
                origin,
                axes: [
                    size[0] / self.page.width_pt(),
                    size[1] / self.page.height_pt(),
                ],
            }),
            style: self.style,
            settings: self.settings,
        })
    }

    pub fn paper(&self) -> &Rectangle {
        &self.paper
    }

    pub fn contains(&self, [x, y]: [f32; 2]) -> bool {
        if self.exact_scale.is_some() || self.raster.is_some() {
            let [left, top, right, bottom] = self.paper_bounds();
            return f64::from(x) > left
                && f64::from(x) < right
                && f64::from(y) > top
                && f64::from(y) < bottom;
        }
        x > self.paper.x as f32
            && x < (self.paper.x + self.paper.width) as f32
            && y > self.paper.y as f32
            && y < (self.paper.y + self.paper.height) as f32
    }

    pub fn canvas_position(&self, point: Point2) -> Result<[f32; 2], Error> {
        let position = self.position_f64(point).map(|value| value as f32);
        if position
            .iter()
            .any(|value| !value.is_finite() || value.abs() > 100_000.0)
        {
            return Err(Error::InvalidStroke);
        }
        Ok(position)
    }

    pub(crate) fn position_f64(&self, point: Point2) -> [f64; 2] {
        if let Some(raster) = &self.raster {
            return [
                raster.origin[0] + point.x_pt() * raster.axes[0],
                raster.origin[1] + point.y_pt() * raster.axes[1],
            ];
        }
        if let Some(scale) = self.exact_scale {
            return [
                f64::from(self.paper.x) + point.x_pt() * scale,
                f64::from(self.paper.y) + point.y_pt() * scale,
            ];
        }
        [
            f64::from(self.paper.x)
                + point.x_pt() / self.page.width_pt() * f64::from(self.paper.width),
            f64::from(self.paper.y)
                + point.y_pt() / self.page.height_pt() * f64::from(self.paper.height),
        ]
    }

    pub fn document_position(&self, [x, y]: [f32; 2]) -> Result<Point2, bondline_model::Error> {
        self.document_position_f64([f64::from(x), f64::from(y)])
    }

    pub(crate) fn document_position_f64(
        &self,
        [x, y]: [f64; 2],
    ) -> Result<Point2, bondline_model::Error> {
        if let Some(raster) = &self.raster {
            return Point2::new(
                (x - raster.origin[0]) / raster.axes[0],
                (y - raster.origin[1]) / raster.axes[1],
            );
        }
        if let Some(scale) = self.exact_scale {
            return Point2::new(
                (x - f64::from(self.paper.x)) / scale,
                (y - f64::from(self.paper.y)) / scale,
            );
        }
        Point2::new(
            (x - f64::from(self.paper.x)) / f64::from(self.paper.width) * self.page.width_pt(),
            (y - f64::from(self.paper.y)) / f64::from(self.paper.height) * self.page.height_pt(),
        )
    }

    pub fn stroke_width(&self) -> f32 {
        self.scale() as f32
    }

    pub(crate) fn with_drawing_style(mut self, style: DrawingStyle) -> Self {
        self.style = style;
        self
    }

    pub(crate) fn with_drawing_settings(mut self, settings: DrawingSettings) -> Self {
        self.settings = settings;
        self
    }

    fn bond_spacing(&self) -> Option<f64> {
        self.style.bond_spacing_percent().map(|percent| {
            // Original own/native fallback, not a foreign style-sheet default.
            percent * self.settings.bond_length_pt().unwrap_or(48.0) / 100.0 * self.scale()
        })
    }

    pub(crate) fn ink_width(&self) -> f32 {
        (self.style.line_width_pt().unwrap_or(1.0) * self.scale()) as f32
    }

    pub(crate) fn scale(&self) -> f64 {
        // One authored point is an original initial style, not an imported journal preset.
        if let Some(raster) = &self.raster {
            return raster.axes[0].min(raster.axes[1]);
        }
        if let Some(scale) = self.exact_scale {
            return scale;
        }
        (f64::from(self.paper.width) / self.page.width_pt())
            .min(f64::from(self.paper.height) / self.page.height_pt())
    }

    pub(crate) fn paper_bounds(&self) -> [f64; 4] {
        if let Some(raster) = &self.raster {
            return [
                raster.origin[0],
                raster.origin[1],
                raster.origin[0] + self.page.width_pt() * raster.axes[0],
                raster.origin[1] + self.page.height_pt() * raster.axes[1],
            ];
        }
        let [left, top] = [f64::from(self.paper.x), f64::from(self.paper.y)];
        let [width, height] = self.exact_scale.map_or(
            [f64::from(self.paper.width), f64::from(self.paper.height)],
            |scale| [self.page.width_pt() * scale, self.page.height_pt() * scale],
        );
        [left, top, left + width, top + height]
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
    validate_display_hydrogens(page, counts, cancel)?;
    graph_scene(width, height, page, Some(counts), cancel)
}

pub(crate) fn validate_display_hydrogens(
    page: &Page,
    counts: &BTreeMap<AtomId, u16>,
    cancel: &Cancellation,
) -> Result<(), Error> {
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
    Ok(())
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
    graph_scene_in_viewport(width, height, page, counts, cancel, &viewport)
}

pub(crate) fn graph_scene_in_viewport(
    width: u32,
    height: u32,
    page: &Page,
    counts: Option<&BTreeMap<AtomId, u16>>,
    cancel: &Cancellation,
    viewport: &PageViewport,
) -> Result<GraphScene, Error> {
    cancel.check()?;
    let mut scene = crate::scene::page_scene_in_viewport(width, height, viewport);
    let mut hit_count = 0_usize;
    let mut count = scene.primitives.len();
    let mut has_multiple = false;
    for fragment in page.fragments() {
        cancel.check()?;
        hit_count = hit_count
            .saturating_add(fragment.atoms().len())
            .saturating_add(fragment.bonds().len());
        if hit_count > MAX_HIT_REGIONS {
            return Err(Error::SceneTooComplex);
        }
        for bond in fragment.bonds() {
            cancel.check()?;
            match (bond.order, bond.double_position) {
                (_, DoublePosition::Unspecified)
                | (
                    BondOrder::Double,
                    DoublePosition::Center | DoublePosition::Left | DoublePosition::Right,
                ) => {}
                _ => return Err(Error::UnsupportedChemicalContent),
            }
            has_multiple |= matches!(bond.order, BondOrder::Double | BondOrder::Triple);
            count = count.saturating_add(match bond.order {
                BondOrder::Double => 2,
                BondOrder::Triple => 3,
                _ => 1,
            });
        }
        if hit_count > MAX_HIT_REGIONS || count > 4096 {
            return Err(Error::SceneTooComplex);
        }
    }
    let mut raster_budget = crate::text::RasterBudget::default();
    let mut authored_range: Option<[f32; 2]> = None;
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
            if atom.authored_label.is_some() {
                crate::validate_atom_label(atom)?;
            }
            let position = viewport
                .canvas_position(atom.position.ok_or(Error::UnsupportedChemicalContent)?)?;
            positions.insert(atom.id, position);
        }
        let mut adjoining = BTreeMap::<AtomId, Vec<(AtomId, [f32; 2])>>::new();
        if fragment.bonds().iter().any(|bond| {
            matches!(
                bond.double_position,
                DoublePosition::Left | DoublePosition::Right
            )
        }) {
            for bond in fragment.bonds() {
                cancel.check()?;
                let begin = *positions
                    .get(&bond.begin)
                    .ok_or(Error::UnsupportedChemicalContent)?;
                let end = *positions
                    .get(&bond.end)
                    .ok_or(Error::UnsupportedChemicalContent)?;
                adjoining
                    .entry(bond.begin)
                    .or_default()
                    .push((bond.end, end));
                adjoining
                    .entry(bond.end)
                    .or_default()
                    .push((bond.begin, begin));
            }
        }
        for bond in fragment.bonds() {
            cancel.check()?;
            if !matches!(
                bond.order,
                BondOrder::Single | BondOrder::Double | BondOrder::Triple
            ) {
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
                width: viewport.ink_width(),
                color: [0, 0, 0, 255],
            };
            if !viewport.style.is_empty()
                && (!stroke.width.is_finite() || stroke.width <= 0.0 || stroke.width > 100_000.0)
            {
                return Err(Error::BondScale);
            }
            stroke.validate()?;
            let strokes = if matches!(
                bond.double_position,
                DoublePosition::Left | DoublePosition::Right
            ) {
                let neighbours = |id, other| {
                    adjoining
                        .get(&id)
                        .into_iter()
                        .flatten()
                        .filter_map(|(neighbour, point)| (*neighbour != other).then_some(*point))
                        .collect::<Vec<_>>()
                };
                crate::inset::strokes_with_unit(
                    &stroke,
                    bond.double_position,
                    &neighbours(bond.begin, bond.end),
                    &neighbours(bond.end, bond.begin),
                    cancel,
                    viewport.stroke_width(),
                    viewport.bond_spacing(),
                )?
            } else if let Some(spacing) = viewport.bond_spacing() {
                crate::stroke::centered_bond_strokes_with_spacing(
                    &stroke,
                    bond.order,
                    viewport.stroke_width(),
                    spacing,
                )?
            } else {
                crate::stroke::centered_bond_strokes_with_unit(
                    &stroke,
                    bond.order,
                    viewport.stroke_width(),
                )?
            };
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
            let polygons = strokes
                .iter()
                .map(|stroke| visible_polygon(stroke, width, height))
                .collect();
            bonds.push(BondHit {
                target: HitTarget::Bond {
                    fragment_id: fragment.id(),
                    bond_id: bond.id,
                },
                endpoints: [bond.begin, bond.end],
                double_position: bond.double_position,
                stroke: stroke.clone(),
                strokes: strokes.clone(),
                polygons,
            });
            scene
                .primitives
                .extend(strokes.into_iter().map(Primitive::Stroke));
        }
        let carbon_labels = crate::skeletal::labels(fragment, counts, cancel)?;
        for atom in fragment.atoms() {
            cancel.check()?;
            let position = *positions
                .get(&atom.id)
                .ok_or(Error::UnsupportedChemicalContent)?;
            let neighbours = neighbours.get(&atom.id).copied().unwrap_or_default();
            let carbon_label = carbon_labels.get(&atom.id);
            let resolved_h =
                if let Some(crate::skeletal::CarbonLabel::Visible(hydrogens)) = carbon_label {
                    Some(*hydrogens)
                } else if let Some(counts) = counts {
                    Some(
                        *counts
                            .get(&atom.id)
                            .ok_or(Error::UnsupportedChemicalContent)?,
                    )
                } else if has_multiple || (neighbours.0 >= 2 && atom.explicit_hydrogens.is_some()) {
                    atom.explicit_hydrogens
                        .or_else(|| (atom.element.get() != 6).then_some(0))
                } else {
                    None
                };
            let label = if atom.authored_label.is_some() {
                let label =
                    crate::authored_label::render(atom, viewport, &mut raster_budget, cancel)?;
                authored_range = Some(match authored_range {
                    Some([min, max]) => [min.min(label.em_px), max.max(label.em_px)],
                    None => [label.em_px; 2],
                });
                Some(label)
            } else if matches!(carbon_label, Some(crate::skeletal::CarbonLabel::Hidden)) {
                None
            } else if let Some(hydrogens) = resolved_h {
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
                    viewport,
                    [width, height],
                    counts.is_some(),
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
        if has_multiple {
            let collision = label.masks().any(|mask| {
                labels.iter().skip(index+1).any(|(_,_,other)| other.masks().any(|other| {
                    mask.x < other.x + other.width && other.x < mask.x + mask.width
                        && mask.y < other.y + other.height && other.y < mask.y + mask.height
                })) || bonds.iter().any(|bond| {
                    let incident = matches!(bond.target, HitTarget::Bond {fragment_id,..} if fragment_id == *fragment) && bond.endpoints.contains(atom);
                    !incident && intersects_label(bond,mask)
                })
            });
            if collision {
                return Err(Error::LabelOverlap);
            }
        } else {
            if labels.iter().skip(index+1).any(|(_,_,other)| {
            label.mask.x<other.mask.x+other.mask.width && other.mask.x<label.mask.x+label.mask.width
            && label.mask.y<other.mask.y+other.mask.height && other.mask.y<label.mask.y+label.mask.height
        }) || bonds.iter().any(|bond| {
            let incident=matches!(bond.target,HitTarget::Bond {fragment_id,..} if fragment_id==*fragment) && bond.endpoints.contains(atom);
            !incident && intersects_label(bond,&label.mask)
        }) { return Err(Error::LabelOverlap); }
        }
        scene
            .primitives
            .extend(label.masks().cloned().map(Primitive::Rectangle));
    }
    if counts.is_some() || authored_range.is_some() {
        for bond in &bonds {
            cancel.check()?;
            if bond.strokes.len() != 1 {
                continue;
            }
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
    if has_multiple {
        let masks = labels
            .iter()
            .flat_map(|(_, _, label)| label.masks())
            .collect::<Vec<_>>();
        let ink = bonds
            .iter()
            .flat_map(|bond| &bond.strokes)
            .collect::<Vec<_>>();
        let mut joined_axes = BTreeMap::<HitTarget, Vec<&Stroke>>::new();
        for bond in &bonds {
            if bond.strokes.len() == 1 {
                continue;
            }
            let inset_gap = if bond.is_sided() {
                Some(crate::inset::Gap::new(
                    &bond.stroke,
                    bond.strokes.get(1).ok_or(Error::InvalidGeometry)?,
                    bond.double_position,
                )?)
            } else {
                None
            };
            for other in &bonds {
                cancel.check()?;
                if bond.target == other.target {
                    continue;
                }
                let shared = bond
                    .endpoints
                    .iter()
                    .find(|id| other.endpoints.contains(id));
                if let Some(shared) = shared {
                    if bond.is_sided()
                        && other.natural_axis(0)
                        && (!other.is_sided() || bond.target < other.target)
                    {
                        let first = bond.strokes.first().ok_or(Error::InvalidGeometry)?;
                        let second = other.strokes.first().ok_or(Error::InvalidGeometry)?;
                        joined_axes.entry(bond.target).or_default().push(second);
                        joined_axes.entry(other.target).or_default().push(first);
                    }
                    let anchor = if bond.endpoints.first() == Some(shared) {
                        bond.stroke.begin
                    } else {
                        bond.stroke.end
                    };
                    let anchor_mask = anchor_pixel_mask(anchor, [width, height]);
                    let mut junction_masks = masks.clone();
                    if let Some(mask) = anchor_mask.as_ref() {
                        junction_masks.push(mask);
                    }
                    for (first_index, first) in bond.strokes.iter().enumerate() {
                        for (second_index, second) in other.strokes.iter().enumerate() {
                            if bond.is_sided()
                                && bond.natural_axis(first_index)
                                && other.natural_axis(second_index)
                            {
                                continue;
                            }
                            if first.has_visible_pixel(
                                [width, height],
                                &junction_masks,
                                Some(second),
                                &[],
                                cancel,
                            )? {
                                return Err(Error::LabelOverlap);
                            }
                        }
                    }
                } else if let Some(gap) = &inset_gap {
                    for component in &other.strokes {
                        for stroke in &bond.strokes {
                            if stroke.has_visible_pixel(
                                [width, height],
                                &masks,
                                Some(component),
                                &[],
                                cancel,
                            )? {
                                return Err(Error::LabelOverlap);
                            }
                        }
                        if gap.has_visible_pixel(
                            [width, height],
                            &masks,
                            Some(component),
                            &[],
                            cancel,
                        )? {
                            return Err(Error::LabelOverlap);
                        }
                    }
                } else {
                    // Original centered profile reserves the full inter-line corridor.
                    let envelope = Stroke {
                        width: if let Some(spacing) = viewport.bond_spacing() {
                            (spacing * (bond.strokes.len() - 1) as f64
                                + f64::from(bond.stroke.width)) as f32
                        } else if bond.stroke.width == viewport.stroke_width() {
                            bond.stroke.width * if bond.strokes.len() == 2 { 4.0 } else { 7.0 }
                        } else {
                            3.0 * viewport.stroke_width() * (bond.strokes.len() - 1) as f32
                                + bond.stroke.width
                        },
                        ..bond.stroke.clone()
                    };
                    for component in &other.strokes {
                        if envelope.has_visible_pixel(
                            [width, height],
                            &masks,
                            Some(component),
                            &[],
                            cancel,
                        )? {
                            return Err(Error::LabelOverlap);
                        }
                    }
                }
            }
            for stroke in &bond.strokes {
                if !stroke.has_visible_pixel([width, height], &[], None, &[], cancel)? {
                    return Err(Error::BondScale);
                }
                if !stroke.has_visible_pixel([width, height], &masks, None, &[], cancel)? {
                    return Err(Error::LabelOverlap);
                }
            }
            if let Some(gap) = &inset_gap {
                if !gap.has_visible_pixel([width, height], &[], None, &ink, cancel)? {
                    return Err(Error::BondScale);
                }
                if !gap.has_visible_pixel([width, height], &masks, None, &ink, cancel)? {
                    return Err(Error::LabelOverlap);
                }
            } else {
                for pair in bond.strokes.windows(2) {
                    let (Some(a), Some(b)) = (pair.first(), pair.get(1)) else {
                        return Err(Error::InvalidGeometry);
                    };
                    let gap = Stroke {
                        begin: [
                            (a.begin[0] + b.begin[0]) / 2.0,
                            (a.begin[1] + b.begin[1]) / 2.0,
                        ],
                        end: [(a.end[0] + b.end[0]) / 2.0, (a.end[1] + b.end[1]) / 2.0],
                        width: if let Some(spacing) = viewport.bond_spacing() {
                            (spacing - f64::from(bond.stroke.width)) as f32
                        } else if bond.stroke.width == viewport.stroke_width() {
                            bond.stroke.width * 2.0
                        } else {
                            3.0 * viewport.stroke_width() - bond.stroke.width
                        },
                        color: [255; 4],
                    };
                    gap.validate()?;
                    if !gap.has_visible_pixel([width, height], &[], None, &ink, cancel)? {
                        return Err(Error::BondScale);
                    }
                    if !gap.has_visible_pixel([width, height], &masks, None, &ink, cancel)? {
                        return Err(Error::LabelOverlap);
                    }
                }
            }
        }
        for bond in &bonds {
            cancel.check()?;
            let Some(joined) = joined_axes.get(&bond.target) else {
                continue;
            };
            if !bond
                .strokes
                .first()
                .ok_or(Error::InvalidGeometry)?
                .has_visible_pixel([width, height], &masks, None, joined, cancel)?
            {
                return Err(Error::LabelOverlap);
            }
        }
    }
    if !viewport.style.is_empty() || authored_range.is_some() {
        let masks = labels
            .iter()
            .flat_map(|(_, _, label)| label.masks())
            .collect::<Vec<_>>();
        for stroke in bonds
            .iter()
            .filter(|bond| bond.strokes.len() == 1)
            .flat_map(|bond| &bond.strokes)
        {
            if !stroke.has_visible_pixel([width, height], &[], None, &[], cancel)? {
                return Err(Error::BondScale);
            }
            if !stroke.has_visible_pixel([width, height], &masks, None, &[], cancel)? {
                return Err(Error::LabelOverlap);
            }
        }
    }
    let label_em_px = labels
        .iter()
        .map(|(_, _, label)| label.em_px)
        .reduce(f32::min);
    for (_, _, label) in labels {
        cancel.check()?;
        scene
            .primitives
            .extend(label.pixels.into_iter().map(Primitive::Rectangle));
    }
    cancel.check()?;
    if counts.is_some() || !viewport.style.is_empty() || authored_range.is_some() {
        validate_paper_ink(
            &scene,
            viewport,
            crate::scene::page_scene_in_viewport(width, height, viewport)
                .primitives
                .len(),
            cancel,
        )?;
    }
    let captions = crate::text::append(page, viewport, &mut scene, &mut raster_budget, cancel)?;
    let text_em_range_px = match (authored_range, captions) {
        (Some([a, b]), Some([c, d])) => Some([a.min(c), b.max(d)]),
        (a, b) => a.or(b),
    };
    Ok(GraphScene {
        label_em_px,
        text_em_range_px,
        scene,
        atoms,
        bonds,
    })
}

/// Pixel centers in the shared anchor's closed half-pixel square. This is a
/// congestion exception only; it never paints a mask or changes source geometry.
fn anchor_pixel_mask([x, y]: [f32; 2], [width, height]: [u32; 2]) -> Option<Rectangle> {
    let left = (f64::from(x) - 1.0).ceil().max(0.0);
    let right = f64::from(x).floor().min(f64::from(width.checked_sub(1)?));
    let top = (f64::from(y) - 1.0).ceil().max(0.0);
    let bottom = f64::from(y).floor().min(f64::from(height.checked_sub(1)?));
    if left > right || top > bottom {
        return None;
    }
    Some(Rectangle {
        x: left as u32,
        y: top as u32,
        width: (right - left + 1.0) as u32,
        height: (bottom - top + 1.0) as u32,
        color: [255; 4],
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
    require_paper: bool,
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
        Layout::ShiftedAbove(-1),
        Layout::ShiftedAbove(1),
        Layout::ShiftedBelow(-1),
        Layout::ShiftedBelow(1),
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
        if require_paper
            && !label
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
    let [left, top, right, bottom] = viewport.paper_bounds();
    x >= left + 1.0 && x <= right - 1.0 && y >= top + 1.0 && y <= bottom - 1.0
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
    fn junction_mask_matches_the_closed_anchor_square_at_pixel_centers() {
        for anchor in [
            [20.0, 20.0],
            [20.125, 20.95],
            [20.5, 20.5],
            [-0.5, 1.0],
            [63.5, 64.0],
            [80.0, 20.0],
        ] {
            let mask = anchor_pixel_mask(anchor, [64, 64]);
            for y in 0..64 {
                for x in 0..64 {
                    let expected = (f64::from(x) + 0.5 - f64::from(anchor[0])).abs() <= 0.5
                        && (f64::from(y) + 0.5 - f64::from(anchor[1])).abs() <= 0.5;
                    let actual = mask.as_ref().is_some_and(|r| {
                        x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height
                    });
                    assert_eq!(actual, expected, "anchor{anchor:?} pixel{x},{y}");
                }
            }
        }
    }

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
