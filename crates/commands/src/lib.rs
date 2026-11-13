//! One typed command registry and authored document history for every interface.

use bondline_io::{SourceInfo, StoredDocument};
use bondline_model::{Document, PageId, PageSize};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::VecDeque, sync::Arc};

mod bond;
mod capture;
pub use bond::BondCreate;
#[cfg(test)]
mod source_tests;
pub use bondline_render::Cancellation;
pub use capture::{CaptureAdapter, PngCapture, Prepared, RenderBackend, RenderCapture};

pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_HISTORY: usize = 128;

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NoParams {}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PageSetSize {
    #[schemars(range(min = 1))]
    pub page_id: u32,
    #[schemars(extend("exclusiveMinimum" = 0.0, "maximum" = bondline_model::MAX_PAGE_DIMENSION_PT))]
    pub width_pt: f64,
    #[schemars(extend("exclusiveMinimum" = 0.0, "maximum" = bondline_model::MAX_PAGE_DIMENSION_PT))]
    pub height_pt: f64,
}

#[derive(Debug, Serialize)]
pub struct CommandInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub read_only: bool,
    pub input_schema: Schema,
}

macro_rules! command_registry {
    ($($variant:ident($params:ty) => ($name:literal, $description:literal, $read_only:literal)),+ $(,)?) => {
        #[derive(Clone, Debug, Serialize)]
        #[serde(tag = "command", content = "params")]
        pub enum Command {
            $(#[serde(rename = $name)] $variant($params)),+
        }

        impl Command {
            pub fn name(&self) -> &'static str {
                match self { $(Self::$variant(_) => $name),+ }
            }

            pub fn parse(name: &str, params: Value) -> Result<Self, Error> {
                match name {
                    $($name => serde_json::from_value(params).map(Self::$variant).map_err(Error::InvalidParams)),+,
                    _ => Err(Error::UnknownCommand(name.to_owned())),
                }
            }
        }

        pub fn registry() -> Vec<CommandInfo> {
            vec![$(CommandInfo { name: $name, description: $description, read_only: $read_only,
                input_schema: schemars::schema_for!($params) }),+]
        }
    };
}

command_registry! {
    Inspect(NoParams) => ("document.inspect", "Inspect the authored document and history state.", true),
    PageSetSize(PageSetSize) => ("page.set_size", "Set a page's width and height in physical points.", false),
    BondCreate(BondCreate) => ("bond.create", "Create an independent single carbon bond between two authored points.", false),
    Undo(NoParams) => ("history.undo", "Restore the previous authored document.", false),
    Redo(NoParams) => ("history.redo", "Restore the next authored document.", false),
    Capture(RenderCapture) => ("render.capture", "Render a page preview to PNG using the explicit CPU or GPU backend.", true),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("invalid command parameters: {0}")]
    InvalidParams(#[from] serde_json::Error),
    #[error(transparent)]
    Model(#[from] bondline_model::Error),
    #[error("page ID must be greater than zero")]
    InvalidPageId,
    #[error("no undo steps available")]
    NothingToUndo,
    #[error("no redo steps available")]
    NothingToRedo,
    #[error("command exceeds the 1 MiB input bound")]
    RequestTooLarge,
    #[error("document revision counter exhausted")]
    RevisionExhausted,
    #[error("document identity space exhausted")]
    IdentityExhausted,
    #[error("a bond needs two distinct endpoint positions")]
    CoincidentEndpoints,
    #[error("bond creation in source-backed documents requires an editable format mapping")]
    SourceBackedEdit,
    #[error(transparent)]
    Render(#[from] bondline_render::Error),
    #[error(transparent)]
    Png(#[from] bondline_io::Error),
    #[error("GPU capture is not available on this target")]
    BackendUnavailable,
    #[error("source-backed documents cannot be faithfully previewed yet")]
    UnsupportedDocumentPreview,
}

#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    pub code: &'static str,
    pub message: String,
}

impl Error {
    pub fn info(&self) -> ErrorInfo {
        let code = match self {
            Self::UnknownCommand(_) => "unknown_command",
            Self::NothingToUndo | Self::NothingToRedo => "empty_history",
            Self::Model(bondline_model::Error::MissingPage(_)) => "not_found",
            Self::RevisionExhausted => "revision_exhausted",
            Self::IdentityExhausted => "identity_exhausted",
            Self::Render(bondline_render::Error::Cancelled) => "cancelled",
            Self::Render(bondline_render::Error::InvalidDimensions) => "invalid_params",
            Self::Render(bondline_render::Error::UnsupportedChemicalContent)
            | Self::Render(bondline_render::Error::LabelPlacement)
            | Self::Render(bondline_render::Error::LabelOverlap)
            | Self::SourceBackedEdit => "unsupported_content",
            Self::Render(bondline_render::Error::LabelScale) => "unsupported_scale",
            Self::Render(_) | Self::Png(_) => "render_failed",
            Self::BackendUnavailable => "backend_unavailable",
            Self::UnsupportedDocumentPreview => "unsupported_content",
            _ => "invalid_params",
        };
        ErrorInfo {
            code,
            message: self.to_string(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    command: String,
    params: Value,
}

/// A consistent snapshot. Documents are shared; later edits cannot change this result.
#[derive(Clone, Debug, Serialize)]
pub struct CommandResult {
    pub changed: bool,
    pub revision: u64,
    pub document: Arc<Document>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    pub undo_depth: usize,
    pub redo_depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture: Option<PngCapture>,
}

#[derive(Debug, Default)]
pub struct Engine {
    document: Arc<StoredDocument>,
    identities: bond::Ids,
    revision: u64,
    undo: VecDeque<Arc<StoredDocument>>,
    redo: VecDeque<Arc<StoredDocument>>,
}

impl Engine {
    /// Initialize a session from an owned snapshot. Import commands are not yet exposed.
    pub fn from_document(document: StoredDocument) -> Self {
        Self {
            document: Arc::new(document),
            identities: bond::Ids::default(),
            revision: 0,
            undo: VecDeque::new(),
            redo: VecDeque::new(),
        }
    }

    pub fn stored_document(&self) -> &StoredDocument {
        &self.document
    }

    pub fn document(&self) -> &Document {
        self.document.model()
    }

    pub fn state(&self) -> CommandResult {
        CommandResult {
            changed: false,
            revision: self.revision,
            document: self.document.model_snapshot(),
            source: self.document.source_info(),
            undo_depth: self.undo.len(),
            redo_depth: self.redo.len(),
            capture: None,
        }
    }

    pub fn execute_json(&mut self, input: &[u8]) -> Result<CommandResult, Error> {
        self.prepare_json(input)?.complete()
    }

    /// Validate and snapshot under the engine lock, then complete outside it.
    pub fn prepare_json(&mut self, input: &[u8]) -> Result<Prepared, Error> {
        if input.len() > MAX_REQUEST_BYTES {
            return Err(Error::RequestTooLarge);
        }
        let request: Request = serde_json::from_slice(input)?;
        self.prepare(Command::parse(&request.command, request.params)?)
    }

    pub fn dispatch(&mut self, name: &str, params: Value) -> Result<CommandResult, Error> {
        self.execute(Command::parse(name, params)?)
    }

    pub fn execute(&mut self, command: Command) -> Result<CommandResult, Error> {
        self.prepare(command)?.complete()
    }

    pub fn prepare(&mut self, command: Command) -> Result<Prepared, Error> {
        let changed = match command {
            Command::Inspect(_) => false,
            Command::PageSetSize(params) => {
                let id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
                let size = PageSize::new(params.width_pt, params.height_pt)?;
                let mut document = (*self.document).clone();
                document.model_mut().set_page_size(id, size)?;
                self.commit(document)?
            }
            Command::BondCreate(params) => {
                let (document, identities) = bond::create(&self.document, params, self.identities)?;
                let changed = self.commit(document)?;
                self.identities = identities;
                changed
            }
            Command::Undo(_) => self.undo()?,
            Command::Redo(_) => self.redo()?,
            Command::Capture(params) => {
                let id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
                let page = self
                    .document()
                    .pages()
                    .iter()
                    .find(|page| page.id() == id)
                    .ok_or(bondline_model::Error::MissingPage(id.get()))?;
                if self.document.source_info().is_some() {
                    return Err(Error::UnsupportedDocumentPreview);
                }
                return Prepared::capture(self.state(), params, page.clone());
            }
        };
        Ok(Prepared::immediate(CommandResult {
            changed,
            ..self.state()
        }))
    }

    fn next_revision(&self) -> Result<u64, Error> {
        self.revision.checked_add(1).ok_or(Error::RevisionExhausted)
    }

    fn commit(&mut self, document: StoredDocument) -> Result<bool, Error> {
        if *self.document == document {
            return Ok(false);
        }
        let revision = self.next_revision()?;
        self.undo
            .push_back(std::mem::replace(&mut self.document, Arc::new(document)));
        // shortcut: history is capped at 128 snapshots; add byte budgeting as chemical documents grow.
        if self.undo.len() > MAX_HISTORY {
            self.undo.pop_front();
        }
        self.redo.clear();
        self.revision = revision;
        Ok(true)
    }

    fn undo(&mut self) -> Result<bool, Error> {
        let previous = self.undo.back().cloned().ok_or(Error::NothingToUndo)?;
        let revision = self.next_revision()?;
        self.undo.pop_back();
        self.redo
            .push_back(std::mem::replace(&mut self.document, previous));
        self.revision = revision;
        Ok(true)
    }

    fn redo(&mut self) -> Result<bool, Error> {
        let next = self.redo.back().cloned().ok_or(Error::NothingToRedo)?;
        let revision = self.next_revision()?;
        self.redo.pop_back();
        self.undo
            .push_back(std::mem::replace(&mut self.document, next));
        self.revision = revision;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn history_is_bounded_and_revision_exhaustion_is_atomic()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = super::Engine::default();
        for width in 1000..1129 {
            engine.dispatch(
                "page.set_size",
                json!({"page_id":1,"width_pt":width,"height_pt":792}),
            )?;
        }
        assert_eq!(engine.state().undo_depth, 128);
        for _ in 0..128 {
            engine.dispatch("history.undo", json!({}))?;
        }
        assert!(engine.dispatch("history.undo", json!({})).is_err());
        assert_eq!(
            engine
                .document()
                .pages()
                .first()
                .ok_or("missing page")?
                .size()
                .width_pt(),
            1000.0
        );
        engine.revision = u64::MAX;
        let before = serde_json::to_vec(&engine.state())?;
        assert!(engine.dispatch("history.redo", json!({})).is_err());
        assert!(
            engine
                .dispatch(
                    "page.set_size",
                    json!({"page_id":1,"width_pt":72,"height_pt":72})
                )
                .is_err()
        );
        assert_eq!(serde_json::to_vec(&engine.state())?, before);
        Ok(())
    }

    #[test]
    fn registry_exposes_typed_closed_schemas() {
        let commands = super::registry();
        let names: std::collections::BTreeSet<_> =
            commands.iter().map(|command| command.name).collect();
        assert_eq!(
            names,
            [
                "bond.create",
                "document.inspect",
                "history.redo",
                "history.undo",
                "page.set_size",
                "render.capture"
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(names.len(), commands.len());
        for command in commands {
            assert_eq!(command.input_schema.as_value()["type"], "object");
            assert_eq!(
                command.input_schema.as_value()["additionalProperties"],
                false
            );
        }
    }

    #[test]
    fn page_command_changes_physical_dimensions_and_inspection_is_read_only()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = super::Engine::default();
        let result = engine.execute_json(
            br#"{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}"#,
        )?;
        assert!(result.changed);
        assert_eq!(result.revision, 1);
        assert_eq!(result.undo_depth, 1);
        let page = result.document.pages().first().ok_or("missing page")?;
        assert_eq!(page.size().width_pt(), 612.0);
        assert_eq!(page.size().height_pt(), 792.0);
        let inspect = engine.dispatch("document.inspect", json!({}))?;
        assert!(!inspect.changed);
        assert_eq!(inspect.revision, 1);
        assert_eq!(inspect.document, result.document);
        Ok(())
    }

    #[test]
    fn bad_commands_do_not_change_state_or_history() -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = super::Engine::default();
        let before = serde_json::to_vec(&engine.state())?;
        let inputs = [
            br#"{"command":"missing","params":{}}"#.as_slice(),
            br#"{"command":"history.undo","params":{}}"#,
            br#"{"command":"page.set_size","params":{"page_id":1,"width_pt":0,"height_pt":72}}"#,
            br#"{"command":"page.set_size","params":{"page_id":0,"width_pt":72,"height_pt":72}}"#,
            br#"{"command":"page.set_size","params":{"page_id":2,"width_pt":72,"height_pt":72}}"#,
            br#"{"command":"page.set_size","params":{"page_id":1,"width_pt":72}}"#,
            br#"{"command":"document.inspect","params":{"typo":true}}"#,
            br#"{"command":"document.inspect","params":{},"typo":true}"#,
        ];
        for input in inputs {
            assert!(engine.execute_json(input).is_err());
            assert_eq!(serde_json::to_vec(&engine.state())?, before);
        }
        assert!(
            engine
                .execute_json(&vec![b' '; super::MAX_REQUEST_BYTES + 1])
                .is_err()
        );
        assert!(
            engine
                .execute(super::Command::PageSetSize(super::PageSetSize {
                    page_id: 1,
                    width_pt: f64::INFINITY,
                    height_pt: 72.0,
                }))
                .is_err()
        );
        assert_eq!(serde_json::to_vec(&engine.state())?, before);
        Ok(())
    }

    #[test]
    fn no_ops_preserve_redo_and_new_edits_discard_the_old_branch()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = super::Engine::default();
        let original = engine.document().clone();
        engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        )?;
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.document(), &original);
        let size = original.pages().first().ok_or("missing page")?.size();
        let no_op = engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":size.width_pt(),"height_pt":size.height_pt()}),
        )?;
        assert!(!no_op.changed);
        assert_eq!(no_op.revision, 2);
        assert_eq!(no_op.redo_depth, 1);
        engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":720,"height_pt":720}),
        )?;
        assert!(engine.dispatch("history.redo", json!({})).is_err());
        assert_eq!(engine.state().redo_depth, 0);
        Ok(())
    }
}
