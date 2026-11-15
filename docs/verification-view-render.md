# Logical page view rendering

2026-11-15. Renderer-only foundation from `2ac7739`, implemented in the isolated
`bondline-view-render` worktree. This delivers geometry and picking APIs, not a registry
command, UI navigation or native workflow. Root owns those integrations and Git. Actions
remains disabled; the complete M0–M5 goal remains active.

## API and original coordinate policy

`PageView::fit(width,height,page)` retains `PageViewport::new` geometry and pixels exactly.
Its `project` identity path returns the original graph after cancellation/dimension checks.
The legacy fit's reported effective percentage is its smaller rounded paper-axis ratio.

`PageView::zoom(width,height,page,percent:f64,center:Point2)` requires finite percentage
within 1–999 and a center inside the closed page extent. At 100%, one authored point is
one logical canvas unit. This is an original Bondline convention, independent of DPI or
physical screen size. The public guide supplies Zoom's role/range, not this conversion:
[public GUIDE25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf), pinned
SHA-256 `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
Reference scale/default/unit semantics remain unknown; no program binary was inspected.

For explicit zoom, `s=percent/100` is one uniform coordinate scale. Raster paper extents
are `ceil(page_width*s)` and `ceil(page_height*s)`; they do not supply separate x/y scales.
Margin `min(32,width/6,height/6)` gives the full virtual canvas, including small extents.
The optional exact-scale path affects coordinate conversion, stroke width and continuous
physical-paper checks; the default path retains its original arithmetic.

Each camera offset is `round(destination_extent/2 - source_center)`, with Rust's
half-away-from-zero rounding. The rendered requested center can differ from the exact
canvas center by at most 0.5 logical unit per axis. Inverse conversion uses that same
offset. Glyph pixel rectangles translate by whole units and are never upscaled/resampled.

Methods are `source_size`, `zoom_percent`, `canvas_position`, `document_position`,
`contains`, `stroke_width`, `project`, `graph_scene` and
`graph_scene_with_display_hydrogens`. The graph methods validate exact PageSize agreement
and preserve IDs, coordinates, authored H/None and separate display counts. No model or
source ownership is added to the camera. New `InvalidView` denotes invalid view parameters
or mismatched page/source frames; existing geometry/cancellation/resource errors remain.

## Generation and clipping boundaries

The graph methods generate full virtual geometry using the shared existing chemical
producer, then project it. Whole-source labels, physical-paper ink, multiple components/
gaps, attachment, congestion and budgets are checked before destination clipping. A camera
entirely inside a valid Double gap therefore yields blank white paper, two retained logical
components and no visible bond hit; it does not turn clipping into a chemistry error.

Rectangles, glyph masks and glyph spans are source-clipped, translated with signed integer
arithmetic and destination-clipped. Stroke endpoints and width translate without shortening
their axes. Logical atom/bond identities remain; component hit polygons are rebuilt with
the existing exact visible-polygon helper. Partially visible labels can remain selectable,
while invisible stroke extensions do not become hits. Projection does not regenerate labels,
infer H or change model/source data.

This first explicit-zoom profile refuses source strokes whose existing butt corners escape
the full virtual canvas. Shortening diagonal centerlines would erase legitimate corner
pixels; keeping unchecked off-source ink would reveal data excluded from source checks.
Fit retains its original clipping behavior. Translated coordinates must retain the existing
finite ±100000 bound. These are explicit capability limits, not claims about reference
defaults or defective source files.

Virtual geometry is bounded to 100000 units per axis. Existing 4096 primitive/hit limits,
8–256 main-label-em bounds and pixel visibility/congestion guards remain. No virtual RGBA
frame is allocated. CPU/GPU readbacks stay limited to 2048 per axis; larger native vector
scenes remain possible. Some valid percentages/page sizes honestly refuse these bounds.
Large full-source congestion searches retain their existing cost and cooperative
cancellation; this slice promises no responsiveness benchmark or universal hardware identity.

## Test-first evidence

All runs used this worktree's absolute target, `CARGO_INCREMENTAL=0`, dev/test debug0 and the existing
Root-pinned external Python. No environment, dependency or lock installation/change occurred.

| Stage | Actual observation | Log under `target/` |
|---|---|---|
| Initial API RED, session 78586 | Exit 101, unresolved public PageView import; compiler evidence, not runtime behavior | `view-render-initial-red.log` |
| Nine tests declared before implementation, session 79705 | First real implementation: 9 passed, 0 failed | `view-render-first-runtime.log` |
| Full renderer, session 18740 | 75 passed: 65 existing + 10 new; 0 failed/ignored | `view-render-package.log` |
| Warnings-denied all-target renderer clippy, session 18740 | Exit 0 | `view-render-clippy.log` |
| Workspace rustfmt check, session 18740 | Exit 0 | `view-render-fmt.log` |
| Large-coordinate anchor RED, session 67573 | Exit 101: zero-tolerance hit returned Bond37 instead of Atom19 | `view-render-forward-map-red.log` |
| Unchanged anchor regression GREEN, session 31146 | 1 passed, 0 failed | `view-render-forward-map-green.log` |
| Final full renderer, session 31146 | 76 passed: 65 existing + 11 new; 0 failed/ignored | `view-render-package-final.log` |
| Final warnings-denied all-target clippy / rustfmt, session 31146 | Both exit 0 | `view-render-clippy-final.log`, `view-render-fmt-final.log` |

The first Fit test is an exact unchanged prefix of the nine preimplementation tests. One
additional Double-gap regression was added after their first GREEN; it passed without a
production change and is not presented as another RED. All predeclared expectations,
old tests/oracles/fixtures/fonts and the human-approved Single golden remain unchanged.

Independent expectations include:

- Fit at 320×240 / 128 pt: literal paper rectangles, 1.40625 stroke, endpoint positions
  (115,120)/(205,120), inverse mapping and stable typed application hits.
- Zoom at 200% / 128 pt: source 320², endpoints (16,60)/(144,60), width 2, and independently
  enumerated Single/Double/Triple black rows/gaps. CPU and GPU match every RGBA byte;
  a one-pixel geometry-shift control differs.
- Pan and a diagonal 400% scene: literal axis (-112,-68)→(144,188), width4, and the
  independent integer band `abs(y-x-44)<=2`. Boundary pixels prove no diagonal-cap trim.
- Fractional 87.5% on 123.5×81.25pt: source133×96, uniform 0.875, point
  (10.5,20.25)→(115.1875,101.71875), exact inverse and continuous paper-edge exclusion.
- Literal OH: actual newly generated 24-em source glyphs, an explicitly declared integer
  crop, clipped destination ink and CPU/GPU agreement. This projection check uses the old
  CPU source raster; it is not independent approval of glyph design. Existing external
  FreeType oracles and the approved golden pass separately in the unchanged package tests.
- Complete display-count coverage/None retention; whole-source LabelScale, BondScale and
  overlap refusals even when their geometry would be off camera; source clipping and
  translated-coordinate refusal; mismatched frame/page; cancellation; 4097-hit preflight.
- Virtual 4064² geometry projected into 100² without a full virtual framebuffer, plus a
  valid 100000² vector destination whose attempted CPU readback refuses before allocation.

The first ten-test source is 20644 bytes, SHA-256
`64be54a218635b2937607ea4e7ab063b0be7089563cf793781e660cd3827d7af`;
it remains an exact prefix of the final eleven-test source, 22637 bytes, SHA-256
`64fbbe525f5ab8c919c333379497946836d9a93f2298eba045acfa1657d51522`.
Initial source/log identities, old production bytes and current source/protected hashes are
retained in ignored `target/view-render-*` records. No new golden image was created.

Root's read-only review identified a forward-map precision discrepancy before integration.
On a 20000 pt page at 100%, the authored middle C3 point (10000.0001,10000.0001) produces a
source f32 anchor which projects to (50,50). The first direct conversion instead kept f64
through translation and returned approximately (50.0001,50.0001); a zero-tolerance hit then
selected the adjoining bond. The added literal regression independently reproduced that
exact wrong target before the fix. Explicit-zoom conversion now uses the same source-f32
anchor and integer translation as graph production. No hit tolerance, input or expected
target was relaxed, and all earlier ten bodies remain exact.

Inverse coordinates retain existing f32 canvas quantization: the rendered (50,50) maps back
to (10000,10000), while the model's authored (10000.0001,10000.0001) stays unchanged. Callers
must preserve stable hit identities and model anchors when snapping; they must not rewrite
authored coordinates from an approximate inverse. This is finite representation, not a
new grid-snapping or source-normalization policy.

## Receiving integration

Use the PageView graph builders for explicit fractional zoom. Passing `source_size` to
the legacy graph entry would select its rounded fit, not the exact uniform view. `project`
is a lower-level operation: matching dimensions alone cannot prove matching Page geometry.
Actual MOL/SDF display facts must still come from IO's source-qualified sidecar; filenames
and camera state grant no new file-edit or export authority.

The UI must include camera state/epoch in hover, gesture and preview freshness, since equal
canvas dimensions can have different coordinate frames. Its DPI/readability rules remain
separate. IO/commands/UI, native input, Windows/Linux execution, saved view preferences,
general panning gestures, performance measurements and complete navigation parity are not
verified by this renderer slice. No live worker job remains after the recorded checks.
