//! Standalone source-preserving CDXML syntax. No document model or native I/O.

mod edit;
mod numbers;
mod patch;
mod resources;
mod writer;
pub use numbers::{ObjectId, parse_integer};
pub use patch::{Edit, MAX_EDITS};
pub use resources::{Color, Font, ResourceError, Resources};
pub use writer::Writer;

pub const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_NODES: u32 = 100_000;
pub const MAX_DEPTH: usize = 256;
pub const MAX_ATTRIBUTES_PER_ELEMENT: usize = 256;
pub const MAX_ATTRIBUTES: usize = 100_000;

/// Lexical information a read-only XML tree may omit. These facts do not grant edit authority.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SyntaxFeatures {
    pub doctype: bool,
    pub processing_instructions: bool,
    pub cdata: bool,
    pub namespace_declarations: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("CDXML input or output exceeds the 64 MiB bound")]
    InputTooLarge,
    #[error("CDXML nesting exceeds 256 elements")]
    DepthLimit,
    #[error("CDXML exceeds 256 attributes per element or 100000 attributes in total")]
    AttributeLimit,
    #[error("CDXML requires an unnamespaced CDXML root element")]
    InvalidRoot,
    #[error("only XML 1.0 with UTF-8 encoding is supported")]
    UnsupportedDeclaration,
    #[error("internal DTD subsets and custom entities are not supported")]
    UnsupportedDtd,
    #[error("attribute edits require an element from this parsed document")]
    InvalidElement,
    #[error("the element has no ordinary attribute with that qualified name")]
    MissingAttribute,
    #[error("attribute value contains a character forbidden in XML 1.0")]
    InvalidAttributeValue,
    #[error(
        "new element/attribute names must be unnamespaced ASCII XML names; xmlns is not an ordinary attribute"
    )]
    InvalidName,
    #[error("new elements cannot contain duplicate attributes")]
    DuplicateAttribute,
    #[error("writer has unclosed elements or attempted to close its owned root")]
    UnbalancedElements,
    #[error("CDXML exceeds 100000 syntax nodes")]
    NodeLimit,
    #[error("attribute source range is inconsistent with the validated XML")]
    InvalidAttributeRange,
    #[error("batch XML edits exceed the 200000-operation bound")]
    EditLimit,
    #[error("batch XML edits address the same attribute/container twice or overlap")]
    ConflictingEdits,
    #[error("appended XML must be a self-contained fragment without a declaration or DTD")]
    InvalidFragment,
    #[error(
        "an exact decimal integer is required; fractional values and exponents are unsupported"
    )]
    InvalidInteger,
    #[error("object ids must fit an unsigned 32-bit integer")]
    InvalidObjectId,
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Xml(#[from] roxmltree::Error),
    #[error(transparent)]
    Token(#[from] quick_xml::Error),
}

/// Validated syntax plus the original source. Unknown content is never regenerated.
#[derive(Debug)]
pub struct Document<'input> {
    source: &'input [u8],
    xml: roxmltree::Document<'input>,
    features: SyntaxFeatures,
}

impl<'input> Document<'input> {
    pub fn parse(input: &'input [u8]) -> Result<Self, Error> {
        if input.len() > MAX_INPUT_BYTES {
            return Err(Error::InputTooLarge);
        }
        let source = std::str::from_utf8(input)?;
        let features = preflight(source)?;
        let xml = roxmltree::Document::parse_with_options(
            source,
            roxmltree::ParsingOptions {
                // Preflight rejects internal subsets before this parser sees them.
                allow_dtd: true,
                nodes_limit: MAX_NODES,
                entity_resolver: None,
            },
        )?;
        let root = xml.root_element();
        if root.tag_name().name() != "CDXML" || root.tag_name().namespace().is_some() {
            return Err(Error::InvalidRoot);
        }
        Ok(Self {
            source: input,
            xml,
            features,
        })
    }

    pub fn xml(&self) -> &roxmltree::Document<'input> {
        &self.xml
    }
    pub fn features(&self) -> SyntaxFeatures {
        self.features
    }
    pub fn as_bytes(&self) -> &'input [u8] {
        self.source
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.source.to_vec()
    }
}

fn preflight(source: &str) -> Result<SyntaxFeatures, Error> {
    use quick_xml::{Reader, events::Event};
    let mut reader = Reader::from_str(source);
    let mut depth = 0_usize;
    let mut attributes = 0_usize;
    let mut features = SyntaxFeatures::default();
    loop {
        let event = reader.read_event()?;
        match &event {
            Event::Decl(declaration) => {
                if declaration.version()? != "1.0"
                    || declaration
                        .encoding()
                        .transpose()
                        .map_err(|_| Error::UnsupportedDeclaration)?
                        .is_some_and(|encoding| !encoding.eq_ignore_ascii_case("UTF-8"))
                    || declaration
                        .standalone()
                        .transpose()
                        .map_err(|_| Error::UnsupportedDeclaration)?
                        .is_some_and(|standalone| standalone != "yes" && standalone != "no")
                {
                    return Err(Error::UnsupportedDeclaration);
                }
            }
            Event::DocType(doctype) => {
                features.doctype = true;
                let mut quote = None;
                for character in doctype.chars() {
                    match (quote, character) {
                        (Some(open), close) if open == close => quote = None,
                        (None, '\'' | '"') => quote = Some(character),
                        (None, '[') => return Err(Error::UnsupportedDtd),
                        _ => {}
                    }
                }
            }
            Event::Start(element) | Event::Empty(element) => {
                if depth >= MAX_DEPTH {
                    return Err(Error::DepthLimit);
                }
                if matches!(&event, Event::Start(_)) {
                    depth += 1;
                }
                for (index, attribute) in element.attributes().with_checks(false).enumerate() {
                    let attribute = attribute.map_err(quick_xml::Error::from)?;
                    let name = attribute.key.as_ref();
                    features.namespace_declarations |=
                        name == "xmlns" || name.starts_with("xmlns:");
                    attributes += 1;
                    if index >= MAX_ATTRIBUTES_PER_ELEMENT || attributes > MAX_ATTRIBUTES {
                        return Err(Error::AttributeLimit);
                    }
                }
            }
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::PI(_) => features.processing_instructions = true,
            Event::CData(_) => features.cdata = true,
            Event::Eof => return Ok(features),
            _ => {}
        }
    }
}
