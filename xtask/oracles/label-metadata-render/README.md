# Metadata-only source rendering: receiving evidence

Bounded tests for [contract8c2b608](../../../docs/research/label-default-metadata.md).
The original source/facts/helper are fixture commit `deb078d` (local `b56fd54`), with
[provenance and independent font checks](../../../crates/io/tests/fixtures/label-metadata/SOURCES.md).
Facts SHA256 is `249ecc97213e1949bf12246bafd0d8ee3654ce31923b40d197a0d896658f9e68`.
No expected image, glyph, geometry, tolerance or uncertainty band is generated here.

The three metadata sources/raw controls are:

| Metadata source | Existing original raw control |
| --- | --- |
| `mixed_a` | `created-labels/base.cdxml` |
| `created_3` | `created-labels/created_3_black.cdxml` |
| `caption_owners` | `created-labels/caption_base.cdxml` |

Each uses Fit576 and Fit1088, giving six actual whole CPU/GPU frame comparisons.
Source-known literal bodies/styles/FontTools boxes, 12/24px em ranges, atom hits and
the old independent Mixed/Created bond endpoint intervals remain exact. Entire primitive
lists and CPU pixels equal their metadata-free raw controls, including the caption case.
All six CPU frames equal real GPU readback on Metal, Apple M3 Max.

The `mixed_a` pair also directly reuses the existing
[`mixed-painted-spans.json`](../mixed-layout/mixed-painted-spans.json) Fit1x/Fit2x
certain/possible bond spans and FreeType glyph mask. Every certain pixel must be black,
every actual black pixel must remain possible, glyph IoU stays>=0.90 and the original
per-case uncertainty counts stay exact. This is direct independent pixel evidence for
two views; the remaining four use whole raw-control equality plus frozen font/layout
controls. No broader independent caption-raster claim is made.

Every actual run remains Roboto12pt face0. Unused default sizes10/14/16, family `Unused
serif`, faces96/32767 and Bond LabelSize1e308 do not change the model, mask, glyphs,
clearance or work. Old Boxed/Connected/Mixed/Flagged and full-authoring predicates remain
false for these new metadata sources; the tests use existing source-aware display APIs,
not a missing-new-getter compile check.

The negative substitutes only the independently pinned16pt OH ink box
`[91.42578125,154.15625,111.56640625,165.84375]` at node(96,160) while its actual run is12pt.
It must stay `IncompleteSource` at both views; unchanged Save still retains the raw source.
No default is consumed to make a mismatching box valid.

## Actual RED/GREEN

On `8c2b608` plus fixture-only `deb078d`, the existing-API runtime result is1 passed /
1 failed /0 ignored: the wrong-box negative passes, and all six positive view calls fail
`IncompleteSource`. The first test receipt precedes the additional strict reuse of the
old pixel mask; the six source/view inputs and negative remain unchanged. The final test
SHA256 before GREEN is `8d67dd0ad3cba93bd4a9258b8f0c02db41aadae7e6dc4089c3155e6d8dc8f33c`.

The exact temporary three-file IO patch SHA256
`8d6ac6f32ac4f9d44086b3d300d0de9e73ef54152f7ed96e65dc40f62ba6b347` first passes2/0/0.
It is then reversed before cherry-picking final IO `47b021b` (local `2854c60`, including
the checked immutable-baseline fast path). The final unchanged tests again pass2/0/0.
No temporary producer patch is committed by this lane.

Reproduce:

```sh
cargo test -p bondline-commands --test label_metadata_render -- --nocapture
cargo test -p bondline-render
```

Actual logs are ignored `target/effective-labels/architecture/render-red.log`,
`render-green.log`, `render-final-green.log`, `render-full.log` and `render-clippy.log`.
Final full renderer passes195/0/0 across40 reports. Warnings-denied all-targets clippy
for commands and render, workspace rustfmt and diff checks pass. Root owns workspace, adapters,
native acceptance and synced integration; other platforms, golden approval, omitted-run
inheritance and full M0–M5 remain separate. This test-only lane changes no producer,
old test/reference, dependency, font, asset, native/user app or Actions setting.
