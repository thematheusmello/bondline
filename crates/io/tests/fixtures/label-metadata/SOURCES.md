# Explicit run labels beside preserved scalar defaults

Original Bondline test data, MIT OR Apache-2.0, frozen on base `0aa97cd` against
[the locked metadata-only contract](../../../../../docs/research/label-default-metadata.md)
`8c2b608`. No reference executable, asset, export, icon or proprietary sample was used.
The graph/edit/creation/movement sources reuse our previously original independently
frozen `mixed-labels`, `created-labels` and single-node boxed-label vectors. New expected
bytes are literal metadata overlays and Expat/Decimal splices, never product output.

The primary [Style](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Style.htm),
[Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm),
[Node](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm),
[LabelFont](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelStyleFont.htm),
[LabelSize](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelStyleSize.htm),
[LabelFace](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelStyleFace.htm)
and public DTD distinguish explicit StyleRun attributes from scalar default metadata.
Document, Node, Bond and Text are legal owners. The contract's original bounded owner
scope is root/n/b/t, including supported Page caption t. Each property selects its nearest
containing owner separately; values remain decoded and lexically preserved source data.
The fresh research manifest/report are pinned in the contract. Omitted-field examples do
not establish arbitrary omitted-s-to-Label*/Caption* substitution. These vectors supply
no such inference, stylesheet resolution, baseline policy or exact26.1 pixel claim.

All twelve files retain all four actual s fields: font, size, face and color. Their actual
labels remain Roboto Regular12pt face0, despite conflicting unused defaults10/14/16pt.
The unused font7 deliberately names `Unused serif`; every admitted scalar font is a
nonnegative signed INT16 resolving an existing resource, of any unused family. Face is
nonnegative signed INT16 metadata, including96 and32767; no face interpretation/rendering
is inferred. XML size retains the standalone reader's positive-finite policy: unused
Bond LabelSize1e308 is deliberate and is not a glyph-work allocation. Scalar-1 is not an
inherit sentinel; actual UINT16 run/font65535 is a separate valid domain. The original
legacy CDX composite sentinel does not change these scalar checks.

`metadata-vectors.json` pins twelve whole source byte lengths and SHA256 hashes.
`metadata-facts.json` records independent graph/chemical IDs, exact flags, literal H,
body/points/boxes/explicit styles/resources/captions/comments plus every scalar owner,
full containing path, decoded value, raw entity/quote/numeric spelling and byte spans.
`mixed_a`/`mixed_b` differ only in immutable unused root metadata and have equal modeled
contents and pixels. Root's source ownership must still distinguish them. Other cases
cover48 endpoint edit,49 partial/all printing, movement/compound edits, all four owner
classes including a Page caption, unsigned actual font65535, a single scalar omission
case and a no-bond boxed source. Source IDs73/97/109/137/149 remain distinct from app1..5.
Object IDs are not font IDs; no table is inferred or changed during expected edits.

The independent reader uses Expat/Decimal and both directly imported unsanitized CDXML
and independently constructed no-implicit-H graphs in RDKit2026.09.2. All twelve actual
files agree on literal graph/formula/SMILES. Original C3H9NO and edited C3H10N2 describe
literal fields, not a general chemical-validity claim. Every no-Text node is independently
checked against current original hidden-C4/C2/angle restrictions; drawn child Text takes
precedence over dormant retained flags. Metadata never supplies actual run fields.

The independent font oracle uses only the existing pinned Roboto font, SHA256
`56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d`, FontTools4.61.1
and unchanged `xtask/oracles/clearance_oracle.py`. All31 actual label boxes match its
outline placement to1e-12pt. Four missing actual run selectors, signed/unsigned confusion,
unresolved metadata font, shadowed invalid value, negative face, unsupported owners/
Caption default/namespace and omitted actual caption size yield14 strict-reader refusals.
The independently calculated wrong16pt box for actual12pt OH at(96,160) is
`[91.42578125,154.15625,111.56640625,165.84375]`; actual typography stays12pt, so this
cannot become a valid box through an unused LabelSize16. Current hidden-C angle drift
also remains a refusal, even when all scalar metadata is legal.

`old49.json` is the actual pre-producer `target/debug/bondline-cli commands` stdout,
47135B, SHA256 `3cfd51a6e72891137cf0cc56fd756ff33652cfc685e5aaf877634b259cc41e4f`.
The archival capture separately asserted all49 records and the exact old48 raw prefix;
no current command output is used to generate expected XML or chemistry/layout facts.
Reproduction copies and verifies this immutable receipt instead of recapturing product
output. The existing old48 fixture is unchanged.

Reproduce with existing external test tools; no install or shipped dependency is added:

```sh
python3 crates/io/tests/fixtures/label-metadata/generate_metadata.py
python3 crates/io/tests/fixtures/label-metadata/check_metadata.py
```

The generator writes ignored `target/effective-labels/reproduced/`; it requires standard
Python and prior tracked original fixtures. The checker requires FontTools4.61.1 plus
`BONDLINE_PYTHON` (default `.cache/oracles/format-python/bin/python`) with RDKit2026.09.2.
It reads frozen inputs and writes ignored `target/effective-labels/reproduced-checks/`.
`--input-dir`, `--output-dir`, and `--python` permit route overrides. Defaults never rewrite
frozen references. Path-only portable adaptation reproduces the same sources, manifest,
facts and checks. Shared receiving tests use `xtask/oracles/label_metadata_support.rs`.
Actual implementation/UI/adapter/native verification belongs to the receiving workflow
records. This source evidence does not enable Actions, approve a golden or finish M0–M5.
