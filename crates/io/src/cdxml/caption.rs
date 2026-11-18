//! Explicit caption capability; it never grants chemical or arbitrary source edits.
use super::{MappingError, is_element, parse_position};
use crate::{DocumentError, StoredDocument};
use bondline_cdxml::{Document as Xml, Edit, ObjectId, Resources, TextObject, Writer};
use bondline_model::{
    Document, MAX_TEXT_BYTES, MAX_TEXT_RUNS, StyledText, TextError, TextId, TextRun, TextStyle,
};
use roxmltree::Node;
use std::collections::BTreeMap;

type Result<T> = std::result::Result<T, DocumentError>;

fn unsupported<T>() -> Result<T> {
    Err(DocumentError::UnsupportedEdit)
}

fn syntax<T>(value: std::result::Result<T, bondline_cdxml::Error>) -> Result<T> {
    value
        .map_err(MappingError::from)
        .map_err(DocumentError::from)
}

fn model<T>(value: std::result::Result<T, bondline_model::TextError>) -> Result<T> {
    value
        .map_err(bondline_model::Error::from)
        .map_err(MappingError::from)
        .map_err(DocumentError::from)
}

/// A best-effort typed view. Unsupported source remains opaque at the owner boundary.
pub(super) fn project<'doc, 'input: 'doc>(
    xml: &'doc Xml<'input>,
    page: Node<'doc, 'input>,
) -> Result<Vec<StyledText>> {
    let resources = xml
        .resources()
        .map_err(|_| DocumentError::UnsupportedEdit)?;
    let mut texts = Vec::new();
    for element in page.children().filter(|node| is_element(*node, "t")) {
        let view = xml
            .styled_text(element)
            .map_err(|_| DocumentError::UnsupportedEdit)?;
        let position = element
            .attribute("p")
            .and_then(|raw| parse_position(raw).ok())
            .ok_or(DocumentError::UnsupportedEdit)?;
        let runs = project_runs(&resources, &view)?;
        let ordinal = u32::try_from(texts.len())
            .ok()
            .and_then(|value| value.checked_add(1))
            .and_then(TextId::new)
            .ok_or(DocumentError::UnsupportedEdit)?;
        texts.push(model(StyledText::new(ordinal, position, runs))?);
    }
    Ok(texts)
}

pub(super) fn project_runs(
    resources: &Resources<'_, '_>,
    view: &TextObject<'_, '_>,
) -> Result<Vec<TextRun>> {
    if view.runs().len() > MAX_TEXT_RUNS {
        return model(Err(TextError::RunLimit));
    }
    if view
        .runs()
        .iter()
        .fold(0_usize, |bytes, run| bytes.saturating_add(run.text().len()))
        > MAX_TEXT_BYTES
    {
        return model(Err(TextError::TextLimit));
    }
    view.runs()
        .iter()
        .map(|run| {
            let font = resources
                .font(run.font().ok_or(DocumentError::UnsupportedEdit)?)
                .map_err(|_| DocumentError::UnsupportedEdit)?;
            let rgb = resources
                .color(run.color().ok_or(DocumentError::UnsupportedEdit)?)
                .map_err(|_| DocumentError::UnsupportedEdit)?;
            model(TextRun::new(
                run.text().to_owned(),
                model(TextStyle::new(
                    font.name().to_owned(),
                    run.size_pt().ok_or(DocumentError::UnsupportedEdit)?,
                    run.face().ok_or(DocumentError::UnsupportedEdit)?,
                    rgb,
                ))?,
            ))
        })
        .collect()
}

fn keys(node: Node<'_, '_>, allowed: &[&str], required: &[&str]) -> bool {
    node.attributes()
        .all(|attribute| attribute.namespace().is_none() && allowed.contains(&attribute.name()))
        && required.iter().all(|name| node.attribute(*name).is_some())
}

fn trivia(node: Node<'_, '_>) -> bool {
    node.children().all(|child| {
        child.is_element()
            || child.is_comment()
            || child.is_text()
                && child
                    .text()
                    .is_none_or(|s| s.bytes().all(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n')))
    })
}

fn assigned(node: Node<'_, '_>) -> bool {
    node.attribute("id")
        .and_then(|id| id.parse::<ObjectId>().ok())
        .is_some_and(ObjectId::is_assigned)
}

fn bindings<'doc, 'input: 'doc>(
    xml: &'doc Xml<'input>,
    baseline: &Document,
) -> Result<Vec<bondline_cdxml::TextObject<'doc, 'input>>> {
    let features = xml.features();
    if features.doctype
        || features.processing_instructions
        || features.namespace_declarations
        || baseline.pages().len() != 1
        || !baseline.drawing_settings().is_empty()
        || !baseline.drawing_style().is_empty()
    {
        return unsupported();
    }
    let root = xml.xml().root_element();
    if !keys(root, &["CreationProgram"], &[]) || !trivia(root) {
        return unsupported();
    }
    let resources = xml
        .resources()
        .map_err(|_| DocumentError::UnsupportedEdit)?;
    if resources
        .color(2)
        .map_err(|_| DocumentError::UnsupportedEdit)?
        != [1.0; 3]
    {
        return unsupported();
    }
    let mut page = None;
    for child in root.children().filter(Node::is_element) {
        if !trivia(child) {
            return unsupported();
        }
        match child.tag_name().name() {
            "fonttable" if is_element(child, "fonttable") && keys(child, &[], &[]) => {
                for font in child.children().filter(Node::is_element) {
                    if !is_element(font, "font")
                        || !keys(font, &["id", "name"], &["id", "name"])
                        || font.children().any(|node| node.is_element())
                        || !trivia(font)
                    {
                        return unsupported();
                    }
                }
            }
            "colortable" if is_element(child, "colortable") && keys(child, &[], &[]) => {
                for color in child.children().filter(Node::is_element) {
                    if !is_element(color, "color")
                        || !keys(color, &["r", "g", "b"], &["r", "g", "b"])
                        || color.children().any(|node| node.is_element())
                        || !trivia(color)
                    {
                        return unsupported();
                    }
                    // A negative source magnitude must not become an admitted -0 colour.
                    for channel in ["r", "g", "b"] {
                        let raw = color
                            .attribute(channel)
                            .ok_or(DocumentError::UnsupportedEdit)?
                            .trim();
                        if raw.starts_with('-')
                            && raw.parse::<f64>().ok() == Some(0.0)
                            && raw
                                .split(['e', 'E'])
                                .next()
                                .is_some_and(|m| m.bytes().any(|b| matches!(b, b'1'..=b'9')))
                        {
                            return unsupported();
                        }
                    }
                }
            }
            "page" if is_element(child, "page") && page.replace(child).is_none() => {}
            _ => return unsupported(),
        }
    }
    let page = page.ok_or(DocumentError::UnsupportedEdit)?;
    if !assigned(page)
        || !keys(
            page,
            &["id", "DrawingSpace", "Width", "Height"],
            &["id", "DrawingSpace", "Width", "Height"],
        )
        || page.attribute("DrawingSpace") != Some("Poster")
    {
        return unsupported();
    }
    let own = baseline
        .pages()
        .first()
        .ok_or(DocumentError::UnsupportedEdit)?;
    if !own.fragments().is_empty() || own.texts().is_empty() {
        return unsupported();
    }
    let projected = project(xml, page)?;
    if projected.len() != own.texts().len()
        || !projected
            .iter()
            .zip(own.texts())
            .all(|(a, b)| a == b.as_ref())
        || page.attribute("Width").and_then(|s| s.parse::<f64>().ok())
            != Some(own.size().width_pt())
        || page.attribute("Height").and_then(|s| s.parse::<f64>().ok())
            != Some(own.size().height_pt())
    {
        return unsupported();
    }
    let mut objects = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for child in page.children().filter(Node::is_element) {
        if !is_element(child, "t")
            || !assigned(child)
            || !ids.insert(
                child
                    .attribute("id")
                    .and_then(|id| id.parse::<ObjectId>().ok()),
            )
            || !keys(
                child,
                &["id", "p", "Justification"],
                &["id", "p", "Justification"],
            )
            || child.attribute("Justification") != Some("Left")
        {
            return unsupported();
        }
        let view = xml
            .styled_text(child)
            .map_err(|_| DocumentError::UnsupportedEdit)?;
        for run in view.runs() {
            if !keys(
                run.element(),
                &["font", "size", "face", "color"],
                &["font", "size", "face", "color"],
            ) || run.face() != Some(0)
                || resources
                    .font(run.font().ok_or(DocumentError::UnsupportedEdit)?)
                    .map_err(|_| DocumentError::UnsupportedEdit)?
                    .name()
                    != "Roboto"
            {
                return unsupported();
            }
        }
        objects.push(view);
    }
    if objects.len() != own.texts().len() {
        return unsupported();
    }
    Ok(objects)
}

pub(crate) fn is_caption(source: &[u8], baseline: &Document) -> bool {
    Xml::parse(source)
        .ok()
        .is_some_and(|xml| bindings(&xml, baseline).is_ok())
}

/// Only text bodies may differ; no geometric/style/source authority is inferred.
pub(crate) fn supports_delta(baseline: &Document, current: &Document) -> bool {
    if baseline.drawing_settings() != current.drawing_settings()
        || baseline.drawing_style() != current.drawing_style()
        || baseline.pages().len() != 1
        || current.pages().len() != 1
    {
        return false;
    }
    let (Some(a), Some(b)) = (baseline.pages().first(), current.pages().first()) else {
        return false;
    };
    a.id() == b.id()
        && a.size() == b.size()
        && a.fragments() == b.fragments()
        && a.texts().len() == b.texts().len()
        && a.texts().iter().zip(b.texts()).all(|(a, b)| {
            a.id() == b.id()
                && a.position() == b.position()
                && a.runs().len() == b.runs().len()
                && a.runs()
                    .iter()
                    .zip(b.runs())
                    .all(|(a, b)| a.style() == b.style())
        })
}

pub(crate) fn write_preserved(
    source: &[u8],
    baseline: &Document,
    current: &Document,
) -> Result<Vec<u8>> {
    if !supports_delta(baseline, current) {
        return unsupported();
    }
    let xml = syntax(Xml::parse(source))?;
    let objects = bindings(&xml, baseline)?;
    let page = current
        .pages()
        .first()
        .ok_or(DocumentError::UnsupportedEdit)?;
    let mut changes = Vec::new();
    for (object, text) in objects.iter().zip(page.texts()) {
        for (source, run) in object.runs().iter().zip(text.runs()) {
            if source.text() != run.text() {
                changes.push(Edit::SetText {
                    element: source.element(),
                    value: run.text(),
                });
            }
        }
    }
    let bytes = syntax(xml.apply_edits(&changes))?;
    let reopened = StoredDocument::from_cdxml(bytes.clone().into(), page.size())?;
    if reopened.model() != current
        || (!reopened.has_caption_projection() && !reopened.is_fully_mapped())
    {
        return unsupported();
    }
    Ok(bytes)
}

/// Native resources have fixed white/black entries so existing chemical defaults stay black.
pub(super) struct NativeResources {
    fonts: BTreeMap<String, u16>,
    colors: Vec<[f64; 3]>,
    color_ids: BTreeMap<[u64; 3], u16>,
}

fn color_key(rgb: [f64; 3]) -> [u64; 3] {
    rgb.map(|v| if v == 0.0 { 0 } else { v.to_bits() })
}

impl NativeResources {
    pub(super) fn new(document: &Document) -> std::result::Result<Self, MappingError> {
        if document.pages().iter().all(|p| p.texts().is_empty())
            && !document.has_authored_atom_labels()
        {
            return Ok(Self {
                fonts: BTreeMap::new(),
                colors: Vec::new(),
                color_ids: BTreeMap::new(),
            });
        }
        let mut result = Self {
            fonts: BTreeMap::new(),
            colors: vec![[1.0; 3], [0.0; 3]],
            color_ids: BTreeMap::from([(color_key([1.0; 3]), 2), (color_key([0.0; 3]), 3)]),
        };
        let captions = document
            .pages()
            .iter()
            .flat_map(|p| p.texts())
            .flat_map(|t| t.runs());
        let labels = document
            .pages()
            .iter()
            .flat_map(|p| p.fragments())
            .flat_map(|f| f.atoms())
            .filter_map(|a| a.authored_label.as_ref())
            .flat_map(|label| label.runs());
        for style in captions.chain(labels).map(TextRun::style) {
            if !result.fonts.contains_key(style.font_family()) {
                let id =
                    u16::try_from(result.fonts.len()).map_err(|_| MappingError::IdentityLimit)?;
                result.fonts.insert(style.font_family().to_owned(), id);
            }
            if !result.color_ids.contains_key(&color_key(style.rgb())) {
                if result.colors.len() >= usize::from(u16::MAX) - 1 {
                    return Err(MappingError::IdentityLimit);
                }
                let id = u16::try_from(result.colors.len() + 2)
                    .map_err(|_| MappingError::IdentityLimit)?;
                result.color_ids.insert(color_key(style.rgb()), id);
                result.colors.push(style.rgb());
            }
        }
        Ok(result)
    }
    pub(super) fn write(&self, writer: &mut Writer) -> std::result::Result<(), MappingError> {
        if self.fonts.is_empty() {
            return Ok(());
        }
        writer.start_element("fonttable", &[])?;
        for (name, id) in &self.fonts {
            writer.empty_element("font", &[("id", &id.to_string()), ("name", name)])?;
        }
        writer.end_element()?;
        writer.start_element("colortable", &[])?;
        for rgb in &self.colors {
            writer.empty_element(
                "color",
                &[
                    ("r", &rgb[0].to_string()),
                    ("g", &rgb[1].to_string()),
                    ("b", &rgb[2].to_string()),
                ],
            )?;
        }
        writer.end_element()?;
        Ok(())
    }
    pub(super) fn write_text(
        &self,
        writer: &mut Writer,
        text: &StyledText,
        id: u32,
    ) -> std::result::Result<(), MappingError> {
        let position = text.position();
        writer.start_element(
            "t",
            &[
                ("id", &id.to_string()),
                ("p", &format!("{} {}", position.x_pt(), position.y_pt())),
                ("Justification", "Left"),
            ],
        )?;
        self.write_runs(writer, text.runs())?;
        writer.end_element()?;
        Ok(())
    }

    pub(super) fn write_runs(
        &self,
        writer: &mut Writer,
        runs: &[TextRun],
    ) -> std::result::Result<(), MappingError> {
        for run in runs {
            let style = run.style();
            let font = self
                .fonts
                .get(style.font_family())
                .ok_or(MappingError::InvalidProperty("font"))?;
            let color = self
                .color_ids
                .get(&color_key(style.rgb()))
                .ok_or(MappingError::InvalidProperty("color"))?;
            writer.start_element(
                "s",
                &[
                    ("font", &font.to_string()),
                    ("size", &style.size_pt().to_string()),
                    ("face", &style.face().to_string()),
                    ("color", &color.to_string()),
                ],
            )?;
            writer.text(run.text())?;
            writer.end_element()?;
        }
        Ok(())
    }
}
