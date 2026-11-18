# Public sources for broader SMILES drawing import

Research only, reviewed 2026-11-15 against Bondline `97fd2ac`. No new layout, dependency, chemistry, import authority or native workflow is verified here. The complete M1 target remains general interoperable import/export; the next increment below is one dependency of that target.

## Recommendation and present boundary

Implement an **additive deterministic neutral-tree layout** in the existing `io::smiles` module, using the existing closed normalizer and renderer preflight. Include branches, heteroatoms and Single/Double/Triple bonds rather than another carbon-chain variant. Preserve `layout_carbon_path` and `neutral_carbon_path_v1` byte/geometry behavior exactly. Do not add a chemistry toolkit dependency for this first increment.

Current code, reviewed rather than executed:

| Boundary | Actual location and meaning |
| --- | --- |
| Source notation and graph | `crates/io/src/smiles.rs`; retained source and literal aromatic/H markers are separate from inferred chemistry. |
| Existing normalization | `smiles/normalize.rs:75`, `write.rs:61`: neutral H/C/N/O/F/Cl/Br/I only, fixed target valences 1/4/3/2/1. Positive isotopes and physical hydrogen nodes survive; incompatible bracket counts, charge and any aromatic atom/bond refuse. This is narrower than OpenSMILES. |
| Existing geometry | `smiles/layout.rs:48`: 1–12 connected nonisotopic carbon/Single paths; actual atom identities, analytic geometry and complete 1024² scene checks. |
| Native conversion | `files/smiles.rs:108`: whole bounded notation to a new create-only CDXML file using that path API. It does not select a record from a source file. |
| Accepted source files | `files/smiles.rs:22`: complete multi-record `SmilesFileDocument`, including ignored lines/opaque application data; source retention is not drawing/Save authority. |
| Display | `render/page.rs:247` and its label/multiple-bond paths; use actual glyph masks, strokes and paper/readability guards, not a second point-only approximation. |

The new geometry API should return a fresh positioned graph/document while leaving the borrowed source/IR unchanged. Shared command/native import and selected-record provenance must then be wired explicitly; never manufacture a temporary notation file or silently convert the first record.

## Format semantics are not layout rules

[OpenSMILES 1.0, 2016-05-15](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc) defines graph connectivity: parentheses attach branches, paired ring labels add edges, and dots separate components. These are not coordinates or a canonical drawing order. Bracket H omission means zero, `H` means one, and bracket H counts differ from physical `[H]` atom nodes. Bare organic atoms use the published normal-valence rules, which include cases Bondline deliberately refuses. Lowercase/aromatic syntax does not itself implement aromaticity perception. Relevant sections: grammar; Hydrogens; Organic Subset; Branches; Rings; Disconnected Structures; More about Hydrogen.

Keep exact atom/bond IDs, endpoints, isotope, charge, physical H and authored/resolved count distinctions. Do not use RDKit sanitization or a layout library to overwrite the original graph/H interpretation. Canonical SMILES and atom-order-independent coordinates remain separate work.

## Depiction recommendations that constrain the next algorithm

[IUPAC graphical recommendations](https://iupac.qmul.ac.uk/drawing/drawing.html), Brecher, *Pure Appl. Chem.* **80** (2008), 277–410, [DOI 10.1351/pac200880020277](https://publications.iupac.org/pac/80/2/0277/index.html), distinguish context, not just atom degree. Multiple bonds count as one drawn adjacency here.

| Context | Recommendation / exact section |
| --- | --- |
| Two ordinary connections | Usually 120°; chains favor trans-like progression, GR4.1.1. |
| Carbon with triple+single or two double bonds | 180°; the two-double central carbon must be labeled, GR4.1.1 / GR2.1.2. |
| Three connections | 120° intervals, GR4.1.2. |
| Four, all neighbors terminal | Four 90° intervals, GR4.1.3. |
| Four, two terminal and two extended neighbors | 60°/90°/120°/90°; one-terminal cases depend on substituent context, GR4.1.3. |
| Four, one terminal and three extended neighbors | 60°/60°/120°/120°; also preferred for three identical terminal neighbors plus one extended neighbor, GR4.1.3. |

GR4.1.3 also covers other contexts and warns against Fischer-like axial arrangements at stereogenic atoms. GR4.1.1 uses a collinear depiction for unknown cis/trans in the specific disubstituted-alkene case. GR4.3 rejects exact overlaps and orders remedies; chain rotation precedes later length/angle changes. Therefore uniform 90° branching and arbitrary force relaxation are insufficient.

**Required decisions before code:** encode the degree-four context table, including its remaining cases; do not infer stereochemistry or draw wedges. For `CC=CC` without stereo, choose a documented unspecified-configuration policy (the cited collinear case is a candidate), rather than silently selecting an apparent E/Z drawing. Confirm central `C` label visibility on `C=C=C` in an actual scene. Do not broaden the existing normalizer to obtain a convenient picture.

## Original, executable algorithm contract

These are proposed Bondline policies, not vendor defaults or a claim of full IUPAC conformance:

1. Normalize once with `normalize_explicit_h_fragment`; require one connected acyclic graph, supported elements/counts/orders and explicit finite input/page/resource budgets. Detect cycles/connectivity from actual edges, not only `edges = atoms - 1`.
2. Preserve normalized IDs and metadata. Choose a longest tree path, with source-index tie breaks; place its chains using the context rules above. Order branches by subtree extent, then source index. Same ordered graph/parameters should reproduce the same result; equivalent reordered SMILES need not share coordinates.
3. Enumerate a bounded, stable order of context-valid branch orientations and rotations around eligible Single bonds. Do not rotate a multiple bond to repair collisions, or use random jitter. Set and test explicit atom/candidate/work limits; return a typed whole-graph refusal if no permitted candidate fits.
4. Validate analytic bond lengths/angles, finite coordinates, nonincident atom-to-segment distance and segment intersections. Then validate the actual rendered labels, isotope/subscript glyphs, masks, parallel strokes and paper margins at the established reference frame. A point-only noncollision score is not enough; do not silently clip or omit an atom/bond.
5. Keep old carbon-path geometry and existing Single/multiple-bond/golden/font checks unchanged. A new profile may fail honestly on crowded graphs. No general force solver, valence engine or style framework is needed for this step.

The [Trepalin et al. 2006 primary paper](https://www.mdpi.org/molecules/papers/11040219.pdf), *Molecules* **11**, 219–231, describes constructive placement, separate complex-ring templates and bounded acyclic-bond rotation combinations. It supports separating placement/collision stages and bounding search. Its MCDL bond-reconstruction rules, proprietary comparison images, Java code and template library are not proposed for reuse.

## Rust component feasibility, pinned source review

| Candidate | Actually reviewed evidence | Decision for this increment |
| --- | --- | --- |
| [`chem` 0.9.4](https://crates.io/crates/chem/0.9.4), VCS [`b28f64f46c0aa914599b02737a7db0f51bea3558`](https://github.com/chiral-data/rust-chem/tree/b28f64f46c0aa914599b02737a7db0f51bea3558) | MIT manifest/root license; `src/core/layout.rs` is Rust constructive rings/components, substituent placement, overlap separation and bounded stress refinement. Public `layout(&mut Molecule)->bool` replaces coordinates and contains production `expect`/fallback coordinates. Source tests measure geometric properties; not executed here. | Useful algorithm comparison, not a drop-in for Bondline's typed errors, exact H/source authority or actual-label collision policy. Dependency closure, ring correctness, determinism and cross-platform behavior are unaudited. |
| [COSMolKit `265e2e2bbdfc481546ef09d1bb26499df9ea7d4c`](https://github.com/cosmol-studio/COSMolKit/tree/265e2e2bbdfc481546ef09d1bb26499df9ea7d4c/crates/cosmolkit-depict) | MIT root/workspace; actual Rust detached coordinate API, RDKit-shaped fragment/collision routines, optional seeded sampling, ring templates, valence/stereo preparation and coordinate regression tests. Read API/defaults/pipeline and selected helpers, not the entire 724,850-byte fragment implementation. | Larger model/core/search/render dependency surface; third-party notices/templates/fonts and closure need a separate audit. Do not import its valence/stereo behavior into the closed normalizer. No adoption or test-pass claim. |
| [CoordGen `d20e735d96480385b2e257522288004038a08cc9`](https://github.com/schrodinger/coordgenlibs/tree/d20e735d96480385b2e257522288004038a08cc9) | BSD-3 license and primary README, C++/CMake/Boost/maeparser build. | Useful public reference; not a pure-Rust layout component. [`molchemist`](https://github.com/rice8y/molchemist) wrapping CoordGen does not change that distinction. |

No listed library was built, installed, linked or copied into product code. Top-level permissive licensing is not a completed dependency/asset audit. Existing external RDKit remains an independent graph/H consumer, not the shipped layout engine.

## First TDD gate and full-import continuation

Use independently declared graph/H/coordinate expectations, not our writer as the sole oracle:

- `CCO` and `CN`: exact O/N H counts and both attachment directions; `CC(C)O`: degree-three branch and C3H8O; `C(C)(C)(C)C`: degree-four terminal context and C5H12.
- `CC#N` / `CC#C`: linear center and C2H3N / C3H4; `C=C=C`: C3H4 plus explicit central C; `CC=CC`: unspecified-stereo policy. Check isotope and physical `[H]O[H]` retention separately.
- Independent analytic angle/distance/collision checks, actual CPU/GPU RGBA and unchanged external MOL/CDXML/SMILES consumers; malformed/unsupported/crowded/limit/cancel cases leave source and session exact. Registry/CLI/MCP/native Save/reopen/history tests follow when that interface lands.
- Next explicit geometry increments: a single simple ring with substituents (`C1CCCCC1`, heterocyclic cases), then disconnected components (`CC.O`) with original packing policy. General fused/spiro/bridged rings, aromaticity, charges, stereo, larger graphs and complete source-qualified native import remain required; a neutral-tree pass does not complete M1.

## Reviewed artifacts, pins and access limitations

Artifacts stay ignored under `target/smiles-general-layout-research/`; full path/byte/hash inventory and reviewed workspace identities accompany the handoff. No reference prose, templates, pictures, fonts or source is shipped by this note.

| Source | Exact reviewed-byte identity |
| --- | --- |
| OpenSMILES pinned AsciiDoc above | 86,816 B, SHA-256 `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21`; GFDL prose, reference only. |
| IUPAC HTML above | 347,564 B, SHA-256 `32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`; cached public bytes re-read, public HTML sections checked. HTML declares amendments separately; this is not a claim of identical PDF bytes. |
| Trepalin PDF | 86,590 B, SHA-256 `c3cc585a87a9b3e3bd5702c087971fd6bf6f9e02e75b04880505856f82b089eb`; read extracted text, no figure reuse. |
| `chem` published archive | 677,078 B, registry checksum / SHA-256 `58e5ffdf41e64c500889865cee20dd490500ae174cf7e385242e867680efd77f`; exact archive verified before selected source reads. |
| COSMolKit depict API | 66,155 B, SHA-256 `a7d2bb3c1e35fd9f55494fe2644131638b8a4b57f9618fb5c59048a0f28603eb`; other reviewed files listed in the artifact receipt. |

`docs.rs/chem/latest` returned different cached versions (0.9.2 and 0.9.4); a source endpoint also failed. The fixed 0.9.4 crates.io metadata/archive and immutable repository pin resolve the reviewed identity. No proprietary runtime observations or claimed reference-product pixels inform this algorithm recommendation. No executable checks were run for this documentation-only research.
