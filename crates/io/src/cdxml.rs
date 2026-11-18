//! Source-backed CDXML projections; application document import remains separate.

mod atom_text;
mod boxed;
mod caption;
mod graph;
mod preserved;
mod write;
pub(crate) use boxed::{
    is_boxed, model_supported as boxed_model_supported, supports_delta as supports_boxed_delta,
    write_preserved as write_boxed_preserved,
};
pub(crate) use caption::{
    is_caption, supports_delta as supports_caption_delta,
    write_preserved as write_caption_preserved,
};
pub use graph::{ElementalGraph, MappingError};
pub(crate) use preserved::{is_preserved, supports_delta, write_preserved};
pub use write::write_document;

use bondline_model::Point2;
use std::collections::BTreeSet;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Syntax(#[from] bondline_cdxml::Error),
    #[error("node position must contain exactly two finite, bounded point coordinates")]
    InvalidPosition,
    #[error("only nodes directly inside CDXML/page/fragment are mapped")]
    UnsupportedNodeLayout,
    #[error("node geometry requires a nonblank source id")]
    InvalidNodeId,
    #[error("duplicate node source ids are ambiguous")]
    DuplicateNodeId,
    #[error("node does not exist in this geometry view")]
    MissingNode,
    #[error("node has no explicit position; coordinate generation is not implemented")]
    MissingPosition,
}

/// A format view. Source IDs stay strings; application identity and chemistry are not inferred.
#[derive(Debug)]
pub struct NodeGeometry {
    id: String,
    position: Option<Point2>,
}

impl NodeGeometry {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn position(&self) -> Option<Point2> {
        self.position
    }
}

#[derive(Debug)]
pub struct Geometry<'input> {
    source: bondline_cdxml::Document<'input>,
    nodes: Vec<NodeGeometry>,
}

impl<'input> Geometry<'input> {
    pub fn parse(input: &'input [u8]) -> Result<Self, Error> {
        let source = bondline_cdxml::Document::parse(input)?;
        let mut nodes = Vec::new();
        let mut ids = BTreeSet::new();
        for node in source.xml().descendants().filter(|node| {
            node.is_element()
                && node.tag_name().namespace().is_none()
                && node.tag_name().name() == "n"
        }) {
            let mut ancestors = node.ancestors().skip(1);
            for name in ["fragment", "page"] {
                if !ancestors.next().is_some_and(|parent| {
                    parent.is_element()
                        && parent.tag_name().namespace().is_none()
                        && parent.tag_name().name() == name
                }) {
                    return Err(Error::UnsupportedNodeLayout);
                }
            }
            if ancestors.next() != Some(source.xml().root_element()) {
                return Err(Error::UnsupportedNodeLayout);
            }
            let id = node
                .attribute("id")
                .filter(|id| !id.trim().is_empty())
                .ok_or(Error::InvalidNodeId)?;
            if !ids.insert(id) {
                return Err(Error::DuplicateNodeId);
            }
            nodes.push(NodeGeometry {
                id: id.to_owned(),
                position: node.attribute("p").map(parse_position).transpose()?,
            });
        }
        drop(ids);
        Ok(Self { source, nodes })
    }

    pub fn nodes(&self) -> &[NodeGeometry] {
        &self.nodes
    }

    pub fn as_bytes(&self) -> &'input [u8] {
        self.source.as_bytes()
    }

    /// Replace only a node's explicit p attribute, preserving other bytes and numeric no-ops.
    /// Dependent labels and bounding boxes are not recomputed; this is not an editor command.
    pub fn with_node_position(&self, id: &str, position: Point2) -> Result<Vec<u8>, Error> {
        let node = self
            .nodes
            .iter()
            .find(|node| node.id == id)
            .ok_or(Error::MissingNode)?;
        let previous = node.position.ok_or(Error::MissingPosition)?;
        if previous == position {
            return Ok(self.source.to_bytes());
        }
        let element = self
            .source
            .xml()
            .descendants()
            .find(|node| {
                node.is_element()
                    && node.tag_name().namespace().is_none()
                    && node.tag_name().name() == "n"
                    && node.attribute("id") == Some(id)
            })
            .ok_or(Error::MissingNode)?;
        let value = format!("{} {}", position.x_pt(), position.y_pt());
        Ok(self.source.replace_attribute(element, "p", &value)?)
    }
}

fn parse_position(value: &str) -> Result<Point2, Error> {
    let mut coordinates = value.split_ascii_whitespace();
    let x = coordinates
        .next()
        .ok_or(Error::InvalidPosition)?
        .parse()
        .map_err(|_| Error::InvalidPosition)?;
    let y = coordinates
        .next()
        .ok_or(Error::InvalidPosition)?
        .parse()
        .map_err(|_| Error::InvalidPosition)?;
    if coordinates.next().is_some() {
        return Err(Error::InvalidPosition);
    }
    Point2::new(x, y).map_err(|_| Error::InvalidPosition)
}

fn is_element(node: roxmltree::Node<'_, '_>, name: &str) -> bool {
    node.is_element() && node.tag_name().namespace().is_none() && node.tag_name().name() == name
}
