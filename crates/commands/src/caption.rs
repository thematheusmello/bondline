use crate::{Error, MAX_CAPTION_DRAFT_BYTES};
use bondline_io::StoredDocument;
use bondline_model::{Page, PageId, Point2, StyledText, TextId, TextRun, TextStyle};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextSetRun {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 1))]
    pub text_id: u32,
    pub run_index: usize,
    /// Decoded UTF-8 text. The aggregate text object's 1 MiB budget still applies.
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextCreate {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    /// Left baseline origin, in document points.
    pub x_pt: f64,
    pub y_pt: f64,
    /// Literal, single-line UTF-8 text, at most 16384 bytes.
    pub text: String,
}

fn supports_xml_source(document: &StoredDocument) -> bool {
    (document.is_fully_mapped() || document.has_caption_projection())
        && document
            .source_info()
            .is_none_or(|source| source.format == "cdxml")
}

pub(super) fn supports_source(document: &StoredDocument) -> bool {
    document.can_edit_cdx_caption_run() || supports_xml_source(document)
}

pub(super) fn find(
    document: &StoredDocument,
    page_id: PageId,
    text_id: TextId,
) -> Result<&StyledText, Error> {
    page(document, page_id)?
        .texts()
        .iter()
        .find(|text| text.id() == text_id)
        .map(AsRef::as_ref)
        .ok_or_else(|| bondline_model::Error::MissingText(text_id.get()).into())
}

fn page(document: &StoredDocument, page_id: PageId) -> Result<&Page, Error> {
    document
        .model()
        .pages()
        .iter()
        .find(|page| page.id() == page_id)
        .ok_or_else(|| bondline_model::Error::MissingPage(page_id.get()).into())
}

pub(super) fn creation_point(
    document: &StoredDocument,
    page_id: u32,
    x_pt: f64,
    y_pt: f64,
) -> Result<(PageId, Point2), Error> {
    if !document.can_create_caption() {
        return Err(Error::UnsupportedCaptionCreationSource);
    }
    let page_id = PageId::new(page_id).ok_or(Error::InvalidPageId)?;
    let size = page(document, page_id)?.size();
    let point = Point2::new(x_pt, y_pt)?;
    if x_pt <= 0.0 || x_pt >= size.width_pt() || y_pt <= 0.0 || y_pt >= size.height_pt() {
        return Err(Error::InvalidCaptionPosition);
    }
    Ok((page_id, point))
}

pub(super) fn create(
    document: &StoredDocument,
    params: TextCreate,
    floor: u32,
) -> Result<(StoredDocument, u32), Error> {
    let (page_id, position) = creation_point(document, params.page_id, params.x_pt, params.y_pt)?;
    if params.text.len() > MAX_CAPTION_DRAFT_BYTES {
        return Err(Error::CaptionDraftLimit);
    }
    if params.text.trim().is_empty() {
        return Err(Error::EmptyCaption);
    }
    let max = page(document, page_id)?
        .texts()
        .iter()
        .fold(floor, |max, caption| max.max(caption.id().get()));
    let id = max
        .checked_add(1)
        .and_then(TextId::new)
        .ok_or(Error::IdentityExhausted)?;
    let style =
        TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3]).map_err(bondline_model::Error::from)?;
    let run = TextRun::new(params.text, style).map_err(bondline_model::Error::from)?;
    let text = StyledText::new(id, position, vec![run]).map_err(bondline_model::Error::from)?;
    if document.can_create_cdx_caption() {
        return Ok((document.with_cdx_caption(page_id, text)?, id.get()));
    }
    let mut candidate = document.clone();
    candidate.model_mut().append_texts(page_id, vec![text])?;
    if !candidate.can_create_caption() {
        return Err(Error::IdentityExhausted);
    }
    certify(&candidate, page_id)?;
    Ok((candidate, id.get()))
}

pub(super) fn set_run(
    document: &StoredDocument,
    params: TextSetRun,
) -> Result<StoredDocument, Error> {
    let cdx_caption = document.can_edit_cdx_caption_run();
    if !cdx_caption && !supports_xml_source(document) {
        return Err(Error::UnsupportedCaptionSource);
    }
    let page_id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
    let text_id = TextId::new(params.text_id).ok_or(Error::InvalidTextId)?;
    if cdx_caption {
        return Ok(document.with_cdx_caption_run(
            page_id,
            text_id,
            params.run_index,
            params.text,
        )?);
    }
    let text = find(document, page_id, text_id)?;
    let run = text
        .runs()
        .get(params.run_index)
        .ok_or(bondline_model::Error::Text(
            bondline_model::TextError::MissingRun(params.run_index),
        ))?;
    if run.text() == params.text.as_str() {
        return Ok(document.clone());
    }
    let replacement = text
        .with_run_text(params.run_index, params.text)
        .map_err(bondline_model::Error::from)?;
    let mut candidate = document.clone();
    candidate.model_mut().replace_text(page_id, replacement)?;
    certify(&candidate, page_id)?;
    Ok(candidate)
}

fn certify(candidate: &StoredDocument, page_id: PageId) -> Result<(), Error> {
    // A valid model edit must also retain every source byte it does not own.
    candidate.to_cdxml()?;
    bondline_io::smiles::validate_authoring_page_with_settings(
        page(candidate, page_id)?,
        candidate.model().drawing_style(),
        candidate.model().drawing_settings(),
        &bondline_render::Cancellation::default(),
    )
    .map_err(|error| match error {
        bondline_io::smiles::LayoutError::Render(error) => Error::Render(error),
        other => Error::SmilesLayout(other),
    })
}
