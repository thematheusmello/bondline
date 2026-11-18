# Independent File → New App acceptance

Four original tests and two separately frozen regressions in `crates/ui/src/new_document_tests.rs` exercise the actual
`BondlineApp::update_workspace`, current accessible controls, the shared registry and
real file workflow. These are input-frame tests, not native OS input/dialog evidence.
The starting checkout is `69e4b37`. Existing production, tests, fixtures, oracles,
dependencies, locks and goldens were unchanged when the tests were frozen.

## Public role and original policy

The public [version 25 guide](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf)
records the File New role (MF-001, printed 13/39). The existing source/parity records pin
that PDF at 17,721,447 bytes, SHA256
`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
This versioned public role is distinct from a claim of complete Professional 26.1
behavior or multiple-document/window support.

Bondline's current single-window replacement, A4 default, clean source-free CDXML
state, empty history, camera Fit/default center and guarded Save/Discard/Cancel are
original policies agreed for this slice. The central control is `file.new`, label
`New Document`, primary-N, carrying `ui.file` action `new`. The native host consumes
the qualified accepted request and invokes the same `document.new` registry operation;
there is no test-created Engine replacement. The earlier proposal to bypass `ui.file`
was superseded before the expanded test freeze.

The dirty-New dialog is `New document`, with `Save`, `Discard Changes` and `Cancel`.
Changed SDF drafts first use the existing `Unapplied field value` dialog and
Apply/Discard/Cancel. Applying a draft may make the document dirty, so it requires a
fresh New decision rather than implicit loss of the applied value.

## Frozen inputs and decisive cases

The original complete mixed SDF is 777 bytes, SHA256
`165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`.
The original 49-byte SMILES vector retains its CRLF, ignored line, non-ASCII/NUL opaque
data and all three records. Its bytes are reused exactly from the existing independent
inspector test. No proprietary file or reference-program asset is used.

1. Open the complete SDF, expand File and require the actual enabled `New Document`
   button. Trigger it and require a clean Drawing with no collection/source/path,
   CDXML association and empty history. The original SDF file stays exact.
2. Open the complete SMILES file, verify the authoritative central File/New command
   and shortcut metadata, then use the actual menu to reach a fresh A4 Drawing. Draw
   a C–C bond through actual toolbar/pointer input, test dirty Cancel/Discard/Save,
   and preserve all authored data on Cancel apart from real transient `ui.file`
   cancellation metadata. Save uses the actual Unicode chooser and shared file job.
   After New, create another editable bond, save to a second Unicode CDXML path and
   reopen it through the actual file workflow. Unchanged Expat/Decimal and pinned
   external RDKit consumers inspect the actual written files for literal geometry,
   neutral carbon/Single/H3 and ethane/C2H6. The original SMILES bytes stay exact.
3. Edit surviving SDF record 2's ORDINARY draft to `reviewed\nline two`. Cancel keeps
   the local draft and source; Discard permits clean New. Apply must first create the
   independently specified 781-byte source, preserve it unsaved until a fresh dirty-New
   Save decision, then save those complete bytes before New. Unchanged per-MOL
   consumers read both actual saved molecular spans; no lossy property map defines
   whole-source fidelity.
4. On dirty SDF data, Undo then primary-N in one ordered input packet must see the
   now-clean source and create a fresh Drawing without a stale dirty prompt. Separate
   pending-New cases use explicit external Open of another source or the same original
   source again. The old intent must disappear without retargeting the replacement,
   including its ordinal/source identity. Files and replacement content remain exact.

The draft's expected 781 bytes are the literal splice `[0,757)` +
`reviewed\nline two\n` + `[771,777)`. New code never generates that expectation.
The current parser/codec/oracle profiles remain unchanged. Native file publication,
close barriers and in-flight save/request retirement have separate producer tests;
these four App cases do not fabricate private completions or add test-only APIs.

## Actual baseline RED

The first 1,472-byte test was frozen before production changes, SHA256
`f9a1031018e37eb11d3c7bab390314ed818c44861687ff9e8ffcf0ed20a906e0`.
Session 44995 compiled and ran the real App, opened the SDF and expanded File, then
exited 101: `current actionable New Document field/control missing`. This is a runtime
missing-control RED, not a missing Rust API/compiler failure. Its 7,654-byte log is SHA256
`46e598ea349bec22f90be9436fd02880fabc838e31501d06c94dfefea4596e29`.

The expanded four-case source is 21,072 bytes, SHA256
`3ce8d29517e02c43555a855eecbfc797d70104d49da5a564f4fbfab71c15eac9`.
Its initial test function is byte-exact. Session 51361 compiled and ran all four against
the unchanged baseline: 0 passed, 4 failed. Two fail at the missing actual New control,
one at missing central `file.new` metadata, and the ordered Undo→New case reaches its
finite ten-second failure bound because primary-N has no New binding. No future Rust
API, backend stub, assertion change or test setup correction is involved.

Run with the pinned external interpreter in `BONDLINE_PYTHON` and a separate absolute
`CARGO_TARGET_DIR` for this checkout:

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo +1.94.1 test -p bondline-ui --lib view_tests::new_document --locked
```

At the pre-port freeze, GREEN is pending exact real backend/UI integration. The four
tests, their original prefix, protected-input manifest and RED evidence are held frozen.
Workspace formatting passes. No native input, hosted Actions, Git or dependency mutation
was performed by this worker. Ignored `target/new-document-ui/` receipts record source,
log/protected-input hashes, commands, terminal sessions and any later receiving result.

## Reviewed new-test setup corrections

The producer's first real implementation run, 41173, passed two cases and failed two.
Both causes were independently checked against unchanged sources before any test edit:

- The first File popup initially exposes an invisible, disabled sizing pass in pinned
  egui. The central New binding was enabled; probes 25541/37501 confirmed Open state,
  no file request/job and an actual sizing-pass disabled widget. `menu.rs` 189–207 and
  `containers/area.rs` 397/548–549 document that pass. The test now waits within the
  existing finite bound for the current reachable New button to be enabled, retaining
  its original enabled assertion and all actions. It adds no fixed settling delay.
- The existing document inspector already paints `New document` for a clean path-free
  Drawing (`lib.rs` 304–308). Only the fourth case's modal-negative assertion now uses
  the agreed unique replacement prompt. A current-tree check additionally requires no
  `Discard Changes` button, including disabled buttons. All fresh-state, revision,
  byte, input-order and persistence assertions remain unchanged.

The corrected source is 22,097 bytes, SHA256
`3ca9e94ca8524aa2842009e3066c224a6179ffe5efb2351c00d25fd4f1914fd1`.
Removing exactly the approved wait and modal-locator/strengthening hunks reconstructs
the full original 21,072-byte source. The first 1,472-byte test function now has the
approved wait addition and is no longer described as byte-exact. Original sources,
both missing-binding REDs, producer 41173 and diagnostic logs remain archived. No
producer behavior was changed for these locators. Corrected GREEN still requires a
real frozen production snapshot; the independent worker has not substituted a stub.

The next producer run, 59433, passed three cases and reached an existing unsaved-data
guard in the fourth setup. The two deliberate external replacement Open calls had
omitted explicit discard permission despite creating dirty 781-byte SDF data. The
unchanged baseline Open API defaults to refusal (`files.rs` 565–571). Root approved
only `discard_unsaved:true` on those two setup calls, with rustfmt's line wrapping;
all stale-request/replacement assertions and all other actions remain unchanged.
The final frozen test source is 22,237 bytes, SHA256
`e5cadc837d32a24689e6893ad3c0d36da2a9d6dd76b3e63b7b4a0a10a682c724`.
Removing this third setup correction exactly reconstructs 22,097 bytes, then removing
the earlier approved hunks reconstructs the original 21,072 bytes. The 3/1 run and
both earlier source versions remain archived; no production change was justified.

## First independent receiving GREEN

The producer's final four-case run 41069 passed on that exact 22,237-byte test source.
Root then authorized independent receiving of its real five-entry backend manifest,
1,011 bytes, SHA256 `c72e2dc11eaa19227a726ee8c34938ef425d1587d217ef9950ca27357c14a37c`,
and the producer's seven-entry UI manifest, 1,401 bytes, SHA256
`9d36cb3fe4ef91dbacf03466c4c65b3a5315ad52c7e21e499232ebed965b5c3f`.
Each entry was verified before copying and after receiving. The independent test and
parent bridge were already byte-equal, so neither was overwritten. Ten real source/test
files were received and classified separately from unchanged inputs.

All 176 pre-port protected files were verified and archived before receiving. The 170
outside the explicit received replacements remain exact. There is no local backend
stub, source reset, generic framework, dependency or oracle change in this acceptance.

Independent session 62649 then compiled and passed **4/0/0**. No further test/action,
assertion or settlement change was made after the final approved source freeze. The
actual App/file/consumer behaviors listed above passed, including the final external
replacement and same-source Open cases that the earlier setup refusal had not reached.
Warnings-denied all-target UI clippy and workspace formatting are separate final checks
recorded in the receipt; this worker does not claim a redundant full-package gate.

No native OS input, reference-program execution, hosted Actions, Git mutation or broader
New/multiple-document parity is established. The ignored final receipt indexes every
source version, approved correction diff/inverse, original runtime RED, producer
diagnostic and independent GREEN/lint log, exact source/binary identity, protected-input
archive and terminal job handle. All older worktrees/evidence remain frozen.

## Lint correction and transition-frame regression

The first independent clippy run, 73180, exited 101: the producer's `finish_new_save`
had eight arguments. Root approved grouping its existing request ID, format and action
through the already-owned `&FileJob`; all later conditions and body remain unchanged.
Only the two exact file-workflow sources were received for this signature correction.
The original source and 6,967-byte lint log, SHA256
`bf9c927e50b7a60195409951b00fde7196666519008d9f555ce2772e70500a4d`, remain archived.

Root authorized one additive fifth case for input sampled on a collection while
Save-to-New retires at frame start. It activates the real known-path Save once, checks
Engine kind before every next frame, and sends coherent pointer drags only while that
pre-frame kind remains SDF. It stops at the first Drawing-kind frame regardless of
fragment count. It then requires empty fresh history/source and exact saved 781 bytes,
followed by working fresh next-frame authoring and independently consumed CDXML output.

The five-case source is 26,124 bytes, SHA256
`8a2221fba0af6b2f2d474b8a1b308242477e1100daad79c064a7f3ef9345aba7`.
Its first 22,237 bytes remain exact. Runtime 53938 exited 101 at the empty-fragment
assertion: the first Drawing contained an unintended C–C bond from the packet sampled
while the pre-frame content was read-only SDF. The failure precedes any fresh-frame
authoring or idle-frame concealment. This is a real transition-input product regression,
distinct from the three reviewed new-test setup corrections. The 1,785-byte log is SHA256
`7b945f39121b49b50804a96faf3a3686f0fb69241fc3f53907bb4d6ed137288b`.
At that source checkpoint, the original four workflows still pass; the fifth remains
RED pending an exact producer fix and independent receiving run.

The producer supplied a lib-only source fence around `FileWorkflow::poll`, verified at
94,963 bytes/SHA256 `aea0939d5edfc450abb95bbc791acbd6487a4f5766a42defcb18726c06483fa5`.
Independent receiving 41211 passed all five unchanged, including the fresh next-frame
gesture and actual saved-file consumers. Before this fix, the original four remained
GREEN (61931), and signature-fixed clippy (90219) and formatting (47676) passed; those
intermediate checks did not claim the fifth regression was green.

Root then authorized a sixth additive case for the same shared registry command
published between App frames. It visibly warms the SDF surface, calls actual
`Engine::dispatch("document.new", …)` under the shared mutex without a helper or frame
warm afterward, then sends the old-surface drag into the first Drawing frame. The
28,497-byte source, SHA256
`fd423e4632d6719db5fb6e1b012a493e47990ddb5b64b56047bb3eb7762c66a4`,
retains the complete 26,124-byte five-case prefix exactly.

Runtime 71635 exited 101: direct New succeeded at revision 2, but the first Drawing
frame again contained an unintended C–C bond from the old SDF surface. The empty-frame
assertion fails before fresh-frame authoring. Log: 1,789 bytes, SHA256
`8b07981dd0d7de43acb375fc1fa781a85ab53b4e9dff7719233a12f5d8cefb5c`.
This confirms a between-frame presentation gap that comparison only around `poll`
cannot catch. It is separate from the three setup corrections, the first transition
regression, and the passing five-case checkpoint. At that poll-only checkpoint, a full
six-case GREEN remained pending the real producer fix and independent receiving checks.

## Final six-case receiving result

Root supplied the actual Rust-only `Engine::session_generation()` getter in
`commands/src/new_document.rs`, 2,246 bytes, SHA256
`efd5bf9406a15779ea64bced6f4b0b1366a2335ce92739f6a5a847bf2f546ada`.
The final UI uses the last presented namespace in the existing `DrawingState`, replacing
comparison only around `poll`. It is scoped to the same App/Engine's input presentation;
it adds no wire field, schema, command, mutation authority, App job owner or SDK-event
rewrite. Source replacement invalidates old canvas input; ordinary edits and fresh
next-frame gestures retain their behavior.

The exact final UI lib is 95,024 bytes, SHA256
`b7797a875ff8bdfb0d3d1c662c298979c38033dc8b33bb5bd17838419409728e`.
The intermediate poll-only lib and both real regression REDs remain archived. Root's
five-entry backend and the producer's unchanged file-workflow/editor inputs are recorded
separately from the 170 untouched protected baseline inputs; all 176 original protected
files were archived before any receiving port.

Independent 84175 compiled the unchanged full 28,497-byte six-case source and passed
**6/0/0**. Its complete 26,124-byte five-case prefix and the earlier 22,237-byte corrected
four-case prefix remain byte-exact. No further setup, action, assertion or expected-byte
change was made for either transition defect. Both now pass their first-empty-frame,
history/revision, original-file and fresh-next-frame authoring/actual-file consumer
checks. Final all-target UI clippy and workspace formatting are separate scoped checks
in the final receipt: warnings-denied clippy 36705 and the final formatting command both
exit 0. No redundant full-package or new native OS execution is claimed. All worker jobs
are terminal and the final source/evidence set is frozen for Root integration.
