# Read-only CDX atom label preview

Reviewed 2026-11-18 from `3ac74a3`. Extend the [owned child Text reader](cdx-node-labels.md)
to preview whole source-qualified pages with isolated atoms and plain
connected structures. Byte-exact unchanged Save remains the write path.

## Page and object coverage

Reuse the [caption page rules](cdx-caption-preview.md) for one direct Page
with explicit Poster drawing space and positive signed 16.16 width and
height. The coordinate origin is the top left. The original framings with
reserved header bytes zero share this interpretation. Use raw palette index
2 only when it resolves to white. [Drawing space](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/DrawingSpaceType.htm),
[width](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Width.htm),
[height](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Height.htm),
[coordinates](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm),
[background](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BackgroundColor.htm).

The closed record tree contains direct Fragments, each with only Element
Nodes and supported Bonds. Every atom has exactly one direct owned label.
The immutable elemental reader certifies scoped identities, literal node
fields, resource tables and exact bond references. The display scan admits
only the known fields on their actual owners. Extra Page captions, groups,
local styles, visible-state overrides, opaque extensions and unproved
Z-order metadata leave the original data-only or refusal path intact.
The earlier pure-caption profile keeps its precedence.

## Label placement and agreement

Use the already qualified [boxed label geometry](boxed-atom-label-workflow.md).
Node.p carries the complete element token's center. With no child point,
the child's explicit enclosing rectangle supplies a placement constraint.
Use the exact Roboto Regular face-zero outlines, advances and pair
positioning. Require explicit Left Justification and Left LabelJustification,
one run, and absence of LabelAlignment. Match the stored ink edges and
the element-token center at the 16.16 coordinate tolerance already used
by the renderer. The stored box supplies the actual painted position.
This checks a constrained geometry and supplies no universal baseline
offset, font inheritance or automatic alignment. [Node](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm),
[2D position](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/2DPosition.htm),
[bounding box](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BoundingBox.htm).

The body must equal the literal element symbol and its explicitly authored
H count using the existing single-line label spelling. The font face governs
the literal digits. Neutral, unnumbered nonradical atoms qualify. Conflict,
missing H and unsupported typography remain importable data where the
earlier reader allows them. They do not supply chemical inference or layout.

## Explicit connected styles

For bonds, reuse the existing [connected-label geometry and style checks](connected-label-styles.md).
The root LineWidth (0x0807) and MarginWidth (0x0808) are signed 16.16
points. Missing widths or margins may refer to unmodelled style sheets.
Allow a valid root BondLength (0x0805) as points and BondSpacing (0x0804)
as a signed INT16 in tenths of a percent. Double and Triple require both
to resolve the rails. The default bond foreground must resolve to exact
black at palette index 3. Reject explicit bond colours, local overrides,
absolute spacing and unknown paper fields in this proof. Explicit valid
ChainAngle may retain its already supported authored value. [LineWidth](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LineWidth.htm),
[MarginWidth](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/MarginWidth.htm),
[BondLength](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondLength.htm),
[BondSpacing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacing.htm),
[foreground](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ForegroundColor.htm).

Use plain Single, centered Double and Triple. The two explicitly recorded
Center values 0 and 256 depict the same rails with distinct original
automatic/manual bytes. Absence leaves placement unresolved. Shared
noncrossing geometry and outline clearance checks precede admission. This
uses Bondline's documented renderer for explicit styles. Reference-specific
clearance fits and additional bond styles remain open. [Double position](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_DoublePosition.htm).

## Authority and acceptance

Certify the actual whole page at 100% before clipping. The normal scene
continues to validate each camera and DPI. Bind display permission to the
immutable CDX source and exact current model. A detached edit revokes that
permission and cannot be saved as the original. Existing commands and
native controls provide view and capture. Chemistry, atom forms, drawing,
conversions, source-edit commands and legacy CDX diagnostics stay guarded.

Pin original binaries, prior Open results, complete projected models,
and independent FontTools outlines and projection vectors before code.
Verify CPU/Metal, pixel placement, atom/bond picking, actual CLI, MCP and
loopback, and native macOS Open/View/Unicode Save/reopen. Include earlier
caption and labelled-data negatives, unavailable placement/styles and a
cropped view that must not hide bad whole-page geometry. Run local gates
and ASan against copied corpora. Independent pixels carry no human golden
approval.
