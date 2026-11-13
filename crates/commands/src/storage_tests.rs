use super::{Command, Engine, NoParams, PageSetSize};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use std::sync::{Arc, Weak};

type SourceFixture = (StoredDocument, Weak<[u8]>);

fn padded_source(tag: u8, length: usize) -> Result<SourceFixture, Box<dyn std::error::Error>> {
    let mut bytes = Vec::with_capacity(length + 32);
    bytes.extend_from_slice(b"<CDXML><!--");
    bytes.push(tag);
    bytes.resize(length, b'x');
    bytes.extend_from_slice(b"--><page id=\"1\"/></CDXML>");
    let bytes: Arc<[u8]> = bytes.into();
    let weak = Arc::downgrade(&bytes);
    Ok((StoredDocument::from_cdxml(bytes, PageSize::A4)?, weak))
}

#[test]
fn oldest_distinct_sources_are_evicted_before_128_snapshots()
-> Result<(), Box<dyn std::error::Error>> {
    // Three independent 45 MiB source payloads exceed the declared 128 MiB threshold.
    let length = 45 * 1024 * 1024;
    let mut engine = Engine::default();
    let (first, released) = padded_source(b'A', length)?;
    engine.commit(first)?;
    assert!(released.upgrade().is_some());
    let (second, _) = padded_source(b'B', length)?;
    engine.commit(second)?;
    assert_eq!(engine.state().undo_depth, 2);
    let (third, released_redo) = padded_source(b'C', length)?;
    engine.commit(third)?;
    assert_eq!(
        engine.state().undo_depth,
        1,
        "oldest snapshots retained too much source data"
    );
    assert!(
        released.upgrade().is_none(),
        "evicted source is still retained"
    );
    let current = engine.stored_document().clone();
    engine.execute(Command::Undo(NoParams {}))?;
    assert!(
        engine
            .stored_document()
            .export_cdxml()?
            .starts_with(b"<CDXML><!--B")
    );
    assert!(matches!(
        engine.execute(Command::Undo(NoParams {})),
        Err(super::Error::NothingToUndo)
    ));
    engine.execute(Command::Redo(NoParams {}))?;
    assert_eq!(engine.stored_document(), &current);
    drop(current);
    engine.execute(Command::Undo(NoParams {}))?;
    let (branch, _) = padded_source(b'D', length)?;
    engine.commit(branch)?;
    assert_eq!(
        engine.state().undo_depth,
        1,
        "discarded redo data consumed the new branch's budget"
    );
    assert_eq!(engine.state().redo_depth, 0);
    assert!(released_redo.upgrade().is_none());
    Ok(())
}

#[test]
fn shared_source_keeps_128_page_edits_without_counting_the_xml_129_times()
-> Result<(), Box<dyn std::error::Error>> {
    let (source, _) = padded_source(b'S', 2 * 1024 * 1024)?;
    let mut engine = Engine::from_document(source);
    for width in 1000..1128 {
        engine.execute(Command::PageSetSize(PageSetSize {
            page_id: 1,
            width_pt: f64::from(width),
            height_pt: 792.0,
        }))?;
    }
    assert_eq!(engine.state().undo_depth, 128);
    for _ in 0..128 {
        engine.execute(Command::Undo(NoParams {}))?;
    }
    assert!(
        engine
            .stored_document()
            .export_cdxml()?
            .starts_with(b"<CDXML><!--S")
    );
    assert!(!engine.state().source.ok_or("source")?.modified);
    for _ in 0..128 {
        engine.execute(Command::Redo(NoParams {}))?;
    }
    assert_eq!(engine.state().undo_depth, 128);
    Ok(())
}

#[test]
fn trimming_keeps_a_recent_suffix_and_always_preserves_the_current_document()
-> Result<(), Box<dyn std::error::Error>> {
    for (budget, expected_undo) in [(0, 0), (32 * 1024, 1), (128 * 1024, 4)] {
        let (first, _) = padded_source(b'A', 256)?;
        let mut engine = Engine::from_document(first);
        for (tag, length) in [(b'B', 4096), (b'C', 65536), (b'D', 128), (b'E', 2048)] {
            engine.commit(padded_source(tag, length)?.0)?;
        }
        let current = engine.stored_document().clone();
        let revision = engine.state().revision;
        engine.trim_history(budget);
        assert_eq!(engine.state().undo_depth, expected_undo, "budget {budget}");
        assert_eq!(engine.state().revision, revision);
        assert_eq!(engine.stored_document(), &current);
        if expected_undo == 1 {
            engine.execute(Command::Undo(NoParams {}))?;
            assert!(
                engine
                    .stored_document()
                    .export_cdxml()?
                    .starts_with(b"<CDXML><!--D")
            );
        }
    }
    Ok(())
}
