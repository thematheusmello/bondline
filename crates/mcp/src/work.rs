use crate::SharedEngine;
use bondline_commands::{Cancellation, Command, CommandResult, Error, ErrorInfo, Prepared};
use std::{future::Future, sync::Arc};
use tokio::sync::Semaphore;

#[derive(Clone)]
pub(crate) struct Workers(Arc<Semaphore>);

/// A clone shares the same two-worker budget across native UI and loopback requests.
#[derive(Clone)]
pub struct CommandExecutor {
    engine: SharedEngine,
    workers: Workers,
}

impl CommandExecutor {
    pub fn new(engine: SharedEngine) -> Self {
        Self {
            engine,
            workers: Workers::default(),
        }
    }

    pub fn engine(&self) -> &SharedEngine {
        &self.engine
    }

    /// Call from a background thread. Engine guards cover only preparation and application.
    pub fn execute_blocking(
        &self,
        command: Command,
        cancel: &Cancellation,
    ) -> Result<CommandResult, ErrorInfo> {
        let prepared = self
            .engine
            .lock()
            .map_err(|_| unavailable())?
            .prepare(command)
            .map_err(|error| error.info())?;
        let _permit = if prepared.is_deferred() {
            Some(self.workers.permit()?)
        } else {
            None
        };
        let completed = prepared
            .complete_with_cancel(cancel)
            .map_err(|error| error.info())?;
        self.engine
            .lock()
            .map_err(|_| unavailable())?
            .finish(completed)
            .map_err(|error| error.info())
    }

    pub async fn execute(
        &self,
        command: Command,
        cancelled: impl Future<Output = ()>,
    ) -> Result<CommandResult, ErrorInfo> {
        let prepared = self
            .engine
            .lock()
            .map_err(|_| unavailable())?
            .prepare(command)
            .map_err(|error| error.info())?;
        self.complete(prepared, cancelled).await
    }

    pub(crate) async fn complete(
        &self,
        prepared: Prepared,
        cancelled: impl Future<Output = ()>,
    ) -> Result<CommandResult, ErrorInfo> {
        if !prepared.is_deferred() {
            let completed = prepared.complete().map_err(|error| error.info())?;
            return self
                .engine
                .lock()
                .map_err(|_| unavailable())?
                .finish(completed)
                .map_err(|error| error.info());
        }
        let engine = Arc::clone(&self.engine);
        self.workers
            .run(
                move |cancel| {
                    let completed = prepared.complete_with_cancel(cancel)?;
                    engine
                        .lock()
                        .map_err(|_| Error::EngineUnavailable)?
                        .finish(completed)
                },
                cancelled,
            )
            .await
    }
}

fn unavailable() -> ErrorInfo {
    ErrorInfo {
        code: "engine_unavailable",
        message: "document engine lock poisoned".to_owned(),
    }
}

impl Default for Workers {
    fn default() -> Self {
        Self(Arc::new(Semaphore::new(2)))
    }
}

struct CancelOnDrop(Cancellation);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

impl Workers {
    fn permit(&self) -> Result<tokio::sync::OwnedSemaphorePermit, ErrorInfo> {
        Arc::clone(&self.0)
            .try_acquire_owned()
            .map_err(|_| ErrorInfo {
                code: "busy",
                message: "two document workers are already running; retry later".to_owned(),
            })
    }

    async fn run(
        &self,
        operation: impl FnOnce(&Cancellation) -> Result<CommandResult, Error> + Send + 'static,
        cancelled: impl Future<Output = ()>,
    ) -> Result<CommandResult, ErrorInfo> {
        tokio::pin!(cancelled);
        tokio::select! {
            biased;
            _ = &mut cancelled => return Err(ErrorInfo {code:"cancelled",message:"operation cancelled before work started".to_owned()}),
            _ = std::future::ready(()) => {},
        }
        let permit = self.permit()?;
        let guard = CancelOnDrop(Cancellation::default());
        let cancel = guard.0.clone();
        let mut job = tokio::task::spawn_blocking(move || {
            // A dropped awaiting task must not free a still-running worker's budget.
            let _permit = permit;
            operation(&cancel)
        });
        let result = tokio::select! {
            biased;
            _ = &mut cancelled => { guard.0.cancel(); job.await },
            result = &mut job => result,
        };
        result
            .map_err(|_| ErrorInfo {
                code: "worker_failed",
                message: "document worker did not complete".to_owned(),
            })?
            .map_err(|error| error.info())
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandExecutor, Workers};
    use bondline_commands::{Command, Engine};
    use serde_json::json;
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    use tokio::{
        sync::{Semaphore, oneshot},
        time::timeout,
    };

    #[tokio::test(flavor = "current_thread")]
    async fn already_cancelled_work_never_starts_and_executor_clones_share_the_budget()
    -> Result<(), Box<dyn std::error::Error>> {
        use std::sync::atomic::{AtomicBool, Ordering};
        let workers = Workers::default();
        let ran = Arc::new(AtomicBool::new(false));
        let observed = Arc::clone(&ran);
        assert_eq!(
            workers
                .run(
                    move |_| {
                        observed.store(true, Ordering::SeqCst);
                        Ok(Engine::default().state())
                    },
                    std::future::ready(())
                )
                .await
                .err()
                .ok_or("ignored pre-cancel")?
                .code,
            "cancelled"
        );
        assert!(!ran.load(Ordering::SeqCst));
        assert_eq!(workers.0.available_permits(), 2);
        let engine = Arc::new(Mutex::new(Engine::default()));
        let executor = CommandExecutor { engine, workers };
        let first = executor.workers.permit().map_err(|error| error.message)?;
        let clone = executor.clone();
        let second = clone.workers.permit().map_err(|error| error.message)?;
        let command = Command::parse(
            "render.capture",
            json!({"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}),
        )?;
        assert_eq!(
            clone
                .execute_blocking(command, &bondline_commands::Cancellation::default())
                .err()
                .ok_or("UI escaped shared budget")?
                .code,
            "busy"
        );
        drop((first, second));
        assert_eq!(executor.workers.0.available_permits(), 2);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn capture_budget_rejects_work_without_blocking_authoring()
    -> Result<(), Box<dyn std::error::Error>> {
        let workers = Workers::default();
        assert_eq!(workers.0.available_permits(), 2);
        let first = Arc::clone(&workers.0).try_acquire_owned()?;
        let second = Arc::clone(&workers.0).try_acquire_owned()?;
        let engine = Arc::new(Mutex::new(Engine::default()));
        let executor = CommandExecutor {
            engine: Arc::clone(&engine),
            workers: workers.clone(),
        };
        let capture = engine
            .lock()
            .map_err(|_| "engine")?
            .prepare(Command::parse(
                "render.capture",
                json!({"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}),
            )?)?;
        assert_eq!(
            executor
                .complete(capture, std::future::pending())
                .await
                .err()
                .ok_or("accepted excess capture")?
                .code,
            "busy"
        );
        let edit = engine
            .lock()
            .map_err(|_| "engine")?
            .prepare(Command::parse(
                "page.set_size",
                json!({"page_id":1,"width_pt":612,"height_pt":792}),
            )?)?;
        assert!(
            executor
                .complete(edit, std::future::ready(()))
                .await
                .map_err(|error| error.message)?
                .changed
        );
        assert_eq!(engine.lock().map_err(|_| "engine")?.state().revision, 1);
        drop((first, second));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn explicit_cancel_and_dropped_future_cancel_workers_without_releasing_their_budget_early()
    -> Result<(), Box<dyn std::error::Error>> {
        for abort in [false, true] {
            let workers = Workers(Arc::new(Semaphore::new(1)));
            let (started, start) = oneshot::channel();
            let (observed, cancelled) = oneshot::channel();
            let (release, wait) = std::sync::mpsc::channel();
            let (stop, stopped) = oneshot::channel();
            let runner = workers.clone();
            let task = tokio::spawn(async move {
                runner
                    .run(
                        move |cancel| {
                            let _ = started.send(());
                            while !cancel.is_cancelled() {
                                std::thread::sleep(Duration::from_millis(1));
                            }
                            let _ = observed.send(());
                            // Keep the worker alive after its awaiting task has gone away.
                            let _ = wait.recv_timeout(Duration::from_secs(1));
                            cancel.check()?;
                            Ok(Engine::default().state())
                        },
                        async {
                            let _ = stopped.await;
                        },
                    )
                    .await
            });
            timeout(Duration::from_secs(1), start).await??;
            if abort {
                task.abort();
            } else {
                let _ = stop.send(());
            }
            timeout(Duration::from_secs(1), cancelled).await??;
            assert_eq!(
                workers.0.available_permits(),
                0,
                "cancelled worker retains its budget"
            );
            release.send(())?;
            if abort {
                assert!(task.await.is_err_and(|error| error.is_cancelled()));
            } else {
                assert_eq!(
                    task.await?.err().ok_or("ignored cancellation")?.code,
                    "cancelled"
                );
            }
            timeout(Duration::from_secs(1), async {
                while workers.0.available_permits() == 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await?;
        }
        Ok(())
    }
}
