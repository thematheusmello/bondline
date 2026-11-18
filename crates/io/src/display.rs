//! Source-aware drawing display; derived facts never change authored data or Save authority.

use crate::StoredDocument;
use bondline_model::{
    AtomId, BondOrder, Document, DoublePosition, Fragment, Page, PageId, Point2, Radical,
};
use bondline_render::{Cancellation, GraphScene};
use std::collections::BTreeMap;

/// The origin of a count supplied to a display, without rewriting the source atom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HydrogenOrigin {
    Literal,
    NeutralDefaultV1,
    PhysicalHydrogenV1,
}

/// Failures preserve source completeness, closed chemistry and renderer boundaries.
#[derive(Debug, thiserror::Error)]
pub enum DisplayError {
    #[error("drawing display cancelled")]
    Cancelled,
    #[error("page {page_id:?} does not exist")]
    MissingPage { page_id: PageId },
    #[error("the retained source is not completely represented by the drawing")]
    IncompleteSource,
    #[error("drawing display does not support {feature}")]
    Unsupported { feature: &'static str },
    #[error("drawing display exceeds {kind}")]
    Limit { kind: &'static str },
    #[error(transparent)]
    Render(#[from] bondline_render::Error),
}

impl DisplayError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Cancelled | Self::Render(bondline_render::Error::Cancelled) => "cancelled",
            Self::MissingPage { .. } => "not_found",
            Self::IncompleteSource
            | Self::Unsupported { .. }
            | Self::Render(bondline_render::Error::UnsupportedText(_))
            | Self::Render(bondline_render::Error::UnsupportedChemicalContent)
            | Self::Render(bondline_render::Error::LabelPlacement)
            | Self::Render(bondline_render::Error::LabelOverlap) => "unsupported_content",
            Self::Limit { .. } => "resource_limit",
            Self::Render(
                bondline_render::Error::InvalidDimensions | bondline_render::Error::InvalidView,
            ) => "invalid_params",
            Self::Render(
                bondline_render::Error::LabelScale | bondline_render::Error::BondScale,
            ) => "unsupported_scale",
            Self::Render(_) => "render_failed",
        }
    }
}

/// Display the actual current Page and IDs from the complete drawing owner.
/// Only original MOL backing selects the closed neutral display-H policy. All other
/// drawings keep literal rendering; neither source bytes nor authored H is rewritten.
pub fn drawing_page_scene(
    width: u32,
    height: u32,
    stored: &StoredDocument,
    page_id: PageId,
    cancel: &Cancellation,
) -> Result<GraphScene, DisplayError> {
    let (page, counts) = display_facts(stored, page_id, cancel)?;
    let view = bondline_render::PageView::fit(width, height, page.size())?
        .with_drawing_style(stored.model().drawing_style())
        .with_drawing_settings(stored.model().drawing_settings());
    Ok(match counts {
        None => view.graph_scene(page, cancel)?,
        Some(counts) => view.graph_scene_with_display_hydrogens(page, &counts, cancel)?,
    })
}

/// The same complete-source/chemical policy with a transient logical-unit camera.
pub fn drawing_page_scene_with_view(
    view: &bondline_render::PageView,
    stored: &StoredDocument,
    page_id: PageId,
    cancel: &Cancellation,
) -> Result<GraphScene, DisplayError> {
    let (page, counts) = display_facts(stored, page_id, cancel)?;
    let view = view
        .clone()
        .with_drawing_style(stored.model().drawing_style())
        .with_drawing_settings(stored.model().drawing_settings());
    Ok(match counts {
        None => view.graph_scene(page, cancel)?,
        Some(counts) => view.graph_scene_with_display_hydrogens(page, &counts, cancel)?,
    })
}

fn connected_geometry(page: &Page) -> bool {
    let vertices: Vec<_> = page
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.atoms())
        .filter_map(|atom| atom.position.map(|point| (atom.id, point)))
        .collect();
    let positions: BTreeMap<_, _> = vertices.iter().copied().collect();
    let edges = page
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.bonds())
        .map(|bond| {
            Some((
                bond.begin,
                bond.end,
                *positions.get(&bond.begin)?,
                *positions.get(&bond.end)?,
            ))
        })
        .collect::<Option<Vec<_>>>();
    let Some(edges) = edges else {
        return false;
    };
    // Reuse point-space predicates, with conservative authored-coordinate roundoff.
    let mut extent = 1.0_f64;
    let mut magnitude = 1.0_f64;
    if let Some((_, origin)) = vertices.first() {
        for (_, point) in &vertices {
            extent = extent
                .max((point.x_pt() - origin.x_pt()).abs())
                .max((point.y_pt() - origin.y_pt()).abs());
            magnitude = magnitude.max(point.x_pt().abs()).max(point.y_pt().abs());
        }
    }
    let epsilon = (extent * 1e-12).max(128.0 * f64::EPSILON * magnitude);
    let area_epsilon = epsilon * extent;
    for (i, (id, point)) in vertices.iter().enumerate() {
        for (_, other) in vertices.iter().skip(i + 1) {
            if (point.x_pt() - other.x_pt()).hypot(point.y_pt() - other.y_pt()) <= epsilon {
                return false;
            }
        }
        for (a, b, start, end) in &edges {
            if id != a
                && id != b
                && crate::smiles::on_segment(*start, *end, *point, epsilon, area_epsilon)
            {
                return false;
            }
        }
    }
    for (i, (a, b, start, end)) in edges.iter().enumerate() {
        for (c, d, other_start, other_end) in edges.iter().skip(i + 1) {
            if a != c
                && a != d
                && b != c
                && b != d
                && crate::smiles::segments_intersect(
                    *start,
                    *end,
                    *other_start,
                    *other_end,
                    epsilon,
                    area_epsilon,
                )
            {
                return false;
            }
        }
    }
    true
}

pub(crate) fn connected_label_bonds(model: &Document, page: &Page) -> bool {
    let style = model.drawing_style();
    if style.line_width_pt().is_none() || style.margin_width_pt().is_none() {
        return false;
    }
    let mut has_bond = false;
    for bond in page
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.bonds())
    {
        has_bond = true;
        match (bond.order, bond.double_position) {
            (BondOrder::Single, DoublePosition::Unspecified) => {}
            (BondOrder::Double, DoublePosition::Center)
            | (BondOrder::Triple, DoublePosition::Unspecified)
                if model.drawing_settings().bond_length_pt().is_some()
                    && style.bond_spacing_percent().is_some() => {}
            _ => return false,
        }
    }
    has_bond && connected_geometry(page)
}

type DisplayFacts<'a> = (&'a bondline_model::Page, Option<BTreeMap<AtomId, u16>>);

fn display_facts<'a>(
    stored: &'a StoredDocument,
    page_id: PageId,
    cancel: &Cancellation,
) -> Result<DisplayFacts<'a>, DisplayError> {
    if cancel.is_cancelled() {
        return Err(DisplayError::Cancelled);
    }
    if !stored.has_display_projection() {
        return Err(DisplayError::IncompleteSource);
    }
    let page = stored
        .model()
        .pages()
        .iter()
        .find(|page| page.id() == page_id)
        .ok_or(DisplayError::MissingPage { page_id })?;
    if stored.mol_version().is_none() {
        return Ok((page, None));
    }
    // Keep the renderer's cheap hit bound ahead of allocating any derived topology.
    let mut hit_count = 0_usize;
    for fragment in page.fragments() {
        if cancel.is_cancelled() {
            return Err(DisplayError::Cancelled);
        }
        hit_count = hit_count
            .saturating_add(fragment.atoms().len())
            .saturating_add(fragment.bonds().len());
        if hit_count > bondline_render::MAX_HIT_REGIONS {
            return Err(DisplayError::Render(
                bondline_render::Error::SceneTooComplex,
            ));
        }
    }
    let mut counts = BTreeMap::new();
    for fragment in page.fragments() {
        let resolved =
            hydrogens(fragment, PhysicalHydrogenPolicy::Literal, cancel).map_err(|error| {
                match error {
                    HydrogenError::Cancelled => DisplayError::Cancelled,
                    HydrogenError::Unsupported(feature) => DisplayError::Unsupported { feature },
                    HydrogenError::Limit(kind) => DisplayError::Limit { kind },
                }
            })?;
        for (id, (count, _origin)) in resolved {
            if cancel.is_cancelled() {
                return Err(DisplayError::Cancelled);
            }
            counts.insert(id, count);
        }
    }
    Ok((page, Some(counts)))
}

pub(crate) enum HydrogenError {
    Cancelled,
    Unsupported(&'static str),
    Limit(&'static str),
}

pub(crate) enum PhysicalHydrogenPolicy {
    // Drawing compatibility preserves authored H0; SDF's stricter topology stays separate.
    Literal,
    SingleBond,
}

fn check_cancel(cancel: &Cancellation) -> Result<(), HydrogenError> {
    if cancel.is_cancelled() {
        Err(HydrogenError::Cancelled)
    } else {
        Ok(())
    }
}

struct Bonding {
    sum: u32,
    degree: usize,
    non_single: bool,
    position: Point2,
}

pub(crate) fn hydrogens(
    fragment: &Fragment,
    physical_policy: PhysicalHydrogenPolicy,
    cancel: &Cancellation,
) -> Result<BTreeMap<AtomId, (u16, HydrogenOrigin)>, HydrogenError> {
    let mut neighbours = BTreeMap::new();
    for atom in fragment.atoms() {
        check_cancel(cancel)?;
        if atom.charge != 0 || atom.isotope.is_some() || atom.radical != Radical::None {
            return Err(HydrogenError::Unsupported(
                "charge, isotope or radical annotations",
            ));
        }
        neighbours.insert(
            atom.id,
            Bonding {
                sum: 0,
                degree: 0,
                non_single: false,
                position: atom
                    .position
                    .ok_or(HydrogenError::Unsupported("missing atom coordinates"))?,
            },
        );
    }
    for bond in fragment.bonds() {
        check_cancel(cancel)?;
        let weight = match bond.order {
            BondOrder::Single => 1,
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            _ => {
                return Err(HydrogenError::Unsupported(
                    "bond order outside Single/Double/Triple",
                ));
            }
        };
        let begin = neighbours
            .get(&bond.begin)
            .ok_or(HydrogenError::Unsupported("missing bond endpoint"))?;
        let end = neighbours
            .get(&bond.end)
            .ok_or(HydrogenError::Unsupported("missing bond endpoint"))?;
        if begin.position == end.position {
            return Err(HydrogenError::Unsupported("coincident bond endpoints"));
        }
        for id in [bond.begin, bond.end] {
            let bonding = neighbours
                .get_mut(&id)
                .ok_or(HydrogenError::Unsupported("missing bond endpoint"))?;
            bonding.sum = bonding
                .sum
                .checked_add(weight)
                .ok_or(HydrogenError::Limit("bond-order sum"))?;
            bonding.degree = bonding
                .degree
                .checked_add(1)
                .ok_or(HydrogenError::Limit("atom degree"))?;
            bonding.non_single |= bond.order != BondOrder::Single;
        }
    }
    let mut output = BTreeMap::new();
    for atom in fragment.atoms() {
        check_cancel(cancel)?;
        let bonding = neighbours
            .get(&atom.id)
            .ok_or(HydrogenError::Unsupported("missing atom topology"))?;
        let resolved = if atom.element.get() == 1 {
            let literal_zero = matches!(physical_policy, PhysicalHydrogenPolicy::Literal)
                && atom.explicit_hydrogens == Some(0);
            if !literal_zero
                && (bonding.degree != 1
                    || bonding.sum != 1
                    || bonding.non_single
                    || atom.explicit_hydrogens.is_some_and(|h| h != 0))
            {
                return Err(HydrogenError::Unsupported(
                    "physical H requires one Single bond and no attached H",
                ));
            }
            (
                0,
                if atom.explicit_hydrogens.is_some() {
                    HydrogenOrigin::Literal
                } else {
                    HydrogenOrigin::PhysicalHydrogenV1
                },
            )
        } else if let Some(count) = atom.explicit_hydrogens {
            (count, HydrogenOrigin::Literal)
        } else {
            // Original closed display policy, not general periodic-table/CTfile valence.
            let target: u32 = match atom.element.get() {
                6 => 4,
                7 => 3,
                8 => 2,
                9 | 17 | 35 | 53 => 1,
                _ => {
                    return Err(HydrogenError::Unsupported(
                        "unresolved H outside neutral C/N/O/halogen profile",
                    ));
                }
            };
            let count = target
                .checked_sub(bonding.sum)
                .ok_or(HydrogenError::Unsupported(
                    "bond sum exceeds neutral default valence",
                ))?;
            (
                u16::try_from(count).map_err(|_| HydrogenError::Limit("display H count"))?,
                HydrogenOrigin::NeutralDefaultV1,
            )
        };
        output.insert(atom.id, resolved);
    }
    Ok(output)
}
