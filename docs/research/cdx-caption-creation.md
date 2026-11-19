# Creating a direct CDX Page caption

Reviewed 2026-11-19. The first receiving profile is a fully certified
caption-only Poster Page whose existing resources resolve the plain
Roboto Regular preset. Use the current Text control and shared
`text.create` request. The exact CDXML creation rule and the established
CDX single-run replacement workflow stay available.

## Caption fields

The public Page and Text definitions permit a direct Text child.
Write the left caption justification and the required styled Text
payload. Encode the baseline as signed fixed-point y then x in units of
1/65,536 pt. Round the request once to the nearest representable point,
with ties away from zero. Validate the resulting point and glyphs
before preparing a commit.
Use the already declared font with a checked ID and ordinal, face 0,
240 twentieths of a point and reserved black index 0. The first
profile requires an unambiguous UTF-8 font declaration and valid
UTF-8 text.
[Page](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Page.htm),
[Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm),
[coordinates](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm),
[position](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/2DPosition.htm),
[caption justification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/CaptionJustification.htm),
[styled string](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm),
[color table](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/ColorTable.htm).

Allocate a new positive raw Text ID above the existing object IDs and
an explicit signed Z value above the existing effective depths. The
already qualified source determines the order, including its single
permitted omitted depth. Exhausted IDs or depths refuse atomically.
[Binary IDs](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm),
[ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm).

## Persisting the live caption ID

Use one standard ObjectTag child (0x8011) on each newly created Text.
The namespaced Name (0x0008) is raw ASCII `org.bondline.caption-id`.
The Name property specifies unformatted bytes for ObjectTag, while
the object table labels the general type CDXString. TagType (0x0D00)
is INT16 value 3, and Value (0x0D05) is the raw ASCII `1:<id>` with a
canonical positive decimal u32. The version and name are Bondline
policies. The child has unassigned ID 0 because no object references it.
Persistent (0x0D04) is explicitly one true byte. Its documented
implicit default also permits the same persistent tag without that
property. Use no tracking, child objects or other properties.
[ObjectTag](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/ObjectTag.htm),
[Name](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Name.htm),
[tag type](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ObjectTag_Type.htm),
[value](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ObjectTag_Value.htm),
[persistence](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ObjectTag_Persistent.htm),
[Boolean width](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXBoolean.htm).

Unmarked captions keep their existing raw-ID-ordered dense IDs within
the unmarked set. The optional tag preserves the newly assigned live ID when history leaves
a gap. Use a shared resolved ID map for projection and run-edit lookup.
Reject duplicate IDs, wrong owners, ambiguous tags, malformed values
and unknown versions in the caption admission proof while retaining
the complete file. The identity tag does not determine display, style
or chemical authority.

## Owned append and acceptance

Append Text immediately before the matched Page end. Reparse those
bytes to obtain the new Text end and append the ObjectTag there. Validate
the whole result after both splices and require exact re-import of the
expected scene and live IDs before updating history or reserving an ID.
All original resources and records outside the insertion stay byte-exact.
Re-use the current CDX source edit and file receipt paths.

The [independent fixtures](../../crates/io/tests/fixtures/cdx-caption-create/SOURCES.md)
share the original 156-byte caption-only CDX. They add explicit raw Text
ID 91, depth 1 and the caption body at (144.25, 216.75) pt. The emitted tag
distinguishes live ID 2 from a burned-gap ID 3. The tag's own physical
owner, version, exact payload and persistence are independently checked.
Follow through typed create, Undo/Redo, Unicode Save and reopen, and new
and old run edits before changing native availability.

The SDK describes resaving persistent ObjectTags. Actual external
resave and preservation of parent IDs, fonts, offsets and the complete
caption profile remain open. Entirely undone historical ID reservations
would need a separate document watermark to outlive reopening. Blank
or mixed CDX pages and inherited styles require additional source proofs.
