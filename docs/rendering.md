# Rendering evidence and scope

Scenes contain a straight RGBA8 background and an ordered Primitive list of integer
rectangles and finite constant-width butt strokes. Every primitive replaces covered pixels,
including alpha. Readbacks have no blending or antialiasing. GPU execution never substitutes
CPU output. The native view consumes the same scene; egui may antialias vector stroke edges.

Stroke coverage samples pixel centers at x+0.5,y+0.5. The CPU reference uses f64 dot/cross
products to test the closed finite-width segment. GPU WGSL evaluates projection and signed
perpendicular distance in f32 inside a conservatively clipped integer bounding rectangle.
An 8-byte viewport header precedes 48-byte primitive records; binding size/alignment and
vertex/fragment storage access were checked against pinned wgpu 22.1.0 and WGSL APIs.
RGBA8Unorm readback removes row padding. Backend tested: Apple M3 Max / Metal.

Independent expectations precede the implementation: exact horizontal/vertical/diagonal
pixel grids, reversed endpoints, interleaved source replacement and an i128 dot/cross oracle
for quarter-pixel inputs. CPU output must match that oracle exactly. GPU differences are
permitted only within 1/32 pixel of a true boundary, declared before comparison; there is no
universal bitwise claim for oblique f32 edges. The approved horizontal scene compares exactly.

Readbacks are 1..2048 pixels per axis with at most 4096 mixed primitives. Rectangle dimensions
must be nonzero with no edge overflow. Stroke endpoints are finite within ±100000 canvas
units, width is finite in (0,100000], and squared length must be normal in f32. Invalid input
fails before adapter initialization. Geometry-only page layout supports 1..100000 units.

PageViewport shares paper fitting, physical-to-canvas mapping and pointer conversion. Margin
is min(32,width/8,height/8); rounded dimensions are centered and occupy at least one unit.
Separate x/y conversion factors follow that rounding, whose aspect error remains at most
(1+aspect_ratio)/2 units. One point is 1/72 inch. The original preview style is gray 232
background, gray 210 shadow, gray 196 border and white interior. This is not a journal preset.
Existing hand-authored 8×8 page-grid, clipping and odd-pitch tests continue to pass.

The first chemical producer supports independent two-carbon fragments with one single bond,
positions, no isotope/charge/radical, and three explicitly authored hydrogens per carbon.
The creation command supplies that closed C2H6 case; the renderer performs no valence inference.
Other structures fail explicitly. Source-backed documents still cannot be faithfully previewed.
This limited workflow is not general atom/bond, style, stereo or reaction rendering.

IUPAC GR-2.1.2 requires terminal labels for an isolated ethane bond. The producer uses H3C/CH3
with the 3 subscripted, keeping C centered at its authored endpoint and facing the bond.
The original style uses one-point strokes and 12-point Roboto labels; subscript em is 2/3
and baseline shift is 1/4 em. The existing Apache font is the only label font. ab_glyph's
height-based scale is explicitly converted from em units. Glyph coverage is thresholded at
0.5 and coalesced into black horizontal spans over white masks; CPU and GPU consume those
same spans. General shaping, styled captions and smooth text rasterization remain missing.

Label em is bounded to 8..256 canvas units, with each glyph bitmap at most 512×512. Unreadable
scales, off-viewport labels, overlap and excessive primitives return errors; content is never
dropped. Masks may extend beyond the paper into the canvas; physical-page clipping/export
is not implemented. FontTools 4.61.1 independently supplied glyph IDs/metrics. External Pillow
12.3.0/FreeType tests require ink intersection-over-union >=0.90 at 96 em, chosen before
comparison; observed values are 0.9660 and 0.9669. This is separate from golden approval.

The user approved the exact 1024×1024 native-document Metal capture on 2026-11-13. Its PNG,
coordinates, SHA-256 and approval scope are in crates/render/tests/goldens. External Pillow
decodes it, verifies its hash and compares every RGBA byte with CPU and actual GPU output;
deliberate pixel corruption must fail. This approves only that scene. The older blank-page
candidate was not approved, and broader images/platforms/publication quality remain open.

Actual native Mac evidence now covers drawing from empty, Undo/Redo, visible overlap errors
and Undo recovery. Raw pointer events are consumed in order, including complete/multiple
drags in a single frame; stale revisions, Escape, focus loss and outside release cancel
pending edits. Click jitter up to three logical units does not create a bond. Cancellation
variants have input-test evidence; native draw/history/error recovery was directly inspected.
A canvas hit target must not advance layout past its error text: an observed clipped-error
regression is tested. No native Windows/Linux or universal performance claim.

Render workers retain cooperative cancellation, two-worker budgets and immutable input
revisions. CPU scanlines and GPU readback polling check cancellation; GPU readback has a
ten-second deadline. Driver calls and submitted GPU commands cannot be preempted. Label/
scene preparation is bounded but is not a general cancellable chemistry-job system.
