//! One deferred append of a complete independently laid-out SMILES graph.

use crate::{Cancellation, CloseState, CommandResult, Engine, Error, StoredContent, bond::Ids};
use bondline_io::{StoredDocument, smiles::SmilesDocument};
use bondline_model::{AtomId, BondId, Fragment, FragmentId, PageId, PageSize, Point2};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

// shortcut: cap the identity scan at 100000 items; expand with large-document authoring budgets.
const MAX_SCAN_ITEMS: usize = 100_000;

#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PasteSmiles {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(length(max = 65_536))]
    pub notation: String,
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub bond_length_pt: f64,
    pub origin_x_pt: f64,
    pub origin_y_pt: f64,
}

impl std::fmt::Debug for PasteSmiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasteSmiles")
            .field("expected_revision", &self.expected_revision)
            .field("page_id", &self.page_id)
            .field("notation_bytes", &self.notation.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub(super) struct PastePlan {
    input: Arc<StoredContent>,
    params: PasteSmiles,
    page: PageSize,
    generation: u64,
    close_epoch: u64,
    floor: Ids,
}

#[derive(Debug)]
pub(super) struct PasteCandidate {
    input: Arc<StoredContent>,
    candidate: StoredDocument,
    revision: u64,
    generation: u64,
    close_epoch: u64,
    floor: Ids,
    cancel: Cancellation,
}

impl Engine {
    pub(super) fn smiles_paste_plan(&self, params: PasteSmiles) -> Result<PastePlan, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if self.close_state() != CloseState::Open {
            return Err(Error::UiBusy);
        }
        if params.notation.len() > bondline_io::smiles::MAX_SMILES_BYTES {
            return Err(Error::SmilesPasteLimit("notation exceeds 64 KiB"));
        }
        if !params.bond_length_pt.is_finite() || params.bond_length_pt <= 0.0 {
            return Err(Error::InvalidSmilesPaste(
                "bond length must be finite and positive",
            ));
        }
        Point2::new(params.origin_x_pt, params.origin_y_pt)?;
        let stored = self.stored_document()?;
        if !stored.is_fully_mapped()
            || stored
                .source_info()
                .is_some_and(|source| source.format != "cdxml")
        {
            return Err(Error::UnsupportedSmilesPasteSource);
        }
        let id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
        let page = stored
            .model()
            .pages()
            .iter()
            .find(|page| page.id() == id)
            .ok_or(bondline_model::Error::MissingPage(id.get()))?
            .size();
        Ok(PastePlan {
            input: Arc::clone(&self.document),
            params,
            page,
            generation: self.generation,
            close_epoch: self.close_policy.epoch,
            floor: self.identities,
        })
    }

    pub(super) fn finish_smiles_paste(
        &mut self,
        completed: PasteCandidate,
    ) -> Result<CommandResult, Error> {
        completed.cancel.check()?;
        if self.close_state() == CloseState::Committed {
            return Err(Error::CloseCommitted);
        }
        if completed.revision != self.revision
            || completed.generation != self.generation
            || !Arc::ptr_eq(&completed.input, &self.document)
        {
            return Err(Error::StaleRevision);
        }
        if self.close_state() != CloseState::Open
            || self.close_policy.epoch != completed.close_epoch
        {
            return Err(Error::UiStaleRequest);
        }
        let changed = self.commit(completed.candidate)?;
        if changed {
            self.identities = completed.floor;
        }
        Ok(CommandResult {
            changed,
            ..self.state()
        })
    }
}

impl PastePlan {
    pub(super) fn complete(self, cancel: &Cancellation) -> Result<PasteCandidate, Error> {
        cancel.check()?;
        let notation = SmilesDocument::parse(Arc::from(self.params.notation.as_bytes()))?;
        let origin = Point2::new(self.params.origin_x_pt, self.params.origin_y_pt)?;
        let drawing =
            notation.layout_authoring(self.params.bond_length_pt, origin, self.page, cancel)?;
        let [page] = drawing.pages() else {
            return Err(Error::InvalidSmilesPaste(
                "layout must return one complete page",
            ));
        };
        let [fragment] = page.fragments() else {
            return Err(Error::InvalidSmilesPaste(
                "layout must return one complete fragment",
            ));
        };
        let stored = self.input.drawing()?;
        let (fragment, floor) = remap(stored, fragment, self.floor, cancel)?;
        let page_id = PageId::new(self.params.page_id).ok_or(Error::InvalidPageId)?;
        let mut candidate = stored.clone();
        candidate
            .model_mut()
            .append_fragments(page_id, vec![fragment])?;
        let page = candidate
            .model()
            .pages()
            .iter()
            .find(|page| page.id() == page_id)
            .ok_or(bondline_model::Error::MissingPage(page_id.get()))?;
        bondline_io::smiles::validate_authoring_page(page, cancel)?;
        cancel.check()?;
        Ok(PasteCandidate {
            input: self.input,
            candidate,
            floor,
            cancel: cancel.clone(),
            revision: self.params.expected_revision,
            generation: self.generation,
            close_epoch: self.close_epoch,
        })
    }
}

fn remap(
    stored: &StoredDocument,
    fragment: &Fragment,
    mut floor: Ids,
    cancel: &Cancellation,
) -> Result<(Fragment, Ids), Error> {
    let mut scanned = 0_usize;
    for previous in stored
        .model()
        .pages()
        .iter()
        .flat_map(|page| page.fragments())
    {
        cancel.check()?;
        scanned = scanned
            .checked_add(1)
            .and_then(|n| n.checked_add(previous.atoms().len()))
            .and_then(|n| n.checked_add(previous.bonds().len()))
            .filter(|n| *n <= MAX_SCAN_ITEMS)
            .ok_or(Error::SmilesPasteLimit("document identity scan"))?;
        floor.fragment = floor.fragment.max(previous.id().get());
        for atom in previous.atoms() {
            cancel.check()?;
            floor.atom = floor.atom.max(atom.id.get());
        }
        for bond in previous.bonds() {
            cancel.check()?;
            floor.bond = floor.bond.max(bond.id.get());
        }
    }
    let mut ids = BTreeMap::new();
    let mut atoms = fragment.atoms().to_vec();
    for atom in &mut atoms {
        cancel.check()?;
        floor.atom = floor.atom.checked_add(1).ok_or(Error::IdentityExhausted)?;
        let new = AtomId::new(floor.atom).ok_or(Error::IdentityExhausted)?;
        ids.insert(atom.id, new);
        atom.id = new;
    }
    let mut bonds = fragment.bonds().to_vec();
    for bond in &mut bonds {
        cancel.check()?;
        floor.bond = floor.bond.checked_add(1).ok_or(Error::IdentityExhausted)?;
        bond.id = BondId::new(floor.bond).ok_or(Error::IdentityExhausted)?;
        bond.begin = *ids
            .get(&bond.begin)
            .ok_or(Error::InvalidSmilesPaste("missing layout endpoint"))?;
        bond.end = *ids
            .get(&bond.end)
            .ok_or(Error::InvalidSmilesPaste("missing layout endpoint"))?;
    }
    floor.fragment = floor
        .fragment
        .checked_add(1)
        .ok_or(Error::IdentityExhausted)?;
    let id = FragmentId::new(floor.fragment).ok_or(Error::IdentityExhausted)?;
    Ok((
        Fragment::new(id, atoms, bonds).map_err(bondline_model::Error::from)?,
        floor,
    ))
}
