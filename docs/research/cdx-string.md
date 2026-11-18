# Raw CDXString values

The public [CDXString datatype](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm)
stores a little-endian unsigned 16-bit run count, ten bytes per run and the remaining
text bytes. Each run contains five unsigned 16-bit fields in order: start, font,
face, size in twentieths of a point, and colour. The
[binary framing specification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm)
supplies the property payload boundary, including extended lengths. The
[Text property definition](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Text.htm)
uses the datatype at tag 0x0700 on Text objects. The raw query can also read other
properties the caller identifies as using the datatype.

The SDK describes starts in terms of characters. It does not settle byte versus
Unicode scalar versus UTF-16 units for arbitrary encodings. Its font-index wording
also conflicts with nonsequential declared IDs. The normative size scale is 20
units per point, despite the 12-point example using a size word of 12. The query
keeps all five fields literally, including zero, 65535, unordered, duplicate and
out-of-range starts. It leaves interpretation to the next layer. The SDK describes
zero runs as ISO-8859-1 with unspecified font and size. The raw view preserves
zero runs and the exact remainder without decoding or inserting an implicit run.

## API and boundaries

`Document::string(&Record)` returns a borrowed `StringValue` for a property record
owned by that exact parsed document. It uses the existing payload check, so a
record from an equal but separately parsed source, an object start or an end
record returns `StringError::Source(Error::InvalidProperty)`. This explicit
datatype query leaves tag and object semantics with the caller.

`StringValue::record` returns the checked source handle.
`text_bytes` borrows the literal remainder and `text_range` reports its absolute
source span. `runs` is an exact-size iterator of `StyleRun` values, with
`start`, `font`, `face`, `size_twentieths`, `color` and absolute `range` getters.
The view allocates no descriptors and borrows just the verified style and text
portions of the property. A missing count word or a declared run block extending
past the end of that property returns `StringError::InvalidPayload { record }`.
The raw view supports 65,535 runs and text larger than 65,535 bytes under the
existing source size limit. It does not change parser admission, raw passthrough,
the generic property writer, IO, app display or chemical edit authority.

## Independent tests

Freeze original Python struct files and an independent reader before adding the
API. Include all five framing profiles, extended properties, a zero-run empty
remainder, unusual literal fields, unequal byte/scalar/UTF-16 lengths, maximal
count, long text, foreign handles, and truncated run blocks followed by unrelated
bytes. Check original record/range identity, borrowed slices and exact unchanged
bytes. A generic property edit must be independently readable and preserve other
records. Record the new-API compiler RED separately from runtime results. Run
standalone, workspace and bounded sanitizer gates before integration.

## Next application workflow

The raw query is a prerequisite for document-open and capture of a direct-Page
CDX caption. The [Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm),
[2D position](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/2DPosition.htm),
[caption justification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/CaptionJustification.htm),
[drawing space](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/DrawingSpaceType.htm)
and [page background](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BackgroundColor.htm)
descriptions supply distinct source proofs. The initial path will require an
explicit left baseline, known fonts and colours, and sourced Poster extents. It
needs its own IO/capture/refusal contract and successful native Open/Save/reopen
evidence before any app preview claim.
