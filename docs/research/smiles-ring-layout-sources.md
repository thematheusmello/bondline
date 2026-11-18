# Sources and contract for simple neutral SMILES rings

Research at `492905a`, 2026-11-15. This is an implementation recommendation, not a layout, chemistry, native-flow or parity verification. Existing carbon-path and neutral-acyclic profiles stay exact. General M1 import, larger/polycyclic rings and complete native editing remain the target beyond this increment.

## Next executable contract

Add `neutral_simple_ring_v1` / native **Neutral ring structures** to the existing converter. Require one actual connected unicyclic graph, one simple cycle of 3–8 vertices with all cycle edges Single, and attached trees using the current neutral/nonisotopic H/C/N/O/halogen normalization and Single/Double/Triple restrictions. Keep the existing 64-atom/resource/page/cancellation bounds and create-only publication lifecycle. This size/order boundary is the first supported profile, not a claim that other rings are invalid chemistry.

Use a regular polygon, ring-specific substituent rules and bounded original tree alternatives. Preserve every normalized atom/bond ID, directed endpoint, source order and fixed H count. Translate the complete molecule's minimum atom-center x/y to the requested origin. Do not anchor only the ring, discard substituents or select a source-file record implicitly. Successful geometry still needs the current whole literal-glyph/stroke certificate before a fresh document/file is produced.

## Primary evidence

[OpenSMILES 1.0](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc), pin `9240a2887158699503605bc838e1de9a9f7883f4`, sections **Rings**, **Branches**, **Disconnected Structures** and **Hydrogens**: paired ring labels add a graph edge; matching bond symbols must agree. Labels are not cycle identities or coordinates. A closure can join previously disconnected chains: topology must be checked after parsing. Bracket H, bare-atom implicit H and physical H nodes remain distinct. No syntax change or aromaticity inference is required for this profile.

[IUPAC graphical recommendations](https://iupac.qmul.ac.uk/drawing/drawing.html), Brecher, *Pure Appl. Chem.* **80** (2008), 277–410, [DOI 10.1351/pac200880020277](https://publications.iupac.org/pac/80/2/0277/index.html), supply these concise constraints:

| Section | Reviewed recommendation |
| --- | --- |
| GR3.3.1 | Isolated rings ≤8: regular polygons. |
| GR3.3.2 / GR4.2.7 | Rings ≥9: reentrant shapes; ring-double configuration has priority; triple centers remain linear. |
| GR3.4.2 | Six: vertical left edge. Other small sizes: horizontal bottom/top edge. |
| GR4.2.1 | One external bond: exterior bisector; snap within5° to a30° multiple. |
| GR4.2.3 | Geminal pair: normally60°; four-ring90° alternative. |
| GR4.2.3 | Both external roots have ≥2 additional bonds: trisect the exterior angle. |
| GR4.2.3 | Hexagon, single-atom versus multi-atom branch: larger radial, smaller bisects an adjacent angle; other sizes retain60°. |
| GR4.2.3 | Prefer the less crowded side; ties favor an axis, then right, then top. |
| GR1.10.4 / GR3.5 | Applicable ring doubles favor inward offsets; double positioning is secondary to other geometry. |
| GR4.3 | Avoid overlap; consider remedies in the documented order. |

These recommendations do not specify a unique atom-index phase, search order, collision tolerance, source identity or implementation algorithm. They are not reference-product pixel measurements.

## Original deterministic choices and contextual angles

- Validate connectivity and the unique cycle from actual edges. Leaf peeling can isolate the cycle; require its vertices to form exactly one degree-two loop. Counts alone or ring-label count are insufficient. Preserve the original bond list instead of rebuilding it in traversal order.
- Start at the lowest source-index cycle atom and follow its lower-index cycle neighbor. For length `L`, use radius `L/(2 sin(π/n))`. In the existing y-down coordinates, an original phase is `θ₀=π−π/n` for six and `θ₀=π/2−π/n` otherwise, incrementing by `2π/n`. This chooses a vertical-left or bottom-horizontal first edge. The phase/tie choices are Bondline policy within the cited orientation family.
- One branch uses the outward radial direction before the cited near-axis snap. Two ordinary branches may use ±30° around that direction. The four-ring ±45° alternative is available if adopted explicitly. Neither branch should be silently placed inward to make a candidate pass.
- For **two bulky immediate roots**, measure their full graph degree, not subtree atom count: each must have at least two bonds besides the ring attachment. If exterior angle is `E`, three equal gaps put the branch rays at `±E/6` around the outward bisector. Do not confuse this exception with two merely long chains.
- For the **hexagon mixed-size case**, count drawn graph atoms in each external subtree, excluding the ring. Put the multi-atom branch radially outward and the one-atom branch at either adjacent bisector, ±60° for a hexagon. Evaluate the documented crowding/axis preferences and use stable source-ID ties only for remaining equivalents. H-count attributes are not additional drawn atoms; physical H nodes are.
- Ring vertices cannot use the ordinary tree's degree-three/four angle templates unchanged: ring closure already fixes two incident directions. Heteroatom and extended geminal branches should be accepted when the normalizer and whole-scene checks permit them, rather than refused for implementation convenience.

## Reuse boundaries in actual Bondline source

`io/src/smiles/normalize.rs:75` already preserves the closed graph/H policy. `smiles/acyclic.rs:173` validates a tree; its parent-only recursive `size`/`root_tree`/`key` at lines237/247/278 are safe behind that tree guard. Passing a full cyclic adjacency into them could recurse indefinitely. This is a **prospective reuse hazard**, not a reachable defect in the existing acyclic API. Build a cycle-bounded attached forest or stop signatures at the ring boundary, while retaining true graph degrees for chemistry and geminal context.

Reuse the actual-edge/finite-coordinate/intersection checks and the certificate in `acyclic.rs:531`: `layout::validate_ink_with_cancel` plus exact literal-versus-resolved authored-H primitives and `stroke_clearance`. A successful candidate must describe the same graph that ordinary Open/capture will paint; validating a different display-H scene or suppressing authored labels would be incorrect.

The current renderer accepts positioned neutral supported-element Single/centered Double/Triple graphs subject to guards; it is not itself a ring-layout engine. `render/page.rs:330` bounds primitives/hit regions; label code requires finite readable8–256px em and bounded coordinates. Actual masks, incident attachment, paper ink and nonincident strokes can refuse a small/crowded ring. Neither3–8 nor64 atoms promises fit. No renderer/pixel behavior was executed in this research.

## Multiple bonds and stereo: explicit remaining work

The first cycle-edge Single restriction is original conservative scope. A regular small polygon is incompatible with a required linear ring triple or cumulene center. Internal ring Double depiction needs its own source/configuration and independently qualified consumer contract; the acyclic antiparallel-unknown rule cannot simply be applied around a polygon. Do not assert that cyclic coordinates always yield `STEREONONE`, or treat `STEREOANY` as assigned E/Z. Assigned atom/bond stereo remains outside the current profile.

Attached tree bonds can retain existing terminal Double, Triple-linear and qualified degree-two internal-Double behavior. Preserve the existing degree-three internal-Double refusal. A general centered Double renderer is not the inward-offset ring style recommended in applicable IUPAC contexts. Ring multiple bonds, aromatic perception, larger/reentrant rings, fused/spiro/bridged systems, disconnected packing and source-qualified import need separate increments; native chemical order-edit authority is a separate existing command contract.

## Permissive Rust references: inspected, not adopted

| Pinned reference | Actual inspected portions and consequence |
| --- | --- |
| [`chem`0.9.4 / `b28f64f46c0aa914599b02737a7db0f51bea3558`](https://github.com/chiral-data/rust-chem/tree/b28f64f46c0aa914599b02737a7db0f51bea3558), MIT | `src/core/layout.rs:294–455` constructs regular rings first, then attaches/fuses and places chains. Its initial phase is0 for any size; this is not the selected IUPAC six-ring orientation, nor a large-ring conformance argument. Existing broad relaxation/mutation API is unnecessary for the bounded next step. |
| [COSMolKit `265e2e2bbdfc481546ef09d1bb26499df9ea7d4c`](https://github.com/cosmol-studio/COSMolKit/tree/265e2e2bbdfc481546ef09d1bb26499df9ea7d4c), MIT root | `cosmolkit-depict/src/geometry.rs:404–428` is an actual Rust regular-polygon primitive; `embedded_frag.rs:1893–1923` shows a separate ring-system owner. Ring generation, templates, stereo and collision stages are separate. Its graph/dependency/template/license closure is not re-audited or adopted here. |

No library, template, icon, font or reference artwork is copied into Bondline. Source copies below remain ignored research artifacts. Polygon mathematics can be implemented with the existing Rust stack and explicit guards; no new dependency build is justified by this research.

## Decisive unexecuted acceptance targets

Independent literal vectors should cover all sizes3–8, exact closure/length/interior angles/orientation, a heterocycle, one exterior branch, near-axis heptagon snap (`CC1CCCCCC1`), simple and contextual geminal families, hetero/extended branches and attached multiple bonds. Verify original graph/H/IDs and no partial graph, actual glyph/stroke hits and CPU/GPU pixels, external chemical consumption and source/session preservation. Include malformed, no-cycle, disconnected, two-cycle, fused/spiro/bridged, ≥9, ring-Double/Triple, isotope/charge/aromatic, tiny-page/length, budget and cancellation refusals.

Then carry the named profile through existing registry/CLI/MCP/native conversion, explicit ordinary Open, supported editing/history and Save/reopen. These are proposed checks, not passing results in this note. The independent implementation/test owners retain their actual RED/GREEN evidence separately.

## Exact source receipts

- Public IUPAC HTML re-fetched2026-11-15 matches the cached reviewed bytes exactly:347,564B, SHA-256 `32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`. Inspected sections are listed above; extracted text is reference-only.
- OpenSMILES pinned AsciiDoc:86,816B, SHA-256 `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`; GFDL prose is not bundled.
- `chem` published archive:677,078B, SHA-256 `58e5ffdf41e64c500889865cee20dd490500ae174cf7e385242e867680efd77f`; reviewed layout51,900B, SHA-256 `addd1d13d9c58b047e98cae0d187192a31601e68fd86ebe073984b6963857e95`.
- COSMolKit geometry55,733B, SHA-256 `f008e68ab17a513f6be6eb9edfe8f9a7dea1cc7c507760021e4158de84be1f64`; fragment724,850B, SHA-256 `ac2afdb023799841c60172f2fc4b054d2fb2b3f7c24fda317d2ee97e16dd91c6`. Only the named portions were inspected, not a whole-module audit.
- `target/smiles-ring-research/source-artifacts.json` records copies/URLs/hashes; `reviewed-workspace-inputs.json` pins12 current orientation/source inputs. No primary-source access blocker occurred. No Cargo, CI, dependency build, proprietary/runtime inspection or native input was used. Only authorized worktree creation and this new research note mutate the repository.
