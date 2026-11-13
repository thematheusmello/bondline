# M0 command and UI audit

Reviewed 2026-11-13 at `fefab8a`; integrated product checkpoint `ccf8c50`.
This is source/caller/test inspection, without a new build, app launch or CUA run.
Existing native evidence in [native-file-ui.md](../native-file-ui.md) retains its exact
binary/platform scope. This audit neither repeats nor expands that evidence.

The primary Bondline prompt explicitly makes M0 the workspace, three-platform CI,
AGENTS/ROADMAP/xtask, a registry with CLI/MCP, and a complete sourced menu/tool/hotkey
inventory. Its architecture also requires every menu item, tool and dialog to be a named
typed command, shared by UI/CLI/loopback/MCP, with offscreen control captures. Later garment
or music prompts do not add their domain requirements to Bondline M0.

The workspace/layer gates, continuation instructions, eight closed-schema commands,
CLI/MCP/loopback, corpus tooling and offscreen page captures exist. Three-platform CI and
actual window smoke are recorded for the integrated product in [ci.md](../ci.md), including
run43463687047. This meets the CI requirement; it does not establish every Windows/Linux
interactive workflow. The native dependency-license blocker is resolved. Remaining M0
issues below must not be described as that former blocker.

| Priority | Concrete gap and scope | Source/caller/test evidence | Small next action |
| --- | --- | --- | --- |
| P0 | Delivered Close Window/CmdW and native Close/Quit approval are outside the named registry. This is an M0 shared-command gap even though their Mac safety workflow has native evidence. | [Registry](../../crates/commands/src/lib.rs#L77) has no close/quit command. [FileWorkflow](../../crates/ui/src/file_workflow.rs#L188) owns closing, Discard/Cancel and approvals; [native update](../../crates/ui/src/lib.rs#L498) sends viewport Close/CancelClose directly. [Lifecycle tests](../../crates/ui/src/file_workflow.rs#L408) test these UI-private decisions, not CLI/MCP/loopback close semantics. | Implement the close-policy vertical slice below; preserve current dirty/stale/save-publication protections. |
| P0 | Complete modern inventory remains unproven. Missing feature implementations may stay `missing` in M0; missing inventory evidence cannot be counted as complete. | [Parity limits](../parity-checklist.md), [modern audit](modern-inventory-audit.md), [support-index audit](support-index-audit.md) and [open questions](../clean-room/open-questions.md) explicitly leave26.1 menu contents, unnamed tool glyphs, context menus and platform/version differences unresolved. | Continue the existing qualified evidence backlog. Do not substitute576 rows, older videos or disabled entries for complete26.1 coverage. |
| P1 | Menu labels, shortcut bindings, enabled conditions and dialog presentation have no shared discoverable control contract. Open/Save submissions are correctly registered; showing/cancelling their chooser and close dialog are UI-only. | [CommandInfo](../../crates/commands/src/lib.rs#L42) has name/description/read_only/schema. [UI menus](../../crates/ui/src/lib.rs#L125), [File menu](../../crates/ui/src/file_workflow.rs#L88) and [chooser](../../crates/ui/src/files.rs#L224) separately define labels, shortcuts and modal actions. Adapter tests discover engine commands, not menu/dialog state. | Bind delivered controls to authoritative registry IDs/typed parameters and explicit presentation metadata. Specify how a dialog request differs from executing its completed command. A second unrelated handler registry would perpetuate the gap. |
| P1 | Cancel Operation is UI-only; no common job ID/status/cancel command exists. This is an automation gap, not an explicit primary M0 scheduler gate. | [UI cancel](../../crates/ui/src/file_workflow.rs#L166) calls a private token. [Executor](../../crates/mcp/src/work.rs#L16) shares two workers; [worker tests](../../crates/mcp/src/work.rs#L177) cover budget/cancellation. [Protocol](../control-protocol.md) documents request cancellation. | Give delivered job actions a shared contract as addressable jobs land. Do not invent generic scheduling/progress/recovery requirements to prevent M0 completion. |

Several tempting additions belong elsewhere, or already work within their stated scope:

- Page presets and Rotate compute typed `page.set_size` parameters; Undo/Redo call the
  same history commands ([UI](../../crates/ui/src/lib.rs#L159),
  [history properties](../../crates/commands/tests/history.rs#L10)). Preset variants do not
  need duplicate authoring implementations. Page extents are not complete Page Setup/printing.
- Bond release calls `bond.create`; the preview is transient and one valid drag creates one
  history step ([ordered input](../../crates/ui/src/lib.rs#L278),
  [drawing tests](../../crates/ui/tests/drawing.rs#L101)). There is no tool selector or
  active-tool API to audit: solid-bond drawing is implicit. Missing palette/selection,
  connected bonds, rings and broader gestures are M2, not existing unregistered buttons.
- Open/Save/Save As and their discard/overwrite confirmations end in the typed file commands
  ([mapping](../../crates/ui/src/file_workflow.rs#L278),
  [independent file tests](../../crates/commands/tests/files.rs#L102)). Directory navigation
  and editing a filename do not mutate authored chemistry. Literal dialog addressability is
  still absent; do not misreport these presentation controls as an alternate chemistry engine.
- Capture prepares an immutable document/revision and emits CPU/GPU page-scene PNG outside
  the engine lock. It never captures window chrome, dialogs or selection/tool state
  ([implementation](../../crates/commands/src/capture.rs#L97),
  [independent snapshot/pixel test](../../crates/commands/tests/capture.rs#L9),
  [loopback test](../../crates/mcp/tests/control.rs#L250)). This satisfies the narrow offscreen
  rendering clause; full-window UI automation is additional scope, not a missing page capture.
- The [pure model](../../crates/model/src/lib.rs) has pages/elemental graphs. Broader models remain M2/M3 work. ROADMAP groups
  connected editing and a general transaction/job engine under M0; the primary M0 bullet does
  not expressly require these. Preserve the full target without treating every later model
  or a generic framework as a prerequisite for the already-tested command foundation.

Next executable slice: typed `document.close` (proposed, not implemented), with
`expected_revision`, explicit `discard_unsaved=false`, and a structured refusal/approval.
Keep authored data/history unchanged. Save remains the existing command; retain receipt
checks before renewed approval. Deliver approval through the shared executor/session contract
to the native adapter, rather than returning approval JSON while leaving external Close
unwired. Test dirty/stale/Cancel/save-published cases, all adapter schemas/wire behavior and
native close/Quit regression. Multi-document semantics and automatic discard are outside
this single-document slice.

This classifies the primary M0 boundary; it does not remove or reorder ROADMAP work.

Pinned PhotoCraft offers two relevant decisions, not a ready-made Bondline implementation:
[CommandSpec](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/engine/src/cmd.rs)
associates IDs/menu/shortcut/enabled metadata, and its registered `file.close` is shared with
the [menu catalog](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/menu_catalog.rs).
Keep Bondline's protected-close defaults and typed schemas; PhotoCraft defaults to discard
and uses descriptive `Value` parameters. Its
[control dispatcher](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/control/dispatch.rs)
distinguishes dialog-opening menu invocation from direct parameterized engine execution.
Use that distinction, without copying its separate UI dispatch tables into Bondline's
one-registry design. These are pinned source observations; no reference code/assets/tests
were imported or executed.

## Resolution and remaining work

The identified P0 close-policy slice is now implemented: document.close has typed conditional
Request/Discard/Cancel, transient epoch and trusted host acknowledgement; CLI/MCP/loopback
use the registry, and native Mac execution verifies the stated workflows. Registered-work
and reply-publication barriers plus receipt/epoch guards prevent premature shutdown and
stale Save continuations. See [the contract](../control-protocol.md#close-requests-and-host-shutdown)
and [exact native evidence](../native-file-ui.md#shared-close-execution-evidence).

This resolves the first table row for the delivered single-document scope. Complete modern
inventory, shared menu/shortcut/enabled/dialog discovery and addressable job cancellation
remain open. The next implementation should expose existing controls through authoritative
registry metadata/presentation requests; do not implement document.close again or treat this
resolution as complete M0, full dialog addressability or multi-document parity.

The next slice adds ui.controls and ui.file through the existing registry. Delivered
File/Edit/Page bindings now have one typed core source consumed by native menus/shortcuts
and discovery. A finite file request drives actual native picker acceptance/cancellation;
headless engines refuse without a host, and atomic guarded preparation prevents stale
selection I/O. This resolves the delivered menu/shortcut binding portion of the P1 row,
not every dialog field/button or running job. Location/name values, directory navigation,
confirmation dismissal, complete close-dialog discovery and Cancel Operation remain gaps.
Exact RED/GREEN, native/platform evidence and still-live jobs belong in verification.md and
STATUS.md. No inventory count or catalog-only assertion establishes a native workflow.
