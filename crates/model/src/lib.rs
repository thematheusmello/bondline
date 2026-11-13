//! GUI-free authored document data. Coordinates are points, increasing right and down.

use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

/// Stable page identity, independent of its position in the document.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageId(NonZeroU32);

impl PageId {
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(id) => Some(Self(id)),
            None => None,
        }
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

/// Dimensions use 1/72 inch points, not screen pixels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawPageSize")]
pub struct PageSize {
    width_pt: f64,
    height_pt: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPageSize {
    width_pt: f64,
    height_pt: f64,
}

impl TryFrom<RawPageSize> for PageSize {
    type Error = Error;
    fn try_from(raw: RawPageSize) -> Result<Self, Error> {
        Self::new(raw.width_pt, raw.height_pt)
    }
}

impl PageSize {
    pub const A4: Self = Self {
        width_pt: 210.0 * 72.0 / 25.4,
        height_pt: 297.0 * 72.0 / 25.4,
    };

    pub fn new(width_pt: f64, height_pt: f64) -> Result<Self, Error> {
        // shortcut: authoring dimensions are capped at 100,000 pt; revisit for larger tiled documents.
        if [width_pt, height_pt]
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0 || *value > 100_000.0)
        {
            return Err(Error::InvalidPageSize);
        }
        Ok(Self {
            width_pt,
            height_pt,
        })
    }

    pub const fn width_pt(self) -> f64 {
        self.width_pt
    }
    pub const fn height_pt(self) -> f64 {
        self.height_pt
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("page dimensions must be finite and greater than zero, up to 100000 points")]
    InvalidPageSize,
    #[error("page {0} does not exist")]
    MissingPage(u32),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Page {
    id: PageId,
    size: PageSize,
}

impl Page {
    pub const fn id(&self) -> PageId {
        self.id
    }
    pub const fn size(&self) -> PageSize {
        self.size
    }
}

/// Authored state only: history, selections, viewport and caches belong to the engine.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Document {
    pages: Vec<Page>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            pages: vec![Page {
                id: PageId(NonZeroU32::MIN),
                size: PageSize::A4,
            }],
        }
    }
}

impl Document {
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    pub fn set_page_size(&mut self, id: PageId, size: PageSize) -> Result<(), Error> {
        let page = self
            .pages
            .iter_mut()
            .find(|page| page.id == id)
            .ok_or(Error::MissingPage(id.get()))?;
        page.size = size;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_page_has_physical_a4_dimensions_and_a_stable_id()
    -> Result<(), Box<dyn std::error::Error>> {
        let document = super::Document::default();
        let page = document.pages().first().ok_or("missing first page")?;
        assert_eq!(page.id().get(), 1);
        // 1 inch = 25.4 mm = 72 points; A4 is 210 by 297 mm.
        assert!((page.size().width_pt() - 595.2755905511812).abs() < 1e-10);
        assert!((page.size().height_pt() - 841.8897637795276).abs() < 1e-10);
        Ok(())
    }

    #[test]
    fn page_size_validation_cannot_be_bypassed_through_json()
    -> Result<(), Box<dyn std::error::Error>> {
        for width in [0.0, -1.0, f64::NAN, f64::INFINITY, 100_001.0] {
            assert!(super::PageSize::new(width, 72.0).is_err());
            assert!(super::PageSize::new(72.0, width).is_err());
        }
        assert!(
            serde_json::from_str::<super::PageSize>(r#"{"width_pt":-1,"height_pt":72}"#).is_err()
        );
        let inch = super::PageSize::new(72.0, 72.0)?;
        assert_eq!(
            serde_json::to_value(inch)?,
            serde_json::json!({"width_pt":72.0,"height_pt":72.0})
        );
        assert!(super::PageId::new(0).is_none());
        Ok(())
    }

    #[test]
    fn missing_page_edit_leaves_the_authored_document_untouched()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut document = super::Document::default();
        let before = serde_json::to_vec(&document)?;
        let missing = super::PageId::new(2).ok_or("invalid test id")?;
        assert!(
            document
                .set_page_size(missing, super::PageSize::new(612.0, 792.0)?)
                .is_err()
        );
        assert_eq!(serde_json::to_vec(&document)?, before);
        let id = super::PageId::new(1).ok_or("invalid test id")?;
        document.set_page_size(id, super::PageSize::new(612.0, 792.0)?)?;
        assert_eq!(document.pages().first().ok_or("missing page")?.id(), id);
        Ok(())
    }
}
