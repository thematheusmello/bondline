use crate::{ConversionOutput, Engine, Error, StoredContent};
use bondline_model::{Fragment, FragmentId, PageId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportSmilesFragment {
    pub expected_revision: u64,
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(range(min = 1))]
    pub fragment_id: u32,
    pub output_path: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SmilesExportReceipt {
    pub profile: &'static str,
    pub scope: &'static str,
    pub not_encoded: [&'static str; 2],
    pub input_revision: u64,
    pub session_generation: u64,
    pub page_id: u32,
    pub fragment_id: u32,
    pub atom_count: usize,
    pub bond_count: usize,
    pub output: ConversionOutput,
}

pub(super) fn selected_fragment(
    content: &StoredContent,
    page: u32,
    fragment: u32,
) -> Result<&Fragment, Error> {
    let page = PageId::new(page).ok_or(Error::InvalidPageId)?;
    let fragment = FragmentId::new(fragment).ok_or(Error::InvalidFragmentId)?;
    let document = content.drawing()?;
    if !document.is_fully_mapped() {
        return Err(Error::IncompleteSmilesExportSource);
    }
    let page = document
        .model()
        .pages()
        .iter()
        .find(|p| p.id() == page)
        .ok_or(bondline_model::Error::MissingPage(page.get()))?;
    Ok(page
        .fragments()
        .iter()
        .find(|f| f.id() == fragment)
        .ok_or(bondline_model::Error::MissingFragment(fragment.get()))?
        .as_ref())
}

impl Engine {
    pub(super) fn smiles_export_plan(
        &self,
        params: ExportSmilesFragment,
    ) -> Result<crate::files::FilePlan, Error> {
        if params.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        selected_fragment(&self.document, params.page_id, params.fragment_id)?;
        Ok(crate::files::FilePlan::ExportSmiles {
            params,
            document: Arc::clone(&self.document),
            save_slot: self.reserve_save()?,
            generation: self.generation,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SmilesExportReceipt {
    pub(super) fn from_file(
        receipt: bondline_io::files::SaveReceipt,
        params: &ExportSmilesFragment,
        generation: u64,
        atom_count: usize,
        bond_count: usize,
    ) -> Result<Self, Error> {
        Ok(Self {
            profile: "neutral_explicit_h_fragment_v1",
            scope: "selected_fragment_chemical_graph",
            not_encoded: ["geometry", "document_context"],
            input_revision: params.expected_revision,
            session_generation: generation,
            page_id: params.page_id,
            fragment_id: params.fragment_id,
            atom_count,
            bond_count,
            output: receipt.try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use serde_json::json;

    #[test]
    fn deferred_export_retains_the_whole_source_owner_through_publication_and_refuses_binding()
    -> Result<(), Box<dyn std::error::Error>> {
        let path = std::env::temp_dir().join(format!(
            "bondline-export-owner-{}-{}.smi",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let mut engine = Engine::default();
        engine.dispatch("bond.create",json!({"page_id":1,"begin_x_pt":100.0,"begin_y_pt":200.0,"end_x_pt":140.0,"end_y_pt":200.0}))?;
        let weak = Arc::downgrade(&engine.document);
        let mut prepared = engine.prepare(Command::parse(
            "file.export_smiles_fragment",
            json!({"expected_revision":1,"page_id":1,"fragment_id":1,"output_path":path}),
        )?)?;
        assert_eq!(
            prepared
                .bind_ui_request(1)
                .err()
                .ok_or("export bound native request")?
                .info()
                .code,
            "invalid_params"
        );
        engine.dispatch("document.open_cdxml",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/bondline-v0.cdxml"),"discard_unsaved":true}))?;
        assert_eq!(
            weak.strong_count(),
            1,
            "only the complete captured owner should remain"
        );
        let completed = prepared.complete()?;
        assert_eq!(
            weak.strong_count(),
            1,
            "published Completion must retain the whole owner"
        );
        assert_eq!(std::fs::read(&path)?, b"[CH3][CH3]\n");
        let before = serde_json::to_value(engine.state())?;
        let result = engine.finish(completed)?;
        assert_eq!(result.smiles_export.ok_or("receipt")?.input_revision, 1);
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(
            weak.strong_count(),
            0,
            "owner may release only after finish/drop"
        );
        std::fs::remove_file(path)?;
        Ok(())
    }
}
