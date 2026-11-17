//! Native authoring geometry and a bounded certificate over the complete target page.

use super::{
    LayoutError, SmilesDocument,
    acyclic::{WORK_PER_ROTATION, check, geometry, polygons_overlap},
    components,
    layout::validate_ink_with_settings,
};
use bondline_model::{AtomId, BondId, Document, Fragment, FragmentId, Page, PageSize, Point2};
use bondline_render::{Cancellation, MAX_HIT_REGIONS};
use std::collections::BTreeMap;

impl SmilesDocument {
    /// Position the complete neutral integer authoring graph without changing source/IR.
    /// Reuses the original bounded tree, small-ring and component-shelf policies.
    /// Isotopes and unavailable topology/ink refuse the whole drawing; no partial import.
    pub fn layout_authoring(
        &self,
        length: f64,
        origin: Point2,
        page: PageSize,
        cancel: &Cancellation,
    ) -> Result<Document, LayoutError> {
        check(cancel)?;
        if !length.is_finite() || length <= 0.0 {
            return Err(LayoutError::InvalidBondLength);
        }
        let fragment = self.normalize_authoring_fragment(cancel)?;
        components::layout_fragment(&fragment, length, origin, page, cancel, true)
    }
}

/// Certify the actual complete authored page before appending native drawing content.
/// Geometry uses a dense temporary view; rendering and stroke lookup use original IDs.
/// Existing positions, literal/unknown H, metadata and fragment ownership stay untouched.
/// Original policy uses the page diagonal for epsilon and at most100000 pair comparisons.
/// This 1024-square certificate is not a promise that every native zoom can be rendered.
pub fn validate_authoring_page(page: &Page, cancel: &Cancellation) -> Result<(), LayoutError> {
    validate_authoring_page_with_style(page, bondline_model::DrawingStyle::default(), cancel)
}

/// The same whole-page certificate over the actual authored ink width.
pub fn validate_authoring_page_with_style(
    page: &Page,
    style: bondline_model::DrawingStyle,
    cancel: &Cancellation,
) -> Result<(), LayoutError> {
    validate_authoring_page_with_settings(
        page,
        style,
        bondline_model::DrawingSettings::default(),
        cancel,
    )
}

/// Certify the final authored appearance, including the preferred spacing reference.
pub fn validate_authoring_page_with_settings(
    page: &Page,
    style: bondline_model::DrawingStyle,
    settings: bondline_model::DrawingSettings,
    cancel: &Cancellation,
) -> Result<(), LayoutError> {
    check(cancel)?;
    let mut items = 0_usize;
    for fragment in page.fragments() {
        check(cancel)?;
        items = items
            .saturating_add(fragment.atoms().len())
            .saturating_add(fragment.bonds().len());
        if items > MAX_HIT_REGIONS {
            return Err(LayoutError::ComparisonLimit);
        }
    }
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    let mut identities = BTreeMap::new();
    let mut positions = Vec::new();
    for fragment in page.fragments() {
        for atom in fragment.atoms() {
            check(cancel)?;
            let point = atom
                .position
                .ok_or(LayoutError::Geometry("missing authored atom position"))?;
            if point.x_pt() < 0.0
                || point.y_pt() < 0.0
                || point.x_pt() > page.size().width_pt()
                || point.y_pt() > page.size().height_pt()
            {
                return Err(LayoutError::Geometry("authored atom outside paper"));
            }
            let ordinal =
                u32::try_from(atoms.len() + 1).map_err(|_| LayoutError::ComparisonLimit)?;
            let id = AtomId::new(ordinal).ok_or(LayoutError::ComparisonLimit)?;
            if identities.insert(atom.id, id).is_some() {
                return Err(LayoutError::Geometry("duplicate page atom identity"));
            }
            let mut dense = atom.clone();
            dense.id = id;
            atoms.push(dense);
            positions.push(Some([point.x_pt(), point.y_pt()]));
        }
    }
    let mut degree = vec![0_usize; atoms.len()];
    for fragment in page.fragments() {
        for bond in fragment.bonds() {
            check(cancel)?;
            let mut dense = *bond;
            dense.id = BondId::new(
                u32::try_from(bonds.len() + 1).map_err(|_| LayoutError::ComparisonLimit)?,
            )
            .ok_or(LayoutError::ComparisonLimit)?;
            dense.begin = *identities
                .get(&bond.begin)
                .ok_or(LayoutError::Geometry("missing page bond endpoint"))?;
            dense.end = *identities
                .get(&bond.end)
                .ok_or(LayoutError::Geometry("missing page bond endpoint"))?;
            for endpoint in [dense.begin, dense.end] {
                let index = usize::try_from(endpoint.get() - 1)
                    .map_err(|_| LayoutError::ComparisonLimit)?;
                let count = degree
                    .get_mut(index)
                    .ok_or(LayoutError::Geometry("missing dense endpoint"))?;
                *count += 1;
            }
            bonds.push(dense);
        }
    }
    let mut remaining = WORK_PER_ROTATION;
    if !atoms.is_empty() {
        // Reserve the exact maximum geometry comparisons before entering its legacy loop.
        let pairs = |count: usize| count * count.saturating_sub(1) / 2;
        let incident = degree.iter().map(|&count| pairs(count)).sum::<usize>();
        let edge_pairs = pairs(bonds.len())
            .checked_sub(incident)
            .ok_or(LayoutError::Geometry("invalid page adjacency"))?;
        let comparisons =
            pairs(atoms.len()) + bonds.len() * atoms.len().saturating_sub(2) + edge_pairs;
        remaining = remaining
            .checked_sub(comparisons)
            .ok_or(LayoutError::ComparisonLimit)?;
        let fragment = Fragment::new(
            FragmentId::new(1).ok_or(LayoutError::ComparisonLimit)?,
            atoms,
            bonds,
        )
        .map_err(bondline_model::Error::from)?;
        let mut work = comparisons;
        if !geometry(
            &fragment,
            &positions,
            page.size().width_pt().hypot(page.size().height_pt()),
            Point2::new(0.0, 0.0)?,
            page.size(),
            &mut work,
            cancel,
        )? {
            return Err(LayoutError::Geometry("whole-page authored geometry"));
        }
    }
    let scene = validate_ink_with_settings(page, style, settings, cancel)?;
    let actual_bonds = page
        .fragments()
        .iter()
        .flat_map(|fragment| {
            fragment
                .bonds()
                .iter()
                .map(move |bond| (fragment.id(), bond))
        })
        .collect::<Vec<_>>();
    for (index, (fragment, bond)) in actual_bonds.iter().enumerate() {
        for (other_fragment, other) in actual_bonds.iter().skip(index + 1) {
            spend(&mut remaining, cancel)?;
            if fragment == other_fragment
                && [bond.begin, bond.end]
                    .iter()
                    .any(|atom| *atom == other.begin || *atom == other.end)
            {
                continue;
            }
            for a in scene.bond_strokes(*fragment, bond.id) {
                for b in scene.bond_strokes(*other_fragment, other.id) {
                    spend(&mut remaining, cancel)?;
                    if polygons_overlap(&a.corners()?, &b.corners()?) {
                        return Err(LayoutError::Geometry(
                            "nonincident whole-page bond ink overlaps",
                        ));
                    }
                }
            }
        }
    }
    check(cancel)
}

fn spend(remaining: &mut usize, cancel: &Cancellation) -> Result<(), LayoutError> {
    check(cancel)?;
    *remaining = remaining
        .checked_sub(1)
        .ok_or(LayoutError::ComparisonLimit)?;
    Ok(())
}
