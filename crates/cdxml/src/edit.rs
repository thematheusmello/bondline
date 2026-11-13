use crate::{Document, Error, MAX_INPUT_BYTES};

impl Document<'_> {
    /// Replace one existing ordinary attribute by its exact, case-sensitive qualified name.
    /// Namespace declarations cannot be changed. Every byte outside the value is retained;
    /// a semantically unchanged value preserves even its original character references.
    /// Returns a revalidated copy without changing this document or its source.
    pub fn replace_attribute(
        &self,
        element: roxmltree::Node<'_, '_>,
        qname: &str,
        value: &str,
    ) -> Result<Vec<u8>, Error> {
        if !element.is_element() || !std::ptr::eq(element.document(), &self.xml) {
            return Err(Error::InvalidElement);
        }
        for attribute in element.attributes() {
            let range = attribute.range();
            // Compressed QName/value offsets can truncate; the full attribute range does not.
            let span = self
                .xml
                .input_text()
                .get(range.clone())
                .ok_or(Error::InvalidAttributeRange)?;
            let (name, raw) = span.split_once('=').ok_or(Error::InvalidAttributeRange)?;
            if name.trim_end_matches([' ', '\t', '\r', '\n']) != qname {
                continue;
            }
            if attribute.value() == value {
                return Ok(self.to_bytes());
            }
            let raw = raw.trim_start_matches([' ', '\t', '\r', '\n']);
            let quote = raw
                .as_bytes()
                .first()
                .filter(|&&quote| quote == b'\'' || quote == b'"')
                .ok_or(Error::InvalidAttributeRange)?;
            if raw.len() < 2 || raw.as_bytes().last() != Some(quote) {
                return Err(Error::InvalidAttributeRange);
            }
            let start = range
                .end
                .checked_sub(raw.len())
                .and_then(|offset| offset.checked_add(1))
                .ok_or(Error::InvalidAttributeRange)?;
            let end = range
                .end
                .checked_sub(1)
                .ok_or(Error::InvalidAttributeRange)?;
            let prefix = self
                .source
                .get(..start)
                .ok_or(Error::InvalidAttributeRange)?;
            let suffix = self.source.get(end..).ok_or(Error::InvalidAttributeRange)?;
            let mut length = prefix
                .len()
                .checked_add(suffix.len())
                .ok_or(Error::InputTooLarge)?;
            for character in value.chars() {
                if !matches!(character, '\t' | '\n' | '\r' | ' '..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
                {
                    return Err(Error::InvalidAttributeValue);
                }
                length = length
                    .checked_add(escape(character).map_or(character.len_utf8(), str::len))
                    .filter(|&length| length <= MAX_INPUT_BYTES)
                    .ok_or(Error::InputTooLarge)?;
            }
            let mut output = Vec::with_capacity(length);
            output.extend_from_slice(prefix);
            for character in value.chars() {
                if let Some(escaped) = escape(character) {
                    output.extend_from_slice(escaped.as_bytes());
                } else {
                    output.extend_from_slice(character.encode_utf8(&mut [0; 4]).as_bytes());
                }
            }
            output.extend_from_slice(suffix);
            Document::parse(&output)?;
            return Ok(output);
        }
        Err(Error::MissingAttribute)
    }
}

fn escape(character: char) -> Option<&'static str> {
    match character {
        '&' => Some("&amp;"),
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '"' => Some("&quot;"),
        '\'' => Some("&apos;"),
        '\t' => Some("&#x9;"),
        '\r' => Some("&#xD;"),
        '\n' => Some("&#xA;"),
        _ => None,
    }
}
