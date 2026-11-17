use super::*;
use crate::{ChainPlacement, ChainSide, OrderHydrogenPolicy};
use bondline_model::BondOrder;

struct Pose {
    priority: f64,
    heading: [f64; 2],
    side: ChainSide,
    mean: [f64; 2],
    tie: u32,
}

fn rotate([x, y]: [f64; 2], angle: f64) -> [f64; 2] {
    let (s, c) = angle.sin_cos();
    [c * x - s * y, s * x + c * y]
}

fn pose(priority: f64, heading: [f64; 2], side: ChainSide, angle: f64, tie: u32) -> Pose {
    let sign = if side == ChainSide::Positive {
        1.0
    } else {
        -1.0
    };
    let r = rotate(heading, sign * (180.0 - angle).to_radians());
    Pose {
        priority,
        heading,
        side,
        mean: [heading[0] + r[0], heading[1] + r[1]],
        tie,
    }
}

fn key(value: f64) -> f64 {
    (value * 1e12).round()
}

impl Engine {
    pub(super) fn chain_numeric_candidates(
        &self,
        page_id: u32,
        target: &ChainTarget,
        count: u8,
    ) -> Result<Vec<ChainCreate>, Error> {
        let preferences = self.chain_preferences()?;
        if !(2..=64).contains(&count) {
            return Err(Error::InvalidChain(
                "carbon count must be 2 through 64, including a shared anchor",
            ));
        }
        let begin = self.chain_target_position(page_id, target)?;
        let mut neighbors = Vec::new();
        if let ChainTarget::Atom {
            fragment_id,
            atom_id,
        } = *target
        {
            let page = crate::chain::page(self.stored_document()?, page_id)?;
            let anchor = crate::chain::attachment(page, fragment_id, atom_id)?;
            for bond in anchor.fragment.bonds() {
                let other = if bond.begin.get() == atom_id {
                    Some(bond.end)
                } else if bond.end.get() == atom_id {
                    Some(bond.begin)
                } else {
                    None
                };
                if let Some(other) = other {
                    let position = anchor
                        .fragment
                        .atoms()
                        .iter()
                        .find(|a| a.id == other)
                        .and_then(|a| a.position)
                        .ok_or(Error::UnsupportedChain(
                            "automatic chain poses need positioned neighbors",
                        ))?;
                    let dx = position.x_pt() - begin.x_pt();
                    let dy = position.y_pt() - begin.y_pt();
                    let norm = dx.hypot(dy);
                    if !norm.is_finite() || norm == 0.0 {
                        return Err(Error::UnsupportedChain(
                            "automatic chain poses need distinct positioned neighbors",
                        ));
                    }
                    neighbors.push((other.get(), [dx / norm, dy / norm], bond.order));
                }
            }
        }
        let angle = preferences.angle_degrees;
        let mut poses = Vec::new();
        if neighbors.is_empty() {
            let (a, b) = (angle * 0.5).to_radians().sin_cos();
            poses.push(pose(0.0, [a, -b], ChainSide::Positive, angle, 0));
        } else if let [(id, old, order)] = neighbors.as_slice() {
            let outward = [-old[0], -old[1]];
            let headings = if *order == BondOrder::Triple {
                vec![outward]
            } else {
                vec![
                    rotate(*old, angle.to_radians()),
                    rotate(*old, -angle.to_radians()),
                ]
            };
            for heading in headings {
                for side in [ChainSide::Positive, ChainSide::Negative] {
                    let mut item = pose(0.0, heading, side, angle, *id);
                    item.priority = item.mean[0] * outward[0] + item.mean[1] * outward[1];
                    poses.push(item);
                }
            }
        } else {
            let mut axes: Vec<_> = neighbors
                .iter()
                .map(|(id, [x, y], _)| (*id, y.atan2(*x).rem_euclid(std::f64::consts::TAU)))
                .collect();
            axes.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
            for (index, (id, start)) in axes.iter().enumerate() {
                let next = axes
                    .get(index + 1)
                    .or_else(|| axes.first())
                    .ok_or(Error::InvalidChain("automatic pose has no neighbor"))?;
                let end = if index + 1 == axes.len() {
                    next.1 + std::f64::consts::TAU
                } else {
                    next.1
                };
                let gap = end - start;
                let (s, c) = (start + gap * 0.5).sin_cos();
                for side in [ChainSide::Positive, ChainSide::Negative] {
                    poses.push(pose(gap, [c, s], side, angle, (*id).min(next.0)));
                }
            }
        }
        poses.sort_by(|a, b| {
            key(b.priority)
                .total_cmp(&key(a.priority))
                .then(key(a.mean[1]).total_cmp(&key(b.mean[1])))
                .then(key(b.mean[0]).total_cmp(&key(a.mean[0])))
                .then(a.tie.cmp(&b.tie))
                .then((a.side == ChainSide::Negative).cmp(&(b.side == ChainSide::Negative)))
        });
        Ok(poses
            .into_iter()
            .map(|pose| {
                let end_x_pt = begin.x_pt() + preferences.bond_length_pt * pose.heading[0];
                let end_y_pt = begin.y_pt() + preferences.bond_length_pt * pose.heading[1];
                let placement = match *target {
                    ChainTarget::Free { x_pt, y_pt } => ChainPlacement::Free {
                        begin_x_pt: x_pt,
                        begin_y_pt: y_pt,
                        end_x_pt,
                        end_y_pt,
                        side: pose.side,
                    },
                    ChainTarget::Atom {
                        fragment_id,
                        atom_id,
                    } => ChainPlacement::Atom {
                        fragment_id,
                        atom_id,
                        end_x_pt,
                        end_y_pt,
                        side: pose.side,
                    },
                };
                ChainCreate {
                    expected_revision: self.revision,
                    page_id,
                    count,
                    angle_degrees: angle,
                    hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
                    placement,
                }
            })
            .collect())
    }
}
