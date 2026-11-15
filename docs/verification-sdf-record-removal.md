# SDF record-removal integration — 2026-11-15

The complete-record codec, one typed command, delivered automation adapters and original
native Records workflow are integrated. This is one bounded M1 feature; full M0/M1 and
M2–M5 remain incomplete. GitHub Actions stayed disabled. Original code/assets remain
within the existing license policy; no dependency, lockfile, SDK, font, corpus fixture,
external oracle or human-approved golden changed.

## Contract and sources

The official [BIOVIA Chemistry2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf),
printed97–99, requires one or more compounds. Removing a valid sole record therefore
returns invalid_sdf; a missing target returns not_found first. Whole-range byte removal,
source-local ordinal reindexing and the confirmation/draft workflow are original Bondline
policies, not verified Professional26.1 screen parity. The pinned PDF/source decisions
remain in clean-room/sources.md and the original UI plan.

`sdf.record.remove` accepts exactly expected_revision and positive record_id. It removes
the complete selected raw record and reparses every surviving byte, retaining original
backing/current/saved distinctions. One successful edit uses the existing deferred
candidate/commit route and complete Arc snapshot history. document stays null; a survivor
stays SDF. Save is separate and publishes the whole current source. Failed/cancelled/
foreign/stale completion and committed close preserve the common guards. Existing field
budget/validation/privacy and no-op behavior remain covered by unchanged tests.

The Records action resolves a changed field draft before fresh confirmation. Keep record
starts no mutation. Confirm invalidates old ordinals/fields/query text, and a same-packet
Save waits for successful commit and actual worker retirement. Late cancellation cannot
relabel committed success. Removal reuses the one editor pending/job owner and existing
two-worker budget; there is no new App field, executor, file lease or native ACK authority.

## Independent tests and reproduced defect

- [Codec](verification-sdf-record-codec.md): seven cases against literal raw spans,
  unchanged scanner/per-MOL consumers, mixed versions, CRLF/EOF/padding/duplicates,
  first/middle/final/last/missing behavior and immutable allocation ownership.
- [Command](verification-sdf-record-command.md): six independent cases cover exact
  saved bytes/history, closed mutating schema, source/generation/foreign/cancel/close
  checks, historical queries and captured older Save versus later edit.
- Root wire tests exercise actual CLI, stdio SDK discovery/calls and authenticated
  loopback, exact315-byte output and unchanged raw/per-MOL consumers. There are29 tools;
  no protocol-version change. These additional transport tests first ran after the
  implementation; they are not claimed as a separate RED.
- [Original UI/lifecycle](verification-sdf-record-removal-workspace.md): four frozen
  independently authored actual-App cases and five finite real-worker cases. They cover
  draft Cancel/Discard/Apply/invalid Apply, ordinal invalidation, Unicode Save/reopen,
  ordered Confirm→Save, publication versus retirement, late cancellation, replacement
  and poisoned-engine draining. Native input evidence is separate below.
- [Independent review](research/sdf-record-removal-review.md) reproduced a real product
  race: a ui.file Save request interposed after confirmation replay and frame unlock
  still allowed a removal worker to spawn. Runtime31582 failed the intended no-job
  assertion before the fix. It did not establish that deletion completed. The unchanged
  3,736-byte regression, SHA256
  d641a6460662544c793b91d99bc47f996dc21e9e0564f7cf2690b904214de292,
  passes independent68063, producer13577 and Root's final aggregate.

The fix retains opaque UiSdfRemoveContext through pending input. Specialized native
preparation checks source/Open/close epoch, path/format/file-request epoch and no active
file request under the origin mutex before reserving/preparing. Generic prepare_ui_sdf
accepts query/field only. Root29536 passes three native removal tests and seven unchanged
field/query cases, including requested/cancelled Save epochs, direct SaveAs association,
shared capacity and early cancellation. Review found no further material defect in its
stated bounded scope. This is not a global lock against later explicit commands.

The original missing-method/UnknownCommand/missing-control REDs, real preparation race,
and test setup corrections are recorded separately in the linked records. Initial new
CRLF/header, private-close accessor and expected NeedsDecision corrections changed no old
oracle or production contract. Initial native boundary compilation11562 used an invalid
borrow of a temporary MutexGuard; an explicit local guard corrected that new test only.
Legacy full-catalogue checks add the29th name/count, retaining every old assertion.
The old private lifecycle fixture changes only its necessary enum/job setup; old bodies
and all four independent App tests remain exact. No blanket assertion relaxation occurred.

## Root aggregate and exact inputs

Root96793, pinned BONDLINE_PYTHON and CARGO_INCREMENTAL=0, completed:

```sh
cargo xtask ci
```

Result: **1060 passed,0 failed,1 ignored**,153 result lines. Formatting, all-target
warnings-denied clippy, layers, runtime/asset licenses and pinned corpus all pass. Log:
target/sdf-record-remove/root-final-ci.log,127700B/SHA256
cbe106e2813aee252f479984047ae10653fa8b1863b8bebb7d8152657ae76525.
Earlier core gate failures were exact catalogue declarations, not format corruption;
focused corrected CLI and SDF catalogue cases pass. The one existing ignored visual
capture remains ignored. No new fuzz or hosted/platform run is claimed.

Root's receiving receipt verifies seven exact backend sources and nine UI/test/doc files
against the final producer handoff3b72be86…da27f. It preserves first handoff80278b68…14b0c
and the actual RED. Independent final review receipt712fed0f…d88c7 retains its original
captures. Sources/commands/logs and protected-input identities are recorded in ignored
target/sdf-record-remove/ and each isolated worker target. These are disposable build
records; the committed tests and documented commands are the reproducible evidence.

## Actual macOS application

Root75073 builds the final binary successfully. CUA launches the ignored bundle
`target/sdf-record-removal-native/Bondline.app`, identifier
`org.bondline.sdf-record-removal-review`, observed PID58926. Executable21,990,344B/SHA256
55c516d02118a2703e6609c16f12a27b40bcf5df98eab0773ac722ff152be285.
The build receipt pins all relevant source hashes. This launch has no control channel.
The user explicitly selected opening Bondline on the desktop.

Actual native keyboard/pointer/chooser/AX and painted window checks:

1. Cmd+O opens the original740-byte ordinary CCO/CN fixture copy through the chooser's
   SDF decoder. Records shows410-byte V3000 and330-byte V2000 entries.
2. Remove record1 opens explicit confirmation. Keep record closes it and retains both
   records/740 bytes/Saved. Reopen confirmation and Confirm removes only that record;
   the UI reports one330-byte V2000 record/Unsaved changes, with its removal disabled.
3. Cmd+Z restores two records/740 bytes/Saved; Shift+Cmd+Z restores one330-byte record.
4. Shift+Cmd+S, actual filename entry and Save create `removed-record-λ.sd`. Cmd+O
   selects/reopens that file. The UI reports Saved/one330-byte V2000 record.
5. F5 and pointer pan show the surviving H2N/CH3 molecule at100%. Inspect record1 fields
   and expanding NOTE show exact `methylamine`,12bytes/one line. That window stays open.

Independent filesystem comparison proves the actual saved file is exactly the original
literal suffix410..740,330B/SHA256
2ef1e06b09acbec94c431117d5346073a391477125eff2b7823c1c9015aac8d1.
The740-byte input remains unchanged, SHA256
e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39.
The unchanged external SDF scanner verifies one record and its NOTE; unchanged external
MOL/RDKit consumer reports CN/CH5N with exactly C,N and one Single bond. These are external
consumer results, not a delivered Bondline formula or canonicalization feature.

Two manual check setup mistakes did not change product/oracles: an extra post-delimiter
blank was incorrectly expected after exact byte equality had already passed, and resolving
the venv interpreter symlink selected base Python without RDKit. The corrected check uses
the literal existing pinned venv path and original suffix bytes. Both independent consumers
then exit0; native-file-receipt.json records the corrections and final observations.

Native removal of another ordinal/three-record source, changed-draft branches, race/close/
poison/same-packet ordering, hardware DPI, wheel and Windows/Linux remain unverified here;
their specific App/library evidence does not replace platform execution. This is a bounded
Mac workflow, not complete native parity. General drawing, record insertion/reordering,
field-header editing, broader chemistry and full M0–M5 remain outstanding.
