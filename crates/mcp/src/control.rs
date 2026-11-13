//! Opt-in, bounded newline JSON control on IPv4 loopback only.

use crate::SharedEngine;
use bondline_commands::{Error as CommandError, ErrorInfo, MAX_REQUEST_BYTES};
use serde_json::{Value, json};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    thread::JoinHandle,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream, tcp::OwnedWriteHalf},
    sync::oneshot,
    task::JoinSet,
    time::timeout,
};

const MAX_CLIENTS: usize = 16;
const IO_DEADLINE: Duration = Duration::from_secs(30);

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
    stop: Option<oneshot::Sender<()>>,
    worker: Option<JoinHandle<io::Result<()>>>,
}

impl LoopbackServer {
    pub fn bind(
        port: u16,
        engine: SharedEngine,
        on_change: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, Error> {
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
                    serve(listener, engine, Arc::new(on_change), shutdown).await
                })
            })?;
        Ok(Self {
            address,
            stop: Some(stop),
            worker: Some(worker),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
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
    engine: SharedEngine,
    on_change: Arc<dyn Fn() + Send + Sync>,
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
                        clients.spawn(connection(stream, Arc::clone(&engine), Arc::clone(&on_change)));
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
    engine: SharedEngine,
    on_change: Arc<dyn Fn() + Send + Sync>,
) -> io::Result<()> {
    let (read, mut write) = stream.into_split();
    let mut reader = BufReader::new(read);
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
        let result = engine
            .lock()
            .map(|mut engine| engine.execute_json(&bytes))
            .map_err(|_| ());
        match result {
            Ok(Ok(result)) => {
                if result.changed {
                    on_change();
                }
                send(&mut write, json!({"result":result})).await?;
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
