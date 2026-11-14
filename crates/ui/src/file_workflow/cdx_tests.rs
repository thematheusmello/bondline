use super::close_tests::Directory;
use super::mol_tests::wait_for_file;
use super::*;
use std::sync::{Arc, Mutex};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("../../../io/tests/fixtures/cdx/metadata-header.cdx");

#[test]
fn cdx_open_known_save_and_save_as_keep_complete_source_and_associated_format() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("original.mol");
    let copy = directory.0.join("copy.cdx");
    std::fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let mut workflow = FileWorkflow::new(directory.0.clone());
    workflow.request_file(&mut engine, UiFileAction::Open, None, &mut None);
    workflow.adopt_request(&mut engine, &mut None);
    let id = workflow.request_id.ok_or("Open request")?;
    workflow.chooser.cancel();
    let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
    workflow.start(
        FileIntentAction {
            request_id: id,
            format: FileFormat::Cdx,
            action: FileAction::Open {
                path: input.to_str().ok_or("path")?.to_owned(),
                discard_unsaved: false,
            },
        },
        &executor,
        &egui::Context::default(),
    )?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    {
        let mut engine = executor.engine().lock().map_err(|_| "engine")?;
        assert_eq!(engine.state().file.format, FileFormat::Cdx);
        assert!(!engine.stored_document().is_fully_mapped());
        assert_eq!(engine.stored_document().export_cdx()?, SOURCE);
        workflow.request_file(&mut engine, UiFileAction::Save, None, &mut None);
        let intent = workflow
            .adopt_request(&mut engine, &mut None)
            .ok_or("known Save")?;
        assert_eq!(intent.format, FileFormat::Cdx);
        drop(engine);
        workflow.start(intent, &executor, &egui::Context::default())?;
    }
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    assert_eq!(std::fs::read(&input)?, SOURCE);
    let id = {
        let mut engine = executor.engine().lock().map_err(|_| "engine")?;
        workflow.request_file(&mut engine, UiFileAction::SaveAs, None, &mut None);
        workflow.adopt_request(&mut engine, &mut None);
        assert_eq!(workflow.chooser.format(), FileFormat::Cdx);
        workflow.request_id.ok_or("Save As request")?
    };
    workflow.chooser.cancel();
    workflow.start(
        FileIntentAction {
            request_id: id,
            format: FileFormat::Cdx,
            action: FileAction::Save {
                path: copy.to_str().ok_or("path")?.to_owned(),
                overwrite: false,
            },
        },
        &executor,
        &egui::Context::default(),
    )?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    assert_eq!(std::fs::read(copy)?, SOURCE);
    let engine = executor.engine().lock().map_err(|_| "engine")?;
    assert_eq!(engine.state().file.format, FileFormat::Cdx);
    assert!(!engine.state().file.dirty);
    Ok(())
}

#[test]
fn changed_cdx_close_save_refuses_and_retains_the_exact_known_path_retry() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("known.cdxml");
    std::fs::write(&input, SOURCE)?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_cdx", serde_json::json!({"path":input}))?;
    engine.dispatch(
        "page.set_size",
        serde_json::json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let snapshot = engine.stored_document().clone();
    let mut workflow = FileWorkflow::new(directory.0.clone());
    workflow.request_close(&mut engine, &mut None);
    workflow.choose_close(CloseChoice::Save, &mut engine, &mut None);
    let intent = workflow
        .adopt_request(&mut engine, &mut None)
        .ok_or("close Save")?;
    assert_eq!(intent.format, FileFormat::Cdx);
    assert_eq!(
        intent.action,
        FileAction::Save {
            path: input.canonicalize()?.to_str().ok_or("path")?.to_owned(),
            overwrite: true
        }
    );
    let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
    workflow.start(intent, &executor, &egui::Context::default())?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_some());
    assert_eq!(std::fs::read(&input)?, SOURCE);
    assert!(workflow.chooser.is_open());
    assert_eq!(workflow.chooser.format(), FileFormat::Cdx);
    let mut engine = executor.engine().lock().map_err(|_| "engine")?;
    assert_eq!(engine.stored_document(), &snapshot);
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    assert!(!workflow.complete_close_if_idle(&mut engine, false, false));
    workflow.cancel_picker(&mut engine, &mut None);
    assert_eq!(engine.state().close, CloseState::Open);
    assert!(!workflow.chooser.is_open());
    Ok(())
}
