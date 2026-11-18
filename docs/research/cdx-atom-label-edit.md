# Editing an existing boxed CDX atom label

Reviewed 2026-11-18 after the [display contract](cdx-atom-label-preview.md).
Keep the certified Page, every fragment and directed bond, and the target's
Node point. Resolve the typed atom in its fragment and use its Node record
and direct child Text record. CDX object IDs are scoped to their containing
objects, so the writer addresses exact record handles. [Binary framing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm).

Require explicit signed INT16 Element and unsigned UINT16 NumHydrogens on
that Node. The display reader can resolve the published missing-Element
carbon convention. Adding an omitted property is a separate write operation.
The edit replaces the literal atomic number and attached H count. [Element](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm),
[NumHydrogens](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm).

Keep the direct Text's one start-zero run and copy its two-byte count and
ten-byte style record verbatim. Replace the UTF-8 body with the shared
element/H spelling. Compute the curve and pair positioning bounds around
the unchanged element-token center using the existing licensed font.
Round each bound to nearest signed 16.16 point with ties to even. Encode
top, left, bottom, right and retain those quantized values in the model.
[CDXString](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm),
[Coordinates](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm),
[BoundingBox](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BoundingBox.htm).

Use one checked binary batch for the four fields. Re-import the complete
candidate, require the same closed display profile and full expected model,
and verify the whole paper and bond clearances through that profile. Keep
original input bytes and the exact current certified bytes independently.
History and captured Save plans own both through the normal immutable
document snapshot. Save and display require equality to the current
certified model. A numeric no-op requires the same field ownership proof.
An edit back to the original model can still carry changed length words.

The shared `atom.set_authored_label` command and guarded native form carry
the new workflow. Existing label creation, general drawing and selection,
opaque sources, caption editing and lossy cross-format writes retain their
previous authority boundaries. Additional field encodings and effective
style inheritance remain open. The [independent edit witnesses](../../crates/io/tests/fixtures/cdx-label-edit/SOURCES.md)
pin before/after bytes, the complete model, exact placement and chemistry.
