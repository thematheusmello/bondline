# Original CDX implicit Element witnesses

The fixtures and their generators are MIT OR Apache-2.0. The inputs
derive from the existing original cyanide and reordered boxed preview
witnesses. The generator reproduces the original binaries and complete
models with Python struct and FontTools 4.61.1 before writing the new
witnesses. [Preview fixture provenance](../cdx-label-preview/SOURCES.md).

The SDK defaults an Element node without the signed Element property
to carbon. The insertion encodes signed 14 or 7 with a short length
immediately before the existing unsigned NumHydrogens property in that
Node. The H-only edit keeps the Element omission. The struct verifier
checks exact object ownership, header, framing and every unaffected
record. [Element](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm),
[NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm),
[Binary framing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm).

The existing run prefix and style remain literal. FontTools computes
the updated body bounds around the unchanged center of the entire
element token. The source stores the rectangle as signed 16.16 top,
left, bottom, and right. The model stores left, top, right, and bottom.
Changed properties use short lengths. [CDXString](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm),
[Coordinates](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm),
[BoundsPen](https://fonttools.readthedocs.io/en/latest/pens/boundsPen.html).

| Case | Before | After | Page / Fragment / Atom | Change |
| --- | --- | --- | --- | --- |
| `cyanide_c_to_si` | `cyanide_implicit.cdx` | `cyanide_si.cdx` | 1 / 1 / 2 | C/H0 → Si/H0, 496 → 503 bytes |
| `boxed_c_h_only` | `boxed_implicit.cdx` | `boxed_h_only.cdx` | 1 / 1 / 1 | CH2 → CH3, 342 → 338 bytes |
| `boxed_c_to_noncarbon` | `boxed_implicit.cdx` | `boxed_nh2.cdx` | 1 / 1 / 1 | CH2 → NH2, 342 → 344 bytes |

Both boxed fragments use Node ID 81 and Text ID 73. The target
belongs to Fragment 901. The full Fragment 902 remains byte exact,
including its extended Text length. The unchanged run size is 24 pt
and the font is the pinned Roboto Regular.

The 490-byte `cyanide_missing_h.cdx` removes the exact H0 record
from `cyanide_implicit.cdx`. The manifest records the data-only
A4 fallback projection, with the literal child body `C` and an
unspecified attached-H count. Preview and edit eligibility require
the explicit attached-H field.

RDKit 2026.09.2 reads all six unique files in binary CDX mode with
sanitization and H removal disabled. The formula observations are
CHN → HNSi, CH2 and Cl → CH3 and Cl, and CH2 and Cl → H2N and Cl.
The single and triple bond directions are unchanged across the
connected case. RDKit reports explicit H0 for the missing-H source.
The struct oracle verifies the field's actual absence. [RDKit binary parser API](https://www.rdkit.org/docs/source/rdkit.Chem.rdmolfiles.html#rdkit.Chem.rdmolfiles.MolsFromCDXML).

Reproduce from the repository root with the pinned FontTools and
external format-oracle environment.

```sh
fixtures=crates/io/tests/fixtures/cdx-implicit-element
out=$(mktemp -d)
python3 -B "$fixtures/generate_implicit.py" --output-dir "$out/vectors"
diff -r -x SOURCES.md -x generate_implicit.py -x check_implicit_rdkit.py "$fixtures" "$out/vectors"
.cache/oracles/format-python/bin/python -B "$fixtures/check_implicit_rdkit.py"
```

Manifest SHA-256 is `13cbe4fbfe3db0141745f4f9cac982780edc22a0261dbfaacbd664d51f7fd0a5`.
