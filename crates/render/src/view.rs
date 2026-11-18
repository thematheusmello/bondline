use crate::{
    Cancellation, Error, GraphScene, MAX_HIT_REGIONS, PageViewport, Primitive, Rectangle, Stroke,
};
use bondline_model::{AtomId, DrawingSettings, DrawingStyle, Page, PageSize, Point2};
use std::collections::BTreeMap;

/// A logical-unit camera. Fit retains the legacy rounded viewport exactly; explicit
/// zoom uses one authored point per logical unit at 100%, independent of device DPI.
/// The integer camera translation never resamples glyph pixels. Its centered point
/// can differ from the exact viewport center by at most half a logical unit per axis.
#[derive(Clone)]
pub struct PageView {
    page: PageSize,
    viewport: PageViewport,
    source: [u32; 2],
    destination: [u32; 2],
    offset: [i64; 2],
    percent: f64,
    fit: bool,
    density: Option<f64>,
}

impl PageView {
    /// Authored ink and separation are independent of this camera's label scale.
    pub fn with_drawing_style(mut self, style: DrawingStyle) -> Self {
        self.viewport = self.viewport.with_drawing_style(style);
        self
    }

    /// Preferred bond length is used by explicit relative spacing, never to move atoms.
    pub fn with_drawing_settings(mut self, settings: DrawingSettings) -> Self {
        self.viewport = self.viewport.with_drawing_settings(settings);
        self
    }

    pub fn fit(width: u32, height: u32, page: PageSize) -> Result<Self, Error> {
        let viewport = PageViewport::new(width, height, page)?;
        let percent = viewport.scale() * 100.0;
        Ok(Self {
            page,
            viewport,
            source: [width, height],
            destination: [width, height],
            offset: [0, 0],
            percent,
            fit: true,
            density: None,
        })
    }

    pub fn zoom(
        width: u32,
        height: u32,
        page: PageSize,
        percent: f64,
        center: Point2,
    ) -> Result<Self, Error> {
        if !(1..=100_000).contains(&width) || !(1..=100_000).contains(&height) {
            return Err(Error::InvalidViewport);
        }
        if !percent.is_finite()
            || !(1.0..=999.0).contains(&percent)
            || center.x_pt() < 0.0
            || center.x_pt() > page.width_pt()
            || center.y_pt() < 0.0
            || center.y_pt() > page.height_pt()
        {
            return Err(Error::InvalidView);
        }
        let (viewport, source) = PageViewport::for_zoom(page, percent / 100.0)?;
        let point = viewport.position_f64(center);
        // Validated source and destination bounds keep these signed integers within ±100000.
        let offset = [
            (f64::from(width) / 2.0 - point[0]).round() as i64,
            (f64::from(height) / 2.0 - point[1]).round() as i64,
        ];
        Ok(Self {
            page,
            viewport,
            source,
            destination: [width, height],
            offset,
            percent,
            fit: false,
            density: None,
        })
    }

    /// Fresh pixel-space geometry for this logical view, without scaling glyph bitmaps.
    /// The returned view retains the logical percentage; use [`Self::pixel_density`]
    /// separately. A pixel view cannot be converted again: rebuild from the logical view.
    pub fn at_pixel_size(&self, width_px: u32, height_px: u32) -> Result<Self, Error> {
        if self.density.is_some() {
            return Err(Error::InvalidView);
        }
        crate::byte_count(width_px, height_px)?;
        let density = (f64::from(width_px) / f64::from(self.destination[0]))
            .min(f64::from(height_px) / f64::from(self.destination[1]));
        let padding = [
            (f64::from(width_px) - f64::from(self.destination[0]) * density) / 2.0,
            (f64::from(height_px) - f64::from(self.destination[1]) * density) / 2.0,
        ];
        self.physical_view([width_px, height_px], density, padding)
    }

    /// Fresh native geometry at the exact physical pixels per logical unit.
    /// The fractional canvas origin aligns coverage with the global device lattice;
    /// neither the logical camera nor glyph bitmaps are scaled after generation.
    /// Vector extents retain the 100000-unit bound; frame readback stays separately bounded.
    pub fn at_display_scale(&self, density: f64, origin_phase: [f64; 2]) -> Result<Self, Error> {
        if self.density.is_some()
            || !density.is_finite()
            || density <= 0.0
            || origin_phase
                .iter()
                .any(|value| !value.is_finite() || !(0.0..1.0).contains(value))
        {
            return Err(Error::InvalidView);
        }
        let destination = [
            f64::from(self.destination[0]) * density + origin_phase[0],
            f64::from(self.destination[1]) * density + origin_phase[1],
        ];
        if destination
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0 || value.ceil() > 100_000.0)
        {
            return Err(Error::InvalidViewport);
        }
        self.physical_view(
            destination.map(|value| value.ceil() as u32),
            density,
            origin_phase,
        )
    }

    fn physical_view(
        &self,
        destination: [u32; 2],
        density: f64,
        padding: [f64; 2],
    ) -> Result<Self, Error> {
        if destination == self.destination && density == 1.0 && padding == [0.0, 0.0] {
            return Ok(Self {
                page: self.page,
                viewport: self.viewport.clone(),
                source: self.source,
                destination: self.destination,
                offset: self.offset,
                percent: self.percent,
                fit: self.fit,
                density: Some(1.0),
            });
        }
        let translation = [
            self.offset[0] as f64 * density + padding[0],
            self.offset[1] as f64 * density + padding[1],
        ];
        let offset = translation.map(f64::floor);
        if offset.iter().any(|value| {
            !value.is_finite() || !((i64::MIN as f64)..-(i64::MIN as f64)).contains(value)
        }) {
            return Err(Error::InvalidViewport);
        }
        let offset = offset.map(|value| value as i64);
        let phase = [
            translation[0] - offset[0] as f64,
            translation[1] - offset[1] as f64,
        ];
        let extents = [
            f64::from(self.source[0]) * density + phase[0],
            f64::from(self.source[1]) * density + phase[1],
        ];
        if extents
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0 || value.ceil() > 100_000.0)
        {
            return Err(Error::InvalidViewport);
        }
        Ok(Self {
            page: self.page,
            viewport: self.viewport.at_density(density, phase)?,
            source: extents.map(|value| value.ceil() as u32),
            destination,
            offset,
            percent: self.percent,
            // Even fitted geometry now needs physical padding/projection and clipping.
            fit: false,
            density: Some(density),
        })
    }

    /// Physical pixels per original logical unit; a normal logical view returns one.
    pub fn pixel_density(&self) -> f64 {
        self.density.unwrap_or(1.0)
    }

    pub fn source_size(&self) -> [u32; 2] {
        self.source
    }

    pub fn zoom_percent(&self) -> f64 {
        self.percent
    }

    pub fn canvas_position(&self, point: Point2) -> Result<[f32; 2], Error> {
        if self.fit {
            return self.viewport.canvas_position(point);
        }
        // Match the source f32 anchors already emitted by graph generation.
        self.position(self.viewport.canvas_position(point)?)
    }

    pub fn document_position(&self, point: [f32; 2]) -> Result<Point2, bondline_model::Error> {
        if self.fit {
            return self.viewport.document_position(point);
        }
        self.viewport.document_position_f64([
            f64::from(point[0]) - self.offset[0] as f64,
            f64::from(point[1]) - self.offset[1] as f64,
        ])
    }

    pub fn contains(&self, point: [f32; 2]) -> bool {
        if self.fit {
            return self.viewport.contains(point);
        }
        point[0] >= 0.0
            && point[1] >= 0.0
            && point[0] < self.destination[0] as f32
            && point[1] < self.destination[1] as f32
            && self.document_position(point).is_ok_and(|point| {
                point.x_pt() > 0.0
                    && point.x_pt() < self.page.width_pt()
                    && point.y_pt() > 0.0
                    && point.y_pt() < self.page.height_pt()
            })
    }

    pub fn stroke_width(&self) -> f32 {
        self.viewport.stroke_width()
    }

    pub fn ink_width(&self) -> f32 {
        self.viewport.ink_width()
    }

    /// Build chemical geometry and run its existing whole-source checks before camera clipping.
    pub fn graph_scene(&self, page: &Page, cancel: &Cancellation) -> Result<GraphScene, Error> {
        self.check_page(page, cancel)?;
        self.project(
            crate::page::graph_scene_in_viewport(
                self.source[0],
                self.source[1],
                page,
                None,
                cancel,
                &self.viewport,
            )?,
            cancel,
        )
    }

    /// Separate display facts stay separate from authored H, with the same exact coverage checks.
    pub fn graph_scene_with_display_hydrogens(
        &self,
        page: &Page,
        counts: &BTreeMap<AtomId, u16>,
        cancel: &Cancellation,
    ) -> Result<GraphScene, Error> {
        self.check_page(page, cancel)?;
        crate::page::validate_display_hydrogens(page, counts, cancel)?;
        self.project(
            crate::page::graph_scene_in_viewport(
                self.source[0],
                self.source[1],
                page,
                Some(counts),
                cancel,
                &self.viewport,
            )?,
            cancel,
        )
    }

    fn check_page(&self, page: &Page, cancel: &Cancellation) -> Result<(), Error> {
        cancel.check()?;
        if page.size() != self.page {
            return Err(Error::InvalidView);
        }
        Ok(())
    }

    /// Project geometry produced for this view's full virtual viewport, not a raster frame.
    /// Prefer the graph builders above: size equality alone does not prove a matching Page.
    /// Zoom's closed profile refuses source strokes escaping that canvas, rather than
    /// shortening a diagonal butt cap or revealing formerly unchecked off-source ink.
    pub fn project(
        &self,
        mut graph: GraphScene,
        cancel: &Cancellation,
    ) -> Result<GraphScene, Error> {
        cancel.check()?;
        if [graph.scene.width, graph.scene.height] != self.source {
            return Err(Error::InvalidView);
        }
        if self.fit {
            return Ok(graph);
        }
        if graph.scene.primitives.len() > 4096
            || graph.atoms.len().saturating_add(graph.bonds.len()) > MAX_HIT_REGIONS
        {
            return Err(Error::SceneTooComplex);
        }
        let mut primitives = Vec::with_capacity(graph.scene.primitives.len());
        for primitive in &graph.scene.primitives {
            cancel.check()?;
            match primitive {
                Primitive::Rectangle(rectangle) => {
                    if let Some(rectangle) = self.rectangle(rectangle)? {
                        primitives.push(Primitive::Rectangle(rectangle));
                    }
                }
                Primitive::Stroke(stroke) => {
                    primitives.push(Primitive::Stroke(self.stroke(stroke)?))
                }
            }
        }
        for atom in &mut graph.atoms {
            cancel.check()?;
            atom.position = self.position(atom.position)?;
            if let Some(mask) = atom.label.take() {
                atom.label = self.rectangle(&mask)?;
            }
            if let Some(mask) = atom.supplement.take() {
                atom.supplement = self.rectangle(&mask)?;
            }
        }
        for bond in &mut graph.bonds {
            cancel.check()?;
            bond.stroke = self.stroke(&bond.stroke)?;
            for stroke in &mut bond.strokes {
                cancel.check()?;
                *stroke = self.stroke(stroke)?;
            }
            bond.polygons = bond
                .strokes
                .iter()
                .map(|stroke| {
                    crate::hits::visible_polygon(stroke, self.destination[0], self.destination[1])
                })
                .collect();
        }
        graph.scene.width = self.destination[0];
        graph.scene.height = self.destination[1];
        graph.scene.primitives = primitives;
        cancel.check()?;
        Ok(graph)
    }

    fn position(&self, point: [f32; 2]) -> Result<[f32; 2], Error> {
        checked_position([
            f64::from(point[0]) + self.offset[0] as f64,
            f64::from(point[1]) + self.offset[1] as f64,
        ])
    }

    fn stroke(&self, source: &Stroke) -> Result<Stroke, Error> {
        if source.corners()?.iter().any(|point| {
            point[0] < 0.0
                || point[1] < 0.0
                || point[0] > self.source[0] as f32
                || point[1] > self.source[1] as f32
        }) {
            return Err(Error::UnsupportedChemicalContent);
        }
        let stroke = Stroke {
            begin: self.position(source.begin)?,
            end: self.position(source.end)?,
            ..source.clone()
        };
        stroke.validate()?;
        Ok(stroke)
    }

    fn rectangle(&self, source: &Rectangle) -> Result<Option<Rectangle>, Error> {
        let right = source
            .x
            .checked_add(source.width)
            .ok_or(Error::InvalidGeometry)?;
        let bottom = source
            .y
            .checked_add(source.height)
            .ok_or(Error::InvalidGeometry)?;
        let left = source.x.min(self.source[0]);
        let top = source.y.min(self.source[1]);
        let right = right.min(self.source[0]);
        let bottom = bottom.min(self.source[1]);
        if left >= right || top >= bottom {
            return Ok(None);
        }
        let left = (i64::from(left) + self.offset[0]).clamp(0, i64::from(self.destination[0]));
        let right = (i64::from(right) + self.offset[0]).clamp(0, i64::from(self.destination[0]));
        let top = (i64::from(top) + self.offset[1]).clamp(0, i64::from(self.destination[1]));
        let bottom = (i64::from(bottom) + self.offset[1]).clamp(0, i64::from(self.destination[1]));
        if left >= right || top >= bottom {
            return Ok(None);
        }
        Ok(Some(Rectangle {
            x: u32::try_from(left).map_err(|_| Error::InvalidGeometry)?,
            y: u32::try_from(top).map_err(|_| Error::InvalidGeometry)?,
            width: u32::try_from(right - left).map_err(|_| Error::InvalidGeometry)?,
            height: u32::try_from(bottom - top).map_err(|_| Error::InvalidGeometry)?,
            color: source.color,
        }))
    }
}

fn checked_position(point: [f64; 2]) -> Result<[f32; 2], Error> {
    let point = point.map(|value| value as f32);
    if point
        .iter()
        .any(|value| !value.is_finite() || value.abs() > 100_000.0)
    {
        return Err(Error::InvalidStroke);
    }
    Ok(point)
}
