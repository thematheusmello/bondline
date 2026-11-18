# Multiple explicit CDX caption runs

Reviewed 2026-11-18. Extend the [ordered caption profile](cdx-caption-order.md)
with multiple literal style runs per Text object. The source-qualified Page
stays read-only. Save preserves the complete original CDX. Earlier placement,
font, white paper, depth, identity and exact owner checks still apply.

## Start offsets and style

The [CDXString specification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm)
describes a start at a character. Its three-run ASCII example gives starts 0, 2
and 3 and separates the corresponding text in three CDXML styles. The
[Text property](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Text.htm)
stores this value in property 0x0700. Keep the existing raw reader and verify
interpretation only in the application projection.

Start at zero and require strictly increasing internal starts. Every run has
a nonempty text slice. In the existing strict UTF-8 profile, a shared offset
under byte, Unicode scalar and UTF-16 counting consumes an ASCII prefix.
Require ASCII before the last start. The final slice can contain supported
Unicode. This follows from the [Unicode encoding forms](https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/)
and leaves CDX starts following multibyte text unresolved. Zero runs, an
empty trailing run and implicit properties from
[CaptionStyle](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/CaptionStyle.htm)
have separate semantics.

Check each run's explicit matching font ID and ordinal, UTF-8 charset, Roboto
Regular face zero, point size in twentieths and palette index. Preserve source
run boundaries, including consecutive identical styles and blank-only runs.
The shared scene already tests per-run em bounds, glyph availability, all
paper ink and raster budget. Enforce its existing page-wide run limit before
allocating run data.

## Glyph layout at joins

The existing renderer advances one baseline, applies pair kerning within a
run and resets it at a styled boundary. For a new CDX join, require a zero
pair adjustment in the pinned Roboto and no eligible ligature pair starting
across it. This avoids choosing whether a style transition breaks kerning or a
ligature in the reference. The [OpenType GPOS](https://learn.microsoft.com/en-us/typography/opentype/spec/gpos)
and [GSUB](https://learn.microsoft.com/en-us/typography/opentype/spec/gsub)
tables distinguish these two effects. Independent FontTools extraction from our
licensed font finds `ff`, `fi`, `fl` and `st` as the non-mark pairs with
an ASCII left operand in its ligature sequences. Keep the existing mark and
unsupported glyph checks. General shaping remains open.

Freeze independent binary, model and FontTools facts before the producer.
Exercise different colours/sizes, an unchanged Unicode tail, a blank style,
multiple caption siblings and the formerly refused two-run witness. Preserve
all other legacy refusals. Verify joined geometry and colours independently,
compare CPU and Metal, and check actual macOS Open and exact Unicode Save and
reopen. The command registry, model schema and edit authority stay unchanged.
