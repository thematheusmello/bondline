use super::{Command, Engine, NoParams, PageSetSize};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use std::sync::Arc;

const SOURCE: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");

#[test]
fn history_retains_opaque_source_and_undo_restores_exportability()
-> Result<(), Box<dyn std::error::Error>> {
    let stored = StoredDocument::from_cdxml(Arc::from(SOURCE), PageSize::A4)?;
    let mut engine = Engine::from_document(stored);
    let original = engine.stored_document().clone();
    let initial = engine.state();
    assert_eq!(
        initial.source.as_ref().ok_or("missing source")?.byte_length,
        SOURCE.len()
    );
    engine.execute(Command::PageSetSize(PageSetSize {
        page_id: 1,
        width_pt: 72.0,
        height_pt: 144.0,
    }))?;
    let edited = engine.stored_document().clone();
    assert!(engine.stored_document().export_cdxml().is_err());
    engine.execute(Command::Undo(NoParams {}))?;
    assert_eq!(engine.stored_document(), &original);
    assert_eq!(engine.stored_document().export_cdxml()?, SOURCE);
    assert!(!engine.commit(original.clone())?);
    assert_eq!(engine.state().redo_depth, 1);
    engine.execute(Command::Redo(NoParams {}))?;
    assert_eq!(engine.stored_document(), &edited);
    assert!(engine.stored_document().export_cdxml().is_err());
    assert!(!initial.source.as_ref().ok_or("missing source")?.modified);
    Ok(())
}

#[test]
fn same_projection_with_different_unknown_source_is_a_real_history_change()
-> Result<(), Box<dyn std::error::Error>> {
    let changed = std::str::from_utf8(SOURCE)?.replace(
        "keep-this-processing-instruction",
        "different-opaque-instruction",
    );
    let first = StoredDocument::from_cdxml(Arc::from(SOURCE), PageSize::A4)?;
    let second = StoredDocument::from_cdxml(Arc::from(changed.as_bytes()), PageSize::A4)?;
    assert_eq!(first.model(), second.model());
    let mut engine = Engine::from_document(first.clone());
    assert!(engine.commit(second.clone())?);
    assert_eq!(engine.state().revision, 1);
    engine.execute(Command::Undo(NoParams {}))?;
    assert_eq!(engine.stored_document(), &first);
    assert_eq!(engine.stored_document().export_cdxml()?, SOURCE);
    assert!(!engine.commit(first.clone())?);
    assert_eq!(engine.state().redo_depth, 1);
    engine.execute(Command::Redo(NoParams {}))?;
    assert_eq!(engine.stored_document(), &second);
    assert_eq!(engine.stored_document().export_cdxml()?, changed.as_bytes());
    Ok(())
}

#[test]
fn unsupported_capture_cannot_succeed_with_a_blank_chemical_document()
-> Result<(), Box<dyn std::error::Error>> {
    let stored = StoredDocument::from_cdxml(Arc::from(SOURCE), PageSize::A4)?;
    let mut native = StoredDocument::default();
    let page = native.model().pages().first().ok_or("missing page")?.id();
    native.model_mut().append_fragments(
        page,
        stored
            .model()
            .pages()
            .first()
            .ok_or("missing imported page")?
            .fragments()
            .iter()
            .map(|fragment| fragment.as_ref().clone())
            .collect(),
    )?;
    for document in [stored, native] {
        let mut engine = Engine::from_document(document);
        let before = engine.stored_document().clone();
        for backend in ["cpu", "gpu"] {
            let error = engine
                .dispatch(
                    "render.capture",
                    serde_json::json!({"page_id":1,"width_px":64,"height_px":64,"backend":backend}),
                )
                .err()
                .ok_or("blank capture succeeded")?;
            assert_eq!(error.info().code, "unsupported_content");
            assert_eq!(engine.stored_document(), &before);
            assert_eq!(engine.state().revision, 0);
        }
    }
    Ok(())
}

#[test]
fn exhausted_revision_rejects_a_source_change_without_losing_the_original()
-> Result<(), Box<dyn std::error::Error>> {
    let first = StoredDocument::from_cdxml(Arc::from(SOURCE), PageSize::A4)?;
    let changed = std::str::from_utf8(SOURCE)?.replace(
        "keep-this-processing-instruction",
        "different-opaque-instruction",
    );
    let second = StoredDocument::from_cdxml(Arc::from(changed.as_bytes()), PageSize::A4)?;
    let mut engine = Engine::from_document(first.clone());
    engine.revision = u64::MAX;
    assert!(matches!(
        engine.commit(second),
        Err(super::Error::RevisionExhausted)
    ));
    assert_eq!(engine.stored_document(), &first);
    assert_eq!(engine.stored_document().export_cdxml()?, SOURCE);
    assert_eq!(engine.state().undo_depth, 0);
    Ok(())
}
