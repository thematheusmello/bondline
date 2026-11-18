//! Contained Label* properties with source provenance, not resolved run typography.
use crate::{Document, parse_integer};
use roxmltree::Node;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LabelDefaultsError {
    #[error("label defaults require an ordinary supported element from this document")]
    InvalidElement,
    #[error("label default {0} is placed on an unsupported containing owner")]
    UnsupportedOwner(&'static str),
    #[error("invalid label default {0} value")]
    InvalidProperty(&'static str),
}

/// One selected value and its original owned element/decoded attribute value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelDefault<'doc, 'input: 'doc, T> {
    value: T,
    owner: Node<'doc, 'input>,
    attribute_value: &'doc str,
}

impl<'doc, 'input: 'doc, T: Copy> LabelDefault<'doc, 'input, T> {
    pub fn value(self) -> T {
        self.value
    }
    pub fn owner(self) -> Node<'doc, 'input> {
        self.owner
    }
    pub fn attribute_value(self) -> &'doc str {
        self.attribute_value
    }
}

/// Each absent property remains unresolved; missing external style sheets are not guessed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LabelDefaults<'doc, 'input: 'doc> {
    font: Option<LabelDefault<'doc, 'input, i16>>,
    size_pt: Option<LabelDefault<'doc, 'input, f64>>,
    face: Option<LabelDefault<'doc, 'input, i16>>,
}

impl<'doc, 'input: 'doc> LabelDefaults<'doc, 'input> {
    pub fn font(self) -> Option<LabelDefault<'doc, 'input, i16>> {
        self.font
    }
    pub fn size_pt(self) -> Option<LabelDefault<'doc, 'input, f64>> {
        self.size_pt
    }
    pub fn face(self) -> Option<LabelDefault<'doc, 'input, i16>> {
        self.face
    }
}

impl<'input> Document<'input> {
    /// Resolve only the three Label* containment defaults. This does not fill `s` fields,
    /// look up a font, validate object layout, or grant display/edit authority.
    pub fn label_defaults<'doc>(
        &'doc self,
        element: Node<'doc, 'input>,
    ) -> Result<LabelDefaults<'doc, 'input>, LabelDefaultsError> {
        let root = self.xml().root_element();
        if !std::ptr::eq(element.document(), self.xml()) || !owner(element, root) {
            return Err(LabelDefaultsError::InvalidElement);
        }
        let mut defaults = LabelDefaults::default();
        for node in element.ancestors().filter(Node::is_element) {
            for name in ["LabelFont", "LabelSize", "LabelFace"] {
                let Some(attribute_value) = node.attributes().find_map(|attribute| {
                    (attribute.namespace().is_none() && attribute.name() == name)
                        .then_some(attribute.value())
                }) else {
                    continue;
                };
                if !owner(node, root) {
                    return Err(LabelDefaultsError::UnsupportedOwner(name));
                }
                if name == "LabelSize" {
                    let value = attribute_value
                        .trim_matches([' ', '\t', '\r', '\n'])
                        .parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite() && *value > 0.0)
                        .ok_or(LabelDefaultsError::InvalidProperty(name))?;
                    defaults.size_pt.get_or_insert(LabelDefault {
                        value,
                        owner: node,
                        attribute_value,
                    });
                } else {
                    let value = parse_integer(attribute_value)
                        .ok()
                        .and_then(|value| i16::try_from(value).ok())
                        .ok_or(LabelDefaultsError::InvalidProperty(name))?;
                    let field = if name == "LabelFont" {
                        &mut defaults.font
                    } else {
                        &mut defaults.face
                    };
                    field.get_or_insert(LabelDefault {
                        value,
                        owner: node,
                        attribute_value,
                    });
                }
            }
        }
        Ok(defaults)
    }
}

fn owner(node: Node<'_, '_>, root: Node<'_, '_>) -> bool {
    node.is_element()
        && node.tag_name().namespace().is_none_or(str::is_empty)
        && (node == root
            || matches!(
                node.tag_name().name(),
                "n" | "b" | "t" | "table" | "spectrum" | "constraint" | "tlcplate"
            ))
}
