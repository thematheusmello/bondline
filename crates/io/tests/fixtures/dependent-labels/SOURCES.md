# Original dependent-label translation vectors

These 19 CDXML diagrams and expected facts are original Bondline test data, dual-licensed
MIT OR Apache-2.0. No proprietary app, binary, exported document, font, icon, template or
other reference asset was used. The source profile and operation are locked in
[dependent-label translation](../../../../../docs/research/dependent-label-translation.md).

The independent reviewer froze these files under the main worktree's ignored
`target/dependent-labels/independent/` on base `9cf7103` before IO production changes.
`move-vectors.json` pins the byte count and SHA256 of every unchanged `.cdxml` input/output,
plus explicit source-to-application IDs. Do not regenerate these files to match production.
`dependent-label-facts.json` records independently read exact graph, styles, runs, colours,
points and boxes. `independent-checks.json` records the 45 FontTools box checks. Glyph boxes
use existing Apache Roboto, SHA256
`56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d`, and independent
FontTools4.61.1 BoundsPen/hmtx/GPOS. These are source-data vectors, not approved images.

The original generator uses Expat to locate only existing Node.p/child BoundingBox spans
and Decimal to add the declared deltas. Compound sources start with the separately frozen
water→ammonia label edit. The tolerated box carries a literal +1/262144pt left offset;
rounding-boundary data deliberately distinguishes unchanged128pt Point2 from a representable
box-side increment. Current-geometry negatives are independent segment configurations.
The reader uses Expat/Decimal and separately constructed and direct-CDXML RDKit2026.09.2
literal formulas without inferring H, valence, source authority or reference-app placement.

The copied generator/checker have only path-routing changes: repository discovery works
from the tracked fixture location and generated/check outputs go to ignored
`target/dependent-labels/independent/`. Their mathematics, methods, values and frozen source
bytes are unchanged. The production test consumes the frozen facts and invokes the copied
independent reader on actual output. Use a Python with FontTools4.61.1 installed; the
checker separately invokes the documented RDKit virtualenv at
`.cache/oracles/format-python/bin/python`:

```sh
python3 -B crates/io/tests/fixtures/dependent-labels/check_moves.py
```

This checks the tracked frozen inputs and writes fresh independent results in the ignored
target folder; it does not overwrite the tracked facts. `generate_moves.py` is provenance
and reproducibility tooling, never an automatic test-reference update.

| Frozen artifact in original ignored location | SHA256 |
|---|---|
| move-vectors.json | `93f28f6a5833c7942aedb9615302b57a25e01a4e87ce694f52af93319626b335` |
| generate_moves.py (before path-only adaptation) | `4227d31414ebf0489461da95ee95fe2ce93c5cf25776d7712f015f3ba543d325` |
| dependent-label-facts.json | `f8d6fcfbf64c86d33b3d83e5f688df4766113fccff1a269e38e4565b87345b81` |
| independent-checks.json | `31f13de9a7fb284032ab95dd6325d90a13014ec62c9e6caf094a98f01f923bd0` |
