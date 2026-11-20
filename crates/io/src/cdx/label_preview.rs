//! Whole-source paper, resource and ink checks for labelled and skeletal CDX pages.
use super::{
    BEGIN, BOND, BOND_LENGTH, BOUNDING_BOX, CHAIN_ANGLE, CHARGE, COLOR_TABLE, DISPLAY, DISPLAY2,
    DOCUMENT, DOUBLE_POSITION, DRAWING_SPACE, ELEMENT, END, ElementalGraph, FONT_TABLE, FRAGMENT,
    HEIGHT, HYDROGENS, ISOTOPE, NODE, NODE_TYPE, Nodes, ORDER, PAGE, POSITION, RADICAL,
    SHOW_NON_TERMINAL_CARBONS, SHOW_TERMINAL_CARBONS, Scope, TEXT, TEXT_OBJECT, WIDTH,
    atom_label::{JUSTIFICATION, LABEL_JUSTIFICATION},
    coordinate, has_preview_header, object, payload, poster_size, project_fragments,
    verify_preview_page,
};
use bondline_cdx::{Document, RecordKind, Resources};
use bondline_model::{DrawingSettings, DrawingStyle, Fragment};
use std::collections::BTreeMap;

const BOND_SPACING: u16 = 0x0804;
const LINE_WIDTH: u16 = 0x0807;
const MARGIN_WIDTH: u16 = 0x0808;
type Fields = BTreeMap<u16, usize>;

pub(super) fn preview(graph: &ElementalGraph<'_>) -> Option<bondline_model::Document> {
    let model = model(graph.source(), graph.fragments(), Scope::All, None)?;
    verify_preview_page(&model)?;
    Some(model)
}

pub(super) fn model(
    source: &Document<'_>,
    fragments: &[Fragment],
    scope: Scope,
    resources: Option<&Resources<'_, '_>>,
) -> Option<bondline_model::Document> {
    if !has_preview_header(source) {
        return None;
    }
    let (mut atoms, mut bonds) = (0_usize, 0_usize);
    for fragment in fragments {
        atoms = atoms.saturating_add(fragment.atoms().len());
        bonds = bonds.saturating_add(fragment.bonds().len());
        if atoms.saturating_add(bonds) > bondline_render::MAX_HIT_REGIONS
            || fragment
                .atoms()
                .iter()
                .any(|atom| atom.authored_label.is_none())
        {
            return None;
        }
    }
    if atoms == 0 {
        return None;
    }
    let fields = source_fields(source, scope, Nodes::Labelled)?;
    paper_model(source, fragments, fields, resources, bonds)
}

pub(super) fn skeletal(source: &Document<'_>) -> Option<bondline_model::Document> {
    if !has_preview_header(source) {
        return None;
    }
    let fields = source_fields(source, Scope::All, Nodes::Skeletal)?;
    if !fields.0.contains_key(&LINE_WIDTH) || !fields.0.contains_key(&MARGIN_WIDTH) {
        return None;
    }
    let fragments = ElementalGraph::read_fragments(source, Nodes::Skeletal).ok()?;
    let bonds = fragments.iter().map(|f| f.bonds().len()).sum();
    if bonds == 0 {
        return None;
    }
    let model = paper_model(source, &fragments, fields, None, bonds)?;
    crate::display::certify_cdx_skeletal_page(&model).then_some(model)
}

fn paper_model(
    source: &Document<'_>,
    fragments: &[Fragment],
    (root_fields, page_fields): (Fields, Fields),
    resources: Option<&Resources<'_, '_>>,
    bonds: usize,
) -> Option<bondline_model::Document> {
    let size = poster_size(source, &page_fields)?;
    let owned;
    let resources = match resources {
        Some(resources) => resources,
        None => {
            owned = source.resources().ok()?;
            &owned
        }
    };
    if resources.color(2).ok()? != [u16::MAX; 3]
        || (bonds > 0 && resources.color(3).ok()? != [0; 3])
    {
        return None;
    }
    let settings = DrawingSettings::new(
        recorded(source, &root_fields, BOND_LENGTH)?.map(coordinate),
        recorded(source, &root_fields, CHAIN_ANGLE)?.map(coordinate),
    )
    .ok()?;
    let spacing = recorded(source, &root_fields, BOND_SPACING)?
        .map(|raw| f64::from(i16::from_le_bytes(raw)) / 10.0);
    let style = DrawingStyle::new(recorded(source, &root_fields, LINE_WIDTH)?.map(coordinate))
        .ok()?
        .with_bond_spacing_percent(spacing)
        .ok()?
        .with_margin_width(recorded(source, &root_fields, MARGIN_WIDTH)?.map(coordinate))
        .ok()?;
    let mut model = project_fragments(source, fragments, size).ok()?;
    if model.drawing_settings() != settings {
        return None;
    }
    model.set_drawing_style(style);
    if bonds > 0 && !crate::display::connected_label_bonds(&model, model.pages().first()?) {
        return None;
    }
    Some(model)
}

fn source_fields(source: &Document<'_>, scope: Scope, nodes: Nodes) -> Option<(Fields, Fields)> {
    let root = source.framing().has_encoded_document().then_some(0);
    let mut page = None;
    let mut hits = 0_usize;
    let (mut root_fields, mut page_fields) = (Fields::new(), Fields::new());
    for (index, record) in scope.records(source) {
        let owner = record.parent();
        let kind = object(source, owner).map(|(tag, _)| tag);
        match *record.kind() {
            RecordKind::ObjectStart { tag, .. } => match tag {
                DOCUMENT if root == Some(index) => {}
                PAGE if owner == root && page.is_none() => page = Some(index),
                FRAGMENT if page.is_some() && page == owner => {}
                NODE | BOND if kind == Some(FRAGMENT) => {
                    hits += 1;
                    if hits > bondline_render::MAX_HIT_REGIONS {
                        return None;
                    }
                }
                TEXT_OBJECT if nodes == Nodes::Labelled && kind == Some(NODE) => {}
                _ => return None,
            },
            RecordKind::Property { tag, .. } => {
                let fields = if owner == root
                    && (matches!(tag, FONT_TABLE | COLOR_TABLE) || is_drawing_property(tag))
                {
                    &mut root_fields
                } else if page.is_some()
                    && page == owner
                    && matches!(tag, DRAWING_SPACE | WIDTH | HEIGHT)
                {
                    &mut page_fields
                } else {
                    // The elemental decoder checks duplicate fields and literal widths.
                    let known = match kind {
                        Some(NODE)
                            if matches!(tag, SHOW_TERMINAL_CARBONS | SHOW_NON_TERMINAL_CARBONS) =>
                        {
                            nodes == Nodes::Skeletal && source.payload(record).ok()?.is_empty()
                        }
                        Some(NODE) if tag == CHARGE => source.payload(record).ok()?.len() == 1,
                        Some(NODE) => matches!(
                            tag,
                            POSITION | NODE_TYPE | ELEMENT | HYDROGENS | ISOTOPE | RADICAL
                        ),
                        Some(BOND) if nodes == Nodes::Skeletal && tag == ORDER => {
                            source.payload(record).ok()? == 1_u16.to_le_bytes()
                        }
                        Some(BOND) => matches!(
                            tag,
                            ORDER | BEGIN | END | DISPLAY | DISPLAY2 | DOUBLE_POSITION
                        ),
                        Some(TEXT_OBJECT) => matches!(
                            tag,
                            TEXT | BOUNDING_BOX | JUSTIFICATION | LABEL_JUSTIFICATION
                        ),
                        _ => false,
                    };
                    if !known {
                        return None;
                    }
                    continue;
                };
                if fields.insert(tag, index).is_some() {
                    return None;
                }
            }
            RecordKind::ObjectEnd { .. } | RecordKind::FileEnd => {}
        }
    }
    Some((root_fields, page_fields))
}

pub(super) fn is_drawing_property(tag: u16) -> bool {
    matches!(
        tag,
        CHAIN_ANGLE | BOND_LENGTH | BOND_SPACING | LINE_WIDTH | MARGIN_WIDTH
    )
}

fn recorded<const N: usize>(
    source: &Document<'_>,
    fields: &Fields,
    tag: u16,
) -> Option<Option<[u8; N]>> {
    fields
        .get(&tag)
        .map(|&record| payload(source, record, tag))
        .transpose()
        .ok()
}
