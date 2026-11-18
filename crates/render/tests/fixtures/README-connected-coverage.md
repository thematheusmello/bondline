# Connected-source stroke coverage regression

`moved-connected-coverage.json` is original numeric test data extracted with Python's
standard-library XML reader from the frozen
[saved source](../../../io/tests/fixtures/dependent-labels/connected_nudge_p25_m2.cdxml).
The source SHA256 is
`757dbeb94faecbfc94b63547af4a09892a21e4458288ad68a4cc744ad9593db4`;
the JSON SHA256 is
`f9d3eb5570746c6f894baa12676dbb5a6fa846c30f2239edec1ff6e0466ced83`.
It records the unchanged point, raw BoundingBox, literal body, size and drawing metrics
after translating all three atoms by (25, -2) points. It is neither a reference-app
artifact nor an approved golden. The test reconstructs the pure model directly to avoid
a renderer-to-IO dependency.

At the default 1088 × 1088 Fit view, the former GPU implementation differed from CPU at
nine pixels: (517,436), (522,436), (521,439), (526,439), (538,448), (537,451),
(542,451), (541,454), (546,454). The test transfers actual f32 primitive bit patterns to
an independent Python `fractions.Fraction` oracle. Its exact dot/cross-product inequalities
classify the closed finite butt rectangle at each pixel centre, followed by ordered
source replacement. All nine classifications agree with the existing CPU predicate;
the test additionally requires exact whole-frame CPU/GPU equality. Separate all-pixel
rational checks cover closed cap/side ties, reversed endpoints, clipping, conservative
bounds containing no covered centres, and translucent ordered replacement.

The defect was the normalized direction/length rounded to f32 before shader coverage.
A shader epsilon would change the frozen pixel contract. Compensated f32 calculations
would also need stronger evaluation guarantees than the public
[WGSL specification, §15.7.2 and §15.7.5](https://www.w3.org/TR/2026/CRD-WGSL-20260825/)
provides: implementations may reassociate and fuse floating-point expressions, and
`fma` is not required to be fused.

The GPU path now reuses the existing monotone closed-stroke lattice search to prepare
one integer [left, right) interval per clipped row. The CPU pixel predicate is unchanged.
The host uploads these geometry intervals and ordered primitives, never a CPU RGBA frame.
The GPU draws the same ordered coarse quads, selects coverage with integer row intervals,
and replaces straight RGBA8 without blending. Integer pixel-cell bounds enclose all
covered centres, including edge ties.

The existing limits remain 4096 primitives and 2048 pixels per capture axis. Checked
preflight arithmetic bounds row storage to 4096 × 2048 × 8 = 64 MiB and primitive storage
to 8 + 4096 × 28 = 114696 bytes. Both host vectors reserve the checked requirement once;
both GPU buffers are checked against the device's storage-binding and buffer-size limits.
Worst-case row data therefore needs 64 MiB on the host plus 64 MiB on the GPU, in addition
to primitive/readback data and allocator/driver overhead. Cancellation is checked per
primitive and per prepared row. This is bounded host scan
conversion plus GPU drawing, not a claim of GPU-only coverage or measured acceleration.

The original nine-pixel runtime RED and subsequent exact GREEN were observed on Metal,
Apple M3 Max. Receiving logs are retained under ignored
`target/dependent-labels/review/render-coverage-*.log`. Other platforms and adapters need
their own execution; this test does not establish reference-application pixel parity.

## Receiving evidence, 2026-11-18

The unchanged-producer runtime RED on base `783a632` failed 0 passed / 1 failed / 0
ignored. Expanded diagnostics independently classified all nine samples before reporting
exactly the same nine whole-frame differences; the failure was not only a first-pixel
comparison. A subsequent compiler-only attempt missed the helper's private visibility;
the scoped visibility correction changes none of its arithmetic.

With the row-geometry producer, the two focused cases pass 2 / 0 / 0. The entire renderer
suite passes 186 / 0 / 0 across 37 test executables/doc-test reports. No old assertions,
fixtures, tolerances or goldens changed. `cargo fmt --all -- --check`,
`cargo clippy -p bondline-render --all-targets -- -D warnings` and `git diff --check` pass.
The full renderer run uses the repository's pinned external Python environment for its
existing FontTools/Pillow tests; the new Fraction oracle itself needs only Python's
standard library.

The decisive logs are `render-coverage-red-complete.log`, `render-coverage-green.log`,
`render-coverage-full.log` and `render-coverage-clippy.log` under the receiving directory
above. This is renderer-scoped verification, not a full-workspace or cross-platform run.
Root integration, adapters and an actual native rebuilt capture remain separate checks.
