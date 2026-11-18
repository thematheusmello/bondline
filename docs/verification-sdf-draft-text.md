# Bounded SDF draft text

This is a headless-verified temporary text component, not a delivered field-editing
workflow or native-window result. Root owns its editor, Apply, navigation and close
integration. Existing SDF syntax, command validation, files and fixtures are unchanged.

## Contract and independent expectations

`sdf_draft_text::SdfDraftText` owns literal UTF-8 text of at most
`MAX_SDF_INSPECT_VALUE_BYTES` (65,536 bytes). Construction checks the size before
copying. Spaces, newlines, Unicode and temporarily invalid SDF characters remain
literal; there is no trimming, line wrapping or chemical interpretation. Apply must
still use the existing command's ASCII/line/delimiter, raw-value and serialized-JSON
validation. A draft inside this byte limit is not necessarily valid command input.

The API is `new(&str) -> Result<Self, DraftTextError>`, `as_str()`,
`last_rejection()` and `edit(closure) -> (R, DraftEditOutcome)`. The outcome has
`changed` and an optional `Copy`/`Display` typed rejection. Debug omits field text.

Trait inserts use character indices and return inserted character counts, while
the storage limit counts UTF-8 bytes. Inserts and replacements reject the entire
operation if they exceed the limit. Reversed/out-of-range mutation ranges and
cursor indices reject without changing text. Read-only invalid ranges return
empty text; read-only character-to-byte lookup clamps to the end, as the pinned
SDK helper does. Cursor insertion clamps an invalid returned cursor and avoids
unchecked addition. Guarded character/word/indentation operations reuse the SDK's
semantics only after validating its character cursor. Finite widget character
limits also reject whole insertions instead of truncating them.

One `edit` call surrounds one complete `TextEdit::show`. It retains one bounded
temporary pre-widget copy. Any rejected mutation immediately restores that copy,
then freezes later text mutations in the same call. The next widget layout
therefore sees the original text. A successful transaction reports a change only
when its final text differs. A rejection rolls back earlier edits in that same
widget transaction as well. The outcome, rather than `Response::changed`, is
authoritative for draft changes: the SDK can mark a rejected paste as changed.
The closure receives only `dyn TextBuffer`, preventing nested draft transactions.

These policies follow inspection of the actual pinned egui 0.29.1 source, not a
remembered API or an inference about the reference application's editor:

- [TextBuffer](https://github.com/emilk/egui/blob/0.29.1/crates/egui/src/widgets/text_edit/text_buffer.rs): character indices/counts, the default clear-then-insert replacement, truncating finite character limits and unchecked cursor addition.
- [TextEdit event handling](https://github.com/emilk/egui/blob/0.29.1/crates/egui/src/widgets/text_edit/builder.rs): Paste/Text delete the selection before inserting; the widget relayouts after edits and feeds undo state before/after events.
- [TextEditState](https://github.com/emilk/egui/blob/0.29.1/crates/egui/src/widgets/text_edit/state.rs) and [Undoer](https://github.com/emilk/egui/blob/0.29.1/crates/egui/src/util/undoer.rs): public cursor/undo replacement and configurable snapshot count.
- [Cursor helpers](https://github.com/emilk/egui/blob/0.29.1/crates/egui/src/text_selection/text_cursor_state.rs): valid-cursor assumptions in word/indentation helpers, and read-only byte-index clamping.

The first-GREEN receipt records exact local source sizes/hashes, including these
five SDK inputs, the existing command limits and workspace plan. No SDK file was
changed for this component.

## Required caller behavior

Use a stable widget ID for a draft's lifetime. At draft start configure its public
`TextEditState` undoer with `Undoer::with_settings(Settings { max_undos: 8, .. })`.
Keep `TextEdit`'s default `char_limit = usize::MAX`; byte bounding is handled by the
buffer. Snapshot the prior character selection, then call `draft.edit(|text|
TextEdit::multiline(text).id(editor_id).show(ui))`.

On rejection, display the typed error, restore the prior cursor in the returned
widget state, replace its undoer with a fresh max-eight undoer, and store that state
under the same ID. This prevents a rejected event's cursor/undo side effects from
surviving. Preserve the visible error in the workflow until an appropriate later
action; `edit` starts a new outcome on every call. At DraftEnd clear/store the
widget's undo state and release the draft. `clear_undoer()` installs the SDK's
default max-100 undoer, so any later editable use must install max-eight again.

The bound describes text bytes, not total process memory. The current draft,
temporary transaction snapshot and bounded undo snapshots are distinct copies.
The SDK may also retain one in-flux undo snapshot. Event/paste strings, layout
galleys, source/inspection data, external copies and allocator overhead are not
covered by a hard RSS claim. Do not retain past editor IDs containing drafts.

## Test-first evidence

Base: `778ea47`, dedicated worktree `bondline-sdf-draft-buffer`. The original
tests-only file was 10,872 bytes, SHA-256
`045bc24c7b822192e1249abc1f1e846a8153d7c02c3f77e2f149345df51d6a14`.
Only the permitted `mod sdf_draft_text;` bridge was added to existing product code.

1. Session `3913`, exit 101: missing component compiler RED, plus a distinct test
   setup error: `egui::text_edit` does not re-export its internal `TextEditUndoer`
   alias. The original source and 14,016-byte log remain in ignored evidence.
2. The helper return type alone changed to the actual public
   `Undoer<(CCursorRange, String)>`. All eight assertions/input sequences remained
   unchanged. Corrected tests-only source: 10,903 bytes, SHA-256
   `a9e2eba1cfd7378d174f3e0a1c3613e9f9778fbf67edf01a4b24767e90f225d9`.
3. Session `34900`, exit 101: corrected missing-component compiler RED; no
   runtime claim. The log is 6,959 bytes, SHA-256
   `c14fc604ecc2939fa6a90e20a91fae0bbae2db0aaeb1585fb6d59c300b37021f`.
4. Minimal component implementation, then session `91022`, exit 0: **8 passed,
   0 failed, 0 ignored**, 102 unrelated tests filtered. The complete corrected
   test module is byte-for-byte identical to its pre-implementation freeze.
   This is the first runtime execution, not a previously observed runtime RED.
5. `rustfmt --edition 2024 --check crates/ui/src/sdf_draft_text.rs
   crates/ui/src/lib.rs` exited 0.

The eight tests establish:

- Literal whitespace/Unicode/control preservation and independent 65,536-byte UTF-8 constructor boundaries.
- Character-indexed insertion/deletion, exact character counts, clear/take and byte lookups.
- Exact-fit insertion, whole oversized insertion/replacement refusal and finite character-limit refusal.
- Reversed ranges, out-of-range and `usize::MAX` cursor calls without panic or data loss.
- Immediate whole-transaction rollback and refusal of later clear/replace/delete/insert/take mutations, followed by a valid transaction and a no-op.
- An actual configured egui `TextEdit` selecting `βeta`, rejecting a 65,538-byte Unicode Paste plus subsequent Text in the same frame, preserving the actual same-frame galley, then accepting a valid Paste into the restored selection.
- The public max-eight undo policy and clearing of undo/redo/in-flux state at draft end.
- Debug text privacy.

These are independent literal byte/character expectations and real pinned-widget
behavior, not an own-reader/own-writer round trip. They do not establish OS
clipboard/IME interaction, native field editing, source mutation, Apply/Save or
draft navigation/close safety. Those remain Root's integration/native gates.

Reproduction uses the worktree's absolute `CARGO_TARGET_DIR`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`, and the existing pinned `BONDLINE_PYTHON`:

```sh
cargo test -p bondline-ui --lib sdf_draft_text::tests --locked
```

Evidence is under ignored `target/sdf-draft-text/`: initial/corrected frozen test
sources, both RED logs/receipts, `first-runtime.log`, `first-green-receipt.json`,
`fmt.log` and the final handoff. The baseline has 846 existing files; 845 remain
exact, and removing the sole new module bridge reconstructs the final old file
exactly. Dependencies, lock, SDK, fonts, all old tests, fixtures, consumers and
approved goldens are unchanged. At the original component freeze, full
UI/workspace and warnings-denied package linting awaited actual caller
integration. No native interaction, hosted workflow or Git action ran here.

## Receiving lint fixture correction

Root subsequently integrated its real editor caller. Its receiving whole-UI
session `29303` passed, including these eight tests. Receiving clippy session
`28956` then rejected the deliberately malformed `3..1` fixture under
`clippy::reversed_empty_ranges`. That was a test-syntax lint, not a changed range
expectation or buffer defect.

Only that constructor changed to `Range { start: 3, end: 1 }`, with rustfmt array
wrapping. The range remains reversed, every assertion/input remains the same,
and the complete production prefix is byte-for-byte unchanged. No suppression,
quota change or production adjustment was made. The previous 20,755-byte source
with SHA-256 `983ca8599a9a8adfa0e9508221f9d09423b8b1957a533df75527c673a5fb462e`
is preserved under ignored evidence.

The corrected source is 20,836 bytes, SHA-256
`578f6a6ab03a3363b8ccb529feda01c7416279d471cb72d18b84cbc168b01d66`.
Own session `47030` repeated the focused suite: **8 passed, 0 failed, 0 ignored**;
the same source/bridge formatting check exited 0. Root owns the receiving
warnings-denied lint gate and workflow/native verification. The bounded delta,
before/after source/doc hashes and fresh logs are recorded in
`target/sdf-draft-text/range-fixture-handoff.json`; all jobs here are terminal.
