# Editing CDX atom labels with an implicit Element

Reviewed 2026-11-18 after the [existing-label contract](cdx-atom-label-edit.md).
For an Element Node with an omitted Element property, the published atomic
number is carbon. An explicitly present value remains a signed INT16.
The editor keeps omission through a C/H no-op and through a change to the
attached H count on carbon. Changing the element inserts its signed
value. A subsequent edit of that explicit record uses ordinary replacement.
Undo restores the entire earlier record sequence, including an omission.
[Element](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm).

The existing NumHydrogens record anchors insertion. Its published absent
case depends on valence, so an explicit H count is still required here.
The view already requires agreement with the explicit child Text and
its placed box. The edit changes the existing H, styled text and bounds
in the same batch and reuses the whole-paper re-import proof. [NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm),
[CDXString](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm).

The binary format makes each property a tagged record owned by its
containing object. Bondline inserts immediately before an existing
property handle. It checks the exact handle in the current parse and
absence of the inserted tag in its owner. Multiple insertions at one
anchor use ascending tag order. An insertion can share the anchor with a
replacement. Existing records keep their relative order and unchanged
bytes. One combined calculation bounds total output bytes and record
count before emission. [Binary framing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm).

The original witnesses cover connected carbon to silicon, isolated
carbon with an H-only edit, insertion beside an unchanged peer with
repeated local IDs, and a retained missing-H source. Full authoring,
label creation, unresolved styles and cross-format loss constraints
retain their existing gates.
