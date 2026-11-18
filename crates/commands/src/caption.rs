use crate::Error;
use bondline_io::StoredDocument;
use bondline_model::{PageId, TextId};
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

pub(super) fn set_run(
    document: &StoredDocument,
    params: TextSetRun,
) -> Result<StoredDocument, Error> {
    let cdx_caption = document.has_cdx_caption_preview();
    if !cdx_caption
        && ((!document.is_fully_mapped() && !document.has_caption_projection())
            || document
                .source_info()
                .is_some_and(|source| source.format != "cdxml"))
    {
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
    let page = document
        .model()
        .pages()
        .iter()
        .find(|page| page.id() == page_id)
        .ok_or(bondline_model::Error::MissingPage(page_id.get()))?;
    let text = page
        .texts()
        .iter()
        .find(|text| text.id() == text_id)
        .ok_or(bondline_model::Error::MissingText(text_id.get()))?;
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
    // A valid model edit must also retain every source byte it does not own.
    candidate.to_cdxml()?;
    let page = candidate
        .model()
        .pages()
        .iter()
        .find(|p| p.id() == page_id)
        .ok_or(Error::InvalidPageId)?;
    bondline_io::smiles::validate_authoring_page_with_settings(
        page,
        candidate.model().drawing_style(),
        candidate.model().drawing_settings(),
        &bondline_render::Cancellation::default(),
    )
    .map_err(|error| match error {
        bondline_io::smiles::LayoutError::Render(error) => Error::Render(error),
        other => Error::SmilesLayout(other),
    })?;
    Ok(candidate)
}
