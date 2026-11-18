//! Source-local records mapped to complete, globally identified drawing fragments.

use super::{
    AuthoringError, LayoutError, RingClosure, SmilesFileDocument, SmilesFileError, components,
    file::RecordId, validate_authoring_page,
};
use bondline_model::{
    AtomId, BondId, BondOrder, Document, Fragment, FragmentId, PageId, PageSize, Point2,
};
use bondline_render::Cancellation;
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    sync::Arc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordSelection {
    Record(RecordId),
    All,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("SMILES record projection cancelled")]
    Cancelled,
    #[error("SMILES record projection exceeds {0}")]
    Limit(&'static str),
    #[error("SMILES record projection refuses {0}")]
    Unsupported(&'static str),
    #[error("invalid SMILES record projection: {0}")]
    Invalid(&'static str),
    #[error(transparent)]
    File(#[from] SmilesFileError),
    #[error(transparent)]
    Authoring(#[from] AuthoringError),
    #[error(transparent)]
    Chemistry(#[from] bondline_chem::AuthoringError),
    #[error(transparent)]
    Layout(#[from] LayoutError),
    #[error(transparent)]
    Write(#[from] super::WriteError),
    #[error(transparent)]
    Model(#[from] bondline_model::Error),
}

impl ProjectionError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::Limit(_) => "resource_limit",
            Self::Unsupported(_) => "unsupported_content",
            Self::Invalid(_) => "invalid_document",
            Self::File(error) => error.code(),
            Self::Authoring(error) => error.code(),
            Self::Chemistry(error) => error.code(),
            Self::Layout(error) => error.code(),
            Self::Write(error) => error.code(),
            Self::Model(
                bondline_model::Error::MissingPage(_) | bondline_model::Error::MissingFragment(_),
            ) => "not_found",
            Self::Model(
                bondline_model::Error::InvalidPageSize | bondline_model::Error::InvalidPosition,
            ) => "invalid_params",
            Self::Model(bondline_model::Error::Graph(bondline_model::GraphError::Limit(_))) => {
                "resource_limit"
            }
            Self::Model(_) => "invalid_document",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmilesAtomMapping {
    pub atom_id: AtomId,
    /// Zero-based atom ordinal in this notation's current IR.
    pub notation_atom: usize,
    pub range: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmilesBondMapping {
    pub bond_id: BondId,
    /// Zero-based bond ordinal in this notation's current IR.
    pub notation_bond: usize,
    /// Current notation direction, which may differ from the authored bond's direction.
    pub atoms: [AtomId; 2],
    pub bond_range: Option<Range<usize>>,
    pub ring: Option<RingClosure>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmilesRecordMapping {
    record_id: RecordId,
    fragment_id: FragmentId,
    atoms: Vec<SmilesAtomMapping>,
    bonds: Vec<SmilesBondMapping>,
}

impl SmilesRecordMapping {
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }
    pub fn fragment_id(&self) -> FragmentId {
        self.fragment_id
    }
    pub fn atoms(&self) -> &[SmilesAtomMapping] {
        &self.atoms
    }
    pub fn bonds(&self) -> &[SmilesBondMapping] {
        &self.bonds
    }
    pub(crate) fn reserved_bytes(&self) -> usize {
        self.atoms
            .capacity()
            .saturating_mul(size_of::<SmilesAtomMapping>())
            .saturating_add(
                self.bonds
                    .capacity()
                    .saturating_mul(size_of::<SmilesBondMapping>()),
            )
    }
}

pub(crate) fn check(cancel: &Cancellation) -> Result<(), ProjectionError> {
    if cancel.is_cancelled() {
        Err(ProjectionError::Cancelled)
    } else {
        Ok(())
    }
}

fn number(index: usize) -> Result<u32, ProjectionError> {
    index
        .checked_add(1)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(ProjectionError::Limit("identity space"))
}

struct RecordPlan {
    record: RecordId,
    fragment: FragmentId,
    atoms: Vec<AtomId>,
    bonds: Vec<BondId>,
}

pub(crate) fn project(
    source: &SmilesFileDocument,
    selection: RecordSelection,
    length: f64,
    origin: Point2,
    page: PageSize,
    cancel: &Cancellation,
) -> Result<(Document, Arc<[SmilesRecordMapping]>), ProjectionError> {
    check(cancel)?;
    if !length.is_finite() || length <= 0.0 {
        return Err(LayoutError::InvalidBondLength.into());
    }
    let records = match selection {
        RecordSelection::Record(id) => {
            source
                .record(id)
                .ok_or(SmilesFileError::MissingRecord { record_id: id })?;
            vec![id]
        }
        RecordSelection::All => {
            if source.records().is_empty() {
                return Err(ProjectionError::Unsupported("empty drawing selection"));
            }
            if source.records().len() > bondline_chem::MAX_AUTHORING_ATOMS {
                return Err(ProjectionError::Limit("64 selected atoms"));
            }
            source.records().iter().map(|record| record.id()).collect()
        }
    };
    let (mut atom_count, mut bond_count) = (0_usize, 0_usize);
    for &id in &records {
        check(cancel)?;
        let record = source
            .record(id)
            .ok_or(SmilesFileError::MissingRecord { record_id: id })?;
        atom_count = atom_count
            .checked_add(record.atom_count())
            .filter(|n| *n <= bondline_chem::MAX_AUTHORING_ATOMS)
            .ok_or(ProjectionError::Limit("64 selected atoms"))?;
        bond_count = bond_count
            .checked_add(record.bond_count())
            .filter(|n| *n <= bondline_chem::MAX_AUTHORING_BONDS)
            .ok_or(ProjectionError::Limit("128 selected bonds"))?;
    }
    let mut atoms = Vec::with_capacity(atom_count);
    let mut bonds = Vec::with_capacity(bond_count);
    let mut semantics = Vec::with_capacity(atom_count);
    let mut plans = Vec::with_capacity(records.len());
    for id in records {
        check(cancel)?;
        let owner = source.read_record(id)?;
        let (raw, intent) = owner.authoring_input(cancel)?;
        let mut atom_ids = Vec::with_capacity(raw.atoms().len());
        let mut local = BTreeMap::new();
        for atom in raw.atoms() {
            check(cancel)?;
            let mut atom = atom.clone();
            let global = AtomId::new(number(atoms.len())?)
                .ok_or(ProjectionError::Limit("atom identities"))?;
            local.insert(atom.id, global);
            atom.id = global;
            atom_ids.push(global);
            atoms.push(atom);
        }
        semantics.extend(intent);
        let mut bond_ids = Vec::with_capacity(raw.bonds().len());
        for bond in raw.bonds() {
            check(cancel)?;
            let mut bond = *bond;
            bond.id = BondId::new(number(bonds.len())?)
                .ok_or(ProjectionError::Limit("bond identities"))?;
            bond.begin = *local
                .get(&bond.begin)
                .ok_or(ProjectionError::Invalid("missing local endpoint"))?;
            bond.end = *local
                .get(&bond.end)
                .ok_or(ProjectionError::Invalid("missing local endpoint"))?;
            bond_ids.push(bond.id);
            bonds.push(bond);
        }
        plans.push(RecordPlan {
            record: id,
            fragment: FragmentId::new(number(plans.len())?)
                .ok_or(ProjectionError::Limit("fragment identities"))?,
            atoms: atom_ids,
            bonds: bond_ids,
        });
    }
    let aggregate = Fragment::new(
        FragmentId::new(1).ok_or(ProjectionError::Limit("fragment identity"))?,
        atoms,
        bonds,
    )
    .map_err(bondline_model::Error::from)?;
    let normalized =
        bondline_chem::normalize_neutral_kekule(&aggregate, &semantics, || cancel.is_cancelled())?;
    let drawing = components::layout_fragment(&normalized, length, origin, page, cancel, true)?;
    let aggregate = drawing
        .pages()
        .first()
        .and_then(|p| p.fragments().first())
        .ok_or(ProjectionError::Invalid("missing aggregate layout"))?;
    let mut fragments = Vec::with_capacity(plans.len());
    for plan in &plans {
        check(cancel)?;
        let atoms = aggregate
            .atoms()
            .iter()
            .filter(|atom| plan.atoms.contains(&atom.id))
            .cloned()
            .collect();
        let bonds = aggregate
            .bonds()
            .iter()
            .filter(|bond| plan.bonds.contains(&bond.id))
            .copied()
            .collect();
        fragments
            .push(Fragment::new(plan.fragment, atoms, bonds).map_err(bondline_model::Error::from)?);
    }
    let mut document = Document::default();
    let page_id = PageId::new(1).ok_or(ProjectionError::Invalid("page identity"))?;
    document.set_page_size(page_id, page)?;
    document.append_fragments(page_id, fragments)?;
    validate_authoring_page(
        document
            .pages()
            .first()
            .ok_or(ProjectionError::Invalid("missing page"))?,
        cancel,
    )?;
    let mut maps = Vec::with_capacity(plans.len());
    for plan in plans {
        let fragment = document
            .pages()
            .first()
            .and_then(|p| p.fragments().iter().find(|f| f.id() == plan.fragment))
            .ok_or(ProjectionError::Invalid("missing split fragment"))?;
        maps.push(mapping(source, plan.record, fragment, &plan.atoms, cancel)?);
    }
    check(cancel)?;
    Ok((document, maps.into()))
}

fn absolute(
    range: Range<usize>,
    start: usize,
    limit: usize,
) -> Result<Range<usize>, ProjectionError> {
    let begin = start
        .checked_add(range.start)
        .ok_or(ProjectionError::Limit("source offsets"))?;
    let end = start
        .checked_add(range.end)
        .filter(|end| *end <= limit)
        .ok_or(ProjectionError::Invalid("source span outside file"))?;
    if begin > end {
        return Err(ProjectionError::Invalid("reversed source span"));
    }
    Ok(begin..end)
}

pub(crate) fn mapping(
    source: &SmilesFileDocument,
    id: RecordId,
    fragment: &Fragment,
    atom_ids: &[AtomId],
    cancel: &Cancellation,
) -> Result<SmilesRecordMapping, ProjectionError> {
    check(cancel)?;
    let metadata = source
        .record(id)
        .ok_or(SmilesFileError::MissingRecord { record_id: id })?;
    let start = metadata.notation_range().start;
    let owner = source.read_record(id)?;
    if owner.atoms().len() != atom_ids.len()
        || atom_ids.len() != fragment.atoms().len()
        || owner.bonds().len() != fragment.bonds().len()
    {
        return Err(ProjectionError::Invalid("incomplete record mapping"));
    }
    let mut seen = BTreeSet::new();
    let mut atoms = Vec::with_capacity(atom_ids.len());
    for (index, (raw, &atom_id)) in owner.atoms().iter().zip(atom_ids).enumerate() {
        check(cancel)?;
        if !seen.insert(atom_id) {
            return Err(ProjectionError::Invalid("duplicate mapped atom"));
        }
        let atom = fragment
            .atoms()
            .iter()
            .find(|atom| atom.id == atom_id)
            .ok_or(ProjectionError::Invalid(
                "mapped atom missing from fragment",
            ))?;
        if atom.element != raw.element
            || atom.charge != raw.charge
            || atom.isotope.map(|mass| mass.get()) != raw.isotope
            || matches!(raw.hydrogens,super::Hydrogens::Bracket(h) if atom.explicit_hydrogens!=Some(u16::from(h)))
        {
            return Err(ProjectionError::Invalid("mapped atom chemistry differs"));
        }
        atoms.push(SmilesAtomMapping {
            atom_id,
            notation_atom: index,
            range: absolute(raw.range.clone(), start, source.as_bytes().len())?,
        });
    }
    let mut bonds = Vec::with_capacity(owner.bonds().len());
    for (index, raw) in owner.bonds().iter().enumerate() {
        check(cancel)?;
        let endpoints = [
            *atom_ids
                .get(raw.atoms[0])
                .ok_or(ProjectionError::Invalid("mapped endpoint"))?,
            *atom_ids
                .get(raw.atoms[1])
                .ok_or(ProjectionError::Invalid("mapped endpoint"))?,
        ];
        let bond = fragment
            .bonds()
            .iter()
            .find(|bond| [bond.begin, bond.end] == endpoints || [bond.end, bond.begin] == endpoints)
            .ok_or(ProjectionError::Invalid("mapped bond missing"))?;
        if !match raw.order {
            super::BondOrder::Single => bond.order == BondOrder::Single,
            super::BondOrder::Double => bond.order == BondOrder::Double,
            super::BondOrder::Triple => bond.order == BondOrder::Triple,
            super::BondOrder::Aromatic => {
                matches!(bond.order, BondOrder::Single | BondOrder::Double)
            }
        } {
            return Err(ProjectionError::Invalid("mapped bond order differs"));
        }
        let offset = |range: Range<usize>| absolute(range, start, source.as_bytes().len());
        let ring = raw
            .ring
            .as_ref()
            .map(|ring| {
                Ok::<_, ProjectionError>(RingClosure {
                    label: ring.label,
                    open_range: offset(ring.open_range.clone())?,
                    close_range: offset(ring.close_range.clone())?,
                    open_bond_range: ring.open_bond_range.clone().map(offset).transpose()?,
                    close_bond_range: ring.close_bond_range.clone().map(offset).transpose()?,
                })
            })
            .transpose()?;
        bonds.push(SmilesBondMapping {
            bond_id: bond.id,
            notation_bond: index,
            atoms: endpoints,
            bond_range: raw.bond_range.clone().map(offset).transpose()?,
            ring,
        });
    }
    Ok(SmilesRecordMapping {
        record_id: id,
        fragment_id: fragment.id(),
        atoms,
        bonds,
    })
}
