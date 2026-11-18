# Original CDX atom label preview witnesses

These 45 original CDX files and their generator are MIT OR Apache-2.0.
The [contract](../../../../../docs/research/cdx-atom-label-preview.md)
records the public SDK evidence for the fields. FontTools 4.61.1 and the
existing independent clearance oracle compute the complete ink boxes
and glyph positions from the pinned bundled Apache Roboto Regular.
The struct reader checks each original record tree. No application pixel
output supplies the expected label geometry.

The manifest declares six independent whole rendered models and 39
controls. The intact baseline CLI at `3ac74a3` supplied the complete
earlier data models for 36 controls and exact decode diagnostics for three.
Two byte-distinct pairs share the expected rendered model. The positive
files cover all five supported outer framings.

```sh
out=$(mktemp -d)
python3 -B crates/io/tests/fixtures/cdx-label-preview/generate.py \
  --output-dir "$out/preview" --baseline-cli /path/to/pre-preview/bondline-cli
diff -r -x SOURCES.md -x generate.py crates/io/tests/fixtures/cdx-label-preview "$out/preview"
```

Manifest SHA-256 is `5158a2c049fac8a9d01b74aa05f69a06b9f668e8437bc514f745dcda4c9a9492`.
The qualification changes only the existing JSON field spelling
`chain_angle_degrees` in the expected cyanide model. Every original
CDX file, numerical value, and other declaration reproduces exactly.
