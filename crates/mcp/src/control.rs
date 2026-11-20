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

fn bind_listener(port: u16) -> io::Result<std::net::TcpListener> {
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    #[cfg(target_os = "macos")]
    {
        use rustix::io::{FdFlags, fcntl_getfd, fcntl_setfd};

        // Darwin sys/fcntl.h defines FD_CLOFORK as 2. CLOEXEC alone retains a
        // listening socket in a concurrent posix_spawn until its exec stage.
        let flags = fcntl_getfd(&listener)?;
        let close_on_fork = FdFlags::from_bits_retain(2);
        fcntl_setfd(&listener, flags | close_on_fork).or_else(allow_unsupported_fd_flag)?;
    }
    Ok(listener)
}

#[cfg(target_os = "macos")]
fn allow_unsupported_fd_flag(error: rustix::io::Errno) -> rustix::io::Result<()> {
    use rustix::io::Errno;
    match error {
        Errno::INVAL | Errno::NOTSUP | Errno::OPNOTSUPP => Ok(()),
        other => Err(other),
    }
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
        let listener = bind_listener(port)?;
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
            || result.view_target_prepared
            || result.analysis_visibility_changed
            || result.file_operation.is_some()
            || result.close_operation.is_some()
            || result.ui_file_operation.is_some()
            || result.ui_atom_label_operation.is_some()
            || result.ui_caption_operation.is_some()
            || result.ui_drawing_settings_operation.is_some()
            || result.ui_preferences_operation.is_some()
            || result.ui_add_chain_operation.is_some()
            || result
                .ui_chain_tool_operation
                .is_some_and(|operation| operation.changed)
            || result
                .ui_selection_operation
                .is_some_and(|operation| operation.changed)
            || result
                .ui_bond_tool_operation
                .is_some_and(|operation| operation.changed)
            || result
                .ui_ring_tool_operation
                .is_some_and(|operation| operation.changed)
            || result
                .ui_view_operation
                .is_some_and(|operation| operation.changed)
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

    #[cfg(target_os = "macos")]
    #[test]
    fn published_listener_preserves_close_on_exec_and_supported_close_on_fork() -> io::Result<()> {
        use rustix::io::{FdFlags, fcntl_getfd, fcntl_setfd};

        // A regular descriptor qualifies the installed kernel independently of the listener.
        let reference = std::fs::File::open("/dev/null")?;
        let close_on_fork = FdFlags::from_bits_retain(2);
        let existing = fcntl_getfd(&reference)?;
        let supported = match fcntl_setfd(&reference, existing | close_on_fork) {
            Ok(()) => fcntl_getfd(&reference)?.contains(close_on_fork),
            Err(error) => {
                super::allow_unsupported_fd_flag(error)?;
                false
            }
        };

        let listener = super::bind_listener(0)?;
        let flags = fcntl_getfd(&listener)?;
        assert!(flags.contains(FdFlags::CLOEXEC));
        assert_eq!(flags.contains(close_on_fork), supported);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn old_kernels_can_decline_the_additional_descriptor_flag() {
        use rustix::io::Errno;
        for unsupported in [Errno::INVAL, Errno::NOTSUP, Errno::OPNOTSUPP] {
            assert_eq!(super::allow_unsupported_fd_flag(unsupported), Ok(()));
        }
        assert_eq!(
            super::allow_unsupported_fd_flag(Errno::BADF),
            Err(Errno::BADF)
        );
        assert_eq!(
            super::allow_unsupported_fd_flag(Errno::ACCESS),
            Err(Errno::ACCESS)
        );
    }

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
    async fn analysis_visibility_repaints_once_per_transition_but_queries_never_wake()
    -> Result<(), Box<dyn std::error::Error>> {
        let engine = Arc::new(Mutex::new(Engine::default()));
        let executor = CommandExecutor::new(engine);
        for (name, params, wake) in [
            (
                "ui.analysis",
                json!({"expected_revision":0,"visible":true}),
                true,
            ),
            (
                "ui.analysis",
                json!({"expected_revision":0,"visible":true}),
                false,
            ),
            (
                "analysis.formula",
                json!({"expected_revision":0,"expected_selection_epoch":0}),
                false,
            ),
            (
                "analysis.properties",
                json!({"expected_revision":0,"expected_selection_epoch":0}),
                false,
            ),
            (
                "ui.analysis",
                json!({"expected_revision":0,"visible":false}),
                true,
            ),
        ] {
            let request = serde_json::to_vec(&json!({"command":name,"params":params}))?;
            let (registered, publication) = executor
                .prepare_json(&request)
                .map_err(|e| e.info.message)?;
            let result = executor.complete(registered, std::future::pending()).await;
            let notified = AtomicBool::new(false);
            publish_result(&mut tokio::io::sink(), result, publication, &|| {
                assert!(!executor.has_active_work());
                notified.store(true, Ordering::SeqCst);
            })
            .await?;
            assert_eq!(notified.load(Ordering::SeqCst), wake, "{name}");
        }
        Ok(())
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
    async fn readonly_sdf_query_reply_keeps_approved_close_blocked_until_flush_without_repaint()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut session = Engine::default();
        session.dispatch("document.open_sdf",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/sdf/ordered-fields-v2000.sdf")}))?;
        session.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":"request"}),
        )?;
        let engine = Arc::new(Mutex::new(session));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        let request = serde_json::to_vec(
            &json!({"command":"sdf.record.inspect","params":{"expected_revision":1,"record_id":1,"field_offset":3,"field_limit":1}}),
        )?;
        let (registered, publication) = executor
            .prepare_json(&request)
            .map_err(|e| e.info.message)?;
        assert!(executor.has_active_work());
        assert!(!acknowledge_if_idle(&executor)?);
        let result = executor.complete(registered, std::future::pending()).await;
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let (flushing, flush_started) = oneshot::channel();
        let (release, released) = oneshot::channel();
        let notified = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&notified);
        let output = Arc::clone(&bytes);
        let task = tokio::spawn(async move {
            let mut writer = FlushGate {
                bytes: output,
                flushing: Some(flushing),
                release: released,
            };
            publish_result(&mut writer, result, publication, &move || {
                flag.store(true, Ordering::SeqCst);
            })
            .await
        });
        timeout(Duration::from_secs(2), flush_started).await??;
        assert!(executor.has_active_work());
        assert!(!acknowledge_if_idle(&executor)?);
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        release.send(()).map_err(|_| "release query flush")?;
        timeout(Duration::from_secs(2), task).await???;
        assert!(!notified.load(Ordering::SeqCst));
        assert!(!executor.has_active_work());
        let response: Value = serde_json::from_slice(&bytes.lock().map_err(|_| "output")?)?;
        assert_eq!(
            response["result"]["sdf_record"]["fields"][0]["dt_number"],
            "0009"
        );
        assert_eq!(response["result"]["sdf_record"]["input_revision"], 1);
        assert_eq!(response["result"]["sdf_record"]["session_generation"], 1);
        assert!(response["result"].get("close_operation").is_none());
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        assert!(acknowledge_if_idle(&executor)?);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn conversion_receipt_reply_holds_close_until_flush_without_repaint_or_session_changes()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = crate::file_test_support::FileTestDirectory::new()?;
        let input = directory.path("reply-conversion.smi");
        let output = directory.path("reply-conversion.cdxml");
        std::fs::write(&input, b"CC\n")?;
        let mut session = Engine::default();
        session.dispatch(
            "document.close",
            json!({"expected_revision":0,"action":"request"}),
        )?;
        let engine = Arc::new(Mutex::new(session));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        let request = serde_json::to_vec(
            &json!({"command":"file.convert_smiles_to_cdxml","params":{
            "expected_revision":0,"input_path":input,"output_path":output,"bond_length_pt":40.0,
            "origin_x_pt":100.0,"origin_y_pt":200.0,"page_width_pt":612.0,"page_height_pt":792.0}}),
        )?;
        let (registered, publication) = executor
            .prepare_json(&request)
            .map_err(|e| e.info.message)?;
        assert!(!acknowledge_if_idle(&executor)?);
        let result = executor.complete(registered, std::future::pending()).await;
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let copy = Arc::clone(&bytes);
        let (flushing, started) = oneshot::channel();
        let (release, released) = oneshot::channel();
        let notified = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&notified);
        let task = tokio::spawn(async move {
            let mut writer = FlushGate {
                bytes: copy,
                flushing: Some(flushing),
                release: released,
            };
            publish_result(&mut writer, result, publication, &move || {
                flag.store(true, Ordering::SeqCst);
            })
            .await
        });
        timeout(Duration::from_secs(2), started).await??;
        assert!(!acknowledge_if_idle(&executor)?);
        assert!(!notified.load(Ordering::SeqCst));
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        release.send(()).map_err(|_| "release conversion flush")?;
        timeout(Duration::from_secs(2), task).await???;
        assert!(!notified.load(Ordering::SeqCst));
        assert!(!executor.has_active_work());
        let mut response: Value = serde_json::from_slice(&bytes.lock().map_err(|_| "output")?)?;
        let receipt = response["result"]
            .as_object_mut()
            .ok_or("result")?
            .remove("conversion")
            .ok_or("receipt")?;
        assert_eq!(response["result"], before);
        assert_eq!(receipt["input_revision"], 0);
        assert_eq!(receipt["session_generation"], 0);
        assert_eq!(receipt["profile"], "neutral_carbon_path_v1");
        crate::file_test_support::independently_read_file(&output, 1)?;
        assert!(acknowledge_if_idle(&executor)?);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn deferred_sdf_edit_reply_keeps_close_blocked_and_repaints_only_after_flush()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut session = Engine::default();
        session.dispatch("document.open_sdf",json!({"path":concat!(env!("CARGO_MANIFEST_DIR"),"/../io/tests/fixtures/sdf/ordered-fields-v2000.sdf")}))?;
        session.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":"request"}),
        )?;
        let engine = Arc::new(Mutex::new(session));
        let executor = CommandExecutor::new(Arc::clone(&engine));
        let before = serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?;
        let request = serde_json::to_vec(
            &json!({"command":"sdf.field.set_value","params":{"expected_revision":1,"record_id":1,"field_index":5,"value_lines":["revised","multiline"]}}),
        )?;
        let (registered, publication) = executor
            .prepare_json(&request)
            .map_err(|e| e.info.message)?;
        assert_eq!(
            serde_json::to_value(engine.lock().map_err(|_| "engine")?.state())?,
            before
        );
        assert!(!acknowledge_if_idle(&executor)?);
        let result = executor.complete(registered, std::future::pending()).await;
        {
            let mut session = engine.lock().map_err(|_| "engine")?;
            assert_eq!(session.state().revision, 2);
            assert!(session.state().file.dirty);
            session.dispatch(
                "document.close",
                json!({"expected_revision":2,"action":"discard"}),
            )?;
        }
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let output = Arc::clone(&bytes);
        let (flushing, flush_started) = oneshot::channel();
        let (release, released) = oneshot::channel();
        let notified = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&notified);
        let task = tokio::spawn(async move {
            let mut writer = FlushGate {
                bytes: output,
                flushing: Some(flushing),
                release: released,
            };
            publish_result(&mut writer, result, publication, &move || {
                flag.store(true, Ordering::SeqCst);
            })
            .await
        });
        timeout(Duration::from_secs(2), flush_started).await??;
        assert!(!acknowledge_if_idle(&executor)?);
        assert!(!notified.load(Ordering::SeqCst));
        release.send(()).map_err(|_| "release setter flush")?;
        timeout(Duration::from_secs(2), task).await???;
        assert!(notified.load(Ordering::SeqCst));
        assert!(!executor.has_active_work());
        let result: Value = serde_json::from_slice(&bytes.lock().map_err(|_| "output")?)?;
        assert_eq!(result["result"]["changed"], true);
        assert_eq!(result["result"]["revision"], 2);
        assert_eq!(result["result"]["source"]["byte_length"], 690);
        assert_eq!(result["result"]["collection"]["byte_length"], 704);
        assert!(acknowledge_if_idle(&executor)?);
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

    mod atom_label_publication_tests;
    mod preferences_publication_tests;
}
