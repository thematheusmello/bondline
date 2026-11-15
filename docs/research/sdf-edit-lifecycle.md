# SDF draft lifecycle: minimal integration plan

Read-only audit, 2026-11-15, supplied base `778ea47`. No editor implementation,
product/test change, Cargo, native input, Git or Actions operation occurred. Root owns
implementation and integration. These are original Bondline lifecycle policies; the
[public-source workspace plan](sdf-workspace-plan.md) does not establish a26.1 desktop
field-editor contract. Keep its CTfile value/occurrence limits and full M0-M5 scope.

## Required state and invariants

Keep one bounded draft separate from the existing read-only FieldsPage/cache. It needs
a checked local draft ID, complete CaptureStamp, source revision, source-local record/
field occurrence, original logical lines and editable text plus original file/field
metadata for honest attribution. The existing weak stamp is sufficient for eligibility;
do not retain/reparse another64MiB source merely to keep a small draft. Preserve stale
text visibly after replacement, with Apply disabled: equal bytes/ordinals are not a
rebase. Do not hide it just because current content is Drawing/SMILES.

Keep only one guarded continuation, as a semantic intent such as record/field/page
navigation, shared File control ID, document history or Close. Do not save a constructed
Command with an old expected_revision, a FileIntentAction or guessed approval. Conflicting
later intents need visible busy/decision handling; no unbounded replay queue is required.
An idle unapplied draft is not worker activity, but it **does block native final close**.

Apply/Discard/Cancel resolve the draft before the held intent:

- Apply starts the existing registered `SdfFieldSet`; retain the draft, pending intent
  and actual job through outcome delivery and thread retirement. Success/no-op ends that
  local transaction and refreshes explicit inspection. Validation/busy/cancel/stale errors
  retain text. Never clear it merely because a worker was accepted or a request sent.
- Discard drops only the local draft, clears its owned TextEdit history and continues a
  still-current held intent. It is not `document.close(discard)` and does not discard the
  authored document or source history.
- Cancel cancels the attempted navigation/File/close continuation and keeps the draft.
  Escape in this decision has the same meaning. A separate editor-end action must have
  explicit discard semantics; do not make Escape silently erase typed data.

Do not set Engine file.dirty merely to represent a local draft. Actual Apply alone makes
the shared authoring commit; read-only/source/saved/history and file receipt truth remain
the existing core's responsibility. Do not start a new editable draft behind a picker,
non-Open close context or an active file job. Existing drafts must nevertheless survive
those contexts if external commands create them.

## Exact caller boundaries

Route recognized semantic activations before existing blocked returns or
`pending_ui_blocks_authoring` continues, so an asynchronous Apply cannot silently swallow
later Save. Reuse central shortcut/control metadata to identify the intent, then resolve
its current enabled Command only when the gate permits execution. Never bypass core
source/close/file guards or accept an unfocused key merely to retain a continuation.

| Current source | Minimal integration |
|---|---|
| `ui/src/sdf_fields.rs::clear`, `sync`, `inspect`, `request`, `deactivate` | Current clear destroys cached payload on source/close changes. Keep draft ownership independent. Gate record/field/page/Close-fields actions **before** these clearing calls. Source change marks the draft stale instead of erasing it; the old read-only page still retires normally. |
| `ui/src/lib.rs::show_workspace_with_preview` and `replay_edit_shortcuts` | SDF returns before the drawing raw-event loop, so the fallback is essential. Extract editor/menu activations as event-indexed intents; put the same narrow draft gate before control resolution/`engine.execute` in **both** loops. Apply must not execute in inspector painting. File/history intents following it in the same packet must be retained, not executed against old content or silently lost behind busy. |
| `FileWorkflow::dialogs` -> `adopt_request` | Adoption is currently first, before close dialogs. Gate it before claiming any externally Requested `ui.file` while an unresolved draft/Apply exists. Never prepare a native file worker or capture its source before the draft outcome is known. |
| `BondlineApp::update_workspace` | After file poll, synchronize draft identity before clearing views or processing native close. Route native/smoke Close through the draft gate before `files.request_close`. Include draft/decision/continuation/pending Apply/job retirement in `complete_close_if_idle` barriers; retain the existing file/executor/preview barriers. Start registered Apply only after frame mutex release. |
| `UiSdfWork` | Reuse the delivered source/close guard, whitelist, shared two-worker permit and complete/finish path. It requires Open plus exact close epoch. Queries may be dropped when stale; mutating Apply outcomes must always be consumed truthfully, as below. No new Save lease or command family is needed. |

For Apply success, validate the actual returned revision against the current originating
Engine under its mutex, plus continuation source/file/close context. Then acquire a fresh
CaptureStamp and requery the selected occurrence. A changed commit deliberately makes
the old stamp stale; that alone is not failure of our own Apply. A later external edit/
Open/history changes revision again, so do not resume the old continuation into it.
Association-only Save As may change path/format without authoring: guard that destination
context too before automatically continuing a held File action.

An Apply may commit before a late cancellation or before its result reaches the UI.
Do **not** reuse read-only retire's cancelled/stale => discard-success rule for edits.
Consume the real result after the thread exits even if the view key changed. A successful
historical commit stays applied; preserve recoverable text/status and suppress stale
continuations rather than claim rollback or automatically Apply again. Core's existing
accepted-earlier completion semantics must not be tightened to hide this case.

## File requests and same-packet Apply -> Save

For a local Save/Open/Save As, hold its shared control ID before creating `ui.file`.
Once Apply actually succeeds and retires, resolve the binding from current Engine
controls and enter the ordinary FileWorkflow. Known Save/Save As/close-Save keep their
existing captured encoding, reservation and trusted receipt rules. No old-byte file
worker may run while Apply is pending. A failed Apply starts no continuation I/O.

An external `ui.file` can already be Requested when the frame notices the draft:

1. Hold its request identity/action **and ui_file.epoch** and defer host adoption. Cancel uses the real
   `UiFileAction::Cancel` with matching ID; never fabricate a successful finish ACK.
2. Discard can adopt that same still-current request unchanged.
3. A **changed** Apply calls commit -> invalidate_close -> invalidate_file_ui, removing
   a non-Running request. After outcome/retirement, issue a fresh same-action intent with
   current revision only if no newer source/destination/close/request superseded it. The file epoch
   distinguishes our non-Running request invalidation from a later Cancel/new request.
4. A true **no-op** Apply preserves the original request. Adopt it if still current;
   blindly issuing another request would fail busy. Do not clear valid request state as
   though a file operation completed.

Leave already Running/publication work truthful and let it finish through existing
barriers. It may save the earlier committed snapshot while the draft remains unapplied;
do not relabel its receipt as saving the draft. Direct CLI/MCP `document.save_sdf` likewise
saves committed Engine content, not hidden UI text, and must not clear a local draft.

Apply can alter JSON-budget pagination without changing occurrence order. Retain a
semantic Next intent and recompute `next_field_offset` from refreshed inspection, rather
than jumping to an offset calculated from the older page. Discard/no-change navigation
can use a still-current page. Record/field ordinals remain source-scoped; no reorder/
deletion identity is promised.

## Native and external close/Open

Native close, File Close and remote Close share the draft resolver, not private close
approval. Remote `document.close` may already report Approved for a clean Engine even
though local text is unapplied: do not acknowledge native close while that draft exists.
Remote Discard concerns authored content; it does not silently discard the local draft.

If Apply is chosen after Close already reached NeedsDecision/Approved, use the shared
`CloseAction::Cancel` under the Engine lock after checking the captured close ticket.
That returns Open and advances epoch, allowing fresh UiSdfWork preparation. Hold only the
local continuation; after a current successful Apply/retirement, issue a **new** Close
Request. The normal dirty-document Save/Discard/Cancel flow then handles the committed
edit. A new close-Save must get a fresh epoch/revision/request and current receipt. Never
reuse an old epoch, ui_request_id or trusted acknowledgement across Cancel/Apply.

Cancel the attempted close through shared Close Cancel while preserving draft text.
Discard the local draft and then let the existing still-current core close policy/file
dialog proceed. A new remote Request/Cancel/Open supersedes an old continuation; its
values returning to an earlier state cannot revive the old ticket.

Direct external Open/history/field changes can complete before the UI sees them; the UI
cannot retroactively veto that commit. File poll or the next frame must retain a stale
draft, stop auto-Apply, clear old read-only pixels/pages and show its original attribution.
Apply must never retarget a matching ordinal in the replacement. Native close remains
blocked until that stale text is explicitly discarded or its attempted close cancelled.
No headless Engine is made dependent on a GUI draft.

## Local text history versus document history

The raw event vector is cloned before TextEdit processing, while the fallback currently
checks final `text_edit_focused`. That alone is unsafe for local CmdZ followed by focus
change/Apply in one packet: egui can process draft undo and the later fallback can also
dispatch document Undo. Capture editor event ownership/ordered intents so draft-local
undo/redo is handled once by TextEdit; an explicit document-history action instead enters
the draft resolver before shared dispatch. Test both focus-transition directions.

Keep normal local TextEdit undo while editing, unlike the read-only values' deliberate
per-render `clear_undoer`. Reset only the owned draft widget history when that transaction
ends/rebases, preserving selection/copy and avoiding retained obsolete draft payloads.
Per-keystroke edits do not make Engine history; one successful changed Apply does. A
validated same-value Apply preserves the core's exact no-op/source/redo/file ACK behavior.

## Decisive tests before implementation

Use the predeclared777B mixed SDF, record1/field_index2 second DUP. Actual Apply -> Save
must produce only the378..385 splice to `reviewed\nline two\n`:788B/SHA
`e63cb9adf5cbc62b5b0be94992246c0d10b94cda405e49b9514993e7d9f9c361`.
Keep unchanged external raw/per-MOL consumers and original inputs.

1. Real App frame: type draft, attempt record/field/page/Close-fields/Open/history;
   Apply/Discard/Cancel retain or resolve the exact text as specified. No Engine change
   before Apply; local text undo never also changes document history.
2. Gate real registered Apply before completion, then issue Save in the same raw packet.
   Old file stays777B until outcome; after retirement it becomes788B. Exercise failure,
   busy, no-op, late cancellation and successful commit before delayed UI delivery.
3. External Requested Save/Open and close-Save: Discard preserves valid request; changed
   Apply creates a fresh one; no-op retains the original; Cancel has no fake success ACK.
4. Native/remote Close on clean and dirty sources, Request/Cancel/Request, and Close while
   Apply result is queued but thread lives. No final host ACK before draft/job resolution;
   fresh post-Apply close-Save uses its own matching receipt.
5. External identical-byte Open, wrong-kind Open, history ABA and field edit during draft/
   Apply. Text remains recoverable, stale target refuses, no continuation overwrites new
   source/destination, and historical successful commits are reported honestly.

Use actual event/worker/result gates, then scoped checks and Root local CI. Native
workflow remains a separate Root-owned gate. UI labels/control contracts are forthcoming;
this audit does not invent those or claim an implemented editor.

## Source identity

Exact read inputs at supplied base; ignored receipt records remaining hashes.

| File | SHA-256 |
|---|---|
| `ui/src/lib.rs` | `c0b46295352508bb74ee35293216dd3d179ad69bf0cfe9930511ec3580f6a7dc` |
| `ui/src/file_workflow.rs` | `582a6a783223bcca43b5bc9186e20a2beeaccfd6d2bf7c06b27a5ec7365be396` |
| `ui/src/sdf_fields.rs` | `e20fd23f5f91af73dd15951c12a6edfb561fc7edebbf59a7af5bf2007d69e2b4` |
| `mcp/src/work.rs` | `2402950d3a282f97f50a80f9955265fe34a53b477efa445f6b96637109b49791` |
| `commands/src/close.rs` | `b58e5cae6e58d1188ae686e25080d81d84eb4b2800a2a72c7dafeb635ac20c07` |
| `commands/src/file_ui.rs` | `e1df4f4c5dae2797d64b03f45f6d9f44fb5cc20c29bb58d4ca46d0c116a5fddf` |

Paths in this table are relative to `crates/`. Existing inspected sources include the
published/locked egui0.29.1 TextEdit event and state APIs; no vendor modification, extra
runtime dependency, parser rewrite or general job framework is proposed.
