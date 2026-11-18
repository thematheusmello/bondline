# Original standalone CDX resource vectors

Original Bondline test data, MIT OR Apache-2.0, frozen on base `dd57e2a` against
[the locked standalone-resource contract](../../../../../docs/research/cdx-resources.md)
`391639c`. Standard-library Python `struct` creates every byte. Expected facts never
come from a Bondline parser, resource API, writer, executable or renderer. No reference
binary/export/sample, font asset, SDK example or copyrighted artwork is copied.

The normative public [CDXFontTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXFontTable.htm),
[FontTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/FontTable.htm),
[CDXColorTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXColorTable.htm),
[ColorTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ColorTable.htm) and
[IntroCDX](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm) supply field layout,
little-endian framing and ownership. The source register/contract record fresh receipts,
the font-index versus declared-ID ambiguity, unsigned binary charset versus signed XML
wording, and the linked charset404. No font-name codec,−1 inheritance, installed-font
availability, style-run resolution or visual fidelity is inferred. The apparently short
printed font-table hex example is neither copied nor repaired by a producer exception.

`resource-vectors.json` pins48 complete original sources:17 valid queries and31 query-only
errors. Every one retains complete raw Document syntax, framing and source bytes. The five
existing framings are ImplicitDocument,ExplicitDocument,HeaderDocument,
ImplicitDocumentFileEnd andExplicitDocumentFileEnd. All five carry the same meaningful
resource data; reordered and forced-small-extended properties are separate controls.
The previously sourced22-byte embedded-root convention remains qualified in the existing
CDX file-end research, rather than being substituted for the literal SDK28-byte header.

Each source contains an original two-carbon/H3 Single graph: Page90/Fragment51/Node7+
65535/Bond900. Font IDs0/7/65535 are a DIFFERENT namespace; the deliberate chemical-ID
collisions must not become duplicate-font errors. Original root/fragment/custom bytes
contain NUL/high bytes, and custom object400000001 remains opaque. The standalone API
does not interpret this graph, and IO is not changed to call resources() automatically.
Wrong-owner query fixtures may retain existing IO chemical-profile restrictions; this
work does not promise that previously unsupported IO shapes now open.

Fonts are source ordered, not ID sorted. The normal entries use IDs0/7/65535 and literal
unsigned charsets0/32768/65535, with empty, NUL-bearing and non-UTF8 original names.
Only platform0/1 is admitted; platform2/65535 is explicitly unsupported data. Empty
binary font tables on both platforms remain distinct from absent tables. ID lookup
means declared-ID lookup only, without claiming a Text style's font number is that ID.
Font ordinal is zero based; duplicate IDs fail at their second source ordinal, including
ordinal65534 in the late-duplicate maximum-size control. The duplicate/entry-truncation
cross-error order is not invented through a mixed-invalid fixture.

Palettes retain RGB UINT16 channels, not float-normalized or colour-space-transformed
values. Reserved0/1 always return black/white. Only ABSENT color tables supply2white/
3black; present-empty defines neither, one entry defines2 but not3, and explicit2/3 can
be nondefault colours. Probes cover UINT32 max lookup, missing65537, and stored index65536.
This wider logical index does not widen any UINT16 style-reference format.

Boundary sources remain compact in facts:

- `max_name` has one platform0/font7/charset65535 entry with exactly65535 name bytes,
  byte i=(53i+29) mod256. Its UINT16 name lengthFFFF is a literal65535, not an extended
  name-length marker. Only the enclosing property uses extended-length framing.
- `max_fonts` has65535 empty-name entries. IDs are0,65535, then1..65534 excluding32768;
  unsigned charsets repeat0,32768,65535. All IDs are unique and source order is retained.
- `max_colors` has65535 entries, ordinal i RGB=((73i+11) mod65536,
  (257i+32768) mod65536,65535-i). The final index65536 is[65401,32254,1].

`resource_oracle.py` independently checks all entries before producing facts. Long names
and large record payloads use byte count/SHA256/spans/prefix/suffix instead of giant hex.
Large tables expose complete count/entry-range/digest, five boundary descriptors and
lookup probes. `check_resources.py` separately verifies EVERY maximum-table entry against
the original formulas, all65535 name bytes and complete record-span byte coverage.
`resource-facts.json` stays compact rather than storing65k entry objects per boundary case.

Facts carry complete framing/header/source identity/ordered records and a `query` result.
Successful queries contain optional original table record handles, platform/counts,
source-ordered Font/Color descriptors, raw-name facts and exact lookup outcomes. Ranges
are absolute entry/name spans; record handles are actual source indices. Missing Font/
Color lookups use explicit error facts. Query errors return `query.ok=false` with the
public variant's actual tag/record/value/ID/ordinal, while the reader still returns the
whole syntactically valid source facts. Only genuinely truncated outer framing raises
an independent raw reader error.

All31 query-error controls cover Page/Group/Fragment/Node/Text/custom/nested-Document
owners for each table, duplicate tables, duplicate IDs, platform restrictions, complete
header/count/name bounds, trailing payload and maximum declared counts with missing
entries. Structural owner/duplicate scanning precedes payload decoding: duplicate-after-
bad-font and wrong-owner-after-bad-font controls pin that locked boundary. Exact user
property tags4100/4300 are positive opaque controls and must never be masked into0100/0300.

Reproduce with standard Python, no product build, network or external chemistry package:

```sh
python3 crates/cdx/tests/fixtures/resources/generate_resources.py
python3 crates/cdx/tests/fixtures/resources/check_resources.py
```

The generator writes ignored `target/cdx-resources/reproduced/`; the checker reads frozen
sources and writes ignored `target/cdx-resources/reproduced-checks/`. `--input-dir` and
`--output-dir` permit routes without rewriting references by default. All48 sources,
manifest, complete compact facts and checks reproduce byte-for-byte. The test helper
`xtask/oracles/cdx_resource_support.rs` exposes source/facts/read and uses only the existing
test serde_json edge. It is not a runtime resource parser. No RDKit font/palette claim,
new dependency, source-edit/authoring/render/UI permission or M1 completion follows.
Actions stays OFF and protected apps are untouched.
