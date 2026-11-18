# SDF record-removal safety review

Review of base `379fed0` plus exact frozen inputs, followed by one explicitly authorized
private regression. No native UI, Git, Actions or implementation fixes were performed
by this reviewer. Prior command acceptance evidence remains frozen in its separate
worktree. One material qualification gap was reproduced and is resolved on the exact
fixed snapshots below; this review does not establish universal correctness or native parity.

## P2: file context is lost between confirmation replay and worker preparation

`crates/ui/src/sdf_editor.rs:276–314` correctly checks the confirmation's complete
source stamp, file path/format/UI-file epoch, Open close state, absent file request,
selected record and nonce. However, its queued `RemoveInput` retains only the source
stamp, close epoch and command parameters. `start_pending` at849–891 calls
`prepare_ui_sdf`; the locked checks in `crates/mcp/src/work.rs:221–254` validate source
and close context but have no captured file context or absent-request condition.

Concrete source-traced interval:

1. Confirm removal while the frame holds the Engine mutex; a valid candidate intent is
   queued and the confirmation is retired.
2. After frame unlock and before `start_pending` obtains the mutex, another registered
   `ui.file` request changes the file epoch, or a direct successful SDF Save associates a
   different path. Neither necessarily changes authored revision, source or close epoch.
3. The existing preparation checks still accept removal from the old confirmation.
   A new file request may then be invalidated by that mutation despite having passed
   the UI's normal no-file-request gate when it was created.

This is a missing pre-spawn qualification, now reproduced at the real worker-spawn
boundary. It is not a claim that removal itself writes disk. Retain the confirmation's file context in its
queued intent and check it under the same preparation mutex before worker reservation.
Keep this removal-specific; existing field-edit/finish policy need not change.

The added private test queues the real confirmation, releases the frame mutex,
interposes shared `ui.file` Save(epoch1), then calls actual `start_pending`. It requires
no job/permit, unchanged complete source/history/request and an honest stale-context
error. This fills an interval absent from the five frozen lifecycle tests. An epoch
A→B→A or direct association-change variant remains optional additional coverage;
neither was executed here.

Worker31582 actually compiled and ran that one test against the captured implementation:
exit101,0 passed/1 failed, at the assertion requiring no worker spawn. The real stale
intent was accepted and a job installed. Lower source/history assertions were not reached,
so this failure proves unexpected preparation/spawn, not an observed completed deletion.
The finite RAII cleanup cancels and joins any unexpectedly spawned real worker. There
was no compiler/setup failure or future-API stub. Exact pre-run source:
`crates/ui/src/sdf_record_removal_prepare_tests.rs`,3,736B/SHA256
`d641a6460662544c793b91d99bc47f996dc21e9e0564f7cf2690b904214de292`.
Log: `target/sdf-record-removal-review/prepare-context-runtime-red.log`.

## Other reviewed paths: no additional material finding

- Confirmation nonce is checked at replay; queuing consumes the confirmation. Old
  Confirm/Keep actions cannot retarget a later confirmation. Sole-record UI refusal
  matches the shared command's existing nonempty-SDF error boundary.
- `request_discard` after successful Keep is bounded and shows no duplicate activation
  or pointer fallthrough defect in the pinned SDK. `egui::Context::run`790–823 obtains
  each pass via `new_input.take()`; `RawInput::take`116–130 moves events out on the first
  pass. Thus the follow-up layout pass does not replay pointer/key/AX activation events.
  Keep has already cleared its confirmation and `begin_frame` clears action intents.
  This reasoning covers the current `Context::run` path; it is not a new runtime probe.
- Draft Apply/Discard/Cancel remains a separate transaction before fresh removal
  confirmation. Apply failure keeps the draft/source and starts no held action. Successful
  Apply refreshes the query and builds fresh confirmation context; its earlier source
  stamp is not silently installed as authority for removal.
- Pending removal clears query/page/reveal/pagination state and cancels an older query.
  Both query retirement and later source/ordinal changes reject old keys. Completion
  clears record-derived state again, so old duplicate-field values cannot attach to a
  survivor with its new ordinal1.
- Confirm→Save event replay is ordered. Pending/worker mutation holds files and close;
  only a current committed result with unchanged destination context resumes the held
  File binding. The actual independent App case checks the complete315B survivor on
  disk, rather than merely observing a Save receipt.
- Late cancellation never hides an already committed mutation. Published results remain
  busy until `JoinHandle::is_finished` and join/receive retirement; current success gives
  truthful removal information, historical success starts no waiting action, and poisoned
  Engine cleanup still retires the thread. Close/epoch changes after registration keep
  the existing shared setter finish policy; this review requests no stricter new policy.
- The old Apply lifecycle helper changes only the pending enum destructure and
  `MutationKind::Field` initializer. Removing those adaptations reconstructs its baseline
  bytes exactly. `view_tests.rs` retains its old prefix with one appended child bridge.

## Captured evidence and limits

All four frozen independent App cases and all five finite removal lifecycle cases were
read in full, including Keep→fresh confirmation, raw survivor/Save/reopen, draft decision
variants, same-packet Confirm→Save, old nonce/close ABA, last record, late cancel,
historical source replacement and poisoned cleanup. The two adapted prior Apply cases
were also traced. Producer195/0/1 and independent/Root four-case results are supplied
execution evidence; they were not rerun by this reviewer and do not cover the finding.
Only the added focused private regression was executed locally. The original source-only
report is retained as `target/sdf-record-removal-review/source-only-report-before-runtime.md`.

Exact manifests were verified before copying into ignored review inputs:

- UI manifest SHA256 `17aedb0ea61fc39bb66d3f8f2e0ecee4debc4db4efdd5b2799e267ec17539025`:
  three UI production files, adapted helper, five-case lifecycle file, four-case App
  file and its bridge. The reviewed editor is36,795B/SHA256
  `964e60187d9426b0fc1550fac62ec6648a6f09eca5eedaf2e5fdbbc3f9a7b089`.
- Root six-backend manifest SHA256
  `ae51112fc080f92201486d9c9c203855acbdb980a7fbdd726801944ac1089ffd`:
  actual IO deletion, shared command/capture/file routing and native-work whitelist.
- `target/sdf-record-removal-review/input-receipt.json` records all13 copied identities;
  `context-receipt.json` records nine additional unchanged context inputs, pinned egui
  multipass source and the exact helper/bridge reconstructions.

For that authorized runtime reproduction, only the13 already captured files were copied
into the isolated checkout with their original checkout bytes archived; no producer
expectation changed. The original editor prefix is exact, followed only by a cfg(test)
child bridge. `runtime-input-port.json` records that promotion separately from the
immutable review captures. The final ignored review receipt records all hashes. A fix
requires separately received production identities and an unchanged regression run.
## Resolution on separately received fixed snapshots

Root's opaque `UiSdfRemoveContext` captures source stamp, close epoch, path, format
and file epoch. The UI creates it only after the existing full confirmation/nonce
check succeeds, then retains it across busy retries. The dedicated
`prepare_ui_sdf_remove` rechecks those fields and absent UI file request under the
same originating Engine mutex, **before** worker reservation and `Engine::prepare`.
The generic native SDF method now allows only query/field editing, so removal cannot
bypass its dedicated qualification through that method. Shared command finish,
query/field behavior, late-success reporting and thread-retirement logic are unchanged.

Exact fixed inputs, separately archived by `context-fix-port.json`:

| File | Bytes | SHA256 |
| --- | ---: | --- |
| `crates/mcp/src/work.rs` |49089 |`71a72453bbf38357bb91482770455f461ca577aa49daa22cddc14783f492d4f0` |
| `crates/mcp/src/lib.rs` |5530 |`f786a4592d4280c13ce9e4d7a4b2b201050b6414a69369e12bc23353ae67c276` |
| `crates/ui/src/sdf_editor.rs`, including private child bridge |36765 |`c5c7e33efde1447d09bf10fe4a395306d5591d4f16e1f9d561c1c00b2193f6ca` |
| `crates/ui/src/sdf_record_removal_lifecycle_tests.rs` |12282 |`ed2b23da754aa43e8bf04f46c396b6dbfd03693e4c48a57f4e4131894e9172ae` |

The lifecycle helper changes only its private preparation call to the specialized
route; all five bodies/assertions and the four independent App cases remain unchanged.
The added private regression remains3,736B/SHA256`d641a646…` byte-exact.
Worker68063 runs that exact test against the real fixed backend: exit0,1 passed/0
failed/146 filtered. It reaches no-job/no-worker-reservation, retired pending intent,
truthful stale-context error and complete unchanged source/state/history/file-request
assertions. Log: `prepare-context-runtime-green.log` beside the preserved original RED.
Worker51297 also exits0 for `cargo clippy --locked -p bondline-ui --all-targets --
-D warnings` and `cargo fmt --all -- --check`; their scoped logs are retained beside it.

The source trace and unchanged runtime regression resolve the reported finding.
No additional material defect was found in this bounded review. Direct association
change/epoch-ABA variants, native interaction and full integration were not executed
by this reviewer; broader correctness remains bounded by the stated coverage. The
original RED report/receipt and all original captured files remain immutable.
All owned jobs are terminal. Final reviewed-input/report/log identities are recorded
in `target/sdf-record-removal-review/review-final-handoff.json`.
