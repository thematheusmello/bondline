//! Explicit native carbon chains; no inferred pose, source conversion or UI defaults.
use crate::{Error, OrderHydrogenPolicy, bond};
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Page, PageId, Point2,
};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::value::StringDeserializer};

mod path;
pub(super) use path::create as create_path;
pub use path::{ChainCreatePath, ChainPathPlacement, ChainPathPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChainSide {
    Positive,
    Negative,
}

impl<'de> Deserialize<'de> for ChainSide {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match String::deserialize(deserializer)?.as_str() {
            "positive" => Ok(Self::Positive),
            "negative" => Ok(Self::Negative),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["positive", "negative"],
            )),
        }
    }
}

fn hydrogen_policy<'de, D: Deserializer<'de>>(d: D) -> Result<OrderHydrogenPolicy, D::Error> {
    OrderHydrogenPolicy::deserialize(StringDeserializer::<D::Error>::new(String::deserialize(d)?))
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum ChainPlacement {
    Free {
        #[schemars(range(min = -100000, max = 100000))]
        begin_x_pt: f64,
        #[schemars(range(min = -100000, max = 100000))]
        begin_y_pt: f64,
        #[schemars(range(min = -100000, max = 100000))]
        end_x_pt: f64,
        #[schemars(range(min = -100000, max = 100000))]
        end_y_pt: f64,
        side: ChainSide,
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
        side: ChainSide,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChainCreate {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 2, max = 64))]
    pub count: u8,
    #[schemars(range(min = 1, max = 179))]
    pub angle_degrees: f64,
    #[serde(deserialize_with = "hydrogen_policy")]
    #[schemars(with = "OrderHydrogenPolicy")]
    pub hydrogen_policy: OrderHydrogenPolicy,
    pub placement: ChainPlacement,
}

fn points(
    count: u8,
    angle_degrees: f64,
    begin: Point2,
    end: Point2,
    side: ChainSide,
) -> Result<Vec<Point2>, Error> {
    let precision =
        || Error::InvalidChain("chain geometry loses fidelity at this coordinate precision");
    let e = (end.x_pt() - begin.x_pt(), end.y_pt() - begin.y_pt());
    let length = e.0.hypot(e.1);
    if !length.is_finite() || length == 0.0 {
        return Err(precision());
    }
    let sign = match side {
        ChainSide::Positive => 1.0,
        ChainSide::Negative => -1.0,
    };
    let (sin, cos) = ((180.0 - angle_degrees).to_radians() * sign).sin_cos();
    let r = (cos * e.0 - sin * e.1, sin * e.0 + cos * e.1);
    let mut points = vec![begin, end];
    for i in 2..count {
        let a = f64::from(i.div_ceil(2));
        let b = f64::from(i / 2);
        let point = Point2::new(
            begin.x_pt() + a * e.0 + b * r.0,
            begin.y_pt() + a * e.1 + b * r.1,
        )?;
        if points.contains(&point) {
            return Err(precision());
        }
        points.push(point);
    }
    for (a, b) in points.iter().zip(points.iter().skip(1)) {
        let norm = ((b.x_pt() - a.x_pt()) / length).hypot((b.y_pt() - a.y_pt()) / length);
        if !norm.is_finite() || (norm - 1.0).abs() > 1e-10 {
            return Err(precision());
        }
    }
    for (i, window) in points.windows(3).enumerate() {
        let [previous, point, next] = window else {
            return Err(precision());
        };
        let u = (
            (previous.x_pt() - point.x_pt()) / length,
            (previous.y_pt() - point.y_pt()) / length,
        );
        let v = (
            (next.x_pt() - point.x_pt()) / length,
            (next.y_pt() - point.y_pt()) / length,
        );
        let cross = u.0 * v.1 - u.1 * v.0;
        let angle = cross.abs().atan2(u.0 * v.0 + u.1 * v.1);
        let turn = if i % 2 == 0 { -sign } else { sign };
        if !angle.is_finite()
            || (angle - angle_degrees.to_radians()).abs() > 1e-10
            || cross * turn <= 0.0
        {
            return Err(precision());
        }
    }
    Ok(points)
}

pub(super) fn create(
    document: &StoredDocument,
    params: ChainCreate,
    floor: bond::Ids,
) -> Result<(StoredDocument, bond::Ids), Error> {
    require_source(document)?;
    if !(2..=64).contains(&params.count) {
        return Err(Error::InvalidChain(
            "carbon count must be 2 through 64, including a shared anchor",
        ));
    }
    if !(1.0..=179.0).contains(&params.angle_degrees) {
        return Err(Error::InvalidChain(
            "interior angle must be finite and 1 through 179 degrees",
        ));
    }
    let OrderHydrogenPolicy::RectifyNeutralCarbon = params.hydrogen_policy;
    let page = page(document, params.page_id)?;
    let (anchor, begin, end, side) = match params.placement {
        ChainPlacement::Free {
            begin_x_pt,
            begin_y_pt,
            end_x_pt,
            end_y_pt,
            side,
        } => (
            None,
            Point2::new(begin_x_pt, begin_y_pt)?,
            Point2::new(end_x_pt, end_y_pt)?,
            side,
        ),
        ChainPlacement::Atom {
            fragment_id,
            atom_id,
            end_x_pt,
            end_y_pt,
            side,
        } => {
            let anchor = attachment(page, fragment_id, atom_id)?;
            let begin = anchor.position;
            (Some(anchor), begin, Point2::new(end_x_pt, end_y_pt)?, side)
        }
    };
    let points = points(params.count, params.angle_degrees, begin, end, side)?;
    append(document, page, anchor, points, floor)
}

fn require_source(document: &StoredDocument) -> Result<(), Error> {
    if !document.is_fully_mapped()
        || document
            .source_info()
            .is_some_and(|source| source.format != "cdxml")
    {
        return Err(Error::UnsupportedChain(
            "source-free or completely mapped native CDXML is required",
        ));
    }
    Ok(())
}

fn page(document: &StoredDocument, page_id: u32) -> Result<&Page, Error> {
    let id = PageId::new(page_id).ok_or(Error::InvalidPageId)?;
    document
        .model()
        .pages()
        .iter()
        .find(|page| page.id() == id)
        .ok_or_else(|| bondline_model::Error::MissingPage(page_id).into())
}

struct Anchor<'a> {
    fragment: &'a Fragment,
    id: AtomId,
    hydrogens: u16,
    position: Point2,
}

fn attachment(page: &Page, fragment_id: u32, atom_id: u32) -> Result<Anchor<'_>, Error> {
    let id = FragmentId::new(fragment_id).ok_or(Error::InvalidFragmentId)?;
    let fragment = page
        .fragments()
        .iter()
        .find(|fragment| fragment.id() == id)
        .ok_or(bondline_model::Error::MissingFragment(fragment_id))?;
    let id = AtomId::new(atom_id).ok_or(Error::InvalidChain("atom ID must be positive"))?;
    let atom = fragment
        .atoms()
        .iter()
        .find(|atom| atom.id == id)
        .ok_or(Error::MissingAtom {
            fragment_id,
            atom_id,
        })?;
    let hydrogens = bond::remaining_carbon_hydrogens(fragment, atom, 1).ok_or(Error::UnsupportedChain(
        "shared atoms need neutral natural carbon, supported incident orders and consistent sufficient literal H",
    ))?;
    let position = atom.position.ok_or(Error::UnsupportedChain(
        "a shared atom has no authored position",
    ))?;
    Ok(Anchor {
        fragment,
        id,
        hydrogens,
        position,
    })
}

fn append(
    document: &StoredDocument,
    page: &Page,
    anchor: Option<Anchor<'_>>,
    points: Vec<Point2>,
    floor: bond::Ids,
) -> Result<(StoredDocument, bond::Ids), Error> {
    if !(2..=64).contains(&points.len()) {
        return Err(Error::InvalidChain(
            "a resolved chain needs 2 through 64 vertices",
        ));
    }
    for (i, point) in points.iter().enumerate() {
        if points.iter().take(i).any(|previous| previous == point) {
            return Err(Error::InvalidChain("chain vertices must be distinct"));
        }
    }
    for (a, b) in points.iter().zip(points.iter().skip(1)) {
        let length = (b.x_pt() - a.x_pt()).hypot(b.y_pt() - a.y_pt());
        if !length.is_finite() || length == 0.0 {
            return Err(Error::InvalidChain(
                "chain edges need finite nonzero length",
            ));
        }
    }
    let page_id = page.id();
    let shared = usize::from(anchor.is_some());
    let target = anchor.as_ref().map(|anchor| anchor.fragment);
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
    let mut previous = if let Some(anchor) = anchor {
        let id = anchor.id;
        atoms
            .iter_mut()
            .find(|atom| atom.id == id)
            .ok_or(Error::MissingAtom {
                fragment_id: fragment_id.get(),
                atom_id: id.get(),
            })?
            .explicit_hydrogens = Some(anchor.hydrogens);
        Some(id)
    } else {
        None
    };
    let carbon = Element::try_from(6).map_err(bondline_model::Error::from)?;
    for (i, point) in points.iter().enumerate().skip(shared) {
        maximum.atom = maximum
            .atom
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let id = AtomId::new(maximum.atom).ok_or(Error::IdentityExhausted)?;
        let mut atom = Atom::new(id, carbon, Some(*point));
        atom.explicit_hydrogens = Some(if i == 0 || i + 1 == points.len() {
            3
        } else {
            2
        });
        atom.show_terminal_carbon_labels = Some(false);
        atom.show_non_terminal_carbon_labels = Some(false);
        atoms.push(atom);
        if let Some(begin) = previous {
            maximum.bond = maximum
                .bond
                .checked_add(1)
                .ok_or(Error::IdentityExhausted)?;
            bonds.push(Bond {
                id: BondId::new(maximum.bond).ok_or(Error::IdentityExhausted)?,
                begin,
                end: id,
                order: BondOrder::Single,
                double_position: bondline_model::DoublePosition::Unspecified,
            });
        }
        previous = Some(id);
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

impl crate::Engine {
    /// Read-only candidate from the same fixed-chain builder used at commit.
    pub fn preview_chain_create(&self, params: &ChainCreate) -> Result<StoredDocument, Error> {
        if self.close_state() == crate::CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        create(self.stored_document()?, params.clone(), self.identities)
            .map(|(document, _)| document)
    }

    /// Read-only literal path; no identity reservation or native gesture authority.
    pub fn preview_chain_path(&self, params: &ChainCreatePath) -> Result<StoredDocument, Error> {
        if self.close_state() == crate::CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        create_path(self.stored_document()?, params, self.identities).map(|(document, _)| document)
    }
}
