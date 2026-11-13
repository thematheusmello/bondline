# File capabilities

| Format | Delivered scope | Evidence | Missing |
|---|---|---|---|
| CDXML | Bounded syntax/source preservation, attribute replacement, geometry and read-only elemental projections with fresh application IDs | Original fixtures/public vectors; independent Expat/Decimal; one pinned RDKit syntax/elemental corpus check; ASan parser/edit/mapping smoke | Complete application document, queries/labels/styles/stereo, reference scopes beyond each fragment, dependent edits, broader corpus and registry/native open/save |
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

`bondline_io::cdxml::Geometry` maps unnamespaced n elements directly inside
CDXML/page/fragment. Other n layouts (including groups and nested fragments) fail explicitly;
namespaced extension elements remain opaque in the original bytes. Each mapped node needs
a nonblank ID, unique by its decoded string among mapped nodes. IDs are retained strings,
not yet interpreted as numeric/global application identities. Font IDs and other object
classes are not validated by this view. No chemical meaning is inferred from labels or NodeType.

An explicit p is exactly two finite decimal coordinates, x then y, in 1/72-inch points.
The published "72 144" vector means one inch right/two inches down. The model's ±100000 pt
bound is an independent product policy, not a CDXML format limit. Malformed/out-of-range
positions fail; absence remains None instead of generating an origin or running cleanup.
The adapter never changes its borrowed source. with_node_position returns new bytes for an
existing p and preserves semantically unchanged numeric spellings/character references.
It refuses a missing position or unknown target. Only p is replaced: dependent caption/
label positions and bounding boxes are not recomputed. This is a format utility, not a
complete atom-move command or application import/save workflow. All unmodelled properties
and objects stay in the source; the four geometry tests include independent Expat evidence.
The corpus gate still checks its unchanged fixture, not this geometry edit path.

The standalone syntax crate also exposes parse_integer for supported integral property
spellings and ObjectId for UINT32 source identity. The integer helper uses exact signed
decimal conversion with an optional nonempty all-zero fraction; it never rounds through
f64. XML whitespace and an optional ASCII sign are accepted. Fractional values, overflow,
exponents, empty fractions and nondecimal notation fail. ObjectId checks the unsigned
32-bit range and reports zero as unassigned. This does not allocate application IDs or
resolve references. Source IDs may repeat across containers; a graph importer must respect
scope and must not create references to unassigned zero. The older string-based geometry
view is unchanged. Original numeric spelling remains in the authoritative source bytes.

`ElementalGraph` is a separate read-only projection. It maps fragments directly under
pages or nested groups, retaining the complete source as its authoritative backing. It
assigns fresh nonzero application IDs in document order; source IDs are decoded exactly
and resolved within each fragment. Reused IDs in different containers and unassigned
zero/missing IDs do not alias application atoms. A reference to zero or outside the local
fragment fails. The mapper checks sibling identity for the common CDXML/page/group/
fragment/n/b/t/graphic/curve/objecttag classes; other object IDs/references remain opaque.
This is not a general nearest-object resolver or full CDXML semantic validator.

Supported atom fields are Element, optional p, Isotope, Charge, Radical and NumHydrogens.
The archive's missing-value defaults are retained without valence/label inference; missing
positions and hydrogen counts stay absent. Isotope zero maps to natural/unspecified mass.
Only Element NodeType is supported. A text-labeled node without explicit Element is
rejected: the public Node guide permits text-only writers, and their labels cannot safely
be guessed as carbon by this projection. Full chemical label parsing remains missing.

All sixteen published single-term bond kinds are mapped. Decimal integer/half-integer
orders are checked exactly, including trailing-zero spellings; tiny nonzero fractions are
not rounded into another order. Begin/end direction is retained. Query combinations and
undefined orders fail. The pure model validates endpoint/identity/topology budgets too.

Unknown attributes/objects, group hierarchy, text, display styles and stereo annotations
remain byte-exact in the source, but are not represented by the flat elemental projection.
to_bytes returns that unchanged source; changing a copied model value is not an editable
file workflow. Application document integration, source bindings for structural edits,
chemical rendering, native/registry open/save and full chemistry interpretation remain missing.
The corpus gate checks one pinned projection against independent external Expat/Decimal,
in addition to its unchanged-byte/XML checks. It explicitly reports chemistry calculations
as missing rather than treating the projection as formula/stereo/SMILES/InChI verification.
