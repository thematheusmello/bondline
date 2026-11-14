# File capabilities

| Format | Delivered scope | Evidence | Missing |
|---|---|---|---|
| CDXML | Source preservation, elemental projections/encoding, registry open/save and native Mac file workflow | Expat/Decimal; pinned corpus; metadata/order/identity tests; actual engine/CLI/MCP/socket/native files read by external RDKit; parser/writer ASan | General imported structural edits, queries/labels/styles/stereo, wider corpus and complete file-menu parity |
| CDX | Standalone bounded binary syntax, checked property edits and new-tree writer | Original field vectors, fetched positive scene, full-byte writer equality, independent Python struct/RDKit and bounded ASAN | Application import/export, model/reference mapping, foreign/general corpus and modern extensions |
| MOL V2000 | Source-preserving parser, strict elemental projection, canonical writer, atomic I/O and shared/native file workflow | Original fixed columns; pinned CH3 corpus/RDKit/Decimal; external-generated input; bounded ASAN; engine/CLI/MCP files; actual Mac chooser/save/reopen/close | Broader fields/layout/export policy and interactive Windows/Linux workflows below |
| SDF V2000 | Source-preserving library records, ordered data fields and validated wrapper writer | Original source spans; fetched pinned two-record corpus; external raw scanner/per-record RDKit; named-field supplier and external-writer vectors | Application data/multiple-record workflow and broader syntax/encodings |
| MOL V3000 | Source-preserving library parser, strict elemental projection and full-precision writer | Original sparse-ID/token/property vectors; external RDKit8dp/1000-atom files,118-symbol writer; bounded ASan; fetched original sparse-ID corpus | Version-aware application snapshots/I/O/commands/UI/SDF, broader queries/stereo/blocks |
| RXN, SMILES | None | Roadmap only | Import/export and independent chemistry verification |
| InChI | None | Roadmap only | Reference-tool comparison and import/export conventions |
| PNG | RGBA8 encoder and registry previews for empty pages and independent C-C/three-H fragments | External Pillow/FreeType, independent pixel/geometry/font vectors, actual native capture and one human-approved golden | General chemical/publishing output, physical resolution and native save workflow |
| SVG/PDF | None | Roadmap only | Authoring geometry/text, export and independent consumers |

## MOL V2000 library

`bondline_io::mol::MolFile` retains accepted original bytes exactly. Parsed views expose
headers, raw atom/bond fields and line-addressed unsupported features. `as_bytes()` is exact
passthrough; a partial model must never replace it. The application adapter now retains that
complete source in StoredDocument and exposes document.open_mol/document.save_mol through
the same registry, CLI, MCP and authenticated loopback as CDXML. Native execution evidence
is recorded separately from these library/adapter tests.

The strict graph projection supports the118 elemental symbols, single/double/triple/aromatic
bonds, charge, radical and absolute isotopes. Its caller supplies MolTransform(scale,origin):
raw x/y increase right/up, drawing points right/down. No units, typical bond length or
chemistry are guessed from the file. Declared valence includes implied H in the published
format; subtracting known integer bond orders decodes authored H exactly. Zero valence is
encoded15, distinct from unmarked0. The hydrogen-query field is not an exact H count.
Aromatic declared-valence combinations remain unsupported.

CHG/RAD properties replace legacy charge/radical codes globally; M ISO replaces legacy
isotope values globally, with unlisted atoms natural. A focused original vector exposes a
pinned RDKit discrepancy: it retains an unlisted legacy mass difference. Bondline follows
the published2026 rule and preserves the raw input; separate tests assert the consumer
behavior. Canonical output has zero legacy mass deltas and explicit absolute properties,
which the consumer reads consistently. A nonzero mass difference without an overriding
property remains unsupported until a verified reference-mass table exists. D/T aliases are
not defined by the reviewed symbol table; no alias/override precedence is invented.

Unknown properties, queries, stereo,3D and unverified metadata stay in raw source and cause
explicit model-conversion refusal. SDF content and trailing nonblank data are not accepted
as a MOL record. No mode silently drops them. ASCII with LF/CRLF is supported. Bounds are
64MiB,999 atoms/bonds,4096bytes per line and100000lines, with checked indices and no recursive
syntax tree. Broader encoding and format coverage remain M1 work.

`write_v2000` encodes supported graphs; `write_v2000_with_header` retains the three supplied
ASCII header lines (at most80bytes each). Coordinates use the format's10-column/four-decimal
representation: maximum rounding error is half a0.0001 raw unit plus floating representation
error. Missing positions, width/range overflow, unrepresentable metadata and unsupported
orders fail without output. Known explicit H becomes an exact declared valence; unknown H
stays unmarked. Canonical writing can change spelling/property order and is not a byte-exact
edited-source claim. Raw passthrough remains exact.

The reviewed170-byte RDKit CH3 document now has actual corpus parsing/source/model checks;
formula and SMILES in those checks are external consumer expectations, not Bondline chemistry
algorithms. Original ethane columns and external-generated vectors independently exercise
writer interoperability. See [test provenance](../crates/io/tests/fixtures/mol/README.md),
[verification](verification.md) and [fuzz limits](../fuzz/README.md).

## MOL V3000 library

`bondline_io::mol::v3000::MolFile` is a separate library codec; existing application
MOL commands and SDF records still use V2000. It borrows exact complete source and
three headers, exposes positive noncontiguous atom/bond source IDs and physical
spans, and keeps a flat bounded block table. Model identities are fresh dense IDs.
Logical entries remove only final continuation dashes and subsequent exact `M  V30 `
prefixes, without adding a space. Quoted strings decode doubled quotes; flat counted
lists validate declared lengths. Unknown attributes, blocks and order remain in
original source, and unsupported projection refuses regeneration. Unknown-block
internal semantic counts/references are not claimed validated.

The strict subset is 2D elements 1..118, mapping 0, charge −15..15, RAD 0..3, positive
integer MASS fitting nonzero u16, VAL 0/absent unspecified, −1 explicit zero and 1..14
declared valence, and single/double/triple bonds. Explicit H comes only from checked
declared valence minus integer bond-order sums. HCOUNT is a query; V3000 type 4 is an
aromatic query. Neither becomes ordinary authored model data. CFG, collections,
maps, 3D, queries, extra properties and unknown blocks cause strict mapping refusal.
MASS's positive-integer profile is a supported subset, not a universal reading of
the published weight field. RDKit's MASS 13.5 truncation and singlet/triplet collapse
remain separate consumer diagnostics.

V3000 CTAB counts are authoritative over ignored legacy counts. Atom/bond counts,
positive unique source IDs and endpoint references are validated. Required COUNTS,
ATOM and optional BOND ordering and balanced blocks are checked iteratively, without
a recursive DOM/drop. Complete `M  END` and strict EOF are required. Printable ASCII,
LF/CRLF and tab-separated entries form the current encoding profile. Limits: 64 MiB
source/output, 100,000 atoms/bonds and counted object categories, 1 million physical
lines, 80 ASCII characters/physical line, 65,536 bytes/logical entry, 1,024 segments/entry,
8,192 tokens/list values per entry, 4 million decoded tokens/list values per file and
256 block depth. Only the 80-character limit is the cited format rule; the other
bounds/flat-list/strict-EOF policies are Bondline choices, not a hard RSS cap.

`write_v3000[_with_header]` canonically writes supported authored graphs and retains
the three supplied ASCII headers. Coordinates use Rust 1.94.1 f64 Display's shortest
round-trip decimal without V2000's four-decimal quantization. New physical lines
are at most 80 characters using published continuation, even within tokens. Raw float
spelling reparses exactly; frame reconstruction is compared within
`16*EPSILON*max(1,abs(point),abs(origin),abs(raw*scale))` drawing points. Nonfinite
export through a subnormal scale refuses. Nonzero source coordinate underflow also
refuses, preventing an underflowing Z from becoming apparently 2D. Canonical writing
is distinct from unchanged-source passthrough and never authorizes erasing unknown
data. See [fixture provenance](../crates/io/tests/fixtures/v3000/README.md).

## MOL application snapshots and saving

Owned MOL snapshots share original bytes, baseline model, three headers and the captured
coordinate transform. History storage accounts for these shared allocations once, including
header capacities. Raw source/header content is absent from Debug and JSON command results.
An unchanged model borrows the original bytes exactly. A changed model can regenerate only
when every accepted source field is modeled and headers/coordinates are representable;
headers and the original drawing frame remain attached. Tests compare actual emitted bytes
with a separately run RDKit consumer; the codec's known isotope-input discrepancy stays explicit.

The registry imports into an A4 preview using24points per raw unit and origin(72,72)pt by
default; callers can supply a positive finite scale and bounded origin. These are original
drawing-placement choices, not inferred physical units. Source-free and proven-native CDXML
exports use that default frame. Imported MOL always uses its retained frame. Encoding rounds
coordinates to four decimal raw units; it does not promise exact model-coordinate reopening.

Saving must preserve one page, one authored fragment and supported metadata. Imported MOL
also retains its baseline page identity/size. Native MOL exports currently require default
A4/page1. Multiple fragments/pages, changed page layout, MOL-to-CDXML source metadata loss
and unknown-CDXML-to-MOL refuse explicitly. A deliberately lossy/export-only workflow remains
missing. Appending another isolated fragment is a valid edit but cannot currently save as
MOL; Undo restores the savable snapshot. This is partial M1 interoperability, not completion.

Native reads reuse bounded ordinary-file validation; both encodings use the same reviewed
atomic/no-clobber/overwrite/cancellation/security publication path. File state/receipts carry
the actual associated encoding, independently of the filename extension and original source
format. Stale imports cannot replace current work; an older published save keeps later edits
dirty. See [control-protocol.md](control-protocol.md) for schemas and close/request guards.

## SDF V2000 container library

`bondline_io::sdf::SdfFile` validates complete V2000 records and exposes borrowed original
bytes, record/Molfile/delimiter spans and ordered data items. Each item retains its raw
header, optional name and textual DT number, value lines and exact raw value/EOL span.
Duplicates, empty/multiline/spaces-only values and combined registry headers remain distinct;
they are never reconstructed from a property dictionary. Molfile views use the existing
codec; callers explicitly choose any later model transform. There is no application SDF
open/save command or implicit multi-record document conversion yet.

BIOVIA2026 printed pages 97–99 define named/DT headers, line-prefix record delimiters,
200-character data lines and empty item terminators. This slice supports ASCII, so those
value lines have a 200-byte limit. Complete M END and column-zero prefix $$$$ delimiters
are required. Structure-only records have no padding before their delimiter. Empty titles
are significant; unfinished later records and global trailing blank padding fail. Extra
empty padding only after a completed data item is a documented reader accommodation for
external empty-value writers. Spaces-only lines remain values, without trimming.

Independent resource policies are 64 MiB total, 1024 records, 65536 fields, one million
lines and 4096-byte structural/header lines. Ambiguous multiple header identities and
unbalanced optional-parenthesis metadata explicitly refuse the supported profile; this is
a conservative interpretation limit, not a claim that all such free text is forbidden by
the general standard. Datfiles without structures, V3000 and other encodings remain open.

`write_v2000` takes ordered raw-header/data-line inputs and validated original MOL blocks.
It keeps MOL bytes/CRLF/unknown properties, adds a final LF if needed, and writes new wrapper
lines with LF and one item terminator. Canonical delimiter tails/padding may differ from
an imported file; exact original passthrough is separate. A reserved delimiter prefix in
an embedded MOL, invalid fields, unsupported syntax or exceeded output budget refuses
before output. It never escapes, silently trims or regenerates opaque MOL content.

The original 690-byte two-record fixture has independent raw-span/ordered-field checks and
two direct RDKit Molfile consumers. A separate named-field writer test uses the full SDF
supplier, and an external writer supplies empty-padding input. The pinned RDKit full supplier
loses data and the second record at a valid DT-only header; duplicate names also use its
last-value map. These diagnostics stay separate from preservation assertions. No complete
supplier agreement is claimed. The fetched corpus pins this original source at public
commit fb86832a0c9b7744fb3ef8f8115c7ed296164505 and checks exact ordered data plus both
independently consumed molecular projections. See [corpus scope](corpus/README.md).

## CDX syntax

`bondline-cdx` is a standalone syntax utility, with no model, filesystem, GUI or third-party
dependency. Its flat borrowed index retains the original header, unknown object/property
tags, payloads, ordering and length prefixes; passthrough returns exactly the input. It
accepts a header-implied root and a leading explicit8000 Document closed by one end record.
The known twelve header bytes are validated; the final sixteen reserved bytes stay opaque.
The archived specification/examples disagree about root framing; external consumer probes
establish only those original vectors, not every historical or modern file variant.

Binary records are decoded little-endian with checked offsets. Limits are independent
Bondline policies:64 MiB input/output,100000 records and256 simultaneously open encoded
objects. Parsing and destruction are iterative. Missing fields/terminators, excessive
budgets and bytes after the matched root end fail without returning a partial document.
Standalone syntax does not validate Page requirements, ID uniqueness/references, payload
types, chemistry or object containment semantics.

`Document::replace_property` requires a property handle belonging to that exact parsed
document and returns size-checked new bytes. A no-op retains the original length prefix,
including a small payload encoded in extended form. A changed value uses a UINT16 length
below65535 or FFFF followed by UINT32 otherwise. Every byte outside that property record
is preserved; the original input stays unchanged. The utility never adds/removes objects
or properties and is not wired into application CDX open/save commands. Independent RDKit
checks two carbons/one bond and CC/C2H6 changing to C=C/C2H4 for one original fixture; that
consumer's raw16.16/y-flipped coordinates are kept separate from point-unit assertions.

The fetched corpus pins the original65750-byte explicit-root chemical vector at public
30ae30e73a4ee3b68d41069376fc5690faa99bc4. All29 record spans/parents/length encodings and
every opaque source byte are checked; the exact-scene external consumer checks ethane.
Changed opaque bytes, bond order, spans or a different explicit scene fail this original-
vector gate. It deliberately does not claim to validate arbitrary CDX chemistry or foreign
files. Four pinned files now span three mandatory source bundles; chemistry calculations
and application CDX remain separate missing coverage.

## CDXML syntax and mapping

CDXML original bytes are authoritative. The read-only XML tree is a derived view; writing
does not reserialize it. Unknown nodes, namespaced attributes, property order, quotes,
numeric spelling, whitespace, comments, CDATA, processing instructions and doctype remain
exactly as received. This is syntax preservation, not a claim that every property is understood.
The borrowed input must outlive its parsed view. No filesystem, URL fetch, fonts or model
dependencies exist in this crate. Font names inside an interchange document are metadata;
they do not bundle those fonts or select a font substitution policy.

The separate `bondline_io::files` native adapter now provides bounded ordinary-file CDXML
reads and atomic complete-source saves. Default publication is no-clobber; overwrite needs
explicit opt-in. It preserves supported access metadata and exposes post-publication/recovery
outcomes instead of claiming every error rolled back. See [the filesystem contract](native-file-io.md)
for platform evidence, metadata bounds and concurrency/durability limits. This library adapter
alone does not establish registry/native file-menu parity or general imported structural editing.

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

`bondline_cdxml::Writer` constructs **new syntax** with a CDXML root and UTF-8 XML 1.0
declaration. It writes nested start/empty/end elements with ordinary unnamespaced ASCII
names and escaped attribute values. It does not validate chemical containment or regenerate
an imported tree. Failed operations leave prior writer state unchanged. Output size includes
reserved closing tags; depth, node and attribute budgets are checked before appending a tag.
Unbalanced output fails, and finish reparses the complete result. Unicode **values** and
numeric tab/LF/CR references are supported; new namespace declarations/prefixed or Unicode
names and text-content writing are not. Imported unknown names/content still remain exact
on the separate source-preserving path.

`bondline_io::cdxml::write_document` encodes a new native elemental model. Every page,
fragment, node and bond gets a globally unique positive UINT32 source ID in document order;
sparse/maximal application IDs do not collide or overflow through a direct domain cast.
Application IDs are distinct and newly allocated on import. Element, optional position,
isotope, charge, radical, optional explicit hydrogen count, bond order and begin/end direction
are retained. All sixteen modeled orders have independent Expat/Decimal projection evidence.
Isotopes above the format's signed INT16 range and syntax-budget excesses fail without output.
This is elemental interchange, not general text/style/stereo display serialization.

Native canvas dimensions are represented as `DrawingSpace="Poster"` plus Width/Height in
points, as documented by the public SDK. This preserves the explicit drawing extent; it
does not create Windows/macOS printer records or establish a physical printing workflow.
The owned reader uses complete, valid explicit Poster dimensions. Otherwise the caller's
preview size remains a fallback: Width/Height have no size meaning for Pages/default mode.
A Poster with one missing dimension or invalid explicit values is rejected by the model
projection; standalone syntax still preserves it. Both absent retain the preview fallback.
BoundingBox is never interpreted as paper size. Printer settings remain opaque.

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
file workflow. StoredDocument now owns the model, original source allocation and baseline,
and Engine history retains the whole snapshot. It accepts at most one direct page, or an
empty CDXML document; table-contained/multiple pages fail explicitly. Complete explicit
Poster extents are read; the caller supplies fallback preview dimensions for other cases.
This does not decode printer/style settings or infer paper from BoundingBox.
export_cdxml borrows the exact original bytes only while the model equals its baseline.
Any unsynchronized model edit fails with UnsupportedEdit; exact restoration/undo permits
passthrough again. That borrowed-only method returns NoCdxmlSource for native data. The new
to_cdxml method instead returns borrowed exact source or owned freshly encoded native bytes;
unsupported source-backed edits return UnsupportedEdit; the proven native profile below can regenerate edits. Never regenerate stored.model() to bypass
this guard: the flat model does not include all original content. Source
bytes survive input lifetime, cloning and history without being exposed in Debug or command
result serialization. Native file commands use this API through the separate filesystem adapter; see native-file-ui.md for their execution scope.
Source bindings for structural edits, faithful imported-file rendering and full interpretation remain missing.
An exact current native elemental encoding is now classified as fully mapped. The constructor
re-encodes the projected model and compares **every source byte**, including metadata and
layout, before caching that decision; a creator name alone grants nothing. Its original
source/baseline stay in history. Unchanged export borrows the exact original; edits can encode
new bytes, and Undo restores original passthrough. Headless rendering and bond.create now
work after reopening this profile. The native workspace/file commands now use this proven profile; see native-file-ui.md.
Added styles, captions, comments, unknown fields, another version or noncanonical spelling
retain opaque source and the edit/preview guards. This narrow proof is an initial supported
profile, not a substitute for general source-preserving CDXML editing. The pinned original
v0 vector in crates/io/tests/fixtures must remain editable through future migrations; do not
refresh it when changing the encoder/version. The profile codec remains separate from the delivered file-command/native workflow. The external RDKit 2026.09.2 consumer checks one actual engine-created
C-C/three-H case as ethane, not every file property, appearance or advanced bond chemistry.
The corpus gate checks one owned snapshot's projection against independent external Expat/Decimal,
in addition to its unchanged-byte/XML checks. It explicitly reports chemistry calculations
as missing rather than treating the projection as formula/stereo/SMILES/InChI verification.

## New CDX tree syntax

The standalone Writer starts the standard header, sixteen zero reserved bytes and explicit
Document8000/ID0; finish closes that root. start_object/property/end_object build flat
ordered syntax. Object tags require bit15; properties require a nonzero tag with bit15 clear.
Lengths below65535 use UINT16;65535 and above use FFFF/UINT32. Raw payloads are unchanged.
No-op imported property replacement still retains its original length spelling separately.

Open objects must balance inside the automatically managed root. Byte/depth/record checks
include all pending terminators before append; failures publish no partial record. Limits
remain64MiB,100000 total records and256 open encoded objects. Allocation failure is typed;
these are logical format limits, not a hard process-RSS promise. Debug prints only counts.

The builder creates new syntax, not imported-file reconstruction. It does not validate IDs,
references, property data types, containment or chemistry. Unknown original bytes remain on
the existing source-preserving path. The original65750-byte chemical scene is reproduced
byte-for-byte and independently read as ethane by RDKit; literal prefix/boundary tests and
source/construction ASAN supplement that narrow proof. Application CDX remains missing.

## Fetched original V3000 corpus

The fourth mandatory source bundle pins public5caf6ce0ecf40badf2666e36dcb73dc8c4337b51's
original344-byte sparse-ID vector, SHA1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380.
An actual cold HTTPS download, exact original bytes, raw source IDs10/40/bond7, dense model
IDs1/2/bond1 and authored H3/single graph agree with separate literal/Decimal/RDKit checks.
The corpus frame24pt/unit, origin(0,0), invertedY is an explicit test policy, not physical
chemical units or the application's separate default origin(72,72). One original vector
is checked, not general V3000 interoperability or Bondline chemistry calculations.
