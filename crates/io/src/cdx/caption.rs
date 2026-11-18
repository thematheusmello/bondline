//! Whole-source proof and stable source identity for explicitly positioned CDX captions.
use super::{
    COLOR_TABLE, DOCUMENT, DRAWING_SPACE, FONT_TABLE, HEIGHT, PAGE, POSITION, TEXT, TEXT_OBJECT,
    WIDTH, Z_ORDER, has_preview_header, payload, poster_size, read_position,
    text::{TextRole, decode_runs},
    verify_preview_page,
};
use bondline_cdx::{Document, Record, RecordKind, Resources};
use bondline_model::{PageSize, StyledText, TextId};
use bondline_render::{MAX_PAGE_TEXT_RUNS, validate_caption_run_joins};
use std::collections::{BTreeMap, BTreeSet};

const CAPTION_JUSTIFICATION: u16 = 0x080c;
type Fields = BTreeMap<u16, usize>;

struct CaptionLayout {
    size: PageSize,
    captions: Vec<(u32, Fields)>,
}

fn layout(source: &Document<'_>) -> Option<CaptionLayout> {
    if !has_preview_header(source) {
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
    let mut captions: Vec<_> = text_fields.into_values().collect();
    // A stable sort keeps anonymous captions in encounter order.
    captions.sort_by_key(|(id, _)| *id);
    Some(CaptionLayout {
        size: poster_size(source, &page_fields)?,
        captions,
    })
}

pub(crate) fn text_record<'record>(
    source: &'record Document<'_>,
    text: TextId,
) -> Option<&'record Record> {
    let ordinal = usize::try_from(text.get()).ok()?.checked_sub(1)?;
    let layout = layout(source)?;
    let (_, fields) = layout.captions.get(ordinal)?;
    source.records().get(*fields.get(&TEXT)?)
}

pub(super) fn preview(source: &Document<'_>) -> Option<bondline_model::Document> {
    let layout = layout(source)?;
    let resources = source.resources().ok()?;
    if resources.color(2).ok()? != [u16::MAX; 3] {
        return None;
    }
    let mut ordered = BTreeMap::new();
    let mut omitted = false;
    let mut remaining_runs = MAX_PAGE_TEXT_RUNS;
    for (ordinal, (_, fields)) in layout.captions.into_iter().enumerate() {
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
    document.set_page_size(page, layout.size).ok()?;
    document
        .append_texts(page, ordered.into_values().collect())
        .ok()?;
    verify_preview_page(&document)?;
    Some(document)
}

fn project_text(
    source: &Document<'_>,
    resources: &Resources<'_, '_>,
    fields: &Fields,
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
