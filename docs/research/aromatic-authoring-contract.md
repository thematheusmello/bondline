# Aromatic notation to an editable integer-bond graph

2026-11-15. Design only, based on Root `a9e52e7`; no chemistry implementation,
runtime acceptance or new dependency is claimed here. This is a prerequisite for
[real SMILES paste](smiles-source-import-plan.md), not another converter profile or a
read-only substitute. M0–M5 and complete source-file import remain unfinished.

## Source facts and choice of representation

[OpenSMILES 1.0](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc),
Hydrogens and Aromaticity, distinguishes bare implicit H, bracket H counts and physical
H vertices. Bracket omission means H0. Lowercase is a supplied aromatic/sp² marker,
not a result of perception; its ring examples include antiaromatic `c1ccc1` as well as
benzene. The proposed Extended Hückel section is explicitly under revision/discussion.
Do **not** add a blanket 4n+2 acceptance rule. Source markers and an external consumer's
perceived aromatic flags are different facts. Fixed `-` between aromatic atoms stays
Single, including the documented biphenyl spelling. Noncyclic lowercase interpretation
is outside the standard policy reviewed here.

The [official Daylight theory manual](https://www.daylight.com/dayhtml/doc/theory/theory.smiles.html),
§3.4.1–3.4.5, independently gives antiaromatic C4/C8 integer forms, pyridine,
`[nH]`/physical-H/N-substituted pyrrole, exocyclic-double examples and explicit tautomers.
These do not justify silently moving H/charge or inferring a different tautomer.
Neutral N5/oxide forms shown there exceed the initial N3 policy below; they are
unsupported input for this slice, not malformed SMILES.

**Adopt a validated deterministic Kekulé projection.** The model already stores
Aromatic bond orders, but has no aromatic atom marker or complete aromatic edit/render
semantics. An integer Single/Double result with fixed H can use existing native formats.
Do not merely relabel atoms, draw a decorative circle, or claim aromaticity perception,
canonicalization, aromatic-output preference or physical aromaticity from this projection.
Choosing a matching is original deterministic policy, not a reference-product default.

## Small real chemistry boundary

Add `bondline-chem` at L1 only when the implemented function and independent tests land;
depend on model, with no dependency on IO, render, GUI or an external chemistry runtime.
The layer checker already recognizes this crate name. Recommended locked interface:

```rust,ignore
pub enum Hydrogens { Implicit, Count(u16) }
pub struct AtomSemantics { pub aromatic: bool, pub hydrogens: Hydrogens }
pub fn normalize_neutral_kekule(
    fragment: &Fragment,
    semantics: &[AtomSemantics],
    cancelled: impl Fn() -> bool,
) -> Result<Fragment, AuthoringError>;
```

The immutable sidecar is aligned to atom **array order**, not `AtomId−1`.
Require exact length and exact input correspondence: Implicit ↔ `explicit_hydrogens=None`,
Count(h) ↔ `Some(h)`. Mismatch is `invalid_document`. IDs need not be dense in chemistry.
Preserve Fragment/Atom/Bond IDs, array order, directed endpoints, positions, positive
isotopes and unrelated metadata. Only materialize implicit H and replace Aromatic orders
by Single/Double. Nonneutral charge, radicals and other bond kinds refuse; do not clear
them. Isotope0 refuses in the IO adapter before conversion to the positive-isotope model.

Add `SmilesDocument::normalize_authoring_fragment(&render::Cancellation)` in IO, mapping
the retained IR to the model/sidecar and passing `|| cancel.is_cancelled()` to chemistry.
The parser/IR remains independent of chem and unchanged. For nonaromatic inputs within
the new limits, delegate to the old normalizer and require **exact old output**; its public
API and all four converter profiles/defaults/refusals remain unchanged.

## Closed neutral H and demand rules

Initial aliphatic support remains H/C/N/O/F/Cl/Br/I with the existing H1/C4/N3/O2/halogen1
targets. Physical H remains an H0 vertex with exactly one Single bond. Positive isotope
mass is retained as chemistry data; depiction eligibility is a later independent gate.
Initial aromatic atoms are C/N/O only. B/P/S/As/Se, charged/radical states and stereo are
explicit required follow-up work, never silently downgraded to supported chemistry.

Let `base` be the incident bond-weight sum with Aromatic=1 and fixed S/D/T unchanged;
`degree` counts actual neighbors, including physical H. Arithmetic is checked. Bracket H
is exact; an unrepresentable deficit is refused, not repaired by changing H or a fixed bond.

| Marked atom | Initial state and H | Required extra Double (`demand`) |
|---|---|---|
| C | degree2/3, no Triple, ≤1 fixed Double; implicit H=`3−degree`, bracket H exact | `4−base−H` must be0/1; fixed-Double count + demand must equal1 |
| N | degree2/3, no Triple, ≤1 fixed Double; implicit H0, bracket H exact | `3−base−H` must be0/1 |
| O | degree2, H0, no fixed Double/Triple | 0 |

Thus pyridine `n` needs one Double; `[nH]`, N-substituted pyrrole and its physical-H form
are donors needing none. An exocyclic C=O consumes the marked carbon's demand while its
fixed Double survives. `[c]` with two ordinary aromatic neighbors has deficit2 and refuses;
it is not interpreted as a radical. Aromatic edges require two marked endpoints: `C:C`
cannot gain aromatic atom semantics from its colon alone.

**sp²-cycle guard:** construct the induced graph of validated resolved/required states:
C has exactly one fixed+required Double and no Triple; N has one Double or a legal neutral
donor state (sigma bonds + attached H total3); O is a degree2/H0 donor. Unmarked sites with
those fixed states participate, allowing mixed Kekulé notation. Every marked atom must
lie on an actual cycle in this graph. In particular `[nH]1CCCC1`, `o1CCCC1` and
`C=c1CCCCC1` refuse; `[nH]1C=CC=C1` and `o1C=CC=C1` qualify for matching. Marked states are
determined by demand, so this check need not depend on a particular matching branch.
Do not require every Aromatic edge itself to be cyclic, impose a ring-size restriction,
or mistake this necessary representation guard for full aromaticity perception.

## Matching, bounds and failure

Validate the complete graph once, including all actual connected components; do not split
on dot tokens (`C1.C1` can be connected). Eligible assignment edges are supplied Aromatic
edges joining two demand1 atoms. Pick the lowest unmatched AtomId, try neighbors in
`(neighbor AtomId, BondId)` order, and return the first **complete** matching. Unchosen
Aromatic edges become Single; fixed Single/Double/Triple never change. Donor vertices are
excluded from Double matching. This is generic graph search, not a six-ring special case:
eligible fused, large and disconnected graphs receive the same chemistry treatment.

Fixed initial limits:64 total atoms,128 bonds,100000 search visits and64 recursion depth.
Check input caps before allocating search state. The bond cap is compatible with the
closed maximum neutral valence4 (`E≤2V`), not permission to skip per-atom validation.
Count a visit consistently before exploring a search state; check cancellation at entry
and through bounded validation/search. No per-component budget reset, partial graph,
unbounded retry or configurable solver framework is needed. Preserve error distinctions:
`unsupported_content` for unsupported semantics/proved no assignment, `resource_limit`
for caps/search exhaustion, `cancelled` for cancellation, `invalid_document` for malformed
model/sidecar correspondence. Do not report budget exhaustion as chemical impossibility.

## Source ownership and the next executable gate

Retain original notation bytes, atom/bond spans, aromatic markers, H mode and explicit
fixed bonds unchanged in `SmilesDocument` through deferred work. The integer result is a
projection, not a source rewrite or proof that it can regenerate those bytes. The real
paste transaction creates native Drawing content; persistence of the original spelling
is not promised without separately designed backing. Whole `.smi` source/current/original
and record-data editing remain the distinct source-file gate in the parent plan.

Geometry must accept a validated integer fragment through an additive authoring route;
the old Single-cycle converter must keep its refusals. Benzene/pyridine/pyrrole need real
ring positions and full resolved/literal-H, labels, multiple-stroke, paper and collision
certificates. Fused/larger topology chemistry can succeed while its unavailable layout
refuses the **whole paste** honestly. Preserve every component/physical H and append via
the shared registry as one checked global-ID/history commit. No temporary conversion file,
first-component fallback, model-only source replacement or fake Save acknowledgment.

Independent RED declarations should cover benzene and its fixed Kekulé equivalent,
pyridine, furan, `[nH]`/physical-H/N-substituted pyrrole, exocyclic carbonyl, positive
isotope, explicit-Single biphenyl, a fused graph and disconnected benzene+ethane. Check
literal H/formula/graph/bond direction plus a separately specified matching tie; do not
copy RDKit's Kekulize order. Include C4/C8 antiaromatic notation without requiring perceived
aromatic flags, saturated-donor/odd-demand/nonring/charge/stereo/unsupported-element cases,
sidecar mismatches, fixed-bond preservation, pre-cancel and resource bounds. The frozen
old normalizer negatives remain negative under their original API.

Then prove actual paste into a nonempty native Drawing through CLI/stdio/authenticated
loopback and native clipboard, full-page render, one-step Undo/Redo, edit and Save/reopen.
Use unchanged independent consumers for actual saved graph/H/isotopes, reject assigned
stereo/radicals and distinguish source flags from perception. No such execution is claimed
by this note. Styling of Kekulé rings, broader stereo/charge chemistry, fused/large-ring
layout and complete source-qualified import remain explicit outstanding requirements.

## Reviewed evidence and limits

All below are public primary/specification or author sources, accessed2026-11-15.
No reference assets/code are transplanted. Cached artifacts stay under ignored target/.

| Evidence | Reviewed sections / exact bytes SHA256 |
|---|---|
| OpenSMILES commit`9240a2887158699503605bc838e1de9a9f7883f4` | Hydrogens190–214/266–332, Aromaticity527–667;86816B `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21` |
| Official Daylight HTML, live4.9 manual | §3.4.1–3.4.5;43582B `a5f583eb3f611c59a57c7317a5cb60f59b06e644e36eac5fd52b7293f7b0d2bc` |
| [Apodaca,2020-02-10](https://depth-first.com/articles/2020/02/10/a-comprehensive-treatment-of-aromaticity-in-the-smiles-language/) | delocalization/pruning/matching discussion;29002B `35299ac6e790deeacccb68738db1dd730b168052d539ff006f41e44f68d2a45d`; its broader noncyclic policy is not adopted over OpenSMILES |
| [Rust chem0.9.4 source](https://github.com/chiral-data/rust-chem/blob/b28f64f46c0aa914599b02737a7db0f51bea3558/crates/chem/src/io/aromaticity.rs) | donor pruning/backtracking and separate heuristic perception inspected;27717B `2c7fdf44f44e6fb2b2a25782f20f78f6bf18ed36f1419290570547385d1c83a7` |

The official chem0.9.4 archive is677078B/
`58e5ffdf41e64c500889865cee20dd490500ae174cf7e385242e867680efd77f`;
its VCS metadata pins the source above and `crates/chem` path. Published metadata saysMIT;
the archive omits a LICENSE member, while the previously reviewed repository MIT text is
1079B/`a9410a68876e201c472443378e4f104a6fb2049f4aa840b7aa402f3e5d29a2ae`.
No dependency/closure/license admission is proposed. Its mutable heuristic perception and
unbounded matching are not the required contract; original small bounded code is preferable.

Current-code review pins: `model/src/graph.rs` `8f74355d…2211e`,
`io/src/smiles.rs` `c82d988c…2b3b0`, `normalize.rs` `e75bfa03…5c4e9e`,
`ring.rs` `03a5b759…01267`, `components.rs` `7ed45b21…e9179`,
`write.rs` `c6ae2531…03c7`. Full hashes, captured sources and orientation/instruction
inputs are in `target/aromatic-authoring/research/research-receipt.json`.
No Cargo, product tests, native interaction or installed-reference inspection was performed
for this design. Any later implementation evidence belongs in a separate verification record.
