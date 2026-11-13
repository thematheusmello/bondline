# File capabilities

| Format | Delivered scope | Evidence | Missing |
|---|---|---|---|
| CDXML | Standalone bounded UTF-8 syntax parser; unchanged source-byte writer; checked existing-attribute value replacement | Original fixture + independent external Expat; one pinned unchanged RDKit corpus file; exact unrelated-byte preservation; BOM/CRLF and hostile-input cases; ASan parse/edit fuzz smoke | Chemical model mapping, additional edit types, broader corpus coverage, registry/native open/save |
| CDX | None | Published specification recorded | Parser, preservation, writer and model mapping |
| MOL/SDF V2000/V3000, RXN, SMILES | None | Roadmap only | Import/export and independent chemistry verification |
| InChI | None | Roadmap only | Reference-tool comparison and import/export conventions |
| PNG | Validated RGBA8 encoder and registry page-preview capture | External Pillow, independent page grid, actual native loopback capture | Chemical/print-resolution output and native save workflow |
| SVG/PDF | None | Roadmap only | Authoring geometry/text, export and independent consumers |

CDXML original bytes are authoritative. The read-only XML tree is a derived view; writing
does not reserialize it. Unknown nodes, namespaced attributes, property order, quotes,
numeric spelling, whitespace, comments, CDATA, processing instructions and doctype remain
exactly as received. This is syntax preservation, not a claim that every property is understood.
The borrowed input must outlive its parsed view. No filesystem, URL fetch, fonts or model
dependencies exist in this crate. Font names inside an interchange document are metadata;
they do not bundle those fonts or select a font substitution policy.

Limits are independent Bondline policies: 64 MiB UTF-8 input, 100000 tree nodes, 256 nested
elements, 256 attributes per element and 100000 total attributes. A streaming quick-xml
preflight rejects excessive nesting/attributes, unsupported declarations and internal DTD
subsets before roxmltree builds a validated tree. XML 1.0 and optional UTF-8 declaration are
supported; other encodings/versions fail. External doctype identifiers are retained without
resolution; no entity resolver is installed. Custom DTD entities are unsupported and rejected.
There is no recovery mode that drops malformed or unsupported content.

`Document::replace_attribute(element, qname, value)` returns a new, fully revalidated byte
buffer. The element must belong to that parsed tree, and qname must match an existing
ordinary attribute's literal qualified name. Namespace declarations cannot be changed.
The API never adds attributes, changes text/structure or updates the input in place.
Every byte outside the value is retained, including the original quote delimiters. A value
equal to the decoded original is a byte-exact no-op, retaining character-reference spelling.
Special characters are escaped; tab, LF and CR use numeric references to survive XML
attribute normalization. Characters forbidden by XML 1.0 fail explicitly. Escaped output
size is checked against the 64 MiB bound before allocation; no partial output is returned.

The writer uses the full attribute source range, not roxmltree's compressed QName/value
positions. Tests exercise a 70000-character name and more than 255 bytes of equals-whitespace.
External Expat verifies decoded values including Unicode character-range boundaries.
The untouched corpus round-trip gate does not yet exercise edits on its RDKit fixture.
See clean-room/sources.md, verification.md and fuzz/README.md for provenance and checks.
