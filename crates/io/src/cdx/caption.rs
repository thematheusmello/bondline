//! Whole-source proof and stable source identity for explicitly positioned CDX captions.
use super::{
    COLOR_TABLE, DOCUMENT, DRAWING_SPACE, FONT_TABLE, HEIGHT, PAGE, POSITION, TEXT, TEXT_OBJECT,
    WIDTH, Z_ORDER, has_preview_header, payload, poster_size, property_bytes, read_position,
    text::{TextRole, decode_runs},
    verify_preview_page,
};
use bondline_cdx::{Document, Record, RecordKind, Resources};
use bondline_model::{PageSize, StyledText, TextId};
use bondline_render::{MAX_PAGE_TEXT_RUNS, validate_caption_run_joins};
use std::collections::{BTreeMap, BTreeSet};

const CAPTION_JUSTIFICATION: u16 = 0x080c;
const OBJECT_TAG: u16 = 0x8011;
const TAG_NAME: u16 = 0x0008;
const TAG_TYPE: u16 = 0x0d00;
const TAG_PERSISTENT: u16 = 0x0d04;
const TAG_VALUE: u16 = 0x0d05;
const CAPTION_ID_NAME: &[u8] = b"org.bondline.caption-id";
const CAPTION_ID_VERSION: &str = "1:";
type Fields = BTreeMap<u16, usize>;

struct SourceCaption {
    raw_id: u32,
    fields: Fields,
    identity: Option<usize>,
}

struct CaptionLayout {
    size: PageSize,
    captions: BTreeMap<TextId, Fields>,
}

fn layout(source: &Document<'_>) -> Option<CaptionLayout> {
    if !has_preview_header(source) {
        return None;
    }
    let root = source.framing().has_encoded_document().then_some(0);
    let mut page = None;
    let mut page_fields = BTreeMap::new();
    let mut text_fields = BTreeMap::<usize, SourceCaption>::new();
    let mut identities = BTreeMap::new();
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
                text_fields.insert(
                    index,
                    SourceCaption {
                        raw_id: id,
                        fields: BTreeMap::new(),
                        identity: None,
                    },
                );
            }
            RecordKind::ObjectStart {
                tag: OBJECT_TAG,
                id: 0,
            } => {
                let caption = text_fields.get_mut(&record.parent()?)?;
                if caption.identity.replace(index).is_some() {
                    return None;
                }
                identities.insert(index, BTreeMap::new());
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
                        &mut text_fields.get_mut(&owner?)?.fields
                    }
                    TAG_NAME | TAG_TYPE | TAG_PERSISTENT | TAG_VALUE => {
                        identities.get_mut(&owner?)?
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
    let mut by_raw_id: Vec<_> = text_fields.into_values().collect();
    // A stable sort keeps anonymous captions in encounter order.
    by_raw_id.sort_by_key(|caption| caption.raw_id);
    let mut captions = BTreeMap::new();
    let mut dense = 0_u32;
    for caption in by_raw_id {
        let id = if let Some(identity) = caption.identity {
            read_identity(source, identities.get(&identity)?)?
        } else {
            dense = dense.checked_add(1)?;
            TextId::new(dense)?
        };
        if captions.insert(id, caption.fields).is_some() {
            return None;
        }
    }
    Some(CaptionLayout {
        size: poster_size(source, &page_fields)?,
        captions,
    })
}

fn read_identity(source: &Document<'_>, fields: &Fields) -> Option<TextId> {
    if property_bytes(source, *fields.get(&TAG_NAME)?, TAG_NAME).ok()? != CAPTION_ID_NAME
        || payload(source, *fields.get(&TAG_TYPE)?, TAG_TYPE).ok()? != [3, 0]
    {
        return None;
    }
    if let Some(&persistent) = fields.get(&TAG_PERSISTENT)
        && payload(source, persistent, TAG_PERSISTENT).ok()? == [0]
    {
        return None;
    }
    let digits = property_bytes(source, *fields.get(&TAG_VALUE)?, TAG_VALUE)
        .ok()?
        .strip_prefix(CAPTION_ID_VERSION.as_bytes())?;
    if digits.len() > 10 || !matches!(digits.first(), Some(b'1'..=b'9')) {
        return None;
    }
    TextId::new(std::str::from_utf8(digits).ok()?.parse().ok()?)
}

pub(crate) fn text_record<'record>(
    source: &'record Document<'_>,
    text: TextId,
) -> Option<&'record Record> {
    let layout = layout(source)?;
    let fields = layout.captions.get(&text)?;
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
    for (id, fields) in layout.captions {
        let depth = if let Some(&property) = fields.get(&Z_ORDER) {
            i16::from_le_bytes(payload(source, property, Z_ORDER).ok()?)
        } else {
            omitted = true;
            0
        };
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
