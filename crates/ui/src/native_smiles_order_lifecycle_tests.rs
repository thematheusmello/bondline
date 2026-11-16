//! Independent actual registered producer: ready result is not thread retirement.
use super::*;
use std::{
    sync::{Arc, Mutex, mpsc},
    thread::JoinHandle,
    time::{Duration, Instant},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "reuse unchanged isolated file directory"
)]
mod file_support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = b"CC first\r\nC1.N.C1 \t\0\xffPRIVATE-META\r\nCC duplicate\tname";
const EDITED: &[u8] = b"CC first\r\n[CH2]=[CH2].[NH3] \t\0\xffPRIVATE-META\r\nCC duplicate\tname";

struct Done(mpsc::SyncSender<()>);
impl Drop for Done {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}
struct HeldProducer {
    workflow: OrderWorkflow,
    release: Option<mpsc::SyncSender<()>>,
    done: Option<mpsc::Receiver<()>>,
}
impl HeldProducer {
    fn release_and_observe(&mut self) -> TestResult {
        let released = self
            .release
            .take()
            .map(|release| release.send(()))
            .transpose();
        let observed = self
            .done
            .take()
            .map(|done| done.recv_timeout(Duration::from_secs(10)))
            .transpose();
        // Even an already-exited producer must be observed before reporting a
        // failed release; cleanup cannot strand the independent done receipt.
        released?;
        observed?;
        Ok(())
    }
}
impl Drop for HeldProducer {
    fn drop(&mut self) {
        let _ = self.release_and_observe();
        // Clean up the actual owned handle on an assertion/error path too.
        // Never substitute a completed handle or fabricate a work result.
        if let Some(job) = self.workflow.job.take() {
            job.cancel.cancel();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !job.worker.is_finished() && Instant::now() < deadline {
                std::thread::yield_now();
            }
            if job.worker.is_finished() {
                let _ = job.worker.join();
            }
        }
    }
}

#[test]
fn real_sent_order_result_remains_busy_and_blocks_close_until_producer_thread_retires() -> TestResult
{
    let directory = file_support::FileTestDirectory::new()?;
    let path = directory.path("held-native-order-原.smi");
    std::fs::write(&path, SOURCE)?;
    let mut engine = bondline_commands::Engine::default();
    engine.set_ui_file_host(true);
    engine.dispatch("document.open_smiles", serde_json::json!({"path":path}))?;
    engine.dispatch("document.import_smiles_records",serde_json::json!({"expected_revision":1,"selection":{"mode":"all"},"bond_length_pt":48.0,"origin_x_pt":72.0,"origin_y_pt":72.0,"page_width_pt":210.0*72.0/25.4,"page_height_pt":297.0*72.0/25.4}))?;
    let context = bondline_mcp::UiSmilesRecordsContext::capture_order(&engine, 1, 2, 2)
        .map_err(|e| e.message)?;
    let engine = Arc::new(Mutex::new(engine));
    let executor = bondline_mcp::CommandExecutor::new(Arc::clone(&engine));
    let params: bondline_commands::BondSetOrder = serde_json::from_value(
        serde_json::json!({"expected_revision":2,"page_id":1,"fragment_id":2,"bond_id":2,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
    )?;
    let work = executor
        .prepare_ui_smiles_order(&context, params)
        .map_err(|e| e.message)?;
    let cancel = bondline_commands::Cancellation::default();
    let producer_cancel = cancel.clone();
    let (sender, result) = mpsc::sync_channel(1);
    let (sent_sender, sent) = mpsc::sync_channel(1);
    let (release, gate) = mpsc::sync_channel(1);
    let (done_sender, done) = mpsc::sync_channel(1);
    let worker: JoinHandle<()> = std::thread::Builder::new()
        .name("independent-native-order-producer".into())
        .spawn(move || {
            let _done = Done(done_sender);
            let actual = work.complete_blocking(&producer_cancel);
            if sender.send(actual).is_ok() {
                let _ = sent_sender.send(());
            }
            // A finite gate, released by the cleanup owner on every exit path.
            let _ = gate.recv_timeout(Duration::from_secs(10));
        })?;
    let mut held = HeldProducer {
        workflow: OrderWorkflow::default(),
        release: Some(release),
        done: Some(done),
    };
    held.workflow.job = Some(Job {
        context,
        cancel,
        result,
        worker,
    });
    sent.recv_timeout(Duration::from_secs(10))?;
    assert!(
        !executor.has_active_work(),
        "registered finish has released its permit"
    );
    assert!(held.workflow.is_busy());
    let ctx = eframe::egui::Context::default();
    let mut error = None;
    let edited = {
        let mut engine = engine.lock().map_err(|_| "engine")?;
        assert_eq!(engine.state().revision, 3);
        assert_eq!(engine.state().undo_depth, 1);
        assert!(engine.state().file.dirty);
        assert_eq!(engine.content().smiles_file()?.as_bytes(), EDITED);
        let edited = engine.content().clone();
        held.workflow.poll(Some(&engine), &ctx, &mut error);
        assert!(
            held.workflow.is_busy(),
            "ready channel must not consume/join an unretired producer"
        );
        assert!(held.workflow.job.is_some());
        assert!(
            !held.workflow.user_cancelled,
            "context cancellation is not a user cancellation"
        );
        engine.dispatch(
            "document.close",
            serde_json::json!({"expected_revision":3,"action":"request"}),
        )?;
        engine.dispatch(
            "document.close",
            serde_json::json!({"expected_revision":3,"action":"discard"}),
        )?;
        assert_eq!(
            engine.state().close,
            bondline_commands::CloseState::Approved
        );
        let files = crate::file_workflow::FileWorkflow::new(directory.path("."));
        assert!(!files.complete_close_if_idle(
            &mut engine,
            held.workflow.is_busy() || executor.has_active_work(),
            false
        ));
        assert_eq!(
            engine.state().close,
            bondline_commands::CloseState::Approved
        );
        engine.dispatch(
            "document.close",
            serde_json::json!({"expected_revision":3,"action":"cancel"}),
        )?;
        edited
    };
    held.release_and_observe()?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let current = engine.lock().map_err(|_| "engine")?;
        held.workflow.poll(Some(&current), &ctx, &mut error);
        if !held.workflow.is_busy() {
            break;
        }
        drop(current);
        if Instant::now() >= deadline {
            return Err("actual sent producer did not retire".into());
        }
        std::thread::yield_now();
    }
    assert!(held.workflow.job.is_none());
    assert!(!executor.has_active_work());
    let engine = engine.lock().map_err(|_| "engine")?;
    assert_eq!(engine.content(), &edited);
    assert_eq!(engine.state().revision, 3);
    assert_eq!(engine.state().undo_depth, 1);
    assert_eq!(engine.state().close, bondline_commands::CloseState::Open);
    assert_eq!(engine.content().smiles_file()?.as_bytes(), EDITED);
    assert!(engine.state().file_operation.is_none());
    assert_eq!(std::fs::read(path)?, SOURCE);
    Ok(())
}
