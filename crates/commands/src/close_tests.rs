use super::{CloseAction, CloseDocument, CloseState, Command, Engine};
use std::sync::Arc;

#[test]
fn exhausted_close_epoch_is_atomic_and_does_not_prevent_trusted_final_acknowledgement()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::default();
    engine.close_policy.epoch = u64::MAX - 1;
    let source = Arc::clone(&engine.document);
    let saved = Arc::clone(&engine.saved);
    let approved = engine.execute(Command::Close(CloseDocument {
        expected_revision: 0,
        action: CloseAction::Request,
    }))?;
    assert_eq!(approved.close, CloseState::Approved);
    assert_eq!(approved.close_epoch, u64::MAX);
    let before = serde_json::to_value(engine.state())?;
    for action in [
        CloseAction::Request,
        CloseAction::Discard,
        CloseAction::Cancel,
    ] {
        let error = engine
            .execute(Command::Close(CloseDocument {
                expected_revision: 0,
                action,
            }))
            .err()
            .ok_or("close epoch wrapped")?;
        assert_eq!(error.info().code, "close_epoch_exhausted");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(Arc::ptr_eq(&engine.document, &source));
        assert!(Arc::ptr_eq(&engine.saved, &saved));
    }
    assert!(engine.complete_close());
    assert_eq!(engine.state().close_epoch, u64::MAX);
    assert_eq!(engine.state().close, CloseState::Committed);
    Ok(())
}
