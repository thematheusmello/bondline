# New labels on hidden CDX atoms

These original MIT OR Apache-2.0 witnesses start with the [skeletal label drawings](../cdx-mixed-skeleton/SOURCES.md). An independent Python struct reader locates one unlabelled Node by raw ID. FontTools reads the pinned Roboto outlines at its original point. The writer inserts missing Element and NumHydrogens properties and replaces existing numeric payloads in place. A new direct child Text carries the specified run, signed 16.16 box and the two explicit Left justifications. Its ID is one above the largest raw object ID in the complete input. All other original bytes remain exact. These are Bondline writing conventions for these controlled cases. [Node](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm) · [Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm) · [NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm) · [BoundingBox](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BoundingBox.htm).

| Source and raw atom | New printed label | Bytes | RDKit result |
| --- | --- | ---: | --- |
| Ethanol, Node 61 | NH₂ | 439 | NCO, CH₅NO |
| Amino alcohol, Node 97 | O | 622 | NCCOO, C₂H₇NO₂ |
| Methanol with an existing H count, Node 73 | NH₂, 14 pt in the source’s blue | 379 | NO, H₃NO |
| [All-carbon propane](../cdx-skeletal/SOURCES.md), Node 7 | O, replaces the existing element and H count | 338 | COC, C₂H₆O |

Run the generator with FontTools 4.61.1 and compare its output to this directory. The witness check reads each CDX using RDKit 2026.09.2, opens the same file in Bondline, reads its formula, and captures its paper. The edit workflow additionally compares bytes through creation, Undo, Redo and Unicode Save/reopen.

```sh
python3 -B xtask/oracles/cdx_skeletal_creation.py --generate /tmp/bondline-cdx-new-labels
"$BONDLINE_PYTHON" -B xtask/oracles/cdx_skeletal_creation.py --verify target/debug/bondline-cli
```
