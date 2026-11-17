//! GUI-free authored document data. Coordinates are points, increasing right and down.

mod drawing;
mod element;
mod graph;
mod point;
mod style;
pub use drawing::DrawingSettings;
pub use graph::{
    Atom, AtomId, Bond, BondId, BondOrder, DoublePosition, Element, Fragment, FragmentId,
    GraphError, MAX_GRAPH_ITEMS, Radical,
};
pub use point::{MAX_POSITION_PT, Point2};
pub use style::DrawingStyle;

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    mem::size_of,
    num::NonZeroU32,
    sync::Arc,
};

pub const MAX_PAGE_DIMENSION_PT: f64 = 100_000.0;

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
            .any(|value| !value.is_finite() || *value <= 0.0 || *value > MAX_PAGE_DIMENSION_PT)
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
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error("point coordinates must be finite, within -100000..100000 points")]
    InvalidPosition,
    #[error("page dimensions must be finite and greater than zero, up to 100000 points")]
    InvalidPageSize,
    #[error(
        "drawing settings need a finite bond length above zero through 100000 pt and a finite chain angle 1..179 degrees"
    )]
    InvalidDrawingSettings,
    #[error(
        "line width must be finite above zero through 100000 pt; bond spacing must be finite within 0..3276.7 percent"
    )]
    InvalidDrawingStyle,
    #[error("page {0} does not exist")]
    MissingPage(u32),
    #[error("fragment {0} does not exist on the requested page")]
    MissingFragment(u32),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Page {
    id: PageId,
    size: PageSize,
    fragments: Vec<Arc<Fragment>>,
}

impl Page {
    pub const fn id(&self) -> PageId {
        self.id
    }
    pub const fn size(&self) -> PageSize {
        self.size
    }
    pub fn fragments(&self) -> &[Arc<Fragment>] {
        &self.fragments
    }
}

/// Authored state only: history, selections, viewport and caches belong to the engine.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Document {
    pages: Vec<Page>,
    #[serde(skip_serializing_if = "DrawingSettings::is_empty")]
    drawing_settings: DrawingSettings,
    #[serde(skip_serializing_if = "DrawingStyle::is_empty")]
    drawing_style: DrawingStyle,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            pages: vec![Page {
                id: PageId(NonZeroU32::MIN),
                size: PageSize::A4,
                fragments: Vec::new(),
            }],
            drawing_settings: DrawingSettings::default(),
            drawing_style: DrawingStyle::default(),
        }
    }
}

impl Document {
    pub const fn drawing_style(&self) -> DrawingStyle {
        self.drawing_style
    }

    pub fn set_drawing_style(&mut self, style: DrawingStyle) {
        self.drawing_style = style;
    }

    pub const fn drawing_settings(&self) -> DrawingSettings {
        self.drawing_settings
    }

    /// Stores validated authored data without changing existing pages or geometry.
    pub fn set_drawing_settings(&mut self, settings: DrawingSettings) {
        self.drawing_settings = settings;
    }

    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    /// Retained payload/container bytes, including reserved vector capacity, shared once.
    /// Excludes Arc/allocator headers and unrelated snapshots; saturates on overflow.
    pub fn retained_bytes<'a>(documents: impl IntoIterator<Item = &'a Self>) -> usize {
        let mut models = HashSet::new();
        let mut fragments = HashSet::new();
        let mut bytes = 0_usize;
        for document in documents {
            if !models.insert(std::ptr::from_ref(document)) {
                continue;
            }
            bytes = bytes
                .saturating_add(size_of::<Self>())
                .saturating_add(document.pages.capacity().saturating_mul(size_of::<Page>()));
            for page in &document.pages {
                bytes = bytes.saturating_add(
                    page.fragments
                        .capacity()
                        .saturating_mul(size_of::<Arc<Fragment>>()),
                );
                for fragment in &page.fragments {
                    if fragments.insert(Arc::as_ptr(fragment)) {
                        bytes = bytes.saturating_add(fragment.storage_bytes());
                    }
                }
            }
        }
        bytes
    }

    /// Append a batch atomically, validating document-wide identities and resource budgets.
    pub fn append_fragments(
        &mut self,
        page_id: PageId,
        fragments: Vec<Fragment>,
    ) -> Result<(), Error> {
        let index = self
            .pages
            .iter()
            .position(|page| page.id == page_id)
            .ok_or(Error::MissingPage(page_id.get()))?;
        if fragments.len() > MAX_GRAPH_ITEMS {
            return Err(GraphError::Limit("fragments").into());
        }
        let existing = self
            .pages
            .iter()
            .flat_map(|page| page.fragments.iter().map(Arc::as_ref));
        validate_fragments(existing.chain(fragments.iter()))?;
        self.pages
            .get_mut(index)
            .ok_or(Error::MissingPage(page_id.get()))?
            .fragments
            .extend(fragments.into_iter().map(Arc::new));
        Ok(())
    }

    /// Replace one same-ID fragment atomically, retaining every unrelated shared payload.
    pub fn replace_fragment(&mut self, page_id: PageId, fragment: Fragment) -> Result<(), Error> {
        self.replace_fragments(page_id, vec![fragment])
    }

    /// Replace a batch with one global identity/budget validation; retain page order.
    pub fn replace_fragments(
        &mut self,
        page_id: PageId,
        fragments: Vec<Fragment>,
    ) -> Result<(), Error> {
        let page_index = self
            .pages
            .iter()
            .position(|page| page.id == page_id)
            .ok_or(Error::MissingPage(page_id.get()))?;
        let page = self
            .pages
            .get(page_index)
            .ok_or(Error::MissingPage(page_id.get()))?;
        if fragments.len() > MAX_GRAPH_ITEMS {
            return Err(GraphError::Limit("fragments").into());
        }
        let ids: BTreeSet<_> = page.fragments.iter().map(|current| current.id()).collect();
        let mut replacements = BTreeMap::new();
        for fragment in fragments {
            let id = fragment.id();
            if !ids.contains(&id) {
                return Err(Error::MissingFragment(id.get()));
            }
            if replacements.insert(id, fragment).is_some() {
                return Err(GraphError::DuplicateFragment(id.get()).into());
            }
        }
        let candidate = self
            .pages
            .iter()
            .flat_map(|page| page.fragments.iter().map(Arc::as_ref))
            .map(|current| replacements.get(&current.id()).unwrap_or(current));
        validate_fragments(candidate)?;
        let page = self
            .pages
            .get_mut(page_index)
            .ok_or(Error::MissingPage(page_id.get()))?;
        for current in &mut page.fragments {
            if let Some(fragment) = replacements.remove(&current.id()) {
                *current = Arc::new(fragment);
            }
        }
        Ok(())
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

fn validate_fragments<'a>(fragments: impl Iterator<Item = &'a Fragment>) -> Result<(), Error> {
    let mut fragment_ids = BTreeSet::new();
    let mut atom_ids = BTreeSet::new();
    let mut bond_ids = BTreeSet::new();
    let (mut fragment_count, mut atom_count, mut bond_count) = (0_usize, 0_usize, 0_usize);
    for fragment in fragments {
        fragment_count = fragment_count.saturating_add(1);
        atom_count = atom_count.saturating_add(fragment.atoms().len());
        bond_count = bond_count.saturating_add(fragment.bonds().len());
        for (count, kind) in [
            (fragment_count, "fragments"),
            (atom_count, "atoms"),
            (bond_count, "bonds"),
        ] {
            if count > MAX_GRAPH_ITEMS {
                return Err(GraphError::Limit(kind).into());
            }
        }
        if !fragment_ids.insert(fragment.id()) {
            return Err(GraphError::DuplicateFragment(fragment.id().get()).into());
        }
        for atom in fragment.atoms() {
            if !atom_ids.insert(atom.id) {
                return Err(GraphError::DuplicateAtom(atom.id.get()).into());
            }
        }
        for bond in fragment.bonds() {
            if !bond_ids.insert(bond.id) {
                return Err(GraphError::DuplicateBond(bond.id.get()).into());
            }
        }
    }
    Ok(())
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
