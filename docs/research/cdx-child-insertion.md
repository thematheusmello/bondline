# Appending a child object to CDX

Reviewed 2026-11-19. The public binary specification gives each object
a little-endian two-byte tag with bit 15 set, a four-byte ID, its
property and child records, and a two-byte zero terminator. The file
end has the same zero bytes. The parsed record and its matched owner
distinguish the two meanings. Properties have a nonzero tag with bit
15 clear and use a short length through 65,534 bytes or a four-byte
length after the 65,535 sentinel. [CDX binary format](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm).

Nonzero object IDs identify objects within their containing object.
The SDK recommends global uniqueness and allows zero for an unassigned,
unreferenced object. Document and application identities are separate.
[Object identifiers](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXObjectID.htm).

## Standalone splice

Append one property-only child directly before the selected owner's
physical end record. A handle must belong to the exact parsed Document
and be an ObjectEnd with the index of its matched opening. The end
record's parent points to the opening's parent. A property handle,
an implicit root end and a separate file end have no eligible owner.
Bondline copies the entire original prefix and suffix, including
opaque records, reserved header bytes and every existing terminator.
The construction accepts the five existing decoded framing variants
where the owner has a physical matched end.

The caller supplies the child tag, raw ID and property payloads. The
new child keeps property order and uses the existing canonical length
encoder. A nonzero raw ID must be unused by immediate object children
of that owner across all tags. Separate scopes may reuse it. The
caller may supply zero for an unassigned object. The existing
64 MiB, 100,000 record and 256 open object limits include the new
object start, properties and end. Reject overflows before reserving
the output buffer.

The independent witness uses Page 55 with opaque extended properties,
an existing nested ID 74 and a later sibling Page. Appending an
18-byte child at the matched Page end allows ID 74 in the direct
scope. Removing the appended interval restores the original bytes.
Reusing direct sibling ID 73 would collide. The same leaf after the
Page end instead belongs to the root. Matching both syntax and exact
ownership is necessary.

## Direct Page Text follow-up

The Page specification permits direct Text children. A later adapter
must encode the explicit resolved style and baseline in the existing
CDX format, choose a noncolliding raw ID and explicit signed depth,
and certify complete source re-import before committing. The current
caption profile maps sorted raw IDs to dense live IDs. Persisting
gaps left by Undo needs an additional identity proof. Resource
inheritance, insertion into blank or mixed CDX pages and exhausted
identity or depth remain separate decisions. This splice supplies
binary syntax and ownership across existing source framing.
[Page and child types](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Page.htm),
[Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm),
[CDXString](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm),
[ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm).
