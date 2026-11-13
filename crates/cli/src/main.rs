//! Headless adapter for the shared command registry.

use bondline_commands::{Engine, ErrorInfo};
use std::{io::Write, process::ExitCode};

const USAGE: &str = "Usage: bondline-cli commands\n       bondline-cli run --cmd '<command JSON>' [--cmd '<command JSON>' ...]\n       bondline-cli mcp\n       bondline-cli --help";

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{USAGE}")]
    Usage,
    #[error(transparent)]
    Command(#[from] bondline_commands::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Mcp(#[from] bondline_mcp::Error),
}

impl Error {
    fn info(&self) -> ErrorInfo {
        match self {
            Self::Command(error) => error.info(),
            _ => ErrorInfo {
                code: match self {
                    Self::Usage => "usage",
                    Self::Io(_) => "io_error",
                    Self::Mcp(_) => "mcp_error",
                    _ => "json_error",
                },
                message: self.to_string(),
            },
        }
    }
}

fn run() -> Result<(), Error> {
    let mut arguments = std::env::args().skip(1);
    let operation = arguments.next();
    if operation.as_deref() == Some("mcp") {
        if arguments.next().is_some() {
            return Err(Error::Usage);
        }
        bondline_mcp::serve_stdio(std::sync::Arc::new(
            std::sync::Mutex::new(Engine::default()),
        ))?;
        return Ok(());
    }
    let mut output = std::io::stdout().lock();
    match operation.as_deref() {
        None | Some("--help") if arguments.next().is_none() => writeln!(output, "{USAGE}")?,
        Some("commands") if arguments.next().is_none() => {
            serde_json::to_writer(&mut output, &bondline_commands::registry())?;
            output.write_all(b"\n")?;
        }
        Some("run") => {
            let mut engine = Engine::default();
            let mut result = None;
            while let Some(flag) = arguments.next() {
                if flag != "--cmd" {
                    return Err(Error::Usage);
                }
                let request = arguments.next().ok_or(Error::Usage)?;
                result = Some(engine.execute_json(request.as_bytes())?);
            }
            serde_json::to_writer(&mut output, &result.ok_or(Error::Usage)?)?;
            output.write_all(b"\n")?;
        }
        _ => return Err(Error::Usage),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // A closed stderr cannot carry a second diagnostic; exit status still signals failure.
            let _ = serde_json::to_writer(
                std::io::stderr().lock(),
                &serde_json::json!({"error":error.info()}),
            );
            ExitCode::from(
                if matches!(error, Error::Io(_) | Error::Json(_) | Error::Mcp(_)) {
                    1
                } else {
                    2
                },
            )
        }
    }
}
