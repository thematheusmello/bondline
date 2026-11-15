# Status — 2026-11-15

The full M0–M5 goal remains **active**. Branch `feat/multiple-bonds`, source checkpoint
`324473d`. This atomic slice delivers typed native SMILES query preparation and its
background/guard/authority tests. The actual inspector UI is separate ongoing work.
Fit`324473d`, renderer`06ab5a9`, field editor`7b64604` and plan
`af7fab0` remain checkpoints. No push/merge/publication. M0's desktop/GPU
license blocker is resolved; full M0 and M1 remain incomplete.

GitHub Actions remains disabled by user instruction: repository Actions off and workflow
417397204 disabled manually. Do not enable/dispatch/rerun or push to manufacture hosted
checks. Restore only when the full project is complete or the user changes that policy.
Earlier hosted results do not verify current source or interactive Windows/Linux workflows.

## Current delivered state

SDF Fit now displays `Fit`, with no fabricated numeric AX value, before/after prepared
bounds and while a picker disables input. Explicit100 Enter/blur still commits Actual;
numeric accessibility and known Drawing percentages retain their behavior. Five actual-
App regressions pass; native read-only AX/painting confirms enabled and disabled Fit
and restored100% structures. Native percentage typing remains unverified in this build.
Legacy physical-margin Fit and logical relative-step math remain distinct above1x.

The registry remains28 typed commands shared by UI, CLI, stdio MCP and authenticated
loopback. Complete source-backed snapshots/history, guarded CDXML/CDX/MOL/SDF/SMILES
files, saved/original distinctions and strict source authority stay intact.

Typed `prepare_ui_smiles`/UiSmilesWork now prepares the existing selected-record query
under the origin lock with full source/Open/close guards and the same pre-spawn two-slot
reservation. Real background completion, exact49-byte source/history, close/source ABA,
shared capacity/cancellation and no Save/native-ACK authority pass three receiving cases.
This is a verified library bridge, not an executed native SMILES inspector. Its original
plan and scoped evidence are now in docs/research/smiles-native-workflow-plan.md and
docs/verification-smiles-ui-work.md. No new wire command or parser/dependency changes.

New source-qualified SDF Actual/Fit/Zoom and pan use that same camera. Actual/explicit
percentage needs no fabricated bounds; successful current-view capture derives every
record and prepares only Weak source identity + PageSize. Pan/relative-from-Fit require
current bounds. Errors/stale/cancelled/foreign/changed-close work cannot install. SDF
stays document:null; no derived Page, source lease, history or edit/Save/ACK authority.
`render.capture.view` is optional/closed; absent/null retains exact legacy whole-page
pixels. Physical output and logical percentage are distinct; fresh glyphs are generated
at actual density, not enlarged from old masks. All old source/glyph/ink/budget guards stay.

Default native Fit keeps its legacy request. Explicit zoom uses current-view capture.
Keys include view epoch, source, close, actual rectangle/physical dimensions/DPI; ABA cannot
reuse a stale image. Source/close/view validation and shared two-worker reservation happen
before spawning. A Rust-only first-bounds notification is omitted from JSON. Actual
publication/decode/thread-retirement/reply barriers protect shutdown. Collection input
runs before painting/scheduling; pan/wheel shares Drawing math and focus/modal/paper gates.

Existing SDF occurrence inspection and explicit temporary-draft editing remain. Apply/
Discard/Cancel, same-packet Apply→Save, source recovery, exact no-op/history and real-worker
cancellation behavior retain their earlier evidence. New App coverage connects NOTE Apply
at100% to a fresh equal-chemistry preview, actual749-byte Save and reopen. Source/molecules
are exact; record/field creation/removal/rename/reorder still missing.

Existing Drawing remains bounded Single/centered Double/Triple carbon create/extend/order,
1/2/3, complete history, editable native CDXML, original light workspace and camera. Rings,
general labels/selection/transforms, styles, chemistry and full reference screens remain.

## Verification

- Latest Root69771 full pinned-Python local gate passes **1024/0/1**,150 result lines:
  rustfmt, all-target warnings-denied clippy, layers, licenses and corpus all pass.
  Log`target/smiles-ui-work/root-ci.log`,123009B/SHA
  `9c54faf30b72d42d4d7503391792594f29ebd829274f8cb0b080ea7dfde14bc7`.
  Root46504 receiving missing-API compile RED and61613 three-case GREEN preserve exact
  worker production/test pins. Independent full MCP64/0/0/clippy/fmt pass. Two reviewed
  new-test setup corrections are distinct from the compile RED and product behavior;
  no old API/test/oracle or unrelated state assertion was weakened.
- Prior Root77768 pinned-Python `CARGO_INCREMENTAL=0 cargo xtask ci`: **1021 passed,
  0 failed,1 ignored**,149 result lines. Fmt, all-target warnings-denied clippy, layers,
  licenses and corpus pass. Log`target/fit-readout/root-ci-final.log`,122094B/SHA
  `fe1bf7c0a94e167d6c57f4da55b221fdfcb24db8fd24c77b7869c49c8976b0ee`.
  Five independent/receiving readout cases preserve earlier prefixes. Unknown-Fit and
  disabled-picker runtime REDs are distinct from the new menu-locator and formatter
  setup corrections; no old oracle, fixture, golden, SDK or dependency changes.
  See docs/verification-fit-readout.md and its independent tests/design review.
- Root77879 pinned-Python `CARGO_INCREMENTAL=0 cargo xtask ci`: **1016 passed,0 failed,
  1 ignored**,149 result lines including3 macOS AccessKit bridge tests. Rustfmt, all-target
  warnings-denied clippy, layers, licenses and corpus pass. Log`target/sdf-view/root-ci-final.log`,
  121562B/SHA`46eae4838170cca5da3621624ba4ee422ab1a7d89d47a6be2ba0798ca0c69294`.
  Earlier91855 passes1015/0/1 before one additive source-aware Drawing case.
- Renderer: eight independent density/identity/geometry/font/CPU-GPU/error cases; all84
  renderer tests, clippy/fmt pass in isolation. Exact1x and old default/golden stay intact.
- Seven independent command cases pass Root59767 and worker62216: literal1x PNG/raw/file,
  exact1368pt twelve-record views, off-camera unsupported content, schema/null/default,
  stale/source/view/close/foreign/cancel and no source/Save/native-ACK authority.
- Four actual-App cases pass independent84986 and final Root gate: native-control-style
  Actual/pan/wheel, fresh view/DPI/source ABA uploads and NOTE Apply→fresh preview→real
  Save/reopen. Earlier6273/14105-byte test prefixes stay exact. Root whole UI7759 passes
  122/0/1 before the two additive cases. App frames are not native OS interaction evidence.
- Root22454 passes two real native-adapter/loopback cases: first extent wakes host once
  without public metadata leak; obsolete close epoch refuses before work/reservation.
- One additive ordinary-MOL current-view case passes its first run at1x/2x with independent
  Pillow regions, complete model/source/stamp retention, exact actual Save and unchanged
  external molecular consumers. It is postimplementation coverage, not an invented RED.
- An actual CLI stdio-SDK initialize/discover/Open/Actual/capture/pan/capture/stale/inspect
  check exits0, discovers28 tools/new schema, preserves full state and matches the actual
  independently checked13044-byte CLI PNG. This is a manual adapter check, not an added
  automated/native test; receipt is under`target/sdf-view/manual-stdio-receipt.json`.
- Genuine Actual/menu/capture/wheel/notification/missing-API REDs remain separate from
  new-oracle H-position and after-Records viewport setup corrections, one explicit old
  SDF Actual capability-contract upgrade, and named-routing clippy refactor. No suppression
  or weakened old input/golden/oracle. Core peer review finds no material defect in its
  exact20-source scope; changing UI/native behavior is outside that review.
- No dependency/lock/SDK/font/original fixture/oracle/approved-golden change. Runtime
  declarations remain154/165/220 and2 assets. Corpus remains8 files/5 bundles, with
  foreign/general CDX and general chemistry comparisons missing. No new fuzz, hosted run
  or interactive Windows/Linux validation. Historical listener-rebind flakiness is not
  explained away by later passing runs.

[Integrated scope](docs/verification-sdf-view-integration.md), [commands](docs/verification-sdf-view-command.md),
[App cases](docs/verification-sdf-view-ui.md), [renderer](docs/verification-view-raster.md)
and [core review](docs/research/sdf-view-core-review.md) retain exact sources, failure
classification, corrected declarations, receipts and bounds. Earlier field/drawing/file
records retain their own scopes; no milestone ends.

## Latest actual native build

Final ignored bundle`target/fit-readout-native/Bondline.app`, identifier
`org.bondline.fit-readout-review`, PID16278. Executable21773960B/SHA
`21efac416ddd1d2933da1d6bdb61b8b0d33aa495e15726acaec78a9a66b1445a`.
Root23857 builds exact final UI4b9342af…7dc72. Native AX and screenshot confirm Fit
while the real accepted picker disables Zoom. Own authenticated registry Cancel/Actual/
capture/pan restores both ordinary structures; revision1/clean740 bytes/Undo0Redo0,
100%/center100143/viewepoch2 remain. No OS mouse/keyboard input was injected while
availability is pending. Earlier enabled/cached Fit evidence uses initial PID84150,
closed through its registry. Immediate restart11741 vanished before diagnostics;
later16278 reaches startup readiness and works. Cause remains unknown.

The following is the earlier native SDF navigation checkpoint, retained separately:

New ignored bundle`target/sdf-view-native/Bondline.app`, identifier
`org.bondline.sdf-view-review`, PID51412. Executable21770456B/SHA
`938289e6371421cc61260758d2c7404e5b3d9e14612d359589622c1e0a494adf`.
Root32589 build passes; build receipt records exact source hashes. The user's explicit
instruction authorized desktop opening; no later availability answer is invented.

Real keyboard/chooser opens original740-byte ordinary CCO/CN copy. Initial Fit honestly
refuses its em guard. Native F5 produces actual100% chemical preview and pan controls;
pointer sliders set horizontal100/vertical143, visibly displaying both chemical records,
OH/H2N labels and original bond directions. Read-only inspection confirms revision1,
clean740 source, Undo0/Redo0, viewepoch3 and exact all-record A4 extent. It did not navigate.
Input remains exact SHA`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.

Numeric slider SetValue(string100) made no observed change; pointer pan works. Cause and
full accessibility remain unestablished. CJK filename pixels show question marks while
AX/path bytes are correct; broader font coverage is missing. Unprepared Fit still has the
old100% fallback readout; explicit Actual is proved by state/F5, not that label. Physical
monitor DPI was not measured, so headless1x/2x is not native hardware-DPI evidence.

A CUA scroll was aborted before dispatch; read-only observation finds no change. Native
input stopped there, leaving both structures visible. Availability clarification is
pending: do not treat elapsed time as an answer. Native wheel, combined field-edit/preview/
Save/reopen, close/cancel races and physical-DPI acceptance remain unverified in this build.
The new combined App workflow's exact749-byte NOTE splice/hash54cdb672…7b61d is recorded
in its verification; earlier788-byte native editor Save/reopen evidence remains valid.

## Missing, wired and next exact work

Full M0 still needs inventory completeness beyond576 anchors, broader dialog/control
addressability, numerical accessibility and interactive Windows/Linux. Full M1 still needs
foreign/general CDX, broader CDXML/model edits, wider MOL metadata/layout, SDF record
operations, native SMILES/conversion/export and general layout/chemistry. M2 general
drawing/selection/labels/styles, M3 chemistry/reactions/text, M4 publishing/clipboard and
M5 naming/templates/biopolymers/WebAssembly/installers remain required. No full26.1 parity.

1. If the availability reply permits desktop work, resume on latest PID16278 with fresh AX state:
   real wheel movement/restoration; NOTE edit at100%/pan; Unicode Save As/reopen and raw
  749-byte comparison plus unchanged per-MOL consumers. Literal expectation is
   `source[..380]+b"reviewed ethanol\n"+source[388..]`, preserving the second NOTE line.
   Otherwise continue independent M1 work offscreen; this does not block the goal.
2. Fit presentation is corrected. Relative steps against the visible high-DPI legacy
   overview remain separate: physical margins differ from logical fit math. Preserve
   default pixels/readability and do not choose camera policy from cache presence.
3. Continue the original readonly native SMILES record inspector and remaining M1/M0,
   then M2–M5. The typed registered bridge and three tests are integrated; five independent
   App cases are frozen and UI implementation is active in isolated worktrees, not
   delivered in this commit. Read docs/research/smiles-native-workflow-plan.md and
   the UI owner's frozen handoff before receiving. Existing source query/file commands remain
   authoritative; no new wire command, chemical depiction or converter shortcut. Keep
   reference roles separate from original policy and missing/wired/verified evidence.
   Numeric AX cause needs a bounded actual-type/action investigation before a patch.
4. Rust1.94.1; select `.cache/oracles/format-python/bin/python`
   (RDKit2026.09.2/NumPy2.5.4/Pillow12.3). Source-cite and test small buildable slices,
   commit locally, never push/enable Actions without a changed user instruction.

## Live handles

All Root Cargo/build checks and Fit tasks/review are terminal; no fuzz or launch waiter.
Native16278 and earlier51412 above; earlier19318(editor),26649(inspector),41233(ordered file),
30298(view navigation),83437(bond menu) were left untouched. Check actual processes and
availability before future native input. Tokens stay only in private ignored startup
stdout; never print/commit/publish. Build/receipt artifacts under target/ are disposable.

Workers used isolated `bondline-view-raster`, `bondline-sdf-view-command-tests` and
`bondline-sdf-view-acceptance` worktrees ataf7fab0, absolute own targets, incremental0/debug0.
Root owns integration, Git and native input. Do not share Root artifacts with staged
source exports; historical mixed-cache corruption stays quarantined/documented.

Next M1 tasks: `/root/native_acceptance` implements UI in isolated
`bondline-smiles-record-workspace` at976ef42, with only its frozen new MCP bridge backing.
`/root/sdf_setter_review` has frozen five independent App tests in
`bondline-smiles-record-ui-tests`, waiting for real source port; no live tests there.
Its first four cases are intended missing-controls RED, fifth is baseline-positive privacy.
The earlier `bondline-smiles-native-plan` bridge handoff0399f8c8…6f523 is received with
three focused cases and full MCP64/0/0/clippy/fmt passing plus the Root1024/0/1 gate.
Those do not establish native SMILES acceptance. Do not restart/duplicate live UI work.
