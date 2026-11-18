//! One source-qualified, registered SMILES bond-order transaction.

use bondline_commands::{BondSetOrder, Cancellation, Engine, ErrorInfo};
use bondline_mcp::{CommandExecutor, UiSmilesRecordsContext, UiSmilesRecordsResult};
use eframe::egui;
use std::{
    sync::mpsc::{self, Receiver},
    thread::JoinHandle,
    time::Duration,
};

struct Pending {
    context: UiSmilesRecordsContext,
    params: BondSetOrder,
}

struct Job {
    context: UiSmilesRecordsContext,
    cancel: Cancellation,
    result: Receiver<Result<UiSmilesRecordsResult, ErrorInfo>>,
    worker: JoinHandle<()>,
}

#[derive(Default)]
pub(crate) struct OrderWorkflow {
    pending: Option<Pending>,
    job: Option<Job>,
    user_cancelled: bool,
    failure: Option<String>,
}

impl OrderWorkflow {
    pub(crate) fn is_busy(&self) -> bool {
        self.pending.is_some() || self.job.is_some()
    }

    pub(crate) fn queue(
        &mut self,
        engine: &Engine,
        params: BondSetOrder,
        context: &egui::Context,
        error: &mut Option<String>,
    ) {
        if self.is_busy() {
            return;
        }
        match UiSmilesRecordsContext::capture_order(
            engine,
            params.page_id,
            params.fragment_id,
            params.bond_id,
        ) {
            Ok(captured) => {
                self.pending = Some(Pending {
                    context: captured,
                    params,
                });
                self.user_cancelled = false;
                self.failure = None;
                *error = None;
                context.request_repaint();
            }
            Err(failure) => *error = Some(failure.message),
        }
    }

    pub(crate) fn cancel(&mut self) {
        self.user_cancelled = true;
        self.failure = None;
        self.cancel_for_context();
    }

    pub(crate) fn cancel_for_context(&mut self) {
        self.pending = None;
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
    }

    pub(crate) fn show_status(&mut self, ui: &mut egui::Ui) {
        if self.is_busy() {
            ui.separator();
            ui.spinner();
            ui.label("Changing bond order…");
            if ui.button("Cancel bond order").clicked() {
                self.cancel();
            }
            ui.ctx().request_repaint_after(Duration::from_millis(40));
        }
    }

    pub(crate) fn poll(
        &mut self,
        engine: Option<&Engine>,
        context: &egui::Context,
        error: &mut Option<String>,
    ) {
        if let Some(failure) = self.failure.take() {
            *error = Some(failure);
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| !engine.is_some_and(|engine| pending.context.is_current(engine)))
        {
            self.pending = None;
            *error = Some("The document context changed. The bond order was not queued.".into());
        }
        if let Some(job) = &self.job {
            if !engine.is_some_and(|engine| job.context.is_current(engine)) {
                job.cancel.cancel();
            }
            if !job.worker.is_finished() {
                context.request_repaint_after(Duration::from_millis(40));
                return;
            }
        }
        let Some(job) = self.job.take() else {
            return;
        };
        let joined = job.worker.join();
        match job.result.try_recv() {
            Ok(Ok(result)) => {
                *error = if engine.is_some_and(|engine| result.is_current(engine)) {
                    self.user_cancelled
                        .then(|| "The bond order was applied before cancellation completed.".into())
                } else {
                    Some("The bond order was applied before the document context changed. Review the current document.".into())
                };
            }
            Ok(Err(failure)) => {
                *error = (!self.user_cancelled).then_some(failure.message);
            }
            Err(_) => {
                *error = Some(
                    if joined.is_err() {
                        "The bond-order worker did not complete."
                    } else {
                        "The bond-order worker stopped without a result."
                    }
                    .into(),
                );
            }
        }
        self.user_cancelled = false;
        context.request_repaint();
    }

    pub(crate) fn start_pending(&mut self, executor: &CommandExecutor, context: &egui::Context) {
        if self.job.is_some() {
            return;
        }
        let Some(pending) = self.pending.take() else {
            return;
        };
        let work = match executor.prepare_ui_smiles_order(&pending.context, pending.params.clone())
        {
            Ok(work) => work,
            Err(failure) if failure.code == "busy" => {
                self.pending = Some(pending);
                context.request_repaint_after(Duration::from_millis(40));
                return;
            }
            Err(failure) => {
                self.user_cancelled = false;
                self.failure = Some(failure.message);
                context.request_repaint();
                return;
            }
        };
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let repaint = context.clone();
        let (sender, result) = mpsc::sync_channel(1);
        match std::thread::Builder::new()
            .name("bondline-smiles-order".into())
            .spawn(move || {
                let _ = sender.send(work.complete_blocking(&worker_cancel));
                repaint.request_repaint();
            }) {
            Ok(worker) => {
                self.job = Some(Job {
                    context: pending.context,
                    cancel,
                    result,
                    worker,
                });
            }
            Err(failure) => {
                self.failure = Some(format!(
                    "Cannot start bond-order change: {failure}. No worker ran."
                ));
                context.request_repaint();
            }
        }
    }

    pub(crate) fn deactivate(&mut self, context: &egui::Context, error: &mut Option<String>) {
        self.cancel_for_context();
        self.poll(None, context, error);
    }
}

impl Drop for OrderWorkflow {
    fn drop(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.cancel();
        }
    }
}

#[cfg(test)]
#[path = "native_smiles_order_lifecycle_tests.rs"]
mod tests;
