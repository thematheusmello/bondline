# Neutral SMILES components: next M1 contract

2026-11-15, reviewed after checkpoint691af6f and the reported1172/0/1 local gate.
This is a design/source review, not an implemented feature or executed acceptance test.
It extends the existing create-only converter toward general source-qualified drawing
import; full M0–M5 and the unsupported chemistry/import work below remain required.

## Sources and boundaries

- [OpenSMILES1.0,2016-05-15](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc), sections Hydrogens, Disconnected Structures and Other Uses of Ring Numbers and Dot Bond: dots omit an adjacency, but do not determine connected components. Its `C1.C1` example is ethane. Discover components from actual edges; never split text or use `chain_starts`. Preserve implicit/bracket H distinctions and physical `[H]` nodes under the existing closed normalization policy.
- [IUPAC2008,GR7.2–7.3](https://iupac.qmul.ac.uk/drawing/drawing.html), Brecher, *Pure Appl. Chem.*80,277–410: salts/related forms have contextual side-by-side, size and stacking recommendations; roughly one average bond length is a reasonable separation in the discussed unspecified forms. Neutral salt representations are also discussed. These do **not** specify a universal source-order shelf algorithm or center-box gap for arbitrary neutral mixtures. Do not infer charge, salt association, a graphical dot/bond, brackets or stoichiometric compression from SMILES dots.
- The public [CDX/CDXML Fragment description](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Fragment.htm) says connectivity within a Fragment is usual but not guaranteed. It supports retaining one complete normalized Fragment for this derived graph. The page includes archived2010 material; it proves no26.1 UI behavior. Bondline's model already explicitly permits disconnected fragments.
- The existing permissive comparison [chem0.9.4](https://github.com/chiral-data/rust-chem/blob/b28f64f46c0aa914599b02737a7db0f51bea3558/src/core/layout.rs) independently lays out actual graph components, then offsets them; its original gap is2×its bond length and later refinement can alter geometry. This is useful separation-of-stages evidence, not Bondline's policy or a drop-in dependency. Do not copy its code, fallback coordinates or force refinement. No new runtime dependency is needed.

Reviewed-byte pins: OpenSMILES86,816B/SHA256`00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`; IUPAC HTML347,564B/`32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`; Fragment HTML18,534B/`c217f44eb95ab682bb691778d73b6361c8898b46f727dbefb8df9d167b3d56b8`; chem layout51,900B/`addd1d13d9c58b047e98cae0d187192a31601e68fd86ebe073984b6963857e95`. Existing ignored research caches were re-read; the immutable OpenSMILES repository and public IUPAC/Fragment pages were consulted. The OpenSMILES website timed out, so the pinned repository resolves that access gap. HTML/source hashes do not attest proprietary runtime behavior, PDF identity or a dependency audit.

## Explicit supported API/profile

Add `SmilesDocument::layout_neutral_components(length,origin,page,&Cancellation)` and an
additive `convert_smiles_components_to_cdxml_file(input,output,length,origin,page,cancel)`.
Use optional profile `neutral_components_v1`, native label **Neutral components**, in the
existing `file.convert_smiles_to_cdxml` command. Keep the same eight required fields,
one30-command registry, closed enum/schema, receipt shape and create-only publisher.
The legacy default/omitted-profile serialization and all three existing public IO/layout
APIs, outputs, errors and refusals remain unchanged. No automatic Open or new file action.

Accept one or more **actual** components, at most64 atoms in total, each either an existing
supported neutral acyclic graph or one supported3–8-member Single cycle with attached trees.
Use existing neutral nonisotopic H/C/N/O/F/Cl/Br/I fixed-H and Single/Double/Triple policies.
Physical H and the qualified unassigned internal-Double treatment retain their exact current
rules. Aromaticity, charges, isotopes, unsupported H/valence/stereo, cycle Double/Triple,
larger/fused/spiro/bridged/polycyclic components or any unsupported member refuse the **whole**
input.64 is a resource ceiling, not a promise of fit or a general layout algorithm.

Normalize the complete immutable notation once, after the total-atom precheck. Keep its
complete raw/IR owner alive through publication. Enumerate BFS components by increasing
minimum original AtomId, with deterministic source-order neighbor ties. Existing normalization
maps source atom/bond array indices to positive IDs; retain every resulting ID, directed
endpoint, order, element and authored H in the final arrays. No deletion, duplicate merging,
first-component fallback, canonical atom reordering or per-component normalization.

Extract the smallest `pub(super)` pure normalized-fragment layout helpers from the existing
acyclic/ring bodies. Their seven arguments are fragment, length, origin, page, cancellation,
work-per-rotation and scenes-per-rotation. Preserve public preflight/error order and existing one-component behavior.
Their current `AtomId−1` indexing assumes a connected dense graph: construct bounded **local
dense Fragment views** plus explicit local↔original maps for component solving, then restore
only positions to the complete original arrays. Never synthesize partial `SmilesDocument`
IR over the whole source or reparse generated sub-SMILES. The new profile's single-component
case must produce the same Document/CDXML as the corresponding existing tree/ring path.

## Original deterministic packing policy

This policy is Bondline's, not a reference-product default or a full salt-style implementation:

1. Solve each component using the existing bounded tree/ring geometry and orientation rules.
   Keep those poses fixed during packing: rigid translation only, no later rotation,
   reflection, force relaxation, scaling, bond-length adjustment or page resizing.
2. Measure each pose's **atom-center** bounding-box width/height. In component order, start
   a shelf at `(origin.x,origin.y)`, with row height0. Place the next component's minimum
   center at the cursor; advance x by its width plus **exactly the supplied length L**.
   Row height is the maximum center-box height already placed on that shelf.
3. If a nonempty shelf's next center box would exceed paper width, start one new shelf at
   `x=origin.x`, `y=previous_row_y+row_height+L`. If that box still exceeds width/height,
   refuse. Equality at a center boundary does not bypass the subsequent ink/paper guards.
   All center minima stay at or above origin, and the overall minimum center x/y is origin.
4. Build one complete positioned Fragment with the original arrays/IDs. Run the **assembled**
   actual vertex/edge/nonincident/intersection geometry, literal ink, same-count resolved-H
   Scene equality and finite-width stroke-corner clearance at the existing1024² reference.
   Include cross-component pairs, every glyph/mask/parallel stroke and physical-paper margin.
   Individual component passes or center boxes alone cannot certify the assembled drawing.
5. If the assembled certificate fails, refuse the whole candidate without publication.
   Do not retry a different gap/order/rotation, shrink, clip or silently omit content. This
   is a bounded shelf policy, not a bin-packer promising every geometrically possible fit.

Gap L is measured between **center boxes**; actual visible ink separation can differ.
It must not be described as the exact IUPAC ink separation. Narrow origins, large labels,
small lengths and inconvenient shelves may honestly refuse even where another packing could
fit. Existing full-ink/readability guards remain authoritative; no synthetic roomy page or
different font/renderer may be used to make a candidate appear valid.

Use cancellation throughout discovery, local solving, packing and final certification.
For K actual components,1≤K≤64, the new profile passes `floor(100000/K)` work units and
`floor(64/K)` scene candidates per rotation to each helper; both are nonzero. Existing
profiles pass unchanged100000/64. With at most24 rotations per component, combined search
is bounded by `24*K*floor(100000/K) ≤ 2400000` work units and
`24*K*floor(64/K) ≤ 1536` candidate checks. Add one final O(n²),64-atom assembled geometry
pass with work cap100000 and one complete candidate certificate, reusing the extracted
`Solver::document` certificate. A certificate includes the actual literal/resolved-H scene
checks; these are logical budgets, not an exact whole-CPU or renderer-invocation cap.
Do not add a shared mutable Budget, new Solver fields or a scheduler. Single-component
inputs retain full existing budgets/results. Atom/scene-size limits use `resource_limit`,
cancellation `cancelled`, invalid scalars `invalid_params`; bounded-search inability,
unsupported chemistry and unsolved ink/fit remain `unsupported_content`. Preserve existing
error families rather than adding a generic framework.

## Existing callers, authority and acceptance

Affected production seams are `io/smiles/{normalize,acyclic,ring,layout}.rs`, a small components
module if useful, `io/files/smiles.rs`, the closed profile/dispatch in
`commands/{smiles_conversion,files}.rs`, and the existing native profile list/actions/receipt
label. The current filesystem helper already counts **all** original atoms/bonds and retains
the owner after publication. The complete Fragment lets existing geometry/stroke checks cover
all components without inventing Fragment IDs. Renderer/model/source parsers need no new
chemistry or structure fields; old approved golden, consumers, fixtures and pins stay exact.

Use the same FilePlan/Completed save reservation, two-worker/reply barriers, cancellation,
revision/generation/source/close qualifications and actual thread retirement. Conversion
retains current Drawing/SDF/SmilesFile, source/saved/history/path/dirty/close state. A historical
receipt reports what was published; it neither associates/cleans the active session nor
creates a trusted Save acknowledgment. The input remains whole-notation≤64KiB with the
existing final-LF/CRLF rule, not a silently selected record from a multi-record source file.

Independent declarations before code should cover:

- Existing JSON rejects the new profile and the new API is absent: distinguish runtime
  missing-profile RED from missing-API compiler failure. Extend exactly3→4 enum choices;
  retain every old schema/default/null/unknown-value and behavioral assertion.
- `CC.O`: original global IDs1/2/3, only1→2 Single, H3/3/2, two actual components; literal
  shelf centers `(72,72),(120,72),(168,72)` at L48 on a roomy page. `C.C` retains two CH4
  labels/components, never ethane or a stoichiometric abbreviation. Mixed tree/ring,
  heteroatom/multiple/physical-H inputs must face independent raw/Expat/Decimal/RDKit checks.
- `C1.C1` is one component and must match old acyclic output; `C1.N.C1` gives one noncontiguous
  source-index C–C component plus NH3. Assert every original ID/directed bond/H by key,
  whole source/IR equality and duplicate preservation, not merely formula or canonical text.
- An independently dimensioned shelf-wrap case, isolated-atom zero-width boxes, exact gaps,
  final minimum origin, pairwise component separation and exact intracomponent lengths.
  Compare actual CPU/GPU captures and literal label regions; new images are not approved goldens.
- Any later unsupported component, full-scene label/stroke collision, paper/readability
  failure, nonfinite/underflow/overflow,64/65 boundary, cumulative budget and cancellation
  must leave source/session/destination exact. Include negative controls joining components,
  dropping a duplicate, changing H/order/direction, shrinking or omitting a row.
- Real CLI/stdio/authenticated-loopback and actual-App fourth-choice flows: Convert while
  a dirty Drawing or complete source-file session remains unchanged, explicit guarded Open
  of the new CDXML, supported edit/Undo/Redo, Unicode Save/reopen and independent consumers.
  Existing-output refusal, stale/close/late-published receipt and held save-slot behavior
  remain covered by the same lifecycle. Native execution is a separate required gate.

## Required continuation and unresolved behavior

This delivers an explicit derived-file profile only. General source-qualified drawing import
still needs its own selected-record/all-record contract, complete original file/opaque-data
backing, baseline/ID mapping, stale-job/history/unsaved decisions and save/loss authority.
Do not silently turn the current source-file session into an editable first-record Drawing.
Wider aromatic/charged/isotopic/stereo chemistry, larger/polycyclic rings, general packing,
clipboard/paste and full M1/M2–M5 workflows remain visible next work. Generic neutral mixtures
have no established vendor spacing/order/association rule here; no26.1 pixel/flow claim is
made. No code, test, dependency, native run or implementation pass is delivered by this plan.
