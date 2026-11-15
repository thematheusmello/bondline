# Truthful Fit zoom readout — independent App checks

2026-11-15. Test-only ownership in `bondline-fit-readout-tests`, declared base `976ef42`.
Root owns production, Git, integration and native execution. Actions stays disabled.
The unchanged original740-byte SDF is
`crates/io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf`, SHA-256
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.
No parser, format, engine, adapter, image, dependency, font or golden is changed here.

## Independent contract and cases

A temporary numeric UI seed is not a measured fitted percentage. SDF Fit must visibly
and accessibly say `Fit`, with no numeric claim, regardless of prepared bounds or whether
the input is enabled. The actual existing Zoom control is used, with its current reachable
SpinButton/TextInput role and text bounds. Idle text is read from painted galleys and the
actual accessibility subtree; no fabricated control/result replaces the app.

Five small cases exercise `BondlineApp::update_workspace` through the existing Fixture:

1. Original SDF at1160×800,1x: unknown Fit reports Fit; idle/focus/Escape never commits a
   camera or authored change. Typing100 is still pending until Enter. Explicit100 must
   become actual100%/epoch1 despite the old fallback seed already being100. Complete
   source/state/history, dirty/file metadata, CaptureStamp and disk bytes stay exact.
2. Explicit100 also commits on actual field blur through the existing View menu. Enter
   is not the only supported commit route; pending typing does not mutate the camera.
3. Real numeric accessibility SetValue100 works at2x and produces successful registered
   SDF extent preparation. Returning to Fit keeps that prepared extent but still reports
   textual/non-numeric Fit. Cache presence and physical scale do not define this label.
4. A Drawing's known fitted percentage and explicit150 remain truthful numbers: actual
   AX numeric value matches Engine's already-computed percentage, and painted percentage
   matches its two-decimal presentation. This is a UI readout check; independent geometry
   of that fit remains the existing renderer/view tests' separate responsibility.
5. A real accepted Open picker disables Zoom. A separate reachable-tree helper observes
   the disabled node without weakening the old Focus-required locator. It must still
   paint/report Fit and no numeric value. Actual file.cancel consumes the matching picker;
   only that explicit transient intent/epoch may differ, never source/history/camera.

All SDF Fit is intentionally textual because legacy whole-page physical-margin rendering
and logical fit math can differ at high DPI. This changes no image, fit math, relative-step
policy or source. Complete relative-from-Fit/native presentation remains separate work.

## Actual RED/GREEN and retained setup corrections

The first4,207-byte case, SHA-256
`8909f2f7df22d3201ad68fa8e63e9820f4ff7a8b827025f44f8b73d0465a90b0`,
compiled and ran in worker57640. It exited101 precisely at the first painted Fit assertion:
the baseline showed the100% fallback. This is an intended runtime RED, not a missing type
or zero-match selector. That source remains the exact prefix of the final module.

Root supplied an authorized exact one-file `view_controls.rs` port, initial SHA-256
`03d5969dd4c6f72c894f44f2c18c2206653cf8b71e049e7d6474980b8416a302`.
It preserves DragValue and tracks explicit setter calls so100 can commit even when equal
to the numeric seed, formats Fit and clears its numeric claim. Worker49367 initially
passes3 cases; the new prepared2x case used the wrong existing menu label `Fit`. Source
`commands/controls.rs` identifies the real label `Fit to window`. Root approved changing
only that new locator. The9,371-byte declaration and failure remain retained. No expected
Fit text, numerical value, input or old assertion was relaxed. The corrected four-case
source is9,381 bytes; its first4,207 bytes remain exact.

Worker8622 then passes all4 cases and warnings-denied UI clippy. Workspace formatting
stops only on Root's unformatted multiline matches! expression. The approved formatter-
only recopy is SHA-256 `337ed87fb89290e8143bd8111b2d98dc480a835b431f1b94da39defa1b4ac8af`;
non-whitespace content is identical. Fmt-only worker29647 passes. These are a locator
setup correction and production formatting issue, not additional behavior REDs.

Source review then finds that `fit = enabled && ...` revives100 while an SDF picker
is accepted. One fifth case is appended after the exact9,381-byte prefix. Worker66097
exits101 precisely at the disabled field's visible Fit assertion after proving that the
actual picker is accepted and the current Zoom node is disabled. This is a second genuine
runtime RED. Root's minimal predicate now derives Fit independently of input enablement.
Its final one-file source is11,494 bytes, SHA-256
`4b9342aff7695cea1d61eed43fec3eddc335690f28b2a39a04fba133a2d7dc72`.

Worker12179 passes5 cases,0 failures,0 ignored against that exact final source, followed
by all-target UI clippy with warnings denied and workspace rustfmt check. Final logs:
`target/fit-readout-final-five-green.log`, `target/fit-readout-five-ui-clippy.log`,
`target/fit-readout-five-fmt-check.log`. Earlier runtime/setup/format logs and source
snapshots remain distinct; Root's receiving aggregate is separate evidence.

Final test module:13,394 bytes, SHA-256
`6f646c111e9454294e4436afc837b2c1dc1083dca9648334a74c24b2f9b9ac87`.
The only parent edit appends45 bytes after the unchanged48,182-byte `view_tests.rs` prefix.
The final ignored handoff verifies287 protected inputs with only the explicitly received
Root source replacement, every source prefix/correction and actual log identity.
No worker production authoring or changes to existing tests/fixtures/oracles occurred.

This is synthetic actual-App/accessibility input, not native OS keyboard/AX, tooltip
hover or physical-monitor evidence. Other content kinds and relative zoom semantics are
outside these cases. Root separately qualifies its native builds and controller-driven
observations. No full-workspace/Windows/Linux execution or M0–M5 completion is claimed
here. All worker processes are terminal.
