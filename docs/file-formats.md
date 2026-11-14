# File capabilities

| Format | Delivered scope | Evidence | Missing |
|---|---|---|---|
| CDXML | Source preservation, elemental projections/encoding, registry open/save and native Mac file workflow | Expat/Decimal; pinned corpus; metadata/order/identity tests; actual engine/CLI/MCP/socket/native files read by external RDKit; parser/writer ASan | General imported structural edits, queries/labels/styles/stereo, wider corpus and complete file-menu parity |
| CDX | Standalone syntax/property edits/new-tree writer; limited source-backed snapshots, atomic I/O and guarded shared open/save commands; chooser wired | Original field vectors, fetched scene, independent struct/RDKit, atomic failure/ASAN and actual engine/CLI/stdio/socket saved-file checks | Actual native CDX workflow, faithful preview, broader metadata/references/chemical mapping, editing/encoder and foreign/general corpus |
| MOL V2000 | Source-preserving parser, strict elemental projection, canonical writer, atomic I/O and shared/native file workflow | Original fixed columns; pinned CH3 corpus/RDKit/Decimal; external-generated input; bounded ASAN; engine/CLI/MCP files; actual Mac chooser/save/reopen/close | Broader fields/layout/export policy and interactive Windows/Linux workflows below |
| SDF V2000/V3000 | Source-preserving mixed library records, per-record MOL version, ordered data fields, generic preserving writer and strict V2000 writer | Original spans/order; fetched V2000 two-record corpus; external raw scanner/per-record RDKit; version-switched SDWriter/ordinary full supplier; 8dp/1000-atom inputs; bounded Asan | Application data/multiple-record workflow, fetched mixed-version corpus and broader syntax/encodings |
| MOL V3000 | Source-preserving codec, version-aware snapshots/atomic I/O/shared commands and native Mac file workflow | Original sparse-ID/token/property vectors; external RDKit8dp/1000-atom files; engine/CLI/stdio/loopback/UI; bounded Asan; actual Mac author/save/reopen/refusal/close; fetched original sparse-ID corpus | Interactive Windows/Linux and broader queries/stereo/blocks |
| RXN V2000 wrapper | Standalone source-preserving reader/new-wrapper writer with ordered roles and raw V2000/V3000 molecular payloads | Published CTfile layout; original field/role/span vectors; external RDKit readers/writers and bounded ASAN | Reaction model, application/native commands, V3000 reaction wrapper, broader registry/extension handling and fetched corpus |
| SMILES | Bounded nonaromatic notation graph/source passthrough and explicit-H neutral Fragment export, library only; generated output is noncanonical | Original literal spans/adjacency/output/limits; qualified pinned unsanitized graph and separate sanitized meaning consumers | Import/model/implicit-H/perception decisions, source-backed edited regeneration, application/clipboard/layout, aromatic/stereo/query and canonicalization |
| InChI | None | Roadmap only | Reference-tool comparison and import/export conventions |
| PNG | RGBA8 encoder and registry previews for empty pages and positioned neutral-carbon/Single graphs with literal authored H | External Pillow/FreeType, independent graph/pixel/geometry/font vectors, actual native capture and one human-approved golden | Other elements/orders/metadata, source-faithful/general chemical publishing, physical resolution and native save workflow |
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

`bondline_io::mol::v3000::MolFile` is a separate library codec. Application MOL commands
and SDF records now dispatch the corresponding V2000/V3000 syntax codec. It borrows exact complete source and
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

Owned MOL snapshots retain their decoded V2000/V3000 version and share original bytes,
baseline model, three headers and the captured
coordinate transform. History storage accounts for these shared allocations once, including
header capacities. Raw source/header content is absent from Debug and JSON command results.
An unchanged model borrows the original bytes exactly. A changed model can regenerate only
when every accepted source field is modeled and headers/coordinates are representable;
headers and the original drawing frame remain attached. Tests compare actual emitted bytes
with a separately run RDKit consumer; the codec's known isotope-input discrepancy stays explicit.

The registry imports into an A4 preview using24points per raw unit and origin(72,72)pt by
default; callers can supply a positive finite scale and bounded origin. These are original
drawing-placement choices, not inferred physical units. Source-free and proven-native CDXML
exports use that default frame. Imported MOL always uses its retained frame. V2000 rounds
coordinates to four decimal raw units; V3000 uses shortest-roundtrip f64 spelling and the
declared reconstruction tolerance above. Neither promises universal bitwise coordinate
identity after an arbitrary drawing transform. Unchanged same-version source remains exact.

`StoredDocument::from_mol` checks the version stamp at line4 columns34–39, then fully
validates the selected codec and strict model projection. A literal unknown stamp is
invalid_mol, not an extension-based fallback. `mol_version` exposes decoded source version;
`to_mol`/`save_mol_file` retain it. Explicit `to_mol_as`/`save_mol_file_as` select a version.
Native/proven native-CDXML defaults to V2000 but may explicitly export V3000. V3000-source
to V2000 conversion always refuses unsupported_content before writing. V2000-source to
V3000 additionally requires complete native-field coverage and a cached raw-coordinate
comparison within16*EPSILON*max(1,abs(before),abs(after)); the V2000 four-decimal allowance
cannot authorize V3000 precision loss. Raw headers/frame/baseline stay shared in history.

The existing two registry MOL commands now support both versions. Auto Save captures the
associated MOL version, otherwise original-source version, otherwise V2000. Receipts and
file state distinguish mol_v2000/mol_v3000 while source.format remains mol. Open presents
one MOL family choice; Save As exposes both concrete versions. A .mol suffix cannot choose
between them, so name edits/filtering never silently downgrade Save. Known-Save and failed
retry retain exact path/version; actual version changes invalidate pending UI/close intents.
See [application verification](verification-v3000-application.md) for evidence and limits.

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

## SDF container library

`bondline_io::sdf::SdfFile` validates complete V2000/V3000 records and exposes borrowed original
bytes, record/Molfile/delimiter spans and ordered data items. Each item retains its raw
header, optional name and textual DT number, value lines and exact raw value/EOL span.
Duplicates, empty/multiline/spaces-only values and combined registry headers remain distinct;
they are never reconstructed from a property dictionary. Each record reports its actual
MOL version and uses the corresponding existing syntax codec; callers explicitly choose
any later model transform. There is no application SDF
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
the general standard. Datfiles without structures and broader encodings remain open.

`write` takes ordered raw-header/data-line inputs and validated original V2000/V3000 MOL
blocks. `write_v2000` shares the framing code but refuses V3000 inputs rather than converting
or silently widening its contract. Both keep MOL bytes/CRLF/unknown properties, add a final LF if needed, and write new wrapper
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
independently consumed molecular projections. See [corpus scope](corpus/README.md). A fetched mixed-version entry remains separate tooling work.

## CDX syntax

`bondline-cdx` is a standalone syntax utility, with no model, filesystem, GUI or third-party
dependency. Its flat borrowed index retains the original header, unknown object/property
tags, payloads, ordering and length prefixes; passthrough returns exactly the input. It
keeps both28-byte legacy profiles: header-implied root or explicit8000 Document with one
root end. A distinct header-embedded Document profile also retains its22-byte prefix,
encoded root and separate file end; see the detailed profile below. The known twelve header
bytes are validated and other prefix bytes stay opaque. A successful legacy interpretation
wins when reserved bytes are ambiguous. Independent vectors/native observations establish
these bounded profiles, not every historical or modern file variant.

Binary records are decoded little-endian with checked offsets. Limits are independent
Bondline policies:64 MiB input/output,100000 records and256 simultaneously open encoded
objects. Parsing and destruction are iterative. Missing fields/terminators, excessive
budgets and bytes after the selected profile's complete terminator fail without returning
a partial document.
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
files. Five pinned files now span four mandatory source bundles; chemistry calculations
and foreign/general CDX remain separate missing coverage.

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

## SDF mixed V2000/V3000 molecular payloads

`SdfFile::parse` now validates each complete embedded MOL using its actual fourth-line
V2000/V3000 stamp and corresponding full syntax codec. `SdfRecord::mol_version()`
reports that per-record encoding; there is no global SDfile version or extension-based
conversion. BIOVIA Chemistry 2026 printed 97 explicitly permits either encoding or a
mixture. The container still borrows exact source and exposes ordered raw fields,
DT digits, duplicate/empty/multiline values and complete absolute spans. Parsed molecular
syntax is transient: only the version/raw payload and existing field views are retained.
No elemental projection or application multi-record document is constructed.

`sdf::write(&[SdfRecordInput])` constructs new SDF framing while preserving each accepted
MOL payload and its actual version. `write_v2000` remains strictly V2000-only and returns
unsupported_content for a V3000 input; it does not convert or silently widen its contract.
Both share one checked preflight/emission body. Unknown accepted molecular properties,
query/stereo metadata, continued/quoted V3000 data and original headers remain in the raw
payload. New SDF lines use LF; imported MOL CRLF stays exact, with only a missing final LF
appended for framing. All existing prefix-delimiter, no-MOL-padding, header, data-value,
strict EOF, ASCII and output/resource policies remain. Invalid nested syntax/stamps yield
invalid_sdf; nested resource/unsupported errors retain their codes and record context.

A tiny crate-private version-stamp helper is shared with StoredDocument. Moving the
existing detector does not change its validation/errors or replace complete parsing.
SDF retains its 64 MiB/1024-record/65536-field/1M-line/4096-byte structural/200-byte data
budgets plus each molecular codec's limits. There is no new global graph-count policy or
hard RSS claim. Generic output preflights the complete file before allocating its bytes.

The old V2000 fixtures/oracles/assertions are unchanged. The original777-byte mixed
vector has pinned literal spans/order; the supplemental RDKit 2026.09.2 full supplier
checks ordinary unique properties and external SDWriter version switching. Unchanged
molecular oracles check8 dp/1,000-atom payloads. The valid DT-only supplier loss remains a
diagnostic and duplicate dictionaries never establish raw fidelity. Application SDF
selection/editing/persistence, data-only datfiles, broader encodings/header identities
and full chemistry remain missing.

## Standalone legacy RXN container codec

`io::rxn::RxnFile` parses the legacy `$RXN` wrapper and borrows its entire source.
It exposes the three metadata lines, three role counts and ordered components with
absolute whole/payload ranges. Ranges include line endings; component whole ranges
include their `$MOL` marker, while `mol_bytes()` starts at the molecule title.
The wrapper version is V2000. Each component reports its actual V2000 or V3000 MOL
version independently, and the existing complete MOL syntax parser validates that
payload without calling its strict elemental projection. Atom maps, unknown supported
MOL properties and molecule headers therefore stay in the exact original bytes.
`as_bytes()` is unchanged-source passthrough, not a reconstructed reaction.

`write_v2000([title, program, comment], &[ComponentInput { role, mol_bytes }])`
constructs a new wrapper from complete validated molecules. It derives all three I3
numeric counts, requires reactant/product/reagent order, and refuses to sort roles,
convert molecules, strip metadata or trim payload padding. It preserves every
accepted payload byte. Only a missing final LF is appended for framing; new wrapper
lines use LF, so an imported CRLF payload can intentionally produce mixed line endings.
There is no lossy projection-to-reaction writer or application import/save command.

The primary source is BIOVIA Chemistry 2026 CTfile Formats, printed 91–92 (PDF indexes
94–95). Actual external RDKit 2026.09.2 output independently establishes two supported
extensions: a legacy wrapper with V3000/V2000 MOL components, and an exact six-character
counts line omitting zero reagents. A present count must be numeric; blank/partial
fields are refused. The writer always emits the full nine-character counts line.
Consumer tolerance for trailing or uncounted molecules is deliberately not adopted.

The first profile requires full EOF, one complete MOL per declared component, exact
`$MOL` marker records, printable ASCII plus tabs/line endings, LF or CRLF without bare
CR, and no payload padding after M END. Bounds are 64 MiB source/output, 1,024 total
components, 999 components per legacy role, 100,000 aggregate atoms and 100,000 aggregate
bonds independently, 1,000,000 lines and 4,096 bytes per physical line. Existing molecular
limits also apply. These are Bondline resource/subset policies, not universal CTfile
limits or a hard process-memory cap. Invalid framing returns `invalid_rxn`, unsupported
wrapper/content returns `unsupported_content`, and resource limits return `resource_limit`.

V3000 reaction wrappers, reaction-model projection, atom-map authoring, reaction-center
semantics, RDfiles, expanded registry records and application/native reaction workflows
remain missing. Constituent raw preservation does not verify every unknown property's
meaning. Independent raw ranges/maps and separately executed RDKit consumers verify the
original vectors; a round trip solely through Bondline is not the compatibility evidence.

## Header-embedded CDX Document profile

The standalone CDX reader additionally accepts the public22-byte-prefix layout in
which an encoded Document8000 occupies bytes22..28 of the historical header. It
returns Framing::HeaderDocument, header().len()==22 and contiguous records from
that root through a distinct RecordKind::FileEnd. The matched root ObjectEnd and
the separate two-byte FileEnd must both be present in this profile, followed by
exact EOF. All unknown prefix bytes, tags, payloads and length spellings remain
borrowed unchanged; checked opaque property edits preserve both terminators.
HEADER_BYTES stays28 for existing profiles and new Writer output;
HEADER_DOCUMENT_BYTES names the22-byte prefix. Writer's canonical new-tree output
is unchanged and is not a source-backed reconstruction path.

Compatibility dispatch deliberately favors existing behavior. The historical28-byte
profile runs first. Only its TrailingData error, a supported12-byte magic/marker and
Document tag8000 at22 enable one second parse; there is no scan, ignored padding or
retry after other failures. Successful legacy parses remain legacy even if their
opaque reserved bytes resemble an encoded root. In particular, removing the separate
file end can leave a complete legacy tree; bytes alone do not establish intent.
This ambiguity is documented rather than treated as universal native validation.

Both bounded profiles retain the64MiB input/output,100000-record and256-open-encoded-
object limits. HeaderDocument's encoded root counts toward depth/records and FileEnd
counts toward records. Two profile attempts are possible; these logical caps are
not a process-memory guarantee. Extra file ends, nonzero final tags and trailing
garbage are refused. Existing implicit/explicit fixtures and exact-EOF assertions
remain unchanged.

The new original65,746-byte field vector has independent stdlib event/byte checks
and actual external RDKit graph checks; the unmodified ignored1196-byte local
native observation also passes syntax/passthrough and one opaque splice. This is
bounded framing evidence, not general property interpretation, foreign-file corpus
coverage, chemical model mapping or an application CDX Open/Save workflow.

## Owned limited CDX model snapshots

StoredDocument::from_cdx now owns the complete source, initial elemental view and model
baseline. The private preview helper uses one caller-sized page; print/style data remain
opaque. Original framing/metadata/unknown bytes stay authoritative. All CDX imports remain
is_fully_mapped=false. export_cdx/to_cdx borrow exact unchanged bytes and refuse changed
models; source-free CDX encoding and cross-format metadata loss remain unsupported.
Full snapshot equality/storage accounting includes raw source and baseline, including
opaque-only differences and shared Arc clones. See verification-cdx-owned-snapshots.md.
The snapshot layer alone does not provide a file workflow; the guarded commands below
now carry it through the application. Source-faithful rendering, editing, chemistry and
full M1 remain missing.

`files::read_cdx_file` and `save_cdx_file` now reuse bounded ordinary-file reads and the
shared atomic publication/security/cancellation implementation. Read retains a complete
snapshot with an explicit A4 preview policy; Save only borrows unchanged imported bytes.
Both existing framing vectors save byte-exactly and face their independent struct/RDKit
consumers. Invalid input, no-clobber, stale/source-free export, cancellation and seven
publication failure stages have filesystem tests. See verification-cdx-native-io.md.

## Source-only CDX metadata

The strict view now admits Fragment Z/BoundingBox, node Z/CIP/AtomID and bond Z/unstyled
ChemicalWarning/CIP with published exact widths/owners. CIP0/U and1/N remain distinct
raw annotations, not calculated labels; other states refuse. AtomID is opaque and cannot
resolve bond endpoints or replace stable model IDs. Bounds/Z do not infer paper, move
atoms or establish source-faithful style rendering. Warnings require a zero-style prefix;
malformed run lengths fail and fully framed styled warnings remain unsupported.

The SDK FragmentZ owner omission and unknown AtomID normative lifecycle/uniqueness remain
qualified source questions. Full raw bytes/baseline/false completeness and export guards
stay authoritative. New original323/319B vectors have independent struct/RDKit checks;
one ignored native1196B original now projects2C/1Single and exports every byte unchanged.
This metadata acceptance alone does not establish broad native compatibility, chemistry,
a CDX model encoder or an application workflow.
See verification-cdx-native-metadata.md for actual RED/GREEN/ASAN scope.

The application now uses those same functions through document.open_cdx/document.save_cdx,
complete FilePlan/Completed snapshots and the shared per-Engine save reservation. File state
and receipts report `cdx`, independent of extension. UI format selection, known Save,
Save As and retry are wired through the same registry, not a separate encoder.

All CDX imports remain incomplete: no default capture/preview or source-backed bond creation
is authorized. Page edits can make the model dirty, but CDX Save refuses until exact Undo
restores the original source baseline. Source-free CDX encoding and CDX/CDXML/MOL metadata
loss refuse before any temporary is created. Accepted source, metadata and unknown records
are saved byte-for-byte. See [application verification](verification-cdx-application.md).
Bounded Mac native CDX acceptance now passes on source450962c; see the linked exact-build
report. Broader compatibility, edited-source encoding and faithful preview remain missing.


## Immutable whole-SDF ownership

io::sdf::SdfDocument now retains complete validated Arc bytes and compact shared record
metadata; every mixed-version molecule and ordered field/header/value/delimiter stays
exact. Positive IDs are local ordinals inside immutable snapshots/clones, not chemical
IDs or promises across a different source. Read reparses the bounded source; Debug prints
counts only. Shared-allocation accounting is explicit and excludes temporary reads/headers.
See verification-sdf-owned-source.md for literal/raw/RDKit/boundary/fuzz evidence. Complete
SDF ownership now also backs Engine current/saved/history and registered Open/Save; see
verification-sdf-session-files.md. It supplies no active record, chemical layout or data-edit
workflow. Never regenerate retained imports
through the generic wrapper writer or silently select/flatten the first molecule.


The immutable SDF owner can now replace one existing ordered field occurrence, returning
a new validated source. Logical no-op keeps original Arcs/CRLF. Changed lines use LF while
all headers/textual DT IDs/blank terminators/padding/MOL/other records/delimiters remain
exact. Missing targets are typed not_found; original syntax/value/quota rules/codes remain.
This is a library candidate operation, not yet an Engine/native data editor. No record
add/delete/reorder, chemical edit or source reconstruction is delivered. Independent exact
704/788/743B outputs, consumers, boundaries and fuzz are in verification-sdf-field-value.md.
