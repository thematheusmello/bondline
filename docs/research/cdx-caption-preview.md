# Direct-Page CDX caption preview

This M1 workflow opens a source-qualified CDX caption, renders it through the
same canvas and capture entry points, and saves and reopens the original bytes.
The standalone source and datatype readers already expose checked resources,
records, style runs and raw text.
See [current verification](../verification-cdx-caption-preview.md) for the
actual command and macOS workflow coverage.

## Source agreement

Accept one direct Document Page containing one direct Text and the supported
resource properties. Check the complete ordered record index and exact owners.
Keep the five supported outer framings when their reserved header bytes agree
with the canonical envelope. Every property has a single owner and valid width.
The [public framing specification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm)
defines scoped source IDs. Preserve the original IDs and use the existing
application Page and Text identity allocation. A producer name alone carries
no projection information.

The Page must declare [Poster drawing space](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/DrawingSpaceType.htm)
with explicit [width](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Width.htm)
and [height](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Height.htm).
Map positive signed fixed-point extents to the current page model. A Text must
provide a [2D position](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/2DPosition.htm)
in signed y/x fixed-point order and explicit
[CaptionJustification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/CaptionJustification.htm)
Left on the same Text. These fields define a leftmost baseline. Accept absence
of BoundingBox under the [Text object](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm)
definition. Bounding-box fallback, legacy justification and inherited placement
remain separate from this source proof.
The [Visible property](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Visible.htm)
defines absent visibility as displayed. The first profile requires exactly the
declared Page and Text fields above.

One [0x0700](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Text.htm)
CDXString must have a single start-zero run with face zero and positive size
in twentieths of a point. Select an exact `Roboto` resource whose declared
ID and ordinal both equal the authored font word. An explicit Windows-platform
table and charset 65001 support the original strict UTF-8 policy for that run.
A nonempty body must pass the existing renderer's glyph, line and work checks.
Font-family agreement selects Bondline's licensed Roboto renderer. UTF8Text
precedence and legacy code-page defaults remain open. [Microsoft code pages](https://learn.microsoft.com/en-us/windows/win32/intl/code-page-identifiers)

Resolve the inline colour using the existing raw table query and scale each
RGB16 channel by 65535 into the existing TextStyle. The white canvas requires
palette index two to resolve to exact white. An absent palette uses the documented
default; a present empty palette has no index two. Explicit background properties
need separate proof. [Color table](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXColorTable.htm),
[background](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BackgroundColor.htm)

Multiple captions, Z-order, unsupported properties and foreign objects have
separate visual requirements. The [ZOrder description](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm)
leaves omitted/equal ordering unresolved for overlapping objects.

## Existing path and evidence

- Build the checked existing PageSize, StyledText, TextRun and TextStyle. Run
  the existing full virtual page scene at 100% as a bounded admission check.
  Its glyph, em, ink and work guards check the original page before camera
  clipping. Every view still runs its ordinary scale, DPI and rendering checks.
- Establish one read-only CDX proof tied to the immutable original source and
  exact current model. Consume it through drawing_page_scene[_with_view] and
  the existing native read-only input path. Leave CDXML caption-edit eligibility,
  tool metadata and command schemas unchanged.
- On a failed profile, delegate to the unchanged ElementalGraph path. Compare
  each old failure code and location. The actual CLI already refused two frozen
  source-equivalent caption originals at Text record five.
- Verify actual Open, CPU/GPU capture on the shared renderer, Fit, Actual and
  fractional density, known Save, Unicode Save As and exact reopen. Compare
  complete models and pixels with independent struct, font and PNG consumers.
- Test model-only page edit retirement of preview and Save, restoration by Undo,
  held-tool and Alt-click refusal, no authoring/history side effects from view
  input, cross-format sentinels, and rejection of old Save work after swapping
  equal-model, different-byte source owners. Run actual CLI, initialized MCP,
  authenticated loopback and an owned macOS review app.

The [original sources and legacy baseline](../../crates/io/tests/fixtures/cdx-caption/SOURCES.md)
fix the expected model, preserved bytes and old refusal details before the producer.
