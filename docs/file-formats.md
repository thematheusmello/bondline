# File capabilities

| Format | Delivered scope | Evidence | Missing |
|---|---|---|---|
| CDXML | Standalone bounded UTF-8 syntax parser; unchanged source-byte writer | Original fixture + independent external Expat; BOM/CRLF preservation; malformed/resource cases; ASan fuzz smoke | Chemical model mapping, supported edits, corpus coverage, registry/native open/save |
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

Future source edits must retain untouched spans and revalidate the result. In particular,
do not blindly use roxmltree's compressed attribute-value positions: its documented QName
and equals-whitespace position limits require checked handling. No patching API is delivered
yet. See clean-room/sources.md, verification.md and fuzz/README.md for provenance and checks.
