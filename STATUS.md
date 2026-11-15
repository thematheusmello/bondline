# Status — 2026-11-15

The full M0–M5 goal remains **active**. Branch `feat/multiple-bonds`. Integrated feature
checkpoint`f44bfa9` adds the shared SDF removal command, native UI, independent tests and
evidence; codec checkpoint`e5c11bb` precedes it. Previous checkpoint`379fed0` contains the
SMILES inspector. The tree was clean after feature integration. Do not restart these
features. No push, merge or publication.

M0's native/GPU license blocker is resolved; full M0 and M1 remain incomplete.
GitHub Actions is disabled by user instruction (repository Actions off; workflow417397204
manually disabled). Do not enable/dispatch/rerun or push to manufacture hosted checks.
Local gates govern development; earlier hosted results do not verify current source or
interactive Windows/Linux workflows. All new research follows the supplied public-source
AGENTS.md boundary; never inspect/execute proprietary reference program binaries/assets.

## Newly delivered and verified

**Whole SDF record removal:** source-preserving codec,29th typed command, CLI, stdio MCP,
authenticated loopback and original Records UI. The command removes one complete raw
record and reindexes surviving local ordinals, with one complete-source history edit.
Original/current/saved distinctions remain; a survivor stays SDF/document:null. Removing
the sole compound refuses per CTFILE-2026 printed97; missing target takes priority.

The UI resolves changed drafts before fresh confirmation. Keep record is a no-op;
Confirm clears stale ordinals/fields/query text. Same-packet Save waits for the actual
mutation result and thread retirement. Late cancellation retains truthful committed
success, and stale replacement cannot resume an old Save. It reuses the existing editor
job and two-worker budget; no new executor, App field, Save lease or ACK authority.

Independent review found a real gap after confirmation/frame unlock: interposed file
requests could still spawn removal. Runtime31582 reproduced it. The fix retains opaque
UiSdfRemoveContext and atomically checks source/Open/close/path/format/file epoch/no
request under the origin mutex before reservation/preparation. Its unchanged regression
passes. Generic prepare_ui_sdf remains query/field only; do not replace this guard with
a pre-call UI check. Review found no additional material issue in its bounded scope.

**Actual native Mac workflow:** user explicitly selected opening Bondline on the desktop.
The final new build passes native Open→Keep/Confirm→Undo/Redo→Unicode Save As→reopen→F5/
pointer pan→retained NOTE. The actual330-byte output equals original740-byte suffix
410..740, with unchanged external raw/per-MOL consumers. This is one bounded ordinary
mixed-source workflow, not native proof of draft/race/close/other-ordinal cases.

See [integrated evidence](docs/verification-sdf-record-removal.md),
[codec](docs/verification-sdf-record-codec.md),
[command](docs/verification-sdf-record-command.md),
[App/lifecycle](docs/verification-sdf-record-removal-workspace.md) and
[independent review](docs/research/sdf-record-removal-review.md).

## Current application scope

- Shared29-command registry, complete snapshot history, guarded CDXML/CDX/MOL/SDF/SMILES
  Open/Save and original/saved distinctions. Source authority is content-based, not suffix.
- Drawing: bounded Single/centered Double/Triple neutral-carbon create/extend/order,
  1/2/3, history and editable native CDXML. Original light workspace, paper, inspector,
  Actual/Fit/zoom/pan/wheel. General selection/labels/rings/styles remain missing.
- SDF: complete collections, explicit occurrence inspection, existing-value draft editor,
  read-only chemical preview and source-qualified camera. Native field editing and
  ordinary Actual/pan have earlier bounded evidence; record removal now adds the flow
  above. Default Fit can still refuse readability limits. Legacy physical-margin Fit
  and logical relative-step math differ above1x; no cache-based policy switch is allowed.
- SMILES: complete accepted files and original read-only ordinal inspector with exact
  notation/counts/ranges and no opaque-data disclosure. App tests verify copy/navigation/
  Unicode Save/reopen and actual worker retirement. This preview also activates native
  Next and Inspect on PID53821, displaying record2 `[13CH3][OH]`; unchanged49-byte source,
  revision1/clean/Undo0Redo0 are checked. Numeric typing, OS copy and the complete native
  SMILES dialog journey remain unverified. Chemical preview/editing remain missing.
- CPU/GPU shared scenes, PNG capture and validated limited ordinary MOL/SDF display-H.
  This is not general chemistry perception, canonicalization or full publishing.

## Verification

Root96793 `BONDLINE_PYTHON=.../.cache/oracles/format-python/bin/python
CARGO_INCREMENTAL=0 cargo xtask ci` completes **1060 passed,0 failed,1 ignored**,153 result
lines. Rustfmt, all-target warnings-denied clippy, layers, licenses and corpus all pass.
Log`target/sdf-record-remove/root-final-ci.log`,127700B/SHA256
`cbe106e2813aee252f479984047ae10653fa8b1863b8bebb7d8152657ae76525`.
The original ignored visual capture remains ignored. Root75073 native build succeeds.

Additive coverage: seven codec, six independent command, three actual wire, three native
adapter, four independently authored App, five finite lifecycle and one reproduced
preparation-race cases. Original REDs, initial full-catalogue failures and reviewed new-
test setup corrections are distinct in the evidence. Existing catalogue names/counts
advance only for the new command; old field tests/assertions and frozen four App cases
remain intact. Necessary private lifecycle fixture setup changes reconstruct originals.

No dependency/lock/SDK/font/fixture/oracle/approved-golden change. Runtime declarations
remain154/165/220 and2 assets. Corpus stays8 files/5 bundles; foreign/general CDX and
chemistry coverage remain missing. No new fuzz, hosted or interactive Windows/Linux
execution. Historical listener-rebind flakiness remains unexplained. Native manual checks
corrected an extra-EOF-blank expectation and accidental venv-symlink resolution; neither
changed product, actual expected suffix, oracle, interpreter environment or packages.

Prior1031/0/1 SMILES,1024/0/1 query bridge,1021/0/1 Fit and1016/0/1 SDF-view gates retain
their exact scopes in docs/verification.md and linked records. The sole approved render
golden remains SHA256`1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc`.
App input tests do not replace native platform execution.

## Current native preview and live handles

Open bundle`target/sdf-record-removal-native/Bondline.app`, identifier
`org.bondline.sdf-record-removal-review`, observed PID**58926**. CUA launched it directly;
no control channel is enabled. Executable21,990,344B/SHA256
`55c516d02118a2703e6609c16f12a27b40bcf5df98eab0773ac722ff152be285`.
It displays saved`removed-record-λ.sd`, one330-byte V2000 record,100% and retained
NOTE`methylamine`. Actual horizontal/vertical pointer values are101/200. Native output SHA
`2ef1e06b09acbec94c431117d5346073a391477125eff2b7823c1c9015aac8d1`;
input740 bytes remains exact SHA
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.
External unchanged scanner/consumer verifies NOTE and CN/CH5N. These formula/SMILES
results are external test evidence, not a Bondline analysis feature.

Earlier native PID53821 (`target/smiles-record-native/Bondline.app`) remains open at
record2. Its executable21,932,120B/SHA`f4d8627a75310282ca0b7eadeddd1d031a999f4d7ff8d80a923ac79684e443a7`.
PID16278 (`target/fit-readout-native/Bondline.app`) remains separate with two ordinary
SDF structures; executable21,773,960B/SHA`21efac416ddd1d2933da1d6bdb61b8b0d33aa495e15726acaec78a9a66b1445a`.
Earlier51412/19318/26649/41233/30298/83437 were left untouched, not freshly verified.
Never print/commit private tokens from earlier opt-in startup stdout.

All Root Cargo/build and three worker jobs are terminal; no fuzz/launch waiter. Workers
used isolated worktrees `bondline-sdf-record-codec`, `bondline-sdf-record-command-tests`,
`bondline-sdf-record-workspace`, `bondline-sdf-record-ui-tests` and
`bondline-sdf-record-review`, with own absolute targets/inc0/debug0. Root owns Git,
integration and native input. Frozen receipts are received; do not reset worktrees or
share Root Cargo artifacts. Current native/build receipts under target/ are disposable.

## Remaining work and next executable steps

1. The current native removal workflow is complete for its documented scope; do not
   reimplement it or infer untested native races. With desktop availability, finish
   remaining SMILES inspector acceptance on PID53821: actual ordinal typing, text
   selection/OS copy, Previous, Unicode Save As and reopen; compare all49 bytes and
   unchanged raw/per-record consumers. Do not use automatic registry output as UI proof.
2. Next unblocked M1 implementation: whole-record SDF reordering. Reread CTFILE-2026
   printed97–99 and current record spans/without_record/writer/callers. First declare
   exact literal mixed-record permutation, preserved fields/MOL blocks and old/new local
   ordinals against independent scanners/consumers. Explicitly decide/refuse the final-
   delimiter-at-EOF case before moving it ahead of another record; never silently fuse
   records or normalize unrelated bytes. Then TDD the smallest codec/shared-command/UI
   slice, including history, stale confirmation/drafts and actual Save/reopen. This note
   describes next work; no future API, implementation or passing result is claimed.
3. M1 also retains record creation, field-header editing, native SMILES conversion/export,
   foreign/general CDX, wider CDXML/MOL metadata/layout and broader syntax/chemistry gaps.
   Full M0 needs inventory completeness beyond576 anchors, broader control/dialog and
   numeric accessibility, plus interactive Windows/Linux. M2 drawing/selection/labels/
   styles, M3 chemistry/reactions/text, M4 publishing/clipboard and M5 naming/templates/
   biopolymers/WebAssembly/installers remain required. No milestone is declared complete.
4. Use Rust1.94.1 and literal pinned venv interpreter path (do not resolve its symlink).
   Source-cite and test small buildable slices, preserve data/guards, run meaningful local
   gates and commit locally. No push/publication or Actions enablement without changed
   user instruction. The high-DPI relative-Fit policy, native wheel, physical DPI and
   wider CJK glyph coverage remain open; see the view/Fit evidence before changing them.
