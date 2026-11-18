//! Closed box-positioned atom labels; source coverage is separate from full authoring.
use super::{MappingError, atom_text, caption, graph, is_element, parse_position, write};
use crate::{DocumentError, StoredDocument, display::connected_label_bonds};
use bondline_cdxml::{
    Document as Xml, Edit, ObjectId, Resources, TextObject, Writer, parse_integer,
};
use bondline_model::{Atom, AtomId, AtomLabel, Bond, Document, Page, Radical};
use roxmltree::Node;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

type Result<T> = std::result::Result<T, DocumentError>;

// Match the shared renderer's run/glyph work bounds before measuring/copying source text.
const MAX_TEXT_WORK: usize = 4096;

#[derive(Clone, Copy)]
enum Profile {
    Boxed,
    Connected,
    Mixed,
    Flagged,
    LabelDefaults,
}

fn unsupported<T>() -> Result<T> {
    Err(DocumentError::UnsupportedEdit)
}

fn syntax<T>(value: std::result::Result<T, bondline_cdxml::Error>) -> Result<T> {
    value
        .map_err(MappingError::from)
        .map_err(DocumentError::from)
}

fn one_page(model: &Document) -> Result<&Page> {
    if model.pages().len() != 1 {
        return unsupported();
    }
    model.pages().first().ok_or(DocumentError::UnsupportedEdit)
}

/// Known model coverage, before physical view, collision and raster-work checks.
pub(crate) fn model_supported(model: &Document) -> bool {
    model_supported_with_flags(model, false)
}

fn model_supported_with_flags(model: &Document, allow_flags: bool) -> bool {
    let Ok(page) = one_page(model) else {
        return false;
    };
    if !model.has_authored_atom_labels() {
        return false;
    }
    let mut hits = 0_usize;
    for fragment in page.fragments() {
        hits = hits
            .saturating_add(fragment.atoms().len())
            .saturating_add(fragment.bonds().len());
        if hits > bondline_render::MAX_HIT_REGIONS {
            return false;
        }
    }
    if page.texts().len() > MAX_TEXT_WORK {
        return false;
    }
    let (mut runs, mut glyphs) = (0_usize, 0_usize);
    let labels = page
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.atoms())
        .filter_map(|atom| atom.authored_label.as_ref());
    for label in labels.clone() {
        // The canonical symbol + H/count spelling uses at most eight ASCII bytes.
        if label.runs().len() != 1 || label.runs().first().is_none_or(|run| run.text().len() > 8) {
            return false;
        }
    }
    let text_runs = page.texts().iter().flat_map(|text| text.runs());
    for run in labels.flat_map(|label| label.runs()).chain(text_runs) {
        runs = runs.saturating_add(1);
        glyphs = glyphs.saturating_add(run.text().chars().take(MAX_TEXT_WORK + 1).count());
        if runs > MAX_TEXT_WORK
            || glyphs > MAX_TEXT_WORK
            || run.style().font_family() != "Roboto"
            || run.style().face() != 0
            || run.text().chars().any(char::is_control)
        {
            return false;
        }
    }
    for atom in page
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.atoms())
    {
        if atom.position.is_none()
            || !allow_flags
                && atom.authored_label.is_some()
                && (atom.show_terminal_carbon_labels.is_some()
                    || atom.show_non_terminal_carbon_labels.is_some())
            || atom.authored_label.is_some() && bondline_render::validate_atom_label(atom).is_err()
        {
            return false;
        }
    }
    true
}

fn allowed_atom_delta(before: &Atom, after: &Atom) -> bool {
    let (Some(old), Some(new)) = (&before.authored_label, &after.authored_label) else {
        let mut point_only = before.clone();
        point_only.position = after.position;
        return point_only == *after;
    };
    before.id == after.id
        && before.isotope == after.isotope
        && before.charge == after.charge
        && before.radical == after.radical
        && before.show_terminal_carbon_labels == after.show_terminal_carbon_labels
        && before.show_non_terminal_carbon_labels == after.show_non_terminal_carbon_labels
        && old.position() == new.position()
        && old.justification() == new.justification()
        && old.label_justification() == new.label_justification()
        && old.label_alignment() == new.label_alignment()
        && old.runs().len() == 1
        && new.runs().len() == 1
        && old
            .runs()
            .first()
            .zip(new.runs().first())
            .is_some_and(|(a, b)| a.style() == b.style())
}

pub(crate) fn supports_delta(baseline: &Document, current: &Document) -> bool {
    model_delta(baseline, current, false)
}

fn model_delta(baseline: &Document, current: &Document, creation: bool) -> bool {
    let (Ok(before), Ok(after)) = (one_page(baseline), one_page(current)) else {
        return false;
    };
    baseline.drawing_settings() == current.drawing_settings()
        && baseline.drawing_style() == current.drawing_style()
        && before.id() == after.id()
        && before.size() == after.size()
        && before.texts() == after.texts()
        && before.fragments().len() == after.fragments().len()
        && model_supported_with_flags(baseline, creation)
        && model_supported_with_flags(current, creation)
        && before
            .fragments()
            .iter()
            .zip(after.fragments())
            .all(|(old, new)| {
                old.id() == new.id()
                    && old.bonds() == new.bonds()
                    && old.atoms().len() == new.atoms().len()
                    && old.atoms().iter().zip(new.atoms()).all(|(a, b)| {
                        allowed_atom_delta(a, b)
                            || creation
                                && a.authored_label.is_none()
                                && b.authored_label.is_some()
                                && a.id == b.id
                                && a.isotope == b.isotope
                                && a.charge == b.charge
                                && a.radical == b.radical
                                && a.show_terminal_carbon_labels == b.show_terminal_carbon_labels
                                && a.show_non_terminal_carbon_labels
                                    == b.show_non_terminal_carbon_labels
                    })
            })
}

pub(crate) fn supports_connected_delta(baseline: &Document, current: &Document) -> bool {
    supports_delta(baseline, current)
        && one_page(current).is_ok_and(|page| connected_label_bonds(current, page))
}

fn mixed_hidden_supported(page: &Page) -> bool {
    hidden_supported(page, true)
}

fn hidden_supported(page: &Page, require_any: bool) -> bool {
    let cancel = bondline_render::Cancellation::default();
    let mut has_hidden = false;
    for fragment in page.fragments() {
        let Ok(hidden) = bondline_render::hidden_carbon_atoms(fragment, &cancel) else {
            return false;
        };
        for atom in fragment
            .atoms()
            .iter()
            .filter(|atom| atom.authored_label.is_none())
        {
            if atom.element.get() != 6
                || atom.charge != 0
                || atom.isotope.is_some()
                || atom.radical != Radical::None
                || !hidden.contains(&atom.id)
            {
                return false;
            }
            has_hidden = true;
        }
    }
    has_hidden || !require_any
}

fn flagged_page_supported(model: &Document, page: &Page) -> bool {
    connected_label_bonds(model, page)
        && hidden_supported(page, false)
        && page.fragments().iter().flat_map(|f| f.atoms()).any(|atom| {
            atom.authored_label.is_some()
                && (atom.show_terminal_carbon_labels.is_some()
                    || atom.show_non_terminal_carbon_labels.is_some())
        })
}

fn label_defaults_page_supported(model: &Document, page: &Page) -> bool {
    if page.fragments().iter().all(|f| f.bonds().is_empty()) {
        page.fragments()
            .iter()
            .flat_map(|f| f.atoms())
            .all(|atom| atom.authored_label.is_some())
    } else {
        connected_label_bonds(model, page) && hidden_supported(page, false)
    }
}

fn flagged_model_delta(baseline: &Document, current: &Document) -> bool {
    model_delta(baseline, current, true)
        && one_page(current).is_ok_and(|page| flagged_page_supported(current, page))
}

fn run_resources(label: &AtomLabel, resources: &Resources<'_, '_>) -> Result<(u16, u16)> {
    let style = label
        .runs()
        .first()
        .ok_or(DocumentError::UnsupportedEdit)?
        .style();
    let font = resources
        .fonts()
        .find(|font| font.name() == style.font_family())
        .ok_or(DocumentError::UnsupportedEdit)?
        .id();
    let color = [0, 1]
        .into_iter()
        .chain(resources.colors().iter().map(|c| c.index()))
        .find(|index| resources.color(*index).ok() == Some(style.rgb()))
        .ok_or(DocumentError::UnsupportedEdit)?;
    Ok((font, color))
}

pub(crate) fn supports_flagged_delta(
    source: &[u8],
    baseline: &Document,
    current: &Document,
) -> bool {
    if !flagged_model_delta(baseline, current) {
        return false;
    }
    created_resources_supported(source, baseline, current)
}

pub(crate) fn supports_label_defaults_delta(
    source: &[u8],
    baseline: &Document,
    current: &Document,
) -> bool {
    model_delta(baseline, current, true)
        && one_page(current).is_ok_and(|page| label_defaults_page_supported(current, page))
        && created_resources_supported(source, baseline, current)
}

fn created_resources_supported(source: &[u8], baseline: &Document, current: &Document) -> bool {
    let Ok(xml) = Xml::parse(source) else {
        return false;
    };
    let Ok(resources) = xml.resources() else {
        return false;
    };
    baseline
        .pages()
        .iter()
        .flat_map(|p| p.fragments())
        .flat_map(|f| f.atoms())
        .zip(
            current
                .pages()
                .iter()
                .flat_map(|p| p.fragments())
                .flat_map(|f| f.atoms()),
        )
        .all(|(before, after)| {
            before.authored_label.is_some()
                || after
                    .authored_label
                    .as_ref()
                    .is_none_or(|label| run_resources(label, &resources).is_ok())
        })
}

pub(crate) fn supports_mixed_delta(baseline: &Document, current: &Document) -> bool {
    supports_connected_delta(baseline, current)
        && one_page(current).is_ok_and(mixed_hidden_supported)
}

fn keys(node: Node<'_, '_>, expected: &[&str]) -> bool {
    node.attributes().len() == expected.len()
        && node
            .attributes()
            .all(|a| a.namespace().is_none() && expected.contains(&a.name()))
}

const LABEL_DEFAULT_KEYS: [&str; 3] = ["LabelFont", "LabelSize", "LabelFace"];

fn label_default_owner(node: Node<'_, '_>) -> bool {
    ["CDXML", "n", "b", "t"]
        .iter()
        .any(|name| is_element(node, name))
}

fn profile_keys(node: Node<'_, '_>, expected: &[&str], profile: Profile) -> bool {
    if !matches!(profile, Profile::LabelDefaults) || !label_default_owner(node) {
        return keys(node, expected);
    }
    let mut expected = expected.to_vec();
    expected.extend(
        LABEL_DEFAULT_KEYS
            .iter()
            .copied()
            .filter(|name| node.attribute(*name).is_some()),
    );
    keys(node, &expected)
}

fn label_defaults_supported(xml: &Xml<'_>, resources: &Resources<'_, '_>) -> bool {
    let mut has_metadata = false;
    for node in xml.xml().descendants().filter(Node::is_element) {
        let has_own = LABEL_DEFAULT_KEYS
            .iter()
            .any(|name| node.attribute(*name).is_some());
        if !label_default_owner(node) {
            if has_own {
                return false;
            }
            continue;
        }
        has_metadata |= has_own;
        // Visiting every owner checks resources even when a nearer default shadows it.
        let Ok(defaults) = xml.label_defaults(node) else {
            return false;
        };
        if defaults.font().is_some_and(|font| {
            u16::try_from(font.value())
                .ok()
                .is_none_or(|id| resources.font(id).is_err())
        }) || defaults.face().is_some_and(|face| face.value() < 0)
        {
            return false;
        }
    }
    has_metadata
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

fn assigned(node: Node<'_, '_>) -> Result<u32> {
    node.attribute("id")
        .and_then(|id| id.parse::<ObjectId>().ok())
        .filter(|id| id.is_assigned())
        .map(ObjectId::get)
        .ok_or(DocumentError::UnsupportedEdit)
}

fn reference(node: Node<'_, '_>, key: &str) -> Result<u32> {
    node.attribute(key)
        .and_then(|id| id.parse::<ObjectId>().ok())
        .filter(|id| id.is_assigned())
        .map(ObjectId::get)
        .ok_or(DocumentError::UnsupportedEdit)
}

fn negative_underflow(raw: &str) -> bool {
    let raw = raw.trim();
    raw.starts_with('-')
        && raw.parse::<f64>().ok() == Some(0.0)
        && raw
            .split(['e', 'E'])
            .next()
            .is_some_and(|m| m.bytes().any(|b| matches!(b, b'1'..=b'9')))
}

struct BoundAtom<'doc, 'input: 'doc> {
    node: Node<'doc, 'input>,
    label: Option<TextObject<'doc, 'input>>,
}

fn bindings<'doc, 'input: 'doc>(
    xml: &'doc Xml<'input>,
    baseline: &Document,
    profile: Profile,
) -> Result<Vec<Vec<BoundAtom<'doc, 'input>>>> {
    if !model_supported_with_flags(
        baseline,
        matches!(profile, Profile::Flagged | Profile::LabelDefaults),
    ) {
        return unsupported();
    }
    let own = one_page(baseline)?;
    let has_unlabelled = own
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.atoms())
        .any(|atom| atom.authored_label.is_none());
    let supported = match profile {
        Profile::Boxed => !has_unlabelled && own.fragments().iter().all(|f| f.bonds().is_empty()),
        Profile::Connected => !has_unlabelled && connected_label_bonds(baseline, own),
        Profile::Mixed => connected_label_bonds(baseline, own) && mixed_hidden_supported(own),
        Profile::Flagged => flagged_page_supported(baseline, own),
        Profile::LabelDefaults => label_defaults_page_supported(baseline, own),
    };
    if !supported {
        return unsupported();
    }
    let features = xml.features();
    if features.doctype
        || features.processing_instructions
        || features.cdata
        || features.namespace_declarations
    {
        return unsupported();
    }
    let root = xml.xml().root_element();
    if !is_element(root, "CDXML") || !trivia(root) {
        return unsupported();
    }
    let root_attributes = write::root_attributes(baseline);
    let mut root_keys: Vec<_> = root_attributes
        .iter()
        .filter(|(name, _)| *name != "CreationProgram")
        .map(|(name, _)| *name)
        .collect();
    if root.attribute("CreationProgram").is_some() {
        root_keys.push("CreationProgram");
    }
    if !profile_keys(root, &root_keys, profile) {
        return unsupported();
    }
    for (name, value) in root_attributes
        .iter()
        .filter(|(name, _)| *name != "CreationProgram")
    {
        if root
            .attribute(*name)
            .and_then(|s| s.trim().parse::<f64>().ok())
            != value.parse::<f64>().ok()
        {
            return unsupported();
        }
    }
    let resources = xml
        .resources()
        .map_err(|_| DocumentError::UnsupportedEdit)?;
    if matches!(profile, Profile::LabelDefaults) && !label_defaults_supported(xml, &resources) {
        return unsupported();
    }
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
            "fonttable" if is_element(child, "fonttable") && keys(child, &[]) => {
                for font in child.children().filter(Node::is_element) {
                    if !is_element(font, "font")
                        || !keys(font, &["id", "name"])
                        || !trivia(font)
                        || font.children().any(|child| child.is_element())
                    {
                        return unsupported();
                    }
                }
            }
            "colortable" if is_element(child, "colortable") && keys(child, &[]) => {
                for color in child.children().filter(Node::is_element) {
                    if !is_element(color, "color")
                        || !keys(color, &["r", "g", "b"])
                        || !trivia(color)
                        || color.children().any(|child| child.is_element())
                        || ["r", "g", "b"]
                            .iter()
                            .any(|name| color.attribute(*name).is_some_and(negative_underflow))
                    {
                        return unsupported();
                    }
                }
            }
            "page" if is_element(child, "page") && page.replace(child).is_none() => {}
            _ => return unsupported(),
        }
    }
    let page = page.ok_or(DocumentError::UnsupportedEdit)?;
    if !keys(page, &["id", "DrawingSpace", "Width", "Height"])
        || page.attribute("DrawingSpace") != Some("Poster")
        || page.attribute("Width").and_then(|s| s.parse::<f64>().ok())
            != Some(own.size().width_pt())
        || page.attribute("Height").and_then(|s| s.parse::<f64>().ok())
            != Some(own.size().height_pt())
    {
        return unsupported();
    }
    assigned(page)?;
    let mut page_ids = BTreeSet::new();
    let mut fragments = Vec::new();
    let mut texts = Vec::new();
    for child in page.children().filter(Node::is_element) {
        if !trivia(child) || !page_ids.insert(assigned(child)?) {
            return unsupported();
        }
        if is_element(child, "fragment") {
            fragments.push(child);
        } else if is_element(child, "t") {
            texts.push(child);
        } else {
            return unsupported();
        }
    }
    let projected_captions = caption::project_with_resources(xml, &resources, page)?;
    if projected_captions.len() != own.texts().len()
        || !projected_captions
            .iter()
            .zip(own.texts())
            .all(|(a, b)| a == b.as_ref())
        || texts.len() != own.texts().len()
        || fragments.len() != own.fragments().len()
    {
        return unsupported();
    }
    for text in texts {
        if !profile_keys(text, &["id", "p", "Justification"], profile)
            || text.attribute("Justification") != Some("Left")
        {
            return unsupported();
        }
        let view = xml
            .styled_text(text)
            .map_err(|_| DocumentError::UnsupportedEdit)?;
        for run in view.runs() {
            if !keys(run.element(), &["font", "size", "face", "color"]) {
                return unsupported();
            }
        }
    }
    let mut bound = Vec::new();
    for (element, fragment) in fragments.into_iter().zip(own.fragments()) {
        if !keys(element, &["id"]) {
            return unsupported();
        }
        let mut source_atoms = Vec::new();
        let mut source_bonds = Vec::new();
        let mut ids = BTreeSet::new();
        for child in element.children().filter(Node::is_element) {
            if !trivia(child) || !ids.insert(assigned(child)?) {
                return unsupported();
            }
            if is_element(child, "n") {
                source_atoms.push(child);
            } else if is_element(child, "b") {
                source_bonds.push(child);
            } else {
                return unsupported();
            }
        }
        if source_atoms.len() != fragment.atoms().len()
            || source_bonds.len() != fragment.bonds().len()
        {
            return unsupported();
        }
        let mut atoms = Vec::new();
        let mut mapping = BTreeMap::new();
        for (node, atom) in source_atoms.into_iter().zip(fragment.atoms()) {
            let source_id = assigned(node)?;
            let expected = write::atom_attributes(atom, source_id)?;
            if !profile_keys(
                node,
                &expected.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
                profile,
            ) || node.attribute("NodeType") != Some("Element")
                || node
                    .attribute("p")
                    .map(parse_position)
                    .transpose()
                    .map_err(|_| DocumentError::UnsupportedEdit)?
                    != atom.position
            {
                return unsupported();
            }
            let mut parsed = graph::read_atom(node, atom.id)?;
            let label = if atom.authored_label.is_some() {
                let mut children = node.children().filter(Node::is_element);
                let text = children.next().ok_or(DocumentError::UnsupportedEdit)?;
                if children.next().is_some()
                    || !is_element(text, "t")
                    || !profile_keys(
                        text,
                        &["id", "BoundingBox", "Justification", "LabelJustification"],
                        profile,
                    )
                    || text.attribute("Justification") != Some("Left")
                    || text.attribute("LabelJustification") != Some("Left")
                {
                    return unsupported();
                }
                assigned(text)?;
                let view = xml
                    .styled_text(text)
                    .map_err(|_| DocumentError::UnsupportedEdit)?;
                if view.runs().len() != 1
                    || !view
                        .runs()
                        .first()
                        .is_some_and(|run| keys(run.element(), &["font", "size", "face", "color"]))
                {
                    return unsupported();
                }
                parsed.authored_label = Some(Arc::new(atom_text::project(xml, &resources, node)?));
                Some(view)
            } else {
                if node.children().any(|child| child.is_element()) {
                    return unsupported();
                }
                None
            };
            if &parsed != atom {
                return unsupported();
            }
            mapping.insert(source_id, atom.id);
            atoms.push(BoundAtom { node, label });
        }
        for (node, bond) in source_bonds.into_iter().zip(fragment.bonds()) {
            if node.children().any(|child| child.is_element()) {
                return unsupported();
            }
            let source_id = assigned(node)?;
            let begin = reference(node, "B")?;
            let end = reference(node, "E")?;
            let expected = write::bond_attributes(bond, source_id, begin, end);
            let mut expected_keys: Vec<_> = expected.iter().map(|(name, _)| *name).collect();
            if matches!(
                profile,
                Profile::Connected | Profile::Mixed | Profile::Flagged | Profile::LabelDefaults
            ) {
                for name in ["Display", "Display2", "color"] {
                    if node.attribute(name).is_some() {
                        expected_keys.push(name);
                    }
                }
                if ["Display", "Display2"]
                    .iter()
                    .any(|name| node.attribute(*name).is_some_and(|value| value != "Solid"))
                {
                    return unsupported();
                }
                let color = match node.attribute("color") {
                    None => 3,
                    Some(raw) => parse_integer(raw)
                        .ok()
                        .and_then(|value| u16::try_from(value).ok())
                        .ok_or(DocumentError::UnsupportedEdit)?,
                };
                if resources
                    .color(color)
                    .map_err(|_| DocumentError::UnsupportedEdit)?
                    != [0.0; 3]
                {
                    return unsupported();
                }
            }
            if !profile_keys(node, &expected_keys, profile) {
                return unsupported();
            }
            let mapped = Bond {
                id: bond.id,
                begin: *mapping.get(&begin).ok_or(DocumentError::UnsupportedEdit)?,
                end: *mapping.get(&end).ok_or(DocumentError::UnsupportedEdit)?,
                order: graph::read_order(node.attribute("Order"))?,
                double_position: graph::read_double_position(node.attribute("DoublePosition"))?,
            };
            if &mapped != bond {
                return unsupported();
            }
        }
        bound.push(atoms);
    }
    Ok(bound)
}

pub(crate) fn is_boxed(source: &[u8], baseline: &Document) -> bool {
    is_profile(source, baseline, Profile::Boxed)
}

pub(crate) fn is_connected(source: &[u8], baseline: &Document) -> bool {
    is_profile(source, baseline, Profile::Connected)
}

pub(crate) fn is_mixed(source: &[u8], baseline: &Document) -> bool {
    is_profile(source, baseline, Profile::Mixed)
}

pub(crate) fn is_flagged(source: &[u8], baseline: &Document) -> bool {
    is_profile(source, baseline, Profile::Flagged)
}

pub(crate) fn is_label_defaults(source: &[u8], baseline: &Document) -> bool {
    is_profile(source, baseline, Profile::LabelDefaults)
}

fn is_profile(source: &[u8], baseline: &Document, profile: Profile) -> bool {
    let Ok(xml) = Xml::parse(source) else {
        return false;
    };
    bindings(&xml, baseline, profile).is_ok()
}

pub(crate) fn write_preserved(
    source: &[u8],
    baseline: &Document,
    current: &Document,
) -> Result<Vec<u8>> {
    write_in_profile(source, baseline, current, Profile::Boxed)
}

pub(crate) fn write_connected_preserved(
    source: &[u8],
    baseline: &Document,
    current: &Document,
) -> Result<Vec<u8>> {
    write_in_profile(source, baseline, current, Profile::Connected)
}

pub(crate) fn write_mixed_preserved(
    source: &[u8],
    baseline: &Document,
    current: &Document,
) -> Result<Vec<u8>> {
    write_in_profile(source, baseline, current, Profile::Mixed)
}

pub(crate) fn write_flagged_preserved(
    source: &[u8],
    baseline: &Document,
    current: &Document,
) -> Result<Vec<u8>> {
    write_in_profile(source, baseline, current, Profile::Flagged)
}

pub(crate) fn write_label_defaults_preserved(
    source: &[u8],
    baseline: &Document,
    current: &Document,
) -> Result<Vec<u8>> {
    write_in_profile(source, baseline, current, Profile::LabelDefaults)
}

fn creation_ids(
    xml: &Xml<'_>,
    before: &Page,
    bound: &[Vec<BoundAtom<'_, '_>>],
) -> Result<BTreeMap<AtomId, u32>> {
    let mut used = BTreeSet::new();
    for node in xml
        .xml()
        .root_element()
        .descendants()
        .filter(Node::is_element)
    {
        if node.attribute("id").is_some() && !is_element(node, "font") {
            used.insert(assigned(node)?);
        }
    }
    let mut next = 1_u32;
    let mut ids = BTreeMap::new();
    for (fragment, nodes) in before.fragments().iter().zip(bound) {
        for (atom, _) in fragment.atoms().iter().zip(nodes) {
            if atom.authored_label.is_none() {
                while used.contains(&next) {
                    next = next.checked_add(1).ok_or(DocumentError::UnsupportedEdit)?;
                }
                ids.insert(atom.id, next);
                used.insert(next);
            }
        }
    }
    Ok(ids)
}

fn label_xml(label: &AtomLabel, id: u32, resources: &Resources<'_, '_>) -> Result<String> {
    let (font, color) = run_resources(label, resources)?;
    let [l, t, r, b] = label.bounding_box().ok_or(DocumentError::UnsupportedEdit)?;
    let run = label.runs().first().ok_or(DocumentError::UnsupportedEdit)?;
    let mut writer = syntax(Writer::new(&[]))?;
    syntax(writer.start_element(
        "t",
        &[
            ("id", &id.to_string()),
            ("BoundingBox", &format!("{l} {t} {r} {b}")),
            ("Justification", "Left"),
            ("LabelJustification", "Left"),
        ],
    ))?;
    syntax(writer.start_element(
        "s",
        &[
            ("font", &font.to_string()),
            ("size", &run.style().size_pt().to_string()),
            ("face", "0"),
            ("color", &color.to_string()),
        ],
    ))?;
    syntax(writer.text(run.text()))?;
    syntax(writer.end_element())?;
    syntax(writer.end_element())?;
    let bytes = syntax(writer.finish())?;
    let parsed = syntax(Xml::parse(&bytes))?;
    let child = parsed
        .xml()
        .root_element()
        .first_element_child()
        .ok_or(DocumentError::UnsupportedEdit)?;
    parsed
        .xml()
        .input_text()
        .get(child.range())
        .map(str::to_owned)
        .ok_or(DocumentError::UnsupportedEdit)
}

fn write_in_profile(
    source: &[u8],
    baseline: &Document,
    current: &Document,
    profile: Profile,
) -> Result<Vec<u8>> {
    let supported = match profile {
        Profile::Boxed => supports_delta(baseline, current),
        Profile::Connected => supports_connected_delta(baseline, current),
        Profile::Mixed => {
            supports_mixed_delta(baseline, current) || flagged_model_delta(baseline, current)
        }
        Profile::Flagged => flagged_model_delta(baseline, current),
        Profile::LabelDefaults => supports_label_defaults_delta(source, baseline, current),
    };
    if !supported {
        return unsupported();
    }
    let xml = syntax(Xml::parse(source))?;
    let bound = bindings(&xml, baseline, profile)?;
    if baseline == current {
        return Ok(source.to_vec());
    }
    let before = one_page(baseline)?;
    let after = one_page(current)?;
    let resources = xml
        .resources()
        .map_err(|_| DocumentError::UnsupportedEdit)?;
    let ids = creation_ids(&xml, before, &bound)?;
    let mut appends = Vec::new();
    let mut values = Vec::new();
    for ((old, new), nodes) in before.fragments().iter().zip(after.fragments()).zip(&bound) {
        for ((a, b), binding) in old.atoms().iter().zip(new.atoms()).zip(nodes) {
            if a.authored_label.is_none()
                && let Some(label) = &b.authored_label
            {
                let id = ids.get(&a.id).ok_or(DocumentError::UnsupportedEdit)?;
                appends.push((binding.node, label_xml(label, *id, &resources)?));
            }
            if a.position != b.position {
                let point = b.position.ok_or(DocumentError::UnsupportedEdit)?;
                values.push((
                    binding.node,
                    "p",
                    format!("{} {}", point.x_pt(), point.y_pt()),
                ));
            }
            if a.element != b.element {
                values.push((binding.node, "Element", b.element.get().to_string()));
            }
            if a.explicit_hydrogens != b.explicit_hydrogens {
                values.push((
                    binding.node,
                    "NumHydrogens",
                    b.explicit_hydrogens
                        .ok_or(DocumentError::UnsupportedEdit)?
                        .to_string(),
                ));
            }
            if let (Some(old), Some(new), Some(text)) =
                (&a.authored_label, &b.authored_label, &binding.label)
                && old.bounding_box() != new.bounding_box()
            {
                let [l, t, r, b] = new.bounding_box().ok_or(DocumentError::UnsupportedEdit)?;
                values.push((text.element(), "BoundingBox", format!("{l} {t} {r} {b}")));
            }
        }
    }
    let mut edits: Vec<_> = values
        .iter()
        .map(|(element, name, value)| Edit::SetAttribute {
            element: *element,
            name,
            value,
        })
        .collect();
    edits.extend(appends.iter().map(|(element, xml)| Edit::AppendChildren {
        element: *element,
        xml,
    }));
    for ((old, new), nodes) in before.fragments().iter().zip(after.fragments()).zip(&bound) {
        for ((a, b), binding) in old.atoms().iter().zip(new.atoms()).zip(nodes) {
            if let (Some(old), Some(new), Some(text)) =
                (&a.authored_label, &b.authored_label, &binding.label)
            {
                let old = old.runs().first().ok_or(DocumentError::UnsupportedEdit)?;
                let new = new.runs().first().ok_or(DocumentError::UnsupportedEdit)?;
                if old.text() != new.text() {
                    edits.push(Edit::SetText {
                        element: text
                            .runs()
                            .first()
                            .ok_or(DocumentError::UnsupportedEdit)?
                            .element(),
                        value: new.text(),
                    });
                }
            }
        }
    }
    let bytes = syntax(xml.apply_edits(&edits))?;
    let reopened = StoredDocument::from_cdxml(bytes.clone().into(), after.size())?;
    let source_supported = match profile {
        Profile::Boxed => reopened.has_boxed_atom_labels(),
        // Fully equal re-import may become byte-exact native after owned lexical edits.
        Profile::Connected => {
            reopened.has_connected_atom_labels() || reopened.has_boxed_atom_labels()
        }
        Profile::Mixed => {
            reopened.has_mixed_atom_labels()
                || reopened.has_flagged_atom_labels()
                || reopened.has_boxed_atom_labels()
        }
        Profile::Flagged => reopened.has_flagged_atom_labels(),
        Profile::LabelDefaults => reopened.has_label_defaults_projection(),
    };
    if reopened.model() != current || !source_supported {
        return unsupported();
    }
    Ok(bytes)
}
