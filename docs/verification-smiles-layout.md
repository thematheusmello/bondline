# Bounded SMILES carbon-path geometry

2026-11-14. Original library-only slice in `bondline-rxn`, supplied branch
`feat/smiles-path-layout`, base `e647196`. No application SMILES command, conversion
file workflow, UI, native interaction, cleanup engine, new crate or dependency is added.
The user was actively using the computer; no GUI/input/activation/state-focus test ran.

## Delivered API and exact scope

`SmilesDocument::layout_carbon_path(bond_length_pt, origin: Point2, page_size: PageSize)`
returns one useful positioned pure `Document` or typed `LayoutError`. Its single page and
fragment use fresh local model identities. The existing normalizer supplies fixed H;
all its atom/fragment/bond identities, metadata, order and connectivity are preserved,
except that the new atoms receive positions. Source bytes/token spans/IR/Arcs remain exact;
the normalized source-free fragment is not modified. No model None semantics change.

Accepted:1–12 atoms, one connected neutral nonisotopic carbon/Single path. Topology is
checked from edges, not token order. The endpoint with the lowest source-index/dense local
ID starts the path. `C(C)C` and its bracketed equivalent place their central source atom
in the middle. `C1.C1` reconnects as an acyclic2atom graph and is accepted; actual cycles,
branch degree3/4, disconnected graphs, Double/Triple, other elements/physical H, isotope,
charge and radical-ambiguous brackets refuse. Aromatic/stereo/class input refuses through
existing syntax/normalization boundaries; no assumption about a new source enum is made.

L must be finite and positive. Origin/page types enforce their existing finite bounds;
vertices and complete visible ink must also fit the supplied page. Unsupported input has
`unsupported_content`; invalid L has `invalid_params`; the12atom/scene resource caps
have `resource_limit`. Normalization and model/render errors retain typed causes.

## Geometry and page proof

Original geometry: unrotated points `(i sqrt(3)L/2, (i mod2)L/2)`. Whole-chain rotation makes
the endpoint chord horizontal; a sole atom stays at the supplied origin. Thus every edge
has lengthL and every interior angle is120degrees. This is deterministic source-order
placement, not canonicalization, molecular3D geometry or general structure cleanup.

Before success the implementation checks actual vertex separation, vertex-on-unrelated-
bond and nonincident-segment intersection, not only a spanning-tree assumption. It then
builds the existing `GraphScene` at **1024×1024 reference canvas units**. The complete page
prefix is obtained from `page_scene` and checked against the produced graph prefix before
it is skipped; there is no fixed three-primitive assumption. Every chemical rectangle,
including actual glyph spans and conservative white label masks, must lie in the page's
white interior. Actual `Stroke::corners()` checks include half-width and the renderer's
butt caps. Font/label overlap/placement/scale, hit and4096primitive limits stay active.
There is no clipped-success, random fallback, count deletion or Some(H)→None workaround.

**Verified starting policy, not implicit API defaults:** explicitly supplied48pt bond
length, origin(72,72)pt and A4 passed all12 path sizes through geometry, actual glyph/label
checks, CPU raster black-ink-in-paper checks and actual generated-file consumers. These
numbers are original Bondline policy, not vendor/journal/IUPAC numeric defaults. Every
API call still supplies them. Independent40pt vectors and a Letter page/origin(100,200)
case also pass. Arbitrary smaller viewports, native DPI, GPU/native rendering, journal
settings and general depiction are not certified by this reference-scene validation.

## Independent expectations and external consumers

Six integration tests preceded production edits. C2 L40/origin72 must be horizontal at
(72,72)/(112,72). C3 must be (72,72), (72+20sqrt(3),92), (72+40sqrt(3),72). All sizes1–12
check edge length within `1e-9*max(1,L)`, normalized interior dot product−0.5 within1e-12,
horizontal chord, increasing path x, nonbonded separation, exact metadata/IDs/H/connectivity
and unchanged source/IR sharing. Private validator vectors cover duplicate vertices,
vertex-on-bond, crossing nonincident segments and separated collinear segments.

Invalid lengths, collapsed/overflowing coordinates, off-page vertices, glyph overlap,
labels extending over each of the four page edges while their centers remain on-page,
unsupported profile/topology and13atoms refuse. All1–12 CPU frames contain actual black
ink entirely within the paper interior. L1 exposes the existing label-overlap refusal;
no assertion or renderer guard was weakened to make it pass.

Actual generated native CDXML bytes, not an alternate synthetic molecule, go to unchanged
`xtask/oracles/cdxml_graph.py` (Expat/Decimal structural projection), unchanged
`native_bond_rdkit.py` forC2, and unchanged `files_rdkit.py` for the independent Letter/
(100,200)→(140,200) case. Original `smiles_layout_oracle.py` additionally checks every raw
XML coordinate with Decimal against authored digits and an independent40-digit analytic
L48/origin72 calculation (absolute coordinate tolerance1e-9, chosen before judging).
Pinned external RDKit2026.09.2 reads every1–12atom output, checking neutral carbon/Single,
canonical external chain meaning and CnH(2n+2). This does not implement Bondline chemistry.

Each output is reopened by `StoredDocument::from_cdxml`, must classify fully mapped and
must equal the entire original positioned Document **exactly**; subsequent `to_cdxml`
retains the complete original bytes. Native-v0 and source guards remain unchanged.
The fixed-coordinate terminal-chain oracle was not edited or adapted to these positions.

A real initial external comparison found a separate JSON-number transport1ULP difference:
C4 x198.99606293110034 became198.9960629311003 after the external JSON Value boundary.
The wire-only comparison now takes both expected/actual through that same boundary.
Raw Decimal XML checks, independent analytic tolerances and strict Document/reopen equality
remain separate and all pass. No positions are rounded, codec behavior changed, dependency
feature enabled or old oracle altered to hide the observation.

## RED/GREEN and environment

All Cargo calls used the absolute RXN target, `CARGO_INCREMENTAL=0`, dev/test debug0,
Rust1.94.1 and the existing unresolved pinned Root Python venv path through BONDLINE_PYTHON.
No Git/install/full-workspace/fuzz/GPU/native call was made; no other checkout target used.

- Missing API RED: `cargo test --locked -p bondline-io --test smiles_layout`, session87687
  exit101, unresolved LayoutError/E0599 layout_carbon_path; `target/smiles-path-layout-api-red.log`.
- Initial runtime RED: session29866 exit101,4passed/2failed. Missing font/ink enforcement
  was decisive; the other failure was the qualified JSON transport observation above.
  `target/smiles-path-layout-geometry-red.log` retains both actual failures.
- First GREEN: session25528 exit0,6passed,0failed/ignored;
  `target/smiles-path-layout-first-green.log`.
- Full scoped IO: session64664 exit0,209passed/0failed/0ignored (plus0doctests), including
  the private geometry validator and all old IO assertions; `target/smiles-path-layout-io-green.log`.
- Final expanded actual-byte/reference-scene acceptance: session44119 exit0,6passed,
  `target/smiles-path-layout-acceptance.log`; adds Letter/custom-origin and all four ink edges.
- All-target warnings-denied IO clippy: sessions95103 and98815; final log
  `target/smiles-path-layout-clippy-final.log`. Workspace fmt check follows in the same
  terminal98815 command. The first expected formatting failure and its owned-file-only
  rustfmt correction are retained; no unrelated source was formatted.

The ignored final handoff records final gate exits, exact source/log SHA256 manifests and
no live own jobs. Root independent peer/receiving/full/fuzz/hosted checks remain separate.
The new result has no source backing: future import/conversion must retain the original
SMILES owner rather than treating this drawing as interchange Save authority. Existing
model/StoredContent accounting already covers the returned ordinary model; no retained
field, hidden source attachment or history ownership is added here.

## Provenance

[Brecher2008/IUPAC, DOI10.1351/pac200880020277](https://doi.org/10.1351/pac200880020277),
actual [IUPAC/QMUL version](https://iupac.qmul.ac.uk/drawing/drawing.html): GR1.1 consistent
bond lengths/label clearance; GR3.2 horizontal chain orientation; GR4.1.1 relevant120degree
Single-bond angles; GR4.3 overlap avoidance. Raw HTML347564B SHA256
`32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`, read in Root's ignored
research. Numeric48pt/72pt/12atom/1024canvas choices and whole-chord/source-order selection
are original explicit policies. The normalization sources remain in verification-smiles-
normalize.md. No upstream implementation, paper figure, template, font or other asset was
transplanted. Existing bundled font and approved golden stay pinned and unmodified.

Final exact-source scoped gate: session53689 exits0,209passed/0failed/0ignored,
`target/smiles-path-layout-io-final.log`24360B/SHA256
`acdcc5cda97e6693a2d5eaf7a390a6cb817e45f7a6bd2030bc81f17da7e60f03`.
Final all-target clippy and workspace fmt-check session98815 both finish0; clippy log170B/
SHA256`d76a47630f8ffcbb15a6f3ba853194ed549d33db0956eabdac091d5a32326979`,
fmt log0B/SHA256`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
Final expanded six-test acceptance log844B/SHA256
`0acf87d21d0c0dfc542003186aefd8f77cd580982b39259dd78d2c62763b6764`.
All own Cargo handles are terminal. These results do not include Root receiving gates.

## Peer-review error-code correction

The independent source review (Root `target/smiles-path-layout-review.md`,11458B/SHA256
`33c88c8b593b2c34d7684e8c23d55c7dd39047338bd2cea1c7077f86da580e16`) found one public
error-contract mismatch: InvalidBondLength used `invalid_parameters` instead of the
project's existing `invalid_params`. A literal `invalid_params` assertion was added to
the existing zero/negative/NaN/infinity cases, preserving all old variant and geometry checks.
Actual RED93737 exits101 (left invalid_parameters/right invalid_params); one production
string correction gives focused GREEN65447 exit0. No geometry, model, raw XML, source IR,
consumer, dependency feature or position value was changed.

Final corrected scoped IO9613 exits0:209passed/0failed/0ignored. All-target warnings-denied
IO clippy and workspace fmt-check65658 finish0. Versioned ignored handoff-v2 retains the
original handoff and every original RED/log identity, plus this new RED/GREEN/final evidence.
Reverting only the code string recovers the exact original geometry-source SHA; removing
only the added literal assertion recovers the exact original test SHA. Root receiving and
publication remain separate, and no GUI/input/activation or next feature was attempted.
