//! Independent real-work regression for conversion-owner destruction.
use super::*;
use serde_json::json;
use std::fs;

struct ReleaseGate(Option<mpsc::Sender<()>>);

impl ReleaseGate {
    fn release(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl Drop for ReleaseGate {
    fn drop(&mut self) {
        self.release();
    }
}

#[test]
fn dropping_conversion_owner_cancels_real_gated_work_before_it_can_publish()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = close_tests::Directory::new()?;
    let input = directory.0.join("drop-owner-化学.smi");
    let output = directory.0.join("drop-owner-new-化学.cdxml");
    fs::write(&input, b"CC\n")?;
    let mut engine = Engine::default();
    engine.set_ui_file_host(true);
    let mut workflow = FileWorkflow::new(directory.0.clone());
    engine.dispatch(
        "ui.file",
        json!({"expected_revision":0,"action":"convert_smiles"}),
    )?;
    assert!(workflow.adopt_request(&mut engine, &mut None).is_none());
    let context = egui::Context::default();
    workflow.queue_conversion(
        ConvertSmilesToCdxml {
            expected_revision: 0,
            input_path: input.to_string_lossy().into_owned(),
            output_path: output.to_string_lossy().into_owned(),
            bond_length_pt: 48.0,
            origin_x_pt: 72.0,
            origin_y_pt: 72.0,
            page_width_pt: 210.0 * 72.0 / 25.4,
            page_height_pt: 297.0 * 72.0 / 25.4,
        },
        &engine,
        &context,
    );
    let pending = workflow.conversion.pending.take().ok_or("queued work")?;
    let captured = workflow
        .conversion
        .form
        .as_ref()
        .ok_or("conversion form")?
        .context
        .clone();
    let engine = std::sync::Arc::new(std::sync::Mutex::new(engine));
    let executor = CommandExecutor::new(std::sync::Arc::clone(&engine));
    let work = executor
        .prepare_ui_smiles_conversion(pending.id, pending.params)
        .map_err(|error| error.message)?;
    workflow
        .conversion
        .form
        .as_mut()
        .ok_or("conversion form")?
        .accepted = None;
    let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
    let cancel = Cancellation::default();
    let worker_cancel = cancel.clone();
    let (release, released) = mpsc::channel();
    let mut gate = ReleaseGate(Some(release));
    let (entered, started) = mpsc::sync_channel(1);
    let (completed, completion) = mpsc::sync_channel(1);
    let (sender, result) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let _ = entered.send(());
        let waited = released.recv_timeout(Duration::from_secs(10));
        let actual = work.complete_blocking(&worker_cancel);
        let summary = match &actual {
            Ok(result) => ("ok".to_owned(), result.conversion.is_some(), waited.is_ok()),
            Err(error) => (error.code.to_owned(), false, waited.is_ok()),
        };
        // This is the real result channel. Dropping the owner also drops its receiver.
        let _ = sender.send(actual);
        let _ = completed.send(summary);
    });
    workflow.conversion.job = Some(Job {
        context: captured,
        cancel: cancel.clone(),
        result,
        worker,
    });
    let started = started.recv_timeout(Duration::from_secs(10));
    drop(workflow);
    let cancelled_at_drop = cancel.is_cancelled();
    gate.release();
    // All expected assertions happen after releasing the gate and observing completion.
    let actual = completion.recv_timeout(Duration::from_secs(10))?;
    started?;
    assert!(actual.2, "worker was not released by the finite test gate");
    assert!(!executor.has_active_work());
    assert_eq!(fs::read(&input)?, b"CC\n");
    assert_eq!(
        serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
        before
    );
    assert!(
        !output.exists(),
        "Dropped conversion owner published after its gate was released: \
         result={actual:?}, cancellation_at_drop={cancelled_at_drop}"
    );
    assert_eq!(actual.0, "cancelled");
    assert!(!actual.1 && cancelled_at_drop);
    Ok(())
}
