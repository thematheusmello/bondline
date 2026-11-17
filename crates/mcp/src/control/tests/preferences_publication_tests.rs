//! Session-only changes still require publication-qualified native repaint.
use super::*;

#[tokio::test(flavor = "current_thread")]
async fn preferences_wake_exactly_for_operations_after_reply_and_activity_retirement()
-> Result<(), Box<dyn std::error::Error>> {
    for (label, params, preopen, expected) in [
        ("inspect", json!({"action":"inspect"}), false, None),
        (
            "noop",
            json!({"action":"set","expected_epoch":0,"sprout_rings_when_clicking":false}),
            false,
            None,
        ),
        (
            "set",
            json!({"action":"set","expected_epoch":0,"sprout_rings_when_clicking":true}),
            false,
            Some(json!({"action":"set","changed":true})),
        ),
        (
            "open",
            json!({"action":"open","expected_revision":0}),
            false,
            Some(json!({"action":"open","changed":false,"request_id":1})),
        ),
        (
            "cancel",
            json!({"action":"cancel","expected_revision":0,"request_id":1}),
            true,
            Some(json!({"action":"cancel","changed":false,"request_id":1})),
        ),
        (
            "apply noop",
            json!({"action":"apply","expected_revision":0,"request_id":1,"sprout_rings_when_clicking":false}),
            true,
            Some(json!({"action":"apply","changed":false,"request_id":1})),
        ),
        (
            "apply",
            json!({"action":"apply","expected_revision":0,"request_id":1,"sprout_rings_when_clicking":true}),
            true,
            Some(json!({"action":"apply","changed":true,"request_id":1})),
        ),
    ] {
        for phase in ["gated", "write", "flush"] {
            let mut session = Engine::default();
            session.set_ui_file_host(true);
            if preopen {
                session.dispatch(
                    "ui.preferences",
                    json!({"action":"open","expected_revision":0}),
                )?;
            }
            let owner = session.capture_stamp();
            let content = session.content().clone();
            let engine = Arc::new(Mutex::new(session));
            let executor = CommandExecutor::new(Arc::clone(&engine));
            let request = serde_json::to_vec(&json!({"command":"ui.preferences","params":params}))?;
            let (registered, publication) = executor
                .prepare_json(&request)
                .map_err(|failure| failure.info.message)?;
            let result = executor
                .complete(registered, std::future::pending())
                .await
                .map_err(|failure| failure.message)?;
            assert!(!result.changed);
            assert!(
                result.file_operation.is_none()
                    && result.ui_file_operation.is_none()
                    && result.close_operation.is_none()
            );
            let value = serde_json::to_value(&result)?;
            assert_eq!(value.get("ui_preferences_operation"), expected.as_ref());
            assert!(value.get("preferences").is_some());
            assert!(executor.has_active_work());
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
                let reply: Value =
                    serde_json::from_slice(&bytes.lock().map_err(|_| "reply bytes")?)?;
                assert_eq!(reply["result"], value);
            } else {
                let failure = publish_result(
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
                assert_eq!(failure.kind(), io::ErrorKind::BrokenPipe);
            }
            assert_eq!(
                notified.load(Ordering::SeqCst),
                expected.is_some(),
                "{label}/{phase}"
            );
            assert!(!executor.has_active_work());
            if expected.is_some() {
                assert!(!active_at_callback.load(Ordering::SeqCst));
            }
            let session = engine.lock().map_err(|_| "engine")?;
            assert!(session.capture_stamp_matches(&owner));
            assert_eq!(session.content(), &content);
            assert_eq!(
                (
                    session.state().revision,
                    session.state().undo_depth,
                    session.state().redo_depth
                ),
                (0, 0, 0)
            );
        }
    }
    Ok(())
}
