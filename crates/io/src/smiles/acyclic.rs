//! Deterministic bounded tree placement; no chemical graph or source mutation.

use super::{LayoutError, SmilesDocument, layout::validate_ink_with_cancel};
use bondline_model::{BondOrder, Document, Fragment, PageId, PageSize, Point2};
use bondline_render::{Cancellation, GraphScene};
use std::collections::BTreeMap;

pub(super) const MAX_ATOMS: usize = 64;
const REFERENCE_SIZE: u32 = 1024;
pub(super) const WORK_PER_ROTATION: usize = 100_000;
pub(super) const SCENES_PER_ROTATION: usize = 64;

#[derive(Clone, Copy)]
pub(super) struct Link {
    pub(super) atom: usize,
    pub(super) order: BondOrder,
}

pub(super) fn check(cancel: &Cancellation) -> Result<(), LayoutError> {
    if cancel.is_cancelled() {
        Err(LayoutError::Cancelled)
    } else {
        Ok(())
    }
}

impl SmilesDocument {
    /// Derive one neutral, nonisotopic connected tree with fixed authored H.
    /// Uses sourced local angle families and bounded original branch alternatives.
    /// Atom-center bounds start at `origin`; all bond lengths stay exactly supplied.
    /// Whole literal/reference-scene ink, attachment and edge clearance must succeed.
    pub fn layout_neutral_acyclic(
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
            return Err(LayoutError::AcyclicAtomLimit);
        }
        let fragment = self.normalize_explicit_h_fragment()?;
        layout_fragment(
            &fragment,
            length,
            origin,
            page,
            cancel,
            WORK_PER_ROTATION,
            SCENES_PER_ROTATION,
        )
    }
}

pub(super) fn layout_fragment(
    fragment: &Fragment,
    length: f64,
    origin: Point2,
    page: PageSize,
    cancel: &Cancellation,
    work_per_rotation: usize,
    scenes_per_rotation: usize,
) -> Result<Document, LayoutError> {
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
    let adjacency = adjacency(fragment)?;
    let root = longest_endpoint(&adjacency, cancel)?;
    let linear = linear_centers(fragment, &adjacency)?;
    let mut parent = vec![None; adjacency.len()];
    let mut children = vec![Vec::new(); adjacency.len()];
    let mut order = Vec::with_capacity(adjacency.len());
    root_tree(
        root,
        None,
        &adjacency,
        &mut parent,
        &mut children,
        &mut order,
        cancel,
    )?;
    let mut alternatives = vec![Vec::new(); adjacency.len()];
    for &at in &order {
        let branches = children
            .get(at)
            .ok_or(LayoutError::Geometry("missing branches"))?;
        let values = if at == root {
            vec![vec![0.0; branches.len()]]
        } else {
            local_alternatives(
                at,
                branches,
                &adjacency,
                fragment,
                *linear
                    .get(at)
                    .ok_or(LayoutError::Geometry("missing angle policy"))?,
                None,
                cancel,
            )?
        };
        *alternatives
            .get_mut(at)
            .ok_or(LayoutError::Geometry("missing alternatives"))? = values;
    }
    let mut solver = Solver {
        fragment,
        parent,
        children,
        order,
        alternatives,
        positions: vec![None; adjacency.len()],
        length,
        origin,
        page,
        cancel,
        work: 0,
        scenes: 0,
        last: LayoutError::Geometry("no permitted bounded tree placement fits"),
    };
    let rotations = if adjacency.len() == 1 { 1 } else { 24 };
    for rotation in 0..rotations {
        check(cancel)?;
        solver.positions.fill(None);
        *solver
            .positions
            .get_mut(root)
            .ok_or(LayoutError::Geometry("missing root"))? = Some([0.0, 0.0]);
        let degrees = if rotation < 12 {
            f64::from(rotation) * 30.0
        } else {
            15.0 + f64::from(rotation - 12) * 30.0
        };
        let initial = solver
            .alternatives
            .get_mut(root)
            .ok_or(LayoutError::Geometry("missing root angles"))?;
        *initial = vec![vec![
            degrees.to_radians();
            solver.children.get(root).map_or(0, Vec::len)
        ]];
        solver.work = work_per_rotation;
        solver.scenes = scenes_per_rotation;
        if let Some(document) = solver.visit(0)? {
            return Ok(document);
        }
    }
    check(cancel)?;
    Err(solver.last)
}

pub(super) fn linear_centers(
    fragment: &Fragment,
    adjacency: &[Vec<Link>],
) -> Result<Vec<bool>, LayoutError> {
    linear_centers_with_cycle(fragment, adjacency, &[])
}

pub(super) fn linear_centers_with_cycle(
    fragment: &Fragment,
    adjacency: &[Vec<Link>],
    cycle: &[bool],
) -> Result<Vec<bool>, LayoutError> {
    let mut linear = vec![false; adjacency.len()];
    for (at, links) in adjacency.iter().enumerate() {
        if links.len() == 2
            && fragment
                .atoms()
                .get(at)
                .is_some_and(|a| a.element.get() == 6)
            && (links.iter().any(|l| l.order == BondOrder::Triple)
                || links.iter().all(|l| l.order == BondOrder::Double))
            && let Some(value) = linear.get_mut(at)
        {
            *value = true;
        }
        for link in links.iter().filter(|link| link.order == BondOrder::Double) {
            if cycle.get(at) == Some(&true) && cycle.get(link.atom) == Some(&true) {
                continue;
            }
            let other = adjacency
                .get(link.atom)
                .ok_or(LayoutError::Geometry("missing neighbor"))?;
            if links.len() > 1 && other.len() > 1 {
                if links.len() != 2 || other.len() != 2 {
                    return Err(LayoutError::Unsupported(
                        "internal Double configuration has no unknown depiction",
                    ));
                }
                if let Some(value) = linear.get_mut(at) {
                    *value = true;
                }
            }
        }
    }
    Ok(linear)
}

fn adjacency(fragment: &Fragment) -> Result<Vec<Vec<Link>>, LayoutError> {
    let count = fragment.atoms().len();
    if count == 0 || fragment.bonds().len().checked_add(1) != Some(count) {
        return Err(LayoutError::Unsupported("not a connected tree"));
    }
    graph_adjacency(fragment)
}

pub(super) fn graph_adjacency(fragment: &Fragment) -> Result<Vec<Vec<Link>>, LayoutError> {
    let count = fragment.atoms().len();
    if count == 0 {
        return Err(LayoutError::Unsupported("empty graph"));
    }
    let mut result = vec![Vec::new(); count];
    for bond in fragment.bonds() {
        let a = usize::try_from(bond.begin.get() - 1).map_err(|_| LayoutError::AcyclicAtomLimit)?;
        let b = usize::try_from(bond.end.get() - 1).map_err(|_| LayoutError::AcyclicAtomLimit)?;
        for (at, other) in [(a, b), (b, a)] {
            result
                .get_mut(at)
                .ok_or(LayoutError::Geometry("missing bond endpoint"))?
                .push(Link {
                    atom: other,
                    order: bond.order,
                });
        }
    }
    let mut seen = vec![false; count];
    let mut pending = vec![0];
    while let Some(at) = pending.pop() {
        let value = seen
            .get_mut(at)
            .ok_or(LayoutError::Geometry("missing connected atom"))?;
        if *value {
            continue;
        }
        *value = true;
        for link in result
            .get(at)
            .ok_or(LayoutError::Geometry("missing adjacency"))?
        {
            pending.push(link.atom);
        }
    }
    if seen.iter().any(|v| !*v) {
        return Err(LayoutError::Unsupported("disconnected graph"));
    }
    Ok(result)
}

fn longest_endpoint(adjacency: &[Vec<Link>], cancel: &Cancellation) -> Result<usize, LayoutError> {
    let mut best = (0, 0, 0);
    for start in 0..adjacency.len() {
        check(cancel)?;
        let mut pending = vec![(start, None, 0)];
        while let Some((at, parent, distance)) = pending.pop() {
            if distance > best.0 {
                best = (distance, start, at);
            }
            for link in adjacency
                .get(at)
                .ok_or(LayoutError::Geometry("missing adjacency"))?
            {
                if Some(link.atom) != parent {
                    pending.push((link.atom, Some(at), distance + 1));
                }
            }
        }
    }
    Ok(best.1)
}

pub(super) fn size(at: usize, parent: Option<usize>, adjacency: &[Vec<Link>]) -> usize {
    1 + adjacency
        .get(at)
        .into_iter()
        .flatten()
        .filter(|l| Some(l.atom) != parent)
        .map(|l| size(l.atom, Some(at), adjacency))
        .sum::<usize>()
}

pub(super) fn root_tree(
    at: usize,
    from: Option<usize>,
    adjacency: &[Vec<Link>],
    parent: &mut [Option<usize>],
    children: &mut [Vec<usize>],
    order: &mut Vec<usize>,
    cancel: &Cancellation,
) -> Result<(), LayoutError> {
    check(cancel)?;
    *parent
        .get_mut(at)
        .ok_or(LayoutError::Geometry("missing parent"))? = from;
    let mut branches = adjacency
        .get(at)
        .ok_or(LayoutError::Geometry("missing adjacency"))?
        .iter()
        .filter(|l| Some(l.atom) != from)
        .map(|l| l.atom)
        .collect::<Vec<_>>();
    branches.sort_by_key(|child| (std::cmp::Reverse(size(*child, Some(at), adjacency)), *child));
    *children
        .get_mut(at)
        .ok_or(LayoutError::Geometry("missing branch storage"))? = branches.clone();
    order.push(at);
    for child in branches {
        root_tree(child, Some(at), adjacency, parent, children, order, cancel)?;
    }
    Ok(())
}

fn key(
    at: usize,
    parent: usize,
    adjacency: &[Vec<Link>],
    fragment: &Fragment,
    ring_boundary: Option<&[bool]>,
) -> Result<String, LayoutError> {
    let atom = fragment
        .atoms()
        .get(at)
        .ok_or(LayoutError::Geometry("missing signature atom"))?;
    // A ring-containing ligand is distinct from every attached acyclic ligand.
    if ring_boundary.and_then(|boundary| boundary.get(at)).copied() == Some(true) {
        return Ok(format!(
            "{}:{:?}(ring)",
            atom.element.get(),
            atom.explicit_hydrogens
        ));
    }
    let mut branches = Vec::new();
    for link in adjacency
        .get(at)
        .ok_or(LayoutError::Geometry("missing signature adjacency"))?
    {
        if link.atom != parent {
            branches.push(format!(
                "{:?}:{}",
                link.order,
                key(link.atom, at, adjacency, fragment, ring_boundary)?
            ));
        }
    }
    branches.sort();
    Ok(format!(
        "{}:{:?}({})",
        atom.element.get(),
        atom.explicit_hydrogens,
        branches.join(",")
    ))
}

pub(super) fn local_alternatives(
    at: usize,
    children: &[usize],
    adjacency: &[Vec<Link>],
    fragment: &Fragment,
    linear: bool,
    ring_boundary: Option<&[bool]>,
    cancel: &Cancellation,
) -> Result<Vec<Vec<f64>>, LayoutError> {
    check(cancel)?;
    let links = adjacency
        .get(at)
        .ok_or(LayoutError::Geometry("missing angle adjacency"))?;
    let templates: Vec<Vec<f64>> = match links.len() {
        1 => vec![vec![0.0]],
        2 => vec![vec![0.0, if linear { 180.0 } else { 120.0 }]],
        3 => vec![vec![0.0, 120.0, 240.0]],
        4 => {
            let terminal = links
                .iter()
                .filter(|l| adjacency.get(l.atom).is_some_and(|a| a.len() == 1))
                .collect::<Vec<_>>();
            let mut identities = links
                .iter()
                .map(|l| key(l.atom, at, adjacency, fragment, ring_boundary))
                .collect::<Result<Vec<_>, _>>()?;
            identities.sort();
            identities.dedup();
            let distinct = identities.len() == 4;
            let carbon_terminals = terminal
                .iter()
                .filter(|link| {
                    fragment
                        .atoms()
                        .get(link.atom)
                        .is_some_and(|atom| atom.element.get() == 6)
                })
                .count();
            let hetero_terminals = terminal
                .iter()
                .filter(|link| {
                    fragment
                        .atoms()
                        .get(link.atom)
                        .is_some_and(|atom| !matches!(atom.element.get(), 1 | 6))
                })
                .map(|link| key(link.atom, at, adjacency, fragment, ring_boundary))
                .collect::<Result<Vec<_>, _>>()?;
            let repeated_hetero = hetero_terminals.iter().enumerate().any(|(index, key)| {
                hetero_terminals
                    .iter()
                    .skip(index + 1)
                    .any(|other| other == key)
            });
            if !distinct
                && (terminal.len() == 4
                    || (terminal.is_empty()
                        && links
                            .iter()
                            .all(|l| adjacency.get(l.atom).is_some_and(|a| a.len() >= 3))))
            {
                vec![vec![0.0, 90.0, 180.0, 270.0]]
            } else if terminal.len() == 1 {
                vec![vec![0.0, 60.0, 120.0, 240.0]]
            } else if terminal.len() == 2 {
                vec![vec![0.0, 60.0, 150.0, 270.0]]
            } else if terminal.len() == 3
                && terminal
                    .iter()
                    .map(|l| key(l.atom, at, adjacency, fragment, ring_boundary))
                    .collect::<Result<std::collections::BTreeSet<_>, _>>()?
                    .len()
                    == 1
            {
                vec![vec![0.0, 60.0, 120.0, 240.0]]
            } else if terminal.len() == 3 && (carbon_terminals == 1 || repeated_hetero) {
                vec![vec![0.0, 60.0, 150.0, 270.0]]
            } else {
                vec![vec![0.0, 60.0, 150.0, 270.0], vec![0.0, 60.0, 120.0, 240.0]]
            }
        }
        _ => return Err(LayoutError::Unsupported("unsupported drawn adjacency")),
    };
    if children.is_empty() {
        return Ok(vec![vec![]]);
    }
    let mut result = Vec::new();
    for template in templates {
        for mirror in [1.0, -1.0] {
            for parent_slot in 0..template.len() {
                let base = *template
                    .get(parent_slot)
                    .ok_or(LayoutError::Geometry("missing parent slot"))?;
                let mut slots = template
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != parent_slot)
                    .map(|(_, a)| ((a - base) * mirror).to_radians())
                    .collect::<Vec<_>>();
                permutations(&mut slots, 0, &mut result);
            }
        }
    }
    result.dedup_by(|a, b| a == b);
    Ok(result)
}

fn permutations(values: &mut [f64], at: usize, result: &mut Vec<Vec<f64>>) {
    if at == values.len() {
        result.push(values.to_vec());
        return;
    }
    for other in at..values.len() {
        values.swap(at, other);
        permutations(values, at + 1, result);
        values.swap(at, other);
    }
}

pub(super) struct Solver<'a> {
    pub(super) fragment: &'a Fragment,
    pub(super) parent: Vec<Option<usize>>,
    pub(super) children: Vec<Vec<usize>>,
    pub(super) order: Vec<usize>,
    pub(super) alternatives: Vec<Vec<Vec<f64>>>,
    pub(super) positions: Vec<Option<[f64; 2]>>,
    pub(super) length: f64,
    pub(super) origin: Point2,
    pub(super) page: PageSize,
    pub(super) cancel: &'a Cancellation,
    pub(super) work: usize,
    pub(super) scenes: usize,
    pub(super) last: LayoutError,
}

impl Solver<'_> {
    pub(super) fn visit(&mut self, index: usize) -> Result<Option<Document>, LayoutError> {
        check(self.cancel)?;
        if self.work == 0 || self.scenes == 0 {
            return Ok(None);
        }
        if index == self.order.len() {
            self.scenes -= 1;
            match self.document() {
                Ok(document) => return Ok(Some(document)),
                Err(LayoutError::Cancelled) => return Err(LayoutError::Cancelled),
                Err(error) => self.last = error,
            }
            return Ok(None);
        }
        let at = *self
            .order
            .get(index)
            .ok_or(LayoutError::Geometry("missing traversal atom"))?;
        let children = self
            .children
            .get(at)
            .ok_or(LayoutError::Geometry("missing children"))?
            .clone();
        if children.is_empty() {
            return self.visit(index + 1);
        }
        let point = self
            .positions
            .get(at)
            .copied()
            .flatten()
            .ok_or(LayoutError::Geometry("missing placed parent"))?;
        let incoming = match self.parent.get(at).copied().flatten() {
            None => 0.0,
            Some(parent) => {
                let p = self
                    .positions
                    .get(parent)
                    .copied()
                    .flatten()
                    .ok_or(LayoutError::Geometry("missing parent position"))?;
                (p[1] - point[1]).atan2(p[0] - point[0])
            }
        };
        let count = self.alternatives.get(at).map_or(0, Vec::len);
        for choice in 0..count {
            check(self.cancel)?;
            if self.work == 0 || self.scenes == 0 {
                break;
            }
            let angles = self
                .alternatives
                .get(at)
                .and_then(|a| a.get(choice))
                .ok_or(LayoutError::Geometry("missing alternative"))?
                .clone();
            for (child, angle) in children.iter().zip(angles) {
                let (sin, cos) = (incoming + angle).sin_cos();
                *self
                    .positions
                    .get_mut(*child)
                    .ok_or(LayoutError::Geometry("missing child position"))? =
                    Some([point[0] + self.length * cos, point[1] + self.length * sin]);
            }
            if geometry(
                self.fragment,
                &self.positions,
                self.length,
                self.origin,
                self.page,
                &mut self.work,
                self.cancel,
            )? && let Some(document) = self.visit(index + 1)?
            {
                return Ok(Some(document));
            }
            for child in &children {
                *self
                    .positions
                    .get_mut(*child)
                    .ok_or(LayoutError::Geometry("missing cleared child"))? = None;
            }
        }
        Ok(None)
    }

    fn document(&self) -> Result<Document, LayoutError> {
        positioned_document(
            self.fragment,
            &self.positions,
            self.origin,
            self.page,
            self.cancel,
        )
    }
}

pub(super) fn positioned_document(
    fragment: &Fragment,
    positions: &[Option<[f64; 2]>],
    origin: Point2,
    page_size: PageSize,
    cancel: &Cancellation,
) -> Result<Document, LayoutError> {
    let min_x = positions
        .iter()
        .flatten()
        .map(|p| p[0])
        .fold(f64::INFINITY, f64::min);
    let min_y = positions
        .iter()
        .flatten()
        .map(|p| p[1])
        .fold(f64::INFINITY, f64::min);
    let mut atoms = fragment.atoms().to_vec();
    for (atom, position) in atoms.iter_mut().zip(positions) {
        let point = position.ok_or(LayoutError::Geometry("incomplete candidate"))?;
        atom.position = Some(Point2::new(
            point[0] - min_x + origin.x_pt(),
            point[1] - min_y + origin.y_pt(),
        )?);
    }
    let mut document = Document::default();
    let id = PageId::new(1).ok_or(LayoutError::Geometry("invalid page identity"))?;
    document.set_page_size(id, page_size)?;
    document.append_fragments(
        id,
        vec![
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())
                .map_err(bondline_model::Error::from)?,
        ],
    )?;
    let page = document
        .pages()
        .first()
        .ok_or(LayoutError::Geometry("missing candidate page"))?;
    let literal = validate_ink_with_cancel(page, cancel)?;
    let counts = fragment
        .atoms()
        .iter()
        .map(|atom| {
            atom.explicit_hydrogens
                .map(|h| (atom.id, h))
                .ok_or(LayoutError::Geometry("missing normalized H"))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let resolved = bondline_render::graph_page_scene_with_display_hydrogens(
        REFERENCE_SIZE,
        REFERENCE_SIZE,
        page,
        &counts,
        cancel,
    )?;
    check(cancel)?;
    if literal.scene().primitives != resolved.scene().primitives {
        return Err(LayoutError::Geometry(
            "literal H attachment differs from validated attachment",
        ));
    }
    stroke_clearance(fragment, &literal, cancel)?;
    Ok(document)
}

fn spend(work: &mut usize, cancel: &Cancellation) -> Result<bool, LayoutError> {
    check(cancel)?;
    if *work == 0 {
        return Ok(false);
    }
    *work -= 1;
    Ok(true)
}

pub(super) fn geometry(
    fragment: &Fragment,
    positions: &[Option<[f64; 2]>],
    length: f64,
    origin: Point2,
    page: PageSize,
    work: &mut usize,
    cancel: &Cancellation,
) -> Result<bool, LayoutError> {
    let points = positions
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.map(|p| (i, p)))
        .collect::<Vec<_>>();
    for axis in 0..2 {
        let low = points
            .iter()
            .map(|(_, p)| p[axis])
            .fold(f64::INFINITY, f64::min);
        let high = points
            .iter()
            .map(|(_, p)| p[axis])
            .fold(f64::NEG_INFINITY, f64::max);
        let available = if axis == 0 {
            page.width_pt() - origin.x_pt()
        } else {
            page.height_pt() - origin.y_pt()
        };
        if !low.is_finite() || !high.is_finite() || high - low > available {
            return Ok(false);
        }
    }
    let epsilon = length.max(1.0) * 1e-10;
    let cross_epsilon = epsilon * length.max(1.0);
    let edges = fragment
        .bonds()
        .iter()
        .filter_map(|bond| {
            let a = usize::try_from(bond.begin.get() - 1).ok()?;
            let b = usize::try_from(bond.end.get() - 1).ok()?;
            Some((
                a,
                b,
                positions.get(a).copied().flatten()?,
                positions.get(b).copied().flatten()?,
            ))
        })
        .collect::<Vec<_>>();
    for (i, (at, p)) in points.iter().enumerate() {
        for (_, q) in points.iter().skip(i + 1) {
            if !spend(work, cancel)? || (p[0] - q[0]).hypot(p[1] - q[1]) <= epsilon {
                return Ok(false);
            }
        }
        for (a, b, u, v) in &edges {
            if *at != *a
                && *at != *b
                && (!spend(work, cancel)? || on_segment(*u, *v, *p, epsilon, cross_epsilon))
            {
                return Ok(false);
            }
        }
    }
    for (i, (a, b, u, v)) in edges.iter().enumerate() {
        for (c, d, p, q) in edges.iter().skip(i + 1) {
            if [a, b].iter().any(|at| *at == c || *at == d) {
                continue;
            }
            if !spend(work, cancel)? || intersects(*u, *v, *p, *q, epsilon, cross_epsilon) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn cross(a: [f64; 2], b: [f64; 2], p: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
}
fn on_segment(a: [f64; 2], b: [f64; 2], p: [f64; 2], epsilon: f64, area: f64) -> bool {
    cross(a, b, p).abs() <= area
        && (0..2).all(|axis| {
            p[axis] >= a[axis].min(b[axis]) - epsilon && p[axis] <= a[axis].max(b[axis]) + epsilon
        })
}
fn intersects(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2], epsilon: f64, area: f64) -> bool {
    let opposite = |a: f64, b: f64| (a > area && b < -area) || (a < -area && b > area);
    (opposite(cross(a, b, c), cross(a, b, d)) && opposite(cross(c, d, a), cross(c, d, b)))
        || on_segment(a, b, c, epsilon, area)
        || on_segment(a, b, d, epsilon, area)
        || on_segment(c, d, a, epsilon, area)
        || on_segment(c, d, b, epsilon, area)
}

fn stroke_clearance(
    fragment: &Fragment,
    scene: &GraphScene,
    cancel: &Cancellation,
) -> Result<(), LayoutError> {
    for (i, bond) in fragment.bonds().iter().enumerate() {
        for other in fragment.bonds().iter().skip(i + 1) {
            check(cancel)?;
            if [bond.begin, bond.end]
                .iter()
                .any(|a| *a == other.begin || *a == other.end)
            {
                continue;
            }
            for a in scene.bond_strokes(fragment.id(), bond.id) {
                for b in scene.bond_strokes(fragment.id(), other.id) {
                    if polygons_overlap(&a.corners()?, &b.corners()?) {
                        return Err(LayoutError::Geometry("nonincident bond ink overlaps"));
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn polygons_overlap(a: &[[f32; 2]; 4], b: &[[f32; 2]; 4]) -> bool {
    for polygon in [a, b] {
        for (index, p) in polygon.iter().enumerate() {
            let Some(q) = polygon.get((index + 1) % 4) else {
                return true;
            };
            let normal = [f64::from(q[1] - p[1]), f64::from(p[0] - q[0])];
            let interval = |points: &[[f32; 2]; 4]| {
                points
                    .iter()
                    .map(|p| f64::from(p[0]) * normal[0] + f64::from(p[1]) * normal[1])
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), v| {
                        (min.min(v), max.max(v))
                    })
            };
            let (amin, amax) = interval(a);
            let (bmin, bmax) = interval(b);
            if amax < bmin || bmax < amin {
                return false;
            }
        }
    }
    true
}
