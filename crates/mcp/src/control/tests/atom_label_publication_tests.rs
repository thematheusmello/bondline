//! Child of the existing control::tests module; reuses its real bounded writers.
use super::*;

#[tokio::test(flavor = "current_thread")]
async fn atom_label_presentation_wakes_only_after_reply_retirement_even_when_write_or_flush_fails()
-> Result<(), Box<dyn std::error::Error>> {
    for phase in ["gated", "write", "flush"] {
        let mut session = Engine::default();
        session.set_ui_file_host(true);
        session.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":72.0,
            "begin_y_pt":72.0,"end_x_pt":120.0,"end_y_pt":72.0}),
        )?;
        let before = serde_json::to_value(session.state())?;
        let owner = session.capture_stamp();
        let engine = Arc::new(Mutex::new(session));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let request = serde_json::to_vec(&json!({"command":"ui.atom_label","params":{
            "action":"open","expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":2}}))?;
        let (registered, publication) = executor
            .prepare_json(&request)
            .map_err(|failure| failure.info.message)?;
        let result = executor
            .complete(registered, std::future::pending())
            .await
            .map_err(|failure| failure.message)?;
        assert!(!result.changed);
        assert!(result.file_operation.is_none() && result.close_operation.is_none());
        assert!(result.ui_file_operation.is_none());
        let id = serde_json::to_value(&result)?["ui_atom_label_operation"]
            .as_u64()
            .ok_or("request ID")?;
        assert!(
            executor.has_active_work(),
            "reply publication still owns activity"
        );
        let notified = Arc::new(AtomicBool::new(false));
        let active_at_callback = Arc::new(AtomicBool::new(true));
        if phase == "gated" {
            let bytes = Arc::new(Mutex::new(Vec::new()));
            let output = Arc::clone(&bytes);
            let (flushing, flush_started) = oneshot::channel();
            let (release, released) = oneshot::channel();
            let flag = Arc::clone(&notified);
            let active = Arc::clone(&active_at_callback);
            let observed = executor.clone();
            let task = tokio::spawn(async move {
                let mut writer = FlushGate {
                    bytes: output,
                    flushing: Some(flushing),
                    release: released,
                };
                publish_result(&mut writer, Ok(result), publication, &move || {
                    active.store(observed.has_active_work(), Ordering::SeqCst);
                    flag.store(true, Ordering::SeqCst);
                })
                .await
            });
            timeout(Duration::from_secs(2), flush_started).await??;
            assert!(executor.has_active_work());
            assert!(!notified.load(Ordering::SeqCst));
            release.send(()).map_err(|_| "release flush")?;
            timeout(Duration::from_secs(2), task).await???;
            let reply: Value = serde_json::from_slice(&bytes.lock().map_err(|_| "reply bytes")?)?;
            assert_eq!(reply["result"]["ui_atom_label_operation"], id);
            assert_eq!(reply["result"]["changed"], false);
        } else {
            let failed = publish_result(
                &mut BrokenWriter {
                    fail_flush: phase == "flush",
                },
                Ok(result),
                publication,
                &|| {
                    active_at_callback.store(executor.has_active_work(), Ordering::SeqCst);
                    notified.store(true, Ordering::SeqCst);
                },
            )
            .await
            .err()
            .ok_or("broken response accepted")?;
            assert_eq!(failed.kind(), io::ErrorKind::BrokenPipe);
        }
        assert!(notified.load(Ordering::SeqCst));
        assert!(!active_at_callback.load(Ordering::SeqCst));
        assert!(!executor.has_active_work());
        let session = engine.lock().map_err(|_| "engine")?;
        assert!(session.capture_stamp_matches(&owner));
        let mut state = serde_json::to_value(session.state())?;
        assert_eq!(state["ui_atom_label"]["id"], id);
        assert_eq!(state["ui_atom_label"]["current"], true);
        state
            .as_object_mut()
            .ok_or("state")?
            .remove("ui_atom_label");
        assert_eq!(state, before);
    }
    Ok(())
}
