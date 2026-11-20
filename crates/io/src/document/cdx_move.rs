//! Translate only the fixed-width coordinate words of a certified skeletal source.
use super::{CdxPreview, DocumentError, StoredDocument};
use crate::cdx::MappingError;
use bondline_cdx::Document;
use bondline_model::{Fragment, PageId, Point2};

const SCALE: f64 = 65_536.0;

/// Round the common displacement once so all selected positions move rigidly.
fn displacement(points: f64) -> Option<i64> {
    let units = (points * SCALE).round_ties_even();
    (units.abs() <= f64::from(u32::MAX)).then_some(units as i64)
}

impl StoredDocument {
    /// Move sorted, unique model atom IDs and re-import every resulting source byte.
    pub fn with_cdx_skeletal_translation(
        &self,
        page_id: PageId,
        ids: &[u32],
        dx: f64,
        dy: f64,
    ) -> Result<Self, DocumentError> {
        let fail =
            || DocumentError::UnsupportedCdxMove("one owned Node position for each selected atom");
        let (bytes, profile) = self
            .cdx_source_for(|p| p == CdxPreview::Skeletal)
            .ok_or_else(fail)?;
        let page = self
            .model
            .pages()
            .iter()
            .find(|p| p.id() == page_id)
            .ok_or_else(fail)?;
        if ids.is_empty()
            || ids.len() > bondline_render::MAX_HIT_REGIONS
            || ids.windows(2).any(|p| p[0] >= p[1])
        {
            return Err(fail());
        }
        let range_error =
            || DocumentError::UnsupportedCdxMove("coordinates within the signed 16.16 range");
        let dx = displacement(dx).ok_or_else(range_error)?;
        let dy = displacement(dy).ok_or_else(range_error)?;
        let source = Document::parse(bytes).map_err(MappingError::from)?;
        let mut positions = crate::cdx::skeletal_positions(&source);
        let mut output = None;
        let mut replacements = Vec::new();
        let mut found = 0;
        for fragment in page.fragments() {
            let mut changed_atoms = None;
            for (index, atom) in fragment.atoms().iter().enumerate() {
                let range = positions.next().ok_or_else(fail)?;
                let raw: &[u8; 8] = bytes
                    .get(range.clone())
                    .and_then(|v| v.try_into().ok())
                    .ok_or_else(fail)?;
                let [y, x] = raw.as_chunks::<4>().0.try_into().map_err(|_| fail())?;
                let (x, y) = (i32::from_le_bytes(x), i32::from_le_bytes(y));
                let point = |x: i32, y: i32| {
                    Point2::new(f64::from(x) / SCALE, f64::from(y) / SCALE)
                        .map_err(MappingError::from)
                };
                if atom.position != Some(point(x, y)?) {
                    return Err(fail());
                }
                if ids.binary_search(&atom.id.get()).is_err() {
                    continue;
                }
                found += 1;
                if dx == 0 && dy == 0 {
                    continue;
                }
                let x = i32::try_from(i64::from(x) + dx).map_err(|_| range_error())?;
                let y = i32::try_from(i64::from(y) + dy).map_err(|_| range_error())?;
                let data = output.get_or_insert_with(|| bytes.to_vec());
                let pair = data.get_mut(range).ok_or_else(fail)?;
                let (y_word, x_word) = pair.split_at_mut(4);
                y_word.copy_from_slice(&y.to_le_bytes());
                x_word.copy_from_slice(&x.to_le_bytes());
                let atoms = changed_atoms.get_or_insert_with(|| fragment.atoms().to_vec());
                atoms.get_mut(index).ok_or_else(fail)?.position = Some(point(x, y)?);
            }
            if let Some(atoms) = changed_atoms {
                replacements.push(
                    Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())
                        .map_err(MappingError::from)?,
                );
            }
        }
        if found != ids.len() || positions.next().is_some() {
            return Err(fail());
        }
        let Some(output) = output else {
            return Ok(self.clone());
        };
        let mut expected = self.model.as_ref().clone();
        expected
            .replace_fragments(page_id, replacements)
            .map_err(MappingError::from)?;
        self.finish_cdx_edit(
            output,
            expected,
            page.size(),
            profile,
            DocumentError::UnsupportedCdxMove(
                "a complete skeletal drawing within the original paper",
            ),
        )
    }
}
