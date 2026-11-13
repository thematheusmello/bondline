//! Opt-in, bounded newline JSON control on IPv4 loopback only.

use crate::work::ActivityGuard;
use crate::{CommandExecutor, SharedEngine};
use bondline_commands::{CommandResult, Error as CommandError, ErrorInfo, MAX_REQUEST_BYTES};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    fmt::Write as _,
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    thread::JoinHandle,
    time::Duration,
};
use subtle::ConstantTimeEq;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::oneshot,
    task::JoinSet,
    time::timeout,
};

const MAX_CLIENTS: usize = 16;
const IO_DEADLINE: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Authentication {
    token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("loopback worker panicked")]
    WorkerPanicked,
}

/// Owns the listener, connection tasks and worker. Drop closes all of them.
pub struct LoopbackServer {
    address: SocketAddr,
    token: Arc<str>,
    stop: Option<oneshot::Sender<()>>,
    worker: Option<JoinHandle<io::Result<()>>>,
}

impl LoopbackServer {
    pub fn bind(
        port: u16,
        engine: SharedEngine,
        on_change: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        Self::bind_with_executor(port, CommandExecutor::new(engine), on_change)
    }

    pub fn bind_with_executor(
        port: u16,
        executor: CommandExecutor,
        on_change: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        let mut random = [0_u8; 32];
        getrandom::fill(&mut random).map_err(io::Error::other)?;
        let mut token = String::with_capacity(64);
        for byte in random {
            write!(&mut token, "{byte:02x}").map_err(io::Error::other)?;
        }
        let token: Arc<str> = token.into();
        let session_token = Arc::clone(&token);
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
        let address = listener.local_addr()?;
        listener.set_nonblocking(true)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let (stop, shutdown) = oneshot::channel();
        let worker = std::thread::Builder::new()
            .name("bondline-control".to_owned())
            .spawn(move || {
                runtime.block_on(async move {
                    let listener = TcpListener::from_std(listener)?;
                    serve(
                        listener,
                        executor,
                        Arc::new(on_change),
                        session_token,
                        shutdown,
                    )
                    .await
                })
            })?;
        Ok(Self {
            address,
            token,
            stop: Some(stop),
            worker: Some(worker),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Secret for the owner's trusted startup channel; never include it in UI or errors.
    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn is_running(&self) -> bool {
        self.worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
    }

    pub fn shutdown(mut self) -> Result<(), Error> {
        self.stop_worker()
    }

    fn stop_worker(&mut self) -> Result<(), Error> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| Error::WorkerPanicked)??;
        }
        Ok(())
    }
}

impl Drop for LoopbackServer {
    fn drop(&mut self) {
        let _ = self.stop_worker();
    }
}

async fn serve(
    listener: TcpListener,
    executor: CommandExecutor,
    on_change: Arc<dyn Fn() + Send + Sync>,
    token: Arc<str>,
    mut stop: oneshot::Receiver<()>,
) -> io::Result<()> {
    let mut clients = JoinSet::new();
    let result = loop {
        tokio::select! {
            biased;
            _ = &mut stop => break Ok(()),
            _ = clients.join_next(), if !clients.is_empty() => {},
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    if clients.len() < MAX_CLIENTS {
                        clients.spawn(connection(stream, executor.clone(), Arc::clone(&on_change), Arc::clone(&token)));
                    }
                }
                Err(error) if matches!(error.kind(), io::ErrorKind::Interrupted | io::ErrorKind::ConnectionAborted) => {},
                Err(error) => break Err(error),
            }
        }
    };
    clients.abort_all();
    while clients.join_next().await.is_some() {}
    result
}

async fn send(writer: &mut (impl AsyncWrite + Unpin), value: Value) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(&value).map_err(io::Error::other)?;
    bytes.push(b'\n');
    timeout(IO_DEADLINE, async {
        writer.write_all(&bytes).await?;
        writer.flush().await
    })
    .await
    .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?
}

async fn publish_result(
    writer: &mut (impl AsyncWrite + Unpin),
    result: Result<CommandResult, ErrorInfo>,
    publication: ActivityGuard,
    on_change: &(dyn Fn() + Send + Sync),
) -> io::Result<()> {
    let notify = result.as_ref().is_ok_and(|result| {
        result.changed
            || result.file_operation.is_some()
            || result.close_operation.is_some()
            || result.ui_file_operation.is_some()
    });
    let value = match result {
        Ok(result) => json!({"result":result}),
        Err(error) => json!({"error":error}),
    };
    let sent = send(writer, value).await;
    drop(publication);
    // Release the publication barrier before waking native finalization, including
    // failed writes: committed engine changes are not rolled back by a lost peer.
    if notify {
        on_change();
    }
    sent
}

async fn connection(
    stream: TcpStream,
    executor: CommandExecutor,
    on_change: Arc<dyn Fn() + Send + Sync>,
    token: Arc<str>,
) -> io::Result<()> {
    let (read, mut write) = stream.into_split();
    let mut reader = BufReader::new(read);
    let mut authenticated = false;
    loop {
        let mut bytes = Vec::new();
        let count = timeout(
            IO_DEADLINE,
            (&mut reader)
                .take(MAX_REQUEST_BYTES as u64 + 1)
                .read_until(b'\n', &mut bytes),
        )
        .await
        .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))??;
        if count == 0 {
            return Ok(());
        }
        let frame_error = if bytes.last() != Some(&b'\n') {
            Some(if bytes.len() > MAX_REQUEST_BYTES {
                CommandError::RequestTooLarge.info()
            } else {
                ErrorInfo {
                    code: "invalid_frame",
                    message: "command frame must end with a newline".to_owned(),
                }
            })
        } else {
            None
        };
        if let Some(error) = frame_error {
            send(&mut write, json!({"error":error})).await?;
            return Ok(());
        }
        bytes.pop();
        if !authenticated {
            authenticated = serde_json::from_slice::<Authentication>(&bytes)
                .is_ok_and(|request| bool::from(request.token.as_bytes().ct_eq(token.as_bytes())));
            if !authenticated {
                send(&mut write, json!({"error":{"code":"unauthorized","message":"valid session token required"}})).await?;
                return Ok(());
            }
            send(&mut write, json!({"authenticated":true})).await?;
            continue;
        }
        match executor.prepare_json(&bytes) {
            Ok((prepared, publication)) => {
                let result = executor
                    .complete(prepared, tokio::time::sleep(IO_DEADLINE))
                    .await;
                publish_result(&mut write, result, publication, on_change.as_ref()).await?;
            }
            Err(error) => {
                send(&mut write, json!({"error":error.info})).await?;
                // A rejected first line must not turn an HTTP body into subsequent commands.
                if error.disconnect {
                    return Ok(());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::publish_result;
    use crate::CommandExecutor;
    use bondline_commands::{CloseState, Engine};
    use serde_json::{Value, json};
    use std::{
        future::Future,
        io,
        pin::Pin,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        task::{Context, Poll},
        time::Duration,
    };
    use tokio::{io::AsyncWrite, sync::oneshot, time::timeout};

    struct FlushGate {
        bytes: Arc<Mutex<Vec<u8>>>,
        flushing: Option<oneshot::Sender<()>>,
        release: oneshot::Receiver<()>,
    }

    impl AsyncWrite for FlushGate {
        fn poll_write(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            bytes: &[u8],
        ) -> Poll<io::Result<usize>> {
            match self.bytes.lock() {
                Ok(mut output) => {
                    output.extend_from_slice(bytes);
                    Poll::Ready(Ok(bytes.len()))
                }
                Err(_) => Poll::Ready(Err(io::Error::other("test writer lock"))),
            }
        }

        fn poll_flush(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
            let this = self.get_mut();
            if let Some(flushing) = this.flushing.take() {
                let _ = flushing.send(());
            }
            match Pin::new(&mut this.release).poll(context) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
                Poll::Ready(Err(_)) => Poll::Ready(Err(io::Error::other("flush gate closed"))),
            }
        }

        fn poll_shutdown(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.poll_flush(context)
        }
    }

    struct BrokenWriter {
        fail_flush: bool,
    }

    impl AsyncWrite for BrokenWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            bytes: &[u8],
        ) -> Poll<io::Result<usize>> {
            if self.fail_flush {
                Poll::Ready(Ok(bytes.len()))
            } else {
                Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()))
            }
        }
        fn poll_flush(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
            if self.fail_flush {
                Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()))
            } else {
                Poll::Ready(Ok(()))
            }
        }
        fn poll_shutdown(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    fn acknowledge_if_idle(executor: &CommandExecutor) -> Result<bool, Box<dyn std::error::Error>> {
        let mut engine = executor.engine().lock().map_err(|_| "engine")?;
        Ok(!executor.has_active_work() && engine.complete_close())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn approved_close_waits_for_response_flush_before_native_acknowledgement_and_repaint()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Arc::new(Mutex::new(Engine::default()));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let request = serde_json::to_vec(
            &json!({"command":"document.close","params":{"expected_revision":0}}),
        )?;
        let (registered, publication) = executor
            .prepare_json(&request)
            .map_err(|error| error.info.message)?;
        assert_eq!(
            engine.lock().map_err(|_| "engine")?.state().close,
            CloseState::Approved
        );
        assert!(
            executor.has_active_work(),
            "approval must be registered before the preparing lock releases"
        );
        assert!(!acknowledge_if_idle(&executor)?);
        let result = executor.complete(registered, std::future::pending()).await;
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let (flushing, flush_started) = oneshot::channel();
        let (release, released) = oneshot::channel();
        let notified = Arc::new(AtomicBool::new(false));
        let active_at_callback = Arc::new(AtomicBool::new(true));
        let flag = Arc::clone(&notified);
        let active = Arc::clone(&active_at_callback);
        let observed = executor.clone();
        let output = Arc::clone(&bytes);
        let task = tokio::spawn(async move {
            let mut writer = FlushGate {
                bytes: output,
                flushing: Some(flushing),
                release: released,
            };
            publish_result(&mut writer, result, publication, &move || {
                active.store(observed.has_active_work(), Ordering::SeqCst);
                flag.store(true, Ordering::SeqCst);
            })
            .await
        });
        timeout(Duration::from_secs(2), flush_started).await??;
        assert!(executor.has_active_work());
        assert!(!acknowledge_if_idle(&executor)?);
        assert!(
            !notified.load(Ordering::SeqCst),
            "native notification preceded response flush"
        );
        release.send(()).map_err(|_| "release flush")?;
        timeout(Duration::from_secs(2), task).await???;
        assert!(notified.load(Ordering::SeqCst));
        assert!(!active_at_callback.load(Ordering::SeqCst));
        assert!(!executor.has_active_work());
        let response: Value = serde_json::from_slice(&bytes.lock().map_err(|_| "output")?)?;
        assert_eq!(response["result"]["close_operation"], "approved");
        assert_eq!(response["result"]["revision"], 0);
        assert!(acknowledge_if_idle(&executor)?);
        assert_eq!(
            engine.lock().map_err(|_| "engine")?.state().close,
            CloseState::Committed
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn failed_writes_and_flushes_still_notify_committed_authored_changes()
    -> Result<(), Box<dyn std::error::Error>> {
        for fail_flush in [false, true] {
            let engine = Arc::new(Mutex::new(Engine::default()));
            let executor = CommandExecutor::new(Arc::clone(&engine));
            let request = serde_json::to_vec(&json!({"command":"page.set_size","params":{
                "page_id":1,"width_pt":612,"height_pt":792
            }}))?;
            let (registered, publication) = executor
                .prepare_json(&request)
                .map_err(|error| error.info.message)?;
            let result = executor.complete(registered, std::future::pending()).await;
            let notified = AtomicBool::new(false);
            let active_at_callback = AtomicBool::new(true);
            let error = publish_result(
                &mut BrokenWriter { fail_flush },
                result,
                publication,
                &|| {
                    active_at_callback.store(executor.has_active_work(), Ordering::SeqCst);
                    notified.store(true, Ordering::SeqCst);
                },
            )
            .await
            .err()
            .ok_or("accepted broken response writer")?;
            assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
            assert!(notified.load(Ordering::SeqCst));
            assert!(!active_at_callback.load(Ordering::SeqCst));
            assert!(!executor.has_active_work());
            let state = engine.lock().map_err(|_| "engine")?.state();
            assert_eq!(state.revision, 1);
            assert_eq!(state.undo_depth, 1);
            assert!(state.file.dirty);
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn presentation_request_repaints_even_when_response_write_fails_without_claiming_display()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Arc::new(Mutex::new(Engine::default()));
        engine.lock().map_err(|_| "engine")?.set_ui_file_host(true);
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let request = serde_json::to_vec(
            &json!({"command":"ui.file","params":{"expected_revision":0,"action":"open"}}),
        )?;
        let (registered, publication) = executor
            .prepare_json(&request)
            .map_err(|error| error.info.message)?;
        let result = executor
            .complete(registered, std::future::pending())
            .await
            .map_err(|error| error.message)?;
        assert!(!result.changed);
        assert!(result.file_operation.is_none());
        assert!(result.close_operation.is_none());
        assert_eq!(result.ui_file_operation, Some(1));
        assert_eq!(
            result
                .ui_file
                .request
                .as_ref()
                .ok_or("requested presentation")?
                .phase,
            bondline_commands::UiFilePhase::Requested
        );
        let notified = AtomicBool::new(false);
        let sent = publish_result(
            &mut BrokenWriter { fail_flush: false },
            Ok(result),
            publication,
            &|| {
                assert!(!executor.has_active_work());
                notified.store(true, Ordering::SeqCst);
            },
        )
        .await;
        assert!(sent.is_err());
        assert!(notified.load(Ordering::SeqCst));
        let state = engine.lock().map_err(|_| "engine")?.state();
        assert_eq!(
            state
                .ui_file
                .request
                .as_ref()
                .ok_or("retained request")?
                .phase,
            bondline_commands::UiFilePhase::Requested
        );
        assert_eq!(state.revision, 0);
        assert_eq!(state.undo_depth, 0);
        assert!(!state.file.dirty);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interleaved_ui_cancel_and_replacement_do_not_relabel_the_original_request_receipt()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Arc::new(Mutex::new(Engine::default()));
        engine.lock().map_err(|_| "engine")?.set_ui_file_host(true);
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let request = serde_json::to_vec(
            &json!({"command":"ui.file","params":{"expected_revision":0,"action":"open"}}),
        )?;
        let (registered, publication) = executor
            .prepare_json(&request)
            .map_err(|error| error.info.message)?;
        {
            let mut engine = engine.lock().map_err(|_| "engine")?;
            engine.dispatch(
                "ui.file",
                json!({"expected_revision":0,"action":"cancel","request_id":1}),
            )?;
            engine.dispatch("ui.file", json!({"expected_revision":0,"action":"save_as"}))?;
        }
        let result = executor
            .complete(registered, std::future::pending())
            .await
            .map_err(|error| error.message)?;
        assert_eq!(
            result.ui_file_operation,
            Some(1),
            "receipt belongs to the accepted original operation"
        );
        assert_eq!(result.ui_file.epoch, 3);
        let current = result
            .ui_file
            .request
            .as_ref()
            .ok_or("replacement request")?;
        assert_eq!(current.id, 3);
        assert_eq!(current.action, bondline_commands::UiFileAction::SaveAs);
        assert_eq!(current.phase, bondline_commands::UiFilePhase::Requested);
        drop(publication);
        assert!(!executor.has_active_work());
        Ok(())
    }
}
