# Native caption creation

Reviewed 2026-11-19. The pinned G25 guide is 17,721,447 bytes,
439 PDF pages, SHA-256
`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
Its unversioned cover and May 22, 2025 creation metadata accompany the
version 25 filename. Printed page p is PDF page p + 1.
[Public G25 guide](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf).

## Caption creation in public documentation

G25 printed p48 describes clicking in the document with the Text
tool to open a text field. The caption example on p92 clicks an
empty spot below a drawing, enters the caption and closes by
clicking outside. Printed p423 demonstrates closing a newly entered
caption with Escape or by changing tools. Printed p38 gives Enter
or Return as the default newline and describes a preference that
requires Alt+Enter for that newline.
[Text field](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=49),
[Caption example](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=93),
[Tutorial](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=424),
[Newlines and closing](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=39).

G25 printed pp48 and 92 describe document text settings for new
captions. The official November 2025 support articles document
choosing the caption font in Document Settings and using a style
sheet for new-document defaults. The September 2026 Desktop 26.1
release announcement reports attached-data selection and clipboard
changes. Exact 26.1 platform gestures, confirmation behavior,
font defaults and the offset from a click to the stored baseline
remain open. [Caption font](https://support.revvitysignals.com/hc/en-us/articles/6796010539284-ChemDraw-How-can-I-Change-the-Text-Font-in-ChemDraw),
[Document and object settings](https://support.revvitysignals.com/hc/en-us/articles/4408239581204-ChemDraw-Document-and-Object-Settings-of-a-ChemDraw-Document),
[New-document defaults](https://support.revvitysignals.com/hc/en-us/articles/6952207034260-ChemDraw-How-to-keep-modified-Document-Settings-as-the-default-for-New-Documents),
[Desktop 26.1](https://support.revvitysignals.com/hc/en-us/articles/54009539308052-Release-Announcement-ChemDraw-Desktop-26-1).

## CDXML data and provenance

A direct Page Text is a `t` with at least one `s` style child.
A left-aligned caption's stored `p` is the leftmost baseline point.
CDXML stores x then y in points, with coordinates increasing right
and down from the top-left origin.
[Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm),
[Text position](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/2DPosition.htm),
[Coordinates](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm).

An `s` carries decoded character data and its explicit font ID, face,
point size and foreground color. The font table maps IDs to names.
The foreground color is a two-based palette index. The archived SDK
prefers CaptionJustification over Justification when present and
describes containing-owner and last-used style-sheet fallback.
The published DTD gives Left defaults for the two Text justification
attributes and makes the style fields optional. Bondline writes
explicit `Justification="Left"` and a resolved style, and separately
checks native source authority and the candidate scene.
[Style](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Style.htm),
[Font](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Font.htm),
[Foreground color](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ForegroundColor.htm),
[Justification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Justification.htm),
[Caption justification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/CaptionJustification.htm).

The official DTD retrieved on 2026-11-19 is 28,821 bytes, SHA-256
`5311978e514ffe154108540c3634c314dc66031a4f3c877a681fc6dca8e128c2`.
Its root content lists colortable before fonttable. The existing
Bondline encoder writes fonttable before colortable, which stays
part of this exact byte contract. Libxml2 2.14.6 also rejected the
unmodified DTD at the `AS` enumeration on line 200. These witnesses
establish independent XML parse, ownership and byte facts. Strict
DTD validation remains open. [Official DTD](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd).

## Bondline scope and identity

Create one caption on an eligible single page from a source-free
drawing or a complete byte-exact native CDXML source. The native
draft supplies the accepted empty-paper point as the baseline and
uses one explicit Roboto Regular face 0 black 12 pt run. The click
anchor and the preset are original Bondline choices. A click on
painted caption ink continues through the existing edit workflow.
Apply sends `text.create` with the expected revision, page ID,
x and y in points, and decoded text. The existing caption editor
retains its own draft state and source guards.

Native offset mode applies to a single page without Text children
on Nodes. The Text XML id is the number of its preceding Page,
Fragment, Node and Bond objects plus the live TextId. This uses
checked raw IDs and preserves the existing dense ID bytes. Recovery
requires positive representable offsets and exact native re-encoding
of the whole candidate. A lexical comment can preserve the projected
drawing while revoking that native creation grant.

The independent fixtures use a 512 × 512 pt Poster page and create
`Phase β & q` at (144.25, 216.75) pt. Expat verifies the raw x/y,
the explicit style, decoded Unicode and ampersand, and distinct
logical and XML identities. First creation uses TextId 1 and XML
id 2. Undo followed by creation gives TextId 2 and XML id 3 on the
same otherwise empty page. The mixed drawing has a five-object
prefix and retains its existing styled caption at XML id 6. It
appends TextId 2 at XML id 7 using the existing resources and exact
unrelated XML bytes. [Independent witnesses](../../crates/io/tests/fixtures/caption-create/SOURCES.md).

The receiving tests still need to verify the command, native form,
post-Apply repaint, exact Save/reopen, and history behavior.
