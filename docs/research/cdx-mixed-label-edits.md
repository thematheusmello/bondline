# Editing an existing atom label beside Page captions

Reviewed 2026-11-20. A Node owns its printed child Text. Each direct Page Text and each child Text have separately scoped identities. The label target is the specified fragment and atom, even when both Nodes repeat a raw ID and three Text records share another. Bondline resolves the exact Node, its direct child Text and their owned properties from the current certified bytes. [Node](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm), [binary containment and object identity](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm).

The [existing label writer](cdx-atom-label-edit.md) synchronizes Element, attached H, one run body and its rectangle. It handles the published missing-Element carbon default according to the [insertion contract](cdx-implicit-element.md). Preserve the Node position, exact run style words, other atoms, bonds, Page captions, marker IDs, encoding choices and Z properties. Bounds use signed 16.16 coordinates in top, left, bottom, right order. Depth uses signed INT16. Replacing the label consumes no ID or depth. [Element](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm), [attached hydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm), [coordinates and rectangle layout](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm), [ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm).

Bondline requires the re-imported page to retain its original chemical-only or mixed display profile and equal the complete expected model. All on-paper ink and connected bond clearance must still qualify. A newly added Page caption belongs to current bytes even when the original source opened as chemical-only. One accepted edit enters history. Detached snapshots and changed bounds outside the page do not pass. The independently constructed edge case puts Cl inside the page at x = 12 pt. Changing it to Mg would move the left edge from 2.888671875 to −3.404296875 pt. [Original fixtures and replay](../../crates/io/tests/fixtures/cdx-mixed-label-edit/SOURCES.md).


Bondline uses exact Alt-primary within the existing atom hit tolerance to
open the atom label editor. This also works with Text selected after annotating a drawing.
An overlying painted caption cell prevents a click from selecting an atom
behind it. An ordinary Text click on the same cell keeps the existing
caption workflow. These pointer rules are Bondline interaction decisions.
The public Z property establishes the visual stacking order.
[Paint order](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm),
[caption picking](native-caption-editing.md).
