//! Owned literal child Text; placement and chemistry agreement remain unverified.
use super::{MappingError, caption, parse_position, write::borrowed};
use crate::DocumentError;
use bondline_cdxml::{Document as Xml, Resources, Writer};
use bondline_model::{AtomLabel, LabelAlignment, TextJustification};
use roxmltree::Node;

pub(super) fn is_text(node: Node<'_, '_>) -> bool {
    node.is_element()
        && node.tag_name().name() == "t"
        && node.tag_name().namespace().is_none_or(str::is_empty)
}

pub(super) fn project<'doc, 'input: 'doc>(
    xml: &'doc Xml<'input>,
    resources: &Resources<'doc, 'input>,
    node: Node<'doc, 'input>,
) -> Result<AtomLabel, DocumentError> {
    let mut children = node.children().filter(|child| is_text(*child));
    let text = children.next().ok_or(DocumentError::UnsupportedEdit)?;
    if children.next().is_some() {
        return Err(DocumentError::UnsupportedEdit);
    }
    let position = text
        .attribute("p")
        .map(parse_position)
        .transpose()
        .map_err(|_| DocumentError::UnsupportedEdit)?;
    let read_justification = |name| {
        text.attribute(name)
            .map(|value| match value {
                "Right" => Ok(TextJustification::Right),
                "Left" => Ok(TextJustification::Left),
                "Center" => Ok(TextJustification::Center),
                "Full" => Ok(TextJustification::Full),
                "Above" => Ok(TextJustification::Above),
                "Below" => Ok(TextJustification::Below),
                "Auto" => Ok(TextJustification::Auto),
                "Best" => Ok(TextJustification::Best),
                _ => Err(DocumentError::UnsupportedEdit),
            })
            .transpose()
    };
    let alignment = text
        .attribute("LabelAlignment")
        .map(|value| match value {
            "Auto" => Ok(LabelAlignment::Auto),
            "Left" => Ok(LabelAlignment::Left),
            "Center" => Ok(LabelAlignment::Center),
            "Right" => Ok(LabelAlignment::Right),
            "Above" => Ok(LabelAlignment::Above),
            "Below" => Ok(LabelAlignment::Below),
            "Best" => Ok(LabelAlignment::Best),
            _ => Err(DocumentError::UnsupportedEdit),
        })
        .transpose()?;
    let view = xml
        .styled_text(text)
        .map_err(|_| DocumentError::UnsupportedEdit)?;
    let label = AtomLabel::new(
        position,
        read_justification("Justification")?,
        read_justification("LabelJustification")?,
        alignment,
        caption::project_runs(resources, &view)?,
    )
    .map_err(bondline_model::Error::from)
    .map_err(MappingError::from)
    .map_err(DocumentError::from)?;
    let bounds = text.attribute("BoundingBox").map(parse_box).transpose()?;
    label
        .with_bounding_box(bounds)
        .map_err(bondline_model::Error::from)
        .map_err(MappingError::from)
        .map_err(DocumentError::from)
}

fn parse_box(raw: &str) -> Result<[f64; 4], DocumentError> {
    let mut fields = raw.split_ascii_whitespace();
    let mut next = || {
        fields
            .next()
            .and_then(|value| value.parse::<f64>().ok())
            .ok_or(DocumentError::UnsupportedEdit)
    };
    let bounds = [next()?, next()?, next()?, next()?];
    if fields.next().is_some() {
        return Err(DocumentError::UnsupportedEdit);
    }
    Ok(bounds)
}

fn justification(value: TextJustification) -> &'static str {
    match value {
        TextJustification::Right => "Right",
        TextJustification::Left => "Left",
        TextJustification::Center => "Center",
        TextJustification::Full => "Full",
        TextJustification::Above => "Above",
        TextJustification::Below => "Below",
        TextJustification::Auto => "Auto",
        TextJustification::Best => "Best",
    }
}

pub(super) fn write(
    writer: &mut Writer,
    resources: &caption::NativeResources,
    label: &AtomLabel,
    id: u32,
) -> Result<(), MappingError> {
    let mut attributes = vec![("id", id.to_string())];
    if let Some(position) = label.position() {
        attributes.push(("p", format!("{} {}", position.x_pt(), position.y_pt())));
    }
    if let Some([left, top, right, bottom]) = label.bounding_box() {
        attributes.push(("BoundingBox", format!("{left} {top} {right} {bottom}")));
    }
    if let Some(value) = label.justification() {
        attributes.push(("Justification", justification(value).to_owned()));
    }
    if let Some(value) = label.label_justification() {
        attributes.push(("LabelJustification", justification(value).to_owned()));
    }
    if let Some(value) = label.label_alignment() {
        let value = match value {
            LabelAlignment::Auto => "Auto",
            LabelAlignment::Left => "Left",
            LabelAlignment::Center => "Center",
            LabelAlignment::Right => "Right",
            LabelAlignment::Above => "Above",
            LabelAlignment::Below => "Below",
            LabelAlignment::Best => "Best",
        };
        attributes.push(("LabelAlignment", value.to_owned()));
    }
    writer.start_element("t", &borrowed(&attributes))?;
    resources.write_runs(writer, label.runs())?;
    writer.end_element()?;
    Ok(())
}
