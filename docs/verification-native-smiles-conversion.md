# Native SMILES conversion integration — 2026-11-15

Status: verified for the independent headless cases and bounded native Mac workflow below.
Full M0/M1 and M2–M5 remain incomplete.
Actions stays disabled. No push, publication, parser/layout/dependency/fixture/oracle/font/
approved-golden change. The [plan](research/smiles-native-conversion-plan.md) fixes scope;
this record owns actual integration, failures and receiving evidence.

## Delivered contract

File → Convert SMILES file… uses the existing typed `ui.file` command's new action and
the same `file.convert_smiles_to_cdxml` operation as CLI/MCP. The registry stays30 commands;
Drawing has18 controls and readonly SDF12. The form selects input and a new output path,
with explicit point geometry (original defaults48/72/72/A4). It supports one complete
ASCII notation with optional single final LF/CRLF,64KiB input maximum, for the existing
connected1–12 neutral nonisotopic carbon/Single path profile. Names, extra records,
aromaticity, stereo, general layout and inference are not silently discarded or inferred.

This original file utility is distinct from the reference's observed Copy As/Paste Special
SMILES roles, which remain missing. Source-backed SMILES Open/Save remains a different
operation. Conversion neither opens its output nor edits/saves the current document.
Existing destinations refuse; explicit ordinary File → Open protects unsaved work.

Specialized native preparation validates the exact current Accepted conversion intent
under the origin mutex and reserves the existing worker before consuming presentation.
Generic `prepare_ui_file` and `Prepared::bind_ui_request` still refuse conversion. No
native Save/close ACK, file_operation, saved baseline or history edit is fabricated. The
ordinary presentation request returns its issued `ui_file_operation` integer epoch;
that is an identifier, not publication authority. Actual conversion retains the existing
save lease and shared worker budget through completion, with a real handle through UI
retirement. Historical publication remains truthful; no automatic Open/New occurs.

## Independent and receiving evidence

- [Six core and two public worker cases](verification-native-smiles-conversion-commands.md)
  preserve complete dirty SDF/source/history, literal defaults and output consumers;
  guard exact accepted IDs, source/close/host ABA, budget/lease/drop/finish and no ACK.
  Minimal existing-action runtime RED26109 precedes separate missing-new-API compilers.
  Unchanged first real42292/82578 pass6+2; independent clippy/fmt pass.
- [Six actual-App cases](verification-native-smiles-conversion-ui.md) cover discovered
  menu/form, Unicode browsing, explicit geometry, real output, dirty state, ordinary
  Open/edit/history/Save/reopen, refusal/retry, modal input and stale unstarted cleanup.
  Missing-role runtime86758 and expanded baseline17502 precede product. The additional
  remote Cancel/old-input case passes first run3958; it is coverage, not a fabricated bug.
  Final independent29790 passes six, with UI all-target clippy62567 and formatting clean.
- [One queued-source and four real-worker gates](verification-native-smiles-conversion-workspace.md)
  plus the independent Drop case retain actual ownership through publication/retirement,
  cancellation, close and poisoned-engine drain. The four worker cases are additional
  postimplementation evidence, not invented test-first REDs.
- Root CLI3844 fails `invalid_params` versus valid headless `ui_unavailable` before the
  action exists. Final86682 passes unchanged. Two stdio/authenticated-loopback tests
  discover the unchanged eight-parameter converter and new native intent without claiming
  a dialog; final93279 passes. Presentation/cancel retains the complete777B SDF source.
  Root receiving24492 passes six core;10348 passes two public worker cases.

The frozen five App declaration was strengthened before product receipt with tool-state
and fresh Single/H3 assertions. An unused appended import was removed only for lint.
The first four input failures were a new snapshot-oracle error: parent AX values update
immediately, while the old chooser's painted InlineTextBox updates in the next requested
frame. Probe65111 verifies both, with no conversion before the failed readback. The approved
helper adds exact immediate-parent validation and a bounded painted-value wait before its
original final assertion. It changes no input, state/history/graph assertion or timeout.
All original declarations, exact inverses and logs remain separate.

Root's two new wire expectation corrections are explicit: readonly SDF has12 rather
than18 Drawing controls; `ui.file` returns its issued integer epoch. The conversion's
no-Save-ACK assertions remain intact. Initial71379/10457/79207 results are preserved;
these new-test corrections did not change producer code or old expectations.

## Reproduced product defects

1. Unstarted forms initially remained over a New/source/close transition. The independently
   declared fifth App case fails; cleanup now dismisses only obsolete unstarted forms and
   focus, preserving running/publication outcomes. The unchanged case passes38967.
2. [Independent review](research/native-smiles-conversion-review.md) reproduces actual
   publication after FileWorkflow destruction:19237 observes cancellationfalse and a real
   output. The narrow Drop cancellation call/helper makes the unchanged4251-byte regression
   pass92618 on the original source plus only that fix. No blocking Drop join is added.
3. Root51446 reproduces a successful retry still displaying its previous existing-file
   failure in the document inspector. Actual new output/consumer/source checks pass first;
   the stale-banner assertion fails. The unchanged1820-byte case passes after clearing the
   earlier message only on explicit valid queued submission, not blanket finish-time clear.
   Producer84596 passes that case and the original five; all13 new UI cases pass69907 and
   Root40672. Independent six-file prefix and Root's separate recovery bridge remain exact.

The producer's first full UI90164 has an old SMILES timeout (170pass/1fail/1ignored);
the exact unchanged case passes isolated0.14s and final default11963 passes223/0/1.
Cause remains unestablished. Root62187's first full aggregate stops at a new positive
browse/reopen timeout (172other library cases pass,1ignored); the unchanged isolated case
passes. Neither failure is hidden by later results or attributed without evidence.

## Shared picker investigation

An independent deterministic component test separately proves a real asynchronous chooser
click loss: stable-loading control succeeds, but accepting one directory entry between
last-painted bounds and press/release changes the enabled Open button's reachable identity
and emits no intent. The old point remains inside both rectangles. This is not proof of
the cause of62187. Initial locator setup correction (Focus/Click verb versus unadvertised
Default action) is separate from actual24236/40618/18503 REDs.

The shared correction keeps asynchronous browser content in one stable bounded region.
Review catches a candidate regression: moving errors into a scrolled list hides an invalid-
filename error. Actual98973 fails painted visibility after real scrolling/typing/Open;
original58678 control41160 passes. A separate always-allocated two-row error region keeps
full text visible/scrollable and resets only for new messages/navigation using one fixed
temporary key. It preserves footer identity/geometry and all old handler/confirmation
semantics. Both frozen component cases pass34280; full final producer UI51281 passes225/0/1.
Exact Root receiving, final aggregate and native scope follow below.

## Final receiving gate

Root57791 receives both frozen shared-picker cases unchanged:2pass. Root84363 runs the
final `cargo xtask ci` with Rust1.94.1, `CARGO_INCREMENTAL=0` and the literal pinned
`BONDLINE_PYTHON=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline/.cache/oracles/format-python/bin/python`:
**1116 passed,0 failed,1 ignored**,158 result lines, exit0. Rustfmt, all-target
warnings-denied clippy, workspace/vendor tests, layers, licenses and corpus pass. The
134205-byte log at `target/native-smiles-conversion/root-final-ci-2.log` has SHA256
`9ec6b893885d7f9812d052a80699d3febe015571f66c14698afa33638d949a04`.
Its exact receipt and earlier failure logs remain ignored build evidence.

Runtime closures remain154/165/220 with two assets; corpus remains8files/5bundles. Foreign/
general CDX and chemistry comparisons remain missing. No parser/layout, dependency/lock,
SDK, fixture, external oracle, font or approved-golden change. No new fuzz, hosted or
interactive Windows/Linux execution. The previous timeout failures above remain recorded;
a passing final gate is not proof of their original cause.

## Actual native Mac acceptance

Root97224 builds the final binary successfully. CUA selects the actual original bundle
`target/native-smiles-conversion-native/Bondline.app`, identifier
`org.bondline.native-smiles-conversion-review`; observed PID**52075**, no control channel.
The executable is22247320bytes, SHA256
`823a0aa7eb52d74d49e22cbe84d0e75e37838e028b9b0a6f5628b57f787809b1`.
Final production sources and executable still match the pinned build receipt after the
workflow. The selected1160×832 native window shows100% on unchanged A4.

1. Draw an independent unsaved C–C at different coordinates. File → Convert SMILES file…
   exposes the original profile explanation and explicit48/72/72/A4 defaults.
2. Browse input to the original3-byte `CC\n` file and browse output to the predeclared
   40-byte destination sentinel. Browse itself does not replace or discard the drawing.
   Convert visibly reports an existing-destination failure, with no overwrite action.
3. Set a new Unicode output path and explicitly retry. The form reports Conversion
   complete/2atoms/1bond and the published path. The prior global error clears; the
   independent drawing remains Untitled*, unsaved, at its original position. No auto Open.
4. Close the form; explicitly Open the actual output. Cancel the dirty decision and
   observe the retained original. Repeat and choose Discard and Open for this disposable
   agent-authored drawing. The converted Single/H3–H3 appears at(72,72)→(120,72), Saved.
5. Select the Double tool and click the existing edge; native pixels show Double/H2–H2
   and Unsaved changes. Edit → Undo restores Single/H3–H3 and Saved. Save As to a separate
   Unicode path, then explicitly reopen that actual file; Single/Saved/counts persist.

The actual conversion `retry-λ.cdxml` and subsequent native Save As `chain-λ.cdxml` are
both**433bytes**, SHA256`8303fa5371ba8ae7644bfcbf763e5e553e6c3205a90668a1b2b4100c76b1865b`.
Before Open/editing, the unchanged external RDKit oracle acceptsCC/C2H6/two neutral C/
H3–H3/one Single. Independent Expat/Decimal also checks exact A4 dimensions, literal72/
120 coordinates and node references. After Save As/reopen both outputs remain exactly
equal to the archived pre-edit conversion and pass that same unchanged consumer. The
3-byte input,5-byte unsupported input and40-byte sentinel remain exact.

`unsupported.smi` was selected into the successful form after completion but never
submitted; its absent output is **not** native refusal evidence. Native observations use
pointer/AX controls, including path SetValue; no geometry-number typing, hotkey, ordinary
OS clipboard, race/cancel/close timing, physical-DPI or other-platform claim follows.
Those have only their stated headless evidence or remain missing. Inline native images
are inspection evidence, not new human-approved goldens. Receipts and disposable app/input/
output artifacts remain ignored under `target/native-smiles-conversion-native/`.
