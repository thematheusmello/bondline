#![no_main]

use bondline_commands::{Engine, UiPreferencesRequest};
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

fn facts(request: Option<&UiPreferencesRequest>) -> Option<(u64, u64, bool, bool, bool)> {
    request.map(|r| {
        (
            r.id,
            r.input_revision,
            r.sprout_rings_when_clicking,
            r.current,
            r.was_cancelled(),
        )
    })
}

fuzz_target!(|params: &[u8]| {
    if params.len() > 16_384 {
        return;
    }
    let mut frame = br#"{"command":"ui.preferences","params":"#.to_vec();
    frame.extend_from_slice(params);
    frame.push(b'}');
    for preopened in [false, true] {
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        if preopened {
            let prepared = engine.prepare_json(br#"{"command":"ui.preferences","params":{"action":"open","expected_revision":0}}"#).unwrap();
            assert!(!prepared.is_deferred());
            drop(prepared);
            let request = engine.ui_preferences_request().unwrap();
            assert_eq!(
                (
                    request.id,
                    request.input_revision,
                    request.sprout_rings_when_clicking,
                    request.current
                ),
                (1, 0, false, true)
            );
        }
        let authored = engine.content().clone();
        let model = engine.stored_document().unwrap().model_snapshot();
        let stamp = engine.capture_stamp();
        let state = engine.state();
        let view = engine.view();
        let tool = engine.bond_tool();
        let mode = engine.effective_tool();
        let selection = engine.selection().clone();
        let generation = engine.session_generation();
        let file_context = engine.ui_file_context_stamp();
        let pref = engine.preferences();
        let request = engine.ui_preferences_request();
        let prior_request = facts(request.as_ref());
        assert_eq!((pref.epoch, pref.sprout_rings_when_clicking), (0, false));
        let accepted = match engine.prepare_json(&frame) {
            Err(_) => false,
            Ok(prepared) => {
                assert!(
                    !prepared.is_deferred(),
                    "a preferences request cannot prepare file/render work"
                );
                drop(prepared);
                true
            }
        };
        // Every invariant below precedes the separate deterministic identity probe.
        assert_eq!(engine.content(), &authored);
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document().unwrap().model_snapshot()
        ));
        assert!(engine.capture_stamp_matches(&stamp));
        assert_eq!(engine.session_generation(), generation);
        assert_eq!(engine.view(), view);
        assert_eq!(engine.effective_tool(), mode);
        assert_eq!(engine.bond_tool().order, tool.order);
        assert_eq!(engine.selection(), &selection);
        assert!(engine.ui_file_context_stamp_matches(&file_context));
        let after = engine.state();
        assert!(!after.changed);
        assert_eq!(
            (after.revision, after.undo_depth, after.redo_depth),
            (state.revision, state.undo_depth, state.redo_depth)
        );
        assert_eq!(
            (after.revision, after.undo_depth, after.redo_depth),
            (0, 0, 0)
        );
        assert!(after.source.is_none() && state.source.is_none());
        assert_eq!(after.file.path, state.file.path);
        assert_eq!(after.file.format, state.file.format);
        assert_eq!(after.file.dirty, state.file.dirty);
        assert_eq!(
            (after.close, after.close_epoch),
            (state.close, state.close_epoch)
        );
        assert_eq!(
            (after.ui_file.available, after.ui_file.epoch),
            (state.ui_file.available, state.ui_file.epoch)
        );
        assert!(after.ui_file.request.is_none());
        assert!(after.file_operation.is_none() && after.ui_file_operation.is_none());
        assert!(
            after.ui_atom_label.is_none()
                && after.ui_drawing_settings.is_none()
                && after.ui_add_chain.is_none()
        );
        assert!(
            after.preferences.is_none() && after.ui_preferences_operation.is_none(),
            "normal state exposes no explicit reply operation"
        );
        let next = engine.preferences();
        let active = engine.ui_preferences_request();
        assert_eq!(facts(after.ui_preferences.as_ref()), facts(active.as_ref()));
        if !accepted {
            assert_eq!(
                (next.epoch, next.sprout_rings_when_clicking),
                (pref.epoch, pref.sprout_rings_when_clicking)
            );
            assert_eq!(engine.bond_tool(), tool);
            assert_eq!(facts(active.as_ref()), prior_request);
            if let Some(old) = &request {
                assert!(
                    !old.was_cancelled(),
                    "error marked a prior exact-Cancel token"
                );
            }
        } else {
            let delta =
                u64::from(next.sprout_rings_when_clicking != pref.sprout_rings_when_clicking);
            assert_eq!(next.epoch, pref.epoch.checked_add(delta).unwrap());
            assert_eq!(
                engine.bond_tool().epoch,
                tool.epoch.checked_add(delta).unwrap()
            );
            if let Some(current) = active.as_ref() {
                assert_eq!(
                    current.id, 1,
                    "one input can only create the first request or retain it"
                );
                assert_eq!(current.input_revision, 0);
                assert_eq!(
                    current.sprout_rings_when_clicking, false,
                    "the draft keeps its captured value"
                );
                assert!(!current.was_cancelled());
                assert_eq!(
                    current.current,
                    delta == 0,
                    "a real preference change stales the captured draft"
                );
                assert_eq!(
                    engine.ui_preferences_request_current(current.id),
                    current.current
                );
                if let Some(old) = &request {
                    assert!(!old.was_cancelled());
                }
            } else if let Some(old) = &request {
                // Apply retires without cancellation; Cancel retires its exact token and changes no preference.
                if old.was_cancelled() {
                    assert_eq!(delta, 0);
                }
                assert!(!engine.ui_preferences_request_current(old.id));
            }
        }
        // Public floors are observed only AFTER the above immutable-state checks.
        // This fixed immediate authoring probe is not fuzz-selected and never completes work.
        let probe = engine.prepare_json(br#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}}"#).unwrap();
        assert!(!probe.is_deferred());
        drop(probe);
        let document = engine.document().unwrap();
        let page = document.pages().first().unwrap();
        assert_eq!(page.fragments().len(), 1);
        let fragment = page.fragments().first().unwrap();
        assert_eq!(fragment.id().get(), 1);
        assert_eq!(
            fragment
                .atoms()
                .iter()
                .map(|a| a.id.get())
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            fragment
                .bonds()
                .iter()
                .map(|b| b.id.get())
                .collect::<Vec<_>>(),
            [1]
        );
        // Discard this Engine. The probe may stale a preopened draft; no policy assertions follow it.
    }
});
