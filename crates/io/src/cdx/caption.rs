//! Whole-source proof for read-only, explicitly positioned CDX captions.
use super::{
    DOCUMENT, PAGE, POSITION, TEXT, TEXT_OBJECT, Z_ORDER, coordinate, payload, read_position,
    text::{TextRole, decode_runs},
};
use bondline_cdx::{Document, RecordKind, Resources};
use bondline_model::{PageSize, Point2, StyledText, TextId};
use bondline_render::{Cancellation, MAX_PAGE_TEXT_RUNS, PageView, validate_caption_run_joins};
use std::collections::{BTreeMap, BTreeSet};

const FONT_TABLE: u16 = 0x0100;
const COLOR_TABLE: u16 = 0x0300;
const CAPTION_JUSTIFICATION: u16 = 0x080c;
const DRAWING_SPACE: u16 = 0x0811;
const WIDTH: u16 = 0x0812;
const HEIGHT: u16 = 0x0813;

pub(super) fn preview(source: &Document<'_>) -> Option<bondline_model::Document> {
    if source.header().get(12..)?.iter().any(|&byte| byte != 0) {
        return None;
    }
    let root = source.framing().has_encoded_document().then_some(0);
    let mut page = None;
    let mut page_fields = BTreeMap::new();
    let mut text_fields = BTreeMap::new();
    let mut named_ids = BTreeSet::new();
    for (index, record) in source.records().iter().enumerate() {
        match *record.kind() {
            RecordKind::ObjectStart { tag: DOCUMENT, .. } if root == Some(index) => {}
            RecordKind::ObjectStart { tag: PAGE, .. }
                if record.parent() == root && page.is_none() =>
            {
                page = Some(index);
            }
            RecordKind::ObjectStart {
                tag: TEXT_OBJECT,
                id,
            } if page.is_some() && record.parent() == page => {
                if id != 0 && !named_ids.insert(id) {
                    return None;
                }
                text_fields.insert(index, (id, BTreeMap::new()));
            }
            RecordKind::ObjectStart { .. } => return None,
            RecordKind::Property { tag, .. } => {
                let owner = record.parent();
                let fields = match tag {
                    FONT_TABLE | COLOR_TABLE if owner == root => continue,
                    DRAWING_SPACE | WIDTH | HEIGHT if page.is_some() && owner == page => {
                        &mut page_fields
                    }
                    TEXT | CAPTION_JUSTIFICATION | POSITION | Z_ORDER => {
                        &mut text_fields.get_mut(&owner?)?.1
                    }
                    _ => return None,
                };
                if fields.insert(tag, index).is_some() {
                    return None;
                }
            }
            RecordKind::ObjectEnd { .. } | RecordKind::FileEnd => {}
        }
    }
    if payload(source, *page_fields.get(&DRAWING_SPACE)?, DRAWING_SPACE).ok()? != [1] {
        return None;
    }
    let width = coordinate(payload(source, *page_fields.get(&WIDTH)?, WIDTH).ok()?);
    let height = coordinate(payload(source, *page_fields.get(&HEIGHT)?, HEIGHT).ok()?);
    let size = PageSize::new(width, height).ok()?;
    let resources = source.resources().ok()?;
    if resources.color(2).ok()? != [u16::MAX; 3] {
        return None;
    }
    let mut captions: Vec<_> = text_fields.into_values().collect();
    // A stable sort keeps anonymous captions in encounter order.
    captions.sort_by_key(|(id, _)| *id);
    let mut ordered = BTreeMap::new();
    let mut omitted = false;
    let mut remaining_runs = MAX_PAGE_TEXT_RUNS;
    for (ordinal, (_, fields)) in captions.into_iter().enumerate() {
        let depth = if let Some(&property) = fields.get(&Z_ORDER) {
            i16::from_le_bytes(payload(source, property, Z_ORDER).ok()?)
        } else {
            omitted = true;
            0
        };
        let id = TextId::new(u32::try_from(ordinal).ok()?.checked_add(1)?)?;
        if ordered
            .insert(
                depth,
                project_text(source, &resources, &fields, id, &mut remaining_runs)?,
            )
            .is_some()
        {
            return None;
        }
    }
    // The published description of the default zero group leaves this case ambiguous.
    if omitted && *ordered.first_key_value()?.0 < 0 {
        return None;
    }
    if ordered.is_empty() {
        return None;
    }
    let mut document = bondline_model::Document::default();
    let page = document.pages().first()?.id();
    document.set_page_size(page, size).ok()?;
    document
        .append_texts(page, ordered.into_values().collect())
        .ok()?;
    // The ordinary scene checks the entire paper before destination clipping.
    let center = Point2::new(width / 2.0, height / 2.0).ok()?;
    let view = PageView::zoom(1, 1, size, 100.0, center).ok()?;
    view.graph_scene(document.pages().first()?, &Cancellation::default())
        .ok()?;
    Some(document)
}

fn project_text(
    source: &Document<'_>,
    resources: &Resources<'_, '_>,
    fields: &BTreeMap<u16, usize>,
    id: TextId,
    remaining_runs: &mut usize,
) -> Option<StyledText> {
    if payload(
        source,
        *fields.get(&CAPTION_JUSTIFICATION)?,
        CAPTION_JUSTIFICATION,
    )
    .ok()?
        != [0]
    {
        return None;
    }
    let position = read_position(source, *fields.get(&POSITION)?).ok()?;
    let runs = decode_runs(
        source,
        resources,
        *fields.get(&TEXT)?,
        TextRole::Caption,
        remaining_runs,
    )?;
    validate_caption_run_joins(&runs).ok()?;
    StyledText::new(id, position, runs).ok()
}
