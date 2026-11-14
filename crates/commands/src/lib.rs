//! One typed command registry and authored document history for every interface.

use bondline_io::{SourceInfo, StoredDocument};
use bondline_model::{Document, PageId, PageSize};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::VecDeque, sync::Arc};

mod bond;
mod capture;
mod close;
mod content;
mod controls;
mod file_ui;
mod files;
pub use bond::{BondCreate, BondExtend, ExtendHydrogenPolicy};
pub use close::{CloseAction, CloseDocument, CloseState};
pub use content::{CollectionSummary, RecordSummary, StoredContent};
pub use controls::{ControlInfo, ControlKey, ControlShortcut};
pub use file_ui::{UiFile, UiFileAction, UiFilePhase, UiFilePlan, UiFileRequest, UiFileState};
pub use files::{
    Completed, FileFormat, FileOperation, FileState, MolVersionChoice, OpenCdx, OpenCdxml, OpenMol,
    OpenSdf, SaveCdx, SaveCdxml, SaveMol, SaveSdf,
};
#[cfg(test)]
mod close_tests;
#[cfg(test)]
mod file_ui_tests;
#[cfg(test)]
mod sdf_tests;
#[cfg(test)]
mod source_tests;
#[cfg(test)]
mod storage_tests;
pub use bondline_render::Cancellation;
pub use capture::{CaptureAdapter, PngCapture, Prepared, RenderBackend, RenderCapture};

pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_HISTORY: usize = 128;
const MAX_HISTORY_BYTES: usize = 128 * 1024 * 1024;

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
    Close(CloseDocument) => ("document.close", "Request, explicitly discard, or cancel closing the current document revision.", false),
    UiFile(UiFile) => ("ui.file", "Request or cancel native file presentation; requires an attached native host.", false),
    UiControls(NoParams) => ("ui.controls", "Discover delivered native controls, shortcuts, typed bindings and contextual availability.", true),
    OpenCdxml(OpenCdxml) => ("document.open_cdxml", "Open an explicitly selected CDXML file without discarding unsaved edits by default.", false),
    SaveCdxml(SaveCdxml) => ("document.save_cdxml", "Atomically save complete CDXML data to an explicitly selected path; overwrite requires opt-in.", false),
    OpenCdx(OpenCdx) => ("document.open_cdx", "Open the supported read-only CDX projection while retaining complete source bytes; unsaved changes require explicit discard.", false),
    SaveCdx(SaveCdx) => ("document.save_cdx", "Atomically save exact unchanged CDX backing; changed or source-free documents are refused.", false),
    OpenMol(OpenMol) => ("document.open_mol", "Open supported MOL V2000 or V3000 data with an explicit coordinate frame and retained source.", false),
    SaveMol(SaveMol) => ("document.save_mol", "Atomically save supported MOL data, retaining its associated version by default without silently losing source metadata or layout.", false),
    OpenSdf(OpenSdf) => ("document.open_sdf", "Open every validated SDF record with complete source and ordered data; chemical preview is unavailable.", false),
    SaveSdf(SaveSdf) => ("document.save_sdf", "Atomically save the complete SDF collection without changing molecular versions or ordered data.", false),
    PageSetSize(PageSetSize) => ("page.set_size", "Set a page's width and height in physical points.", false),
    BondCreate(BondCreate) => ("bond.create", "Create an independent single carbon bond between two authored points.", false),
    BondExtend(BondExtend) => ("bond.extend", "Extend a terminal neutral carbon with a Single bond, explicitly authoring its H3 to H2 and the new carbon's H3; requires native complete data.", false),
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
    #[error(
        "terminal-carbon extension requires source-free or completely mapped native CDXML data"
    )]
    UnsupportedExtensionSource,
    #[error("fragment and atom IDs must be greater than zero")]
    InvalidBondTarget,
    #[error("atom {atom_id} does not exist in fragment {fragment_id}")]
    MissingAtom { fragment_id: u32, atom_id: u32 },
    #[error(
        "extension supports a positioned neutral terminal carbon with one Single carbon bond and literal H3 only"
    )]
    UnsupportedBondExtension,
    #[error("a new atom cannot occupy an existing atom's position")]
    OccupiedEndpoint,
    #[error(transparent)]
    Render(#[from] bondline_render::Error),
    #[error(transparent)]
    Png(#[from] bondline_io::Error),
    #[error("GPU capture is not available on this target")]
    BackendUnavailable,
    #[error("source contains unmapped content and cannot be faithfully previewed yet")]
    UnsupportedDocumentPreview,
    #[error("this operation requires {required} content; the session contains {actual} content")]
    WrongContent {
        required: &'static str,
        actual: &'static str,
    },
    #[error("opening a new document requires explicit permission to discard unsaved changes")]
    UnsavedChanges,
    #[error("document revision no longer matches the operation; no change was applied")]
    StaleRevision,
    #[error("the session close is committed; new commands are refused")]
    CloseCommitted,
    #[error("close operation identity counter exhausted")]
    CloseEpochExhausted,
    #[error("no native file-presentation host is attached")]
    UiUnavailable,
    #[error("native file presentation is busy")]
    UiBusy,
    #[error("another save is still in progress; try again after it finishes")]
    SaveBusy,
    #[error("native file request or its document context is obsolete")]
    UiStaleRequest,
    #[error("native file request identity counter exhausted")]
    UiEpochExhausted,
    #[error("invalid native file request parameters: {0}")]
    InvalidUiFileParameters(&'static str),
    #[error(transparent)]
    Mol(#[from] bondline_io::mol::MolError),
    #[error("native file access is unavailable on this target")]
    FileBackendUnavailable,
    #[error("document engine lock poisoned")]
    EngineUnavailable,
    #[error("canonical file path is not UTF-8 (file bytes published: {published})")]
    UnrepresentableFilePath { published: bool },
    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    File(#[from] bondline_io::files::FileError),
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
            Self::Model(
                bondline_model::Error::MissingPage(_) | bondline_model::Error::MissingFragment(_),
            )
            | Self::MissingAtom { .. } => "not_found",
            Self::RevisionExhausted => "revision_exhausted",
            Self::IdentityExhausted => "identity_exhausted",
            Self::Render(bondline_render::Error::Cancelled) => "cancelled",
            Self::Render(bondline_render::Error::InvalidDimensions) => "invalid_params",
            Self::Render(bondline_render::Error::UnsupportedChemicalContent)
            | Self::Render(bondline_render::Error::LabelPlacement)
            | Self::Render(bondline_render::Error::LabelOverlap)
            | Self::SourceBackedEdit
            | Self::WrongContent { .. }
            | Self::UnsupportedExtensionSource
            | Self::UnsupportedBondExtension => "unsupported_content",
            Self::Render(bondline_render::Error::LabelScale) => "unsupported_scale",
            Self::Render(_) | Self::Png(_) => "render_failed",
            Self::BackendUnavailable => "backend_unavailable",
            Self::UnsupportedDocumentPreview => "unsupported_content",
            Self::UnsavedChanges => "unsaved_changes",
            Self::StaleRevision => "stale_revision",
            Self::CloseCommitted => "close_committed",
            Self::CloseEpochExhausted => "close_epoch_exhausted",
            Self::UiUnavailable => "ui_unavailable",
            Self::UiBusy => "ui_busy",
            Self::SaveBusy => "busy",
            Self::UiStaleRequest => "ui_stale_request",
            Self::UiEpochExhausted => "ui_epoch_exhausted",
            Self::FileBackendUnavailable => "backend_unavailable",
            Self::Mol(error) => error.code(),
            Self::EngineUnavailable => "engine_unavailable",
            Self::UnrepresentableFilePath { published: true } => "save_published",
            Self::UnrepresentableFilePath { published: false } => "invalid_path",
            #[cfg(not(target_arch = "wasm32"))]
            Self::File(error) => {
                if error.published() {
                    "save_published"
                } else {
                    error.code()
                }
            }
            _ => "invalid_params",
        };
        let mut message = self.to_string();
        #[cfg(not(target_arch = "wasm32"))]
        if let Self::File(error) = self {
            if error.published() {
                message = format!(
                    "New file bytes were published; {} at {}. The old file was not restored. {}",
                    error.code(),
                    error.stage(),
                    message
                );
            }
            if let Some(path) = error.recovery_path() {
                message.push_str(&format!(" Recover the retained original at {path:?}."));
            }
        }
        ErrorInfo { code, message }
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
    pub document: Option<Arc<Document>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    pub undo_depth: usize,
    pub redo_depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture: Option<PngCapture>,
    pub file: FileState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_operation: Option<FileOperation>,
    pub close: CloseState,
    pub close_epoch: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_operation: Option<CloseState>,
    pub ui_file: UiFileState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_file_operation: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controls: Option<Vec<ControlInfo>>,
}

#[derive(Debug)]
pub struct Engine {
    document: Arc<StoredContent>,
    saved: Arc<StoredContent>,
    file_path: Option<String>,
    file_format: FileFormat,
    save_slot: Arc<()>,
    generation: u64,
    identities: bond::Ids,
    revision: u64,
    undo: VecDeque<Arc<StoredContent>>,
    redo: VecDeque<Arc<StoredContent>>,
    close_policy: close::ClosePolicy,
    file_ui: file_ui::FileUiPolicy,
}

impl Default for Engine {
    fn default() -> Self {
        Self::from_document(StoredDocument::default())
    }
}

impl Engine {
    /// Initialize a clean unnamed session, sharing its complete current and saved snapshot.
    pub fn from_document(document: StoredDocument) -> Self {
        Self::from_content(document.into())
    }

    fn from_content(document: StoredContent) -> Self {
        let file_format = match &document {
            StoredContent::Sdf { .. } => FileFormat::Sdf,
            StoredContent::Drawing(document) => {
                if document
                    .source_info()
                    .is_some_and(|source| source.format == "cdx")
                {
                    FileFormat::Cdx
                } else {
                    document
                        .mol_version()
                        .map_or(FileFormat::Cdxml, FileFormat::from_mol)
                }
            }
        };
        let document = Arc::new(document);
        Self {
            saved: Arc::clone(&document),
            document,
            file_path: None,
            file_format,
            save_slot: Arc::new(()),
            generation: 0,
            identities: bond::Ids::default(),
            revision: 0,
            undo: VecDeque::new(),
            redo: VecDeque::new(),
            close_policy: close::ClosePolicy::default(),
            file_ui: file_ui::FileUiPolicy::default(),
        }
    }

    pub fn content(&self) -> &StoredContent {
        &self.document
    }

    pub fn stored_document(&self) -> Result<&StoredDocument, Error> {
        self.document.drawing()
    }

    pub fn document(&self) -> Result<&Document, Error> {
        Ok(self.stored_document()?.model())
    }

    pub fn state(&self) -> CommandResult {
        CommandResult {
            changed: false,
            revision: self.revision,
            document: match self.document.as_ref() {
                StoredContent::Drawing(document) => Some(document.model_snapshot()),
                StoredContent::Sdf { .. } => None,
            },
            collection: self.document.collection(),
            source: self.document.source_info(),
            undo_depth: self.undo.len(),
            redo_depth: self.redo.len(),
            capture: None,
            file: FileState {
                format: self.file_format,
                path: self.file_path.clone(),
                dirty: self.document != self.saved,
            },
            file_operation: None,
            close: self.close_state(),
            close_epoch: self.close_policy.epoch,
            close_operation: None,
            ui_file: self.file_ui_state(),
            ui_file_operation: None,
            controls: None,
        }
    }

    pub fn execute_json(&mut self, input: &[u8]) -> Result<CommandResult, Error> {
        let completed = self.prepare_json(input)?.complete()?;
        self.finish(completed)
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
        let completed = self.prepare(command)?.complete()?;
        self.finish(completed)
    }

    pub fn prepare(&mut self, command: Command) -> Result<Prepared, Error> {
        if self.close_state() == CloseState::Committed && !matches!(command, Command::Inspect(_)) {
            return Err(Error::CloseCommitted);
        }
        let changed = match command {
            Command::Inspect(_) => false,
            Command::Close(params) => return Ok(Prepared::immediate(self.close(params)?)),
            Command::UiFile(params) => return Ok(Prepared::immediate(self.ui_file(params)?)),
            Command::UiControls(_) => {
                return Ok(Prepared::immediate(CommandResult {
                    controls: Some(self.controls()),
                    ..self.state()
                }));
            }
            Command::OpenCdxml(params) => {
                if self.document != self.saved && !params.discard_unsaved {
                    return Err(Error::UnsavedChanges);
                }
                return Ok(Prepared::file(
                    self.state(),
                    files::FilePlan::Open {
                        params,
                        encoding: files::OpenEncoding::Cdxml,
                        ui_request_id: None,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::SaveCdxml(params) => {
                let save_slot = self.reserve_save()?;
                self.stored_document()?;
                return Ok(Prepared::file(
                    self.state(),
                    files::FilePlan::Save {
                        params,
                        format: FileFormat::Cdxml,
                        ui_request_id: None,
                        document: Arc::clone(&self.document),
                        save_slot,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::OpenCdx(params) => {
                if self.document != self.saved && !params.discard_unsaved {
                    return Err(Error::UnsavedChanges);
                }
                return Ok(Prepared::file(
                    self.state(),
                    files::FilePlan::Open {
                        params: OpenCdxml {
                            path: params.path,
                            discard_unsaved: params.discard_unsaved,
                        },
                        encoding: files::OpenEncoding::Cdx,
                        ui_request_id: None,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::SaveCdx(params) => {
                let save_slot = self.reserve_save()?;
                self.stored_document()?;
                return Ok(Prepared::file(
                    self.state(),
                    files::FilePlan::Save {
                        params: SaveCdxml {
                            path: params.path,
                            overwrite: params.overwrite,
                        },
                        format: FileFormat::Cdx,
                        ui_request_id: None,
                        document: Arc::clone(&self.document),
                        save_slot,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::OpenMol(params) => {
                let transform = params.transform()?;
                if self.document != self.saved && !params.discard_unsaved {
                    return Err(Error::UnsavedChanges);
                }
                return Ok(Prepared::file(
                    self.state(),
                    files::FilePlan::Open {
                        params: OpenCdxml {
                            path: params.path,
                            discard_unsaved: params.discard_unsaved,
                        },
                        encoding: files::OpenEncoding::Mol(transform),
                        ui_request_id: None,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::SaveMol(params) => {
                let save_slot = self.reserve_save()?;
                let format = self.mol_save_format(params.version)?;
                return Ok(Prepared::file(
                    self.state(),
                    files::FilePlan::Save {
                        params: SaveCdxml {
                            path: params.path,
                            overwrite: params.overwrite,
                        },
                        format,
                        ui_request_id: None,
                        document: Arc::clone(&self.document),
                        save_slot,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::OpenSdf(params) => {
                if self.document != self.saved && !params.discard_unsaved {
                    return Err(Error::UnsavedChanges);
                }
                return Ok(Prepared::file(
                    self.state(),
                    files::FilePlan::Open {
                        params: OpenCdxml {
                            path: params.path,
                            discard_unsaved: params.discard_unsaved,
                        },
                        encoding: files::OpenEncoding::Sdf,
                        ui_request_id: None,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::SaveSdf(params) => {
                let save_slot = self.reserve_save()?;
                self.document.sdf()?;
                return Ok(Prepared::file(
                    self.state(),
                    files::FilePlan::Save {
                        params: SaveCdxml {
                            path: params.path,
                            overwrite: params.overwrite,
                        },
                        format: FileFormat::Sdf,
                        ui_request_id: None,
                        document: Arc::clone(&self.document),
                        save_slot,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::PageSetSize(params) => {
                let id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
                let size = PageSize::new(params.width_pt, params.height_pt)?;
                let mut document = self.stored_document()?.clone();
                document.model_mut().set_page_size(id, size)?;
                self.commit(document)?
            }
            Command::BondCreate(params) => {
                let (document, identities) =
                    bond::create(self.stored_document()?, params, self.identities)?;
                let changed = self.commit(document)?;
                self.identities = identities;
                changed
            }
            Command::BondExtend(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let (document, identities) =
                    bond::extend(self.stored_document()?, params, self.identities)?;
                let changed = self.commit(document)?;
                self.identities = identities;
                changed
            }
            Command::Undo(_) => self.undo()?,
            Command::Redo(_) => self.redo()?,
            Command::Capture(params) => {
                let id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
                let page = self
                    .document()?
                    .pages()
                    .iter()
                    .find(|page| page.id() == id)
                    .ok_or(bondline_model::Error::MissingPage(id.get()))?;
                if !self.stored_document()?.is_fully_mapped() {
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

    fn commit(&mut self, document: impl Into<StoredContent>) -> Result<bool, Error> {
        let document = document.into();
        if *self.document == document {
            return Ok(false);
        }
        let revision = self.next_revision()?;
        self.undo
            .push_back(std::mem::replace(&mut self.document, Arc::new(document)));
        if self.undo.len() > MAX_HISTORY {
            self.undo.pop_front();
        }
        self.redo.clear();
        self.trim_history(MAX_HISTORY_BYTES);
        self.revision = revision;
        self.invalidate_close();
        Ok(true)
    }

    fn trim_history(&mut self, budget: usize) {
        let retained = |skip| {
            StoredContent::retained_bytes(
                std::iter::once(self.document.as_ref())
                    .chain(std::iter::once(self.saved.as_ref()))
                    .chain(self.undo.iter().skip(skip).map(Arc::as_ref))
                    .chain(self.redo.iter().map(Arc::as_ref)),
            )
        };
        if retained(0) <= budget {
            return;
        }
        let (mut low, mut high) = (0, self.undo.len());
        // Retained suffixes are monotonic; find the smallest oldest-prefix eviction.
        while low < high {
            let middle = low + (high - low) / 2;
            if retained(middle) > budget {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        self.undo.drain(..low);
        while !self.redo.is_empty()
            && StoredContent::retained_bytes(
                std::iter::once(self.document.as_ref())
                    .chain(std::iter::once(self.saved.as_ref()))
                    .chain(self.undo.iter().map(Arc::as_ref))
                    .chain(self.redo.iter().map(Arc::as_ref)),
            ) > budget
        {
            self.redo.pop_front();
        }
    }

    fn undo(&mut self) -> Result<bool, Error> {
        let previous = self.undo.back().cloned().ok_or(Error::NothingToUndo)?;
        let revision = self.next_revision()?;
        self.undo.pop_back();
        self.redo
            .push_back(std::mem::replace(&mut self.document, previous));
        self.revision = revision;
        self.invalidate_close();
        Ok(true)
    }

    fn redo(&mut self) -> Result<bool, Error> {
        let next = self.redo.back().cloned().ok_or(Error::NothingToRedo)?;
        let revision = self.next_revision()?;
        self.redo.pop_back();
        self.undo
            .push_back(std::mem::replace(&mut self.document, next));
        self.revision = revision;
        self.invalidate_close();
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
                .document()?
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
                "bond.extend",
                "document.inspect",
                "document.close",
                "document.open_cdxml",
                "document.save_cdxml",
                "document.open_cdx",
                "document.save_cdx",
                "document.open_mol",
                "document.save_mol",
                "document.open_sdf",
                "document.save_sdf",
                "history.redo",
                "history.undo",
                "page.set_size",
                "render.capture",
                "ui.controls",
                "ui.file"
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
        let page = result
            .document
            .as_ref()
            .ok_or("drawing result")?
            .pages()
            .first()
            .ok_or("missing page")?;
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
        let original = engine.document()?.clone();
        engine.dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":612,"height_pt":792}),
        )?;
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(engine.document()?, &original);
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
