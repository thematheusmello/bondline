//! Fixed component poses and original source-order center-box shelves; no source mutation.

use super::{
    LayoutError, SmilesDocument,
    acyclic::{
        self, MAX_ATOMS, SCENES_PER_ROTATION, WORK_PER_ROTATION, check, geometry,
        positioned_document,
    },
    ring,
};
use bondline_model::{AtomId, BondId, Document, Fragment, PageSize, Point2};
use bondline_render::Cancellation;
use std::collections::VecDeque;

impl SmilesDocument {
    /// Derive all actual neutral tree/simple-ring components into one complete Fragment.
    /// Keeps normalized arrays, IDs, directed edges and fixed H; only positions change.
    /// Component poses are rigidly shelved with a center-box gap of the supplied length.
    /// The complete assembled drawing must pass the existing geometry and ink guards.
    pub fn layout_neutral_components(
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
        if self.atoms().len() > MAX_ATOMS {
            return Err(LayoutError::ComponentsAtomLimit);
        }
        let fragment = self.normalize_explicit_h_fragment()?;
        check(cancel)?;
        if fragment.atoms().iter().any(|atom| atom.isotope.is_some()) {
            return Err(LayoutError::Unsupported("isotopic drawing labels"));
        }
        if origin.x_pt() < 0.0
            || origin.y_pt() < 0.0
            || origin.x_pt() > page.width_pt()
            || origin.y_pt() > page.height_pt()
        {
            return Err(LayoutError::Geometry("atom-center origin outside paper"));
        }
        if fragment.atoms().len() > 1 && length > page.width_pt().hypot(page.height_pt()) {
            return Err(LayoutError::Geometry("bond length exceeds paper"));
        }
        let components = component_indices(&fragment, cancel)?;
        let count = components.len();
        if count == 1 {
            // No packing arithmetic may perturb an existing one-component result.
            return solve_component(
                &fragment,
                length,
                origin,
                page,
                cancel,
                WORK_PER_ROTATION,
                SCENES_PER_ROTATION,
            );
        }
        let work = WORK_PER_ROTATION
            .checked_div(count)
            .filter(|limit| *limit > 0)
            .ok_or(LayoutError::ComponentsAtomLimit)?;
        let scenes = SCENES_PER_ROTATION
            .checked_div(count)
            .filter(|limit| *limit > 0)
            .ok_or(LayoutError::ComponentsAtomLimit)?;
        let mut positions = vec![None; fragment.atoms().len()];
        let mut cursor = [origin.x_pt(), origin.y_pt()];
        let mut row_height = 0.0_f64;
        let mut row_nonempty = false;
        for indices in components {
            check(cancel)?;
            let local = local_fragment(&fragment, &indices, cancel)?;
            let solved = solve_component(&local, length, origin, page, cancel, work, scenes)?;
            let pose = solved
                .pages()
                .first()
                .and_then(|page| page.fragments().first())
                .ok_or(LayoutError::Geometry("missing component pose"))?;
            if pose.atoms().len() != indices.len() {
                return Err(LayoutError::Geometry("incomplete component pose"));
            }
            let bounds = center_bounds(pose, cancel)?;
            let width = bounds[2] - bounds[0];
            let height = bounds[3] - bounds[1];
            if row_nonempty && cursor[0] + width > page.width_pt() {
                cursor[0] = origin.x_pt();
                cursor[1] += row_height + length;
                row_height = 0.0;
            }
            let right = cursor[0] + width;
            let bottom = cursor[1] + height;
            if !right.is_finite()
                || !bottom.is_finite()
                || right > page.width_pt()
                || bottom > page.height_pt()
            {
                return Err(LayoutError::Geometry("component shelves exceed paper"));
            }
            for atom in pose.atoms() {
                check(cancel)?;
                let local = usize::try_from(atom.id.get() - 1)
                    .map_err(|_| LayoutError::ComponentsAtomLimit)?;
                let original = *indices
                    .get(local)
                    .ok_or(LayoutError::Geometry("missing original component mapping"))?;
                let point = atom
                    .position
                    .ok_or(LayoutError::Geometry("missing component position"))?;
                *positions
                    .get_mut(original)
                    .ok_or(LayoutError::Geometry("missing original component atom"))? = Some([
                    point.x_pt() - bounds[0] + cursor[0],
                    point.y_pt() - bounds[1] + cursor[1],
                ]);
            }
            cursor[0] += width + length;
            row_height = row_height.max(height);
            row_nonempty = true;
        }
        let mut work = WORK_PER_ROTATION;
        if !geometry(
            &fragment, &positions, length, origin, page, &mut work, cancel,
        )? {
            return Err(LayoutError::Geometry("assembled component geometry"));
        }
        positioned_document(&fragment, &positions, origin, page, cancel)
    }
}

fn solve_component(
    fragment: &Fragment,
    length: f64,
    origin: Point2,
    page: PageSize,
    cancel: &Cancellation,
    work: usize,
    scenes: usize,
) -> Result<Document, LayoutError> {
    if fragment.bonds().len().checked_add(1) == Some(fragment.atoms().len()) {
        acyclic::layout_fragment(fragment, length, origin, page, cancel, work, scenes)
    } else if fragment.bonds().len() == fragment.atoms().len() {
        ring::layout_fragment(fragment, length, origin, page, cancel, work, scenes)
    } else {
        Err(LayoutError::Unsupported(
            "component other than a tree or single-cycle graph",
        ))
    }
}

fn component_indices(
    fragment: &Fragment,
    cancel: &Cancellation,
) -> Result<Vec<Vec<usize>>, LayoutError> {
    let count = fragment.atoms().len();
    if count == 0 {
        return Err(LayoutError::Unsupported("empty graph"));
    }
    let mut adjacency = vec![Vec::new(); count];
    for bond in fragment.bonds() {
        check(cancel)?;
        let a =
            usize::try_from(bond.begin.get() - 1).map_err(|_| LayoutError::ComponentsAtomLimit)?;
        let b =
            usize::try_from(bond.end.get() - 1).map_err(|_| LayoutError::ComponentsAtomLimit)?;
        for (at, other) in [(a, b), (b, a)] {
            adjacency
                .get_mut(at)
                .ok_or(LayoutError::Geometry("missing component endpoint"))?
                .push(other);
        }
    }
    for links in &mut adjacency {
        links.sort_unstable();
    }
    let mut seen = vec![false; count];
    let mut result = Vec::new();
    for start in 0..count {
        check(cancel)?;
        let visited = seen
            .get_mut(start)
            .ok_or(LayoutError::Geometry("missing component start"))?;
        if *visited {
            continue;
        }
        *visited = true;
        let mut pending = VecDeque::from([start]);
        let mut indices = Vec::new();
        while let Some(at) = pending.pop_front() {
            check(cancel)?;
            indices.push(at);
            for &other in adjacency
                .get(at)
                .ok_or(LayoutError::Geometry("missing component adjacency"))?
            {
                let visited = seen
                    .get_mut(other)
                    .ok_or(LayoutError::Geometry("missing component neighbor"))?;
                if !*visited {
                    *visited = true;
                    pending.push_back(other);
                }
            }
        }
        // Dense local views preserve the original array order, not BFS discovery order.
        indices.sort_unstable();
        result.push(indices);
    }
    Ok(result)
}

fn local_fragment(
    fragment: &Fragment,
    indices: &[usize],
    cancel: &Cancellation,
) -> Result<Fragment, LayoutError> {
    let mut local_ids = vec![None; fragment.atoms().len()];
    let mut atoms = Vec::with_capacity(indices.len());
    for &original in indices {
        check(cancel)?;
        let id = atoms
            .len()
            .checked_add(1)
            .and_then(|value| u32::try_from(value).ok())
            .and_then(AtomId::new)
            .ok_or(LayoutError::ComponentsAtomLimit)?;
        let mut atom = fragment
            .atoms()
            .get(original)
            .ok_or(LayoutError::Geometry("missing original atom"))?
            .clone();
        atom.id = id;
        *local_ids
            .get_mut(original)
            .ok_or(LayoutError::Geometry("missing local atom mapping"))? = Some(id);
        atoms.push(atom);
    }
    let mut bonds = Vec::new();
    for original in fragment.bonds() {
        check(cancel)?;
        let a = usize::try_from(original.begin.get() - 1)
            .map_err(|_| LayoutError::ComponentsAtomLimit)?;
        let b = usize::try_from(original.end.get() - 1)
            .map_err(|_| LayoutError::ComponentsAtomLimit)?;
        let begin = local_ids.get(a).copied().flatten();
        let end = local_ids.get(b).copied().flatten();
        match (begin, end) {
            (Some(begin), Some(end)) => {
                let mut bond = *original;
                bond.id = bonds
                    .len()
                    .checked_add(1)
                    .and_then(|value| u32::try_from(value).ok())
                    .and_then(BondId::new)
                    .ok_or(LayoutError::ComponentsAtomLimit)?;
                bond.begin = begin;
                bond.end = end;
                bonds.push(bond);
            }
            (None, None) => {}
            _ => return Err(LayoutError::Geometry("edge crosses discovered components")),
        }
    }
    Ok(Fragment::new(fragment.id(), atoms, bonds).map_err(bondline_model::Error::from)?)
}

fn center_bounds(fragment: &Fragment, cancel: &Cancellation) -> Result<[f64; 4], LayoutError> {
    let mut bounds = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for atom in fragment.atoms() {
        check(cancel)?;
        let point = atom
            .position
            .ok_or(LayoutError::Geometry("missing component bounds position"))?;
        bounds[0] = bounds[0].min(point.x_pt());
        bounds[1] = bounds[1].min(point.y_pt());
        bounds[2] = bounds[2].max(point.x_pt());
        bounds[3] = bounds[3].max(point.y_pt());
    }
    Ok(bounds)
}
