//! Whole-source proof for one read-only, explicitly positioned CDX caption.
use super::{DOCUMENT, PAGE, POSITION, coordinate, payload, read_position};
use bondline_cdx::{Document, RecordKind};
use bondline_model::{MAX_TEXT_BYTES, PageSize, Point2, StyledText, TextId, TextRun, TextStyle};
use bondline_render::{Cancellation, PageView};
use std::collections::BTreeMap;

const FONT_TABLE: u16 = 0x0100;
const COLOR_TABLE: u16 = 0x0300;
const TEXT_OBJECT: u16 = 0x8006;
const TEXT: u16 = 0x0700;
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
    let mut text = None;
    let mut page_fields = BTreeMap::new();
    let mut text_fields = BTreeMap::new();
    for (index, record) in source.records().iter().enumerate() {
        match *record.kind() {
            RecordKind::ObjectStart { tag: DOCUMENT, .. } if root == Some(index) => {}
            RecordKind::ObjectStart { tag: PAGE, .. }
                if record.parent() == root && page.is_none() =>
            {
                page = Some(index);
            }
            RecordKind::ObjectStart {
                tag: TEXT_OBJECT, ..
            } if page.is_some() && record.parent() == page && text.is_none() => {
                text = Some(index);
            }
            RecordKind::ObjectStart { .. } => return None,
            RecordKind::Property { tag, .. } => {
                let owner = record.parent();
                let fields = match tag {
                    FONT_TABLE | COLOR_TABLE if owner == root => continue,
                    DRAWING_SPACE | WIDTH | HEIGHT if page.is_some() && owner == page => {
                        &mut page_fields
                    }
                    TEXT | CAPTION_JUSTIFICATION | POSITION if text.is_some() && owner == text => {
                        &mut text_fields
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
    if payload(source, *page_fields.get(&DRAWING_SPACE)?, DRAWING_SPACE).ok()? != [1]
        || payload(
            source,
            *text_fields.get(&CAPTION_JUSTIFICATION)?,
            CAPTION_JUSTIFICATION,
        )
        .ok()?
            != [0]
    {
        return None;
    }
    let width = coordinate(payload(source, *page_fields.get(&WIDTH)?, WIDTH).ok()?);
    let height = coordinate(payload(source, *page_fields.get(&HEIGHT)?, HEIGHT).ok()?);
    let size = PageSize::new(width, height).ok()?;
    let position = read_position(source, *text_fields.get(&POSITION)?).ok()?;

    let resources = source.resources().ok()?;
    if resources.font_platform() != Some(1) || resources.color(2).ok()? != [u16::MAX; 3] {
        return None;
    }
    let value = source
        .string(source.records().get(*text_fields.get(&TEXT)?)?)
        .ok()?;
    let mut runs = value.runs();
    if runs.len() != 1 {
        return None;
    }
    let run = runs.next()?;
    if run.start() != 0 || run.face() != 0 {
        return None;
    }
    let font = resources.font_by_id(run.font()).ok()?;
    if font.ordinal() != usize::from(run.font())
        || font.charset() != 65001
        || font.name_bytes() != b"Roboto"
    {
        return None;
    }
    let text = value.text_bytes();
    if text.is_empty() || text.len() > MAX_TEXT_BYTES {
        return None;
    }
    let body = std::str::from_utf8(text).ok()?;
    let rgb = resources
        .color(u32::from(run.color()))
        .ok()?
        .map(|channel| f64::from(channel) / f64::from(u16::MAX));
    let style = TextStyle::new(
        "Roboto".to_owned(),
        f64::from(run.size_twentieths()) / 20.0,
        run.face(),
        rgb,
    )
    .ok()?;
    let text = StyledText::new(
        TextId::new(1)?,
        position,
        vec![TextRun::new(body.to_owned(), style).ok()?],
    )
    .ok()?;
    let mut document = bondline_model::Document::default();
    let page = document.pages().first()?.id();
    document.set_page_size(page, size).ok()?;
    document.append_texts(page, vec![text]).ok()?;
    // The ordinary scene checks the entire paper before destination clipping.
    let center = Point2::new(width / 2.0, height / 2.0).ok()?;
    let view = PageView::zoom(1, 1, size, 100.0, center).ok()?;
    view.graph_scene(document.pages().first()?, &Cancellation::default())
        .ok()?;
    Some(document)
}
