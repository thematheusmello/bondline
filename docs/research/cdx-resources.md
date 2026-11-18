# Standalone CDX font and colour resources

Locked2026-11-18 ondd57e2a, after full local2882. This is a planned checked data query in
the standalone cdx crate. It changes no syntax admission, IO/model/command/render/UI
policy. No Text/font availability, styled-run reference, codec or authoring grant follows.

## Public evidence and decisions

Primary [FontTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/FontTable.htm),
[CDXFontTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXFontTable.htm),
[ColorTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ColorTable.htm),
[CDXColorTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXColorTable.htm),
[Unformatted](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/Unformatted.htm),
[CDXString](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm) and
[Font](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Font.htm) are reviewed. Audit receipt
manifest6375B SHA256 `a8574c064609a29687d1a4cc1632283766d4372cc1241607ced3fbe32adad9e4`
pins five fresh200 bodies, linked charset404 and five reused current receipts. The table
datatype/property bodies are independently reopened during this source lock. Only format
facts are used; no reference sample, binary, font or help artwork is copied.

FontTable0100 is Document-owned. Payload is UINT16 platform(0 Macintosh/1 Windows),
UINT16 count, then count entries: UINT16 declared ID/charset/name-byte-length plus name
bytes. Preserve unsigned charset high values and raw name bytes, including NUL/non-UTF8/
present-empty names. Admit only platform0/1; other platforms return a typed unsupported
platform query error, without changing raw parser admission. IDs are unique but need not
be positive/sequential; do not confuse them with chemical IDs. Present count0 is admitted
as an empty binary table, distinct from absence. No default font is fabricated.

The normative offset table defines the layout; its apparently short printed hex example
is not copied or repaired as a compatibility exception. CDXString's font-index wording
conflicts with nonsequential declared IDs. A lookup explicitly named font_by_id is only
declared-ID data lookup; it does not resolve a style-run font. XML Font's signed charset
wording and the linked404 establish no−1 inheritance or name codec.

ColorTable0300 is Document-owned. Payload is UINT16 count plus count RGB UINT16 triples.
Preserve raw0..65535 integer channels; no alpha/colour-space/transfer function or float
normalization is added. Reserved indices0/1 are black/white and are not stored. Stored
indices start at2. Only absent table supplies default2white/3black. Present count0 is an
empty table; a present one-entry table defines2 but not3. Never use absent defaults for
present-empty/short data. Use u32 for exposed colour indices/lookups, preserving all65535
stored entries and last index65536 without wrapping or dropping it. This wider data API
does not widen any UINT16 style-reference format.

## Query contract

Add Document::resources() -> Result<Resources<'_, 'input>, ResourceError>, with public
Resources/Font/Color/ResourceError exported from lib.rs. Keep runtime dependencies empty;
reuse the existing checked Cursor, records and payload ownership. Test-only use of an
existing workspace dependency is allowed when it simplifies independent facts; record
any manifest change precisely rather than implying a new runtime dependency.

Resources exposes optional original font_table()/color_table() Record handles,
font_platform()->Option<u16>, source-ordered fonts()/colors() slices,
font_by_id(u16)->Result<&Font,...> and color(u32)->Result<[u16;3],...>.
Font getters: id(), charset(), name_bytes(), ordinal(), range(), record().
Color getters: index(), rgb(), range(), record(). Ranges are absolute source entry spans;
font names borrow their exact input bytes, not decoded/copied strings. Fields/constructors
stay private; source provenance is not forged through public deserialization.

Discover exact0100/0300 Property tags only. For encoded framing, tables must directly
belong to opening Document record0; for implied framing, they must have no encoded parent.
Reject Page/Group/Fragment/Node/Text/custom/nested-Document owners. Do not mask user tags
or invent ancestor table ownership from generic fallback prose. Scan owners/duplicate
tables before decoding either payload; a second table refuses even when byte-identical.
Duplicate font IDs refuse at their second source ordinal. Unrelated data stays opaque.

Public ResourceError variants are Source(Error), UnsupportedTable{tag,record},
DuplicateTable{tag,record}, InvalidTable{tag,record},
UnsupportedPlatform{record,value}, DuplicateFontId{record,id,ordinal}, MissingFont(u16),
MissingColor(u32), Allocation. Use manual std Display/Error, not a new runtime dependency.
Table errors identify the actual property record; these are original query diagnostics,
not claimed reference-product errors. Keep old Error and parser precedence unchanged.

Validate complete headers/counts/name extents/trailing consumption using checked math.
Reuse64MiB/100000-record limits; no arbitrary smaller table cap. Fallible bounded vectors
may own entry descriptors, while names/Record handles borrow source. Validate each whole
table before reserving descriptor capacity. Duplicate-ID detection must be linear or
nlogn rather than quadratic over65535 entries. Preserve source order and all entries.
Query failure never changes Document/as_bytes/to_bytes/replace_property or becomes an
automatic IO Open failure. No syntax writer/default resource insertion is added.

## TDD and evidence

1. Freeze original Python-struct source/SHA and an independent reader's complete record/
   entry spans, raw names/charsets/platform/channels/presence/lookup facts. Cover all five
   existing framings, short/extended length, reordered tables, sparse0/7/65535 IDs,
   chemical-ID collisions, charset0/32768/65535, empty/NUL/non-UTF8/max65535-byte names,
   absent/empty/short palettes, nonstandard2/3 colours and max65535-entry colour table.
2. Declare the new API compiler probe explicitly; it cannot be called an existing-API
   runtime RED. A minimal fail-closed absence/empty-table implementation may establish
   runtime RED for nonempty tables before completing the decoder. Leave no placeholder,
   panic/todo or partial query in a committed feature. Independent oracle facts precede
   any producer. Record actual phases without manufacturing or conflating evidence.
3. Test every duplicate/owner/platform/length/count/truncation/trailing/boundary/missing
   lookup control. Check borrowed names/provenance, optional presence, unchanged raw
   round trips and generic property replacement with an independent actual-byte reader.
   Preserve equal-data/different-source handles and query-error syntax passthrough.
4. Prove existing CDX IO unchanged/no-op/drift/history/loss authority stays closed and
   malformed resource queries do not newly fail raw Open/Save. Do not change IO to call
   resources() automatically. Registry49/catalogue16/124, models/renderers/XML views and
   current source guards remain exact. RDKit graph equality is not a font/palette oracle.
5. Extend the existing CDX fuzzer with bounded query/source invariants; original immutable
   seeds stay outside writable corpus. Run scoped/full local gates, keep Actions OFF and
   push each atomic commit. No new native/platform/rendering claim follows from this API.

Actual CDX String/Text/effective styles/placement, faithful CPU/GPU/native preview and
source-preserving chemical edits remain separate required M1 work. Full M0–M5 stays active.
