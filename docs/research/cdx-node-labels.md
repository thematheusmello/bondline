# Owned CDX Node labels

Reviewed 2026-11-18. Extend the read-only elemental CDX view to retain authored
Node child Text in the existing `AtomLabel` model. The original CDX is the
source of record for unchanged Save.

## Ownership and numeric data

The [Node](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm) and
[Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm) descriptions
distinguish a contained label from the numeric fields a writer can also supply.
Retain both independently, even if the body and numeric fields disagree.
Keep the existing Element node restriction, numeric
[Element](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm)
default, literal H presence, isotope, charge, radical and bond graph.
An absent [attached H count](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm)
stays unspecified. Do not infer chemical properties from the child body.

Admit one direct Text per supported Node, including an unassigned source
Text ID. Source IDs remain scoped under their respective owner. No Page
caption ID is assigned to a Node label. Incomplete child resources or
unadmitted child fields leave the earlier elemental refusal intact. The
existing pure caption preview, other graph layouts, old errors and
opaque source preservation keep their scope.

## Literal fields and styles

Retain the optional Text position and box separately from Node position.
Read signed 16.16 points in binary y/x order and the optional rectangle in
top/left/bottom/right order. The model uses left/top/right/bottom. Check each
payload width and model bounds. Position remains literal until layout is
proved. [Coordinates](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm),
[BoundingBox](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BoundingBox.htm).

Read [Justification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Justification.htm)
0x0701 and [LabelJustification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelJustification.htm)
0x0823 as separate signed-byte values from −1 through 6.
[LabelAlignment](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelAlignment.htm)
0x0705 has a separate 0 through 6 mapping. Preserve exact absence and keep
stored conflicting values. A root LabelJustification stays in raw source.
Child ZOrder can remain in source after the existing width check. Unknown
user properties keep their earlier opaque scope.

Reuse the explicit [CDXString run proof](cdx-caption-runs.md) for style
offsets and UTF-8. Require checked root tables, Windows platform 1, charset
65001 and a coincident declared font ID and ordinal for each resolved run.
Printable ASCII font names, literal face words, numeric sizes and resolved
RGB values can be retained as data without a matching bundled font. Keep
an explicit empty body with one start-zero style. Zero-run strings, other
code pages and unproved defaults remain open. Check model run budgets before
allocation. Glyph joins and the caption raster checks still apply to the
caption workflow. Child typography projection grants no display authority.

## Acceptance

Freeze original binary, complete model, owner and field witnesses before
production. Check differing stored labels and numeric fields, separate
positions and signed boxes, all enum values, explicit empty text, scoped
repeated Text IDs, malformed resources and peers, original byte identity,
and old format/authoring guards. Verify native macOS Open, known Save,
Unicode Save As and reopen. Keep CDX label layout, chemical editing and
cross-format conversion outside this data authority.
