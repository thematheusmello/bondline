//! Stdio MCP adapter. The SDK handles protocol lifecycle; the registry handles operations.

pub mod control;
mod work;
pub use work::{CommandExecutor, UiFileWork};

use bondline_commands::{Command, Engine, MAX_REQUEST_BYTES};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ErrorCode, Implementation,
        ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
        ToolAnnotations,
    },
    service::RequestContext,
    transport::async_rw::AsyncRwTransport,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub type SharedEngine = Arc<Mutex<Engine>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("registry schema for {0} is not an object")]
    InvalidSchema(&'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Initialize(Box<rmcp::service::ServerInitializeError>),
    #[error(transparent)]
    Task(#[from] tokio::task::JoinError),
}

impl From<rmcp::service::ServerInitializeError> for Error {
    fn from(error: rmcp::service::ServerInitializeError) -> Self {
        Self::Initialize(Box::new(error))
    }
}

pub struct CommandServer {
    tools: Vec<Tool>,
    executor: CommandExecutor,
}

#[cfg(test)]
#[path = "../../../xtask/oracles/files_test_support.rs"]
mod file_test_support;

impl CommandServer {
    pub fn new(engine: SharedEngine) -> Result<Self, Error> {
        let tools = bondline_commands::registry()
            .into_iter()
            .map(|info| {
                let schema = info
                    .input_schema
                    .as_value()
                    .as_object()
                    .cloned()
                    .ok_or(Error::InvalidSchema(info.name))?;
                Ok(
                    Tool::new(info.name, info.description, schema).with_annotations(
                        ToolAnnotations::new()
                            .read_only(info.read_only)
                            .open_world(false),
                    ),
                )
            })
            .collect::<Result<_, Error>>()?;
        Ok(Self {
            executor: CommandExecutor::new(engine),
            tools,
        })
    }
}

impl ServerHandler for CommandServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("bondline", env!("CARGO_PKG_VERSION")))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.iter().find(|tool| tool.name == name).cloned()
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        if request.is_some_and(|request| request.cursor.is_some()) {
            return Err(McpError::invalid_params("unknown tool-list cursor", None));
        }
        Ok(ListToolsResult::with_all_items(self.tools.clone()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        if self.get_tool(&request.name).is_none() {
            return Err(McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                "unknown Bondline tool",
                None,
            ));
        }
        if request.input_responses.is_some() || request.request_state.is_some() {
            return Err(McpError::invalid_params(
                "input continuations are not supported by these tools",
                None,
            ));
        }
        let prepared = Command::parse(
            &request.name,
            Value::Object(request.arguments.unwrap_or_default()),
        )
        .map_err(|error| error.info())
        .and_then(|command| self.executor.prepare_command(command));
        let result = match prepared {
            Ok(prepared) => {
                self.executor
                    .complete(prepared, context.ct.cancelled())
                    .await
            }
            Err(error) if error.code == "engine_unavailable" => {
                return Err(McpError::internal_error(
                    "document engine lock poisoned",
                    None,
                ));
            }
            Err(error) => Err(error),
        };
        let response = match result {
            Ok(result) => {
                CallToolResult::structured(serde_json::to_value(result).map_err(|_| {
                    McpError::internal_error("could not serialize command result", None)
                })?)
            }
            Err(error) => CallToolResult::structured_error(json!({"error":error})),
        };
        Ok(response.into())
    }
}

/// Run one offline process-local session. No network listener is opened.
pub fn serve_stdio(engine: SharedEngine) -> Result<(), Error> {
    let server = CommandServer::new(engine)?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let transport =
                AsyncRwTransport::<RoleServer, _, _>::new(tokio::io::stdin(), tokio::io::stdout())
                    .with_max_line_length(MAX_REQUEST_BYTES);
            server.serve(transport).await?.waiting().await?;
            Ok(())
        })
}
