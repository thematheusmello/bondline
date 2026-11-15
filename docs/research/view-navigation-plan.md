# View navigation: bounded next gate

Research only, 2026-11-15, supplied Root `2ac7739`. No implementation, test, native
execution or parity claim follows. Root owns Git and integration. Existing app PID83437
was not inspected, activated or changed.

## Public evidence and its limits

[GUIDE-25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf) is the
already-pinned public vendor-authored guide:17,721,447B, SHA-256
`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`,439 PDF pages.
Printed page N is PDF index N / one-based page N+1. Printed26,74,79 were rendered
with Poppler and visually reviewed; images/text remain ignored under
`target/view-navigation-research/`. Filename25 and2025 metadata/footer qualify this
as25 evidence, not a measured26.1 default.

| Observed guide fact | Location |
|---|---|
| Zoom changes appearance without changing drawing dimensions; percentage entry accepts1-999. |26|
| View roles increase/reduce magnification, restore actual size, show the whole document, and set a percentage in the General toolbar. |26|
| The magnification procedure keeps a selected object in view; without selection it identifies the last drawn object as the magnification focus. |26|
| Actual size F5; fit-to-window F6; magnify F7; reduce F8. |79|
| Spacebar selects Marquee in the generic tool-switching table. |74|

The last-object note belongs to the magnification procedure, whose option list also
mentions Actual/whole-document/percent. It does **not** give separate anchor/reset
rules for those actions. Do not infer that Fit must center a last object or that Actual
must reset pan. Numerical100% screen units, magnify/reduce factors, initial zoom, fit
padding/multipage bounds, wheel/pinch/Hand/Pan modifiers and per-platform Fn behavior
remain unestablished. Full-text search found no usable2D pan contract; this is a review
limit, not proof of feature absence. Do not introduce Space-held Hand as sourced parity.

The [official26.1 release announcement](https://support.revvitysignals.com/hc/en-us/articles/54009539308052-Release-Announcement-ChemDraw-Desktop-26-1),
updated2026-09-28, is publicly readable through the web tool but supplies no zoom/pan
contract. A direct HTTP research fetch returned403; no access control was bypassed.
Bounded official-site searches did not locate a complete public26.1 navigation guide.
Existing inventory MV-015..018 and KM-024..026 stays version-qualified; add F5's role
when wiring controls rather than claiming the modern menu is fully inventoried.

## What PhotoCraft actually contributes

Pinned source `0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e` declares MIT OR Apache-2.0.
Five relevant cached files were compared byte-for-byte with their public pinned raw
versions. Exact hashes are in the ignored source receipt. No upstream app was run;
no code, asset, zoom ladder, shortcut or dependency is transplanted.

- [app/view.rs](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/app/view.rs)
  explicitly separates device pixels per raster-document pixel from egui-point zoom
  by dividing by canvas PPP. Its100% is raster-specific, unsuitable as Bondline's
  chemical-point convention.
- [canvas/view_xform.rs](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/canvas/view_xform.rs)
  shares forward/inverse camera mapping, fit resolution and zoom-about-anchor math.
  Its fit caps at raster100%; that cap is not adopted for physical-point pages.
- [panels/actions.rs::run_view](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/panels/actions.rs)
  defers fit until actual canvas layout and changes transient UI view state. Those
  navigation handlers are UI-side; do not replace Bondline's shared registry with
  that dispatch split.
- [canvas/zoom_levels.rs](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/canvas/zoom_levels.rs)
  centralizes limits and stepping. Reuse that design principle, not its12800% raster
  range/ladder or non-finite fallback. Bondline should reject invalid parameters.

## Agreed original Bondline policies

These are engineering decisions, **not vendor defaults**:

- Keep default Fit and its existing bytes/pixels exact. Do not resize the chemical page
  to make a small scene readable. Fit resets to the page center and remains responsive
  to canvas resize; document-vs-page fitting is partial while only one page is shown.
- At100%, one authored point maps to one logical canvas unit:12pt main label em becomes
 12 logical units and1pt stroke becomes1. DPI only changes physical rasterization.
  This is not physical-monitor calibration or a promise of printed real-world size.
- Explicit percent range1-999; Magnify multiplies by1.25, Reduce divides by1.25,
  clamping at those limits. Equal effective state is a no-op. Fit remains a distinct
  derived scale, preserving legacy output even when its computed percentage lies
  outside the entry range. Pan can preserve that Fit-derived scale independently
  of its absolute center; invoking Fit again resets both scale and center.
- Pan uses a finite absolute center in authored points, bounded by existing Point2
  limits. Scrollbar/wheel deltas convert through the same current inverse transform.
  No Space-Hand binding. No rotation, animation, generic camera framework or new crate
  is needed for this gate.

Recommended anchor policy to confirm while writing tests: Fit centers the page; Actual
selects100% while preserving current center. Magnify/Reduce/percent may use an explicitly
provided valid own-last-drawn focus, preserving that point's canvas position, otherwise
use the current view center. This is a deliberate partial policy; selected-object focus
waits for real persistent selection. The guide does not establish each of these choices.

Own-last-drawn tracking needs no import-order guess. After a successful, changed
BondCreate/Extend commit, its handler already knows the committed segment endpoints;
retain their midpoint plus page/source-generation identity outside authored history.
For shared-interface behavior prefer this small Engine transient fact; the GUI may instead
submit its validated own-commit anchor explicitly. Never choose a source's last array
entry, maximum ID or last parsed fragment after Open. Clear on successful Open/content
replacement/page switch/Undo/Redo or unrelated geometry edits; a failed Open preserves it.
Own set_order may retain it only after revalidating the same IDs/endpoints. Initial/imported
focus is unknown, with the documented original fallback above.

## Small shared contract and integration boundary

Suggested single registry command `view.navigate` with closed typed parameters:
`expected_revision`, `expected_view_epoch`, `page_id`, logical viewport width/height,
and a tagged action: fit, actual, percent(value), magnify, reduce, or pan(absolute center).
Only zoom actions accept an optional authored-point anchor. Reject unused/unknown fields,
non-finite values, invalid ranges/IDs and stale revision/epoch atomically. Viewport dimensions
use existing limits; they are explicit headless context, not guessed remembered window size.
Resolve a step from Fit only after current layout is known. The GUI attaches current context
and dispatches the same command; it does not implement another zoom factor or clamp.

Engine transient view state/checked epoch and a separate operation result follow the existing
`ui.bond_tool` pattern: authored `changed:false`, view-operation `changed:true` when appropriate,
no authored revision/history/redo/dirty/source/baseline/identity-floor/close-epoch change.
A->B->A advances the view epoch. Open resets to Fit; save, tool changes and authoring Undo/Redo
do not restore a historical camera. Current close/save registrations and committed-close
refusal stay intact. Loopback repaint must observe the view operation, not only authored
`changed`. Expose actual current view state through shared discovery/results.

Keep source identity and view identity distinct: existing CaptureStamp stays source/revision/
generation semantics. Add view epoch/resolved transform to positional contexts and preview
keys, alongside current size/origin/DPI/close epoch. Reject stale capture publication and
cancel/clear gestures, stationary conversion hotspots and hover until fresh ordered input.
Process mixed view/menu/key/pointer events in their real order, reacquiring transform and
hits before later actions. Do not add another early shortcut/menu prepass.

| Existing location | Required narrow change |
|---|---|
| `render/src/page.rs::PageViewport` | One shared forward/inverse mapping; retain existing Fit constructor exactly; support signed unclipped camera extent and a separate visible clip. |
| `commands/src/{lib,capture,controls}.rs` | Transient view/registry metadata, immutable capture view snapshot, shared scene request and repaint marker; preserve full source authority. |
| `ui/src/lib.rs` | Both initial and post-command viewport/scene acquisition sites use the same current transform, including pointer conversion and hit/hover geometry. |
| `ui/src/preview.rs` | Carry view identity into the actual registered PNG path before enabling SDF preview navigation; do not stretch a previous fitted PNG. |

Important renderer dependency: current fitted unsigned paper Rectangle conflates physical
paper and camera visibility. Enlarged/panned paper may begin at negative canvas coordinates.
Generate at requested scale and then clip; never upscale old pixels or allocate a full
999%-page image. Chemical/source validity remains whole-page validation. Off-camera ink is
not chemically hidden ink; per-component label masking, low-scale distinguishability and
physical-paper guards must stay honest without rejecting ordinary viewport cropping. Retain
bounded primitive/hit/font/readback work and cancellation. Unsupported projection remains
unsupported even when panned offscreen. SDF/SMILES wrong-kind/fallback behavior must remain
explicit; a drawing-only first integration does not establish collection navigation.

## First decisive test and completion gate

Before production, add an analytic RED for an explicit100% view: canvas800x600,
center(100,100)pt maps document(120,140)pt to(420,340) logical units and back. At125%
with that point anchored, new center(104,108)pt leaves it at(420,340). A24pt axis becomes
30 logical units; DPI2 makes60 physical pixels without changing document coordinates.
These are original numeric expectations, independent of implementation helpers.

Then require legacy Fit approved-golden equality; percent bounds/non-finite/no-op/epoch/
clamp cases; anchored zoom and absolute pan; resize/DPI; wholly/partly off-camera labels
and multiple components; paper-edge/hidden-ink/tiny-scale refusals; exact hit and inverse
mapping; source/history/file/close invariance; stale registered image rejection; and mixed
view+history+pointer event order in real app frames. Compare actual CPU/GPU images and
independent pixel/metric expectations, not only nonblank scenes.

The native gate must use unchanged A4, navigate to readable scale, draw Single/Double/
Triple, hit/convert/Undo/Redo at zoom, pan and reset Fit, then Save/reopen with exact authored
geometry and independent consumers. Native work requires a separate ownership/availability
handoff; PID83437 remains untouched here. No UI/renderer/command implementation, build,
Git, Actions, native input or Windows/Linux execution occurred in this research task.


## Root adoption decisions before production

The preceding proposed contract remains research, with its original11660B/SHA
`3ff3b2efb7c1704d37b49ea47424dd0bdc0660440b2afe3ccf42d7ef503f3cbf`.
The implemented gate uses one `ui.view` tagged closed object, following existing
`ui.bond_tool` current-window semantics. Actions: inspect, fit, actual, set_zoom(percent),
magnify/reduce(width_px,height_px), pan(center_x_pt,center_y_pt). Relative dimensions
are explicit integer logical canvas units, independent of device DPI. It has no source
revision/view-epoch parameters: view intents mutate neither authored content nor saved
baselines, and are applied to the current window. Positional UI contexts still carry a
checked view epoch and current source/size/origin/DPI to cancel stale gestures/hotspots.

Fit clears explicit zoom/center; Actual100, percentage and Reduce preserve center.
Magnify alone may center the known successful own Create/Extend segment midpoint under
a current source generation/page, matching the guide's qualified last-drawn note. No
source ordering is used. Unrelated geometry/history clears that focus cache, a failed
operation preserves it and own set_order can retain it because that command guarantees
unchanged endpoints. Selection-based zoom awaits real selection. The camera itself
persists across Open/Undo/Redo as an original window policy; after a different page opens,
only projection clamps a retained center. This supersedes the proposed Open-to-Fit reset.

Pan requires explicit Zoom; its finite absolute center is inclusive within the current
first page. Relative bounds clamp1..999 with no-op at limits. A fitted legacy scale may
lie outside the numeric entry range: Magnify at or above999 does not reduce it. The GUI
records actual canvas metrics only as host infrastructure for discoverable typed F7/F8
bindings; before those metrics exist, those disabled bindings contain valid1×1 context
and report viewport_unavailable. Direct relative commands accept explicit headless context.
Inspect is a pure view query, including wrong-kind/picker/NeedsDecision states; generic
committed-close refusal remains. Other actions keep close/picker/source guards before
mutation and no-op. Whole-page render.capture and its source capture stamp stay unchanged;
current-view export/collection navigation is not silently claimed in this drawing gate.

The render API is PageView::fit/zoom with uniform percent/100 geometric scale, nearest-
integer signed camera translation and the same inverse for picking. It builds new glyphs
at the requested zoom on bounded full-page geometry, then projects/clips; no full-page
raster image is allocated or stretched. Fit is an identity path. The first projection
profile refuses source strokes whose butt corners leave the full source canvas, and
translated strokes that exceed existing coordinate bounds. Arbitrary source-clipped
strokes remain Fit-only until a separately verified clip primitive/contract exists.
This honest new-view restriction does not change or weaken legacy source/chemical guards.
