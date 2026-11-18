# Original SDF record-removal UI plan

Design only at `379fed0`, 2026-11-15. Root owns the codec/command implementation;
this plan does not implement UI, execute tests or establish native acceptance.

## Source and smallest scope

[CTFILE-2026](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf),
printed97, requires one or more compounds in an SDfile. The already reviewed official
PDF is1,580,820 bytes, SHA256
`959f2bbf5487e00fa7535ea7ee36f7ce04f39ea72920844641952276de146c94`;
printed97–99/PDF indexes100–102 are recorded in `docs/clean-room/sources.md`.
Therefore disable removal of the sole record, and retain the codec's `Empty` refusal
as independent enforcement. Do not invent an empty-SDF session. SMILES's zero-record
policy is a different format and stays unchanged.

Root's closed command is `sdf.record.remove { expected_revision, record_id }`, backed
by `SdfDocument::without_record(RecordId)`. Remove the whole explicit source record;
the UI never splices bytes, decodes MOL or rebuilds data fields. Confirmation,
ordinal selection and draft resolution below are original Bondline policies,
not a claimed Professional26.1 browser or shortcut.

## Proposed flow and extension points

1. In `ui/lib.rs`'s existing SDF Records grid, add an accessible **Remove record N**
   button beside Inspect. Disable it for count1, non-Open close/picker context or
   conflicting actual work; keep it reachable with a changed local draft so the
   existing resolution dialog can run. No global Delete hotkey.
2. Add `sdf_editor::Target::RemoveRecord(u32)` and a narrow
   `SdfFieldsWorkflow::request_remove(engine, context, record_id)`. Route first through
   `editor.intercept`, exactly like `inspect`. Changed drafts use existing
   Apply/Discard/Cancel. Cancel retains the draft and starts no removal. Discard ends
   it; Apply must finish and retire successfully before resuming.
3. Extend `SdfFieldsWorkflow::resume_target` to open a fresh confirmation through
   the editor. Capture full Weak source stamp, revision, close epoch, file epoch,
   path/format, ordinal and request sequence. Show the exact ordinal/current count
   and Undo behavior; default to Cancel. Revalidate all context and count>1 at
   confirmation. Source replacement, including identical-byte Open, invalidates it;
   never silently retarget a new occupant of the same ordinal.
   Use one checked editor-local nonce in confirmation actions/widget IDs; old
   confirm/cancel callbacks cannot act on a reopened request. Exhaustion refuses.
4. Resolve the draft **before** this confirmation. Applying a draft to another record
   cannot reorder records, so its known successful continuation may preserve the
   requested ordinal with a fresh stamp. Cancelling the later removal keeps that
   applied value. Any intervening source/destination change prevents continuation.

## One mutation owner, not another pool

Extend the existing `SdfEditor` pending/job machinery with a private two-case input
(`Field(ApplyInput)` / `Remove(RemoveInput)`) and remembered mutation kind. Removal
has no draft text. Reuse `Held`, the one waiting target, cancellation, actual-thread
retirement and fresh `resume_command` bindings. No new BondlineApp field or fixture
initializers are needed.

Add confirmation/confirm/cancel actions to `show`/`replay`'s existing indexed event
timeline. Do not execute a deletion while painting or prepass later Save before an
earlier confirmation. Confirm→Save in one input packet must queue Save behind the
actual mutation result and retirement, just as Apply→Save does. Busy/conflicting
intents keep the existing one-waiter refusal policy.

`Effect.refresh` currently assumes a field triple, and `sync` emits an effect only
when `draft` exists. Extend this narrowly to `Field { record, offset, field }` versus
`RecordsInvalidated`, independently of draft presence. On accepted removal clear
old field selection/pending queries; retain running queries for actual retirement.
On commit clear `FieldsPage`, offsets/reveal/after-query and readonly text state;
do not automatically select a surviving ordinal. Engine metadata redraws all rows.
The preview's source key naturally invalidates its image/extent. Undo/Redo also hide
old field selection; they restore complete source snapshots, not UI selections.

Never reuse `FieldsJob::retire` for mutations: it discards cancelled/stale query
replies. A committed removal remains committed after late cancellation. Report that
truthfully; do not retry it, auto-Undo it or run a waiting File action when its
post-result source/destination is no longer current. Earlier-document success gets
an explicit notice without replacing current presentation. Failure starts no waiting
Save/Open/close action. Keep status text specific when there is no recoverable draft.

## Registration, barriers and callbacks

Extend `CommandExecutor::prepare_ui_sdf`'s whitelist only with `SdfRecordRemove`,
retaining originating mutex, exact Open/close epoch/stamp validation and existing
two-worker reservation before spawning. Core finish remains the revision/generation/
owner/cancellation and single-history-commit authority; accepted earlier work may
invalidate pending close approval. Do not impose a new query-style result-discard rule.

Include confirmation, pending work and actual job retirement in `is_busy`,
`blocks_close` and `holds_files`; update `guard_external` so confirmation-only state
is not skipped by its current draft/busy early return. Existing `BondlineApp` and
FileWorkflow callsites already consume those gates. Start via `start_pending` only
after releasing the frame Engine lock. Preserve poisoned-lock draining and join only
after `is_finished`. Native worker repaint and generic loopback `result.changed`
notification already cover removal; no extra observer flag, Save lease or close ACK.

## Decisive test-first gates

- Actual-App baseline RED: missing per-record Remove role. Mixed777-byte source:
  Cancel confirmation changes nothing; confirm removing the first record keeps the
  exact second MOL/fields/delimiter bytes. Sole-record action is disabled. Undo restores
  every original duplicate/DT/raw byte; Redo removes again; Unicode Save/reopen matches
  literal surviving bytes and unchanged raw/per-MOL external consumers.
- Changed draft on the other record: Cancel/Discard/Apply branches. Apply then removal
  creates two coherent history edits; two Undos restore the original. Invalid Apply
  starts no deletion. Test exact ordinal rebasing and no stale field/preview display.
- Confirm→Save in one packet must write the removed source, not the pre-delete source;
  finish/drop/error must preserve the existing file reservation and trusted receipt.
- Finite real-worker gates: hold completion after core commit, then cancel/request
  close or poison the engine; close waits for retirement and success is not relabelled
  cancelled. Separately test stale source/close ABA before reservation, saturation,
  early cancellation and source replacement before/after commit. Old queries must not
  reinstall deleted-record fields. Use barriers, not fixed sleeps or fake completions.

## Impact and limits

UI changes stay in the existing grid, sdf_fields and sdf_editor. FileWorkflow/preview
algorithms need no new parser or publication path. Commands/type export/registry,
deferred completion and UiSdfWork permission tests belong to Root's parallel slice;
CLI/MCP discovery/catalogues must reflect the new mutating command. No wire-version,
dependency, SDK, font, asset or new crate is required. Existing SDF Save is the export
path; Drawing/cross-format refusals remain. Native input stays paused, Actions disabled.
Exact reviewed sources and no-execution qualification are in the ignored receipt.
