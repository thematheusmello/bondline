//! Ordered decoded style runs, without layout, resource resolution or model authority.
use crate::{Document, parse_integer};
use roxmltree::Node;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TextError {
    #[error("styled text requires an ordinary t element from this parsed document")]
    InvalidElement,
    #[error("styled text requires at least one direct s element")]
    MissingRuns,
    #[error("unsupported child content in styled text element {0}")]
    UnexpectedContent(&'static str),
    #[error("invalid styled text {0} property")]
    InvalidProperty(&'static str),
}

/// A borrowed run; absent selectors remain unresolved, and unknown attributes stay in its node.
#[derive(Clone, Copy, Debug)]
pub struct StyleRun<'doc, 'input: 'doc> {
    element: Node<'doc, 'input>,
    text: &'doc str,
    font: Option<u16>,
    size_pt: Option<f64>,
    face: Option<u16>,
    color: Option<u16>,
}

impl<'doc, 'input: 'doc> StyleRun<'doc, 'input> {
    pub fn element(&self) -> Node<'doc, 'input> {
        self.element
    }
    pub fn text(&self) -> &str {
        self.text
    }
    pub fn font(&self) -> Option<u16> {
        self.font
    }
    pub fn size_pt(&self) -> Option<f64> {
        self.size_pt
    }
    pub fn face(&self) -> Option<u16> {
        self.face
    }
    pub fn color(&self) -> Option<u16> {
        self.color
    }
}

/// A checked ordered view, not a certificate for all source attributes or rendered appearance.
#[derive(Debug)]
pub struct TextObject<'doc, 'input: 'doc> {
    element: Node<'doc, 'input>,
    runs: Vec<StyleRun<'doc, 'input>>,
}

impl<'doc, 'input: 'doc> TextObject<'doc, 'input> {
    pub fn element(&self) -> Node<'doc, 'input> {
        self.element
    }
    pub fn runs(&self) -> &[StyleRun<'doc, 'input>] {
        &self.runs
    }
}

impl<'input> Document<'input> {
    /// Read one owned ordinary Text object without changing syntax admission or source bytes.
    /// XML whitespace may separate direct style runs. Other mixed content is unsupported.
    /// Resource lookup, inherited values, font availability and application budgets are separate.
    pub fn styled_text<'doc>(
        &'doc self,
        element: Node<'doc, 'input>,
    ) -> Result<TextObject<'doc, 'input>, TextError> {
        if !std::ptr::eq(element.document(), self.xml()) || !ordinary(element, "t") {
            return Err(TextError::InvalidElement);
        }
        let mut runs = Vec::new();
        for child in element.children() {
            if ordinary(child, "s") {
                runs.push(read_run(child)?);
            } else if !child.is_text()
                || child
                    .text()
                    .is_some_and(|text| !text.bytes().all(xml_space))
            {
                return Err(TextError::UnexpectedContent("t"));
            }
        }
        if runs.is_empty() {
            return Err(TextError::MissingRuns);
        }
        Ok(TextObject { element, runs })
    }
}

fn ordinary(element: Node<'_, '_>, name: &str) -> bool {
    element.is_element()
        && element.tag_name().name() == name
        && element.tag_name().namespace().is_none_or(str::is_empty)
}

fn xml_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

fn attribute<'doc>(element: Node<'doc, '_>, name: &str) -> Option<&'doc str> {
    element
        .attributes()
        .find(|attribute| attribute.namespace().is_none() && attribute.name() == name)
        .map(|attribute| attribute.value())
}

fn integer(element: Node<'_, '_>, name: &'static str) -> Result<Option<u16>, TextError> {
    attribute(element, name)
        .map(|value| {
            parse_integer(value)
                .ok()
                .and_then(|value| u16::try_from(value).ok())
                .ok_or(TextError::InvalidProperty(name))
        })
        .transpose()
}

fn read_run<'doc, 'input: 'doc>(
    element: Node<'doc, 'input>,
) -> Result<StyleRun<'doc, 'input>, TextError> {
    if element.children().any(|child| !child.is_text()) {
        return Err(TextError::UnexpectedContent("s"));
    }
    let size_pt = attribute(element, "size")
        .map(|value| {
            value
                .trim_matches([' ', '\t', '\r', '\n'])
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
                .ok_or(TextError::InvalidProperty("size"))
        })
        .transpose()?;
    Ok(StyleRun {
        element,
        // The pinned XML parser merges adjacent character data, references and CDATA.
        text: element.text().unwrap_or(""),
        font: integer(element, "font")?,
        size_pt,
        face: integer(element, "face")?,
        color: integer(element, "color")?,
    })
}
