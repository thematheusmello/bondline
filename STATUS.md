# Status — 2026-11-15

The full M0–M5 goal remains **active**. Branch `feat/multiple-bonds`; parent checkpoint
`778ea4755fc99a92dbfce6f9ed7e9ab4ca3fe0c8` (`feat(ui): inspect ordered SDF field occurrences`).
This atomic slice adds transactional editing of an existing SDF field occurrence, its
independent tests, source decisions and finite native evidence. No push/merge/publication.
M0's native dependency-license blocker is resolved; full M0 remains incomplete.

GitHub Actions remains disabled by user instruction: repository Actions off and workflow
417397204 disabled manually. Do not enable/dispatch/rerun or push to manufacture hosted
checks. Restore only when the full project is complete or the user changes this policy.
Earlier hosted results do not verify current source or interactive Windows/Linux workflows.

## Current delivered state

The registry remains28 typed commands shared by UI, CLI, stdio MCP and authenticated
loopback. Complete source-backed snapshots/history, guarded CDXML/CDX/MOL/SDF/SMILES
files, original/saved baseline distinctions and strict source authority stay intact.
README/file-formats/parity distinguish bounded support from general chemistry/full parity.

The original SDF inspector now offers Edit for an explicitly returned existing occurrence.
A65536-byte bounded temporary draft changes no source/history/dirty state until Apply.
Apply calls the existing registered setter through UiSdfWork and creates one history edit,
or a complete-state no-op. No new pool/command/schema/Save authority or dependency.
Empty/multiline values, duplicate order, DT digits and every unrelated source byte remain.
Record/field creation, removal, rename and reorder are still missing.

Unapplied drafts offer Apply/Discard/Cancel across navigation, File and close intents.
Both real input paths order Apply before later Save. Continuations resolve fresh bindings
only after the actual result and thread retirement; failed Apply starts no old-byte Save.
External Open, including identical bytes, detaches and retains recovery text. Successful
late-cancelled edits remain applied and are truthfully reported. Pending/live jobs, draft
resolution and existing file/executor/reply barriers protect final close acknowledgement.
The previous read-only inspector's omission/selection/text-storage protections remain.

Existing drawing/view behavior remains: bounded Single/centered Double/Triple carbon
create/extend/set_order,1/2/3, complete history, editable native CDXML, original light
header/tool rail/document inspector, numeric zoom, Actual/Fit/Magnify/Reduce and pan/wheel.
View changes never resize source paper or change history. Rings, general labels/selection/
chemistry and full reference screens/flows are still missing.

## Verification

- Root44537 `BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
  CARGO_INCREMENTAL=0 cargo xtask ci`: **994 passed,0 failed,1 ignored**,147 result lines,
  including3 macOS AccessKit bridge tests. Rustfmt, warnings-denied all-target clippy,
  layers, licenses and corpus pass. Log`target/sdf-editor/root-ci-nine-pinned.log`,118847B,
  SHA`ac370a87068394ed5f73e33bb1834273643bba270674d354f7a91ee75c46eca7`.
- Nine independent actual-App cases pass; exact second-DUP788-byte Save/Undo/Redo/reopen,
  navigation decisions, same-packet Apply→Save, external/identical-byte Open, real close
  requests, CRLF no-op, invalid-line nonpublication, refreshed Previous-page navigation
  and externally requested Save. Old31,003/14,341/7,908-byte test prefixes remain exact.
- Eight bounded TextBuffer cases and two finite real registered-worker gates pass. Gates
  distinguish actual commit/publication from thread retirement, protect close, truthfully
  report late cancellation and preserve replacement source/recovery text.
- Missing Edit, lost Previous and missing late-cancel notice have actual runtime RED/GREEN.
  Incorrect filter/API/import/path/environment setup and two narrow test-lint corrections
  are recorded separately. An omitted BONDLINE_PYTHON caused two unchanged RDKit imports
  to fail; correcting only the invocation produced the full pass above. No test was weakened.
- Dependency/lock/font/SDK/fixture/oracle/golden bytes are unchanged. Runtime declarations
  remain154/165/220 for Mac/Windows/Linux and2 assets. Corpus remains8 files/5 bundles;
  foreign/general CDX and general chemistry comparisons remain missing. No new fuzz,
  hosted run or interactive Windows/Linux execution occurred.

[Integrated evidence](docs/verification-sdf-editor-integration.md), [App cases](docs/verification-sdf-editor.md),
[buffer](docs/verification-sdf-draft-text.md), [worker gates](docs/verification-sdf-editor-lifecycle.md)
and [lifecycle plan](docs/research/sdf-edit-lifecycle.md) retain exact sources, RED/GREEN,
corrections, ignored receipt identities and limitations. Earlier inspector/view/bond/file
records retain their original scope. The historical listener-rebind AddrInUse remains an
unexplained prior flake; unchanged later full runs pass, not proof that it cannot recur.

## Latest actual native build

New Mac/Metal executable21,656,392B/SHA
`be69b1d42682b3800b7bac2a2d12a4d30ca54d377362d34b381b1374604d72ee`,
ignored bundle`target/sdf-editor-native/Bondline.app`, identifier
`org.bondline.sdf-editor-review`, PID19318. The user's explicit desktop-open instruction
was followed; no new availability reply is claimed. Older preview sessions are untouched.

Actual chooser/controls: open777-byte mixed SDF copy, inspect record1/second DUP, Edit,
native multiline input, Apply, Unicode Save As `révision-β.sdf`, CmdZ/CmdShiftZ, reopen
and inspect. Actual saved bytes exactly match the predeclared788-byte literal splice,
SHA`e63cb9adf5cbc62b5b0be94992246c0d10b94cda405e49b9514993e7d9f9c361`.
Both actual saved MOL spans pass unchanged pinned external V3000/MOL consumers; unrelated
raw data is proved by full-byte comparison. The original777-byte input remains exact.
Final source after reopening is788 bytes, revision5, clean, Undo0/Redo0; query-only
loopback inspection confirms it and did not perform the edit.

The accessibility-index draft click did not give observed typing; a coordinate click on
the visible line did, then native paste/Apply succeeded. Native general focus/setter,
draft/cancel/close races and huge values remain unverified. Default native chemical preview
still refuses its8..256-em guard. No oversampling, source-paper change, H/record omission,
font/fixture/oracle/golden change or new human approval was used to claim success.

## Missing, wired and next exact work

Full M0 still needs inventory completeness beyond576 anchors, broader dialog/control
addressability and interactive Windows/Linux evidence. Full M1 still needs foreign/general
CDX, broader CDXML/model edits, wider MOL metadata/layout, usable native SDF chemical
preview and record operations, native SMILES/conversion/export and general layout/chemistry.
M2 general drawing/selection/labels/styles, M3 chemistry/reactions/text, M4 publishing/
clipboard and M5 naming/templates/biopolymers/WebAssembly/installers remain required.
No milestone or full26.1 screen/flow parity is complete.

1. Resolve native SDF zoom/pan through the same registry/shared projection, with measured
   logical/physical-DPI behavior. The exact all-record virtual extent is derived off-lock;
   never invent A4 for taller data, parse during painting or convert source into a Drawing.
   Preserve default whole-page captures/golden, full-source guards and em limits. Add an
   independent1x Actual/Zoom recovery test before code; cache/source/error semantics need
   an explicit decision. A bounded research proposal is prepared in the buffer worker.
2. Add separate native draft/navigation/cancel/close acceptance when desktop availability
   permits, including actual recovery/focus behavior. Do not call synthetic App input a
   native OS race check. Keep current field-edit evidence finite and unchanged.
3. Continue remaining M1/M0, then M2–M5. Source-cite all decisions; retain missing/wired/
   verified scopes. Commit small, buildable slices locally; no push or Actions.
4. Rust1.94.1. Pinned Python is `.cache/oracles/format-python/bin/python`
   (RDKit2026.09.2/NumPy2.5.4/Pillow12.3). Preserve original inputs/consumer hashes and use
   meaningful scoped checks followed by the required local aggregate gate.

## Live handles and worker ownership

All Root Cargo/build checks and three independent worker tasks are terminal. No fuzz or
launch waiter. Native PID19318 above; earlier PID26649 (inspector),41233 (ordered file),
30298 (view navigation),83437 (bond menu) were left untouched. Check actual processes and
user availability before new desktop input. Private tokens stay only in ignored startup
stdout; never print/commit/publish them. Ignored build/receipt artifacts are disposable.

Workers used isolated `bondline-sdf-edit-ui`, `bondline-sdf-edit-lifecycle` and
`bondline-sdf-draft-buffer` worktrees with absolute per-worktree Cargo targets, incremental0
and debug0. Root owns product integration, Git and native input. Never share Root artifacts
with staged source exports; earlier mixed-cache corruption remains quarantined/documented.
