use super::close_tests::Directory;
use super::mol_tests::wait_for_file;
use super::*;
use std::sync::{Arc, Mutex};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("../../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf");

#[test]
fn sdf_worker_open_known_save_and_save_as_retain_all_records_and_retry_the_same_family()
-> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("original.mol");
    let output = directory.0.join("collection λ.sd");
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
            format: FileFormat::Sdf,
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
        assert_eq!(engine.state().file.format, FileFormat::Sdf);
        assert!(engine.document().is_err());
        assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
        assert_eq!(engine.content().sdf()?.records().len(), 2);
        workflow.request_file(&mut engine, UiFileAction::Save, None, &mut None);
        let intent = workflow
            .adopt_request(&mut engine, &mut None)
            .ok_or("known Save")?;
        assert_eq!(intent.format, FileFormat::Sdf);
        drop(engine);
        workflow.start(intent, &executor, &egui::Context::default())?;
    }
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    assert_eq!(std::fs::read(&input)?, SOURCE);
    let id = {
        let mut engine = executor.engine().lock().map_err(|_| "engine")?;
        workflow.request_file(&mut engine, UiFileAction::SaveAs, None, &mut None);
        workflow.adopt_request(&mut engine, &mut None);
        assert_eq!(workflow.chooser.format(), FileFormat::Sdf);
        workflow.request_id.ok_or("Save As request")?
    };
    workflow.chooser.cancel();
    std::fs::write(&output, b"existing destination")?;
    workflow.start(
        FileIntentAction {
            request_id: id,
            format: FileFormat::Sdf,
            action: FileAction::Save {
                path: output.to_str().ok_or("path")?.to_owned(),
                overwrite: false,
            },
        },
        &executor,
        &egui::Context::default(),
    )?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_some());
    assert_eq!(std::fs::read(&output)?, b"existing destination");
    assert!(workflow.chooser.is_open());
    assert_eq!(workflow.chooser.format(), FileFormat::Sdf);
    assert_eq!(workflow.request_id, Some(id));
    workflow.chooser.cancel();
    workflow.start(
        FileIntentAction {
            request_id: id,
            format: FileFormat::Sdf,
            action: FileAction::Save {
                path: output.to_str().ok_or("path")?.to_owned(),
                overwrite: true,
            },
        },
        &executor,
        &egui::Context::default(),
    )?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    assert_eq!(std::fs::read(&output)?, SOURCE);
    let engine = executor.engine().lock().map_err(|_| "engine")?;
    assert_eq!(engine.state().file.format, FileFormat::Sdf);
    assert_eq!(
        engine.state().file.path.as_deref(),
        output.canonicalize()?.to_str()
    );
    assert!(!engine.state().file.dirty);
    assert_eq!(engine.content().sdf()?.as_bytes(), SOURCE);
    Ok(())
}
