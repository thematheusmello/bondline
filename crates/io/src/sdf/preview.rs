//! Derived all-record geometry and display-H facts; never authored data or Save authority.

use super::{RecordId, SdfDocument};
use crate::mol::{MolError, MolFile, MolTransform, Version, v3000};
use bondline_model::{
    AtomId, BondId, BondOrder, Document, Fragment, FragmentId, MAX_GRAPH_ITEMS,
    MAX_PAGE_DIMENSION_PT, Page, PageId, PageSize, Point2, Radical,
};
use bondline_render::Cancellation;
use std::collections::BTreeMap;

/// The origin of a count supplied to a display, without rewriting the source atom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HydrogenOrigin {
    Literal,
    NeutralDefaultV1,
    PhysicalHydrogenV1,
}

/// Local IDs are the existing MOL mapper's dense IDs, not V3000 source indexes.
/// Map keys are preview IDs; values map back to the original per-record local IDs.
#[derive(Debug)]
pub struct PreviewRecord {
    record_id: RecordId,
    fragment_id: FragmentId,
    atom_mapping: BTreeMap<AtomId, AtomId>,
    bond_mapping: BTreeMap<BondId, BondId>,
}

impl PreviewRecord {
    pub const fn record_id(&self) -> RecordId {
        self.record_id
    }

    pub const fn fragment_id(&self) -> FragmentId {
        self.fragment_id
    }

    pub fn atom_mapping(&self) -> &BTreeMap<AtomId, AtomId> {
        &self.atom_mapping
    }

    pub fn bond_mapping(&self) -> &BTreeMap<BondId, BondId> {
        &self.bond_mapping
    }
}

/// Ephemeral data only. The caller must retain the complete SDF source separately.
/// Atom bounds do not certify glyph/stroke clearance, renderer support or readability.
pub struct SdfPreview {
    page: Page,
    display_hydrogens: BTreeMap<AtomId, u16>,
    hydrogen_origins: BTreeMap<AtomId, HydrogenOrigin>,
    records: Vec<PreviewRecord>,
}

impl std::fmt::Debug for SdfPreview {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("SdfPreview")
            .field("record_count", &self.records.len())
            .field("atom_count", &self.display_hydrogens.len())
            .finish()
    }
}

impl SdfPreview {
    pub fn page(&self) -> &Page {
        &self.page
    }

    pub fn display_hydrogens(&self) -> &BTreeMap<AtomId, u16> {
        &self.display_hydrogens
    }

    pub fn hydrogen_origins(&self) -> &BTreeMap<AtomId, HydrogenOrigin> {
        &self.hydrogen_origins
    }

    pub fn records(&self) -> &[PreviewRecord] {
        &self.records
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PreviewError {
    #[error("SDF preview cancelled")]
    Cancelled,
    #[error("invalid retained MOL span for SDF record {record_id}")]
    InvalidSpan { record_id: u32 },
    #[error("MOL projection in SDF record {record_id}: {source}")]
    Mol {
        record_id: u32,
        #[source]
        source: MolError,
    },
    #[error("V3000 projection in SDF record {record_id}: {source}")]
    MolV3000 {
        record_id: u32,
        #[source]
        source: v3000::Error,
    },
    #[error("SDF record {record_id} cannot be previewed: {feature}")]
    Unsupported {
        record_id: u32,
        feature: &'static str,
    },
    #[error("SDF preview exceeds {kind} while processing record {record_id}")]
    Limit { record_id: u32, kind: &'static str },
    #[error("invalid SDF preview model at record {record_id}: {source}")]
    Model {
        record_id: u32,
        #[source]
        source: bondline_model::Error,
    },
}

impl PreviewError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::Unsupported { .. } => "unsupported_content",
            Self::Limit { .. }
            | Self::Model {
                source: bondline_model::Error::Graph(bondline_model::GraphError::Limit(_)),
                ..
            } => "resource_limit",
            Self::Mol { source, .. } => source.code(),
            Self::MolV3000 { source, .. } => source.code(),
            Self::InvalidSpan { .. } | Self::Model { .. } => "invalid_document",
        }
    }

    pub fn record_id(&self) -> Option<RecordId> {
        match self {
            Self::Cancelled => None,
            Self::InvalidSpan { record_id }
            | Self::Mol { record_id, .. }
            | Self::MolV3000 { record_id, .. }
            | Self::Unsupported { record_id, .. }
            | Self::Limit { record_id, .. }
            | Self::Model { record_id, .. } => RecordId::new(*record_id),
        }
    }
}

fn check_cancel(cancel: &Cancellation) -> Result<(), PreviewError> {
    if cancel.is_cancelled() {
        Err(PreviewError::Cancelled)
    } else {
        Ok(())
    }
}

fn unsupported(record_id: u32, feature: &'static str) -> PreviewError {
    PreviewError::Unsupported { record_id, feature }
}

fn total(
    old: usize,
    added: usize,
    record_id: u32,
    kind: &'static str,
) -> Result<usize, PreviewError> {
    old.checked_add(added)
        .filter(|count| *count <= MAX_GRAPH_ITEMS)
        .ok_or(PreviewError::Limit { record_id, kind })
}

struct Bonding {
    sum: u32,
    degree: usize,
    non_single: bool,
    position: Point2,
}

fn hydrogens(
    fragment: &Fragment,
    record_id: u32,
    cancel: &Cancellation,
) -> Result<BTreeMap<AtomId, (u16, HydrogenOrigin)>, PreviewError> {
    let mut neighbours = BTreeMap::new();
    for atom in fragment.atoms() {
        check_cancel(cancel)?;
        if atom.charge != 0 || atom.isotope.is_some() || atom.radical != Radical::None {
            return Err(unsupported(
                record_id,
                "charge, isotope or radical annotations",
            ));
        }
        neighbours.insert(
            atom.id,
            Bonding {
                sum: 0,
                degree: 0,
                non_single: false,
                position: atom
                    .position
                    .ok_or_else(|| unsupported(record_id, "missing atom coordinates"))?,
            },
        );
    }
    for bond in fragment.bonds() {
        check_cancel(cancel)?;
        let weight = match bond.order {
            BondOrder::Single => 1,
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            _ => {
                return Err(unsupported(
                    record_id,
                    "bond order outside Single/Double/Triple",
                ));
            }
        };
        let begin = neighbours
            .get(&bond.begin)
            .ok_or_else(|| unsupported(record_id, "missing bond endpoint"))?;
        let end = neighbours
            .get(&bond.end)
            .ok_or_else(|| unsupported(record_id, "missing bond endpoint"))?;
        if begin.position == end.position {
            return Err(unsupported(record_id, "coincident bond endpoints"));
        }
        for id in [bond.begin, bond.end] {
            let bonding = neighbours
                .get_mut(&id)
                .ok_or_else(|| unsupported(record_id, "missing bond endpoint"))?;
            bonding.sum = bonding.sum.checked_add(weight).ok_or(PreviewError::Limit {
                record_id,
                kind: "bond-order sum",
            })?;
            bonding.degree = bonding.degree.checked_add(1).ok_or(PreviewError::Limit {
                record_id,
                kind: "atom degree",
            })?;
            bonding.non_single |= bond.order != BondOrder::Single;
        }
    }
    let mut output = BTreeMap::new();
    for atom in fragment.atoms() {
        check_cancel(cancel)?;
        let bonding = neighbours
            .get(&atom.id)
            .ok_or_else(|| unsupported(record_id, "missing atom topology"))?;
        let resolved = if atom.element.get() == 1 {
            if bonding.degree != 1
                || bonding.sum != 1
                || bonding.non_single
                || atom.explicit_hydrogens.is_some_and(|h| h != 0)
            {
                return Err(unsupported(
                    record_id,
                    "physical H requires one Single bond and no attached H",
                ));
            }
            (
                0,
                if atom.explicit_hydrogens.is_some() {
                    HydrogenOrigin::Literal
                } else {
                    HydrogenOrigin::PhysicalHydrogenV1
                },
            )
        } else if let Some(count) = atom.explicit_hydrogens {
            (count, HydrogenOrigin::Literal)
        } else {
            // Original closed display policy, not general periodic-table/CTfile valence.
            let target: u32 = match atom.element.get() {
                6 => 4,
                7 => 3,
                8 => 2,
                9 | 17 | 35 | 53 => 1,
                _ => {
                    return Err(unsupported(
                        record_id,
                        "unresolved H outside neutral C/N/O/halogen profile",
                    ));
                }
            };
            let count = target.checked_sub(bonding.sum).ok_or_else(|| {
                unsupported(record_id, "bond sum exceeds neutral default valence")
            })?;
            (
                u16::try_from(count).map_err(|_| PreviewError::Limit {
                    record_id,
                    kind: "display H count",
                })?,
                HydrogenOrigin::NeutralDefaultV1,
            )
        };
        output.insert(atom.id, resolved);
    }
    Ok(output)
}

/// Validate and arrange every strictly projectable record, or fail without a partial view.
/// Counts remain a display sidecar; authored Some/None and every source byte stay exact.
/// The original packing uses a shared 24pt/unit frame and record translation only.
/// This data prerequisite does not certify renderer support, ink bounds or viewport scale.
pub fn prepare_sdf_preview(
    document: &SdfDocument,
    cancel: &Cancellation,
) -> Result<SdfPreview, PreviewError> {
    check_cancel(cancel)?;
    let first = document
        .records()
        .first()
        .ok_or(PreviewError::InvalidSpan { record_id: 1 })?
        .id()
        .get();
    let origin = Point2::new(72.0, 72.0).map_err(|source| PreviewError::Model {
        record_id: first,
        source,
    })?;
    let frame = MolTransform::new(24.0, origin).map_err(|source| PreviewError::Mol {
        record_id: first,
        source,
    })?;
    let mut fragments = Vec::with_capacity(document.records().len());
    let mut records = Vec::with_capacity(document.records().len());
    let mut display_hydrogens = BTreeMap::new();
    let mut hydrogen_origins = BTreeMap::new();
    let (mut atom_count, mut bond_count) = (0, 0);
    let (mut width, mut height) = (PageSize::A4.width_pt(), PageSize::A4.height_pt());
    let mut top = 72.0;
    let mut last_record = first;
    for record in document.records() {
        check_cancel(cancel)?;
        let id = record.id().get();
        last_record = id;
        let atom_offset = u32::try_from(atom_count).map_err(|_| PreviewError::Limit {
            record_id: id,
            kind: "atom IDs",
        })?;
        let bond_offset = u32::try_from(bond_count).map_err(|_| PreviewError::Limit {
            record_id: id,
            kind: "bond IDs",
        })?;
        let bytes = document
            .as_bytes()
            .get(record.mol_range())
            .ok_or(PreviewError::InvalidSpan { record_id: id })?;
        // These existing bounded parser/mapper calls are not internally cancellable.
        let fragment = match record.mol_version() {
            Version::V2000 => {
                let source = MolFile::parse(bytes).map_err(|source| PreviewError::Mol {
                    record_id: id,
                    source,
                })?;
                atom_count = total(atom_count, source.atoms().len(), id, "total atoms")?;
                bond_count = total(bond_count, source.bonds().len(), id, "total bonds")?;
                check_cancel(cancel)?;
                source
                    .to_fragment(frame)
                    .map_err(|source| PreviewError::Mol {
                        record_id: id,
                        source,
                    })?
            }
            Version::V3000 => {
                let source =
                    v3000::MolFile::parse(bytes).map_err(|source| PreviewError::MolV3000 {
                        record_id: id,
                        source,
                    })?;
                atom_count = total(atom_count, source.atoms().len(), id, "total atoms")?;
                bond_count = total(bond_count, source.bonds().len(), id, "total bonds")?;
                check_cancel(cancel)?;
                source
                    .to_fragment(frame)
                    .map_err(|source| PreviewError::MolV3000 {
                        record_id: id,
                        source,
                    })?
            }
        };
        check_cancel(cancel)?;
        if fragment.atoms().is_empty() {
            return Err(unsupported(id, "empty molecular record"));
        }
        let resolved = hydrogens(&fragment, id, cancel)?;
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for atom in fragment.atoms() {
            check_cancel(cancel)?;
            let point = atom
                .position
                .ok_or_else(|| unsupported(id, "missing atom coordinates"))?;
            min_x = min_x.min(point.x_pt());
            max_x = max_x.max(point.x_pt());
            min_y = min_y.min(point.y_pt());
            max_y = max_y.max(point.y_pt());
        }
        let needed_width = max_x - min_x + 144.0;
        let needed_height = top + max_y - min_y + 72.0;
        if [needed_width, needed_height]
            .iter()
            .any(|value| !value.is_finite() || *value > MAX_PAGE_DIMENSION_PT)
        {
            return Err(PreviewError::Limit {
                record_id: id,
                kind: "page dimensions",
            });
        }
        width = width.max(needed_width);
        height = height.max(needed_height);
        let fragment_id = FragmentId::new(id).ok_or(PreviewError::Limit {
            record_id: id,
            kind: "fragment IDs",
        })?;
        let mut mapping = PreviewRecord {
            record_id: record.id(),
            fragment_id,
            atom_mapping: BTreeMap::new(),
            bond_mapping: BTreeMap::new(),
        };
        let mut atoms = Vec::with_capacity(fragment.atoms().len());
        for original in fragment.atoms() {
            check_cancel(cancel)?;
            let mut atom = original.clone();
            atom.id = atom_offset
                .checked_add(original.id.get())
                .and_then(AtomId::new)
                .ok_or(PreviewError::Limit {
                    record_id: id,
                    kind: "atom IDs",
                })?;
            let position = original
                .position
                .ok_or_else(|| unsupported(id, "missing atom coordinates"))?;
            atom.position = Some(
                Point2::new(
                    72.0 + position.x_pt() - min_x,
                    top + position.y_pt() - min_y,
                )
                .map_err(|source| PreviewError::Model {
                    record_id: id,
                    source,
                })?,
            );
            mapping.atom_mapping.insert(atom.id, original.id);
            let (count, origin) = resolved
                .get(&original.id)
                .copied()
                .ok_or_else(|| unsupported(id, "missing resolved H count"))?;
            display_hydrogens.insert(atom.id, count);
            hydrogen_origins.insert(atom.id, origin);
            atoms.push(atom);
        }
        let mut bonds = Vec::with_capacity(fragment.bonds().len());
        for original in fragment.bonds() {
            check_cancel(cancel)?;
            let mut bond = *original;
            bond.id = bond_offset
                .checked_add(original.id.get())
                .and_then(BondId::new)
                .ok_or(PreviewError::Limit {
                    record_id: id,
                    kind: "bond IDs",
                })?;
            bond.begin = atom_offset
                .checked_add(original.begin.get())
                .and_then(AtomId::new)
                .ok_or(PreviewError::Limit {
                    record_id: id,
                    kind: "atom IDs",
                })?;
            bond.end = atom_offset
                .checked_add(original.end.get())
                .and_then(AtomId::new)
                .ok_or(PreviewError::Limit {
                    record_id: id,
                    kind: "atom IDs",
                })?;
            mapping.bond_mapping.insert(bond.id, original.id);
            bonds.push(bond);
        }
        fragments.push(Fragment::new(fragment_id, atoms, bonds).map_err(|source| {
            PreviewError::Model {
                record_id: id,
                source: source.into(),
            }
        })?);
        records.push(mapping);
        top = needed_height;
    }
    check_cancel(cancel)?;
    let mut model = Document::default();
    let page_id = PageId::new(1).ok_or(PreviewError::InvalidSpan {
        record_id: last_record,
    })?;
    let size = PageSize::new(width, height).map_err(|source| PreviewError::Model {
        record_id: last_record,
        source,
    })?;
    model
        .set_page_size(page_id, size)
        .map_err(|source| PreviewError::Model {
            record_id: last_record,
            source,
        })?;
    model
        .append_fragments(page_id, fragments)
        .map_err(|source| PreviewError::Model {
            record_id: last_record,
            source,
        })?;
    check_cancel(cancel)?;
    let page = model
        .pages()
        .first()
        .cloned()
        .ok_or(PreviewError::InvalidSpan {
            record_id: last_record,
        })?;
    Ok(SdfPreview {
        page,
        display_hydrogens,
        hydrogen_origins,
        records,
    })
}
