//! Published CDXML resource tables, independent of font availability or drawing policy.
use crate::{Document, parse_integer};
use roxmltree::Node;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ResourceError {
    #[error("duplicate document {0} resource table")]
    DuplicateTable(&'static str),
    #[error("{0} must be an unnamespaced direct child of CDXML")]
    UnsupportedTable(&'static str),
    #[error("{0} resource table requires at least one entry")]
    EmptyTable(&'static str),
    #[error("unsupported child content in resource element {0}")]
    UnexpectedContent(&'static str),
    #[error("font id must be an exact unsigned 16-bit integer")]
    InvalidFontId,
    #[error("font requires a name attribute")]
    MissingFontName,
    #[error("duplicate font id {0}")]
    DuplicateFontId(u16),
    #[error("color component {0} must be present, finite and in 0..=1")]
    InvalidColorComponent(&'static str),
    #[error("color table exceeds the unsigned 16-bit reference domain")]
    ColorIndexLimit,
    #[error("font id {0} is not defined")]
    MissingFont(u16),
    #[error("color index {0} is not defined")]
    MissingColor(u16),
}

/// A logical font reference, not an installed or bundled font. Charset remains source text.
#[derive(Clone, Copy, Debug)]
pub struct Font<'doc, 'input> {
    id: u16,
    name: &'doc str,
    charset: Option<&'doc str>,
    element: Node<'doc, 'input>,
}

impl<'doc, 'input> Font<'doc, 'input> {
    pub fn id(&self) -> u16 {
        self.id
    }
    pub fn name(&self) -> &'doc str {
        self.name
    }
    pub fn charset(&self) -> Option<&'doc str> {
        self.charset
    }
    pub fn element(&self) -> Node<'doc, 'input> {
        self.element
    }
}

/// A source-ordered palette entry. Its index is implicit; channels remain unquantized.
#[derive(Clone, Copy, Debug)]
pub struct Color<'doc, 'input> {
    index: u16,
    rgb: [f64; 3],
    element: Node<'doc, 'input>,
}

impl<'doc, 'input> Color<'doc, 'input> {
    pub fn index(&self) -> u16 {
        self.index
    }
    pub fn rgb(&self) -> [f64; 3] {
        self.rgb
    }
    pub fn element(&self) -> Node<'doc, 'input> {
        self.element
    }
}

/// A checked read view. Success does not certify all source attributes, font rendering,
/// style inheritance, or model editability. Original nodes retain unprojected data.
#[derive(Debug)]
pub struct Resources<'doc, 'input> {
    font_table: Option<Node<'doc, 'input>>,
    color_table: Option<Node<'doc, 'input>>,
    fonts: BTreeMap<u16, Font<'doc, 'input>>,
    colors: Vec<Color<'doc, 'input>>,
}

impl<'doc, 'input> Resources<'doc, 'input> {
    pub fn font_table(&self) -> Option<Node<'doc, 'input>> {
        self.font_table
    }
    pub fn color_table(&self) -> Option<Node<'doc, 'input>> {
        self.color_table
    }

    /// Logical fonts in ascending ID order, independently of their source order.
    pub fn fonts(&self) -> impl ExactSizeIterator<Item = &Font<'doc, 'input>> {
        self.fonts.values()
    }
    pub fn colors(&self) -> &[Color<'doc, 'input>] {
        &self.colors
    }
    pub fn font(&self, id: u16) -> Result<&Font<'doc, 'input>, ResourceError> {
        self.fonts.get(&id).ok_or(ResourceError::MissingFont(id))
    }
    pub fn color(&self, index: u16) -> Result<[f64; 3], ResourceError> {
        match index {
            0 => Ok([0.0; 3]),
            1 => Ok([1.0; 3]),
            2 if self.color_table.is_none() => Ok([1.0; 3]),
            3 if self.color_table.is_none() => Ok([0.0; 3]),
            _ => self
                .colors
                .get(usize::from(index).saturating_sub(2))
                .map(Color::rgb)
                .ok_or(ResourceError::MissingColor(index)),
        }
    }
}

impl<'input> Document<'input> {
    /// Decode document-level logical font and colour resources without changing parser
    /// admission or source bytes. Unsupported table scopes/namespaces fail conservatively.
    pub fn resources(&self) -> Result<Resources<'_, 'input>, ResourceError> {
        let root = self.xml().root_element();
        let mut resources = Resources {
            font_table: None,
            color_table: None,
            fonts: BTreeMap::new(),
            colors: Vec::new(),
        };
        for node in root.descendants().filter(Node::is_element) {
            let (name, table) = match node.tag_name().name() {
                "fonttable" => ("fonttable", &mut resources.font_table),
                "colortable" => ("colortable", &mut resources.color_table),
                _ => continue,
            };
            if node
                .tag_name()
                .namespace()
                .is_some_and(|namespace| !namespace.is_empty())
                || node.parent() != Some(root)
            {
                return Err(ResourceError::UnsupportedTable(name));
            }
            if table.replace(node).is_some() {
                return Err(ResourceError::DuplicateTable(name));
            }
        }
        if let Some(table) = resources.font_table {
            contents(table, "fonttable", Some("font"))?;
            for node in table.children().filter(Node::is_element) {
                contents(node, "font", None)?;
                let id = attribute(node, "id")
                    .and_then(|value| parse_integer(value).ok())
                    .and_then(|value| u16::try_from(value).ok())
                    .ok_or(ResourceError::InvalidFontId)?;
                let font = Font {
                    id,
                    name: attribute(node, "name").ok_or(ResourceError::MissingFontName)?,
                    charset: attribute(node, "charset"),
                    element: node,
                };
                if resources.fonts.insert(id, font).is_some() {
                    return Err(ResourceError::DuplicateFontId(id));
                }
            }
            if resources.fonts.is_empty() {
                return Err(ResourceError::EmptyTable("fonttable"));
            }
        }
        if let Some(table) = resources.color_table {
            contents(table, "colortable", Some("color"))?;
            for node in table.children().filter(Node::is_element) {
                contents(node, "color", None)?;
                let index = resources
                    .colors
                    .len()
                    .checked_add(2)
                    .and_then(|index| u16::try_from(index).ok())
                    .ok_or(ResourceError::ColorIndexLimit)?;
                let mut rgb = [0.0; 3];
                for (value, name) in rgb.iter_mut().zip(["r", "g", "b"]) {
                    *value = attribute(node, name)
                        .and_then(|value| {
                            value
                                .trim_matches([' ', '\t', '\r', '\n'])
                                .parse::<f64>()
                                .ok()
                        })
                        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
                        .ok_or(ResourceError::InvalidColorComponent(name))?;
                }
                resources.colors.push(Color {
                    index,
                    rgb,
                    element: node,
                });
            }
            if resources.colors.is_empty() {
                return Err(ResourceError::EmptyTable("colortable"));
            }
        }
        Ok(resources)
    }
}

fn attribute<'doc>(element: Node<'doc, '_>, name: &str) -> Option<&'doc str> {
    // roxmltree's string-name lookup also matches namespaced attributes by local name.
    element
        .attributes()
        .find(|attr| attr.namespace().is_none() && attr.name() == name)
        .map(|attr| attr.value())
}

fn contents(
    element: Node<'_, '_>,
    owner: &'static str,
    child_name: Option<&str>,
) -> Result<(), ResourceError> {
    for child in element.children() {
        if child.is_element() {
            if child
                .tag_name()
                .namespace()
                .is_some_and(|namespace| !namespace.is_empty())
                || child_name != Some(child.tag_name().name())
            {
                return Err(ResourceError::UnexpectedContent(owner));
            }
        } else if child.is_text()
            && child
                .text()
                .is_some_and(|text| !text.trim_matches([' ', '\t', '\r', '\n']).is_empty())
        {
            return Err(ResourceError::UnexpectedContent(owner));
        }
    }
    Ok(())
}
