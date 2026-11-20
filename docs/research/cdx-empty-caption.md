# First caption on an empty CDX Poster page

Reviewed 2026-11-19. A CDX Page can be empty and can own a Text child.
Use the same closed, single-page layout as the existing caption writer.
An empty caption-ready page keeps its explicit Poster extents and its
paper and font resources. This profile covers the five already admitted
binary framings. [Page object](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Page.htm),
[DrawingSpace](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/DrawingSpaceType.htm),
[width](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Width.htm),
[height](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Height.htm).

Require the same root font entry used by the caption writer. The table
declares platform 1, UTF-8 code page 65001, and Roboto with an agreeing
font ID and ordinal. Both absent default white and an explicit white paper
entry use the existing palette check. These are Bondline's supported
rendering and writing rules. [CDXFontTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXFontTable.htm),
[CDXColorTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXColorTable.htm),
[Microsoft code pages](https://learn.microsoft.com/en-us/windows/win32/intl/code-page-identifiers).

The first authored Text has explicit depth 0, the effective depth the
specification gives to an omitted Z field. That explicit value is a
Bondline encoding decision. Subsequent appends still allocate above the
highest existing effective signed depth. Preserve the Page and root
boundaries, property order, raw resource records and framing through the
existing owned child splice. [ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm),
[creation and live IDs](cdx-caption-creation.md).

Open displays the empty page at its declared physical size. Clicking
blank paper with Text uses the same draft and command. The first live ID
is 1. Undo restores exact empty bytes, and a new caption on that history
branch takes live ID 2. Save/reopen preserves the existing sparse-ID
contract. Unknown children and resources, multiple pages, unresolved
paper and mixed object classes need separate admission evidence.
