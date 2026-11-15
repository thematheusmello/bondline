use bondline_commands::{Command, Engine, UiFilePlan};
use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn request(engine: &mut Engine) -> TestResult<u64> {
    let revision = engine.state().revision;
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":revision,"action":"new"}),
    )?;
    Ok(engine.state().ui_file.request.ok_or("New request")?.id)
}

fn snapshot(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.state())?)
}

#[test]
fn new_presentation_is_a_qualified_immediate_session_change_without_file_or_close_ack() -> TestResult
{
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let stamp = engine.capture_stamp();
    let id = request(&mut engine)?;
    assert_eq!(engine.state().revision, 0);
    let before_claim = snapshot(&engine)?;
    assert_eq!(before_claim["ui_file"]["request"]["action"], "new");
    assert_eq!(before_claim["ui_file"]["request"]["phase"], "requested");
    assert_eq!(
        engine
            .create_ui_document(id, false)
            .err()
            .ok_or("unclaimed New accepted")?
            .info()
            .code,
        "ui_busy"
    );
    assert_eq!(snapshot(&engine)?, before_claim);
    assert!(matches!(engine.claim_ui_file(id)?, UiFilePlan::NewDocument));
    let before = snapshot(&engine)?;
    let forged_file = engine.prepare_ui_file(
        id,
        Command::parse("document.new", json!({"expected_revision":0}))?,
    );
    assert_eq!(
        forged_file
            .err()
            .ok_or("New bound to file work")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(snapshot(&engine)?, before);
    let created = engine.create_ui_document(id, false)?;
    assert!(created.changed && created.document.is_some());
    assert_eq!(created.revision, 1);
    assert!(
        created.source.is_none() && created.collection.is_none() && created.smiles_file.is_none()
    );
    assert!(created.file_operation.is_none() && created.ui_file_operation.is_none());
    assert!(created.ui_file.available && created.ui_file.request.is_none());
    assert!(!created.file.dirty && created.file.path.is_none());
    assert!(!engine.capture_stamp_matches(&stamp));
    assert!(
        !engine.finish_ui_file(id, false),
        "New is not a trusted published Save"
    );
    let current = snapshot(&engine)?;
    assert_eq!(
        engine
            .create_ui_document(id, true)
            .err()
            .ok_or("New replayed")?
            .info()
            .code,
        "ui_stale_request"
    );
    assert_eq!(snapshot(&engine)?, current);
    assert!(!engine.complete_close());
    Ok(())
}

#[test]
fn dirty_new_requires_explicit_discard_and_cancelled_or_wrong_intents_cannot_replace_content()
-> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let id = request(&mut engine)?;
    engine.claim_ui_file(id)?;
    let dirty = snapshot(&engine)?;
    let stamp = engine.capture_stamp();
    let content = engine.content().clone();
    assert_eq!(
        engine
            .create_ui_document(id, false)
            .err()
            .ok_or("dirty New discarded")?
            .info()
            .code,
        "unsaved_changes"
    );
    assert_eq!(snapshot(&engine)?, dirty);
    assert_eq!(engine.content(), &content);
    assert!(engine.capture_stamp_matches(&stamp));
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":1,"action":"cancel","request_id":id}),
    )?;
    let cancelled = snapshot(&engine)?;
    assert_eq!(
        engine
            .create_ui_document(id, true)
            .err()
            .ok_or("cancelled New accepted")?
            .info()
            .code,
        "ui_stale_request"
    );
    assert_eq!(snapshot(&engine)?, cancelled);
    engine.dispatch("ui.file", json!({"expected_revision":1,"action":"open"}))?;
    let open = engine.state().ui_file.request.ok_or("Open")?.id;
    engine.claim_ui_file(open)?;
    let before = snapshot(&engine)?;
    assert_eq!(
        engine
            .create_ui_document(open, true)
            .err()
            .ok_or("Open became New")?
            .info()
            .code,
        "invalid_params"
    );
    assert_eq!(snapshot(&engine)?, before);
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":1,"action":"cancel","request_id":open}),
    )?;
    let new = request(&mut engine)?;
    engine.claim_ui_file(new)?;
    let created = engine.create_ui_document(new, true)?;
    assert_eq!(
        (created.revision, created.undo_depth, created.redo_depth),
        (2, 0, 0)
    );
    assert!(!created.file.dirty);
    assert!(!engine.capture_stamp_matches(&stamp));
    Ok(())
}

#[test]
fn old_new_presentation_cannot_survive_source_or_close_context_aba() -> TestResult {
    for close_aba in [false, true] {
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        let id = request(&mut engine)?;
        engine.claim_ui_file(id)?;
        if close_aba {
            engine.dispatch(
                "document.close",
                json!({"expected_revision":0,"action":"request"}),
            )?;
            engine.dispatch(
                "document.close",
                json!({"expected_revision":0,"action":"cancel"}),
            )?;
        } else {
            engine.dispatch("document.new", json!({"expected_revision":0}))?;
        }
        let before = snapshot(&engine)?;
        let stamp = engine.capture_stamp();
        assert_eq!(
            engine
                .create_ui_document(id, true)
                .err()
                .ok_or("obsolete New accepted")?
                .info()
                .code,
            "ui_stale_request"
        );
        assert_eq!(snapshot(&engine)?, before);
        assert!(engine.capture_stamp_matches(&stamp));
    }
    Ok(())
}
