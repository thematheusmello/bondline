use super::close_tests::Directory;
use super::*;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn add_bond(engine: &mut Engine, y: f64) -> Result<(), bondline_commands::Error> {
    engine.dispatch("bond.create",serde_json::json!({"page_id":1,"begin_x_pt":72.0,"begin_y_pt":y,"end_x_pt":108.0,"end_y_pt":y}))?;
    Ok(())
}

fn wait_for_file(
    workflow: &mut FileWorkflow,
    executor: &CommandExecutor,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut error = None;
    while workflow.is_busy() {
        {
            let mut engine = executor.engine().lock().map_err(|_| "engine lock")?;
            workflow.poll(&mut engine, &mut error);
        }
        if workflow.is_busy() {
            if Instant::now() >= deadline {
                return Err("file worker did not complete".into());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    Ok(error)
}

#[test]
fn known_save_routes_the_associated_mol_format_even_when_the_name_ends_cdxml() -> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("associated.cdxml");
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    add_bond(&mut engine, 72.0)?;
    engine.dispatch("document.save_mol", serde_json::json!({"path":path}))?;
    let mut workflow = FileWorkflow::new(directory.0.clone());
    workflow.request_file(&mut engine, UiFileAction::Save, None, &mut None);
    let intent = workflow
        .adopt_request(&mut engine, &mut None)
        .ok_or("known Save")?;
    assert_eq!(intent.format, FileFormat::MolV2000);
    let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
    workflow.start(intent, &executor, &egui::Context::default())?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    assert_eq!(
        executor
            .engine()
            .lock()
            .map_err(|_| "engine lock")?
            .state()
            .file
            .format,
        FileFormat::MolV2000
    );
    let text = std::fs::read_to_string(path)?;
    assert_eq!(
        text.lines().nth(3).and_then(|line| line.get(33..39)),
        Some(" V2000")
    );
    Ok(())
}

#[test]
fn selected_mol_decoder_refuses_xml_and_explicit_cdxml_retry_can_open_the_same_name() -> TestResult
{
    let directory = Directory::new()?;
    let path = directory.0.join("misnamed.mol");
    let document = Engine::default();
    std::fs::write(&path, document.stored_document().to_cdxml()?.as_ref())?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    add_bond(&mut engine, 72.0)?;
    let original = engine.stored_document().clone();
    let revision = engine.state().revision;
    let mut workflow = FileWorkflow::new(directory.0.clone());
    workflow.request_file(&mut engine, UiFileAction::Open, None, &mut None);
    workflow.adopt_request(&mut engine, &mut None);
    let id = workflow.request_id.ok_or("request")?;
    let action = FileAction::Open {
        path: path.to_str().ok_or("path")?.to_owned(),
        discard_unsaved: true,
    };
    let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
    workflow.chooser.cancel();
    workflow.start(
        FileIntentAction {
            request_id: id,
            format: FileFormat::MolV2000,
            action: action.clone(),
        },
        &executor,
        &egui::Context::default(),
    )?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_some());
    {
        let engine = executor.engine().lock().map_err(|_| "engine lock")?;
        assert_eq!(engine.stored_document(), &original);
        assert_eq!(engine.state().revision, revision);
        assert_eq!(engine.state().file.format, FileFormat::Cdxml);
    }
    assert!(workflow.chooser.is_open());
    assert_eq!(workflow.chooser.format(), FileFormat::MolV2000);
    workflow.chooser.cancel();
    workflow.start(
        FileIntentAction {
            request_id: id,
            format: FileFormat::Cdxml,
            action,
        },
        &executor,
        &egui::Context::default(),
    )?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    let engine = executor.engine().lock().map_err(|_| "engine lock")?;
    assert_eq!(engine.state().file.format, FileFormat::Cdxml);
    assert!(
        engine
            .document()
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .is_empty()
    );
    Ok(())
}

#[test]
fn mol_save_as_rejects_multiple_fragments_without_publication_and_retains_format_for_retry()
-> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("unsupported.mol");
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    add_bond(&mut engine, 72.0)?;
    add_bond(&mut engine, 144.0)?;
    let original = engine.stored_document().clone();
    let revision = engine.state().revision;
    let mut workflow = FileWorkflow::new(directory.0.clone());
    workflow.request_file(&mut engine, UiFileAction::SaveAs, None, &mut None);
    workflow.adopt_request(&mut engine, &mut None);
    let id = workflow.request_id.ok_or("request")?;
    workflow.chooser.cancel();
    let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
    let start_error = workflow
        .start(
            FileIntentAction {
                request_id: id,
                format: FileFormat::MolV2000,
                action: FileAction::Save {
                    path: path.to_str().ok_or("path")?.to_owned(),
                    overwrite: false,
                },
            },
            &executor,
            &egui::Context::default(),
        )
        .err();
    let error = wait_for_file(&mut workflow, &executor)?.or(start_error);
    assert!(error.is_some());
    assert!(!path.exists());
    let engine = executor.engine().lock().map_err(|_| "engine lock")?;
    assert_eq!(engine.stored_document(), &original);
    assert_eq!(engine.state().revision, revision);
    assert!(engine.state().file.dirty);
    assert_eq!(workflow.chooser.format(), FileFormat::MolV2000);
    assert!(workflow.chooser.is_open());
    Ok(())
}
