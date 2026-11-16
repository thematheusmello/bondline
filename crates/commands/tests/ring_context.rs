//! Separate missing-API stage; the existing-JSON runtime RED is a different file.
use bondline_commands::{
    Command, EffectiveTool, Engine, OrderHydrogenPolicy, RingCreate, RingPlacement, RingSide,
    SelectRingTool,
};
use serde_json::json;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn ring_native_context_and_preview_are_read_only_source_qualified_interfaces() -> TestResult {
    let mut engine = Engine::default();
    assert_eq!(engine.effective_tool(), EffectiveTool::Bond);
    engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
    assert_eq!(engine.effective_tool(), EffectiveTool::Ring { size: 6 });
    let context = engine.ring_context(1)?;
    assert!(context.is_current(&engine));
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let params = RingCreate {
        expected_revision: 0,
        page_id: 1,
        size: 6,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement: RingPlacement::Free {
            begin_x_pt: 120.0,
            begin_y_pt: 168.0,
            end_x_pt: 120.0,
            end_y_pt: 120.0,
            side: RingSide::Positive,
        },
    };
    let preview = engine.preview_ring_create(&params)?;
    let fragment = &preview.model().pages()[0].fragments()[0];
    assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (6, 6));
    assert_eq!(
        serde_json::to_value(fragment.atoms()[0].position)?,
        json!({"x_pt":120.0,"y_pt":168.0})
    );
    assert_eq!(
        serde_json::to_value(fragment.atoms()[1].position)?,
        json!({"x_pt":120.0,"y_pt":120.0})
    );
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(engine.capture_stamp_matches(&owner));
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert!(context.is_current(&engine));
    engine.execute(Command::RingCreate(params))?;
    assert_eq!(engine.document()?, preview.model());
    assert!(!context.is_current(&engine));
    Ok(())
}

// Context/preview semantics only; sealed ring geometry/precision/render oracles stay separate.
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use std::{fs, path::PathBuf};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated test directory")]
mod support;
const NATIVE: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");

fn explicit_ring(engine: &Engine) -> RingCreate {
    RingCreate {
        expected_revision: engine.state().revision,
        page_id: 1,
        size: 6,
        hydrogen_policy: OrderHydrogenPolicy::RectifyNeutralCarbon,
        placement: RingPlacement::Free {
            begin_x_pt: 300.0,
            begin_y_pt: 168.0,
            end_x_pt: 300.0,
            end_y_pt: 120.0,
            side: RingSide::Positive,
        },
    }
}
fn named_source(directory: &support::FileTestDirectory) -> TestResult<(Engine, PathBuf)> {
    let mut engine =
        Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    let path = directory.path("ring-context-A-原.cdxml");
    engine.dispatch("document.save_cdxml", json!({"path":path}))?;
    engine.set_ui_file_host(true);
    engine.dispatch("ui.view", json!({"action":"actual"}))?;
    engine.dispatch(
        "ui.view",
        json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
    )?;
    engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
    Ok((engine, path))
}

#[test]
fn ring_context_is_weak_and_each_real_context_aba_permanently_invalidates_it() -> TestResult {
    let mut inactive = Engine::default();
    assert_eq!(inactive.effective_tool(), EffectiveTool::Bond);
    assert!(inactive.ring_context(1).is_err());
    inactive.dispatch(
        "ui.selection",
        json!({"action":"activate","expected_revision":0}),
    )?;
    assert_eq!(inactive.effective_tool(), EffectiveTool::Selection);
    assert!(inactive.ring_context(1).is_err());
    let (old_context, weak_model) = {
        let mut engine =
            Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
        engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
        let context = engine.ring_context(1)?;
        let weak = Arc::downgrade(&engine.stored_document()?.model_snapshot());
        (context, weak)
    };
    assert!(
        weak_model.upgrade().is_none(),
        "the gesture proof must not retain the model/source owner"
    );
    let mut other = Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    other.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
    assert!(
        !old_context.is_current(&other),
        "equal values in another Engine do not confer identity"
    );
    for change in [
        "source",
        "namespace",
        "history",
        "size",
        "bond-mode",
        "selection-mode",
        "zoom",
        "pan",
        "close",
        "file-request",
        "association",
        "host",
    ] {
        let directory = support::FileTestDirectory::new()?;
        let (mut engine, path) = named_source(&directory)?;
        let model = serde_json::to_value(engine.document()?)?;
        let bytes = fs::read(&path)?;
        let before = serde_json::to_value(engine.state())?;
        let context = engine.ring_context(1)?;
        assert!(context.is_current(&engine));
        for id in [0, 9] {
            let error = engine
                .ring_context(id)
                .err()
                .ok_or("invalid context page accepted")?;
            assert_eq!(
                error.info().code,
                if id == 0 {
                    "invalid_params"
                } else {
                    "not_found"
                }
            );
        }
        engine.dispatch("document.inspect", json!({}))?;
        engine.dispatch("ui.controls", json!({}))?;
        engine.dispatch("ui.view", json!({"action":"inspect"}))?;
        engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
        engine.dispatch(
            "ui.selection",
            json!({"action":"clear","expected_revision":engine.state().revision}),
        )?;
        let preview = engine.preview_ring_create(&explicit_ring(&engine))?;
        assert_eq!(preview.model().pages()[0].fragments().len(), 2);
        drop(preview);
        engine.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(
            context.is_current(&engine),
            "no-op/inspection/preview/same-association Save stays current"
        );
        // Canvas size is a UI HoverContext qualifier, not a hidden Engine gesture epoch.
        engine.set_view_canvas_size(848, 728)?;
        assert!(context.is_current(&engine));
        match change {
            "source" => {
                engine.dispatch("document.open_cdxml", json!({"path":path}))?;
            }
            "namespace" => {
                engine.dispatch(
                    "document.new",
                    json!({"expected_revision":engine.state().revision}),
                )?;
                engine.dispatch("document.open_cdxml", json!({"path":path}))?;
                engine.dispatch("ui.view", json!({"action":"actual"}))?;
                engine.dispatch(
                    "ui.view",
                    json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
                )?;
            }
            "history" => {
                let p = explicit_ring(&engine);
                engine.execute(Command::RingCreate(p))?;
                engine.dispatch("history.undo", json!({}))?;
            }
            "size" => {
                engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 7 }))?;
                engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
            }
            "bond-mode" => {
                engine.dispatch("ui.bond_tool", json!({"order":"single"}))?;
                engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
            }
            "selection-mode" => {
                engine.dispatch("ui.selection",json!({"action":"set","expected_revision":engine.state().revision,"page_id":1,"atom_ids":[1,2]}))?;
                engine.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
            }
            "zoom" => {
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
            "close" => {
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":engine.state().revision,"action":"request"}),
                )?;
                assert!(engine.ring_context(1).is_err());
                engine.dispatch(
                    "document.close",
                    json!({"expected_revision":engine.state().revision,"action":"cancel"}),
                )?;
            }
            "file-request" => {
                let request = engine.dispatch(
                    "ui.file",
                    json!({"expected_revision":engine.state().revision,"action":"save"}),
                )?;
                let id = request.ui_file_operation.ok_or("file request")?;
                assert!(engine.ring_context(1).is_err());
                engine.dispatch("ui.file",json!({"expected_revision":engine.state().revision,"action":"cancel","request_id":id}))?;
            }
            "association" => {
                let b = directory.path("ring-context-B-λ.cdxml");
                engine.dispatch("document.save_cdxml", json!({"path":b}))?;
                engine.dispatch("document.save_cdxml", json!({"path":path,"overwrite":true}))?;
                assert_eq!(fs::read(b)?, bytes);
            }
            _ => {
                engine.set_ui_file_host(false);
                engine.set_ui_file_host(true);
            }
        }
        assert_eq!(serde_json::to_value(engine.document()?)?, model);
        assert_eq!(fs::read(&path)?, bytes);
        assert_eq!(engine.effective_tool(), EffectiveTool::Ring { size: 6 });
        assert!(!engine.selection().active);
        assert_eq!(engine.view().zoom_percent, Some(100.0));
        assert_eq!(
            engine.view().center,
            Some(bondline_model::Point2::new(100.0, 144.0)?)
        );
        assert!(
            !context.is_current(&engine),
            "{change} A→B→A must not revive old authority"
        );
        assert!(engine.ring_context(1)?.is_current(&engine));
    }
    let directory = support::FileTestDirectory::new()?;
    let (mut engine, path) = named_source(&directory)?;
    let old = engine.ring_context(1)?;
    let bytes = fs::read(&path)?;
    let request = engine.dispatch(
        "ui.file",
        json!({"expected_revision":engine.state().revision,"action":"save"}),
    )?;
    let id = request.ui_file_operation.ok_or("request")?;
    assert!(engine.ring_context(1).is_err());
    let plan = engine.claim_ui_file(id)?;
    assert!(engine.ring_context(1).is_err());
    let bondline_commands::UiFilePlan::SaveExisting { params, .. } = plan else {
        return Err("expected the actual captured SaveExisting plan".into());
    };
    let work = engine.prepare_ui_file(id, Command::SaveCdxml(params))?;
    assert!(
        engine.ring_context(1).is_err(),
        "Running file intent still blocks a native Ring gesture"
    );
    drop(work);
    assert!(!engine.finish_ui_file(id, false));
    assert_eq!(fs::read(path)?, bytes);
    assert!(!old.is_current(&engine));
    assert!(engine.ring_context(1)?.is_current(&engine));
    Ok(())
}

fn assert_preview_error(engine: &Engine, params: &RingCreate, code: &str) -> TestResult {
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.content().clone();
    let stamp = engine.capture_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let tool = engine.bond_tool();
    let mode = engine.effective_tool();
    let view = engine.view();
    let file = engine.ui_file_context_stamp();
    let error = engine
        .preview_ring_create(params)
        .err()
        .ok_or("invalid preview accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &owner);
    assert_eq!(engine.capture_stamp(), stamp);
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.effective_tool(), mode);
    assert_eq!(engine.view(), view);
    assert!(engine.ui_file_context_stamp_matches(&file));
    Ok(())
}

#[test]
fn explicit_preview_is_host_mode_independent_and_never_clears_selection_or_spends_ids() -> TestResult
{
    let mut engine =
        Engine::from_document(StoredDocument::from_cdxml(NATIVE.into(), PageSize::A4)?);
    assert_eq!(engine.effective_tool(), EffectiveTool::Bond);
    engine.dispatch(
        "ui.selection",
        json!({"action":"set","expected_revision":0,"page_id":1,"atom_ids":[1,2]}),
    )?;
    assert_eq!(engine.effective_tool(), EffectiveTool::Selection);
    let selection = engine.selection_context()?;
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let tool = engine.bond_tool();
    let model = engine.stored_document()?.model_snapshot();
    let old_fragment = Arc::clone(&engine.document()?.pages()[0].fragments()[0]);
    let raw_ptr = engine.stored_document()?.export_cdxml()?.as_ptr();
    let params = explicit_ring(&engine);
    let first = engine.preview_ring_create(&params)?;
    assert_eq!(first.model().pages()[0].fragments().len(), 2);
    assert!(Arc::ptr_eq(
        &old_fragment,
        &first.model().pages()[0].fragments()[0]
    ));
    let fragment = &first.model().pages()[0].fragments()[1];
    assert_eq!(fragment.id().get(), 2);
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|a| a.id.get())
            .collect::<Vec<_>>(),
        [3, 4, 5, 6, 7, 8]
    );
    assert_eq!(first.source_info().ok_or("preview source")?.format, "cdxml");
    assert_eq!(
        first.source_info().ok_or("preview source")?.byte_length,
        NATIVE.len()
    );
    assert!(first.source_info().ok_or("preview source")?.modified);
    for _ in 0..3 {
        let again = engine.preview_ring_create(&params)?;
        assert_eq!(again.model(), first.model());
    }
    let mut invalid = params.clone();
    invalid.expected_revision = 1;
    assert_preview_error(&engine, &invalid, "stale_revision")?;
    let mut invalid = params.clone();
    invalid.page_id = 9;
    assert_preview_error(&engine, &invalid, "not_found")?;
    let mut invalid = params.clone();
    invalid.size = 2;
    assert_preview_error(&engine, &invalid, "invalid_params")?;
    let mut invalid = params.clone();
    invalid.placement = RingPlacement::Bond {
        fragment_id: 1,
        bond_id: 99,
        side: RingSide::Negative,
    };
    assert_preview_error(&engine, &invalid, "not_found")?;
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut invalid = params.clone();
        invalid.placement = RingPlacement::Free {
            begin_x_pt: 300.0,
            begin_y_pt: 168.0,
            end_x_pt: bad,
            end_y_pt: 120.0,
            side: RingSide::Positive,
        };
        assert_preview_error(&engine, &invalid, "invalid_params")?;
    }
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &content);
    assert_eq!(engine.capture_stamp(), stamp);
    assert_eq!(engine.bond_tool(), tool);
    assert_eq!(engine.effective_tool(), EffectiveTool::Selection);
    assert!(selection.is_current(&engine));
    assert!(engine.ui_file_context_stamp_matches(&file));
    assert!(Arc::ptr_eq(
        &model,
        &engine.stored_document()?.model_snapshot()
    ));
    assert_eq!(engine.stored_document()?.export_cdxml()?, NATIVE);
    assert_eq!(engine.stored_document()?.export_cdxml()?.as_ptr(), raw_ptr);
    engine.execute(Command::RingCreate(params.clone()))?;
    assert_eq!(
        engine.document()?,
        first.model(),
        "preview/commit consistency, not a second analytic geometry oracle"
    );
    assert_eq!(engine.state().undo_depth, 1);
    assert_eq!(engine.effective_tool(), EffectiveTool::Selection);
    assert!(engine.selection().active && engine.selection().atom_ids.is_empty());
    assert!(!selection.is_current(&engine));
    engine.dispatch(
        "document.close",
        json!({"expected_revision":engine.state().revision,"action":"discard"}),
    )?;
    assert!(engine.complete_close());
    let mut obsolete = params;
    obsolete.expected_revision = 99;
    assert_preview_error(&engine, &obsolete, "close_committed")?;
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("retained.mol");
    let bytes = include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol");
    fs::write(&path, bytes)?;
    let mut foreign = Engine::default();
    foreign.execute(Command::UiRingToolSelect(SelectRingTool { size: 6 }))?;
    foreign.dispatch("document.open_mol", json!({"path":path}))?;
    let p = explicit_ring(&foreign);
    assert_preview_error(&foreign, &p, "unsupported_content")?;
    assert!(foreign.ring_context(1).is_err());
    assert_eq!(fs::read(path)?, bytes);
    // No renderer/glyph/native evidence is inferred from this complete candidate API.
    Ok(())
}
