//! Closed native-elemental source profile. Chemical and file authority remain above syntax.
use super::{ElementalGraph, MappingError, write};
use crate::DocumentError;
use bondline_cdxml::{Document as Xml, Edit, MAX_EDITS, MAX_INPUT_BYTES, ObjectId, Writer};
use bondline_model::{Atom, AtomId, Document, Fragment, Page};
use roxmltree::Node;
use std::collections::{BTreeMap, BTreeSet};

type Result<T> = std::result::Result<T, DocumentError>;

fn syntax<T>(result: std::result::Result<T, bondline_cdxml::Error>) -> Result<T> {
    result
        .map_err(MappingError::from)
        .map_err(DocumentError::from)
}

fn unsupported<T>() -> Result<T> {
    Err(DocumentError::UnsupportedEdit)
}

fn one_page(model: &Document) -> Result<&Page> {
    if model.pages().len() != 1 {
        return unsupported();
    }
    model.pages().first().ok_or(DocumentError::UnsupportedEdit)
}

pub(crate) fn supports_delta(baseline: &Document, current: &Document) -> bool {
    if baseline.has_authored_atom_labels() || current.has_authored_atom_labels() {
        return false;
    }
    let (Ok(before), Ok(after)) = (one_page(baseline), one_page(current)) else {
        return false;
    };
    if before.id() != after.id() || before.fragments().len() > after.fragments().len() {
        return false;
    }
    if before.texts() != after.texts() {
        return false;
    }
    if after
        .fragments()
        .iter()
        .flat_map(|f| f.atoms())
        .any(|a| a.isotope.is_some_and(|value| value.get() > i16::MAX as u16))
    {
        return false;
    }
    before
        .fragments()
        .iter()
        .zip(after.fragments())
        .all(|(old, new)| {
            old.id() == new.id()
                && old.atoms().len() <= new.atoms().len()
                && old.bonds().len() <= new.bonds().len()
                && old
                    .atoms()
                    .iter()
                    .zip(new.atoms())
                    .all(|(a, b)| a.id == b.id)
                && old
                    .bonds()
                    .iter()
                    .zip(new.bonds())
                    .all(|(a, b)| a.id == b.id && a.begin == b.begin && a.end == b.end)
        })
}

pub(crate) fn is_preserved(source: &[u8], baseline: &Document) -> bool {
    let Ok(xml) = Xml::parse(source) else {
        return false;
    };
    bindings(&xml, baseline)
        .and_then(|_| certify_projection(source, baseline))
        .is_ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FragmentIds {
    fragment: u32,
    atoms: Vec<u32>,
    bonds: Vec<u32>,
}

struct BoundFragment<'doc, 'input: 'doc> {
    node: Node<'doc, 'input>,
    atoms: Vec<Node<'doc, 'input>>,
    bonds: Vec<Node<'doc, 'input>>,
    ids: FragmentIds,
}

struct Bindings<'doc, 'input: 'doc> {
    page: Node<'doc, 'input>,
    page_id: u32,
    fragments: Vec<BoundFragment<'doc, 'input>>,
}

fn assigned(node: Node<'_, '_>) -> Result<u32> {
    let id = node
        .attribute("id")
        .and_then(|value| value.parse::<ObjectId>().ok())
        .filter(|id| id.is_assigned())
        .ok_or(DocumentError::UnsupportedEdit)?;
    Ok(id.get())
}

fn keys(node: Node<'_, '_>, attributes: &write::Attributes) -> bool {
    node.attributes().len() == attributes.len()
        && node.attributes().all(|attribute| {
            attribute.namespace().is_none()
                && attributes.iter().any(|(name, _)| *name == attribute.name())
        })
}

fn bindings<'doc, 'input: 'doc>(
    xml: &'doc Xml<'input>,
    model: &Document,
) -> Result<Bindings<'doc, 'input>> {
    let features = xml.features();
    if features.doctype
        || features.processing_instructions
        || features.cdata
        || features.namespace_declarations
    {
        return unsupported();
    }
    for node in xml.xml().descendants() {
        if node.is_element() && node.tag_name().namespace().is_some() {
            return unsupported();
        }
        if node.is_text()
            && node.text().is_some_and(|text| {
                !text
                    .bytes()
                    .all(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
            })
        {
            return unsupported();
        }
    }
    let page_model = one_page(model)?;
    let root = xml.xml().root_element();
    if !keys(root, &write::root_attributes(model))
        || root.attribute("CreationProgram")
            != Some(concat!("Bondline ", env!("CARGO_PKG_VERSION")))
    {
        return unsupported();
    }
    let mut pages = root.children().filter(|node| node.is_element());
    let page = pages.next().ok_or(DocumentError::UnsupportedEdit)?;
    if pages.next().is_some()
        || page.tag_name().name() != "page"
        || page.attribute("DrawingSpace") != Some("Poster")
    {
        return unsupported();
    }
    let page_id = assigned(page)?;
    if !keys(page, &write::page_attributes(page_model, page_id)) {
        return unsupported();
    }
    let source_fragments: Vec<_> = page.children().filter(|n| n.is_element()).collect();
    if source_fragments.len() != page_model.fragments().len() {
        return unsupported();
    }
    let mut fragments = Vec::with_capacity(source_fragments.len());
    for (node, fragment) in source_fragments.into_iter().zip(page_model.fragments()) {
        let id = assigned(node)?;
        if node.tag_name().name() != "fragment" || !keys(node, &vec![("id", id.to_string())]) {
            return unsupported();
        }
        let mut atoms = Vec::new();
        let mut bonds = Vec::new();
        for child in node.children().filter(|n| n.is_element()) {
            if child.children().any(|n| n.is_element()) {
                return unsupported();
            }
            match child.tag_name().name() {
                "n" => atoms.push(child),
                "b" => bonds.push(child),
                _ => return unsupported(),
            }
        }
        if atoms.len() != fragment.atoms().len() || bonds.len() != fragment.bonds().len() {
            return unsupported();
        }
        let mut atom_ids = Vec::with_capacity(atoms.len());
        let mut atom_map = BTreeMap::new();
        for (node, atom) in atoms.iter().zip(fragment.atoms()) {
            let id = assigned(*node)?;
            if node.attribute("NodeType") != Some("Element")
                || !keys(*node, &write::atom_attributes(atom, id)?)
            {
                return unsupported();
            }
            atom_ids.push(id);
            atom_map.insert(atom.id, id);
        }
        let mut bond_ids = Vec::with_capacity(bonds.len());
        for (node, bond) in bonds.iter().zip(fragment.bonds()) {
            let id = assigned(*node)?;
            let begin = *atom_map
                .get(&bond.begin)
                .ok_or(DocumentError::UnsupportedEdit)?;
            let end = *atom_map
                .get(&bond.end)
                .ok_or(DocumentError::UnsupportedEdit)?;
            if !keys(*node, &write::bond_attributes(bond, id, begin, end)) {
                return unsupported();
            }
            bond_ids.push(id);
        }
        fragments.push(BoundFragment {
            node,
            atoms,
            bonds,
            ids: FragmentIds {
                fragment: id,
                atoms: atom_ids,
                bonds: bond_ids,
            },
        });
    }
    Ok(Bindings {
        page,
        page_id,
        fragments,
    })
}

// Rebind the fresh projection through array correspondence; current model IDs may be sparse
// or burned by Undo branches. Comparing a dense re-import directly would be incorrect.
fn certify_projection(source: &[u8], expected: &Document) -> Result<()> {
    let page = one_page(expected)?;
    let graph = ElementalGraph::parse(source)?;
    let projected = graph.preview_document(page.size())?;
    let actual = one_page(&projected)?;
    if projected.drawing_settings() != expected.drawing_settings()
        || projected.drawing_style() != expected.drawing_style()
        || actual.size() != page.size()
        || actual.fragments().len() != page.fragments().len()
    {
        return unsupported();
    }
    for (actual, expected) in actual.fragments().iter().zip(page.fragments()) {
        if actual.atoms().len() != expected.atoms().len()
            || actual.bonds().len() != expected.bonds().len()
        {
            return unsupported();
        }
        let mut rebind = BTreeMap::new();
        for (actual, expected) in actual.atoms().iter().zip(expected.atoms()) {
            let mut atom = actual.clone();
            atom.id = expected.id;
            if atom != *expected {
                return unsupported();
            }
            rebind.insert(actual.id, expected.id);
        }
        for (actual, expected) in actual.bonds().iter().zip(expected.bonds()) {
            let mut bond = *actual;
            bond.id = expected.id;
            bond.begin = *rebind
                .get(&actual.begin)
                .ok_or(DocumentError::UnsupportedEdit)?;
            bond.end = *rebind
                .get(&actual.end)
                .ok_or(DocumentError::UnsupportedEdit)?;
            if bond != *expected {
                return unsupported();
            }
        }
    }
    Ok(())
}

struct Ids {
    used: BTreeSet<u32>,
    next: u32,
}

impl Ids {
    fn new(bound: &Bindings<'_, '_>) -> Self {
        let mut used = BTreeSet::from([bound.page_id]);
        for fragment in &bound.fragments {
            used.insert(fragment.ids.fragment);
            used.extend(fragment.ids.atoms.iter().copied());
            used.extend(fragment.ids.bonds.iter().copied());
        }
        Self { used, next: 1 }
    }

    fn allocate(&mut self) -> Result<u32> {
        while self.used.contains(&self.next) {
            self.next = self
                .next
                .checked_add(1)
                .ok_or(MappingError::IdentityLimit)?;
        }
        let id = self.next;
        self.used.insert(id);
        Ok(id)
    }
}

enum Change<'doc, 'input: 'doc> {
    Attribute {
        node: Node<'doc, 'input>,
        name: &'static str,
        value: Option<String>,
    },
    Children {
        node: Node<'doc, 'input>,
        xml: String,
    },
}

struct Changes<'doc, 'input: 'doc> {
    entries: Vec<Change<'doc, 'input>>,
    inserted: usize,
}

impl<'doc, 'input: 'doc> Changes<'doc, 'input> {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            inserted: 0,
        }
    }

    fn push(&mut self, change: Change<'doc, 'input>) -> Result<()> {
        if self.entries.len() >= MAX_EDITS {
            return syntax(Err(bondline_cdxml::Error::EditLimit));
        }
        let bytes = match &change {
            Change::Attribute {
                name,
                value: Some(value),
                ..
            } => value
                .len()
                .checked_add(name.len())
                .and_then(|length| length.checked_add(4)),
            Change::Attribute { value: None, .. } => Some(0),
            Change::Children { xml, .. } => Some(xml.len()),
        }
        .ok_or(MappingError::Syntax(bondline_cdxml::Error::InputTooLarge))?;
        self.inserted = self
            .inserted
            .checked_add(bytes)
            .filter(|&length| length <= MAX_INPUT_BYTES)
            .ok_or(MappingError::Syntax(bondline_cdxml::Error::InputTooLarge))?;
        self.entries.push(change);
        Ok(())
    }

    fn attributes(
        &mut self,
        node: Node<'doc, 'input>,
        attributes: &write::Attributes,
        fields: &[(&'static str, bool)],
    ) -> Result<()> {
        for &(name, changed) in fields {
            if changed {
                let value = attributes
                    .iter()
                    .find(|(key, _)| *key == name)
                    .map(|(_, value)| value.clone());
                self.push(Change::Attribute { node, name, value })?;
            }
        }
        Ok(())
    }

    fn append(&mut self, node: Node<'doc, 'input>, xml: String) -> Result<()> {
        if !xml.is_empty() {
            self.push(Change::Children { node, xml })?;
        }
        Ok(())
    }

    fn apply(&self, document: &Xml<'_>) -> Result<Vec<u8>> {
        let edits: Vec<_> = self
            .entries
            .iter()
            .map(|change| match change {
                Change::Attribute {
                    node,
                    name,
                    value: Some(value),
                } => Edit::SetAttribute {
                    element: *node,
                    name,
                    value,
                },
                Change::Attribute {
                    node,
                    name,
                    value: None,
                } => Edit::RemoveAttribute {
                    element: *node,
                    name,
                },
                Change::Children { node, xml } => Edit::AppendChildren {
                    element: *node,
                    xml,
                },
            })
            .collect();
        syntax(document.apply_edits(&edits))
    }
}

fn atom_fields(old: &Atom, new: &Atom) -> [(&'static str, bool); 8] {
    [
        ("Element", old.element != new.element),
        ("Charge", old.charge != new.charge),
        ("Radical", old.radical != new.radical),
        ("p", old.position != new.position),
        ("Isotope", old.isotope != new.isotope),
        (
            "NumHydrogens",
            old.explicit_hydrogens != new.explicit_hydrogens,
        ),
        (
            "ShowTerminalCarbonLabels",
            old.show_terminal_carbon_labels != new.show_terminal_carbon_labels,
        ),
        (
            "ShowNonTerminalCarbonLabels",
            old.show_non_terminal_carbon_labels != new.show_non_terminal_carbon_labels,
        ),
    ]
}

fn snippet(emit: impl FnOnce(&mut Writer) -> Result<()>) -> Result<String> {
    let mut writer = syntax(Writer::new(&[]))?;
    emit(&mut writer)?;
    let bytes = syntax(writer.finish())?;
    let parsed = syntax(Xml::parse(&bytes))?;
    let root = parsed.xml().root_element();
    match (root.first_element_child(), root.last_element_child()) {
        (Some(first), Some(last)) => parsed
            .xml()
            .input_text()
            .get(first.range().start..last.range().end)
            .map(str::to_owned)
            .ok_or(DocumentError::UnsupportedEdit),
        (None, None) => Ok(String::new()),
        _ => unsupported(),
    }
}

fn emit_graph(
    writer: &mut Writer,
    fragment: &Fragment,
    ids: &FragmentIds,
    start: [usize; 2],
) -> Result<()> {
    if fragment.atoms().len() != ids.atoms.len() || fragment.bonds().len() != ids.bonds.len() {
        return unsupported();
    }
    let map: BTreeMap<AtomId, u32> = fragment
        .atoms()
        .iter()
        .zip(&ids.atoms)
        .map(|(atom, id)| (atom.id, *id))
        .collect();
    for (atom, id) in fragment.atoms().iter().zip(&ids.atoms).skip(start[0]) {
        write::write_atom(writer, atom, *id)?;
    }
    for (bond, id) in fragment.bonds().iter().zip(&ids.bonds).skip(start[1]) {
        let begin = *map.get(&bond.begin).ok_or(DocumentError::UnsupportedEdit)?;
        let end = *map.get(&bond.end).ok_or(DocumentError::UnsupportedEdit)?;
        write::write_bond(writer, bond, *id, begin, end)?;
    }
    Ok(())
}

pub(crate) fn write_preserved(
    source: &[u8],
    baseline: &Document,
    current: &Document,
) -> Result<Vec<u8>> {
    if !supports_delta(baseline, current) {
        return unsupported();
    }
    let parsed = syntax(Xml::parse(source))?;
    let bound = bindings(&parsed, baseline)?;
    certify_projection(source, baseline)?;
    if baseline == current {
        return Ok(source.to_vec());
    }
    let before = one_page(baseline)?;
    let after = one_page(current)?;
    let mut plan = Changes::new();
    let old = baseline.drawing_settings();
    let new = current.drawing_settings();
    plan.attributes(
        parsed.xml().root_element(),
        &write::root_attributes(current),
        &[
            ("BondLength", old.bond_length_pt() != new.bond_length_pt()),
            (
                "ChainAngle",
                old.chain_angle_degrees() != new.chain_angle_degrees(),
            ),
            (
                "LineWidth",
                baseline.drawing_style().line_width_pt() != current.drawing_style().line_width_pt(),
            ),
            (
                "BondSpacing",
                baseline.drawing_style().bond_spacing_percent()
                    != current.drawing_style().bond_spacing_percent(),
            ),
            (
                "MarginWidth",
                baseline.drawing_style().margin_width_pt()
                    != current.drawing_style().margin_width_pt(),
            ),
        ],
    )?;
    plan.attributes(
        bound.page,
        &write::page_attributes(after, bound.page_id),
        &[
            ("Width", before.size().width_pt() != after.size().width_pt()),
            (
                "Height",
                before.size().height_pt() != after.size().height_pt(),
            ),
        ],
    )?;
    let mut allocator = Ids::new(&bound);
    let mut expected_ids = Vec::with_capacity(after.fragments().len());
    for (index, fragment) in after.fragments().iter().enumerate() {
        if let Some(old) = before.fragments().get(index) {
            let binding = bound
                .fragments
                .get(index)
                .ok_or(DocumentError::UnsupportedEdit)?;
            let mut ids = binding.ids.clone();
            for _ in old.atoms().len()..fragment.atoms().len() {
                ids.atoms.push(allocator.allocate()?);
            }
            for _ in old.bonds().len()..fragment.bonds().len() {
                ids.bonds.push(allocator.allocate()?);
            }
            for ((node, old), new) in binding.atoms.iter().zip(old.atoms()).zip(fragment.atoms()) {
                plan.attributes(
                    *node,
                    &write::atom_attributes(new, assigned(*node)?)?,
                    &atom_fields(old, new),
                )?;
            }
            let source_atoms: BTreeMap<AtomId, u32> = fragment
                .atoms()
                .iter()
                .zip(&ids.atoms)
                .map(|(atom, id)| (atom.id, *id))
                .collect();
            for ((node, old), new) in binding.bonds.iter().zip(old.bonds()).zip(fragment.bonds()) {
                // Only these two fields can change; original B/E and ID lexemes are untouched.
                let begin = *source_atoms
                    .get(&new.begin)
                    .ok_or(DocumentError::UnsupportedEdit)?;
                let end = *source_atoms
                    .get(&new.end)
                    .ok_or(DocumentError::UnsupportedEdit)?;
                let attributes = write::bond_attributes(new, assigned(*node)?, begin, end);
                plan.attributes(
                    *node,
                    &attributes,
                    &[
                        ("Order", old.order != new.order),
                        ("DoublePosition", old.double_position != new.double_position),
                    ],
                )?;
            }
            if old.atoms().len() != fragment.atoms().len()
                || old.bonds().len() != fragment.bonds().len()
            {
                let xml = snippet(|writer| {
                    emit_graph(
                        writer,
                        fragment,
                        &ids,
                        [old.atoms().len(), old.bonds().len()],
                    )
                })?;
                plan.append(binding.node, xml)?;
            }
            expected_ids.push(ids);
        } else {
            let fragment_id = allocator.allocate()?;
            let mut atoms = Vec::with_capacity(fragment.atoms().len());
            let mut bonds = Vec::with_capacity(fragment.bonds().len());
            for _ in fragment.atoms() {
                atoms.push(allocator.allocate()?);
            }
            for _ in fragment.bonds() {
                bonds.push(allocator.allocate()?);
            }
            expected_ids.push(FragmentIds {
                fragment: fragment_id,
                atoms,
                bonds,
            });
        }
    }
    if after.fragments().len() > before.fragments().len() {
        let xml = snippet(|writer| {
            for (fragment, ids) in after
                .fragments()
                .iter()
                .zip(&expected_ids)
                .skip(before.fragments().len())
            {
                syntax(writer.start_element("fragment", &[("id", &ids.fragment.to_string())]))?;
                emit_graph(writer, fragment, ids, [0, 0])?;
                syntax(writer.end_element())?;
            }
            Ok(())
        })?;
        plan.append(bound.page, xml)?;
    }
    let output = plan.apply(&parsed)?;
    let written = syntax(Xml::parse(&output))?;
    let actual = bindings(&written, current)?;
    if actual.page_id != bound.page_id
        || actual.fragments.len() != expected_ids.len()
        || actual
            .fragments
            .iter()
            .zip(&expected_ids)
            .any(|(actual, expected)| actual.ids != *expected)
    {
        return unsupported();
    }
    certify_projection(&output, current)?;
    Ok(output)
}
