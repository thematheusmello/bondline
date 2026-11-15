# Independent neutral acyclic SMILES layout acceptance

This is test-first acceptance from `97fd2ac`, not a geometry implementation or native
workflow claim. The existing carbon-path layout, normalization, codec, fixtures and
consumers remain unchanged. The proposed additive API is
`SmilesDocument::layout_neutral_acyclic(length, origin, page, &Cancellation)` returning
an ordinary `Document` or typed `LayoutError`.

The original profile is one connected tree of at most64 atoms, with the existing
closed neutral nonisotopic H/C/N/O/halogen normalization. The ceiling precedes
normalization/search. Physical H stays an H0 node. Origins specify the minimum x/y
of **atom centers** after layout; glyphs and full strokes must separately fit the paper.
There is no automatic scaling, source-backed SMILES Save authority, canonicalization,
stereo assignment, cyclic/disconnected layout or general chemistry claim.

## Independent expectations

The first test declares `CC(C)C` as atoms1–4, literal H `[3,1,3,3]`, directed bonds
1→2,2→3,2→4, neutral metadata and present positions. Expected chemistry never comes
from Bondline normalization. The original1,970-byte declaration is SHA256
`d39579167c95e26a9f4c7a905dd5d1deb0f90d4c9b002786f21d45004b0b5836`.
First actual compile76197 exits101 solely at E0599, missing `layout_neutral_acyclic`.
Its3,850-byte log SHA256 is
`955033db10f7eb1eb37f744d40c7175a5e862a4ecb8fd9fdfee2a37771be2fa3`.
All186 protected source/test/fixture/consumer/lock/golden inputs remain byte-exact.

The expanded eight cases retain that test's assertions, with only owned-file rustfmt
whitespace changes. The ignored receipt preserves the initial source and exact diff.
They cover:

1. The independent four-carbon branch above and complete raw/token/graph sharing.
2. Literal CCO/CN/halogen branches, physical-H nodes, Single/Double/Triple, carbonyls,
   cumulenes, terminal/internal alkenes and degree-four graphs. Fifteen source vectors
   have separately checked pinned RDKit elements/H/formulas before any product output.
3. Degree-three120° angles, all-terminal neopentane90° gaps, and the agreed contextual
   degree-four60°/90°/120° patterns.
4. 180° triple/Single and two-Double centers. The agreed degree-two internal unspecified
   Double uses collinear substituents at both ends; potentially stereogenic mixed/
   degree-three cases refuse. Actual CDXML must not acquire E/Z or atom chirality.
5. Rigid translation/minimum-center origin, deterministic repeats, single-atom position,
   bracket equivalents and graph connectivity across `C1.C1` notation.
6. Whole-input refusals for actual rings, components, aromaticity, isotopes, charges,
   radical ambiguity and unsupported elements; existing syntax refusals stay intact.
7. Literal invalid-length/cancel/resource codes,65-atom rejection before normalization
   (including a leading unsupported P), at-cap bounded outcomes, nonfinite/collapsed/
   out-of-page geometry and all four glyph-clearance edges.64 is a ceiling, not a
   promise that every such graph fits A4/1024px or existing renderer limits.
8. Actual generated CDXML, independent Expat/Decimal/RDKit checks, exact native-model
   reopen/classification and a subsequent coordinate edit/re-encode/reopen. The original
   input/source/IR and previous owned snapshot remain unchanged.

The shared CPU scene is compared against independently rendered blank paper outside
its physical border, including white masks as well as black ink. This is a1024²
reference-scene check, not GPU, all-viewport or native platform proof. Literal H and
metadata are never removed to make a scene fit. Full source signatures and Arc-backed
source/token/bond sharing are checked on success and refusal.

## New external consumer

`python -B crates/io/tests/smiles_acyclic_oracle.py ORIGINAL_SMILES [OPTIONS_JSON]`
reads actual CDXML from stdin. Optional closed keys are `atoms`, `bonds`, `formula`,
`length`, `origin`, `page`; geometry defaults to48pt, minimum centers(72,72), A4.
Explicit literal atom/H/formula vectors must also agree with the independently parsed
original notation. Optional directed bonds are checked exactly; semantic connectivity
is separately checked with the pinned external consumer. JSON output contains `atoms`,
`bonds`, `formula`, `stereo_none`, `physical_h`.

The1MiB input cap and64-atom reference cap belong to this external test tool. Expat
retains raw coordinates as Decimal, validates IDs/references and metadata; separate
checks cover finite coordinates, length with chosen relative tolerance1e-9, center
bounds, nonincident intersections and vertices on unrelated bonds. Independent RDKit
uses sanitization with `removeHs=False` for both source and actual CDXML, checks fixed
H/physical nodes/formula/graph and forbids invented stereo. An independently literal
methanol XML seed succeeds; wrong H, length, actual coordinate, nonfinite coordinate
and bond-order controls each fail. These qualify the new oracle, not Bondline layout.

The existing carbon-only ≤12 layout consumer and fixed-ethane file consumer are
unchanged. Their restrictions are preserved rather than relabeled as broad coverage.
The new tests also call the unchanged general Expat graph consumer.

## Sources and current execution scope

[IUPAC2008 GR4.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR41) supports the
relevant120°/180° and contextual four-coordinate depiction rules; GR4.3 addresses
overlap. These are diagram conventions, not molecular3D measurements or vendor defaults.
The [pinned OpenSMILES source](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc)
and the existing normalization record retain the closed H policy. No upstream source,
template or asset is transplanted. The public
[RDKit CDXML API](https://www.rdkit.org/docs/source/rdkit.Chem.rdmolfiles.html#rdkit.Chem.rdmolfiles.MolsFromCDXML)
documents the separate H-removal control; the inspected web documentation identifies
2026.09.1, while actual external executions assert installed2026.09.2.

At declaration, only the missing-API RED and external-oracle/vector qualification have
run. No product layout GREEN, native UI, GPU, hosted or Windows/Linux execution is
claimed. All future receiving results must bind the exact frozen inputs and source.
No dependency, lock, existing fixture/oracle/golden, Git or native input was changed.

## Qualified unknown-Double consumer correction

The producer's supplied run on the corrected renderer passes7/8 cases, then stops at
the new oracle's blanket `STEREONONE` assertion for actual `CC=CC`. The original assertion
message, "no invented E/Z", is misleading here: independently reading that exact713-byte
CDXML (SHA256 `2361037ba0a8a242692ff2e0fbcc5abb4ffe58ea8a9d0ade258f3cba370f8199`)
returns `[STEREONONE,STEREOANY,STEREONONE]`, canonical `CC=CC`, unassigned atom tags.
Exact horizontal, vertical, diagonal and reversed variants reproduce that result.
Noncollinear actual-file controls instead produce assigned E/Z configurations.

This is an independently established **oracle contract error**, not a demonstrated
Bondline E/Z assignment bug. The original25,016-byte/bccc Rust declaration,
7,576-byte/c745 oracle and producer7/8 log (2,190bytes/SHA256
`c2954cbdaa8f0e817d920c7ae7bca23312557184febde46a7569cc412bacb39b`) are preserved.

The pinned public [RDKit Bond.h enum](https://github.com/rdkit/rdkit/blob/f86b7dc9b31e2531200bbd1f54589bcbd968bb05/Code/GraphMol/Bond.h#L89-L99)
distinguishes intentionally unspecified ANY from assigned configurations. Its
[linear-coordinate handling](https://github.com/rdkit/rdkit/blob/f86b7dc9b31e2531200bbd1f54589bcbd968bb05/Code/GraphMol/Chirality.cpp#L197-L238)
sets ANY for a linear arrangement. This agrees with the unknown-configurational
collinear convention in the IUPAC GR4.1.1 source above. These public source pins are
separate evidence from the installed2026.09.2 wheel; its exact build commit is not inferred.

The reviewed correction permits NONE/ANY only for the source-unassigned Double with
degree-two endpoints and one other Single on each end. Both actual raw Decimal
substituent vectors must be antiparallel to the Double, even when the consumer says NONE.
The prechosen length-relative1e-9 tolerance remains; RDKit's broader angular tolerance
does not replace it. Every other bond still requires NONE, and all existing atom-chiral,
graph, H, formula and canonical checks remain. No assigned E/Z/cis/trans/atrop is accepted.

`stereo_none` now truthfully reports whether every actual enum is NONE. Added fields
`no_assigned_stereo` and `bond_stereo` report the validated condition and actual strings.
Only the new Rust output assertions change: `CC=CC` expects false and
`[STEREONONE,STEREOANY,STEREONONE]`; every other declared example still expects true/all
NONE. The exact assertion-block inverse reconstructs the original bccc source.

Qualification21061 passes16 actual-file controls: the prior positive/seven negative
controls remain, supplied/rotated collinear files pass, real assigned E/Z files refuse,
and a0.5° noncollinear file refuses although RDKit calls it ANY. Ten **separate explicit
in-memory bond/geometry guard controls** reject all six assigned bond enums, wrong-context
ANY and noncollinear NONE; they are not described as actual-file evidence. The initial
read-only orientation probe encountered ElementTree's generated ASCII XML-declaration
setup refusal; changing only that in-memory serialization to no declaration allowed the
independent variants. No source file, coordinate or product was changed for that setup.

Ignored `target/smiles-acyclic-layout/stereo-oracle-v3/` retains original declarations/logs,
exact deltas/inverses, primary source hashes, real-file controls and separately labeled
in-memory checks. No backend/API GREEN or native workflow follows from oracle qualification.
