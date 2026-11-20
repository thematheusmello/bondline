//! Translate only the fixed-width coordinate words of a certified skeletal source.
use super::{CdxPreview, DocumentError, StoredDocument};
use crate::cdx::MappingError;
use bondline_cdx::Document;
use bondline_model::{Fragment, PageId, Point2};
use std::{ops::Range, sync::Arc};

const SCALE: f64 = 65_536.0;

/// Round the common displacement once so all selected positions move rigidly.
fn displacement(points: f64) -> Option<i64> {
    let units = (points * SCALE).round_ties_even();
    (units.abs() <= f64::from(u32::MAX)).then_some(units as i64)
}

fn invalid_geometry() -> DocumentError {
    DocumentError::UnsupportedCdxMove(
        "each Node position and owned label box must match the current model",
    )
}

fn coordinate_overflow() -> DocumentError {
    DocumentError::UnsupportedCdxMove("coordinates within the signed 16.16 range")
}

/// Check the current field and replace its payload only; retain the original size header.
fn translate_field<const N: usize>(
    bytes: &[u8],
    output: &mut Option<Vec<u8>>,
    range: Range<usize>,
    expected: [f64; N],
    delta: [i64; N],
) -> Result<[f64; N], DocumentError> {
    let raw = bytes.get(range.clone()).ok_or_else(invalid_geometry)?;
    let (words, rest) = raw.as_chunks::<4>();
    if !rest.is_empty() || words.len() != N {
        return Err(invalid_geometry());
    }
    let mut changed = [[0; 4]; N];
    for (((raw, expected), shift), out) in words.iter().zip(expected).zip(delta).zip(&mut changed) {
        let value = i32::from_le_bytes(*raw);
        if f64::from(value) / SCALE != expected {
            return Err(invalid_geometry());
        }
        let moved = i32::try_from(i64::from(value) + shift).map_err(|_| coordinate_overflow())?;
        *out = moved.to_le_bytes();
    }
    if delta.iter().any(|d| *d != 0) {
        output
            .get_or_insert_with(|| bytes.to_vec())
            .get_mut(range)
            .ok_or_else(invalid_geometry)?
            .copy_from_slice(changed.as_flattened());
    }
    Ok(changed.map(|v| f64::from(i32::from_le_bytes(v)) / SCALE))
}

impl StoredDocument {
    pub(super) fn cdx_skeletal_source(&self) -> Option<(&[u8], CdxPreview)> {
        self.cdx_source_for(|p| matches!(p, CdxPreview::Skeletal | CdxPreview::SkeletalLabels))
    }

    /// Source-preserving translation for qualified unlabelled and mixed skeletal pages.
    pub fn can_move_cdx_skeletal_atoms(&self) -> bool {
        self.cdx_skeletal_source().is_some()
    }

    /// Move sorted, unique model atom IDs and re-import every resulting source byte.
    pub fn with_cdx_skeletal_translation(
        &self,
        page_id: PageId,
        ids: &[u32],
        dx: f64,
        dy: f64,
    ) -> Result<Self, DocumentError> {
        let (bytes, profile) = self.cdx_skeletal_source().ok_or_else(invalid_geometry)?;
        let page = self
            .model
            .pages()
            .iter()
            .find(|p| p.id() == page_id)
            .ok_or_else(invalid_geometry)?;
        if ids.is_empty()
            || ids.len() > bondline_render::MAX_HIT_REGIONS
            || ids.windows(2).any(|p| p[0] >= p[1])
        {
            return Err(invalid_geometry());
        }
        let dx = displacement(dx).ok_or_else(coordinate_overflow)?;
        let dy = displacement(dy).ok_or_else(coordinate_overflow)?;
        let source = Document::parse(bytes).map_err(MappingError::from)?;
        let mut positions = crate::cdx::skeletal_positions(&source);
        let mut boxes = crate::cdx::skeletal_label_bounds(&source).peekable();
        let mut output = None;
        let mut replacements = Vec::new();
        let mut found = 0;
        for fragment in page.fragments() {
            let mut changed_atoms = None;
            for (index, atom) in fragment.atoms().iter().enumerate() {
                let (node, range) = positions.next().ok_or_else(invalid_geometry)?;
                let selected = ids.binary_search(&atom.id.get()).is_ok();
                found += usize::from(selected);
                let [y_shift, x_shift] = if selected { [dy, dx] } else { [0, 0] };
                let position = atom.position.ok_or_else(invalid_geometry)?;
                let [y, x] = translate_field(
                    bytes,
                    &mut output,
                    range,
                    [position.y_pt(), position.x_pt()],
                    [y_shift, x_shift],
                )?;
                let bounds = boxes.next_if(|(owner, _)| *owner == node);
                let moved_box = match (bounds, atom.authored_label.as_ref()) {
                    (None, None) => None,
                    (Some((_, range)), Some(label)) => {
                        let [l, t, r, b] = label.bounding_box().ok_or_else(invalid_geometry)?;
                        let [t, l, b, r] = translate_field(
                            bytes,
                            &mut output,
                            range,
                            [t, l, b, r],
                            [y_shift, x_shift, y_shift, x_shift],
                        )?;
                        Some([l, t, r, b])
                    }
                    _ => return Err(invalid_geometry()),
                };
                if y_shift == 0 && x_shift == 0 {
                    continue;
                }
                let atoms = changed_atoms.get_or_insert_with(|| fragment.atoms().to_vec());
                let target = atoms.get_mut(index).ok_or_else(invalid_geometry)?;
                target.position = Some(Point2::new(x, y).map_err(MappingError::from)?);
                if let (Some(label), Some(bounds)) = (&atom.authored_label, moved_box) {
                    target.authored_label = Some(Arc::new(
                        label
                            .with_bounding_box(Some(bounds))
                            .map_err(|_| invalid_geometry())?,
                    ));
                }
            }
            if let Some(atoms) = changed_atoms {
                replacements.push(
                    Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())
                        .map_err(MappingError::from)?,
                );
            }
        }
        if found != ids.len() || positions.next().is_some() || boxes.next().is_some() {
            return Err(invalid_geometry());
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
