# Neutral acyclic SMILES drawing increment

2026-11-15, base97fd2ac. This is the next executable M1 increment toward general SMILES
drawing import. It does not complete M1 or replace the complete M0–M5 target. The previous
turn delivered the native restricted converter; this turn starts from that committed work.

The [primary-source review](smiles-general-layout-sources.md) separates notation, fixed-H
normalization, depiction and source authority. Existing parsing already retains a broader
graph. Existing normalization supports neutral H/C/N/O/F/Cl/Br/I with a closed fixed-H
policy. Existing layout and its default conversion profile deliberately accept only
1–12 nonisotopic carbon/Single paths. Keep their geometry, refusals and output exact.

## Agreed addition

- Add `SmilesDocument::layout_neutral_acyclic(length, origin, page, &Cancellation)` in IO.
  It returns a fresh positioned Document, leaving original source/IR/IDs/metadata intact.
  Use the existing fixed-H normalization; do not introduce valence/aromaticity perception.
- Accept a single connected tree of at most64 atoms, including physical-H nodes, branches
  and Single/Double/Triple edges. Test connectivity from actual edges, not dot/ring tokens.
  Isotopes, charge, aromatic markers and unsupported normalization still refuse, without
  discarding a component.64 is a resource ceiling, not a promise every graph fits paper.
- Use IUPAC's degree/context recommendations, deterministic source-ID tie breaks and
  bounded branch alternatives. Ordinary two/three-neighbor centers use120°; triple/Single
  and two-Double carbon centers use180°. Degree-four templates depend on terminal versus
  extended neighbors. Keep literal H0 central-carbon labels visible in actual rendering.
- Preserve unspecified stereo. For the specific disubstituted internal-Double case with
  two neighbors at each end, use the published collinear substituent treatment and require
  actual CDXML consumer StereoNone. Potential degree-three internal-Double cases refuse
  until explicit unknown-configuration depiction exists. Do not choose an apparent E/Z
  configuration or introduce wedges/Fischer implications from SMILES traversal order.
- Translate the completed atom-center bounds so their minimum x/y equals the supplied
  origin. This new profile's origin is a center-bounds anchor; the old profile's endpoint
  anchor stays unchanged. Preserve exact bond length; no automatic scaling/page changes.
- Validate actual vertices/edges, nonincident overlaps and the unchanged literal rendered
  glyphs/masks/multiple strokes/paper interior at1024². No clipped or point-only success.
  Honor cancellation and finite search budgets with typed errors, leaving source exact.

## Shared conversion and native presentation

Extend the existing `file.convert_smiles_to_cdxml` parameter object with an optional closed
`profile`: `neutral_carbon_path_v1` remains the default; `neutral_acyclic_v1` selects the
addition. All eight existing scalar fields remain required and closed. Registry30, old
omitted-profile serialization and old receipt profiles remain unchanged. A few direct Rust
parameter literals need an explicit default field; no old behavioral assertion is relaxed.

Preserve the original public IO converter. An additive acyclic converter reuses its bounded
whole-notation read and create-only publisher. The receipt reports the selected profile.
No source-file record is silently selected, overwritten or replaced. Complete SmilesFile
sessions, active source/history/baselines, save lease and worker/retirement/close guards stay
authoritative. This derives a new file, not editing authority over an imported source file.

The original native form will offer both typed choices before Convert, with truthful limits
and origin meaning. Choice changes participate in existing ordered form actions. No second
registry, pool, chooser, automatic Open or active-document Save/close acknowledgment.
CLI, stdio MCP and authenticated loopback invoke the same command; native execution is
separate evidence from command/frame tests.

## TDD and continuation gates

Independent first declarations precede product edits: branch layout test76197 fails only
for the missing API; Root73477's existing JSON command fails on unknown `profile`. Their
exact first files/logs are retained under ignored target receipts. Expanded tests must use
literal graphs/H, analytic angles/lengths, actual ink and a new independent bounded
Expat/Decimal/RDKit consumer. Keep every existing oracle/fixture/golden and path assertion.
Separate new-test setup errors from intended product REDs.

Then implement, receive unchanged tests, verify actual output and default compatibility,
exercise native profile selection/derived-file Open/edit/history/Save/reopen, run the full
local gate and commit the feature with sources/evidence. Actions remains disabled.
Simple rings with substituents, component packing, general fused/spiro/bridged graphs,
aromaticity/stereo/charges and complete source-qualified import remain explicit next work.
