use bondline_commands::{Command, Engine, NoParams, PageSetSize};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        rng_seed: proptest::test_runner::RngSeed::Fixed(0xb0_0d_11_0e),
        ..ProptestConfig::default()
    })]

    #[test]
    fn every_generated_edit_can_be_undone_and_redone_exactly(
        sizes in proptest::collection::vec((1u32..10_000, 1u32..10_000), 1..64)
    ) {
        let mut engine = Engine::default();
        let original = serde_json::to_vec(engine.document())?;
        let mut count = 0;
        for (width, height) in sizes {
            let before = serde_json::to_vec(engine.document())?;
            let result = engine.execute(Command::PageSetSize(PageSetSize {
                page_id: 1, width_pt: f64::from(width), height_pt: f64::from(height),
            }))?;
            let after = serde_json::to_vec(engine.document())?;
            let page = engine.document().pages().first().ok_or_else(|| TestCaseError::fail("missing page"))?;
            prop_assert_eq!(page.size().width_pt(), f64::from(width));
            prop_assert_eq!(page.size().height_pt(), f64::from(height));
            if result.changed {
                count += 1;
                engine.execute(Command::Undo(NoParams {}))?;
                prop_assert_eq!(serde_json::to_vec(engine.document())?, before);
                engine.execute(Command::Redo(NoParams {}))?;
                prop_assert_eq!(serde_json::to_vec(engine.document())?, after);
            }
        }
        let final_document = serde_json::to_vec(engine.document())?;
        for _ in 0..count { engine.execute(Command::Undo(NoParams {}))?; }
        prop_assert_eq!(serde_json::to_vec(engine.document())?, original);
        for _ in 0..count { engine.execute(Command::Redo(NoParams {}))?; }
        prop_assert_eq!(serde_json::to_vec(engine.document())?, final_document);
    }

    #[test]
    fn source_backed_generated_edits_restore_the_full_snapshot_and_original_bytes(
        sizes in proptest::collection::vec((1u32..10_000,1u32..10_000),1..64)
    ) {
        let source=include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");
        let stored=bondline_io::StoredDocument::from_cdxml(std::sync::Arc::from(source.as_slice()),bondline_model::PageSize::A4)?;
        let mut engine=Engine::from_document(stored);
        let original=engine.stored_document().clone();
        let mut count=0;
        for (width,height) in sizes {
            let before=engine.stored_document().clone();
            let result=engine.execute(Command::PageSetSize(PageSetSize {page_id:1,width_pt:f64::from(width),height_pt:f64::from(height)}))?;
            let after=engine.stored_document().clone();
            if result.changed {
                count+=1;
                engine.execute(Command::Undo(NoParams{}))?;
                prop_assert_eq!(engine.stored_document(),&before);
                engine.execute(Command::Redo(NoParams{}))?;
                prop_assert_eq!(engine.stored_document(),&after);
            }
        }
        let final_document=engine.stored_document().clone();
        for _ in 0..count {engine.execute(Command::Undo(NoParams{}))?;}
        prop_assert_eq!(engine.stored_document(),&original);
        prop_assert_eq!(engine.stored_document().export_cdxml()?,source.as_slice());
        for _ in 0..count {engine.execute(Command::Redo(NoParams{}))?;}
        prop_assert_eq!(engine.stored_document(),&final_document);
    }

    #[test]
    fn generated_bond_edits_restore_full_graphs_and_metadata(
        positions in proptest::collection::vec((10u32..500,10u32..500),1..40)
    ) {
        let mut engine=Engine::default();let original=engine.stored_document().clone();
        let mut snapshots=Vec::new();
        for (x,y) in positions {
            engine.dispatch("bond.create",serde_json::json!({"page_id":1,
                "begin_x_pt":x,"begin_y_pt":y,"end_x_pt":x+40,"end_y_pt":y+20}))?;
            snapshots.push(engine.stored_document().clone());
        }
        for expected in snapshots.iter().rev().skip(1) {
            engine.execute(Command::Undo(NoParams{}))?;
            prop_assert_eq!(engine.stored_document(),expected);
        }
        engine.execute(Command::Undo(NoParams{}))?;
        prop_assert_eq!(engine.stored_document(),&original);
        for expected in &snapshots {
            engine.execute(Command::Redo(NoParams{}))?;
            prop_assert_eq!(engine.stored_document(),expected);
        }
    }
}
