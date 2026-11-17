//! Bounded native ring construction; existing source and shared geometry stay authoritative.
use crate::{Error, OrderHydrogenPolicy, bond};
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Page, PageId, Point2,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod chair;
mod request;
pub use chair::ChairKind;
pub(crate) use request::present_chair_kind;
pub(crate) use request::present_kind;
pub use request::{ChairRingCreate, NamedRingCreate, RingCreateRequest, RingKind, RingPhase};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RingSide {
    Positive,
    Negative,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum RingPlacement {
    Free {
        #[schemars(range(min = -100000, max = 100000))]
        begin_x_pt: f64,
        #[schemars(range(min = -100000, max = 100000))]
        begin_y_pt: f64,
        #[schemars(range(min = -100000, max = 100000))]
        end_x_pt: f64,
        #[schemars(range(min = -100000, max = 100000))]
        end_y_pt: f64,
        side: RingSide,
    },
    Atom {
        #[schemars(range(min = 1))]
        fragment_id: u32,
        #[schemars(range(min = 1))]
        atom_id: u32,
        #[schemars(range(min = -100000, max = 100000))]
        end_x_pt: f64,
        #[schemars(range(min = -100000, max = 100000))]
        end_y_pt: f64,
        side: RingSide,
    },
    Bond {
        #[schemars(range(min = 1))]
        fragment_id: u32,
        #[schemars(range(min = 1))]
        bond_id: u32,
        side: RingSide,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RingCreate {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 3, max = 8))]
    pub size: u8,
    pub hydrogen_policy: OrderHydrogenPolicy,
    pub placement: RingPlacement,
}

fn fragment(page: &Page, id: u32) -> Result<&Fragment, Error> {
    let id = FragmentId::new(id).ok_or(Error::InvalidFragmentId)?;
    page.fragments()
        .iter()
        .find(|fragment| fragment.id() == id)
        .map(AsRef::as_ref)
        .ok_or_else(|| bondline_model::Error::MissingFragment(id.get()).into())
}

fn atom(fragment: &Fragment, id: u32) -> Result<&Atom, Error> {
    let id = AtomId::new(id).ok_or(Error::InvalidRing("atom ID must be positive"))?;
    fragment
        .atoms()
        .iter()
        .find(|atom| atom.id == id)
        .ok_or(Error::MissingAtom {
            fragment_id: fragment.id().get(),
            atom_id: id.get(),
        })
}

fn position(atom: &Atom) -> Result<Point2, Error> {
    atom.position.ok_or(Error::UnsupportedRing(
        "a shared atom has no authored position",
    ))
}

fn remaining_hydrogens(fragment: &Fragment, atom: &Atom, consumed: u16) -> Result<u16, Error> {
    let unsupported = || {
        Error::UnsupportedRing(
            "shared atoms need neutral natural carbon, supported incident orders and consistent sufficient literal H",
        )
    };
    bond::remaining_carbon_hydrogens(fragment, atom, consumed).ok_or_else(unsupported)
}

fn polygon(size: u8, begin: Point2, end: Point2, side: RingSide) -> Result<Vec<Point2>, Error> {
    if begin == end {
        return Err(Error::InvalidRing(
            "the first edge needs distinct positions",
        ));
    }
    let angle = std::f64::consts::TAU / f64::from(size)
        * match side {
            RingSide::Positive => 1.0,
            RingSide::Negative => -1.0,
        };
    let (sin, cos) = angle.sin_cos();
    let mut edge = (end.x_pt() - begin.x_pt(), end.y_pt() - begin.y_pt());
    let mut points = vec![begin, end];
    while points.len() < usize::from(size) {
        edge = (cos * edge.0 - sin * edge.1, sin * edge.0 + cos * edge.1);
        let previous = points
            .last()
            .ok_or(Error::InvalidRing("missing polygon vertex"))?;
        let point = Point2::new(previous.x_pt() + edge.0, previous.y_pt() + edge.1)?;
        if points.contains(&point) {
            return Err(Error::InvalidRing(
                "polygon vertices collapse at this precision",
            ));
        }
        points.push(point);
    }
    // Distinct rounded vertices can still turn a regular triangle into a right triangle.
    let length = (end.x_pt() - begin.x_pt()).hypot(end.y_pt() - begin.y_pt());
    let precision_error =
        || Error::InvalidRing("ring geometry loses regularity at this coordinate precision");
    if !length.is_finite() || length == 0.0 {
        return Err(precision_error());
    }
    let expected_angle = (f64::from(size) - 2.0) * std::f64::consts::PI / f64::from(size);
    let mut previous = *points.last().ok_or_else(precision_error)?;
    for (point, next) in points.iter().zip(points.iter().cycle().skip(1)) {
        let u = (
            (previous.x_pt() - point.x_pt()) / length,
            (previous.y_pt() - point.y_pt()) / length,
        );
        let v = (
            (next.x_pt() - point.x_pt()) / length,
            (next.y_pt() - point.y_pt()) / length,
        );
        if [u.0.hypot(u.1), v.0.hypot(v.1)]
            .iter()
            .any(|norm| !norm.is_finite() || (norm - 1.0).abs() > 1e-10)
        {
            return Err(precision_error());
        }
        let cross = u.0 * v.1 - u.1 * v.0;
        let angle = cross.abs().atan2(u.0 * v.0 + u.1 * v.1);
        let oriented = match side {
            RingSide::Positive => cross < 0.0,
            RingSide::Negative => cross > 0.0,
        };
        if !oriented || !angle.is_finite() || (angle - expected_angle).abs() > 1e-10 {
            return Err(precision_error());
        }
        previous = *point;
    }
    Ok(points)
}

#[derive(Clone, Copy)]
enum SharedBudget {
    Free,
    Atom(u16),
    Bond { begin: u16, end: u16, weight: u16 },
}

fn named_weights(
    kind: RingKind,
    phase: Option<RingPhase>,
    budget: SharedBudget,
) -> Result<Vec<u16>, Error> {
    let base = kind.base_weights();
    let size = base.len();
    let last = size
        .checked_sub(1)
        .ok_or(Error::InvalidRing("empty ring pattern"))?;
    let rotations = match kind {
        RingKind::Benzene => 2,
        RingKind::Cyclopentadiene => 5,
    };
    for rotation in 0..rotations {
        let at = |index: usize| -> Result<u16, Error> {
            base.get((rotation + index) % size)
                .copied()
                .ok_or(Error::InvalidRing("incomplete ring pattern"))
        };
        let first = at(0)?;
        if match phase {
            None => false,
            Some(RingPhase::DoubleFirst) => first != 2,
            Some(RingPhase::SingleFirst) => first != 1,
        } {
            continue;
        }
        let fits = match budget {
            SharedBudget::Free => true,
            SharedBudget::Atom(available) => first
                .checked_add(at(last)?)
                .is_some_and(|needed| needed <= available),
            SharedBudget::Bond { begin, end, weight } => {
                first == weight && at(last)? <= begin && at(1)? <= end
            }
        };
        if fits {
            return (0..size).map(at).collect();
        }
    }
    Err(Error::UnsupportedRing(
        "no permitted ring phase matches the shared edge and available literal H",
    ))
}

fn weight_at(weights: &[u16], index: usize) -> Result<u16, Error> {
    weights
        .get(index)
        .copied()
        .ok_or(Error::InvalidRing("incomplete ring pattern"))
}

fn incident_weight(weights: &[u16], index: usize) -> Result<u16, Error> {
    let previous = if index == 0 {
        weights.len().checked_sub(1)
    } else {
        index.checked_sub(1)
    }
    .ok_or(Error::InvalidRing("empty ring pattern"))?;
    weight_at(weights, previous)?
        .checked_add(weight_at(weights, index)?)
        .ok_or(Error::InvalidRing("ring bond-order sum is too large"))
}

fn subtract_new_weight(available: u16, needed: u16) -> Result<u16, Error> {
    available.checked_sub(needed).ok_or(Error::UnsupportedRing(
        "the selected ring phase exceeds available literal H",
    ))
}

pub(super) fn create(
    document: &StoredDocument,
    params: RingCreate,
    floor: bond::Ids,
) -> Result<(StoredDocument, bond::Ids), Error> {
    create_inner(document, params, floor, Pattern::Regular)
}

#[derive(Clone, Copy)]
enum Pattern {
    Regular,
    Named(RingKind, Option<RingPhase>),
    Chair(ChairKind),
}

pub(super) fn create_request(
    document: &StoredDocument,
    request: RingCreateRequest,
    floor: bond::Ids,
) -> Result<(StoredDocument, bond::Ids), Error> {
    match request {
        RingCreateRequest::Sized(params) => create(document, params, floor),
        RingCreateRequest::Named(params) => {
            let NamedRingCreate {
                expected_revision,
                page_id,
                kind,
                hydrogen_policy,
                placement,
                phase,
            } = params;
            create_inner(
                document,
                RingCreate {
                    expected_revision,
                    page_id,
                    size: kind.size(),
                    hydrogen_policy,
                    placement,
                },
                floor,
                Pattern::Named(kind, phase),
            )
        }
        RingCreateRequest::Chair(params) => create_inner(
            document,
            RingCreate {
                expected_revision: params.expected_revision,
                page_id: params.page_id,
                size: 6,
                hydrogen_policy: params.hydrogen_policy,
                placement: params.placement,
            },
            floor,
            Pattern::Chair(params.kind),
        ),
    }
}

fn create_inner(
    document: &StoredDocument,
    params: RingCreate,
    floor: bond::Ids,
    pattern: Pattern,
) -> Result<(StoredDocument, bond::Ids), Error> {
    if !document.is_fully_mapped()
        || document
            .source_info()
            .is_some_and(|source| source.format != "cdxml")
    {
        return Err(Error::UnsupportedRing(
            "source-free or completely mapped native CDXML is required",
        ));
    }
    if !(3..=8).contains(&params.size) {
        return Err(Error::InvalidRing("ring size must be 3 through 8"));
    }
    let named = match pattern {
        Pattern::Named(kind, phase) => Some((kind, phase)),
        _ => None,
    };
    let OrderHydrogenPolicy::RectifyNeutralCarbon = params.hydrogen_policy;
    let page_id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
    let page = document
        .model()
        .pages()
        .iter()
        .find(|page| page.id() == page_id)
        .ok_or(bondline_model::Error::MissingPage(params.page_id))?;
    let mut weights = vec![1_u16; usize::from(params.size)];
    let (target, mut perimeter, hydrogen_updates, begin, end, side) =
        match params.placement {
            RingPlacement::Free {
                begin_x_pt,
                begin_y_pt,
                end_x_pt,
                end_y_pt,
                side,
            } => {
                if let Some((kind, phase)) = named {
                    weights = named_weights(kind, phase, SharedBudget::Free)?;
                }
                (
                    None,
                    Vec::new(),
                    Vec::new(),
                    Point2::new(begin_x_pt, begin_y_pt)?,
                    Point2::new(end_x_pt, end_y_pt)?,
                    side,
                )
            }
            RingPlacement::Atom {
                fragment_id,
                atom_id,
                end_x_pt,
                end_y_pt,
                side,
            } => {
                let fragment = fragment(page, fragment_id)?;
                let atom = atom(fragment, atom_id)?;
                let h = if let Some((kind, phase)) = named {
                    // A zero consumption proves old H once before the bounded phase checks.
                    let available = remaining_hydrogens(fragment, atom, 0)?;
                    weights = named_weights(kind, phase, SharedBudget::Atom(available))?;
                    subtract_new_weight(available, incident_weight(&weights, 0)?)?
                } else {
                    remaining_hydrogens(fragment, atom, 2)?
                };
                (
                    Some(fragment),
                    vec![atom.id],
                    vec![(atom.id, h)],
                    position(atom)?,
                    Point2::new(end_x_pt, end_y_pt)?,
                    side,
                )
            }
            RingPlacement::Bond {
                fragment_id,
                bond_id,
                side,
            } => {
                let fragment = fragment(page, fragment_id)?;
                let id =
                    BondId::new(bond_id).ok_or(Error::InvalidRing("bond ID must be positive"))?;
                let bond = fragment.bonds().iter().find(|bond| bond.id == id).ok_or(
                    Error::MissingBond {
                        fragment_id,
                        bond_id,
                    },
                )?;
                let supported_edge = bond.order == BondOrder::Single
                    || (named.is_some() && bond.order == BondOrder::Double);
                if !supported_edge {
                    return Err(Error::UnsupportedRing(if named.is_some() {
                        "a named shared ring edge must be Single or Double"
                    } else {
                        "a shared ring edge must be Single"
                    }));
                }
                let begin = atom(fragment, bond.begin.get())?;
                let end = atom(fragment, bond.end.get())?;
                let (begin_h, end_h) = if let Some((kind, phase)) = named {
                    let begin_available = remaining_hydrogens(fragment, begin, 0)?;
                    let end_available = remaining_hydrogens(fragment, end, 0)?;
                    let weight = bond::bond_weight(bond.order).ok_or(Error::UnsupportedRing(
                        "a named shared ring edge must have integer order",
                    ))?;
                    weights = named_weights(
                        kind,
                        phase,
                        SharedBudget::Bond {
                            begin: begin_available,
                            end: end_available,
                            weight,
                        },
                    )?;
                    let last = weights
                        .len()
                        .checked_sub(1)
                        .ok_or(Error::InvalidRing("empty ring pattern"))?;
                    (
                        subtract_new_weight(begin_available, weight_at(&weights, last)?)?,
                        subtract_new_weight(end_available, weight_at(&weights, 1)?)?,
                    )
                } else {
                    (
                        remaining_hydrogens(fragment, begin, 1)?,
                        remaining_hydrogens(fragment, end, 1)?,
                    )
                };
                (
                    Some(fragment),
                    vec![begin.id, end.id],
                    vec![(begin.id, begin_h), (end.id, end_h)],
                    position(begin)?,
                    position(end)?,
                    side,
                )
            }
        };
    let shared = perimeter.len();
    let points = match pattern {
        Pattern::Chair(kind) => chair::points(kind, begin, end, side)?,
        _ => polygon(params.size, begin, end, side)?,
    };
    for point in points.iter().skip(shared) {
        if page
            .fragments()
            .iter()
            .flat_map(|fragment| fragment.atoms())
            .any(|atom| atom.position == Some(*point))
        {
            return Err(Error::OccupiedEndpoint);
        }
    }
    let mut maximum = bond::maxima(document, floor);
    let fragment_id = if let Some(fragment) = target {
        fragment.id()
    } else {
        maximum.fragment = maximum
            .fragment
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        FragmentId::new(maximum.fragment).ok_or(Error::IdentityExhausted)?
    };
    let mut atoms = target.map_or_else(Vec::new, |fragment| fragment.atoms().to_vec());
    let mut bonds = target.map_or_else(Vec::new, |fragment| fragment.bonds().to_vec());
    for (id, h) in hydrogen_updates {
        atoms
            .iter_mut()
            .find(|atom| atom.id == id)
            .ok_or(Error::MissingAtom {
                fragment_id: fragment_id.get(),
                atom_id: id.get(),
            })?
            .explicit_hydrogens = Some(h);
    }
    let carbon = Element::try_from(6).map_err(bondline_model::Error::from)?;
    for (index, point) in points.iter().enumerate().skip(shared) {
        maximum.atom = maximum
            .atom
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let id = AtomId::new(maximum.atom).ok_or(Error::IdentityExhausted)?;
        let mut atom = Atom::new(id, carbon, Some(*point));
        atom.explicit_hydrogens =
            Some(4_u16.checked_sub(incident_weight(&weights, index)?).ok_or(
                Error::UnsupportedRing("ring bond orders exceed neutral carbon valence"),
            )?);
        if !matches!(pattern, Pattern::Regular) {
            atom.show_non_terminal_carbon_labels = Some(false);
        }
        atoms.push(atom);
        perimeter.push(id);
    }
    for (index, (begin, end)) in perimeter
        .iter()
        .copied()
        .zip(perimeter.iter().copied().cycle().skip(1))
        .take(perimeter.len())
        .enumerate()
    {
        if shared == 2 && index == 0 {
            continue;
        }
        maximum.bond = maximum
            .bond
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let (order, double_position) = match weight_at(&weights, index)? {
            1 => (
                BondOrder::Single,
                bondline_model::DoublePosition::Unspecified,
            ),
            2 => (
                BondOrder::Double,
                match side {
                    RingSide::Positive => bondline_model::DoublePosition::Right,
                    RingSide::Negative => bondline_model::DoublePosition::Left,
                },
            ),
            _ => {
                return Err(Error::InvalidRing(
                    "ring pattern needs Single or Double bonds",
                ));
            }
        };
        bonds.push(Bond {
            double_position,
            id: BondId::new(maximum.bond).ok_or(Error::IdentityExhausted)?,
            begin,
            end,
            order,
        });
    }
    let replacement =
        Fragment::new(fragment_id, atoms, bonds).map_err(bondline_model::Error::from)?;
    let mut candidate = document.clone();
    if target.is_some() {
        candidate
            .model_mut()
            .replace_fragment(page_id, replacement)?;
    } else {
        candidate
            .model_mut()
            .append_fragments(page_id, vec![replacement])?;
    }
    Ok((candidate, maximum))
}

#[cfg(test)]
#[path = "ring_tests.rs"]
mod tests;
