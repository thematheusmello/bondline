//! Independent Rust-only selection gesture proof and nonauthoring preview.
use bondline_commands::{Cancellation, Engine, SelectionTranslate};
use bondline_model::PageId;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated directory")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn wire(engine: &Engine) -> TestResult<Value> {
    Ok(serde_json::to_value(engine.state())?)
}

fn select(engine: &mut Engine, ids: &[u32]) -> TestResult {
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":engine.state().revision,
        "page_id":1,"atom_ids":ids}),
    )?;
    Ok(())
}

fn named_pair(directory: &support::FileTestDirectory) -> TestResult<(Engine, PathBuf)> {
    let mut engine = Engine::default();
    for (a, b) in [(72, 120), (220, 268)] {
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":a,"begin_y_pt":72,
            "end_x_pt":b,"end_y_pt":72}),
        )?;
    }
    engine.dispatch(
        "atom.set_label",
        json!({"expected_revision":2,"page_id":1,
        "fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}),
    )?;
    let path = directory.path("gesture-source-原.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    engine.dispatch("document.open_cdxml", json!({"path":path}))?;
    engine.dispatch("ui.view", json!({"action":"actual"}))?;
    engine.dispatch(
        "ui.view",
        json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
    )?;
    select(&mut engine, &[1, 2])?;
    Ok((engine, path))
}

#[test]
fn gesture_context_is_weak_and_full_context_aba_never_restores_its_authority() -> TestResult {
    let mut empty = Engine::default();
    assert!(
        empty.selection_context().is_err(),
        "a bond-tool session has no selection gesture proof"
    );
    empty.dispatch(
        "ui.selection",
        json!({"action":"activate","expected_revision":0}),
    )?;
    let empty_context = empty.selection_context()?;
    assert!(
        empty_context.is_current(&empty),
        "active empty selection supports a marquee"
    );
    empty.set_view_canvas_size(848, 728)?;
    assert!(
        empty_context.is_current(&empty),
        "canvas geometry remains a separate UI qualifier"
    );
    let weak = Arc::downgrade(&empty.stored_document()?.model_snapshot());
    drop(empty);
    assert!(
        weak.upgrade().is_none(),
        "gesture context must not retain the complete model owner"
    );
    assert!(!empty_context.is_current(&Engine::default()));

    for change in [
        "source",
        "selection",
        "view",
        "pan",
        "tool",
        "close",
        "file-request",
        "association",
        "host",
    ] {
        let directory = support::FileTestDirectory::new()?;
        let (mut engine, path) = named_pair(&directory)?;
        engine.set_ui_file_host(true);
        let bytes = fs::read(&path)?;
        let model = serde_json::to_value(engine.document()?)?;
        let before = wire(&engine)?;
        let context = engine.selection_context()?;
        assert!(context.is_current(&engine));
        let revision = engine.state().revision;
        engine.dispatch(
            "ui.selection",
            json!({"action":"inspect","expected_revision":revision}),
        )?;
        select(&mut engine, &[2, 1])?;
        engine.dispatch("ui.view", json!({"action":"inspect"}))?;
        engine.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
        assert_eq!(wire(&engine)?, before);
        assert!(
            context.is_current(&engine),
            "no-op selection/inspection/same-association Save stays current"
        );
        match change {
            "source" => {
                engine.dispatch("document.open_cdxml", json!({"path":path}))?;
                select(&mut engine, &[1, 2])?;
            }
            "selection" => {
                select(&mut engine, &[3, 4])?;
                select(&mut engine, &[2, 1])?;
            }
            "view" => {
                engine.dispatch("ui.view", json!({"action":"set_zoom","percent":150}))?;
                engine.dispatch("ui.view", json!({"action":"actual"}))?;
                engine.dispatch(
                    "ui.view",
                    json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
                )?;
            }
            "pan" => {
                engine.dispatch(
                    "ui.view",
                    json!({"action":"pan","center_x_pt":200,"center_y_pt":200}),
                )?;
                engine.dispatch(
                    "ui.view",
                    json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
                )?;
            }
            "tool" => {
                engine.dispatch("ui.bond_tool", json!({"order":"double"}))?;
                engine.dispatch("ui.bond_tool", json!({"order":"single"}))?;
                select(&mut engine, &[1, 2])?;
            }
            "close" => {
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":revision,"action":"request"}),
                )?;
                assert!(
                    engine.selection_context().is_err(),
                    "pending/approved close prevents capture"
                );
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":revision,"action":"cancel"}),
                )?;
            }
            "file-request" => {
                let requested = engine.dispatch(
                    "ui.file",
                    json!({"expected_revision":revision,"action":"save"}),
                )?;
                let id = requested.ui_file_operation.ok_or("actual file request")?;
                assert!(
                    engine.selection_context().is_err(),
                    "file intent prevents gesture capture"
                );
                engine.dispatch(
                    "ui.file",
                    json!({"expected_revision":revision,"action":"cancel","request_id":id}),
                )?;
            }
            "association" => {
                let other = directory.path("association-B-λ.cdxml");
                engine.dispatch("document.save_cdxml", json!({"path":other}))?;
                engine.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
                assert_eq!(fs::read(other)?, bytes);
            }
            _ => {
                engine.set_ui_file_host(false);
                engine.set_ui_file_host(true);
            }
        }
        assert_eq!(serde_json::to_value(engine.document()?)?, model);
        assert_eq!(wire(&engine)?["ui_selection"]["active"], true);
        assert_eq!(wire(&engine)?["ui_selection"]["atom_ids"], json!([1, 2]));
        assert_eq!(engine.view().zoom_percent, Some(100.0));
        assert_eq!(
            engine.view().center,
            Some(bondline_model::Point2::new(100.0, 144.0)?)
        );
        assert_eq!(fs::read(&path)?, bytes);
        assert!(
            !context.is_current(&engine),
            "{change} A→B→A cannot revive the old gesture proof"
        );
        assert!(
            engine.selection_context()?.is_current(&engine),
            "a fresh explicit proof must work"
        );
    }
    Ok(())
}

fn no_file_change(path: &Path, original: &[u8]) -> TestResult {
    assert_eq!(fs::read(path)?, original);
    Ok(())
}

#[test]
fn literal_preview_uses_authoring_geometry_without_changing_engine_and_errors_are_atomic()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let (mut engine, path) = named_pair(&directory)?;
    let bytes = fs::read(&path)?;
    let before = wire(&engine)?;
    let content = engine.content().clone();
    let source = engine
        .stored_document()?
        .source_info()
        .ok_or("native source")?;
    let owner = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let other = Arc::clone(&engine.document()?.pages()[0].fragments()[1]);
    let tool = engine.bond_tool();
    let view_state = engine.view();
    let context = engine.selection_context()?;
    let revision = engine.state().revision;
    let epoch = before["ui_selection"]["epoch"]
        .as_u64()
        .ok_or("selection epoch")?;
    let params = SelectionTranslate {
        expected_revision: revision,
        expected_selection_epoch: epoch,
        delta_x_pt: 24.0,
        delta_y_pt: -12.0,
    };
    let preview = engine.preview_selection_translation(&params)?;
    let mut expected = before["document"].clone();
    expected["pages"][0]["fragments"][0]["atoms"][0]["position"] = json!({"x_pt":96.0,"y_pt":60.0});
    expected["pages"][0]["fragments"][0]["atoms"][1]["position"] =
        json!({"x_pt":144.0,"y_pt":60.0});
    assert_eq!(serde_json::to_value(preview.model())?, expected);
    assert_eq!(
        preview.source_info().ok_or("preview backing")?.format,
        source.format
    );
    assert_eq!(
        preview.source_info().ok_or("preview backing")?.byte_length,
        source.byte_length
    );
    assert!(preview.source_info().ok_or("preview backing")?.modified);
    assert!(Arc::ptr_eq(
        &other,
        &preview.model().pages()[0].fragments()[1]
    ));
    let page = PageId::new(1).ok_or("page")?;
    let view = engine.page_view(1024, 1024, page)?;
    let cancel = Cancellation::default();
    let original_graph = engine.drawing_scene_with_view(&view, page, &cancel)?;
    let preview_graph =
        bondline_io::display::drawing_page_scene_with_view(&view, &preview, page, &cancel)?;
    let original_frame = bondline_render::cpu_scene_with_cancel(original_graph.scene(), &cancel)?;
    let preview_frame = bondline_render::cpu_scene_with_cancel(preview_graph.scene(), &cancel)?;
    assert_ne!(
        original_frame.rgba, preview_frame.rgba,
        "literal moved chemistry must actually move in the shared renderer"
    );
    for (r, e, x, y, code) in [
        (revision - 1, epoch, 24.0, -12.0, "stale_revision"),
        (revision, epoch + 1, 24.0, -12.0, "stale_selection"),
        (revision, epoch, f64::NAN, 0.0, "invalid_params"),
        (revision, epoch, 0.0, f64::INFINITY, "invalid_params"),
        (revision, epoch, f64::MAX, 0.0, "invalid_params"),
    ] {
        let invalid = SelectionTranslate {
            expected_revision: r,
            expected_selection_epoch: e,
            delta_x_pt: x,
            delta_y_pt: y,
        };
        let error = engine
            .preview_selection_translation(&invalid)
            .err()
            .ok_or("invalid preview accepted")?;
        assert_eq!(error.info().code, code);
    }
    let zero = SelectionTranslate {
        delta_x_pt: 0.0,
        delta_y_pt: 0.0,
        ..params.clone()
    };
    let unchanged = engine.preview_selection_translation(&zero)?;
    assert!(Arc::ptr_eq(&model, &unchanged.model_snapshot()));
    assert_eq!(wire(&engine)?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), owner);
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.view(), view_state);
    assert!(context.is_current(&engine));
    no_file_change(&path, &bytes)?;
    // This is a candidate/commit consistency control, not an independent renderer oracle/golden.
    engine.dispatch("selection.translate", serde_json::to_value(&params)?)?;
    assert_eq!(serde_json::to_value(engine.document()?)?, expected);
    let committed_graph = engine.drawing_scene_with_view(&view, page, &cancel)?;
    assert_eq!(
        bondline_render::cpu_scene_with_cancel(committed_graph.scene(), &cancel)?,
        preview_frame
    );
    assert!(!context.is_current(&engine));
    assert_eq!(engine.state().undo_depth, 1);
    no_file_change(&path, &bytes)?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"discard"}),
    )?;
    assert!(engine.complete_close());
    let closed = wire(&engine)?;
    let obsolete = SelectionTranslate {
        expected_revision: 0,
        ..params
    };
    let error = engine
        .preview_selection_translation(&obsolete)
        .err()
        .ok_or("committed-close preview accepted")?;
    assert_eq!(
        error.info().code,
        "close_committed",
        "global guard precedes stale_revision"
    );
    assert_eq!(wire(&engine)?, closed);
    no_file_change(&path, &bytes)?;
    Ok(())
}

#[path = "selection_context/modal_regressions.rs"]
mod modal_regressions;

#[path = "selection_context/active_modal_supplement.rs"]
mod active_modal_supplement;
