//! One immutable byte plan; syntax edits never grant chemical-model authority.
use crate::{Document, Error, MAX_ATTRIBUTES, MAX_INPUT_BYTES, MAX_NODES};
use roxmltree::Node;
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    ops::Range,
};

pub const MAX_EDITS: usize = MAX_ATTRIBUTES + MAX_NODES as usize;

/// Elements must belong to the receiver. Appended XML is self-contained, without a DTD
/// or declaration or destination-dependent namespace names. Namespace declarations
/// themselves cannot be edited.
#[derive(Clone, Copy, Debug)]
pub enum Edit<'doc, 'input: 'doc, 'value> {
    SetAttribute {
        element: Node<'doc, 'input>,
        name: &'value str,
        value: &'value str,
    },
    RemoveAttribute {
        element: Node<'doc, 'input>,
        name: &'value str,
    },
    AppendChildren {
        element: Node<'doc, 'input>,
        xml: &'value str,
    },
}

struct Change {
    range: Range<usize>,
    text: String,
    owner_start: usize,
    order: usize,
}

#[derive(Default)]
struct Plan {
    changes: Vec<Change>,
    inserted: usize,
}

impl Plan {
    fn remaining(&self) -> usize {
        MAX_INPUT_BYTES.saturating_sub(self.inserted)
    }

    fn push(&mut self, range: Range<usize>, text: String, owner_start: usize) -> Result<(), Error> {
        self.inserted = self
            .inserted
            .checked_add(text.len())
            .filter(|&length| length <= MAX_INPUT_BYTES)
            .ok_or(Error::InputTooLarge)?;
        self.changes.push(Change {
            range,
            text,
            owner_start,
            order: self.changes.len(),
        });
        Ok(())
    }

    fn apply(mut self, source: &[u8]) -> Result<Vec<u8>, Error> {
        // Close a descendant expansion before its ancestor appends at the same byte.
        self.changes.sort_unstable_by_key(|change| {
            (
                change.range.start,
                change.range.end,
                Reverse(change.owner_start),
                change.order,
            )
        });
        let mut cursor = 0;
        let mut removed = 0_usize;
        for change in &self.changes {
            if change.range.end < change.range.start || change.range.end > source.len() {
                return Err(Error::InvalidAttributeRange);
            }
            if change.range.start < cursor {
                return Err(Error::ConflictingEdits);
            }
            removed = removed
                .checked_add(change.range.len())
                .ok_or(Error::InputTooLarge)?;
            cursor = change.range.end;
        }
        let length = source
            .len()
            .checked_sub(removed)
            .and_then(|length| length.checked_add(self.inserted))
            .filter(|&length| length <= MAX_INPUT_BYTES)
            .ok_or(Error::InputTooLarge)?;
        let mut output = Vec::with_capacity(length);
        cursor = 0;
        for change in self.changes {
            output.extend_from_slice(
                source
                    .get(cursor..change.range.start)
                    .ok_or(Error::InvalidAttributeRange)?,
            );
            output.extend_from_slice(change.text.as_bytes());
            cursor = change.range.end;
        }
        output.extend_from_slice(source.get(cursor..).ok_or(Error::InvalidAttributeRange)?);
        Document::parse(&output)?;
        Ok(output)
    }
}

#[derive(Clone)]
struct AttributeRange {
    whole: Range<usize>,
    value: Range<usize>,
    index: usize,
}

struct Tag<'input> {
    name: &'input str,
    delimiter: usize,
    content_start: usize,
    element_end: usize,
    empty: bool,
}

impl Document<'_> {
    /// Change ordinary attributes and append children in one checked source-preserving
    /// batch. Duplicate targets conflict even when individually unchanged; unrelated bytes,
    /// quote delimiters and decoded-string no-op lexemes remain exact.
    pub fn apply_edits(&self, edits: &[Edit<'_, '_, '_>]) -> Result<Vec<u8>, Error> {
        if edits.len() > MAX_EDITS {
            return Err(Error::EditLimit);
        }
        let mut seen = BTreeSet::new();
        let mut attributes = BTreeMap::new();
        let mut tags = BTreeMap::new();
        let mut plan = Plan::default();
        for edit in edits {
            let (element, name) = match *edit {
                Edit::SetAttribute { element, name, .. }
                | Edit::RemoveAttribute { element, name } => (element, Some(name)),
                Edit::AppendChildren { element, .. } => (element, None),
            };
            if !element.is_element() || !std::ptr::eq(element.document(), &self.xml) {
                return Err(Error::InvalidElement);
            }
            if !seen.insert((element.id().get(), name)) {
                return Err(Error::ConflictingEdits);
            }
            if name.is_some_and(|name| name == "xmlns" || name.starts_with("xmlns:")) {
                return Err(Error::InvalidName);
            }
            let tag = match tags.entry(element.id().get()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(self.tag(element)?),
            };
            // Index source spans once per element, including repeated absent-name lookups.
            let attribute = if let Some(name) = name {
                let index = match attributes.entry(element.id().get()) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => entry.insert(self.attribute_index(element)?),
                };
                index.get(name).cloned()
            } else {
                None
            };
            match *edit {
                Edit::SetAttribute {
                    element,
                    name,
                    value,
                } => {
                    if let Some(attribute) = attribute {
                        let previous = element
                            .attributes()
                            .nth(attribute.index)
                            .ok_or(Error::InvalidAttributeRange)?;
                        if previous.value() != value {
                            plan.push(
                                attribute.value,
                                escaped(value, plan.remaining())?,
                                element.range().start,
                            )?;
                        }
                    } else {
                        if !crate::writer::valid_name(name) {
                            return Err(Error::InvalidName);
                        }
                        let overhead = name.len().checked_add(4).ok_or(Error::InputTooLarge)?;
                        let available = plan
                            .remaining()
                            .checked_sub(overhead)
                            .ok_or(Error::InputTooLarge)?;
                        let value = escaped(value, available)?;
                        let mut text = String::with_capacity(overhead + value.len());
                        text.push(' ');
                        text.push_str(name);
                        text.push_str("=\"");
                        text.push_str(&value);
                        text.push('"');
                        let at = tag.delimiter;
                        plan.push(at..at, text, element.range().start)?;
                    }
                }
                Edit::RemoveAttribute { element, .. } => {
                    if let Some(attribute) = attribute {
                        plan.push(attribute.whole, String::new(), element.range().start)?;
                    }
                }
                Edit::AppendChildren { element, xml } => {
                    if xml.is_empty() {
                        continue;
                    }
                    if xml.len() > plan.remaining() {
                        return Err(Error::InputTooLarge);
                    }
                    validate_fragment(
                        xml,
                        element
                            .lookup_namespace_uri(None)
                            .is_some_and(|uri| !uri.is_empty()),
                    )?;
                    if tag.empty {
                        let length = xml
                            .len()
                            .checked_add(tag.name.len())
                            .and_then(|length| length.checked_add(3))
                            .filter(|&length| length <= plan.remaining())
                            .ok_or(Error::InputTooLarge)?;
                        let mut text = String::with_capacity(length);
                        text.push_str(xml);
                        text.push_str("</");
                        text.push_str(tag.name);
                        text.push('>');
                        plan.push(
                            tag.delimiter..tag.delimiter + 1,
                            String::new(),
                            element.range().start,
                        )?;
                        plan.push(
                            tag.content_start..tag.content_start,
                            text,
                            element.range().start,
                        )?;
                    } else {
                        let mut name_end = tag
                            .element_end
                            .checked_sub(1)
                            .ok_or(Error::InvalidAttributeRange)?;
                        while name_end > 0
                            && self
                                .source
                                .get(name_end - 1)
                                .is_some_and(|&byte| xml_space(byte))
                        {
                            name_end -= 1;
                        }
                        let name_start = name_end
                            .checked_sub(tag.name.len())
                            .ok_or(Error::InvalidAttributeRange)?;
                        let at = name_start
                            .checked_sub(2)
                            .ok_or(Error::InvalidAttributeRange)?;
                        if self.source.get(at..name_start) != Some(b"</")
                            || self.source.get(name_start..name_end) != Some(tag.name.as_bytes())
                        {
                            return Err(Error::InvalidAttributeRange);
                        }
                        plan.push(at..at, xml.to_owned(), element.range().start)?;
                    }
                }
            }
        }
        plan.apply(self.source)
    }

    fn attribute_index(
        &self,
        element: Node<'_, '_>,
    ) -> Result<BTreeMap<&str, AttributeRange>, Error> {
        let mut attributes = BTreeMap::new();
        for (index, attribute) in element.attributes().enumerate() {
            let whole = attribute.range();
            let span = self
                .xml
                .input_text()
                .get(whole.clone())
                .ok_or(Error::InvalidAttributeRange)?;
            let (qname, raw) = span.split_once('=').ok_or(Error::InvalidAttributeRange)?;
            let name = qname.trim_end_matches([' ', '\t', '\r', '\n']);
            let raw = raw.trim_start_matches([' ', '\t', '\r', '\n']);
            let quote = raw
                .as_bytes()
                .first()
                .filter(|&&byte| byte == b'\'' || byte == b'"')
                .ok_or(Error::InvalidAttributeRange)?;
            if raw.len() < 2 || raw.as_bytes().last() != Some(quote) {
                return Err(Error::InvalidAttributeRange);
            }
            let start = whole
                .end
                .checked_sub(raw.len())
                .and_then(|start| start.checked_add(1))
                .ok_or(Error::InvalidAttributeRange)?;
            let end = whole
                .end
                .checked_sub(1)
                .ok_or(Error::InvalidAttributeRange)?;
            attributes.insert(
                name,
                AttributeRange {
                    whole,
                    value: start..end,
                    index,
                },
            );
        }
        Ok(attributes)
    }

    fn tag(&self, element: Node<'_, '_>) -> Result<Tag<'_>, Error> {
        let range = element.range();
        let span = self
            .xml
            .input_text()
            .get(range.clone())
            .ok_or(Error::InvalidAttributeRange)?;
        let mut quote = None;
        let mut close = None;
        for (index, byte) in span.bytes().enumerate() {
            match (quote, byte) {
                (Some(open), byte) if byte == open => quote = None,
                (None, b'\'' | b'"') => quote = Some(byte),
                (None, b'>') => {
                    close = Some(index);
                    break;
                }
                _ => {}
            }
        }
        let close = close.ok_or(Error::InvalidAttributeRange)?;
        let before = close.checked_sub(1).ok_or(Error::InvalidAttributeRange)?;
        let empty = span.as_bytes().get(before) == Some(&b'/');
        let name_end = span
            .bytes()
            .enumerate()
            .skip(1)
            .find(|&(_, byte)| xml_space(byte) || byte == b'/' || byte == b'>')
            .map(|(index, _)| index)
            .ok_or(Error::InvalidAttributeRange)?;
        Ok(Tag {
            name: span.get(1..name_end).ok_or(Error::InvalidAttributeRange)?,
            delimiter: range.start + if empty { before } else { close },
            content_start: range.start + close + 1,
            element_end: range.end,
            empty,
        })
    }
}

fn xml_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

fn escaped(value: &str, available: usize) -> Result<String, Error> {
    let mut length = 0_usize;
    for character in value.chars() {
        if !crate::edit::valid_character(character) {
            return Err(Error::InvalidAttributeValue);
        }
        length = length
            .checked_add(crate::edit::escape(character).map_or(character.len_utf8(), str::len))
            .filter(|&length| length <= available)
            .ok_or(Error::InputTooLarge)?;
    }
    let mut output = String::with_capacity(length);
    for character in value.chars() {
        if let Some(escaped) = crate::edit::escape(character) {
            output.push_str(escaped);
        } else {
            output.push(character);
        }
    }
    Ok(output)
}

fn validate_fragment(xml: &str, inherits_default_namespace: bool) -> Result<(), Error> {
    let length = xml
        .len()
        .checked_add("<CDXML></CDXML>".len())
        .filter(|&length| length <= MAX_INPUT_BYTES)
        .ok_or(Error::InputTooLarge)?;
    let mut wrapped = String::with_capacity(length);
    wrapped.push_str("<CDXML>");
    wrapped.push_str(xml);
    wrapped.push_str("</CDXML>");
    match Document::parse(wrapped.as_bytes()) {
        Ok(fragment) => {
            // An explicit empty default binding is retained by roxmltree as Some("").
            if inherits_default_namespace
                && fragment
                    .xml()
                    .root_element()
                    .descendants()
                    .skip(1)
                    .any(|node| {
                        node.is_element()
                            && node.tag_name().namespace().is_none()
                            && node.lookup_namespace_uri(None).is_none()
                    })
            {
                return Err(Error::InvalidFragment);
            }
            Ok(())
        }
        Err(
            error @ (Error::InputTooLarge
            | Error::DepthLimit
            | Error::AttributeLimit
            | Error::NodeLimit),
        )
        | Err(error @ Error::Xml(roxmltree::Error::NodesLimitReached)) => Err(error),
        Err(_) => Err(Error::InvalidFragment),
    }
}
