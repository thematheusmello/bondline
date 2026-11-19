//! Typed native bindings. Metadata and actual command invocation share one registry.

use crate::{
    BondDrawOrder, ChainKind, ChairKind, CloseAction, CloseDocument, CloseState, Command,
    EffectiveTool, Engine, NoParams, PageSetSize, RingKind, SelectBondTool, SelectChairRingTool,
    SelectNamedRingTool, SelectRingTool, UiCaption, UiChainTool, UiDrawingSettings, UiFile,
    UiFileAction, UiFilePhase, UiPreferences, UiSelection, UiView,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlKey {
    A,
    N,
    O,
    P,
    S,
    T,
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
    #[serde(skip_serializing_if = "is_false")]
    pub alt: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, Serialize)]
pub struct ControlInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub menu: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submenu: Option<&'static str>,
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
            .is_some_and(|request| request.phase != UiFilePhase::Running)
            || state.ui_atom_label.is_some()
            || state.ui_caption.is_some()
            || state.ui_drawing_settings.is_some()
            || state.ui_preferences.is_some()
            || state.ui_add_chain.is_some();
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
                alt: false,
            })
        };
        let mut controls = Vec::with_capacity(20);
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
                submenu: None,
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
        let settings_reason =
            file_reason.or((!self.drawing_settings_available()).then_some("unsupported_content"));
        controls.push(ControlInfo {
            id: "file.document_settings",
            label: "Document Settings…",
            menu: "File",
            submenu: None,
            shortcut: None,
            command: Command::UiDrawingSettings(UiDrawingSettings::Open {
                expected_revision: state.revision,
            }),
            enabled: settings_reason.is_none(),
            disabled_reason: settings_reason,
            selected: None,
        });
        controls.push(ControlInfo {
            id: "file.convert_smiles",
            label: "Convert SMILES file…",
            menu: "File",
            submenu: None,
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
        controls.push(ControlInfo {
            id: "file.preferences",
            label: "Preferences…",
            menu: "File",
            submenu: None,
            shortcut: None,
            command: Command::UiPreferences(UiPreferences::Open {
                expected_revision: state.revision,
            }),
            enabled: file_reason.is_none(),
            disabled_reason: file_reason,
            selected: None,
        });
        let import_reason = file_reason.or((!matches!(
            self.content(),
            crate::StoredContent::SmilesFile(_)
        ))
        .then_some("unsupported_content"));
        controls.push(ControlInfo {
            id: "file.import_smiles_records",
            label: "Import SMILES records…",
            menu: "File",
            submenu: None,
            shortcut: None,
            command: Command::UiFile(UiFile {
                expected_revision: state.revision,
                action: UiFileAction::ImportSmilesRecords,
                request_id: None,
                close_epoch: None,
            }),
            enabled: import_reason.is_none(),
            disabled_reason: import_reason,
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
            submenu: None,
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
                submenu: None,
                shortcut: shortcut(ControlKey::Z, shift),
                command,
                enabled: reason.is_none(),
                disabled_reason: reason,
                selected: None,
            });
        }
        let paste_reason = file_reason.or_else(|| {
            self.smiles_paste_source()
                .is_err()
                .then_some("unsupported_content")
        });
        controls.push(ControlInfo {
            id: "edit.paste_smiles",
            label: "SMILES…",
            menu: "Edit",
            submenu: Some("Paste Special"),
            // GUIDE25 establishes Ctrl+Alt+P on Windows; the Mac equivalent is unknown.
            shortcut: (!cfg!(target_os = "macos")).then_some(ControlShortcut {
                key: ControlKey::P,
                primary: true,
                shift: false,
                alt: true,
            }),
            command: Command::UiFile(UiFile {
                expected_revision: state.revision,
                action: UiFileAction::PasteSmiles,
                request_id: None,
                close_epoch: None,
            }),
            enabled: paste_reason.is_none(),
            disabled_reason: paste_reason,
            selected: None,
        });
        let stored = self.stored_document().ok();
        let page_reason = authoring_reason.or(stored
            .is_some_and(|stored| {
                stored
                    .source_info()
                    .is_some_and(|source| source.format == "smiles")
                    || stored.has_caption_projection()
                    || stored.model().has_authored_atom_labels()
            })
            .then_some("unsupported_content"));
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
                    submenu: None,
                    shortcut: None,
                    command: Command::PageSetSize(PageSetSize {
                        page_id: page.id().get(),
                        width_pt,
                        height_pt,
                    }),
                    enabled: page_reason.is_none(),
                    disabled_reason: page_reason,
                    selected: None,
                });
            }
        }
        if matches!(self.content(), crate::StoredContent::Sdf { .. })
            || stored.is_some_and(|stored| !stored.model().pages().is_empty())
        {
            let [width_px, height_px] = self.view_canvas_size.unwrap_or([1, 1]);
            let percent = self.view_percent();
            let source_reason = authoring_reason.or(stored
                .is_some_and(|stored| !stored.has_display_projection())
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
                    submenu: None,
                    command: Command::UiView(action),
                    shortcut: Some(ControlShortcut {
                        key,
                        primary: false,
                        shift: false,
                        alt: false,
                    }),
                    enabled: reason.is_none(),
                    disabled_reason: reason,
                    selected,
                });
            }
        }
        if let Ok(stored) = self.stored_document() {
            let reason = authoring_reason.or((!stored.is_fully_mapped()
                && !stored.has_smiles_projection())
            .then_some("unsupported_content"));
            for (id, label, order) in [
                ("tool.bond.single", "Single bond", BondDrawOrder::Single),
                ("tool.bond.double", "Double bond", BondDrawOrder::Double),
                ("tool.bond.triple", "Triple bond", BondDrawOrder::Triple),
            ] {
                controls.push(ControlInfo {
                    id,
                    label,
                    menu: "Tools",
                    submenu: None,
                    shortcut: None,
                    command: Command::UiBondToolSelect(SelectBondTool { order }),
                    enabled: reason.is_none(),
                    disabled_reason: reason,
                    selected: Some(
                        self.effective_tool() == EffectiveTool::Bond
                            && self.bond_tool.order == order,
                    ),
                });
            }
        }
        if (state.ui_file.available && self.caption_creation_available())
            || self
                .document()
                .ok()
                .and_then(|d| d.pages().first())
                .is_some_and(|p| !p.texts().is_empty())
        {
            let reason =
                file_reason.or((!self.caption_available()).then_some("unsupported_content"));
            controls.push(ControlInfo {
                id: "tool.caption",
                label: "Text",
                menu: "Tools",
                submenu: None,
                shortcut: Some(ControlShortcut {
                    key: ControlKey::T,
                    primary: false,
                    shift: false,
                    alt: false,
                }),
                command: Command::UiCaption(UiCaption::Select {}),
                enabled: reason.is_none(),
                disabled_reason: reason,
                selected: Some(self.effective_tool() == EffectiveTool::Caption),
            });
        }
        if self.selection_available() {
            controls.push(ControlInfo {
                id: "tool.selection",
                label: "Marquee selection",
                menu: "Tools",
                submenu: None,
                shortcut: None,
                command: Command::UiSelection(UiSelection::Activate {
                    expected_revision: state.revision,
                }),
                enabled: authoring_reason.is_none(),
                disabled_reason: authoring_reason,
                selected: Some(self.effective_tool() == EffectiveTool::Selection),
            });
            if let Some(page) = self
                .document()
                .ok()
                .and_then(|document| document.pages().first())
            {
                controls.push(ControlInfo {
                    id: "edit.select_all",
                    label: "Select All",
                    menu: "Edit",
                    submenu: None,
                    shortcut: shortcut(ControlKey::A, false),
                    command: Command::UiSelection(UiSelection::All {
                        expected_revision: state.revision,
                        page_id: page.id().get(),
                    }),
                    enabled: authoring_reason.is_none(),
                    disabled_reason: authoring_reason,
                    selected: None,
                });
                let reason = authoring_reason.or(self
                    .selection()
                    .atom_ids
                    .is_empty()
                    .then_some("empty_selection"));
                controls.push(ControlInfo {
                    id: "edit.deselect",
                    label: "Deselect",
                    menu: "Edit",
                    submenu: None,
                    shortcut: None,
                    command: Command::UiSelection(UiSelection::Clear {
                        expected_revision: state.revision,
                    }),
                    enabled: reason.is_none(),
                    disabled_reason: reason,
                    selected: None,
                });
            }
        }
        if self.has_full_native_source() {
            for (size, id, label) in [
                (3, "tool.ring.3", "Cyclopropane"),
                (4, "tool.ring.4", "Cyclobutane"),
                (5, "tool.ring.5", "Cyclopentane"),
                (6, "tool.ring.6", "Cyclohexane"),
                (7, "tool.ring.7", "Cycloheptane"),
                (8, "tool.ring.8", "Cyclooctane"),
            ] {
                controls.push(ControlInfo {
                    id,
                    label,
                    menu: "Tools",
                    submenu: None,
                    shortcut: None,
                    command: Command::UiRingToolSelect(SelectRingTool { size }.into()),
                    enabled: authoring_reason.is_none(),
                    disabled_reason: authoring_reason,
                    selected: Some(self.effective_tool() == (EffectiveTool::Ring { size })),
                });
            }
            for (kind, id, label) in [
                (RingKind::Benzene, "tool.ring.benzene", "Benzene"),
                (
                    RingKind::Cyclopentadiene,
                    "tool.ring.cyclopentadiene",
                    "Cyclopentadiene",
                ),
            ] {
                controls.push(ControlInfo {
                    id,
                    label,
                    menu: "Tools",
                    submenu: None,
                    shortcut: None,
                    command: Command::UiRingToolSelect(SelectNamedRingTool { kind }.into()),
                    enabled: authoring_reason.is_none(),
                    disabled_reason: authoring_reason,
                    selected: Some(self.effective_tool() == (EffectiveTool::NamedRing { kind })),
                });
            }
            for (kind, id, label) in [
                (ChairKind::One, "tool.ring.chair_1", "Cyclohexane chair 1"),
                (ChairKind::Two, "tool.ring.chair_2", "Cyclohexane chair 2"),
            ] {
                controls.push(ControlInfo {
                    id,
                    label,
                    menu: "Tools",
                    submenu: None,
                    shortcut: None,
                    command: Command::UiRingToolSelect(SelectChairRingTool { kind }.into()),
                    enabled: authoring_reason.is_none(),
                    disabled_reason: authoring_reason,
                    selected: Some(self.effective_tool() == EffectiveTool::ChairRing { kind }),
                });
            }
        }
        if self.has_full_native_source()
            && let Ok(preferences) = self.chain_preferences()
        {
            for (id, label, kind) in [
                ("chain.acyclic", "Acyclic chain", ChainKind::Acyclic),
                ("chain.snaking", "Snaking chain", ChainKind::Snaking),
            ] {
                controls.push(ControlInfo {
                    id,
                    label,
                    menu: "Tools",
                    submenu: None,
                    shortcut: None,
                    command: Command::UiChainTool(UiChainTool::Select { kind }),
                    enabled: authoring_reason.is_none(),
                    disabled_reason: authoring_reason,
                    selected: Some(self.effective_tool() == (EffectiveTool::Chain { kind })),
                });
            }
            let reason = authoring_reason.or((!matches!(
                self.effective_tool(),
                EffectiveTool::Chain { .. }
            ))
            .then_some("chain_tool_required"));
            for (id, label, lengths) in [
                ("chain.fixed_lengths", "Fixed Lengths", true),
                ("chain.fixed_angles", "Fixed Angles", false),
            ] {
                controls.push(ControlInfo {
                    id,
                    label,
                    menu: "Object",
                    submenu: Some("Chain constraints"),
                    shortcut: None,
                    command: Command::UiChainTool(UiChainTool::Constraints {
                        fixed_lengths: if lengths {
                            !preferences.fixed_lengths
                        } else {
                            preferences.fixed_lengths
                        },
                        fixed_angles: if lengths {
                            preferences.fixed_angles
                        } else {
                            !preferences.fixed_angles
                        },
                    }),
                    enabled: reason.is_none(),
                    disabled_reason: reason,
                    selected: Some(if lengths {
                        preferences.fixed_lengths
                    } else {
                        preferences.fixed_angles
                    }),
                });
            }
        }
        if let Some(request) = state.ui_file.request {
            let reason = (request.phase == UiFilePhase::Running).then_some("ui_busy");
            controls.push(ControlInfo {
                id: "file.cancel",
                label: "Cancel",
                menu: "File dialog",
                submenu: None,
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
