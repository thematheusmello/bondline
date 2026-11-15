//! Original bounded carbon-path geometry; no source-backed import or cleanup claim.

use super::{NormalizeError, SmilesDocument};
use bondline_model::{BondOrder, Document, Fragment, PageId, PageSize, Point2, Radical};

const MAX_PATH_ATOMS: usize = 12;
const REFERENCE_SIZE: u32 = 1024;

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("SMILES path layout requires a finite positive bond length in points")]
    InvalidBondLength,
    #[error("SMILES path layout supports at most 12 atoms")]
    AtomLimit,
    #[error("neutral acyclic SMILES layout supports at most 64 atoms")]
    AcyclicAtomLimit,
    #[error("neutral simple-ring SMILES layout supports at most 64 atoms")]
    SimpleRingAtomLimit,
    #[error("neutral component SMILES layout supports at most 64 atoms")]
    ComponentsAtomLimit,
    #[error("SMILES layout cancelled")]
    Cancelled,
    #[error("SMILES path layout refuses {0}")]
    Unsupported(&'static str),
    #[error("SMILES path geometry is unsupported: {0}")]
    Geometry(&'static str),
    #[error(transparent)]
    Normalize(#[from] NormalizeError),
    #[error(transparent)]
    Model(#[from] bondline_model::Error),
    #[error(transparent)]
    Render(#[from] bondline_render::Error),
}

impl LayoutError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidBondLength => "invalid_params",
            Self::Cancelled | Self::Render(bondline_render::Error::Cancelled) => "cancelled",
            Self::AtomLimit
            | Self::AcyclicAtomLimit
            | Self::SimpleRingAtomLimit
            | Self::ComponentsAtomLimit
            | Self::Model(bondline_model::Error::Graph(bondline_model::GraphError::Limit(_)))
            | Self::Render(bondline_render::Error::SceneTooComplex) => "resource_limit",
            Self::Normalize(error) => error.code(),
            Self::Model(_) => "invalid_document",
            Self::Unsupported(_) | Self::Geometry(_) | Self::Render(_) => "unsupported_content",
        }
    }
}

impl SmilesDocument {
    /// Position one connected neutral, nonisotopic carbon/Single path of 1..=12 atoms.
    /// Explicitly materializes the existing fixed-H policy; source/IR remain untouched.
    /// Local IDs/connectivity/metadata stay identical to normalization, except positions.
    /// Length/angle topology and complete ink must fit the supplied page at 1024x1024.
    /// This reference-scene check does not certify every viewport or native UI.
    /// This is a new pure drawing, not application import, Save authority or cleanup.
    pub fn layout_carbon_path(
        &self,
        bond_length_pt: f64,
        origin: Point2,
        page_size: PageSize,
    ) -> Result<Document, LayoutError> {
        if !bond_length_pt.is_finite() || bond_length_pt <= 0.0 {
            return Err(LayoutError::InvalidBondLength);
        }
        if self.atoms().len() > MAX_PATH_ATOMS {
            return Err(LayoutError::AtomLimit);
        }
        let normalized = self.normalize_explicit_h_fragment()?;
        if normalized.atoms().iter().any(|atom| {
            atom.element.get() != 6
                || atom.isotope.is_some()
                || atom.charge != 0
                || atom.radical != Radical::None
        }) {
            return Err(LayoutError::Unsupported(
                "nonneutral, isotopic or non-carbon atoms",
            ));
        }
        if normalized
            .bonds()
            .iter()
            .any(|bond| bond.order != BondOrder::Single)
        {
            return Err(LayoutError::Unsupported("bond orders other than Single"));
        }
        let count = normalized.atoms().len();
        if self.bonds().len().checked_add(1) != Some(count) {
            return Err(LayoutError::Unsupported("cycles or disconnected graphs"));
        }
        let mut neighbors = vec![Vec::new(); count];
        for bond in self.bonds() {
            let [a, b] = bond.atoms;
            for (at, other) in [(a, b), (b, a)] {
                neighbors
                    .get_mut(at)
                    .ok_or(LayoutError::Geometry("missing endpoint"))?
                    .push(other);
            }
        }
        if neighbors.iter().any(|links| links.len() > 2) {
            return Err(LayoutError::Unsupported("branching graphs"));
        }
        // Normalization assigns dense IDs in source-index order, not ring-label order.
        let start = if count == 1 {
            0
        } else {
            neighbors
                .iter()
                .position(|links| links.len() == 1)
                .ok_or(LayoutError::Unsupported("no path endpoint"))?
        };
        let mut order = Vec::with_capacity(count);
        let mut visited = vec![false; count];
        let mut previous = None;
        let mut current = Some(start);
        while let Some(at) = current {
            let seen = visited
                .get_mut(at)
                .ok_or(LayoutError::Geometry("missing atom"))?;
            if *seen {
                return Err(LayoutError::Unsupported("cyclic graph"));
            }
            *seen = true;
            order.push(at);
            current = neighbors
                .get(at)
                .ok_or(LayoutError::Geometry("missing adjacency"))?
                .iter()
                .copied()
                .find(|neighbor| Some(*neighbor) != previous);
            previous = Some(at);
        }
        if order.len() != count {
            return Err(LayoutError::Unsupported("disconnected graph"));
        }
        let mut atoms = normalized.atoms().to_vec();
        let mut points = Vec::with_capacity(count);
        let half = bond_length_pt / 2.0;
        let step = half * 3.0_f64.sqrt();
        let (cos, sin) = if count == 1 {
            (1.0, 0.0)
        } else {
            let chord_x = (count - 1) as f64 * 3.0_f64.sqrt() / 2.0;
            let chord_y = if count % 2 == 0 { 0.5 } else { 0.0 };
            let length = chord_x.hypot(chord_y);
            (chord_x / length, chord_y / length)
        };
        for (index, at) in order.into_iter().enumerate() {
            let x = index as f64 * step;
            let y = if index % 2 == 0 { 0.0 } else { half };
            let point = Point2::new(
                origin.x_pt() + x * cos + y * sin,
                origin.y_pt() + y * cos - x * sin,
            )?;
            points.push(point);
            atoms
                .get_mut(at)
                .ok_or(LayoutError::Geometry("missing output atom"))?
                .position = Some(point);
        }
        validate_geometry(&points, bond_length_pt, page_size)?;
        let fragment = Fragment::new(normalized.id(), atoms, normalized.bonds().to_vec())
            .map_err(bondline_model::Error::from)?;
        let mut document = Document::default();
        let page = PageId::new(1).ok_or(LayoutError::Geometry("invalid page identity"))?;
        document.set_page_size(page, page_size)?;
        document.append_fragments(page, vec![fragment])?;
        validate_ink(
            document
                .pages()
                .first()
                .ok_or(LayoutError::Geometry("missing page"))?,
        )?;
        Ok(document)
    }
}

fn cross(a: Point2, b: Point2, p: Point2) -> f64 {
    (b.x_pt() - a.x_pt()) * (p.y_pt() - a.y_pt()) - (b.y_pt() - a.y_pt()) * (p.x_pt() - a.x_pt())
}
fn on_segment(a: Point2, b: Point2, p: Point2, epsilon: f64, area_epsilon: f64) -> bool {
    cross(a, b, p).abs() <= area_epsilon
        && p.x_pt() >= a.x_pt().min(b.x_pt()) - epsilon
        && p.x_pt() <= a.x_pt().max(b.x_pt()) + epsilon
        && p.y_pt() >= a.y_pt().min(b.y_pt()) - epsilon
        && p.y_pt() <= a.y_pt().max(b.y_pt()) + epsilon
}
fn segments_intersect(
    a: Point2,
    b: Point2,
    c: Point2,
    d: Point2,
    epsilon: f64,
    area_epsilon: f64,
) -> bool {
    let opposite = |x: f64, y: f64| {
        (x > area_epsilon && y < -area_epsilon) || (x < -area_epsilon && y > area_epsilon)
    };
    opposite(cross(a, b, c), cross(a, b, d)) && opposite(cross(c, d, a), cross(c, d, b))
        || on_segment(a, b, c, epsilon, area_epsilon)
        || on_segment(a, b, d, epsilon, area_epsilon)
        || on_segment(c, d, a, epsilon, area_epsilon)
        || on_segment(c, d, b, epsilon, area_epsilon)
}
fn validate_geometry(points: &[Point2], length: f64, page: PageSize) -> Result<(), LayoutError> {
    let epsilon = length.max(1.0) * 1e-12;
    let area_epsilon = epsilon * length.max(1.0);
    for (i, &point) in points.iter().enumerate() {
        if !(0.0..=page.width_pt()).contains(&point.x_pt())
            || !(0.0..=page.height_pt()).contains(&point.y_pt())
        {
            return Err(LayoutError::Geometry("vertex outside the page"));
        }
        for &other in points.iter().skip(i + 1) {
            if (point.x_pt() - other.x_pt()).hypot(point.y_pt() - other.y_pt()) <= epsilon {
                return Err(LayoutError::Geometry("coincident vertices"));
            }
        }
        for (edge, segment) in points.windows(2).enumerate() {
            if i != edge
                && i != edge + 1
                && let [a, b] = segment
                && on_segment(*a, *b, point, epsilon, area_epsilon)
            {
                return Err(LayoutError::Geometry("vertex on an unrelated bond"));
            }
        }
    }
    for (i, first) in points.windows(2).enumerate() {
        for second in points.windows(2).skip(i + 2) {
            if let ([a, b], [c, d]) = (first, second)
                && segments_intersect(*a, *b, *c, *d, epsilon, area_epsilon)
            {
                return Err(LayoutError::Geometry("nonincident bonds intersect"));
            }
        }
    }
    Ok(())
}

fn validate_ink(page: &bondline_model::Page) -> Result<(), LayoutError> {
    validate_ink_with_cancel(page, &bondline_render::Cancellation::default()).map(|_| ())
}

pub(super) fn validate_ink_with_cancel(
    page: &bondline_model::Page,
    cancel: &bondline_render::Cancellation,
) -> Result<bondline_render::GraphScene, LayoutError> {
    use bondline_render::{PageViewport, Primitive, graph_page_scene_with_cancel, page_scene};
    if cancel.is_cancelled() {
        return Err(LayoutError::Cancelled);
    }
    let background = page_scene(REFERENCE_SIZE, REFERENCE_SIZE, page.size())?;
    let graph = graph_page_scene_with_cancel(REFERENCE_SIZE, REFERENCE_SIZE, page, cancel)
        .map_err(|error| {
            if matches!(error, bondline_render::Error::Cancelled) {
                LayoutError::Cancelled
            } else {
                LayoutError::Render(error)
            }
        })?;
    let scene = graph.scene();
    // graph_page_scene's page prefix is checked, never assumed to contain three items.
    if !scene.primitives.starts_with(&background.primitives) {
        return Err(LayoutError::Geometry("unexpected page-scene prefix"));
    }
    let viewport = PageViewport::new(REFERENCE_SIZE, REFERENCE_SIZE, page.size())?;
    let paper = viewport.paper();
    if paper.width < 3 || paper.height < 3 {
        return Err(LayoutError::Geometry("no page interior"));
    }
    let left = f64::from(paper.x) + 1.0;
    let top = f64::from(paper.y) + 1.0;
    let right = f64::from(paper.x) + f64::from(paper.width) - 1.0;
    let bottom = f64::from(paper.y) + f64::from(paper.height) - 1.0;
    let inside = |x: f64, y: f64| x >= left && x <= right && y >= top && y <= bottom;
    for primitive in scene.primitives.iter().skip(background.primitives.len()) {
        if cancel.is_cancelled() {
            return Err(LayoutError::Cancelled);
        }
        let contained = match primitive {
            Primitive::Rectangle(rect) => {
                inside(f64::from(rect.x), f64::from(rect.y))
                    && inside(
                        f64::from(rect.x) + f64::from(rect.width),
                        f64::from(rect.y) + f64::from(rect.height),
                    )
            }
            // These are the renderer's actual butt-cap corners, including half width.
            Primitive::Stroke(stroke) => stroke
                .corners()?
                .iter()
                .all(|p| inside(f64::from(p[0]), f64::from(p[1]))),
        };
        if !contained {
            return Err(LayoutError::Geometry("ink outside the page interior"));
        }
    }
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x: f64, y: f64) -> Result<Point2, LayoutError> {
        Ok(Point2::new(x, y)?)
    }
    #[test]
    fn actual_geometry_rejects_duplicate_vertex_on_bond_and_crossing_segments()
    -> Result<(), LayoutError> {
        let page = PageSize::A4;
        assert!(matches!(
            validate_geometry(
                &[p(10.0, 10.0)?, p(20.0, 10.0)?, p(10.0, 10.0)?],
                10.0,
                page
            ),
            Err(LayoutError::Geometry("coincident vertices"))
        ));
        assert!(matches!(
            validate_geometry(
                &[p(10.0, 10.0)?, p(30.0, 10.0)?, p(20.0, 10.0)?],
                20.0,
                page
            ),
            Err(LayoutError::Geometry("vertex on an unrelated bond"))
        ));
        assert!(matches!(
            validate_geometry(
                &[
                    p(10.0, 10.0)?,
                    p(30.0, 30.0)?,
                    p(10.0, 30.0)?,
                    p(30.0, 10.0)?
                ],
                20.0,
                page
            ),
            Err(LayoutError::Geometry("nonincident bonds intersect"))
        ));
        assert!(!segments_intersect(
            p(10.0, 10.0)?,
            p(20.0, 10.0)?,
            p(30.0, 10.0)?,
            p(40.0, 10.0)?,
            1e-12,
            1e-12
        ));
        Ok(())
    }
}
