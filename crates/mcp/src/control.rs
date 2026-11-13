//! Opt-in, bounded newline JSON control on IPv4 loopback only.

use crate::{CommandExecutor, SharedEngine};
use bondline_commands::{Error as CommandError, ErrorInfo, MAX_REQUEST_BYTES};
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
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream, tcp::OwnedWriteHalf},
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

async fn send(writer: &mut OwnedWriteHalf, value: Value) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(&value).map_err(io::Error::other)?;
    bytes.push(b'\n');
    timeout(IO_DEADLINE, writer.write_all(&bytes))
        .await
        .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?
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
        let result = executor
            .engine()
            .lock()
            .map(|mut engine| engine.prepare_json(&bytes))
            .map_err(|_| ());
        match result {
            Ok(Ok(prepared)) => {
                match executor
                    .complete(prepared, tokio::time::sleep(IO_DEADLINE))
                    .await
                {
                    Ok(result) => {
                        if result.changed || result.file_operation.is_some() {
                            on_change();
                        }
                        send(&mut write, json!({"result":result})).await?;
                    }
                    Err(error) => send(&mut write, json!({"error":error})).await?,
                }
            }
            Ok(Err(error)) => {
                let malformed = matches!(error, CommandError::InvalidParams(_));
                send(&mut write, json!({"error":error.info()})).await?;
                // A rejected first line must not turn an HTTP body into subsequent commands.
                if malformed {
                    return Ok(());
                }
            }
            Err(()) => {
                send(&mut write, json!({"error":{"code":"engine_unavailable","message":"document engine lock poisoned"}})).await?;
                return Ok(());
            }
        }
    }
}
