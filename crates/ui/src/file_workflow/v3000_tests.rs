use super::close_tests::Directory;
use super::mol_tests::{add_bond, wait_for_file};
use super::*;
use std::sync::{Arc, Mutex};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORIGINAL_CC: &[u8] = include_bytes!("../../../io/tests/fixtures/v3000/noncontiguous-cc.mol");

#[test]
fn save_as_uses_the_explicit_mol_version_and_known_save_keeps_it_with_a_cdxml_suffix() -> TestResult
{
    for (format, version, stamp) in [
        (FileFormat::MolV2000, "v2000", " V2000"),
        (FileFormat::MolV3000, "v3000", " V3000"),
    ] {
        let directory = Directory::new()?;
        let path = directory.0.join("selected.mol");
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        add_bond(&mut engine, 72.0)?;
        let mut workflow = FileWorkflow::new(directory.0.clone());
        workflow.request_file(&mut engine, UiFileAction::SaveAs, None, &mut None);
        workflow.adopt_request(&mut engine, &mut None);
        let id = workflow.request_id.ok_or("request")?;
        workflow.chooser.cancel();
        let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
        workflow.start(
            FileIntentAction {
                request_id: id,
                format,
                action: FileAction::Save {
                    path: path.to_str().ok_or("path")?.to_owned(),
                    overwrite: false,
                },
            },
            &executor,
            &egui::Context::default(),
        )?;
        assert!(wait_for_file(&mut workflow, &executor)?.is_none());
        assert_eq!(
            std::fs::read_to_string(&path)?
                .lines()
                .nth(3)
                .and_then(|line| line.get(33..39)),
            Some(stamp)
        );
        let misleading = directory.0.join("associated.cdxml");
        let intent = {
            let mut engine = executor.engine().lock().map_err(|_| "engine lock")?;
            assert_eq!(engine.state().file.format, format);
            engine.dispatch(
                "document.save_mol",
                serde_json::json!({"path":misleading,"version":version}),
            )?;
            workflow.request_file(&mut engine, UiFileAction::SaveAs, None, &mut None);
            workflow.adopt_request(&mut engine, &mut None);
            assert_eq!(workflow.chooser.format(), format);
            workflow.cancel_picker(&mut engine, &mut None);
            workflow.request_file(&mut engine, UiFileAction::Save, None, &mut None);
            workflow
                .adopt_request(&mut engine, &mut None)
                .ok_or("known Save")?
        };
        assert_eq!(intent.format, format);
        let canonical = std::fs::canonicalize(&misleading)?;
        assert!(
            matches!(&intent.action,FileAction::Save{path,overwrite:true} if std::path::Path::new(path)==canonical)
        );
        workflow.start(intent, &executor, &egui::Context::default())?;
        assert!(wait_for_file(&mut workflow, &executor)?.is_none());
        assert_eq!(
            std::fs::read_to_string(&misleading)?
                .lines()
                .nth(3)
                .and_then(|line| line.get(33..39)),
            Some(stamp)
        );
    }
    Ok(())
}

#[test]
fn mol_family_open_decodes_actual_v3000_bytes_even_with_a_cdxml_suffix() -> TestResult {
    for format in [FileFormat::MolV2000, FileFormat::MolV3000] {
        let directory = Directory::new()?;
        let path = directory.0.join("actual-version.cdxml");
        std::fs::write(&path, ORIGINAL_CC)?;
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        let mut workflow = FileWorkflow::new(directory.0.clone());
        workflow.request_file(&mut engine, UiFileAction::Open, None, &mut None);
        workflow.adopt_request(&mut engine, &mut None);
        let request_id = workflow.request_id.ok_or("request")?;
        workflow.chooser.cancel();
        let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
        workflow.start(
            FileIntentAction {
                request_id,
                format,
                action: FileAction::Open {
                    path: path.to_str().ok_or("path")?.to_owned(),
                    discard_unsaved: false,
                },
            },
            &executor,
            &egui::Context::default(),
        )?;
        assert!(wait_for_file(&mut workflow, &executor)?.is_none());
        let engine = executor.engine().lock().map_err(|_| "engine lock")?;
        assert_eq!(engine.state().file.format, FileFormat::MolV3000);
        assert!(!engine.state().file.dirty);
        assert_eq!(engine.stored_document().export_mol()?, ORIGINAL_CC);
    }
    Ok(())
}

#[test]
fn v3000_downgrade_refusal_keeps_source_state_and_the_guarded_picker_retry() -> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("precise.mol");
    let source = std::str::from_utf8(ORIGINAL_CC)?.replace("-1.25", "-1.23456789");
    std::fs::write(&path, &source)?;
    let output = directory.0.join("must-not-publish.mol");
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_mol", serde_json::json!({"path":path}))?;
    let original = engine.stored_document().clone();
    let state = engine.state();
    let mut workflow = FileWorkflow::new(directory.0.clone());
    workflow.request_file(&mut engine, UiFileAction::SaveAs, None, &mut None);
    workflow.adopt_request(&mut engine, &mut None);
    assert_eq!(workflow.chooser.format(), FileFormat::MolV3000);
    let request_id = workflow.request_id.ok_or("request")?;
    workflow.chooser.cancel();
    let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
    workflow.start(
        FileIntentAction {
            request_id,
            format: FileFormat::MolV2000,
            action: FileAction::Save {
                path: output.to_str().ok_or("path")?.to_owned(),
                overwrite: false,
            },
        },
        &executor,
        &egui::Context::default(),
    )?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_some());
    let mut engine = executor.engine().lock().map_err(|_| "engine lock")?;
    assert_eq!(engine.stored_document(), &original);
    assert_eq!(engine.state().revision, state.revision);
    assert_eq!(engine.state().file.format, FileFormat::MolV3000);
    assert_eq!(engine.state().file.path, state.file.path);
    assert_eq!(std::fs::read(&path)?, source.as_bytes());
    assert!(!output.exists());
    assert!(workflow.chooser.is_open());
    assert_eq!(workflow.chooser.format(), FileFormat::MolV2000);
    assert_eq!(workflow.request_id, Some(request_id));
    workflow.cancel_picker(&mut engine, &mut None);
    assert!(engine.state().ui_file.request.is_none());
    Ok(())
}

#[test]
fn close_specific_known_v3000_save_uses_matching_receipt_before_approval() -> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("close-v3000.mol");
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "document.save_mol",
        serde_json::json!({"path":path,"version":"v3000"}),
    )?;
    add_bond(&mut engine, 72.0)?;
    let mut workflow = FileWorkflow::new(directory.0.clone());
    workflow.request_close(&mut engine, &mut None);
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    workflow.choose_close(CloseChoice::Save, &mut engine, &mut None);
    let intent = workflow
        .adopt_request(&mut engine, &mut None)
        .ok_or("close Save")?;
    assert_eq!(intent.format, FileFormat::MolV3000);
    let executor = CommandExecutor::new(Arc::new(Mutex::new(engine)));
    workflow.start(intent, &executor, &egui::Context::default())?;
    assert!(wait_for_file(&mut workflow, &executor)?.is_none());
    let mut engine = executor.engine().lock().map_err(|_| "engine lock")?;
    assert_eq!(engine.state().close, CloseState::Approved);
    assert_eq!(engine.state().file.format, FileFormat::MolV3000);
    assert!(!engine.state().file.dirty);
    assert!(workflow.complete_close_if_idle(&mut engine, executor.has_active_work(), false));
    assert_eq!(engine.state().close, CloseState::Committed);
    Ok(())
}
