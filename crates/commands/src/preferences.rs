//! Session preferences and one weak native draft, separate from authored content.
use crate::{
    BondToolState, CaptureStamp, CloseState, CommandResult, Engine, Error, UiFileContextStamp,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(extend("type" = "object", "unevaluatedProperties" = false))]
pub enum UiPreferences {
    Inspect {},
    Set {
        expected_epoch: u64,
        sprout_rings_when_clicking: bool,
    },
    Open {
        expected_revision: u64,
    },
    Apply {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        request_id: u64,
        sprout_rings_when_clicking: bool,
    },
    Cancel {
        expected_revision: u64,
        #[schemars(range(min = 1))]
        request_id: u64,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ApplicationPreferences {
    pub sprout_rings_when_clicking: bool,
    pub epoch: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct UiPreferencesRequest {
    pub id: u64,
    pub input_revision: u64,
    pub sprout_rings_when_clicking: bool,
    pub current: bool,
    #[serde(skip)]
    cancelled: crate::Cancellation,
}

impl UiPreferencesRequest {
    /// Only explicit matching Cancel signals this token, including after retirement.
    pub fn was_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct UiPreferencesOperation {
    pub action: &'static str,
    pub changed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u64>,
}

#[derive(Debug)]
struct Request {
    view: UiPreferencesRequest,
    source: CaptureStamp,
    file_context: UiFileContextStamp,
    file_epoch: u64,
    close_epoch: u64,
    view_epoch: u64,
    tool: BondToolState,
    mode: crate::EffectiveTool,
    preference_epoch: u64,
}

#[derive(Debug, Default)]
pub(super) struct PreferencesPolicy {
    value: ApplicationPreferences,
    open_epoch: u64,
    active: Option<Request>,
}

impl PreferencesPolicy {
    pub(super) fn has_request(&self) -> bool {
        self.active.is_some()
    }
}

impl Engine {
    pub fn preferences(&self) -> ApplicationPreferences {
        self.preferences_ui.value
    }

    /// Monotonic modal-open identity, distinct from changes to the preference value.
    pub fn ui_preferences_epoch(&self) -> u64 {
        self.preferences_ui.open_epoch
    }

    pub fn ui_preferences_request(&self) -> Option<UiPreferencesRequest> {
        let request = self.preferences_ui.active.as_ref()?;
        let mut view = request.view.clone();
        view.current = self.ui_preferences_request_current(view.id);
        Some(view)
    }

    pub fn ui_preferences_request_current(&self, id: u64) -> bool {
        self.preferences_ui.active.as_ref().is_some_and(|request| {
            request.view.id == id
                && self.preferences_foreground_idle()
                && self.capture_stamp_matches(&request.source)
                && self.ui_file_context_stamp_matches(&request.file_context)
                && self.file_ui.epoch == request.file_epoch
                && self.close_policy.epoch == request.close_epoch
                && self.view().epoch == request.view_epoch
                && self.bond_tool() == request.tool
                && self.effective_tool() == request.mode
                && self.preferences().epoch == request.preference_epoch
        })
    }

    /// Retire only this host draft, even after committed close; no Save/close ACK.
    pub fn finish_ui_preferences(&mut self, id: u64) -> bool {
        if self
            .preferences_ui
            .active
            .as_ref()
            .is_some_and(|request| request.view.id == id)
        {
            self.preferences_ui.active = None;
            true
        } else {
            false
        }
    }

    pub(super) fn detach_preferences(&mut self) {
        self.preferences_ui.active = None;
    }

    pub(super) fn ui_preferences(&mut self, params: UiPreferences) -> Result<CommandResult, Error> {
        match params {
            UiPreferences::Inspect {} => Ok(self.preferences_result(None)),
            UiPreferences::Set {
                expected_epoch,
                sprout_rings_when_clicking,
            } => {
                let changed =
                    self.set_application_preferences(expected_epoch, sprout_rings_when_clicking)?;
                Ok(
                    self.preferences_result(changed.then_some(UiPreferencesOperation {
                        action: "set",
                        changed,
                        request_id: None,
                    })),
                )
            }
            UiPreferences::Open { expected_revision } => {
                self.require_preferences_presentation(expected_revision)?;
                if self.preferences_ui.has_request() || !self.preferences_foreground_idle() {
                    return Err(Error::UiBusy);
                }
                let id = self
                    .preferences_ui
                    .open_epoch
                    .checked_add(1)
                    .ok_or(Error::UiEpochExhausted)?;
                let request = Request {
                    view: UiPreferencesRequest {
                        id,
                        input_revision: self.revision,
                        sprout_rings_when_clicking: self.preferences().sprout_rings_when_clicking,
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
                    preference_epoch: self.preferences().epoch,
                };
                self.preferences_ui.open_epoch = id;
                self.preferences_ui.active = Some(request);
                Ok(self.preferences_result(Some(UiPreferencesOperation {
                    action: "open",
                    changed: false,
                    request_id: Some(id),
                })))
            }
            UiPreferences::Apply {
                expected_revision,
                request_id,
                sprout_rings_when_clicking,
            } => {
                self.require_preferences_presentation(expected_revision)?;
                if !self.ui_preferences_request_current(request_id) {
                    return Err(Error::UiStaleRequest);
                }
                let expected_epoch = self
                    .preferences_ui
                    .active
                    .as_ref()
                    .ok_or(Error::UiStaleRequest)?
                    .preference_epoch;
                let changed =
                    self.set_application_preferences(expected_epoch, sprout_rings_when_clicking)?;
                self.finish_ui_preferences(request_id);
                Ok(self.preferences_result(Some(UiPreferencesOperation {
                    action: "apply",
                    changed,
                    request_id: Some(request_id),
                })))
            }
            UiPreferences::Cancel {
                expected_revision,
                request_id,
            } => {
                self.require_preferences_presentation(expected_revision)?;
                let request = self
                    .preferences_ui
                    .active
                    .as_ref()
                    .filter(|request| request.view.id == request_id)
                    .ok_or(Error::UiStaleRequest)?;
                request.view.cancelled.cancel();
                self.finish_ui_preferences(request_id);
                Ok(self.preferences_result(Some(UiPreferencesOperation {
                    action: "cancel",
                    changed: false,
                    request_id: Some(request_id),
                })))
            }
        }
    }

    fn require_preferences_presentation(&self, expected_revision: u64) -> Result<(), Error> {
        if expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if !self.file_ui_state().available {
            return Err(Error::UiUnavailable);
        }
        Ok(())
    }

    // Raw ownership only: never call state() or another qualified modal getter here.
    fn preferences_foreground_idle(&self) -> bool {
        let file = self.file_ui_state();
        file.available
            && file.request.is_none()
            && self.close_state() == CloseState::Open
            && !self.atom_ui.has_request()
            && !self.drawing_settings_ui.has_request()
            && !self.chain_ui.has_request()
    }

    fn set_application_preferences(
        &mut self,
        expected_epoch: u64,
        sprout_rings_when_clicking: bool,
    ) -> Result<bool, Error> {
        let old = self.preferences();
        if expected_epoch != old.epoch {
            return Err(Error::UiStaleRequest);
        }
        if old.sprout_rings_when_clicking == sprout_rings_when_clicking {
            return Ok(false);
        }
        let epoch = old.epoch.checked_add(1).ok_or(Error::UiEpochExhausted)?;
        let tool_epoch = self
            .bond_tool
            .epoch
            .checked_add(1)
            .ok_or(Error::UiBondToolEpochExhausted)?;
        self.preferences_ui.value = ApplicationPreferences {
            sprout_rings_when_clicking,
            epoch,
        };
        self.bond_tool.epoch = tool_epoch;
        Ok(true)
    }

    fn preferences_result(&self, operation: Option<UiPreferencesOperation>) -> CommandResult {
        CommandResult {
            preferences: Some(self.preferences()),
            ui_preferences_operation: operation,
            ..self.state()
        }
    }
}
