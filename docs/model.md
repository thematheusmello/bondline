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

Document::append_fragments validates a whole batch before installing any state. It rejects
missing pages and duplicate fragment/atom/bond IDs, including collisions with existing
fragments or within the batch. The same explicit budget caps atoms, bonds and fragments
at 100000 each per document; an individual fragment also checks atom/bond caps before
identity/topology work. Batch validation uses ordered sets, avoiding quadratic pair scans.

Fragments expose immutable slices and cannot be deserialized without their validation
constructor. Atoms and scalar identities have closed validated deserialization. Atom and
Bond values may be assembled before Fragment construction; public values cannot mutate
the fragment's private vectors after insertion. Page/document cloning shares fragment
Arcs, so page-size edits do not duplicate molecule arrays. History still needs chemical
command coverage when those commands land.

CLI/MCP/loopback authored snapshots now include each page's fragments array, empty in the
current default application. These interfaces still have only page/history/capture commands;
no chemical import/editing UI or chemical renderer is connected. This model is not a native
project format, complete CDXML mapping or a working chemical authoring workflow.

Io's ElementalGraph projects supported CDXML properties into these types and retains the
source separately. StoredDocument now owns both the model and original CDXML allocation;
Engine history retains the whole snapshot, including opaque data and the export baseline.
Copying fragments into a bare model Document still does not copy that backing. Source-backed
undo/redo has unit and generated-sequence evidence; imported chemical editing, an editable
writer and native open/save remain missing. The model itself stays format independent.
