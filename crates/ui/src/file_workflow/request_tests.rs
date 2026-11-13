use super::*;
use bondline_commands::{UiFileAction, UiFilePhase};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn request(engine: &mut Engine, action: &str) -> Result<CommandResult, bondline_commands::Error> {
    engine.dispatch(
        "ui.file",
        serde_json::json!({"expected_revision":engine.state().revision,"action":action}),
    )
}

fn frame(workflow: &mut FileWorkflow, engine: &mut Engine, context: &egui::Context) {
    context.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1000.0, 700.0),
        )),
        ..Default::default()
    });
    workflow.dialogs(context, engine, &mut None, false);
    let _ = context.end_pass();
}

#[test]
fn remote_open_and_cancel_drive_the_real_picker_without_document_io() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let original = engine.stored_document().clone();
    let mut workflow = FileWorkflow::new(std::env::temp_dir());
    let context = egui::Context::default();
    crate::configure_context(&context);
    let requested = request(&mut engine, "open")?;
    let id = requested.ui_file.request.ok_or("missing file request")?.id;
    frame(&mut workflow, &mut engine, &context);
    assert!(
        workflow.chooser.is_open(),
        "remote Open must open the native picker"
    );
    assert_eq!(
        engine
            .state()
            .ui_file
            .request
            .ok_or("missing accepted request")?
            .phase,
        UiFilePhase::Accepted
    );
    engine.dispatch(
        "ui.file",
        serde_json::json!({"expected_revision":0,"action":"cancel","request_id":id}),
    )?;
    frame(&mut workflow, &mut engine, &context);
    assert!(
        !workflow.chooser.is_open(),
        "remote Cancel must dismiss the same picker"
    );
    assert!(engine.state().ui_file.request.is_none());
    assert_eq!(engine.stored_document(), &original);
    assert_eq!(engine.state().revision, 0);
    Ok(())
}

#[test]
fn native_open_shortcut_creates_the_same_shared_request_as_remote_open() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let mut workflow = FileWorkflow::new(std::env::temp_dir());
    let context = egui::Context::default();
    crate::configure_context(&context);
    context.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1000.0, 700.0),
        )),
        events: vec![egui::Event::Key {
            key: egui::Key::O,
            physical_key: Some(egui::Key::O),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        }],
        ..Default::default()
    });
    crate::show_workspace_with_files(
        &context,
        &mut engine,
        &mut None,
        &mut crate::DrawingState::default(),
        Some(&mut workflow),
        false,
    );
    let _ = context.end_pass();
    let accepted = engine
        .state()
        .ui_file
        .request
        .ok_or("shortcut bypassed ui.file")?;
    assert_eq!(accepted.action, UiFileAction::Open);
    assert_eq!(accepted.phase, UiFilePhase::Accepted);
    assert!(workflow.chooser.is_open());
    Ok(())
}

#[test]
fn close_specific_save_is_an_addressable_request_bound_to_the_current_close_epoch() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "page.set_size",
        serde_json::json!({"page_id":1,"width_pt":640,"height_pt":480}),
    )?;
    let mut workflow = FileWorkflow::new(std::env::temp_dir());
    workflow.request_close(&mut engine, &mut None);
    let epoch = engine.state().close_epoch;
    assert!(
        workflow
            .choose_close(CloseChoice::Save, &mut engine, &mut None)
            .is_none()
    );
    let pending = engine
        .state()
        .ui_file
        .request
        .ok_or("close Save bypassed ui.file")?;
    assert_eq!(pending.action, UiFileAction::Save);
    assert_eq!(pending.close_epoch, Some(epoch));
    assert_eq!(pending.input_revision, engine.state().revision);
    Ok(())
}

#[test]
fn an_old_picker_cancel_cannot_cancel_a_new_remote_request() -> TestResult {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let mut workflow = FileWorkflow::new(std::env::temp_dir());
    request(&mut engine, "open")?;
    workflow.adopt_request(&mut engine, &mut None);
    let old = workflow.request_id.ok_or("old picker")?;
    engine.dispatch(
        "ui.file",
        serde_json::json!({"expected_revision":0,"action":"cancel","request_id":old}),
    )?;
    let next = request(&mut engine, "save_as")?
        .ui_file
        .request
        .ok_or("new request")?;
    workflow.cancel_picker(&mut engine, &mut None);
    assert_eq!(
        engine.state().ui_file.request,
        Some(next),
        "the old picker's Cancel must retain request B"
    );
    Ok(())
}

#[test]
fn shifted_save_shortcut_selects_save_as_even_when_ordinary_save_is_available() -> TestResult {
    let directory = super::close_tests::Directory::new()?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch(
        "document.save_cdxml",
        serde_json::json!({"path":directory.0.join("known.cdxml")}),
    )?;
    let mut workflow = FileWorkflow::new(directory.0.clone());
    let context = egui::Context::default();
    crate::configure_context(&context);
    context.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1000.0, 700.0),
        )),
        events: vec![egui::Event::Key {
            key: egui::Key::S,
            physical_key: Some(egui::Key::S),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        }],
        ..Default::default()
    });
    let action = crate::show_workspace_with_files(
        &context,
        &mut engine,
        &mut None,
        &mut crate::DrawingState::default(),
        Some(&mut workflow),
        false,
    );
    let _ = context.end_pass();
    assert!(
        action.is_none(),
        "Save As must show a picker, not execute the known-path Save"
    );
    let request = engine.state().ui_file.request.ok_or("Save As request")?;
    assert_eq!(request.action, UiFileAction::SaveAs);
    assert_eq!(request.phase, UiFilePhase::Accepted);
    assert!(workflow.chooser.is_open());
    Ok(())
}
