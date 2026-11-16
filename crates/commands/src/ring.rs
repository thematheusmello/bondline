//! Bounded native ring construction; existing source and shared geometry stay authoritative.
use crate::{Error, OrderHydrogenPolicy, bond};
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Page, PageId, Point2,
    Radical,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    if atom.element.get() != 6
        || atom.charge != 0
        || atom.isotope.is_some()
        || atom.radical != Radical::None
    {
        return Err(unsupported());
    }
    let used = fragment
        .bonds()
        .iter()
        .filter(|bond| bond.begin == atom.id || bond.end == atom.id)
        .try_fold(0_u16, |sum, bond| {
            sum.checked_add(bond::bond_weight(bond.order)?)
                .filter(|sum| *sum <= 4)
        })
        .ok_or_else(unsupported)?;
    let available = 4 - used;
    if atom.explicit_hydrogens != Some(available) {
        return Err(unsupported());
    }
    available.checked_sub(consumed).ok_or_else(unsupported)
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

pub(super) fn create(
    document: &StoredDocument,
    params: RingCreate,
    floor: bond::Ids,
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
    let OrderHydrogenPolicy::RectifyNeutralCarbon = params.hydrogen_policy;
    let page_id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
    let page = document
        .model()
        .pages()
        .iter()
        .find(|page| page.id() == page_id)
        .ok_or(bondline_model::Error::MissingPage(params.page_id))?;
    let (target, mut perimeter, hydrogen_updates, begin, end, side) =
        match params.placement {
            RingPlacement::Free {
                begin_x_pt,
                begin_y_pt,
                end_x_pt,
                end_y_pt,
                side,
            } => (
                None,
                Vec::new(),
                Vec::new(),
                Point2::new(begin_x_pt, begin_y_pt)?,
                Point2::new(end_x_pt, end_y_pt)?,
                side,
            ),
            RingPlacement::Atom {
                fragment_id,
                atom_id,
                end_x_pt,
                end_y_pt,
                side,
            } => {
                let fragment = fragment(page, fragment_id)?;
                let atom = atom(fragment, atom_id)?;
                let h = remaining_hydrogens(fragment, atom, 2)?;
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
                if bond.order != BondOrder::Single {
                    return Err(Error::UnsupportedRing("a shared ring edge must be Single"));
                }
                let begin = atom(fragment, bond.begin.get())?;
                let end = atom(fragment, bond.end.get())?;
                let begin_h = remaining_hydrogens(fragment, begin, 1)?;
                let end_h = remaining_hydrogens(fragment, end, 1)?;
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
    let points = polygon(params.size, begin, end, side)?;
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
    for point in points.iter().skip(shared) {
        maximum.atom = maximum
            .atom
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let id = AtomId::new(maximum.atom).ok_or(Error::IdentityExhausted)?;
        let mut atom = Atom::new(id, carbon, Some(*point));
        atom.explicit_hydrogens = Some(2);
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
        bonds.push(Bond {
            id: BondId::new(maximum.bond).ok_or(Error::IdentityExhausted)?,
            begin,
            end,
            order: BondOrder::Single,
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
