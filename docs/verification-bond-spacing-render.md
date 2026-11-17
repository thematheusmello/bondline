# Explicit root bond-spacing renderer

This isolated renderer slice uses explicit document-root percentage spacing and the
preferred DrawingSettings bond length. It does not resolve foreign defaults, absolute
spacing or per-bond inheritance, and adds no command, native binding or source authority.
Missing spacing retains the original3pt separation independently of authored line width.
The original48pt fallback for an absent preference applies to Bondline's own eligible
native/source-free scope; it is not a CDXML-wide default.

## Independent inputs

The original numeric fixture in
`crates/render/tests/fixtures/bond-spacing-oracle.json` records ten source-coordinate
outputs from Apache-2.0
[Octet-Draw1.1.2 at6395bd5](https://github.com/SamChemMan/Octet-Draw/tree/6395bd582d2c5ba9404e9ddaf175e52e2a9f387c).
The test-only external program parsed our manually authored skeletal C3/C4 documents,
then returned actual filled-polygon cross sections. Its unchanged parser/style/render
sources were Git-blob verified; font/pictogram shims rejected use and recorded zero calls.
No upstream source, asset, font, template or reference-program output is shipped here.

The external first run passed28 declared records/201 analytic comparisons, with14
separately classified capability observations. Its raw output SHA256 is
`d72a89e6823959cce0127cd5edb3fd5464bc78ef1063ac59c8385f5987be15c7`;
terminal receipt SHA256 is
`119dd1b0c27e10b6d93b315ae8c0d8c27d05c0abb723ff2aa553f91f75b4e5da`.
All ten transcribed numeric vectors match the frozen summary in76 comparisons. These
are program-specific centreline observations, not Professional26.1, raster or native
verification. Its unhonoured Abs/invalid-value controls are excluded, and its automatic
median rescaling is not used. See the public [evidence](research/bond-spacing-evidence.md).

Independent Rust expectations cover separation `p*preferred/100`, clear gap `s-w`,
centered Double offsets±s/2, Triple offsets±s, actual/preferred contrasts, width changes,
90-degree sided bisectors/reversal, physical density/fractional origin, picking, literal
H/glyph preservation and bounded refusal. Actual20/preferred40/p18 and actual40/
preferred80/p18 distinguish nominal spacing from actual length and exceed the legacy
one-third restriction. Explicit100% is a separate original analytic acceptance case.

## Test-first chronology and receiving

The isolated branch starts atb377456 and receives data commitf518128 as local8dc84c6.
That dependency is separate from the renderer commit. The first existing-renderer-API
test was frozen before production edits at2641B/SHA256
`927a3b2e0766333945302083e48f353c79ec27644e0b509f1064518b09dab361`.
It deserializes explicit15% and expects14.4px separation at2x with the original48pt
fallback. A separate expanded test unit uses the future `with_drawing_settings` API;
its missing-method qualification is separate from a runtime RED.

After receiving only the data dependency, first44595 exits101:0/1 at the literal row
assertion. The unchanged renderer still uses its old separation. Log3425B/SHA256
`37803b63a8ba89d37c03e4f462037d20a16c198d220472fbdd780608270de076`.
The expanded target then exits101 with exactly three E0599 instances of the missing
`PageView::with_drawing_settings` method, not a behavioral test result. Log2141B/SHA256
`315c0ba7e7bdc8d9785b9edbefa0ca777a2c1e84625d0ed25b2f7b5362dcc53d`.

The implementation adds that settings builder and carries the Copy settings through
existing physical-view transforms. Only explicit percentages use the new spacing path.
Absent spacing keeps the old offset multiplications and one-third guard. Explicit
spacing uses the same bisectors, strict finite ink-gap search, masks, joined-arm rules,
collision corridors and paper certificate, with its resolved separation replacing the
old3pt constants. It retains real positive-gap/visible-pixel, coordinate/width, hit,
primitive and cancellation bounds. Degenerate data-valid spacing is a rendering refusal;
it never becomes a hidden fallback, rescale, clamp or moved endpoint. Label em/glyphs and
literal H are independent of spacing. Geometry assertions allow less than1/4096 canvas
unit for f64-to-f32 construction; CPU/GPU RGBA comparisons are exact.

First product37859 exits0: initial1 plus expanded6 pass unchanged, including actual
Metal/Apple M3 Max CPU/GPU equality. Log1390B/SHA256
`0ca99fdd515dc774443044ec13b908e0dfc03c5d95fad40ab538c816a981b7e9`.
Formatting leaves the initial source exact; expanded16505B/SHA256
`6d6f11ee92eaa56d9e71508b157e11ee53cfe79f7abeb96a36fcf8dd99aafa30`
is exactly rustfmt2024 of declared13403B/SHA256
`155603cfce2b2395732023b9d9138ae9239c12cbbb3ccc9b501cf420b6f697c3`.
The original declarations, compiler failure, runtime failure and exact formatting proof
remain in ignored `target/bond-spacing-render/`; no expectation/input was changed.

After formatting, gate8594 exits0: **153 passed/0 failed/0 ignored** across30 result
lines, including the renderer library and every renderer integration test. Log21446B/
SHA256 `795201a2c8993acae7055d43ea4030049e180b522fd9608894bcbf6987d1ec37`.
All34 pre-existing test/oracle/golden files remain hash-exact. Warnings-denied all-target
clippy82176 exits0, log2234B/SHA256
`c0d73f522d8097c2329033737fd4773963661929dc942a0939e6470de45eb5d0`;
renderer formatting check exits0.

Reproduction in this checkout:

```sh
export CARGO_TARGET_DIR="$PWD/target/cargo"
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export BONDLINE_PYTHON="/Users/sarenaberg/Workspace/doozle/open-source-world/bondline/.cache/oracles/format-python/bin/python"
cargo test -p bondline-render --lib --tests --locked -- --nocapture
cargo clippy -p bondline-render --all-targets --locked -- -D warnings
cargo fmt --package bondline-render -- --check
```

The actual target path was the absolute isolated
`/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-spacing-render/target/cargo`.
All jobs are terminal. Goldens, fonts, dependencies, locks, workflows and old assertions
are unchanged. This receiving record covers the renderer only; complete-owner IO/command/
UI propagation and the full workspace gate belong to Root integration. No native-window,
OS-DPI, Windows/Linux, foreign-default, Abs or per-bond inheritance result is claimed.
