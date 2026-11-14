# Neutral-carbon graph scene and hit testing

2026-11-14, worktree bondline-rxn, branch feat/carbon-graph-scene from0830a23.
Implemented with independent automated and bounded actual macOS native evidence. This
integrated source still needs its own hosted run. Full M2-02, general atom display, connected editing
and full M1 are not complete. No new command, dependency, shader or asset was added.

## Delivered boundary

One GraphScene producer serves native painting and registered capture, with private
bounded hit regions and typed fragment/atom/bond IDs. carbon_page_scene delegates to
the same producer; it has no fixture-specific renderer. Arbitrary supported graph
topology includes chains, branches, cycles and disconnected components. Positioned
neutral carbon and Single bonds are supported; heteroatoms, charge, isotope, radical,
other15 bond orders and missing coordinates refuse explicitly. Isolated unknown-H
carbon refuses. No H count is inferred or repaired; Some(0) draws bare C, one draws CH,
larger counts have literal decimal subscripts. Unknown H on connected carbon can be
skeletal. These are bounded original display policies, not a complete chemistry engine.

The existing Roboto/12-em/2/3-subscript/+em/4 baseline and CH3 left/right arithmetic
remain shared. All strokes precede masks and glyphs. Labels are emitted once per atom;
collisions and masks covering unrelated bonds refuse. Limits include4096 primitives
and4096 atom+bond hit regions, glyph512x512/8..256-em, validated projections and finite
butt geometry. Cancellation is checked through construction, glyphs/spans and existing
CPU/GPU/readback. Captures remain bounded2048 per axis; native viewports have their
separate preexisting bound. These are logical limits, not an RSS/performance promise.

Hit tests clip to visible stroke polygons, exclude zero-area tangencies, use atom/label
priority then geometric distance/stable IDs, and validate query/tolerance values. Native
input uses five logical canvas units; no one-sided device-DPI multiplication. That
priority/tie/DPI policy is original, qualified by guide25's five-pixel documentation.
Hover outlines live only in the UI painter, clear on revision/viewport invalidation,
pointer/focus loss and unavailable previews, and do not change authoring data.

StoredDocument completeness/export guards are unchanged. A wedge-display source that
projects to the same basic Single bond still refuses default capture and exports exact
source. This feature does not make unknown source styles, text or CDX annotations
faithfully rendered, and does not add application CDX commands.

## Test-first evidence

- target/carbon-graph-red.log: actual existing carbon_page_scene three-carbon success
  assertion fails UnsupportedChemicalContent before production edits. The first GREEN
  matches a separately specified41-pixel L on CPU and actual GPU.
- target/carbon-hit-clip-red.log: an entirely outside stroke tangent to the viewport
  returned a Bond hit. The regression now returns None after zero-area clipped ink is
  excluded. Local-origin polygon area also avoids large absolute-coordinate products.
- target/carbon-hover-red.log: the real workspace renderer lacks the desired transient
  atom outline; the behavioral assertion fails before UI implementation. Its first
  GREEN and later3-test hover suite cover atoms/bonds, revision/resize/focus/pointer loss
  and native_pixels_per_point=2 synthetic input. The first DPI setup incorrectly used
  Context.set_pixels_per_point, which is deferred UI zoom in actual egui0.29.1 source;
  it was corrected to ViewportInfo.native_pixels_per_point without weakening the target
  assertion. This is still synthetic input evidence, not native Retina acceptance.

Final scoped command uses this checkout's absolute CARGO_TARGET_DIR, incremental0,
DEV/TEST_DEBUG=0 and root's existing absolute BONDLINE_PYTHON:

```sh
cargo test -p bondline-render -p bondline-commands -p bondline-ui --locked
cargo clippy -p bondline-render -p bondline-commands -p bondline-ui --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

All166 scoped tests, warnings-denied clippy and rustfmt pass. Logs are
 target/carbon-graph-scoped-tests.log, carbon-graph-clippy.log and carbon-graph-fmt.log.
This is not a full-workspace/hosted gate. All existing command history/file/close and
native gesture tests in those packages pass; no registry/schema/count changes occurred.

Ten focused graph tests use literal coordinates/pixels, independent finite geometry,
storage/endpoint reversals, cycles/branches/disconnected components, supported/rejected
metadata, all15 other orders, label occlusion, hit/glyph budgets and cancellation. The
80x80/60pt L has points(10.5,20.5),(30.5,20.5),(30.5,40.5), mapped to canvas
(20.5,30.5),(40.5,30.5),(40.5,50.5); only row30/x20..40 plus column40/y30..50 is black
chemical ink. The entire independently specified paper+ink RGBA matches CPU and GPU.
Existing independent quarter-pixel oblique tests retain their predeclared1/32-pixel
GPU boundary allowance; no tolerance was relaxed.

External pinned Pillow12.3.0/FreeType checks authored0/1/2/3/12/65535 counts in both
orientations at96em and the existing >=90% ink-IoU rule. Wrong invented H3 from zero
and reversed12/21 must fail that oracle. Original C/H/3 metric vectors remain intact.
The old H2-negative assertion was replaced with a positive literal-H2 case because
that is explicitly delivered; unsupported atom metadata/order negatives remain strong.

The unchanged approved1024x1024 single-bond image, SHA
1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc,
still passes the external Pillow every-RGBA-byte CPU/GPU comparison through this shared
producer. Its corruption test, JSON, image and approval scope are unchanged. No new
image was promoted to an approved golden.

## Registered imported-file workflow

commands/tests/graph_capture.rs authors a literal original V3000 propane: three
carbons, two Single bonds, sparse source IDs and VAL4 yielding authored H3/H2/H3.
Its actual bytes are independently read by RDKit2026.09.2 as CCC/C3H8 with the exact
connectivity and source positions(0,0),(2,0),(4,0). The existing open_mol command maps
these to(72,72),(120,72),(168,72)pt. CPU/GPU captures agree, leave all state/history/source
unchanged, cancellation is truthful, save keeps exact headers/source and a fresh Engine
reopens the same model and bytes. A separate unmapped WedgeBegin source refuses capture
while retaining exact CDXML bytes. These are actual registry/filesystem tests, not
own-writer/own-reader-only interoperability proof.

The prepared native input is ignored target/carbon-graph-propane.mol,373 bytes,
SHA47ae7c8e8575d77738b5b7afbab0c360e6ad71b3eec048e29eaf7ac4d852f50d.
It is original MIT OR Apache-2.0 test data, not a reference application export.
A new specific three-carbon CLI/stdio/socket process workflow is not claimed; those
adapters share the unchanged registered command and await receiving integration checks.

## Pending actual window acceptance

Root must build the exact frozen source, record source/binary/platform/DPI, and assign
one GUI owner. No GUI was launched by this worker. Acceptance: actual chooser Open of
that original MOL, visible three-carbon graph/literal H labels, atom/label/bond/empty
hover at known positions, resize and actual high-DPI observation, no hover revision/
dirty/history changes, Save As/reopen exact bytes, and existing independent C-C drag/
Undo/Redo/error/dirty-close behavior. Hover remains a prerequisite; no attachment,
selection, transforms, inference or full Professional26.1 workflow is delivered.
Only after those observations may the native part be marked verified. New graphs
remain unapproved as image references. Root owns aggregate/hosted checks, provenance
reconciliation, commits and publication. No worker Git operation or live job remains.

## Native baseline and hover freshness correction

The later baseline native pass used macOS27.2 arm64 binary SHA
4bfc733f83a1ab60d286dd8850275082ea6a060d37b7e80f3ea848627615aa87,
built with UI source SHA
43fce9a3dbb2850a5ccc5056f6829db13ce7bbd57dde636e15ab87ee59d0a2c5.
Its ignored evidence remains in target/native-carbon-graph-ui/, including
build-identity.json and verification.json. That pass found a real hover freshness
failure: after a remote reopen advanced revision3 to4, settled OS-only captures
still showed the prior atom outline; Escape down/up also failed to keep it clear.
The before/after receipts and pixels are29-before-remote-revision,
30-after-remote-revision,31-remote-revision-OS-only-after and32-after-native-Escape.
The1x-to2x public AX window move also retained an old outline while the OS pointer
was outside the moved window. No delivered PointerGone/focus-loss event was proved
for that move, and the reverse2x-to1x move cleared: this is qualified window-event
evidence, not a claim that every move or focus transition fails. Baseline processes
and CUA were closed before this correction; their evidence was not overwritten.

Original Bondline policy now keeps hover invalid after revision, Escape, pointer/
focus loss, blocked input or viewport context changes until a new positional event.
Key release, focus gain and cached egui hover_pos alone do not rearm it. PointerMoved
and PointerButton events are processed in order; an active gesture, queued edit,
unfocused input or held Escape still suppresses hover. The private context includes
page/revision, logical viewport size, available.min, effective pixels_per_point and
the supplied public ViewportInfo.inner_rect origin, falling back to outer_rect.
These are actual egui0.29.1 APIs; no OS cursor polling or dependency was added.
Unavailable/error previews and committed gestures clear the same freshness state.
This policy is independently authored, not asserted as a documented reference flow.

Test-first log target/hover-freshness-red.log records the unchanged baseline's
3 passing tests and7 intended runtime failures. Revision recreated Atom1 on the
second idle frame; Escape release/focus gain and the previously untracked canvas/
native-origin/DPI changes restored or retained it immediately. The new regressions
require three idle frames to remain clear and then require a new position to restore
the correct atom. They include same-geometry revision changes via edit/Undo, Escape
press/release both across frames and within one frame, focus return, native inner/
outer origins, native pixels-per-point and fractional canvas-origin changes. Engine
state remains byte-identical across the hover-only work. An additional ordered-event
case checks PointerGone/PointerMoved ordering and a jitter click that rearms without
an authoring edit. No previous assertion or approved golden was weakened.

With this checkout's own absolute target, incremental0, DEV/TEST_DEBUG=0 and the same
root external interpreter, cargo test -p bondline-ui --locked passes61 tests:
43 library,1 binary,6 drawing and11 hover. All-target warnings-denied clippy and
cargo fmt --all -- --check pass. Exact logs are target/hover-freshness-first-green.log,
hover-freshness-ui-tests.log, hover-freshness-clippy.log and hover-freshness-fmt.log.
The earlier166-test render/commands/UI result remains the baseline result; those
other packages were not rerun for this UI-only correction. Rendering/capture source,
approved golden, source backing, file/close/history logic and dependencies are unchanged.

New native acceptance is pending for the corrected exact source. Root must rebuild,
record its binary/source identity, then verify remote reopen and Escape remain clear
through settled idle captures until delivered positional input, actual origin/DPI
changes where observable, and the existing graph/drawing/save/close checks. Synthetic
input GREEN is not native verification. target/hover-freshness-handoff.json extends
the original14-file source manifest without replacing baseline evidence. No worker
GUI, Git operation or live job remains.

## Receiving-root integration

Root validates the14-file handoff and every exact byte/hash before copying13 unchanged
files and appending the source-register addition without replacing newer root records.
The six-production-file concatenation remains
c079c7363a836729567e16e35203a3ce80f738eb83335d94ec3838071c780a6a.
Root source additionally retains the independent CDX source/snapshot/filesystem features
and the test-only Windows socket correction; no renderer dependency/schema changes follow.

The receiving-root full gate passes467 workspace plus3 macOS bridge tests, formatting,
warnings-denied all-target clippy, layers,154/165/220 declared native closures/two assets
and the unchanged five-file/four-bundle corpus. Log target/carbon-graph-integrated-ci.log,
session6066 exit0, SHA-256
0976db69119bac158187dfac005dbb13301f628c3980319e3250aa51b9637e06.
The approved single-bond image remains byte-identical. This is local integration evidence;
its hosted run and corrected native acceptance are recorded separately when completed.

## Receiving-root ordered input correction

Independent source review target/carbon-graph-integration-review.md, SHA-256
c23fbec2c376157a77ad3651bff6a2baa6be49b968fdbcc120bc2ff86489e5c0,
found a combined-event gap after the preceding integration gate. egui0.29.1's
InputState.key_down/focused describe final state, not eligibility at each event.
EscapeDown→Move→EscapeUp and FocusFalse→Move→FocusTrue could rearm hover. The adjacent
gesture loop could also author a bond when its starting press occurred before release/
focus gain. These control-flow findings were subsequently reproduced as runtime REDs;
the preceding467+3 gate remains evidence for its earlier exact source, not this fix.

Actual ROOT RED logs: target/ordered-canvas-input-red.log records6 old drawing tests
passing and2 new blocked-start traces incorrectly changing StoredDocument; Cargo stops
there before the hover target. The separate target/ordered-canvas-hover-red.log records
11 old hover tests passing and3 new cases incorrectly returning Atom1. Those cover
EscapeDown→Move→EscapeUp, FocusFalse→Move→FocusTrue and PointerButton events delivered
while either condition blocks input. Both commands exit101 before production edits.

Hover and gesture input now share one forward ordered eligibility scan. A preceding
reverse pass over the same vector recovers initial Escape/focus from the final InputState and its
transition events, including egui's computed Escape-repeat flag. Each eligible position
can rearm hover; each gesture press/move/release uses the same current eligibility.
Escape press, focus loss and PointerGone cancel pending gestures and freshness. Final
held/unfocused/blocked state still suppresses transient feedback. Valid positional input
and a complete gesture after release/focus gain within the same frame still work; the
fix does not discard every frame containing a cancellation event. No persistent authoring
or OS state is added, and all existing context/source/error guards remain intact.

The new drawing tests compare the complete stored snapshot and serialized Engine state
after refusal, then require a valid same-frame post-unblock drag to create exactly one
history step. Additional cases exercise a position/press before the release of Escape
held from a prior frame, including a computed repeat, and a position before focus gain.
Hover remains clear through three idle frames, then responds to a qualifying new position.
The previous multi-idle-frame/context tests and valid batched gestures remain unchanged.

ROOT uses its own absolute CARGO_TARGET_DIR with CARGO_INCREMENTAL=0 and its already
compiled default dev/test debug settings; the sibling's debug0 override was not applied.
The external interpreter remains the existing absolute root BONDLINE_PYTHON. Final
cargo test -p bondline-ui --locked passes68 tests:43 library,1 binary,9 drawing and
15 hover. All-target warnings-denied UI clippy and cargo fmt --all -- --check pass.
Logs are target/ordered-canvas-input-first-green.log, ordered-canvas-input-ui-tests.log,
ordered-canvas-input-clippy.log and ordered-canvas-input-fmt.log. No broader gate was
rerun by this worker. Render/capture/model/command/filesystem/dependency/lock/golden
bytes are unchanged; this correction changes only UI input handling and its tests.

Exact receiving-root source hashes and all logs are in ignored
target/ordered-canvas-input-handoff.json. The sibling native acceptance source/evidence
was not changed. A new exact native binary/regression pass and receiving aggregate are
pending root ownership; synthetic tests do not verify those workflows. This worker
launched no GUI, performed no Git operation and has no live job.

## Event-position canvas eligibility

The ordered correction then passed root's474 workspace plus3 bridge full gate:
target/carbon-graph-ordered-integrated-ci.log, session71669 exit0, SHA-256
a9461a3ba05ffa2d517f7fb4b7f5c4db6d57cac05e8480e3879b5f8fa4387eb5.
That exact log remains unchanged; the following new regression means it is still
pre-correction evidence for the next source, not a current aggregate result.

Actual runtime RED target/canvas-press-position-red.log, session58291 exit101:
9 existing drawing tests pass, but a completed Move(a)/Down(a)/Move(b)/Up(b) followed
by a move into the top bar loses the command (revision0 instead of1). A foreground
area test also finds the inverse: a covered initial press can author a bond when
release leaves that overlay. The same test preserves the previously rejected case
whose final pointer stays covered. Both failures arise from using final-state
Response.contains_pointer to decide eligibility of an earlier press.

Eligible PrimaryDown now checks its own position against Response.interact_rect,
the existing paper viewport and Context.layer_id_at(position)==canvas layer. These
public egui0.29.1 APIs were checked in the installed pinned context.rs, response.rs,
ui.rs and memory/mod.rs: interact_rect includes parent clipping, and layer_id_at
queries the topmost interactable area at the supplied position. The ordered event
vector is copied out of Context::input before the shared eligibility scan so the
layer query never nests Context locks. Escape/focus eligibility, blocked input,
source/paper guards and final transient-hover checks retain their separate roles.
No OS polling, dependency or generic input framework is added.

The completed-drag test requires one fragment and one history step, then exact
StoredDocument restoration through Undo/Redo. The foreground test establishes its
area size/order, verifies that the initial position is actually on that layer,
and requires unchanged complete source/model and Engine state for releases both
inside and outside the overlay. The15 hover and earlier ordered/gesture tests
remain unchanged. First GREEN is target/canvas-press-position-first-green.log.

Final ROOT cargo test -p bondline-ui --locked passes70 tests:43 library,1 binary,
11 drawing and15 hover. All-target warnings-denied UI clippy and workspace rustfmt
pass, with own absolute root target, incremental0 and default dev/test debuginfo.
Logs are target/canvas-press-position-ui-tests.log, canvas-press-position-clippy.log
and canvas-press-position-fmt.log. Exact extended identities are in ignored
target/canvas-press-position-handoff.json. Render/capture/model/commands/filesystem,
locks/assets/golden and the sibling native source/evidence were not changed.
Root owns the new exact-native regression pass and current aggregate; neither is
claimed from these synthetic tests. This worker has no live job or GUI/Git action.

## Final integrated source and actual native acceptance

Final six-production-file concatenation is
8c2143f18f47ce48bb1beaae86c0194c6169f0695588bff495a53c107fea832b;
UI lib e80b5faef141a7718a3928e996726a00a150d542e4ec0f86caff1e3f14334f56.
The final15-file handoff is target/canvas-press-position-handoff.json, SHA
d2f3b8096361ee9071a8184c41a19240e1e17e5923f37b068e841251bf535a95.
Runtime/test bytes match that frozen source; later source-register/report updates add
documentation only. All70 UI tests and the final receiving-root full gate pass:
476workspace+3macOSbridge, fmt, warnings-denied all-target clippy, layers, unchanged
154/165/220 declared native closures/two assets and five-file/four-bundle corpus.
Log target/carbon-graph-final-ci.log, session21877 exit0, SHA
804b8d7a260bb8366102d4abec9babe7f1ffcad3ed9bf38a3b8e55fa176b2657.

The actual ROOT native executable is17,473,656B, SHA
2963b4442dd4ad5f08f61e6c5e0df3a17eda0f307b343fc16487338df2b91629.
It was built/accepted2026-11-14 on macOS27.2 arm64/AppleM3Max/Metal, default root dev
debug and its own absolute target. All376 Rust/Cargo/WGSL source identities match
manifest1b64e648d0567138c861d3e9a8263fec7aba5f906af723b22fc65f12c11d4dec.
This is the receiving root, rather than a claim based on an older sibling binary.

From empty, actual native drag paints a continuous dark bond plus black terminal labels
in three settled captures; visible Undo removes them and Redo restores them. Actual
Unicode CDXML Save As/reopen retains graph/positions and492 source bytes, SHA
a30c6d71132b4ef519c77ff395a87bdb6ed5c0417f1f6883f2ba5ecd3dae4159.
External ElementTree checks2n/1b and pinned RDKit reads2C/1single/CC/C2H6. Native C3MOL
Open and atom/label/bond/empty hover pass at actual1x and2x backing scales. Remote
reopen, Escape down/up/held movement, actual held-Escape drag, actual CmdH focus loss/
return and both programmatic display-context moves clear feedback until fresh movement;
held drag changes no graph/revision/history. Foreground chooser drag/Cancel does not
draw, while deterministic tests separately isolate layer authorization and batching.

Ignored exact report target/native-carbon-graph-final-handoff.md SHA
b92dee0cafe8ef9fee61718ae457c8c20e2c4d665fa023c600d3dc7c99c6a6d0
records preconditions/action order, actual pixels, external consumers and exclusions.
Flow ledger52 SHA613abe1a1bb95bb117acd4cba416e40b7eface36f587dc34a479cc1bafc39672;
index53 SHA f8fce4c1c57ae1451eb272d023b275b51479c9272005c8170ce72f65e5fa2538
hashes136 actual evidence files. Initial shadow-ROI, guard-refused input and failed
focus-loss probes are preserved/excluded. No native same-frame event timing, arbitrary
element/stereo/source-style rendering, persistent selection/attachment, full parity,
new golden approval or interactive Windows/Linux evidence is inferred.

Final cleanup49 SHA803ea7c4ab6bcec990ad72dfeb15dc0d2aa87e04b6eb81349e45884f2e708b6b
confirms native32764/controller32742/session81950 exit0, all three held-key helpers
exit0/EscapeUp, no live jobs, exact cursor restoration and sampled original reference
window IDs/frames/layers unchanged. CUA completed exactly once after observed cleanup.
Earlier baseline failures and restoration incident remain separately recorded.
Independent read-only final input review finds the earlier ordering issue resolved,
no confirmed new touched-path defect: target/carbon-graph-final-input-review.md SHA
71fb49f42da98818e676b22611528ac14f5d41c0317467f8a921394cfc2951cc.
The reviewer read source/evidence, not a second execution. This source needs its own
hosted run before main advances; M0/M1/M2 remain incomplete.
