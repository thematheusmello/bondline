use super::*;
use bondline_commands::{CloseAction, CloseDocument, CloseState, Engine};
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn native_engine() -> Engine {
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    engine
}
static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

pub(super) struct Directory(pub(super) PathBuf);
impl Directory {
    pub(super) fn new() -> Result<Self, std::io::Error> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
        fs::create_dir_all(&root)?;
        let path = root.join(format!(
            "ui-shared-close-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn close(
    engine: &mut Engine,
    action: CloseAction,
) -> Result<CommandResult, bondline_commands::Error> {
    engine.execute(Command::Close(CloseDocument {
        expected_revision: engine.state().revision,
        action,
    }))
}

fn edit(engine: &mut Engine, width: u32) -> Result<(), bondline_commands::Error> {
    engine.dispatch(
        "page.set_size",
        serde_json::json!({"page_id":1,"width_pt":width,"height_pt":480}),
    )?;
    Ok(())
}

fn frame(
    workflow: &mut FileWorkflow,
    engine: &mut Engine,
    context: &egui::Context,
) -> egui::FullOutput {
    frame_with_input(workflow, engine, context, vec![], false)
}

fn frame_with_input(
    workflow: &mut FileWorkflow,
    engine: &mut Engine,
    context: &egui::Context,
    events: Vec<egui::Event>,
    active_work: bool,
) -> egui::FullOutput {
    context.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1000.0, 700.0),
        )),
        events,
        ..Default::default()
    });
    workflow.dialogs(context, engine, &mut None, active_work);
    context.end_pass()
}

fn has_text(output: &egui::FullOutput, needle: &str) -> bool {
    output.shapes.iter().any(|shape| matches!(&shape.shape,egui::Shape::Text(text) if text.galley.text().contains(needle)))
}

#[test]
fn remote_close_state_is_adopted_and_ui_decisions_use_the_registry() -> TestResult {
    let mut engine = native_engine();
    edit(&mut engine, 640)?;
    let original = engine.stored_document()?.clone();
    let revision = engine.state().revision;
    let mut workflow = FileWorkflow::new(std::env::temp_dir());
    let context = egui::Context::default();
    crate::configure_context(&context);
    close(&mut engine, CloseAction::Request)?;
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    assert!(workflow.blocks_authoring(&engine.state()));
    // The first egui Window pass measures its area before painting visible text.
    let _ = frame(&mut workflow, &mut engine, &context);
    assert!(has_text(
        &frame(&mut workflow, &mut engine, &context),
        "Save your changes"
    ));
    close(&mut engine, CloseAction::Cancel)?;
    assert!(!workflow.blocks_authoring(&engine.state()));
    assert!(!has_text(
        &frame(&mut workflow, &mut engine, &context),
        "Save your changes"
    ));
    close(&mut engine, CloseAction::Request)?;
    let epoch = engine.state().close_epoch;
    workflow.choose_close(CloseChoice::Cancel, &mut engine, &mut None);
    assert_eq!(engine.state().close, CloseState::Open);
    assert_eq!(engine.state().close_epoch, epoch + 1);
    close(&mut engine, CloseAction::Request)?;
    let epoch = engine.state().close_epoch;
    workflow.choose_close(CloseChoice::Discard, &mut engine, &mut None);
    assert_eq!(engine.state().close, CloseState::Approved);
    assert_eq!(engine.state().close_epoch, epoch + 1);
    assert_eq!(engine.state().revision, revision);
    assert_eq!(engine.stored_document()?, &original);
    assert!(workflow.complete_close_if_idle(&mut engine, false, false));
    assert_eq!(engine.state().close, CloseState::Committed);
    Ok(())
}

#[test]
fn host_close_waits_for_both_shared_work_and_unconsumed_ui_results() -> TestResult {
    let mut engine = native_engine();
    let mut workflow = FileWorkflow::new(std::env::temp_dir());
    assert!(!workflow.complete_close_if_idle(&mut engine, false, false));
    close(&mut engine, CloseAction::Request)?;
    assert!(!workflow.complete_close_if_idle(&mut engine, true, false));
    let (_sender, result) = mpsc::sync_channel(1);
    workflow.job = Some(FileJob {
        format: FileFormat::Cdxml,
        request_id: 0,
        action: FileAction::Save {
            path: "pending.cdxml".to_owned(),
            overwrite: false,
        },
        result,
        cancel: Cancellation::default(),
        cancelling: false,
        close_save: None,
    });
    assert!(!workflow.complete_close_if_idle(&mut engine, false, false));
    assert_eq!(engine.state().close, CloseState::Approved);
    workflow.job = None;
    assert!(workflow.complete_close_if_idle(&mut engine, false, false));
    assert_eq!(engine.state().close, CloseState::Committed);
    Ok(())
}

#[test]
fn approved_close_cancels_an_ordinary_picker_before_it_can_accept_a_save() -> TestResult {
    for committed in [false, true] {
        let mut engine = native_engine();
        let mut workflow = FileWorkflow::new(std::env::temp_dir());
        workflow.request_file(&mut engine, UiFileAction::SaveAs, None, &mut None);
        let context = egui::Context::default();
        crate::configure_context(&context);
        let _ = frame(&mut workflow, &mut engine, &context);
        close(&mut engine, CloseAction::Request)?;
        if committed {
            assert!(engine.complete_close());
        }
        context.begin_pass(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 700.0),
            )),
            events: vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: Some(egui::Key::Enter),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        });
        let action = workflow.dialogs(&context, &mut engine, &mut None, false);
        let _ = context.end_pass();
        assert!(
            action.is_none(),
            "approved shutdown must cancel the ordinary picker before accepting its Save"
        );
        assert!(!workflow.chooser.is_open());
    }
    Ok(())
}

#[test]
fn accepted_file_action_failure_keeps_the_window_open_beyond_one_frame() -> TestResult {
    let mut engine = native_engine();
    let mut workflow = FileWorkflow::new(std::env::temp_dir());
    close(&mut engine, CloseAction::Request)?;
    assert!(!workflow.complete_close_if_idle(&mut engine, false, true));
    let mut error = Some("file worker could not start".to_owned());
    workflow.keep_open_after_file_failure(&mut engine, &mut error);
    assert_eq!(engine.state().close, CloseState::Open);
    assert!(
        error
            .as_deref()
            .is_some_and(|text| text.contains("file worker could not start"))
    );
    assert!(!workflow.complete_close_if_idle(&mut engine, false, false));
    close(&mut engine, CloseAction::Request)?;
    let (sender, result) = mpsc::sync_channel(1);
    workflow.job = Some(FileJob {
        format: FileFormat::Cdxml,
        request_id: 0,
        action: FileAction::Save {
            path: "failed.cdxml".to_owned(),
            overwrite: false,
        },
        result,
        cancel: Cancellation::default(),
        cancelling: false,
        close_save: None,
    });
    sender
        .send(Err(ErrorInfo {
            code: "permission_denied",
            message: "Save was not published".to_owned(),
        }))
        .map_err(|_| "completion send")?;
    workflow.poll(&mut engine, &mut error);
    assert_eq!(engine.state().close, CloseState::Open);
    assert!(
        !workflow.chooser.is_open(),
        "invalidated request cannot privately reopen"
    );
    assert!(
        error
            .as_deref()
            .is_some_and(|text| text.contains("Save was not published"))
    );
    assert!(!workflow.complete_close_if_idle(&mut engine, false, false));
    Ok(())
}

#[test]
fn only_the_matching_close_save_receipt_can_renew_the_same_close_epoch() -> TestResult {
    for scenario in [
        "matching",
        "ordinary_save",
        "stale_receipt",
        "wrong_path",
        "wrong_format",
        "wrong_receipt_format",
        "wrong_ui_request",
        "missing_ui_receipt",
        "new_edit",
        "cancel",
        "cancel_then_request",
        "published_error",
    ] {
        let directory = Directory::new()?;
        let path = directory.0.join("drawing.cdxml");
        let mut engine = native_engine();
        engine.dispatch("document.save_cdxml", serde_json::json!({"path":path}))?;
        edit(&mut engine, 640)?;
        close(&mut engine, CloseAction::Request)?;
        let mut workflow = FileWorkflow::new(directory.0.clone());
        workflow.choose_close(CloseChoice::Save, &mut engine, &mut None);
        let intent = workflow
            .adopt_request(&mut engine, &mut None)
            .ok_or("known path save")?;
        let ticket = workflow.save_for_close.take().ok_or("close save ticket")?;
        let FileAction::Save {
            path: exact_path,
            overwrite,
        } = &intent.action
        else {
            return Err("expected save".into());
        };
        let prepared = engine.prepare_ui_file(
            intent.request_id,
            Command::SaveCdxml(SaveCdxml {
                path: exact_path.clone(),
                overwrite: *overwrite,
            }),
        )?;
        if scenario == "cancel" || scenario == "cancel_then_request" {
            close(&mut engine, CloseAction::Cancel)?;
        }
        if scenario == "cancel_then_request" {
            close(&mut engine, CloseAction::Request)?;
        }
        let completion = if scenario == "published_error" {
            Err(ErrorInfo {
                code: "save_published",
                message: "Bytes were published but durability confirmation failed".to_owned(),
            })
        } else {
            let mut saved = engine.finish(prepared.complete()?)?;
            if scenario == "stale_receipt" {
                saved
                    .file_operation
                    .as_mut()
                    .ok_or("receipt")?
                    .input_revision = 0;
            }
            if scenario == "wrong_path" {
                saved.file_operation.as_mut().ok_or("receipt")?.path = "different.cdxml".to_owned();
            }
            if scenario == "wrong_format" {
                saved.file.format = FileFormat::MolV2000;
            }
            if scenario == "wrong_receipt_format" {
                saved.file_operation.as_mut().ok_or("receipt")?.format = FileFormat::MolV2000;
            }
            if scenario == "wrong_ui_request" {
                saved
                    .file_operation
                    .as_mut()
                    .ok_or("receipt")?
                    .ui_request_id = Some(0);
            }
            if scenario == "missing_ui_receipt" {
                saved
                    .file_operation
                    .as_mut()
                    .ok_or("receipt")?
                    .ui_request_current = None;
            }
            Ok(saved)
        };
        if scenario == "new_edit" {
            edit(&mut engine, 641)?;
        }
        let before = engine.state();
        let (sender, result) = mpsc::sync_channel(1);
        workflow.job = Some(FileJob {
            format: FileFormat::Cdxml,
            request_id: intent.request_id,
            action: intent.action,
            result,
            cancel: Cancellation::default(),
            cancelling: false,
            close_save: (scenario != "ordinary_save").then_some(ticket),
        });
        sender.send(completion).map_err(|_| "completion send")?;
        let mut error = None;
        workflow.poll(&mut engine, &mut error);
        let expected = if scenario == "matching" {
            CloseState::Approved
        } else {
            before.close
        };
        assert_eq!(engine.state().close, expected, "{scenario}");
        assert_eq!(
            engine.state().close_epoch,
            before.close_epoch + u64::from(scenario == "matching"),
            "{scenario}"
        );
        assert_eq!(engine.state().revision, before.revision, "{scenario}");
        if scenario == "published_error" {
            assert!(
                error
                    .as_deref()
                    .is_some_and(|text| text.contains("Bytes were published"))
            );
            assert!(engine.state().file.dirty);
            assert!(
                !workflow.chooser.is_open() && workflow.save_for_close.is_none(),
                "a published error must not reopen an obsolete retry picker"
            );
        }
    }
    Ok(())
}

#[test]
fn old_close_save_receipt_cannot_renew_after_a_same_path_revision_and_format_aba() -> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("closing.cdxml");
    let mut engine = native_engine();
    engine.dispatch("document.save_cdxml", serde_json::json!({"path":path}))?;
    engine.dispatch("bond.create",serde_json::json!({"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":108,"end_y_pt":72}))?;
    close(&mut engine, CloseAction::Request)?;
    let mut workflow = FileWorkflow::new(directory.0.clone());
    workflow.choose_close(CloseChoice::Save, &mut engine, &mut None);
    let intent = workflow
        .adopt_request(&mut engine, &mut None)
        .ok_or("known Save")?;
    let ticket = workflow.save_for_close.take().ok_or("close continuation")?;
    let FileAction::Save {
        path: exact_path,
        overwrite,
    } = &intent.action
    else {
        return Err("save action".into());
    };
    let prepared = engine.prepare_ui_file(
        intent.request_id,
        Command::SaveCdxml(SaveCdxml {
            path: exact_path.clone(),
            overwrite: *overwrite,
        }),
    )?;
    // The worker has finished and released its save reservation, but the native
    // adapter has not polled this successful result yet.
    let result = engine.finish(prepared.complete()?)?;
    let receipt = result.file_operation.as_ref().ok_or("receipt")?;
    assert_eq!(receipt.ui_request_id, Some(intent.request_id));
    assert_eq!(receipt.ui_request_current, Some(true));
    assert_eq!(result.file.format, FileFormat::Cdxml);
    assert_eq!(result.revision, ticket.revision);
    assert!(!result.file.dirty);
    engine.dispatch(
        "document.save_mol",
        serde_json::json!({"path":path,"overwrite":true}),
    )?;
    engine.dispatch(
        "document.save_cdxml",
        serde_json::json!({"path":path,"overwrite":true}),
    )?;
    let before_poll = engine.state();
    assert_eq!(before_poll.close, CloseState::NeedsDecision);
    assert_eq!(before_poll.file.format, result.file.format);
    assert_eq!(before_poll.file.path, result.file.path);
    assert_eq!(before_poll.revision, result.revision);
    assert!(!before_poll.file.dirty);
    let epoch = before_poll.close_epoch;
    let (sender, receiver) = mpsc::sync_channel(1);
    workflow.job = Some(FileJob {
        request_id: intent.request_id,
        format: intent.format,
        action: intent.action,
        result: receiver,
        cancel: Cancellation::default(),
        cancelling: false,
        close_save: Some(ticket),
    });
    sender.send(Ok(result)).map_err(|_| "send")?;
    let mut error = None;
    workflow.poll(&mut engine, &mut error);
    assert_eq!(engine.state().close, CloseState::NeedsDecision);
    assert_eq!(engine.state().close_epoch, epoch);
    assert!(
        error
            .as_deref()
            .is_some_and(|message| message.contains("Review"))
    );
    assert!(!workflow.complete_close_if_idle(&mut engine, false, false));
    Ok(())
}

#[test]
fn escape_cancels_shared_close_from_picker_or_busy_dialog_without_cancelling_work() -> TestResult {
    for mode in ["picker", "ui_work", "shared_work"] {
        let busy = mode != "picker";
        let shared_work = mode == "shared_work";
        let mut engine = native_engine();
        edit(&mut engine, 640)?;
        close(&mut engine, CloseAction::Request)?;
        let epoch = engine.state().close_epoch;
        let mut workflow = FileWorkflow::new(std::env::temp_dir());
        let mut worker_cancel = None;
        if mode == "ui_work" {
            let (_sender, result) = mpsc::sync_channel(1);
            let cancel = Cancellation::default();
            worker_cancel = Some(cancel.clone());
            workflow.job = Some(FileJob {
                format: FileFormat::Cdxml,
                request_id: 0,
                action: FileAction::Save {
                    path: "pending.cdxml".to_owned(),
                    overwrite: false,
                },
                result,
                cancel,
                cancelling: false,
                close_save: None,
            });
        } else if !busy {
            assert!(
                workflow
                    .choose_close(CloseChoice::Save, &mut engine, &mut None)
                    .is_none()
            );
            workflow.adopt_request(&mut engine, &mut None);
            assert!(workflow.chooser.is_open());
        }
        let context = egui::Context::default();
        crate::configure_context(&context);
        let _ = frame_with_input(&mut workflow, &mut engine, &context, vec![], shared_work);
        if busy {
            let visible =
                frame_with_input(&mut workflow, &mut engine, &context, vec![], shared_work);
            assert!(
                has_text(&visible, "Document work is still running"),
                "{mode}"
            );
        }
        let escape = egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: Some(egui::Key::Escape),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        let _ = frame_with_input(
            &mut workflow,
            &mut engine,
            &context,
            vec![escape],
            shared_work,
        );
        assert_eq!(engine.state().close, CloseState::Open);
        assert_eq!(engine.state().close_epoch, epoch + 1);
        assert!(!workflow.chooser.is_open());
        if let Some(cancel) = worker_cancel {
            assert!(!cancel.is_cancelled());
            assert!(workflow.is_busy());
        }
    }
    Ok(())
}
