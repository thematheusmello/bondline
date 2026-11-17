# Source-preserving Bondline-v0 CDXML profile

2026-11-17; implementation contract following batch primitive `1176039`. Root reviewed
and adopted the independently prepared policy below. It supersedes the earlier looser
architecture/reference/test-design suggestions. This profile is **not implemented**;
the original 464-byte CLI edit still refuses. Full M1–M5 remains required.

The source-only draft was 10,420B/SHA256
`a85ca8ccc2a3a38c5b5cdbb964b6b677ec42c6e1a127d29fa98dad4e616c2fd0`;
its input pins remain under ignored `target/noncanonical-cdxml/io-profile-lock/`.
[XML1.0 §3.1/3.3.3](https://www.w3.org/TR/2008/REC-xml-20081126/#sec-starttags),
[CDXML lexical rules](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDXML.htm) and
[containment/order](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/General.htm) support
lexical equivalence. The grammar, authority, ID allocation and loss boundaries below
are original Bondline policies, not authenticated provenance or reference behavior.

## Three distinct source states

Use a private CDXML classification: Canonical, Preserved, Opaque. Canonical remains
the current every-byte-native-writer equality, including the pinned v0 path/output.
Preserved gains native edit/display completeness ONLY with the proof and complete
update/append serializer below. Opaque keeps exact unchanged Open/Save and existing
refusals. This is not foreign-style support or recognition by producer name alone.

`OriginalSource::Cdxml` keeps original bytes and original model baseline as today.
A small profile enum can replace the CDXML bool; MOL's own bool stays separate.
No persistent mapping arrays are necessary: reconstruct scoped source↔baseline
bindings using the original import traversal at classification/Save. Do not own
borrowed roxmltree Nodes across snapshots. Temporary mapping allocations are bounded;
if persistent heap ownership is later added it needs accounting, not a hidden cache.

## Positive input proof (all conditions, after unchanged syntax/projection)

1. Root is unnamespaced CDXML and its XML-decoded CreationProgram equals the current
   native writer's value. This is a conservative grammar restriction, not authenticated
   provenance. All semantic element names/attribute-name sets match a canonical encoding
   of the imported baseline, after the following specifically permitted differences.
2. Exactly one direct page: explicit DrawingSpace=Poster, Width and Height in the
   existing model domain; direct fragments; leaf n/b. Fragment order, atom encounter
   order and bond encounter order define the existing dense model correspondence.
   Original n/b interleaving and forward references are allowed and retained.
3. Page/fragment/n/b IDs are assigned positive parsed ObjectIds. Existing scoped
   uniqueness and same-fragment B/E resolution remain; reference direction is exact.
   Arbitrary ID magnitudes, sparse IDs, integral lexical forms and cross-scope repeated
   IDs do not become application IDs. No new general nearest-ancestor resolver.
4. Quotes, attribute order, XML whitespace/CRLF, ordinary comments and equivalent
   typed numeric spellings may differ. Keep original bytes. BOM/declaration remain
   allowed only as accepted by the current parser. No DTD, PI, namespace declaration,
   namespaced element/attribute, group, extra object/attribute, Text/style/table/bounds,
   non-whitespace character data or other unmodelled semantic content qualifies.
   For an initially strict trivia grammar, CDATA stays opaque; Root may explicitly
   lock whitespace-only CDATA later. Leaf n/b may have trivia, never semantic children.
5. Missing optional/default attributes qualify only when the native baseline emits
   the SAME keys. Thus absent Charge/Radical/Element/Order and explicit Isotope=0 do
   not become positives merely because the mapper supplies an equivalent default.
   Both flag tri-states, missing H/p, dormant DoublePosition and supported root settings
   retain the existing model meanings. Invalid root settings projected to None cannot
   qualify: their extra raw keys differ from native output. No inference or new epsilon.

The classifier must enumerate EVERY element/attribute and prolog/trivia node, not
only graph.rs's known-object filter. Namespace declarations are not ordinary roxmltree
attributes. Also check fixed enum/provenance values: matching keys alone cannot make
DrawingSpace=Pages with fallback dimensions equivalent to explicit Poster. Validate
all referenced endpoints through actual scoped bindings. Failure of this additional
proof keeps earlier successful read-only Open intact; it does not change parser policy.
A baseline which cannot be natively represented within old bounds stays Opaque.

The above explicitly supersedes older proposals allowing arbitrary CreationProgram,
missing/zero IDs, PIs/DTD and persistent generated-ID bindings. Their old projection/
source retention remains; they are not new editable positives in this first profile.

## Current model and complete writing contract

For Preserved, validate that current page identity is unchanged and singular;
existing fragment identities/order, each fragment's atom/bond identity/order and
membership remain prefixes. Original bond directed endpoints remain exact. Existing
modeled scalar fields may change; new fragments/atoms/bonds append. Reject removals,
reorder, reparenting, ID replacement or unsupported raw model drift. Every current
native command fits this grammar: atom label/flags, bond order/H/create/extend,
selection translation, rings/chairs/named/sprout, chains/fixed paths, paste, page size
and drawing settings. New objects must also fit the existing native encoder domains.

A shared cheap typed-delta validator should qualify Preserved current edit/display
eligibility AND export planning. The cached original profile alone cannot authorize
an unsupported raw model_mut change. Do not serialize or run filesystem I/O in a
capability getter. Keep fast baseline Arc/equality paths; keep Canonical semantics
unchanged. Full writer validation remains authoritative before publishing bytes.

On unchanged baseline, return original Cow::Borrowed bytes, preserving lexical no-op
and Undo restoration. On supported changes:

- Rebuild original bindings by page/fragment scope and baseline array encounters;
  source ID is neither a model ID nor a globally unique lookup key.
- Compare baseline/current TYPED fields. Unchanged values leave source lexemes alone:
  e.g. changing Order/H must not rewrite an old p, ID or equal numeric spelling.
  Changed modeled attributes use batch Set/Remove, including optional settings/flag
  insertion/removal. Input key-presence qualification does not forbid later supported
  optional field edits. Existing B/E/ID spellings are never rewritten.
- Aggregate one AppendChildren per existing container, coalescing appended atoms/bonds
  and new fragments as needed; duplicate append ownership conflicts in the primitive.
  Empty paired/self-closing parents use its tested expansion/composition behavior.
  Preserve every original byte outside approved value/token/insertion ranges and retain
  all old object order. Never route Preserved through whole write_document(current).
- Generate new-only subtrees through shared existing native attribute helpers and the
  existing bounded syntax Writer. Helpers take the chosen XML IDs/B/E. A private helper
  can use a temporary synthetic root and validated child ranges; do not strip guessed
  wrapper strings or duplicate escaping. Keep canonical field order/output exact.
- Reserve EVERY original assigned ID, including IDs repeated in separate scopes. Choose
  new IDs deterministically from unused positive values, e.g. lowest-free traversal.
  A max-ID+1 allocator is insufficient when a legal original ID is u32::MAX despite
  abundant free IDs. Search work is bounded by original+generated object counts.
- Allocation is PER SNAPSHOT. Appending to an earlier fragment can renumber generated-only
  XML IDs on a later Save. Original source IDs never change; stronger stability for new
  IDs requires stored state and is explicitly future work, not silently promised.
  Engine model identity floors/history retain their separate existing semantics.

Reparse final syntax and verify correspondence/fields/arrays/directions against the
candidate using explicit rebinding, not naive dense-model ID equality. This internal
certificate does not replace independent raw Expat/Decimal/chemical expectations.
Keep batch/output/node/attribute/depth/ID bounds and checked arithmetic. Unsupported
model delta is UnsupportedEdit; preserve existing typed Mapping/Syntax resource errors
rather than normalizing malformed output or falling back to canonical bytes.

## IO and caller integration / loss boundary

`document.rs:171–187` classifies Canonical first, then Preserved proof, else Opaque.
`to_cdxml:404–422` keeps exact-original passthrough; changed Canonical uses the old
writer, changed Preserved uses only its source-preserving writer, Opaque refuses.
`export_cdxml` remains borrowed unchanged-only. Save never overwrites original baseline
to clear source.modified. Current/saved/history and FilePlan lease/finish rules remain.
Enum/storage equality and size_of accounting remain accurate; no new heap/RSS claim.

The new true completeness affects commands atom/bond/selection/ring/chain/paste/settings,
controls, view/capture, IO display and native eligibility. Keep their independent source,
H/metadata, geometry/ink and physical/context gates. All update/append writing must land
before broadening this permission. Do not add a new registry command/framework.

`to_mol_as:517–532` currently shares the CDXML bool: split that decision explicitly.
Whole-MOL output stays SOURCE-FREE or CANONICAL-only (plus existing MOL backing rules),
with every current flag/style/layout/settings loss guard. Preserved comments/order/
lexemes are not a whole-format conversion license; same-model equality is insufficient.
CDX/SMILES backing/cross-format guards stay. Explicit selected-fragment SMILES export
is already graph-only/geometry-losing; document its existing loss contract separately
when new complete CDXML becomes eligible, retaining chemistry/refusal/publication rules.
It must not turn into whole-session Save or be mistaken for lexical source preservation.

Smallest reuse: one private profile/delta helper and Preserved writer in IO, existing
ElementalGraph traversal and native attribute builders, existing apply_edits plus syntax
Writer. No new mutable engine/source owner, parser policy, dependency, registry/schema,
renderer geometry or general XML/source style system. The pinned native-v0 fixture and
old whole-writer output remain protected. User apps/Actions remain untouched.

This is an initial Bondline-v0 elemental compatibility profile, not general foreign
CDXML/style/inheritance support, proof of26.1 behavior, or completion of M1–M5.

## Implementation and independent receiving order

1. Declare existing-public-API tests before producer changes: the original 464-byte
   order/H-only vector; page/settings/label/position/optional-field edits; every current
   append family and both empty container forms; scoped/sparse/u32::MAX IDs and burned
   model IDs; unknown syntax/metadata/semantic-node refusals; source drift and whole-MOL
   loss boundaries. Assert original and actual saved bytes with independent literal
   Expat/Decimal/RDKit consumers, not just our own model round trip. Observe actual RED.
2. The classifier needs lexical evidence for DTD, PI, CDATA and every namespace
   declaration, including redundant `xmlns:xml`. roxmltree's ordinary attributes omit
   declarations and its text nodes lose the CDATA distinction. Collect these four pure
   syntax facts during the existing bounded cdxml preflight (without changing admission),
   or use an equally complete existing-token path. Do not guess from substring searches,
   ordinary attribute counts or namespace bindings alone. Test comments/quoted values,
   empty/reset/prefix declarations and prolog/trivia controls independently.
3. Complete the private IO profile/delta validator and writer, factoring existing new-
   object attribute helpers without changing canonical output. Only then wire the
   private profile enum, current eligibility and preserved Save, with canonical-only MOL.
   Keep exact source passthrough, source.modified, history and publisher lease semantics.
4. Verify registered edits and actual Unicode files through engine/CLI/stdio/loopback;
   test current model/frame eligibility and full-scene refusal. Then use a separate own
   native app for edit/Undo/Redo/Save/reopen and inspect actual files independently.
   Unavailable native execution remains unverified; headless passes do not complete it.
5. Finish relevant parser/edit fuzzing, warnings-denied/full local gates and scoped
   parity/evidence records in the same atomic format-feature commit. Actions stays OFF.
