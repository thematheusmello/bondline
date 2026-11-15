//! One regular small ring with bounded attached-tree placement, never source mutation.

use super::{
    LayoutError, SmilesDocument,
    acyclic::{
        Link, MAX_ATOMS, SCENES_PER_ROTATION, Solver, WORK_PER_ROTATION, check, geometry,
        graph_adjacency, linear_centers, local_alternatives, root_tree, size,
    },
};
use bondline_model::{BondOrder, Document, Fragment, PageSize, Point2};
use bondline_render::Cancellation;
use std::{
    collections::VecDeque,
    f64::consts::{PI, TAU},
};

impl SmilesDocument {
    /// Derive one connected neutral, nonisotopic 3..=8-member Single ring and trees.
    /// Keeps fixed H, source-local identities and all directed edges unchanged.
    /// Minimum atom-center bounds start at `origin`; no length or paper scaling.
    /// The same literal drawing must pass attachment, actual-edge and whole-ink guards.
    pub fn layout_neutral_simple_ring(
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
            return Err(LayoutError::SimpleRingAtomLimit);
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
    if length > page.width_pt().hypot(page.height_pt()) {
        return Err(LayoutError::Geometry("bond length exceeds paper"));
    }
    if fragment.bonds().len() != fragment.atoms().len() {
        return Err(LayoutError::Unsupported("not a single-cycle graph"));
    }
    let adjacency = graph_adjacency(fragment)?;
    let cycle = cycle(&adjacency, cancel)?;
    let mut ring = vec![false; adjacency.len()];
    for &at in &cycle {
        *ring
            .get_mut(at)
            .ok_or(LayoutError::Geometry("missing ring atom"))? = true;
    }
    for &at in &cycle {
        if adjacency.get(at).into_iter().flatten().any(|link| {
            ring.get(link.atom).copied() == Some(true) && link.order != BondOrder::Single
        }) {
            return Err(LayoutError::Unsupported(
                "cycle bond order other than Single",
            ));
        }
    }
    let linear = linear_centers(fragment, &adjacency)?;
    let mut parent = vec![None; adjacency.len()];
    let mut children = vec![Vec::new(); adjacency.len()];
    let mut order = cycle.clone();
    for &at in &cycle {
        check(cancel)?;
        let mut branches = adjacency
            .get(at)
            .ok_or(LayoutError::Geometry("missing ring adjacency"))?
            .iter()
            .filter(|link| ring.get(link.atom).copied() == Some(false))
            .map(|link| link.atom)
            .collect::<Vec<_>>();
        // E=V and connectivity prove these components are trees, each with one ring root.
        branches.sort_by_key(|child| {
            (
                std::cmp::Reverse(size(*child, Some(at), &adjacency)),
                *child,
            )
        });
        *children
            .get_mut(at)
            .ok_or(LayoutError::Geometry("missing ring branches"))? = branches.clone();
        for child in branches {
            root_tree(
                child,
                Some(at),
                &adjacency,
                &mut parent,
                &mut children,
                &mut order,
                cancel,
            )?;
        }
    }
    if order.len() != adjacency.len() {
        return Err(LayoutError::Geometry("incomplete attached forest"));
    }
    let mut alternatives = vec![Vec::new(); adjacency.len()];
    for &at in order
        .iter()
        .filter(|at| ring.get(**at).copied() == Some(false))
    {
        let branches = children
            .get(at)
            .ok_or(LayoutError::Geometry("missing tree branches"))?;
        *alternatives
            .get_mut(at)
            .ok_or(LayoutError::Geometry("missing tree alternatives"))? = local_alternatives(
            at,
            branches,
            &adjacency,
            fragment,
            *linear
                .get(at)
                .ok_or(LayoutError::Geometry("missing linear policy"))?,
            Some(&ring),
            cancel,
        )?;
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
        last: LayoutError::Geometry("no permitted bounded ring placement fits"),
    };
    let n = cycle.len();
    let step = TAU / n as f64;
    let radius = length / (2.0 * (PI / n as f64).sin());
    let phase = if n == 6 {
        PI - PI / n as f64
    } else {
        PI / 2.0 - PI / n as f64
    };
    // Only symmetry-equivalent phases retain the sourced polygon orientation.
    for shift in 0..n {
        for direction in [1.0, -1.0] {
            check(cancel)?;
            solver.positions.fill(None);
            for (index, &at) in cycle.iter().enumerate() {
                let theta = phase + step * (shift as f64 + direction * index as f64);
                let (sin, cos) = theta.sin_cos();
                *solver
                    .positions
                    .get_mut(at)
                    .ok_or(LayoutError::Geometry("missing cycle position"))? =
                    Some([radius * cos, radius * sin]);
                let branches = solver
                    .children
                    .get(at)
                    .ok_or(LayoutError::Geometry("missing branches"))?;
                *solver
                    .alternatives
                    .get_mut(at)
                    .ok_or(LayoutError::Geometry("missing ring alternatives"))? =
                    ring_alternatives(at, branches, &adjacency, theta, n)?;
            }
            solver.work = work_per_rotation;
            solver.scenes = scenes_per_rotation;
            if geometry(
                fragment,
                &solver.positions,
                length,
                origin,
                page,
                &mut solver.work,
                cancel,
            )? && let Some(document) = solver.visit(0)?
            {
                return Ok(document);
            }
        }
    }
    check(cancel)?;
    Err(solver.last)
}

fn cycle(adjacency: &[Vec<Link>], cancel: &Cancellation) -> Result<Vec<usize>, LayoutError> {
    let mut degree = adjacency.iter().map(Vec::len).collect::<Vec<_>>();
    let mut core = vec![true; adjacency.len()];
    let mut leaves = degree
        .iter()
        .enumerate()
        .filter(|(_, d)| **d < 2)
        .map(|(at, _)| at)
        .collect::<VecDeque<_>>();
    while let Some(at) = leaves.pop_front() {
        check(cancel)?;
        let present = core
            .get_mut(at)
            .ok_or(LayoutError::Geometry("missing peeled atom"))?;
        if !*present {
            continue;
        }
        *present = false;
        for link in adjacency
            .get(at)
            .ok_or(LayoutError::Geometry("missing peeled adjacency"))?
        {
            if core.get(link.atom).copied() == Some(true) {
                let d = degree
                    .get_mut(link.atom)
                    .ok_or(LayoutError::Geometry("missing peeled degree"))?;
                *d = d
                    .checked_sub(1)
                    .ok_or(LayoutError::Geometry("invalid peeled degree"))?;
                if *d == 1 {
                    leaves.push_back(link.atom);
                }
            }
        }
    }
    let count = core.iter().filter(|v| **v).count();
    if !(3..=8).contains(&count) {
        return Err(LayoutError::Unsupported("simple ring size outside 3..8"));
    }
    if core
        .iter()
        .enumerate()
        .any(|(at, present)| *present && degree.get(at).copied() != Some(2))
    {
        return Err(LayoutError::Unsupported("cycle is not one simple loop"));
    }
    let start = core
        .iter()
        .position(|v| *v)
        .ok_or(LayoutError::Unsupported("no ring"))?;
    let mut result = Vec::with_capacity(count);
    let mut previous = None;
    let mut current = start;
    loop {
        check(cancel)?;
        if result.contains(&current) || result.len() >= count {
            return Err(LayoutError::Unsupported("cycle is not one simple loop"));
        }
        result.push(current);
        let next = adjacency
            .get(current)
            .ok_or(LayoutError::Geometry("missing cycle adjacency"))?
            .iter()
            .filter(|link| {
                core.get(link.atom).copied() == Some(true) && Some(link.atom) != previous
            })
            .map(|link| link.atom)
            .min()
            .ok_or(LayoutError::Unsupported("open cycle"))?;
        if next == start {
            break;
        }
        previous = Some(current);
        current = next;
    }
    if result.len() != count {
        return Err(LayoutError::Unsupported("more than one cycle"));
    }
    Ok(result)
}

fn ring_alternatives(
    at: usize,
    children: &[usize],
    adjacency: &[Vec<Link>],
    radial: f64,
    n: usize,
) -> Result<Vec<Vec<f64>>, LayoutError> {
    match children {
        [] => Ok(vec![vec![]]),
        [_] => {
            let nearest = (radial / (PI / 6.0)).round() * (PI / 6.0);
            let angle = if (nearest - radial).abs() <= 5.0_f64.to_radians() + 1e-12 {
                nearest
            } else {
                radial
            };
            Ok(vec![vec![angle]])
        }
        [a, b] => {
            let mut result = Vec::new();
            if [a, b]
                .iter()
                .all(|child| adjacency.get(**child).is_some_and(|a| a.len() >= 3))
            {
                let half = (PI + TAU / n as f64) / 6.0;
                pair(&mut result, radial - half, radial + half);
            } else {
                let sa = size(*a, Some(at), adjacency);
                let sb = size(*b, Some(at), adjacency);
                if n == 6 && ((sa == 1 && sb > 1) || (sb == 1 && sa > 1)) {
                    let mut sides = [radial - PI / 3.0, radial + PI / 3.0];
                    sides.sort_by(|a, b| direction_order(*a, *b));
                    for side in sides {
                        result.push(if sa > sb {
                            vec![radial, side]
                        } else {
                            vec![side, radial]
                        });
                    }
                } else {
                    // Symmetry is the original first candidate, not a unique IUPAC requirement.
                    for offset in [0.0_f64, 15.0, -15.0, 30.0, -30.0, 45.0, -45.0] {
                        let center = radial + offset.to_radians();
                        pair(&mut result, center - PI / 6.0, center + PI / 6.0);
                    }
                    if n == 4 {
                        for offset in [0.0_f64, 15.0, -15.0, 30.0, -30.0] {
                            let center = radial + offset.to_radians();
                            pair(&mut result, center - PI / 4.0, center + PI / 4.0);
                        }
                    }
                }
            }
            Ok(result)
        }
        _ => Err(LayoutError::Unsupported(
            "more than two external ring bonds",
        )),
    }
}

fn pair(result: &mut Vec<Vec<f64>>, a: f64, b: f64) {
    if direction_order(a, b).is_le() {
        result.push(vec![a, b]);
        result.push(vec![b, a]);
    } else {
        result.push(vec![b, a]);
        result.push(vec![a, b]);
    }
}

fn direction_order(a: f64, b: f64) -> std::cmp::Ordering {
    let (sa, ca) = a.sin_cos();
    let (sb, cb) = b.sin_cos();
    let axis = |sin: f64, cos: f64| sin.abs() < 1e-10 || cos.abs() < 1e-10;
    (!axis(sa, ca))
        .cmp(&(!axis(sb, cb)))
        .then_with(|| {
            if (ca - cb).abs() <= 1e-10 {
                std::cmp::Ordering::Equal
            } else {
                cb.total_cmp(&ca)
            }
        })
        .then_with(|| sa.total_cmp(&sb))
}
