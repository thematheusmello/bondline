# Styled caption renderer receiving

This isolated slice implements explicit single-line regular captions from the checked
model, not source admission, an editor or general text shaping. Read Root's
`docs/research/styled-captions.md` contract during integration. Existing chemical-label
rendering, metrics, font bytes and approved goldens remain unchanged.

## Geometry and limits

Every Page text and run is visited. Exact `Roboto`, face0 uses the existing Apache-2.0
Roboto Regular file; unknown families/faces, missing cmap glyphs, line controls,
combining marks and complex/bidirectional sequences refuse with `UnsupportedText`.
Literal precomposed Unicode glyphs use their cmap outlines; no alias, tofu, synthetic
face, chemical interpretation, GSUB, contextual/mark positioning or bidi algorithm is
claimed. Caption position is a left baseline in authored points. Font em is explicit
point size times the existing viewport scale, separately from chemical12pt labels.
RGB channels become opaque8-bit color with nearest rounding. Kerning resets at each
authored run boundary; empty/no-outline glyphs still advance and keep run metadata.

The actual bundled font has **no legacy kern table**. Existing ab_glyph's kerning helper
therefore cannot implement the required pair behavior. Root's separate eff16ae dependency
commit exposes the already pinned ttf-parser0.25.1 package; no new external package/font
is introduced here. The original checked renderer uses its public GPOS API and the pinned
font's DFLT kern features, covering pair formats1/2 with horizontal advance only. FontTools
independently confirms all of this font's Latin/Greek/Cyrillic/default kern features share
lookup13. [Primary OpenType GPOS pair specification](https://learn.microsoft.com/en-us/typography/opentype/spec/gpos#lookup-type-2-subtable-pair-adjustment-positioning)
describes these advance adjustments. No third-party implementation is copied.

A first immutable walk validates every run/font/cmap, actual outline/paper bounds,
per-run physical em8..256, at most4096 runs and4096 glyphs, and at most1048576 total
glyph-bounding pixels before glyph bitmap allocation. Spaces count toward work. Every
outlined glyph needs at least one remaining scene primitive; actual coalesced spans are
checked before each push against the existing4096 scene limit. A second walk rasterizes
bounded glyphs and emits thresholded horizontal rectangles with existing CPU/GPU painting.
Cancellation is checked during scanning, raster callbacks and rows. No atlas, shader,
font, dependency package, system font access or hit target is added.

`GraphScene::text_em_range_px()` returns the actual minimum **and maximum** caption-run
em including blank runs; logical/native readability must evaluate both after undoing
device density. The old `label_em_px()` reports only chemical labels. The same viewport/
physical-phase/project paths carry captions; no text bitmap is enlarged after rendering.
Text editing/selection and native/other-platform interaction remain Root integration work.

## Test-first and independent evidence

Initial1340B/SHA256
`72e2e54a9997af456715d4c642c4a4d9766d301b4820aba687ad9856a5260c09`
declares a red H caption through model APIs and the existing graph producer. Before the
dependency it gets only intended E0432/E0599 model-API qualification (45020 exits101;
log3919B/SHA256 `6bead8c6106c241d52b9d21f2bc5e78bbae612f8e98852eb9b920e5c9917cdf7`).
After exact dependency commits eff16ae and3f001095 (local fcd6f9d/7835fb4), **the same test**
reaches runtime0/1 at missing red ink on the unchanged renderer (87777 exits101;
log1155B/SHA256 `8da2fad655c964f83d3b083806b186873e504e2c94b742cbb84699a07d64d27b`).
Expanded declarations then get only five E0599 instances of the missing text-range getter,
not a runtime result. Seven expanded cases remain independent of the renderer; the
complex-shaping suffix retains the original six-case declaration as an exact prefix.

Original `caption_font_vectors.py` actually runs with global FontTools4.61.1 against the
unchanged font SHA256
`56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d`.
It extracts cmap/hmtx/glyf and GPOS expectations, including UPEM2048, A1336/V1303 advance,
AV−87, VA−75, To−99 and space-T−40. The original first extractor's nonexistent-kern
assumption failed and is retained separately; the corrected GPOS extraction precedes
production. Numeric fixture2090B/SHA256
`7a84a042718f8b999dfa9235d0f5d294fd13fb9c451cdbcf5fb0defaa3f30534`
contains only original measurements of the already licensed asset, not external code.

The separate `caption_oracle.py` uses these OpenType positions plus pinned Pillow12.3.0/
FreeType at4x, then compares each exact run color against actual renderer rectangles.
The predeclared coverage threshold is90% per-color IoU, allowing different hint/raster
implementations without moving expected glyphs. Actual positives cover multiple runs/
sizes/colors, Greek/precomposed Unicode, spaces, pair kerning and fractional device phase;
wrong baseline, color, glyph and missing kerning controls must fail. CPU/GPU checks compare
every RGBA byte, with actual Metal/Apple M3 Max. No approved golden is regenerated.

First product88262 compiles but gets3 passed/4 failed: the four new checker failures occur at its
last unexpected-color assertion after the font IoUs pass. The new Rust helper omitted
the existing page shadow210 from its background filter. A one-predicate setup correction
excludes that third known background color, retaining all glyph/color/position expectations
and all checker assertions; the old independent page grid explicitly defines shadow210.
The failed source/log, inverse and qualification are retained. No product change fixes it.
Qualified9773 exits0: expanded7 + initial1 pass. Five actual external positives have
minimum per-color IoU0.9653998865570051; all negative controls refuse. Log2082B/SHA256
`b93b272363db69fadf3f96f7f9867424e351f060c8a655604a34898d6bbea19b`.

Rustfmt-only proof retains every declaration input/assertion. Initial source becomes
1400B/079adcef; expanded13272B/999f58bd. First full renderer5631 exits0:161/0/0 across32
result lines, before the final necessary primitive-lower-bound preflight. Log23338B/
SHA256 `2f953739fbe0eb7af8c4488ddb214743f1237668e4e7ab53d409c8f63b706c96`.
All-target warnings-denied clippy73899 on the final producer exits0, log2234B/SHA256
`8e82f20685241f918ec11b187de7d1e50104cdef0dbb3415073c390902a37f23`;
renderer formatting check exits0. Final post-bound receiving37305 exits0:
**161 passed/0 failed/0 ignored**,32 result lines, log23338B/SHA256
`0763cf02c19d012b113a7823a5107ba6636d3eb4b5a2156742bbf45c40f19204`.
All37 pre-existing renderer test/oracle/golden inputs remain hash-exact. No producer or
test input changes follow these gates; all owned jobs are terminal at handoff.

Execution uses a separate absolute `bondline-caption-render/target/cargo`, jobs1,
incremental0, dev/test debug0 and literal Root
`.cache/oracles/format-python/bin/python` for Pillow. FontTools extraction uses the
already available global4.61.1; Cargo tests consume its checked numeric fixture and need
no new installation. Reproduction:

```sh
python3 crates/render/tests/caption_font_vectors.py assets/fonts/Roboto-Regular.ttf
cargo test -p bondline-render --lib --tests --locked -- --nocapture
cargo clippy -p bondline-render --all-targets --locked -- -D warnings
cargo fmt --package bondline-render -- --check
```

Ignored declarations, exact source/receipt pins and failed/qualified logs are under
`target/styled-captions-render/`. Whole-source CDXML authoring/commands/UI, full workspace
gates and native OS/DPI/Windows/Linux execution are outside this renderer receiving.
No app or user drawing is touched; Actions remains OFF.
