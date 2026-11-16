//! Literal ordered paths for one-commit turning gestures; native constraints stay separate.
use crate::{Error, OrderHydrogenPolicy, bond};
use bondline_io::StoredDocument;
use bondline_model::Point2;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChainPathPoint {
    #[schemars(range(min = -100000, max = 100000))]
    pub x_pt: f64,
    #[schemars(range(min = -100000, max = 100000))]
    pub y_pt: f64,
}

impl<'de> Deserialize<'de> for ChainPathPoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Require a map before reusing Point2's closed, bounded coordinate decoder.
        let object = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;
        let point: Point2 =
            serde_json::from_value(serde_json::Value::Object(object)).map_err(D::Error::custom)?;
        Ok(Self {
            x_pt: point.x_pt(),
            y_pt: point.y_pt(),
        })
    }
}

impl ChainPathPoint {
    fn position(self) -> Result<Point2, Error> {
        Ok(Point2::new(self.x_pt, self.y_pt)?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum ChainPathPlacement {
    Free {
        #[schemars(length(min = 2, max = 64))]
        points: Vec<ChainPathPoint>,
    },
    Atom {
        #[schemars(range(min = 1))]
        fragment_id: u32,
        #[schemars(range(min = 1))]
        atom_id: u32,
        #[schemars(length(min = 1, max = 63))]
        new_points: Vec<ChainPathPoint>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChainCreatePath {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[serde(deserialize_with = "super::hydrogen_policy")]
    #[schemars(with = "OrderHydrogenPolicy")]
    pub hydrogen_policy: OrderHydrogenPolicy,
    pub placement: ChainPathPlacement,
}

pub(crate) fn create(
    document: &StoredDocument,
    params: &ChainCreatePath,
    floor: bond::Ids,
) -> Result<(StoredDocument, bond::Ids), Error> {
    super::require_source(document)?;
    let valid_count = match &params.placement {
        ChainPathPlacement::Free { points } => (2..=64).contains(&points.len()),
        ChainPathPlacement::Atom { new_points, .. } => (1..=63).contains(&new_points.len()),
    };
    if !valid_count {
        return Err(Error::InvalidChain(
            "free paths need 2..64 points; attachments need 1..63 new points",
        ));
    }
    let OrderHydrogenPolicy::RectifyNeutralCarbon = params.hydrogen_policy;
    let page = super::page(document, params.page_id)?;
    let (anchor, points) = match &params.placement {
        ChainPathPlacement::Free { points } => (
            None,
            points
                .iter()
                .copied()
                .map(ChainPathPoint::position)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        ChainPathPlacement::Atom {
            fragment_id,
            atom_id,
            new_points,
        } => {
            let anchor = super::attachment(page, *fragment_id, *atom_id)?;
            let mut points = Vec::with_capacity(new_points.len() + 1);
            points.push(anchor.position);
            for point in new_points {
                points.push(point.position()?);
            }
            (Some(anchor), points)
        }
    };
    super::append(document, page, anchor, points, floor)
}
