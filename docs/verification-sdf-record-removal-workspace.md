# SDF record-removal workspace evidence

This slice adds original native-workspace controls at `379fed0`, using Root's
six-file frozen codec/command/worker port. All execution below is headless on
macOS; no desktop window, OS input, activation or hosted Actions was used.
The four independent tests run the actual `BondlineApp`, its egui frames,
registered workers and FileWorkflow, rather than a substitute removal UI.

## Delivered behavior

Each expanded Records row has **Remove record N**. The sole record's control is
disabled, with the codec/command enforcing the same minimum independently.
The [official CTfile specification](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf),
printed97, requires one or more compounds; the source pin and original-policy
distinction are in [the frozen design plan](research/sdf-record-removal-ui-plan.md).
These controls are original Bondline UI, with no Professional26.1 browser claim.

A changed field draft must resolve through Apply/Discard/Cancel before a fresh
**Remove SDF record** confirmation. **Keep record** is an authoring no-op.
Confirmation captures full source identity, file/close context and a checked
request nonce, and rechecks these when **Confirm record removal** is replayed.
It describes removal of the structure and all data fields; it does not promise
unlimited Undo. Save remains a separate action.

Field edits and removal share one pending mutation and actual worker owner.
Confirmation and mutations block unsafe file/close adoption; accepted removal
uses the existing registry and `prepare_ui_sdf` after the frame lock is released.
Pending and committed removal clear old field ordinals, query selection and
displayed values even without a draft. No surviving ordinal is auto-selected.
A cancelled running query still retires before its owner disappears.

Ordered confirmation followed by Save in one input packet queues a fresh Save
until the mutation's result and actual thread retirement. Historical success,
failure, cancellation and changed file/close context do not start a waiting file
action. A real committed removal remains reported as committed after late
cancellation or engine unavailability; it is not silently discarded or retried.

## Test-first sequence and corrections

- Independent owner baseline33163 compiled and executed Open777→Records, then
  failed because the actionable Remove role was absent. Its initial test was
  1,198 bytes/SHA256 `5c4076a177ff5131b87b8c1b0902d05f5dabfae40cc40d4b4d14f46a4de3f8e5`.
  Owner22745 then observed the same behavioral RED in all four frozen App cases.
- Own4275 exited101 with eleven intended missing-private-API errors for the new
  confirmation/nonce lifecycle tests. This compiler RED is separate from the
  actual-App behavioral RED above.
- First real editor run24758 compiled: twelve passed, one new setup assertion
  failed before removal. The chosen `ordered-fields-v2000.sdf` actually contains
  two records. Root approved replacing only that sole-record setup with the
  independent literal mixed-source span462..777, exactly315 bytes. All refusal,
  source, state and no-op assertions remain. The failed source/log and exact
  correction receipt are retained; no product fix is claimed for it.
- First frozen App run50420 passed three and failed one: Keep cleared internal
  confirmation state but its just-painted accessible button remained in that
  final frame. The production fix requests egui's bounded discard pass on the
  valid dismissal; the final frame omits the stale modal. No App assertion,
  input or settling delay changed. Run5934 passed all four unchanged cases.
- Three further finite worker tests are additional post-implementation coverage,
  not additional claimed RED→GREEN cycles. Together with confirmation and sole-
  record checks, the five new finite tests pass. They use actual registered
  removals and finite publication/retirement channels, without timing sleeps.

The independent App file remains exactly19,026 bytes/SHA256
`911f583620bbd1188e46d1c1b18dd5f372bd20dd504fc46bc793ad66c1dcd5d9`.
Its expectations include the literal315-byte survivor, field invisibility,
Undo/Redo, all changed-draft choices, invalid201-byte Apply, Unicode chooser
Save/reopen, and confirm-release→Save in one packet. Actual saved bytes are
checked using the unchanged raw SDF consumer and per-MOL RDKit consumer; a
round trip through Bondline alone is not the interoperability oracle.

The two old finite Apply tests retain every body/input/assertion byte. Their
private helper only unwraps the Field variant and supplies the job's Field kind.
Removing those two setup hunks reconstructs the original helper file exactly.
The new App module declaration likewise reconstructs the old `view_tests.rs`.
The ignored receipt includes both proofs and the minimal helper diff.

## Reproducible local checks

Run from the owning workspace with its absolute `CARGO_TARGET_DIR`,
`CARGO_INCREMENTAL=0`, dev/test debug0, and the pinned `BONDLINE_PYTHON` described
in [development.md](development.md). No dependency, SDK, font, old fixture,
oracle or approved golden changed.

```sh
cargo test -p bondline-ui --lib view_tests::sdf_record_remove_tests --locked
cargo test -p bondline-ui --lib sdf_editor:: --locked
cargo test -p bondline-ui --locked
cargo clippy -p bondline-ui --all-targets --locked -- -D warnings
cargo fmt -p bondline-ui -- --check
```

Actual results:5934 four App cases passed;9593 sixteen editor-related tests passed
(five new finite, two original finite, nine original App editor tests).30210 full
UI package passed **195/0/1**, including **145/0/1** library tests and50 integration
tests.38081 all-target clippy exited0; package rustfmt check exited0. The one
ignored visual capture test was already ignored. No full-workspace result is
claimed by this producer; Root owns the receiving aggregate gate.

Exact source/log hashes and exit codes are in ignored
`target/sdf-record-removal-ui/handoff.json`. The879-file baseline has868 files
unchanged; the other eleven are six explicitly received backend files, three
owned UI production files, the approved old helper adaptation and the new child
bridge. The7,800-byte research plan remains unchanged. All jobs are terminal.

Native interactive removal, platform dialogs, clipboard and real desktop close
remain unverified by this slice. Headless App and source preservation evidence
does not establish cross-platform UI behavior or complete M1/M2 parity.

## Follow-up: atomically retain confirmation file context

The checks and868-file count above describe the first frozen implementation
(`handoff.json`, SHA256
`80278b685f26dcb9e84ece9efe3fbeb61e3160c01107213ec8c374a14ca14b0c`).
Independent review then found a real gap in its registration guarantee: removal
rechecked path/format/file epoch during confirmation replay, but dropped them
between frame unlock and worker preparation. An interposed `ui.file` Save could
therefore reserve and spawn removal under an obsolete confirmation context.
Review31582 reproduced that intended runtime RED before any fix. Its unchanged
private test is3,736 bytes/SHA256
`d641a6460662544c793b91d99bc47f996dc21e9e0564f7cf2690b904214de292`.
This was a product defect, not a fixture or test-timing problem.

Current removal input retains Root's opaque Rust-only `UiSdfRemoveContext`,
captured after the existing confirmation checks. Registration now calls
`prepare_ui_sdf_remove(&context, params)`. Under the originating Engine mutex,
the shared MCP helper checks source identity, Open/close epoch, exact path,
format, file epoch and absence of a file request before reserving a worker or
preparing the command. The generic `prepare_ui_sdf` again accepts query and
field edit only; its field caller is unchanged. No new registry command or wire
field was introduced.

Only the removal input/capture/registration and its new finite test helper's
registration call changed. The five removal test bodies, old Apply tests and
four independent App cases remain unchanged. Replacing that single finite
helper call reconstructs its previous full file exactly. A new append-only
private test bridge imports the independent regression without changing it.

Receiving13577 passed all17 editor-related tests, including the new pre-spawn
regression. `context-app-tests.log` passed the same four App workflows with
exit0. Current all-target clippy and rustfmt checks also exited0. The earlier
195/0/1 full package result belongs to the first frozen source; Root owns the
aggregate gate for the final source. No unrun package result is attributed to
this correction.

The final context-fix receipt is ignored
`target/sdf-record-removal-ui/context-fix-handoff.json`. It preserves the first
handoff, before/after source hashes, exact helper reconstruction, independent
RED provenance, two received MCP API files, current logs and the867 unchanged
baseline files. Native interactive verification remains outstanding; no
desktop input, Git, Actions or installations were performed. All jobs ended.
