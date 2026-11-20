# Printed atom edits on skeletal CDX paper

These four original, MIT OR Apache-2.0 files are expected results for edits to
the [skeletal molecules](../cdx-mixed-skeleton/SOURCES.md) and their
[moved and repeated fragments](../cdx-mixed-skeletal-move/SOURCES.md).
They were constructed with Python struct and FontTools independently of Bondline.

| File | Target and expected chemical change |
| --- | --- |
| ethanol.cdx | The oxygen in ethanol becomes NH₂, giving ethylamine. |
| methanol.cdx | The oxygen in methanol becomes Cl, giving chloromethane. |
| printed.cdx | The previously moved oxygen becomes NH₂ at its current position. |
| twins.cdx | Only the second fragment's oxygen becomes NH₂. The first remains methanol. |

The generator resolves the raw Node within its Fragment, then its own Text.
It copies the original style-run count and five run words, constructs the glyph
box using the bundled Roboto font, and writes the four affected fields with
their existing length-header widths. The twins input deliberately repeats raw
IDs across Fragments. The test compares the complete saved files and reads
their SMILES and formulas with RDKit. The runtime test uses the checked-in files.

Reproduce in an empty output directory, using the FontTools version in
[the clearance evidence](../mixed-labels/SOURCES.md).

```sh
python3 -B xtask/oracles/cdx_skeletal_label.py --generate /tmp/bondline-label-witnesses
```
