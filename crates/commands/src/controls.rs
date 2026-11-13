//! Typed native bindings. Metadata and actual command invocation share one registry.

use crate::{
    CloseAction, CloseDocument, CloseState, Command, Engine, NoParams, PageSetSize, UiFile,
    UiFileAction, UiFilePhase,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlKey {
    O,
    S,
    W,
    Z,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ControlShortcut {
    pub key: ControlKey,
    pub primary: bool,
    pub shift: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ControlInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub menu: &'static str,
    pub shortcut: Option<ControlShortcut>,
    pub command: Command,
    pub enabled: bool,
    pub disabled_reason: Option<&'static str>,
}

impl Engine {
    /// Discover only delivered bindings; this is not the complete reference inventory.
    pub fn controls(&self) -> Vec<ControlInfo> {
        let state = self.state();
        let pending = state
            .ui_file
            .request
            .as_ref()
            .is_some_and(|request| request.phase != UiFilePhase::Running);
        let authoring_reason = if state.close != CloseState::Open {
            Some("close_pending")
        } else if pending {
            Some("ui_busy")
        } else {
            None
        };
        let file_reason = if !state.ui_file.available {
            Some("ui_unavailable")
        } else if state.ui_file.request.is_some() {
            Some("ui_busy")
        } else {
            authoring_reason
        };
        let shortcut = |key, shift| {
            Some(ControlShortcut {
                key,
                primary: true,
                shift,
            })
        };
        let mut controls = Vec::with_capacity(10);
        for (id, label, key, shift, action) in [
            (
                "file.open",
                "Open…",
                ControlKey::O,
                false,
                UiFileAction::Open,
            ),
            (
                "file.save",
                "Save",
                ControlKey::S,
                false,
                UiFileAction::Save,
            ),
            (
                "file.save_as",
                "Save As…",
                ControlKey::S,
                true,
                UiFileAction::SaveAs,
            ),
        ] {
            controls.push(ControlInfo {
                id,
                label,
                menu: "File",
                shortcut: shortcut(key, shift),
                command: Command::UiFile(UiFile {
                    expected_revision: state.revision,
                    action,
                    request_id: None,
                    close_epoch: None,
                }),
                enabled: file_reason.is_none(),
                disabled_reason: file_reason,
            });
        }
        let close_reason = if state.ui_file.available {
            authoring_reason
        } else {
            Some("ui_unavailable")
        };
        controls.push(ControlInfo {
            id: "file.close",
            label: "Close Window",
            menu: "File",
            shortcut: shortcut(ControlKey::W, false),
            command: Command::Close(CloseDocument {
                expected_revision: state.revision,
                action: CloseAction::Request,
            }),
            enabled: close_reason.is_none(),
            disabled_reason: close_reason,
        });
        for (id, label, command, depth, shift) in [
            (
                "edit.undo",
                "Undo",
                Command::Undo(NoParams {}),
                state.undo_depth,
                false,
            ),
            (
                "edit.redo",
                "Redo",
                Command::Redo(NoParams {}),
                state.redo_depth,
                true,
            ),
        ] {
            let reason = authoring_reason.or((depth == 0).then_some("empty_history"));
            controls.push(ControlInfo {
                id,
                label,
                menu: "Edit",
                shortcut: shortcut(ControlKey::Z, shift),
                command,
                enabled: reason.is_none(),
                disabled_reason: reason,
            });
        }
        if let Some(page) = self.document().pages().first() {
            let size = page.size();
            for (id, label, width_pt, height_pt) in [
                (
                    "page.a4",
                    "A4 portrait",
                    210.0 * 72.0 / 25.4,
                    297.0 * 72.0 / 25.4,
                ),
                ("page.letter", "Letter portrait", 612.0, 792.0),
                (
                    "page.rotate",
                    "Rotate page",
                    size.height_pt(),
                    size.width_pt(),
                ),
            ] {
                controls.push(ControlInfo {
                    id,
                    label,
                    menu: "Page",
                    shortcut: None,
                    command: Command::PageSetSize(PageSetSize {
                        page_id: page.id().get(),
                        width_pt,
                        height_pt,
                    }),
                    enabled: authoring_reason.is_none(),
                    disabled_reason: authoring_reason,
                });
            }
        }
        if let Some(request) = state.ui_file.request {
            let reason = (request.phase == UiFilePhase::Running).then_some("ui_busy");
            controls.push(ControlInfo {
                id: "file.cancel",
                label: "Cancel",
                menu: "File dialog",
                shortcut: None,
                command: Command::UiFile(UiFile {
                    expected_revision: state.revision,
                    action: UiFileAction::Cancel,
                    request_id: Some(request.id),
                    close_epoch: None,
                }),
                enabled: reason.is_none(),
                disabled_reason: reason,
            });
        }
        controls
    }
}
