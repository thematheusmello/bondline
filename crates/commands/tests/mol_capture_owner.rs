use bondline_commands::{Cancellation, Command, Engine, Prepared};
use bondline_io::{StoredDocument, mol::MolTransform};
use bondline_model::{PageSize, Point2};
use serde_json::json;
use std::sync::{Arc, Weak};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn prepared_capture() -> TestResult<(Prepared, Weak<[u8]>)> {
    let bytes = include_bytes!("../../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");
    let source: Arc<[u8]> = Arc::from(bytes.get(..371).ok_or("original ethanol MOL span")?);
    let weak = Arc::downgrade(&source);
    let stored = StoredDocument::from_mol(
        source,
        PageSize::A4,
        MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?,
    )?;
    assert!(stored.is_fully_mapped());
    let mut engine = Engine::from_document(stored);
    let prepared = engine.prepare(Command::parse(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
    )?)?;
    assert!(prepared.is_deferred());
    drop(engine);
    Ok((prepared, weak))
}

#[test]
fn deferred_drawing_capture_keeps_complete_source_until_completion_then_releases_it() -> TestResult
{
    let (prepared, source) = prepared_capture()?;
    assert!(
        source.upgrade().is_some(),
        "deferred display classification needs the complete immutable source owner"
    );
    let completed = prepared.complete()?;
    assert!(
        source.upgrade().is_none(),
        "the reply must not lease source"
    );
    let mut receiver = Engine::default();
    let before = serde_json::to_value(receiver.state())?;
    let result = receiver.finish(completed)?;
    assert!(result.capture.is_some());
    assert_eq!(serde_json::to_value(receiver.state())?, before);
    Ok(())
}

#[test]
fn cancelled_or_dropped_drawing_work_releases_the_complete_source_owner() -> TestResult {
    let (prepared, source) = prepared_capture()?;
    assert!(source.upgrade().is_some());
    let cancel = Cancellation::default();
    cancel.cancel();
    let error = prepared
        .complete_with_cancel(&cancel)
        .err()
        .ok_or("capture ignored cancellation")?;
    assert_eq!(error.info().code, "cancelled");
    assert!(source.upgrade().is_none());

    let (prepared, source) = prepared_capture()?;
    assert!(source.upgrade().is_some());
    drop(prepared);
    assert!(source.upgrade().is_none());
    Ok(())
}
