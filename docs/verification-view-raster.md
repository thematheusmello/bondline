# Physical output for a logical PageView

Renderer-only continuation at base `af7fab0`, 2026-11-15. This implements the
approved bounded API from the SDF view proposal; it does not implement current-view
commands, SDF navigation, native presentation or a new source authority.

## API and geometry

`PageView::at_pixel_size(&self, width_px, height_px) -> Result<PageView, Error>`
returns an immutable rendering copy whose coordinates are physical output pixels.
The original view remains logical. `pixel_density()` returns one for an ordinary
logical view, or the copy's effective physical pixels per original logical unit.
`zoom_percent()` retains exactly the original percentage, including Fit's resolved
percentage; DPI is never multiplied into its public 1..999 range.

For original logical destination `L` and requested physical output `P`, the policy is

```text
d = min(P.width / L.width, P.height / L.height)
padding = (P - d*L) / 2
physical_position = d*original_logical_position + padding
```

The already-rounded logical camera offset is preserved. Its physical translation
is split into an integer floor and a fractional phase. The private viewport folds
that phase into fresh glyph/geometry generation; projection then translates the
result by integer pixels. No raster frame or glyph bitmap is stretched. A second
`at_pixel_size` call on any returned pixel view rejects with `InvalidView`, including
an identity pixel view: callers must start from the original logical view.

At identical logical/output dimensions, a dedicated identity path clones the old
viewport and projection fields. Existing Fit rounding, scene primitives, glyphs,
hits and RGBA stay exact. Other densities retain Fit's two independently rounded
logical paper axes rather than pretending its small anisotropy is uniform. Zoom's
exact uniform scale remains uniform.

The private density viewport keeps an f64 origin and axes. Floating-point stroke
anchors have the same bounded f32 storage/projection as the existing renderer.
The independently declared visible vectors use a 1/256 physical-pixel tolerance;
do not extrapolate that into a universal bound on off-camera coordinates. Two
bounded f32 casts give a conservative 1/128-pixel bound globally, plus f64 arithmetic
roundoff. Physical paper containment rectangles round outward with floor/ceil,
less than one pixel per edge. Glyph pixels are newly evaluated against their actual
physical outlines, not obtained by rounding old pixel rectangles. The existing
paper border/shadow producer retains its physical-pixel styling.

## Compatibility and guards

The returned copy works with the existing `graph_scene` and
`graph_scene_with_display_hydrogens` methods. The source-aware IO drawing adapter
can therefore receive the copy without duplicating source/H policy. SDF callers
still need a separately retained complete source and the all-record preview's Page
and exact display-H sidecar. This API never chooses a valence or mutates a Page.

Existing chemical/label/overlap checks, exact sidecar coverage and authored-H
agreement run before camera clipping. Resolved-H paper-ink checks remain enabled.
Nonidentity copies use the existing signed projection's closed safe-source
profile: source strokes escaping the virtual canvas are refused, not shortened.
The 1x identity path preserves legacy Fit's behavior instead of silently imposing
that newer restriction on it.

Physical output remains 1..2048 pixels per axis. Full virtual geometry/coordinates,
primitive/hit limits, physical glyph 8..256, stroke validation and cancellation
remain bounded by the existing producer. Large virtual images are not allocated.
The future UI must separately check actual logical main-label readability using
physical em divided by effective density, including capped output; a larger DPI
alone must not make tiny logical text acceptable.

**Logical Fit at higher density is distinct from legacy whole-page pixel Fit.**
For example, the protected UI oracle fits with a 32-physical-pixel margin, while
logical Fit at 2x scales its 32-logical-unit margin to 64 physical pixels. Their
difference exceeds rounding noise. Default whole-page captures and UI Fit requests
remain on their old path; only an explicit opted-in logical Fit may use this API.
No old capture, UI oracle, approved golden, dependency or font was changed here.

## Test-first and verification evidence

The initial seven tests were frozen before implementation: 14,127 bytes, SHA-256
`176d4785b3ef4ff58fd7c46f411f95fb2aa496c48c6f2ef071ebb881d92d0869`.
They use original literal model vectors and arithmetic expectations, plus the
unchanged external font consumer. They were not generated from renderer output.

- Session `91766`, exit 101: missing `at_pixel_size`/`pixel_density` compiler RED and
  resulting type-inference errors. No runtime failure is claimed. Log: 6,862 bytes,
  SHA-256 `b4c2eec8039531a22c9f0e732b8a366a21a650a5974c97f9d5707058a6c1d0b3`.
- Minimal implementation, then session `68213`, exit 0: the unchanged seven tests
  passed at their first runtime execution. No expectation/setup correction occurred.
- One additional test was appended for nonsquare Fit's independently rounded axes
  at 1.5x and 0.5x. This is postimplementation coverage, not an observed RED. The
  complete original seven-test prefix remains byte-for-byte exact.
- Session `96521`, `cargo test -p bondline-render --locked`, exit 0: **84 passed,
  0 failed, 0 ignored**, across 16 result lines. This includes the new eight tests,
  old 76 tests, CPU/GPU comparisons, unchanged font consumers and approved golden.
- Session `40201`, `cargo clippy -p bondline-render --all-targets --locked --
  -D warnings`, exit 0. No lint suppression was introduced.
- `rustfmt --edition 2024 --check crates/render/src/view.rs
  crates/render/src/page.rs crates/render/tests/page_raster.rs` exited 0.

The new tests establish:

1. Exact 1x scene/RGBA identity for Fit/Zoom, literal authored-H and display-H,
   unchanged source Pages and rejection of compounded density operations.
2. Independently declared 2x logical Fit anchors/stroke width and its intentional
   distinction from the unchanged legacy pixel Fit.
3. Signed 175.5% camera offsets at density 1.5625: anchors `(119,188)` and
   `(294.5,188)`, width `2.7421875`, inverse mapping and CPU/GPU equality.
4. Capped unequal output aspect: density `2.0475`, horizontal padding `0.25`,
   and literal anchor/center coordinates without percentage drift.
5. Fresh physical 96-em NH2 glyphs versus the protected FreeType consumer's
   uniform 4x/BASIC/BOX policy and unchanged 0.90 threshold; a wrong-symbol control
   rejects. Actual physical ink differs from nearest enlargement of 48-em logical
   pixels. CPU/GPU RGBA agree, and the source retains authored H=None.
6. Invalid physical dimensions, bounded virtual geometry and original 999%
   percentage independent of density.
7. Off-camera display-count disagreement, chemical unsupportedness, overlapping
   labels and paper-escaping ink still refuse; lower/upper physical glyph limits
   and cancellation remain active before clipping.
8. Nonsquare Fit keeps both rounded axes at fractional and subunit densities,
   with independent anchors/widths and CPU/GPU equality.

Reproduction uses the unique absolute worktree `CARGO_TARGET_DIR`,
`CARGO_INCREMENTAL=0`, dev/test debug=0 and existing pinned `BONDLINE_PYTHON`.
Ignored `target/view-raster/` retains the baseline, frozen source, RED/GREEN logs,
source manifest and final handoff. Only two existing renderer source files changed;
all other baseline files, assets, old tests, fixtures, oracles and goldens remain
exact. No Git, native window/input, hosted Actions, installer, dependency or broader
workspace run occurred here. All own jobs are terminal. Root owns receiving
integration, global gates and native/platform acceptance.
