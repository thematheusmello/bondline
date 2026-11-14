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

/// One native file selection, validated and reserved before its worker is spawned.
/// The originating engine is private and cannot be replaced by a different session.
#[must_use = "complete the file work, or report its failed start to finish_ui_file"]
pub struct UiFileWork {
    engine: SharedEngine,
    registered: RegisteredWork,
}

impl UiFileWork {
    /// Complete the real file command, not presentation acknowledgement. The native
    /// host consumes the result with finish_ui_file; a retry must retain the same ID.
    pub fn complete_blocking(self, cancel: &Cancellation) -> Result<CommandResult, ErrorInfo> {
        let Self { engine, registered } = self;
        complete_registered_blocking(&engine, registered, cancel)
    }
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

    /// Called before the native thread is spawned. File work is known to be deferred;
    /// reserve first so a full worker budget cannot consume the accepted request.
    pub fn prepare_ui_file(&self, id: u64, actual: Command) -> Result<UiFileWork, ErrorInfo> {
        let mut engine = self.engine.lock().map_err(|_| unavailable())?;
        let permit = self.workers.permit()?;
        let prepared = engine
            .prepare_ui_file(id, actual)
            .map_err(|error| error.info())?;
        Ok(UiFileWork {
            engine: Arc::clone(&self.engine),
            registered: RegisteredWork {
                prepared,
                permit: Some(permit),
            },
        })
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
        complete_registered_blocking(&self.engine, self.prepare_command(command)?, cancel)
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

fn complete_registered_blocking(
    engine: &SharedEngine,
    registered: RegisteredWork,
    cancel: &Cancellation,
) -> Result<CommandResult, ErrorInfo> {
    let RegisteredWork {
        prepared,
        permit: _permit,
    } = registered;
    let completed = prepared
        .complete_with_cancel(cancel)
        .map_err(|error| error.info())?;
    engine
        .lock()
        .map_err(|_| unavailable())?
        .finish(completed)
        .map_err(|error| error.info())
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
    fn separate_executor_budgets_cannot_bypass_the_engines_outstanding_save()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Arc::new(Mutex::new(Engine::default()));
        let first = CommandExecutor::new(Arc::clone(&engine));
        let second = CommandExecutor::new(Arc::clone(&engine));
        let mut nonce = [0_u8; 16];
        getrandom::fill(&mut nonce).map_err(|error| format!("test path nonce: {error}"))?;
        let path = std::env::temp_dir().join(format!(
            "bondline-unpublished-save-reservation-{:032x}.cdxml",
            u128::from_le_bytes(nonce)
        ));
        let command = |name| Command::parse(name, json!({"path":path}));
        let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        let pending = first
            .prepare_command(command("document.save_cdxml")?)
            .map_err(|error| error.message)?;
        assert!(first.has_active_work());
        assert!(!second.has_active_work());
        assert_eq!(
            second
                .prepare_command(command("document.save_mol")?)
                .err()
                .ok_or("independent executor bypassed the Engine save reservation")?
                .code,
            "busy"
        );
        assert!(!second.has_active_work());
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        let capture = second
            .prepare_command(Command::parse(
                "render.capture",
                json!({"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}),
            )?)
            .map_err(|error| error.message)?;
        drop(capture);
        drop(pending);
        assert!(!first.has_active_work());
        assert!(!second.has_active_work());
        drop(
            second
                .prepare_command(command("document.save_mol")?)
                .map_err(|error| error.message)?,
        );
        assert!(!path.try_exists()?);
        assert!(!second.has_active_work());
        Ok(())
    }

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

    fn accepted_open(engine: &mut Engine) -> Result<u64, Box<dyn std::error::Error>> {
        engine.set_ui_file_host(true);
        let revision = engine.state().revision;
        let requested = engine.dispatch(
            "ui.file",
            json!({"expected_revision":revision,"action":"open"}),
        )?;
        let id = requested.ui_file.request.as_ref().ok_or("UI request")?.id;
        engine.claim_ui_file(id)?;
        Ok(id)
    }

    #[test]
    fn ui_file_reservation_precedes_consumption_and_survives_the_native_spawn_gap()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Arc::new(Mutex::new(Engine::default()));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let id = accepted_open(&mut *engine.lock().map_err(|_| "engine")?)?;
        let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        let actual = || {
            Command::parse(
                "document.open_cdxml",
                json!({"path":"not-read-before-start.cdxml"}),
            )
        };
        let first = executor.workers.permit().map_err(|error| error.message)?;
        let second = executor.workers.permit().map_err(|error| error.message)?;
        assert_eq!(
            executor
                .prepare_ui_file(id, actual()?)
                .err()
                .ok_or("accepted excess UI file work")?
                .code,
            "busy"
        );
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before,
            "failed worker reservation must not consume the accepted presentation"
        );
        drop(second);
        let work = executor
            .prepare_ui_file(id, actual()?)
            .map_err(|error| error.message)?;
        {
            let state = engine.lock().map_err(|_| "engine")?.state();
            assert_eq!(
                state
                    .ui_file
                    .request
                    .as_ref()
                    .ok_or("running request")?
                    .phase,
                bondline_commands::UiFilePhase::Running
            );
            assert!(executor.has_active_work());
            assert!(executor.clone().has_active_work());
            assert_eq!(state.revision, 0);
            assert_eq!(state.undo_depth, 0);
        }
        {
            let mut engine = engine.lock().map_err(|_| "engine")?;
            let before = serde_json::to_value(engine.state())?;
            assert_eq!(
                engine
                    .dispatch(
                        "ui.file",
                        json!({
                            "expected_revision":0,"action":"cancel","request_id":id
                        })
                    )
                    .err()
                    .ok_or("presentation Cancel claimed to cancel reserved file I/O")?
                    .info()
                    .code,
                "ui_busy"
            );
            assert_eq!(serde_json::to_value(engine.state())?, before);
        }
        // A failed std::thread::spawn drops the closure/opaque work before UI reports retry.
        drop(work);
        assert!(
            engine
                .lock()
                .map_err(|_| "engine")?
                .finish_ui_file(id, true)
        );
        assert_eq!(
            engine
                .lock()
                .map_err(|_| "engine")?
                .state()
                .ui_file
                .request
                .as_ref()
                .ok_or("restored request")?
                .phase,
            bondline_commands::UiFilePhase::Accepted
        );
        drop(first);
        assert!(!executor.has_active_work());
        Ok(())
    }

    #[test]
    fn cancelled_replaced_or_edited_ui_requests_never_prepare_a_file_operation()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Arc::new(Mutex::new(Engine::default()));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let first = accepted_open(&mut *engine.lock().map_err(|_| "engine")?)?;
        let second = {
            let mut engine = engine.lock().map_err(|_| "engine")?;
            engine.dispatch(
                "ui.file",
                json!({"expected_revision":0,"action":"cancel","request_id":first}),
            )?;
            let next =
                engine.dispatch("ui.file", json!({"expected_revision":0,"action":"open"}))?;
            let id = next
                .ui_file
                .request
                .as_ref()
                .ok_or("replacement request")?
                .id;
            engine.claim_ui_file(id)?;
            id
        };
        assert!(second > first);
        let actual =
            || Command::parse("document.open_cdxml", json!({"path":"must-not-open.cdxml"}));
        let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        assert_eq!(
            executor
                .prepare_ui_file(first, actual()?)
                .err()
                .ok_or("replayed cancelled request")?
                .code,
            "ui_stale_request"
        );
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        assert!(!executor.has_active_work());
        {
            engine.lock().map_err(|_| "engine")?.dispatch(
                "page.set_size",
                json!({"page_id":1,"width_pt":612,"height_pt":792}),
            )?;
        }
        let edited = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        assert_eq!(
            executor
                .prepare_ui_file(second, actual()?)
                .err()
                .ok_or("prepared obsolete dialog selection")?
                .code,
            "ui_stale_request"
        );
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            edited
        );
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
    async fn sdf_queries_share_workers_without_prepare_side_effects_or_save_reservations()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut session = Engine::default();
        session.dispatch("document.open_sdf",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/sdf/ordered-fields-v2000.sdf")}))?;
        session.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":"request"}),
        )?;
        let engine = Arc::new(Mutex::new(session));
        let workers = Workers::default();
        let executor = CommandExecutor {
            engine: Arc::clone(&engine),
            workers: workers.clone(),
        };
        let query = |record_id| {
            Command::parse(
                "sdf.record.inspect",
                json!({"expected_revision":1,"record_id":record_id}),
            )
        };
        let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        let held1 = workers.permit().map_err(|e| e.message)?;
        let held2 = workers.permit().map_err(|e| e.message)?;
        assert_eq!(
            executor
                .prepare_command(query(1)?)
                .err()
                .ok_or("busy query")?
                .code,
            "busy"
        );
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        drop((held1, held2));
        let first = executor.prepare_command(query(1)?).map_err(|e| e.message)?;
        let second = executor.prepare_command(query(2)?).map_err(|e| e.message)?;
        assert_eq!(workers.slots.available_permits(), 0);
        assert_eq!(
            executor
                .prepare_command(query(1)?)
                .err()
                .ok_or("third query")?
                .code,
            "busy"
        );
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        // Queries reserve only workers, never the Engine's independent one-save lease.
        drop(
            engine
                .lock()
                .map_err(|_| "engine")?
                .prepare(Command::parse(
                    "document.save_sdf",
                    json!({"path":"not-published-by-this-query.sdf"}),
                )?)?,
        );
        assert!(executor.has_active_work());
        let result = executor
            .complete(second, std::future::pending())
            .await
            .map_err(|e| e.message)?;
        assert_eq!(serde_json::to_value(result)?["sdf_record"]["record_id"], 2);
        assert!(executor.has_active_work());
        let result = executor
            .complete(first, std::future::pending())
            .await
            .map_err(|e| e.message)?;
        assert_eq!(serde_json::to_value(result)?["sdf_record"]["record_id"], 1);
        assert!(!executor.has_active_work());
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        assert!(engine.lock().map_err(|_| "engine")?.complete_close());
        assert_eq!(
            executor
                .prepare_command(query(1)?)
                .err()
                .ok_or("committed close")?
                .code,
            "close_committed"
        );
        assert!(!executor.has_active_work());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn sdf_field_candidates_are_deferred_and_busy_or_reverse_completion_never_mutates_early()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut session = Engine::default();
        session.dispatch("document.open_sdf",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/sdf/ordered-fields-v2000.sdf")}))?;
        let engine = Arc::new(Mutex::new(session));
        let workers = Workers::default();
        let executor = CommandExecutor {
            engine: Arc::clone(&engine),
            workers: workers.clone(),
        };
        let setter = |text: &str| {
            Command::parse(
                "sdf.field.set_value",
                json!({"expected_revision":1,"record_id":1,"field_index":5,"value_lines":[text]}),
            )
        };
        let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        let held1 = workers.permit().map_err(|e| e.message)?;
        let held2 = workers.permit().map_err(|e| e.message)?;
        assert_eq!(
            executor
                .prepare_command(setter("busy")?)
                .err()
                .ok_or("busy candidate")?
                .code,
            "busy"
        );
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        drop((held1, held2));
        let first = executor
            .prepare_command(setter("first")?)
            .map_err(|e| e.message)?;
        let second = executor
            .prepare_command(setter("second")?)
            .map_err(|e| e.message)?;
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        assert_eq!(
            executor
                .prepare_command(setter("third")?)
                .err()
                .ok_or("third candidate")?
                .code,
            "busy"
        );
        let result = executor
            .complete(second, std::future::pending())
            .await
            .map_err(|e| e.message)?;
        assert!(result.changed);
        assert_eq!(result.revision, 2);
        assert_eq!(result.undo_depth, 1);
        let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        assert_eq!(
            executor
                .complete(first, std::future::pending())
                .await
                .err()
                .ok_or("reverse candidate")?
                .code,
            "stale_revision"
        );
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        assert!(!executor.has_active_work());
        assert_eq!(workers.slots.available_permits(), 2);
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
