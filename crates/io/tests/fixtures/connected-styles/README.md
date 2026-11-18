# Original connected atom-label vectors

`vectors.json` is original Bondline test data, licensed MIT OR Apache-2.0. The CDXML
strings are generated from new H–O–H, O=C=O and H–C≡N inputs; no vendor sample, template,
font, export or reference program was used. Sparse scoped source IDs intentionally differ
from model IDs. The NH middle-label vector independently specifies the ammonia edit.

`generate.py` is original MIT OR Apache-2.0 oracle tooling. It uses external FontTools4.61.1
BoundsPen/hmtx and the bundled Roboto Regular DFLT GPOS kerning, without importing any
Bondline parser, model or renderer. Roboto's existing Apache-2.0 attribution remains in
`assets/fonts/`; its SHA256 is
`56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d`.
The element-token ink-centre agreement is the declared Bondline admission policy, not a
universal reference alignment claim. Digits/letters use literal face0.

Reproduce from the workspace root with FontTools4.61.1 available:

```sh
python3 crates/io/tests/fixtures/connected-styles/generate.py
```

The Rust tests independently consume actual input/output CDXML using Python Expat,
Decimal, and direct RDKit2026.09.2 CDXML decoding. Those external chemistry results do not
certify glyph clipping, pixel fidelity, valence validity in Bondline, or native acceptance.
The connected renderer and its physical outline-margin oracles are verified separately.
