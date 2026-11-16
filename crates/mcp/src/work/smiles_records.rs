//! Qualified native record work and post-finish presentation without Save authority.

use super::{CommandExecutor, RegisteredWork, unavailable};
use crate::SharedEngine;
use bondline_commands::{
    BondSetOrder, BondToolState, Cancellation, CaptureStamp, CloseState, Command, CommandResult,
    Engine, Error, ErrorInfo, FileFormat, ImportSmilesRecords, StoredContent, UiFileAction,
    UiFileContextStamp, UiFilePhase,
};
use std::sync::Arc;

#[derive(Clone)]
struct NativeState {
    source: CaptureStamp,
    file: UiFileContextStamp,
    close_epoch: u64,
    view_epoch: u64,
    tool: BondToolState,
    file_epoch: u64,
    path: Option<String>,
    format: FileFormat,
}

impl NativeState {
    fn capture(engine: &Engine) -> Self {
        let state = engine.state();
        Self {
            source: engine.capture_stamp(),
            file: engine.ui_file_context_stamp(),
            close_epoch: state.close_epoch,
            view_epoch: engine.view().epoch,
            tool: engine.bond_tool(),
            file_epoch: state.ui_file.epoch,
            path: state.file.path,
            format: state.file.format,
        }
    }

    fn validate(&self, engine: &Engine) -> Result<(), ErrorInfo> {
        let state = engine.state();
        if state.close == CloseState::Committed {
            return Err(Error::CloseCommitted.info());
        }
        if !engine.capture_stamp_matches(&self.source) {
            return Err(Error::StaleRevision.info());
        }
        if state.close != CloseState::Open
            || !state.ui_file.available
            || state.close_epoch != self.close_epoch
            || engine.view().epoch != self.view_epoch
            || engine.bond_tool() != self.tool
            || state.ui_file.epoch != self.file_epoch
            || state.file.path != self.path
            || state.file.format != self.format
            || !engine.ui_file_context_stamp_matches(&self.file)
        {
            return Err(Error::UiStaleRequest.info());
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Target {
    Import {
        request_id: u64,
    },
    Order {
        page_id: u32,
        fragment_id: u32,
        bond_id: u32,
    },
}

/// A native presentation or existing-bond gesture, captured before the frame lock releases.
#[derive(Clone)]
pub struct UiSmilesRecordsContext {
    state: NativeState,
    target: Target,
}

impl UiSmilesRecordsContext {
    pub fn capture_import(engine: &Engine, request_id: u64) -> Result<Self, ErrorInfo> {
        let state = engine.state();
        if !matches!(engine.content(), StoredContent::SmilesFile(_))
            || !state.ui_file.request.as_ref().is_some_and(|request| {
                request.id == request_id
                    && request.action == UiFileAction::ImportSmilesRecords
                    && request.phase == UiFilePhase::Accepted
            })
        {
            return Err(Error::UiStaleRequest.info());
        }
        let context = Self {
            state: NativeState::capture(engine),
            target: Target::Import { request_id },
        };
        context.validate(engine)?;
        Ok(context)
    }

    pub fn capture_order(
        engine: &Engine,
        page_id: u32,
        fragment_id: u32,
        bond_id: u32,
    ) -> Result<Self, ErrorInfo> {
        let stored = engine.stored_document().map_err(|error| error.info())?;
        if !stored.has_smiles_projection() {
            return Err(Error::WrongContent {
                required: "smiles drawing",
                actual: "drawing",
            }
            .info());
        }
        let target = stored
            .model()
            .pages()
            .iter()
            .find(|page| page.id().get() == page_id)
            .and_then(|page| {
                page.fragments()
                    .iter()
                    .find(|fragment| fragment.id().get() == fragment_id)
            })
            .and_then(|fragment| {
                fragment
                    .bonds()
                    .iter()
                    .find(|bond| bond.id.get() == bond_id)
            });
        if target.is_none() {
            return Err(Error::InvalidBondTarget.info());
        }
        let context = Self {
            state: NativeState::capture(engine),
            target: Target::Order {
                page_id,
                fragment_id,
                bond_id,
            },
        };
        context.validate(engine)?;
        Ok(context)
    }

    /// Consuming an import intent does not change the captured source or revive old work.
    pub fn is_current(&self, engine: &Engine) -> bool {
        self.validate(engine).is_ok()
    }

    fn validate(&self, engine: &Engine) -> Result<(), ErrorInfo> {
        self.state.validate(engine)?;
        match self.target {
            Target::Import { .. } if !matches!(engine.content(), StoredContent::SmilesFile(_)) => {
                Err(Error::UiStaleRequest.info())
            }
            Target::Order { .. }
                if engine
                    .state()
                    .ui_file
                    .request
                    .as_ref()
                    .is_some_and(|request| request.phase != UiFilePhase::Running)
                    || !engine
                        .stored_document()
                        .is_ok_and(|stored| stored.has_smiles_projection()) =>
            {
                Err(Error::UiStaleRequest.info())
            }
            _ => Ok(()),
        }
    }
}

/// Proof of the actual post-finish native context; neither it nor its result is a Save ACK.
pub struct UiSmilesRecordsResult {
    pub result: CommandResult,
    state: NativeState,
}

impl UiSmilesRecordsResult {
    pub fn is_current(&self, engine: &Engine) -> bool {
        self.state.validate(engine).is_ok()
            && !engine
                .state()
                .ui_file
                .request
                .as_ref()
                .is_some_and(|request| request.phase != UiFilePhase::Running)
    }
}

#[must_use = "complete or drop registered record work to release its reservation"]
pub struct UiSmilesRecordsWork {
    engine: SharedEngine,
    registered: RegisteredWork,
    context: UiSmilesRecordsContext,
}

impl UiSmilesRecordsWork {
    /// Parse and layout off-lock; validate, finish and seal presentation under one lock.
    pub fn complete_blocking(
        self,
        cancel: &Cancellation,
    ) -> Result<UiSmilesRecordsResult, ErrorInfo> {
        let Self {
            engine,
            registered,
            context,
        } = self;
        let RegisteredWork {
            prepared,
            permit: _permit,
        } = registered;
        let completed = prepared
            .complete_with_cancel(cancel)
            .map_err(|error| error.info())?;
        let mut engine = engine.lock().map_err(|_| unavailable())?;
        context.validate(&engine)?;
        let result = engine.finish(completed).map_err(|error| error.info())?;
        let state = NativeState::capture(&engine);
        Ok(UiSmilesRecordsResult { result, state })
    }
}

impl CommandExecutor {
    pub fn prepare_ui_smiles_import(
        &self,
        context: &UiSmilesRecordsContext,
        params: ImportSmilesRecords,
    ) -> Result<UiSmilesRecordsWork, ErrorInfo> {
        let mut engine = self.engine.lock().map_err(|_| unavailable())?;
        context.validate(&engine)?;
        let Target::Import { request_id } = context.target else {
            return Err(
                Error::InvalidUiFileParameters("native import requires an import context").info(),
            );
        };
        if params.expected_revision != engine.state().revision {
            return Err(Error::StaleRevision.info());
        }
        let permit = self.workers.permit()?;
        let prepared = engine
            .prepare_ui_smiles_import(request_id, params)
            .map_err(|error| error.info())?;
        Ok(UiSmilesRecordsWork {
            engine: Arc::clone(&self.engine),
            context: context.clone(),
            registered: RegisteredWork {
                prepared,
                permit: Some(permit),
            },
        })
    }

    pub fn prepare_ui_smiles_order(
        &self,
        context: &UiSmilesRecordsContext,
        params: BondSetOrder,
    ) -> Result<UiSmilesRecordsWork, ErrorInfo> {
        let mut engine = self.engine.lock().map_err(|_| unavailable())?;
        context.validate(&engine)?;
        let Target::Order {
            page_id,
            fragment_id,
            bond_id,
        } = context.target
        else {
            return Err(Error::InvalidUiFileParameters(
                "native order work requires a bond context",
            )
            .info());
        };
        if params.expected_revision != engine.state().revision {
            return Err(Error::StaleRevision.info());
        }
        if (params.page_id, params.fragment_id, params.bond_id) != (page_id, fragment_id, bond_id) {
            return Err(Error::InvalidUiFileParameters(
                "bond differs from the native gesture target",
            )
            .info());
        }
        let permit = self.workers.permit()?;
        let prepared = engine
            .prepare(Command::BondSetOrder(params))
            .map_err(|error| error.info())?;
        Ok(UiSmilesRecordsWork {
            engine: Arc::clone(&self.engine),
            context: context.clone(),
            registered: RegisteredWork {
                prepared,
                permit: Some(permit),
            },
        })
    }
}
