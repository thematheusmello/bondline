//! Typed native bindings. Metadata and actual command invocation share one registry.

use crate::{
    BondDrawOrder, CloseAction, CloseDocument, CloseState, Command, Engine, NoParams, PageSetSize,
    SelectBondTool, UiFile, UiFileAction, UiFilePhase, UiView,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlKey {
    N,
    O,
    S,
    W,
    Z,
    Y,
    F5,
    F6,
    F7,
    F8,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
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
        let mut controls = Vec::with_capacity(18);
        for (id, label, key, shift, action) in [
            (
                "file.new",
                "New Document",
                ControlKey::N,
                false,
                UiFileAction::New,
            ),
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
                selected: None,
            });
        }
        controls.push(ControlInfo {
            id: "file.convert_smiles",
            label: "Convert SMILES file…",
            menu: "File",
            shortcut: None,
            command: Command::UiFile(UiFile {
                expected_revision: state.revision,
                action: UiFileAction::ConvertSmiles,
                request_id: None,
                close_epoch: None,
            }),
            enabled: file_reason.is_none(),
            disabled_reason: file_reason,
            selected: None,
        });
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
            selected: None,
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
                selected: None,
            });
        }
        if let Ok(document) = self.document()
            && let Some(page) = document.pages().first()
        {
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
                    selected: None,
                });
            }
        }
        let stored = self.stored_document().ok();
        if matches!(self.content(), crate::StoredContent::Sdf { .. })
            || stored.is_some_and(|stored| !stored.model().pages().is_empty())
        {
            let [width_px, height_px] = self.view_canvas_size.unwrap_or([1, 1]);
            let percent = self.view_percent();
            let source_reason = authoring_reason.or(stored
                .is_some_and(|stored| !stored.is_fully_mapped())
                .then_some("unsupported_content"));
            for (id, label, key, action, selected, limit) in [
                (
                    "view.actual",
                    "Actual size",
                    ControlKey::F5,
                    UiView::Actual {},
                    Some(self.view.zoom_percent == Some(100.0)),
                    false,
                ),
                (
                    "view.fit",
                    "Fit to window",
                    ControlKey::F6,
                    UiView::Fit {},
                    Some(self.view.zoom_percent.is_none()),
                    false,
                ),
                (
                    "view.magnify",
                    "Magnify",
                    ControlKey::F7,
                    UiView::Magnify {
                        width_px,
                        height_px,
                    },
                    None,
                    percent.is_some_and(|p| p >= 999.0),
                ),
                (
                    "view.reduce",
                    "Reduce",
                    ControlKey::F8,
                    UiView::Reduce {
                        width_px,
                        height_px,
                    },
                    None,
                    percent.is_some_and(|p| p <= 1.0),
                ),
            ] {
                let relative = matches!(action, UiView::Magnify { .. } | UiView::Reduce { .. });
                let reason = source_reason
                    .or(
                        (relative && (self.view_canvas_size.is_none() || percent.is_none()))
                            .then_some("viewport_unavailable"),
                    )
                    .or(limit.then_some("zoom_limit"));
                controls.push(ControlInfo {
                    id,
                    label,
                    menu: "View",
                    command: Command::UiView(action),
                    shortcut: Some(ControlShortcut {
                        key,
                        primary: false,
                        shift: false,
                    }),
                    enabled: reason.is_none(),
                    disabled_reason: reason,
                    selected,
                });
            }
        }
        if let Ok(stored) = self.stored_document() {
            let reason =
                authoring_reason.or((!stored.is_fully_mapped()).then_some("unsupported_content"));
            for (id, label, order) in [
                ("tool.bond.single", "Single bond", BondDrawOrder::Single),
                ("tool.bond.double", "Double bond", BondDrawOrder::Double),
                ("tool.bond.triple", "Triple bond", BondDrawOrder::Triple),
            ] {
                controls.push(ControlInfo {
                    id,
                    label,
                    menu: "Tools",
                    shortcut: None,
                    command: Command::UiBondToolSelect(SelectBondTool { order }),
                    enabled: reason.is_none(),
                    disabled_reason: reason,
                    selected: Some(self.bond_tool.order == order),
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
                selected: None,
            });
        }
        controls
    }
}
