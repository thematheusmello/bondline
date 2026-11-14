# Terminal-carbon attachment: UI wiring verification

2026-11-14. Isolated `bondline-rxn`, branch `feat/connected-bond-integration`,
base501a315. This follows the engine/headless feature in
[verification-terminal-carbon-extension.md](verification-terminal-carbon-extension.md).
Only the workspace input/presentation code and focused new tests change. No model,
command, renderer, codec, dependency/lock, asset, pinned fixture or approved golden
changes. Native platform acceptance is still required; synthetic egui input is not
native runtime evidence.

## Delivered behavior and boundaries

The visible guidance explains the closed operation before a gesture: dragging empty
paper creates C-C; extending a terminal CH3 authors CH2. That guidance stays present
beside a prior error. The UI sends the required named
`hydrogen_policy=rectify_terminal_carbon` to the existing typed `bond.extend` registry
operation. Its engine authority, literal-H policy and failure guards are unchanged.

Each eligible primary press picks the current bounded GraphScene, rather than a cached
hover. An atom/label hit retains its fragment/atom identity and exact model anchor.
The raw logical press position is retained separately: a click or displacement of at
most three logical units does not edit, even when a label extends far from its atom.
Intentional unsupported atom/bond starts produce a diagnostic without falling back
to independent bond creation. An existing atom/bond at release is refused; this slice
does not implement ring closure, merging, branching from an internal carbon, or other
bond/element/label tools. Unknown-H hover and click behavior remains unchanged.

Press and release independently require visible clipped paper and the canvas's
foreground layer. A new ineligible press cancels an earlier gesture. Revision, page,
canvas dimensions/origin, effective DPI, supplied native window origin, Escape,
focus loss, pointer loss and close intent cancel held gestures. Idle frames, key
release and focus return do not restore a canceled gesture.

Qualified releases execute immediately in event order through Engine. A later press
in the same frame repicks the graph/revision that actually committed, including an
atom just created by the preceding release. A still-held gesture retains its captured
revision; it is never relabeled after a different operation. Paint uses the same fresh
snapshot. Transient preview/hover remains outside the document and history.

## Actual test-first evidence

All logs below are ignored in this checkout's own `target/`.

- `attachment-ui-red.log`: before production edits, all five original behavioral
  regressions ran and failed, exit101. Actual old behavior created an independent
  fragment for atom drags, failed chained attachment, anchored a label drag at raw
  press coordinates, retained a gesture after a new ineligible press, and committed
  a release behind a foreground area. Expectations use literal graph/ID/H values,
  independently located painted paper/label geometry and unchanged full snapshots.
- `attachment-ui-first-green.log`: the minimal input change passed those five tests,
  all11 existing drawing tests and all15 existing hover tests.
- `attachment-ui-expanded-tests.log`: all13 intermediate attachment tests passed.
- `attachment-ui-file-tests.log`: a misplaced test-only one-pixel-jitter block caused
  a compile error because `press` was out of scope. Moving the block into the intended
  label-click test fixed the test code; no production behavior/assertion changed.
- `attachment-ui-file-tests-green.log`: all15 final attachment tests passed.
- `attachment-ui-all-tests.log`: all88 UI tests passed,0 failed/ignored, comprising
  the73 existing tests and15 new attachment tests. Existing assertions are unchanged.
- `attachment-ui-clippy.log`: warnings-denied all-target UI clippy passed.

The new suite covers literal one-fragment C3/H3,2,3 and same-frame C4/H3,2,2,3,
preserved old IDs/coordinates, exact full-snapshot Undo/Redo, a new bond followed by
attachment in the same frame, a held later press completed next frame, fresh button
position overriding cached hover, label-anchor preview and raw click/jitter, visible
guidance with prior errors, unsupported/occupied starts and ends, MOL-source refusal,
foreground-covered press/release and stale/context/cancellation behavior across idle
frames. Effective2x DPI preserves document coordinates without doubling input.

## Independent saved-file consumer

The test creates an original C-C graph and saves through the existing shared command,
then performs the actual workspace attachment gesture, saves again, undoes/redoes and
reopens. The old ethane consumer is unchanged. A separate inline Python consumer uses
Expat/Decimal on the actual saved bytes: one page/fragment, three unique neutral carbon
nodes, literal authored H3,2,3, two Single bond references and original points(100,200)/
(140,200). The added point targets(180,240); a0.0001pt absolute bound is declared before
the check because native egui positions use f32. The exact authored starting atom is
retained, not rounded through the press coordinate. No renderer/golden tolerance changes.

External pinned RDKit2026.09.2 must consume those same bytes as CCC/C3H8 with H3,2,3;
missing consumers fail instead of skipping. Undo restores the full saved ethane snapshot
and dirty state; Redo restores the saved propane snapshot and clean state. Reopen
retains the exact saved bytes and model, then the real workspace paint path draws it.
This is external chemistry/interchange evidence, not a Bondline chemistry engine.

## Reproducible checks and remaining validation

Use the documented isolated interpreter and this checkout's absolute target directory:

```sh
export CARGO_TARGET_DIR=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-rxn/target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export BONDLINE_PYTHON=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline/.cache/oracles/format-python/bin/python
cargo test -p bondline-ui --locked
cargo clippy -p bondline-ui --all-targets --locked -- -D warnings
cargo fmt --all -- --check
cargo xtask ci
```

The first frozen own-checkout `cargo xtask ci` passes535 workspace plus3 native bridge tests,
538 total,0 failures/ignored. It includes rustfmt, warnings-denied workspace all-target
clippy, layers, unchanged154/165/220 declared runtime closures and two licensed assets,
and the existing five pinned files/four source bundles with independent raw/consumer
corpus checks. `target/attachment-ui-ci.log` prints the correct absolute `bondline-rxn`
root and ends successfully, session19771 exit0,65798 bytes, SHA256
`68e06e766ff32d3aeb66927c60e98a0355cbe130108c3588233074df970893ee`.

That first frozen production `crates/ui/src/lib.rs` is39948 bytes, SHA256
`e029047f216f4ea4064106379be953a4de78c21dd46e51f6da26befd596de68e`;
the new27215-byte test is SHA256
`5f4c64bd805825e0077af27789d389ff0c3f406fc6ecb09258732284f981e0ae`.
Existing drawing/hover tests, file workflow, executable source and both locks match
the parent's receiving source. Native-v0 remains408 bytes/SHA256
`ead553df9d8c6bf7a371e7e36909a9c8895f34b9231f20c3c3a796680d64bbc6`;
the approved golden remains22933 bytes/SHA256
`1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc`.

No GUI was launched by this implementation worker. Actual macOS gesture/layout/save/
reopen/error/history, native1x/2x and context-cancellation acceptance, independent source
review and this new source's hosted checks remain separate. The next native acceptance
must visibly create C-C from empty paper, extend an existing terminal label/atom to
C3 and then C4, inspect guidance/preview/H labels, Undo/Redo, save/reopen/edit again and
independently consume the saved output. It must also inspect label-click jitter,
unsupported/occupied targets, guarded imported sources, foreground/cancellation cases
and cleanup without confusing synthetic event batching with observed OS input. This
handoff has no running jobs. Do not refresh the approved golden to accept this feature.

## Independent review correction: invalid viewport cancellation

After that first handoff, independent read-only review found an inherited early-return
path: PageViewport construction failure cleared hover but retained the held gesture.
Restoring the original dimensions could therefore make the old gesture context match
again; Escape down/up in the invalid frame was also skipped by that early return. This
was initially a source-level finding, not an observed native runtime failure.

Two new deterministic tests reproduced the exact failure on the first frozen source:
press an existing terminal atom, present a100001-unit-wide viewport (asserting the
actual viewport diagnostic), restore original geometry, retain three idle frames, then
move/release on free paper. Both the ordinary invalid frame and an invalid frame with
batched EscapeDown/Up failed final full StoredDocument equality, exit101, in
`target/attachment-ui-invalid-viewport-red.log`, SHA256
`fc243a2af3201c11ef3230695ad1f33400ad0cba6b6687905aee45ec5785571b`.
The snapshots/state are checked before and after the invalid frame, including history.

The production correction is one line: clear `drawing.gesture` alongside hover before
returning from the PageViewport error branch. No input, chemistry, source or history
policy is weakened. All73 pre-existing and15 original new tests are unchanged; the two
new regressions bring the UI suite to90 passing/0 failed/0 ignored, session41987 exit0,
`target/attachment-ui-invalid-viewport-all-green.log`, SHA256
`18fdfcfa3361017e307e81fd8de37d2621c2f12c0aa0ca29e5dd089872dcfeb7`.

Corrected frozen production is39992 bytes, SHA256
`23410115d78a0673f2b10a0ee37bbf90d6f802451b11d47bf49e0af849af198e`;
the28730-byte attachment test is SHA256
`d20c147b8c28670f9b2c4d435dd76675ca22ddb88e76713637c2b3bd31599170`.
The first handoff and logs remain historical, unchanged. A separate final handoff records
the corrected full gate. Native acceptance and hosted checks still remain separate.

The corrected own-checkout `cargo xtask ci` passes537 workspace plus3 native bridge
tests,540 total,0 failures/ignored, session4960 exit0. All original aggregate gates
pass, including warnings-denied workspace all-target clippy, rustfmt, layers, the
unchanged runtime/asset license closures and the pinned independent corpus checks.
`target/attachment-ui-final-ci.log` again prints the correct absolute `bondline-rxn`
workspace and ends successfully,65977 bytes, SHA256
`1eabfd7186deee09d9f80b0f08d0b6b8f39c52b6983f7ac3aaf9e9d9de0a4753`.
No job remains. The corrected source/tests are frozen for receiving review/native
acceptance; this full local gate does not substitute for either or for hosted checks.
