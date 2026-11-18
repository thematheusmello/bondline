# Original CDX literal atom metadata vectors

Original Bondline test data, MIT OR Apache-2.0, frozen on base `42ccefb` against
[the locked literal-atom contract](../../../../../docs/research/cdx-literal-atom-metadata.md).
Python `struct` constructs every byte. No Bondline writer/parser output, reference
executable, export, sample, artwork or downloaded binary supplies expected data.

The public SDK [IntroCDX](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm)
defines little-endian tags, object IDs, short/extended property lengths and ownership.
[Isotope](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_Isotope.htm)
is signed16 with natural-abundance0/absence;
[Charge](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_Charge.htm)
is signed8 and explicitly documents its property-specific savedINT32 arithmetic shift24
for four-byte payloads;
[Radical](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_Radical.htm)
is unsigned8 with distinct0/1/2/3 None/Singlet/Doublet/Triplet enums;
[NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm)
is unsigned16 inline hydrogen count, separate from drawn H nodes. Their owner is Node.
[NodeType](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Type.htm)
explicitly distinguishes0 Unspecified from1 Element and defaults omission to Element;
[Element](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm)
defaults omission to carbon. No label/default chemistry is inferred from opaque bytes.

Accepted explicit28-byte, header-implied and22-byte embedded-root/file-end envelopes
reuse the existing original format coverage. The embedded-root convention's prior
public/open-source evidence remains in `docs/research/cdx-file-end-plan.md` and
`xtask/oracles/cdx_header_oracle.py`; this fixture does not promote it to the literal SDK
header definition. Extended length markers around small payloads deliberately exercise
the already accepted tolerant framing; payload widths remain independently2/(1 or4)/1/2.
The SDK normally describes extended lengths for large fields. Other wider numeric
encodings are outside this bounded projection, not universally malformed CDX.

`literal-vectors.json` pins all18 complete files and SHA256 hashes. The original scene
uses Page90/Fragment51/Node73+149/Bond900, distinct from app page1/fragment1/atoms1+2/bond1.
Positions are independently encoded signed16.16 points with y before x. Original opaque
user properties occupy distinct root/page/fragment/atom/bond/custom owners, retain null/
high bytes, and never become chemistry. Original custom objects use IDs400000001/2.
`literal-facts.json` records complete ordered framing/owners/offsets/payload hex and each
known property's presence, width, length encoding, signed value and model projection.
Charge4 records also expose the unshifted signed saved integer. Bonds begin/end are app
IDs; source references73/149 remain in exact raw records.

Cases cover absent fields, all present zeroes, an equal-model absence-versus-scalar0
pair with H still absent, three envelopes, small extended lengths, property reversal,
13C methylammonium,13CH3 doublet, natural CH2 singlet/triplet, literal scalar extrema,
absent NodeType/Element carbon and a drawn2H+ with present inlineH0. Omitted H stays
unresolved None, whereas present0 stays Some(0). Nonexistent isotope/valence extrema are
literal-domain tests, not physical nuclide or chemical-validity claims.

All five Charge4 collisions are frozen before production:

| Little-endian payload | SignedINT32>>24 expected | RDKit observed charge |
|---|---:|---:|
|01000000|0|1|
|00000001|1|1|
|000000ff|-1|-1|
|aabbcc7f|127|-86|
|00000080|-128|-128|

The expected algorithm comes from the property-specific SDK instruction, never RDKit's
output. The two differences are explicit external-consumer limitations; no producer or
version-string exception is inferred. Low bytes remain original unchanged source data.

`check_literal.py` independently decodes every source and qualifies36 reader refusal
controls for widths, duplicates, negative isotope, radical enums, wrong owners and
truncation. It also rejects explicit NodeType0. The first pre-freeze draft mistakenly
encoded that unspecified type. Primary enum review corrected the original generator and
reader to explicitElement1, retained the deliberate omitted-default case, regenerated
all source/fact hashes and reran all external checks. No old negative or producer guard
was changed, and those discarded draft pins were never committed as frozen references.

The actual installed external RDKit2026.09.2 reports binary CDX support. Eight selected
original cases agree on explicit element/isotope/charge/inline-H data, including chemically
valid13C methylammonium in three envelopes, methyl doublet, default carbon and deuteron.
Singlet and triplet both expose two radical electrons externally; this cannot establish
their distinct coupling states. Their exact distinction is supported by the independently
decoded enum and public specification only. Five additional unsanitized Charge4 consumer
observations retain the differences above. `rdkit-observed.json` freezes actual results;
`independent-checks.json` records the qualifications. No chemistry result is claimed for
absent-H inference or the deliberately extreme literal metadata.

`old49.json` copies the previously frozen actual pre-producer CLI stdout from
`label-metadata/old49.json`:47135B, SHA256
`3cfd51a6e72891137cf0cc56fd756ff33652cfc685e5aaf877634b259cc41e4f`.
No command record/schema/order is added or changed. The helper is
`xtask/oracles/cdx_literal_support.rs`, exposing source/facts and a standard-library
actual-file reader. That reader reports exact fixture framing/data; it does not use
RDKit to determine expected values or claim to be a general CDX reader.

Reproduce without a product build or network:

```sh
python3 crates/io/tests/fixtures/cdx-literal-metadata/generate_literal.py
.cache/oracles/format-python/bin/python crates/io/tests/fixtures/cdx-literal-metadata/check_literal.py
```

The first command uses standard Python and writes ignored `target/cdx-literal/reproduced/`.
The second requires the existing pinned RDKit interpreter, reads frozen inputs and writes
ignored `target/cdx-literal/reproduced-checks/`. `--input-dir` and `--output-dir` permit
explicit route changes; defaults never overwrite references. Paths are the only portable
script adaptation. All18 sources/manifest, complete facts, external observations and
checks reproduce byte-for-byte. Shipped dependencies/assets/fonts/old fixtures/goldens/
Actions remain unchanged. Read-only projection is not display, edit or full CDX parity.
