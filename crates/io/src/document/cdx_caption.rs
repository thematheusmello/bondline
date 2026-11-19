//! Edit one existing caption run while preserving its current CDX source.
use super::{DocumentError, StoredDocument};
use crate::cdx::{MappingError, append_caption, caption_text_record, encode_caption_runs};
use bondline_cdx::Document;
use bondline_model::{PageId, StyledText, TextId};

impl StoredDocument {
    /// Append a plain caption to a qualified CDX page and verify its saved identity.
    pub fn with_cdx_caption(
        &self,
        page_id: PageId,
        caption: StyledText,
    ) -> Result<Self, DocumentError> {
        if !self.has_cdx_caption_preview() {
            return Err(DocumentError::UnsupportedCdxEdit);
        }
        let page = self
            .model
            .pages()
            .first()
            .filter(|page| page.id() == page_id)
            .ok_or(bondline_model::Error::MissingPage(page_id.get()))
            .map_err(MappingError::from)?;
        let source = Document::parse(self.export_cdx()?).map_err(MappingError::from)?;
        let (output, caption) = append_caption(&source, caption)?;
        let mut expected = self.model.as_ref().clone();
        expected
            .append_texts(page_id, vec![caption])
            .map_err(MappingError::from)?;
        self.finish_cdx_edit(
            output,
            expected,
            page.size(),
            DocumentError::UnsupportedCdxCaption(
                "complete caption agreement and paper fit after creation",
            ),
        )
    }

    /// Change one existing run of a qualified CDX caption, preserving its style and placement.
    pub fn with_cdx_caption_run(
        &self,
        page_id: PageId,
        text_id: TextId,
        run_index: usize,
        body: String,
    ) -> Result<Self, DocumentError> {
        if !self.has_cdx_caption_preview() {
            return Err(DocumentError::UnsupportedCdxEdit);
        }
        let page = self
            .model
            .pages()
            .iter()
            .find(|page| page.id() == page_id)
            .ok_or(bondline_model::Error::MissingPage(page_id.get()))
            .map_err(MappingError::from)?;
        let caption = page
            .texts()
            .iter()
            .find(|caption| caption.id() == text_id)
            .ok_or(bondline_model::Error::MissingText(text_id.get()))
            .map_err(MappingError::from)?;
        if caption
            .runs()
            .get(run_index)
            .is_some_and(|run| run.text() == body)
        {
            return Ok(self.clone());
        }
        let replacement = caption
            .with_run_text(run_index, body)
            .map_err(bondline_model::Error::from)
            .map_err(MappingError::from)?;
        let source = Document::parse(self.export_cdx()?).map_err(MappingError::from)?;
        let property = caption_text_record(&source, text_id).ok_or(
            DocumentError::UnsupportedCdxCaption("an owned Text property"),
        )?;
        let encoded = encode_caption_runs(&source, property, replacement.runs()).ok_or(
            DocumentError::UnsupportedCdxCaption("nonempty runs and unambiguous 16-bit starts"),
        )?;
        let output = source
            .replace_property(property, &encoded)
            .map_err(MappingError::from)?;
        let mut expected = self.model.as_ref().clone();
        expected
            .replace_text(page_id, replacement)
            .map_err(MappingError::from)?;
        self.finish_cdx_edit(
            output,
            expected,
            page.size(),
            DocumentError::UnsupportedCdxCaption(
                "complete caption agreement and paper fit after the edit",
            ),
        )
    }
}
