# Native benzene and cyclopentadiene: locked integration

2026-11-16. Root locks this integration after command89aa03d and contract105b220, before new UI/API tests or product changes. No named native implementation/execution follows from this document. GUIDE25 supports named tools and Shift Single-first, not the press/release timing below or verified26.1 defaults. All defaults/ties/timing below are original Bondline choices. The user's open bundle/draft stays untouched.

## Exact API and compatibility

- Keep public SelectRingTool{size:u8} and EffectiveTool::Ring{size} unchanged. Add SelectRingToolRequest::Sized(old)/Named(SelectNamedRingTool{kind:RingKind}), From conversions and the two closed JSON alternatives: {size:3..8} OR {kind:"benzene"|"cyclopentadiene"}. Root type=object; each branch additionalProperties:false. Reject both/neither selectors, null/unit-wrapper kind, unknown keys and any phase. Registry remains38; only new named fields use strict string decoding.
- Add EffectiveTool::NamedRing{kind} to the ONE existing mode authority. No extra active flag or selected-kind cache. Remembered bond order is untouched. Same named tool is a fully validated no-op; Single-six↔Benzene is a real change using the existing checked tool/selection epochs. New enum variants require exhaustive-match migration even though old constructors remain valid.
- The receipt remains EXACTLY the existing RingToolOperation{size,epoch,changed} for both paths: derived size6/5 for Named, no new kind/phase field. Outer changed:false and normal omitted receipt stay exact. Its size is not a mode proof: ui.controls exposes selected named binding, and RingContext must capture the exact EffectiveTool rather than size. Existing weak source/page/tool/view/close/file/opaque association checks and changed-only notification remain.
- Add Engine::preview_ring_request(&RingCreateRequest)->Result<StoredDocument,Error> over the real create_request builder. Preserve old preview_ring_create(&RingCreate). UI Candidate/Gesture stores RingCreateRequest; local union matches expose placement/page without a second parameter authority. No worker, file intent, converter, temporary source or new dependency.

## Shift timing and shared candidate

For Named only, latch phase from the ordered primary PRESS: None modifiers→phase None/contextual; Shift alone→Some(SingleFirst). Store it in NamedRingCreate, not global tool state. PointerMoved changes geometry only. Release with None or Shift is allowed and NEVER changes phase: Shift press→None release stays Single-first; None press→Shift release stays contextual, also in one packet. Ctrl/Alt/Command or combinations at either primary boundary refuse/cancel without Bond/atom-form fallback. Do not infer a phase from full-frame final modifiers. Every modified legacy sized-ring primary gesture, including Shift, remains unsupported.

Reuse48pt click poses, derived6/5 geometry, old atom bisectors/ties and preferred/opposite bond-side logic. Keep the latched phase on both side trials; do not retry contextual phase after Shift failure or render all rotations. Native remains at most two complete side candidates; backend phase/H checks remain cheap2/5. Preserve reused edge order/direction/style; native may truthfully refuse its actual ink without restyling or detaching.

Preview and release use the same whole candidate, physical-DPI/fractional-origin rendering, logical label readability and stroke/gap/hit/paper guards. A glyphless scene has no nominal font, not weaker stroke validation. Recheck RingContext before one ring.create dispatch. Preserve threshold latch, off-canvas/PointerGone/Escape cancellation, namespace/source/view/tool/file/close/DPI/layer proofs and frame-start text/modal fences. Release→tool/Undo/Save acts first; reversed order cancels the old gesture. Multiple completed gestures use current revision at each raw boundary.

## Implementation seams

| Actual file/caller | Minimal change |
|---|---|
| commands/ring_tool.rs:12/34/67/109/139 | Request union, common selection transition, exact captured mode and additive request preview. There is no ui_ring.rs. |
| commands/bond_tool.rs:23; selection.rs:200/316 | NamedRing enum case; same epoch/deactivation authority. |
| commands/controls.rs:444; lib.rs:190/716 | Append tool.ring.benzene/Benzene and tool.ring.cyclopentadiene/Cyclopentadiene, kind-only bindings and exact selected mode. Native29→31; retained20/SDF14/raw10 unchanged. Registry/export/typed argument .into() only. |
| ui/lib.rs:226/266/418/431/1583 | Derive procedural6/5 polygon plus3/2 original inward icon strokes; keep house28pt AX controls. Add named guidance/event routing. No proprietary art or circle substitute. |
| ui/ring.rs:12/18/33/62/116/264/307 | Union parameters, press phase, common geometry updates and same preview/release. |
| ui/lib.rs:1032/1392/2030–2078 | Preserve ordered Control(id), all early-refusal cancellation and actual preview paint. No App field. |

Schema upgrades must preserve old size assertions on Sized: commands/tests/ring_tool.rs:340–358 and cli/tests/support/ring_tool_wire.rs:141–152. Control sets/counts gain exactly two names. Typed SelectRingTool callers may need .into(); their struct fields, old semantic assertions, private overflow/no-op proofs and17 existing App cases stay protected.

## Independent evidence before product

1. Freeze a tiny existing-harness actionable Benzene App case and observe missing-control runtimeRED. Accept the house Button/Checkbox role and actual procedural glyph. Separate JSON named-tool/schema RED from new-type compile qualification. Verify exclusive mode selection/no-op/epoch and unchanged source/history.
2. Free click/drag for both names: literal orders/H/false flags/inward sides, actual complete preview with no commit, real physical components/gaps/picks, one release/edit, Undo/Redo and Unicode Save/reopen through unchanged Expat/Decimal+RDKit. Include both press/release Shift permutations and same-packet future-Shift leakage. No new golden.
3. Real terminal benzene/toluene, shared-Single fusion and Cp spiro, preserving all old data. Include one render-qualified shared Double without restyling and a Shift/Double conflict with no contextual fallback. Use existing headless phase/asymmetric tables instead of duplicating a large App matrix.
4. Named/sized and kind ABA, same named no-op while armed, ordered tool/release/Undo/Save and two completed gestures. Add a bounded named modifier/modal+Cancel-frame and invalid-view→restored-view cancellation check with fresh recovery. Retain old sized Shift negatives and existing source/file/DPI guards.

Root adopts all preceding API, receipt, timing, candidate and acceptance policies before declarations/producer code. Existing legacy APIs/negative expectations remain exact; only explicit new schema branches and two native control/count additions may positively extend inventory assertions. Native OS/platform workflows remain unverified and separate from App/Metal results. Ctrl delocalized circle, Alt sprout, arbitrary resonance rotation, chair/style/fused-face solving and full M0–M5 remain required; no public-source default beyond the cited version is inferred.


The reviewed source proposal is6908B/SHA2562d5f428404abd4ab5ac79654615a472db52910a645e81f3dd0132b7204f06614. Read the existing [Single-ring native contract](ring-native-plan.md) and [named command lock](unsaturated-ring-authoring.md) for public-source pins and retained safeguards. No reference artwork or code is adopted.
