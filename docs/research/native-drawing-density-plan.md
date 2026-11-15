# Native Drawing density: source-backed implementation plan

2026-11-15, reported base `6ecc4b1`. **Design only; implementation and acceptance are pending.**
This note changes no product, test, dependency, asset, threshold or camera default.
M0–M5 remain incomplete; hosted Actions remains disabled. Root owns implementation and native work.

## Observed limit and its scope

The bounded native component workflow saved/reopened the complete C8H18O Drawing after
changing its first C–C bond to Double. Fit displayed78.79% and refused with `BondScale`;
100% rendered again. Exact native canvas dimensions/effective DPI were **not measured**.
The [receiving record](../verification-smiles-components-integration.md) retains that limit.

An **unexecuted analytic candidate**, not a native reproduction, is A4 Fit848×728:
rounded paper469×664, stroke width0.78787037 and axis y88.78650. The Double centers
are approximately87.60469/89.96830, half-width0.393935. The second contains neither
pixel-center89.5 nor90.5. The current logical-raster refusal is therefore plausible
and consistent with its contract, not evidence of a faulty visibility search.
[page.rs](../../crates/render/src/page.rs) requires a pixel in each component and gap;
[multiple_bonds.rs](../../crates/render/tests/multiple_bonds.rs) preserves both rejecting
and accepting subpixel phases. Labels masking otherwise visible ink use `LabelOverlap` instead.

The separate policy gap is that native Drawing checks a logical lattice before painting
egui vector shapes. At2× density the same logical geometry can have distinct physical
component/gap pixels. This does not justify weakening the1× guard or moving atoms/lines.

## Primary sources actually inspected

- [PhotoCraft canvas](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/canvas.rs), especially218–240/355–410/2330–2440, keeps render density and the input/overlay transform related. [ViewXform](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/canvas/view_xform.rs),20–130, shares forward/inverse mapping. Its raster100%, optional alignment, rotation and zoom policies are **not adopted**; Bondline100% remains one logical unit/authored point.
- [egui0.29.1 Context](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/egui/src/context.rs),1885–1920/2453–2470, defines effective physical pixels per logical point and uses that value for tessellation feathering. [RawInput](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/egui/src/data/input.rs),17–20, specifies logical event coordinates. Use current `Context::pixels_per_point()`, including UI zoom, not monitor scale alone.
- [eframe0.29.1 WGPU integration](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/eframe/src/native/wgpu_integration.rs),615–654, passes the frame's effective density through tessellation and painting. [ScreenDescriptor](https://github.com/emilk/egui/blob/fe368bacc4a15697e347607e73a56c0299b3d42d/crates/egui-wgpu/src/renderer.rs),124–139, separates physical screen size from logical size.

Cached PhotoCraft HEAD names the exact0fb40c5 pin above. SDK VCS is
`fe368bacc4a15697e347607e73a56c0299b3d42d`; the installed egui-wgpu archive metadata agrees.
PhotoCraft/egui/eframe are reviewed permissive source, not code/assets to transplant.
Pinned public ViewXform/Context/eframe URLs were accessible in this review; other sections
were read from existing local caches/vendor sources. No reference application was run.

| Reviewed file | Bytes | SHA-256 |
| --- | ---: | --- |
| PhotoCraft `crates/ui-egui/src/canvas.rs` |182271|`fe8933427deecfeddab8a4632a76cf2c2fce29b94cec3362c6f34b00a6c87e13`|
| PhotoCraft `canvas/view_xform.rs` |12022|`e6fbb4e2f1a15a92fbde3df852a545a89fd745477e8faffc922b8e7101ea460d`|
| Bondline vendor egui `context.rs` |146953|`daea6760cddd4001480bcdfe8457968c491141ff56607215463832c1c3888ae8`|
| Bondline vendor egui `data/input.rs` |41217|`993cfd72386989f5155aac10f8cbd48091e9564457a7b37c3b03402a373bd3c5`|
| Bondline vendor eframe `native/wgpu_integration.rs` |38756|`7e7d12b34828607f456c96003d4bc6681d83877323e37d0a1562ec7dffc64e8b`|
| Published egui-wgpu `renderer.rs` |43585|`48ca52f12b8f8921bdf31574f3d2c28f17849282bbfeef7c1e0d32baf67a3f3c`|

## Locked original Bondline policy

Add `PageView::at_display_scale(density: f64, origin_phase: [f64; 2]) -> Result<PageView, Error>`.
It derives geometry only, reusing the existing physical transform; no frame allocation.
The input view is logical. Reject a second density conversion and nonfinite/nonpositive
density or phase outside `[0,1)`. Logical percentage, authored point center and rounded
Fit axes remain unchanged. Explicit Zoom stays uniform; no physical-inch calibration claim.

For floored logical canvas size `L`, current effective density `d`, and actual egui
canvas minimum `c`, use the following **one shared affine mapping**:

```text
a = c*d; i = floor(a); phase = a-i
physical_destination = ceil(L*d + phase)
physical_local = logical_local*d + phase
paint_origin_in_egui_points = i/d
paint_position = paint_origin_in_egui_points + physical_local/d
hit_position = (event_position - paint_origin_in_egui_points)*d
```

Thus the fractional origin reaches actual device-pixel coverage without snapping the
canvas/camera. Use `a-floor(a)`, including signed origins, rather than negative Rust
`fract()`. Preserve the actual canvas clip/foreground guards, including fractional edges.
Fold the old logical camera offset times density plus phase into the existing integer
translation/fractional viewport split. Generate fresh glyph pixels at that physical phase.
Do not infer density from rounded width/height ratios or stretch axes independently.

Extract the smallest private transform body shared with existing `at_pixel_size`.
That capture API retains its exact validation/error order,2048 bound, `min(P/L)` density,
centered padding, identity branch and rejection of repeated conversion. Its default
whole-page captures and every existing fixture/oracle/golden remain unchanged.
The new native geometry entry uses existing1..100000 source/destination/coordinate bounds;
`at_display_scale(1,[0,0])` retains the exact old viewport/projection clone path.
The UI keeps its legacy1× path, including old outputs on large vector surfaces.
Native fractional-density origin handling is separate from any future1× policy change.

## Minimal complete integration and call sites

- Add thin `Engine::drawing_scene_with_view(&PageView, page, cancel)` delegating to existing source-qualified `io::display::drawing_page_scene_with_view`. Existing `drawing_view_scene` delegates after its logical `page_view` construction. No new command/schema/worker/IO dependency in UI.
- In `ui/src/lib.rs`, construct the same current density view for initial graph acquisition and both post-history/post-command rebuild sites. Retain model/page/camera reacquisition in raw-event order; a later same-packet press must pick what committed. No logical graph preflight may reject the physical graph first.
- Use that physical graph/view for all **four** hit-test sites: key-hotspot fallback, press, release-free guard and hover. Scale the five-logical-unit tolerance by `d`; preserve label/atom priority, stable IDs and one logical bond hit. Keep response/paper/layer/modifier/text-focus/revision guards.
- Convert event points with the shared mapping before physical-view inverse/containment. Atom attachment still snaps to the exact authored model anchor. Preserve raw press versus anchor threshold and three-logical-unit drag/jitter behavior; DPI must not change chemical coordinates or gesture intent.
- Paint physical rectangles/stroke corners through the exact inverse map into egui floats; do **not** round them into an integer logical Scene. Fresh glyph masks remain physical pixels. Atom halos stay6logical units; bond outline expansion stays4logical units and outline pen1logical unit. Transform those sizes once. `paint_stroke` also serves the blue gesture preview: avoid double scaling or mixing a logical preview with physical scene coordinates.
- The current `HoverContext` already includes canvas dimensions/origin, effective DPI, host origin, revision/tool/page/view epoch. Clear gesture/hover/hotspot/navigation on changed context and every renderer/viewport failure; do not revive from cached hover. Tag the presented namespace only after successful current paint. Native Drawing remains synchronous/no new cache, worker, texture or job owner.

The existing registered capture uses `capture_scene`→source-qualified IO and optional
`PageView::at_pixel_size`; retain its snapshots, cancellation, worker/reply/close barriers.
SDF's `PreviewWorkflow` already keys full source/close/view/rect/DPI/physical dimensions,
uses capped registered PNGs and checks logical em after decoding. It remains read-only,
with its exact legacy Fit versus opted-in logical-view behavior; do not copy its texture
cap into native Drawing or grant SDF hits/authoring. SMILES-file content stays non-Drawing.

## Bounds, readability and remaining risks

The existing graph producer/projector does not allocate a full frame or call `byte_count`.
Geometry-only physical destinations above2048 can therefore use existing bounded vector
scenes; CPU/GPU/PNG readback still refuses above2048. No silent DPI cap, upscale, resize,
legacy-density fallback or enlarged resource threshold is allowed. Ordinary4k surfaces
must not acquire the capture limit. Preserve checked ceil/casts,100000 virtual/source/
destination and stroke limits,4096 primitives/hits, physical8..256 em and cancellation.

When an actual main-label metric is `Some(em)`, also require `em/d` within8..256 logical
units; higher DPI must not certify tiny logical text. `None` means no labels, not failure.
New physical glyphs can consume more existing primitive budget; large zoom/DPI may
honestly refuse unchanged physical bounds. No hard RSS/latency guarantee follows: major-
axis visibility scans and scene generation remain cancellable bounded synchronous work.

Keep existing f32 geometry/anchor quantization; prior physical tests use1/256pixel on
their declared vectors, not a universal bound. Scaled hit tolerance must satisfy existing
finite bounds rather than clamp silently. Egui tessellation feathers vector edges, unlike
the frozen opaque CPU/GPU pixel-center reference; no OS/compositor pixel-identity claim.
No valence, source/baseline/IDs/dirty/history/path/Save/close-ACK authority changes.

## Next decisive independent acceptance — unexecuted by this author

1. Freeze the fixed A4 two-carbon Double with unknown authored H, separately from the whole9-atom candidate: logical1× must retain the existing phase refusal; independently declared2× strips/gap must have actual CPU/GPU coverage. Preserve every old subpixel negative/positive and approved Single image.
2. Before implementation, actual-App DPI2 routing must fail on the old logical path. Inspect actual painted primitives/mesh against **global** physical pixel centers, with nonzero/fractional canvas origin as well as phase0; fresh physical glyphs must differ from enlarged logical glyph pixels.
3. Add a geometry-only physical destination above2048: scene construction/picking succeeds under existing vector bounds while actual readback still refuses.1× old Fit/Zoom scenes, pixels, errors and large vector destinations remain exact.
4. Preserve logical100%/pan/fit/readout and source/Arc/history. Cover actual press/release/attachment/order edit, stationary hotspot, Undo/Redo and later same-frame press; DPI/origin/resize/ABA/focus/Escape/modal/invalid-viewport cancellation must not reuse old hits. Include source-qualified MOL H display and guarded wrong-kind/incomplete source.
5. Root's later native run must record actual logical canvas, effective DPI/origin phase, view/camera and build, then inspect Fit and explicit zoom/edit/save/reopen without source change. Headless App/mesh or CPU/GPU evidence alone is not native verification. Other platforms and full desktop parity remain open.

Read baseline pins: `render/view.rs`13125B/`e68580393a94c621aa412a83271b8e3a00eb9d2430f3f8775090944331aec27e`;
`render/page.rs`33875B/`c594d2e1b5e5a2876cca356cf0179ab3da4c3be0adc5ac50032bd519d67b050e`;
`render/stroke.rs`22943B/`5a68bbe6bfacc71002ecfc38ff1dc2968ff72ee363e6ae69d4d5696c3da4a464`;
`commands/view.rs`13583B/`a4d341526582e36ff20e37974799ee4e02d88ccfc9a4b91f745480e1baab5c4a`;
`commands/capture.rs`19021B/`7342240185015410735a0435c42f47e123396410081af71f3cb1e275cf28170a`;
`io/display.rs`10285B/`d572a1f2cd6525cc628e1092581ab2dd0451264840149afb7b6c640f97d0700a`;
`ui/lib.rs`95602B/`4138d9212fc603378c851c1e55e4b739822a4d722094ffc59deb1bdc6945fcd0`;
`ui/preview.rs`13353B/`fe79432cc37b723e28bf3fd7ed3d24d0abc63a3be43ed8726dc88a0106b7d6aa`.
These identify reviewed pre-implementation source, not future implementation or test results.
