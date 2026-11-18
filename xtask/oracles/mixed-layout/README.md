# Independent mixed-source rendering evidence

Original FontTools/FreeType vectors for the locked
[mixed-label contract](../../../docs/research/mixed-skeletal-labels.md), on base
`f16b79f`, contract `b30f0d6` plus metadata amendment `5f58276`. The nine original sources,
strict independent reader and frozen Expat/Decimal facts were adopted separately in
fixture-only commit `71129ab`, before the IO producer. See the
[source fixtures](../../../crates/io/tests/fixtures/mixed-labels).

The generators consume those frozen facts, the bundled Apache-2.0 Roboto font and the
existing independent [bounded outline oracle](../clearance_oracle.py). They import no
Bondline product code or output. These original MIT OR Apache-2.0 scripts are portable
copies of the review lane's independent scripts; only input/output routes and command-line
options changed. No numeric method, fixture or uncertainty band changed after testing.

| Frozen artifact | SHA256 |
| --- | --- |
| `mixed-layout-vectors.json` | `321c94a52122074e525b7a5b0f4dc077faa960a475c9c19eff33ba034ff26ed0` |
| `mixed-painted-spans.json` | `2fecdebfc6a03f4d2e8422e6329191d991ad8c4e04491752f082b91a1ef748c1` |
| Source `mixed-label-facts.json` | `674375c3c610c1990692a2ec2d7724daceae408cef8604fa8ec315c96c2f7372` |
| Original layout generator | `9660ca3959e8dde116ff5e2d507f252894f573079f52315b5c65b2916ab48f00` |
| Original pixel generator | `92631f5f7dd6462fa64832cdc09f14441b41e6ae5e2e8bd6222ff4007f6ab1ff` |
| Portable `generate_layout.py` | `f2cc277c8b3b6130fa295546eb980c71dd278f25a9a120985335c5749fee925b` |
| Portable `generate_pixels.py` | `96f6f8404a857cf24850d326c728333b33b43d125637db932d3747a37cd1a2b0` |

## Reproduce without overwriting references

Use Python with FontTools4.61.1 and Pillow12.3.0. The default outputs are ignored files
under `target/mixed-labels/reproduced/`; `--output`, `--layout` and `--images` permit a
different ignored destination. Never regenerate the tracked references to fit output.

```sh
python3 -B xtask/oracles/mixed-layout/generate_layout.py
python3 -B xtask/oracles/mixed-layout/generate_pixels.py \
  --layout target/mixed-labels/reproduced/mixed-layout-vectors.json
cargo test -p bondline-commands --test mixed_render
```

Both generators were executed with explicit ignored receiving destinations, reproducing
the two frozen JSON files **byte for byte**, including the independent PNG hashes.
There are 21 layout scenes / 75 bonds and five painted scenes. The font SHA256 is in both
JSON headers and is checked by the independent font/pixel generators.

The layout vectors contain true glyph placements, hidden model IDs/anchors and separate
bounded begin/end cuts. Hidden endpoints have exact zero cut. Numeric cut acceptance
keeps the original 1e-4 canvas producer bracket, separate 8×f32 epsilon output-coordinate
allowance, and the oracle's 1e-5 canvas bracket. The tests also require exact unchanged
hidden endpoints and exact literal source bytes after display.

The painted vectors contain certain/possible finite butt-strip pixel-centre spans and
independent 4× FreeType glyph spans. Actual black ink must include every certain pixel
and stay within possible pixels. The frozen possible-minus-certain band contains exactly
**one pixel overall**, which remains unchanged. Glyph IoU must be at least 0.90. Nearby
certain-black probes at every hidden anchor detect an erasing mask or shortened stub.
These are independent original-rule references, not reference-app captures or human
approved goldens.

## Receiving evidence, 2026-11-18

The new [source-aware tests](../../../crates/commands/tests/mixed_render.rs) use actual
`StoredDocument` loading and existing `drawing_page_scene(_with_view)` APIs. The exact
test SHA256 before and after the producer is
`6d4f9d6ff0d5f4d5c8ca74e741b78106e1a4eb5f1acdbaede9480cc4a1396365`.
An initial test-only private-field getter compile miss is preserved separately; it is
not counted as a product runtime failure.

On fixture-only `71129ab`, the actual runtime RED is 0 passed / 2 failed / 0 ignored:
all 21 layout scenes and all 5 painted scenes fail specifically with `IncompleteSource`.
After integrating only IO producer `48d22a0`, the unchanged tests pass 2/0/0. All 21 whole
CPU/GPU frames agree exactly, 75 bonds satisfy the independent scalar/coordinate intervals,
and hidden IDs/hits/zero-cut anchors agree. Authored model run bodies/sizes are checked
against all 21 tuple records; actual cross-raster glyph placement is checked in the five
painted scenes. Private renderer glyph metadata is not presented as a public observation.

There is one explicit oracle setup qualification: frozen physical2x dimensions declare
1152×1152, while existing `PageView::at_display_scale(2,[.25,.75])` uses
ceil(576×2+phase)=1153 on each axis. Numeric origin (64.25,64.75), scale, glyph and cut tuples
are unchanged. The frozen painted spans are therefore tested as the 1152×1152 interior
crop, while exact CPU/GPU equality covers the entire actual 1153×1153 frame. The extra
outer row/column has no independent painted oracle; it is not claimed otherwise.
No producer, camera, frozen data, tolerance or band was changed for this qualification.

`cargo clippy -p bondline-commands --all-targets -- -D warnings`,
`cargo fmt --all -- --check` and `git diff --check` pass. The full commands/workspace
execution is Root's integration gate; it is not claimed by this focused test run.

Logs are retained under ignored `target/mixed-labels/render/`:
`source-render-compile-qualification.log`, `source-render-red.log`,
`source-render-green.log`, `reproduced-pixels.log` and `source-render-clippy.log`.
This lane changes tests/oracles only. Root owns full integration, command/UI/native
receipts and the aggregate gate; other platforms, reference pixels, human golden approval
and full M0–M5 completion remain separate. Actions and protected user apps stay untouched.
