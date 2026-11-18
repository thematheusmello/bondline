# Simple-ring SMILES drawing increment

2026-11-15, base492905a, after verified neutral acyclic conversion77d05a8. This continues
M1 toward general source-qualified drawing import; full M0–M5 remains the target. No
existing profile, unknown source data or negative chemistry check is replaced.

## Contract before implementation

Add `SmilesDocument::layout_neutral_simple_ring(length,origin,page,&Cancellation)`.
Require an actual connected graph whose sole cycle is a simple3..8-member ring with
Single cycle edges; attached trees retain the existing neutral fixed-H H/C/N/O/F/Cl/Br/I
normalization and Single/Double/Triple rules. Preserve physical H nodes, all local atom/
bond IDs/direction/counts, source/IR and exact supplied lengths. Check the64-atom ceiling
before normalization/search. `SimpleRingAtomLimit` reports `resource_limit`.

Use regular-ring geometry with preferred orientation, deterministic source-ID traversal/
phase ties and bounded attached-branch alternatives. Six-member polygons have a vertical
left edge; other sizes use a horizontal bottom edge as the initial original phase choice.
Exterior single substituents follow the larger-angle bisector, with the published nearby
30° snap. An ordinary equal geminal pair starts symmetrically at±30° as original Bondline
policy; the source requires60° separation, not that unique symmetry. Choices follow their
source context, including bulky roots and the
six-ring one-atom/larger-subtree distinction; do not substitute uniform thirds or refuse
all geminal input. Exact angle families and source identities are recorded in the companion
source review. These are2D depiction conventions, not molecular conformations or vendor defaults.

As in the existing acyclic profile, rigidly translate minimum atom-center x/y to origin,
without shrinking lengths or paper. Reuse actual-edge/vertex/intersection/stroke clearance,
full literal ink/paper limits and exact same-authored-H literal/resolved Scene certificate.
Do not pass cyclic adjacency into parent-only recursive tree helpers: use the ring boundary
and attached forest while preserving real neighbor context. No global label/renderer change
is authorized by a speculative collision concern; reproduce a defect independently first.

Preserve unassigned configuration. Terminal exocyclic C=O/C=C is supported; outside-ring
internal Double follows the existing qualified collinear unknown policy. Potential degree-
three internal Double, cycle Double/Triple,≥9-member and polycyclic graphs, components,
acyclic input, aromaticity, isotopes, charges and other unsupported metadata refuse as a
whole. They remain required later work, with no component/source loss or fake fallback.

## Shared conversion and native flow

Add the closed optional value `neutral_simple_ring_v1` to the existing conversion profile,
with native label **Neutral ring structures**. The default carbon path and named acyclic
policy retain output geometry/refusals, omission serialization and receipt identity. All
eight scalar parameters stay required; one30-command registry and the same create-only
whole-notation publisher/lease/worker serve UI, CLI, stdio MCP and authenticated loopback.
The new receipt captures its profile. Native radio choices replay at raw event boundaries.
No active-document Save/ACK, automatic Open or complete SMILES-source editing authority.

## Reproduced label prerequisites and original placement extension

The first real geometry implementation compiles, then passes1/8; strict scene equality
exposes literal degree-two CH2 bond occlusion. Independent fixed-hex25524 reproduces an
erased bond pixel and Atom hit where the analytic reference/same-count safe scene gives
black/Bond. Extending the existing authored-H attachment predicate from degree≥3 to≥2
passes the unchanged regression and88 renderer cases; unknown H still stays unknown.

The remaining bulky80° graph is accepted by the unchanged independent raw/chemistry/
geometry consumer but refuses rendering. A high-density10°/130°/250° star passes first
run: it does not establish a general rotated-star defect. The exact1000pt/1024px phase
instead fails all four incident-H-mask choices at em11.52, before global collisions.
Independent trace/SAT and density-matched runtime48721 identify that bounded search limit.

Add four compact candidates only AFTER the four original choices: above/below with H
shifted horizontally by−em/3 or+em/3. Keep the font/em, element anchor, authored H and
subscript rules, masks/padding, finite-width stroke intersections, global overlap/paper/
ink guards unchanged. This is original Bondline policy, not a reference default. A pinned
independent font/SAT proof qualifies the above-left mask with0.890626px minimum separation
from the three incident strokes; it does not by itself prove a full scene. The unchanged
density-matched test must establish full labels/bonds, source and CPU/GPU acceptance.
Existing successful four-choice results retain their exact first choice and pixels.

One existing schema assertion intentionally evolves from exactly2 to exactly3 allowed
names before product code. Its closure, required fields, every legacy serialization/output/
refusal and unknown-value check remain. This declared schema extension is not permission
to relax any old chemistry, timeout, fixture, oracle, source guard or golden.

## Test-first and acceptance

First independent cyclohexane declaration122bdee9 is3231bytes and precedes implementation.
Session14483 exits101 solely for missing API E0599, not a runtime geometry failure. Root
first840-byte JSON declarationf0369992 fails on the missing named profile; first511-byte
App declarationb3e17130 and expanded53156 fail on missing controls. Root expanded5677
has four runtime missing-profile failures. CLI/authenticated34574 and subsequent stdio/
authenticated52468 similarly fail before product code; reordering the new test makes
stdio reachable first while retaining its identical CLI checks.

Freeze independent expanded IO/oracle expectations before geometry edits. Literal graphs/H,
analytic polygon/attachment geometry, real ink, unspecified stereo, source retention and
bounded/cancel/error cases must pass unchanged or have a separately reviewed test setup
error. Root owns command/wire/App declarations and integration; independent workers own
layout/source research and tests in fresh isolated worktrees. Never restart old frozen work.

Native acceptance will use actual conversion selection/refusal/Unicode retry, retained
unsaved work, explicit guarded Open, external methyl C–C→C=C edit, Undo/Redo, separate Save
As/reopen and independent actual-file consumption (`CC1CCCCC1`→`C=C1CCCCC1`). This avoids
using a newly unsupported cycle-Double notation as the verifier for an unrelated ring edit.
Run full local gates and record exact source/log/binary/file pins. Actions stays disabled;
no push/publication/new golden approval is inferred. Larger rings, components, fused/spiro/
bridged graphs, stereo/chemistry and complete import/paste workflows remain required.
