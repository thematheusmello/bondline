use crate::{
    Document, Error, MAX_ATTRIBUTES, MAX_ATTRIBUTES_PER_ELEMENT, MAX_DEPTH, MAX_INPUT_BYTES,
    MAX_NODES,
    edit::{escape, valid_character},
};
use std::collections::BTreeSet;

/// Bounded new CDXML syntax, without model or filesystem dependencies.
/// Names are unnamespaced ASCII; containment/chemical semantics belong to format adapters.
/// Existing documents must use source-preserving edits, never this regenerating writer.
pub struct Writer {
    output: Vec<u8>,
    open: Vec<String>,
    closing_bytes: usize,
    attributes: usize,
    nodes: usize,
}

impl Writer {
    pub fn new(attributes: &[(&str, &str)]) -> Result<Self, Error> {
        let mut writer = Self {
            output: b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_vec(),
            open: Vec::new(),
            closing_bytes: 0,
            attributes: 0,
            nodes: 1,
        };
        writer.start_element("CDXML", attributes)?;
        Ok(writer)
    }

    pub fn start_element(&mut self, name: &str, attributes: &[(&str, &str)]) -> Result<(), Error> {
        self.element(name, attributes, false)
    }

    pub fn empty_element(&mut self, name: &str, attributes: &[(&str, &str)]) -> Result<(), Error> {
        self.element(name, attributes, true)
    }

    pub fn end_element(&mut self) -> Result<(), Error> {
        if self.open.len() <= 1 {
            return Err(Error::UnbalancedElements);
        }
        self.close_top()
    }

    pub fn finish(mut self) -> Result<Vec<u8>, Error> {
        if self.open.len() != 1 {
            return Err(Error::UnbalancedElements);
        }
        self.close_top()?;
        Document::parse(&self.output)?;
        Ok(self.output)
    }

    fn close_top(&mut self) -> Result<(), Error> {
        let name = self.open.pop().ok_or(Error::UnbalancedElements)?;
        self.output.extend_from_slice(b"</");
        self.output.extend_from_slice(name.as_bytes());
        self.output.push(b'>');
        self.closing_bytes -= name.len() + 3;
        Ok(())
    }

    fn element(
        &mut self,
        name: &str,
        attributes: &[(&str, &str)],
        empty: bool,
    ) -> Result<(), Error> {
        if self.open.len() >= MAX_DEPTH {
            return Err(Error::DepthLimit);
        }
        if self.nodes >= MAX_NODES as usize {
            return Err(Error::NodeLimit);
        }
        if attributes.len() > MAX_ATTRIBUTES_PER_ELEMENT
            || self.attributes.saturating_add(attributes.len()) > MAX_ATTRIBUTES
        {
            return Err(Error::AttributeLimit);
        }
        let closing = if empty {
            0
        } else {
            name.len().saturating_add(3)
        };
        let mut length = self
            .output
            .len()
            .saturating_add(self.closing_bytes)
            .saturating_add(closing)
            .saturating_add(name.len())
            .saturating_add(if empty { 3 } else { 2 });
        if length > MAX_INPUT_BYTES {
            return Err(Error::InputTooLarge);
        }
        if !valid_name(name) {
            return Err(Error::InvalidName);
        }
        let mut names = BTreeSet::new();
        for &(key, value) in attributes {
            if key == "xmlns" || !valid_name(key) {
                return Err(Error::InvalidName);
            }
            if !names.insert(key) {
                return Err(Error::DuplicateAttribute);
            }
            length = length.saturating_add(key.len()).saturating_add(4);
            for character in value.chars() {
                if !valid_character(character) {
                    return Err(Error::InvalidAttributeValue);
                }
                length =
                    length.saturating_add(escape(character).map_or(character.len_utf8(), str::len));
                if length > MAX_INPUT_BYTES {
                    return Err(Error::InputTooLarge);
                }
            }
            if length > MAX_INPUT_BYTES {
                return Err(Error::InputTooLarge);
            }
        }
        self.output.push(b'<');
        self.output.extend_from_slice(name.as_bytes());
        for &(key, value) in attributes {
            self.output.push(b' ');
            self.output.extend_from_slice(key.as_bytes());
            self.output.extend_from_slice(b"=\"");
            for character in value.chars() {
                if let Some(escaped) = escape(character) {
                    self.output.extend_from_slice(escaped.as_bytes());
                } else {
                    self.output
                        .extend_from_slice(character.encode_utf8(&mut [0; 4]).as_bytes());
                }
            }
            self.output.push(b'"');
        }
        self.output
            .extend_from_slice(if empty { b"/>" } else { b">" });
        if !empty {
            self.open.push(name.to_owned());
        }
        self.closing_bytes += closing;
        self.attributes += attributes.len();
        self.nodes += 1;
        Ok(())
    }
}

fn valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b'A'..=b'Z' | b'a'..=b'z' | b'_'))
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
