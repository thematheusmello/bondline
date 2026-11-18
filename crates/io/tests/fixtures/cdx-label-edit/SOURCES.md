# Original CDX label edit witnesses

The fixtures and their generators are MIT OR Apache-2.0. The original
inputs are the existing preview witnesses. The generator reproduces
their bytes before applying the edits. [Preview fixture provenance](../cdx-label-preview/SOURCES.md).

The struct oracle walks exact property owners using Page, Fragment, Node
and child Text records. It writes existing signed Element and unsigned
NumHydrogens words, the existing one-run prefix with the new UTF-8 body,
and the signed 16.16 Text rectangle in top-left-bottom-right order.
Changed properties use short lengths. The verifier rereads each file
and checks every record in order, including unchanged long prefixes and
the original header and terminators. [Binary framing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm),
[Element](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm),
[NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm),
[CDXString](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm),
[Coordinates](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm),
[BoundingBox](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BoundingBox.htm).

FontTools 4.61.1 computes outline bounds and pair positions with the
bundled SHA-256-pinned Roboto Regular. The complete element token stays
centered at the unchanged Node anchor. The expected model stores the
quantized rectangle. [BoundsPen](https://fonttools.readthedocs.io/en/latest/pens/boundsPen.html),
[Existing label preview contract](../../../../../docs/research/cdx-atom-label-preview.md).

| Output | Page / Fragment / Atom | Label change | File sizes | Framing |
| --- | --- | --- | --- | --- |
| `water_to_nh.cdx` | 1 / 1 / 2 | O → NH with both existing single bonds | 480 → 481 bytes | Implicit root with file end |
| `boxed_nh2_to_oh.cdx` | 1 / 1 / 1 | NH2 → OH with the second fragment preserved | 348 → 343 bytes | Explicit root with file end |
| `boxed_cl_to_br.cdx` | 1 / 2 / 2 | Cl → Br with explicit zero H preserved | 348 → 344 bytes | Explicit root with file end |

Both boxed atoms use Node source ID 81 and Text source ID 73. The
first edit shortens the Text payload by one byte and its length prefix
by four. The second edit keeps the Text payload length and still
shortens its length prefix by four. The other fragment's Text record
keeps its extended length in each edited file.

RDKit 2026.09.2 opens all six inputs and outputs in binary CDX mode
with sanitization and H removal disabled. It independently verifies
fragment order, literal atomic numbers and attached-H counts, bond
order and direction, and formulas. The formulas change from H2O to
H3N, from H2N and Cl to HO and Cl, and from H2N and Cl to H2N and Br.
[RDKit binary parser API](https://www.rdkit.org/docs/source/rdkit.Chem.rdmolfiles.html#rdkit.Chem.rdmolfiles.MolsFromCDXML).

Reproduce from the repository root with FontTools 4.61.1 and run the
existing format-oracle environment's binary readback.

```sh
fixtures=crates/io/tests/fixtures/cdx-label-edit
out=$(mktemp -d)
python3 -B "$fixtures/generate_edits.py" --output-dir "$out/vectors"
diff -r -x SOURCES.md -x generate_edits.py -x check_rdkit.py "$fixtures" "$out/vectors"
.cache/oracles/format-python/bin/python -B "$fixtures/check_rdkit.py"
```

Manifest SHA-256 is `08251600762359c66abf0011053066a3be6e756bd5e6cade27dd0783e6081824`.
