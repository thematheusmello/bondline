//! Stdio MCP adapter. The SDK handles protocol lifecycle; the registry handles operations.

#[cfg(not(target_arch = "wasm32"))]
pub mod control;
mod work;
pub use work::{
    CommandExecutor, UiCaptureWork, UiFileWork, UiSdfRemoveContext, UiSdfWork,
    UiSmilesConversionWork, UiSmilesPasteContext, UiSmilesPasteWork, UiSmilesRecordsContext,
    UiSmilesRecordsResult, UiSmilesRecordsWork, UiSmilesWork,
};

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{CommandServer, Error, serve_stdio};

pub type SharedEngine = std::sync::Arc<std::sync::Mutex<bondline_commands::Engine>>;

#[cfg(test)]
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_test_support;
