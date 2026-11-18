use crate::{MAX_POSITION_PT, Point2};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor, value::MapAccessDeserializer},
};
use std::{mem::size_of, num::NonZeroU32};

pub const MAX_TEXT_RUNS: usize = 4096;
pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
pub const MAX_DOCUMENT_TEXT_BYTES: usize = 64 * MAX_TEXT_BYTES;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", try_from = "String")]
pub enum TextJustification {
    Right,
    Left,
    Center,
    Full,
    Above,
    Below,
    Auto,
    Best,
}

impl TryFrom<String> for TextJustification {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "right" => Ok(Self::Right),
            "left" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "full" => Ok(Self::Full),
            "above" => Ok(Self::Above),
            "below" => Ok(Self::Below),
            "auto" => Ok(Self::Auto),
            "best" => Ok(Self::Best),
            _ => Err("unknown text justification"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", try_from = "String")]
pub enum LabelAlignment {
    Auto,
    Left,
    Center,
    Right,
    Above,
    Below,
    Best,
}

impl TryFrom<String> for LabelAlignment {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "auto" => Ok(Self::Auto),
            "left" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" => Ok(Self::Right),
            "above" => Ok(Self::Above),
            "below" => Ok(Self::Below),
            "best" => Ok(Self::Best),
            _ => Err("unknown label alignment"),
        }
    }
}

/// Application text identity, independent of atom and source font identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextId(NonZeroU32);

impl TextId {
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TextError {
    #[error("font family must contain 1..256 UTF-8 bytes")]
    InvalidFontFamily,
    #[error("text size must be finite above zero through 100000 points")]
    InvalidSize,
    #[error("text colour channels must be finite within 0..1")]
    InvalidColor,
    #[error("text contains a character forbidden by XML 1.0")]
    InvalidCharacter,
    #[error("a text object must contain 1..4096 runs")]
    RunLimit,
    #[error("a text object exceeds 1 MiB of UTF-8 text")]
    TextLimit,
    #[error("text run {0} does not exist")]
    MissingRun(usize),
    #[error("text id {0} is already used")]
    DuplicateText(u32),
    #[error("document exceeds 100000 text objects")]
    TextCountLimit,
    #[error("document exceeds 100000 text runs")]
    DocumentRunLimit,
    #[error("document exceeds 64 MiB of UTF-8 text")]
    DocumentTextLimit,
}

/// Literal resolved style data; a renderer separately proves font/face support.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TextStyle {
    font_family: String,
    size_pt: f64,
    face: u16,
    rgb: [f64; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStyle {
    font_family: String,
    size_pt: f64,
    face: u16,
    rgb: [f64; 3],
}

impl TextStyle {
    pub fn new(
        font_family: String,
        size_pt: f64,
        face: u16,
        rgb: [f64; 3],
    ) -> Result<Self, TextError> {
        if font_family.is_empty() || font_family.len() > 256 {
            return Err(TextError::InvalidFontFamily);
        }
        validate_characters(&font_family)?;
        if !size_pt.is_finite() || size_pt <= 0.0 || size_pt > MAX_POSITION_PT {
            return Err(TextError::InvalidSize);
        }
        if rgb
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(TextError::InvalidColor);
        }
        Ok(Self {
            font_family,
            size_pt,
            face,
            rgb,
        })
    }

    pub fn font_family(&self) -> &str {
        &self.font_family
    }
    pub const fn size_pt(&self) -> f64 {
        self.size_pt
    }
    pub const fn face(&self) -> u16 {
        self.face
    }
    pub const fn rgb(&self) -> [f64; 3] {
        self.rgb
    }
}

impl TryFrom<RawStyle> for TextStyle {
    type Error = TextError;
    fn try_from(raw: RawStyle) -> Result<Self, Self::Error> {
        Self::new(raw.font_family, raw.size_pt, raw.face, raw.rgb)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TextRun {
    text: String,
    style: TextStyle,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRun {
    text: String,
    style: TextStyle,
}

impl TextRun {
    pub fn new(text: String, style: TextStyle) -> Result<Self, TextError> {
        validate_text(&text)?;
        Ok(Self { text, style })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
    pub const fn style(&self) -> &TextStyle {
        &self.style
    }

    pub fn with_text(&self, text: String) -> Result<Self, TextError> {
        validate_text(&text)?;
        Ok(Self {
            text,
            style: self.style.clone(),
        })
    }
}

impl TryFrom<RawRun> for TextRun {
    type Error = TextError;
    fn try_from(raw: RawRun) -> Result<Self, Self::Error> {
        Self::new(raw.text, raw.style)
    }
}

/// Positioned baseline and ordered literal runs, independent of chemical labels.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StyledText {
    id: TextId,
    position: Point2,
    runs: Vec<TextRun>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawText {
    id: TextId,
    #[serde(deserialize_with = "deserialize_position")]
    position: Point2,
    runs: Vec<TextRun>,
}

impl StyledText {
    pub fn new(id: TextId, position: Point2, runs: Vec<TextRun>) -> Result<Self, TextError> {
        if runs.is_empty() || runs.len() > MAX_TEXT_RUNS {
            return Err(TextError::RunLimit);
        }
        if runs
            .iter()
            .fold(0_usize, |bytes, run| bytes.saturating_add(run.text.len()))
            > MAX_TEXT_BYTES
        {
            return Err(TextError::TextLimit);
        }
        Ok(Self { id, position, runs })
    }

    pub const fn id(&self) -> TextId {
        self.id
    }
    pub const fn position(&self) -> Point2 {
        self.position
    }
    pub fn runs(&self) -> &[TextRun] {
        &self.runs
    }

    pub fn with_run_text(&self, index: usize, text: String) -> Result<Self, TextError> {
        let current = self.runs.get(index).ok_or(TextError::MissingRun(index))?;
        validate_text(&text)?;
        if self.text_bytes() - current.text.len() + text.len() > MAX_TEXT_BYTES {
            return Err(TextError::TextLimit);
        }
        let mut runs = self.runs.clone();
        let run = runs.get_mut(index).ok_or(TextError::MissingRun(index))?;
        run.text = text;
        Ok(Self {
            id: self.id,
            position: self.position,
            runs,
        })
    }

    pub fn with_position(&self, position: Point2) -> Result<Self, TextError> {
        Self::new(self.id, position, self.runs.clone())
    }

    pub(crate) fn text_bytes(&self) -> usize {
        self.runs.iter().map(|run| run.text.len()).sum()
    }

    pub(crate) fn storage_bytes(&self) -> usize {
        self.runs.iter().fold(
            size_of::<Self>()
                .saturating_add(self.runs.capacity().saturating_mul(size_of::<TextRun>())),
            |bytes, run| {
                bytes
                    .saturating_add(run.text.capacity())
                    .saturating_add(run.style.font_family.capacity())
            },
        )
    }
}

impl TryFrom<RawText> for StyledText {
    type Error = TextError;
    fn try_from(raw: RawText) -> Result<Self, Self::Error> {
        Self::new(raw.id, raw.position, raw.runs)
    }
}

/// Authored Node child Text. Position and alignment are literal, not layout authority.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AtomLabel {
    position: Option<Point2>,
    justification: Option<TextJustification>,
    label_justification: Option<TextJustification>,
    label_alignment: Option<LabelAlignment>,
    runs: Vec<TextRun>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAtomLabel {
    #[serde(default, deserialize_with = "deserialize_optional_position")]
    position: Option<Point2>,
    justification: Option<TextJustification>,
    label_justification: Option<TextJustification>,
    label_alignment: Option<LabelAlignment>,
    runs: Vec<TextRun>,
}

impl AtomLabel {
    pub fn new(
        position: Option<Point2>,
        justification: Option<TextJustification>,
        label_justification: Option<TextJustification>,
        label_alignment: Option<LabelAlignment>,
        runs: Vec<TextRun>,
    ) -> Result<Self, TextError> {
        if runs.is_empty() || runs.len() > MAX_TEXT_RUNS {
            return Err(TextError::RunLimit);
        }
        if runs
            .iter()
            .fold(0_usize, |bytes, run| bytes.saturating_add(run.text.len()))
            > MAX_TEXT_BYTES
        {
            return Err(TextError::TextLimit);
        }
        Ok(Self {
            position,
            justification,
            label_justification,
            label_alignment,
            runs,
        })
    }

    pub const fn position(&self) -> Option<Point2> {
        self.position
    }
    pub const fn justification(&self) -> Option<TextJustification> {
        self.justification
    }
    pub const fn label_justification(&self) -> Option<TextJustification> {
        self.label_justification
    }
    pub const fn label_alignment(&self) -> Option<LabelAlignment> {
        self.label_alignment
    }
    pub fn runs(&self) -> &[TextRun] {
        &self.runs
    }

    pub fn with_run_text(&self, index: usize, text: String) -> Result<Self, TextError> {
        let current = self.runs.get(index).ok_or(TextError::MissingRun(index))?;
        validate_text(&text)?;
        if self.runs.iter().map(|run| run.text.len()).sum::<usize>() - current.text.len()
            + text.len()
            > MAX_TEXT_BYTES
        {
            return Err(TextError::TextLimit);
        }
        let mut runs = self.runs.clone();
        runs.get_mut(index)
            .ok_or(TextError::MissingRun(index))?
            .text = text;
        Self::new(
            self.position,
            self.justification,
            self.label_justification,
            self.label_alignment,
            runs,
        )
    }

    pub fn with_position(&self, position: Option<Point2>) -> Result<Self, TextError> {
        Self::new(
            position,
            self.justification,
            self.label_justification,
            self.label_alignment,
            self.runs.clone(),
        )
    }

    pub(crate) fn storage_bytes(&self) -> usize {
        self.runs.iter().fold(
            size_of::<Self>()
                .saturating_add(self.runs.capacity().saturating_mul(size_of::<TextRun>())),
            |bytes, run| {
                bytes
                    .saturating_add(run.text.capacity())
                    .saturating_add(run.style.font_family.capacity())
            },
        )
    }
}

impl TryFrom<RawAtomLabel> for AtomLabel {
    type Error = TextError;
    fn try_from(raw: RawAtomLabel) -> Result<Self, Self::Error> {
        Self::new(
            raw.position,
            raw.justification,
            raw.label_justification,
            raw.label_alignment,
            raw.runs,
        )
    }
}

#[derive(Default)]
pub(crate) struct TextBudget {
    objects: usize,
    runs: usize,
    bytes: usize,
}

impl TextBudget {
    pub(crate) fn add(&mut self, runs: &[TextRun]) -> Result<(), TextError> {
        self.objects = self.objects.saturating_add(1);
        self.runs = self.runs.saturating_add(runs.len());
        self.bytes = runs.iter().fold(self.bytes, |bytes, run| {
            bytes.saturating_add(run.text.len())
        });
        if self.objects > crate::MAX_GRAPH_ITEMS {
            return Err(TextError::TextCountLimit);
        }
        if self.runs > crate::MAX_GRAPH_ITEMS {
            return Err(TextError::DocumentRunLimit);
        }
        if self.bytes > MAX_DOCUMENT_TEXT_BYTES {
            return Err(TextError::DocumentTextLimit);
        }
        Ok(())
    }
}

macro_rules! map_deserialize {
    ($name:ident, $raw:ident, $description:literal) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct ObjectVisitor;
                impl<'de> Visitor<'de> for ObjectVisitor {
                    type Value = $name;
                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        formatter.write_str($description)
                    }
                    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                        let raw = $raw::deserialize(MapAccessDeserializer::new(map))?;
                        $name::try_from(raw).map_err(A::Error::custom)
                    }
                }
                deserializer.deserialize_map(ObjectVisitor)
            }
        }
    };
}

map_deserialize!(TextStyle, RawStyle, "a text style object");
map_deserialize!(TextRun, RawRun, "a text run object");
map_deserialize!(StyledText, RawText, "a styled text object");
map_deserialize!(AtomLabel, RawAtomLabel, "an atom label object");

fn deserialize_optional_position<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Point2>, D::Error> {
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Position(#[serde(deserialize_with = "deserialize_position")] Point2);
    Ok(Option::<Position>::deserialize(deserializer)?.map(|position| position.0))
}

fn deserialize_position<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Point2, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Position {
        x_pt: f64,
        y_pt: f64,
    }
    struct PositionVisitor;
    impl<'de> Visitor<'de> for PositionVisitor {
        type Value = Point2;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a point object")
        }
        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            let raw = Position::deserialize(MapAccessDeserializer::new(map))?;
            Point2::new(raw.x_pt, raw.y_pt).map_err(A::Error::custom)
        }
    }
    deserializer.deserialize_map(PositionVisitor)
}

fn validate_text(text: &str) -> Result<(), TextError> {
    if text.len() > MAX_TEXT_BYTES {
        return Err(TextError::TextLimit);
    }
    validate_characters(text)
}

fn validate_characters(value: &str) -> Result<(), TextError> {
    if value.chars().any(|c| !matches!(c, '\u{9}' | '\u{a}' | '\u{d}' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')) {
        return Err(TextError::InvalidCharacter);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Document, Error, Page, PageId, PageSize};
    use std::sync::Arc;

    #[test]
    fn text_ids_are_unique_across_pages_and_replacement_stays_on_requested_page()
    -> Result<(), Box<dyn std::error::Error>> {
        let first = PageId::new(1).ok_or("page")?;
        let second = PageId::new(2).ok_or("page")?;
        let text = StyledText::new(
            TextId::new(77).ok_or("text id")?,
            Point2::new(72.0, 96.0)?,
            vec![TextRun::new(
                "first".into(),
                TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
            )?],
        )?;
        let mut document = Document::default();
        document.pages.push(Page {
            id: second,
            size: PageSize::A4,
            fragments: vec![],
            texts: vec![],
        });
        document.append_texts(first, vec![text.clone()])?;
        let before = document.clone();
        assert!(matches!(
            document.append_texts(second, vec![text.clone()]),
            Err(Error::Text(TextError::DuplicateText(77)))
        ));
        assert!(matches!(
            document.replace_text(second, text.clone()),
            Err(Error::MissingText(77))
        ));
        assert_eq!(document, before);
        assert!(Arc::ptr_eq(
            &document.pages()[0].texts()[0],
            &before.pages()[0].texts()[0]
        ));
        document.append_texts(
            second,
            vec![StyledText::new(
                TextId::new(78).ok_or("id")?,
                text.position(),
                text.runs().to_vec(),
            )?],
        )?;
        assert_eq!(document.pages()[1].texts()[0].id().get(), 78);
        Ok(())
    }
}
