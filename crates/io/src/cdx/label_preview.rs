//! Whole-source display admission for explicitly boxed CDX atom labels.
use super::{
    BEGIN, BOND, BOND_LENGTH, BOUNDING_BOX, CHAIN_ANGLE, CHARGE, COLOR_TABLE, DISPLAY, DISPLAY2,
    DOCUMENT, DOUBLE_POSITION, DRAWING_SPACE, ELEMENT, END, ElementalGraph, FONT_TABLE, FRAGMENT,
    HEIGHT, HYDROGENS, ISOTOPE, NODE, NODE_TYPE, ORDER, PAGE, POSITION, RADICAL, Scope, TEXT,
    TEXT_OBJECT, WIDTH,
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
    let (root_fields, page_fields) = source_fields(source, scope)?;
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

fn source_fields(source: &Document<'_>, scope: Scope) -> Option<(Fields, Fields)> {
    let root = source.framing().has_encoded_document().then_some(0);
    let mut page = None;
    let (mut root_fields, mut page_fields) = (Fields::new(), Fields::new());
    for (index, record) in scope.records(source) {
        let owner = record.parent();
        let kind = object(source, owner).map(|(tag, _)| tag);
        match *record.kind() {
            RecordKind::ObjectStart { tag, .. } => match tag {
                DOCUMENT if root == Some(index) => {}
                PAGE if owner == root && page.is_none() => page = Some(index),
                FRAGMENT if page.is_some() && page == owner => {}
                NODE | BOND if kind == Some(FRAGMENT) => {}
                TEXT_OBJECT if kind == Some(NODE) => {}
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
                    // The elemental reader has already checked duplicate fields and literal widths.
                    let known = match kind {
                        Some(NODE) if tag == CHARGE => source.payload(record).ok()?.len() == 1,
                        Some(NODE) => matches!(
                            tag,
                            POSITION | NODE_TYPE | ELEMENT | HYDROGENS | ISOTOPE | RADICAL
                        ),
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
