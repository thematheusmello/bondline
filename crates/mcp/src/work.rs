use crate::SharedEngine;
use bondline_commands::{Cancellation, Command, CommandResult, Error, ErrorInfo, Prepared};
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::Semaphore;

#[derive(Clone)]
pub(crate) struct Workers {
    slots: Arc<Semaphore>,
    active: Arc<AtomicUsize>,
}

pub(crate) struct ActivityGuard(Arc<AtomicUsize>);

impl Drop for ActivityGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

struct WorkPermit {
    _slot: tokio::sync::OwnedSemaphorePermit,
    _activity: ActivityGuard,
}

pub(crate) struct RegisteredWork {
    prepared: Prepared,
    permit: Option<WorkPermit>,
}

pub(crate) struct Rejected {
    pub info: ErrorInfo,
    pub disconnect: bool,
}

impl Rejected {
    fn command(error: Error) -> Self {
        Self {
            disconnect: matches!(error, Error::InvalidParams(_)),
            info: error.info(),
        }
    }
}

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

    /// Nonblocking, including while the caller holds the Engine mutex. A native close
    /// acknowledgement must check this under that mutex before committing shutdown.
    pub fn has_active_work(&self) -> bool {
        self.workers.active.load(Ordering::Acquire) != 0
    }

    fn prepare_with<T>(
        &self,
        operation: impl FnOnce(&mut bondline_commands::Engine) -> Result<Prepared, Error>,
        publication: impl FnOnce(&Workers) -> T,
    ) -> Result<(RegisteredWork, T), Rejected> {
        let mut engine = self.engine.lock().map_err(|_| Rejected {
            info: unavailable(),
            disconnect: true,
        })?;
        // Register replies before a short close command can approve shutdown. Deferred
        // plans also reserve their worker before this same engine lock is released.
        let publication = publication(&self.workers);
        let prepared = operation(&mut engine).map_err(Rejected::command)?;
        let permit = if prepared.is_deferred() {
            Some(self.workers.permit().map_err(|info| Rejected {
                info,
                disconnect: false,
            })?)
        } else {
            None
        };
        Ok((RegisteredWork { prepared, permit }, publication))
    }

    pub(crate) fn prepare_command(&self, command: Command) -> Result<RegisteredWork, ErrorInfo> {
        self.prepare_with(|engine| engine.prepare(command), |_| ())
            .map(|(registered, ())| registered)
            .map_err(|error| error.info)
    }

    pub(crate) fn prepare_json(
        &self,
        request: &[u8],
    ) -> Result<(RegisteredWork, ActivityGuard), Rejected> {
        self.prepare_with(|engine| engine.prepare_json(request), Workers::activity)
    }

    /// Call from a background thread. Engine guards cover only preparation and application.
    pub fn execute_blocking(
        &self,
        command: Command,
        cancel: &Cancellation,
    ) -> Result<CommandResult, ErrorInfo> {
        let RegisteredWork {
            prepared,
            permit: _permit,
        } = self.prepare_command(command)?;
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
        let prepared = self.prepare_command(command)?;
        self.complete(prepared, cancelled).await
    }

    pub(crate) async fn complete(
        &self,
        registered: RegisteredWork,
        cancelled: impl Future<Output = ()>,
    ) -> Result<CommandResult, ErrorInfo> {
        let RegisteredWork { prepared, permit } = registered;
        let engine = Arc::clone(&self.engine);
        let operation = move |cancel: &Cancellation| {
            let completed = prepared.complete_with_cancel(cancel)?;
            engine
                .lock()
                .map_err(|_| Error::EngineUnavailable)?
                .finish(completed)
        };
        if let Some(permit) = permit {
            self.workers.run(permit, operation, cancelled).await
        } else {
            // Short commands already committed in prepare; cancellation must not replay
            // them or report that a completed authoring edit was cancelled.
            operation(&Cancellation::default()).map_err(|error| error.info())
        }
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
        Self {
            slots: Arc::new(Semaphore::new(2)),
            active: Arc::new(AtomicUsize::new(0)),
        }
    }
}

struct CancelOnDrop(Cancellation);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

impl Workers {
    fn activity(&self) -> ActivityGuard {
        self.active.fetch_add(1, Ordering::AcqRel);
        ActivityGuard(Arc::clone(&self.active))
    }

    fn permit(&self) -> Result<WorkPermit, ErrorInfo> {
        let slot = Arc::clone(&self.slots)
            .try_acquire_owned()
            .map_err(|_| ErrorInfo {
                code: "busy",
                message: "two document workers are already running; retry later".to_owned(),
            })?;
        Ok(WorkPermit {
            _slot: slot,
            _activity: self.activity(),
        })
    }

    async fn run(
        &self,
        permit: WorkPermit,
        operation: impl FnOnce(&Cancellation) -> Result<CommandResult, Error> + Send + 'static,
        cancelled: impl Future<Output = ()>,
    ) -> Result<CommandResult, ErrorInfo> {
        tokio::pin!(cancelled);
        tokio::select! {
            biased;
            _ = &mut cancelled => return Err(ErrorInfo {code:"cancelled",message:"operation cancelled before work started".to_owned()}),
            _ = std::future::ready(()) => {},
        }
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

    #[test]
    fn registered_deferred_work_is_visible_under_the_engine_lock_before_spawning()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Arc::new(Mutex::new(Engine::default()));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let clone = executor.clone();
        assert!(!executor.has_active_work());
        let capture = || {
            Command::parse(
                "render.capture",
                json!({
                    "page_id":1,"width_px":8,"height_px":8,"backend":"cpu"
                }),
            )
        };
        let first = executor
            .prepare_command(capture()?)
            .map_err(|error| error.message)?;
        {
            let _engine = engine.lock().map_err(|_| "engine")?;
            assert!(executor.has_active_work());
            assert!(clone.has_active_work());
        }
        let second = clone
            .prepare_command(capture()?)
            .map_err(|error| error.message)?;
        assert_eq!(
            executor
                .prepare_command(capture()?)
                .err()
                .ok_or("excess prepared work")?
                .code,
            "busy"
        );
        drop(first);
        assert!(clone.has_active_work());
        drop(second);
        assert!(!executor.has_active_work());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn committed_close_rejects_new_blocking_async_and_json_work_without_reserving_workers()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Arc::new(Mutex::new(Engine::default()));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let close = executor
            .execute_blocking(
                Command::parse("document.close", json!({"expected_revision":0}))?,
                &bondline_commands::Cancellation::default(),
            )
            .map_err(|error| error.message)?;
        assert_eq!(close.close, bondline_commands::CloseState::Approved);
        {
            let mut engine = engine.lock().map_err(|_| "engine")?;
            assert!(!executor.has_active_work());
            assert!(engine.complete_close());
        }
        for (name, params) in [
            (
                "render.capture",
                json!({"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}),
            ),
            ("document.open_cdxml", json!({"path":"must-not-open.cdxml"})),
            (
                "document.save_cdxml",
                json!({"path":"must-not-write.cdxml"}),
            ),
        ] {
            assert_eq!(
                executor
                    .execute_blocking(
                        Command::parse(name, params.clone())?,
                        &bondline_commands::Cancellation::default()
                    )
                    .err()
                    .ok_or("blocking work after close")?
                    .code,
                "close_committed"
            );
            assert_eq!(
                executor
                    .execute(
                        Command::parse(name, params.clone())?,
                        std::future::pending()
                    )
                    .await
                    .err()
                    .ok_or("async work after close")?
                    .code,
                "close_committed"
            );
            let json = serde_json::to_vec(&json!({"command":name,"params":params}))?;
            assert_eq!(
                executor
                    .prepare_json(&json)
                    .err()
                    .ok_or("JSON work after close")?
                    .info
                    .code,
                "close_committed"
            );
            assert!(!executor.has_active_work());
        }
        let inspected = executor
            .execute(
                Command::parse("document.inspect", json!({}))?,
                std::future::pending(),
            )
            .await
            .map_err(|error| error.message)?;
        assert_eq!(inspected.close, bondline_commands::CloseState::Committed);
        assert_eq!(inspected.revision, 0);
        assert_eq!(inspected.undo_depth, 0);
        assert!(!executor.has_active_work());
        Ok(())
    }

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
                    workers.permit().map_err(|error| error.message)?,
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
        assert_eq!(workers.slots.available_permits(), 2);
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
        assert_eq!(executor.workers.slots.available_permits(), 2);
        assert!(!executor.has_active_work());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn capture_budget_rejects_work_without_blocking_authoring()
    -> Result<(), Box<dyn std::error::Error>> {
        let workers = Workers::default();
        assert_eq!(workers.slots.available_permits(), 2);
        let first = workers.permit().map_err(|error| error.message)?;
        let second = workers.permit().map_err(|error| error.message)?;
        let engine = Arc::new(Mutex::new(Engine::default()));
        let executor = CommandExecutor {
            engine: Arc::clone(&engine),
            workers: workers.clone(),
        };
        assert_eq!(
            executor
                .prepare_command(Command::parse(
                    "render.capture",
                    json!({"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}),
                )?)
                .err()
                .ok_or("accepted excess capture")?
                .code,
            "busy"
        );
        let edit = executor
            .prepare_command(Command::parse(
                "page.set_size",
                json!({"page_id":1,"width_pt":612,"height_pt":792}),
            )?)
            .map_err(|error| error.message)?;
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
            let workers = Workers {
                slots: Arc::new(Semaphore::new(1)),
                ..Workers::default()
            };
            let (started, start) = oneshot::channel();
            let (observed, cancelled) = oneshot::channel();
            let (release, wait) = std::sync::mpsc::channel();
            let (stop, stopped) = oneshot::channel();
            let runner = workers.clone();
            let permit = runner.permit().map_err(|error| error.message)?;
            let task = tokio::spawn(async move {
                runner
                    .run(
                        permit,
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
                workers.slots.available_permits(),
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
                while workers.active.load(std::sync::atomic::Ordering::Acquire) != 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await?;
        }
        Ok(())
    }
}
