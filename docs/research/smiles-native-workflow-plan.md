# Native SMILES file and record workflow

Research/implementation plan, 2026-11-15, reviewed source `976ef42`. No native input,
build or implementation is claimed here. Next slice: **open a complete accepted file,
explicitly inspect any record, select/copy its exact notation, Save As and reopen every
source byte**. This completes a useful native record workflow; chemical preview,
record editing, conversion/export controls and full M1–M5 remain required.

## Existing implementation and source boundary

The chooser already offers `.smi`/`.smiles`; `FileWorkflow::start` routes through
`document.open_smiles/save_smiles`, `UiFileWork`, FilePlan and the shared save reservation.
`show_workspace_with_preview` currently returns after count/byte/unavailable-preview
labels in its `smiles_summary()` branch. It exposes no record control.

`smiles.record.inspect` already accepts closed `{expected_revision,record_id}`. Its worker
reparses only that ordinal with `read_record` and returns exact notation, literal counts,
physical line and global spans. It never discloses application-data bytes. The selected
object has an actual128KiB serialized budget; notation is at most65536 bytes. The complete
source owner survives work; the self-contained reply does not retain another whole file.
There is no persistent chemical selection, graph geometry, naming or canonicalization.

Do not route a selected record through `file.convert_smiles_to_cdxml`: that existing
converter rereads an explicit whole path, accepts only a <=64KiB notation-only input and
lays out a closed1–12 neutral carbon/Single path. General multi-record files and opaque
application data have a different contract. Do not write a temporary record file to
pretend this is selected-record conversion. `file.export_smiles_fragment` instead requires
an explicitly selected, fully mapped Drawing fragment and writes chemistry only; it is
not SMILES Save or current-record export. Those future native flows need separate honest
contracts and acceptance.

## Original UI and lifecycle contract

Keep the compact source summary and explicit unavailable chemical-preview message. Add
one original “SMILES record” inspector: bounded positive ordinal, Inspect, Previous,
Next, Close record; no full65536-row list. Open starts with no notation disclosed. Inspect
or Previous/Next explicitly requests an ordinal. At zero records, show an empty state and
disable record actions. Navigation has truthful first/last limits; typed/out-of-range
input cannot silently clamp to another record. Present returned record/line, literal
atom/bond counts, notation/global ranges and application-data byte count only. Distinguish
absent spans from present empty spans. Never decode/display metadata, synthesize a name,
normalize bracket spelling, infer H or call notation a chemical preview.

Use one bounded, immutable, selectable notation TextEdit in a bounded scroll area.
Visual wrapping must not change copied text. Follow `SdfFieldsWorkflow::show`: AccessKit
ReadOnly; clear/store the returned TextEdit undoer each render while preserving current
selection/copy. On hide/source change clear focus and retained text state. Typing,
paste/cut and local Undo must not change notation, source, dirty or authoring history.
No new font/icon asset is required.

A `SmilesRecordWorkflow` stays in `BondlineApp`, outside Engine/history. Its key is the
full weak CaptureStamp, close epoch, source revision, ordinal and checked monotonic request
sequence. Keep at most one pending intent, one actual worker and one selected reply.
Clear old notation immediately on navigation, hide, Close or source replacement, including
identical-byte Open; never keep stale pixels until completion. Cancel obsolete work;
accept only the current key after actual worker retirement. Busy/retry/error/cancel are
visible and bounded. No camera/DPI key, Save lease or source mutation belongs to this query.

`prepare_ui_sdf` explicitly permits only its two existing SDF variants. Add the narrow
analogue `prepare_ui_smiles(&CaptureStamp,close_epoch:u64,SmilesRecordInspect)` returning
opaque `UiSmilesWork`. Under the originating Engine lock check Open/exact close epoch and
full stamp, then reserve the **existing** two-worker permit before Core prepare. Retain
that origin privately and use existing registered completion. Do not spawn first and
validate later, add another pool, call a parser from painting or broaden the SDF whitelist.
The registry remains28 commands. No new wire command or generic job framework is needed.

In `BondlineApp::update`, sync/deactivate beside preview/fields; start pending only after
the frame Engine guard is released. Include pending/job retirement in `FileWorkflow`
dialog and `complete_close_if_idle` busy gates. Publication/permit release is not actual
UI thread retirement: join only after `is_finished`, including engine-free poisoned-lock
cleanup. Accepted historical query success cannot reinstall text after Open/close. Keep
existing ordered File/Edit routing and trusted Save acknowledgement unchanged. This
read-only inspector adds no draft-resolution dialog or close authorization.

## Decisive test-first acceptance

1. First actual-App RED: opening a three-record fixture offers no record inspector.
   Separately freeze a missing-bridge compile RED, then the real shared wrapper behavior.
   Reuse the original49-byte query vector, not a new convenient chemical subset:
   `CC first\r\n\n \xffignored\n[13CH3][OH] \0\xffPRIVATE\nO last`.
   Inspect2 must show exact `[13CH3][OH]`, line4, literal2 atoms/1 bond and global
   full21..43/notation21..32/separator32..33/data33..42/LF42..43 ranges. PRIVATE,
   NUL and non-UTF8 data must never reach shapes, AX, Debug or clipboard.
2. Real App controls select first/nonfirst/last, enforce boundaries, handle zero records,
   clear old text on A→B→A/Open, and render a readable bounded65536-byte notation.
   Use unchanged raw-record/per-record external consumers plus literal spans; a full RDKit
   supplier is not the file/opaque-metadata oracle. Test exact standard copy of the selected
   notation, readonly AX and no retained TextEdit undo snapshots after hide/source change.
3. Bridge tests use actual background completion, full source/history/dirty equality,
   foreign/stale/close refusals before registration and shared two-slot saturation,
   cancellation/drop/release. UI gates hold the real worker after publication to prove
   Close waits for retirement and poisoned-lock cleanup drains without stale installation.
4. Actual filesystem App test: Unicode `.smiles` Save As, existing-target refusal/retry,
   known Save, reopen and full49-byte equality. Malformed later notation must retain the
   previous whole source/selection context and failed decoder retry. Do not manufacture a
   dirty-SMILES edit: this source variant has no editor.
5. After desktop availability is explicitly established, Root runs that real chooser→
   Inspect2→select/copy→Next/Previous→Unicode Save As→reopen→Close journey. Check actual
   saved bytes with unchanged consumers and observe stale text disappearance/native
   accessibility. Until then, actual-App input/AX tests remain headless evidence; native,
   platform and clipboard execution stay explicitly unverified.

Implementation ownership can split cleanly: MCP bridge+tests; UI workflow/three lib
integration points+App tests; independent file/native acceptance. Reuse existing modules,
not new crates. Root then runs local workspace gates, records parity/status/provenance and
commits small buildable slices. Actions remains disabled; no push or desktop polling.

## Evidence, adoption and remaining questions

- [OpenSMILES v1.0 source, Files/string sections](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc): notation and opaque application data are distinct; LF/CRLF frame records. Cached86816B/SHA256
  `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21` was reread.
  Bondline's final-EOF and ignored-prefix policies remain the existing qualified original
  codec decisions, not new full-standard conformance claims.
- [Public Guide25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
  printed350–351, documents selected-structure Copy As and pasted SMILES text workflows.
  Cached text was reviewed; PDF17721447B/SHA256
  `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
  These are future clipboard/structure roles, not evidence of a26.1 file-record browser.
  No new screen/visual-runtime evidence was obtained and no help prose/assets reused.
- [PhotoCraft jobs UI](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/jobs_ui.rs),
  MIT OR Apache-2.0: app-owned background work and explicit finished state support keeping
  lifecycle outside painting. Its batch/tab/save machinery is unnecessary here; Bondline's
  existing registered work/retirement contracts remain authoritative. No code copied.
- [VoiceStudio inspector](https://github.com/debpalash/voicestudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/electron/src/renderer/src/features/workflows/workflow-inspector.tsx)
  and [job state](https://github.com/debpalash/voicestudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/electron/src/renderer/src/features/transcriptions/transcription-job.ts),
  **AGPL-3.0-only**: conceptual evidence for contextual labelled inspection and job
  ownership separate from page visibility. No code/assets/runtime are copied or linked.
  Its persistent transcription-on-navigation policy is not adopted for cancellable
  readonly source disclosure.

The ordinal inspector, labels, privacy and lifecycle choices above are original Bondline
policy. Exact26.1 record-browser design/shortcuts, general SMILES depiction/layout,
name/structure, selected-record conversion, broader syntax and native Copy As/Paste
Special remain unknown/missing. They are not silently closed by this slice. Reviewed
local/public-source hashes and exact caller anchors are in ignored
`target/smiles-native-plan/research-handoff.json`.
