# Cyclobutane CDX fixtures

These four original CDX files, their Python generator and their verification
code are dual-licensed MIT OR Apache-2.0. The generator packs each record from
public field definitions. It uses sparse Node IDs 19, 2, 300 and 77 and omits
Element, NodeType and NumHydrogens on all four atoms. [Public sources and display rules](../../../../../docs/research/cdx-skeletal.md).

The four single bonds connect (128, 128), (256, 128), (256, 256) and
(128, 256) on a 512 × 512 pt Poster. Ink is 4 pt wide, the margin is 2 pt
and the palette supplies white at index 2 and black at index 3.

The ordinary ring has no carbon visibility flags. The dormant variant sets the
terminal flag on a two-neighbor atom. Neither requires a printed label. In the
active variant, the nonterminal flag requires one. The inconsistent-H case
declares three attached H on that same atom, exceeding the neutral carbon
valence given its two bonds.

| File | Bytes | SHA256 |
| --- | ---: | --- |
| cycle.cdx | 301 | 2da55160f64fa3e3bfd0aafa186581ce6ee1bfadeb101f5c446227f73a04d563 |
| dormant.cdx | 305 | 46a261839c6c2d2dfa454c141b75649236caf88dddf959c018e399331085cb10 |
| active.cdx | 305 | 3c0d5df87b79c36dce543e12a9d9574c3ecf1272a8ff3a1de4df322cbd341d26 |
| h-inconsistent.cdx | 307 | 6bbc950cd9637928e39ad6e817f88b6d9f4b1dc0c0ccd9a9c28390ee788533fd |

At 512 × 512 in Fit view, the 32 px margin gives a scale of 7/8. Each side
extends from pixel 144 to 256 and is 3.5 px wide. The union of these four
rectangles covers 1,776 pixel centers. The regression checks every interior
paper pixel before and after Unicode Save As and reopening. It also checks
all four original H omissions and bonds. External RDKit 2026.09.2 reads the
saved files and returns four carbons, two calculated H on each, molecular
formula C4H8 and SMILES C1CCC1.

```sh
export BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
python3 -B crates/io/tests/fixtures/cdx-skeletal-cycle/generate.py
"$BONDLINE_PYTHON" -B crates/io/tests/fixtures/cdx-skeletal-cycle/check_rdkit.py crates/io/tests/fixtures/cdx-skeletal-cycle/cycle.cdx
cargo test -p bondline-io --test cdx_skeletal_cycle --locked
```

The generator checks the committed bytes by default. Pass `--output-dir DIR`
to create fresh copies. [Integrated verification](../../../../../docs/verification-cdx-skeletal.md).
