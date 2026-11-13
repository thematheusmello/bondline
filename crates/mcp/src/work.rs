use bondline_commands::{Cancellation, CommandResult, Error, ErrorInfo, Prepared};
use std::{future::Future, sync::Arc};
use tokio::sync::Semaphore;

#[derive(Clone)]
pub(crate) struct Workers(Arc<Semaphore>);

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
    pub(crate) async fn complete(
        &self,
        prepared: Prepared,
        cancelled: impl Future<Output = ()>,
    ) -> Result<CommandResult, ErrorInfo> {
        if !prepared.is_deferred() {
            return prepared.complete().map_err(|error| error.info());
        }
        self.run(
            move |cancel| prepared.complete_with_cancel(cancel),
            cancelled,
        )
        .await
    }

    async fn run(
        &self,
        operation: impl FnOnce(&Cancellation) -> Result<CommandResult, Error> + Send + 'static,
        cancelled: impl Future<Output = ()>,
    ) -> Result<CommandResult, ErrorInfo> {
        let permit = Arc::clone(&self.0)
            .try_acquire_owned()
            .map_err(|_| ErrorInfo {
                code: "busy",
                message: "two capture workers are already running; retry later".to_owned(),
            })?;
        let guard = CancelOnDrop(Cancellation::default());
        let cancel = guard.0.clone();
        let job = tokio::task::spawn_blocking(move || {
            // A dropped awaiting task must not free a still-running worker's budget.
            let _permit = permit;
            operation(&cancel)
        });
        tokio::select! {
            biased;
            _ = cancelled => Err(ErrorInfo { code: "cancelled", message: "capture cancelled".to_owned() }),
            result = job => result.map_err(|_| ErrorInfo {
                code: "worker_failed", message: "capture worker did not complete".to_owned(),
            })?.map_err(|error| error.info()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Workers;
    use bondline_commands::{Command, Engine};
    use serde_json::json;
    use std::{sync::Arc, time::Duration};
    use tokio::{
        sync::{Semaphore, oneshot},
        time::timeout,
    };

    #[tokio::test(flavor = "current_thread")]
    async fn capture_budget_rejects_work_without_blocking_authoring()
    -> Result<(), Box<dyn std::error::Error>> {
        let workers = Workers::default();
        assert_eq!(workers.0.available_permits(), 2);
        let first = Arc::clone(&workers.0).try_acquire_owned()?;
        let second = Arc::clone(&workers.0).try_acquire_owned()?;
        let mut engine = Engine::default();
        let capture = engine.prepare(Command::parse(
            "render.capture",
            json!({"page_id":1,"width_px":8,"height_px":8,"backend":"cpu"}),
        )?)?;
        assert_eq!(
            workers
                .complete(capture, std::future::pending())
                .await
                .err()
                .ok_or("accepted excess capture")?
                .code,
            "busy"
        );
        let edit = engine.prepare(Command::parse(
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        )?)?;
        assert!(
            workers
                .complete(edit, std::future::ready(()))
                .await
                .map_err(|error| error.message)?
                .changed
        );
        assert_eq!(engine.state().revision, 1);
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
                assert!(task.await.is_err_and(|error| error.is_cancelled()));
            } else {
                let _ = stop.send(());
                assert_eq!(
                    task.await?.err().ok_or("ignored cancellation")?.code,
                    "cancelled"
                );
            }
            timeout(Duration::from_secs(1), cancelled).await??;
            assert_eq!(
                workers.0.available_permits(),
                0,
                "worker still owns its resource budget"
            );
            release.send(())?;
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
