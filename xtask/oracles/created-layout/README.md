# Independent created-label render evidence

Bounded acceptance for [contract 126f99e](../../../docs/research/printed-label-creation.md)
on base 7113237. The original 20 source fixtures, 64 FontTools boxes and independent
Expat/Decimal/RDKit facts were frozen separately in `9acd1a5` before the IO producer;
see [their provenance](../../../crates/io/tests/fixtures/created-labels/SOURCES.md).

This lane derives only five representative sources: `created_3_black`,
`created_3_blue14`, `created_3_nh`, `created_all_black` and
`created_3_selected_p12_5_m7_25`. Three established views give15 scenes and60 bonds:
Fit576 (scale1/origin32), Fit1088 (scale2/origin32), and logical576 at2× plus phase(.25,.75).
The latter declares actual 1153×1153 dimensions upfront, with origin(64.25,64.75).
No frozen crop or old-reference dimension is changed.

The scripts reuse the existing independent
[FontTools bounded-outline oracle](../clearance_oracle.py) and
[mixed-scene reference method](../mixed-layout/README.md). They import no Bondline
product code/output. Endpoint cuts use the same analytic line/quadratic envelopes,
Euclidean butt-strip margin,1e-5 canvas oracle bracket and the separately locked output
float allowance. The consumer retains the original1e-4 canvas producer bracket and
8×f32 epsilon coordinate rounding; hidden endpoints have exactly zero cut.

The five pixel cases are black1×, blue14pt2×, NH physical2×, all-black2× and moved
physical2×. Certain/possible **bond** spans use the unchanged finite butt pixel-centre
rule. Before the actual numeric/pixel test run, these independent spans declared exactly
**three uncertain pixels overall**. Glyph spans are retained per model atom/RGB using
4× FreeType, with the existing predeclared cross-raster IoU>=0.90 threshold. No threshold,
span, band or numeric vector was fitted after viewing product output.

New black glyphs share a colour with black bonds. The tests therefore inspect actual
public Stroke primitives and RGB Rectangle glyph primitives separately against their
independent references, then require exact full original-scene CPU/GPU frame equality.
They do not count a black child glyph as bond ink or erase uncertain pixels. Retained
hidden-anchor probes also require actual black ink in the complete composite frame.

| Frozen artifact | SHA256 |
| --- | --- |
| Source `creation-facts.json` | `15f6f0cb086bc300327cc141f72db669d86f373017f5d5fff5bcfe6d1a7e264f` |
| `created-layout-vectors.json` | `8166019c6636c919e1cf4f4a8ea0096aedb50ba94db94d6bbe2b3173803287d3` |
| `created-painted-spans.json` | `1174d36fca0d2cb17f0474826757c73ad86d07c29bb52cf7dc246fb5c6ca3139` |
| `generate_layout.py` | `8b28d6a13f2f33c2487b2abcc83892cd3da60c45e6d28e5c4370e2eedaef29f6` |
| `generate_pixels.py` | `8215af02ed71378287f1cc5df214e7abea9f2170649a709bc269c17b4d7cb707` |
| `created_label_render.rs` before numeric/pixel execution | `f0b0e26669c49643ecc59435c2ed9763736ce412c264269059625c10e0806a2f` |

## Reproduce without changing references

Use FontTools4.61.1 and Pillow12.3.0. Font SHA256 is pinned in the vectors and independent
generator. Defaults write ignored outputs; never overwrite frozen references to fit a
product result. `--output`/`--layout`/`--images` can select other ignored destinations.

```sh
python3 -B xtask/oracles/created-layout/generate_layout.py
python3 -B xtask/oracles/created-layout/generate_pixels.py \
  --layout target/label-creation/reproduced/created-layout-vectors.json
cargo test -p bondline-commands --test created_label_render
```

Both scripts were independently rerun to ignored receiving destinations. They reproduce
the frozen JSON bytes exactly, including the five independent PNG hashes. PNGs are oracle
illustrations, not actual app/reference captures or human-approved goldens; none is shipped.

## Actual receiving evidence

On fixture-only 9acd1a5, the first existing-API case constructs a current created-label model
on the immutable original source using deliberately supplied style and independent child
facts. Actual source-aware display fails `IncompleteSource`:0 passed/1 failed/0 ignored.
This is a baseline-backed creation failure, not a claim that standalone flagged sources
failed (some could render through the temporary old-profile widening after renderer work).

After cherry-picking only IO producer 33092ca, the original case is GREEN and the expanded
independently frozen suite passes 3/0/0. All 15 current-source scenes retain exact expected
source bytes; all 60 scalar/coordinate cut intervals and hidden anchors/hits agree; all 15
entire CPU/GPU frames agree. The five stroke/glyph comparisons pass their unchanged bands
and per-RGB IoU threshold. Current data is constructed on the original source, never rebased
onto a generated file or numerically guessed from product output.

Logs are under ignored `target/label-creation/render/`: `created-source-red.log`,
`created-source-green.log`, `created-pixels-freeze.log`, `created-pixels-reproduced.log`,
`created-full-render.log` and `created-render-clippy.log`.
Full renderer execution passes 195/0/0 across 40 reports. Warnings-denied all-targets
clippy for commands and render, workspace rustfmt and diff checks pass. No old test,
reference, tolerance or producer changed. Root owns full commands/workspace, adapter, native and release
verification. Other platforms, reference pixel parity, golden approval and full M0–M5
remain separate. No renderer producer, old test/reference, dependency, font, asset,
workflow, user app or Actions change is made in this lane.
