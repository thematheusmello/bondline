//! Transient caption tool and draft. Authored edits go through text.set_run.
use crate::{
    Cancellation, CaptureStamp, CloseState, Command, CommandResult, EffectiveTool, Engine, Error,
    TextSetRun, UiFileContextStamp, caption,
};
use bondline_model::{PageId, TextId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Four bytes per scalar at the existing 4096-character page scene limit.
pub const MAX_CAPTION_DRAFT_BYTES: usize = 16_384;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum UiCaption {
    Select {},
    Open {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        page_id: u32,
        #[schemars(range(min = 1))]
        text_id: u32,
        run_index: usize,
    },
    Apply {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        request_id: u64,
        /// Replacement UTF-8, at most 16384 bytes.
        text: String,
    },
    Cancel {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        request_id: u64,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct UiCaptionRequest {
    pub id: u64,
    pub input_revision: u64,
    pub page_id: u32,
    pub text_id: u32,
    pub run_index: usize,
    pub run_count: usize,
    pub text: String,
    pub current: bool,
    #[serde(skip)]
    cancelled: Cancellation,
}

impl UiCaptionRequest {
    pub fn was_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }
}

/// The native host separately checks hit, layer, dimensions and pointer chronology.
#[derive(Debug)]
pub struct CaptionContext {
    source: CaptureStamp,
    file_context: UiFileContextStamp,
    file_epoch: u64,
    close_epoch: u64,
    view_epoch: u64,
    tool_epoch: u64,
    caption_epoch: u64,
    other_modal_epochs: [u64; 4],
}

impl CaptionContext {
    fn matches(&self, engine: &Engine) -> bool {
        engine.caption_foreground().is_ok()
            && engine.effective_tool() == EffectiveTool::Caption
            && engine.bond_tool().epoch == self.tool_epoch
            && engine.view().epoch == self.view_epoch
            && engine.close_policy.epoch == self.close_epoch
            && engine.file_ui.epoch == self.file_epoch
            && engine.caption_ui.epoch == self.caption_epoch
            && engine.other_caption_modal_epochs() == self.other_modal_epochs
            && engine.capture_stamp_matches(&self.source)
            && engine.ui_file_context_stamp_matches(&self.file_context)
    }

    pub fn is_current(&self, engine: &Engine) -> bool {
        !engine.caption_ui.has_request() && self.matches(engine)
    }
}

#[derive(Debug)]
struct Request {
    view: UiCaptionRequest,
    context: CaptionContext,
}

#[derive(Debug, Default)]
pub(super) struct CaptionUiPolicy {
    epoch: u64,
    active: Option<Request>,
}

impl CaptionUiPolicy {
    pub(super) fn has_request(&self) -> bool {
        self.active.is_some()
    }
}

impl Engine {
    pub fn caption_available(&self) -> bool {
        self.stored_document().is_ok_and(|stored| {
            stored
                .model()
                .pages()
                .first()
                .is_some_and(|page| !page.texts().is_empty())
                && caption::supports_source(stored)
        })
    }

    fn caption_foreground(&self) -> Result<(), Error> {
        let file = self.file_ui_state();
        if !file.available {
            return Err(Error::UiUnavailable);
        }
        // Raw ownership prevents recursion through state() and qualified modal getters.
        if file.request.is_some()
            || self.close_state() != CloseState::Open
            || self.atom_ui.has_request()
            || self.drawing_settings_ui.has_request()
            || self.chain_ui.has_request()
            || self.preferences_ui.has_request()
        {
            return Err(Error::UiBusy);
        }
        Ok(())
    }

    fn other_caption_modal_epochs(&self) -> [u64; 4] {
        [
            self.atom_ui.epoch(),
            self.drawing_settings_ui.epoch(),
            self.chain_ui.epoch(),
            self.ui_preferences_epoch(),
        ]
    }

    pub fn caption_context(&self) -> Result<CaptionContext, Error> {
        self.caption_foreground()?;
        if self.caption_ui.has_request() {
            return Err(Error::UiBusy);
        }
        if self.effective_tool() != EffectiveTool::Caption {
            return Err(Error::CaptionToolRequired);
        }
        if !self.caption_available() {
            return Err(Error::UnsupportedCaptionSource);
        }
        Ok(CaptionContext {
            source: self.capture_stamp(),
            file_context: self.ui_file_context_stamp(),
            file_epoch: self.file_ui.epoch,
            close_epoch: self.close_policy.epoch,
            view_epoch: self.view().epoch,
            tool_epoch: self.bond_tool().epoch,
            caption_epoch: self.caption_ui.epoch,
            other_modal_epochs: self.other_caption_modal_epochs(),
        })
    }

    pub fn ui_caption_request(&self) -> Option<UiCaptionRequest> {
        let active = self.caption_ui.active.as_ref()?;
        let mut view = active.view.clone();
        view.current = active.context.matches(self);
        Some(view)
    }

    pub fn ui_caption_request_current(&self, id: u64) -> bool {
        self.caption_ui
            .active
            .as_ref()
            .is_some_and(|request| request.view.id == id && request.context.matches(self))
    }

    /// Retire only this presentation, including on the committed close path.
    pub fn finish_ui_caption(&mut self, id: u64) -> bool {
        if self.caption_ui.active.as_ref().map(|r| r.view.id) == Some(id) {
            self.caption_ui.active = None;
            true
        } else {
            false
        }
    }

    pub(super) fn detach_caption(&mut self) {
        self.caption_ui.active = None;
    }

    pub(super) fn ui_caption(&mut self, params: UiCaption) -> Result<CommandResult, Error> {
        if !self.file_ui_state().available {
            return Err(Error::UiUnavailable);
        }
        let id = match params {
            UiCaption::Select {} => {
                self.caption_foreground()?;
                if self.caption_ui.has_request() {
                    return Err(Error::UiBusy);
                }
                if !self.caption_available() {
                    return Err(Error::UnsupportedCaptionSource);
                }
                if self.effective_tool() != EffectiveTool::Caption {
                    let selection_epoch = self.selection_epoch_for_bond_tool()?;
                    let epoch = self
                        .bond_tool
                        .epoch
                        .checked_add(1)
                        .ok_or(Error::UiBondToolEpochExhausted)?;
                    self.deactivate_selection(selection_epoch);
                    self.bond_tool.epoch = epoch;
                    self.effective_tool = EffectiveTool::Caption;
                }
                self.bond_tool.epoch
            }
            UiCaption::Open {
                expected_revision,
                page_id,
                text_id,
                run_index,
            } => {
                if expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let mut context = self.caption_context()?;
                let caption = caption::find(
                    self.stored_document()?,
                    PageId::new(page_id).ok_or(Error::InvalidPageId)?,
                    TextId::new(text_id).ok_or(Error::InvalidTextId)?,
                )?;
                let run = caption
                    .runs()
                    .get(run_index)
                    .ok_or(bondline_model::Error::Text(
                        bondline_model::TextError::MissingRun(run_index),
                    ))?;
                if run.text().len() > MAX_CAPTION_DRAFT_BYTES {
                    return Err(Error::CaptionDraftLimit);
                }
                let id = self
                    .caption_ui
                    .epoch
                    .checked_add(1)
                    .ok_or(Error::UiEpochExhausted)?;
                context.caption_epoch = id;
                self.caption_ui = CaptionUiPolicy {
                    epoch: id,
                    active: Some(Request {
                        view: UiCaptionRequest {
                            id,
                            input_revision: expected_revision,
                            page_id,
                            text_id,
                            run_index,
                            run_count: caption.runs().len(),
                            text: run.text().to_owned(),
                            current: true,
                            cancelled: Cancellation::default(),
                        },
                        context,
                    }),
                };
                id
            }
            UiCaption::Apply {
                expected_revision,
                request_id,
                text,
            } => {
                if expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let active = self
                    .caption_ui
                    .active
                    .as_ref()
                    .filter(|active| active.view.id == request_id && active.context.matches(self))
                    .ok_or(Error::UiStaleRequest)?;
                if text.len() > MAX_CAPTION_DRAFT_BYTES {
                    return Err(Error::CaptionDraftLimit);
                }
                let result = self.execute(Command::TextSetRun(TextSetRun {
                    expected_revision,
                    page_id: active.view.page_id,
                    text_id: active.view.text_id,
                    run_index: active.view.run_index,
                    text,
                }))?;
                self.finish_ui_caption(request_id);
                return Ok(CommandResult {
                    ui_caption: None,
                    ui_caption_operation: Some(request_id),
                    ..result
                });
            }
            UiCaption::Cancel {
                expected_revision,
                request_id,
            } => {
                if expected_revision != self.revision {
                    return Err(Error::StaleRevision);
                }
                let active = self
                    .caption_ui
                    .active
                    .as_ref()
                    .filter(|active| active.view.id == request_id)
                    .ok_or(Error::UiStaleRequest)?;
                active.view.cancelled.cancel();
                self.finish_ui_caption(request_id);
                request_id
            }
        };
        Ok(CommandResult {
            ui_caption_operation: Some(id),
            ..self.state()
        })
    }
}
