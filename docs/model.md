# Authored chemical graph

The model is GUI/format/I/O independent. Pages now own Arc<Fragment> values. Fragment
construction validates atom and bond identities, existing endpoint references, distinct
endpoints and one bond object per unordered pair. Cycles, empty fragments and disconnected
components are accepted. These are structural checks, not chemical validity/perception.
The single-bond-object-per-pair restriction is an explicit current model policy.

AtomId, BondId and FragmentId are distinct nonzero-u32 types. IDs are stable and unique
document-wide within each kind. The three namespaces may overlap numerically. Source file
IDs have separate scoping/zero rules and must be mapped rather than reused blindly.

An Atom stores a validated 1..118 Element, optional Point2, optional positive isotope mass
number, signed i8 formal charge, a radical state and an optional u16 explicit-hydrogen count.
Isotope None means unspecified/natural abundance. Explicit hydrogen None stays distinct
from Some(0); valence-derived hydrogen generation is not implemented. Radical states are
none, singlet, doublet and triplet. No isotope-existence, valence or stereo inference occurs.
The element range is the reviewed IUPAC catalog, not a claim that future elements cannot exist.

BondOrder stores the sixteen single-term kinds in the archived file specification: single
through sextuple, half/aromatic/two-and-half through five-and-half, dative, ionic, hydrogen
and three-center. Begin/end direction is retained, including for dative bonds. Query
alternatives, non-element nodes, stereo/display styles and reactions remain future work.
Storing an authored aromatic kind does not establish an aromaticity-perception algorithm.

Bond also retains DoublePosition: Unspecified, Center, Right or Left, with side relative
to authored begin→end. JSON defaults to Unspecified and omits that default; old JSON
stays exact. Specified values remain data even on non-Double orders. No automatic side,
CDX manual/automatic flag or chemical rule is inferred. Whole-bond copies/order edits
preserve the field; size_of::<Bond>() already covers its fixed-size storage. The
[data/format prerequisite](research/sided-double-prerequisite.md) and
[independent evidence](verification-double-position.md) distinguish persistence from
the [inset renderer](verification-inset-doubles.md) from still-missing automatic styles,
skeletal labels and alignment/tool workflows.

Document::append_fragments validates a whole batch before installing any state. It rejects
missing pages and duplicate fragment/atom/bond IDs, including collisions with existing
fragments or within the batch. The same explicit budget caps atoms, bonds and fragments
at 100000 each per document; an individual fragment also checks atom/bond caps before
identity/topology work. Batch validation uses ordered sets, avoiding quadratic pair scans.

Document::replace_fragment uses that same global validator, excluding the existing
same-ID fragment and including its candidate before mutation. Missing page/fragment,
identity collisions and limits fail atomically. It preserves fragment order/IDs and
unrelated Arc payloads; no unvalidated mutable atom/bond slices are exposed. The first
bond.extend operation deliberately replaces target H3 with H2 and adds a new C/H3 and
Single bond under a required closed policy. This is no general hydrogen inference.


Fragments expose immutable slices and cannot be deserialized without their validation
constructor. Atoms and scalar identities have closed validated deserialization. Atom and
Bond values may be assembled before Fragment construction; public values cannot mutate
the fragment's private vectors after insertion. Page/document cloning shares fragment
Arcs, so page-size edits do not duplicate molecule arrays. History still needs chemical
command coverage when those commands land.

CLI/MCP/loopback snapshots include page fragments, empty in a new session. bond.create now
appends one independent C-C fragment, explicitly authoring three H per carbon from the
documented ethane case. It does not run general valence inference or attach to existing
atoms. Native drag creation, history and limited labeled rendering use those same values.
Identity floors remain in the engine session across Undo branches, updating only on success.
This model is not a native project format or complete CDXML mapping. General chemical
editing and a native project format remain missing; closed native attachment is wired
and awaits actual native verification;
the closed headless extension and supported interchange persistence are documented separately.

Io's ElementalGraph projects supported CDXML properties into these types and retains the
source separately. StoredDocument now owns both the model and original CDXML allocation;
Engine history retains the whole snapshot, including opaque data and the export baseline.
Copying fragments into a bare model Document still does not copy that backing. Source-backed
undo/redo has unit and generated-sequence evidence. The elemental writer and guarded native
file adapters now exist; broader imported chemical editing remains missing. The model
itself stays format independent.

`Document::retained_bytes` accounts for inline models/pages, reserved page/fragment-handle
vectors and each shared fragment's reserved atom/bond vectors once. Io's corresponding
StoredDocument function also accounts for distinct XML allocations and export baselines.
Engine commits use that approximate total to bound retained history. Allocation identities
are used only during local accounting and never serialized. Arc/allocator headers, caches,
temporary work and external copies are excluded; this is not a whole-process memory bound.
When a new authored type gains heap-owned data, extend the accounting and its allocation/
sharing tests in the same commit. Undo/Redo preserve complete source-backed snapshots.
