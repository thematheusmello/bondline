use super::close_tests::Directory;
use super::mol_tests::wait_for_file;
use super::*;
use std::sync::{Arc, Mutex};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = b"\n \xffprivate ignored\r\nCC \xff\0private name\nC#N second\r\n";

#[test]
fn smiles_worker_open_save_as_retry_known_save_and_close_retain_the_whole_file() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("source.mol");
    let output = directory.0.join("retained λ.smiles");
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
            format: FileFormat::Smiles,
            action: FileAction::Open {
                path: input.to_str().ok_or("path")?.to_owned(),
                discard_unsaved: false,
            },
        },
        &executor,
        &egui::Context::default(),
    )?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    let id = {
        let mut engine = executor.engine().lock().map_err(|_| "engine")?;
        assert_eq!(engine.state().file.format, FileFormat::Smiles);
        assert!(engine.document().is_err());
        assert_eq!(
            engine
                .content()
                .smiles_summary()
                .ok_or("summary")?
                .record_count,
            2
        );
        workflow.request_file(&mut engine, UiFileAction::SaveAs, None, &mut None);
        workflow.adopt_request(&mut engine, &mut None);
        assert_eq!(workflow.chooser.format(), FileFormat::Smiles);
        workflow.request_id.ok_or("Save As request")?
    };
    workflow.chooser.cancel();
    std::fs::write(&output, b"existing destination")?;
    let output_path = output.to_str().ok_or("path")?.to_owned();
    let save = |overwrite| FileIntentAction {
        request_id: id,
        format: FileFormat::Smiles,
        action: FileAction::Save {
            path: output_path.clone(),
            overwrite,
        },
    };
    workflow.start(save(false), &executor, &egui::Context::default())?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_some());
    assert_eq!(std::fs::read(&output)?, b"existing destination");
    assert!(workflow.chooser.is_open());
    assert_eq!(workflow.chooser.format(), FileFormat::Smiles);
    assert_eq!(workflow.request_id, Some(id));
    workflow.chooser.cancel();
    workflow.start(save(true), &executor, &egui::Context::default())?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    assert_eq!(std::fs::read(&output)?, SOURCE);
    let known = {
        let mut engine = executor.engine().lock().map_err(|_| "engine")?;
        assert_eq!(
            engine.state().file.path.as_deref(),
            output.canonicalize()?.to_str()
        );
        assert!(!engine.state().file.dirty);
        workflow.request_file(&mut engine, UiFileAction::Save, None, &mut None);
        workflow
            .adopt_request(&mut engine, &mut None)
            .ok_or("known Save")?
    };
    assert_eq!(known.format, FileFormat::Smiles);
    workflow.start(known, &executor, &egui::Context::default())?;
    {
        let mut engine = executor.engine().lock().map_err(|_| "engine")?;
        workflow.request_close(&mut engine, &mut None);
        assert!(!workflow.complete_close_if_idle(&mut engine, executor.has_active_work(), false));
    }
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    assert_eq!(std::fs::read(&output)?, SOURCE);
    assert_eq!(std::fs::read(&input)?, SOURCE);
    let mut engine = executor.engine().lock().map_err(|_| "engine")?;
    assert!(workflow.complete_close_if_idle(&mut engine, executor.has_active_work(), false));
    assert_eq!(engine.state().close, CloseState::Committed);
    Ok(())
}

#[test]
fn malformed_later_smiles_record_preserves_the_open_session_and_decoder_retry() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("source.smi");
    let invalid = directory.0.join("bad.smiles");
    std::fs::write(&input, SOURCE)?;
    std::fs::write(&invalid, b"CC first\nC1 invalid second\n")?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_smiles", serde_json::json!({"path":input}))?;
    let original = engine.content().clone();
    let before = engine.state();
    let mut workflow = FileWorkflow::new(directory.0.clone());
    workflow.request_file(&mut engine, UiFileAction::Open, None, &mut None);
    workflow.adopt_request(&mut engine, &mut None);
    let id = workflow.request_id.ok_or("request")?;
    workflow.chooser.cancel();
    let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
    workflow.start(
        FileIntentAction {
            request_id: id,
            format: FileFormat::Smiles,
            action: FileAction::Open {
                path: invalid.to_str().ok_or("path")?.to_owned(),
                discard_unsaved: false,
            },
        },
        &executor,
        &egui::Context::default(),
    )?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_some());
    let engine = executor.engine().lock().map_err(|_| "engine")?;
    assert_eq!(engine.content(), &original);
    assert_eq!(engine.state().revision, before.revision);
    assert_eq!(engine.state().file.path, before.file.path);
    assert_eq!(engine.state().file.format, FileFormat::Smiles);
    assert!(!engine.state().file.dirty);
    assert!(workflow.chooser.is_open());
    assert_eq!(workflow.chooser.format(), FileFormat::Smiles);
    assert_eq!(workflow.request_id, Some(id));
    assert_eq!(std::fs::read(&input)?, SOURCE);
    Ok(())
}
