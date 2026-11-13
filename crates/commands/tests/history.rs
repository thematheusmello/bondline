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
}
