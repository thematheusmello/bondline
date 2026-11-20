# Ordinary skeletal cycle witnesses

These four original binary fixtures, their standalone struct-only generator, the RDKit
consumer and the Rust optics regression are MIT OR Apache-2.0. They were independently
constructed during integration review on2026-11-20, rather than exported by Bondline or
copied from a reference application. The earlier24-file chain generator is unchanged.
The [public source/admission contract](../../../../../docs/research/cdx-skeletal.md)
records the CDX object/coordinate/flag and carbon/H decisions.

The four-carbon cycle has positions(128,128),(256,128),(256,256),(128,256), single bonds,
512×512pt paper,4pt black width and2pt margin. Source Node IDs19,2,300,77 are deliberately
sparse and out of order; all Element, NodeType and NumHydrogens properties are absent.
There is no font table or Text because all four atoms are genuinely hidden. The binary
palette supplies white2 and black3. The `dormant` twin adds a zero-length true terminal
flag on its first degree-two Node. `active` instead adds the active nonterminal true
flag; `h-inconsistent` assigns that same Node literal H3, exceeding neutral C4 with its
two Single bonds. These controls must not become glyphless cycle previews.

| File | Bytes | SHA256 |
| --- | ---: | --- |
| cycle.cdx | 301 | 2da55160f64fa3e3bfd0aafa186581ce6ee1bfadeb101f5c446227f73a04d563 |
| dormant.cdx | 305 | 46a261839c6c2d2dfa454c141b75649236caf88dddf959c018e399331085cb10 |
| active.cdx | 305 | 3c0d5df87b79c36dce543e12a9d9574c3ecf1272a8ff3a1de4df322cbd341d26 |
| h-inconsistent.cdx | 307 | 6bbc950cd9637928e39ad6e817f88b6d9f4b1dc0c0ccd9a9c28390ee788533fd |

At512×512 Fit,32px margins give scale7/8. The square's axes become144..256 and its
stroke width3.5px. Independent rectangle-union arithmetic gives exactly1,776 black pixel
cells, with all remaining inner-paper cells white. No product renderer computes these
expected rectangles. Both positive sources must produce this exact complete inner-paper
result before and after atomic Unicode Save/reopen, retaining authored HNone and topology.
The external RDKit2026.09.2 consumer separately reads the actual saved CDX and requires
four carbons, zero explicitly attached H, totals2/2/2/2, the literal ring edges, SMILES
C1CCC1 and formulaC4H8. This test does not grant general chemistry inference or editing.

```sh
python3 -B crates/io/tests/fixtures/cdx-skeletal-cycle/generate.py
.cache/oracles/format-python/bin/python -B crates/io/tests/fixtures/cdx-skeletal-cycle/check_rdkit.py crates/io/tests/fixtures/cdx-skeletal-cycle/cycle.cdx
cargo test -p bondline-io --test cdx_skeletal_cycle --locked
```

The generator verifies committed source bytes by default. `--output-dir DIR` generates
separate copies. Runtime test and gate results belong to the feature receiving record;
the fixture declarations and independent review alone do not claim those gates passed.
