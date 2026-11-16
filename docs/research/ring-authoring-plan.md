# Next M2 increment: native neutral Single-ring construction

2026-11-16. Source/design proposal only, following selection feature07aa562 and its1361/0/1 local gate. Actual native selection acceptance is still pending while the desktop is in use. No extra preview is opened. No new command,
ring tool, test, native flow or parity result is implemented or executed here. Complete
M0–M5, aromatic/chair/chain/style and broader-source authoring remain the target.

## Recommendation

Deliver **one original ring.create operation for regular3–8 carbon Single rings**, with
standalone, shared-atom and shared-Single-bond placements, plus an actual native ring
palette/gesture path. Include attachments in this increment: a detached polygon alone
would leave the guide's ordinary ring-building workflow incomplete. Do not route native
construction through a SMILES converter, temporary file or pasted synthetic source.

The first bounded family deliberately excludes aromatic/cyclopentadiene/chair/sprout
variants, heteroatom ring creation, larger/reentrant rings, automatic layout/merge, stereo,
style preferences and hotkeys. These are named next requirements, not invalid chemistry
or completed ring parity. Benzene is the immediate next ring-tool extension; existing
Kekule paste/order editing does not substitute for its documented dedicated tool.

## Documented evidence versus original policy

- **GUIDE25 printed101:** standard3–8 ring tools; drag changes orientation; existing atom
  or bond is a shared attachment target. Printed101–102 distinguishes spiro, Alt-sprout
  and the configurable sprout preference, plus two chair orientations. Printed103 and
  tutorial423–424 cover aromatic/resonance variants, Shift behavior and ring fusion.
  This is version25 role/flow evidence, not measured26.1 default geometry, Mac modifier,
  screen, snapping, drag-length or precise cursor behavior.
- **IUPAC2008 GR3.3.1/3.4.2:** isolated small rings use regular polygons; six preferentially
  has a vertical left edge, other3–8 sizes a top/bottom horizontal edge. GR3.3.3 supports
  ordinary fused polygons abutting a shared edge with their interiors on opposite sides.
  GR4.2.4 supports collinear opposing angle bisectors at a spiro union. More complex
  fusions may require distortion; this increment must not claim a general ring-system solver.
- **Original:** typed placements, ID order, literal-H policy, tool lifecycle, orientation
  tie, click length, permitted attachment target subset and all numeric limits. No public
  source establishes a48pt reference default or the proposed parameter vocabulary.

## Proposed closed registry and model contract — Root must lock before tests

Add ring.create with required expected_revision, page_id, size(3..8),
hydrogen_policy:'rectify_neutral_carbon', and a closed tagged placement object:

- free: begin_x_pt,begin_y_pt,end_x_pt,end_y_pt,side;
- atom: fragment_id,atom_id,end_x_pt,end_y_pt,side;
- bond: fragment_id,bond_id,side.

All supplied fields are required for their variant; no surplus/null coercion. IDs are
positive; side is positive/negative cross-product half-plane in the existing y-down
coordinates. The first edge is A→B: explicit free endpoints, existing atom→explicit new
endpoint, or the existing bond's authored begin→end. Construct the remaining vertices
by equal signed exterior turns2π/n. Keep exact L=|B−A|, original endpoints/geometry and
paper size; no normalization, rescale or hidden relocation. Reject nonfinite/coincident/
rounded-collapsed/out-of-Point2 geometry and unintended exact occupied new endpoints.
No automatic atom fusion occurs outside the declared shared target. Tolerances belong
in separately frozen independent tests, not magic producer matching.

Use source-free or completely mapped native CDXML only; require full source classification,
not merely a rendered graph/is_fully_mapped from another format. Preserve the complete
StoredDocument owner and all unrelated source/model data. For attachments require the
shared carbon(s) neutral, natural/nonradical, positioned, every incident order Single/
Double/Triple, and literal H exactly4−sum(incident weights); reject unknown/inconsistent
or insufficient H. Do not infer H, alter neighbors or silently generalize to other elements.

- Free: n fresh C/n fresh Single bonds, each new C has H2, one new Fragment.
- Atom: keep the shared atom/Fragment, append n−1 C/n bonds; shared H decreases2.
- Bond: preserve the existing Single bond/ID/direction and both endpoint positions;
  append n−2 C/n−1 bonds; endpoint H decreases1 each.
- Append new IDs in perimeter order, closure last; preserve every old array prefix/ID/
  directed bond and unrelated Arc. Reuse checked document-global maxima plus Engine's
  identity floors, and update floors only after successful one-commit publication.
- Reuse existing global committed-close and stale-revision precedence, model bounds,
  immutable candidate/Fragment validation, source-aware encoding, selection-epoch
  preflight and whole history. A changed operation is exactly one Undo step; Undo does
  not spend/recycle old namespaces. Failure changes no source, selection, floor, history,
  dirty/baseline, close or file state. No Save lease/ACK or worker is invented for this
  bounded synchronous authoring command. Native Save still uses the existing real owner.

## Native completion

Add ui.ring_tool with a closed size and six real central controls, using the existing
mutually exclusive effective tool/checked epoch (Selection/Bond/Ring), remembered bond
order and no independent competing active flag. Selection targets clear atomically when
leaving selection mode. Tool selection itself changes no authored state/history/file.
Existing controls remain authoritative; procedural polygon icons are original assets.

Use actual fresh GraphScene Atom/Bond IDs and PageView inverse geometry. Empty paper,
atom and bond choose the corresponding typed placement, never a new-fragment fallback.
Click uses an explicitly adopted original length/orientation (48pt is a possible policy,
not a reference fact); drag shows actual orientation/side before release. For a bond,
the cursor half-plane chooses side; a target already in a ring should initially propose
its outside face. Atom defaults should oppose the existing bond-angle bisector where
it is defined, including the source-supported spiro arrangement; do not move old atoms.
Ambiguous orientation/ties need an explicit stable policy before native tests, not a
claim that the guide specifies a unique answer. Unsupported modifiers report their gap.

A disposable preview must use the same candidate builder and ordinary chemical renderer,
including literal labels, masks, physical-DPI transform, logical hits and all existing
readability/pixel/resource guards; no line-only fake successful preview. Cancel/Escape/
focus/source/view/tool/file/close/canvas/DPI changes discard only preview. Capture weak
whole-source and existing epochs/opaque file proof in the native gesture owner; do not
reuse selection_context while Ring is the active mode. Ordered release invokes the one
registry command after fresh validation. Same-packet history/tool/Save and source-frame
fences stay intact. Current Single/golden/selection paths must retain their behavior.

## Decisive independent acceptance and first unexecuted RED

First freeze existing-API JSON ring.create for a six-ring, then run it on the unchanged
baseline: intended UnknownCommand, not a future Rust-type compiler error. An actual-App
missing Cyclohexane control is a separate runtime RED before native implementation.

1. Free explicit A=(120,168),B=(120,120), positive side,L48: literal vertices A,B,
   (120+24√3,96),(120+48√3,120),(120+48√3,168),(120+24√3,192).
   Require six exact IDs/edges/closure/H2, C6H12, equal edge lengths/120° interior angles,
   vertical left edge, independent raw/Decimal XML plus pinned RDKit consumption of
   ACTUAL Unicode Save/reopen. Other3–8 sizes/rotations need analytic n-gon controls.
2. Share a cyclohexane bond on its outside: decalin10C/11bonds, shared H1/H1, others H2,
   C10H18. Share one ring atom on the opposing bisector: spiro11C/12bonds, shared H0,
   others H2, C11H20. Preserve old edge direction, coordinates and unselected fragment.
   A terminal-ethane shared-atom case yields methylcyclohexane C7H14, not a detached ring.
3. One commit/Undo/Redo/new-ID-floor branch, current/saved/source distinctions, clean
   native-CDXML reopening/editing and real CLI/initialized stdio/authenticated socket.
   Test closed schemas, stale IDs/revision, unsupported complete owners/targets/H,
   occupied/degenerate/nonfinite/bounded geometry and atomic identity/selection overflow.
4. Actual-App empty/atom/bond gestures and preview cancellation, same-packet tool/history/
   Save, old-surface/source/view/DPI invalidation, precise hits and unchanged originals.
   Use independently authored literal-page CPU/GPU/analytic controls, not a regenerated
   golden or same-producer round trip. Then mandatory actual native palette→draw→attach→
   Undo/Redo→Unicode Save/reopen/consumer checks; other platforms remain unverified.

Current reuse: commands/bond.rs create/extend/set_order/Ids, lib.rs commit and identity
application, model replace_fragment/append_fragments, complete IO StoredDocument,
render GraphScene hits, UI DrawingState/HoverContext/raw ordered loop and central controls.
The converter's ring.rs is source-driven layout with private solver/IR policy, not a
native creation API. Reuse original polygon mathematics/certification, not a partial
SmilesDocument, toolkit or copied template. No new crate/runtime dependency is justified.

## Pins, limits and handoff

The public artifacts below pin this durable proposal; disposable target reports are not required to continue. Root separately reads the guide and IUPAC passages. The original10248-byte recommendation has SHA256`068e7ea0cdcc21efa8c1d62fa81f16a91e505fb65bde560abb2eda514a84cf22`.

| Source | Exact pin | Reviewed sections |
|---|---|---|
| [GUIDE25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf) | 17721447B; SHA256`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3` | printed101–103: Drawing Rings, spiro/sprout/chairs/aromatic/resonance; printed423–424: ring fusion tutorial; printed82–83: ring modifiers (version/platform qualification retained) |
| [IUPAC2008](https://iupac.qmul.ac.uk/drawing/drawing.html) | 347564B; SHA256`32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e` | GR3.3.1–3.3.3 small/large/fused rings; GR3.4.2 orientation; GR4.2.4 spiro unions; GR4.3 overlap; GR3.5 alternating forms (follow-on) |
| [OpenSMILES1.0](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc) | 86816B; SHA256`00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21` | organic-subset H/normal-valence definitions,lines278–310; ring closure semantics; not a drawing-tool contract |
| [chem0.9.4 layout](https://github.com/chiral-data/rust-chem/blob/b28f64f46c0aa914599b02737a7db0f51bea3558/src/core/layout.rs) | 51900B; SHA256`addd1d13d9c58b047e98cae0d187192a31601e68fd86ebe073984b6963857e95` | lines294–327 regular-ring/fusion stage, feasibility only |
| [COSMolKit polygon primitive](https://github.com/cosmol-studio/COSMolKit/blob/265e2e2bbdfc481546ef09d1bb26499df9ea7d4c/crates/cosmolkit-depict/src/geometry.rs) | 55733B; SHA256`f008e68ab17a513f6be6eb9edfe8f9a7dea1cc7c507760021e4158de84be1f64` | lines404–428 embed_ring only |
| [chem MIT license](https://github.com/chiral-data/rust-chem/blob/b28f64f46c0aa914599b02737a7db0f51bea3558/LICENSE) | 1079B; SHA256`a9410a68876e201c472443378e4f104a6fb2049f4aa840b7aa402f3e5d29a2ae` | license notice |
| [COSMolKit root MIT license](https://github.com/cosmol-studio/COSMolKit/blob/265e2e2bbdfc481546ef09d1bb26499df9ea7d4c/LICENSE) | 1079B; SHA256`318ada8a479dc3483285c569ab09a4a7044b7c324dedef241625495ecdb1971e` | root license notice only |

 All evidence was read from existing public-document/permissive-source caches;
no new network fetch, proprietary app/binary/asset inspection, Cargo/test/Git/CUA action,
installation or product edit occurred. COSMolKit's inspected polygon function includes
RDKit-derived commentary; no code/template/dependency/asset is adopted or license closure
claimed. The old source-register rows remain untouched. This proposal does not complete
ring/chain/style/transform authoring, M1 interoperability, M3 chemistry or any milestone.

## Locked first command contract, 2026-11-16

The first implementation checkpoint registers only `ring.create` (36→37); the native
tool/palette/gesture is the following required integration, not an omitted final scope.
No native click/default/modifier policy is inferred by implementing this command first.

The closed root requires `expected_revision:u64`, positive `page_id:u32`, `size:u8`
in3..8, `hydrogen_policy:"rectify_neutral_carbon"` and `placement`. Placement uses
the closed `mode` tag and exactly these required fields:

```json
{"mode":"free","begin_x_pt":120,"begin_y_pt":168,"end_x_pt":120,"end_y_pt":120,"side":"positive"}
{"mode":"atom","fragment_id":1,"atom_id":1,"end_x_pt":72,"end_y_pt":168,"side":"negative"}
{"mode":"bond","fragment_id":1,"bond_id":1,"side":"negative"}
```

`side` has exactly positive/negative. Positive means positive algebraic cross product
from A→B to the polygon interior in the existing right/down point system; there is no
implicit clockwise/left/right reinterpretation. The literal first two vertices remain
exact. Remaining vertices use equal signed exterior turns; closure reuses the original
first atom ID/position, never a numerically recomputed extra atom.

Exact occupancy means Point2 equality with any existing positioned atom on the same
page for each newly introduced vertex. Shared declared anchors alone are exempt.
Other pages/old coincident data are untouched. Every polygon vertex must be distinct;
all new edges must remain finite/nonzero. This is no general overlap/face solver or
proximity-based fusion. Positions need not fit the paper, but remain within the existing
finite±100000pt bound; rendering retains its separate legibility/paper guards.

Global committed-close refusal precedes execution; current authored revision is checked
before source/target/geometry validation. Missing page/fragment/atom/bond is `not_found`;
malformed schema/IDs/size/geometry/exact occupied endpoint is `invalid_params`;
unsupported source, shared bond order, target metadata, absent position or literal H is
`unsupported_content`. Use existing revision/identity/model-budget/selection-overflow
codes and atomicity. Shared atoms preserve all metadata except the expressly consumed
literal H. Old atom/bond array prefixes and directed edges remain exact. Each added atom
is neutral natural nonradical carbon with literal H2. New IDs follow perimeter order;
new edges follow that perimeter with closure last. A preserved shared edge is not added
again. No new Fragment ID is consumed for an attachment.

Independent ordinary-scale analytic geometry uses≤1e-9pt coordinate/length tolerance
and≤1e-10 radians for interior angles, fixed before implementation; exact IDs/metadata/
shared coordinates/source bytes receive no tolerance. General cross-hardware bitwise
floating-point identity is not promised. First existing-JSON and actual CLI tests must
observe runtime UnknownCommand before code; closed schemas, all three placements,
chemistry/files/history/identity/error/selection/complete-source checks follow. Typed
NaN/Inf and private exhausted counters need explicit tests. Do not replace old oracles,
fixtures, fonts, dependencies, approved goldens or the unresolved MCP shutdown test.
