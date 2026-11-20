//! Whole-source proof and stable source identity for explicitly positioned CDX captions.
use super::{
    COLOR_TABLE, DOCUMENT, DRAWING_SPACE, FONT_TABLE, HEIGHT, MappingError, PAGE, POSITION,
    Preview, Scope, TEXT, TEXT_OBJECT, WIDTH, Z_ORDER, coordinate, has_preview_header, payload,
    poster_size, property_bytes, read_position,
    text::{TextRole, decode_runs},
    verify_preview_page,
};
use crate::DocumentError;
use bondline_cdx::{Document, Record, RecordKind, Resources};
use bondline_model::{PageSize, Point2, StyledText, TextId};
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
    page: usize,
    size: PageSize,
    captions: BTreeMap<TextId, Fields>,
}

fn layout(source: &Document<'_>, scope: Scope) -> Option<CaptionLayout> {
    if !has_preview_header(source) {
        return None;
    }
    let root = source.framing().has_encoded_document().then_some(0);
    let mut page = None;
    let mut page_fields = BTreeMap::new();
    let mut text_fields = BTreeMap::<usize, SourceCaption>::new();
    let mut identities = BTreeMap::new();
    let mut named_ids = BTreeSet::new();
    for (index, record) in scope.records(source) {
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
        page: page?,
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
    profile: Preview,
) -> Option<&'record Record> {
    let scope = match profile {
        Preview::Caption => Scope::All,
        Preview::Mixed => Scope::Captions,
        _ => return None,
    };
    let layout = layout(source, scope)?;
    let fields = layout.captions.get(&text)?;
    source.records().get(*fields.get(&TEXT)?)
}

pub(crate) fn append_caption(
    source: &Document<'_>,
    caption: StyledText,
    profile: Preview,
) -> Result<(Vec<u8>, StyledText), DocumentError> {
    let unsupported = DocumentError::UnsupportedCdxCaption;
    let scope = match profile {
        Preview::Caption => Scope::All,
        Preview::Mixed | Preview::AtomLabels => Scope::Captions,
        Preview::Elemental | Preview::Skeletal | Preview::SkeletalLabels => {
            return Err(DocumentError::UnsupportedCdxEdit);
        }
    };
    let layout = layout(source, scope).ok_or(unsupported("a certified Page caption layout"))?;
    if layout.captions.contains_key(&caption.id()) {
        return Err(unsupported("an unused live caption ID"));
    }
    let [run] = caption.runs() else {
        return Err(unsupported("one caption run"));
    };
    let style = run.style();
    if run.text().trim().is_empty()
        || style.font_family() != "Roboto"
        || style.size_pt() != 12.0
        || style.face() != 0
        || style.rgb() != [0.0; 3]
    {
        return Err(unsupported("nonblank 12 pt black Roboto Regular text"));
    }
    let [Some(y), Some(x)] = [caption.position().y_pt(), caption.position().x_pt()].map(fixed)
    else {
        return Err(unsupported("a representable caption position"));
    };
    let position = Point2::new(coordinate(x), coordinate(y)).map_err(MappingError::from)?;
    if position.x_pt() <= 0.0
        || position.y_pt() <= 0.0
        || position.x_pt() >= layout.size.width_pt()
        || position.y_pt() >= layout.size.height_pt()
    {
        return Err(unsupported("a rounded baseline inside the paper"));
    }
    let raw_id = super::next_object_id(source).ok_or(unsupported("an available raw object ID"))?;
    let depth = if layout.captions.is_empty() {
        // Chemical objects in the qualified profile use the published default depth zero.
        i16::from(profile == Preview::AtomLabels)
    } else {
        layout
            .captions
            .values()
            .try_fold(i16::MIN, |front, fields| {
                paint_depth(source, fields).map(|depth| front.max(depth))
            })
            .and_then(|front| front.checked_add(1))
            .ok_or(unsupported("an available caption depth"))?
    };
    let resources = source
        .resources()
        .map_err(|_| unsupported("font resources"))?;
    let font = creation_font_id(&resources).ok_or(unsupported("a declared UTF-8 Roboto font"))?;
    let words = [1, 0, font, 0, 240, 0].map(u16::to_le_bytes);
    let mut text = Vec::new();
    text.try_reserve_exact(words.as_flattened().len() + run.text().len())
        .map_err(|_| MappingError::from(bondline_cdx::Error::Allocation))?;
    text.extend_from_slice(words.as_flattened());
    text.extend_from_slice(run.text().as_bytes());
    let page_end = matched_end(source, layout.page).ok_or(unsupported("an owned Page end"))?;
    let with_text = source
        .append_child(
            page_end,
            TEXT_OBJECT,
            raw_id,
            &[
                (POSITION, [y, x].as_flattened()),
                (CAPTION_JUSTIFICATION, &[0]),
                (Z_ORDER, &depth.to_le_bytes()),
                (TEXT, &text),
            ],
        )
        .map_err(MappingError::from)?;
    let intermediate = Document::parse(&with_text).map_err(MappingError::from)?;
    let text_start = intermediate.records().iter().position(|record| {
        record.parent() == Some(layout.page)
            && *record.kind()
                == RecordKind::ObjectStart {
                    tag: TEXT_OBJECT,
                    id: raw_id,
                }
    });
    let text_end = text_start
        .and_then(|start| matched_end(&intermediate, start))
        .ok_or(unsupported("an owned new Text end"))?;
    let value = format!("{CAPTION_ID_VERSION}{}", caption.id().get());
    let output = intermediate
        .append_child(
            text_end,
            OBJECT_TAG,
            0,
            &[
                (TAG_NAME, CAPTION_ID_NAME),
                (TAG_TYPE, &[3, 0]),
                (TAG_PERSISTENT, &[1]),
                (TAG_VALUE, value.as_bytes()),
            ],
        )
        .map_err(MappingError::from)?;
    let caption = caption
        .with_position(position)
        .map_err(bondline_model::Error::from)
        .map_err(MappingError::from)?;
    Ok((output, caption))
}

fn fixed(value: f64) -> Option<[u8; 4]> {
    let scaled = (value * 65_536.0).round();
    (f64::from(i32::MIN)..=f64::from(i32::MAX))
        .contains(&scaled)
        .then_some((scaled as i32).to_le_bytes())
}

pub(super) fn matched_end<'record>(
    source: &'record Document<'_>,
    owner: usize,
) -> Option<&'record Record> {
    source
        .records()
        .iter()
        .find(|record| *record.kind() == RecordKind::ObjectEnd { start: Some(owner) })
}

fn paint_depth(source: &Document<'_>, fields: &Fields) -> Option<i16> {
    fields.get(&Z_ORDER).map_or(Some(0), |&index| {
        Some(i16::from_le_bytes(payload(source, index, Z_ORDER).ok()?))
    })
}

pub(super) fn creation_font_id(resources: &Resources<'_, '_>) -> Option<u16> {
    if resources.font_platform() != Some(1) {
        return None;
    }
    resources
        .fonts()
        .iter()
        .find(|font| {
            font.name_bytes() == b"Roboto"
                && font.charset() == 65001
                && font.ordinal() == usize::from(font.id())
        })
        .map(|font| font.id())
}

pub(super) fn preview(source: &Document<'_>) -> Option<bondline_model::Document> {
    let layout = layout(source, Scope::All)?;
    let resources = source.resources().ok()?;
    if resources.color(2).ok()? != [u16::MAX; 3]
        || (layout.captions.is_empty() && creation_font_id(&resources).is_none())
    {
        return None;
    }
    let size = layout.size;
    let mut remaining_runs = MAX_PAGE_TEXT_RUNS;
    let texts = ordered_texts(source, layout, &resources, &mut remaining_runs, false)?;
    let mut document = bondline_model::Document::default();
    let page = document.pages().first()?.id();
    document.set_page_size(page, size).ok()?;
    document.append_texts(page, texts).ok()?;
    verify_preview_page(&document)?;
    Some(document)
}

pub(super) fn mixed_texts(
    source: &Document<'_>,
    resources: &Resources<'_, '_>,
    remaining_runs: &mut usize,
) -> Option<Vec<StyledText>> {
    let layout = layout(source, Scope::Captions)?;
    ordered_texts(source, layout, resources, remaining_runs, true)
}

fn ordered_texts(
    source: &Document<'_>,
    layout: CaptionLayout,
    resources: &Resources<'_, '_>,
    remaining_runs: &mut usize,
    above_chemistry: bool,
) -> Option<Vec<StyledText>> {
    let mut ordered = BTreeMap::new();
    let mut omitted = false;
    for (id, fields) in layout.captions {
        let depth = paint_depth(source, &fields)?;
        if above_chemistry && depth <= 0 {
            return None;
        }
        omitted |= !fields.contains_key(&Z_ORDER);
        if ordered
            .insert(
                depth,
                project_text(source, resources, &fields, id, remaining_runs)?,
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
    Some(ordered.into_values().collect())
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
