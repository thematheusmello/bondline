//! Native presentation owns a weak draft; the existing settings command authors.
use crate::{
    BondToolState, CaptureStamp, CloseState, Command, CommandResult, DocumentSettingsSet,
    DrawingSettingsSet, EffectiveTool, Engine, Error, UiFileContextStamp,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum UiDrawingSettings {
    Open {
        expected_revision: u64,
    },
    Cancel {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        request_id: u64,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct UiDrawingSettingsRequest {
    pub id: u64,
    pub input_revision: u64,
    pub bond_length_pt: Option<f64>,
    pub chain_angle_degrees: Option<f64>,
    pub line_width_pt: Option<f64>,
    pub bond_spacing_percent: Option<f64>,
    pub current: bool,
    #[serde(skip)]
    cancelled: crate::Cancellation,
}

impl UiDrawingSettingsRequest {
    /// Explicit cancellation survives retirement without treating context loss as Cancel.
    pub fn was_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }
}

#[derive(Debug)]
struct Request {
    view: UiDrawingSettingsRequest,
    source: CaptureStamp,
    file_context: UiFileContextStamp,
    file_epoch: u64,
    close_epoch: u64,
    view_epoch: u64,
    tool: BondToolState,
    mode: EffectiveTool,
}

#[derive(Debug, Default)]
pub(super) struct DrawingSettingsUiPolicy {
    epoch: u64,
    active: Option<Request>,
}

impl DrawingSettingsUiPolicy {
    pub(super) fn has_request(&self) -> bool {
        self.active.is_some()
    }

    pub(super) fn epoch(&self) -> u64 {
        self.epoch
    }
}

impl Engine {
    pub fn drawing_settings_available(&self) -> bool {
        self.stored_document()
            .is_ok_and(|document| crate::drawing_settings::require_source(document).is_ok())
    }

    pub fn ui_drawing_settings_request(&self) -> Option<UiDrawingSettingsRequest> {
        let request = self.drawing_settings_ui.active.as_ref()?;
        let mut view = request.view.clone();
        view.current = self.ui_drawing_settings_request_current(view.id);
        Some(view)
    }

    pub fn ui_drawing_settings_request_current(&self, id: u64) -> bool {
        self.drawing_settings_ui
            .active
            .as_ref()
            .is_some_and(|request| {
                request.view.id == id
                    && self.file_ui_state().available
                    && self.capture_stamp_matches(&request.source)
                    && self.ui_file_context_stamp_matches(&request.file_context)
                    && self.file_ui.epoch == request.file_epoch
                    && self.file_ui_state().request.is_none()
                    && !self.preferences_ui.has_request()
                    && self.ui_atom_label_request().is_none()
                    && self.close_state() == CloseState::Open
                    && self.close_policy.epoch == request.close_epoch
                    && self.view().epoch == request.view_epoch
                    && self.bond_tool() == request.tool
                    && self.effective_tool() == request.mode
            })
    }

    /// Retire only the matching draft; this never acknowledges Save or close.
    pub fn finish_ui_drawing_settings(&mut self, id: u64) -> bool {
        if self
            .drawing_settings_ui
            .active
            .as_ref()
            .is_some_and(|r| r.view.id == id)
        {
            self.drawing_settings_ui.active = None;
            true
        } else {
            false
        }
    }

    pub(super) fn detach_drawing_settings(&mut self) {
        self.drawing_settings_ui.active = None;
    }

    pub(super) fn ui_drawing_settings(
        &mut self,
        params: UiDrawingSettings,
    ) -> Result<CommandResult, Error> {
        let expected = match params {
            UiDrawingSettings::Open { expected_revision }
            | UiDrawingSettings::Cancel {
                expected_revision, ..
            } => expected_revision,
        };
        if expected != self.revision {
            return Err(Error::StaleRevision);
        }
        if !self.file_ui_state().available {
            return Err(Error::UiUnavailable);
        }
        let id = match params {
            UiDrawingSettings::Open { .. } => {
                if self.drawing_settings_ui.active.is_some()
                    || self.preferences_ui.has_request()
                    || self.ui_atom_label_request().is_some()
                    || self.ui_add_chain_request().is_some()
                    || self.file_ui_state().request.is_some()
                    || self.close_state() != CloseState::Open
                {
                    return Err(Error::UiBusy);
                }
                let stored = self.stored_document()?;
                crate::drawing_settings::require_source(stored)?;
                let settings = stored.model().drawing_settings();
                let id = self
                    .drawing_settings_ui
                    .epoch
                    .checked_add(1)
                    .ok_or(Error::UiEpochExhausted)?;
                let request = Request {
                    view: UiDrawingSettingsRequest {
                        id,
                        input_revision: self.revision,
                        bond_length_pt: settings.bond_length_pt(),
                        chain_angle_degrees: settings.chain_angle_degrees(),
                        line_width_pt: stored.model().drawing_style().line_width_pt(),
                        bond_spacing_percent: stored.model().drawing_style().bond_spacing_percent(),
                        current: true,
                        cancelled: crate::Cancellation::default(),
                    },
                    source: self.capture_stamp(),
                    file_context: self.ui_file_context_stamp(),
                    file_epoch: self.file_ui.epoch,
                    close_epoch: self.close_policy.epoch,
                    view_epoch: self.view().epoch,
                    tool: self.bond_tool(),
                    mode: self.effective_tool(),
                };
                self.drawing_settings_ui.epoch = id;
                self.drawing_settings_ui.active = Some(request);
                id
            }
            UiDrawingSettings::Cancel { request_id, .. } => {
                let request = self
                    .drawing_settings_ui
                    .active
                    .as_ref()
                    .filter(|request| request.view.id == request_id)
                    .ok_or(Error::UiStaleRequest)?;
                request.view.cancelled.cancel();
                if !self.finish_ui_drawing_settings(request_id) {
                    return Err(Error::UiStaleRequest);
                }
                request_id
            }
        };
        Ok(CommandResult {
            ui_drawing_settings_operation: Some(id),
            ..self.state()
        })
    }

    /// Revalidate and author once while the native host holds the same Engine guard.
    pub fn apply_ui_drawing_settings(
        &mut self,
        id: u64,
        bond_length_pt: Option<f64>,
        chain_angle_degrees: Option<f64>,
    ) -> Result<CommandResult, Error> {
        if !self.ui_drawing_settings_request_current(id) {
            return Err(Error::UiStaleRequest);
        }
        let mut result = self.execute(Command::DrawingSettingsSet(DrawingSettingsSet {
            expected_revision: self.revision,
            bond_length_pt,
            chain_angle_degrees,
        }))?;
        self.finish_ui_drawing_settings(id);
        result.ui_drawing_settings = None;
        result.ui_drawing_settings_operation = Some(id);
        Ok(result)
    }

    /// The same current draft, committing the full settings form through one registry entry.
    pub fn apply_ui_document_settings(
        &mut self,
        id: u64,
        bond_length_pt: Option<f64>,
        chain_angle_degrees: Option<f64>,
        line_width_pt: Option<f64>,
    ) -> Result<CommandResult, Error> {
        if !self.ui_drawing_settings_request_current(id) {
            return Err(Error::UiStaleRequest);
        }
        let mut result = self.execute(Command::DocumentSettingsSet(DocumentSettingsSet {
            expected_revision: self.revision,
            bond_length_pt,
            chain_angle_degrees,
            line_width_pt,
            bond_spacing_percent: None,
        }))?;
        self.finish_ui_drawing_settings(id);
        result.ui_drawing_settings = None;
        result.ui_drawing_settings_operation = Some(id);
        Ok(result)
    }

    /// Apply the complete form once, preserving all weak presentation guards.
    pub fn apply_ui_document_settings_with_spacing(
        &mut self,
        id: u64,
        bond_length_pt: Option<f64>,
        chain_angle_degrees: Option<f64>,
        line_width_pt: Option<f64>,
        bond_spacing_percent: Option<f64>,
    ) -> Result<CommandResult, Error> {
        if !self.ui_drawing_settings_request_current(id) {
            return Err(Error::UiStaleRequest);
        }
        let mut result = self.execute(Command::DocumentSettingsSet(DocumentSettingsSet {
            expected_revision: self.revision,
            bond_length_pt,
            chain_angle_degrees,
            line_width_pt,
            bond_spacing_percent: Some(bond_spacing_percent),
        }))?;
        self.finish_ui_drawing_settings(id);
        result.ui_drawing_settings = None;
        result.ui_drawing_settings_operation = Some(id);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_ids_are_checked_and_only_exact_explicit_cancel_marks_its_token()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.ui_drawing_settings(UiDrawingSettings::Open {
            expected_revision: 0,
        })?;
        let noop = engine.ui_drawing_settings_request().ok_or("no-op draft")?;
        engine.apply_ui_drawing_settings(noop.id, None, None)?;
        assert!(!noop.was_cancelled());
        engine.ui_drawing_settings(UiDrawingSettings::Open {
            expected_revision: 0,
        })?;
        let cancelled = engine.ui_drawing_settings_request().ok_or("cancel draft")?;
        assert_eq!(cancelled.id, 2);
        engine.ui_drawing_settings(UiDrawingSettings::Cancel {
            expected_revision: 0,
            request_id: cancelled.id,
        })?;
        assert!(cancelled.was_cancelled());
        assert!(!noop.was_cancelled());
        engine.ui_drawing_settings(UiDrawingSettings::Open {
            expected_revision: 0,
        })?;
        let detached = engine
            .ui_drawing_settings_request()
            .ok_or("detached draft")?;
        engine.set_ui_file_host(false);
        engine.set_ui_file_host(true);
        assert!(!detached.was_cancelled());
        engine.drawing_settings_ui.epoch = u64::MAX - 1;
        engine.ui_drawing_settings(UiDrawingSettings::Open {
            expected_revision: 0,
        })?;
        let final_request = engine.ui_drawing_settings_request().ok_or("last draft")?;
        assert_eq!(final_request.id, u64::MAX);
        assert!(matches!(
            engine.ui_drawing_settings(UiDrawingSettings::Cancel {
                expected_revision: 0,
                request_id: detached.id,
            }),
            Err(Error::UiStaleRequest)
        ));
        assert!(!final_request.was_cancelled());
        assert!(engine.finish_ui_drawing_settings(final_request.id));
        assert!(!final_request.was_cancelled());
        let before = serde_json::to_value(engine.state())?;
        let source = engine.capture_stamp();
        assert!(matches!(
            engine.ui_drawing_settings(UiDrawingSettings::Open {
                expected_revision: 0,
            }),
            Err(Error::UiEpochExhausted)
        ));
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert!(engine.capture_stamp_matches(&source));
        Ok(())
    }
}
