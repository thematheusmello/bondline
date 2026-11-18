//! One typed command registry and authored document history for every interface.

use bondline_io::{SourceInfo, StoredDocument};
use bondline_model::{Document, PageId, PageSize};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::VecDeque, sync::Arc};

mod atom;
mod atom_ui;
mod bond;
mod bond_tool;
mod boxed_atom;
mod caption;
mod capture;
mod chain;
mod chain_tool;
mod close;
mod content;
mod controls;
mod drawing_settings;
mod drawing_settings_ui;
mod file_ui;
mod files;
mod new_document;
mod preferences;
mod ring;
mod ring_tool;
mod sdf_edit;
mod sdf_query;
mod selection;
mod smiles_conversion;
mod smiles_export;
mod smiles_paste;
mod smiles_query;
mod smiles_records;
mod view;
pub use atom::AtomSetLabel;
pub use atom_ui::{UiAtomLabel, UiAtomLabelRequest};
pub use bond::{
    BondCreate, BondDrawOrder, BondExtend, BondSetOrder, ExtendHydrogenPolicy, OrderHydrogenPolicy,
};
pub use bond_tool::{BondToolOperation, BondToolState, EffectiveTool, SelectBondTool};
pub use boxed_atom::{AtomCreateLabel, AtomLabelStyle};
pub use caption::TextSetRun;
pub use chain::{
    ChainCreate, ChainCreatePath, ChainPathPlacement, ChainPathPoint, ChainPlacement, ChainSide,
};
pub use chain_tool::{
    ChainContext, ChainKind, ChainPreferences, ChainTarget, ChainToolOperation, UiAddChainRequest,
    UiChainTool,
};
pub use close::{CloseAction, CloseDocument, CloseState};
pub use content::{CollectionSummary, RecordSummary, SmilesFileSummary, StoredContent};
pub use controls::{ControlInfo, ControlKey, ControlShortcut};
pub use drawing_settings::{DocumentSettingsSet, DrawingSettingsSet};
pub use drawing_settings_ui::{UiDrawingSettings, UiDrawingSettingsRequest};
pub use file_ui::{
    UiFile, UiFileAction, UiFileContextStamp, UiFilePhase, UiFilePlan, UiFileRequest, UiFileState,
};
pub use files::{
    Completed, FileFormat, FileOperation, FileState, MolVersionChoice, OpenCdx, OpenCdxml, OpenMol,
    OpenSdf, OpenSmiles, SaveCdx, SaveCdxml, SaveMol, SaveSdf, SaveSmiles,
};
pub use new_document::NewDocument;
pub use preferences::{
    ApplicationPreferences, UiPreferences, UiPreferencesOperation, UiPreferencesRequest,
};
pub use ring::{
    ChairKind, ChairRingCreate, NamedRingCreate, RingCreate, RingCreateRequest, RingKind,
    RingPhase, RingPlacement, RingSide, RingSprout, SproutRing,
};
pub use ring_tool::{
    RingContext, RingToolOperation, SelectChairRingTool, SelectNamedRingTool, SelectRingTool,
    SelectRingToolRequest,
};
pub use sdf_edit::{SdfFieldSet, SdfRecordRemove};
pub use sdf_query::{
    MAX_SDF_INSPECT_VALUE_BYTES, MAX_SDF_RECORD_PAGE_BYTES, SdfFieldInspection, SdfRecordInspect,
    SdfRecordInspection, SdfValueOmission,
};
pub use selection::{
    SelectionContext, SelectionOperation, SelectionState, SelectionTranslate, UiSelection,
};
pub use smiles_conversion::{
    ConversionOutput, ConversionReceipt, ConvertSmilesToCdxml, SmilesConversionProfile,
};
pub use smiles_export::{ExportSmilesFragment, SmilesExportReceipt};
pub use smiles_paste::PasteSmiles;
pub use smiles_query::{
    MAX_SMILES_RECORD_INSPECT_BYTES, SmilesRecordInspect, SmilesRecordInspection,
};
pub use smiles_records::{ImportSmilesRecords, SmilesRecordSelection};
pub use view::{UiView, ViewOperation, ViewState};
#[cfg(test)]
mod close_tests;
#[cfg(test)]
mod drawing_settings_tests;
#[cfg(test)]
mod file_ui_tests;
#[cfg(test)]
mod new_document_tests;
#[cfg(test)]
mod sdf_tests;
#[cfg(test)]
mod source_tests;
#[cfg(test)]
mod storage_tests;
pub use bondline_render::Cancellation;
pub use capture::{
    CaptureAdapter, CaptureImageError, CaptureStamp, CaptureView, PngCapture, Prepared,
    RenderBackend, RenderCapture,
};

pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
pub const MAX_SMILES_PASTE_BYTES: usize = bondline_io::smiles::MAX_SMILES_BYTES;
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
                    $($name => {
                        if !params.is_object() {
                            return Err(Error::InvalidParams(
                                <serde_json::Error as serde::de::Error>::custom(
                                    "command parameters must be a JSON object"
                                )
                            ));
                        }
                        serde_json::from_value(params).map(Self::$variant).map_err(Error::InvalidParams)
                    }),+,
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
    NewDocument(NewDocument) => ("document.new", "Start a fresh source-free drawing session; unsaved data requires explicit discard and existing saved files remain untouched.", false),
    Close(CloseDocument) => ("document.close", "Request, explicitly discard, or cancel closing the current document revision.", false),
    UiFile(UiFile) => ("ui.file", "Request or cancel native document/file presentation, including the explicit SMILES paste form; requires an attached native host.", false),
    UiAtomLabel(UiAtomLabel) => ("ui.atom_label", "Open or cancel an exact existing-atom literal element/H editor; requires an attached native host and grants no authoring or Save authority.", false),
    UiControls(NoParams) => ("ui.controls", "Discover delivered native controls, shortcuts, typed bindings and contextual availability.", true),
    UiView(UiView) => ("ui.view", "Inspect or navigate the transient drawing view without editing content or history.", false),
    UiBondToolSelect(SelectBondTool) => ("ui.bond_tool", "Select a drawing bond order without editing the document or its history; gestures pass this order explicitly.", false),
    OpenCdxml(OpenCdxml) => ("document.open_cdxml", "Open an explicitly selected CDXML file without discarding unsaved edits by default.", false),
    SaveCdxml(SaveCdxml) => ("document.save_cdxml", "Atomically save complete CDXML data to an explicitly selected path; overwrite requires opt-in.", false),
    OpenCdx(OpenCdx) => ("document.open_cdx", "Open the supported read-only CDX projection while retaining complete source bytes; unsaved changes require explicit discard.", false),
    SaveCdx(SaveCdx) => ("document.save_cdx", "Atomically save exact unchanged CDX backing; changed or source-free documents are refused.", false),
    OpenMol(OpenMol) => ("document.open_mol", "Open supported MOL V2000 or V3000 data with an explicit coordinate frame and retained source.", false),
    SaveMol(SaveMol) => ("document.save_mol", "Atomically save supported MOL data, retaining its associated version by default without silently losing source metadata or layout.", false),
    OpenSdf(OpenSdf) => ("document.open_sdf", "Open every validated SDF record with complete source and ordered data; chemical preview is unavailable.", false),
    SaveSdf(SaveSdf) => ("document.save_sdf", "Atomically save the complete SDF collection without changing molecular versions or ordered data.", false),
    OpenSmiles(OpenSmiles) => ("document.open_smiles", "Open every supported SMILES record while retaining the exact complete file and opaque application data; chemical preview is unavailable.", false),
    ImportSmilesRecords(ImportSmilesRecords) => ("document.import_smiles_records", "Create a clean source-backed drawing from explicitly selected records of the already-open complete SMILES file; generated layout and partial projection never discard unselected or opaque source bytes.", false),
    SaveSmiles(SaveSmiles) => ("document.save_smiles", "Atomically save the complete current SMILES file, preserving opaque data and supported record edits.", false),
    ConvertSmilesToCdxml(ConvertSmilesToCdxml) => ("file.convert_smiles_to_cdxml", "Create new native CDXML with a chosen fixed-H layout: the default 1–12 neutral carbon/Single path, a bounded neutral acyclic structure, one 3–8 member Single ring with supported attached trees, or a bounded collection of these components. Supplied geometry and the current session stay unchanged.", false),
    ExportSmilesFragment(ExportSmilesFragment) => ("file.export_smiles_fragment", "Create a noncanonical SMILES file from one explicitly selected, complete-source fragment's supported literal-H chemical graph; geometry and document context are not encoded and the session stays unchanged.", false),
    SdfRecordInspect(SdfRecordInspect) => ("sdf.record.inspect", "Read a bounded ordered field page from an explicitly identified captured SDF record; oversized values are omitted explicitly.", true),
    SmilesRecordInspect(SmilesRecordInspect) => ("smiles.record.inspect", "Read the exact notation and file-relative metadata of one explicit captured SMILES record; opaque application data is not disclosed or decoded.", true),
    PasteSmiles(PasteSmiles) => ("structure.paste_smiles", "Append one complete supported neutral SMILES structure as a single undoable native drawing edit, preserving existing content; supplied aromatic markers resolve to a bounded deterministic integer-bond graph.", false),
    SdfFieldSet(SdfFieldSet) => ("sdf.field.set_value", "Replace one existing ordered SDF field value through a bounded immutable candidate, preserving every other source byte.", false),
    SdfRecordRemove(SdfRecordRemove) => ("sdf.record.remove", "Remove one explicit SDF record while preserving every remaining source byte and retaining at least one complete record.", false),
    PageSetSize(PageSetSize) => ("page.set_size", "Set a page's width and height in physical points.", false),
    BondCreate(BondCreate) => ("bond.create", "Create an independent carbon bond of the requested Single/Double/Triple order, authoring fixed neutral-carbon H counts; omitted order is Single.", false),
    BondExtend(BondExtend) => ("bond.extend", "Extend a neutral terminal carbon with consistent literal H and one Single/Double/Triple carbon bond using an explicit requested order/H policy; omitted order is Single and native complete data is required.", false),
    BondSetOrder(BondSetOrder) => ("bond.set_order", "Set an existing bond's Single/Double/Triple order and explicitly rectify both neutral-carbon endpoints' consistent literal hydrogen counts, preserving identities and geometry; requires editable native data or a synchronized SMILES record.", false),
    AtomSetLabel(AtomSetLabel) => ("atom.set_label", "Set one existing neutral atom's explicit element and literal attached-H count, preserving IDs, geometry and neighbors; source-free or completely mapped native CDXML is required and no valence inference is performed.", false),
    Undo(NoParams) => ("history.undo", "Restore the previous authored document.", false),
    Redo(NoParams) => ("history.redo", "Restore the next authored document.", false),
    Capture(RenderCapture) => ("render.capture", "Render a page preview to PNG using the explicit CPU or GPU backend.", true),
    UiSelection(UiSelection) => ("ui.selection", "Select native atoms without authoring", false),
    SelectionTranslate(SelectionTranslate) => ("selection.translate", "Translate the current native atom selection atomically", false),
    RingCreate(RingCreateRequest) => ("ring.create", "Construct a native carbon Single ring or named benzene/cyclopentadiene integer-bond ring, preserving shared data with explicit standalone or shared-atom/edge placement.", false),
    UiRingToolSelect(SelectRingToolRequest) => ("ui.ring_tool", "Select a native carbon Single-ring or named benzene/cyclopentadiene tool without editing the document or its history.", false),
    ChainCreate(ChainCreate) => ("chain.create", "Construct an explicit native carbon chain of Single bonds with a required interior angle and literal hydrogen policy, optionally sharing one existing neutral carbon.", false),
    ChainCreatePath(ChainCreatePath) => ("chain.create_path", "Construct a literal ordered native carbon path in one edit, optionally starting at an existing carbon; no snapping or angle/length inference.", false),
    DrawingSettingsSet(DrawingSettingsSet) => ("document.set_drawing_settings", "Replace both optional document-root creation preferences in one native drawing edit; explicit null clears a value and existing geometry stays unchanged.", false),
    UiDrawingSettings(UiDrawingSettings) => ("ui.drawing_settings", "Open or cancel a native document creation-preference draft; requires an attached host and grants no authoring or Save authority.", false),
    UiChainTool(UiChainTool) => ("ui.chain_tool", "Select a chain tool, set its transient constraints, or open/cancel a source-qualified numeric Add Chain draft; authoring uses the existing chain commands.", false),
    RingSprout(RingSprout) => ("ring.sprout", "Attach an independent native carbon ring through one Single connector in one edit, with explicit cycle geometry and literal hydrogen policy.", false),
    UiPreferences(UiPreferences) => ("ui.preferences", "Inspect or set session application preferences, or open/apply/cancel their native draft; never authors document data or acknowledges Save.", false),
    DocumentSettingsSet(DocumentSettingsSet) => ("document.set_document_settings", "Replace explicit bond length, chain angle and line width atomically; optionally set or clear spacing. Appearance or active spacing-reference changes must pass the whole-page ink certificate without moving geometry or changing label metrics.", false),
    TextSetRun(TextSetRun) => ("text.set_run", "Replace the decoded text of one existing styled caption run, preserving its font, size, face, colour, placement and every unrelated source byte. Requires native or caption-qualified CDXML and validates the final page and Save representation before one history edit.", false),
    AtomSetAuthoredLabel(AtomSetLabel) => ("atom.set_authored_label", "Replace one qualified box-positioned atom label's explicit element/H, literal text and bounds in one source-preserving edit; keeps node/bond geometry and style, validates the final page and Save representation, and performs no valence inference.", false),
    AtomCreateAuthoredLabel(AtomCreateLabel) => ("atom.create_authored_label", "Create one explicit box-positioned child label on a qualified imported hidden carbon using a supplied style and existing file resources; preserves IDs, geometry and carbon-display flags, synchronizes literal element/H and validates source and the final page before one history edit.", false),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("caption editing requires native or completely supported caption CDXML source")]
    UnsupportedCaptionSource,
    #[error("text ID must be greater than zero")]
    InvalidTextId,
    #[error("invalid command parameters: {0}")]
    InvalidParams(#[from] serde_json::Error),
    #[error(transparent)]
    Model(#[from] bondline_model::Error),
    #[error("page ID must be greater than zero")]
    InvalidPageId,
    #[error("fragment ID must be greater than zero")]
    InvalidFragmentId,
    #[error("SMILES fragment export requires a completely mapped drawing source")]
    IncompleteSmilesExportSource,
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
    #[error("fragment, atom and bond IDs must be greater than zero")]
    InvalidBondTarget,
    #[error("bond {bond_id} does not exist in fragment {fragment_id}")]
    MissingBond { fragment_id: u32, bond_id: u32 },
    #[error(
        "changing bond order requires editable native drawing data or a synchronized SMILES record"
    )]
    UnsupportedBondOrderSource,
    #[error(
        "changing bond order requires neutral nonisotopic carbon endpoints, Single/Double/Triple incident bonds and consistent sufficient literal hydrogens"
    )]
    UnsupportedBondOrder,
    #[error("{0}")]
    UnsupportedBondConversion(&'static str),
    #[error("atom {atom_id} does not exist in fragment {fragment_id}")]
    MissingAtom { fragment_id: u32, atom_id: u32 },
    #[error("invalid atom label: {0}")]
    InvalidAtomLabel(&'static str),
    #[error("atom label editing unavailable: {0}")]
    UnsupportedAtomLabel(&'static str),
    #[error(
        "extension requires a positioned neutral terminal carbon with one Single/Double/Triple carbon bond and consistent sufficient literal hydrogen count"
    )]
    UnsupportedBondExtension,
    #[error("a new atom cannot occupy an existing atom's position")]
    OccupiedEndpoint,
    #[error("invalid ring construction: {0}")]
    InvalidRing(&'static str),
    #[error("ring construction unavailable: {0}")]
    UnsupportedRing(&'static str),
    #[error("invalid chain construction: {0}")]
    InvalidChain(&'static str),
    #[error("chain construction unavailable: {0}")]
    UnsupportedChain(&'static str),
    #[error("drawing settings require source-free or completely mapped native CDXML data")]
    UnsupportedDrawingSettingsSource,
    #[error("the requested document style cannot be displayed safely: {0}")]
    UnsupportedDrawingStyle(&'static str),
    #[error("bond spacing must be a finite percentage from 1 through 100, or explicit null")]
    InvalidBondSpacing,
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
    #[error("invalid SDF record query: {0}")]
    InvalidSdfQuery(&'static str),
    #[error("invalid SMILES record query: {0}")]
    InvalidSmilesQuery(&'static str),
    #[error("invalid SDF field edit: {0}")]
    InvalidSdfField(&'static str),
    #[error("invalid SDF record operation: {0}")]
    InvalidSdfRecord(&'static str),
    #[error("invalid SMILES conversion parameters: {0}")]
    InvalidSmilesConversion(&'static str),
    #[error("invalid SMILES paste parameters: {0}")]
    InvalidSmilesPaste(&'static str),
    #[error("SMILES paste requires source-free or completely mapped native CDXML data")]
    UnsupportedSmilesPasteSource,
    #[error("SMILES paste resource limit: {0}")]
    SmilesPasteLimit(&'static str),
    #[error(transparent)]
    SmilesNotation(#[from] bondline_io::smiles::SmilesError),
    #[error(transparent)]
    SmilesLayout(#[from] bondline_io::smiles::LayoutError),
    #[error(transparent)]
    Document(#[from] bondline_io::DocumentError),
    #[error("invalid SMILES record import: {0}")]
    InvalidSmilesImport(&'static str),
    #[error("SDF field input exceeds the 64 KiB authored-LF or serialized-value budget")]
    SdfFieldInputLimit,
    #[error("a single SDF field exceeds the 1 MiB serialized query page budget")]
    SdfQueryPageLimit,
    #[error("SMILES record inspection exceeds the 128 KiB serialized object budget")]
    SmilesQueryLimit,
    #[error(transparent)]
    Sdf(#[from] bondline_io::sdf::SdfError),
    #[error(transparent)]
    SdfPreview(#[from] bondline_io::sdf::preview::PreviewError),
    #[error(transparent)]
    Display(#[from] bondline_io::display::DisplayError),
    #[error(transparent)]
    SmilesFile(#[from] bondline_io::smiles::SmilesFileError),
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
    #[error("drawing tool epoch space exhausted")]
    UiBondToolEpochExhausted,
    #[error("selection epoch space exhausted")]
    UiSelectionEpochExhausted,
    #[error("the selected atoms or selection epoch are obsolete")]
    StaleSelection,
    #[error("invalid selection: {0}")]
    InvalidSelection(&'static str),
    #[error("selection operation unavailable: {0}")]
    UnsupportedSelection(&'static str),
    #[error("atom {0} does not exist on the selected page")]
    MissingSelectionAtom(u32),
    #[error("view epoch space exhausted")]
    UiViewEpochExhausted,
    #[error("invalid view parameters: {0}")]
    InvalidView(&'static str),
    #[error("view navigation unavailable: {0}")]
    UnsupportedView(&'static str),
    #[error("the current SDF view bounds are not prepared; render an explicit zoom first")]
    ViewTargetUnavailable,
    #[error("the requested camera epoch is obsolete")]
    StaleView,
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
                bondline_model::Error::MissingPage(_)
                | bondline_model::Error::MissingFragment(_)
                | bondline_model::Error::MissingText(_)
                | bondline_model::Error::Text(bondline_model::TextError::MissingRun(_)),
            )
            | Self::MissingAtom { .. }
            | Self::MissingBond { .. } => "not_found",
            Self::MissingSelectionAtom(_) => "not_found",
            Self::StaleSelection => "stale_selection",
            Self::UiSelectionEpochExhausted => "ui_selection_epoch_exhausted",
            Self::UnsupportedSelection(_) => "unsupported_content",
            Self::RevisionExhausted => "revision_exhausted",
            Self::IdentityExhausted => "identity_exhausted",
            Self::Render(bondline_render::Error::Cancelled) => "cancelled",
            Self::Render(
                bondline_render::Error::InvalidDimensions | bondline_render::Error::InvalidView,
            ) => "invalid_params",
            Self::Render(bondline_render::Error::UnsupportedChemicalContent)
            | Self::Render(bondline_render::Error::UnsupportedText(_))
            | Self::Render(bondline_render::Error::LabelPlacement)
            | Self::Render(bondline_render::Error::LabelOverlap)
            | Self::SourceBackedEdit
            | Self::WrongContent { .. }
            | Self::UnsupportedExtensionSource
            | Self::UnsupportedBondExtension
            | Self::UnsupportedBondOrderSource
            | Self::UnsupportedBondOrder
            | Self::UnsupportedAtomLabel(_)
            | Self::UnsupportedRing(_)
            | Self::UnsupportedChain(_)
            | Self::UnsupportedDrawingSettingsSource
            | Self::UnsupportedCaptionSource
            | Self::UnsupportedDrawingStyle(_)
            | Self::UnsupportedSmilesPasteSource
            | Self::UnsupportedBondConversion(_) => "unsupported_content",
            Self::IncompleteSmilesExportSource => "unsupported_content",
            Self::Render(
                bondline_render::Error::LabelScale | bondline_render::Error::BondScale,
            ) => "unsupported_scale",
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
            Self::UiBondToolEpochExhausted | Self::UiViewEpochExhausted => "ui_epoch_exhausted",
            Self::InvalidView(_) => "invalid_params",
            Self::UnsupportedView(_) => "unsupported_content",
            Self::ViewTargetUnavailable => "viewport_unavailable",
            Self::StaleView => "stale_view",
            Self::FileBackendUnavailable => "backend_unavailable",
            Self::Mol(error) => error.code(),
            Self::Sdf(error) => error.code(),
            Self::SdfPreview(error) => error.code(),
            Self::Display(error) => error.code(),
            Self::SmilesFile(error) => error.code(),
            Self::SmilesNotation(error) => error.code(),
            Self::SmilesLayout(error) => error.code(),
            Self::Document(error) => error.code(),
            Self::SmilesPasteLimit(_) => "resource_limit",
            Self::SdfQueryPageLimit => "resource_limit",
            Self::SmilesQueryLimit => "resource_limit",
            Self::SdfFieldInputLimit => "resource_limit",
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
    #[serde(skip)]
    pub view_target_prepared: bool,
    /// Rust-only native result identity, captured atomically with its actual finish.
    /// It cannot authorize a Save/close acknowledgment and is absent from every wire reply.
    #[serde(skip)]
    pub ui_file_context_stamp: Option<UiFileContextStamp>,
    pub changed: bool,
    pub revision: u64,
    pub document: Option<Arc<Document>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smiles_file: Option<SmilesFileSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdf_record: Option<SdfRecordInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smiles_record: Option<SmilesRecordInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    pub undo_depth: usize,
    pub redo_depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture: Option<PngCapture>,
    pub file: FileState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_operation: Option<FileOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversion: Option<ConversionReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smiles_export: Option<SmilesExportReceipt>,
    pub close: CloseState,
    pub close_epoch: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_operation: Option<CloseState>,
    pub ui_file: UiFileState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_file_operation: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_atom_label: Option<UiAtomLabelRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_atom_label_operation: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_drawing_settings: Option<UiDrawingSettingsRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_drawing_settings_operation: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<ApplicationPreferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_preferences: Option<UiPreferencesRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_preferences_operation: Option<UiPreferencesOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_add_chain: Option<UiAddChainRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_add_chain_operation: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_chain_tool_operation: Option<ChainToolOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_selection: Option<SelectionState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_selection_operation: Option<SelectionOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_bond_tool_operation: Option<BondToolOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_ring_tool_operation: Option<RingToolOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_view_operation: Option<ViewOperation>,
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
    atom_ui: atom_ui::AtomLabelUiPolicy,
    drawing_settings_ui: drawing_settings_ui::DrawingSettingsUiPolicy,
    preferences_ui: preferences::PreferencesPolicy,
    chain_ui: chain_tool::ChainUiPolicy,
    selection: selection::SelectionPolicy,
    bond_tool: BondToolState,
    effective_tool: EffectiveTool,
    view: ViewState,
    view_canvas_size: Option<[u32; 2]>,
    sdf_view_extent: Option<(CaptureStamp, bondline_model::PageSize)>,
    view_focus: Option<(u64, PageId, bondline_model::Point2)>,
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
            StoredContent::SmilesFile(_) => FileFormat::Smiles,
            StoredContent::Drawing(document) => {
                if document
                    .source_info()
                    .is_some_and(|source| source.format == "smiles")
                {
                    FileFormat::Smiles
                } else if document
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
            atom_ui: atom_ui::AtomLabelUiPolicy::default(),
            drawing_settings_ui: drawing_settings_ui::DrawingSettingsUiPolicy::default(),
            preferences_ui: preferences::PreferencesPolicy::default(),
            chain_ui: chain_tool::ChainUiPolicy::default(),
            selection: selection::SelectionPolicy::default(),
            bond_tool: BondToolState::default(),
            effective_tool: EffectiveTool::default(),
            view: ViewState::default(),
            view_canvas_size: None,
            sdf_view_extent: None,
            view_focus: None,
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
            view_target_prepared: false,
            changed: false,
            revision: self.revision,
            document: match self.document.as_ref() {
                StoredContent::Drawing(document) => Some(document.model_snapshot()),
                StoredContent::Sdf { .. } | StoredContent::SmilesFile(_) => None,
            },
            collection: self.document.collection(),
            smiles_file: self.document.smiles_summary(),
            sdf_record: None,
            smiles_record: None,
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
            conversion: None,
            smiles_export: None,
            close: self.close_state(),
            close_epoch: self.close_policy.epoch,
            close_operation: None,
            ui_file: self.file_ui_state(),
            ui_file_operation: None,
            ui_atom_label: self.ui_atom_label_request(),
            ui_atom_label_operation: None,
            ui_drawing_settings: self.ui_drawing_settings_request(),
            ui_drawing_settings_operation: None,
            preferences: None,
            ui_preferences: self.ui_preferences_request(),
            ui_preferences_operation: None,
            ui_add_chain: self.ui_add_chain_request(),
            ui_add_chain_operation: None,
            ui_chain_tool_operation: None,
            ui_selection: (self.selection().epoch != 0).then(|| self.selection().clone()),
            ui_selection_operation: None,
            ui_bond_tool_operation: None,
            ui_ring_tool_operation: None,
            ui_view_operation: None,
            controls: None,
            ui_file_context_stamp: None,
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
        let mut drawing_focus = None;
        let changed = match command {
            Command::Inspect(_) => false,
            Command::NewDocument(params) => {
                return Ok(Prepared::immediate(self.new_document(params)?));
            }
            Command::UiView(params) => return Ok(Prepared::immediate(self.navigate_view(params)?)),
            Command::Close(params) => return Ok(Prepared::immediate(self.close(params)?)),
            Command::UiFile(params) => return Ok(Prepared::immediate(self.ui_file(params)?)),
            Command::UiAtomLabel(params) => {
                return Ok(Prepared::immediate(self.ui_atom_label(params)?));
            }
            Command::UiDrawingSettings(params) => {
                return Ok(Prepared::immediate(self.ui_drawing_settings(params)?));
            }
            Command::UiPreferences(params) => {
                return Ok(Prepared::immediate(self.ui_preferences(params)?));
            }
            Command::UiChainTool(params) => {
                return Ok(Prepared::immediate(self.ui_chain_tool(params)?));
            }
            Command::UiSelection(params) => {
                return Ok(Prepared::immediate(self.ui_selection(params)?));
            }
            Command::SelectionTranslate(params) => {
                return Ok(Prepared::immediate(self.translate_selection(params)?));
            }
            Command::UiBondToolSelect(params) => {
                return Ok(Prepared::immediate(self.select_bond_tool(params)?));
            }
            Command::UiRingToolSelect(params) => {
                return Ok(Prepared::immediate(self.select_ring_tool(params)?));
            }
            Command::UiControls(_) => {
                return Ok(Prepared::immediate(CommandResult {
                    controls: Some(self.controls()),
                    ..self.state()
                }));
            }
            Command::SdfRecordInspect(params) => {
                let plan = self.sdf_record_plan(params)?;
                return Ok(Prepared::sdf_record(self.state(), plan));
            }
            Command::SmilesRecordInspect(params) => {
                let plan = self.smiles_record_plan(params)?;
                return Ok(Prepared::smiles_record(self.state(), plan));
            }
            Command::PasteSmiles(params) => {
                let plan = self.smiles_paste_plan(params)?;
                return Ok(Prepared::smiles_paste(self.state(), plan));
            }
            Command::ImportSmilesRecords(params) => {
                let plan = self.smiles_import_plan(params)?;
                return Ok(Prepared::smiles_records(self.state(), plan));
            }
            Command::SdfFieldSet(params) => {
                let plan = self.sdf_field_plan(params)?;
                return Ok(Prepared::sdf_edit(self.state(), plan));
            }
            Command::SdfRecordRemove(params) => {
                let plan = self.sdf_record_remove_plan(params)?;
                return Ok(Prepared::sdf_edit(self.state(), plan));
            }
            Command::ConvertSmilesToCdxml(params) => {
                let plan = self.smiles_conversion_plan(params)?;
                return Ok(Prepared::file(self.state(), plan));
            }
            Command::ExportSmilesFragment(params) => {
                let plan = self.smiles_export_plan(params)?;
                return Ok(Prepared::file(self.state(), plan));
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
            Command::OpenSmiles(params) => {
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
                        encoding: files::OpenEncoding::Smiles,
                        ui_request_id: None,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::SaveSmiles(params) => {
                let save_slot = self.reserve_save()?;
                self.document.smiles_file()?;
                return Ok(Prepared::file(
                    self.state(),
                    files::FilePlan::Save {
                        params: SaveCdxml {
                            path: params.path,
                            overwrite: params.overwrite,
                        },
                        format: FileFormat::Smiles,
                        ui_request_id: None,
                        document: Arc::clone(&self.document),
                        save_slot,
                        revision: self.revision,
                        generation: self.generation,
                    },
                ));
            }
            Command::PageSetSize(params) => {
                if self.stored_document()?.model().has_authored_atom_labels() {
                    return Err(bondline_io::DocumentError::UnsupportedEdit.into());
                }
                if self.stored_document()?.has_caption_projection() {
                    return Err(Error::UnsupportedCaptionSource);
                }
                if self
                    .stored_document()?
                    .source_info()
                    .is_some_and(|source| source.format == "smiles")
                {
                    return Err(Error::SourceBackedEdit);
                }
                let id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
                let size = PageSize::new(params.width_pt, params.height_pt)?;
                let mut document = self.stored_document()?.clone();
                document.model_mut().set_page_size(id, size)?;
                self.commit(document)?
            }
            Command::ChainCreatePath(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let (document, identities) =
                    chain::create_path(self.stored_document()?, &params, self.identities)?;
                let changed = self.commit(document)?;
                self.identities = identities;
                drawing_focus = self.view_focus;
                changed
            }
            Command::DrawingSettingsSet(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let candidate = drawing_settings::set(self.stored_document()?, params)?;
                let changed = self.commit(candidate)?;
                drawing_focus = self.view_focus;
                changed
            }
            Command::DocumentSettingsSet(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let candidate = drawing_settings::set_document(self.stored_document()?, params)?;
                let changed = self.commit(candidate)?;
                drawing_focus = self.view_focus;
                changed
            }
            Command::TextSetRun(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let candidate = caption::set_run(self.stored_document()?, params)?;
                let changed = self.commit(candidate)?;
                drawing_focus = self.view_focus;
                changed
            }
            Command::ChainCreate(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let (document, identities) =
                    chain::create(self.stored_document()?, params, self.identities)?;
                let changed = self.commit(document)?;
                self.identities = identities;
                drawing_focus = self.view_focus;
                changed
            }
            Command::RingCreate(params) => {
                if params.expected_revision() != self.revision {
                    return Err(Error::StaleRevision);
                }
                let (document, identities) =
                    ring::create_request(self.stored_document()?, params, self.identities)?;
                let changed = self.commit(document)?;
                self.identities = identities;
                drawing_focus = self.view_focus;
                changed
            }
            Command::RingSprout(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let (document, identities) =
                    ring::create_sprout(self.stored_document()?, params, self.identities)?;
                let changed = self.commit(document)?;
                self.identities = identities;
                drawing_focus = self.view_focus;
                changed
            }
            Command::BondCreate(params) => {
                let focus = (
                    params.page_id,
                    params.begin_x_pt,
                    params.begin_y_pt,
                    params.end_x_pt,
                    params.end_y_pt,
                );
                let (document, identities) =
                    bond::create(self.stored_document()?, params, self.identities)?;
                let changed = self.commit(document)?;
                self.identities = identities;
                drawing_focus = PageId::new(focus.0)
                    .zip(
                        bondline_model::Point2::new(
                            focus.1 / 2.0 + focus.3 / 2.0,
                            focus.2 / 2.0 + focus.4 / 2.0,
                        )
                        .ok(),
                    )
                    .map(|(page, point)| (self.generation, page, point));
                changed
            }
            Command::BondExtend(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let focus = (
                    params.page_id,
                    params.fragment_id,
                    params.begin_atom_id,
                    params.end_x_pt,
                    params.end_y_pt,
                );
                let (document, identities) =
                    bond::extend(self.stored_document()?, params, self.identities)?;
                let changed = self.commit(document)?;
                self.identities = identities;
                drawing_focus = self.document().ok().and_then(|document| {
                    document
                        .pages()
                        .iter()
                        .find(|page| page.id().get() == focus.0)
                        .and_then(|page| {
                            page.fragments()
                                .iter()
                                .find(|fragment| fragment.id().get() == focus.1)
                                .and_then(|fragment| {
                                    fragment
                                        .atoms()
                                        .iter()
                                        .find(|atom| atom.id.get() == focus.2)
                                })
                                .and_then(|atom| atom.position)
                                .and_then(|begin| {
                                    bondline_model::Point2::new(
                                        begin.x_pt() / 2.0 + focus.3 / 2.0,
                                        begin.y_pt() / 2.0 + focus.4 / 2.0,
                                    )
                                    .ok()
                                })
                                .map(|point| (self.generation, page.id(), point))
                        })
                });
                changed
            }
            Command::BondSetOrder(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                if self.stored_document()?.has_smiles_projection() {
                    let plan = self.smiles_order_plan(params)?;
                    return Ok(Prepared::smiles_records(self.state(), plan));
                }
                let document = bond::set_order(self.stored_document()?, params)?;
                drawing_focus = self.view_focus;
                self.commit(document)?
            }
            Command::AtomSetLabel(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let document = atom::set_label(self.stored_document()?, params)?;
                drawing_focus = self.view_focus;
                self.commit(document)?
            }
            Command::AtomSetAuthoredLabel(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let document = boxed_atom::set_label(self.stored_document()?, params)?;
                drawing_focus = self.view_focus;
                self.commit(document)?
            }
            Command::AtomCreateAuthoredLabel(params) => {
                if params.expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let document = boxed_atom::create_label(self.stored_document()?, params)?;
                drawing_focus = self.view_focus;
                self.commit(document)?
            }
            Command::Undo(_) => self.undo()?,
            Command::Redo(_) => self.redo()?,
            Command::Capture(params) => {
                let id = PageId::new(params.page_id).ok_or(Error::InvalidPageId)?;
                if matches!(self.document.as_ref(), StoredContent::Sdf { .. }) {
                    if id.get() != 1 {
                        return Err(bondline_model::Error::MissingPage(id.get()).into());
                    }
                    return self.prepare_capture(params);
                }
                self.document()?
                    .pages()
                    .iter()
                    .find(|page| page.id() == id)
                    .ok_or(bondline_model::Error::MissingPage(id.get()))?;
                if !self.stored_document()?.has_display_projection() {
                    return Err(Error::UnsupportedDocumentPreview);
                }
                return self.prepare_capture(params);
            }
        };
        if changed {
            self.view_focus = drawing_focus;
        }
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
        let selection_epoch = self.selection_epoch_after_edit()?;
        self.undo
            .push_back(std::mem::replace(&mut self.document, Arc::new(document)));
        if self.undo.len() > MAX_HISTORY {
            self.undo.pop_front();
        }
        self.redo.clear();
        self.trim_history(MAX_HISTORY_BYTES);
        self.revision = revision;
        self.clear_selection_after_edit(selection_epoch);
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
        let selection_epoch = self.selection_epoch_after_edit()?;
        self.undo.pop_back();
        self.redo
            .push_back(std::mem::replace(&mut self.document, previous));
        self.revision = revision;
        self.clear_selection_after_edit(selection_epoch);
        self.invalidate_close();
        Ok(true)
    }

    fn redo(&mut self) -> Result<bool, Error> {
        let next = self.redo.back().cloned().ok_or(Error::NothingToRedo)?;
        let revision = self.next_revision()?;
        let selection_epoch = self.selection_epoch_after_edit()?;
        self.redo.pop_back();
        self.undo
            .push_back(std::mem::replace(&mut self.document, next));
        self.revision = revision;
        self.clear_selection_after_edit(selection_epoch);
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
    fn registry_exposes_typed_closed_schemas() -> Result<(), Box<dyn std::error::Error>> {
        let commands = super::registry();
        let names: std::collections::BTreeSet<_> =
            commands.iter().map(|command| command.name).collect();
        assert_eq!(
            names,
            [
                "atom.create_authored_label",
                "atom.set_authored_label",
                "atom.set_label",
                "bond.create",
                "bond.extend",
                "bond.set_order",
                "chain.create",
                "chain.create_path",
                "document.import_smiles_records",
                "document.inspect",
                "document.new",
                "document.close",
                "document.open_cdxml",
                "document.save_cdxml",
                "document.open_cdx",
                "document.save_cdx",
                "document.open_mol",
                "document.save_mol",
                "document.open_sdf",
                "document.open_smiles",
                "document.save_sdf",
                "document.save_smiles",
                "document.set_drawing_settings",
                "file.convert_smiles_to_cdxml",
                "file.export_smiles_fragment",
                "history.redo",
                "history.undo",
                "page.set_size",
                "render.capture",
                "ring.create",
                "sdf.field.set_value",
                "sdf.record.inspect",
                "sdf.record.remove",
                "smiles.record.inspect",
                "structure.paste_smiles",
                "ui.controls",
                "ui.drawing_settings",
                "ui.atom_label",
                "ui.selection",
                "selection.translate",
                "ui.bond_tool",
                "ui.ring_tool",
                "ui.view",
                "ui.file",
                "ui.chain_tool",
                "ring.sprout",
                "ui.preferences",
                "document.set_document_settings",
                "text.set_run",
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(names.len(), commands.len());
        for command in commands {
            assert_eq!(command.input_schema.as_value()["type"], "object");
            let schema = command.input_schema.as_value();
            if let Some(variants) = schema["oneOf"].as_array() {
                assert_eq!(schema["unevaluatedProperties"], false);
                assert!(!variants.is_empty());
                for variant in variants {
                    assert_eq!(variant["type"], "object");
                    assert_eq!(variant["additionalProperties"], false);
                }
            } else if let Some(variants) = schema["anyOf"].as_array() {
                assert!(schema.get("additionalProperties").is_none());
                assert!(!variants.is_empty());
                for variant in variants {
                    let variant = match variant.get("$ref") {
                        None => variant,
                        Some(reference) => {
                            let reference = reference
                                .as_str()
                                .ok_or("schema reference must be a string")?;
                            let pointer = reference
                                .strip_prefix('#')
                                .ok_or("schema reference must be local")?;
                            schema
                                .pointer(pointer)
                                .ok_or("missing local schema reference target")?
                        }
                    };
                    assert_eq!(variant["type"], "object");
                    assert_eq!(variant["additionalProperties"], false);
                }
            } else {
                assert_eq!(schema["additionalProperties"], false);
            }
        }
        Ok(())
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
